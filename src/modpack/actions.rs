use std::io::Error as IOErr;
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use const_format::concatcp;
use regex::Regex;

use crate::read_to_string_buf;
use crate::{ini, nmf};


pub enum Error {
    FileRead(IOErr),
    FileParse(String),
    Validation(Vec<String>),
}


#[derive(Debug)]
pub struct ModActions {
    pub scale: Option<f64>,
    pub offset: Option<(f32, f32, f32)>,
    pub mirror: bool,
    pub objects: Option<(ObjectVerb, Vec<String>)>,
    pub rename_sm: Vec<(String, String)>,
}


#[derive(Debug)]
pub enum ObjectVerb {
    Remove,
    Keep,
}

impl ObjectVerb {
    const VERB_KEEP:   &'static str = "KEEP";
    const VERB_REMOVE: &'static str = "REMOVE";
}


pub fn read_actions(actions_path: &Path, buf: &mut String) -> Result<ModActions, Error> {
    const RX_FLOAT: &str = r"(-?\d+(?:\.\d+)?)";

    lazy_static! {
        static ref RX_TOKENS:  Regex = Regex::new(r"(?s)(^|(\s*\r?\n)+)\$").unwrap();

        static ref RX_SCALE:   Regex = Regex::new(r"(?s)^SCALE\s+(\d+(?:\.\d+)?)\s*$").unwrap();
        static ref RX_OFFSET:  Regex = Regex::new(concatcp!(r"(?s)^OFFSET\s+", RX_FLOAT, r"\s+", RX_FLOAT, r"\s+", RX_FLOAT, r"\s*$")).unwrap();
        static ref RX_MIRROR:  Regex = Regex::new(r"(?s)^MIRROR\s*$").unwrap();
        static ref RX_OBJECTS: Regex = Regex::new(r"(?s)^OBJECTS\s+([A-Z]+)(.+)").unwrap();
        static ref RX_NAMES:   Regex = Regex::new(r"(?s)\s+([^\s]+)").unwrap();

        static ref RX_SUBMAT:  Regex = Regex::new(r"(?s)^SUBMATERIAL_RENAME\s+([^\s]+)\s+([^\s]+)").unwrap();
    }

    buf.clear();
    read_to_string_buf(actions_path, buf).map_err(Error::FileRead)?;

    let mut scale = None;
    let mut offset = None;
    let mut mirror = false;
    let mut objects = None;
    let mut rename_sm = Vec::with_capacity(0);

    for token in RX_TOKENS.split(&buf) {
        if token.is_empty() {
            continue;
        }

        if let Some(cap) = RX_SCALE.captures(token) {
            let factor = f64::from_str(cap.get(1).unwrap().as_str())
                .map_err(|e| Error::FileParse(format!("Could not parse SCALE as float: {:?}", e)))?;
            scale = Some(factor);
        } else if let Some(cap) = RX_OFFSET.captures(token) {
            let x = f32::from_str(&cap[1]).map_err(|e| Error::FileParse(format!("Could not parse OFFSET x as float: {:?}", e)))?;
            let y = f32::from_str(&cap[2]).map_err(|e| Error::FileParse(format!("Could not parse OFFSET y as float: {:?}", e)))?;
            let z = f32::from_str(&cap[3]).map_err(|e| Error::FileParse(format!("Could not parse OFFSET z as float: {:?}", e)))?;
            offset = Some((x, y, z));
        } else if RX_MIRROR.is_match(token) {
            mirror = true;
        } else if let Some(cap) = RX_OBJECTS.captures(token) {
            let verb = cap.get(1).unwrap().as_str();
            let rest = cap.get(2).unwrap().as_str();

            let names = { 
                let mut res = Vec::with_capacity(64);
                for cap in RX_NAMES.captures_iter(rest) {
                    let cap = &cap[1];
                    if res.iter().any(|r| r == cap) {
                        return Err(Error::FileParse(format!("Object {} action uses duplicate object name '{}'", verb, cap)));
                    }

                    res.push(cap.to_string());
                }

                res
            };
                
            if names.len() == 0 {
                return Err(Error::FileParse("Could not parse object action: no object names were specified".to_string()));
            }

            let verb = match verb {
                ObjectVerb::VERB_KEEP   => ObjectVerb::Keep,
                ObjectVerb::VERB_REMOVE => ObjectVerb::Remove,
                _ => { return Err(Error::FileParse(format!("Could not parse objects action verb: [{}]", verb))) }
            };

            objects = Some((verb, names));
        } else if let Some(cap) = RX_SUBMAT.captures(token) {
            let from_name = cap[1].to_string();
            let to_name = cap[2].to_string();

            rename_sm.push((from_name, to_name));
        } else {
            return Err(Error::FileParse(format!("Unknown token: [{}]", token)))
        }

    }

    Ok(ModActions { scale, offset, mirror, objects, rename_sm })
}


impl ModActions {
    pub fn apply_to(&self, nmf: &mut nmf::NmfInfo) {
        if let Some((verb, names)) = &self.objects {
            let mut new_objs = Vec::with_capacity(nmf.objects.len());

            for o in nmf.objects.drain(..) {
                let keep = match verb {
                    ObjectVerb::Keep   => names.iter().any(|n| n == o.name.as_str()),
                    ObjectVerb::Remove => names.iter().all(|n| n != o.name.as_str())
                };

                if keep { 
                    new_objs.push(o);
                }
            }

            nmf.objects = new_objs;
        }

        for (old_name, new_name) in self.rename_sm.iter() {
            for sm in nmf.submaterials.iter_mut() {
                if sm.as_str() == old_name {
                    sm.push_str(&new_name);
                }
            }
        }
    }

    pub fn validate<'a>(&self, bld_ini: &Path, nmf_info: &nmf::NmfInfo, str_buf: &mut String) -> Result<(), Error> {
        if self.scale.is_none() && !self.mirror && self.objects.is_none() && self.rename_sm.is_empty() {
            return Err(Error::Validation(vec!["Empty ModActions".to_string()]));
        }

        let mut errors = Vec::with_capacity(0);

        if let Some((verb, names)) = &self.objects {
            use ini::BuildingNodeRef as REF;

            // TODO: This mess with building.ini cheks is temporary here (I hope). 
            //       Ideally this should be removed  when the ini can cleansed
            //       from removed nodes automatically. 

            read_to_string_buf(&bld_ini, str_buf).map_err(Error::FileRead)?;
            let bld_ini = ini::parse_building_ini(str_buf).unwrap();
            let model_refs = bld_ini.get_model_refs();

            match verb {
                ObjectVerb::Keep => { 
                    for mref in model_refs {
                        match mref {
                            REF::Exact(node)  => if names.iter().all(|kept| kept != node) {
                                errors.push(format!("building.ini refers to model node '{}', but this node is not present in the actions' KEEP list", node));
                            },
                            REF::Keyword(key) => if names.iter().all(|kept| !(kept.starts_with(key))) {
                                errors.push(format!("Node-referring keyword '${}' is used in the building.ini, but is not present in the actions' KEEP list", key));
                            }
                        }
                    }
                },
                ObjectVerb::Remove => {
                    for mref in model_refs {
                        match mref {
                            REF::Exact(node)  => if names.iter().any(|remd| remd == node) {
                                errors.push(format!("building.ini refers to model node '{}', but this node is present in actions' REMOVE list", node));
                            },
                            REF::Keyword(key) => {
                                errors.push(format!("Node-referring keyword '{}' is used in the building.ini. OBJECTS REMOVE action is not supported in this case.", key));
                            }
                        }
                    }
                }
            }

            for name in names.iter() {
                if nmf_info.object_names().all(|o| o != name) {
                    errors.push(format!("Cannot {} object '{}' in the NMF, because such object does not exist", verb, name));
                }
            }

            if names.len() == nmf_info.objects.len() {
                match verb {
                    ObjectVerb::Remove => errors.push(format!("Possible attempt to remove all objects. Entries count equals nmf objects count.")),
                    ObjectVerb::Keep   => errors.push(format!("Possible attempt to keep all objects. Entries count equals nmf objects count."))
                }
            }

        } //------------- objects end

        for (r, _) in self.rename_sm.iter() {
            if nmf_info.submaterials.iter().all(|sm| sm.as_str() != r) {
                errors.push(format!("Cannot rename submaterial '{}' in the NMF, because such submaterial does not exist", r));
            }
        }


        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(errors))
        }
    }
}

use std::fmt;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::FileRead(e)   => write!(f, "Could not read file: {}", e),
            Error::FileParse(e)  => write!(f, "Could not parse file: {}", e),
            Error::Validation(e) => {
                writeln!(f, "Validation failed: ")?;
                for i in e.iter() {
                    writeln!(f, "    {}", i)?;
                }
                Ok(())
            }
        }
    }
}


impl fmt::Display for ObjectVerb {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ObjectVerb::Keep   => write!(f, "{}", ObjectVerb::VERB_KEEP),
            ObjectVerb::Remove => write!(f, "{}", ObjectVerb::VERB_REMOVE)
        }
    }
}

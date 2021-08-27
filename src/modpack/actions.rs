use std::io::Error as IOErr;
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{read_to_string_buf};


#[derive(Debug)]
pub enum Error {
    FileRead(IOErr),
    FileParse(String),
    Validation(Vec<String>),
}


#[derive(Debug)]
pub struct ModActions {
    pub scale: Option<f64>,
    pub mirror: bool,
    pub objects: Option<ObjectActions>
}


#[derive(Debug)]
pub enum ObjectActions {
    Remove(Vec<String>),
    Keep(Vec<String>),
}

impl ObjectActions {
    const VERB_KEEP:   &'static str = "KEEP";
    const VERB_REMOVE: &'static str = "REMOVE";
}


pub fn read_actions(actions_path: &Path, buf: &mut String) -> Result<ModActions, Error> {
    lazy_static! {
        static ref RX_TOKENS:  Regex = Regex::new(r"(?s)(^|(\s*\r?\n)+)\$").unwrap();

        static ref RX_SCALE:   Regex = Regex::new(r"(?s)^SCALE\s+(\d+(\.\d+))\s*$").unwrap();
        static ref RX_MIRROR:  Regex = Regex::new(r"(?s)^MIRROR\s*$").unwrap();
        static ref RX_OBJECTS: Regex = Regex::new(r"(?s)^OBJECTS\s+([A-Z]+)(.+)").unwrap();
        static ref RX_NAMES:   Regex = Regex::new(r"(?s)\s+([^\s]+)").unwrap();
    }

    buf.clear();
    read_to_string_buf(actions_path, buf).map_err(Error::FileRead)?;

    let mut scale = None;
    let mut mirror = false;
    let mut objects = None;

    for token in RX_TOKENS.split(&buf) {
        if token.is_empty() {
            continue;
        }

        if let Some(cap) = RX_SCALE.captures(token) {
            let factor = f64::from_str(cap.get(1).unwrap().as_str())
                .map_err(|e| Error::FileParse(format!("Could not parse SCALE as float: {:?}", e)))?;
            scale = Some(factor);
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

            match verb {
                ObjectActions::VERB_KEEP   => { objects = Some(ObjectActions::Keep(names)) },
                ObjectActions::VERB_REMOVE => { objects = Some(ObjectActions::Remove(names)) },
                _ => { return Err(Error::FileParse(format!("Could not parse objects action verb: [{}]", verb))) }
            }
        } else {
            return Err(Error::FileParse(format!("Unknown token: [{}]", token)))
        }

    }

    Ok(ModActions { scale, mirror, objects })
}


impl ModActions {
    pub fn validate<'a>(&self, nmf_objects: impl Iterator<Item = &'a str> + Clone) -> Result<(), Error> {
        match &self.objects {
            None => {
                if self.scale.is_some() || self.mirror {
                    Ok(())
                } else {
                    Err(Error::Validation(vec!["Empty ModActions".to_string()]))
                }
            },
            Some(act) => {
                use ObjectActions as OA;
                let (verb, names) = match act {
                    OA::Keep(objs)   => (OA::VERB_KEEP, objs),
                    OA::Remove(objs) => (OA::VERB_REMOVE, objs),
                };

                let mut errors = Vec::with_capacity(0);

                for name in names.iter() {
                    let mut obj_iter = nmf_objects.clone();
                    if !obj_iter.any(|o| o == name) {
                        errors.push(format!("Cannot {} object '{}' in the NMF, because it was not found", verb, name));
                    }
                }

                if errors.is_empty() {
                    if names.len() == nmf_objects.count() {
                        match act {
                            OA::Remove(_) => errors.push(format!("Possible attempt to remove all objects. Review the actions file.")),
                            OA::Keep(_)   => errors.push(format!("Possible attempt to keep all objects. Review the actions file."))
                        }

                        Err(Error::Validation(errors))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(Error::Validation(errors))
                }
            }
        }
    }
}

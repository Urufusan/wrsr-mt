//use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ops::Range;
use std::convert::{TryFrom, TryInto};

use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

use crate::cfg::{AppSettings, APP_SETTINGS};

//--------------------------------------------
//                SOURCES

#[derive(Debug, Clone)]
pub struct BuildingDef<'stock> {
    pub render_config: RenderConfig<'stock>,

    pub building_ini: PathBuf,
    pub imagegui: Option<PathBuf>,

    pub model: ModelDef,
    pub model_lod1: Option<ModelDef>,
    pub model_lod2: Option<ModelDef>,
    pub model_emissive: Option<ModelDef>,

    pub material: MaterialDef,
    pub material_emissive: Option<MaterialDef>,

    pub skins: Vec<Skin>
}

#[derive(Debug, Clone)]
pub struct IniToken<T> {
    pub range: Range<usize>, 
    pub value: T
}

pub type IniTokenPath = IniToken<PathBuf>;
pub type IniTokenTexture = IniToken<Texture>;

#[derive(Debug, Clone)]
pub enum RenderConfig<'stock> {
    Stock { key: &'stock str, data: &'stock str },
    Mod(PathBuf)
}

#[derive(Debug, Clone)]
pub struct ModelDef {
    pub ini_token: IniTokenPath,
    pub patch: Option<ModelPatch>,
}

#[derive(Debug, Clone)]
pub enum ModelPatch {
    Keep(Vec<String>),
    Remove(Vec<String>)
}

#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub render_token: IniTokenPath,
    pub textures: Vec<IniTokenTexture>
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub num: char,
    pub path: PathBuf
}

#[derive(Debug, Clone)]
pub struct Skin {
   pub material: SkinMaterial,
   pub material_emissive: Option<SkinMaterial>
}

#[derive(Debug, Clone)]
pub struct SkinMaterial {
    pub path: PathBuf,
    pub textures: Vec<IniTokenTexture>
}

#[derive(Debug, Clone)]
pub enum PathPrefix {
    Stock,
    Workshop,
    CurrentDir
}


//----------------------------------------------
//           STOCK BUILDINGS MAP

pub type StockBuildingsMap<'stock> = HashMap<&'stock str, (&'stock str, StockBuilding<'stock>)>;

#[derive(Debug)]
pub enum StockBuilding<'stock> {
    Unparsed(&'stock str),
    Parsed(BuildingDef<'stock>)
}


//--------------------------------------------------------
impl BuildingDef<'_> {
    pub fn validate(&self) {
        assert!(self.building_ini.exists());
        assert!(path_option_valid(&self.imagegui));

        let _mtl_model = validate_modeldef(&self.model);
        let _mtl_model_lod1 = self.model_lod1.as_ref().map(validate_modeldef);
        let _mtl_model_lod2 = self.model_lod2.as_ref().map(validate_modeldef);
        let _mtl_model_emissive = self.model_emissive.as_ref().map(validate_modeldef);

        // NOTE: DEBUG
        //println!("Model's actual use of submaterials: {:?}", mtl_model);

        // TODO: look for *.mtl <-> *.nmf mismatches for all model types

        validate_material(&self.material.render_token.value, self.material.textures.as_slice());
        if let Some(m) = &self.material_emissive {
            validate_material(&m.render_token.value, m.textures.as_slice());
        }

        for skin in self.skins.iter() {
            validate_material(&skin.material.path, skin.material.textures.as_slice());
            if let Some(m) = &skin.material_emissive {
                validate_material(&m.path, m.textures.as_slice());
            }
        }

        //------------------------------------
        fn validate_material(pathbuf: &PathBuf, txs: &[IniTokenTexture]) {
            assert!(pathbuf.exists());
            assert!(txs.len() > 0);
            for tx in txs.iter() {
                assert!(tx.value.path.exists(), "Material missing texture: \"{}\"", tx.value.path.to_str().unwrap());
            }
        }

        // TODO: this function sucks
        fn validate_modeldef(_m: &ModelDef) -> Vec<String> {
            todo!()

/*
            assert!(m.ini_token.value.exists());

            let buf = fs::read(&m.ini_token.value).unwrap();
            let (nmf, rest) = nmf::Nmf::parse_bytes(buf.as_slice()).expect("Failed to parse the model nmf");
            // NOTE: debug
            //println!("{}", nmf);
            assert_eq!(rest.len(), 0, "Model nmf parsed with leftovers");

            let mut used: Vec<(&nmf::Submaterial, bool)> = nmf.submaterials.iter().zip(std::iter::repeat(false)).collect();

            #[inline]
            fn set_used<'a, 'b, T>(used: &mut Vec<(&'b nmf::Submaterial<'a>, bool)>, objs: T)
            where T: Iterator<Item = &'b nmf::Object<'a>> {
                for obj in objs {
                    for idx in obj.submaterials.iter() {
                        used[*idx].1 = true;
                    }
                };
            }
            
            if let Some(ref p) = m.patch {
                match p {
                    ModelPatch::Keep(keeps) => {
                        let objs = keeps.iter()
                                        .map(|k| nmf.objects.iter()
                                                            .find(|o| k == o.name.as_str().unwrap())
                                                            .expect(&format!("ModelPatch error: cannot find object to keep {:?}", k)));
                        set_used(&mut used, objs);
                    },
                    ModelPatch::Remove(rems) => {
                        let to_keep: Vec<&nmf::Object> = 
                            nmf.objects.iter()
                                       .filter(|o| !rems.iter().any(|r| *r == o.name.as_str().unwrap()))
                                       .collect();
                        
                        // TODO: this is not good
                        let undeleted = to_keep.len() + rems.len() - nmf.objects.len();
                        if undeleted != 0 {
                            panic!("ModelPatch error: cannot find {} objects to delete", undeleted);
                        }

                        set_used(&mut used, to_keep.iter().map(|x| *x));
                    }
                }
            } else {
                let objs = nmf.objects.iter();
                set_used(&mut used, objs);
            }

            used.iter()
                .filter_map(|(sm, b)| 
                    if *b {
                        Some(sm.name.as_str().unwrap().to_string()) 
                    } else {
                        None 
                    }
                ).collect()
        */
        }
    }
}


#[inline]
fn path_option_valid(opt: &Option<PathBuf>) -> bool {
    match opt {
        None => true,
        Some(ref p) => p.exists()
    }
}

/*
#[inline]
fn ini_token_valid(opt: &Option<IniTokenPath>) -> bool {
    match opt {
        None => true,
        Some(IniToken { value: ref p, .. }) => p.exists()
    }
}
*/

impl fmt::Display for BuildingDef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const INDENT: &str = "    ";

        write!(f, "{}renderconfig      {}\n", INDENT, self.render_config)?;
        write!(f, "{}building_ini      {}\n", INDENT, self.building_ini.to_str().unwrap())?;
        write!(f, "{}imagegui          ",     INDENT)?;
        write_path_option_ln(f, &self.imagegui)?;
        write!(f, "{}model             {}\n", INDENT, &self.model)?;

        write!(f, "{}model_lod1        ",     INDENT)?;
        write_option_ln(f, &self.model_lod1)?;
        write!(f, "{}model_lod2        ",     INDENT)?;
        write_option_ln(f, &self.model_lod2)?;
        write!(f, "{}model_emissive    ",     INDENT)?;
        write_option_ln(f, &self.model_emissive)?;

        write!(f, "{}material          {}\n", INDENT, &self.material)?;
        write!(f, "{}material_emissive ",     INDENT)?;
        write_option_ln(f, &self.model_emissive)?;

        write!(f, "{}Skins: {:#?}", INDENT, self.skins)?;

        return Ok(());

        //------------------------------------------------

        #[inline]
        fn write_option_ln<T>(f: &mut fmt::Formatter, option: &Option<T>) -> fmt::Result
        where T: fmt::Display {
            match option {
                None => write!(f, "<none>\n"),
                Some(ref p) => write!(f, "{}\n", p)
            }
        }

        #[inline]
        fn write_path_option_ln(f: &mut fmt::Formatter, option: &Option<PathBuf>) -> fmt::Result {
            write_option_ln(f, &option.as_ref().map(|x| x.to_str().unwrap()))
        }
    }
}



//--------------------------------------------------------
impl fmt::Display for RenderConfig<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderConfig::Stock { key, .. } => write!(f, "Stock '{}'", key),
            RenderConfig::Mod(path) => write!(f, "Mod '{}'", path.to_str().unwrap())
        }
    }
}

impl<'stock> RenderConfig<'stock> {
    pub fn root_path(&self) -> &Path {
        match self {
            RenderConfig::Stock { .. } => APP_SETTINGS.path_stock.as_path(),
            RenderConfig::Mod(render_cfg_path) => render_cfg_path.parent().unwrap()
        }
    }
}

//--------------------------------------------------------
impl ModelDef {
    #[inline]
    pub fn new(ini_token: IniTokenPath) -> ModelDef {
        ModelDef { ini_token, patch: None }
    }
}

impl fmt::Display for ModelDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",  self.ini_token)?;
        if let Some(ref p) = self.patch {
            write!(f, " ({})", p)
        } else { Ok(()) }
    }
}

//--------------------------------------------------------
        /*
impl ModelPatch {
    pub fn apply<'data>(&self, src: &nmf::Nmf<'data>) -> nmf::Nmf<'data> {

        // TODO
        todo!()

        let mut sm_usage: Vec<Option<usize>> = vec![None; src.submaterials.len()];
        let mut set_used = |obj: &nmf::Object<'data>| for &idx in obj.submaterials.iter() {
            sm_usage[idx] = Some(idx);
        };

        // Removing objects
        let mut objects: Vec<_> = match self {
            ModelPatch::Keep(keeps) => keeps.iter().map(|k| {
                let obj = src.objects.iter()
                    .find(|o| o.name.as_str().unwrap() == k)
                    .expect(&format!("ModelPatch error: cannot find object to keep - '{}'", k));
                
                set_used(&obj);
                obj.clone()
            }).collect(),

            ModelPatch::Remove(rems) => {
                let mut rems: Vec<&str> = rems.iter().map(|r| r.as_str()).collect();
                let kept = src.objects.iter().filter_map(|o| {
                    if let Some((i, _)) = rems.iter().enumerate().find(|(_, &r)| r == o.name.as_str().unwrap()) {
                        rems.remove(i);
                        None
                    } else {
                        set_used(&o);
                        Some(o.clone())
                    }
                }).collect();

                if !rems.is_empty() {
                    panic!("ModelPatch error: could not delete some objects ({:?})", rems);
                }

                kept
            }
        };

        // Removing unused submaterials
        let mut offset = 0usize;
        for new_i in sm_usage.iter_mut() {
            if let Some(idx) = *new_i {
                *new_i = Some(idx - offset);
            } else {
                offset += 1;
            }
        }

        // NOTE: DEBUG
        // println!("sm usage: {:?}", &sm_usage);

        let submaterials = sm_usage.iter().enumerate().filter_map(|(i, opt)| 
            opt.map(|_| src.submaterials[i].clone())
        ).collect();

        // fixing objects' submaterial references
        for obj in objects.iter_mut() {
            for old_idx in obj.submaterials.iter_mut() {
                let new_idx = sm_usage[*old_idx].unwrap();
                *old_idx = new_idx;
            }
        }
        
        nmf::Nmf {
            header: src.header,
            submaterials,
            objects
        }
 
    }
}
*/

impl fmt::Display for ModelPatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            ModelPatch::Keep(v)   => { write!(f, "patch-keep: {{")?; v },
            ModelPatch::Remove(v) => { write!(f, "patch-remove: {{")?; v }
        };
        
        for x in v.iter() {
            write!(f, " \"{}\";", x)?;
        }

        write!(f, " }}")
    }
}

impl TryFrom<&str> for ModelPatch 
{
    type Error = String;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RX_LINES: Regex = Regex::new(r"\r?\n").unwrap();
        }

        let mut lines = RX_LINES.split(text);
        let ptype = lines.next().ok_or_else(|| String::from("Cannot parse ModelPatch"))?;

        let mut tokens = Vec::<String>::with_capacity(64);
        for l in lines {
            if l.len() > 0 {
                tokens.push(String::from(l));
            }
        }

        match ptype {
            "KEEP" => Ok(ModelPatch::Keep(tokens)),
            "REMOVE" => Ok(ModelPatch::Remove(tokens)),
            z => Err(format!("Unknown patch action: '{}'", z))
        }
    }
} 


//--------------------------------------------------------
impl<T> From<(Range<usize>, T)> for IniToken<T> {
    #[inline]
    fn from((range, value): (Range<usize>, T)) -> IniToken<T> {
        IniToken { range, value }
    }
}

impl fmt::Display for IniTokenPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}..{}) {}", self.range.start, self.range.end, self.value.to_str().unwrap())
    }
}


//--------------------------------------------------------
impl MaterialDef {
    pub fn new(render_token: IniTokenPath) -> Result<MaterialDef, String> {
        let textures = get_material_textures(&render_token.value)?;
        Ok(MaterialDef { render_token, textures })
    }
}

impl fmt::Display for MaterialDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let t = &self.render_token;
        write!(f, "({}..{}) {} ({} textures)", t.range.start, t.range.end, t.value.to_str().unwrap(), self.textures.len())
        //write!(f, "({}..{}) {} (textures: {:#?})", t.range.start, t.range.end, t.value.to_str().unwrap(), self.textures)
    }
}


//--------------------------------------------------------
impl TryFrom<&str> for PathPrefix {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "~/" => Ok(Self::Stock),
            "$/" => Ok(Self::Workshop),
            "./" => Ok(Self::CurrentDir),
            p => Err(format!("Unknown path prefix '{}'", p))
        }
    }
}




pub fn get_material_textures(material_path: &Path) -> Result<Vec<IniTokenTexture>, String> {
    let material_dir = material_path.parent().unwrap();

    match material_path.extension().and_then(|x| x.to_str()) {
        Some("mtl") => {
            let material_src = fs::read_to_string(material_path)
                .map_err(|e| format!("Cannot read material file {:?}: {}", material_path, e))?;
            get_texture_tokens(material_dir, &material_src)
                .map_err(|e| format!("Cannot get texture tokens from {:?}: {}", material_path, e))
        },
        Some("mtlx") => {
            let material_src = fs::read_to_string(material_path)
                .map_err(|e| format!("Cannot read material file {:?}: {}", material_path, e))?;
            get_texture_tokens_ext(material_dir, &material_src)
                .map_err(|e| format!("Cannot get texture tokens from {:?}: {}", material_path, e))
        },
        Some(ext) => Err(format!("Unsupported material file extension: '{}'", ext)),
        None => Err(String::from("Material file is missing extension")),
    }
}


fn get_texture_tokens(mtl_dir: &Path, mtl_src: &str) -> Result<Vec<IniToken<Texture>>, String> {
    use path_slash::PathBufExt;

    lazy_static! {
        static ref RX: Regex = Regex::new(concatcp!(r"(?m)^(\$TEXTURE(_MTL)?\s+?([012])\s+?", AppSettings::SRX_PATH, ")", AppSettings::SRX_EOL)).unwrap();
    }

    let res = RX.captures_iter(&mtl_src).map(move |cap| {
        let range = cap.get(1).unwrap().range();
        // NOTE: Debug
        // println!("CAPTURE: {:?}, {:?}", &range, cap.get(1).unwrap().as_str());
        let is_mtl = cap.get(2).is_some();
        let num = cap.get(3).unwrap().as_str().chars().next().unwrap();
        let tx_path_str = cap.get(4).unwrap().as_str();

        let tx_root = if is_mtl { 
            mtl_dir 
        } else {
            APP_SETTINGS.path_stock.as_path()
        };

        let path = tx_root.join(PathBuf::from_slash(tx_path_str));

        IniToken {
            range,
            value: Texture { num, path }
        }
    }).collect();

    Ok(res)
}

fn get_texture_tokens_ext(mtlx_dir: &Path, mtlx_src: &str) -> Result<Vec<IniTokenTexture>, String> {

    lazy_static! {
        static ref RX_LINE:   Regex = Regex::new(r"(?m)^\$TEXTURE_EXT\s+([^\r\n]+)").unwrap();
        static ref RX_VAL:    Regex = Regex::new(concatcp!(r"([012])\s+", "\"", AppSettings::SRX_PATH_PREFIX, AppSettings::SRX_PATH_EXT, "\"")).unwrap();
        static ref RX_REJECT: Regex = Regex::new(r"(?m)^\s*\$TEXTURE(_MTL)?\s").unwrap();
    }

    if RX_REJECT.is_match(&mtlx_src) {
        return Err(String::from("Invalid mtlx file: $TEXTURE and $TEXTURE_MTL tokens are not allowed here."));
    }

    let mut res = Vec::new();
    for cap_line in RX_LINE.captures_iter(&mtlx_src) {
        let m = cap_line.get(0).unwrap();
        let range = m.range();
        // NOTE: Debug
        //println!("[MTLX] Captured line at {:?}: [{}]", &range, m.as_str());

        let values_str = cap_line.get(1).unwrap().as_str();
        if let Some(cap) = RX_VAL.captures(values_str) {

            let num = cap.get(1).unwrap().as_str().chars().next().unwrap();
            let tx_path_pfx: PathPrefix = cap.get(2).unwrap().as_str().try_into()?;
            let tx_path_str = cap.get(3).unwrap().as_str();
            let path = resolve_prefixed_path(tx_path_pfx, tx_path_str, mtlx_dir);

            res.push(IniToken {
                range,
                value: Texture { num, path }
            });
        } else {
            return Err(format!("Invalid MATERIAL_EXT line: [{}]", m.as_str()));
        }
    }

    Ok(res)
}


//---------------------------------------------------------

pub fn resolve_prefixed_path(pfx: PathPrefix, path_str: &str, local_root: &Path) -> PathBuf {
    use path_slash::PathBufExt;

    let root = match pfx {
        PathPrefix::Stock => APP_SETTINGS.path_stock.as_path(),
        PathPrefix::Workshop => APP_SETTINGS.path_workshop.as_path(),
        PathPrefix::CurrentDir => local_root,
    };

    root.join(PathBuf::from_slash(path_str))
}


pub fn read_to_string_buf(path: &Path, buf: &mut String) {
    use std::io::Read;

    if let Ok(mut file) = fs::File::open(path) {
        let meta = file.metadata().unwrap();
        let sz: usize = meta.len().try_into().unwrap();
        buf.reserve(sz);
        file.read_to_string(buf).unwrap();
    } else {
        panic!("Cannot read file \"{}\"", path.display());
    }
}

/*
fn read_to_buf(path: &Path, buf: &mut Vec<u8>) {
    use std::io::Read;

    if let Ok(mut file) = fs::File::open(path) {
        let meta = file.metadata().unwrap();
        let sz: usize = meta.len().try_into().unwrap();
        buf.reserve(sz);
        file.read(buf).unwrap();
    } else {
        panic!("Cannot read file \"{}\"", path.display());
    }
}
*/

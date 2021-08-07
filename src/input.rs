use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::convert::{TryInto};

use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

use crate::cfg::{AppSettings, APP_SETTINGS};

use crate::data::{StockBuilding, StockBuildingsMap, RenderConfig, 
                  BuildingDef, ModelDef, MaterialDef, Skin, SkinMaterial, 
                  PathPrefix, IniTokenPath,

                  get_material_textures, resolve_prefixed_path, read_to_string_buf};


#[derive(Debug)]
enum SourceType<'a> {
    Stock(&'a str),
    Mod(PathBuf)
}


pub fn read_validate_sources<'stock>(src: &Path, stock_buildings: &mut StockBuildingsMap<'stock>) -> Result<Vec<BuildingDef<'stock>>, Vec<String>> {

    let mut buf_sources = String::with_capacity(512);
    let mut pathbuf = src.to_path_buf();
    let mut data: Vec<BuildingDef<'stock>> = Vec::with_capacity(1000);
    let mut errors = Vec::<String>::with_capacity(0);

    push_buildings(&mut pathbuf, &mut data, &mut errors, &mut buf_sources, stock_buildings, &mut String::with_capacity(20));

    if data.len() > AppSettings::MAX_BUILDINGS {
        errors.push(format!("There are too many source objects ({}). Max supported is {}.", data.len(), AppSettings::MAX_BUILDINGS));
    }

    if errors.is_empty() {
        Ok(data)
    } else {
        Err(errors)
    }
}


fn push_buildings<'stock>(dirbuf: &mut PathBuf, 
                          data: &mut Vec<BuildingDef<'stock>>,
                          errors: &mut Vec<String>,
                          buf_sources: &mut String,
                          stock_buildings: &mut StockBuildingsMap<'stock>,
                          indent: &mut String
                          )
{
    lazy_static! {
        static ref RX_SOURCE_STOCK: Regex = Regex::new(r"^#([_[:alnum:]]+)").unwrap();
        static ref RX_SOURCE_MOD: Regex = Regex::new(r"^[0-9]{10}\\[^\s\r\n]+").unwrap();
    }

    dirbuf.push("building.source");
    if dirbuf.exists() {
        // TODO: move into separate function and do proper error handling
        // leaf dir (building)
        let source_path = dirbuf.clone();
        dirbuf.pop();

        println!("{}* {}", indent, dirbuf.file_name().unwrap().to_str().unwrap());

        buf_sources.clear();
        match fs::File::open(&source_path) {
            Ok(mut f) => {
                match f.read_to_string(buf_sources) {
                    Ok(_) => (),
                    Err(e) => {
                        errors.push(format!("Cannot read source file {:?}: {}", source_path, e));
                        return;
                    }
                }
            },
            Err(e) => {
                errors.push(format!("Cannot open source file {:?}: {}", source_path, e));
                return;
            }
        }

        let src_type = {
            if let Some(src_stock) = RX_SOURCE_STOCK.captures(&buf_sources) {
                SourceType::Stock(src_stock.get(1).unwrap().as_str())
            } else if let Some(src_mod) = RX_SOURCE_MOD.find(&buf_sources) {
                SourceType::Mod(APP_SETTINGS.path_workshop.join(src_mod.as_str()))
            } else {
                errors.push(format!("Cannot parse source file {:?}", source_path));
                return;
            }
        };

        match source_to_def(&dirbuf, src_type, stock_buildings) {
            Ok(b) => data.push(b),
            Err(e) => {
                errors.push(format!("Cannot construct BuildingDef from {:?}: {}", source_path, e));
                return;
            }
        }
    } else {
        dirbuf.pop();

        println!("{}{}", indent, dirbuf.file_name().unwrap().to_str().unwrap());

        for subdir in get_subdirs(&dirbuf) {
            let dir_name = subdir.file_name();
            if dir_name.to_str().unwrap().starts_with(&['_', '.'][..]) {
                continue;
            }

            let old_indent = indent.len();
            indent.push_str("  ");
            dirbuf.push(dir_name);

            push_buildings(dirbuf, data, errors, buf_sources, stock_buildings, indent);

            dirbuf.pop();
            indent.truncate(old_indent);
        }
    }
}

fn get_subdirs(path: &Path) -> impl Iterator<Item=fs::DirEntry>
{
    fs::read_dir(path)
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_dir())
}

fn source_to_def<'ini, 'map>(path: &Path, source_type: SourceType, hmap: &'map mut StockBuildingsMap<'ini>) -> Result<BuildingDef<'ini>, String> {
    let mut def = match source_type {
        SourceType::Stock(key) => {
            get_stock_building(&key, hmap)?
        },
        SourceType::Mod(mut bld_dir_path) => {
            bld_dir_path.push("renderconfig.ini");
            parse_ini_to_def(RenderConfig::Mod(bld_dir_path))?
        }
    };

    // overriding with custom files (if they exist in dir):
    // ---------------------------

    let mut pathbuf = path.to_path_buf();
    
    pathbuf.push("building.ini");
    if pathbuf.exists() { 
        def.building_ini.push(&pathbuf) 
    }

    pathbuf.set_file_name("imagegui.png");
    if pathbuf.exists() {
        def.imagegui.replace(pathbuf.clone());
    }

    // TODO
    //pathbuf.set_file_name("model.patch");
    //if pathbuf.exists() {
        //let pfile = fs::read_to_string(&pathbuf).unwrap();
        //let patch = ModelPatch::try_from(pfile.as_str()).map_err(|e| format!("Cannot parse ModelPatch at '{:?}': {}", &pathbuf, e))?;
        //def.model.patch = Some(patch);
    //}

    pathbuf.set_file_name("building.skins");
    if pathbuf.exists() {
        def.skins = get_skins(&pathbuf).map_err(|e| format!("Cannot get skins data from {:?}: {}", &pathbuf, e))?;
    }

    // TODO: continue other errors
    pathbuf.set_file_name("material.mtlx");
    if pathbuf.exists() {
        def.material.render_token.value.push(&pathbuf);
        def.material.textures = get_material_textures(&pathbuf).map_err(|e| format!("Material {:?}: {}", &pathbuf, e))?;
    }

    pathbuf.set_file_name("material_e.mtlx");
    if pathbuf.exists() {
        if let Some(ref mut mat_e) = def.material_emissive {
            mat_e.render_token.value.push(&pathbuf);
            mat_e.textures = get_material_textures(&pathbuf).map_err(|e| format!("Emissive material {:?}: {}", &pathbuf, e))?;
        }
        else {
            panic!("Trying to override material_e, while renderconfig does not have it");
        }
    }

    pathbuf.pop();
    // -----------------------------

    // NOTE: Debug
    //println!("{}", &def);

    def.validate();

    Ok(def)
}


fn get_stock_building<'a, 'ini, 'map>(key: &'a str, hmap: &'map mut StockBuildingsMap<'ini>) -> Result<BuildingDef<'ini>, String> {
    if let Some(mref) = hmap.get_mut(key) {
        match mref {
            (_, StockBuilding::Parsed(ref x)) => Ok(x.clone()),
            (k, StockBuilding::Unparsed(y)) => {
                let x = parse_ini_to_def(RenderConfig::Stock { key: k, data: y })?; 
                *mref = (k, StockBuilding::Parsed(x.clone()));
                Ok(x)
            }
        }
    } else {
        Err(format!("Cannot find stock building with key '{}'", key))
    }
}

fn parse_ini_to_def<'ini>(render_config: RenderConfig<'ini>) -> Result<BuildingDef<'ini>, String> {

    fn mk_tokenpath_rx(token: &str) -> Regex {
        Regex::new(&format!(r"(?m)^\s?{}\s+?{}{}", token, AppSettings::SRX_PATH, AppSettings::SRX_EOL))
        .unwrap()
    }

    lazy_static! {
        static ref RX_MODEL:      Regex = mk_tokenpath_rx("MODEL");
        static ref RX_MODEL_LOD1: Regex = mk_tokenpath_rx("MODEL_LOD");
        static ref RX_MODEL_LOD2: Regex = mk_tokenpath_rx("MODEL_LOD2");
        static ref RX_MODEL_E:    Regex = mk_tokenpath_rx("MODELEMISSIVE");

        static ref RX_MATERIAL:   Regex = mk_tokenpath_rx("MATERIAL");
        static ref RX_MATERIAL_E: Regex = mk_tokenpath_rx("MATERIALEMISSIVE");
    }

    let mut buf_mod_renderconfig = String::with_capacity(0);
    let root_path = render_config.root_path();

    let (render_source, building_ini, imagegui) = match render_config {
        RenderConfig::Stock { key, data } => {
            let mut building_ini = root_path.join("buildings_types");
            building_ini.push(format!("{}.ini", key));

            (data, building_ini, None)
        },
        RenderConfig::Mod(ref cfg_path) => {
            read_to_string_buf(cfg_path.as_path(), &mut buf_mod_renderconfig);

            let bld_ini = root_path.join("building.ini");
            let imgui   = root_path.join("imagegui.png");
            let imgui = if imgui.exists() { Some(imgui) } else { None };

            (buf_mod_renderconfig.as_str(), bld_ini, imgui)
        }
    };

    let model          = grep_ini_token(&RX_MODEL,      render_source, root_path).map(ModelDef::new).unwrap();
    let model_lod1     = grep_ini_token(&RX_MODEL_LOD1, render_source, root_path).map(ModelDef::new);
    let model_lod2     = grep_ini_token(&RX_MODEL_LOD2, render_source, root_path).map(ModelDef::new);
    let model_emissive = grep_ini_token(&RX_MODEL_E,    render_source, root_path).map(ModelDef::new);

    let material = MaterialDef::new(grep_ini_token(&RX_MATERIAL, render_source, root_path).unwrap())?;
    let material_emissive = grep_ini_token(&RX_MATERIAL_E, render_source, root_path).map(|x| MaterialDef::new(x)).transpose()?;

    Ok(BuildingDef { 
        render_config, building_ini, imagegui,
        model, model_lod1, model_lod2, model_emissive, 
        material, material_emissive, skins: Vec::with_capacity(0)
    })
}



fn get_skins(skinfile_path: &PathBuf) -> Result<Vec<Skin>, String> {
    const SKIN_RX: &str = concatcp!(
        r"(?m)^", 
        AppSettings::SRX_PATH_PREFIX, 
        AppSettings::SRX_PATH, 
        r"(\s+?\+\s+?", 
        AppSettings::SRX_PATH_PREFIX, 
        AppSettings::SRX_PATH, 
        r")?\r\n"
    );

    lazy_static! {
        static ref RX: Regex = Regex::new(SKIN_RX).unwrap();
    }

    let mut result = {
        let md = fs::metadata(skinfile_path).map_err(|e| format!("Cannot get size of skins file '{:?}'. Error: {}", skinfile_path, e))?;
        let c: u64 = md.len() / 50 + 1;
        Vec::with_capacity(c.try_into().unwrap())
    };

    let cfg = fs::read_to_string(skinfile_path).map_err(|e| format!("Cannot read skins file '{:?}'. Error: {}", skinfile_path, e))?;
    let skinfile_dir = skinfile_path.parent().unwrap();

    for cap in RX.captures_iter(&cfg) {
        let type1: PathPrefix = cap.get(1).unwrap().as_str().try_into()?;
        let path1 = cap.get(2).unwrap().as_str();
        let m_path = resolve_prefixed_path(type1, path1, skinfile_dir);

        let material = SkinMaterial { 
            path: m_path.to_path_buf(),
            textures: get_material_textures(&m_path)? 
        };

        let material_emissive = cap.get(4).map(|x| -> Result<SkinMaterial, String> {
            let type2: PathPrefix = x.as_str().try_into()?;
            let path2 = cap.get(5).unwrap().as_str();
            let m_path = resolve_prefixed_path(type2, path2, skinfile_dir);

            Ok(SkinMaterial {
                path: m_path.to_path_buf(),
                textures: get_material_textures(&m_path)?
            })
        }).transpose()?;

        result.push(Skin { material, material_emissive });
    }

    Ok(result)
}


fn grep_ini_token(rx: &Regex, source: &str, root: &Path) -> Option<IniTokenPath> {
    use path_slash::PathBufExt;

    rx.captures(source).and_then(|cap| {
        let m = cap.get(1)?;
        let pth = root.join(PathBuf::from_slash(m.as_str()));
        Some(IniTokenPath { range: m.range(), value: pth })
    })
}



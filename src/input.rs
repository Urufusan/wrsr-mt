//use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::Read;

use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

use crate::{
    StockBuilding, RenderConfig, StockBuildingsMap,
    Category, Style, BuildingDef, MaterialDef, Skin, SkinMaterial,

    grep_ini_token, get_texture_tokens,

    SRX_PATH, SRX_EOL, 
    ROOT_MODS, PATH_ROOT_STOCK, PATH_ROOT_MODS,
    MAX_BUILDINGS,
    };


#[derive(Debug)]
enum SourceType<'a> {
    Stock(&'a str),
    Mod(PathBuf),
}


pub(crate) fn read_validate_sources<'ini>(src: &Path, stock_buildings: &mut StockBuildingsMap<'ini>) -> Vec<Category<'ini>> {

    let mut buf_sources = String::with_capacity(512);
    
    let rx_source_stock = Regex::new(r"^#([_[:alnum:]]+)").unwrap();
    let rx_source_mod = Regex::new(r"^[0-9]{10}\\[_[:alnum:]]+").unwrap();

    let mut pathbuf = src.to_path_buf();
    let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
    let mut categories = Vec::<Category>::with_capacity(subdirs.len());

    let mut bld_count = 0usize;

    for dir_cat in subdirs {
        let dir_name = dir_cat.file_name();
        pathbuf.push(&dir_name);

        let (cat_pfx, cat_name) = get_dir_parts(&dir_name);
        println!("{}: {}", &cat_pfx, &cat_name);

        let mut cat = Category::new(cat_pfx, cat_name);
        let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
        cat.styles.reserve_exact(subdirs.len());

        for dir_style in subdirs.iter() {
            let dir_name = dir_style.file_name();
            pathbuf.push(&dir_name);

            let (style_pfx, style_name) = get_dir_parts(&dir_name);
            println!(" {}: {}", &style_pfx, &style_name);

            let mut style = Style::new(style_pfx, style_name);
            let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
            style.buildings.reserve_exact(subdirs.len());

            for dir_bld in subdirs {
                bld_count += 1;
                assert!(bld_count <= MAX_BUILDINGS);

                let dir_name = dir_bld.file_name();
                pathbuf.push(&dir_name);

                println!("  Building '{}'", dir_name.to_str().unwrap());

                pathbuf.push("building.source");
                buf_sources.clear();
                File::open(&pathbuf).unwrap().read_to_string(&mut buf_sources).unwrap();
                pathbuf.pop(); //pop .source

                let src_type: SourceType = {
                    if let Some(src_stock) = rx_source_stock.captures(&buf_sources) {
                        SourceType::Stock(src_stock.get(1).unwrap().as_str())
                    } else if let Some(src_mod) = rx_source_mod.find(&buf_sources) {
                        SourceType::Mod([ROOT_MODS, src_mod.as_str()].iter().collect())
                    } else {
                        panic!("Cannot parse building source ({:?})", &buf_sources);
                    }
                };

                style.buildings.push(source_to_def(&mut pathbuf, src_type, stock_buildings));
                pathbuf.pop(); // pop building dir
            }

            cat.styles.push(style);
            pathbuf.pop(); // pop style dir
        }

        categories.push(cat);
        pathbuf.pop(); // pop caterory dir
    }

    categories
}


fn get_dir_parts<'a, 'b>(dir_name: &'a std::ffi::OsStr) -> (&'a str, &'a str) {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"^(\d+?) - (.+)$").unwrap();
    }

    let c = RX.captures(dir_name.to_str().unwrap()).unwrap();
    (c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str())
}


fn get_subdirs(path: &PathBuf) -> impl Iterator<Item=fs::DirEntry>
{
    fs::read_dir(path)
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_dir())
}


fn source_to_def<'ini, 'map>(pathbuf: &mut PathBuf, source_type: SourceType, hmap: &'map mut StockBuildingsMap<'ini>) -> BuildingDef<'ini> {
    let mut def = match source_type {
        SourceType::Stock(key) => {
            get_stock_building(&key, hmap).unwrap()
        },
        SourceType::Mod(_path) => {
            todo!()
        }
    };
    

    // TODO: overriding with custom files (if they exist in dir):
    // ---------------------------
    pathbuf.push("building.ini");
    if pathbuf.exists() { 
        def.building_ini.push(&pathbuf) 
    }

    pathbuf.set_file_name("imagegui.png");
    if pathbuf.exists() {
        def.imagegui.replace(pathbuf.clone());
    }

    pathbuf.set_file_name("building.skins");
    if pathbuf.exists() {
        def.skins = get_skins(&pathbuf);
    }

    pathbuf.pop();
    // -----------------------------

    // NOTE: Debug
    //println!("{}", &def);

    def.validate_paths();
    def
}


fn get_stock_building<'a, 'ini, 'map>(key: &'a str, hmap: &'map mut StockBuildingsMap<'ini>) -> Option<BuildingDef<'ini>> {
    if let Some(mref) = hmap.get_mut(key) {
        match mref {
            (_, StockBuilding::Parsed(ref x)) => Some(x.clone()),
            (k, StockBuilding::Unparsed(y)) => {
                let x = parse_ini_to_def(RenderConfig::Stock { key: k, data: y }); 
                *mref = (k, StockBuilding::Parsed(x.clone()));
                Some(x)
            }
        }
    } else { None }
}


fn parse_ini_to_def<'ini>(render_config: RenderConfig<'ini>) -> BuildingDef<'ini> {

    lazy_static! {
        static ref RX_MODEL:      Regex = Regex::new(concatcp!(r"(?m)^\sMODEL\s+?",            SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_LOD1: Regex = Regex::new(concatcp!(r"(?m)^\sMODEL_LOD\s+?",        SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_LOD2: Regex = Regex::new(concatcp!(r"(?m)^\sMODEL_LOD2\s+?",       SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_E:    Regex = Regex::new(concatcp!(r"(?m)^\sMODELEMISSIVE\s+?",    SRX_PATH, SRX_EOL)).unwrap();

        static ref RX_MATERIAL:   Regex = Regex::new(concatcp!(r"(?m)^\sMATERIAL\s+?",         SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MATERIAL_E: Regex = Regex::new(concatcp!(r"(?m)^\sMATERIALEMISSIVE\s+?", SRX_PATH, SRX_EOL)).unwrap();
    }

    let root_path = render_config.root_path();

    let (render_source, building_ini, bbox, fire) = match render_config {
        RenderConfig::Stock { key, data } => {
            let mut bld_ini = root_path.join("buildings_types");

            let bbox = bld_ini.join(format!("{}.bbox", key));
            let fire = bld_ini.join(format!("{}.fire", key));
            bld_ini.push(format!("{}.ini", key));

            (data, bld_ini, bbox, fire)
        },
        RenderConfig::Mod(_path) => {
            // TODO: read from mod folder
            todo!()
        }
    };

    let fire = if fire.exists() { Some(fire) } else { None };

    let model =          grep_ini_token(&RX_MODEL,      render_source, root_path).unwrap();
    let model_lod1 =     grep_ini_token(&RX_MODEL_LOD1, render_source, root_path);
    let model_lod2 =     grep_ini_token(&RX_MODEL_LOD2, render_source, root_path);
    let model_emissive = grep_ini_token(&RX_MODEL_E,    render_source, root_path);

    let material = MaterialDef::new(grep_ini_token(&RX_MATERIAL, render_source, root_path).unwrap(), root_path);
    let material_emissive = grep_ini_token(&RX_MATERIAL_E, render_source, root_path).map(|x| MaterialDef::new(x, root_path));

    BuildingDef { 
        render_config, building_ini, bbox, fire, imagegui: None,
        model, model_lod1, model_lod2, model_emissive, 
        material, material_emissive, skins: Vec::with_capacity(0)
    }
}


// -------------------------------------------
fn get_skins(skinfile_path: &PathBuf) -> Vec<Skin> {
    const SRX_PATH_PREFIX: &str = "([~.$])/";
    lazy_static! {
        static ref RX: Regex = Regex::new(concatcp!(r"(?m)^", SRX_PATH_PREFIX, SRX_PATH, r"(\s+?\+\s+?", SRX_PATH_PREFIX, SRX_PATH, r")?\r\n")).unwrap();
    }

    // TODO: can estimate better
    let mut result = Vec::with_capacity(8);
    let cfg = fs::read_to_string(skinfile_path).unwrap();

    for cap in RX.captures_iter(&cfg) {
        let type1 = cap.get(1).unwrap().as_str();
        let path1 = cap.get(2).unwrap().as_str();

        let material = get_skin_material(type1, path1, skinfile_path.as_path());

        let material_emissive = cap.get(4).map(|x| {
            let type2 = x.as_str();
            let path2 = cap.get(5).unwrap().as_str();

            get_skin_material(type2, path2, skinfile_path.as_path())
        });

        result.push(Skin { material, material_emissive });
    }

    result
}


fn get_skin_material(path_type: &str, path: &str, local_path: &Path) -> SkinMaterial {
    use path_slash::PathBufExt;

    let root = match path_type {
        "~" => PATH_ROOT_STOCK.as_path(),
        "." => local_path,
        "$" => PATH_ROOT_MODS.as_path(),
        t => panic!("Unknown path type {}", t)
    };

    let path = root.join(PathBuf::from_slash(path));
    let src = fs::read_to_string(&path).unwrap();
    let textures = get_texture_tokens(&src, root);

    SkinMaterial { path, textures }
}

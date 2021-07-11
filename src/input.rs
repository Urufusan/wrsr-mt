//use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::Read;
use std::convert::TryInto;

use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

use crate::{
    StockBuilding, RenderConfig, StockBuildingsMap,
    BuildingDef, ModelDef, ModelPatch, MaterialDef, Skin, SkinMaterial, 
    PathPrefix, Texture, IniToken, IniTokenTexture, IniTokenPath,

    get_texture_tokens,
    resolve_prefixed_path, read_to_string_buf,

    SRX_PATH_PREFIX, SRX_PATH, SRX_PATH_EXT, SRX_EOL, 
    PATH_ROOT_MODS,
    MAX_BUILDINGS,
    };


#[derive(Debug)]
enum SourceType<'a> {
    Stock(&'a str),
    Mod(PathBuf),
}


pub(crate) fn read_validate_sources<'stock>(src: &Path, stock_buildings: &mut StockBuildingsMap<'stock>) -> Vec<BuildingDef<'stock>> {

    let mut buf_sources = String::with_capacity(512);
    let mut pathbuf = src.to_path_buf();
    let mut data: Vec<BuildingDef<'stock>> = Vec::with_capacity(1000);

    push_buildings(&mut pathbuf, &mut data, &mut buf_sources, stock_buildings, &mut String::with_capacity(10));

    assert!(data.len() <= MAX_BUILDINGS);

    data
}


fn push_buildings<'stock>(pathbuf: &mut PathBuf, 
                          data: &mut Vec<BuildingDef<'stock>>,
                          buf_sources: &mut String,
                          stock_buildings: &mut StockBuildingsMap<'stock>,
                          indent: &mut String
                          )
{
    lazy_static! {
        static ref RX_SOURCE_STOCK: Regex = Regex::new(r"^#([_[:alnum:]]+)").unwrap();
        static ref RX_SOURCE_MOD: Regex = Regex::new(r"^[0-9]{10}\\[^\s\r\n]+").unwrap();
    }

    // NOTE: Debug
    // println!("+++ {:?} +++", &pathbuf);

    pathbuf.push("building.source");

    if pathbuf.exists() {
        // leaf dir (building)

        buf_sources.clear();
        File::open(&pathbuf).unwrap().read_to_string(buf_sources).unwrap();
        pathbuf.pop(); //pop .source

        println!("{}* {}", indent, pathbuf.file_name().unwrap().to_str().unwrap());

        let src_type: SourceType = {
            if let Some(src_stock) = RX_SOURCE_STOCK.captures(&buf_sources) {
                SourceType::Stock(src_stock.get(1).unwrap().as_str())
            } else if let Some(src_mod) = RX_SOURCE_MOD.find(&buf_sources) {
                SourceType::Mod(PATH_ROOT_MODS.join(src_mod.as_str()))
            } else {
                panic!("Cannot parse building source ({:?})", &buf_sources);
            }
        };

        data.push(source_to_def(pathbuf, src_type, stock_buildings));

        return;
    } else {
        pathbuf.pop();

        println!("{}{}", indent, pathbuf.file_name().unwrap().to_str().unwrap());

        for subdir in get_subdirs(&pathbuf) {
            let dir_name = subdir.file_name();
            if dir_name.to_str().unwrap().starts_with("_") {
                continue;
            }

            let old_indent = indent.len();
            indent.push_str("  ");
            pathbuf.push(dir_name);

            push_buildings(pathbuf, data, buf_sources, stock_buildings, indent);

            pathbuf.pop();
            indent.truncate(old_indent);
        }
    }
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
        SourceType::Mod(mut bld_dir_path) => {
            bld_dir_path.push("renderconfig.ini");
            parse_ini_to_def(RenderConfig::Mod(bld_dir_path))
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

    // TODO: read model patch
    def.model.patch = Some(ModelPatch::Remove(
        vec![String::from("Лепнина_1-235-0.004"),
             String::from("Куб"),
             String::from("Куб.001"),
             //String::from("Куб.003"),
             String::from("Куб.004"),
            ]
    ));

    pathbuf.set_file_name("building.skins");
    if pathbuf.exists() {
        def.skins = get_skins(&pathbuf);
    }

    pathbuf.set_file_name("material.mtlx");
    if pathbuf.exists() {
        def.material.render_token.value.push(&pathbuf);
        def.material.textures = get_texture_tokens_ext(&pathbuf);
    }

    pathbuf.set_file_name("material_e.mtlx");
    if pathbuf.exists() {
        if let Some(ref mut mat_e) = def.material_emissive {
            mat_e.render_token.value.push(&pathbuf);
            mat_e.textures = get_texture_tokens_ext(pathbuf);
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
        static ref RX_MODEL:      Regex = Regex::new(concatcp!(r"(?m)^\s?MODEL\s+?",            SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_LOD1: Regex = Regex::new(concatcp!(r"(?m)^\s?MODEL_LOD\s+?",        SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_LOD2: Regex = Regex::new(concatcp!(r"(?m)^\s?MODEL_LOD2\s+?",       SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MODEL_E:    Regex = Regex::new(concatcp!(r"(?m)^\s?MODELEMISSIVE\s+?",    SRX_PATH, SRX_EOL)).unwrap();

        static ref RX_MATERIAL:   Regex = Regex::new(concatcp!(r"(?m)^\s?MATERIAL\s+?",         SRX_PATH, SRX_EOL)).unwrap();
        static ref RX_MATERIAL_E: Regex = Regex::new(concatcp!(r"(?m)^\s?MATERIALEMISSIVE\s+?", SRX_PATH, SRX_EOL)).unwrap();
    }

    let mut buf_mod_renderconfig = String::with_capacity(0);
    let root_path = render_config.root_path();

    let (render_source, building_ini, bbox, fire, imagegui) = match render_config {
        RenderConfig::Stock { key, data } => {
            let mut building_ini = root_path.join("buildings_types");

            let bbox = building_ini.join(format!("{}.bbox", key));
            let fire = building_ini.join(format!("{}.fire", key));
            building_ini.push(format!("{}.ini", key));

            (data, building_ini, bbox, fire, None)
        },
        RenderConfig::Mod(ref cfg_path) => {
            read_to_string_buf(cfg_path.as_path(), &mut buf_mod_renderconfig);

            let bld_ini = root_path.join("building.ini");
            let bbox    = root_path.join("building.bbox");
            let fire    = root_path.join("building.fire");
            let imgui   = root_path.join("imagegui.png");
            let imgui = if imgui.exists() { Some(imgui) } else { None };

            (buf_mod_renderconfig.as_str(), bld_ini, bbox, fire, imgui)
        }
    };

    let fire = if fire.exists() { Some(fire) } else { None };

    let model      =     grep_ini_token(&RX_MODEL,      render_source, root_path).map(ModelDef::new).unwrap();
    let model_lod1 =     grep_ini_token(&RX_MODEL_LOD1, render_source, root_path).map(ModelDef::new);
    let model_lod2 =     grep_ini_token(&RX_MODEL_LOD2, render_source, root_path).map(ModelDef::new);
    let model_emissive = grep_ini_token(&RX_MODEL_E,    render_source, root_path).map(ModelDef::new);

    let material = MaterialDef::new(grep_ini_token(&RX_MATERIAL, render_source, root_path).unwrap());
    let material_emissive = grep_ini_token(&RX_MATERIAL_E, render_source, root_path).map(|x| MaterialDef::new(x));

    BuildingDef { 
        render_config, building_ini, bbox, fire, imagegui,
        model, model_lod1, model_lod2, model_emissive, 
        material, material_emissive, skins: Vec::with_capacity(0)
    }
}



fn get_skins(skinfile_path: &PathBuf) -> Vec<Skin> {
    lazy_static! {
        static ref RX: Regex = Regex::new(concatcp!(r"(?m)^", SRX_PATH_PREFIX, SRX_PATH, r"(\s+?\+\s+?", SRX_PATH_PREFIX, SRX_PATH, r")?\r\n")).unwrap();
    }

    let mut result = {
        let md = fs::metadata(skinfile_path).unwrap();
        let c: u64 = md.len() / 50 + 1;
        Vec::with_capacity(c.try_into().unwrap())
    };

    let cfg = fs::read_to_string(skinfile_path).unwrap();
    let skinfile_dir = skinfile_path.parent().unwrap();

    for cap in RX.captures_iter(&cfg) {
        let type1: PathPrefix = cap.get(1).unwrap().as_str().try_into().unwrap();
        let path1 = cap.get(2).unwrap().as_str();
        let m_path = resolve_prefixed_path(type1, path1, skinfile_dir);

        let material = SkinMaterial { 
            path: m_path.to_path_buf(),
            textures: get_material_textures(&m_path) 
        };

        let material_emissive = cap.get(4).map(|x| {
            let type2: PathPrefix = x.as_str().try_into().unwrap();
            let path2 = cap.get(5).unwrap().as_str();
            let m_path = resolve_prefixed_path(type2, path2, skinfile_dir);

            SkinMaterial {
                path: m_path.to_path_buf(),
                textures: get_material_textures(&m_path)
            }
        });

        result.push(Skin { material, material_emissive });
    }

    result
}


fn get_material_textures(material_path: &Path) -> Vec<IniTokenTexture> {
    let ext = material_path.extension().unwrap();

    match ext.to_str().unwrap() {
        "mtl" => get_texture_tokens(material_path),
        "mtlx" => get_texture_tokens_ext(material_path),
        e => panic!("Unknown material extension '{}'", e)
    }
}


fn grep_ini_token(rx: &Regex, source: &str, root: &Path) -> Option<IniTokenPath> {
    use path_slash::PathBufExt;

    rx.captures(source).map(|cap| {
        let m = cap.get(1).unwrap();
        // NOTE: Debug
        // println!("CAPTURE: {:?}, {:?}", &m.range(), m.as_str());
        let pth = [root, PathBuf::from_slash(m.as_str()).as_path()].iter().collect();
        IniTokenPath { range: m.range(), value: pth }
    })
}


fn get_texture_tokens_ext(mtlx_path: &Path) -> Vec<IniTokenTexture> {

    lazy_static! {
        static ref RX_LINE: Regex = Regex::new(r"(?m)^\$TEXTURE_EXT\s+([^\r\n]+)").unwrap();
        static ref RX_VAL:  Regex = Regex::new(concatcp!(r"([012])\s+", "\"", SRX_PATH_PREFIX, SRX_PATH_EXT, "\"")).unwrap();
        static ref RX_REJECT: Regex = Regex::new(r"(?m)^\s*\$TEXTURE(_MTL)?\s").unwrap();
    }

    let ext = mtlx_path.extension().unwrap();
    assert_eq!(ext.to_str().unwrap(), "mtlx", "This function must be called only for *.mtlx files"); 

    let mtlx_dir = mtlx_path.parent().unwrap();
    let mtlx_src = fs::read_to_string(mtlx_path).expect(&format!("Cannot read mtlx file '{}'", mtlx_path.to_str().unwrap()));

    if RX_REJECT.is_match(&mtlx_src) {
        panic!("Invalid mtlx file ({}): $TEXTURE and $TEXTURE_MTL tokens are not allowed here.", mtlx_path.to_str().unwrap());
    }

    RX_LINE.captures_iter(&mtlx_src).map(move |cap_line| {
        let m = cap_line.get(0).unwrap();
        let range = m.range();
        // NOTE: Debug
        //println!("Captured line at {:?}: [{}]", &range, m.as_str());

        let values_str = cap_line.get(1).unwrap().as_str();
        if let Some(cap) = RX_VAL.captures(values_str) {

            let num = cap.get(1).unwrap().as_str().chars().next().unwrap();
            let tx_path_pfx: PathPrefix = cap.get(2).unwrap().as_str().try_into().unwrap();
            let tx_path_str = cap.get(3).unwrap().as_str();
            let path = resolve_prefixed_path(tx_path_pfx, tx_path_str, mtlx_dir);

            IniToken {
                range,
                value: Texture { num, path }
            }
        } else {
            panic!("Invalid MATERIAL_EXT line: [{}]", m.as_str());
        }
    }).collect()
}



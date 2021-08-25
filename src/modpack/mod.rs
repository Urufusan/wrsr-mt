use std::fs;
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::fmt::{self, Write as FmtWrite};

//use const_format::concatcp;
use regex::Regex;
use normpath::{BasePath, BasePathBuf, PathExt};
use lazy_static::lazy_static;

use crate::building_def::{ModBuildingDef, StockBuildingDef, BuildingError as DefError, StockBuildingsMap, fetch_stock_with_ini, fetch_stock_building};
use crate::cfg::{AppSettings, APP_SETTINGS, RENDERCONFIG_INI, BUILDING_INI};
use crate::{read_to_buf, read_to_string_buf};


pub enum SourceType {
    Mod(ModBuildingDef),
    Stock(StockBuildingDef),
}

pub struct BuildingSource {
    source_dir: PathBuf,
    def: SourceType,
    skins: Skins,
    actions: Option<()>,
}

type Skins = Vec<(PathBuf, Option<PathBuf>)>;

#[derive(Debug)]
pub enum SourceError{
    NoRenderconfig,
    MultiRenderconfig,
    Def(DefError),
    RefRead(std::io::Error),
    RefParse,
    Skins(String),
}

const RENDERCONFIG_SOURCE: &str = "renderconfig.source";
const RENDERCONFIG_REF: &str = "renderconfig.ref";
const BUILDING_SKINS: &str = "building.skins";

pub fn read_validate_sources(source_dir: &Path, stock: &mut StockBuildingsMap) -> Result<(Vec::<BuildingSource>, usize), usize> {
    let mut result = Vec::<BuildingSource>::with_capacity(10000);

    let mut errors: usize = 0;
    let mut skins_count: usize = 0;

    let mut str_buf = String::with_capacity(1024 * 16);
    let mut rev_buf = Vec::<PathBuf>::with_capacity(100);
    let mut backlog = Vec::<PathBuf>::with_capacity(100);
    backlog.push(source_dir.to_path_buf());

    while let Some(mut path) = backlog.pop() {
        macro_rules! log_err {
            ($err:expr $(, $v:expr)*) => {{
                errors += 1;
                eprintln!("{}: {}", path.strip_prefix(source_dir).expect("Impossible: could not strip root prefix").display(), $err);
                $($v)*
            }};
        }

        path.push(BUILDING_INI);
        if path.exists() {
            // try to push this building source
            let bld_ini = path.clone();

            path.set_file_name(RENDERCONFIG_SOURCE);
            let render_src = if path.exists() { Some(path.to_path_buf()) } else { None }; 
            path.set_file_name(RENDERCONFIG_REF);
            let render_ref = if path.exists() { Some(path.normalize_virtually().unwrap()) } else { None };

            path.pop();

            let building_source_type = match (render_src, render_ref) {
                (None, Some(render_ref)) => get_source_type_from_ref(bld_ini, render_ref, stock, &mut str_buf),
                (Some(render_src), None) => ModBuildingDef::from_render_path(&bld_ini, &render_src, resolve_source_path, true)
                                                .map_err(SourceError::Def)
                                                .map(SourceType::Mod),
                (None, None)       => Err(SourceError::NoRenderconfig), 
                (Some(_), Some(_)) => Err(SourceError::MultiRenderconfig),
            };

            let building_source = building_source_type.and_then(|def| {
                // NOTE: debug
                //println!("{}: {}", path.strip_prefix(source_dir).unwrap().display(), def);

                path.push(BUILDING_SKINS);
                let skins = if path.exists() {
                    read_skins(path.as_path(), &mut str_buf)
                } else { 
                    Ok(Skins::with_capacity(0))
                };
                path.pop();

                skins.and_then(|skins| {
                    skins_count += skins.len();
                    read_actions(&mut path).and_then(|actions| {
                        Ok(BuildingSource { source_dir: path.clone(), def, skins, actions })
                    })
                })
            });

            let building_source = building_source.and_then(|bs| {
                // TODO: check if skins cover active submaterials from the main model
                // TODO: check if actions are applicable (obj deletion?)
                Ok(bs)
            });

            match building_source {
                Ok(bs) => {
                    println!("{}: OK", path.strip_prefix(source_dir).expect("Impossible: could not strip root prefix").display());
                    result.push(bs)
                },
                Err(e) => log_err!(format!("{:?}", e))
            }
        } else {
            // try to push sub-dirs to backlog
            path.pop();
            match fs::read_dir(&path) {
                Ok(r_d) => {
                    for dir_entry in r_d {
                        if let Err(e) = dir_entry.and_then(|dir_entry| 
                            dir_entry.file_type().and_then(|filetype| {
                                if filetype.is_dir() && !dir_entry.file_name().to_string_lossy().starts_with(&['_', '.'][..]) {
                                    rev_buf.push(dir_entry.path());
                                }
                                Ok(())
                            })
                        ) { log_err!(e) }
                    }

                    while let Some(x) = rev_buf.pop() {
                        backlog.push(x);
                    }
                },
                Err(e) => log_err!(e)
            }
        }
    }

    if errors == 0 {
        Ok((result, skins_count))
    } else {
        Err(errors)
    }
}

fn read_skins(path: &Path, buf: &mut String) -> Result<Skins, SourceError> {
    lazy_static! {
        static ref RX_SKINS: Regex = Regex::new(r"(?s)^([^\s]+)(\s+([^\s]+))?$").unwrap();
    }

    // TODO: read skins
    read_to_string_buf(path, buf).map_err(|e| SourceError::Skins(format!("Could not read skins: {:?}", e)))?;
    let mut result = Skins::with_capacity(0);
    result.reserve(16);
    for cap in RX_SKINS.captures_iter(buf) {
        let mtl = resolve_source_path(&path.parent().unwrap().normalize_virtually().expect("skins path not normalized"), cap.get(1).unwrap().as_str());
        let mtl_e = cap.get(3).map(|x| x.as_str());
    }

    Ok(result)
}

fn read_actions(_path: &mut PathBuf) -> Result<Option<()>, SourceError> {
    //const BUILDING_ACTIONS: &str = "building.skins";
    Ok(Some(()))
}


type AssetsMap = std::collections::HashMap::<PathBuf, String>;

pub const MODPACK_LOG: &'static str = "modpack.log";
pub const MATERIAL_MTL: &'static str = "material.mtl";
pub const MATERIAL_E_MTL: &'static str = "material_e.mtl";
pub const WORKSHOPCONFIG: &'static str = "workshopconfig.ini";

pub fn install(sources: Vec<BuildingSource>, target: &Path, log_file: &mut BufWriter<fs::File>, stock_map: &mut StockBuildingsMap) {
    
    let mut pathbuf = target.to_path_buf();
    let mut assets_map = AssetsMap::with_capacity(10000);
    let mut str_buf = String::with_capacity(16 * 1024);
    let mut byte_buf = Vec::<u8>::with_capacity(32 * 1024 * 1024);

    let mut src_iter = sources.iter();
    for mod_id in AppSettings::MOD_IDS_START .. AppSettings::MOD_IDS_END {
        str_buf.truncate(0);
        write!(str_buf, "{}", mod_id).unwrap();
        pathbuf.push(&str_buf);
        for bld_id in 0 .. AppSettings::MAX_BUILDINGS_IN_MOD {
            if let Some(src) = src_iter.next() {
                str_buf.truncate(0);
                write!(str_buf, "{:0>2}", bld_id).unwrap();
                writeln!(log_file, "{}/{} {}", mod_id, &str_buf, src.source_dir.display()).unwrap();
                pathbuf.push(&str_buf);

                fs::create_dir_all(&pathbuf).unwrap();

                install_building(&src.def, &pathbuf, stock_map, &mut assets_map, &mut str_buf, &mut byte_buf).unwrap();

                pathbuf.pop();
            } else {
                pathbuf.push(WORKSHOPCONFIG);
                write_workshop_ini_buildings(pathbuf.as_path(), mod_id, bld_id, &mut str_buf);
                pathbuf.pop();
                return;
            }
        }

        pathbuf.push(WORKSHOPCONFIG);
        write_workshop_ini_buildings(pathbuf.as_path(), mod_id, AppSettings::MAX_BUILDINGS_IN_MOD, &mut str_buf);
        pathbuf.pop();
        pathbuf.pop();
    }
}

fn write_workshop_ini_buildings(path: &Path, mod_id: usize, count: usize, buf: &mut String) {
    if count == 0 {
        return;
    }

    buf.truncate(0);
    writeln!(buf, 
        "$ITEM_ID {}\n\
         $OWNER_ID 12345678901234567\n\
         $ITEM_TYPE WORKSHOP_ITEMTYPE_BUILDING\n\
         $VISIBILITY 2\n", 
        mod_id).unwrap();

    for i in 0 .. count {
        writeln!(buf, "$OBJECT_BUILDING {:0>2}", i).unwrap();
    }

    writeln!(buf, "\n$ITEM_NAME \"Automatically generated by wrsr-mt modpack installer\"\n\
                   $ITEM_DESC \"Automatically generated by wrsr-mt modpack installer\"\n\n\
                   $END").unwrap();

    fs::write(path, buf).unwrap();
}

fn install_building(src: &SourceType, 
                    destination: &Path, 
                    stock_map: &mut StockBuildingsMap, 
                    assets_map: &mut AssetsMap, 
                    str_buf: &mut String,
                    byte_buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
    use crate::ini;

    str_buf.truncate(0);
    byte_buf.truncate(0);

    let nmf_root = destination.parent().unwrap().parent().unwrap().join("nmf");
    fs::create_dir_all(&nmf_root)?;
    let dds_root = destination.parent().unwrap().parent().unwrap().join("dds");
    fs::create_dir_all(&dds_root)?;
    let new_render_path = destination.join(RENDERCONFIG_INI);

    let src_data = match src {
        SourceType::Mod(d) => {
            fs::copy(&d.render, &new_render_path)?;
            &d.data
        },
        SourceType::Stock(StockBuildingDef { render, data }) => {
            let mut new_render_file = fs::OpenOptions::new().write(true).create_new(true).open(&new_render_path)?;
            let (chunk, _) = fetch_stock_building(render, stock_map).expect("Invalid stock building source");
            write!(&mut new_render_file, "{}\r\n", ini::renderconfig::Token::TYPE_WORKSHOP)?;
            write!(new_render_file, "{}", chunk)?;
            &data
        }
    };

    let mut data = src_data.clone();

    //------------------- Local helper macros -----------------------

    macro_rules! copy_fld {
        ($src_fld:expr, $dest_fld:expr, $dest_name:expr) => {
            $dest_fld.push(destination);
            $dest_fld.push($dest_name);
            fs::copy(&$src_fld, &$dest_fld)?;
        };
    }

    macro_rules! copy_fld_opt {
        ($fld:ident, $dest_name:expr) => {
            if let (Some(src_fld), Some(dest_fld)) = (src_data.$fld.as_ref(), data.$fld.as_mut()) {
                copy_fld!(src_fld, dest_fld, $dest_name);
            }
        }
    }

    macro_rules! copy_asset_md5 {
        ($path:expr, $assets_root:ident, $asset_type:expr) => { 
            copy_asset_md5($path, &$assets_root, $asset_type, byte_buf, assets_map)
        }
    }
    
    macro_rules! copy_nmf_md5_opt { 
        ($fld:ident) => { 
            if let Some(path) = data.$fld.as_mut() {
                Some(copy_asset_md5!(path, nmf_root, "nmf")).transpose()
            } else { Ok(None) }
        };
    }

    macro_rules! update_tx_token {
        ($token:ident) => {{
            let mut tx_path = APP_SETTINGS.path_stock.join($token.as_str()).into_path_buf();
            let tx_token = copy_asset_md5!(&mut tx_path, dds_root, "dds").unwrap();
            ini::common::IdStringParam::new_owned(tx_token)
        }};
        ($token:ident, $mtl_root:expr) => {{
            let mut tx_path = resolve_source_path($mtl_root, $token.as_str()).into_path_buf();
            let tx_token = copy_asset_md5!(&mut tx_path, dds_root, "dds").unwrap();
            ini::common::IdStringParam::new_owned(tx_token)
        }};
    }

    macro_rules! update_mtl {
        ($path:expr, $old_mtl_path:expr) => {{
            let old_mtl_root = $old_mtl_path.parent().unwrap().normalize_virtually().unwrap();
            read_to_string_buf($path, str_buf)?;
            let mut mtl = ini::parse_mtl(str_buf).expect("Invalid *.mtl");
            for token_state in mtl.tokens_mut() {
                token_state.modify(|t| {
                    use ini::material::Token as MT;
                    
                    match t {
                        MT::Texture(        (i, p)) => Some(MT::TextureMtl(     (*i, update_tx_token!(p)) )),
                        MT::TextureNoMip(   (i, p)) => Some(MT::TextureNoMipMtl((*i, update_tx_token!(p)) )),
                        MT::TextureMtl(     (i, p)) => Some(MT::TextureMtl(     (*i, update_tx_token!(p, &old_mtl_root)) )),
                        MT::TextureNoMipMtl((i, p)) => Some(MT::TextureNoMipMtl((*i, update_tx_token!(p, &old_mtl_root)) )),
                        _ => None
                    }
                });
            }

            mtl.write_file($path)?;
        }}
    }

    //-----------------------------------------------------------------


    copy_fld!(src_data.building_ini, data.building_ini, BUILDING_INI);
    copy_fld!(src_data.material,     data.material,     MATERIAL_MTL);
    copy_fld_opt!(material_e, MATERIAL_E_MTL); 
    copy_fld_opt!(image_gui, "imagegui.png"); 
    
    // UPDATE renderconfig tokens
    {   
        let model_token:      String         = copy_asset_md5!(&mut data.model, nmf_root, "nmf")?;
        let model_lod_token:  Option<String> = copy_nmf_md5_opt!(model_lod)?;
        let model_lod2_token: Option<String> = copy_nmf_md5_opt!(model_lod2)?;
        let model_e_token:    Option<String> = copy_nmf_md5_opt!(model_e)?;

        // Update renderconfig.ini
        use ini::{renderconfig, parse_renderconfig_ini};

        read_to_string_buf(&new_render_path, str_buf)?;
        let mut render_ini = parse_renderconfig_ini(str_buf).expect("Invalid building renderconfig");
        for token_state in render_ini.tokens_mut() {
            token_state.modify(|t| {
                use renderconfig::Token as RT;
                use ini::common::IdStringParam;
                
                match t {
                    RT::Model(_)            => Some(RT::Model(IdStringParam::new_cloned(&model_token))),
                    RT::ModelLod((_, z))    => model_lod_token.as_ref().map(|t| RT::ModelLod((IdStringParam::new_cloned(t), *z))),
                    RT::ModelLod2((_, z))   => model_lod2_token.as_ref().map(|t| RT::ModelLod2((IdStringParam::new_cloned(t), *z))),
                    RT::ModelEmissive(_)    => model_e_token.as_ref().map(|t| RT::ModelEmissive(IdStringParam::new_cloned(t))),
                    RT::Material(_)         => Some(RT::Material(IdStringParam::new_cloned(MATERIAL_MTL))),
                    RT::MaterialEmissive(_) => data.material_e.as_ref().map(|_| RT::MaterialEmissive(IdStringParam::new_cloned(MATERIAL_E_MTL))),
                    _ => None
                }
            });
        }

        render_ini.write_file(new_render_path)?;
    }

    // Copy textures and update *.mtl files
    update_mtl!(&data.material, src_data.material);
    if let (Some(material_e), Some(src_mtl_e)) = (&data.material_e, &src_data.material_e) {
        update_mtl!(material_e, src_mtl_e);
    }

    Ok(())
}


lazy_static! {
    static ref RX_REF: Regex = Regex::new(r"^(#(\d{10}/[^\s]+))|(~([^\s]+))|([^\r\n]+)").unwrap();
}


fn get_source_type_from_ref(bld_ini: PathBuf, mut render_ref: BasePathBuf, stock: &mut StockBuildingsMap, buf: &mut String) -> Result<SourceType, SourceError> {
    read_to_string_buf(&render_ref, buf).map_err(SourceError::RefRead)?;
    let caps = RX_REF.captures(buf).ok_or(SourceError::RefParse)?;
    if let Some(c) = caps.get(4) {
        // stock, get def directly from stock buildings
        fetch_stock_with_ini(c.as_str(), stock, bld_ini)
            .map_err(SourceError::Def)
            .map(SourceType::Stock)
    } else {
        let mut root: BasePathBuf = if let Some(c) = caps.get(2) {
            // workshop
            Ok(APP_SETTINGS.path_workshop.join(c.as_str()))
        } else if let Some(c) = caps.get(5) {
            // relative path
            render_ref.pop().unwrap();
            Ok(render_ref.join(c.as_str()))
        } else {
            Err(SourceError::RefParse)
        }?;

        root.push(RENDERCONFIG_INI);
        ModBuildingDef::from_render_path(&bld_ini, root.as_path(), resolve_source_path, true)
            .map_err(SourceError::Def)
            .map(SourceType::Mod)
    }
}



fn resolve_source_path(local_root: &BasePath, tail: &str) -> BasePathBuf {
    let mut iter = tail.chars();
    let pfx = iter.next().expect("resolve_source_path called with empty tail");
    match pfx {
        '#' => APP_SETTINGS.path_workshop.join(iter.as_str()),
        '~' => APP_SETTINGS.path_stock.join(iter.as_str()),
        _   => local_root.join(tail)
    }
}

fn copy_asset_md5(path: &mut PathBuf, assets_root: &Path, asset_type: &'static str, byte_buf: &mut Vec<u8>, assets_map: &mut AssetsMap) -> Result<String, std::io::Error> {
    if let Some(v) = assets_map.get(path) {
        Ok(v.clone())
    } else {
        let new_key = path.to_path_buf();
        read_to_buf(&path, byte_buf)?;
        let asset_md5name = format!("{:x}.{}", md5::compute(byte_buf.as_mut_slice()), asset_type);
        path.push(&assets_root);
        path.push(&asset_md5name);
        if !path.exists() {
            fs::write(&path, byte_buf.as_slice())?;
        }

        let v = format!("../../{}/{}", asset_type, asset_md5name);
        assets_map.insert(new_key, v.clone());
        Ok(v)
    }
}

impl fmt::Display for BuildingSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "{}", self.def)?;
        writeln!(f, "skins: {:?}", self.skins)?;
        writeln!(f, "actions: {:?}", self.actions)
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            SourceType::Mod(def)   => write!(f, "mod {}", def),
            SourceType::Stock(def) => write!(f, "stock {}", def),
        }
    }
}

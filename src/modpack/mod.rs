use std::fs;
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::fmt;

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
    skins: Option<()>,
    actions: Option<()>,
}

#[derive(Debug)]
pub enum SourceError{
    NoRenderconfig,
    MultiRenderconfig,
    Def(DefError),
    RefRead(std::io::Error),
    RefParse,
}

const RENDERCONFIG_SOURCE: &str = "renderconfig.source";
const RENDERCONFIG_REF: &str = "renderconfig.ref";

pub fn read_validate_sources(source_dir: &Path, stock: &mut StockBuildingsMap) -> Result<(Vec::<BuildingSource>, usize), usize> {
    let mut result = Vec::<BuildingSource>::with_capacity(10000);

    let mut errors: usize = 0;
    let skins_count: usize = 0;

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

                // TODO: add local imagegui.png

                // TODO: read skins, increase skin counter
                let skins = None;
                // TODO: check if skins cover active submaterials from the main model

                // TODO: read actions
                let actions = None;

                // TODO: check if actions are applicable (obj deletion?)

                Ok(BuildingSource { source_dir: path.clone(), def, skins, actions })
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
                                if filetype.is_dir() && !dir_entry.file_name().to_string_lossy().starts_with('_') {
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


pub const MODPACK_LOG: &'static str = "modpack.log";
pub const MATERIAL_MTL: &'static str = "material.mtl";
pub const MATERIAL_E_MTL: &'static str = "material_e.mtl";

pub fn install(sources: Vec<BuildingSource>, target: &Path, log_file: &mut BufWriter<fs::File>, stock_map: &mut StockBuildingsMap) {
    
    let mut pathbuf = target.to_path_buf();

    let dirs_iter = (AppSettings::MOD_IDS_START .. AppSettings::MOD_IDS_END).map(|x| std::iter::repeat(x).zip(1 .. 100)).flatten();

    let mod_iter = dirs_iter.zip(sources.iter());
    for ((mod_id, subdir_id), src) in mod_iter {
        let mod_id = mod_id.to_string();
        let subdir_id = format!("{:0>2}", subdir_id);
        write!(log_file, "{}/{} {}\r\n", mod_id, subdir_id, src.source_dir.display()).unwrap();

        pathbuf.push(mod_id);
        pathbuf.push(subdir_id);
        fs::create_dir_all(&pathbuf).unwrap();

        let _def = copy_source(&src.def, &pathbuf, stock_map).unwrap();

        pathbuf.pop();
        pathbuf.pop();
    }
}

fn copy_source(src: &SourceType, destination: &Path, stock_map: &mut StockBuildingsMap) -> Result<ModBuildingDef, std::io::Error> {
    use crate::ini;
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
    let mut str_buf = String::with_capacity(16 * 1024);
    let mut byte_buf = Vec::<u8>::with_capacity(0);

    macro_rules! copy_fld {
        ($src_fld:expr, $dest_fld:expr, $dest_name:expr) => {
            $dest_fld.push(destination);
            $dest_fld.push($dest_name);
            fs::copy(&$src_fld, &$dest_fld)?;
        };
    }

    macro_rules! copy_asset_md5 {
        ($path:expr, $assets_root:ident, $asset_type:expr) => {{
            read_to_buf(&$path, &mut byte_buf)?;
            let asset_md5name = format!("{:x}.{}", md5::compute(byte_buf.as_mut_slice()), $asset_type);
            $path.push(&$assets_root);
            $path.push(&asset_md5name);
            if !$path.exists() {
                fs::write(&$path, byte_buf.as_slice())?;
            }

            format!("../../{}/{}", $asset_type, asset_md5name)
        }};
    }

    macro_rules! copy_fld_opt {
        ($fld:ident, $dest_name:expr) => {
            if let (Some(src_fld), Some(dest_fld)) = (src_data.$fld.as_ref(), data.$fld.as_mut()) {
                copy_fld!(src_fld, dest_fld, $dest_name);
            }
        };
        ($fld:ident) => {
            if let Some(dest_path) = data.$fld.as_mut() {
                Some(copy_asset_md5!(dest_path, nmf_root, "nmf"))
            } else { None }
        };
    }

    copy_fld!(src_data.building_ini, data.building_ini, BUILDING_INI);
    copy_fld!(src_data.material,     data.material,     MATERIAL_MTL);
    copy_fld_opt!(material_e, MATERIAL_E_MTL); 
    copy_fld_opt!(image_gui, "imagegui.png"); 

    let model_token:      String         = copy_asset_md5!(data.model, nmf_root, "nmf");
    let model_lod_token:  Option<String> = copy_fld_opt!(model_lod);
    let model_lod2_token: Option<String> = copy_fld_opt!(model_lod2);
    let model_e_token:    Option<String> = copy_fld_opt!(model_e);

    {
        // Update renderconfig.ini
        use ini::{renderconfig, parse_renderconfig_ini};

        read_to_string_buf(&new_render_path, &mut str_buf)?;
        let mut render_ini = parse_renderconfig_ini(&mut str_buf).expect("Invalid building renderconfig");
        for token_state in render_ini.tokens_mut() {
            token_state.modify(|t| {
                use renderconfig::Token as RT;
                use ini::common::IdStringParam;
                
                match t {
                    RT::Material(_)         => Some(RT::Material(IdStringParam::new_owned(MATERIAL_MTL))),
                    RT::MaterialEmissive(_) => Some(RT::MaterialEmissive(IdStringParam::new_owned(MATERIAL_E_MTL))),
                    RT::Model(_)            => Some(RT::Model(IdStringParam::new_owned(&model_token))),
                    RT::ModelLod((_, z))    => model_lod_token.as_ref().map(|t| RT::ModelLod((IdStringParam::new_owned(t), *z))),
                    RT::ModelLod2((_, z))   => model_lod2_token.as_ref().map(|t| RT::ModelLod2((IdStringParam::new_owned(t), *z))),
                    RT::ModelEmissive(_)    => model_e_token.as_ref().map(|t| RT::ModelEmissive(IdStringParam::new_owned(t))),
                    _ => None
                }
            });
        }

        let mut new_render_file = BufWriter::new(fs::OpenOptions::new().write(true).create(false).truncate(true).open(&new_render_path)?);
        render_ini.write_to(&mut new_render_file)?;
        new_render_file.flush()?;
    }

    {
        // TODO: copy textures and update materials

        read_to_string_buf(&data.material, &mut str_buf)?;
        let mut mtl = ini::parse_mtl(&mut str_buf).expect("Invalid material.mtl");
        for token_state in mtl.tokens_mut() {
            token_state.modify(|t| {
                use ini::material::Token as MT;
                use ini::common::IdStringParam;
                
                match t {
                    //MT::Texture((i, p))         => todo!(),
                    //MT::TextureNoMip((i, p))    => todo!(),
                    //MT::TextureMtl((i, p))      => todo!(),
                    //MT::TextureNoMipMtl((i, p)) => todo!(),
                    _ => None
                }
            });
        }
    }

    Ok(ModBuildingDef {
        render: new_render_path,
        data
    })
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



fn resolve_source_path(root: &BasePath, tail: &str) -> BasePathBuf {
    let mut iter = tail.chars();
    let pfx = iter.next().expect("resolve_source_path called with empty tail");
    match pfx {
        '#' => APP_SETTINGS.path_workshop.join(iter.as_str()),
        '~' => APP_SETTINGS.path_stock.join(iter.as_str()),
        _   => root.join(tail)
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

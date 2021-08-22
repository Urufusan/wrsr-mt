use std::fs;
use std::path::{Path, PathBuf};
use std::fmt;

use const_format::concatcp;
use regex::Regex;
use normpath::{BasePath, BasePathBuf, PathExt};
use lazy_static::lazy_static;

use crate::building_def::{ModBuildingDef, StockBuildingDef, BuildingError as DefError, StockBuildingsMap, fetch_stock_with_ini};
use crate::cfg::{APP_SETTINGS, RENDERCONFIG_INI, BUILDING_INI};
use crate::read_to_string_buf;


pub enum SourceType {
    Mod(ModBuildingDef),
    Stock(StockBuildingDef),
}

pub struct BuildingSource {
    def: SourceType,
    skins: Option<()>,
    actions: Option<()>,
}

#[derive(Debug)]
pub enum SourceError{
    NoRenderconfig,
    MultiRenderconfig,
    Def(DefError),
    Validation(String),
    RefRead(std::io::Error),
    RefParse,
}

const RENDERCONFIG_SOURCE: &str = "renderconfig.source";
const RENDERCONFIG_REF: &str = "renderconfig.ref";

pub fn read_validate_sources(source_dir: &Path, mut stock: StockBuildingsMap) -> Result<Vec::<BuildingSource>, usize> {
    let mut result = Vec::<BuildingSource>::with_capacity(10000);

    let mut errors: usize = 0;

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
                (None, Some(render_ref)) => get_source_type_from_ref(bld_ini, render_ref, &mut stock, &mut str_buf),
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

                // TODO: read skins
                let skins = None;
                // TODO: check if skins cover active submaterials from the main model

                // TODO: read actions
                let actions = None;

                // TODO: check if actions are applicable (obj deletion?)

                Ok(BuildingSource { def, skins, actions })
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
                        match dir_entry {
                            Ok(dir_entry) => match dir_entry.file_type() {
                                Ok(filetype) => if filetype.is_dir() {
                                    rev_buf.push(dir_entry.path());
                                },
                                Err(e) => log_err!(e)
                            },
                            Err(e) => log_err!(e)
                        }
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
        Ok(result)
    } else {
        Err(errors)
    }
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

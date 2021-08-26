use std::io::Error as IOErr;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{read_to_string_buf};
use crate::ini;
use crate::building_def;

use super::resolve_source_path;


pub const BUILDING_SKINS: &str = "building.skins";


#[derive(Debug)]
pub enum Error {
    SkinsFileRead(IOErr),
    SkinsFileParse(String),
    MtlRead(IOErr),
    MtlParse(PathBuf, Vec<String>),
    SkinValidation(Vec<String>),
    TexturePathInvalid(PathBuf),
}


pub type Skins = Vec<(PathBuf, Option<PathBuf>)>;


pub fn read_skins(path: &Path, buf: &mut String) -> Result<Skins, Error> {
    lazy_static! {
        static ref RX_SKIN: Regex = Regex::new(r"(?s)^([^\s]+)(\s+([^\s]+))?$").unwrap();
        static ref RX_LINES: Regex = Regex::new(r"(?s)(\s*\r?\n)+").unwrap();
    }

    buf.truncate(0);
    read_to_string_buf(path, buf).map_err(Error::SkinsFileRead)?;
    let mut result = Skins::with_capacity(16);

    for line in RX_LINES.split(&buf) {
        if !line.is_empty() {
            match RX_SKIN.captures(line) {
                Some(cap) => {
                    let root = path.parent().unwrap();
                    let mtl = resolve_source_path(root, cap.get(1).unwrap().as_str()).into_path_buf();
                    let mtl_e = cap.get(3).map(|x| resolve_source_path(root, x.as_str()).into_path_buf());
                    result.push((mtl, mtl_e));
                },
                None => return Err(Error::SkinsFileParse(line.to_string()))
            }
        }
    }

    Ok(result)
}


pub fn validate_skins(root: &Path, skins: &Skins, used_submaterials: &[&str], buf: &mut String) -> Result<(), Error> {
    let mut validation_errors = Vec::with_capacity(0);

    macro_rules! check_mtl {
        ($mtl_path:ident) => {
            buf.truncate(0);
            read_to_string_buf($mtl_path, buf).map_err(Error::MtlRead)?;
            let mtl = ini::parse_mtl(buf).map_err(|e| Error::MtlParse(
                $mtl_path.clone(), 
                e.into_iter().map(|(_, e)|  e).collect())
                )?;

            building_def::push_mtl_errors(&mtl, used_submaterials, &mut validation_errors, $mtl_path.display());

            for tx in mtl.get_texture_paths(|p| resolve_source_path(root, p).into_path_buf()) {
                if !tx.exists() {
                    return Err(Error::TexturePathInvalid(tx));
                }
            }
        }
    }

    for (mtl, mtl_e) in skins {
        check_mtl!(mtl);
        if let Some(mtl) = mtl_e {
            check_mtl!(mtl);
        }
    }

    if validation_errors.is_empty() {
        Ok(())
    } else {
        Err(Error::SkinValidation(validation_errors))
    }
}

use std::io::Error as IOErr;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{read_to_string_buf};
use crate::ini::{self, resolve_source_path};
use crate::building_def;



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
    use ini::common::IdStringParam;
    lazy_static! {
        static ref RX_SKIN: Regex = Regex::new(r"(?s)^([^\s]+)(\s+([^\s]+))?$").unwrap();
        static ref RX_LINES: Regex = Regex::new(r"(?s)(\s*\r?\n)+").unwrap();
    }

    buf.clear();
    read_to_string_buf(path, buf).map_err(Error::SkinsFileRead)?;
    let mut result = Skins::with_capacity(16);

    for line in RX_LINES.split(&buf) {
        if !line.is_empty() {
            match RX_SKIN.captures(line) {
                Some(cap) => {
                    let root = path.parent().unwrap();
                    let mtl = resolve_source_path(root, &IdStringParam::new_borrowed(cap.get(1).unwrap().as_str()));
                    let mtl_e = cap.get(3).map(|x| resolve_source_path(root, &IdStringParam::new_borrowed(x.as_str())));
                    result.push((mtl, mtl_e));
                },
                None => return Err(Error::SkinsFileParse(line.to_string()))
            }
        }
    }

    Ok(result)
}


pub fn validate(skins: &Skins, root: &Path, used_submaterials: &[&str], buf: &mut String) -> Result<(), Error> {
    let mut validation_errors = Vec::with_capacity(0);

    macro_rules! check_mtl {
        ($mtl_path:ident) => {
            buf.clear();
            read_to_string_buf($mtl_path, buf).map_err(Error::MtlRead)?;
            let mtl = ini::parse_mtl(buf).map_err(|e| Error::MtlParse(
                $mtl_path.clone(), 
                e.into_iter().map(|(_, e)|  e).collect())
                )?;

            building_def::push_mtl_errors(&mtl, used_submaterials.iter(), &mut validation_errors, $mtl_path.display());

            for tx in mtl.get_texture_paths(|p| resolve_source_path(root, p)) {
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

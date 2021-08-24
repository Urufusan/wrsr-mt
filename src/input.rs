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



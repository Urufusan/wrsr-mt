use std::path::{Path, PathBuf};
use std::fs;
use std::fmt::{Display, Formatter, Write};
use std::io::Error as IOErr;

use crate::ini::{self, BuildingIni, MaterialMtl};
use crate::ini::common::{IdStringParam, StrValue};
use crate::nmf;


#[derive(Debug)]
pub struct BuildingDef {
    building_ini: PathBuf,
    renderconfig: PathBuf,

    model: PathBuf,
    model_lod: Option<PathBuf>,
    model_lod2: Option<PathBuf>,
    model_e: Option<PathBuf>,

    material: PathBuf,
    material_e: Option<PathBuf>,

    textures: Vec<PathBuf>,
}


#[derive(Debug)]
pub enum BuildingError {
    FileIO(IOErr),
    Parse(PathBuf, String),
    Other(String),
    ModelMissing,
    MaterialMissing,
}


use crate::ini::RenderToken as RT;
use crate::ini::MaterialToken as MT;

macro_rules! get_ini_value {
    ($ini:ident, $p:pat, $s:ident, $root:ident) => {{
        let mut res = None;
        for t in $ini.tokens() {
            match t {
                $p => {
                    res = Some($root.join($s.as_str()));
                    break;
                }, 
                _ => ()
            }
        }

        res
    }};
}


impl BuildingDef {
    pub fn from_config(building_ini: &Path, renderconfig: &Path) -> Result<Self, BuildingError> {
        let render_root = renderconfig.parent().ok_or_else(|| BuildingError::Other(format!("Cannot get render root from {:?}", renderconfig)))?;

        let render_buf = fs::read_to_string(renderconfig).map_err(BuildingError::FileIO)?;
        let render_ini = ini::parse_renderconfig_ini(&render_buf).map_err(|e| BuildingError::Parse(renderconfig.to_path_buf(), concat_parse_errors(e)))?;

        let model      = get_ini_value!(render_ini, RT::Model(s), s, render_root).ok_or(BuildingError::ModelMissing)?;
        let model_lod  = get_ini_value!(render_ini, RT::ModelLod((s, _)),  s, render_root);
        let model_lod2 = get_ini_value!(render_ini, RT::ModelLod2((s, _)), s, render_root);
        let model_e    = get_ini_value!(render_ini, RT::ModelEmissive(s),  s, render_root);

        let material   = get_ini_value!(render_ini, RT::Material(s), s, render_root).ok_or(BuildingError::MaterialMissing)?;
        let material_e = get_ini_value!(render_ini, RT::MaterialEmissive(s), s, render_root);

        let mut textures = Vec::with_capacity(10);

        push_textures(&material, &mut textures)?;
        if let Some(ref material_e) = material_e {
            push_textures(material_e, &mut textures)?;
        }

        Ok(BuildingDef {
            building_ini: building_ini.to_path_buf(),
            renderconfig: renderconfig.to_path_buf(),
            model,
            model_lod,
            model_lod2,
            model_e,
            material,
            material_e,
            textures
        })
    }


    pub fn parse_and_validate(&self) -> Result<(), String> {
        let mut errors = String::with_capacity(0);

        fn check_path(name: &'static str, path: &PathBuf, errors: &mut String) {
            if !path.exists() { 
                writeln!(errors, "{} does not exist: {:?}", name, path).unwrap() 
            }
        }

        fn check_path_opt(name: &'static str, path: &Option<PathBuf>, errors: &mut String) {
            path.as_ref().map(|path| check_path(name, path, errors));
        }

        check_path("building.ini",     &self.building_ini, &mut errors);
        check_path("renderconfig.ini", &self.renderconfig, &mut errors);
        check_path("model",            &self.model,        &mut errors);
        check_path_opt("model_lod",    &self.model_lod,    &mut errors);
        check_path_opt("model_lod_2",  &self.model_lod2,   &mut errors);
        check_path_opt("model_e",      &self.model_e,      &mut errors);
        check_path("material",         &self.material,     &mut errors);
        check_path_opt("material_e",   &self.material_e,   &mut errors);
        for tx in self.textures.iter() {
            check_path("texture", tx, &mut errors);
        }

        let model = match nmf::NmfInfo::from_path(&self.model) {
            Ok(model) => Some(model),
            Err(e) => { 
                writeln!(errors, "Cannot load model nmf: {:?}", e).unwrap();
                None
            }
        };

        let mut str_buf = fs::read_to_string(&self.building_ini);
        match str_buf {
            Ok(ref building_ini_buf) => match ini::parse_building_ini(building_ini_buf) {
                Ok(building_ini) => {
                    // TODO: pure building_ini checks
                    if let Some(model) = &model {
                        check_model_buildingini_errors(&model, &building_ini, &mut errors)
                    }
                },
                Err(e) => writeln!(errors, "Cannot parse building.ini: {:#?}", e).unwrap()
            },
            Err(e) => writeln!(errors, "Cannot load building.ini: {:?}", e).unwrap()
        };

        str_buf = fs::read_to_string(&self.material);
        match str_buf {
            Ok(ref mtl_buf) => match ini::parse_mtl(&mtl_buf) {
                Ok(mtl) => {
                    // TODO: pure material checks?? Secondary models sumbaterials??
                    if let Some(model) = &model {
                        check_model_mtl_errors(&model, &mtl, &mut errors)
                    }
                },
                Err(e) => writeln!(errors, "Cannot parse material: {:#?}", e).unwrap()
            },
            Err(e) => writeln!(errors, "Cannot load material: {:?}", e).unwrap()
        };

        if let Some(material_e) = &self.material_e {
            str_buf = fs::read_to_string(material_e);
            match str_buf {
                Ok(ref mtl_buf) => match ini::parse_mtl(&mtl_buf) {
                    Ok(mtl) => {
                        // TODO: pure material checks?? Secondary models sumbaterials??
                        if let Some(model) = &model {
                            check_model_mtl_errors(&model, &mtl, &mut errors)
                        }
                    },
                    Err(e) => writeln!(errors, "Cannot parse material_e: {:#?}", e).unwrap()
                },
                Err(e) => writeln!(errors, "Cannot load material_e: {:?}", e).unwrap()
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

}

fn check_model_buildingini_errors(model: &nmf::NmfInfo, building_ini: &ini::BuildingIni, errors: &mut String) {
    //TODO
    todo!()
}

fn check_model_mtl_errors(model: &nmf::NmfInfo, mtl: &ini::MaterialMtl, errors: &mut String) {
    //TODO
    todo!()
}


fn concat_parse_errors(errors: Vec<(&str, String)>) -> String {
    let mut result = String::with_capacity(4 * 1024);
    for (chunk, err) in errors.iter() {
        write!(result, "Error: {}\nChunk: [{}]\n", err, chunk).unwrap();
    }

    result
}


fn push_textures (mtl_path: &Path, textures: &mut Vec<PathBuf>) -> Result<(), BuildingError> {
    use crate::cfg::APP_SETTINGS;

    let mtl_root = mtl_path.parent().ok_or_else(|| BuildingError::Other(format!("Cannot get mtl root from {:?}", mtl_path)))?;
    let mtl_buf = fs::read_to_string(mtl_path).map_err(BuildingError::FileIO)?;
    let mtl = ini::parse_mtl(&mtl_buf).map_err(|e| BuildingError::Parse(mtl_path.to_path_buf(), concat_parse_errors(e)))?;
    for t in mtl.tokens() {
        let tx_path = match t {
            MT::Texture((_, s))         => Some(APP_SETTINGS.path_stock.join(s.as_str())),
            MT::TextureNoMip((_, s))    => Some(APP_SETTINGS.path_stock.join(s.as_str())),
            MT::TextureMtl((_, s))      => Some(mtl_root.join(s.as_str())),
            MT::TextureNoMipMtl((_, s)) => Some(mtl_root.join(s.as_str())),
            _ => None
        };

        if let Some(tx_path) = tx_path {
            if textures.iter().all(|x| *x != tx_path) {
                textures.push(tx_path);
            }
        }
    }

    Ok(())
}


macro_rules! w_opt {
    ($f:ident, $fstr:expr, $v:expr) => {
        if let Some(ref v) = $v {
            write!($f, $fstr, v)
        } else {    
            write!($f, $fstr, "<NONE>")
        }
    };
}

impl Display for BuildingDef {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Building {{\n")?;
        write!(f, "    building.ini:     {:?}\n", self.building_ini)?;
        write!(f, "    renderconfig.ini: {:?}\n", self.renderconfig)?;
        write!(f, "    model:            {:?}\n", self.model)?;
        w_opt!(f, "    model_lod:        {:?}\n", self.model_lod)?;
        w_opt!(f, "    model_lod2:       {:?}\n", self.model_lod2)?;
        w_opt!(f, "    model_e:          {:?}\n", self.model_e)?;
        write!(f, "    material:         {:?}\n", self.material)?;
        w_opt!(f, "    material_e:       {:?}\n", self.material_e)?;

        write!(f, "    textures: [\n")?;
        for tx in self.textures.iter() {
            write!(f, "        {:?}\n", tx)?;
        }

        write!(f, "    ]\n}}\n")
    }
}

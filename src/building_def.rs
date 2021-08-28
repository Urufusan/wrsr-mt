use std::path::{Path, PathBuf};
use std::fs;
use std::fmt::{Display, Formatter, Write};
use std::io::Error as IOErr;

use crate::{read_to_string_buf};
use crate::nmf::NmfInfo;
use crate::ini::{self,
                 BuildingIni,
                 RenderIni,
                 MaterialMtl,
                 RenderToken as RT,
                 MaterialToken as MT,
                 common::IdStringParam,
                 };



#[derive(Debug, Clone)]
pub struct ModBuildingDef {
    pub render: PathBuf,
    pub building_ini: PathBuf,
    pub image_gui: Option<PathBuf>,

    pub model: PathBuf,
    pub model_lod: Option<PathBuf>,
    pub model_lod2: Option<PathBuf>,
    pub model_e: Option<PathBuf>,

    pub material: PathBuf,
    pub material_e: Option<PathBuf>,

    pub textures: Vec<PathBuf>,
}


#[derive(Debug, Clone)]
pub enum BuildingError {
    FileIO(PathBuf, String),
    Parse(PathBuf, String),
    ModelMissing,
    MaterialMissing,
    Validation(Vec<String>),
}



impl ModBuildingDef {
    fn from_render_ini(
        building_ini: &Path, 
        render: &Path,
        render_root: &Path, 
        render_ini: RenderIni, 
        render_path_resolver: fn(&Path, &IdStringParam) -> PathBuf,
        mtl_path_resolver:    fn(&Path, &IdStringParam) -> PathBuf) -> Result<Self, BuildingError> 
    {
        macro_rules! get_render_value {
            ($p:pat, $s:ident) => {{
                let mut res = None;
                for t in render_ini.tokens() {
                    if let $p = t {
                        res = Some(render_path_resolver(render_root, $s));
                        break;
                    }
                }

                res
            }};
        }

        let model      = get_render_value!(RT::Model(s),            s).ok_or(BuildingError::ModelMissing)?;
        let model_lod  = get_render_value!(RT::ModelLod((s, _)),    s);
        let model_lod2 = get_render_value!(RT::ModelLod2((s, _)),   s);
        let model_e    = get_render_value!(RT::ModelEmissive(s),    s);
        let material   = get_render_value!(RT::Material(s),         s).ok_or(BuildingError::MaterialMissing)?;
        let material_e = get_render_value!(RT::MaterialEmissive(s), s);

        let mut textures = Vec::with_capacity(10);
        push_textures(&material, &mut textures, mtl_path_resolver)?;
        if let Some(ref material_e) = material_e {
            push_textures(material_e, &mut textures, mtl_path_resolver)?;
        }

        Ok(ModBuildingDef {
            render: render.to_path_buf(),
            building_ini: building_ini.to_path_buf(),
            image_gui: None,
            model,
            model_lod,
            model_lod2,
            model_e,
            material,
            material_e,
            textures
        })
    }


    // Does not re-parse renderconfig!
    pub fn parse_and_validate(&self, nmf_override: Option<&NmfInfo>) -> Result<(), BuildingError> {
        let mut errors = Vec::<String>::with_capacity(0);

        macro_rules! check_path {
            ($name:expr, $path:expr) => { 
                if !$path.exists() {
                    errors.push(format!("{} ({}) does not exist", $name, $path.display())); 
                }
            };
        }

        macro_rules! check_popt {
            ($name:expr, $path:expr) => { 
                if let Some(path) = $path {
                    check_path!($name, path);
                }
            };
        }

        check_path!("renderconfig.ini", &self.render);
        check_path!("building.ini",     &self.building_ini);
        check_popt!("imagegui",         &self.image_gui);
        check_path!("MODEL",            &self.model);
        check_popt!("MODEL_LOD",        &self.model_lod);
        check_popt!("MODEL_LOD2",       &self.model_lod2);
        check_popt!("MODELEMISSIVE",    &self.model_e);
        check_path!("MATERIAL",         &self.material);
        check_popt!("MATERIALEMISSIVE", &self.material_e);
        for tx in self.textures.iter() {
            check_path!("texture", tx);
        }

        match NmfInfo::from_path(&self.model) {
            Ok(model) => {
                let model = nmf_override.unwrap_or(&model);
                let mut str_buf = String::with_capacity(0);
                macro_rules! push_errors {
                    ($ini_path:expr, $parser:expr, $model_data:expr, $pusher:ident, $pfx:expr) => {
                        let read_res = read_to_string_buf($ini_path, &mut str_buf);
                        match read_res {
                            Ok(()) => match $parser(&str_buf) {
                                Ok(ini) => {
                                    $pusher(&ini, $model_data, &mut errors, $pfx)
                                },
                                Err(e) => errors.push(format!("Cannot parse file {}: {:#?}", $ini_path.display(), e))
                            },
                            Err(e) => errors.push(format!("Cannot read file {}: {:#?}", $ini_path.display(), e))
                        };
                    };
                }

                push_errors!(&self.building_ini, ini::parse_building_ini, &model,        push_buildingini_errors, "building.ini");

                let sm_usage = model.get_used_sumbaterials().collect::<Vec<_>>();
                push_errors!(&self.material,     ini::parse_mtl,          sm_usage.iter(), push_mtl_errors,         "primary material");
                if let Some(material_e) = &self.material_e {
                    push_errors!(&material_e,    ini::parse_mtl,          sm_usage.iter(), push_mtl_errors,         "emissive material");
                }
            },
            Err(e) => { 
                errors.push(format!("Cannot load model nmf: {:?}", e));
            }
        };


        if errors.is_empty() {
            Ok(())
        } else {
            Err(BuildingError::Validation(errors))
        }
    }


    pub fn from_render_path(building_ini: &Path, renderconfig: &Path, path_resolver: fn(&Path, &IdStringParam) -> PathBuf, validate: bool) -> Result<Self, BuildingError> {
        let render_root = renderconfig.parent().expect(&format!("Cannot get render root from {}", renderconfig.display()));

        let render_buf = fs::read_to_string(renderconfig).map_err(|e| BuildingError::FileIO(renderconfig.to_path_buf(), e.to_string()))?;
        let render_ini = ini::parse_renderconfig_ini(&render_buf).map_err(|e| BuildingError::Parse(renderconfig.to_path_buf(), concat_parse_errors(e)))?;

        let mut result = Self::from_render_ini(building_ini, renderconfig, render_root, render_ini, path_resolver, path_resolver)?;

        result.image_gui = {
            let img_path = render_root.join("imagegui.png");
            if img_path.exists() {
                Some(img_path)
            } else {
                None
            }
        };

        if validate {
            result.parse_and_validate(None)?;
        }

        Ok(result)
    }


    pub fn shallow_copy_to(&self, target_dir: &Path) -> Result<Self, IOErr> {
        let source_root = self.render.parent().unwrap();
        
        let mk_fld = |fld: &PathBuf| -> Result<PathBuf, IOErr> {
            if fld.starts_with(source_root) {
                let new_fld = target_dir.join(fld.strip_prefix(source_root).unwrap());
                fs::create_dir_all(new_fld.parent().unwrap())?;
                fs::copy(fld, &new_fld)?;

                Ok(new_fld)
            } else {
                Ok(fld.clone())
            }
        };

        macro_rules! mk_fld_opt {
            ($fld:expr) => { $fld.as_ref().map(|x| mk_fld(x)).transpose() };
        }

        let render       = mk_fld(&self.render)?;
        let building_ini = mk_fld(&self.building_ini)?;
        let image_gui    = mk_fld_opt!(self.image_gui)?;
        let model        = mk_fld(&self.model)?;
        let model_lod    = mk_fld_opt!(self.model_lod)?;
        let model_lod2   = mk_fld_opt!(self.model_lod2)?;
        let model_e      = mk_fld_opt!(self.model_e)?;
        let material     = mk_fld(&self.material)?;
        let material_e   = mk_fld_opt!(&self.material_e)?;

        let mut textures = Vec::with_capacity(self.textures.len());
        for tx in self.textures.iter() {
            let tx = mk_fld(tx)?;
            textures.push(tx);
        }

        Ok(ModBuildingDef {
            render,
            building_ini,
            image_gui,
            model,
            model_lod,
            model_lod2,
            model_e,
            material,
            material_e,
            textures
        })
    }
}


pub fn validate_building_ini_refs<'a, REFS, N>(ini_refs: REFS, object_names: &[N]) -> Result<(), Vec<String>>
where REFS: Iterator<Item = ini::BuildingNodeRef<'a>>,
      N: AsRef<str>,
{
    let mut errors = Vec::<String>::with_capacity(0);
    for r in ini_refs {
        match r {
            ini::BuildingNodeRef::Exact(node) => if object_names.iter().all(|obj| obj.as_ref() != node) {
                errors.push(format!("building.ini contains invalid reference to node '{}'. No object in the NMF has such name", node));
            },
            ini::BuildingNodeRef::Keyword(key) => if object_names.iter().all(|obj| !obj.as_ref().starts_with(key)) {
                errors.push(format!("building.ini contains invalid node-keyword '{}'. No object in the NMF starts with that key", key));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}


fn push_buildingini_errors(building_ini: &BuildingIni, model: &NmfInfo, errors: &mut Vec<String>, _pfx: &str) {
    let obj_names: Vec<_> = model.object_names().collect();
    if let Err(mut e) = validate_building_ini_refs(building_ini.get_model_refs(), &obj_names[..]) {
        errors.append(&mut e);
    }

    // TODO: add other building.ini checks
}

pub fn validate_mtl_refs<REF, SM, SMS>(mtl_refs: &[REF], used_submaterials: SMS) -> Result<(), Vec<String>>
where REF:  AsRef<str>,
      SM:   AsRef<str>,
      SMS:  Iterator<Item = SM>
{      
    let mut errors = Vec::<String>::with_capacity(0);

    for sm in used_submaterials {
        let sm = sm.as_ref();
        if mtl_refs.iter().all(|r| sm != r.as_ref()) {
            errors.push(format!("NMF uses submaterial '{}', but mtl file has no corresponding token", sm));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn push_mtl_errors<P: Display, SM, SMS>(mtl: &MaterialMtl, used_submaterials: SMS, errors: &mut Vec<String>, pfx: P)
where SM:  AsRef<str>,
      SMS: Iterator<Item = SM>
{
    // For now there is only 1 hard rule:
    // "all submaterials that are used by objects in NMF must have a token in mtl file"
    // other checks could be added later


    let mtl_tokens = mtl.tokens().filter_map(|t| match t {
        MT::Submaterial(mtl_sm) => Some(mtl_sm),
        _ => None
    }).collect::<Vec<_>>();

    if let Err(mut e) = validate_mtl_refs(&mtl_tokens[..], used_submaterials) {
        errors.push(format!("Errors in {}", pfx));
        errors.append(&mut e);
    }
}


fn concat_parse_errors(errors: Vec<(&str, String)>) -> String {
    let mut result = String::with_capacity(4 * 1024);
    for (chunk, err) in errors.iter() {
        writeln!(result, "Error: {}\nChunk: [{}]", err, chunk).unwrap();
    }

    result
}


fn push_textures<F>(mtl_path: &Path, textures: &mut Vec<PathBuf>, mtl_path_resolver: F) -> Result<(), BuildingError>
where F: Fn(&Path, &IdStringParam) -> PathBuf 
{
    let mtl_root = mtl_path.parent().expect(&format!("Cannot get mtl root from {}", mtl_path.display()));
    let mtl_buf = fs::read_to_string(mtl_path).map_err(|e| BuildingError::FileIO(mtl_path.to_path_buf(), e.to_string()))?;
    let mtl = ini::parse_mtl(&mtl_buf).map_err(|e| BuildingError::Parse(mtl_path.to_path_buf(), concat_parse_errors(e)))?;
    for tx_path in mtl.get_texture_paths(|p| mtl_path_resolver(mtl_root, p)) {
        if textures.iter().all(|x| *x != tx_path) {
            textures.push(tx_path);
        }
    }

    Ok(())
}



//-----------------------------------------------------------------------

macro_rules! w_optln {
    ($f:ident, $fstr:expr, $v:expr) => {
        if let Some(ref v) = $v {
            writeln!($f, $fstr, v.display())
        } else {    
            writeln!($f, $fstr, "<NONE>")
        }
    };
}


impl Display for ModBuildingDef {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "building {{")?;
        writeln!(f, "  render:           {}", self.render.display())?;
        writeln!(f, "  building.ini:     {}", self.building_ini.display())?;
        w_optln!(f, "  imagegui.png:     {}", self.image_gui)?;
        writeln!(f, "  model:            {}", self.model.display())?;
        w_optln!(f, "  model_lod:        {}", self.model_lod)?;
        w_optln!(f, "  model_lod2:       {}", self.model_lod2)?;
        w_optln!(f, "  model_e:          {}", self.model_e)?;
        writeln!(f, "  material:         {}", self.material.display())?;
        w_optln!(f, "  material_e:       {}", self.material_e)?;
        writeln!(f, "  textures: [")?;
        for tx in self.textures.iter() {
            writeln!(f, "    {}", tx.display())?;
        }
        writeln!(f, "  ]\n}}")
    }
}


impl Display for BuildingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            BuildingError::FileIO(path, e)    => write!(f, "File error ({}): {}", path.display(), e),
            BuildingError::Parse(path, e)     => write!(f, "Parse error ({}): {}", path.display(), e),
            BuildingError::ModelMissing       => write!(f, "Model is missing"),
            BuildingError::MaterialMissing    => write!(f, "Material is missing"),
            BuildingError::Validation(e)      => write!(f, "Validation failed: {:#?}", e),
        }
    }
}

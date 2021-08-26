use std::path::{Path, PathBuf};
use std::fs;
use std::fmt::{Display, Formatter, Write};
use std::io::Error as IOErr;
use std::collections::HashMap;

use crate::{read_to_string_buf, normalize_join};
use crate::cfg::APP_SETTINGS;
use crate::nmf::NmfInfo;
use crate::ini::{self,
                 BuildingIni,
                 RenderIni,
                 MaterialMtl,
                 BuildingToken as BT,
                 RenderToken as RT,
                 MaterialToken as MT,
                 };


use normpath::{BasePathBuf};


#[derive(Debug, Clone)]
pub struct BuildingDef<T> {
    pub render: T,
    pub data: DefData,
}

#[derive(Debug, Clone)]
pub struct DefData {
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


pub type StockBuildingDef = BuildingDef<String>;
pub type ModBuildingDef   = BuildingDef<PathBuf>;


#[derive(Debug, Clone)]
pub enum BuildingError {
    FileIO(PathBuf, String),
    Parse(PathBuf, String),
    ParseStock(String, String),
    ModelMissing,
    MaterialMissing,
    UnknownStockKey(String),
    Validation(Vec<String>),
}


pub type StockBuildingsMap<'stock> = HashMap<&'stock str, (&'stock str, StockBuilding<'stock>)>;

pub enum StockBuilding<'stock> {
    Unparsed(&'stock str),
    Parsed(&'stock str, StockBuildingDef),
    Invalid(&'stock str, BuildingError),

}

impl StockBuilding<'_> {
    pub fn parse_map<'stock>(stock_buildings_ini: &'stock str) -> StockBuildingsMap<'stock> {
        let mut mp = HashMap::with_capacity(512);
        let rx = regex::Regex::new(r"\$TYPE ([_[:alnum:]]+?)\r\n((?s).+?\n END\r\n)").expect("Stock buildings: cannot create parsing regex");

        for caps in rx.captures_iter(stock_buildings_ini) {
            let key = caps.get(1).unwrap().as_str();
            let raw_value = caps.get(2).unwrap().as_str();
            mp.insert(
                key, 
                (key, StockBuilding::Unparsed(raw_value))
            );
        }
        
        mp
    }
}


impl<T> BuildingDef<T> {
    fn from_render_ini(
        building_ini: &Path, 
        render: T,
        render_root: &Path, 
        render_ini: RenderIni, 
        render_path_resolver: fn(&Path, &str) -> BasePathBuf,
        mtl_path_resolver:    fn(&Path, &str) -> BasePathBuf) -> Result<Self, BuildingError> 
    {
        macro_rules! get_render_value {
            ($p:pat, $s:ident) => {{
                let mut res = None;
                for t in render_ini.tokens() {
                    if let $p = t {
                        res = Some(render_path_resolver(render_root, $s.as_str()).into_path_buf());
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

        Ok(BuildingDef {
            render,
            data: DefData {
                building_ini: building_ini.to_path_buf(),
                image_gui: None,
                model,
                model_lod,
                model_lod2,
                model_e,
                material,
                material_e,
                textures
            }
        })
    }


    // Does not re-parse renderconfig!
    fn parse_and_validate(&self) -> Result<(), BuildingError> {
        let mut errors = Vec::<String>::with_capacity(0);

        macro_rules! check_path {
            ($name:expr, $path:expr) => { 
                if !$path.exists() {
                    errors.push(format!("{} does not exist: {}", $name, $path.display())); 
                }
            };
        }

        macro_rules! check_popt {
            ($name:expr, $path:expr) => { 
                $path.as_ref().map(|p| check_path!($name, p)); 
            };
        }

        check_path!("building.ini",     &self.data.building_ini);
        check_popt!("imagegui",         &self.data.image_gui);
        check_path!("MODEL",            &self.data.model);
        check_popt!("MODEL_LOD",        &self.data.model_lod);
        check_popt!("MODEL_LOD2",       &self.data.model_lod2);
        check_popt!("MODELEMISSIVE",    &self.data.model_e);
        check_path!("MATERIAL",         &self.data.material);
        check_popt!("MATERIALEMISSIVE", &self.data.material_e);
        for tx in self.data.textures.iter() {
            check_path!("texture", tx);
        }

        let model = match NmfInfo::from_path(&self.data.model) {
            Ok(model) => Some(model),
            Err(e) => { 
                errors.push(format!("Cannot load model nmf: {:?}", e));
                None
            }
        };

        let mut str_buf = String::with_capacity(0);

        if let Some(model) = &model {
            let sm_usage = model.get_used_sumbaterials();

            macro_rules! push_errors {
                ($path:expr, $parser:expr, $model_data:expr, $pusher:ident, $pfx:expr) => {
                    let read_res = read_to_string_buf(&$path, &mut str_buf);
                    match read_res {
                        Ok(()) => match $parser(&str_buf) {
                            Ok(ini) => {
                                $pusher(&ini, $model_data, &mut errors, $pfx)
                            },
                            Err(e) => errors.push(format!("Cannot parse file {}: {:#?}", $path.display(), e))
                        },
                        Err(e) => errors.push(format!("Cannot read file {}: {:?}", $path.display(), e))
                    };
                };
            }

            push_errors!(self.data.building_ini, ini::parse_building_ini, &model,        push_buildingini_errors, "building.ini");
            push_errors!(self.data.material,     ini::parse_mtl,          &sm_usage[..], push_mtl_errors,         "material");
            if let Some(material_e) = &self.data.material_e {
                push_errors!(material_e,         ini::parse_mtl,          &sm_usage[..], push_mtl_errors,         "emissive material");
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(BuildingError::Validation(errors))
        }
    }
}


impl StockBuildingDef {
    fn from_slice(key: &str, chunk: &str) -> Result<Self, BuildingError> {
        let render_ini = ini::parse_renderconfig_ini(chunk)
            .map_err(|e| BuildingError::ParseStock(key.to_string(), concat_parse_errors(e)))?;

        let bld_ini = APP_SETTINGS.path_stock.join(format!("buildings_types/{}.ini", key));

        let mut result = Self::from_render_ini(
            bld_ini.as_path(), 
            key.to_string(), 
            APP_SETTINGS.path_stock.as_path(), 
            render_ini, 
            |_, tail| APP_SETTINGS.path_stock.join(tail),
            normalize_join)?;

        result.data.image_gui = {
            let img_path = APP_SETTINGS.path_stock.join(format!("editor/tool_{}.png", key));
            if img_path.exists() {
                Some(img_path.into_path_buf())
            } else {
                None
            }
        };

        Ok(result)
    }
}


impl ModBuildingDef {
    pub fn from_render_path(building_ini: &Path, renderconfig: &Path, path_resolver: fn(&Path, &str) -> BasePathBuf, validate: bool) -> Result<Self, BuildingError> {
        let render_root = renderconfig.parent().expect(&format!("Cannot get render root from {}", renderconfig.display()));

        let render_buf = fs::read_to_string(renderconfig).map_err(|e| BuildingError::FileIO(renderconfig.to_path_buf(), e.to_string()))?;
        let render_ini = ini::parse_renderconfig_ini(&render_buf).map_err(|e| BuildingError::Parse(renderconfig.to_path_buf(), concat_parse_errors(e)))?;

        let mut result = Self::from_render_ini(building_ini, renderconfig.to_path_buf(), render_root, render_ini, path_resolver, path_resolver)?;

        result.data.image_gui = {
            let img_path = render_root.join("imagegui.png");
            if img_path.exists() {
                Some(img_path)
            } else {
                None
            }
        };

        if validate {
            result.parse_and_validate()?;
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
        let building_ini = mk_fld(&self.data.building_ini)?;
        let image_gui    = mk_fld_opt!(self.data.image_gui)?;
        let model        = mk_fld(&self.data.model)?;
        let model_lod    = mk_fld_opt!(self.data.model_lod)?;
        let model_lod2   = mk_fld_opt!(self.data.model_lod2)?;
        let model_e      = mk_fld_opt!(self.data.model_e)?;
        let material     = mk_fld(&self.data.material)?;
        let material_e   = mk_fld_opt!(&self.data.material_e)?;

        let mut textures = Vec::with_capacity(self.data.textures.len());
        for tx in self.data.textures.iter() {
            let tx = mk_fld(tx)?;
            textures.push(tx);
        }

        Ok(ModBuildingDef {
            render,
            data: DefData {
                building_ini,
                image_gui,
                model,
                model_lod,
                model_lod2,
                model_e,
                material,
                material_e,
                textures
            }
        })
    }
}


fn push_buildingini_errors(building_ini: &BuildingIni, model: &NmfInfo, errors: &mut Vec<String>, pfx: &'static str) {
    for t in building_ini.tokens() {
        macro_rules! check_model_node {
            ($node:ident, $cmp:ident) => {
                if model.objects.iter().all(|o| !o.name.as_str().$cmp($node.as_str())) {
                    errors.push(format!("Error in {}: invalid token '{}', matching node was not found in the model nmf", pfx, t));
                }
            };
        }

        match t {
            BT::StorageLivingAuto(id)          => check_model_node!(id, eq),
            BT::CostWorkBuildingNode(id)       => check_model_node!(id, eq),
            BT::CostWorkVehicleStationNode(id) => check_model_node!(id, eq),
            BT::CostWorkBuildingKeyword(key)   => check_model_node!(key, starts_with),
            _ => {}
        }

        // TODO: other checks
    }
}

pub fn push_mtl_errors<P: Display>(mtl: &MaterialMtl, used_sumbaterials: &[&str], errors: &mut Vec<String>, pfx: P) {
    // For now there is only 1 hard rule:
    // "all submaterials that are used by objects in NMF must have a token in mtl file"
    // other checks could be added later

    'obj_iter: for obj_sm in used_sumbaterials {
        for t in mtl.tokens() {
            match t {
                MT::Submaterial(mtl_sm) => {
                    if *obj_sm == mtl_sm.as_str() {
                        continue 'obj_iter;
                    }
                },
                _ => {}
            }
        }

        errors.push(format!("Error in {}: NMF uses submaterial '{}', but the MTL file has no corresponding token", pfx, obj_sm));
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
where F: Fn(&Path, &str) -> BasePathBuf 
{
    let mtl_root = mtl_path.parent().expect(&format!("Cannot get mtl root from {}", mtl_path.display()));
    let mtl_buf = fs::read_to_string(mtl_path).map_err(|e| BuildingError::FileIO(mtl_path.to_path_buf(), e.to_string()))?;
    let mtl = ini::parse_mtl(&mtl_buf).map_err(|e| BuildingError::Parse(mtl_path.to_path_buf(), concat_parse_errors(e)))?;
    for tx_path in mtl.get_texture_paths(|p| mtl_path_resolver(mtl_root, p).into_path_buf()) {
        if textures.iter().all(|x| *x != tx_path) {
            textures.push(tx_path);
        }
    }

    Ok(())
}


pub fn fetch_stock_building<'a, 'ini, 'map>(key: &'a str, hmap: &'map mut StockBuildingsMap<'ini>) -> Result<(&'ini str, StockBuildingDef), BuildingError> {
    if let Some(mref) = hmap.get_mut(key) {
        match mref {
            (_, StockBuilding::Parsed(chunk, ref x)) => Ok((chunk, x.clone())),
            (key, StockBuilding::Unparsed(chunk)) => {
                match StockBuildingDef::from_slice(key, chunk)
                    .and_then(|def| def.parse_and_validate().map(|_| def)) {
                    Ok(def) => {
                        let res = (chunk.clone(), def.clone());
                        *mref = (key, StockBuilding::Parsed(chunk, def));
                        Ok(res)
                    },
                    Err(e) => {
                        *mref = (key, StockBuilding::Invalid(chunk, e.clone()));
                        Err(e)
                    }
                }
            },
            (_, StockBuilding::Invalid(_, e)) => Err(e.clone())
        }
    } else {
        Err(BuildingError::UnknownStockKey(key.to_string()))
    }
}


pub fn fetch_stock_with_ini<'a, 'ini, 'map>(
    key: &'a str, 
    hmap: &'map mut StockBuildingsMap<'ini>, 
    building_ini: PathBuf) -> Result<StockBuildingDef, BuildingError> {

    let (_, mut def) = fetch_stock_building(key, hmap)?;
    def.data.building_ini = building_ini;
    def.parse_and_validate()?;
    Ok(def)
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

fn write_building_def<'a, T: 'a, D: Display + 'a, F: Fn(&'a T) -> D>(f: &mut Formatter, def: &'a BuildingDef<T>, writer: F) -> Result<(), std::fmt::Error> {
    writeln!(f, "building {{")?;
    writeln!(f, "  render:           {}", writer(&def.render))?;
    writeln!(f, "  building.ini:     {}", def.data.building_ini.display())?;
    w_optln!(f, "  imagegui.png:     {}", def.data.image_gui)?;
    writeln!(f, "  model:            {}", def.data.model.display())?;
    w_optln!(f, "  model_lod:        {}", def.data.model_lod)?;
    w_optln!(f, "  model_lod2:       {}", def.data.model_lod2)?;
    w_optln!(f, "  model_e:          {}", def.data.model_e)?;
    writeln!(f, "  material:         {}", def.data.material.display())?;
    w_optln!(f, "  material_e:       {}", def.data.material_e)?;
    writeln!(f, "  textures: [")?;
    for tx in def.data.textures.iter() {
        writeln!(f, "    {}", tx.display())?;
    }
    writeln!(f, "  ]\n}}")
}

impl Display for StockBuildingDef {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write_building_def(f, self, |x| x)
    }
}

impl Display for ModBuildingDef {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write_building_def(f, self, |x| x.display())
    }
}


impl Display for BuildingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            BuildingError::FileIO(path, e)    => write!(f, "File error ({}): {}", path.display(), e),
            BuildingError::Parse(path, e)     => write!(f, "Parse error ({}): {}", path.display(), e),
            BuildingError::ParseStock(key, e) => write!(f, "Parse stock error ({}): {}", key, e),
            BuildingError::ModelMissing       => write!(f, "Model is missing"),
            BuildingError::MaterialMissing    => write!(f, "Material is missing"),
            BuildingError::UnknownStockKey(k) => write!(f, "Unknown stock building key '{}'", k),
            BuildingError::Validation(e)      => write!(f, "Validation failed: {:#?}", e),
        }
    }
}

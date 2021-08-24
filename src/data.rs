//use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ops::Range;
use std::convert::{TryFrom, TryInto};

use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

use crate::cfg::{AppSettings, APP_SETTINGS};

//--------------------------------------------
//                SOURCES

#[derive(Debug, Clone)]
pub struct BuildingDef<'stock> {
    pub render_config: RenderConfig<'stock>,

    pub building_ini: PathBuf,
    pub imagegui: Option<PathBuf>,

    pub model: ModelDef,
    pub model_lod1: Option<ModelDef>,
    pub model_lod2: Option<ModelDef>,
    pub model_emissive: Option<ModelDef>,

    pub material: MaterialDef,
    pub material_emissive: Option<MaterialDef>,

    pub skins: Vec<Skin>
}


#[derive(Debug, Clone)]
pub struct Skin {
   pub material: SkinMaterial,
   pub material_emissive: Option<SkinMaterial>
}

#[derive(Debug, Clone)]
pub struct SkinMaterial {
    pub path: PathBuf,
    pub textures: Vec<IniTokenTexture>
}




//--------------------------------------------------------
        /*
impl ModelPatch {
    pub fn apply<'data>(&self, src: &nmf::Nmf<'data>) -> nmf::Nmf<'data> {

        // TODO
        todo!()

        let mut sm_usage: Vec<Option<usize>> = vec![None; src.submaterials.len()];
        let mut set_used = |obj: &nmf::Object<'data>| for &idx in obj.submaterials.iter() {
            sm_usage[idx] = Some(idx);
        };

        // Removing objects
        let mut objects: Vec<_> = match self {
            ModelPatch::Keep(keeps) => keeps.iter().map(|k| {
                let obj = src.objects.iter()
                    .find(|o| o.name.as_str().unwrap() == k)
                    .expect(&format!("ModelPatch error: cannot find object to keep - '{}'", k));
                
                set_used(&obj);
                obj.clone()
            }).collect(),

            ModelPatch::Remove(rems) => {
                let mut rems: Vec<&str> = rems.iter().map(|r| r.as_str()).collect();
                let kept = src.objects.iter().filter_map(|o| {
                    if let Some((i, _)) = rems.iter().enumerate().find(|(_, &r)| r == o.name.as_str().unwrap()) {
                        rems.remove(i);
                        None
                    } else {
                        set_used(&o);
                        Some(o.clone())
                    }
                }).collect();

                if !rems.is_empty() {
                    panic!("ModelPatch error: could not delete some objects ({:?})", rems);
                }

                kept
            }
        };

        // Removing unused submaterials
        let mut offset = 0usize;
        for new_i in sm_usage.iter_mut() {
            if let Some(idx) = *new_i {
                *new_i = Some(idx - offset);
            } else {
                offset += 1;
            }
        }

        // NOTE: DEBUG
        // println!("sm usage: {:?}", &sm_usage);

        let submaterials = sm_usage.iter().enumerate().filter_map(|(i, opt)| 
            opt.map(|_| src.submaterials[i].clone())
        ).collect();

        // fixing objects' submaterial references
        for obj in objects.iter_mut() {
            for old_idx in obj.submaterials.iter_mut() {
                let new_idx = sm_usage[*old_idx].unwrap();
                *old_idx = new_idx;
            }
        }
        
        nmf::Nmf {
            header: src.header,
            submaterials,
            objects
        }
 
    }
}
*/




/*
fn read_to_buf(path: &Path, buf: &mut Vec<u8>) {
    use std::io::Read;

    if let Ok(mut file) = fs::File::open(path) {
        let meta = file.metadata().unwrap();
        let sz: usize = meta.len().try_into().unwrap();
        buf.reserve(sz);
        file.read(buf).unwrap();
    } else {
        panic!("Cannot read file \"{}\"", path.display());
    }
}
*/

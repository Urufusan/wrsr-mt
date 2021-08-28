use std::fs;
use std::io::{Write, BufWriter, Error as IOErr};
use std::path::{Path, PathBuf};
use std::fmt::{self, Write as FmtWrite};

//use const_format::concatcp;
use regex::Regex;
use normpath::{BasePathBuf, PathExt};
use lazy_static::lazy_static;

mod skins;
mod actions;

use crate::{read_to_buf, read_to_string_buf};
use crate::cfg::{AppSettings, APP_SETTINGS, RENDERCONFIG_INI, BUILDING_INI};
use crate::building_def::{ModBuildingDef, BuildingError as DefError};
use crate::nmf;
use crate::ini::{self, resolve_source_path, resolve_stock_path};
use crate::ini::common::IdStringParam;

use skins::{Skins, Error as SkinsError};
use actions::{ModActions, Error as ActionsError};



pub struct BuildingSource {
    source_dir: PathBuf,
    def: ModBuildingDef,
    skins: Skins,
    actions: Option<ModActions>,
}


pub enum SourceError {
    NoRenderconfig,
    MultiRenderconfig,
    Def(DefError),
    RefRead(IOErr),
    RefParse,
    Skins(SkinsError),
    Actions(ActionsError),
    Nmf(nmf::Error),
}

pub const MODPACK_LOG:     &str = "modpack.log";

const RENDERCONFIG_SOURCE: &str = "renderconfig.source";
const RENDERCONFIG_REF:    &str = "renderconfig.ref";
const BUILDING_SKINS:      &str = "building.skins";
const BUILDING_ACTIONS:    &str = "building.actions";

const MATERIAL_MTL:        &str = "material.mtl";
const MATERIAL_E_MTL:      &str = "material_e.mtl";
const WORKSHOPCONFIG:      &str = "workshopconfig.ini";


pub fn read_validate_sources(source_dir: &Path) -> Result<(Vec::<BuildingSource>, usize), usize> {
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

            let building_source_clean = match (render_src, render_ref) {
                (Some(render_src), None) => ModBuildingDef::from_render_path(&bld_ini, &render_src, resolve_source_path, false)
                                            .map_err(SourceError::Def),
                (None, Some(render_ref)) => get_source_type_from_ref(bld_ini, render_ref, &mut str_buf),
                (None, None)       => Err(SourceError::NoRenderconfig), 
                (Some(_), Some(_)) => Err(SourceError::MultiRenderconfig),
            };

            let building_source = building_source_clean.and_then(|def| {
                // NOTE: debug
                //println!("{}: {}", path.strip_prefix(source_dir).unwrap().display(), def);

                path.push(BUILDING_SKINS);
                let skins = if path.exists() {
                    skins::read_skins(path.as_path(), &mut str_buf).map_err(SourceError::Skins)
                } else { 
                    Ok(Skins::with_capacity(0))
                };
                path.pop();

                skins.and_then(|skins| {
                    skins_count += skins.len();
                    path.push(BUILDING_ACTIONS);
                    let actions = if path.exists() {
                        actions::read_actions(&mut path, &mut str_buf).map(Some).map_err(SourceError::Actions)
                    } else {
                        Ok(None)
                    };
                    path.pop();

                    actions.and_then(|actions| {
                        // NOTE: debug
                        //println!("skins:\n{:#?}", bs.skins);
                        //println!("actions:\n{:?}", actions);
                        Ok(BuildingSource { source_dir: path.clone(), def, skins, actions })
                    })
                })
            });

            // VALIDATIONS
            let building_source = building_source.and_then(|bs| {
                let mut nmf_info = nmf::NmfInfo::from_path(bs.def.model.as_path()).map_err(SourceError::Nmf)?;
                if let Some(act) = &bs.actions {
                    act.validate(&bs.def.building_ini, &nmf_info, &mut str_buf).map_err(SourceError::Actions)?;
                    act.apply_to(&mut nmf_info);
                }

                bs.def.parse_and_validate(Some(&nmf_info)).map_err(SourceError::Def)?;

                let sm_used = nmf_info.get_used_sumbaterials().collect::<Vec<_>>();
                skins::validate(&bs.skins, &path, &sm_used[..], &mut str_buf).map_err(SourceError::Skins)?;

                Ok(bs)
            });

            match building_source {
                Ok(bs) => {
                    println!("{}: OK", path.strip_prefix(source_dir).expect("Impossible: could not strip root prefix").display());
                    result.push(bs)
                },
                Err(e) => log_err!(e)
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




type AssetsMap = ahash::AHashMap::<PathBuf, PathBuf>;

pub fn install(sources: Vec<BuildingSource>, target: &Path, log_file: &mut BufWriter<fs::File>) {
    
    let dds_root = target.join("dds");
    fs::create_dir_all(&dds_root).unwrap();
    let nmf_root = target.join("nmf");
    fs::create_dir_all(&nmf_root).unwrap();

    let mut pathbuf = target.to_path_buf();
    let mut assets_map = AssetsMap::with_capacity(10000);
    let mut str_buf = String::with_capacity(16 * 1024);
    let mut byte_buf = Vec::<u8>::with_capacity(32 * 1024 * 1024);
    let mut skins_buf = Vec::<(usize, usize, &PathBuf, Option<&PathBuf>)>::with_capacity(AppSettings::MAX_SKINS_IN_MOD);

    let mut src_iter = sources.iter();
    let mut mod_id_iter = (AppSettings::MOD_IDS_START .. AppSettings::MOD_IDS_END).into_iter();
    while let Some(mod_id) = mod_id_iter.next() {
        str_buf.clear();
        write!(str_buf, "{}", mod_id).unwrap();
        pathbuf.push(&str_buf);
        for bld_id in 0 .. AppSettings::MAX_BUILDINGS_IN_MOD {
            if let Some(src) = src_iter.next() {
                str_buf.clear();
                write!(str_buf, "{:0>2}", bld_id).unwrap();
                writeln!(log_file, "{}/{} {}", mod_id, &str_buf, src.source_dir.display()).unwrap();
                pathbuf.push(&str_buf);

                fs::create_dir_all(&pathbuf).unwrap();

                install_building(&src.def, &src.actions, &pathbuf, &dds_root, &nmf_root, &mut assets_map, &mut str_buf, &mut byte_buf).unwrap();
                for (skin, skin_e) in src.skins.iter() {
                    skins_buf.push((mod_id, bld_id, skin, skin_e.as_ref()));
                    if skins_buf.len() == AppSettings::MAX_SKINS_IN_MOD {
                        let skin_mod_id = write_skins_mod(target, &mut mod_id_iter, &skins_buf[..], &dds_root, &mut assets_map, &mut str_buf, &mut byte_buf);
                        skins_buf.clear();
                        writeln!(log_file, "{} <SKINS>", skin_mod_id).unwrap();
                    }
                }

                pathbuf.pop();
            } else {
                pathbuf.push(WORKSHOPCONFIG);
                write_workshop_ini_buildings(pathbuf.as_path(), mod_id, bld_id, &mut str_buf);
                if !skins_buf.is_empty() {
                    let skin_mod_id = write_skins_mod(target, &mut mod_id_iter, &skins_buf[..], &dds_root, &mut assets_map, &mut str_buf, &mut byte_buf);
                    writeln!(log_file, "{} <SKINS>", skin_mod_id).unwrap();
                }
                return;
            }
        }

        pathbuf.push(WORKSHOPCONFIG);
        write_workshop_ini_buildings(pathbuf.as_path(), mod_id, AppSettings::MAX_BUILDINGS_IN_MOD, &mut str_buf);
        pathbuf.pop();
        pathbuf.pop();
    }
}

#[must_use]
fn write_skins_mod(target: &Path, 
                   mod_id_iter: &mut impl Iterator<Item = usize>, 
                   skins: &[(usize, usize, &PathBuf, Option<&PathBuf>)], 
                   dds_root: &Path,
                   assets_map: &mut AssetsMap,
                   str_buf: &mut String,
                   byte_buf: &mut Vec<u8>
                   ) -> usize 
{
    let mod_id = mod_id_iter.next().expect("Too many mods");
    let mut pathbuf = target.to_path_buf();

    str_buf.clear();
    write!(str_buf, "{}", mod_id).unwrap();
    pathbuf.push(&str_buf);
    fs::create_dir(&pathbuf).unwrap();

    let mut config_buf = String::with_capacity(4 * 1024);
    writeln!(config_buf, 
        "$ITEM_ID {}\n\
         $OWNER_ID 12345678901234567\n\
         $ITEM_TYPE WORKSHOP_ITEMTYPE_BUILDINGSKIN\n\
         $VISIBILITY 2\n", 
        mod_id).unwrap();

    for ((m, b, mtl, mtl_e), i) in skins.iter().zip(1..) {
        str_buf.clear();
        write!(str_buf, "{:0>2}.mtl", i).unwrap();
        write!(config_buf, "\n$TARGET_BUILDING_SKIN {}/{:0>2} {}", m, b, str_buf).unwrap();

        pathbuf.push(&str_buf);
        fs::copy(&mtl, &pathbuf).expect("Could not copy skin's mtl file");
        update_mtl(&pathbuf, &mtl, dds_root, assets_map, str_buf, byte_buf).unwrap();
        pathbuf.pop();

        if let Some(mtl) = mtl_e {
            str_buf.clear();
            write!(str_buf, "{:0>2}_e.mtl", i).unwrap();
            write!(config_buf, " {}", str_buf).unwrap();

            pathbuf.push(&str_buf);
            fs::copy(mtl, &pathbuf).expect("Could not copy skin's mtl_e file");
            update_mtl(&pathbuf, &mtl, dds_root, assets_map, str_buf, byte_buf).unwrap();
            pathbuf.pop();
        }
    }

    writeln!(config_buf, "\n\n$ITEM_NAME \"Automatically generated by wrsr-mt modpack installer\"\
                            \n$ITEM_DESC \"Automatically generated by wrsr-mt modpack installer\"\
                          \n\n$END").unwrap();

    pathbuf.push(WORKSHOPCONFIG);
    fs::write(pathbuf, config_buf).unwrap();

    mod_id
}

fn write_workshop_ini_buildings(path: &Path, mod_id: usize, count: usize, buf: &mut String) {
    if count == 0 {
        return;
    }

    buf.clear();
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

fn install_building(src_def: &ModBuildingDef,
                    actions: &Option<actions::ModActions>,
                    destination: &Path, 
                    dds_root: &Path,
                    nmf_root: &Path,
                    assets_map: &mut AssetsMap, 
                    str_buf: &mut String,
                    byte_buf: &mut Vec<u8>) -> Result<(), IOErr> {

    str_buf.clear();
    byte_buf.clear();

    let new_render_path = destination.join(RENDERCONFIG_INI);
    fs::copy(&src_def.render, &new_render_path)?;

    let mut new_def = src_def.clone();

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
            if let (Some(src_fld), Some(dest_fld)) = (src_def.$fld.as_ref(), new_def.$fld.as_mut()) {
                copy_fld!(src_fld, dest_fld, $dest_name);
            }
        }
    }

    macro_rules! copy_nmf_token {
        ($nmf_path:expr) => {{
            let nmf_path = $nmf_path;
            match actions {
                None          => nmf_path.push(copy_asset_md5(nmf_path, nmf_root, byte_buf, assets_map)?),
                Some(actions) => nmf_path.push(copy_nmf_with_actions(nmf_path, nmf_root, byte_buf, actions)?)
            };

            Result::<String, IOErr>::Ok(make_relative_token(&new_render_path, nmf_path).expect("Could not construct relative nmf token"))
        }}
    }
    
    macro_rules! copy_nmf_token_opt { 
        ($nmf_path:expr) => { 
            if let Some(path) = $nmf_path.as_mut() {
                Some(copy_nmf_token!(path)).transpose()
            } else { Ok(None) }
        };
    }

    macro_rules! update_mtl {
        ($mtl_path:expr, $old_mtl_path:expr) => {
            update_mtl($mtl_path, $old_mtl_path, &dds_root, assets_map, str_buf, byte_buf)
        }
    }

    //-----------------------------------------------------------------

    copy_fld!(src_def.building_ini, new_def.building_ini, BUILDING_INI);
    copy_fld!(src_def.material,     new_def.material,     MATERIAL_MTL);
    copy_fld_opt!(material_e, MATERIAL_E_MTL); 
    copy_fld_opt!(image_gui, "imagegui.png"); 
    
    // Update config files
    {   
        let model_token:      String         = copy_nmf_token!(&mut new_def.model)?;
        let model_lod_token:  Option<String> = copy_nmf_token_opt!(new_def.model_lod)?;
        let model_lod2_token: Option<String> = copy_nmf_token_opt!(new_def.model_lod2)?;
        let model_e_token:    Option<String> = copy_nmf_token_opt!(new_def.model_e)?;

        // Update renderconfig.ini

        read_to_string_buf(&new_render_path, str_buf)?;
        let mut render_ini = ini::parse_renderconfig_ini(str_buf).expect("Invalid building renderconfig");
        for token_state in render_ini.tokens_mut() {
            token_state.modify(|t| {
                use ini::renderconfig::Token as RT;
                
                match t {
                    RT::Model(_)            => Some(RT::Model(IdStringParam::new_cloned(&model_token))),
                    RT::ModelLod((_, z))    => model_lod_token.as_ref().map(|t| RT::ModelLod((IdStringParam::new_cloned(t), *z))),
                    RT::ModelLod2((_, z))   => model_lod2_token.as_ref().map(|t| RT::ModelLod2((IdStringParam::new_cloned(t), *z))),
                    RT::ModelEmissive(_)    => model_e_token.as_ref().map(|t| RT::ModelEmissive(IdStringParam::new_cloned(t))),
                    RT::Material(_)         => Some(RT::Material(IdStringParam::new_cloned(MATERIAL_MTL))),
                    RT::MaterialEmissive(_) => new_def.material_e.as_ref().map(|_| RT::MaterialEmissive(IdStringParam::new_cloned(MATERIAL_E_MTL))),
                    _ => None
                }
            });
        }

        // Apply actions to renderconfig
        if let Some(actions) = actions {
            if let Some(factor) = actions.scale {
                ini::transform::scale_render(&mut render_ini, factor)
            }

            if actions.mirror {
                ini::transform::mirror_z_render(&mut render_ini)
            }
        }

        render_ini.write_file(new_render_path)?;

        // Apply actions to building.ini
        read_to_string_buf(&new_def.building_ini, str_buf)?;
        let mut bld_ini = ini::parse_building_ini(str_buf).expect("Invalid building ini");
        if let Some(actions) = actions {
            if let Some(factor) = actions.scale {
                ini::transform::scale_building(&mut bld_ini, factor)
            }

            if actions.mirror {
                ini::transform::mirror_z_building(&mut bld_ini)
            }
        }
        bld_ini.write_file(&new_def.building_ini)?;
    }

    // Copy textures and update *.mtl files
    update_mtl!(&new_def.material, &src_def.material)?;
    if let (Some(material_e), Some(src_mtl_e)) = (&new_def.material_e, &src_def.material_e) {
        update_mtl!(material_e, src_mtl_e)?;
    }

    Ok(())
}


lazy_static! {
    static ref RX_REF: Regex = Regex::new(r"^(#(\d{10}/[^\s]+))|([^\r\n]+)").unwrap();
}


fn get_source_type_from_ref(bld_ini: PathBuf, mut render_ref: BasePathBuf, buf: &mut String) -> Result<ModBuildingDef, SourceError> {
    read_to_string_buf(&render_ref, buf).map_err(SourceError::RefRead)?;
    let caps = RX_REF.captures(buf).ok_or(SourceError::RefParse)?;
    let mut root: BasePathBuf = if let Some(c) = caps.get(2) {
        // workshop
        Ok(APP_SETTINGS.path_workshop.join(c.as_str()))
    } else if let Some(c) = caps.get(3) {
        // relative path
        render_ref.pop().unwrap();
        Ok(render_ref.join(c.as_str()))
    } else {
        Err(SourceError::RefParse)
    }?;

    root.push(RENDERCONFIG_INI);
    ModBuildingDef::from_render_path(&bld_ini, root.as_path(), resolve_source_path, true)
        .map_err(SourceError::Def)
}


fn copy_asset_md5<'map>(asset_path: &Path, assets_root: &Path, byte_buf: &mut Vec<u8>, assets_map: &'map mut AssetsMap) -> Result<&'map Path, IOErr> {

    // TODO: update this when borrowchecker is made less stupid
    if !assets_map.contains_key(asset_path) {
        let file_ext = asset_path.extension()
            .ok_or_else(|| IOErr::new(std::io::ErrorKind::Other, "Asset has no extension"))?
            .to_string_lossy();

        read_to_buf(asset_path, byte_buf)?;
        let asset_md5name = format!("{:x}.{}", md5::compute(byte_buf.as_mut_slice()), file_ext);

        let new_key = asset_path.to_path_buf();
        let new_val = assets_root.join(&asset_md5name);

        if !new_val.exists() {
            fs::write(&new_val, byte_buf.as_slice())?;
        }

        assets_map.insert(new_key, new_val);
    }

    let v = assets_map.get(asset_path).expect("HasMap 'get' failed right after insertion").as_path();
    Ok(v)
}


fn copy_nmf_with_actions(asset_path: &Path, assets_root: &Path, byte_buf: &mut Vec<u8>, actions: &ModActions) -> Result<PathBuf, IOErr> {
    let mut model = nmf::NmfBufFull::from_path(asset_path).expect(&format!("Could not read NMF at {}", asset_path.display()));

    if let Some(obj_act) = &actions.objects {
        let mut tmp_objects = Vec::<nmf::ObjectFull>::with_capacity(model.objects.len());

        match obj_act {
            (actions::ObjectVerb::Keep, kept) =>
                for o in model.objects.drain(..) {
                    if kept.iter().any(|k| k == o.name()) {
                        tmp_objects.push(o);
                    }
                },
            (actions::ObjectVerb::Remove, remd) =>
                for o in model.objects.drain(..) {
                    if remd.iter().all(|r| r != o.name()) {
                        tmp_objects.push(o);
                    }
                },
        }

        model.objects = tmp_objects;
    }

    for obj in model.objects.iter_mut() {
        if let Some(factor) = actions.scale {
            obj.scale(factor);
        }

        if actions.mirror {
            obj.mirror_z();
        }
    }

    'outer: for (old_name, new_name) in actions.rename_sm.iter() {
        for sm in model.submaterials.iter_mut() {
            if sm.as_str() == old_name {
                sm.push_str(new_name);
                continue 'outer;
            }
        }

        panic!("Invalid submaterial rename action. The building source validation should have caught this.");
    }

    byte_buf.clear();
    let mut cursor = std::io::Cursor::new(byte_buf);
    model.write_to(&mut cursor).expect("Failed to write modified NMF into memory buffer");
    let byte_buf = cursor.into_inner();
    let asset_md5name = format!("{:x}.nmf", md5::compute(byte_buf.as_slice()));
    let new_file = assets_root.join(asset_md5name);

    if !new_file.exists() {
        fs::write(&new_file, byte_buf)?;
    }

    Ok(new_file)
}


// panics on invalid mtl
fn update_mtl(mtl_path: &Path, 
              old_mtl_path: &Path, 
              dds_root: &Path, 
              assets_map: &mut AssetsMap,
              str_buf: &mut String, 
              byte_buf: &mut Vec<u8>
              ) -> Result<(), IOErr> {
    let old_mtl_root = old_mtl_path.parent().unwrap();
    read_to_string_buf(mtl_path, str_buf)?;
    let mut mtl = ini::parse_mtl(str_buf).expect("Invalid *.mtl");

    macro_rules! update_tx_token {
        ($token:ident, $path_resolver:expr) => {{
            let src_tx_path = $path_resolver($token);
            let new_tx_path = copy_asset_md5(&src_tx_path, dds_root, byte_buf, assets_map).expect("Could not copy texture when updating mtl");
            let tx_token = make_relative_token(mtl_path, &new_tx_path).expect("Could not construct relative texture token");
            ini::common::IdStringParam::new_owned(tx_token)
        }}
    }


    for token_state in mtl.tokens_mut() {
        token_state.modify(|t| {
            use ini::material::Token as MT;
            
            match t {
                MT::Texture(        (i, p)) => Some(MT::TextureMtl(     (*i, update_tx_token!(p, resolve_stock_path)) )),
                MT::TextureNoMip(   (i, p)) => Some(MT::TextureNoMipMtl((*i, update_tx_token!(p, resolve_stock_path)) )),
                MT::TextureMtl(     (i, p)) => Some(MT::TextureMtl(     (*i, update_tx_token!(p, |p| resolve_source_path(&old_mtl_root, p)) ))),
                MT::TextureNoMipMtl((i, p)) => Some(MT::TextureNoMipMtl((*i, update_tx_token!(p, |p| resolve_source_path(&old_mtl_root, p)) ))), 
                _ => None
            }
        });
    }

    mtl.write_file(mtl_path)
}

pub fn make_relative_token(path_from: &Path, path_to: &Path) -> Option<String> {

    let mut iter_from = path_from.parent()?.components();
    let mut iter_to = path_to.parent()?.components();

    while let (Some(c_from), Some(c_to)) = (iter_from.next(), iter_to.next()) {
        if c_from != c_to {
            let mut new_token = String::with_capacity(128);
            new_token.push_str("../");
            for _ in iter_from {
                new_token.push_str("../");
            }

            new_token.push_str(&c_to.as_os_str().to_string_lossy());
            new_token.push_str("/");
            for c in iter_to {
                new_token.push_str(&c.as_os_str().to_string_lossy());
                new_token.push_str("/");
            }

            new_token.push_str(&path_to.file_name()?.to_string_lossy());

            return Some(new_token);
        }
    }

    None
}

impl fmt::Display for BuildingSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "{}", self.def)?;
        writeln!(f, "skins: {:?}", self.skins)?;
        writeln!(f, "actions: {:?}", self.actions)
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use SourceError as E;
        match self {
            E::NoRenderconfig    => write!(f, "Building source is missing one of renderconfig.source or renderconfig.ref"),
            E::MultiRenderconfig => write!(f, "Building source has both renderconfig.source and renderconfig.ref. Only one is required."),
            E::Def(e)            => write!(f, "BuildingDef error: {}", e),
            E::RefRead(e)        => write!(f, "Error reading building reference: {}", e),
            E::RefParse          => write!(f, "Cannot parse building reference"),
            E::Skins(e)          => write!(f, "Skins error: {:#?}", e),
            E::Actions(e)        => write!(f, "Actions error: {}", e),
            E::Nmf(e)            => write!(f, "Nmf error: {:#?}", e),
        }
    }
}

//use std::env;
use std::fmt::Write;
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::ops::Range;

use crate::{Category, BuildingDef, RenderConfig, IniToken, Texture, Skin,
            MAX_BUILDINGS_IN_MOD, MAX_SKINS_IN_MOD, MOD_IDS_START, MOD_IDS_END,
           };


const FILENAME_MTL: &str = "material.mtl";
const FILENAME_MTL_E: &str = "material_e.mtl";


type Replacement = (Range<usize>, String);

type AssetsMap<'a> = HashMap<&'a Path, String>;

pub(crate) fn generate_mods<'stock>(dest: &Path, data: Vec<Category<'stock>>) {
    let mut pathbuf = dest.to_path_buf();
    fs::create_dir(&pathbuf).unwrap();

    let mut pathbuf_models = pathbuf.clone();
    pathbuf_models.push("nmf");
    if !pathbuf_models.exists() {
        fs::create_dir(&pathbuf_models).unwrap();
    }
    
    let mut pathbuf_textures = pathbuf.clone();
    pathbuf_textures.push("dds");
    if !pathbuf_textures.exists() {
        fs::create_dir(&pathbuf_textures).unwrap();
    }

    let mut assets_map = AssetsMap::with_capacity(10_000);
    let mut buf_assetbytes = Vec::<u8>::with_capacity(1024*1024*64);
    let mut mod_id_iter = MOD_IDS_START ..= MOD_IDS_END;

    let bld_iter = data.iter()
                       .flat_map(|c| c.styles.iter())
                       .flat_map(|s| s.buildings.iter());

    let skins: Vec<(&Vec<Skin>, String)> = write_mod_objects(
        bld_iter, 
        &mut mod_id_iter,
        MAX_BUILDINGS_IN_MOD,
        &mut pathbuf,
        "BUILDING",
        |buf, _, dirname| 
            write!(buf, "$OBJECT_BUILDING {}\n", dirname).unwrap(),
        |bld, pth| 
            install_building_files(bld, pth, &mut pathbuf_models, &mut pathbuf_textures, &mut buf_assetbytes, &mut assets_map)
    );

    let skins_iter = skins.iter().flat_map(|(v, s)| v.iter().zip(std::iter::repeat(s.as_str())));
    
    write_mod_objects(
        skins_iter,
        &mut mod_id_iter,
        MAX_SKINS_IN_MOD,
        &mut pathbuf,
        "BUILDINGSKIN",
        |buf, (skin, bld_ref), dirname| {
            write!(buf, "$TARGET_BUILDING_SKIN {0} {1}/{2}", bld_ref, dirname, FILENAME_MTL).unwrap();
            if skin.material_emissive.is_some() {
                write!(buf, " {0}/{1}", dirname, FILENAME_MTL_E).unwrap();
            }
            write!(buf, "\n").unwrap();
        },
        |(skin, _), pth| 
            install_building_skin(skin, pth, &mut pathbuf_textures, &mut buf_assetbytes, &mut assets_map)
    );

}


fn write_mod_objects<T, I1, I2, F1, F2, OUT>(
    obj_iter: I1, 
    mod_id_iter: &mut I2, 
    max_objects_per_mod: u8,
    pathbuf: &mut PathBuf, 
    modtype_token: &str,
    fn_add_obj_token: F1,
    mut fn_process_obj: F2
    ) -> Vec<OUT>
where I1: Iterator<Item = T>,
      I2: Iterator<Item = u32>,
      F1: Fn(&mut String, &T, &str),
      F2: FnMut(&T, &mut PathBuf) -> Option<OUT>,
{
    let mut mod_id = 0u32;
    let mut obj_id = 1u8;

    let mut buf_dirname = String::with_capacity(7);
    let mut buf_workshopconfig = String::with_capacity(1024);

    let mut result = Vec::<OUT>::with_capacity(obj_iter.size_hint().0);

    for obj in obj_iter {
        if obj_id == 1 {
            // setup new mod folder for this object
            mod_id = mod_id_iter.next().unwrap();

            buf_dirname.clear();
            write!(&mut buf_dirname, "{}", mod_id).unwrap();
            pathbuf.push(&buf_dirname);
            fs::create_dir(&pathbuf).unwrap();
            
            buf_workshopconfig.clear();
            write!(&mut buf_workshopconfig, 
                   "$ITEM_ID {}\n\
                    $OWNER_ID 12345678901234567\n\
                    $ITEM_TYPE WORKSHOP_ITEMTYPE_{}\n\
                    $VISIBILITY 2\n\n",
                   &buf_dirname, modtype_token).unwrap();
        }

        buf_dirname.clear();
        write!(&mut buf_dirname, "{:0>2}", obj_id).unwrap();
        pathbuf.push(&buf_dirname); // mod dir -> object subdir
        fs::create_dir(&pathbuf).unwrap();

        fn_add_obj_token(&mut buf_workshopconfig, &obj, &buf_dirname);

        if let Some(r) = fn_process_obj(&obj, pathbuf) {
            result.push(r);
        }

        pathbuf.pop(); // obj subdir -> mod dir

        obj_id += 1;
        if obj_id > max_objects_per_mod {
            obj_id = 1;
            mod_id += 1;

            finish_mod(&mut buf_workshopconfig, pathbuf);
        }
    }

    if obj_id > 1 {
        finish_mod(&mut buf_workshopconfig, pathbuf);
    }

    result
}


fn finish_mod(buf_config: &mut String, pathbuf: &mut PathBuf) {
    buf_config.push_str("\n\
        $ITEM_NAME \"generated\"\n\
        $ITEM_DESC \"generated descr\"\n\
        $END");

    pathbuf.push("workshopconfig.ini");
    fs::write(&pathbuf, &buf_config).unwrap();
    pathbuf.pop();

    pathbuf.pop(); // mod dir -> root
}


//--------------------------------------------------------------
fn install_building_files<'bld, 'stock>(
    bld: &'bld BuildingDef<'stock>, 
    pathbuf: &mut PathBuf, 
    pathbuf_models: &mut PathBuf, 
    pathbuf_textures: &mut PathBuf, 
    buf_assets: &mut Vec<u8>, 
    assets_map: &mut AssetsMap<'bld>) -> Option<(&'bld Vec<Skin>, String)>
{
    copy_file(&bld.building_ini, pathbuf, "building.ini");
    copy_file(&bld.bbox, pathbuf, "building.bbox");
    copy_file_opt(&bld.fire, pathbuf, "building.fire");
    copy_file_opt(&bld.imagegui, pathbuf, "imagegui.png");

    let mut copy_asset = |x| format!("../../nmf/{}", copy_asset_md5(x, pathbuf_models, buf_assets, assets_map));

    let new_model = (bld.model.range.clone(), copy_asset(&bld.model.value));
    let new_model_lod1 = bld.model_lod1.as_ref().map(|x| (x.range.clone(), copy_asset(&x.value)));
    let new_model_lod2 = bld.model_lod2.as_ref().map(|x| (x.range.clone(), copy_asset(&x.value)));
    let new_model_emissive = bld.model_emissive.as_ref().map(|x| (x.range.clone(), copy_asset(&x.value)));

    let new_material = {
        pathbuf.push(FILENAME_MTL);
        create_material(&bld.material.render_token.value, &bld.material.textures, pathbuf, pathbuf_textures, buf_assets, assets_map);
        pathbuf.pop();

        (bld.material.render_token.range.clone(), String::from(FILENAME_MTL))
    };

    let new_material_emissive = bld.material_emissive.as_ref().map(|x| {
        pathbuf.push(FILENAME_MTL_E);
        create_material(&x.render_token.value, &x.textures, pathbuf, pathbuf_textures, buf_assets, assets_map);
        pathbuf.pop();

        (x.render_token.range.clone(), String::from(FILENAME_MTL_E))
    });

    write_renderconfig(
        &bld.render_config, 
        pathbuf, 
        new_model, 
        new_model_lod1, 
        new_model_lod2, 
        new_model_emissive,
        new_material,
        new_material_emissive
    );


    if bld.skins.len() > 0 {
        let bld_ref = { 
            let bld_dir = pathbuf.file_name().unwrap().to_str().unwrap();
            let mod_dir = pathbuf.parent().unwrap().file_name().unwrap().to_str().unwrap();
            format!("{}/{}", mod_dir, bld_dir)
        };

        Some((&bld.skins, bld_ref))
    } else {
        None
    }
}

fn install_building_skin<'bld>(
    skin: &'bld Skin, 
    pathbuf: &mut PathBuf, 
    pathbuf_textures: &mut PathBuf, 
    buf_assets: &mut Vec<u8>,
    assets_map: &mut AssetsMap<'bld>) -> Option<()>
{
    pathbuf.push(FILENAME_MTL);
    create_material(&skin.material.path, &skin.material.textures, pathbuf, pathbuf_textures, buf_assets, assets_map);

    if let Some(ref mat_e) = skin.material_emissive {
        pathbuf.set_file_name(FILENAME_MTL_E);
        create_material(&mat_e.path, &mat_e.textures, pathbuf, pathbuf_textures, buf_assets, assets_map);
    }

    pathbuf.pop();

    None
}


#[inline]
fn copy_file(src: &Path, dest: &mut PathBuf, dest_name: &str) {
    dest.push(dest_name);
    fs::copy(src, &dest).unwrap();
    dest.pop();
}

#[inline]
fn copy_file_opt<P>(src: &Option<P>, dest: &mut PathBuf, dest_name: &str) 
where P: AsRef<Path>
{
    if let Some(p) = src.as_ref() {
        copy_file(p.as_ref(), dest, dest_name);
    }
}

fn create_material<'bld>(
    src_mtl_path: &Path, 
    mtl_textures: &'bld Vec<IniToken<Texture>>, 
    dest_mtl_path: &PathBuf, 
    pathbuf_textures: &mut PathBuf, 
    buf_assets: &mut Vec<u8>,
    assets_map: &mut AssetsMap<'bld>) 
{
    let mut replacements: Vec<Option<Replacement>> = mtl_textures.iter().map(|tx| {
        let rng = tx.range.clone();
        let val = format!("$TEXTURE_MTL {} ../../dds/{}", &tx.value.num, copy_asset_md5(&tx.value.path, pathbuf_textures, buf_assets, assets_map));
        // NOTE: Debug
        // println!("{:?}", &val);
        Some((rng, val))
    }).collect();

    let src_mtl = fs::read_to_string(&src_mtl_path).unwrap();

    write_config(&src_mtl, dest_mtl_path, replacements.as_mut_slice(), None);
}

fn copy_asset_md5<'bld, 'map>(
    src_file: &'bld PathBuf, 
    dest_dir: &mut PathBuf, 
    buf: &mut Vec<u8>, 
    assets_map: &'map mut AssetsMap<'bld>) -> &'map str
{
    assets_map.entry(src_file.as_path()).or_insert_with(|| {
        use std::convert::TryInto;
        buf.clear();
        
        let mut file = fs::File::open(src_file).unwrap();
        let meta = file.metadata().unwrap();
        let sz: usize = meta.len().try_into().unwrap();
        if sz > buf.len() {
            buf.reserve(sz);
        }

        file.read_to_end(buf).unwrap();
        let dig = md5::compute(buf.as_mut_slice());
        let ext = src_file.extension().unwrap().to_str().unwrap();
        let filename = format!("{:x}.{}", dig, ext);

        dest_dir.push(&filename);
        if !dest_dir.exists() {
            fs::write(&dest_dir, &buf).unwrap();
        }
        dest_dir.pop();

        filename
    })
}


//--------------------------------------
fn write_renderconfig<'stock>(
    render_config: &RenderConfig<'stock>,
    pathbuf: &mut PathBuf,
    new_model: Replacement,
    new_model_lod1: Option<Replacement>,
    new_model_lod2: Option<Replacement>,
    new_model_emissive: Option<Replacement>,
    new_material: Replacement,
    new_material_emissive: Option<Replacement>)
{
    // TODO: optimal default order
    let mut tokens = [
       Some(new_model),
       new_model_lod1,
       new_model_lod2,
       Some(new_material),
       new_model_emissive,
       new_material_emissive,
    ];

    let src = match render_config {
        RenderConfig::Stock { data, .. } => data,
        RenderConfig::Mod(_) => todo!()
    };

    pathbuf.push("renderconfig.ini");
    write_config(src, pathbuf, &mut tokens, Some("$TYPE_WORKSHOP\r\n"));
    pathbuf.pop();
}

fn write_config(
    src: &str,
    pathbuf: &PathBuf,
    tokens: &mut [Option<Replacement>],
    prefix: Option<&str>)
{
    let mut buf = String::with_capacity(src.len() + 500);

    if let Some(pfx) = prefix {
        buf.push_str(pfx);
    }
    
    build_config(src, tokens, &mut buf);

    fs::write(&pathbuf, buf).unwrap();
}

fn build_config(src: &str, tokens: &mut [Option<Replacement>], buf: &mut String) {
    use std::cmp::Ordering;

    tokens.sort_by(|x, y| {
        match (x, y) {
            (Some((Range { start: s1, ..}, _)), Some((Range { start: s2, ..}, _))) => s1.cmp(&s2),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    });

    // NOTE: Debug
    /*for s in tokens.iter() {
        println!("++ {:?} ++", &s);
    }*/

    let mut z = 0usize;

    for pair in tokens.iter() {
        if let Some((rng, path)) = pair {
            buf.push_str(&src[z .. rng.start]);
            buf.push_str(path);
            z = rng.end;
        } else {
            break;
        }
    }

    buf.push_str(&src[z .. ]);
}

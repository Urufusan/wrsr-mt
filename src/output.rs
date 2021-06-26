//use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::ops::Range;

use crate::{Category, BuildingDef, RenderConfig, MaterialDef};


type Replacement = (Range<usize>, String);

const WSH_CFG_1: &str = "$ITEM_ID ";

const WSH_CFG_2: &str = "\n\
$OWNER_ID 12345678901234567\n\
$ITEM_TYPE WORKSHOP_ITEMTYPE_BUILDING\n\
$VISIBILITY 2\n\n";

const WSH_CFG_3: &str = "\n\
$ITEM_NAME \"generated\"\n\
$ITEM_DESC \"generated descr\"\n\
$END";


pub(crate) fn generate_mods<'stock>(dest: &Path, data: Vec<Category<'stock>>) {
    let mut pathbuf = dest.to_path_buf();
    fs::create_dir_all(&pathbuf).unwrap();

    let mut pathbuf_models = pathbuf.clone();
    pathbuf_models.push("nmf");
    fs::create_dir_all(&pathbuf_models).unwrap();
    
    let mut pathbuf_textures = pathbuf.clone();
    pathbuf_textures.push("dds");
    fs::create_dir_all(&pathbuf_textures).unwrap();

    let mut buf_workshopconfig = String::with_capacity(1024);
    buf_workshopconfig.push_str(WSH_CFG_1);

    let mut buf_str_mod = String::with_capacity(64);
    let mut buf_str_bld = String::with_capacity(2);

    let mut buf_assets = Vec::<u8>::with_capacity(1024*1024*64);

    for (i_cat, category) in data.iter().enumerate() {
        // mod dir must not start from 0 (otherwise it is ignored)
        let i_cat = i_cat + 10;
        assert!(i_cat < 100); // 2 digits max
        write!(&mut buf_str_mod, "{:0>2}", i_cat).unwrap();

        for (i_stl, style) in category.styles.iter().enumerate() {
            assert!(i_stl < 100); // 2 digits max
            write!(&mut buf_str_mod, "{:0>2}", i_stl).unwrap();

            // No more than MOD_BUILDINGS_MAX buildings in a single mod
            const MOD_BUILDINGS_MAX: u8 = 16;
            let mut i_mod = 0u16;
            let mut cnt = 0u8;

            assert!(style.buildings.len() > 0);
            for building in style.buildings.iter() {
                if cnt == 0 {
                    // setup new mod folder for this building
                    buf_str_mod.truncate(4);
                    assert!(i_mod < 1000); // 3 digits max
                    write!(&mut buf_str_mod, "{:0>3}", i_mod).unwrap();
                    pathbuf.push(&buf_str_mod);
                    fs::create_dir(&pathbuf).unwrap();

                    // TODO: mod root files
                    buf_workshopconfig.truncate(WSH_CFG_1.len());
                    buf_workshopconfig.push_str(&buf_str_mod);
                    buf_workshopconfig.push_str(WSH_CFG_2);
                }


                buf_str_bld.clear();
                write!(&mut buf_str_bld, "{:0>2}", cnt).unwrap();
                pathbuf.push(&buf_str_bld); // mod dir -> building subdir
                fs::create_dir(&pathbuf).unwrap();

                write!(&mut buf_workshopconfig, "$OBJECT_BUILDING {}\n", &buf_str_bld).unwrap();

                install_building_files(&building, &mut pathbuf, &mut pathbuf_models, &mut pathbuf_textures, &mut buf_assets);

                pathbuf.pop(); // building subdir -> mod dir

                cnt += 1;
                if cnt == MOD_BUILDINGS_MAX {
                    cnt = 0;
                    i_mod += 1;

                    finish_mod(&mut buf_workshopconfig, &mut pathbuf);
                }
            }

            if cnt != 0 {
                finish_mod(&mut buf_workshopconfig, &mut pathbuf);
            }


            buf_str_mod.truncate(2);
        }

        buf_str_mod.clear();
    }

}


fn finish_mod(buf_config: &mut String, pathbuf: &mut PathBuf) {
    buf_config.push_str(WSH_CFG_3);

    pathbuf.push("workshopconfig.ini");
    fs::write(&pathbuf, &buf_config).unwrap();
    pathbuf.pop();

    pathbuf.pop(); // mod dir -> root
}


//--------------------------------------------------------------
fn install_building_files<'stock>(
    bld: &BuildingDef<'stock>, 
    pathbuf: &mut PathBuf, 
    pathbuf_models: &mut PathBuf, 
    pathbuf_textures: &mut PathBuf, 
    buf_assets: &mut Vec<u8>) 
{
    copy_file(&bld.building_ini, pathbuf, "building.ini");
    copy_file(&bld.bbox, pathbuf, "building.bbox");
    copy_file_opt(&bld.fire, pathbuf, "building.fire");
    copy_file_opt(&bld.imagegui, pathbuf, "imagegui.png");

    let new_model = (bld.model.range.clone(), copy_asset_md5(&bld.model.value, pathbuf_models, buf_assets));
    let new_model_lod1 = bld.model_lod1.as_ref().map(|x| (x.range.clone(), copy_asset_md5(&x.value, pathbuf_models, buf_assets)));
    let new_model_lod2 = bld.model_lod2.as_ref().map(|x| (x.range.clone(), copy_asset_md5(&x.value, pathbuf_models, buf_assets)));
    let new_model_emissive = bld.model_emissive.as_ref().map(|x| (x.range.clone(), copy_asset_md5(&x.value, pathbuf_models, buf_assets)));

    static FILENAME_MTL: &str = "material.mtl";
    static FILENAME_MTL_E: &str = "material_e.mtl";

    let new_material = {
        pathbuf.push(FILENAME_MTL);
        create_material(&bld.material, pathbuf, pathbuf_textures, buf_assets);
        pathbuf.pop();

        (bld.material.render_token.range.clone(), String::from(FILENAME_MTL))
    };

    let new_material_emissive = bld.material_emissive.as_ref().map(|x| {
        pathbuf.push(FILENAME_MTL_E);
        create_material(x, pathbuf, pathbuf_textures, buf_assets);
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

fn create_material(mtl: &MaterialDef, path_mtl: &PathBuf, pathbuf_textures: &mut PathBuf, buf_assets: &mut Vec<u8>) {
    let mut replacements: Vec<Option<Replacement>> = mtl.textures.iter().map(|tx| {
        let rng = tx.range.clone();
        let val = format!("$TEXTURE_MTL {} {}", &tx.value.num, copy_asset_md5(&tx.value.path, pathbuf_textures, buf_assets));
        // NOTE: Debug
        // println!("{:?}", &val);
        Some((rng, val))
    }).collect();

    let src_mtl = fs::read_to_string(&mtl.render_token.value).unwrap();

    write_config(&src_mtl, path_mtl, replacements.as_mut_slice(), None);
}

fn copy_asset_md5(src_file: &PathBuf, dest_dir: &mut PathBuf, buf: &mut Vec<u8>) -> String {
    // TODO: memoize processed src_file's to avoid 
    // doing anything here more than once for each file

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
    let mut filename = format!("{:x}.{}", dig, ext);

    dest_dir.push(&filename);
    if !dest_dir.exists() {
        fs::write(&dest_dir, &buf).unwrap();
    }
    dest_dir.pop();

    // TODO: fix this ugly hack
    filename = format!("../../{}/{}", ext, filename);

    filename
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

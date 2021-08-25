

pub(crate) fn generate_mods<'stock>(dest: &Path, data: Vec<BuildingDef<'stock>>) {
    let mut pathbuf = dest.to_path_buf();
    if !pathbuf.exists() {
        fs::create_dir(&pathbuf).unwrap();
    }

    let skins: Vec<(&Vec<Skin>, String)> = write_mod_objects(
        data.iter(), 
        &mut mod_id_iter,
        AppSettings::MAX_BUILDINGS_IN_MOD,
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
        AppSettings::MAX_SKINS_IN_MOD,
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

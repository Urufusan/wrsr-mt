use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use regex::Regex;
use const_format::concatcp;

mod nmf;
mod ini;
mod cfg;
mod data;
mod input;
mod output;

mod building_def;

use cfg::APP_SETTINGS;


fn main() {

    match &APP_SETTINGS.command {
        cfg::AppCommand::Install(cfg::InstallCommand{ source, destination, is_check }) => {
            print_dirs();

            println!("Installing from source: {}", source.to_str().unwrap());
            assert!(source.exists(), "Pack source directory does not exist!");

            println!("Installing to:          {}", destination.to_str().unwrap());
            assert!(destination.exists(), "Destination directory does not exist.");
            
            let mut pathbuf: PathBuf = APP_SETTINGS.path_stock.clone();
            pathbuf.push("buildings");
            pathbuf.push("buildingtypes.ini");

            let stock_buildings_ini = fs::read_to_string(&pathbuf).expect("Stock buildings: cannot read buildingtypes.ini");
            let mut stock_buildings = { 
                let mut mp = HashMap::with_capacity(512);
                let rx = Regex::new(r"\$TYPE ([_[:alnum:]]+?)\r\n((?s).+?\n END\r\n)").expect("Stock buildings: cannot create parsing regex");

                for caps in rx.captures_iter(&stock_buildings_ini) {
                    let key = caps.get(1).unwrap().as_str();
                    let raw_value = caps.get(2).unwrap().as_str();
                    mp.insert(
                        key, 
                        (key, data::StockBuilding::Unparsed(raw_value))
                    );
                }
                
                mp
            };

            println!("Found {} stock buildings", stock_buildings.len());

            pathbuf.push(source);
            println!("Reading modpack sources...");

            match input::read_validate_sources(pathbuf.as_path(), &mut stock_buildings) {
                Ok(data) => {
                    println!("Modpack sources verified.");

                    if *is_check {
                        println!("Check complete.");
                    } else {
                        println!("Creating mods...");
                        pathbuf.push(destination);

                        output::generate_mods(pathbuf.as_path(), data);
                    }
                },
                Err(errs) => {
                    eprintln!("\nThe following {} errors were encountered when processing modpack sources:\n", errs.len());
                    for (i, e) in errs.iter().enumerate() {
                        eprintln!("{}) {}", i + 1, e);
                    }

                    eprintln!();

                    std::process::exit(1);
                }
            }
        },


        cfg::AppCommand::Nmf(cmd) => {
            match cmd {
                cfg::NmfCommand::Show(cfg::NmfShowCommand { path }) => {
                    let nmf = nmf::NmfInfo::from_path(path).expect("Failed to read the nmf file");
                    println!("{}", nmf);
                },

                cfg::NmfCommand::ToObj(cfg::NmfToObjCommand { input, output }) => {
                    let nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");

                    let f_out = fs::OpenOptions::new()
                                    .write(true)
                                    //.create(true)
                                    .create_new(true)
                                    .open(output)
                                    .expect("Cannot create output file");

                    let mut wr = std::io::BufWriter::new(f_out);

                    let mut d_v = 1_usize;

                    for obj in nmf.objects.iter() {
                        writeln!(wr, "o {}", obj.name()).unwrap();

                        let verts = obj.vertices();
                        for v in verts {
                            writeln!(wr, "v {:.6} {:.6} {:.6}", v.x, v.y, v.z).unwrap();
                        }

                        let uvs = obj.uv_map();
                        for uv in uvs {
                            writeln!(wr, "vt {:.6} {:.6}", uv.x, uv.y).unwrap();
                        }

                        let ns = obj.normals_1();
                        for n in ns {
                            writeln!(wr, "vn {:.6} {:.6} {:.6}", n.x, n.y, n.z).unwrap();
                        }

                        writeln!(wr, "s off").unwrap();

                        for f in obj.faces() {
                            writeln!(wr, "f {0:}/{0:}/{0:} {1:}/{1:}/{1:} {2:}/{2:}/{2:}", f.v1 as usize + d_v, f.v2 as usize + d_v, f.v3 as usize + d_v).unwrap();
                        }

                        d_v += verts.len();
                    }

                    wr.flush().expect("Failed flushing the output");
                    println!("Done");
                },

                cfg::NmfCommand::Scale(cfg::NmfScaleCommand { input, factor, output }) => {
                    let mut nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");
                    for o in nmf.objects.iter_mut() {
                        o.scale(*factor);
                    }
                    nmf.write_to_file(output).unwrap();
                    println!("Done");
                },

                cfg::NmfCommand::MirrorX(cfg::NmfMirrorXCommand { input, output }) => {
                    let mut nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");
                    for o in nmf.objects.iter_mut() {
                        o.mirror_x();
                    }
                    nmf.write_to_file(output).unwrap();
                    println!("Done");
                },
                
                cfg::NmfCommand::Patch(cfg::NmfPatchCommand { input: _, patch: _, output: _ }) => {
/*                
                    let buf = fs::read(input).expect("Cannot read nmf file at the specified path");
                    let (nmf, rest) = NmfSlice::parse_slice(buf.as_slice()).expect("Failed to parse the model nmf");
                    if !rest.is_empty() {
                        panic!("Nmf parsed with leftovers ({} bytes)", rest.len());
                    };

                    todo!()

                    let buf_patch = fs::read_to_string(patch).expect("Cannot read patch file at the specified path");
                    let patch = data::ModelPatch::try_from(buf_patch.as_str()).unwrap();

                    let patched = patch.apply(&nmf);

                    // NOTE: DEBUG
                    //println!("{}", &patched);

                    let file = std::fs::File::create(output).unwrap();
                    let mut writer = std::io::BufWriter::new(file);
                    patched.write_bytes(&mut writer);

                    println!("OK");
                    */
                }
            }
        },


        //---------------- mod subcommand --------------------------------
        cfg::AppCommand::ModBuilding(cmd) => {
            const RENDERCONFIG_INI: &str = "renderconfig.ini";
            const BUILDING_INI: &str = "building.ini";

            match cmd {
                cfg::ModCommand::Validate(cfg::ModValidateCommand { dir_input }) => {
                    let bld_ini = dir_input.join(BUILDING_INI);
                    let render_ini = dir_input.join(RENDERCONFIG_INI);
                    match building_def::BuildingDef::from_config(&bld_ini, &render_ini) {
                        Ok(bld) => {
                            println!("{}\nValidating...", bld);
                            match bld.parse_and_validate() {
                                Ok(()) => println!("OK"),
                                Err(e) => {
                                    eprintln!("Building has errors:\n{}", e);
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Building has errors:\n{}", e);
                            std::process::exit(1);
                        }
                    }
                },

                cfg::ModCommand::Scale(cfg::ModScaleCommand { dir_input, factor, dir_output }) => {

                    assert!(dir_input != dir_output, "Input and output directories must be different");

                    let render_ini = dir_input.join(RENDERCONFIG_INI);
                    let bld_ini = dir_input.join(BUILDING_INI);
                    let bld_def = building_def::BuildingDef::from_config(&bld_ini, &render_ini)
                        .expect("Cannot parse building");

                    // NOTE: debug
                    //println!("{}", bld_def);

                    {
                        macro_rules! check_path {
                            ($path:expr) => { assert!($path.starts_with(dir_input), 
                                              "To update the whole building in one operation, all potentially modified files (building.ini, \
                                              renderconfig.ini, *.nmf) must be located in the input directory. Otherwise you should update \
                                              files individually, one-by-one."); };
                        }

                        macro_rules! check_path_opt {
                            ($opt:expr) => { if let Some(path) = &$opt { 
                                    check_path!(path); 
                                } 
                            };
                        }

                        check_path!(bld_def.renderconfig);
                        check_path!(bld_def.building_ini);
                        check_path!(bld_def.model);
                        check_path_opt!(bld_def.model_lod);
                        check_path_opt!(bld_def.model_lod2);
                        check_path_opt!(bld_def.model_e);
                    }

                    println!("Building parsed successfully. Copying files...");
                    let bld_def = bld_def.shallow_copy_to(dir_output).expect("Cannot copy building files");
                    println!("Files copied. Updating...");

                    // Update INI files
                    let mut buf = String::with_capacity(16 * 1024);

                    macro_rules! scale_ini {
                        ($path:expr, $name:expr, $parser:expr, $scaler:expr) => {{
                            read_to_string_buf($path, &mut buf).expect(concatcp!("Cannot read ", $name));
                            let mut ini = $parser(&buf).expect(concatcp!("Cannot parse ", $name));
                            $scaler(&mut ini, *factor);
                            let out_writer = io::BufWriter::new(fs::OpenOptions::new().write(true).truncate(true).open($path).unwrap());
                            ini.write_to(out_writer).unwrap();
                            println!("{}: OK", $name);
                        }};
                    }

                    scale_ini!(&bld_def.building_ini, BUILDING_INI,     ini::parse_building_ini,     ini::transform::scale_building);
                    scale_ini!(&bld_def.renderconfig, RENDERCONFIG_INI, ini::parse_renderconfig_ini, ini::transform::scale_render);

                    // Update NMF models
                    let scale_nmf = |path: Option<&PathBuf>| {
                        if let Some(path) = path {
                            let mut nmf = nmf::NmfBufFull::from_path(path).expect("Failed to read the nmf file");
                            for o in nmf.objects.iter_mut() {
                                o.scale(*factor);
                            }

                            nmf.write_to_file(path).expect("Failed to write the updated nmf");
                            println!("{}: OK", path.strip_prefix(dir_output).unwrap().display());
                        }
                    };

                    scale_nmf(Some(&bld_def.model));
                    scale_nmf(bld_def.model_lod.as_ref());
                    scale_nmf(bld_def.model_lod2.as_ref());
                    scale_nmf(bld_def.model_e.as_ref());
                },
            }
        },


        //---------------- ini subcommand --------------------------------
        cfg::AppCommand::Ini(cmd) => {

            fn process_tokens<T: std::fmt::Display>(ts: Vec<(&str, ini::common::ParseResult<T>)>) {
                for (t_str, t_val) in ts.iter() {
                    match t_val {
                        Ok((t, rest)) => {
                            print!("{}", t);
                            if let Some(rest) = rest {
                                print!(" [remainder: {:?}]", rest);
                            }
                            println!();
                        },
                        Err(e) => println!(" > > > Error > > >\n > > > {}\n > > > chunk: [{}]", e, t_str),
                    }
                }
            }

            fn save_ini_as<U: ini::IniToken>(path: &Path, ini: ini::IniFile<U>) {
                let out_writer = io::BufWriter::new(fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap());
                ini.write_to(out_writer).expect("Could not write modified file");
                println!("Done. File saved as {}", path.display());
            }

            match cmd {
                cfg::IniCommand::ParseBuilding(path) => {
                    let buf = fs::read_to_string(path).expect("Cannot read the specified file");
                    let tokens = ini::parse_building_tokens(&buf);
                    process_tokens(tokens);
                },
                cfg::IniCommand::ParseRender(path) => {
                    let buf = fs::read_to_string(path).expect("Cannot read the specified file");
                    let tokens = ini::parse_render_tokens(&buf);
                    process_tokens(tokens);
                },
                cfg::IniCommand::ParseMaterial(path) => {
                    let buf = fs::read_to_string(path).expect("Cannot read the specified file");
                    let tokens = ini::parse_material_tokens(&buf);
                    process_tokens(tokens);
                },
                cfg::IniCommand::ScaleBuilding(path, factor) => {
                    let file = fs::read_to_string(path).expect("Cannot read the specified file");
                    let mut ini = ini::parse_building_ini(&file).expect("Cannot parse building.ini");
                    ini::transform::scale_building(&mut ini, *factor);
                    let mut path = path.clone();

                    path.set_file_name(format!("{}_x{}", path.file_name().unwrap().to_str().unwrap(), factor));
                    save_ini_as(&path, ini);
                },
                cfg::IniCommand::ScaleRender(path, factor) => {
                    let file = fs::read_to_string(path).expect("Cannot read the specified file");
                    let mut ini = ini::parse_renderconfig_ini(&file).expect("Cannot parse renderconfig");
                    ini::transform::scale_render(&mut ini, *factor);

                    let mut path = path.clone();
                    path.set_file_name(format!("{}_x{}", path.file_name().unwrap().to_str().unwrap(), factor));
                    save_ini_as(&path, ini);
                }
            }

        },

        //---------------- subcommands end --------------------------------
    };
}


fn print_dirs() {
    println!("Stock game files:   {}", APP_SETTINGS.path_stock.to_str().unwrap());
    assert!(APP_SETTINGS.path_stock.exists(), "Stock game files directory does not exist.");

    println!("Workshop directory: {}", APP_SETTINGS.path_workshop.to_str().unwrap());
    assert!(APP_SETTINGS.path_workshop.exists(), "Workshop directory does not exist.");
}


pub fn read_to_buf(path: &Path, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
    use std::io::Read;
    use std::convert::TryInto;
    buf.truncate(0);

    let mut file = fs::File::open(path)?;
    let meta = file.metadata()?;
    let sz: usize = meta.len().try_into().expect("Cannot get file length");
    buf.reserve(sz);
    file.read_to_end(buf)?;
    Ok(())
}


pub fn read_to_string_buf(path: &Path, buf: &mut String) -> Result<(), std::io::Error> {
    use std::io::Read;
    use std::convert::TryInto;
    buf.truncate(0);

    let mut file = fs::File::open(path)?;
    let meta = file.metadata()?;
    let sz: usize = meta.len().try_into().expect("Cannot get file length");
    buf.reserve(sz);
    file.read_to_string(buf)?;
    Ok(())
}

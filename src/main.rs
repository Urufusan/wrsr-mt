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
                cfg::NmfCommand::Show(path) => {
                    let nmf = nmf::NmfInfo::from_path(path).expect("Failed to read the nmf file");
                    println!("{}", nmf);
                },

                cfg::NmfCommand::ToObj(cfg::NmfToObjCommand { input, output }) => {
                    let nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");

                    let f_out = fs::OpenOptions::new()
                                    .write(true)
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

                cfg::NmfCommand::Scale(cfg::ScaleCommand { input, factor, output }) => {
                    let mut nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");
                    for o in nmf.objects.iter_mut() {
                        o.scale(*factor);
                    }
                    nmf.write_to_file(output).unwrap();
                    println!("Done");
                },

                cfg::NmfCommand::Mirror(cfg::MirrorCommand { input, output }) => {
                    let mut nmf = nmf::NmfBufFull::from_path(input).expect("Failed to read the nmf file");
                    for o in nmf.objects.iter_mut() {
                        o.mirror_z();
                    }
                    nmf.write_to_file(output).unwrap();
                    println!("Done");
                },
            }
        },


        //---------------- mod subcommand --------------------------------
        cfg::AppCommand::ModBuilding(cmd) => {
            use building_def::BuildingDef;

            const RENDERCONFIG_INI: &str = "renderconfig.ini";
            const BUILDING_INI: &str = "building.ini";

            fn check_and_copy_building(dir_input: &PathBuf, dir_output: &PathBuf) -> BuildingDef {
                let render_ini = dir_input.join(RENDERCONFIG_INI);
                let bld_ini = dir_input.join(BUILDING_INI);
                let bld_def = BuildingDef::from_config(&bld_ini, &render_ini)
                    .expect("Cannot parse building");

                {
                    let check_path = |path: &Path| assert!(path.starts_with(dir_input), 
                                          "To update the whole building in one operation, all potentially modified files (building.ini, \
                                          renderconfig.ini, *.nmf) must be located in the input directory. Otherwise you should update \
                                          files individually, one-by-one (using appropriate commands).");

                    let check_path_opt = |opt: &Option<PathBuf>| if let Some(p) = opt.as_ref() { check_path(p) };

                    check_path(&bld_def.renderconfig);
                    check_path(&bld_def.building_ini);
                    check_path(&bld_def.model);
                    check_path_opt(&bld_def.model_lod);
                    check_path_opt(&bld_def.model_lod2);
                    check_path_opt(&bld_def.model_e);
                }

                println!("Building parsed successfully. Copying files...");
                let bld_def = bld_def.shallow_copy_to(dir_output).expect("Cannot copy building files");
                println!("Files copied.");
                bld_def
            }

            macro_rules! modify_ini {
                ($buf:ident, $path:expr, $name:expr, $parser:expr, $modifier:expr $(, $m_p:expr)*) => {{
                    read_to_string_buf($path, &mut $buf).expect(concatcp!("Cannot read ", $name));
                    let mut ini = $parser(&mut $buf).expect(concatcp!("Cannot parse ", $name));
                    $modifier(&mut ini $(, $m_p)*);
                    let out_writer = io::BufWriter::new(fs::OpenOptions::new().write(true).truncate(true).open($path).unwrap());
                    ini.write_to(out_writer).unwrap();
                    println!("{}: OK", $name);
                }};
            }

            fn modify_models<F: Fn(&mut nmf::ObjectFull)>(bld_def: &BuildingDef, pfx: &Path, obj_modifier: F) {
                let modify_nmf = |path: Option<&PathBuf>| {
                    if let Some(path) = path {
                        let mut nmf = nmf::NmfBufFull::from_path(path).expect("Failed to read the nmf file");
                        for o in nmf.objects.iter_mut() {
                            obj_modifier(o);
                        }

                        nmf.write_to_file(path).expect("Failed to write the updated nmf");
                        println!("{}: OK", path.strip_prefix(pfx).unwrap().display());
                    }
                };

                modify_nmf(Some(&bld_def.model));
                modify_nmf(bld_def.model_lod.as_ref());
                modify_nmf(bld_def.model_lod2.as_ref());
                modify_nmf(bld_def.model_e.as_ref());
            }


            match cmd {
                cfg::ModCommand::Validate(dir_input) => {
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

                cfg::ModCommand::Scale(cfg::ScaleCommand { input: dir_input, factor, output: dir_output }) => {

                    let bld_def = check_and_copy_building(dir_input, dir_output);
                    println!("Updating...");

                    let mut buf = String::with_capacity(16 * 1024);
                    modify_ini!(buf, &bld_def.building_ini, BUILDING_INI,     ini::parse_building_ini,     ini::transform::scale_building, *factor);
                    modify_ini!(buf, &bld_def.renderconfig, RENDERCONFIG_INI, ini::parse_renderconfig_ini, ini::transform::scale_render,   *factor);
                    modify_models(&bld_def, dir_output, |o| o.scale(*factor));
                },
                cfg::ModCommand::Mirror(cfg::MirrorCommand { input: dir_input, output: dir_output }) => {
                    let bld_def = check_and_copy_building(dir_input, dir_output);
                    println!("Updating...");

                    let mut buf = String::with_capacity(16 * 1024);
                    modify_ini!(buf, &bld_def.building_ini, BUILDING_INI,     ini::parse_building_ini,     ini::transform::mirror_z_building);
                    modify_ini!(buf, &bld_def.renderconfig, RENDERCONFIG_INI, ini::parse_renderconfig_ini, ini::transform::mirror_z_render);
                    modify_models(&bld_def, dir_output, |o| o.mirror_z());
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
                cfg::IniCommand::ParseMtl(path) => {
                    let buf = fs::read_to_string(path).expect("Cannot read the specified file");
                    let tokens = ini::parse_material_tokens(&buf);
                    process_tokens(tokens);
                },
                cfg::IniCommand::ScaleBuilding(cfg::ScaleCommand { input, factor, output }) => {
                    let file = fs::read_to_string(input).expect("Cannot read the specified file");
                    let mut ini = ini::parse_building_ini(&file).expect("Cannot parse building.ini");
                    ini::transform::scale_building(&mut ini, *factor);
                    save_ini_as(output, ini);
                },
                cfg::IniCommand::ScaleRender(cfg::ScaleCommand { input, factor, output }) => {
                    let file = fs::read_to_string(input).expect("Cannot read the specified file");
                    let mut ini = ini::parse_renderconfig_ini(&file).expect("Cannot parse renderconfig");
                    ini::transform::scale_render(&mut ini, *factor);
                    save_ini_as(output, ini);
                },
                cfg::IniCommand::MirrorBuilding(cfg::MirrorCommand { input, output }) => {
                    let file = fs::read_to_string(input).expect("Cannot read the specified file");
                    let mut ini = ini::parse_building_ini(&file).expect("Cannot parse building.ini");
                    ini::transform::mirror_z_building(&mut ini);
                    save_ini_as(output, ini);
                },
                cfg::IniCommand::MirrorRender(cfg::MirrorCommand { input, output }) => {
                    let file = fs::read_to_string(input).expect("Cannot read the specified file");
                    let mut ini = ini::parse_renderconfig_ini(&file).expect("Cannot parse renderconfig");
                    ini::transform::mirror_z_render(&mut ini);
                    save_ini_as(output, ini);
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

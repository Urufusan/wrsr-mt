use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashMap;

use regex::Regex;

mod nmf;
mod cfg;
mod data;
mod input;
mod output;

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
                                    .create(true)
                                    //.create_new(true)
                                    .open(output)
                                    .expect("Cannot create output file");

                    let mut wr = std::io::BufWriter::new(f_out);

                    let mut d_v = 1_usize;
                    //let mut d_uv = 1_usize;
                    //let mut d_n = 1_usize;

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
                        //d_uv += uvs.len();
                        //d_n += ns.len();
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
*/
                    /*
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
        }
    };
}


fn print_dirs() {
    println!("Stock game files:   {}", APP_SETTINGS.path_stock.to_str().unwrap());
    assert!(APP_SETTINGS.path_stock.exists(), "Stock game files directory does not exist.");

    println!("Workshop directory: {}", APP_SETTINGS.path_workshop.to_str().unwrap());
    assert!(APP_SETTINGS.path_workshop.exists(), "Workshop directory does not exist.");
}

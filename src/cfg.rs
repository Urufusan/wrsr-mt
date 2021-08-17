use std::path::PathBuf;

use lazy_static::lazy_static;
use std::str::FromStr;



pub enum AppCommand {
    Install(InstallCommand),
    Nmf(NmfCommand),
    ModBuilding(ModCommand),
    Ini(IniCommand),
}

//-----------------------------

pub struct InstallCommand {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_check: bool
}

//-----------------------------

pub enum NmfCommand {
    Show(NmfShowCommand),
    ToObj(NmfToObjCommand),
    Scale(NmfScaleCommand),
    MirrorX(NmfMirrorXCommand),
    Patch(NmfPatchCommand),
}

pub struct NmfShowCommand {
    pub path: PathBuf,
}

pub struct NmfToObjCommand {
    pub input: PathBuf,
    pub output: PathBuf
}

pub struct NmfScaleCommand {
    pub input: PathBuf,
    pub factor: f64,
    pub output: PathBuf
}

pub struct NmfMirrorXCommand {
    pub input: PathBuf,
    pub output: PathBuf
}

pub struct NmfPatchCommand {
    pub input: PathBuf,
    pub patch: PathBuf,
    pub output: PathBuf
}

//-------------------------------

pub enum ModCommand {
    Validate(ModValidateCommand),
    Scale(ModScaleCommand),
}

pub struct ModValidateCommand {
    pub dir_input: PathBuf
}

pub struct ModScaleCommand {
    pub dir_input: PathBuf,
    pub factor: f64,
    pub dir_output: PathBuf
}

//-------------------------------

pub enum IniCommand {
    ParseBuilding(PathBuf),
    ParseRender(PathBuf),
    ParseMaterial(PathBuf),
}

//-------------------------------

pub struct AppSettings {
    pub path_stock: PathBuf,
    pub path_workshop: PathBuf,

    pub command: AppCommand,
}


impl AppSettings {

    // mod folder is 7 digits and cannot start from zero.
    pub const MOD_IDS_START:        u32 = 1_000_000;
    pub const MOD_IDS_END:          u32 = 9_999_999;
    pub const MAX_BUILDINGS_IN_MOD: u8  = 99;
    pub const MAX_SKINS_IN_MOD:     u8  = 16;

    pub const MAX_MODS:      usize = (AppSettings::MOD_IDS_END - AppSettings::MOD_IDS_START) as usize;
    pub const MAX_BUILDINGS: usize = AppSettings::MAX_MODS * (AppSettings::MAX_BUILDINGS_IN_MOD as usize);

    // Paths in ini files:
    pub const SRX_PATH_PREFIX: &'static str =  "([~.$]/)";
    pub const SRX_PATH:        &'static str = r"([^\r\s\n]+?)";
    pub const SRX_PATH_EXT:    &'static str =  "([^\"\\r\\n]+?)";
    pub const SRX_EOL:         &'static str = r"(:?[\s\r\n$])";
}



lazy_static! {
    pub static ref APP_SETTINGS: AppSettings = {
        // TODO: read from configuration + arguments
        use clap::{App, Arg, SubCommand};

        let cmd_install = SubCommand::with_name("install")
            .about("Install modpack from specified source")
            .arg(Arg::with_name("in").required(true))
            .arg(Arg::with_name("out")
                .default_value(r"C:\Program Files (x86)\Steam\steamapps\common\SovietRepublic\media_soviet\workshop_wip"))
            .arg(Arg::with_name("check").long("check").takes_value(false));

        let cmd_nmf = {
            let cmd_nmf_show = SubCommand::with_name("show")
                .about("Parse the specified *.nmf and print it's structure")
                .arg(Arg::with_name("nmf-path").required(true));

            let cmd_nmf_toobj = SubCommand::with_name("to-obj")
                .about("Convert the specified *.nmf to *.obj format")
                .arg(Arg::with_name("nmf-input").required(true))
                .arg(Arg::with_name("obj-output").required(true));

            let cmd_nmf_scale = SubCommand::with_name("scale")
                .about("Scale the specified *.nmf by given factor")
                .arg(Arg::with_name("nmf-input").required(true))
                .arg(Arg::with_name("factor").required(true))
                .arg(Arg::with_name("nmf-output").required(true));

            let cmd_nmf_mirror_x = SubCommand::with_name("mirror-x")
                .about("Mirror the specified *.nmf along the X-axis")
                .arg(Arg::with_name("nmf-input").required(true))
                .arg(Arg::with_name("nmf-output").required(true));

            /*
            let cmd_nmf_patch = SubCommand::with_name("patch")
                .arg(Arg::with_name("nmf-input").required(true))
                .arg(Arg::with_name("nmf-patch").required(true))
                .arg(Arg::with_name("nmf-output").required(true));*/

            SubCommand::with_name("nmf")
                .about("Operations for *.nmf files")
                .subcommand(cmd_nmf_show)
                .subcommand(cmd_nmf_toobj)
                .subcommand(cmd_nmf_scale)
                .subcommand(cmd_nmf_mirror_x)
                //.subcommand(cmd_nmf_patch)
        };

        let cmd_modbuilding = {
            let cmd_mod_validate = SubCommand::with_name("validate")
                .about("Checks the specified building mod for errors")
                .arg(Arg::with_name("dir-input").required(true));

            let cmd_modbuilding_scale = SubCommand::with_name("scale")
                .about("Not implemented (WIP)")
                .arg(Arg::with_name("dir-input").required(true))
                .arg(Arg::with_name("factor").required(true))
                .arg(Arg::with_name("dir-output").required(true));

            SubCommand::with_name("mod-building")
                .about("Operations for whole mods")
                .subcommand(cmd_mod_validate)
                .subcommand(cmd_modbuilding_scale)
        };

        let cmd_ini = {
            let cmd_ini_parsebuilding = SubCommand::with_name("parse-building")
                .about("Parse the specified building.ini, check for errors, print results")
                .arg(Arg::with_name("path").required(true));

            let cmd_ini_parserender = SubCommand::with_name("parse-renderconfig")
                .about("Parse the specified renderconfig.ini, check for errors, print results")
                .arg(Arg::with_name("path").required(true));

            let cmd_ini_parsemtl = SubCommand::with_name("parse-mtl")
                .about("Parse the specified *.mtl, check for errors, print results")
                .arg(Arg::with_name("path").required(true));

            SubCommand::with_name("ini")
                .about("Operations for individual configuration files")
                .subcommand(cmd_ini_parsebuilding)
                .subcommand(cmd_ini_parserender)
                .subcommand(cmd_ini_parsemtl)
        };

        let m = App::new("wrsr-mt")
            .author("kromgart@gmail.com")
            .version("0.3")
            .about("Modding tools for \"Workers & Resources: Soviet Rebuplic\"")
            .long_about("Modding tools for \"Workers & Resources: Soviet Rebuplic\"\n\
                         homepage: https://github.com/Kromgart/wrsr-mt")
            .arg(
                Arg::with_name("stock")
                    .long("stock")
                    .default_value(r"C:\Program Files (x86)\Steam\steamapps\common\SovietRepublic\media_soviet")
            )
            .arg(
                Arg::with_name("workshop")
                    .long("workshop")
                    .default_value(r"C:\Program Files (x86)\Steam\steamapps\workshop\content\784150")
            )
            .subcommand(cmd_install)
            .subcommand(cmd_nmf)
            .subcommand(cmd_modbuilding)
            .subcommand(cmd_ini)
            .get_matches();

        let path_stock = PathBuf::from(m.value_of("stock").unwrap());
        let path_workshop = PathBuf::from(m.value_of("workshop").unwrap());


        let command = { 
            let run_dir = std::env::current_dir().unwrap();
            let mk_path = |m: &clap::ArgMatches, p| run_dir.join(m.value_of(p).unwrap());

            match m.subcommand() {
                ("install", Some(m)) => {
                    let source = mk_path(m, "in");
                    let destination = mk_path(m, "out");
                    let is_check = m.is_present("check");

                    AppCommand::Install(InstallCommand { source, destination, is_check })
                },

                ("ini", Some(m)) => {
                    let command = match m.subcommand() {
                        ("parse-building", Some(m)) => {
                            let path = mk_path(m, "path");
                            IniCommand::ParseBuilding(path)
                        },
                        ("parse-renderconfig", Some(m)) => {
                            let path = mk_path(m, "path");
                            IniCommand::ParseRender(path)
                        },
                        ("parse-mtl", Some(m)) => {
                            let path = mk_path(m, "path");
                            IniCommand::ParseMaterial(path)
                        },
                        (cname, _) => panic!("Unknown ini subcommand '{}'" , cname)
                    };

                    AppCommand::Ini(command)
                },

                ("mod-building", Some(m)) => {
                    let command = match m.subcommand() {
                        ("validate", Some(m)) => {
                            let dir_input = mk_path(m, "dir-input");
                            ModCommand::Validate(ModValidateCommand { dir_input })
                        },
                        ("scale", Some(m)) => {
                            let dir_input = mk_path(m, "dir-input");
                            let factor = f64::from_str(m.value_of("factor").unwrap()).expect("Cannot parse scale factor as float");
                            let dir_output = mk_path(m, "dir-output");
                            ModCommand::Scale(ModScaleCommand { dir_input, factor, dir_output })
                        },
                        (cname, _) => panic!("Unknown mod subcommand '{}'" , cname)
                    };

                    AppCommand::ModBuilding(command)
                },

                ("nmf", Some(m)) => {
                    let command = match m.subcommand() {
                        ("show", Some(m)) => {
                            let path = mk_path(m, "nmf-path");
                            NmfCommand::Show(NmfShowCommand { path })
                        },

                        ("to-obj", Some(m)) => {
                            let input = mk_path(m, "nmf-input");
                            let output = mk_path(m, "obj-output");

                            NmfCommand::ToObj(NmfToObjCommand { input, output })
                        },

                        ("scale", Some(m)) => {
                            let input = mk_path(m, "nmf-input");
                            let factor = f64::from_str(m.value_of("factor").unwrap()).expect("Cannot parse scale factor as float");
                            let output = mk_path(m, "nmf-output");

                            NmfCommand::Scale(NmfScaleCommand { input, factor, output })
                        },

                        ("mirror-x", Some(m)) => {
                            let input = mk_path(m, "nmf-input");
                            let output = mk_path(m, "nmf-output");

                            NmfCommand::MirrorX(NmfMirrorXCommand { input, output })
                        },

                        ("patch", Some(m)) => {
                            let input = mk_path(m, "nmf-input");
                            let patch = mk_path(m, "nmf-patch");
                            let output = mk_path(m, "nmf-output");

                            NmfCommand::Patch(NmfPatchCommand { input, patch, output })
                        },

                        (cname, _) => panic!("Unknown nmf subcommand '{}'" , cname)
                    };

                    AppCommand::Nmf(command)
                },
                _ => {
                    eprintln!("Error: missing arguments. Run with '--help' to see usage instructions");
                    std::process::exit(1);
                }
            }
        };

        AppSettings {
            path_stock,
            path_workshop,
            command
        }
    };
}

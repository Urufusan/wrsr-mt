use std::path::PathBuf;
use std::str::FromStr;

use lazy_static::lazy_static;
use normpath::BasePathBuf;


pub const RENDERCONFIG_INI: &str = "renderconfig.ini";
pub const BUILDING_INI: &str = "building.ini";


pub enum AppCommand {
    Modpack(ModpackCommand),
    Nmf(NmfCommand),
    ModBuilding(ModCommand),
    Ini(IniCommand),
}

//-----------------------------

pub enum NmfCommand {
    Show(PathBuf),
    ToObj(NmfToObjCommand),
    Scale(ScaleCommand),
    Mirror(MirrorCommand),
}

pub struct NmfToObjCommand {
    pub input: PathBuf,
    pub output: PathBuf
}

//-------------------------------

pub enum ModCommand {
    Validate(PathBuf),
    Scale(ScaleCommand),
    Mirror(MirrorCommand),
}

//-------------------------------

pub enum IniCommand {
    ParseBuilding(PathBuf),
    ParseRender(PathBuf),
    ParseMtl(PathBuf),
    ScaleBuilding(ScaleCommand),
    ScaleRender(ScaleCommand),
    MirrorBuilding(MirrorCommand),
    MirrorRender(MirrorCommand),
}

//-------------------------------

pub enum ModpackCommand {
    Install(ModpackInstallCommand),
    Validate(PathBuf),
}

pub struct ModpackInstallCommand {
    pub source: PathBuf,
    pub destination: PathBuf,
}

//-------------------------------

pub struct MirrorCommand {
    pub input: PathBuf,
    pub output: PathBuf
}

pub struct ScaleCommand {
    pub input: PathBuf,
    pub factor: f64,
    pub output: PathBuf
}

//-------------------------------

pub struct AppSettings {
    pub path_stock: BasePathBuf,
    pub path_workshop: BasePathBuf,

    pub command: AppCommand,
}


impl AppSettings {

    // mod folder is 7 digits and cannot start from zero.
    pub const MOD_IDS_START:        usize = 1_000_000;
    pub const MOD_IDS_END:          usize = 9_999_999;
    pub const MAX_BUILDINGS_IN_MOD: usize = 100; // [0..99]
    pub const MAX_SKINS_IN_MOD:     usize = 16;

    pub const MAX_MODS:      usize = AppSettings::MOD_IDS_END - AppSettings::MOD_IDS_START;
    pub const MAX_BUILDINGS: usize = AppSettings::MAX_MODS * AppSettings::MAX_BUILDINGS_IN_MOD;

    // Paths in ini files:
    //pub const SRX_PATH_PREFIX: &'static str =  "([~.$]/)";
    //pub const SRX_PATH:        &'static str = r"([^\r\s\n]+?)";
    //pub const SRX_PATH_EXT:    &'static str =  "([^\"\\r\\n]+?)";
    //pub const SRX_EOL:         &'static str = r"(:?[\s\r\n$])";
}



lazy_static! {
    pub static ref APP_SETTINGS: AppSettings = {
        // TODO: read from configuration
        use clap::{App, Arg, SubCommand};

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

            let cmd_nmf_mirror = SubCommand::with_name("mirror")
                .about("Mirror the specified *.nmf, save to a new file")
                .arg(Arg::with_name("nmf-input").required(true))
                .arg(Arg::with_name("nmf-output").required(true));

            SubCommand::with_name("nmf")
                .about("Operations for *.nmf files")
                .subcommand(cmd_nmf_show)
                .subcommand(cmd_nmf_toobj)
                .subcommand(cmd_nmf_scale)
                .subcommand(cmd_nmf_mirror)
        };

        let cmd_modbuilding = {
            let cmd_mod_validate = SubCommand::with_name("validate")
                .about("Checks the specified building mod for errors")
                .arg(Arg::with_name("dir-input").required(true));

            let cmd_modbuilding_scale = SubCommand::with_name("scale")
                .about("Scales the whole building (models and .ini files) by the specified factor")
                .arg(Arg::with_name("dir-input").required(true))
                .arg(Arg::with_name("factor").required(true))
                .arg(Arg::with_name("dir-output").required(true));

            let cmd_modbuilding_mirror = SubCommand::with_name("mirror")
                .about("Mirrors the whole building (models and .ini files)")
                .arg(Arg::with_name("dir-input").required(true))
                .arg(Arg::with_name("dir-output").required(true));

            SubCommand::with_name("mod-building")
                .about("Operations for whole mods")
                .subcommand(cmd_mod_validate)
                .subcommand(cmd_modbuilding_scale)
                .subcommand(cmd_modbuilding_mirror)
        };

        let cmd_modpack = {
            let cmd_modpack_install = SubCommand::with_name("install")
                .about("Installs modpack from the specified source directory")
                .arg(Arg::with_name("dir-source").required(true))
                .arg(Arg::with_name("dir-destination")
                    .default_value(r"C:\Program Files (x86)\Steam\steamapps\common\SovietRepublic\media_soviet\workshop_wip"));

            let cmd_modpack_validate = SubCommand::with_name("validate")
                .about("Checks the modpack source in the specified directory for errors")
                .arg(Arg::with_name("dir-source").required(true));

            SubCommand::with_name("modpack")
                .about("Modpacks management")
                .subcommand(cmd_modpack_install)
                .subcommand(cmd_modpack_validate)
        };

        let cmd_ini = {
            let cmd_ini_parse = {
                let cmd_ini_parse_building = SubCommand::with_name("building")
                    .about("Parse the specified building.ini, check for errors, print results")
                    .arg(Arg::with_name("path").required(true));

                let cmd_ini_parse_render = SubCommand::with_name("renderconfig")
                    .about("Parse the specified renderconfig.ini, check for errors, print results")
                    .arg(Arg::with_name("path").required(true));

                let cmd_ini_parse_mtl = SubCommand::with_name("mtl")
                    .about("Parse the specified *.mtl, check for errors, print results")
                    .arg(Arg::with_name("path").required(true));

                SubCommand::with_name("parse")
                    .about("Parsing and validating *.ini and *.mtl files")
                    .subcommand(cmd_ini_parse_building)
                    .subcommand(cmd_ini_parse_render)
                    .subcommand(cmd_ini_parse_mtl)
            };

            let cmd_ini_scale = {
                let cmd_ini_scale_building = SubCommand::with_name("building")
                    .about("Parse the specified building.ini, scale by a given factor, save to a new file")
                    .arg(Arg::with_name("ini-input").required(true))
                    .arg(Arg::with_name("factor").required(true))
                    .arg(Arg::with_name("ini-output").required(true));

                let cmd_ini_scale_render = SubCommand::with_name("renderconfig")
                    .about("Parse the specified renderconfig.ini, scale by a given factor, save to a new file")
                    .arg(Arg::with_name("ini-input").required(true))
                    .arg(Arg::with_name("factor").required(true))
                    .arg(Arg::with_name("ini-output").required(true));

                SubCommand::with_name("scale")
                    .about("Scaling *.ini files")
                    .subcommand(cmd_ini_scale_building)
                    .subcommand(cmd_ini_scale_render)
            };


            let cmd_ini_mirror = {
                let cmd_ini_mirror_building = SubCommand::with_name("building")
                    .about("Parse the specified building.ini, mirror Z coordinates, save to a new file")
                    .arg(Arg::with_name("ini-input").required(true))
                    .arg(Arg::with_name("ini-output").required(true));

                let cmd_ini_mirror_render = SubCommand::with_name("renderconfig")
                    .about("Parse the specified building.ini, mirror Z coordinates, save to a new file")
                    .arg(Arg::with_name("ini-input").required(true))
                    .arg(Arg::with_name("ini-output").required(true));

                SubCommand::with_name("mirror")
                    .about("Mirroring *.ini files")
                    .subcommand(cmd_ini_mirror_building)
                    .subcommand(cmd_ini_mirror_render)
            };

            SubCommand::with_name("ini")
                .about("Operations for individual text-based files")
                .subcommand(cmd_ini_parse)
                .subcommand(cmd_ini_scale)
                .subcommand(cmd_ini_mirror)
        };

        let m = App::new("wrsr-mt")
            .author("kromgart@gmail.com")
            .version("0.4")
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
            .subcommand(cmd_nmf)
            .subcommand(cmd_modbuilding)
            .subcommand(cmd_ini)
            .subcommand(cmd_modpack)
            .get_matches();

        let path_stock    = BasePathBuf::new(m.value_of("stock").unwrap()).unwrap();
        let path_workshop = BasePathBuf::new(m.value_of("workshop").unwrap()).unwrap();

        let command = { 
            use normpath::BasePathBuf;
            let run_dir = BasePathBuf::try_new(std::env::current_dir().unwrap()).unwrap();
            let mk_path = |m: &clap::ArgMatches, p| run_dir.join(m.value_of(p).unwrap()).into_path_buf();

            let mk_scale = |m, p_in, p_out| -> ScaleCommand {
                let input = mk_path(m, p_in);
                let factor = f64::from_str(m.value_of("factor").unwrap()).expect("Cannot parse scale factor as float");
                let output = mk_path(m, p_out);
                assert!(input != output, "{} and {} cannot be the same", p_in, p_out);
                ScaleCommand { input, factor, output }
            };
            
            let mk_mirror = |m, p_in, p_out| -> MirrorCommand {
                let input = mk_path(m, p_in);
                let output = mk_path(m, p_out);
                assert!(input != output, "{} and {} cannot be the same", p_in, p_out);
                MirrorCommand { input, output }
            };

            match m.subcommand() {
                ("modpack", Some(m)) => AppCommand::Modpack(match m.subcommand() {
                    ("install", Some(m)) => {
                        let source = mk_path(m, "dir-source");
                        let destination = mk_path(m, "dir-destination");
                        ModpackCommand::Install(ModpackInstallCommand { source, destination })
                    },
                    ("validate", Some(m)) => ModpackCommand::Validate(mk_path(m, "dir-source")),
                    (cname, _)            => panic!("Unknown modpack subcommand '{}'", cname)
                }),

                ("ini", Some(m)) => AppCommand::Ini( match m.subcommand() {
                    ("parse", Some(m)) => match m.subcommand() {
                        ("building",     Some(m)) => IniCommand::ParseBuilding(mk_path(m, "path")),
                        ("renderconfig", Some(m)) => IniCommand::ParseRender(mk_path(m, "path")),
                        ("mtl",          Some(m)) => IniCommand::ParseMtl(mk_path(m, "path")),
                        (cname, _)                => panic!("Unknown ini parse subcommand '{}'" , cname)
                    },
                    ("scale", Some(m)) => match m.subcommand() {
                        ("building", Some(m))     => IniCommand::ScaleBuilding(mk_scale(m, "ini-input", "ini-output")),
                        ("renderconfig", Some(m)) => IniCommand::ScaleRender(mk_scale(m, "ini-input", "ini-output")),
                        (cname, _)                => panic!("Unknown ini scale subcommand '{}'" , cname)
                    },
                    ("mirror", Some(m)) => match m.subcommand() {
                        ("building", Some(m))     => IniCommand::MirrorBuilding(mk_mirror(m, "ini-input", "ini-output")),
                        ("renderconfig", Some(m)) => IniCommand::MirrorRender(mk_mirror(m, "ini-input", "ini-output")),
                        (cname, _)                => panic!("Unknown ini mirror subcommand '{}'" , cname)
                    },
                    (cname, _) => panic!("Unknown ini subcommand '{}'" , cname)
                }),

                ("mod-building", Some(m)) => AppCommand::ModBuilding(match m.subcommand() {
                    ("validate", Some(m)) => ModCommand::Validate(mk_path(m, "dir-input")),
                    ("scale", Some(m))    => ModCommand::Scale(mk_scale(m, "dir-input", "dir-output")),
                    ("mirror", Some(m))   => ModCommand::Mirror(mk_mirror(m, "dir-input", "dir-output")),
                    (cname, _)            => panic!("Unknown mod subcommand '{}'" , cname)
                }),

                ("nmf", Some(m)) => AppCommand::Nmf(match m.subcommand() {
                    ("show", Some(m)) => NmfCommand::Show(mk_path(m, "nmf-path")),
                    ("to-obj", Some(m)) => {
                        let input = mk_path(m, "nmf-input");
                        let output = mk_path(m, "obj-output");
                        assert!(input != output, "input and output cannot be the same");
                        NmfCommand::ToObj(NmfToObjCommand { input, output })
                    },
                    ("scale", Some(m))  => NmfCommand::Scale(mk_scale(m, "nmf-input", "nmf-output")),
                    ("mirror", Some(m)) => NmfCommand::Mirror(mk_mirror(m, "nmf-input", "nmf-output")),
                    (cname, _)          => panic!("Unknown nmf subcommand '{}'" , cname)
                }),

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

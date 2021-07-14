use std::path::PathBuf;

use lazy_static::lazy_static;



pub enum AppCommand {
    Install(InstallCommand),
    Nmf(NmfCommand)
}

pub struct InstallCommand {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_check: bool
}

pub enum NmfCommand {
    Show(NmfShowCommand),
    Patch(NmfPatchCommand)
}

pub struct NmfShowCommand {
    pub path: PathBuf,
    pub with_patch: Option<PathBuf>
}

pub struct NmfPatchCommand {
    pub path: PathBuf,
    pub patch: PathBuf,
    pub in_place: bool
}

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
            .arg(Arg::with_name("in").required(true))
            .arg(Arg::with_name("out")
                .default_value(r"C:\Program Files (x86)\Steam\steamapps\common\SovietRepublic\media_soviet\workshop_wip"))
            .arg(Arg::with_name("check").long("check").takes_value(false));

        let m = App::new("wrsr-mt")
            .author("kromgart@gmail.com")
            .version("0.1")
            .about("Modding tools for \"Workers & Resources: Soviet Rebuplic\"")
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
            .get_matches();

        let path_stock = PathBuf::from(m.value_of("stock").unwrap());
        let path_workshop = PathBuf::from(m.value_of("workshop").unwrap());


        let command = { 
            let mut src = PathBuf::from(std::env::current_dir().unwrap());
            src.push("pack");

            match m.subcommand() {
                ("install", Some(m)) => {
                    let run_dir = std::env::current_dir().unwrap();
                    let source = run_dir.join(m.value_of("in").unwrap());
                    let destination = run_dir.join(m.value_of("out").unwrap());
                    let is_check = m.is_present("check");

                    AppCommand::Install(InstallCommand {
                        source,
                        destination,
                        is_check
                    })
                },
                (cname, _) => panic!("Unknown command '{}'" , cname)
            }
        };

        AppSettings {
            path_stock,
            path_workshop,
            command
        }
    };
}

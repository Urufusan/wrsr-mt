use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use regex::Regex;

mod cfg;
mod data;
mod input;
mod output;
mod nmf;

use cfg::{APP_SETTINGS, AppCommand, InstallCommand};




fn main() {
/*
    let test = fs::read_to_string(r"z:\wrsr-mg\pack\7L\model.patch").unwrap();
    let res = ModelPatch::from(&test);

    println!("{}", res);

    return;
*/

    match &APP_SETTINGS.command {
        AppCommand::Install(InstallCommand{ source, destination, is_check }) => {

            println!("Installing from source: {}", source.to_str().unwrap());
            assert!(source.exists(), "Pack source directory does not exist!");

            println!("Installing to:          {}", destination.to_str().unwrap());
            assert!(destination.exists(), "Destination directory does not exist.");
            
            println!("Stock game files:       {}", APP_SETTINGS.path_stock.to_str().unwrap());
            assert!(APP_SETTINGS.path_stock.exists(), "Stock game files directory does not exist.");

            println!("Workshop directory:     {}", APP_SETTINGS.path_workshop.to_str().unwrap());
            assert!(APP_SETTINGS.path_workshop.exists(), "Workshop directory does not exist.");


            let mut pathbuf: PathBuf = APP_SETTINGS.path_stock.clone();
            pathbuf.push("buildings");
            pathbuf.push("buildingtypes.ini");

            let stock_buildings_ini = fs::read_to_string(&pathbuf).unwrap();
            let mut stock_buildings = { 
                let mut mp = HashMap::with_capacity(512);
                let rx = Regex::new(r"\$TYPE ([_[:alnum:]]+?)\r\n((?s).+?\n END\r\n)").unwrap();

                for caps in rx.captures_iter(&stock_buildings_ini) {
                    let key = caps.get(1).unwrap().as_str();
                    mp.insert(
                        key, 
                        (key, data::StockBuilding::Unparsed(caps.get(2).unwrap().as_str()))
                    );
                }
                
                mp
            };

            println!("Found {} stock buildings", stock_buildings.len());

            pathbuf.push(source);
            println!("Reading sources...");
            let data = input::read_validate_sources(pathbuf.as_path(), &mut stock_buildings);
            println!("Sources verified.");

            if *is_check {
                println!("Check complete.");
            } else {
                println!("Creating mods...");
                pathbuf.push(destination);

                output::generate_mods(pathbuf.as_path(), data);
            }
        },


        AppCommand::Nmf(_) => {
        }
    };
}

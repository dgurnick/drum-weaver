extern crate pretty_env_logger;
#[macro_use] extern crate log;

use clap::{Arg, ArgMatches};

mod player;
use player::run_cli;

mod common;
use common::PlayerArguments;

mod ui;
use ui::run_ui;

fn main() {

    pretty_env_logger::init();
    info!("Starting the eDrums Wav Player");

    let matches = clap::Command::new("eDrums Wav Player")
        .version("0.1")
        .arg(Arg::new("music_folder").long("music_folder").required(true).help("Where your music files are stored"))
        .arg(Arg::new("track").long("track").required(true).help("Position in the csv file to play"))
        .arg(Arg::new("track_volume").long("track_volume").required(false).default_value("100"))
        .arg(Arg::new("click_volume").long("click_volume").required(false).default_value("100"))
        .arg(Arg::new("track_device").long("track_device").required(false).default_value("1"))
        .arg(Arg::new("click_device").long("click_device").required(false).default_value("1"))
        .arg(Arg::new("combined").long("combined").required(false).default_value("1"))
        .arg(Arg::new("ui").long("ui").required(false).default_value("1"))
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

/// Runs the program as determined by the main function
fn run(matches: &ArgMatches) -> Result<i32, String> {

    // Parse the arguments
    let music_folder = matches.get_one::<String>("music_folder").expect("No folder provided");
    
    let track_position = matches.get_one::<String>("track")
        .unwrap_or(&"1.0".to_string())
        .parse::<usize>()
        .unwrap_or(1);

   let track_volume = matches.get_one::<String>("track_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let click_volume = matches.get_one::<String>("click_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let track_device_position = matches.get_one::<String>("track_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let click_device_position = matches.get_one::<String>("click_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let combined = matches.get_one::<String>("combined")
        .unwrap_or(&"1".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let ui = matches
        .get_one::<String>("ui")
        .map(|value| value == "1")
        .unwrap_or(false);

    let arguments = PlayerArguments {  
        music_folder: music_folder.to_string(),
        track_position: track_position,
        track_volume: track_volume,
        click_volume: click_volume,
        track_device_position: track_device_position,
        click_device_position: click_device_position,
        combined: combined, 
    };

    if ui {
        match run_ui(arguments) {
            Ok(_) => {},
            Err(err) => return Err(format!("Could not start the ui. {}", err)),
        }
    } else {
        match run_cli(arguments) {
            Ok(_) => {},
            Err(err) => return Err(format!("Could not run the cli: {}", err)),
        }
   }

    Ok(0)

}

use std::path::Path;

use clap::{Arg, ArgMatches};
use log::{info, LevelFilter};

use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::{trigger::size::SizeTrigger, CompoundPolicy};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

mod common;
use common::{dump_devices, PlayerArguments};
mod playlist;

mod ui;
use ui::App;

mod songlist;

use crate::songlist::import_songs;
mod audio;

fn main() {
    init_logging();

    let matches = clap::Command::new("eDrums Wav Player")
        .version("0.1")
        .arg(
            Arg::new("music_folder")
                .long("music_folder")
                .required(false)
                .help("Where your music files are stored"),
        )
        .arg(
            Arg::new("track")
                .long("track")
                .required(false)
                .help("Position in the csv file to play"),
        )
        .arg(
            Arg::new("track_volume")
                .long("track_volume")
                .required(false)
                .default_value("100"),
        )
        .arg(
            Arg::new("click_volume")
                .long("click_volume")
                .required(false)
                .default_value("100"),
        )
        .arg(
            Arg::new("track_device")
                .long("track_device")
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::new("click_device")
                .long("click_device")
                .required(false)
                .default_value("0"),
        )
        .arg(Arg::new("ui").long("ui").required(false).default_value("1"))
        .arg(
            Arg::new("playback_speed")
                .long("playback_speed")
                .required(false)
                .default_value("1"),
        )
        .arg(
            Arg::new("print_devices")
                .long("print_devices")
                .required(false)
                .default_value("0"),
        )
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

fn init_logging() {
    if let Err(_err) = log4rs::init_file("logging_config.yml", Default::default()) {
        // Create the file appender
        let file_appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}",
            )))
            .build(
                "log/my.log",
                Box::new(CompoundPolicy::new(
                    Box::new(SizeTrigger::new(50 * 1024)), // 50 KB
                    Box::new(DeleteRoller::new()),
                )),
            )
            .unwrap();

        // Create the root logger
        let root = Root::builder().appender("file").build(LevelFilter::Trace);

        // Create the configuration with the file appender and root logger
        let config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file_appender)))
            .build(root)
            .unwrap();

        // Initialize the logger with the configuration
        log4rs::init_config(config).unwrap();
    };
}

/// Runs the program as determined by the main function
fn run(matches: &ArgMatches) -> Result<i32, String> {
    // Parse the arguments
    let music_folder = matches
        .get_one::<String>("music_folder")
        .map(|folder| folder.to_owned());

    if music_folder.is_none() {
        info!("No music folder specified, will prompt for one");
    } else if !Path::new(&music_folder.as_ref().unwrap()).exists() {
        return Err(format!("Music folder does not exist."));
    }

    let track_position = matches
        .get_one::<String>("track")
        .unwrap_or(&"1.0".to_string())
        .parse::<usize>()
        .unwrap_or(1);
    info!("Initializing with track at position: {}", track_position);

    let track_volume = matches
        .get_one::<String>("track_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0)
        / 100.0;
    info!("Initializing with track volume: {}", track_volume);

    let click_volume = matches
        .get_one::<String>("click_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0)
        / 100.0;
    info!("Initializing with click volume: {}", click_volume);

    let track_device = matches
        .get_one::<String>("track_device")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
        .unwrap_or(0);
    info!("Initializing with track device: {}", track_device);

    let click_device = matches
        .get_one::<String>("click_device")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
        .unwrap_or(0);
    info!("Initializing with click device: {}", click_device);

    let ui = matches
        .get_one::<String>("ui")
        .map(|value| value == "1")
        .unwrap_or(true);
    info!("Initializing with UI option: {}", ui);

    let print_devices = matches
        .get_one::<String>("print_devices")
        .map(|value| value == "1")
        .unwrap_or(false);
    info!(
        "Initializing with option to print devices: {}",
        print_devices
    );

    let playback_speed = matches
        .get_one::<String>("playback_speed")
        .unwrap_or(&"1.0".to_string())
        .parse::<f64>()
        .unwrap_or(100.0);
    info!("Initializing with playback speed: {}", playback_speed);

    if print_devices {
        dump_devices();
        return Ok(0);
    }

    // make sure the file exists. If it doesn't try to find the compressed version and decompress it.
    //let (track_file, click_file) = get_file_paths(music_folder, track_position);

    let arguments = PlayerArguments {
        music_folder: music_folder,
        track_song: None,
        click_song: None,
        track_volume: track_volume,
        click_volume: click_volume,
        track_device_position: track_device,
        click_device_position: click_device,
        playback_speed: playback_speed,
    };

    match import_songs() {
        Ok(songs) => {
            let mut app = App::new(arguments, songs);
            match app.run_ui() {
                Ok(_) => {}
                Err(err) => return Err(format!("Could not start the ui. {}", err)),
            }
        }
        Err(err) => return Err(format!("Could not import songs: {}", err)),
    };
    Ok(0)
}

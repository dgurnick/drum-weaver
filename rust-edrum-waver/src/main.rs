use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    fs,
    io,
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clap::{Arg, ArgMatches};
use csv;
use rodio::{
    self, 
    cpal, 
    cpal::traits::HostTrait,
    Decoder,
    Source,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
    Terminal,
};
use lazy_static::lazy_static;

mod common;
use common::Song;

mod player;
use player::play_combined;
use player::play_separate;

mod util;
use util::get_file_paths;



fn main() {

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

    let (track_path_str, click_path_str) = match get_file_paths(music_folder, track_position) {
        Ok((track_path_str, click_path_str)) => (track_path_str, click_path_str),
        Err(err) => return Err(err),
    };
    
    println!("Playing track: {}", track_path_str);
    println!("Playing click: {}", click_path_str);

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

    if ui {
        setupUi(); 
    } else {
        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
        if available_devices.is_empty() {
            return Err("No output devices found".to_string());
        }

        // Check if the device positions are valid
        let num_devices = available_devices.len();
        if track_device_position > num_devices {
            return Err("Invalid track output device".to_string());
        }
        if click_device_position > num_devices {
            return Err("Invalid click output device".to_string());
        }

        let track = fs::File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
        let click = fs::File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

        let track_source = Decoder::new(io::BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
        let click_source = Decoder::new(io::BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;
        let track_source_amplify = track_source.amplify(track_volume);
        let click_source_amplify = click_source.amplify(click_volume);

        if combined {
            match play_combined(
                track_source_amplify, 
                click_source_amplify, 
                &available_devices[track_device_position]
            ) {
                Ok(_) => {},
                Err(err) => return Err(err),
            }
        
        } else {
            match play_separate(
                track_source_amplify, 
                click_source_amplify, 
                &available_devices[track_device_position],
                &available_devices[click_device_position],
                track_volume,
                click_volume
            ) {
                Ok(_) => {},
                Err(err) => return Err(err),

            }
        }

    }

    Ok(0)

}










#[derive(Serialize, Deserialize, Clone)]
struct Playlist {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}


enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
    #[error("error parsing the CsV file: {0}")]
    CsvError(#[from] csv::Error),
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Playlists,
    Songs,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Playlists => 1,
            MenuItem::Songs => 2,
        }
    }
}

// see: https://github.com/zupzup/rust-commandline-example/blob/main/src/main.rs

       



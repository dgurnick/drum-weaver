use crate::playlist::SongRecord;
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use std::fs::metadata;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerArguments {
    pub music_folder: Option<String>,
    pub track_song: Option<String>,
    pub click_song: Option<String>,
    pub track_volume: f32,
    pub click_volume: f32,
    pub track_device_position: usize,
    pub click_device_position: usize,
    pub playback_speed: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused_variables)]
pub struct DeviceDetail {
    pub name: String,
    pub position: usize,
}

pub enum Event<I> {
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
pub enum MenuItem {
    Songs,
    Devices,
    Help,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Songs => 1,
            MenuItem::Devices => 2,
            MenuItem::Help => 3,
        }
    }
}

pub fn get_file_paths(
    music_folder: &String,
    song_position: usize,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path("assets/song_list.csv").unwrap();
    let _ = reader.headers();

    let mut position = 1;
    #[allow(unused_assignments)]
    let mut track_path_str: String = String::new();
    #[allow(unused_assignments)]
    let mut click_path_str: String = String::new();

    for record in reader.deserialize() {
        let song: SongRecord = record.unwrap();

        if position == song_position {
            let mut track_path = PathBuf::new();
            track_path.push(music_folder);
            track_path.push(&song.folder);
            track_path.push(format!("{}.wav", song.file_name));

            let mut click_path = PathBuf::new();
            click_path.push(music_folder);
            click_path.push(&song.folder);
            click_path.push(format!("{}_click.wav", song.file_name));

            if !track_path.exists() {
                // if there's a 7z file with the same name, decompress it
                //let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
                let mut archive_path = PathBuf::new();
                archive_path.push(music_folder);
                archive_path.push(song.folder.clone());
                archive_path.push(format!("{}.7z", song.file_name));

                let mut output_folder = PathBuf::new();
                output_folder.push(music_folder);
                output_folder.push(song.folder);

                let result = sevenz_rust::decompress_file(&archive_path, output_folder);
                match result {
                    Ok(_) => {
                        info!("Decompressed file: {}", archive_path.display());
                    }
                    Err(_) => {
                        info!("Failed to decompress file: {}", archive_path.display());
                    }
                }
            }

            track_path_str = track_path.display().to_string();
            click_path_str = click_path.display().to_string();
            info!("Will play track from file: {}", track_path_str);
            info!("Will play click from file: {}", click_path_str);

            break;
        } else {
            position += 1;
        }
    }

    // throw an error if the file does not exist
    if !std::fs::metadata(track_path_str.clone()).is_ok() {
        Err(format!("Track file does not exist: {}", track_path_str).into())
    } else if !std::fs::metadata(click_path_str.clone()).is_ok() {
        Err(format!("Click file does not exist: {}", click_path_str).into())
    } else {
        Ok((track_path_str, click_path_str))
    }
}

#[allow(dead_code)]
fn check_file_existence(folder_path: String, file_name: String) -> Result<(), String> {
    let mut path = PathBuf::new();
    path.push(folder_path);
    path.push(file_name);

    if let Err(_) = metadata(&path) {
        return Err(format!("File '{}' does not exist", path.display()));
    }
    Ok(())
}

lazy_static! {
    static ref DEVICES: Mutex<Vec<DeviceDetail>> = Mutex::new(Vec::new());
}

pub fn read_devices() -> Result<Vec<DeviceDetail>, Error> {
    let mut devices = DEVICES.lock().unwrap();

    if devices.is_empty() {
        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

        for (position, device) in available_devices.iter().enumerate() {
            let detail = DeviceDetail {
                name: device.name().unwrap(),
                position: position,
            };
            devices.push(detail);
        }
    }

    Ok(devices.clone())
}

pub fn dump_devices() {
    let devices = read_devices().unwrap();

    println!("Available devices:");
    for device in devices.iter() {
        println!(
            "Position: {} | Description: {}",
            device.position, device.name
        );
    }
}

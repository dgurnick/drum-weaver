use crate::playlist::SongRecord;
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use std::fs::metadata;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

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

pub enum UiEvent<I> {
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

pub fn needs_unzipping(
    music_folder: &String,
    song_title: &String,
    artist_name: &String,
    album_name: &String,
) -> bool {
    for song_record in read_song_file().unwrap().iter() {
        let song: SongRecord = song_record.clone();

        if &song.album == album_name && &song.artist == artist_name && &song.title == song_title {
            let (track_path, _) = get_track_and_click_path(music_folder, &song.folder, &song.title);

            if !track_path.exists() {
                return true;
            } else {
                return true;
            }
        }
    }

    return false;
}

lazy_static! {
    static ref SONGS: Mutex<Vec<SongRecord>> = Mutex::new(Vec::new());
}

// To have global immutable access to the file.
const CSV_CONTENT: &'static [u8] = include_bytes!("../assets/song_list.csv");

fn read_song_file() -> Result<Vec<SongRecord>, Error> {
    let mut songs = SONGS.lock().unwrap();

    if songs.is_empty() {
        let mut reader = csv::Reader::from_reader(CSV_CONTENT);
        let _ = reader.headers();

        for record in reader.deserialize() {
            let song: SongRecord = record.unwrap();
            songs.push(song);
        }
    }

    Ok(songs.clone())
}

fn get_track_and_click_path(
    music_folder: &String,
    song_folder: &String,
    song_name: &String,
) -> (PathBuf, PathBuf) {
    let mut track_path = PathBuf::new();
    track_path.push(music_folder);
    track_path.push(song_folder);
    track_path.push(format!("{}.wav", song_name));

    let mut click_path = PathBuf::new();
    click_path.push(music_folder);
    click_path.push(song_folder);
    click_path.push(format!("{}_click.wav", song_name));

    (track_path, click_path)
}

pub fn get_file_paths(
    music_folder: &String,
    song_name: &String,
    artist_name: &String,
    album_name: &String,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    #[allow(unused_assignments)]
    let mut track_path_str: String = String::new();
    #[allow(unused_assignments)]
    let mut click_path_str: String = String::new();

    for record in read_song_file()?.iter() {
        let song: SongRecord = record.clone();

        if &song.album == album_name && &song.artist == artist_name && &song.title == song_name {
            let (track_path, click_path) =
                get_track_and_click_path(music_folder, &song.folder, &song.file_name);

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

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs::metadata;
use std::path::PathBuf;
use thiserror::Error;
use std::fs;
use std::io;
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::lib::{Player, Song};

use cpal::traits::{DeviceTrait, HostTrait};



pub struct PlayerArguments {
    pub music_folder: String,
    pub track_song: String,
    pub click_song: String,
    pub track_volume: f32,
    pub click_volume: f32,
    pub track_device_position: usize,
    pub click_device_position: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused_variables)]
pub struct SongRecord {
    pub file_name: String,
    #[allow(dead_code)] pub genre: String,
    #[allow(dead_code)] pub year: String, 
    #[allow(dead_code)] pub artist: String,
    #[allow(dead_code)] pub song: String,
    #[allow(dead_code)] pub album: String,
    #[allow(dead_code)] pub length: String,
    #[allow(dead_code)] pub bpm: usize,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: usize,
    pub name: String,
    pub category: String,
    pub age: usize,
    pub created_at: DateTime<Utc>,
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

pub fn get_file_paths(music_folder: &String, song_position: usize) -> (String, String) {

    let mut reader = csv::Reader::from_path("assets/song_list.csv").unwrap();
    let _ = reader.headers();

    let mut position = 1;
    let mut track_path_str: String = String::new();
    let mut click_path_str: String = String::new();

    for record in reader.deserialize() {

        let song: SongRecord = record.unwrap();

        if position == song_position {
            let mut track_path  = PathBuf::new();
            track_path.push(music_folder);
            track_path.push(&song.folder);
            track_path.push(format!("{}.wav", song.file_name));

            let mut click_path  = PathBuf::new();
            click_path.push(music_folder);
            click_path.push(&song.folder);
            click_path.push(format!("{}_click.wav", song.file_name));

            if ! track_path.exists() {
                // if there's a 7z file with the same name, decompress it 
                 //let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
                 let mut archive_path = PathBuf::new();
                 archive_path.push(music_folder);
                 archive_path.push(song.folder.clone());
                 archive_path.push(format!("{}.7z", song.file_name));

                 let mut output_folder = PathBuf::new();
                 output_folder.push(music_folder);
                 output_folder.push(song.folder);

                 sevenz_rust::decompress_file(&archive_path, output_folder).expect("Failed to decompress file");

            } 

            track_path_str = track_path.display().to_string();
            click_path_str = click_path.display().to_string();
            return (track_path_str, click_path_str);

        } else {
            position += 1;
        }
    }

    Err("Could not find song".to_string()).unwrap()

}

fn check_file_existence(folder_path: String, file_name: String) -> Result<(), String> {
    let mut path = PathBuf::new();
    path.push(folder_path);
    path.push(file_name);

    if let Err(_) = metadata(&path) {
        return Err(format!("File '{}' does not exist", path.display()));
    }
    Ok(())
}

pub fn read_playlists() -> Result<Vec<Playlist>, Error> {
    let db_content = fs::read_to_string("assets/playlists.json")?;
    let parsed: Vec<Playlist> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

lazy_static! {
    static ref SONGS: Mutex<Vec<SongRecord>> = Mutex::new(Vec::new());
}

pub fn read_songs() -> Result<Vec<SongRecord>, Error> {

    let mut songs = SONGS.lock().unwrap();

    if songs.is_empty() {
        let db_content = fs::read_to_string("assets/song_list.csv")?;
        let mut reader = csv::Reader::from_reader(db_content.as_bytes());

        for result in reader.deserialize() {
            let song: SongRecord = result?;
            songs.push(song);
        }
    }

    Ok(songs.clone())

}

pub fn play_song(arguments: PlayerArguments) -> Result<(Player, Player), String> {
    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    let track_device = &available_devices[arguments.track_device_position];
    let click_device = &available_devices[arguments.track_device_position];

    let track_play = Player::new(None, track_device).expect("Could not create track player");
    let click_play = Player::new(None, click_device).expect("Could not create click player");

    let track_volume = Some(arguments.track_volume);
    let click_volume = Some(arguments.click_volume);

    let track_song = Song::from_file(arguments.track_song, track_volume).expect("Could not create track song");
    let click_song = Song::from_file(arguments.click_song, click_volume).expect("Could not create click song");

    track_play.play_song_now(&track_song, None).expect("Could not play track song");
    click_play.play_song_now(&click_song, None).expect("Could not play click song");

    Ok((track_play, click_play))
    
    // while track_play.has_current_song() && click_play.has_current_song() {
    //     std::thread::sleep(std::time::Duration::from_secs(1));

    //     // let (track_samples, track_position) = track_play.get_playback_position().expect("Could not get track playback position");
    //     // let (click_samples, click_position) = click_play.get_playback_position().expect("Could not get click playback position");

    //     // println!("Track: {}/{} Click: {}/{}", 
    //     //     track_position.as_secs(), 
    //     //     track_samples.as_secs(), 
    //     //     click_position.as_secs(), 
    //     //     click_samples.as_secs());
        
    // }

}

pub fn dump_devices() {
    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    println!("Available devices:");
    for (position, device) in available_devices.iter().enumerate() {
    if let Ok(name) = device.name() {
        println!("Position: {} | Description: {}", position, name);
    } else {
        println!("Position: {} | Description: Unknown", position);
    }
}
}

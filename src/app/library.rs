use std::path::PathBuf;

use log::info;
use serde::{Deserialize, Serialize};

use super::player::SongStub;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SongRecord {
    pub file_name: String,
    pub genre: String,
    pub year: String,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub length: String,
    pub bpm: usize,
    pub folder: String,
}

#[derive(Clone)]
pub struct Library {
    pub path: String, // the root path of the library (i.e. ...\Drumless)
    pub songs: Vec<SongRecord>,
}

impl Library {
    pub fn new(path: String) -> Self {
        Library { path, songs: Vec::new() }
    }

    // Reload the library from the embedded CSV file
    pub fn load_csv(&mut self) {
        let file_contents: &str = include_str!("../../assets/song_list.csv");
        let mut reader = csv::Reader::from_reader(file_contents.as_bytes());
        self.songs.clear();

        let base_path = self.path.clone();

        for result in reader.deserialize() {
            let mut song: SongRecord = result.unwrap();
            let mut song_path = PathBuf::from(base_path.clone());
            song_path.push(&song.folder);
            song.folder = song_path.display().to_string();
            self.songs.push(song);
        }

        info!("Loaded {} songs from CSV", self.songs.len());
    }

    pub fn remove_song_by_stub(&mut self, stub: SongStub) {
        if let Some(index) = self.songs.iter().position(|song| song.file_name == stub.file_name) {
            self.songs.remove(index);
        }
    }
}

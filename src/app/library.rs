use std::{path::PathBuf, sync::Mutex};

use log::info;
use serde::{Deserialize, Serialize};

use super::player::SongStub;
use lazy_static::lazy_static;

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
    songs: Vec<SongRecord>,
    original_songs: Vec<SongRecord>,
}

lazy_static! {
    static ref SONGS: Mutex<Vec<SongRecord>> = Mutex::new(Vec::new());
}

impl Library {
    pub fn new(path: String) -> Self {
        Library {
            path,
            songs: Vec::new(),
            original_songs: Vec::new(),
        }
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

        self.original_songs = self.songs.clone();

        info!("Loaded {} songs from CSV", self.songs.len());
    }

    pub fn remove_song_by_stub(&mut self, stub: SongStub) {
        if let Some(index) = self.songs.iter().position(|song| song.file_name == stub.file_name) {
            self.songs.remove(index);
        }
    }

    pub fn get_songs(&self) -> &Vec<SongRecord> {
        &self.songs
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.songs.shuffle(&mut rng);
    }

    pub fn search(&mut self, query: &str) {
        self.songs = self.original_songs.clone();

        let terms: Vec<&str> = query.split('!').collect();

        let include_term = terms[0].trim().to_lowercase();
        let exclude_terms: Vec<String> = terms[1..].iter().map(|term| term.trim().to_lowercase()).collect();

        self.songs.retain(|song| {
            (song.title.to_lowercase().contains(&include_term) || song.artist.to_lowercase().contains(&include_term) || song.genre.to_lowercase().contains(&include_term))
                && exclude_terms
                    .iter()
                    .all(|term| !song.title.to_lowercase().contains(term) && !song.artist.to_lowercase().contains(term) && !song.genre.to_lowercase().contains(term))
        });

        // self.songs.retain(|song| {
        //     song.title.to_lowercase().contains(&query.to_lowercase()) || song.artist.to_lowercase().contains(&query.to_lowercase()) || song.genre.to_lowercase().contains(&query.to_lowercase())
        // });
    }

    pub fn reset(&mut self) {
        self.songs = self.original_songs.clone();
    }
}

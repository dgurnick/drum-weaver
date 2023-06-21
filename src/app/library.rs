use log::info;
use serde::{Deserialize, Serialize};

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

        for result in reader.deserialize() {
            let song: SongRecord = result.unwrap();
            self.songs.push(song);
        }

        info!("Loaded {} songs from CSV", self.songs.len());
    }
}

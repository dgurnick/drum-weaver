use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    music_folder: PathBuf,
    items: Vec<LibraryItem>,
}

impl Library {
    pub fn new(music_folder: PathBuf) -> Self {
        Self {
            music_folder,
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: LibraryItem) {
        self.items.push(item);
    }

    pub fn music_folder(&self) -> PathBuf {
        self.music_folder.clone()
    }

    pub fn items(&self) -> Vec<LibraryItem> {
        self.items.clone()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    #[serde(skip)]
    key: usize,

    #[serde(skip)]
    track_path: PathBuf,

    #[serde(skip)]
    click_path: PathBuf,

    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
    track_number: Option<u32>,
}

impl LibraryItem {
    pub fn key(&self) -> usize {
        self.key
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn artist(&self) -> Option<String> {
        self.artist.clone()
    }

    pub fn album(&self) -> Option<String> {
        self.album.clone()
    }

    pub fn year(&self) -> Option<i32> {
        self.year.clone()
    }

    pub fn genre(&self) -> Option<String> {
        self.genre.clone()
    }

    pub fn track_number(&self) -> Option<u32> {
        self.track_number.clone()
    }
}

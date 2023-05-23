use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    root_path: PathBuf,
    items: Vec<LibraryItem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    key: usize,
    track_path: PathBuf,
    click_path: PathBuf,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
    track_number: Option<u32>,
}

impl Library {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path: root_path,
            items: Vec::new(),
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    pub fn items(&self) -> Vec<LibraryItem> {
        self.items.clone()
    }
}

use crate::app::LibraryItem;
use crate::AudioCommand;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    name: Option<String>,
    pub tracks: Vec<LibraryItem>,
    pub selected: Option<LibraryItem>,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            name: None,
            tracks: vec![],
            selected: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn get_name(&mut self) -> Option<String> {
        self.name.clone()
    }

    pub fn add(&mut self, track: LibraryItem) {
        self.tracks.push(track);
    }

    pub fn remove(&mut self, idx: usize) {
        self.tracks.remove(idx);
    }

    pub fn select(&mut self, idx: usize, audio_cmd_tx: &Sender<AudioCommand>) {
        let track = self.tracks[idx].clone();
        audio_cmd_tx
            .send(AudioCommand::PlayTrack(track.clone()))
            .expect("Failed to send track selection to the audio thread");
    }

    pub fn get_position(&self, track: &LibraryItem) -> Option<usize> {
        self.tracks.iter().position(|t| t == track)
    }
}

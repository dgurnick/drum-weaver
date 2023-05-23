use library::{Library, LibraryItem};
use player::Player;
use playlist::Playlist;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

mod app;
mod components;
mod library;
pub mod player;
mod playlist;

#[derive(Clone, Debug)]
pub enum TempError {
    MissingAppState,
}

pub enum AudioCommand {
    Stop,
    Play,
    Pause,
    Seek(u32), // Maybe this should represent a duration?
    PlayTrack(LibraryItem),
    Select(usize),
    SetVolume(f32),
}

#[derive(Deserialize, Serialize)]
pub struct App {
    pub library: Option<Library>,

    #[serde(skip_serializing, skip_deserializing)]
    pub player: Option<Player>,

    #[serde(skip_serializing, skip_deserializing)]
    pub quit: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub library_sender: Option<Sender<Library>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub library_receiver: Option<Receiver<Library>>,

    pub playlists: Vec<Playlist>,

    pub current_playlist: Option<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: None,
            player: None,
            quit: false,
            library_sender: None,
            library_receiver: None,
            playlists: vec![],
            current_playlist: None,
        }
    }
}

impl App {
    pub fn load() -> Result<Self, TempError> {
        confy::load("drummer", None).map_err(|_| TempError::MissingAppState)
    }

    pub fn save_state(&self) {
        let store_result = confy::store("drummer", None, &self);
        match store_result {
            Ok(_) => tracing::info!("Stored application state"),
            Err(err) => tracing::error!("Failed to store application state: {}", err),
        }
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }
}

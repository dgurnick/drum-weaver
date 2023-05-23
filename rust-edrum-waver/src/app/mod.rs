use library::{Library, LibraryItem};
use player::Player;
use serde::{Deserialize, Serialize};

mod app;
mod library;
pub mod player;

#[derive(Clone, Debug)]
pub enum TempError {
    MissingAppState,
}

pub enum AudioCommand {
    Stop,
    Play,
    Pause,
    Seek(u32), // Maybe this should represent a duration?
    LoadFile(std::path::PathBuf),
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
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: None,
            player: None,
            quit: false,
        }
    }
}

impl App {
    pub fn load() -> Result<Self, TempError> {
        confy::load("drummer", None).map_err(|_| TempError::MissingAppState)
    }
}

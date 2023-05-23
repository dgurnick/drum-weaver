use crate::app::library::LibraryItem;
use crate::AudioCommand;
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub enum TrackState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    pub track_state: TrackState,
    pub selected_track: Option<LibraryItem>,
    pub audio_tx: Sender<AudioCommand>,
    pub volume: f32,
    pub cursor: Arc<AtomicU32>,
}

use crate::app::library::LibraryItem;
use crate::app::playlist::Playlist;
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
    pub seek_in_seconds: u32,
    pub cursor: Arc<AtomicU32>,
}

impl Player {
    pub fn new(audio_cmd_tx: Sender<AudioCommand>, cursor: Arc<AtomicU32>) -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            audio_tx: audio_cmd_tx,
            volume: 1.0,
            seek_in_seconds: 0,
            cursor,
        }
    }

    pub fn select_track(&mut self, track: Option<LibraryItem>) {
        self.selected_track = track;
        if let Some(track) = &self.selected_track {
            self.audio_tx
                .send(AudioCommand::PlayTrack(track.clone()))
                .expect("Failed to send play request to audio thread");
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.track_state {
            TrackState::Stopped => true,
            _ => false,
        }
    }

    pub fn seek_to(&mut self, seconds: u32) {
        self.seek_in_seconds = seconds;
        self.audio_tx
            .send(AudioCommand::Seek(seconds))
            .expect("Failed to send seek to the audio thread");
    }

    pub fn stop(&mut self) {
        match &self.track_state {
            TrackState::Playing | TrackState::Paused => {
                self.track_state = TrackState::Stopped;
                self.audio_tx
                    .send(AudioCommand::Stop)
                    .expect("Failed to send stop to the audio thread");
            }
            _ => (),
        }
    }

    pub fn play(&mut self) {
        if let Some(_selected_trac) = &self.selected_track {
            match self.track_state {
                TrackState::Unstarted | TrackState::Stopped | TrackState::Playing => {
                    self.track_state = TrackState::Playing;
                    self.audio_tx
                        .send(AudioCommand::Play)
                        .expect("Failed to send play command to the audio thread");
                }
                TrackState::Paused => {
                    self.track_state = TrackState::Playing;
                    self.audio_tx
                        .send(AudioCommand::Play)
                        .expect("Failed to send play command to the audio thread")
                }
            }
        }
    }

    pub fn pause(&mut self) {
        match self.track_state {
            TrackState::Playing => {
                self.track_state = TrackState::Paused;
                self.audio_tx
                    .send(AudioCommand::Pause)
                    .expect("Failed to send pause command to the audio thread");
            }
            TrackState::Paused => {
                self.track_state = TrackState::Playing;
                self.audio_tx
                    .send(AudioCommand::Play)
                    .expect("Failed to send play command to the audio thread");
            }
            _ => (),
        }
    }

    pub fn previous(&mut self, playlist: &Playlist) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_position(&selected_track) {
                if current_track_position > 0 {
                    let previous_track = &playlist.tracks[current_track_position - 1];
                    self.select_track(Some((*previous_track).clone()));
                    self.play();
                }
            }
        }
    }

    pub fn next(&mut self, playlist: &Playlist) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_position(&selected_track) {
                if current_track_position < playlist.tracks.len() - 1 {
                    let next_track = &playlist.tracks[current_track_position + 1];
                    self.select_track(Some((*next_track).clone()));
                    self.play();
                }
            }
        }
    }
}

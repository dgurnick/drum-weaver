pub mod audio;
pub mod commands;
pub mod devices;
pub mod events;
pub mod library;
pub mod player;
pub mod render;
pub mod setup;

use crate::app::render::UiRenderTrait;
use crate::app::setup::UiSetupTrait;

use std::{
    io::{stdout, Stdout},
    thread,
    time::Duration,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::{
    event::EnableMouseCapture,
    event::KeyEvent,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, SetTitle},
    ExecutableCommand,
};

use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};

use self::{
    events::UiEventTrait,
    library::{Library, SongRecord},
    player::{PlaybackStatus, PlayerCommand, PlayerEvent},
};

#[derive(PartialEq)]
pub enum ActiveFocus {
    Library,
    Queue,
}

#[derive(PartialEq)]
pub enum PlayerStatus {
    Ready,
    Playing(String),
    Paused,
    Stopped,
    Decompressing,
    Decompressed,
    Ended,
    Error(String),
}

impl PlayerStatus {
    pub fn as_string(&self) -> String {
        match self {
            PlayerStatus::Ready => "Ready".to_string(),
            PlayerStatus::Playing(s) => format!("Playing: {}", s),
            PlayerStatus::Paused => "Paused".to_string(),
            PlayerStatus::Stopped => "Stopped".to_string(),
            PlayerStatus::Decompressing => "Decompressing".to_string(),
            PlayerStatus::Decompressed => "Decompressed".to_string(),
            PlayerStatus::Ended => "Ended".to_string(),
            PlayerStatus::Error(s) => format!("Error loading song: {}", s),
        }
    }
}

#[derive(Clone)]
pub struct Track {
    main_file: String,
    click_file: String,
}

pub struct App {
    pub player_command_sender: Sender<PlayerCommand>,
    pub player_event_receiver: Receiver<PlayerEvent>,
    pub ui_command_receiver: Receiver<UiEvent<KeyEvent>>,
    pub ui_command_sender: Sender<UiEvent<KeyEvent>>,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub library: Option<Library>,
    pub queue: Vec<SongRecord>,
    pub is_running: bool,
    pub active_menu_item: MenuItem,
    pub active_focus: ActiveFocus,
    pub library_state: TableState,
    pub queue_state: TableState,
    pub device_state: TableState,
    pub active_track: Option<Track>,
    pub is_exiting: bool,
    pub playback_status: Option<PlaybackStatus>,
    pub player_status: PlayerStatus,
    pub track_device_idx: usize,
    pub click_device_idx: usize,
    pub track_volume: usize,
    pub click_volume: usize,
}

pub enum UiEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Library,
    Devices,
    Help,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Library => 1,
            MenuItem::Devices => 2,
            MenuItem::Help => 3,
        }
    }
}

impl App {
    pub fn new(player_command_sender: Sender<PlayerCommand>, player_event_receiver: Receiver<PlayerEvent>) -> Self {
        // Set up the terminal
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("Unable to setup terminal");
        enable_raw_mode().expect("Unable to setup terminal in raw mode");

        let mut backend = CrosstermBackend::new(stdout);
        backend.execute(SetTitle("Drum Weaver")).expect("Unable to set terminal title");

        let mut terminal = Terminal::new(backend).expect("Unable to create terminal");
        terminal.clear().expect("Error clearing terminal");
        terminal.hide_cursor().expect("Error hiding cursor");

        // set up the signals for ui commands
        let (ui_command_sender, ui_command_receiver) = unbounded();

        App {
            player_command_sender,
            player_event_receiver,
            ui_command_sender,
            ui_command_receiver,
            terminal,
            is_running: true,
            library: None,
            active_menu_item: MenuItem::Library,
            active_focus: ActiveFocus::Library,
            library_state: TableState::default(),
            queue_state: TableState::default(),
            device_state: TableState::default(),
            queue: Vec::new(),
            active_track: None,
            is_exiting: false,
            playback_status: None,
            player_status: PlayerStatus::Ready,
            track_device_idx: 0,
            click_device_idx: 0,
            track_volume: 100,
            click_volume: 100,
        }
    }

    pub fn run(&mut self) {
        self.setup_ui_signal_loop();
        self.setup_library();

        match self.queue_state.selected() {
            Some(index) => index,
            None => {
                self.queue_state.select(Some(0));
                0
            }
        };

        match self.library_state.selected() {
            Some(index) => index,
            None => {
                self.library_state.select(Some(0));
                0
            }
        };

        match self.device_state.selected() {
            Some(index) => index,
            None => {
                self.device_state.select(Some(0));
                0
            }
        };

        let player_command_sender_clone = self.player_command_sender.clone();

        // listen for position updates
        thread::spawn(move || loop {
            player_command_sender_clone.send(PlayerCommand::GetStatus).unwrap();
            thread::sleep(Duration::from_millis(1000));
        });

        while self.is_running {
            // Wait for the user to pick a folder
            if self.library.is_none() {
                continue;
            };

            self.handle_ui_events();
            self.handle_player_events();
            self.render_ui();
        }
    }

    pub fn get_songs(&self) -> Vec<SongRecord> {
        match &self.library {
            Some(library) => library.songs.clone(),
            None => vec![],
        }
    }
}

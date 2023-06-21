pub mod audio;
pub mod commands;
pub mod events;
pub mod library;
pub mod player;
pub mod render;
pub mod setup;

use crate::app::render::UiRenderTrait;
use crate::app::setup::UiSetupTrait;

use std::io::{stdout, Stdout};

use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::{
    event::EnableMouseCapture,
    event::KeyEvent,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, SetTitle},
    ExecutableCommand,
};

use ratatui::{backend::CrosstermBackend, Terminal};

use self::{
    events::UiEventTrait,
    library::Library,
    player::{PlayerCommand, PlayerEvent},
};

pub struct App {
    pub player_command_sender: Sender<PlayerCommand>,
    pub player_event_receiver: Receiver<PlayerEvent>,
    pub ui_command_receiver: Receiver<UiEvent<KeyEvent>>,
    pub ui_command_sender: Sender<UiEvent<KeyEvent>>,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub library: Option<Library>,
    pub is_running: bool,
}

pub enum UiEvent<I> {
    Input(I),
    Tick,
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
        }
    }

    pub fn run(&mut self) {
        self.setup_exit_handler();
        self.setup_ui_signal_loop();
        self.setup_library();

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
}

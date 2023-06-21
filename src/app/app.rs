use crate::app::commands::UiCommandTrait;

use std::{
    io::{stdout, Stdout},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::enable_raw_mode,
};
use log::info;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Terminal,
};

use super::player::{PlayerCommand, PlayerEvent};

pub struct App {
    pub player_command_sender: Sender<PlayerCommand>,
    pub player_event_receiver: Receiver<PlayerEvent>,
    pub ui_command_receiver: Receiver<UiEvent<KeyEvent>>,
    pub ui_command_sender: Sender<UiEvent<KeyEvent>>,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

pub enum UiEvent<I> {
    Input(I),
    Tick,
}

impl App {
    pub fn new(player_command_sender: Sender<PlayerCommand>, player_event_receiver: Receiver<PlayerEvent>) -> Self {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Unable to create terminal");
        terminal.clear().expect("Error clearing terminal");

        let (ui_command_sender, ui_command_receiver) = unbounded();

        App {
            player_command_sender,
            player_event_receiver,
            ui_command_sender,
            ui_command_receiver,
            terminal,
        }
    }

    fn setup_exit_handler(&mut self) {
        let player_command_sender = Arc::new(Mutex::new(self.player_command_sender.clone()));

        // Set up the Ctrl+C signal handler.
        // We send a quit event to ensure clean shutdown
        ctrlc::set_handler(move || {
            println!("BYE!");
            let player_command_sender = player_command_sender.lock().unwrap();
            player_command_sender.send(PlayerCommand::Quit).unwrap();
        })
        .expect("Error setting Ctrl+C handler");
    }

    fn setup_signal_loop(&mut self) {
        let tick_rate = Duration::from_millis(200);
        let ui_command_sender = Arc::new(Mutex::new(self.ui_command_sender.clone()));

        // create our transmit-receive loop
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
                let ui_command_sender = ui_command_sender.lock().unwrap();

                if event::poll(timeout).expect("Polling works") {
                    if let CrosstermEvent::Key(key) = event::read().expect("can read events") {
                        ui_command_sender.send(UiEvent::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate && ui_command_sender.send(UiEvent::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.setup_exit_handler();
        self.setup_signal_loop();

        loop {
            if let Ok(event) = self.ui_command_receiver.try_recv() {
                match event {
                    UiEvent::Input(input) => match input.code {
                        KeyCode::Char('q') => {
                            self.player_command_sender.send(PlayerCommand::Quit).unwrap();
                        }
                        _ => {}
                    },
                    UiEvent::Tick => {}
                }
            }
            // handle signals from the player
            if let Ok(event) = self.player_event_receiver.try_recv() {
                match event {
                    PlayerEvent::Decompressing => {
                        info!("App received Decompressing signal");
                    }
                    PlayerEvent::Decompressed => {
                        info!("App received Decompressed signal");
                    }
                    PlayerEvent::Playing(file_name) => {
                        info!("App received Playing signal: {}", file_name);
                        self.player_command_sender.send(PlayerCommand::Pause).unwrap();
                    }
                    PlayerEvent::Paused => {
                        info!("App received Paused signal");
                    }
                    PlayerEvent::Stopped => {
                        info!("App received Stopped signal");
                    }
                    PlayerEvent::Quit => {
                        info!("App received Quit signal. Exiting.");
                        break;
                    }
                    PlayerEvent::LoadFailure(file_name) => {
                        info!("App received LoadFailure: {}", file_name);
                        // TODO: Remove song from list and queue
                        self.player_command_sender.send(PlayerCommand::Play("test2.mp3".to_string())).unwrap();
                    }
                }
            }

            // render UI elements
            self.terminal
                .draw(|frame| {
                    let size = frame.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(3), Constraint::Length(1)].as_ref())
                        .split(size);

                    let block = Block::default().title("Block").borders(Borders::ALL);
                    frame.render_widget(block, size);
                })
                .expect("Unable to draw UI");
        }
    }
}

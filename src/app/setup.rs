use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};

use crate::app::{library::Library, player::PlayerCommand};

use super::{App, UiEvent};

pub trait UiSetupTrait {
    fn setup_exit_handler(&mut self);
    fn setup_ui_signal_loop(&mut self);
    fn setup_library(&mut self);
}

impl UiSetupTrait for App {
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

    fn setup_ui_signal_loop(&mut self) {
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
                        if key.kind == KeyEventKind::Release {
                            ui_command_sender.send(UiEvent::Input(key)).expect("can send events");
                        }
                    }
                }

                if last_tick.elapsed() >= tick_rate && ui_command_sender.send(UiEvent::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            }
        });
    }

    fn setup_library(&mut self) {
        use native_dialog::FileDialog;
        let path = FileDialog::new().show_open_single_dir();

        match path {
            Ok(path) => {
                if let Some(p) = path {
                    let library = Library::new(p.display().to_string());
                    self.library = Some(library);
                } else {
                    std::process::exit(0);
                }
            }
            Err(_e) => {
                std::process::exit(0);
            }
        };
    }
}

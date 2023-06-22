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
    fn init_library(&mut self, path: String);
}

impl UiSetupTrait for App {
    // We intercept ctrl+c and send a quit event to ensure clean shutdown
    fn setup_exit_handler(&mut self) {
        let player_command_sender = Arc::new(Mutex::new(self.player_command_sender.clone()));

        ctrlc::set_handler(move || {
            println!("BYE!");
            let player_command_sender = player_command_sender.lock().unwrap();
            player_command_sender.send(PlayerCommand::Quit).unwrap();
        })
        .expect("Error setting Ctrl+C handler");
    }

    // We set up a thread to handle UI events
    fn setup_ui_signal_loop(&mut self) {
        let tick_rate = Duration::from_millis(200);
        let ui_command_sender = Arc::new(Mutex::new(self.ui_command_sender.clone()));

        // clear the event queue
        let cleared = event::poll(Duration::from_millis(0));
        match cleared {
            Ok(true) => {
                let _ = event::read();
            }
            _ => {}
        }

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

    // We set up the library that includes the root folder path for all songs as well as the songs themselves
    fn setup_library(&mut self) {
        use native_dialog::FileDialog;
        let path = FileDialog::new().show_open_single_dir();

        match path {
            Ok(path) => {
                if let Some(p) = path {
                    self.init_library(p.display().to_string());
                } else {
                    std::process::exit(0);
                }
            }
            Err(_e) => {
                std::process::exit(0);
            }
        };
    }

    // Builds the actual library
    fn init_library(&mut self, path: String) {
        let mut library = Library::new(path);
        library.load_csv();
        self.library = Some(library);
    }
}

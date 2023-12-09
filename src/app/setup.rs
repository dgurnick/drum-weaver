use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CrosstermEvent, MouseButton, MouseEventKind};
use log::info;

use crate::app::library::Library;

use super::{App, InputEvent, UiEvent};

pub trait UiSetupTrait {
    fn setup_ui_signal_loop(&mut self);
    fn setup_library(&mut self);
    fn init_library(&mut self, path: String);
}

impl UiSetupTrait for App {
    // We intercept ctrl+c and send a quit event to ensure clean shutdown
    // We set up a thread to handle UI events
    fn setup_ui_signal_loop(&mut self) {
        let tick_rate = Duration::from_millis(500);
        let ui_command_sender = Arc::new(Mutex::new(self.ui_command_sender.clone()));

        // clear the event queue
        let cleared = event::poll(Duration::from_millis(0));
        if let Ok(true) = cleared {
            let _ = event::read();
        }

        // create our transmit-receive loop
        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(1));
                let ui_command_sender = ui_command_sender.lock().unwrap();

                if event::poll(timeout).expect("Polling works") {
                    match event::read().expect("can read events") {
                        CrosstermEvent::Key(key) => {
                            //if key.kind == KeyEventKind::Release {
                                info!("Key released: {:?}", key);
                                ui_command_sender.send(UiEvent::Input(InputEvent::Key(key))).expect("can send events");
                            //} else {
                            //    info!("Key not released: {:?}", key);
                            //}
                        }
                        CrosstermEvent::Mouse(me) => {
                            if me.kind == MouseEventKind::Up(MouseButton::Left) {
                                ui_command_sender.send(UiEvent::Input(InputEvent::Mouse(me))).expect("can send events");
                            }
                        }
                        // handle other types of events if needed
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate && ui_command_sender.send(UiEvent::Input(InputEvent::Tick)).is_ok() {
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
                    info!("Selected folder: {}", p.display());
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

use crossterm::event::KeyCode;
use log::{error, info};

use crate::app::player::PlayerCommand;

use super::{commands::UiCommandTrait, player::PlayerEvent, App, UiEvent};

pub trait UiEventTrait {
    fn handle_ui_events(&mut self);
    fn handle_player_events(&mut self);
    fn send_player_command(&mut self, command: PlayerCommand);
}

impl UiEventTrait for App {
    fn handle_ui_events(&mut self) {
        // Handle UI events
        // This comes first since we want any interaction to
        // change state as a priority

        if !self.is_exiting {
            if let Ok(event) = self.ui_command_receiver.try_recv() {
                match event {
                    UiEvent::Input(input) => match input.code {
                        KeyCode::Char('q') => self.do_exit(),
                        KeyCode::Char(' ') => self.do_pause(),
                        KeyCode::Enter => self.do_playback(),
                        KeyCode::Down => self.do_next(),
                        KeyCode::Up => self.do_previous(),
                        KeyCode::Tab => self.do_tab(),
                        _ => {}
                    },
                    UiEvent::Tick => {}
                }
            }
        }
    }

    fn handle_player_events(&mut self) {
        // handle signals from the player
        if let Ok(event) = self.player_event_receiver.try_recv() {
            match event {
                PlayerEvent::Decompressing => {
                    info!("App received Decompressing signal");
                    self.player_status = "Decompressing".to_string();
                }
                PlayerEvent::Decompressed => {
                    info!("App received Decompressed signal");
                    self.player_status = "Decompressed".to_string();
                }
                PlayerEvent::Playing(stub) => {
                    info!("App received Playing signal: {}", stub.title);
                    self.player_status = format!("Playing: {}", stub.title);
                }
                PlayerEvent::Paused => {
                    info!("App received Paused signal");
                    self.player_status = "Paused".to_string();
                }
                PlayerEvent::Continuing(stub) => {
                    self.player_status = format!("Playing: {}", stub.unwrap().title);
                }
                PlayerEvent::Stopped => {
                    info!("App received Stopped signal");
                }
                PlayerEvent::Quit => {
                    info!("App received Quit signal. Exiting.");
                    self.on_exit();
                    self.is_running = false;
                }
                PlayerEvent::LoadFailure(stub) => {
                    error!("App received LoadFailure: {}", stub.title);
                    // TODO: Remove song from list and queue
                }
                PlayerEvent::Position(optional_position) => {
                    self.current_position = optional_position;
                }
            }
        }
    }

    fn send_player_command(&mut self, command: PlayerCommand) {
        info!("Sending player command: {:?}", command);
        self.player_command_sender.send(command).unwrap();
        info!("Sent player command");
    }
}

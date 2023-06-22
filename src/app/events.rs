use crossterm::event::{KeyCode, KeyModifiers};
use log::{error, info};

use crate::app::{player::PlayerCommand, PlayerStatus};

use super::{commands::UiCommandTrait, player::PlayerEvent, App, MenuItem, UiEvent};

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
                    UiEvent::Input(event) if event.modifiers.contains(KeyModifiers::SHIFT) => match event.code {
                        KeyCode::Left => self.do_slowdown(),
                        KeyCode::Right => self.do_speedup(),
                        _ => {}
                    },
                    UiEvent::Input(input) => match input.code {
                        KeyCode::Char('s') => self.active_menu_item = MenuItem::Songs,
                        KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                        KeyCode::Char('q') => self.do_exit(),
                        KeyCode::Char(' ') => self.do_pause(),
                        KeyCode::Enter => self.do_playback(),
                        KeyCode::Down => self.do_next(),
                        KeyCode::Up => self.do_previous(),
                        KeyCode::Tab => self.do_tab(),
                        KeyCode::Left => self.do_backward(),
                        KeyCode::Right => self.do_forward(),
                        _ => {}
                    },
                    UiEvent::Tick => {
                        //self.do_autoplay();
                    }
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
                    self.player_status = PlayerStatus::Decompressing;
                }
                PlayerEvent::Decompressed => {
                    info!("App received Decompressed signal");
                    self.player_status = PlayerStatus::Decompressed;
                }
                PlayerEvent::Playing(stub) => {
                    info!("App received Playing signal: {}", stub.title);
                    self.player_status = PlayerStatus::Playing(stub.title);
                }
                PlayerEvent::Paused => {
                    info!("App received Paused signal");
                    self.player_status = PlayerStatus::Paused;
                }
                PlayerEvent::Continuing(stub) => {
                    self.player_status = PlayerStatus::Playing(stub.unwrap().title);
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
                PlayerEvent::Ended => {
                    self.player_status = PlayerStatus::Ended;
                    self.do_autoplay();
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

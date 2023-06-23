use crossterm::event::{KeyCode, KeyModifiers};
use log::{error, info};

use crate::app::{player::PlayerCommand, PlayerStatus};

use super::{
    commands::UiCommandTrait,
    player::{DeviceType, PlayerEvent},
    App, MenuItem, UiEvent,
};

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

                    // Commands for the library/queue
                    UiEvent::Input(event) if self.active_menu_item == MenuItem::Library => match event.code {
                        KeyCode::Char('s') => self.active_menu_item = MenuItem::Library,
                        KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                        KeyCode::Char('q') => self.do_exit(),
                        KeyCode::Char('r') => self.do_reset_speed(),
                        KeyCode::Char('1') => self.do_decrease_volume(DeviceType::Track),
                        KeyCode::Char('2') => self.do_reset_volume(DeviceType::Track),
                        KeyCode::Char('3') => self.do_increase_volume(DeviceType::Track),
                        KeyCode::Char('4') => self.do_decrease_volume(DeviceType::Click),
                        KeyCode::Char('5') => self.do_reset_volume(DeviceType::Click),
                        KeyCode::Char('6') => self.do_increase_volume(DeviceType::Click),
                        KeyCode::Char(' ') => self.do_pause(),
                        KeyCode::Char('n') => self.do_play_next(),
                        KeyCode::Char('x') => self.do_shuffle_library(),
                        KeyCode::Delete => self.do_delete_queue(),
                        KeyCode::Insert => self.do_insert_queue(),
                        KeyCode::Enter => self.do_playback(),
                        KeyCode::Down => self.do_select_next(),
                        KeyCode::Up => self.do_select_previous(),
                        KeyCode::Tab => self.do_tab(),
                        KeyCode::Left => self.do_backward(),
                        KeyCode::Right => self.do_forward(),
                        _ => {}
                    },

                    // Commands for the device view
                    UiEvent::Input(event) if self.active_menu_item == MenuItem::Devices => match event.code {
                        KeyCode::Char('s') => self.active_menu_item = MenuItem::Library,
                        KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                        KeyCode::Char('q') => self.do_exit(),
                        KeyCode::Char(' ') => self.do_pause(),
                        KeyCode::Down => self.do_next_device(),
                        KeyCode::Up => self.do_previous_device(),
                        KeyCode::Char('t') => self.do_set_device(DeviceType::Track),
                        KeyCode::Char('c') => self.do_set_device(DeviceType::Click),
                        _ => {}
                    },

                    UiEvent::Tick => {
                        //self.do_autoplay();
                    }
                    _ => {}
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
                    let stub_clone = stub.clone();
                    info!("App received Playing signal: {}", stub.title);
                    self.player_status = PlayerStatus::Playing(stub.title);
                    self.active_stub = Some(stub_clone);
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
                    self.player_status = PlayerStatus::Error(stub.title);
                }
                PlayerEvent::Status(status) => {
                    self.playback_status = Some(status);
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

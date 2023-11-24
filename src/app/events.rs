use crossterm::event::{KeyCode, KeyModifiers};
use log::error;

use crate::app::{player::PlayerCommand, PlayerStatus};

use super::{
    commands::UiCommandTrait,
    player::{DeviceType, PlayerEvent},
    App, InputEvent, MenuItem, UiEvent,
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
                if matches!(self.player_status, PlayerStatus::Waiting | PlayerStatus::Decompressing | PlayerStatus::Decompressed) {
                    return;
                }
                match event {
                    UiEvent::Input(input_event) => {
                        match input_event {
                            // Commands for the help view
                            InputEvent::Key(event) if event.modifiers.contains(KeyModifiers::ALT) && self.active_menu_item == MenuItem::Library && self.is_searching => {
                                if event.code == KeyCode::Char('r') {
                                    self.do_replace_queue()
                                }
                            }

                            InputEvent::Key(event) if event.modifiers.contains(KeyModifiers::SHIFT) && !self.is_searching => match event.code {
                                KeyCode::Char('s') => self.active_menu_item = MenuItem::Library,
                                KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                                KeyCode::Char('h') => self.active_menu_item = MenuItem::Help,
                                KeyCode::Char('q') => self.do_exit(),
                                KeyCode::Left => self.do_slowdown(),
                                KeyCode::Right => self.do_speedup(),
                                _ => {}
                            },

                            // Commands for searching
                            InputEvent::Key(event) if self.active_menu_item == MenuItem::Library && self.is_searching => match event.code {
                                KeyCode::Char(char) => {
                                    self.search_query.push(char);
                                    self.do_search();
                                }
                                KeyCode::Backspace => {
                                    self.search_query.pop();
                                    self.do_search();
                                }
                                KeyCode::Esc => {
                                    self.is_searching = false;
                                    self.search_query.clear();
                                    self.do_cancel_search();
                                }
                                KeyCode::Enter => {
                                    self.is_searching = false;
                                    self.do_complete_search();
                                }
                                KeyCode::Tab => {
                                    self.is_searching = false;
                                    self.do_empty_queue();
                                    self.do_complete_search();
                                }
                                _ => {}
                            },

                            // Commands for the device view
                            InputEvent::Key(event) if self.active_menu_item == MenuItem::Help => match event.code {
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

                            // Commands for the device view
                            InputEvent::Key(event) if self.active_menu_item == MenuItem::Devices => match event.code {
                                KeyCode::Char('s') => self.active_menu_item = MenuItem::Library,
                                KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                                KeyCode::Char('h') => self.active_menu_item = MenuItem::Help,
                                KeyCode::Char('q') => self.do_exit(),
                                KeyCode::Char(' ') => self.do_pause(),
                                KeyCode::Down => self.do_next_device(),
                                KeyCode::Up => self.do_previous_device(),
                                KeyCode::Char('t') => self.do_set_device(DeviceType::Track),
                                KeyCode::Char('c') => self.do_set_device(DeviceType::Click),
                                _ => {}
                            },

                            // Commands for the library/queue
                            InputEvent::Key(event) if self.active_menu_item == MenuItem::Library => match event.code {
                                KeyCode::Char('s') => self.active_menu_item = MenuItem::Library,
                                KeyCode::Char('d') => self.active_menu_item = MenuItem::Devices,
                                KeyCode::Char('h') => self.active_menu_item = MenuItem::Help,
                                KeyCode::Char('q') => self.do_exit(),
                                KeyCode::Char('r') => self.do_reset_speed(),
                                KeyCode::Char('a') => self.do_set_repeat(),
                                KeyCode::Char('z') => self.do_restart_song(),
                                KeyCode::Char('1') => self.do_decrease_volume(DeviceType::Track),
                                KeyCode::Char('2') => self.do_reset_volume(DeviceType::Track),
                                KeyCode::Char('3') => self.do_increase_volume(DeviceType::Track),
                                KeyCode::Char('4') => self.do_decrease_volume(DeviceType::Click),
                                KeyCode::Char('5') => self.do_reset_volume(DeviceType::Click),
                                KeyCode::Char('6') => self.do_increase_volume(DeviceType::Click),
                                KeyCode::Char('7') => self.do_decrease_volume(DeviceType::Bleed),
                                KeyCode::Char('8') => self.do_reset_volume(DeviceType::Bleed),
                                KeyCode::Char('9') => self.do_increase_volume(DeviceType::Bleed),
                                KeyCode::Char('g') => self.do_start_search(),
                                KeyCode::Char(' ') => self.do_pause(),
                                KeyCode::Char('n') => self.do_play_next(),
                                KeyCode::Char('x') => self.do_shuffle_library(),
                                KeyCode::Char('/') => self.do_empty_queue(),
                                KeyCode::Delete => self.do_delete_queue(),
                                KeyCode::Insert => self.do_insert_queue(),
                                KeyCode::Enter => self.do_playback(),
                                KeyCode::Down => self.do_select_next(),
                                KeyCode::Up => self.do_select_previous(),
                                KeyCode::Tab => self.do_tab(),
                                KeyCode::Left => self.do_backward(),
                                KeyCode::Right => self.do_forward(),
                                KeyCode::Home => self.do_goto_first(),
                                KeyCode::End => self.do_goto_last(),
                                KeyCode::PageDown => self.do_page_down(),
                                KeyCode::PageUp => self.do_page_up(),
                                _ => {}
                            },
                            InputEvent::Key(_) => {} // ignorable
                            InputEvent::Mouse(_) => {
                                // TODO: Option to scroll through library or queue
                                // TODO: Double click on item to play it
                                // All that requires tracking rows and pages
                                // which has little benefit compared to the effort
                                // Should rethink proper UI (egui or some webview)
                            }
                            InputEvent::Tick => {}
                        }
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
                    self.player_status = PlayerStatus::Decompressing;
                }
                PlayerEvent::Decompressed => {
                    self.player_status = PlayerStatus::Decompressed;
                }
                PlayerEvent::Playing(stub) => {
                    let stub_clone = stub.clone();
                    self.player_status = PlayerStatus::Playing(stub.title);
                    self.active_stub = Some(stub_clone);
                }
                PlayerEvent::Paused => {
                    self.player_status = PlayerStatus::Paused;
                }
                PlayerEvent::Continuing(stub) => {
                    self.player_status = PlayerStatus::Playing(stub.unwrap().title);
                }
                PlayerEvent::Quit => {
                    self.on_exit();
                    self.is_running = false;
                }
                PlayerEvent::LoadFailure(stub) => {
                    error!("App received LoadFailure: {}", stub.title);
                    self.library.as_mut().unwrap().remove_song_by_stub(stub.clone());
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
        self.player_command_sender.send(command).unwrap();
    }
}

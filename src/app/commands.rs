use std::io;

use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use native_dialog::{MessageDialog, MessageType};

use super::{
    devices::read_devices,
    events::UiEventTrait,
    player::{DeviceType, PlayerCommand, SongStub},
    ActiveFocus, App, AppConfig, MenuItem, PlayerStatus,
};

pub trait UiCommandTrait {
    fn do_exit(&mut self);
    fn on_exit(&mut self);
    fn do_pause(&mut self);
    fn do_playback(&mut self);
    fn do_select_next(&mut self);
    fn do_select_previous(&mut self);
    fn do_tab(&mut self);
    fn do_autoplay(&mut self);
    fn do_forward(&mut self);
    fn do_backward(&mut self);
    fn do_speedup(&mut self);
    fn do_slowdown(&mut self);
    fn do_reset_speed(&mut self);
    fn do_next_device(&mut self);
    fn do_previous_device(&mut self);
    fn do_set_device(&mut self, device_type: DeviceType);
    fn do_increase_volume(&mut self, device_type: DeviceType);
    fn do_decrease_volume(&mut self, device_type: DeviceType);
    fn do_reset_volume(&mut self, device_type: DeviceType);
    fn do_shuffle_library(&mut self);
    fn do_play_next(&mut self);
    fn do_delete_queue(&mut self);
    fn do_insert_queue(&mut self);
    fn do_empty_queue(&mut self);
    fn do_goto_first(&mut self);
    fn do_goto_last(&mut self);
    fn do_page_down(&mut self);
    fn do_page_up(&mut self);
    fn do_start_search(&mut self);
    fn do_search(&mut self);
    fn do_complete_search(&mut self);
    fn do_cancel_search(&mut self);
    fn do_replace_queue(&mut self);
    fn do_restart_song(&mut self);
    fn do_set_repeat(&mut self);
}

impl UiCommandTrait for App {
    fn do_exit(&mut self) {
        self.is_exiting = true;
        let dialog_result = MessageDialog::new().set_title("Confirm exit").set_text("Are you sure?").set_type(MessageType::Info).show_confirm();

        match dialog_result {
            Ok(true) => {
                self.send_player_command(PlayerCommand::Quit);
            }
            _ => {
                self.is_exiting = false;
            }
        }
    }

    fn on_exit(&mut self) {
        let track_device_name = read_devices()[self.track_device_idx].name.clone();
        let click_device_name = read_devices()[self.click_device_idx].name.clone();

        let config = AppConfig {
            track_device_name: Some(track_device_name),
            click_device_name: Some(click_device_name),
            track_volume: Some(self.track_volume),
            click_volume: Some(self.click_volume),
            bleed_volume: Some(self.bleed_volume),
            search_query: Some(self.search_query.clone()),
            queue: self.queue.clone(),
        };
        confy::store("drum-weaver", None, config).expect("Unable to save configuration");

        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        std::process::exit(0);
    }

    fn do_pause(&mut self) {
        self.send_player_command(PlayerCommand::Pause);
    }

    fn do_playback(&mut self) {
        self.player_status = PlayerStatus::Waiting;
        match self.active_focus {
            ActiveFocus::Queue => {
                let idx = self.queue_state.selected().unwrap_or(0);
                let song = self.queue[idx].clone();

                self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));
            }
            ActiveFocus::Library => {
                let idx = self.library_state.selected().unwrap_or(0);
                let song = self.library.as_ref().unwrap().get_songs()[idx].clone();

                self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));
            }
        }
    }

    fn do_select_next(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {
                let mut idx = self.queue_state.selected().unwrap_or(0);
                idx += 1;
                if idx > self.queue.len() - 1 {
                    idx = 0;
                }
                self.queue_state.select(Some(idx));
            }
            ActiveFocus::Library => {
                let mut idx = self.library_state.selected().unwrap_or(0);
                idx += 1;
                if idx > self.library.as_ref().unwrap().get_songs().len() - 1 {
                    idx = 0;
                }
                self.library_state.select(Some(idx));
            }
        }
    }

    fn do_select_previous(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {
                let mut idx = self.queue_state.selected().unwrap_or(0) as i32;
                idx -= 1;
                if idx < 0 {
                    idx = (self.queue.len() - 1) as i32;
                }
                self.queue_state.select(Some(idx as usize));
            }
            ActiveFocus::Library => {
                let mut idx = self.library_state.selected().unwrap_or(0) as i32;
                idx -= 1;
                if idx < 0 {
                    idx = (self.library.as_ref().unwrap().get_songs().len() - 1) as i32;
                }
                self.library_state.select(Some(idx as usize));
            }
        }
    }

    fn do_tab(&mut self) {
        if self.active_focus == ActiveFocus::Library {
            self.active_focus = ActiveFocus::Queue;
        } else {
            self.active_focus = ActiveFocus::Library;
        }
    }

    fn do_autoplay(&mut self) {
        if self.is_repeating && self.active_stub.is_some() {
            self.send_player_command(PlayerCommand::Play(self.active_stub.clone().unwrap()));
        } else {
            self.do_play_next()
        }
    }

    fn do_forward(&mut self) {
        self.send_player_command(PlayerCommand::Forward);
    }

    fn do_backward(&mut self) {
        self.send_player_command(PlayerCommand::Backward);
    }

    fn do_speedup(&mut self) {
        self.send_player_command(PlayerCommand::SpeedUp);
    }

    fn do_slowdown(&mut self) {
        self.send_player_command(PlayerCommand::SlowDown);
    }

    fn do_next_device(&mut self) {
        let mut idx = self.device_state.selected().unwrap_or(0);
        let devices = read_devices();
        idx += 1;
        if idx > devices.len() - 1 {
            idx = devices.len() - 1;
        }
        self.device_state.select(Some(idx));
    }

    fn do_previous_device(&mut self) {
        let mut idx = self.device_state.selected().unwrap_or(0);
        if idx == 0 {
            return;
        }
        idx -= 1;
        self.device_state.select(Some(idx));
    }

    fn do_set_device(&mut self, device_type: DeviceType) {
        let idx = self.device_state.selected().unwrap_or(0);
        let device_name = match device_type {
            DeviceType::Track => {
                self.track_device_idx = idx;
                read_devices()[self.track_device_idx].clone().name
            }
            DeviceType::Click => {
                self.click_device_idx = idx;
                read_devices()[self.click_device_idx].clone().name
            }
            DeviceType::Bleed => read_devices()[self.click_device_idx].clone().name,
        };

        self.send_player_command(PlayerCommand::SetDevice(device_type, device_name));
    }

    fn do_reset_speed(&mut self) {
        self.send_player_command(PlayerCommand::ResetSpeed);
    }

    fn do_increase_volume(&mut self, device_type: DeviceType) {
        let volume = match device_type {
            DeviceType::Track => {
                self.track_volume = std::cmp::min(self.track_volume + 1, 200);
                self.track_volume
            }
            DeviceType::Click => {
                self.click_volume = std::cmp::min(self.click_volume + 1, 200);
                self.click_volume
            }
            DeviceType::Bleed => {
                self.bleed_volume = std::cmp::min(self.bleed_volume + 1, 200);
                self.bleed_volume
            }
        };

        self.send_player_command(PlayerCommand::SetVolume(device_type, volume));
    }

    fn do_decrease_volume(&mut self, device_type: DeviceType) {
        let volume = match device_type {
            DeviceType::Track => {
                self.track_volume = std::cmp::max(self.track_volume.saturating_sub(1), 0);
                self.track_volume
            }
            DeviceType::Click => {
                self.click_volume = std::cmp::max(self.click_volume.saturating_sub(1), 0);
                self.click_volume
            }
            DeviceType::Bleed => {
                self.bleed_volume = std::cmp::max(self.bleed_volume.saturating_sub(1), 0);
                self.bleed_volume
            }
        };

        self.send_player_command(PlayerCommand::SetVolume(device_type, volume));
    }

    fn do_reset_volume(&mut self, device_type: DeviceType) {
        match device_type {
            DeviceType::Track => {
                self.track_volume = 100;
            }
            DeviceType::Click => {
                self.click_volume = 100;
            }
            DeviceType::Bleed => {
                self.bleed_volume = 100;
            }
        }
        self.send_player_command(PlayerCommand::ResetVolume(device_type));
    }

    fn do_shuffle_library(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }
        match self.active_focus {
            ActiveFocus::Library => {
                self.library.as_mut().unwrap().shuffle();
            }
            ActiveFocus::Queue => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                self.queue.shuffle(&mut rng);
            }
        }
    }

    fn do_play_next(&mut self) {
        self.player_status = PlayerStatus::Waiting;
        if !self.queue.is_empty() {
            let mut idx = self.queue_state.selected().unwrap_or(0);
            if self.active_stub.is_some() {
                idx += 1; // when we first start, we should play the first song. ;)
            }

            if idx > self.queue.len() - 1 {
                idx = 0;
            }

            self.queue_state.select(Some(idx));

            let song = self.queue[idx].clone();

            self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));

            // select it in the library
            let idx = self.library.as_ref().unwrap().get_songs().iter().position(|s| s.file_name == song.file_name).unwrap();
            self.library_state.select(Some(idx));
        } else {
            let mut idx = self.library_state.selected().unwrap_or(0);
            if self.active_stub.is_some() {
                idx += 1; // when we first start, we should play the first song. ;)
            }

            if idx > self.library.as_ref().unwrap().get_songs().len() - 1 {
                idx = 0;
            }

            self.library_state.select(Some(idx));

            let song = self.library.as_ref().unwrap().get_songs()[idx].clone();

            self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));
        }

        self.active_stub = None; // dangerous if the play commands execute first. But they shouldn't.
    }

    fn do_delete_queue(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        if self.queue.is_empty() {
            return;
        }

        match self.active_focus {
            ActiveFocus::Queue => {
                let idx = self.queue_state.selected().unwrap_or(0);
                self.queue.remove(idx);
                self.queue_state.select(Some(idx));
            }
            ActiveFocus::Library => {}
        }
    }

    fn do_insert_queue(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }
        match self.active_focus {
            ActiveFocus::Queue => {}
            ActiveFocus::Library => {
                let idx = self.library_state.selected().unwrap_or(0);
                let song = self.library.as_ref().unwrap().get_songs()[idx].clone();

                if self.queue.contains(&song) {
                    return;
                }

                self.queue.push(song);
            }
        }
    }

    fn do_empty_queue(&mut self) {
        self.queue.clear();
    }

    fn do_goto_first(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        match self.active_focus {
            ActiveFocus::Queue => {
                self.queue_state.select(Some(0));
            }
            ActiveFocus::Library => {
                self.library_state.select(Some(0));
            }
        }
    }

    fn do_goto_last(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        match self.active_focus {
            ActiveFocus::Queue => {
                self.queue_state.select(Some(self.queue.len() - 1));
            }
            ActiveFocus::Library => {
                self.library_state.select(Some(self.library.as_ref().unwrap().get_songs().len() - 1));
            }
        }
    }

    fn do_page_down(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        match self.active_focus {
            ActiveFocus::Queue => {
                let mut idx = self.queue_state.selected().unwrap_or(0);
                idx += self.page_size - 1;
                if idx > self.queue.len() - 1 {
                    idx = self.queue.len() - 1;
                }
                self.queue_state.select(Some(idx));
            }
            ActiveFocus::Library => {
                let library_length = self.library.as_ref().unwrap().get_songs().len();
                let mut idx = self.library_state.selected().unwrap_or(0);
                idx += self.page_size - 1;
                if idx > library_length - 1 {
                    idx = library_length - 1;
                }
                self.library_state.select(Some(idx));
            }
        }
    }

    fn do_page_up(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        match self.active_focus {
            ActiveFocus::Queue => {
                let mut idx = self.queue_state.selected().unwrap_or(0);
                idx = idx.saturating_sub(self.page_size - 1);
                self.queue_state.select(Some(idx));
            }
            ActiveFocus::Library => {
                let mut idx = self.library_state.selected().unwrap_or(0);
                idx = idx.saturating_sub(self.page_size - 1);
                self.library_state.select(Some(idx));
            }
        }
    }

    fn do_start_search(&mut self) {
        if self.active_menu_item != MenuItem::Library {
            return;
        }

        self.is_searching = true;
        self.do_search();
    }

    fn do_search(&mut self) {
        self.library.as_mut().unwrap().search(self.search_query.as_str());
    }

    fn do_complete_search(&mut self) {
        self.is_searching = false;
        for song in self.library.as_mut().unwrap().get_songs() {
            if !self.queue.contains(song) {
                self.queue.push(song.clone());
            }
        }
        self.do_cancel_search();
    }

    fn do_cancel_search(&mut self) {
        self.is_searching = false;
        self.library.as_mut().unwrap().reset();
    }

    fn do_replace_queue(&mut self) {
        self.do_search();
        self.queue.clear();
        self.do_complete_search();
    }

    fn do_restart_song(&mut self) {
        self.send_player_command(PlayerCommand::Restart);
    }

    fn do_set_repeat(&mut self) {
        self.is_repeating = !self.is_repeating;
    }
}

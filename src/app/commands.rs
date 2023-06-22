use std::io;

use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use log::info;
use native_dialog::{MessageDialog, MessageType};

use super::{
    devices::read_devices,
    events::UiEventTrait,
    player::{DeviceType, PlayerCommand, SongStub},
    ActiveFocus, App,
};

pub trait UiCommandTrait {
    fn do_exit(&mut self);
    fn on_exit(&mut self);
    fn do_pause(&mut self);
    fn do_playback(&mut self);
    fn do_next(&mut self);
    fn do_previous(&mut self);
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
}

impl UiCommandTrait for App {
    fn do_exit(&mut self) {
        self.is_exiting = true;
        info!("Showing confirmation dialog");
        let dialog_result = MessageDialog::new().set_title("Confirm exit").set_text("Are you sure?").set_type(MessageType::Info).show_confirm();

        match dialog_result {
            Ok(true) => {
                info!("User confirmed exit");
                self.send_player_command(PlayerCommand::Quit);
            }
            _ => {
                self.is_exiting = false;
            }
        }
    }

    fn on_exit(&mut self) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        std::process::exit(0);
    }

    fn do_pause(&mut self) {
        self.send_player_command(PlayerCommand::Pause);
    }

    fn do_playback(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {}
            ActiveFocus::Library => {
                let idx = self.library_state.selected().unwrap_or(0);
                let song = self.get_songs()[idx].clone();

                self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));
            }
        }
    }

    fn do_next(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {}
            ActiveFocus::Library => {
                let mut idx = self.library_state.selected().unwrap_or(0);
                idx += 1;
                if idx > self.get_songs().len() - 1 {
                    idx = 0;
                }
                self.library_state.select(Some(idx));
            }
        }
    }

    fn do_previous(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {}
            ActiveFocus::Library => {
                let mut idx = self.library_state.selected().unwrap_or(0) as i32;
                idx -= 1;
                if idx < 0 {
                    idx = (self.get_songs().len() - 1) as i32;
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
        if self.active_focus == ActiveFocus::Library {
            let mut idx = self.library_state.selected().unwrap_or(0);
            idx = idx + 1;
            if idx > self.get_songs().len() - 1 {
                idx = 0;
            }
            let song = self.get_songs()[idx].clone();
            self.library_state.select(Some(idx));

            self.send_player_command(PlayerCommand::Play(SongStub::from_song_record(&song)));
        }
    }

    fn do_forward(&mut self) {
        self.send_player_command(PlayerCommand::Forward);
    }

    fn do_backward(&mut self) {
        self.send_player_command(PlayerCommand::Backward);
    }

    fn do_speedup(&mut self) {
        info!("Speeding up");
        self.send_player_command(PlayerCommand::SpeedUp);
    }

    fn do_slowdown(&mut self) {
        info!("Slowing down");
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
        self.track_device_idx = self.device_state.selected().unwrap_or(0);
        let device_name = read_devices()[self.track_device_idx].clone().name;
        self.send_player_command(PlayerCommand::SetDevice(device_type, device_name));
    }

    fn do_reset_speed(&mut self) {
        self.send_player_command(PlayerCommand::ResetSpeed);
    }

    fn do_increase_volume(&mut self, device_type: DeviceType) {
        self.send_player_command(PlayerCommand::IncreaseVolume(device_type));
    }

    fn do_decrease_volume(&mut self, device_type: DeviceType) {
        self.send_player_command(PlayerCommand::DecreaseVolume(device_type));
    }

    fn do_reset_volume(&mut self, device_type: DeviceType) {
        self.send_player_command(PlayerCommand::ResetVolume(device_type));
    }
}

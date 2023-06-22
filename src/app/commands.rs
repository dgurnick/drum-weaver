use std::{io, thread, time::Duration};

use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use native_dialog::{MessageDialog, MessageType};

use super::{
    player::{PlayerCommand, SongStub},
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
}

impl UiCommandTrait for App {
    fn do_exit(&mut self) {
        self.is_exiting = true;
        let dialog_result = MessageDialog::new().set_title("Confirm exit").set_text("Are you sure?").set_type(MessageType::Info).show_confirm();

        match dialog_result {
            Ok(true) => {
                self.player_command_sender.send(PlayerCommand::Quit).unwrap();
                thread::sleep(Duration::from_millis(500));
                std::process::exit(0);
            }
            _ => {
                self.is_exiting = false;
            }
        }
    }

    fn do_pause(&mut self) {
        self.player_command_sender.send(PlayerCommand::Pause).unwrap();
    }

    fn do_playback(&mut self) {
        match self.active_focus {
            ActiveFocus::Queue => {}
            ActiveFocus::Library => {
                let idx = self.library_state.selected().unwrap_or(0);
                let song = self.get_songs()[idx].clone();

                self.player_command_sender.send(PlayerCommand::Play(SongStub::from_song_record(&song))).unwrap();
            }
        }
    }

    fn on_exit(&mut self) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
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
}

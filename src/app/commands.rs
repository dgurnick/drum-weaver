use std::io;

use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

use super::{player::PlayerCommand, App};

pub trait UiCommandTrait {
    fn do_exit(&mut self);
    fn on_exit(&mut self);
    fn do_pause(&mut self);
    fn do_playback(&mut self);
}

impl UiCommandTrait for App {
    fn do_exit(&mut self) {
        self.player_command_sender.send(PlayerCommand::Quit).unwrap();
    }

    fn do_pause(&mut self) {
        self.player_command_sender.send(PlayerCommand::Pause).unwrap();
    }

    fn do_playback(&mut self) {
        self.player_command_sender.send(PlayerCommand::Play("test2.mp3".to_string())).unwrap();
    }

    fn on_exit(&mut self) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    }
}

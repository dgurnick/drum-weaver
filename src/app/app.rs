use std::thread;

use crossbeam_channel::{Receiver, Sender};
use log::info;

use super::player::{PlayerCommand, PlayerEvent};

pub struct App {
    player_command_sender: Sender<PlayerCommand>,
    player_event_receiver: Receiver<PlayerEvent>,
}

impl App {
    pub fn new(player_command_sender: Sender<PlayerCommand>, player_event_receiver: Receiver<PlayerEvent>) -> Self {
        App {
            player_command_sender,
            player_event_receiver,
        }
    }

    pub fn run(&mut self) {
        self.player_command_sender.send(PlayerCommand::Play("test.mp3".to_string())).unwrap();
        loop {
            match self.player_event_receiver.try_recv() {
                Ok(event) => match event {
                    PlayerEvent::Decompressing => {
                        info!("App received Decompressing signal");
                    }
                    PlayerEvent::Decompressed => {
                        info!("App received Decompressed signal");
                    }
                    PlayerEvent::Playing(file_name) => {
                        info!("App received Playing signal: {}", file_name);
                        self.player_command_sender.send(PlayerCommand::Pause).unwrap();
                    }
                    PlayerEvent::Paused => {
                        info!("App received Paused signal");
                    }
                    PlayerEvent::Stopped => {
                        info!("App received Stopped signal");
                    }
                    PlayerEvent::Quit => {
                        info!("App received Quit signal. Exiting.");
                        break;
                    }
                    PlayerEvent::LoadFailure(file_name) => {
                        info!("App received LoadFailure: {}", file_name);
                        // TODO: Remove song from list and queue
                        self.player_command_sender.send(PlayerCommand::Play("test2.mp3".to_string())).unwrap();
                    }
                },
                Err(_err) => {}
            }

            // handle input events

            // render UI elements
        }
    }
}

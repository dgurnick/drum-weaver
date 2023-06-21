use std::thread;

use crossbeam_channel::{Receiver, Sender};

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
                        println!("App received Decompressing");
                    }
                    PlayerEvent::Decompressed => {
                        println!("App received Decompressed");
                    }
                    PlayerEvent::Playing(file_name) => {
                        println!("App received Playing: {}", file_name);
                        thread::sleep(std::time::Duration::from_millis(1000));
                        self.player_command_sender.send(PlayerCommand::Pause).unwrap();
                    }
                    PlayerEvent::Paused => {
                        println!("App received Paused");
                        thread::sleep(std::time::Duration::from_millis(1000));
                        self.player_command_sender.send(PlayerCommand::Stop).unwrap();
                        thread::sleep(std::time::Duration::from_millis(1000));
                        self.player_command_sender.send(PlayerCommand::Quit).unwrap();
                    }
                    PlayerEvent::Stopped => {
                        println!("App received Stopped");
                    }
                    PlayerEvent::Quit => {
                        println!("App received Quit");
                        break;
                    }
                    PlayerEvent::LoadFailure(file_name) => {
                        println!("App received LoadFailure: {}", file_name);
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

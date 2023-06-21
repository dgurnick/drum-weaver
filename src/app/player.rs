use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use log::info;
pub struct Player {
    player_command_receiver: Receiver<PlayerCommand>,
    player_event_sender: Sender<PlayerEvent>,
}

#[derive(Debug)]
pub enum PlayerCommand {
    Play(String),
    Pause,
    Stop,
    Quit,
}

#[derive(Debug)]
pub enum PlayerEvent {
    Playing(String),
    LoadFailure(String),
    Paused,
    Stopped,
    Decompressing,
    Decompressed,
    Quit,
}

impl Player {
    pub fn new(player_command_receiver: Receiver<PlayerCommand>, player_event_sender: Sender<PlayerEvent>) -> Self {
        Player {
            player_command_receiver,
            player_event_sender,
        }
    }

    pub fn run(&mut self) {
        let player_command_receiver = Arc::new(Mutex::new(self.player_command_receiver.clone()));
        let player_event_sender = Arc::new(Mutex::new(self.player_event_sender.clone()));

        thread::spawn(move || loop {
            {
                let player_command_receiver = player_command_receiver.lock().unwrap();
                let player_event_sender = player_event_sender.lock().unwrap();

                match player_command_receiver.try_recv() {
                    Ok(command) => match command {
                        PlayerCommand::Play(file_name) => {
                            info!("Player will load: {:?}", file_name);

                            // todo: check if decompression is required and execute in a separate thread
                            player_event_sender.send(PlayerEvent::Decompressing).unwrap();
                            player_event_sender.send(PlayerEvent::Decompressed).unwrap();

                            // TODO: capture any errors and send LoadFailure event
                            player_event_sender.send(PlayerEvent::Playing(file_name)).unwrap();
                        }
                        PlayerCommand::Pause => {
                            info!("Player is pausing");
                            player_event_sender.send(PlayerEvent::Paused).unwrap();
                        }
                        PlayerCommand::Stop => {
                            info!("Player will stop");
                            player_event_sender.send(PlayerEvent::Stopped).unwrap();
                        }
                        PlayerCommand::Quit => {
                            info!("Player will quit. Exiting.");
                            player_event_sender.send(PlayerEvent::Quit).unwrap();
                            break;
                        }
                    },
                    Err(_err) => {}
                }
            }
        });
    }
}

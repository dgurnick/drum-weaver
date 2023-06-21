use std::{
    sync::{Arc, Mutex},
    thread,
};

use rand::Rng;

use crossbeam_channel::{Receiver, Sender};
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
                            println!("Player will load: {:?}", file_name);
                            thread::sleep(std::time::Duration::from_millis(1000));
                            player_event_sender.send(PlayerEvent::Decompressing).unwrap();
                            thread::sleep(std::time::Duration::from_millis(1000));
                            player_event_sender.send(PlayerEvent::Decompressed).unwrap();
                            thread::sleep(std::time::Duration::from_millis(1000));

                            let mut rng = rand::thread_rng();
                            match rng.gen_range(0..=1) {
                                0 => player_event_sender.send(PlayerEvent::LoadFailure(file_name)).unwrap(),
                                _ => player_event_sender.send(PlayerEvent::Playing(file_name)).unwrap(),
                            };
                        }
                        PlayerCommand::Pause => {
                            println!("Player is pausing");
                            thread::sleep(std::time::Duration::from_millis(1000));
                            player_event_sender.send(PlayerEvent::Paused).unwrap();
                        }
                        PlayerCommand::Stop => {
                            println!("Player will stop");
                            player_event_sender.send(PlayerEvent::Stopped).unwrap();
                        }
                        PlayerCommand::Quit => {
                            println!("Player will quit");
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

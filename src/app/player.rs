use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

use cpal::traits::HostTrait;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

use crate::app::audio::Song;

use super::audio::AudioPlayer;
#[derive(Clone)]
pub struct Player {
    player_command_receiver: Receiver<PlayerCommand>,
    player_event_sender: Sender<PlayerEvent>,
    is_paused: Arc<Mutex<bool>>,
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
    Continuing,
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
            is_paused: Arc::new(Mutex::new(false)),
        }
    }

    pub fn run(&mut self) {
        let player_command_receiver = Arc::new(Mutex::new(self.player_command_receiver.clone()));
        let player_event_sender = Arc::new(Mutex::new(self.player_event_sender.clone()));
        let is_paused = self.is_paused.clone();
        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

        thread::spawn(move || {
            let mut track_device = &available_devices[0];
            let mut click_device = &available_devices[0];

            let mut track_player = AudioPlayer::new(None, track_device).expect("Could not create track player");
            let mut click_player = AudioPlayer::new(None, click_device).expect("Could not create click player");

            track_player.set_playback_speed(1.0);
            click_player.set_playback_speed(1.0);

            loop {
                let player_command_receiver = player_command_receiver.lock().unwrap();
                let player_event_sender = player_event_sender.lock().unwrap();

                // See if any commands have been sent to the player
                match player_command_receiver.try_recv() {
                    Ok(command) => match command {
                        PlayerCommand::Play(file_name) => {
                            info!("Player will load: {:?}", file_name);

                            // todo: check if decompression is required and execute in a separate thread
                            player_event_sender.send(PlayerEvent::Decompressing).unwrap();
                            player_event_sender.send(PlayerEvent::Decompressed).unwrap();

                            match track_player.play_song_next(&Song::from_file(get_beep_file(), Some(0.5)).unwrap(), None) {
                                Ok(_) => {
                                    info!("Playing song: {:?}", file_name);
                                    player_event_sender.send(PlayerEvent::Playing(file_name)).unwrap();
                                }
                                Err(err) => {
                                    info!("Failed to play song: {:?}", err);
                                    player_event_sender.send(PlayerEvent::LoadFailure(file_name)).unwrap();
                                    continue;
                                }
                            }
                        }
                        PlayerCommand::Pause => {
                            let mut is_paused = is_paused.lock().unwrap();

                            if *is_paused {
                                info!("Player is continuing");
                                player_event_sender.send(PlayerEvent::Continuing).unwrap();
                            } else {
                                info!("Player is pausing");
                                player_event_sender.send(PlayerEvent::Paused).unwrap();
                            }
                            *is_paused = !*is_paused;
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

fn get_beep_file() -> String {
    let mut path = env::current_dir().expect("Failed to get current exe path");

    // Append the relative path to your asset
    path.push("assets");
    path.push("beep.wav");

    path.display().to_string()
}

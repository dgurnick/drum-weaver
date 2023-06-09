use std::{error::Error, path::PathBuf, thread, time::Duration};

use cpal::traits::HostTrait;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use symphonia::core::{
    io::{MediaSourceStream, MediaSourceStreamOptions},
    probe::Hint,
};

use crate::app::{audio::Song, AppConfig};
use cpal::traits::DeviceTrait;

use super::{audio::AudioPlayer, beep::BeepMediaSource, library::SongRecord};
pub struct Player {
    player_command_receiver: Receiver<PlayerCommand>,
    player_event_sender: Sender<PlayerEvent>,
}

#[derive(PartialEq, Debug)]
pub enum DeviceType {
    Track,
    Click,
    Bleed,
}

#[derive(Debug, Clone)]
pub struct SongStub {
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub folder: String,
}

#[derive(Debug)]
pub struct PlaybackStatus {
    pub track_position: Option<Duration>,
    pub track_duration: Option<Duration>,
    pub track_volume: f32,
    pub click_volume: f32,
    pub bleed_volume: f32,
}
impl SongStub {
    pub fn from_song_record(song_record: &SongRecord) -> Self {
        SongStub {
            file_name: song_record.file_name.clone(),
            title: song_record.title.clone(),
            folder: song_record.folder.clone(),
            artist: song_record.artist.clone(),
        }
    }
}
#[derive(Debug)]
pub enum PlayerCommand {
    Play(SongStub),
    Pause,
    Quit,
    GetStatus,
    Forward,
    Backward,
    SpeedUp,
    SlowDown,
    SetDevice(DeviceType, String),
    ResetSpeed,
    SetVolume(DeviceType, usize),
    ResetVolume(DeviceType),
    Restart,
}

#[derive(Debug)]
pub enum PlayerEvent {
    Playing(SongStub),
    LoadFailure(SongStub),
    Status(PlaybackStatus),
    Paused,
    Continuing(Option<SongStub>),
    Ended,
    Decompressing,
    Decompressed,
    Quit,
}

const BEEP_BYTES: &[u8] = include_bytes!("../../assets/beep.wav");

impl Player {
    pub fn new(player_command_receiver: Receiver<PlayerCommand>, player_event_sender: Sender<PlayerEvent>) -> Self {
        Player {
            player_command_receiver,
            player_event_sender,
        }
    }

    pub fn run(&mut self) {
        let player_event_sender = self.player_event_sender.clone();
        let player_command_receiver = self.player_command_receiver.clone();

        thread::spawn(move || {
            let host = cpal::default_host();
            let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

            // TODO: Devices from configuration
            let mut track_device = &available_devices[0];
            let mut click_device = &available_devices[0];

            let mut track_player = AudioPlayer::new(None, track_device).expect("Could not create track player");
            let mut click_player = AudioPlayer::new(None, click_device).expect("Could not create click player");
            let mut bleed_player = AudioPlayer::new(None, click_device).expect("Could not create click player");

            track_player.set_playback_speed(1.0);
            click_player.set_playback_speed(1.0);
            bleed_player.set_playback_speed(1.0);

            let mut current_stub: Option<SongStub> = None;

            // run a thread that monitors the player and sends an Ended event to the UI thread when the song is complete. Used for autoplay

            loop {
                // See if any commands have been sent to the player
                match player_command_receiver.try_recv() {
                    Ok(command) => match command {
                        PlayerCommand::Play(stub) => {
                            //info!("Player will load: {:?}", stub.file_name);4
                            // 1. Check if the wav and click files exist. If not, decompress from the 7z file
                            // 2. Load the wav and click files into the player
                            // 3. Play the song
                            let (track_path, click_path) = Self::get_file_paths(stub.folder.as_str(), stub.file_name.as_str());
                            track_player.set_playing(false);
                            click_player.set_playing(false);
                            bleed_player.set_playing(false);

                            if !track_path.exists() {
                                info!("Track file does not exist. Decompressing.");
                                player_event_sender.send(PlayerEvent::Decompressing).unwrap();

                                match Self::decompress_files(stub.folder.as_str(), stub.file_name.as_str()) {
                                    Ok(()) => {
                                        info!("Decompression complete");
                                        player_event_sender.send(PlayerEvent::Decompressed).unwrap();
                                    }
                                    Err(err) => {
                                        info!("Decompression failed: {:?}", err);
                                        player_event_sender.send(PlayerEvent::LoadFailure(stub.clone())).unwrap();
                                        continue;
                                    }
                                }
                            }

                            let track_song = Song::from_file(track_path.clone(), None);
                            let click_song = Song::from_file(click_path.clone(), None);
                            let bleed_song = Song::from_file(track_path.clone(), None);

                            if let Err(err) = track_song {
                                error!("Failed to load song: {:?}", err);
                                player_event_sender.send(PlayerEvent::LoadFailure(stub.clone())).unwrap();
                                current_stub = None;
                                continue;
                            }

                            if let Err(err) = click_song {
                                error!("Failed to load click: {:?}", err);
                                player_event_sender.send(PlayerEvent::LoadFailure(stub.clone())).unwrap();
                                current_stub = None;
                                continue;
                            }

                            let track_song = track_song.unwrap();
                            let click_song = click_song.unwrap();
                            let bleed_song = bleed_song.unwrap();

                            track_player.stop();
                            click_player.stop();
                            bleed_player.stop();

                            let track_status = track_player.play_song_now(&track_song, None);
                            let click_status = click_player.play_song_now(&click_song, None);
                            bleed_player.play_song_now(&bleed_song, None).expect("Unable to play bleed");

                            match (track_status, click_status) {
                                (Ok(_), Ok(_)) => {
                                    // Both track and click songs were played successfully
                                    track_player.set_playing(true);
                                    click_player.set_playing(true);
                                    bleed_player.set_playing(true);
                                    current_stub = Some(stub.clone());

                                    player_event_sender.send(PlayerEvent::Playing(stub.clone())).unwrap();
                                }
                                (Err(track_err), _) => {
                                    // Failed to play the track song
                                    error!("Failed to play track song: {:?}", track_err);
                                    player_event_sender.send(PlayerEvent::LoadFailure(stub.clone())).unwrap();
                                    track_player.stop();
                                    click_player.stop();
                                    bleed_player.stop();
                                    current_stub = None;
                                    continue;
                                }
                                (_, Err(click_err)) => {
                                    // Failed to play the click song
                                    error!("Failed to play click song: {:?}", click_err);
                                    player_event_sender.send(PlayerEvent::LoadFailure(stub.clone())).unwrap();
                                    track_player.stop();
                                    click_player.stop();
                                    bleed_player.stop();
                                    current_stub = None;
                                    continue;
                                }
                            }
                        }
                        PlayerCommand::Pause => {
                            let is_playing = track_player.is_playing();

                            track_player.set_playing(!is_playing);
                            click_player.set_playing(!is_playing);
                            bleed_player.set_playing(!is_playing);

                            if !is_playing {
                                player_event_sender.send(PlayerEvent::Continuing(current_stub.clone())).unwrap();
                            } else {
                                player_event_sender.send(PlayerEvent::Paused).unwrap();
                            }
                        }
                        PlayerCommand::Quit => {
                            info!("Player received quit signal. Exiting.");
                            track_player.stop();
                            click_player.stop();
                            bleed_player.stop();
                            player_event_sender.send(PlayerEvent::Quit).unwrap();
                            thread::sleep(std::time::Duration::from_millis(100)); // time for the exit to propagate
                            break;
                        }
                        PlayerCommand::GetStatus => {
                            let mut status = PlaybackStatus {
                                track_volume: track_player.get_volume_adjustment(),
                                click_volume: click_player.get_volume_adjustment(),
                                bleed_volume: bleed_player.get_volume_adjustment(),
                                track_duration: None,
                                track_position: None,
                            };

                            if let Some((position, duration)) = track_player.get_playback_position() {
                                if track_player.is_playing() {
                                    status.track_position = Some(position);
                                    status.track_duration = Some(duration);
                                }
                            }

                            player_event_sender.send(PlayerEvent::Status(status)).unwrap();
                        }
                        PlayerCommand::Forward => {
                            if current_stub.is_none() {
                                continue;
                            }

                            if let Some((position, duration)) = track_player.get_playback_position() {
                                let new_position = position.checked_add(Duration::from_secs(10));
                                if let Some(seek) = new_position {
                                    if seek > duration {
                                        // nope
                                    } else {
                                        track_player.seek(seek);
                                        click_player.seek(seek);
                                    }
                                } else {
                                    track_player.seek(Duration::from_micros(0));
                                    click_player.seek(Duration::from_micros(0));
                                    bleed_player.seek(Duration::from_micros(0));
                                }
                            }
                        }
                        PlayerCommand::Backward => {
                            if current_stub.is_none() {
                                continue;
                            }
                            if let Some((position, _)) = track_player.get_playback_position() {
                                let new_position = position.checked_sub(Duration::from_secs(10));
                                if let Some(seek) = new_position {
                                    track_player.seek(seek);
                                    click_player.seek(seek);
                                    bleed_player.seek(seek);
                                } else {
                                    track_player.seek(Duration::from_micros(0));
                                    click_player.seek(Duration::from_micros(0));
                                    bleed_player.seek(Duration::from_micros(0));
                                }
                            }
                        }
                        PlayerCommand::SpeedUp => {
                            let current_speed = track_player.get_playback_speed();
                            let new_speed = current_speed + 0.1;
                            track_player.set_playback_speed(new_speed);
                            click_player.set_playback_speed(new_speed);
                            bleed_player.set_playback_speed(new_speed);
                        }
                        PlayerCommand::SlowDown => {
                            let current_speed = track_player.get_playback_speed();
                            let new_speed = current_speed - 0.1;
                            if new_speed < 0.1 {
                                continue;
                            }

                            track_player.set_playback_speed(new_speed);
                            click_player.set_playback_speed(new_speed);
                            bleed_player.set_playback_speed(new_speed);
                        }
                        PlayerCommand::SetDevice(device_type, device_name) => {
                            track_player.stop();
                            click_player.stop();
                            bleed_player.stop();

                            let device = available_devices.iter().find(|d| d.name().ok() == Some(device_name.clone()));

                            let beep_source = Box::new(BeepMediaSource::new(BEEP_BYTES));
                            let beep_options = MediaSourceStreamOptions { buffer_len: 64 * 1024 };

                            let beep_stream = MediaSourceStream::new(beep_source, beep_options);
                            let beep_song = Song::new(Box::new(beep_stream), &Hint::new(), None).unwrap();

                            match device {
                                Some(device) => {
                                    if device_type == DeviceType::Track {
                                        track_device = device;
                                        track_player = AudioPlayer::new(None, track_device).expect("Could not create track player");
                                        track_player.play_song_now(&beep_song, None).expect("Could not play beep on track player");
                                        //track_player.play_song_now(&Song::from_file(Self::get_beep_file(), None).unwrap(), None).unwrap();
                                    } else {
                                        click_device = device;
                                        click_player = AudioPlayer::new(None, click_device).expect("Could not create click player");
                                        bleed_player = AudioPlayer::new(None, click_device).expect("Could not create click player");
                                        //click_player.play_song_now(&Song::from_file(Self::get_beep_file(), None).unwrap(), None).unwrap();
                                        click_player.play_song_now(&beep_song, None).expect("Could not play beep on click player");
                                    }
                                }
                                None => {
                                    error!("Could not find device with name {}", device_name);
                                    error!("This is unrecoverable. Resetting configuration. Please restart the application.");
                                    confy::store("drum-weaver", None, AppConfig::default()).unwrap();
                                    std::process::exit(1);
                                }
                            }
                        }
                        PlayerCommand::ResetSpeed => {
                            track_player.set_playback_speed(1.0);
                            click_player.set_playback_speed(1.0);
                            bleed_player.set_playback_speed(1.0);
                        }
                        PlayerCommand::SetVolume(device_type, volume) => {
                            let new_volume = volume as f32 / 100.0;

                            match device_type {
                                DeviceType::Track => {
                                    track_player.set_volume_adjustment(new_volume);
                                }
                                DeviceType::Click => {
                                    click_player.set_volume_adjustment(new_volume);
                                }
                                DeviceType::Bleed => {
                                    bleed_player.set_volume_adjustment(new_volume);
                                }
                            }
                        }

                        PlayerCommand::ResetVolume(device_type) => match device_type {
                            DeviceType::Track => {
                                track_player.set_volume_adjustment(1.0);
                            }
                            DeviceType::Click => {
                                click_player.set_volume_adjustment(1.0);
                            }
                            DeviceType::Bleed => {
                                bleed_player.set_volume_adjustment(1.0);
                            }
                        },
                        PlayerCommand::Restart => {
                            track_player.seek(Duration::from_micros(0));
                            click_player.seek(Duration::from_micros(0));
                            bleed_player.seek(Duration::from_micros(0));
                        }
                    },
                    Err(_err) => {}
                }

                // if we have a current_stub, but the player is not playing, then we need to send a stopped event
                if current_stub.clone().is_some() && !track_player.has_current_song() {
                    player_event_sender.send(PlayerEvent::Ended).unwrap();
                    current_stub = None;
                    track_player.stop();
                    click_player.stop();
                    bleed_player.stop();
                }
            }
        });
    }

    // Helper that returns the full paths for the main and click files
    // It does not check if they exist
    fn get_file_paths(song_folder: &str, song_title: &str) -> (PathBuf, PathBuf) {
        let mut track_path = PathBuf::new();
        track_path.push(song_folder);
        track_path.push(format!("{}.wav", song_title));

        let mut click_path = PathBuf::new();
        click_path.push(song_folder);
        click_path.push(format!("{}_click.wav", song_title));

        (track_path, click_path)
    }

    fn decompress_files(song_folder: &str, song_title: &str) -> Result<(), Box<dyn Error>> {
        // if there's a 7z file with the same name, decompress it
        //let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
        let mut archive_path = PathBuf::new();
        archive_path.push(song_folder);
        archive_path.push(format!("{}.7z", song_title));

        let mut output_folder = PathBuf::new();
        output_folder.push(song_folder);

        match sevenz_rust::decompress_file(&archive_path, output_folder) {
            Ok(_) => {
                info!("Decompressed file: {:?}", archive_path);
                Ok(())
            }
            Err(err) => {
                error!("Failed to decompress file: {:?}", err);
                Err(Box::new(err))
            }
        }
    }
}

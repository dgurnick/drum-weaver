pub use crate::app::library::Library;
pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;
use clap::Arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig, SupportedStreamConfig};
use eframe::egui;
use ringbuf::HeapRb;
use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

mod app;

pub enum PlayerState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

const RING_BUFFER_SIZE: usize = 48000;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("starting app");

    let matches = clap::Command::new("Drummer's Karaoke")
        .version("0.1")
        .arg(
            Arg::new("music_folder")
                .long("music_folder")
                .required(false)
                .help("Where your music files are stored"),
        )
        .arg(
            Arg::new("song_file")
                .long("song_file")
                .required(false)
                .help("The CSV file with the song list"),
        )
        .get_matches();

    let music_folder = matches
        .get_one::<String>("music_folder")
        .map(|s| s.to_owned());

    let song_file = matches.get_one::<String>("song_file").map(|s| s.to_owned());

    let (audio_tx, audio_rx) = channel();
    let (tx, rx) = channel();
    let cursor = Arc::new(AtomicU32::new(0));
    let _cursor_clone = cursor.clone();
    let mut player = Player::new(audio_tx, cursor);
    let initial_library = match music_folder {
        Some(folder) => {
            tracing::info!("Will use music from: {}", folder);
            let path = PathBuf::from(folder.clone());

            let mut library = Library::new(path);

            if let Some(song_file) = song_file {
                let song_file = PathBuf::from(song_file);
                library.load_songs(song_file);
            }
            player.set_music_folder(folder.clone());

            Some(library)
        }
        None => {
            tracing::info!("No default music folder was provided.");
            None
        }
    };

    let mut app = App::load().unwrap_or_default();
    app.player = Some(player);
    app.library = initial_library;
    app.library_sender = Some(tx);
    app.library_receiver = Some(rx);

    let _audio_thread = thread::spawn(move || {
        let state = Arc::new(Mutex::new(PlayerState::Stopped));
        let _state_clone = state.clone();

        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

        let mut track_device = host.default_output_device().unwrap();
        let mut click_device = host.default_output_device().unwrap();

        let track_configuration = get_device_configuration(track_device);
        let click_configuration = get_device_configuration(click_device);
        let track_sample_format = track_configuration.sample_format();
        let click_sample_format = click_configuration.sample_format();
        let track_stream_config: StreamConfig = track_configuration.into();
        let click_stream_config: StreamConfig = click_configuration.into();

        let track_device_sample_rate = track_stream_config.sample_rate.0 as f32;
        let click_device_sample_rate = click_stream_config.sample_rate.0 as f32;
        let track_sample_rate = Arc::new(Mutex::new(0u32));
        let click_sample_rate = Arc::new(Mutex::new(0u32));

        let mut track_volume = 1.0;
        let mut click_volume = 1.0;

        let output_err_fn = |err| tracing::error!("Error in output stream: {}", err);
        let state = Arc::new(Mutex::new(PlayerState::Stopped));
        let state_clone = state.clone();
        let (mut audio_producer, mut audio_consumer) = HeapRb::<i16>::new(RING_BUFFER_SIZE).split();
        let mut next_sample = move || {
            let guard = state_clone.lock().unwrap();
            match *guard {
                PlayerState::Playing => match audio_consumer.pop() {
                    Some(data) => data,
                    None => 0i16,
                },
                _ => 0i16,
            }
        };

        let track_stream = match track_sample_format {
            SampleFormat::F32 => {
                let next_sample_f32 = || next_sample() as f32;
                &track_device.build_output_stream(
                    &track_stream_config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_f32)
                    },
                    output_err_fn,
                    None, // timeout duration
                )
            }
            SampleFormat::I16 => {
                let next_sample_i16 = || next_sample() as i16;
                &track_device.build_output_stream(
                    &track_stream_config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_i16)
                    },
                    output_err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let next_sample_u16 = || next_sample() as u16;
                &track_device.build_output_stream(
                    &track_stream_config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_u16)
                    },
                    output_err_fn,
                    None, // timeout duration
                )
            }
        }
        .expect("Failed to build track output stream");

        let click_stream = match click_sample_format {
            SampleFormat::F32 => {
                let next_sample_f32 = || next_sample() as f32;
                click_device.build_output_stream(
                    &click_stream_config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_f32)
                    },
                    output_err_fn,
                    None, // timeout duration
                )
            }
            SampleFormat::I16 => {
                let next_sample_i16 = || next_sample() as i16;
                click_device.build_output_stream(
                    &click_stream_config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_i16)
                    },
                    output_err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let next_sample_u16 = || next_sample() as u16;
                click_device.build_output_stream(
                    &click_stream_config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        write_sample(data, next_sample_u16)
                    },
                    output_err_fn,
                    None, // timeout duration
                )
            }
        }
        .expect("Failed to build click output stream");

        let _ = &track_stream.play().unwrap();
        let _ = &click_stream.play().unwrap();

        loop {
            // check for new commands
            match audio_rx.try_recv() {
                Ok(cmd) => match cmd {
                    AudioCommand::SetTrackDevice(device) => {
                        tracing::info!("Audio will use the track device: {}", device);
                    }
                    AudioCommand::SetClickDevice(device) => {
                        tracing::info!("Audio will use the click device: {}", device);
                    }
                    AudioCommand::PlayTrack(music_folder, track) => {
                        tracing::info!("Audio will load the song: {}", track.title().unwrap());

                        // get the song names
                        let mut track_path = PathBuf::new();
                        track_path.push(music_folder.clone());
                        track_path.push(track.folder.clone().unwrap());
                        track_path.push(format!("{}.wav", track.file_name.clone().unwrap()));

                        let mut click_path = PathBuf::new();
                        click_path.push(music_folder.clone());
                        click_path.push(track.folder.clone().unwrap());
                        click_path.push(format!("{}_click.wav", track.file_name.clone().unwrap()));

                        if !track_path.exists() {
                            // if there's a 7z file with the same name, decompress it
                            //let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
                            let mut archive_path = PathBuf::new();
                            archive_path.push(music_folder.clone());
                            archive_path.push(track.folder.clone().unwrap());
                            archive_path.push(format!("{}.7z", track.file_name.clone().unwrap()));

                            let mut output_folder = PathBuf::new();
                            output_folder.push(music_folder);
                            output_folder.push(track.folder.clone().unwrap());

                            sevenz_rust::decompress_file(&archive_path, output_folder)
                                .expect("Failed to decompress file");
                        }

                        let track_path_str = track_path.display().to_string();
                        let click_path_str = click_path.display().to_string();
                        tracing::info!("Will play track from file: {}", track_path_str);
                        tracing::info!("Will play click from file: {}", click_path_str);
                    }
                    AudioCommand::Play => {
                        tracing::info!("Audio will play the song");
                        let mut guard = state.lock().unwrap();
                        *guard = PlayerState::Playing;
                    }
                    AudioCommand::Pause => {
                        tracing::info!("Audio will pause the song");
                        let mut guard = state.lock().unwrap();
                        *guard = PlayerState::Paused;
                    }
                    AudioCommand::Stop => {
                        tracing::info!("Audio will stop the song");
                        let mut guard = state.lock().unwrap();
                        *guard = PlayerState::Stopped;
                    }
                    _ => {
                        tracing::info!("Audio does not implement the requested event");
                    }
                },
                _ => {} // throw away bad events
            }
        }

        // play if needed
        // get the audio sources

        // if the audio source is none, we inform the player
    });

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(
        "Drum Karaoke Player",
        window_options,
        Box::new(|_| Box::new(app)),
    )
    .expect("eframe failed: I should change main to return a result and use anyhow");
}

fn get_device_configuration(device: Device) -> SupportedStreamConfig {
    let configs = device
        .supported_output_configs()
        .expect("Unable to get output configurations for the selected device");
    configs
        .next()
        .expect("Missing output configurations for this device")
        .with_max_sample_rate()
}

fn write_sample<T, F>(data: &mut [T], mut next_sample: F)
where
    T: cpal::Sample,
    F: FnMut() -> T,
{
    for frame in data.chunks_mut(1) {
        let value = next_sample();
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

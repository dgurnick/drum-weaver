use std::fs::{metadata, File};
use std::io::{BufReader, BufRead};
use std::path::PathBuf;
use std::time::Duration;
use rodio::*;
use rodio::source::Amplify;
use rodio::cpal::traits::{HostTrait};
use std::{thread, println};
use clap::{Arg, ArgMatches};
use csv;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use serde::Deserialize;
use sevenz_rust;


#[derive(Debug, serde::Deserialize)]
struct Song {
    file_name: String,
    genre: String,
    year: String,
    artist: String,
    song: String,
    album: String,
    length: String, // is actually a duration
    bpm: usize,
    folder: String,
}

fn main() {

    let matches = clap::Command::new("eDrums Wav Player")
        .version("0.1")
        .arg(Arg::new("music_folder").long("music_folder").required(true).help("Where your music files are stored"))
        .arg(Arg::new("track").long("track").required(true).help("Position in the csv file to play"))
        .arg(Arg::new("track_volume").long("track_volume").required(false).default_value("100"))
        .arg(Arg::new("click_volume").long("click_volume").required(false).default_value("100"))
        .arg(Arg::new("track_device").long("track_device").required(false).default_value("1"))
        .arg(Arg::new("click_device").long("click_device").required(false).default_value("1"))
        .arg(Arg::new("combined").long("combined").required(false).default_value("1"))
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

/// Retrieves file paths for music files in a specified folder.
/// If the file does not exist, but a matching "7z" file does,
/// it will automatically decompress the 7z file for you.
///
/// # Arguments
///
/// * `music_folder` - A string slice representing the path to the music folder.
/// * `song_position` - An `usize` indicating the position of the desired song.
///
/// # Returns
///
/// A `Result` containing a tuple with the file paths of two music files, or an error message as a `String`.
/// If successful, the tuple contains two `String`s representing the file paths.
/// If unsuccessful, an error message is returned as a `String`.
///
/// # Example
///
/// ```rust
/// let result = get_file_paths("/path/to/music/folder", 0);
/// match result {
///     Ok((file1, file2)) => {
///         println!("File 1: {}", file1);
///         println!("File 2: {}", file2);
///     },
///     Err(error) => {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
fn get_file_paths(music_folder: &str, song_position: usize) -> Result<(String, String), String> {

    let mut reader = csv::Reader::from_path("assets/song_list.csv").unwrap();
    let headers = reader.headers();
    println!("{:?}", headers);

    let mut position = 1;
    for record in reader.deserialize() {
        let song: Song = record.unwrap();

        if position == song_position {
            let track_path_str = format!("{}/{}/{}.wav", music_folder, song.folder, song.file_name);
            let click_path_str = format!("{}/{}/{}_click.wav", music_folder, song.folder, song.file_name);

            let mut path = PathBuf::new();
            path.push(music_folder);
            path.push(&track_path_str);

            if !path.exists() {
                // if there's a 7z file with the same name, decompress it 
                 let archive_path = PathBuf::from(format!("{}/{}/{}.7z", music_folder, song.folder, song.file_name));
                 if ! archive_path.exists() {
                    return Err(format!("Failed to find file or 7z archive for {}", archive_path.display()));
                 } 
                 println!("Decompressing file: {}", archive_path.display());

                 let mut output_folder = PathBuf::new();
                 output_folder.push(music_folder);
                 output_folder.push(song.folder);

                 sevenz_rust::decompress_file(&archive_path, output_folder).expect("Failed to decompress file");

            }

            check_file_existence(music_folder, &track_path_str)?;
            check_file_existence(music_folder, &click_path_str)?;

            return Ok((track_path_str, click_path_str));
        } else {
            position += 1;

        }
    }

    Err("Could not find song".to_string())

}

/// Checks the existence of a file in a specified folder.
///
/// # Arguments
///
/// * `folder_path` - A string slice representing the path to the folder.
/// * `file_name` - A string slice representing the name of the file to check.
///
/// # Returns
///
/// A `Result` indicating the result of the existence check.
/// If the file exists, `Ok(())` is returned.
/// If the file does not exist or encounters an error, an error message is returned as a `String`.
///
/// # Example
///
/// ```rust
/// let result = check_file_existence("/path/to/folder", "example.txt");
/// match result {
///     Ok(()) => {
///         println!("File exists.");
///     },
///     Err(error) => {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
fn check_file_existence(folder_path: &str, file_name: &str) -> Result<(), String> {
    let mut path = PathBuf::new();
    path.push(folder_path);
    path.push(file_name);

    if let Err(_) = metadata(&path) {
        return Err(format!("File '{}' does not exist", path.display()));
    }
    Ok(())
}

/// Runs the program as determined by the main function
fn run(matches: &ArgMatches) -> Result<i32, String> {

    // Parse the arguments
    let music_folder = matches.get_one::<String>("music_folder").expect("No folder provided");
    
    let track_position = matches.get_one::<String>("track")
        .unwrap_or(&"1.0".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let (track_path_str, click_path_str) = match get_file_paths(music_folder, track_position) {
        Ok((track_path_str, click_path_str)) => (track_path_str, click_path_str),
        Err(err) => return Err(err),
    };
    
    println!("Playing track: {}", track_path_str);
    println!("Playing click: {}", click_path_str);

    let track_volume = matches.get_one::<String>("track_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let click_volume = matches.get_one::<String>("click_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(100.0) / 100.0;
    let track_device_position = matches.get_one::<String>("track_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let click_device_position = matches.get_one::<String>("click_device")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1) - 1;
    let combined = matches.get_one::<String>("combined")
        .unwrap_or(&"1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
    if available_devices.is_empty() {
        return Err("No output devices found".to_string());
    }

    // Check if the device positions are valid
    let num_devices = available_devices.len();
    if track_device_position > num_devices {
        return Err("Invalid track output device".to_string());
    }
    if click_device_position > num_devices {
        return Err("Invalid click output device".to_string());
    }

    let track = File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
    let click = File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

    let track_source = Decoder::new(BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
    let click_source = Decoder::new(BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;
    let track_source_amplify = track_source.amplify(track_volume);
    let click_source_amplify = click_source.amplify(click_volume);

    if combined == 1 {
        match play_combined(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[track_device_position]
        ) {
            Ok(_) => {},
            Err(err) => return Err(err),
        }
    
    } else {
        match play_separate(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[track_device_position],
            &available_devices[click_device_position],
            track_volume,
            click_volume
        ) {
            Ok(_) => {},
            Err(err) => return Err(err),

        }
    }

    Ok(0)

}

fn play_combined(track_source: Amplify<Decoder<BufReader<File>>>, click_source: Amplify<Decoder<BufReader<File>>>, device: &Device) -> Result<(), String> {
    let (_stream, stream_handle) = OutputStream::try_from_device(&device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let combined_source = track_source.mix(click_source);

    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(combined_source);
    sink.sleep_until_end();
    Ok(())
}

fn play_separate(track_source: Amplify<Decoder<BufReader<File>>>, click_source: Amplify<Decoder<BufReader<File>>>, track_device: &Device, click_device: &Device, track_volume: f32, click_volume: f32) -> Result<(), String> {
    
    let (_track_stream, track_stream_handle) = OutputStream::try_from_device(track_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let (_click_stream, click_stream_handle) = OutputStream::try_from_device(click_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;

    let track_sink = rodio::Sink::try_new(&track_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    let click_sink = rodio::Sink::try_new(&click_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    track_sink.set_volume(track_volume);
    click_sink.set_volume(click_volume);

    track_sink.append(track_source);
    click_sink.append(click_source);

    let track_thread = thread::spawn(move || {
        track_sink.sleep_until_end();
    });

    let click_thread = thread::spawn(move || {
        click_sink.sleep_until_end();

    }); 
    

    track_thread.join().expect("track thread panicked");
    click_thread.join().expect("click thread panicked");

    Ok(())
}
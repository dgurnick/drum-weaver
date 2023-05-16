use std::fs::{metadata, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use rodio::*;
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
        .arg(Arg::new("music_folder").required(true).index(1).help("Where your music files are stored"))
        .arg(Arg::new("track").required(true).index(2).help("Position in the csv file to play"))
        .arg(Arg::new("track_volume").required(false).default_value("100").index(3))
        .arg(Arg::new("click_volume").required(false).default_value("100").index(4))
        .arg(Arg::new("track_device").required(false).default_value("1").index(5))
        .arg(Arg::new("click_device").required(false).default_value("1").index(6))
        .arg(Arg::new("click_delay").required(false).default_value("0").index(7))
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

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
                    return Err("Failed to find file or 7z archive".to_string());
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

fn check_file_existence(folder_path: &str, file_name: &str) -> Result<(), String> {
    let mut path = PathBuf::new();
    path.push(folder_path);
    path.push(file_name);

    if let Err(_) = metadata(&path) {
        return Err(format!("File '{}' does not exist", path.display()));
    }
    Ok(())
}

fn run(matches: &ArgMatches) -> Result<i32, String> {

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
    let click_delay = matches.get_one::<String>("click_delay")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
        .unwrap_or(0);

    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
    if available_devices.is_empty() {
        return Err("No output devices found".to_string());
    }

    // Check if the device positions are valid
    let num_devices = available_devices.len();
    if track_device_position > num_devices || click_device_position > num_devices {
        return Err("Invalid device position".to_string());
    }

    let track = File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
    let click = File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

    let track_source = Decoder::new(BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
    let click_source = Decoder::new(BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;
    let track_source_amplify = track_source.amplify(track_volume);
    let click_source_amplify = click_source.amplify(click_volume);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let combined_source = track_source_amplify.mix(click_source_amplify);

    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(combined_source);
    sink.sleep_until_end();

/*     track_sink.append(track_source);
    click_sink.append(click_source);

    let track_thread = thread::spawn(move || {
        track_sink.sleep_until_end();
    });

    let click_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(click_delay as u64));
        click_sink.sleep_until_end();

    }); 
    
    track_thread.join().expect("track thread panicked");
    click_thread.join().expect("click thread panicked"); */

    Ok(0)

}

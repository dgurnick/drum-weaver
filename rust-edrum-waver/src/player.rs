use std::println;
use std::{
    fs::File,
    io::BufReader,
    thread,
};
use rodio::*;
use rodio::source::Amplify;
use rodio::cpal::traits::{HostTrait};
use crate::common::PlayerArguments;
use crate::common::get_file_paths;

pub fn play_combined(track_source: Amplify<Decoder<BufReader<File>>>, click_source: Amplify<Decoder<BufReader<File>>>, device: &Device) -> Result<(), String> {
    let (_stream, stream_handle) = OutputStream::try_from_device(&device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let combined_source = track_source.mix(click_source);

    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(combined_source);
    sink.sleep_until_end();
    Ok(())
}

pub fn play_separate(track_source: Amplify<Decoder<BufReader<File>>>, click_source: Amplify<Decoder<BufReader<File>>>, track_device: &Device, click_device: &Device, track_volume: f32, click_volume: f32) -> Result<(), String> {
    
    let (_track_stream, track_stream_handle) = OutputStream::try_from_device(track_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;
    let (_click_stream, click_stream_handle) = OutputStream::try_from_device(click_device).map_err(|e| format!("Failed to create track output stream: {}", e))?;

    let track_sink = Sink::try_new(&track_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    let click_sink = Sink::try_new(&click_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
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

pub fn run_cli(arguments: PlayerArguments) -> Result<(), String>{
    let host = cpal::default_host();

    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
    if available_devices.is_empty() {
        println!("No output devices found");
        return Err("No output devices found".to_string());
    }

    // Check if the device positions are valid
    let num_devices = available_devices.len();
    if arguments.track_device_position > num_devices {
        println!("Invalid track output device");
        return Err("Invalid track output device".to_string());
    }
    if arguments.click_device_position > num_devices {
        println!("Invalid click output device");
        return Err("Invalid click output device".to_string());
    }

    let (track_path_str, click_path_str) = get_file_paths(&arguments.music_folder, arguments.track_position);
    
    let track = File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
    let click = File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

    let track_source = Decoder::new(BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
    let click_source = Decoder::new(BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;
    let track_source_amplify = track_source.amplify(arguments.track_volume);
    let click_source_amplify = click_source.amplify(arguments.click_volume);

    if arguments.combined {
        play_combined(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[arguments.track_device_position]
        )
    
    } else {
        play_separate(
            track_source_amplify, 
            click_source_amplify, 
            &available_devices[arguments.track_device_position],
            &available_devices[arguments.click_device_position],
            arguments.track_volume,
            arguments.click_volume
        )
    }
}
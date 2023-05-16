use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream};
use std::thread;
use clap::{Arg, ArgMatches};

fn main() {

    let matches = clap::Command::new("eDrums Wav Player")
        .version("0.1")
        .arg(Arg::new("track").required(true).index(1))
        .arg(Arg::new("click").required(true).index(2))
        .arg(Arg::new("click_volume").required(false).default_value("1").index(3))
        .get_matches();

    if let Err(err) = run(&matches) {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

fn run(matches: &ArgMatches) -> Result<i32, String> {

    let track_path_str = matches.get_one::<String>("track").expect("No track file provided");
    let click_path_str = matches.get_one::<String>("click").expect("No click file provided");
    let click_volume = matches.get_one::<String>("click_volume")
        .unwrap_or(&"1.0".to_string())
        .parse::<f32>()
        .unwrap_or(1.0);

    let track = File::open(track_path_str).map_err(|e| format!("Failed to open track file: {}", e))?;
    let click = File::open(click_path_str).map_err(|e| format!("Failed to open click file: {}", e))?;

    let (_track_stream, track_stream_handle) = OutputStream::try_default().unwrap();
    let (_click_stream, click_stream_handle) = OutputStream::try_default().unwrap();
    

    let track_sink = rodio::Sink::try_new(&track_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    let click_sink = rodio::Sink::try_new(&click_stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;
    click_sink.set_volume(click_volume);

    let track_source = Decoder::new(BufReader::new(track)).map_err(|e| format!("Failed to decode track file: {}", e))?;
    let click_source = Decoder::new(BufReader::new(click)).map_err(|e| format!("Failed to decode click file: {}", e))?;

    let track_thread = thread::spawn(move || {
        //track_stream_handle.play_raw(track_source.convert_samples()).unwrap();
        track_sink.append(track_source);
        track_sink.sleep_until_end();
        
    });

    let click_thread = thread::spawn(move || {
        //click_stream_handle.play_raw(click_source.convert_samples()).unwrap();
        click_sink.append(click_source);
        click_sink.sleep_until_end();
    });

    track_thread.join().expect("track thread panicked");
    click_thread.join().expect("click thread panicked");

    Ok(0)

}

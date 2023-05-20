use cpal::traits::{HostTrait};
use lazy_static::lazy_static;
use std::fs;
use std::sync::Mutex;

use crate::audio::{Player, Song};
use crate::common::{Error, PlayerArguments};
use crate::playlist::SongRecord;

lazy_static! {
    static ref SONGS: Mutex<Vec<SongRecord>> = Mutex::new(Vec::new());
}

pub fn import_songs() -> Result<Vec<SongRecord>, Error> {

    let mut songs = SONGS.lock().unwrap();

    if songs.is_empty() {
        let db_content = fs::read_to_string("assets/song_list.csv")?;
        let mut reader = csv::Reader::from_reader(db_content.as_bytes());

        for result in reader.deserialize() {
            let song: SongRecord = result?;
            songs.push(song);
        }
    }

    Ok(songs.clone())

}

pub fn play_song(arguments: PlayerArguments) -> Result<(Player, Player), String> {
    let host = cpal::default_host();
    let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    let track_device = &available_devices[arguments.track_device_position];
    let click_device = &available_devices[arguments.click_device_position];

    let track_play = Player::new(None, track_device).expect("Could not create track player");
    let click_play = Player::new(None, click_device).expect("Could not create click player");

    track_play.set_playback_speed(arguments.playback_speed);
    click_play.set_playback_speed(arguments.playback_speed);

    let track_volume = Some(arguments.track_volume);
    let click_volume = Some(arguments.click_volume);

    let track_song = Song::from_file(arguments.track_song, track_volume).expect("Could not create track song");
    let click_song = Song::from_file(arguments.click_song, click_volume).expect("Could not create click song");

    track_play.play_song_now(&track_song, None).expect("Could not play track song");
    click_play.play_song_now(&click_song, None).expect("Could not play click song");

    Ok((track_play, click_play))
    
    // while track_play.has_current_song() && click_play.has_current_song() {
    //     std::thread::sleep(std::time::Duration::from_secs(1));

    //     // let (track_samples, track_position) = track_play.get_playback_position().expect("Could not get track playback position");
    //     // let (click_samples, click_position) = click_play.get_playback_position().expect("Could not get click playback position");

    //     // println!("Track: {}/{} Click: {}/{}", 
    //     //     track_position.as_secs(), 
    //     //     track_samples.as_secs(), 
    //     //     click_position.as_secs(), 
    //     //     click_samples.as_secs());
        
    // }

}
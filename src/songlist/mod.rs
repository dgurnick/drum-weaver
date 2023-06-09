use lazy_static::lazy_static;
use std::fs;
use std::sync::Mutex;

use crate::common::Error;
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

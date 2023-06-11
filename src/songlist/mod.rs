use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::common::Error;
use crate::playlist::SongRecord;

lazy_static! {
    static ref SONGS: Mutex<Vec<SongRecord>> = Mutex::new(Vec::new());
}

pub fn import_songs() -> Result<Vec<SongRecord>, Error> {
    let mut songs = SONGS.lock().unwrap();

    if songs.is_empty() {
        let file_contents: &str = include_str!("../../assets/song_list.csv");

        let mut reader = csv::Reader::from_reader(file_contents.as_bytes());

        for result in reader.deserialize() {
            let song: SongRecord = result?;
            songs.push(song);
        }
    }

    Ok(songs.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_songs_empty() {
        // Clear the SONGS mutex before running the test
        SONGS.lock().unwrap().clear();

        // Call the import_songs function
        let result = import_songs();

        // Ensure the result is Ok
        assert!(result.is_ok());

        // Ensure the songs vector is not empty
        let songs = result.unwrap();
        assert!(!songs.is_empty());
    }

    #[test]
    fn test_import_songs_non_empty() {
        // Add some dummy songs to the SONGS mutex before running the test
        let song1 = SongRecord {
            file_name: "song1.mp3".to_string(),
            genre: "Rock".to_string(),
            year: "2020".to_string(),
            artist: "Artist1".to_string(),
            title: "Song 1".to_string(),
            album: "Album 1".to_string(),
            length: "3:20".to_string(),
            bpm: 120,
            folder: "/music/rock".to_string(),
        };
        let song2 = SongRecord {
            file_name: "song2.mp3".to_string(),
            genre: "Pop".to_string(),
            year: "2018".to_string(),
            artist: "Artist2".to_string(),
            title: "Song 2".to_string(),
            album: "Album 2".to_string(),
            length: "4:20".to_string(),
            bpm: 100,
            folder: "/music/pop".to_string(),
        };

        SONGS.lock().unwrap().push(song1.clone());
        SONGS.lock().unwrap().push(song2.clone());

        // Call the import_songs function
        let result = import_songs();

        // Ensure the result is Ok
        assert!(result.is_ok());

        // Ensure the songs vector is the same as the one in the mutex
        let songs = result.unwrap();
        assert_eq!(songs.len(), 5448);
        assert!(songs.contains(&song1));
        assert!(songs.contains(&song2));
    }
}

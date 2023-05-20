use lazy_static::lazy_static;
use log::info;
use serde::{Serialize, Deserialize};
use std::fs;
use std::sync::Mutex;
use crate::songlist::import_songs;
use crate::common::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: usize,
    pub name: String,
    pub songs: Vec<SongRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(unused_variables)]
pub struct SongRecord {
    pub file_name: String,
    #[allow(dead_code)] pub genre: String,
    #[allow(dead_code)] pub year: String, 
    #[allow(dead_code)] pub artist: String,
    #[allow(dead_code)] pub song: String,
    #[allow(dead_code)] pub album: String,
    #[allow(dead_code)] pub length: String,
    #[allow(dead_code)] pub bpm: usize,
    pub folder: String,
}

lazy_static! {
    static ref PLAYLISTS: Mutex<Vec<Playlist>> = Mutex::new(Vec::new());
}
pub fn read_playlists() -> Result<Vec<Playlist>, Error> {
    let mut playlists = PLAYLISTS.lock().unwrap();

    if playlists.len() == 0 {
        let mut db_content = fs::read_to_string("assets/playlists.json")?;
        if db_content.is_empty() {
            info!("No playlists found, creating default playlist");
            db_content = create_default_playlist();

        } 

        let parsed: Vec<Playlist> = serde_json::from_str(&db_content)?;

        for playlist in parsed {
            playlists.push(playlist);
        }

    } 
    Ok(playlists.clone())
}

pub fn create_default_playlist() -> String {
    let mut playlists = Vec::new();
    let playlist = Playlist {
        id: 1,
        name: "All songs".to_string(),
        songs: Vec::new(),
    };
    playlists.push(playlist);

    // add all existing songs to the default playlist
    for song in import_songs().unwrap() {
        playlists[0].songs.push(song);
    }

    let serialized = serde_json::to_string(&playlists).unwrap();
    fs::write("assets/playlists.json", serialized.clone()).unwrap();
    return serialized;
}

#[allow(dead_code)]
pub fn create_playlist(name: String) {
    let mut playlists = read_playlists().unwrap();
    let id = playlists.len() + 1;
    let playlist = Playlist {
        id,
        name,
        songs: Vec::new(),
    };
    playlists.push(playlist);
    let serialized = serde_json::to_string(&playlists).unwrap();
    fs::write("assets/playlists.json", serialized).unwrap();

    // force reread after create
    let mut playlist_sync = PLAYLISTS.lock().unwrap();
    playlist_sync.clear();
}

#[allow(dead_code)]
pub fn delete_playlist(id: usize) -> Result<(), Error> {
    let mut playlists = read_playlists()?;
    playlists.retain(|playlist| playlist.id != id);
    let serialized = serde_json::to_string(&playlists)?;
    fs::write("assets/playlists.json", serialized)?;
    info!("Deleted playlist: {}", id);

    // force reread after delete
    let mut playlist_sync = PLAYLISTS.lock().unwrap();
    playlist_sync.clear();

    Ok(())
}

#[allow(dead_code)]
pub fn get_songs(playlist_id: usize) -> Vec<SongRecord> {
    let playlists_result = read_playlists();
    if let Ok(playlists) = playlists_result {
        if let Some(playlist) = playlists.get(playlist_id) {
            return playlist.songs.clone();
        }
    }
    Vec::new() // Return an empty vector if the playlist is not found or there was an error
}
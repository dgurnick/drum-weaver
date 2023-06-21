pub struct SongRecord {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub path: String,
    pub duration: i32,
    pub track: i32,
    pub year: i32,
    pub genre: String,
}

pub struct Library {
    pub path: String, // the root path of the library (i.e. ...\Drumless)
    pub songs: Vec<SongRecord>,
}

impl Library {
    pub fn new(path: String) -> Self {
        Library { path, songs: Vec::new() }
    }

    // Reload the library from the embedded CSV file
    pub fn load_csv(&mut self) {
        let file_contents: &str = include_str!("../../assets/song_list.csv");
    }
}

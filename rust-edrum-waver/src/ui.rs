use std::sync::{Arc, Mutex};

use cpal::{traits::HostTrait, Device};
use eframe;
use egui;
use egui_extras::{TableBuilder, Column};
use log::{info, LevelFilter};

use crate::{
    common::{PlayerArguments, get_file_paths, read_devices, DeviceDetail}, 
    songlist::import_songs, 
    playlist::SongRecord, 
    audio::{Player, Song}
};

pub fn run_ui(arguments: &mut PlayerArguments) -> Result<i32, String> {

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    let mut app = MyApp::default();
    app.ui_device_volume = (&arguments.track_volume * 100.0).round() as usize;
    app.ui_click_volume = (&arguments.click_volume * 100.0).round() as usize;
    app.actual_click_volume = arguments.click_volume;
    app.actual_device_volume = arguments.track_volume;
    app.songs = import_songs().expect("Could not import songs");

    let _ = match eframe::run_native(
        "Drummer's Karaoke",
        options,
        Box::new(move |_cc| Box::new(app.clone())),
    ) {
        Ok(_) => Ok(0),
        Err(e) => Err(e.to_string())
    };

    Ok(0)

}


#[derive(Clone)]
struct MyApp {

    ui_device_volume: usize,
    ui_click_volume: usize,
    actual_device_volume: f32,
    actual_click_volume: f32,
    search_for: String,
    songs: Vec<SongRecord>,
    filtered_songs: Vec<SongRecord>,
    track_play: Option<Arc<Mutex<Player>>>,
    click_play: Option<Arc<Mutex<Player>>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            ui_device_volume: 100,
            ui_click_volume: 100,
            actual_device_volume: 1.0,
            actual_click_volume: 1.0,
            search_for: String::new(),
            songs: Vec::new(),
            filtered_songs: Vec::new(),
            track_play: None,
            click_play: None,
        }
    }
}

trait MusicPlayer {
    fn play(&mut self, arguments: PlayerArguments, song: SongRecord);
}

impl MusicPlayer for MyApp {

    fn play(&mut self, arguments: PlayerArguments, song: SongRecord) {
        info!("Play song: {}", song.song);

        let (track_file, click_file) = get_file_paths(&arguments.music_folder, song.id);
        info!("Track file: {}", track_file);
        info!("Click file: {}", click_file);

        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();
        let track_device = &available_devices[arguments.track_device_position];
        let click_device = &available_devices[arguments.click_device_position];

        let track_volume = Some(arguments.track_volume);
        let click_volume = Some(arguments.click_volume);

        let track_song = Song::from_file(track_file, track_volume).expect("Could not create track song");
        let click_song = Song::from_file(click_file, click_volume).expect("Could not create click song");

        info!("Songs are loaded. Grabbing players");
        // Create the player instances and retain ownership using Arc<Mutex>
        let track_play = Arc::new(Mutex::new(Player::new(None, track_device).expect("Could not create track player")));
        let click_play = Arc::new(Mutex::new(Player::new(None, click_device).expect("Could not create click player")));

        // Store the Arc<Mutex<Player>> instances in MyApp
        self.track_play = Some(track_play.clone());
        self.click_play = Some(click_play.clone());

        info!("Waiting to lock player");
        {
            // Acquire the lock and play the track song
            let mut track_play_guard = track_play.lock().unwrap();
            let track_play_ref = &mut *track_play_guard;

            // Borrow the track_song within the lock
            let track_song_ref = &track_song;
            track_play_ref.set_playback_speed(arguments.playback_speed);
            track_play_ref.play_song_now(track_song_ref, None).expect("Could not play track song");
        }

        {
            // Acquire the lock and play the click song
            let mut click_play_guard = click_play.lock().unwrap();
            let click_play_ref = &mut *click_play_guard;

            // Borrow the click_song within the lock
            let click_song_ref = &click_song;
            click_play_ref.set_playback_speed(arguments.playback_speed);
            click_play_ref.play_song_now(click_song_ref, None).expect("Could not play click song");
        }

        info!("Music is playing");
    }
}

impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("Drummer's Karaoke");
            ui.separator();

            ui.horizontal(|ui| {

                ui.label(format!("Device Volume: {}" , self.actual_device_volume));
                let device_volume_slider = egui::Slider::new(&mut self.ui_device_volume, 0..=100);
                if ui.add(device_volume_slider).changed() {
                    self.actual_device_volume = (self.ui_device_volume as f32) / 100.0;
                }

                ui.label(format!("Click Volume: {}" , self.actual_click_volume));
                let click_volume_slider = egui::Slider::new(&mut self.ui_click_volume, 0..=100);
                if ui.add(click_volume_slider).changed() {
                    self.actual_click_volume = (self.ui_click_volume as f32) / 100.0;
                }
                

            });
        
            ui.horizontal(|ui| {
                let search_label = ui.label("Find a song: ");

                if ui.text_edit_singleline(&mut self.search_for).labelled_by(search_label.id).changed() {
                    if self.search_for.is_empty() {
                        self.filtered_songs = Vec::new();
                        return;
                    }
                    self.filtered_songs = filter_songs(self.songs.clone(), self.search_for.clone());
                    
                }

            });
            ui.separator();
            
            ui.allocate_ui(ui.available_size(), |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto_with_initial_suggestion(200.0))
                    .column(Column::remainder())
                    .column(Column::auto_with_initial_suggestion(50.0))
                    .header(0.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Artist");
                        });
                        header.col(|ui| {
                            ui.heading("Title");
                        });
                        header.col(|ui| {
                        
                        });

                    })
                    .body( |mut body| {
                        let songs = if self.filtered_songs.is_empty() {
                            &self.songs
                        } else {
                            &self.filtered_songs
                        };

                        for song in songs {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(song.artist.clone());
                                });
                                row.col(|ui| {
                                    ui.label(song.song.clone());
                                });
                                row.col(|ui| {
                                    if ui.button("Play").clicked() {
                                        play_song(song.clone());
                                    }

                                });
                            });

                        }

                    });
            
            });

        });
    }
}

fn filter_songs(songs: Vec<SongRecord>, filter: String) -> Vec<SongRecord> {
    let mut filtered_songs = Vec::new();

    for song in songs {
        if song.artist.contains(&filter) || song.song.contains(&filter) {
            filtered_songs.push(song);
        }
    }

    filtered_songs
}

fn play_song(song: SongRecord) {
    info!("Play song: {}", song.song);
    
}   
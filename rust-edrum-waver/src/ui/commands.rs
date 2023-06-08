use std::time::Duration;

use crate::device::read_devices;
use crate::playlist::SongRecord;
use crate::ui::App;
use crate::ui::AppConfig;
use crate::ui::MenuItem;
use crate::ui::Player;
use crossterm::terminal::disable_raw_mode;
use log::info;
use rand::seq::SliceRandom;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use ratatui::widgets::TableState;
use ratatui::Terminal;

#[rustfmt::skip]
pub trait KeyHandler {
    fn do_reset_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_select_next_item( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, songlist_state: &mut TableState, );
    fn do_select_previous_item( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, songlist_state: &mut TableState, );
    fn do_pause_playback( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_restart_song( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_select_next_page( &mut self, active_menu_item: &mut MenuItem, songlist_state: &mut TableState, );
    fn do_select_previous_page( &mut self, active_menu_item: &mut MenuItem, songlist_state: &mut TableState, );
    fn do_reduce_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_increase_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_reduce_track_volume( &mut self, track_player: &mut Player);
    fn do_reset_track_volume( &mut self, track_player: &mut Player);
    fn do_increase_track_volume( &mut self, track_player: &mut Player);
    fn do_reduce_click_volume( &mut self, click_player: &mut Player);
    fn do_reset_click_volume( &mut self, click_player: &mut Player);
    fn do_increase_click_volume( &mut self, click_player: &mut Player);
    fn do_add_song_to_playlist( &mut self, songlist_state: &mut TableState);
    fn do_remove_song_from_playlist( &mut self, songlist_state: &mut TableState);
    fn do_check_quit( &mut self, track_player: &mut Player, click_player: &mut Player, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>);
    fn do_check_stay_or_next( &mut self, track_player: &mut Player, click_player: &mut Player);
    fn do_start_search( &mut self );
    fn do_shuffle_songs( &mut self );
    fn do_delete_track( &mut self, songlist_state: &mut TableState, track_player: &mut Player, click_player: &mut Player);
    fn do_cancel_search( &mut self );
    fn do_clear_playlist( &mut self );
    fn do_shuffle_playlist( &mut self );
    fn do_start_playlist( &mut self );
}

#[rustfmt::enable]
impl KeyHandler for App {
    fn do_reset_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                track_player.set_playback_speed(1.0);
                click_player.set_playback_speed(1.0);
                info!("Reset playback speed to 1x ");
            }
            _ => {}
        }
    }

    fn do_select_next_item(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Devices => {
                if let Some(selected) = device_list_state.selected() {
                    let amount_devices = read_devices().len();
                    if selected >= amount_devices - 1 {
                        device_list_state.select(Some(0));
                    } else {
                        device_list_state.select(Some(selected + 1));
                    }
                }
                info!("Set device to {}", device_list_state.selected().unwrap());
            }
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    #[allow(unused_assignments)]
                    let mut new_position = selected;
                    if selected >= amount_songs - 1 {
                        new_position = 0;
                    } else {
                        new_position = selected + 1;
                    }
                    songlist_state.select(Some(new_position));
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn do_select_previous_item(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Devices => {
                if let Some(selected) = device_list_state.selected() {
                    let amount_devices = read_devices().len();
                    if selected > 0 {
                        device_list_state.select(Some(selected - 1));
                    } else {
                        device_list_state.select(Some(amount_devices - 1));
                    }
                }
                info!("Set device to {}", device_list_state.selected().unwrap());
            }
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    #[allow(unused_assignments)]
                    let mut new_position = 0;
                    if selected > 0 {
                        new_position = selected - 1;
                    } else {
                        new_position = amount_songs - 1;
                    }
                    songlist_state.select(Some(new_position));
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn do_pause_playback(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                track_player.set_playing(!track_player.is_playing());
                click_player.set_playing(!click_player.is_playing());

                info!("Stopped playback of song");
            }
            _ => {}
        }
    }

    fn do_restart_song(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if track_player.is_playing() {
                    track_player.seek(Duration::from_secs(0));
                    click_player.seek(Duration::from_secs(0));
                }
                info!("Restarted song");
            }
            _ => {}
        }
    }

    fn do_select_next_page(
        &mut self,
        active_menu_item: &mut MenuItem,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected + 10 > amount_songs {
                        songlist_state.select(Some(0));
                    } else {
                        songlist_state.select(Some(selected + 10));
                    }
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn do_select_previous_page(
        &mut self,
        active_menu_item: &mut MenuItem,
        songlist_state: &mut TableState,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                if let Some(selected) = songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected > 10 {
                        songlist_state.select(Some(selected - 10));
                    } else {
                        songlist_state.select(Some(amount_songs - 1));
                    }
                }
                info!("Set song to {}", songlist_state.selected().unwrap());
            }
            _ => {}
        }
    }

    fn do_reduce_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                let current_speed = track_player.get_playback_speed();
                if current_speed > 0.1 {
                    track_player.set_playback_speed(current_speed - 0.01);
                    click_player.set_playback_speed(current_speed - 0.01);
                }

                info!(
                    "Set playback speed to {}",
                    track_player.get_playback_speed()
                );
            }
            _ => {}
        }
    }

    fn do_increase_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        match active_menu_item {
            MenuItem::Songs => {
                let current_speed = track_player.get_playback_speed();
                if current_speed > 0.1 {
                    track_player.set_playback_speed(current_speed + 0.01);
                    click_player.set_playback_speed(current_speed + 0.01);
                }

                info!(
                    "Set playback speed to {}",
                    track_player.get_playback_speed()
                );
            }
            _ => {}
        }
    }

    fn do_reduce_track_volume(&mut self, track_player: &mut Player) {
        if self.track_volume > 0 {
            self.track_volume = self.track_volume - 1;
        }

        track_player.set_volume_adjustment(self.track_volume as f32 / 100.0);
    }

    fn do_reset_track_volume(&mut self, track_player: &mut Player) {
        self.track_volume = 100;
        track_player.set_volume_adjustment(1.0);
    }

    fn do_increase_track_volume(&mut self, track_player: &mut Player) {
        self.track_volume = self.track_volume + 1;
        if self.track_volume > 200 {
            self.track_volume = 200;
        }

        track_player.set_volume_adjustment(self.track_volume as f32 / 100.0);
    }

    fn do_reduce_click_volume(&mut self, click_player: &mut Player) {
        if self.click_volume > 0 {
            self.click_volume = self.click_volume - 1;
        }

        click_player.set_volume_adjustment(self.click_volume as f32 / 100.0);
    }

    fn do_reset_click_volume(&mut self, click_player: &mut Player) {
        self.click_volume = 100;
        click_player.set_volume_adjustment(1.0);
    }

    fn do_increase_click_volume(&mut self, click_player: &mut Player) {
        self.click_volume = self.click_volume + 1;
        if self.click_volume > 200 {
            self.click_volume = 200;
        }

        click_player.set_volume_adjustment(self.click_volume as f32 / 100.0);
    }

    fn do_add_song_to_playlist(&mut self, songlist_state: &mut TableState) {
        if let Some(selected) = songlist_state.selected() {
            // add it to the queue. We can keep adding. No issue.
            let song = self.songs[selected].clone();
            let position = self
                .current_playlist
                .values()
                .position(|song_record| song_record.title == song.title);

            if position.is_none() {
                self.current_playlist
                    .insert(self.current_playlist.len() + 1, song.clone());
                info!("Added song to queue: {}", &song.title);
            }

            self.reindex_playlist();
        }
    }

    fn do_remove_song_from_playlist(&mut self, songlist_state: &mut TableState) {
        if let Some(selected) = songlist_state.selected() {
            // add it to the queue. We can keep adding. No issue.
            let song = self.songs[selected].clone();
            let position = self
                .current_playlist
                .values()
                .position(|song_record| song_record.title == song.title);

            if let Some(pos) = position {
                self.current_playlist.remove(&(&pos + 1));
                info!("Removed song from queue: {}", song.title);
            }

            self.reindex_playlist();
        }
    }

    fn do_check_quit(
        &mut self,
        track_player: &mut Player,
        click_player: &mut Player,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) {
        if self.is_quitting {
            info!("Quitting");
            track_player.stop();
            click_player.stop();
            disable_raw_mode().expect("Can not disable raw mode");
            terminal.clear().expect("Failed to clear the terminal");
            terminal.show_cursor().expect("Failed to show cursor");

            let track_device_name = read_devices()[self.track_device_idx].name.clone();
            let click_device_name = read_devices()[self.click_device_idx].name.clone();

            let config = AppConfig {
                track_device_name: Some(track_device_name),
                click_device_name: Some(click_device_name),
                ..Default::default()
            };
            confy::store("drum-weaver", None, config.clone())
                .expect("Unable to save configuration");
            println!("Stored config {}", config.clone());

            std::process::exit(0);
        }
    }

    fn do_check_stay_or_next(&mut self, track_player: &mut Player, click_player: &mut Player) {
        if self.is_quitting {
            self.is_quitting = false;
        } else {
            track_player.skip();
            click_player.skip();
        }
    }

    fn do_start_search(&mut self) {
        self.searching_for = String::new();
        self.is_searching = true;
    }

    fn do_shuffle_songs(&mut self) {
        if self.is_playing_random {
            self.songs = self.original_songs.clone();
        } else {
            self.songs.shuffle(&mut rand::thread_rng());
        }
        self.is_playing_random = !self.is_playing_random;
    }

    fn do_delete_track(
        &mut self,
        songlist_state: &mut TableState,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        if let Some(selected) = songlist_state.selected() {
            track_player.stop();
            click_player.stop();

            self.songs.remove(selected);
        }
    }

    fn do_cancel_search(&mut self) {
        self.songs = self.original_songs.clone();
    }

    fn do_clear_playlist(&mut self) {
        self.current_playlist.clear();
        self.current_playlist_idx = 0;
    }

    fn do_shuffle_playlist(&mut self) {
        // Convert the BTreeMap into a vector of key-value pairs
        let mut playlist_vec: Vec<(usize, SongRecord)> =
            self.current_playlist.clone().into_iter().collect();
        self.current_playlist.clear();

        // Shuffle the vector using the Fisher-Yates algorithm
        let mut rng = rand::thread_rng();
        playlist_vec.shuffle(&mut rng);

        for (idx, song) in playlist_vec.into_iter().enumerate() {
            self.current_playlist.insert(idx, song.1);
        }
    }

    fn do_start_playlist(&mut self) {
        self.is_playing = true;
    }
}

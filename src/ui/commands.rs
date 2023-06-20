use std::collections::BTreeMap;
use std::time::Duration;

use crate::device::read_devices;
use crate::playlist::SongRecord;
use crate::ui::ActiveFocus;
use crate::ui::App;
use crate::ui::AppConfig;
use crate::ui::MenuItem;
use crate::ui::Player;
use crossterm::terminal::disable_raw_mode;
use log::info;
use rand::seq::SliceRandom;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use ratatui::Terminal;

#[derive(PartialEq)]
pub enum SortBy {
    Artist,
    Title,
    Album,
    Genre,
    Duration,
}

#[rustfmt::skip]
pub trait KeyHandler {
    fn do_reset_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_select_next_item( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, );
    fn do_select_previous_item( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, );
    fn do_pause_playback( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_restart_song( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_select_next_page( &mut self, active_menu_item: &mut MenuItem, );
    fn do_select_previous_page( &mut self, active_menu_item: &mut MenuItem, );
    fn do_reduce_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_increase_playback_speed( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn do_reduce_track_volume( &mut self, track_player: &mut Player);
    fn do_reset_track_volume( &mut self, track_player: &mut Player);
    fn do_increase_track_volume( &mut self, track_player: &mut Player);
    fn do_reduce_click_volume( &mut self, click_player: &mut Player);
    fn do_reset_click_volume( &mut self, click_player: &mut Player);
    fn do_increase_click_volume( &mut self, click_player: &mut Player);
    fn do_add_song_to_playlist( &mut self);
    fn do_remove_song_from_playlist( &mut self);
    fn do_check_quit( &mut self, track_player: &mut Player, click_player: &mut Player, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>);
    fn do_check_stay_or_next( &mut self, track_player: &mut Player, click_player: &mut Player);
    fn do_start_search( &mut self );
    fn do_shuffle_songs( &mut self );
    fn do_delete_track( &mut self, track_player: &mut Player, click_player: &mut Player);
    fn do_cancel_search( &mut self );
    fn do_clear_playlist( &mut self );
    fn do_shuffle_playlist( &mut self );
    fn do_start_playlist( &mut self  );
    fn do_sort( &mut self, sort_by: SortBy );
}

impl KeyHandler for App {
    fn do_reset_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        if let MenuItem::Songs = active_menu_item {
            track_player.set_playback_speed(1.0);
            click_player.set_playback_speed(1.0);
            info!("Reset playback speed to 1x ");
        }
    }

    fn do_select_next_item(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
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
                if self.active_focus == ActiveFocus::Songs {
                    if let Some(selected) = self.songlist_state.selected() {
                        let amount_songs = self.songs.len();
                        #[allow(unused_assignments)]
                        let mut new_position = selected;
                        if selected >= amount_songs - 1 {
                            new_position = 0;
                        } else {
                            new_position = selected + 1;
                        }
                        self.songlist_state.select(Some(new_position));
                    }
                } else if let Some(selected) = self.queue_state.selected() {
                    let amount_songs = self.queue.len();
                    #[allow(unused_assignments)]
                    let mut new_position = selected;
                    if selected >= amount_songs - 1 {
                        new_position = 0;
                    } else {
                        new_position = selected + 1;
                    }
                    self.queue_state.select(Some(new_position));
                }
            }
            _ => {}
        }
    }

    fn do_select_previous_item(
        &mut self,
        active_menu_item: &mut MenuItem,
        device_list_state: &mut ListState,
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
                if self.active_focus == ActiveFocus::Songs {
                    if let Some(selected) = self.songlist_state.selected() {
                        let amount_songs = self.songs.len();
                        #[allow(unused_assignments)]
                        let mut new_position = 0;
                        if selected > 0 {
                            new_position = selected - 1;
                        } else {
                            new_position = amount_songs - 1;
                        }
                        self.songlist_state.select(Some(new_position));
                    }
                } else if let Some(selected) = self.queue_state.selected() {
                    let amount_songs = self.queue.len();
                    #[allow(unused_assignments)]
                    let mut new_position = 0;
                    if selected > 0 {
                        new_position = selected - 1;
                    } else {
                        new_position = amount_songs - 1;
                    }
                    self.queue_state.select(Some(new_position));
                }
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
        if let MenuItem::Songs = active_menu_item {
            track_player.set_playing(!track_player.is_playing());
            click_player.set_playing(!click_player.is_playing());

            info!("Stopped playback of song");
        }
    }

    fn do_restart_song(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        if let MenuItem::Songs = active_menu_item {
            if track_player.is_playing() {
                track_player.seek(Duration::from_secs(0));
                click_player.seek(Duration::from_secs(0));
            }
            info!("Restarted song");
        }
    }

    fn do_select_next_page(&mut self, active_menu_item: &mut MenuItem) {
        if let MenuItem::Songs = active_menu_item {
            if self.active_focus == ActiveFocus::Songs {
                if let Some(selected) = self.songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected + 10 > amount_songs {
                        self.songlist_state.select(Some(0));
                    } else {
                        self.songlist_state.select(Some(selected + 10));
                    }
                }
            } else if let Some(selected) = self.queue_state.selected() {
                let amount_songs = self.queue.len();
                if selected + 10 > amount_songs {
                    self.queue_state.select(Some(0));
                } else {
                    self.queue_state.select(Some(selected + 10));
                }
            }
        }
    }

    fn do_select_previous_page(&mut self, active_menu_item: &mut MenuItem) {
        if let MenuItem::Songs = active_menu_item {
            if self.active_focus == ActiveFocus::Songs {
                if let Some(selected) = self.songlist_state.selected() {
                    let amount_songs = self.songs.len();
                    if selected > 10 {
                        self.songlist_state.select(Some(selected - 10));
                    } else {
                        self.songlist_state.select(Some(amount_songs - 1));
                    }
                }
            } else if let Some(selected) = self.queue_state.selected() {
                let amount_songs = self.queue.len();
                if selected > 10 {
                    self.queue_state.select(Some(selected - 10));
                } else {
                    self.queue_state.select(Some(amount_songs - 1));
                }
            }
        }
    }

    fn do_reduce_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        if let MenuItem::Songs = active_menu_item {
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
    }

    fn do_increase_playback_speed(
        &mut self,
        active_menu_item: &mut MenuItem,
        track_player: &mut Player,
        click_player: &mut Player,
    ) {
        if let MenuItem::Songs = active_menu_item {
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
    }

    fn do_reduce_track_volume(&mut self, track_player: &mut Player) {
        if self.track_volume > 0 {
            self.track_volume -= 1;
        }

        track_player.set_volume_adjustment(self.track_volume as f32 / 100.0);
    }

    fn do_reset_track_volume(&mut self, track_player: &mut Player) {
        self.track_volume = 100;
        track_player.set_volume_adjustment(1.0);
    }

    fn do_increase_track_volume(&mut self, track_player: &mut Player) {
        self.track_volume += 1;
        if self.track_volume > 200 {
            self.track_volume = 200;
        }

        track_player.set_volume_adjustment(self.track_volume as f32 / 100.0);
    }

    fn do_reduce_click_volume(&mut self, click_player: &mut Player) {
        if self.click_volume > 0 {
            self.click_volume -= 1;
        }

        click_player.set_volume_adjustment(self.click_volume as f32 / 100.0);
    }

    fn do_reset_click_volume(&mut self, click_player: &mut Player) {
        self.click_volume = 100;
        click_player.set_volume_adjustment(1.0);
    }

    fn do_increase_click_volume(&mut self, click_player: &mut Player) {
        self.click_volume += 1;
        if self.click_volume > 200 {
            self.click_volume = 200;
        }

        click_player.set_volume_adjustment(self.click_volume as f32 / 100.0);
    }

    fn do_add_song_to_playlist(&mut self) {
        if let Some(selected) = self.songlist_state.selected() {
            // add it to the queue. We can keep adding. No issue.
            let song = self.songs[selected].clone();
            let position = self
                .queue
                .values()
                .position(|song_record| song_record.title == song.title);

            if position.is_none() {
                self.queue.insert(self.queue.len() + 1, song.clone());
                info!("Added song to queue: {}", &song.title);
            }

            self.reindex_playlist();
        }
    }

    fn do_remove_song_from_playlist(&mut self) {
        if let Some(selected) = self.songlist_state.selected() {
            // add it to the queue. We can keep adding. No issue.
            let song = self.songs[selected].clone();
            let position = self
                .queue
                .values()
                .position(|song_record| song_record.title == song.title);

            if let Some(pos) = position {
                self.queue.remove(&(&pos + 1));
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
            };
            confy::store("drum-weaver", None, config.clone())
                .expect("Unable to save configuration");
            println!("Stored config {}", config);

            let playlist_str: BTreeMap<String, SongRecord> = self
                .queue
                .clone()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();

            // Save playlist using confy
            confy::store("drum-weaver", "playlist", playlist_str).expect("Failed to save playlist");

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

    fn do_delete_track(&mut self, track_player: &mut Player, click_player: &mut Player) {
        if let Some(selected) = self.songlist_state.selected() {
            track_player.stop();
            click_player.stop();

            self.songs.remove(selected);
        }
    }

    fn do_cancel_search(&mut self) {
        self.songs = self.original_songs.clone();
    }

    fn do_clear_playlist(&mut self) {
        self.queue.clear();
        self.active_queue_idx = 0;
    }

    fn do_shuffle_playlist(&mut self) {
        // Convert the BTreeMap into a vector of key-value pairs
        let mut playlist_vec: Vec<(usize, SongRecord)> = self.queue.clone().into_iter().collect();
        self.queue.clear();

        // Shuffle the vector using the Fisher-Yates algorithm
        let mut rng = rand::thread_rng();
        playlist_vec.shuffle(&mut rng);

        for (idx, song) in playlist_vec.into_iter().enumerate() {
            if let Some(track_file) = self.track_file.clone() {
                if track_file.contains(song.1.file_name.as_str()) {
                    self.active_queue_idx = idx;
                    self.songlist_state.select(Some(idx));
                }
            }
            self.queue.insert(idx, song.1);
        }
    }

    fn do_start_playlist(&mut self) {
        self.is_playing = true;
    }

    fn do_sort(&mut self, sort_by: SortBy) {
        if self.last_sort.is_some() && self.last_sort.as_ref().unwrap() == &sort_by {
            self.sort_reverse = !self.sort_reverse;
        } else {
            self.sort_reverse = false;
        }

        if self.sort_reverse {
            match sort_by {
                SortBy::Title => self.songs.sort_by(|b, a| a.title.cmp(&b.title)),
                SortBy::Artist => self.songs.sort_by(|b, a| a.artist.cmp(&b.artist)),
                SortBy::Album => self.songs.sort_by(|b, a| a.album.cmp(&b.album)),
                SortBy::Genre => self.songs.sort_by(|b, a| a.genre.cmp(&b.genre)),
                SortBy::Duration => self
                    .songs
                    .sort_by(|b, a| duration_cmp(&a.length, &b.length)),
            }
        } else {
            match sort_by {
                SortBy::Title => self.songs.sort_by(|a, b| a.title.cmp(&b.title)),
                SortBy::Artist => self.songs.sort_by(|a, b| a.artist.cmp(&b.artist)),
                SortBy::Album => self.songs.sort_by(|a, b| a.album.cmp(&b.album)),
                SortBy::Genre => self.songs.sort_by(|a, b| a.genre.cmp(&b.genre)),
                SortBy::Duration => self
                    .songs
                    .sort_by(|a, b| duration_cmp(&a.length, &b.length)),
            }
        }

        self.last_sort = Some(sort_by);
    }
}

fn duration_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<_> = a.split(':').collect();
    let b_parts: Vec<_> = b.split(':').collect();

    let a_minutes: u32 = a_parts[0].parse().unwrap_or(0);
    let a_seconds: u32 = a_parts[1].parse().unwrap_or(0);

    let b_minutes: u32 = b_parts[0].parse().unwrap_or(0);
    let b_seconds: u32 = b_parts[1].parse().unwrap_or(0);

    if a_minutes == b_minutes {
        a_seconds.cmp(&b_seconds)
    } else {
        a_minutes.cmp(&b_minutes)
    }
}

use std::time::Duration;

use crate::device::read_devices;
use crate::ui::App;
use crate::ui::MenuItem;
use crate::ui::Player;
use log::info;
use ratatui::widgets::ListState;
use ratatui::widgets::TableState;

#[rustfmt::skip]
pub trait KeyHandler {
    fn handle_r_event( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn handle_down_event( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, songlist_state: &mut TableState, );
    fn handle_up_event( &mut self, active_menu_item: &mut MenuItem, device_list_state: &mut ListState, songlist_state: &mut TableState, );
    fn handle_space_event( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn handle_z_event( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn handle_page_down_event( &mut self, active_menu_item: &mut MenuItem, songlist_state: &mut TableState, );
    fn handle_page_up_event( &mut self, active_menu_item: &mut MenuItem, songlist_state: &mut TableState, );
    fn handle_left_arrow_event( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
    fn handle_right_arrow_event( &mut self, active_menu_item: &mut MenuItem, track_player: &mut Player, click_player: &mut Player, );
}

#[rustfmt::enable]
impl KeyHandler for App {
    fn handle_r_event(
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

    fn handle_down_event(
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

    fn handle_up_event(
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

    fn handle_space_event(
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

    fn handle_z_event(
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

    fn handle_page_down_event(
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

    fn handle_page_up_event(
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

    fn handle_left_arrow_event(
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

    fn handle_right_arrow_event(
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
}

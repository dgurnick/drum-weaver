use crate::ui::App;
use crate::ui::MenuItem;
use crate::ui::Player;
use log::info;

impl App {
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
}

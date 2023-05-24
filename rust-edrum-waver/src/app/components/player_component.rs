use super::AppComponent;
use crate::app::App;

pub struct PlayerComponent;

impl AppComponent for PlayerComponent {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            let _stop_btn = ui.button("■");
            let _play_btn = ui.button("▶");
            let _pause_btn = ui.button("⏸");
            let _prev_btn = ui.button("|◀");
            let _next_btn = ui.button("▶|");

            let mut volume = ctx.player.as_ref().unwrap().volume;
            ui.add(
                eframe::egui::Slider::new(&mut volume, (0.0 as f32)..=(5.0 as f32))
                    .logarithmic(false)
                    .show_value(false)
                    .clamp_to_range(true),
            );
            ctx.player.as_mut().unwrap().set_volume(volume);
        });
    }
}

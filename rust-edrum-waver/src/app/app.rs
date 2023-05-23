use eframe::egui;

use super::App;
use crate::app::components::{menu_bar::MenuBar, AppComponent};

impl eframe::App for App {
    fn update(&mut self, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.quit {
            tracing::info!("Exiting app");
            frame.close();
        }

        egui::TopBottomPanel::top("MusicPlayer").show(_ctx, |ui| {
            MenuBar::add(self, ui);
        });
    }
}

use super::App;
use eframe::egui;

impl eframe::App for App {
    fn update(&mut self, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.quit {
            tracing::info!("Exiting app");
            frame.close();
        }
    }
}

use eframe::egui;

use super::App;
use crate::app::components::{
    footer::Footer, menu_bar::MenuBar, player_component::PlayerComponent, song_list::SongList,
    AppComponent,
};

impl eframe::App for App {
    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        tracing::info!("Exiting the application");
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.quit {
            tracing::info!("Exiting app");
            frame.close();
        }

        ctx.request_repaint();

        if let Some(rx) = &self.library_receiver {
            match rx.try_recv() {
                Ok(library) => {
                    self.library = Some(library);
                }
                Err(_) => (),
            }
        }

        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            MenuBar::add(self, ui);
        });

        egui::TopBottomPanel::top("Player").show(ctx, |ui| {
            PlayerComponent::add(self, ui);
        });

        egui::TopBottomPanel::bottom("Fooder").show(ctx, |ui| {
            Footer::add(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(library) = &mut self.library {
                egui::ScrollArea::both().show(ui, |ui| {
                    SongList::add(self, ui);
                });
            }
        });
    }
}

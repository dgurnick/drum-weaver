use eframe::egui;
use eframe::egui::{FontFamily::Proportional, FontId, TextStyle::*};

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

        let mut style: egui::Style = (*ctx.style()).clone();

        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ]
        .into();

        ctx.set_style(style);

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
            if let Some(_) = &mut self.library {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    SongList::add(self, ui);
                });
            }
        });
    }
}

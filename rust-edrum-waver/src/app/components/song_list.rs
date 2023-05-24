use super::AppComponent;
use crate::app::App;
use eframe::egui;

pub struct SongList;

impl AppComponent for SongList {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        if let Some(library) = &mut ctx.library {
            egui::Grid::new("Songs")
                .striped(true)
                .min_col_width(100.)
                .show(ui, |ui| {
                    ui.label("Playing");
                    ui.label("#");
                    ui.label("Artist");
                    ui.label("Album");
                    ui.label("Title");
                    ui.label("Genre");
                    ui.end_row();

                    for song in ctx.library.as_ref().unwrap().items().iter() {
                        if let Some(selected_song) = &ctx.player.as_ref().unwrap().selected_track {
                            if selected_song == song {
                                ui.label("▶".to_string());
                            } else {
                                ui.label(" ".to_string());
                            }
                        } else {
                            ui.label("");
                        }

                        let artist_label = ui.add(
                            egui::Label::new(&song.artist().unwrap_or("?".to_string()))
                                .sense(egui::Sense::click()),
                        );

                        ui.label(&song.album().unwrap_or("?".to_string()));
                        ui.label(&song.title().unwrap_or("?".to_string()));
                        ui.label(&song.genre().unwrap_or("?".to_string()));

                        ui.end_row();
                    }
                });
        }
    }
}

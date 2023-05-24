use super::AppComponent;
use crate::app::App;
use eframe::egui;
use eframe::egui::{Color32, RichText};

pub struct SongList;

impl AppComponent for SongList {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        if let Some(_) = &mut ctx.library {
            ui.set_width(ui.available_width());
            egui::Grid::new("Songs")
                .spacing(egui::vec2(8.0, 8.0))
                .striped(true)
                .min_col_width(100.)
                .show(ui, |ui| {
                    ui.label("Playing");
                    ui.label("#");
                    ui.label("Artist");
                    ui.label("Album");
                    ui.label("Title");
                    ui.label("Genre");
                    ui.label("");
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

                        ui.label(&song.key().to_string());

                        ui.add(
                            egui::Label::new(&song.artist().unwrap_or("?".to_string()))
                                .sense(egui::Sense::click()),
                        );

                        ui.add(
                            egui::Label::new(&song.album().unwrap_or("?".to_string()))
                                .sense(egui::Sense::click()),
                        );

                        let song_label = ui.add(
                            egui::Button::new(&song.title().unwrap_or("?".to_string()))
                                .sense(egui::Sense::click()),
                        );

                        ui.label(&song.genre().unwrap_or("?".to_string()));

                        ui.label(RichText::new("▶").color(Color32::LIGHT_YELLOW));

                        if song_label.clicked() || song_label.double_clicked() {
                            ctx.player
                                .as_mut()
                                .unwrap()
                                .select_track(Some(song.clone()));
                            ctx.player.as_mut().unwrap().play();
                        }

                        ui.end_row();
                    }
                });
        }
    }
}

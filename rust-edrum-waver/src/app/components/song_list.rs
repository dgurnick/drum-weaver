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
                    ui.label("Title");
                    ui.label("Album");
                    ui.label("Genre");
                    ui.end_row();

                    for song in ctx.library.as_ref().unwrap().items().iter() {
                        #[warn(unused_assignments)]
                        let mut _is_selected = false;
                        #[warn(unused_assignments)]
                        let mut _is_clicked = false;
                        if let Some(selected_song) = &ctx.player.as_ref().unwrap().selected_track {
                            if selected_song == song {
                                _is_clicked =
                                    ui.button(RichText::new("■").color(Color32::RED)).clicked();
                                _is_selected = true;
                            } else {
                                _is_clicked = ui
                                    .button(RichText::new("▶").color(Color32::LIGHT_GRAY))
                                    .clicked();
                            }
                        } else {
                            _is_clicked = ui
                                .button(RichText::new("▶").color(Color32::LIGHT_GRAY))
                                .clicked();
                        }

                        if _is_selected {
                            ui.label(
                                RichText::new(&song.key().to_string()).color(Color32::LIGHT_GREEN),
                            );
                            ui.label(
                                RichText::new(&song.artist().unwrap()).color(Color32::LIGHT_GREEN),
                            );
                            ui.label(
                                RichText::new(&song.title().unwrap()).color(Color32::LIGHT_GREEN),
                            );
                            ui.label(
                                RichText::new(&song.album().unwrap_or("".to_string()))
                                    .color(Color32::LIGHT_GREEN),
                            );
                            ui.label(
                                RichText::new(&song.genre().unwrap()).color(Color32::LIGHT_GREEN),
                            );
                        } else {
                            ui.label(&song.key().to_string());
                            ui.label(&song.artist().unwrap_or(" ".to_string()));
                            ui.label(&song.title().unwrap_or(" ".to_string()));
                            ui.label(&song.album().unwrap_or("".to_string()));
                            ui.label(&song.genre().unwrap_or("?".to_string()));
                        }

                        if _is_clicked && _is_selected {
                            ctx.player.as_mut().unwrap().stop();
                        } else if _is_clicked {
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

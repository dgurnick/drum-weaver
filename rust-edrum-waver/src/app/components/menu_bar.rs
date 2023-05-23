use super::AppComponent;
use crate::app::{App, Playlist};

pub struct MenuBar;

impl AppComponent for MenuBar {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        eframe::egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let _open_btn = ui.button("Open");

                ui.separator();

                let _add_files_btn = ui.button("Add Files");
                let _add_folders_btn = ui.button("Add Folders");

                ui.separator();

                ui.separator();

                let _pref_btn = ui.button("Preferences");

                ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.quit();
                }
            });

            ui.menu_button("Edit", |ui| {
                let _remove_dup_btn = ui.button("Remove duplicates");
            });

            ui.menu_button("Playback", |ui| {
                let play_btn = ui.button("Play");
                let stop_btn = ui.button("Stop");
                let pause_btn = ui.button("Pause");
                let next_btn = ui.button("Next");
                let prev_btn = ui.button("Previous");

                if let Some(_selected_track) = &ctx.player.as_mut().unwrap().selected_track {
                    if play_btn.clicked() {
                        ctx.player.as_mut().unwrap().play();
                    }

                    if stop_btn.clicked() {
                        ctx.player.as_mut().unwrap().stop();
                    }

                    if pause_btn.clicked() {
                        ctx.player.as_mut().unwrap().pause();
                    }

                    if next_btn.clicked() {
                        ctx.player
                            .as_mut()
                            .unwrap()
                            .next(&ctx.playlists[(ctx.current_playlist).unwrap()])
                    }

                    if prev_btn.clicked() {
                        ctx.player
                            .as_mut()
                            .unwrap()
                            .previous(&ctx.playlists[(ctx.current_playlist).unwrap()])
                    }
                }
            });

            ui.menu_button("Library", |ui| {
                let _cgf_btn = ui.button("Configure");
            });

            ui.menu_button("Help", |ui| {
                let _about_btn = ui.button("About");
            });
        });
    }
}

use super::AppComponent;
use crate::app::{library::LibraryItem, App, Library};
use eframe::egui::Button;
use std::fs;

pub struct MenuBar;

impl AppComponent for MenuBar {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        eframe::egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Set music location").clicked() {
                    if let Some(music_folder) = rfd::FileDialog::new().pick_folder() {
                        tracing::info!("Setting music location to : {}", music_folder.display());
                        let mut new_library = Library::new(music_folder);

                        ctx.library = Some(new_library.clone());
                    }
                }

                let enable_menu = match ctx.library {
                    Some(_) => true,
                    _ => false,
                };

                if ui
                    .add_enabled(enable_menu, Button::new("Open file list"))
                    .clicked()
                {
                    if let Some(csv_file) = rfd::FileDialog::new()
                        .add_filter("csv", &["csv"])
                        .pick_file()
                    {
                        tracing::info!("Processing file: {}", csv_file.display());

                        let mut new_library =
                            Library::new(ctx.library.as_ref().unwrap().music_folder());
                        let tx = ctx.library_sender.as_ref().unwrap().clone();
                        std::thread::spawn(move || {
                            let content =
                                fs::read_to_string(csv_file).expect("Unable to read provided file");
                            let mut reader = csv::Reader::from_reader(content.as_bytes());
                            let mut counter = 1;
                            for record in reader.deserialize() {
                                let mut song: LibraryItem = record.unwrap();
                                song.set_key(counter);
                                new_library.add(song);
                                counter += 1;
                            }

                            tx.send(new_library)
                                .expect("Failed to send library to UI thread");
                        });
                    };
                }

                ui.separator();

                //let _add_files_btn = ui.button("Add Files");
                //let _add_folders_btn = ui.button("Add Folders");

                //ui.separator();

                //ui.separator();

                //let _pref_btn = ui.button("Preferences");

                //ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.quit();
                }
            });

            //ui.menu_button("Edit", |ui| {
            //    let _remove_dup_btn = ui.button("Remove duplicates");
            //});

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

            //ui.menu_button("Library", |ui| {
            //    let _cgf_btn = ui.button("Configure");
            //});

            //ui.menu_button("Help", |ui| {
            //    let _about_btn = ui.button("About");
            //});
        });
    }
}

pub mod footer;
//pub mod library_component;
pub mod menu_bar;
pub mod player_component;
pub mod song_list;
//pub mod playlist_tabs;

pub trait AppComponent {
    type Context;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui);
}

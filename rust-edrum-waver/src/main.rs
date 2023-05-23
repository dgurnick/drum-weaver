pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use eframe::egui;
use std::sync::mpsc::channel;
use std::thread;

mod app;

pub enum playerstate {
    unstarted,
    stopped,
    playing,
    paused,
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("starting app ...");

    //    let (audio_rx, audio_tx) = channel();
    //    let (rx, tx) = channel();

    let mut app = App::load().unwrap_or_default();

    let _audio_thread = thread::spawn(move || {});

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(
        "Drum Karaoke Player",
        window_options,
        Box::new(|_| Box::new(app)),
    )
    .expect("eframe failed: I should change main to return a result and use anyhow");
}

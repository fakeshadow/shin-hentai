#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod error;
mod file;
mod icon;
mod image;

fn main() {
    let options = eframe::NativeOptions {
        icon_data: Some(icon::icon()),
        ..Default::default()
    };

    eframe::run_native(
        "maji hentai",
        options,
        Box::new(|_| Box::new(app::MyApp::default())),
    );
}

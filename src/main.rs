#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

fn main() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "maji hentai",
        options,
        Box::new(|_| {
            Box::new(app::MyApp::default())
        }),
    );
}

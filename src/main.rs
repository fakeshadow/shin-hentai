#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod file;
mod image;
mod ui;

// generated with build.rs
mod const_image {
    include!(concat!(env!("OUT_DIR"), "/const_image.rs"));
}

fn main() {
    let options = eframe::NativeOptions {
        icon_data: Some(image::icon()),
        ..Default::default()
    };

    eframe::run_native(
        "maji hentai",
        options,
        Box::new(|ctx| Box::new(ui::UiObj::new(&ctx.egui_ctx))),
    );
}

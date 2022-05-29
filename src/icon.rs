//! provide icon data to app.

use eframe::IconData;

pub(crate) fn icon() -> IconData {
    let bytes = include_bytes!("../resource/shin-hentai.png");

    let icon = image::load_from_memory(bytes)
        .expect("Failed to open icon path")
        .to_rgba8();

    let (width, height) = icon.dimensions();

    IconData {
        rgba: icon.into_raw(),
        width,
        height,
    }
}

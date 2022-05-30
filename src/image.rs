use eframe::egui::ColorImage;

// generated with build.rs
mod const_image {
    include!(concat!(env!("OUT_DIR"), "/const_image.rs"));
}

#[cold]
#[inline(never)]
fn broken_image() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(const_image::BROKEN_IMAGE_SIZE, const_image::BROKEN_IMAGE)
}

pub(crate) fn render_image(buf: &[u8]) -> ColorImage {
    image::load_from_memory(buf)
        .map(|image| {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
        })
        .unwrap_or_else(|_| broken_image())
}

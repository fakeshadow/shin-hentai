use eframe::{egui::ColorImage, IconData};

use crate::const_image::*;

#[cold]
#[inline(never)]
pub(crate) fn icon() -> IconData {
    let [width, height] = ICON_IMAGE_SIZE;
    IconData {
        rgba: ICON_IMAGE.to_vec(),
        width,
        height,
    }
}

#[cold]
#[inline(never)]
pub(crate) fn drag_drop() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(DRAG_DROP_IMAGE_SIZE, DRAG_DROP_IMAGE)
}

#[cold]
#[inline(never)]
fn broken() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(BROKEN_IMAGE_SIZE, BROKEN_IMAGE)
}

pub(crate) fn render_image(buf: &[u8]) -> ColorImage {
    image::load_from_memory(buf)
        .map(|image| {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
        })
        .unwrap_or_else(|_| broken())
}

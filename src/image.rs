use eframe::egui::ColorImage;
use image::imageops::FilterType;

use crate::const_image::*;

#[cfg(not(target_arch = "wasm32"))]
#[cold]
#[inline(never)]
pub fn icon() -> eframe::egui::IconData {
    let [width, height] = ICON_IMAGE_SIZE;
    eframe::egui::IconData {
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

pub(crate) fn render_image(buf: &[u8], base_res: &[u32; 2]) -> ColorImage {
    image::load_from_memory(buf)
        .map(|mut image| {
            let [base_w, base_h] = *base_res;
            let w = image.width();
            let h = image.height();

            if w > base_w || h > base_h {
                image = image.resize(base_w, base_h, FilterType::Triangle);
            }

            let w = image.width() as _;
            let h = image.height() as _;

            let pixels = image.into_rgba8().into_flat_samples();

            ColorImage::from_rgba_unmultiplied([w, h], pixels.as_slice())
        })
        .unwrap_or_else(|_| broken())
}

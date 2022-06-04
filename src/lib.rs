mod error;
mod file;
pub mod image;
pub mod ui;

// generated with build.rs
mod const_image {
    include!(concat!(env!("OUT_DIR"), "/const_image.rs"));
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let res = [3840, 2160];
    eframe::start_web(
        canvas_id,
        Box::new(move |ctx| Box::new(ui::UiObj::new(&ctx.egui_ctx, res))),
    )
}

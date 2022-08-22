mod error;
mod file;
pub mod image;
pub mod ui;

// generated with build.rs
mod const_image {
    include!(concat!(env!("OUT_DIR"), "/const_image.rs"));
}

#[cfg(target_arch = "wasm32")]
use eframe::{
    wasm_bindgen::{self, prelude::*},
    web::AppRunnerRef,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<WebHandle, wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    let res = [3840, 2160];

    eframe::start_web(
        canvas_id,
        eframe::WebOptions::default(),
        Box::new(move |ctx| Box::new(ui::UiObj::new(&ctx.egui_ctx, res))),
    )
    .map(WebHandle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle(AppRunnerRef);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen]
    #[cfg(target_arch = "wasm32")]
    pub fn stop_web(&self) -> Result<(), wasm_bindgen::JsValue> {
        self.0.lock().destroy()
    }
}

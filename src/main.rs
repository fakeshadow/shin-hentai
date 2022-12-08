#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions {
        icon_data: Some(shin_hentai::image::icon()),
        ..Default::default()
    };

    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];

    eframe::run_native(
        "maji_hentai",
        options,
        Box::new(move |ctx| Box::new(shin_hentai::ui::UiObj::new(&ctx.egui_ctx, res))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];
    let opt = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        eframe::start_web(
            "maji_hentai",
            opt,
            Box::new(move |ctx| Box::new(shin_hentai::ui::UiObj::new(&ctx.egui_ctx, res))),
        )
        .await
        .expect("failed to start shin-hentai");
    });
}

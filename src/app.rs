#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let options = eframe::NativeOptions {
        icon_data: Some(shin_hentai::image::icon()),
        ..Default::default()
    };

    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];

    eframe::run_native(
        "maji hentai",
        options,
        Box::new(move |ctx| Box::new(shin_hentai::ui::UiObj::new(&ctx.egui_ctx, res))),
    );
}

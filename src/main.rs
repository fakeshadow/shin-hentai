#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::CreationContext;
use shin_hentai::ui::UiObj;

const EXPECT_MSG: &str = "failed to start shin-hentai";

fn main() {
    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];

    let name = "maji_hentai";

    let creator =
        Box::new(move |ctx: &CreationContext| Box::new(UiObj::new(&ctx.egui_ctx, res)) as _);

    #[cfg(not(target_arch = "wasm32"))]
    {
        eframe::run_native(
            name,
            eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default()
                    .with_app_id("shin_hentai_bin")
                    .with_icon(shin_hentai::image::icon()),
                ..Default::default()
            },
            creator,
        )
        .expect(EXPECT_MSG);
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Make sure panics are logged using `console.error`.
        console_error_panic_hook::set_once();

        wasm_bindgen_futures::spawn_local(async move {
            eframe::WebRunner::new()
                .start(name, eframe::WebOptions::default(), creator)
                .await
                .expect(EXPECT_MSG);
        });
    }
}

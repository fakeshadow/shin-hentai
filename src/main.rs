#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::CreationContext;
use shin_hentai::ui::UiObj;

fn main() {
    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];

    let creator =
        Box::new(move |ctx: &CreationContext| Ok(Box::new(UiObj::new(&ctx.egui_ctx, res)) as _));

    #[cfg(not(target_arch = "wasm32"))]
    {
        eframe::run_native(
            "maji_hentai",
            eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default()
                    .with_app_id("shin_hentai_bin")
                    .with_icon(shin_hentai::image::icon()),
                ..Default::default()
            },
            creator,
        )
        .expect("failed to start shin-hentai");
    }

    #[cfg(target_arch = "wasm32")]
    {
        use eframe::wasm_bindgen::JsCast as _;

        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("maji_hentai")
                .expect("Failed to find maji_hentai")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("maji_hentai was not a HtmlCanvasElement");

            let start_result = eframe::WebRunner::new()
                .start(canvas, web_options, creator)
                .await;

            // Remove the loading text and spinner:
            if let Some(loading_text) = document.get_element_by_id("loading_text") {
                match start_result {
                    Ok(_) => {
                        loading_text.remove();
                    }
                    Err(e) => {
                        loading_text.set_inner_html(
                            "<p> The app has crashed. See the developer console for details. </p>",
                        );
                        panic!("Failed to start eframe: {e:?}");
                    }
                }
            }
        });
    }
}

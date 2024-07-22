#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::CreationContext;
use shin_hentai::ui::UiObj;

fn main() {
    // TODO: get monitor resolution somehow.
    let res = [1920, 1080];

    let name = "maji_hentai";

    let creator =
        Box::new(move |ctx: &CreationContext| Ok(Box::new(UiObj::new(&ctx.egui_ctx, res)) as _));

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
        .expect("failed to start shin-hentai");
    }

    #[cfg(target_arch = "wasm32")]
    {
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        wasm_bindgen_futures::spawn_local(async {
            let start_result = eframe::WebRunner::new()
                .start(name, eframe::WebOptions::default(), creator)
                .await;

            // Remove the loading text and spinner:
            let loading_text = eframe::web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("loading_text"));
            if let Some(loading_text) = loading_text {
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

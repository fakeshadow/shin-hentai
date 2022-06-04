#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            icon_data: Some(shin_hentai::image::icon()),
            ..Default::default()
        };

        let size = winit::event_loop::EventLoop::new()
            .primary_monitor()
            .unwrap()
            .size();

        let res = [size.width, size.height];

        eframe::run_native(
            "maji hentai",
            options,
            Box::new(move |ctx| Box::new(shin_hentai::ui::UiObj::new(&ctx.egui_ctx, res))),
        );
    }
}

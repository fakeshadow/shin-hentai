use eframe::{
    egui::{
        Align, Align2, CentralPanel, ColorImage, Context, Key, Layout, Spinner, TextureHandle,
        TextureOptions, TopBottomPanel, Ui, Widget, Window,
    },
    App, Frame,
};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use crate::{error::Error, file::FileObj};

pub struct UiObj {
    file: FileObj,
    current: TextureHandle,
    error: Option<Error>,
    is_loading: bool,
    #[cfg(target_arch = "wasm32")]
    async_value: Rc<RefCell<Option<Vec<u8>>>>,
}

impl UiObj {
    pub fn new(ctx: &Context, res: [u32; 2]) -> Self {
        Self {
            file: FileObj::new(res),
            current: ctx.load_texture(
                "current-image",
                crate::image::drag_drop(),
                TextureOptions::LINEAR,
            ),
            error: None,
            is_loading: false,
            #[cfg(target_arch = "wasm32")]
            async_value: Rc::new(RefCell::new(None)),
        }
    }

    fn set_error(&mut self, error: Error) {
        self.error = Some(error);
        self.is_loading = false;
    }

    fn set_image(&mut self, image: ColorImage, ctx: &Context) {
        self.current = ctx.load_texture("current-image", image, TextureOptions::LINEAR);
        self.is_loading = false;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn try_open(&mut self, path: PathBuf, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_first(path)? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn try_open(&mut self, buf: impl AsRef<[u8]> + 'static, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_first(buf)? {
            self.set_image(image, ctx);
        }

        Ok(())
    }

    fn try_next(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_next()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_previous(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_previous()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_listen_input(&mut self, ctx: &Context) -> Result<(), Error> {
        let scroll = ctx.input().scroll_delta;
        let arrow_up = ctx.input().key_pressed(Key::W);
        let arrow_down = ctx.input().key_pressed(Key::S);

        if scroll.y < -10.0 || arrow_down {
            self.try_next(ctx)?;
        } else if scroll.y > 10.0 || arrow_up {
            self.try_previous(ctx)?;
        }

        Ok(())
    }

    fn try_listen_drop(&mut self, ctx: &Context) -> Result<(), Error> {
        let file = ctx.input_mut().raw.dropped_files.pop();

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = file.and_then(|file| file.path) {
                self.is_loading = true;
                self.try_open(path, ctx)?;
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(bytes) = file.and_then(|file| file.bytes) {
                self.is_loading = true;
                self.try_open(bytes, ctx)?;
            }
        }

        Ok(())
    }

    // on web operations are mostly happen in async and async_value
    // is used to pass output of async calls to Ui.
    #[cfg(target_arch = "wasm32")]
    fn try_listen_async(&mut self, ctx: &Context) -> Result<(), Error> {
        let opt = self.async_value.borrow_mut().take();
        if let Some(file) = opt {
            let res = self.try_open(file, ctx);
            res?;
        }

        Ok(())
    }

    fn try_update(&mut self, ctx: &Context, _frame: &mut Frame) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        self.try_listen_async(ctx)?;

        self.try_listen_drop(ctx)?;
        self.try_listen_input(ctx)?;

        self.render_top_bar(ctx);

        CentralPanel::default().show(ctx, |ui| {
            self.render_error(ui);

            if self.is_loading {
                self.render_loading(ui);
            } else {
                self.render_img(ui);
            }
        });

        Ok(())
    }

    fn render_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("control-bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                ui.menu_button("💻 Menu", |ui| {
                    ui.set_style(ui.ctx().style());
                    if ui.button("📂 Open").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Err(e) = self.try_open(path, ui.ctx()) {
                                    self.set_error(e);
                                }
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            // See Ui::try_listen_async for explain.
                            let fut = rfd::AsyncFileDialog::new().pick_file();
                            let ctx = ui.ctx().clone();
                            let value = self.async_value.clone();
                            self.is_loading = true;
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Some(file) = fut.await {
                                    let buf = file.read().await;
                                    *value.borrow_mut() = Some(buf);
                                    ctx.request_repaint();
                                }
                            })
                        }

                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn render_img(&mut self, ui: &mut Ui) {
        let window_size = ui.available_size();
        let org_size = self.current.size_vec2();

        let x_ratio = org_size.x / window_size.x;
        let y_ratio = org_size.y / window_size.y;

        let size = if x_ratio > 1.0 || y_ratio > 1.0 {
            if x_ratio > y_ratio {
                [window_size.x, org_size.y / x_ratio]
            } else {
                [org_size.x / y_ratio, window_size.y]
            }
        } else {
            [org_size.x, org_size.y]
        };

        ui.centered_and_justified(|ui| ui.image(&self.current, size));
    }

    fn render_loading(&mut self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| Spinner::default().ui(ui));
    }

    fn render_error(&mut self, ui: &mut Ui) {
        if self.error.is_some() {
            Window::new("Error occurred")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.heading(format!("{}", self.error.as_ref().unwrap()));
                        if ui.button("Confirm").clicked() {
                            self.error = None;
                            ui.ctx().request_repaint();
                        }
                    });
                });
        }
    }
}

impl App for UiObj {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Err(e) = self.try_update(ctx, frame) {
            self.set_error(e);
        }
    }
}

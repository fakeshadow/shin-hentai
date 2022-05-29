use std::path::PathBuf;

use eframe::{
    egui::{CentralPanel, ColorImage, Context, Key, TextureHandle, TopBottomPanel, Ui, Window},
    App, Frame,
};

use crate::{error::Error, file::File};

#[derive(Default)]
pub struct MyApp {
    file: File,
    current: Option<TextureHandle>,
    error: Option<Error>,
}

impl MyApp {
    fn set_error(&mut self, error: Error) {
        self.error = Some(error);
    }

    fn set_image(&mut self, image: ColorImage, ctx: &Context) {
        self.current = Some(ctx.load_texture("current-image", image));
    }

    fn try_open(&mut self, path: &PathBuf, ctx: &Context) -> Result<(), Error> {
        self.file.try_open(path)?;
        let image = self.file.try_image()?;
        self.set_image(image, ctx);
        Ok(())
    }

    fn render_img(&mut self, ui: &mut Ui) {
        if let Some(texture) = self.current.as_ref() {
            let window_size = ui.available_size();
            let org_size = texture.size_vec2();

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

            ui.centered_and_justified(|ui| ui.image(texture, size));
        }
    }

    fn try_next(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_next_image()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_previous(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_previous_image()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_listen_input(&mut self, ctx: &Context) -> Result<(), Error> {
        if self.file.is_some() {
            let scroll = ctx.input().scroll_delta;
            let arrow_up = ctx.input().key_released(Key::W);
            let arrow_down = ctx.input().key_released(Key::S);

            if scroll.y < -10.0 || arrow_down {
                self.try_next(ctx)?;
            } else if scroll.y > 10.0 || arrow_up {
                self.try_previous(ctx)?;
            }
        }

        Ok(())
    }

    fn try_listen_drop(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(path) = ctx
            .input()
            .raw
            .dropped_files
            .get(0)
            .and_then(|file| file.path.as_ref())
        {
            self.try_open(path, ctx)?;
        }

        Ok(())
    }

    fn try_update(&mut self, ctx: &Context, _frame: &mut Frame) -> Result<(), Error> {
        self.try_listen_drop(ctx)?;
        self.try_listen_input(ctx)?;

        self.render_top_bar(ctx);

        CentralPanel::default().show(ctx, |ui| {
            self.render_error(ui);
            self.render_img(ui);
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
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Err(e) = self.try_open(&path, ui.ctx()) {
                                self.set_error(e);
                            }
                        }

                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn render_error(&mut self, ui: &mut Ui) {
        if self.error.is_some() {
            Window::new("Error occurred")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.heading(format!("{}", self.error.as_ref().unwrap()));
                    ui.horizontal(|ui| {
                        if ui.button("Confirm").clicked() {
                            self.error = None;
                            ui.ctx().request_repaint();
                        }
                    });
                });
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Err(e) = self.try_update(ctx, frame) {
            self.set_error(e);
        }
    }
}

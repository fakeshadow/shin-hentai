use std::{fs::File, io::Read, path::PathBuf};

use eframe::{
    egui::{CentralPanel, ColorImage, Context, TextureHandle, TopBottomPanel, Ui, Key},
    App, Frame,
};
use image::imageops::FilterType;
use zip::ZipArchive;

pub struct MyApp {
    resolution: [u32; 2],
    next: usize,
    images: Box<[ColorImage]>,
    current: Option<TextureHandle>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            resolution: [1920, 1080],
            next: 0,
            images: Box::default(),
            current: None
        }
    }
}

impl MyApp {
    fn open(&mut self, path: &PathBuf, ctx: &Context) {
        let file = File::open(path).unwrap();

        let mut file = ZipArchive::new(file).unwrap();

        let images = {
            (0..file.len())
                .map(|i| {
                    let mut f = file.by_index(i).unwrap();
                    let mut buf = Vec::with_capacity(f.size() as usize);

                    f.read_to_end(&mut buf).unwrap();

                    let r = self.resolution;
                    std::thread::spawn(move || load_image_from_memory(&r, &buf).unwrap())
                })
                .collect::<Vec<_>>()
        };

        let images = images
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        self.images = images;
        self.next = 0;
        self.set_current(self.next, ctx);
    }

    fn render_img(&mut self, ui: &mut Ui) {
        if let Some(texture) = self.current.as_ref() {
            let window_size = ui.available_size();

            let org_size = texture.size_vec2();

            ui.centered_and_justified(|ui| {
                ui.image(
                    texture,
                    [org_size.x * window_size.y / org_size.y, window_size.y],
                )
            });
        }
    }

    fn next(&mut self, ctx: &Context) {
        if self.next < self.images.len() - 1 {
            self.next += 1;
            self.set_current(self.next, ctx);
        }
    }

    fn set_current(&mut self, idx: usize, ctx: &Context) {
        self.current = Some(ctx.load_texture("current-image", self.images[idx].clone()));
    }

    fn previous(&mut self, ctx: &Context) {
        if self.next > 0 {
            self.next -= 1;
            self.set_current(self.next, ctx);
        }
    }

    fn listen_input(&mut self, ctx: &Context) {
        if self.images.len() > 0 {
            let scroll = ctx.input().scroll_delta;
            let arrow_up = ctx.input().key_released(Key::ArrowUp);
            let arrow_down = ctx.input().key_released(Key::ArrowDown);

            if scroll.y < -10.0 || arrow_down {
                self.next(ctx);
            } else if scroll.y > 10.0 || arrow_up {
                self.previous(ctx);
            }
        }
    }

    fn listen_drop(&mut self, ctx: &Context) {
        if let Some(path) = ctx.input().raw.dropped_files.get(0).and_then(|file| file.path.as_ref()) {
            self.open(path, ctx);
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {

        self.listen_drop(ctx);

        self.listen_input(ctx);

        TopBottomPanel::top("control-bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                ui.menu_button("💻 Menu", |ui| {
                    ui.set_style(ui.ctx().style());
                    if ui.button("💻 Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.open(&path, ui.ctx());
                        }

                        ui.close_menu();
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            self.render_img(ui);
        });
    }
}

fn load_image_from_memory(resolution: &[u32], image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let (w, h) = (resolution[0], resolution[1]);
    let image = image::load_from_memory(image_data)?.resize(w, h, FilterType::Triangle);

    let size = [image.width() as _, image.height() as _];

    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

use eframe::{
    egui::{
        Align, Align2, CentralPanel, ColorImage, Context, Key, KeyboardShortcut, Layout, Modifiers,
        Spinner, TextureHandle, TextureOptions, TopBottomPanel, Ui, Widget, Window,
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
    show_navi: bool,
    #[cfg(not(target_arch = "wasm32"))]
    state: State,
    #[cfg(target_arch = "wasm32")]
    state: StateWasm,
}

enum State {
    Loading,
    #[cfg(target_arch = "wasm32")]
    Buf(Vec<u8>),
    Show(TextureHandle),
    ShowError(Error),
}

impl State {
    fn set(&mut self, other: Self) {
        *self = other;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_mut(&mut self) -> &mut Self {
        self
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct StateWasm(Rc<RefCell<State>>);

#[cfg(target_arch = "wasm32")]
impl StateWasm {
    fn new(state: State) -> Self {
        Self(Rc::new(RefCell::new(state)))
    }

    fn set(&mut self, other: State) {
        self.0.borrow_mut().set(other);
    }

    fn get_mut(&self) -> std::cell::RefMut<'_, State> {
        self.0.borrow_mut()
    }
}

#[cold]
#[inline(never)]
fn default_image_texture(ctx: &Context) -> TextureHandle {
    ctx.load_texture(
        "current-image",
        crate::image::drag_drop(),
        TextureOptions::LINEAR,
    )
}

impl UiObj {
    #[cold]
    #[inline(never)]
    pub fn new(ctx: &Context, res: [u32; 2]) -> Self {
        let state = State::Show(default_image_texture(ctx));
        Self {
            file: FileObj::new(res),
            show_navi: false,
            #[cfg(not(target_arch = "wasm32"))]
            state,
            #[cfg(target_arch = "wasm32")]
            state: StateWasm::new(state),
        }
    }

    #[cold]
    #[inline(never)]
    fn set_error(&mut self, error: Error) {
        self.state.set(State::ShowError(error));
    }

    fn set_image(&mut self, image: ColorImage, ctx: &Context) {
        self.state.set(State::Show(ctx.load_texture(
            "current-image",
            image,
            TextureOptions::LINEAR,
        )));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn try_open(&mut self, path: PathBuf, ctx: &Context) -> Result<(), Error> {
        self.state.set(State::Loading);
        if let Some(image) = self.file.try_first(path)? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn try_open(&mut self, buf: impl AsRef<[u8]> + 'static, ctx: &Context) -> Result<(), Error> {
        self.state.set(State::Loading);
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

    fn try_rewind(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_rewind()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_skip(&mut self, ctx: &Context) -> Result<(), Error> {
        if let Some(image) = self.file.try_skip()? {
            self.set_image(image, ctx);
        }
        Ok(())
    }

    fn try_listen_input(&mut self, ctx: &Context) -> Result<(), Error> {
        const CTRL_W: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::W);
        const CTRL_S: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);

        let rewind = ctx.input_mut().consume_shortcut(&CTRL_W);
        let skip = ctx.input_mut().consume_shortcut(&CTRL_S);

        if rewind {
            self.try_rewind(ctx)
        } else if skip {
            self.try_skip(ctx)
        } else {
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
    }

    fn try_listen_drop(&mut self, ctx: &Context) -> Result<(), Error> {
        let file = ctx.input_mut().raw.dropped_files.pop();

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = file.and_then(|file| file.path) {
                self.try_open(path, ctx)?;
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(bytes) = file.and_then(|file| file.bytes) {
                self.try_open(bytes, ctx)?;
            }
        }

        Ok(())
    }

    fn try_update(&mut self, ctx: &Context, _frame: &mut Frame) -> Result<(), Error> {
        self.try_listen_drop(ctx)?;
        self.try_listen_input(ctx)?;

        self.render_top_bar(ctx);

        #[allow(clippy::drop_ref)]
        CentralPanel::default()
            .show(ctx, |ui| {
                self.render_navi(ui);
                #[allow(unused_mut)]
                let mut state = self.state.get_mut();
                match *state {
                    State::ShowError(ref e) => {
                        let string = format!("{e}");
                        drop(state);
                        self.render_error(string, ui)
                    }
                    #[cfg(target_arch = "wasm32")]
                    State::Buf(_) => match std::mem::replace(&mut *state, State::Loading) {
                        State::Buf(buf) => {
                            drop(state);
                            self.try_open(buf, ctx)?
                        }
                        _ => unreachable!(),
                    },
                    State::Loading => {
                        drop(state);
                        self.render_loading(ui)
                    }
                    State::Show(ref handle) => Self::render_img(handle, ui),
                }
                Ok(())
            })
            .inner
    }

    fn render_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("control-bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
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
                        let mut state = self.state.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            if let Some(file) = fut.await {
                                state.set(State::Loading);
                                ctx.request_repaint();
                                let buf = file.read().await;
                                state.set(State::Buf(buf));
                                ctx.request_repaint();
                            }
                        })
                    }
                }
                if ui.button("⏩ Navi").clicked() {
                    self.show_navi = !self.show_navi;
                };
            });
        });
    }

    fn render_img(handle: &TextureHandle, ui: &mut Ui) {
        let window_size = ui.available_size();
        let org_size = handle.size_vec2();

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

        ui.centered_and_justified(|ui| ui.image(handle, size));
    }

    fn render_loading(&mut self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| Spinner::default().ui(ui));
    }

    fn render_navi(&mut self, ui: &mut Ui) {
        if self.show_navi {
            Window::new("navigator")
                .collapsible(true)
                .title_bar(false)
                .frame(eframe::egui::Frame::popup(ui.style()).multiply_with_opacity(0.3))
                .anchor(Align2::CENTER_TOP, [0.0, 3.0])
                .show(ui.ctx(), |ui| {
                    ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                        if ui.button("⏮").clicked() {
                            if let Err(e) = self.try_rewind(ui.ctx()) {
                                self.set_error(e);
                                ui.ctx().request_repaint();
                            }
                        }
                        if ui.button("◀").clicked() {
                            if let Err(e) = self.try_previous(ui.ctx()) {
                                self.set_error(e);
                                ui.ctx().request_repaint();
                            }
                        }
                        if ui.button("▶").clicked() {
                            if let Err(e) = self.try_next(ui.ctx()) {
                                self.set_error(e);
                                ui.ctx().request_repaint();
                            }
                        }
                        if ui.button("⏭").clicked() {
                            if let Err(e) = self.try_skip(ui.ctx()) {
                                self.set_error(e);
                                ui.ctx().request_repaint();
                            }
                        }
                    })
                });
        }
    }

    #[cold]
    #[inline(never)]
    fn render_error(&mut self, e: String, ui: &mut Ui) {
        Window::new("Error occurred")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.heading(e);
                    if ui.button("Confirm").clicked() {
                        self.state.set(State::Show(default_image_texture(ui.ctx())));
                        ui.ctx().request_repaint();
                    }
                });
            });
    }
}

impl App for UiObj {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Err(e) = self.try_update(ctx, frame) {
            self.set_error(e);
        }
    }
}

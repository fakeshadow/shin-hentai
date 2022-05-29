use std::{fs, io::Read, path::PathBuf};

use eframe::egui::ColorImage;
use zip::ZipArchive;

use crate::error::Error;

#[derive(Default)]
pub(crate) struct File {
    idx: usize,
    file: Option<ZipArchive<fs::File>>,
    buf: Vec<u8>,
}

impl File {
    pub(crate) fn is_some(&self) -> bool {
        self.file.is_some()
    }

    pub(crate) fn try_open(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = fs::File::open(path)?;

        let file = ZipArchive::new(file)?;

        self.file = Some(file);
        self.idx = 0;
        self.buf.clear();

        Ok(())
    }

    pub(crate) fn try_next_image(&mut self) -> Result<Option<ColorImage>, Error> {
        if self.idx < self.file.as_ref().unwrap().len() - 1 {
            self.idx += 1;
            let image = self.try_image()?;
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn try_previous_image(&mut self) -> Result<Option<ColorImage>, Error> {
        if self.idx > 0 {
            self.idx -= 1;
            let image = self.try_image()?;
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn try_image(&mut self) -> Result<ColorImage, Error> {
        let res = self._try_image();
        self.buf.clear();
        res
    }

    fn _try_image(&mut self) -> Result<ColorImage, Error> {
        let mut file = self.file.as_mut().unwrap().by_index(self.idx)?;

        self.buf.reserve(file.size() as usize);

        file.read_to_end(&mut self.buf)?;

        _try_image(&self.buf)
    }
}

fn _try_image(image_data: &[u8]) -> Result<ColorImage, Error> {
    let image = image::load_from_memory(image_data)?;

    let size = [image.width() as _, image.height() as _];

    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

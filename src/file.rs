use std::{fs, io::Read, path::PathBuf};

use eframe::egui::ColorImage;
use zip::ZipArchive;

use crate::error::Error;

pub(crate) struct File {
    idx: usize,
    file: _File,
    buf: Vec<u8>,
}

impl Default for File {
    fn default() -> Self {
        Self {
            idx: 0,
            file: _File::None,
            buf: Vec::new(),
        }
    }
}

pub(crate) enum _File {
    File(ZipArchive<fs::File>),
    Path(Box<[PathBuf]>),
    None,
}

impl _File {
    fn len(&self) -> usize {
        match *self {
            Self::File(ref file) => file.len(),
            Self::Path(ref path) => path.len(),
            Self::None => unreachable!("No _File found"),
        }
    }

    fn is_some(&self) -> bool {
        !matches!(*self, Self::None)
    }

    fn read_file(
        &mut self,
        idx: &mut usize,
        buf: &mut Vec<u8>,
        direction: Direction,
    ) -> Result<(), Error> {
        match *self {
            Self::File(ref mut file) => {
                match direction {
                    Direction::Next => {
                        while *idx < file.len() {
                            if file.by_index(*idx)?.is_file() {
                                break;
                            }
                            *idx += 1;
                        }
                    }
                    Direction::Prev => {
                        while *idx > 0 {
                            if file.by_index(*idx)?.is_file() {
                                break;
                            }
                            *idx -= 1;
                        }
                    }
                }

                let mut file = file.by_index(*idx)?;
                buf.reserve(file.size() as usize);
                file.read_to_end(buf)?;
            }
            Self::Path(ref path) => {
                let mut file = fs::File::open(&path[*idx])?;

                if let Ok(meta) = file.metadata() {
                    buf.reserve(meta.len() as usize);
                }

                file.read_to_end(buf)?;
            }
            Self::None => unreachable!("No _File found"),
        };

        Ok(())
    }
}

impl File {
    pub(crate) fn is_some(&self) -> bool {
        self.file.is_some()
    }

    pub(crate) fn try_first(&mut self, path: &PathBuf) -> Result<ColorImage, Error> {
        self.try_open(path)?;
        self.try_read(Direction::Next)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<ColorImage>, Error> {
        if self.idx < self.file.len() - 1 {
            self.idx += 1;
            let image = self.try_read(Direction::Next)?;
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn try_previous(&mut self) -> Result<Option<ColorImage>, Error> {
        if self.idx > 0 {
            self.idx -= 1;
            let image = self.try_read(Direction::Prev)?;
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    fn try_open(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = if path.is_dir() {
            let mut files = Vec::new();
            visit_dirs(path, &mut |p| files.push(p))?;
            _File::Path(files.into_boxed_slice())
        } else {
            let file = fs::File::open(path)?;
            let file = ZipArchive::new(file)?;
            _File::File(file)
        };

        self.file = file;
        self.idx = 0;
        self.buf.clear();

        Ok(())
    }

    fn try_read(&mut self, direction: Direction) -> Result<ColorImage, Error> {
        let res = self._try_read(direction);
        self.buf.clear();
        res
    }

    fn _try_read(&mut self, direction: Direction) -> Result<ColorImage, Error> {
        self.file
            .read_file(&mut self.idx, &mut self.buf, direction)?;

        let image = image::load_from_memory(&self.buf)?;

        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
    }
}

pub(crate) enum Direction {
    Next,
    Prev,
}

fn visit_dirs(dir: &PathBuf, cb: &mut dyn FnMut(PathBuf)) -> Result<(), Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(path);
            }
        }
    }

    Ok(())
}

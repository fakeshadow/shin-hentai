use std::{fs, io::Read, path::PathBuf};

use eframe::egui::ColorImage;
use zip::ZipArchive;

use crate::error::Error;

trait File {
    fn is_eof(&self) -> bool;

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error>;
}

impl File for () {
    fn is_eof(&self) -> bool {
        true
    }

    fn read(&mut self, _: &mut Vec<u8>, _: Direction) -> Result<(), Error> {
        Ok(())
    }
}

struct ZipFile {
    idx: usize,
    file: ZipArchive<fs::File>,
}

impl File for ZipFile {
    fn is_eof(&self) -> bool {
        self.idx + 1 == self.file.len()
    }

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error> {
        match direction {
            Direction::Next | Direction::Current => {
                while !self.is_eof() {
                    if matches!(direction, Direction::Next) {
                        self.idx += 1;
                    }

                    let mut file = self.file.by_index(self.idx)?;

                    if file.is_file() {
                        buf.reserve(file.size() as usize);
                        file.read_to_end(buf)?;
                        return Ok(());
                    }

                    if matches!(direction, Direction::Current) {
                        self.idx += 1;
                    }
                }
            }
            Direction::Prev => {
                while self.idx != 0 {
                    self.idx -= 1;

                    let mut file = self.file.by_index(self.idx)?;

                    if file.is_file() {
                        buf.reserve(file.size() as usize);
                        file.read_to_end(buf)?;
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }
}

struct PathFile {
    idx: usize,
    file: Box<[PathBuf]>,
}

impl File for PathFile {
    fn is_eof(&self) -> bool {
        self.idx + 1 == self.file.len()
    }

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error> {
        match direction {
            Direction::Next if self.is_eof() => return Ok(()),
            Direction::Prev if self.idx == 0 => return Ok(()),
            Direction::Next => self.idx += 1,
            Direction::Prev => self.idx -= 1,
            Direction::Current => {
                if self.is_eof() {
                    return Ok(());
                }
            }
        }

        let mut file = fs::File::open(&self.file[self.idx])?;

        if let Ok(meta) = file.metadata() {
            buf.reserve(meta.len() as usize);
        }

        file.read_to_end(buf)?;

        Ok(())
    }
}

pub(crate) struct FileObj {
    file: Box<dyn File>,
    buf: Vec<u8>,
}

impl Default for FileObj {
    fn default() -> Self {
        Self {
            file: Box::new(()),
            buf: Vec::new(),
        }
    }
}

impl FileObj {
    pub(crate) fn try_first(&mut self, path: &PathBuf) -> Result<Option<ColorImage>, Error> {
        self.try_open(path)?;
        self.try_read(Direction::Current)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<ColorImage>, Error> {
        self.try_read(Direction::Next)
    }

    pub(crate) fn try_previous(&mut self) -> Result<Option<ColorImage>, Error> {
        self.try_read(Direction::Prev)
    }

    fn try_open(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = if path.is_dir() {
            let mut files = Vec::new();
            visit_dirs(path, &mut |p| files.push(p))?;
            Box::new(PathFile {
                idx: 0,
                file: files.into_boxed_slice(),
            }) as _
        } else {
            let file = fs::File::open(path)?;
            let file = ZipArchive::new(file)?;
            Box::new(ZipFile { idx: 0, file }) as _
        };

        self.file = file;
        self.buf.clear();

        Ok(())
    }

    fn try_read(&mut self, direction: Direction) -> Result<Option<ColorImage>, Error> {
        let res = self._try_read(direction);
        self.buf.clear();
        res
    }

    fn _try_read(&mut self, direction: Direction) -> Result<Option<ColorImage>, Error> {
        self.file.read(&mut self.buf, direction)?;

        if self.buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(crate::image::render_image(&self.buf)))
        }
    }
}

enum Direction {
    Current,
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

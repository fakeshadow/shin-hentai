use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use eframe::egui::ColorImage;
use zip::ZipArchive;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use crate::error::Error;

trait File {
    fn is_eof(&self) -> bool;

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error>;
}

struct NoFile;

impl File for NoFile {
    fn is_eof(&self) -> bool {
        true
    }

    fn read(&mut self, _: &mut Vec<u8>, _: Direction) -> Result<(), Error> {
        Ok(())
    }
}

struct ZipFile<R> {
    idx: usize,
    file: ZipArchive<R>,
}

impl<R> File for ZipFile<R>
where
    R: Read + Seek,
{
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
            Direction::Offset(idx) => {
                assert!(idx < self.file.len());
                self.idx = idx;
                let mut file = self.file.by_index(self.idx)?;

                if file.is_file() {
                    buf.reserve(file.size() as usize);
                    file.read_to_end(buf)?;
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

    fn read(&mut self, _: &mut Vec<u8>, _: Direction) -> Result<(), Error> {
        unreachable!("PathFile should not be called on File::read")
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NestFile {
    parent: PathFile,
    child: Box<dyn File>,
}

#[cfg(not(target_arch = "wasm32"))]
impl File for NestFile {
    fn is_eof(&self) -> bool {
        self.parent.is_eof() && self.child.is_eof()
    }

    #[inline(never)]
    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error> {
        if self.child.is_eof() {
            let this = &mut self.parent;
            match direction {
                Direction::Next if this.is_eof() => return Ok(()),
                Direction::Prev if this.idx == 0 => return Ok(()),
                Direction::Next => this.idx += 1,
                Direction::Prev => this.idx -= 1,
                Direction::Offset(idx) => {
                    assert!(idx < this.file.len());
                    this.idx = idx;
                }
                Direction::Current => {}
            }

            let path = &this.file[this.idx];

            let is_zip = path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "zip")
                    .unwrap_or(false);

            let mut file = fs::File::open(path)?;

            if is_zip {
                let file = ZipArchive::new(file)?;

                self.child = Box::new(ZipFile { idx: 0, file });

                self.read(buf, direction)
            } else {
                if let Ok(meta) = file.metadata() {
                    buf.reserve(meta.len() as usize);
                }

                file.read_to_end(buf)?;

                Ok(())
            }
        } else {
            self.child.read(buf, direction)
        }
    }
}

pub(crate) struct FileObj {
    res: [u32; 2],
    file: Box<dyn File>,
    buf: Vec<u8>,
    #[allow(dead_code)]
    directory_hint: PathBuf,
}

impl FileObj {
    pub(crate) fn new(res: [u32; 2]) -> Self {
        Self {
            res,
            file: Box::new(NoFile),
            buf: Vec::new(),
            directory_hint: PathBuf::default(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn try_first(&mut self, path: PathBuf) -> Result<Option<ColorImage>, Error> {
        self.try_open(path)?;
        self.try_read(Direction::Current)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn try_first(
        &mut self,
        file: impl AsRef<[u8]> + 'static,
    ) -> Result<Option<ColorImage>, Error> {
        let file = ZipArchive::new(std::io::Cursor::new(file))?;
        let file = Box::new(ZipFile { idx: 0, file }) as _;

        self.file = file;
        self.buf.clear();

        self.try_read(Direction::Current)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<ColorImage>, Error> {
        match self.try_read(Direction::Next)? {
            #[cfg(not(target_arch = "wasm32"))]
            None if self.file.is_eof() && self.directory_hint.exists() => self.try_next_obj(),
            res => Ok(res),
        }
    }

    pub(crate) fn try_previous(&mut self) -> Result<Option<ColorImage>, Error> {
        self.try_read(Direction::Prev)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn try_open(&mut self, path: PathBuf) -> Result<(), Error> {
        self.buf.clear();
        // regardless the outcome advance path to skip bad files.
        self.directory_hint = path;
        let path = &self.directory_hint;

        let file = if path.is_dir() {
            let mut files = Vec::new();
            visit_dirs(path, &mut |p| files.push(p))?;

            Box::new(NestFile {
                parent: PathFile {
                    idx: 0,
                    file: files.into_boxed_slice(),
                },
                child: Box::new(NoFile),
            }) as _
        } else {
            let file = fs::File::open(&path)?;
            let file = ZipArchive::new(file)?;
            Box::new(ZipFile { idx: 0, file }) as _
        };

        self.file = file;

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn try_next_obj(&mut self) -> Result<Option<ColorImage>, Error> {
        match next_file_path(&self.directory_hint) {
            Ok(Some(path)) => self.try_first(path),
            Ok(None) => {
                self.directory_hint = PathBuf::default();
                Ok(None)
            }
            Err(e) => {
                self.directory_hint = PathBuf::default();
                Err(e)
            }
        }
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
            Ok(Some(crate::image::render_image(&self.buf, &self.res)))
        }
    }
}

#[allow(dead_code)]
enum Direction {
    Current,
    Next,
    Prev,
    Offset(usize),
}

#[cfg(not(target_arch = "wasm32"))]
#[inline(never)]
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

#[cfg(not(target_arch = "wasm32"))]
#[inline(never)]
fn next_file_path(path: &PathBuf) -> Result<Option<PathBuf>, Error> {
    match path.parent() {
        Some(p) => {
            assert!(p.is_dir());

            let mut entries = fs::read_dir(p)?;

            for entry in entries.by_ref() {
                let entry = entry?;
                if &entry.path() == path {
                    break;
                }
            }

            match entries.next() {
                Some(entry) => {
                    let entry = entry?;
                    Ok(Some(entry.path()))
                }
                None => Ok(None),
            }
        }
        None => Ok(None),
    }
}

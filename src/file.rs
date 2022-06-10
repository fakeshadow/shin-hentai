use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use eframe::egui::ColorImage;
use zip::ZipArchive;

use crate::error::Error;

#[allow(dead_code)]
enum Direction {
    First,
    Last,
    Next,
    Prev,
    Offset(usize),
}

trait File {
    fn is_head(&self) -> bool;

    fn is_eof(&self) -> bool;

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error>;
}

struct NoFile;

impl File for NoFile {
    fn is_head(&self) -> bool {
        true
    }

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

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<&PathBuf> for ZipFile<std::fs::File> {
    type Error = Error;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = std::fs::File::open(path)?;
        let file = ZipArchive::new(file)?;
        Ok(Self { idx: 0, file })
    }
}

impl<R> ZipFile<R>
where
    R: Read + Seek,
{
    // a loop read auto advance the zip file index to skip nested folders inside zip file.
    // TODO: properly handle nested folders in zip.
    fn _read<F, F1>(
        &mut self,
        buf: &mut Vec<u8>,
        mut condition: F,
        mut on_continue: F1,
    ) -> Result<(), Error>
    where
        F: FnMut(&mut Self) -> bool,
        F1: FnMut(&mut Self),
    {
        while condition(self) {
            let n = self.read_by_index(buf)?;

            if n > 0 {
                return Ok(());
            }

            on_continue(self);
        }

        Ok(())
    }

    // return Ok(n) when current index is a file and filled given buf with n bytes.
    // (include 0 bytes).
    fn read_by_index(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        let mut file = self.file.by_index(self.idx)?;

        let n = if file.is_file() {
            buf.reserve(file.size() as usize);
            file.read_to_end(buf)?
        } else {
            0
        };

        Ok(n)
    }
}

impl<R> File for ZipFile<R>
where
    R: Read + Seek,
{
    fn is_head(&self) -> bool {
        self.idx == 0
    }

    fn is_eof(&self) -> bool {
        self.idx == self.file.len().saturating_sub(1)
    }

    fn read(&mut self, buf: &mut Vec<u8>, direction: Direction) -> Result<(), Error> {
        match direction {
            Direction::Next => self._read(
                buf,
                |this| {
                    let cond = !this.is_eof();
                    if cond {
                        this.idx += 1;
                    }
                    cond
                },
                |_| {},
            ),
            Direction::First => self._read(
                buf,
                |this| !this.is_eof(),
                |this| {
                    this.idx += 1;
                },
            ),
            Direction::Prev => self._read(
                buf,
                |this| {
                    let cond = this.idx != 0;

                    if cond {
                        this.idx -= 1;
                    }

                    cond
                },
                |_| {},
            ),
            Direction::Offset(idx) => {
                assert!(idx < self.file.len());
                self.idx = idx;

                let _ = self.read_by_index(buf)?;

                Ok(())
            }
            Direction::Last => {
                self.idx = self.file.len();
                self.read(buf, Direction::Prev)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
use nest::ListFile;

#[cfg(not(target_arch = "wasm32"))]
mod nest {
    use super::*;

    use std::fs;

    pub(super) struct ListFile {
        idx: usize,
        file: Box<[PathBuf]>,
        child: Box<dyn File>,
    }

    impl TryFrom<&PathBuf> for ListFile {
        type Error = Error;

        fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
            let mut files = Vec::new();
            visit_dirs(path, &mut |p| files.push(p))?;

            Ok(ListFile {
                idx: 0,
                file: files.into_boxed_slice(),
                child: Box::new(NoFile),
            })
        }
    }

    impl ListFile {
        fn _is_eof(&self) -> bool {
            self.idx == self.file.len().saturating_sub(1)
        }

        fn _is_head(&self) -> bool {
            self.idx == 0
        }
    }

    impl File for ListFile {
        fn is_head(&self) -> bool {
            self._is_head() && self.child.is_head()
        }

        fn is_eof(&self) -> bool {
            self._is_eof() && self.child.is_eof()
        }

        fn read(&mut self, buf: &mut Vec<u8>, mut direction: Direction) -> Result<(), Error> {
            match direction {
                Direction::Next if !self.child.is_eof() => return self.child.read(buf, direction),
                Direction::Prev if !self.child.is_head() => return self.child.read(buf, direction),
                Direction::Next if self._is_eof() => return Ok(()),
                Direction::Prev if self._is_head() => return Ok(()),
                Direction::Next => {
                    self.idx += 1;
                    direction = Direction::First;
                }
                Direction::Prev => {
                    self.idx -= 1;
                    direction = Direction::Last;
                }
                Direction::First => {
                    self.idx = 0;
                }
                Direction::Last => {
                    self.idx = self.file.len().saturating_sub(1);
                }
                Direction::Offset(idx) => {
                    assert!(!self._is_eof());
                    self.idx = idx;
                }
            }

            let path = &self.file[self.idx];

            if !path.is_file() {
                assert!(path.is_dir());

                self.child = Box::new(ListFile::try_from(path)?) as _;

                return self.child.read(buf, direction);
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            match ext {
                "jpg" | "jpeg" | "png" => {
                    let mut file = fs::File::open(path)?;

                    if let Ok(meta) = file.metadata() {
                        buf.reserve(meta.len() as usize);
                    }
                    file.read_to_end(buf)?;
                    Ok(())
                }
                // treat all uncertain file extensions as zip file.
                // zip archive would return a format error for all files that are not supported.
                // TODO: add special error handling for determined non zip files.
                _ => {
                    self.child = Box::new(ZipFile::try_from(path)?) as _;
                    self.child.read(buf, direction)
                }
            }
        }
    }

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
        self.try_read(Direction::First)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn try_first(
        &mut self,
        buf: impl AsRef<[u8]> + 'static,
    ) -> Result<Option<ColorImage>, Error> {
        let file = ZipArchive::new(std::io::Cursor::new(buf))?;
        self.file = Box::new(ZipFile { idx: 0, file }) as _;
        self.buf.clear();
        self.try_read(Direction::First)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn try_last(&mut self, path: PathBuf) -> Result<Option<ColorImage>, Error> {
        self.try_open(path)?;
        self.try_read(Direction::Last)
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<ColorImage>, Error> {
        match self.try_read(Direction::Next)? {
            #[cfg(not(target_arch = "wasm32"))]
            None if self.file.is_eof() && self.directory_hint.exists() => self.try_next_obj(),
            res => Ok(res),
        }
    }

    pub(crate) fn try_previous(&mut self) -> Result<Option<ColorImage>, Error> {
        match self.try_read(Direction::Prev)? {
            #[cfg(not(target_arch = "wasm32"))]
            None if self.file.is_head() && self.directory_hint.exists() => self.try_previous_obj(),
            res => Ok(res),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FileObj {
    fn try_open(&mut self, path: PathBuf) -> Result<(), Error> {
        self.buf.clear();
        // regardless the outcome advance path to skip bad files.
        self.directory_hint = path;
        let path = &self.directory_hint;

        self.file = if path.is_dir() {
            Box::new(ListFile::try_from(path)?) as _
        } else {
            Box::new(ZipFile::try_from(path)?) as _
        };

        Ok(())
    }

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

    fn try_previous_obj(&mut self) -> Result<Option<ColorImage>, Error> {
        match previous_file_path(&self.directory_hint) {
            Ok(Some(path)) => self.try_last(path),
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
}

impl FileObj {
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

#[cfg(not(target_arch = "wasm32"))]
#[inline(never)]
fn next_file_path(path: &PathBuf) -> Result<Option<PathBuf>, Error> {
    match path.parent() {
        Some(p) => {
            assert!(p.is_dir());

            let mut entries = std::fs::read_dir(p)?;

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

#[cfg(not(target_arch = "wasm32"))]
#[inline(never)]
fn previous_file_path(path: &PathBuf) -> Result<Option<PathBuf>, Error> {
    match path.parent() {
        Some(p) => {
            assert!(p.is_dir());

            let entries = std::fs::read_dir(p)?;

            let mut res = None;

            for entry in entries {
                let entry = entry?;
                let p = entry.path();
                if &p == path {
                    break;
                }

                res = Some(p);
            }

            Ok(res)
        }
        None => Ok(None),
    }
}

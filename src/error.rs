use std::{error, fmt, io};

use image::ImageError;

pub(crate) enum Error {
    Io(io::Error),
    #[cfg(not(target_arch = "wasm32"))]
    Zip(zip::result::ZipError),
    Image(ImageError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref e) => e.fmt(f),
            Self::Image(ref e) => e.fmt(f),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Zip(ref e) => e.fmt(f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref e) => e.fmt(f),
            Self::Image(ref e) => e.fmt(f),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Zip(ref e) => e.fmt(f),
        }
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Self::Image(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl error::Error for Error {}

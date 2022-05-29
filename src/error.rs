use std::{error, fmt, io};

use image::ImageError;
use zip::result::ZipError;

pub(crate) enum Error {
    Io(io::Error),
    Zip(ZipError),
    Image(ImageError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref e) => e.fmt(f),
            Self::Image(ref e) => e.fmt(f),
            Self::Zip(ref e) => e.fmt(f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref e) => e.fmt(f),
            Self::Image(ref e) => e.fmt(f),
            Self::Zip(ref e) => e.fmt(f),
        }
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Self::Image(e)
    }
}

impl From<ZipError> for Error {
    fn from(e: ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl error::Error for Error {}

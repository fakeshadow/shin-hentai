mod error;
mod file;
pub mod image;
pub mod ui;

// generated with build.rs
mod const_image {
    include!(concat!(env!("OUT_DIR"), "/const_image.rs"));
}

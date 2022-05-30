use std::{env, fs, io::Read, path::Path};

fn main() {
    generate_broken_image();
}

// generate raw bytes of broken-image.svg as const var.
// this would remove the runtime cost of parsing it.
fn generate_broken_image() {
    let mut buf = Vec::new();

    let mut file = fs::File::open("./resource/broken-image.svg").unwrap();

    file.read_to_end(&mut buf).unwrap();

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();

    let rtree = usvg::Tree::from_data(&buf, &opt.to_ref()).unwrap();

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let [w, h] = [pixmap_size.width(), pixmap_size.height()];

    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();

    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("broken_image.rs");

    let s = format!(
        "pub const BROKEN_IMAGE: &[u8] = &{:?};\r\npub const BROKEN_IMAGE_SIZE: [usize; 2] = [{}, {}];",
        pixmap.data(),
        pixmap.width() as usize,
        pixmap.height() as usize
    );

    fs::write(&path, s).unwrap();
}

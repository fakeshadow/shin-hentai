use std::{env, error, fs, io::Read, path::Path};

use tiny_skia::Pixmap;

fn main() {
    generate_broken_image();
}

// generate raw bytes of broken-image.svg as const var.
// this would remove the runtime cost of parsing it.
fn generate_broken_image() {
    generate_svg(
        "./resource/broken-image.svg",
        "const_image.rs",
        |map| {
            format!(
                "pub const BROKEN_IMAGE: &[u8] = &{:?};\r\npub const BROKEN_IMAGE_SIZE: [usize; 2] = [{}, {}];",
                map.data(),
                map.width() as usize,
                map.height() as usize
            )
        }
    ).unwrap();
}

fn generate_svg<O>(
    in_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
    cb: impl FnOnce(Pixmap) -> O,
) -> Result<(), Box<dyn error::Error + Send + Sync>>
where
    O: AsRef<[u8]>,
{
    let mut buf = Vec::new();

    let mut file = fs::File::open(in_path)?;

    file.read_to_end(&mut buf)?;

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();

    let rtree = usvg::Tree::from_data(&buf, &opt.to_ref())?;

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let [w, h] = [pixmap_size.width(), pixmap_size.height()];

    let mut pixmap = Pixmap::new(w, h).unwrap();

    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join(out_path);

    fs::write(&path, cb(pixmap))?;

    Ok(())
}

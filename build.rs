use std::{env, error, fs, io::Read, path::Path};

use tiny_skia::Pixmap;

fn main() {
    generate_image().unwrap();
}

// generate raw bytes of svg as const var.
// this would remove the runtime cost of parsing them.
fn generate_image() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("const_image.rs");

    let mut buf = Vec::new();

    let map = render_svg("./resource/broken-image.svg")?;
    buf.extend_from_slice(
        format!(
        "pub const BROKEN_IMAGE: &[u8] = &{:?};pub const BROKEN_IMAGE_SIZE: [usize; 2] = [{}, {}];",
        map.data(),
        map.width() as usize,
        map.height() as usize
    )
        .as_bytes(),
    );

    let map = render_svg("./resource/shin-hentai.svg")?;
    buf.extend_from_slice(
        format!(
            "pub const ICON_IMAGE: &[u8] = &{:?};pub const ICON_IMAGE_SIZE: [u32; 2] = [{}, {}];",
            map.data(),
            map.width(),
            map.height()
        )
        .as_bytes(),
    );

    let map = render_svg("./resource/drag-drop.svg")?;
    buf.extend_from_slice(format!(
        "pub const DRAG_DROP_IMAGE: &[u8] = &{:?};pub const DRAG_DROP_IMAGE_SIZE: [usize; 2] = [{}, {}];",
        map.data(),
        map.width() as usize,
        map.height() as usize
    ).as_bytes());

    fs::write(path, &buf)?;

    Ok(())
}

fn render_svg(path: impl AsRef<Path>) -> Result<Pixmap, Box<dyn error::Error + Send + Sync>> {
    let mut buf = Vec::new();

    let mut file = fs::File::open(path)?;

    file.read_to_end(&mut buf)?;

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();

    let rtree = usvg::Tree::from_data(&buf, &opt.to_ref())?;

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let [w, h] = [pixmap_size.width(), pixmap_size.height()];

    let mut map = Pixmap::new(w, h).unwrap();

    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        map.as_mut(),
    )
    .unwrap();

    Ok(map)
}

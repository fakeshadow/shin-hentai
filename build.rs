use std::{env, error, fs, io::Read, path::Path};

use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree, TreeParsing};

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
    let string = format!(
        "pub const BROKEN_IMAGE: &[u8] = &{:?};pub const BROKEN_IMAGE_SIZE: [usize; 2] = [{}, {}];",
        map.data(),
        map.width() as usize,
        map.height() as usize
    );
    buf.extend_from_slice(string.as_bytes());

    let map = render_svg("./resource/shin-hentai.svg")?;
    let string = format!(
        "#[allow(dead_code)]pub const ICON_IMAGE: &[u8] = &{:?};#[allow(dead_code)]pub const ICON_IMAGE_SIZE: [u32; 2] = [{}, {}];",
        map.data(),
        map.width(),
        map.height()
    );
    buf.extend_from_slice(string.as_bytes());

    let map = render_svg("./resource/drag-drop.svg")?;
    let string = format!(
        "pub const DRAG_DROP_IMAGE: &[u8] = &{:?};pub const DRAG_DROP_IMAGE_SIZE: [usize; 2] = [{}, {}];",
        map.data(),
        map.width() as usize,
        map.height() as usize
    );
    buf.extend_from_slice(string.as_bytes());

    fs::write(path, &buf)?;

    Ok(())
}

fn render_svg(path: impl AsRef<Path>) -> Result<Pixmap, Box<dyn error::Error + Send + Sync>> {
    let mut buf = Vec::new();

    let path = path.as_ref();

    let mut file = fs::File::open(path)?;
    file.read_to_end(&mut buf)?;

    let tree = Tree::from_data(&buf, &Options::default())?;

    let pixmap_size = tree.size.to_int_size();
    let [w, h] = [pixmap_size.width(), pixmap_size.height()];

    Pixmap::new(w, h)
        .map(|mut map| {
            resvg::Tree::from_usvg(&tree).render(Transform::default(), &mut map.as_mut());
            map
        })
        .ok_or_else(|| format!("can not render svg from path: {path:?}",).into())
}

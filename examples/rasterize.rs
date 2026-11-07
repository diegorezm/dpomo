//! Rasterizes one SVG to a transparent square PNG (used to build the app icons).
//!
//! cargo run --example rasterize -- <in.svg> <size> <out.png>

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes = std::fs::read(&args[1]).unwrap();
    let size: u32 = args[2].parse().unwrap();
    let hint = egui::SizeHint::Size {
        width: size,
        height: size,
        maintain_aspect_ratio: true,
    };
    let img =
        egui_extras::image::load_svg_bytes_with_size(&bytes, hint, &Default::default()).unwrap();
    let [w, h] = img.size;
    // egui gives premultiplied alpha; PNG wants straight alpha.
    let mut rgba = Vec::with_capacity(w * h * 4);
    for c in &img.pixels {
        let [r, g, b, a] = c.to_srgba_unmultiplied();
        rgba.extend_from_slice(&[r, g, b, a]);
    }
    let file = std::fs::File::create(&args[3]).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&rgba).unwrap();
}

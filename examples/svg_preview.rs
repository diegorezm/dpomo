//! Renders SVGs into a PNG grid so you can check character art without the app.
//!
//! cargo run --example svg_preview -- <out.png> <bg hex> <columns> <file.svg>...

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = &args[1];
    let bg = u32::from_str_radix(args[2].trim_start_matches('#'), 16).unwrap();
    let cols: usize = args[3].parse().unwrap();
    let files = &args[4..];
    let (cw, ch) = (320usize, 256usize);
    let rows = files.len().div_ceil(cols);
    let (w, h) = (cw * cols, ch * rows);
    let bgc = [(bg >> 16) as u8, (bg >> 8) as u8, bg as u8];
    let mut img = vec![0u8; w * h * 4];
    for p in img.chunks_mut(4) {
        p[..3].copy_from_slice(&bgc);
        p[3] = 255;
    }
    for (i, f) in files.iter().enumerate() {
        let bytes = std::fs::read(f).unwrap();
        let ci = egui_extras::image::load_svg_bytes_with_size(
            &bytes,
            egui::SizeHint::Size {
                width: cw as u32,
                height: ch as u32,
                maintain_aspect_ratio: true,
            },
            &Default::default(),
        )
        .unwrap();
        let (ox, oy) = ((i % cols) * cw, (i / cols) * ch);
        let [iw, ih] = ci.size;
        for y in 0..ih {
            for x in 0..iw {
                let c = ci.pixels[y * iw + x];
                let a = c.a() as f32 / 255.0;
                let o = ((oy + y) * w + ox + x) * 4;
                for k in 0..3 {
                    let src = [c.r(), c.g(), c.b()][k] as f32; // premultiplied
                    img[o + k] = (src + img[o + k] as f32 * (1.0 - a)).round() as u8;
                }
            }
        }
    }
    let file = std::fs::File::create(out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img).unwrap();
}

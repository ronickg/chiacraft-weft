//! Coats painted from genes, as the game paints them, onto a species' texture: a dozen wild
//! animals (the last few young) as PNGs, to look at on the model (the blocky-model skill's
//! `render.py`) when a model's texture or layers change.
//!
//!     cargo run -p animals_core --example coat_sheet -- <species> <texture.png> <out dir> [count] [seed]
//!
//! `<species>` is its name or its model (`chihuahua`, `cat`...). Writes `<out>/<k>.png`.

use animals_core::coats::{paint, sample, wild_pool};
use animals_core::species::{ADULT, BABY, SPECIES};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (who, tex, out) = (&a[1], &a[2], &a[3]);
    let count: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(12);
    let mut state: u64 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(7);
    let index = SPECIES.iter().position(|s| s.name == who || s.model == who).expect("no such species");
    let mut rng = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 40) as f32 / (1u64 << 24) as f32
    };
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(tex).expect("texture")));
    let mut reader = decoder.read_info().expect("png");
    let mut buf = vec![0; reader.output_buffer_size().expect("size")];
    let info = reader.next_frame(&mut buf).expect("frame");
    assert_eq!(info.color_type, png::ColorType::Rgba, "an RGBA texture");
    let (w, h) = (info.width as usize, info.height as usize);
    std::fs::create_dir_all(out).expect("out dir");
    let pool = wild_pool(index, "").expect("a gene file");
    for k in 0..count {
        let mut g = sample(index, k % 2 == 0, &pool, &mut rng).expect("a genome");
        g.stage = if k + 3 >= count { BABY } else { ADULT };
        let mut rgba = buf[..w * h * 4].to_vec();
        paint(index, &g, &mut rgba, w, h);
        let f = std::fs::File::create(format!("{out}/{k}.png")).expect("write");
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().and_then(|mut wr| wr.write_image_data(&rgba)).expect("png write");
    }
    eprintln!("{count} {} coats into {out}", SPECIES[index].name);
}

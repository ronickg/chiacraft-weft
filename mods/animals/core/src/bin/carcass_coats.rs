//! Paints the dead bodies' texture in sample coats, for `mods/animals/source/carcass/sheet.py --coats`:
//! `cargo run -p animals_core --bin carcass_coats -- <body.png> <out dir> [species ...]` writes
//! `<out>/<species>_<n>.png` (four wild individuals each, their genes drawn from the species' wild pool) through
//! `coats::paint_dead`, as the game paints a carcass.

use animals_core::carcass::{COAT_RGB, UNDER_RGB, body_scale, plan_of};
use animals_core::coats::{paint_dead, sample, wild_pool};
use animals_core::species::SPECIES;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(src), Some(out)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: carcass_coats <body.png> <out dir> [species ...]");
        std::process::exit(2);
    };
    let mut d = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(src).expect("texture")));
    d.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut r = d.read_info().expect("png");
    let mut tex = vec![0; r.output_buffer_size().expect("size")];
    let info = r.next_frame(&mut tex).expect("frame");
    let (w, h) = (info.width as usize, info.height as usize);
    let names: Vec<&str> = if args.len() > 3 {
        args[3..].iter().map(String::as_str).collect()
    } else {
        vec!["grey_wolf", "jaguar", "white_tailed_deer", "cat", "golden_eagle", "raptor", "yacare_caiman", "pacu"]
    };
    let mut seed = 12345u32;
    let mut rng = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 1_000_000) as f32 / 1_000_000.0
    };
    for name in names {
        let Some(i) = SPECIES.iter().position(|s| s.name == name) else {
            eprintln!("{name}: no such species");
            continue;
        };
        let Some(pool) = wild_pool(i, "") else {
            eprintln!("{name}: no gene file");
            continue;
        };
        let s = &SPECIES[i];
        let scale = body_scale(s.bio.mass, plan_of(s.model));
        for n in 0..4 {
            let Some(g) = sample(i, n % 2 == 0, &pool, &mut rng) else { continue };
            let mut rgba = tex.clone();
            paint_dead(&g, &mut rgba, w, h, scale, COAT_RGB, UNDER_RGB);
            let path = format!("{out}/{name}_{n}.png");
            let f = std::io::BufWriter::new(std::fs::File::create(&path).expect("out"));
            let mut e = png::Encoder::new(f, w as u32, h as u32);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .and_then(|mut wr| wr.write_image_data(&rgba))
                .expect("write");
            println!("{path}");
        }
    }
}

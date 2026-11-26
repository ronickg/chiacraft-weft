//! Every overworld biome's share and how far a new player walks to the nearest of it: the biomes
//! as the rules pick them (`land::overworld_biome`), plus the ocean (its share of all the ground)
//! and the mesas (columns the mesas lift, a share of the land), sampled every `step` blocks out
//! to `radius`. Distances are straight lines from the world's middle (spawn), in blocks.
//!
//!     cargo run -p base_api --example land_survey -- <seed> <radius> <step>

use base_api::land as l;

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(8000) as i32;
    let step = a.get(2).copied().unwrap_or(48) as usize;
    let f = l::fields(seed);
    let mut rows: Vec<(&str, usize, i64)> = Vec::new();
    let (mut all, mut land) = (0usize, 0usize);
    let mut add = |name: &'static str, d2: i64| match rows.iter_mut().find(|r| r.0 == name) {
        Some(r) => (r.1, r.2) = (r.1 + 1, r.2.min(d2)),
        None => rows.push((name, 1, d2)),
    };
    for z in (-radius..=radius).step_by(step) {
        for x in (-radius..=radius).step_by(step) {
            let d2 = x as i64 * x as i64 + z as i64 * z as i64;
            if d2 > radius as i64 * radius as i64 {
                continue;
            }
            all += 1;
            let b = l::overworld_biome(&f, x * 16, z * 16);
            add(b, d2);
            if b != "ocean" {
                land += 1;
                if l::mesa(&f, x * 16, z * 16) > 0 {
                    add("(mesas)", d2);
                }
            }
        }
    }
    rows.sort_by_key(|r| r.0);
    println!("seed {seed}, a circle {radius} blocks round spawn, every {step}");
    for (name, n, d2) in rows {
        let of = if name == "ocean" { all } else { land };
        println!("  {name:14} {:5.1}%  nearest {:5} blocks", n as f64 * 100.0 / of.max(1) as f64, (d2 as f64).sqrt().round());
    }
}

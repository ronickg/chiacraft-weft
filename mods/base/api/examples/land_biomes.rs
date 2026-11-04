//! Each overworld biome's share of the land and how near the middle it first turns up, as the
//! rules pick it (`base_api::land::overworld_biome`), and the climate fields' spread.
//!
//!     cargo run -p base_api --example land_biomes -- <seed> <radius> <step>

use base_api::land as l;
use weft_sdk::noise::fixed::Fields;

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(16000) as i32;
    let step = a.get(2).copied().unwrap_or(64) as usize;
    let f = Fields { seed };
    let (mut ts, mut hs) = (Vec::new(), Vec::new());
    for z in (-radius..=radius).step_by(step * 2) {
        for x in (-radius..=radius).step_by(step * 2) {
            ts.push(f.temperature(x * 16, z * 16));
            hs.push(f.humidity(x * 16, z * 16));
        }
    }
    ts.sort();
    hs.sort();
    let spread = |v: &Vec<i32>| {
        [5, 20, 50, 80, 95]
            .map(|p| format!("{p}%: {:.2}", v[v.len() * p / 100] as f64 / 4096.0))
            .join("  ")
    };
    println!("seed {seed}, {radius} blocks out, every {step}");
    println!("  temperature  {}", spread(&ts));
    println!("  humidity     {}", spread(&hs));
    for (b, share, near) in l::biome_shares(seed, radius, step) {
        // A spot well inside it near the middle: its 8 neighbours 24 blocks out are the same.
        let inside = |x: i32, z: i32| (-1..=1).all(|i| (-1..=1).all(|k| l::overworld_biome(&f, (x + i * 24) * 16, (z + k * 24) * 16) == b));
        let spot = (0..radius / 16)
            .flat_map(|r| (0..8 * r.max(1)).map(move |i| (r, i)))
            .map(|(r, i)| {
                let (r, i) = (r * 16, i * 16);
                match i / (2 * r).max(1) {
                    0 => (-r + i, -r),
                    1 => (r, -r + i % (2 * r).max(1)),
                    2 => (r - i % (2 * r).max(1), r),
                    _ => (-r, r - i % (2 * r).max(1)),
                }
            })
            .find(|&(x, z)| inside(x, z));
        let at =
            spot.map_or(String::new(), |(x, z)| format!(", a spot inside at x {x} z {z} (ground {})", l::height(&f, x * 16, z * 16) >> 4));
        println!("  {b:14} {:5.1}% of the land, nearest {near} blocks out{at}", share * 100.0);
    }
}

//! The biggest giant sequoia groves near the world's middle: grove patches by their share of a
//! 160-block square round each point on a 32-block grid, with a spot inside and its ground, for
//! look scripts (Plants).
//!
//!     cargo run -p base_api --example grove_find -- <seed> <radius>
use base_api::land as l;

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(4000) as i32;
    let f = l::fields(seed);
    let grove = |x: i32, z: i32| l::overworld_biome(&f, x * 16, z * 16) == "sequoia_grove";
    let mut best: Vec<(usize, i32, i32)> = Vec::new();
    for z in (-radius..=radius).step_by(32) {
        for x in (-radius..=radius).step_by(32) {
            if !grove(x, z) {
                continue;
            }
            let n = (-5..=5)
                .flat_map(|i| (-5..=5).map(move |k| (i, k)))
                .filter(|&(i, k)| grove(x + i * 16, z + k * 16))
                .count();
            best.push((n, x, z));
        }
    }
    best.sort_by(|a, b| b.cmp(a));
    let mut shown: Vec<(i32, i32)> = Vec::new();
    for (n, x, z) in best {
        if shown
            .iter()
            .any(|&(sx, sz)| (sx - x).abs() < 400 && (sz - z).abs() < 400)
        {
            continue;
        }
        let ground = l::height(&f, x * 16, z * 16) >> 4;
        println!("grove {n}/121 of the 160-block square at x {x} z {z}, ground {ground}");
        shown.push((x, z));
        if shown.len() == 6 {
            break;
        }
    }
}

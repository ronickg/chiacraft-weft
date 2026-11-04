//! The nearest spots to spawn of a kind, with a camera for a look script: for testers, so a shot
//! of a lake or a river shows one. Kinds: `lake`, `river` (water at least two blocks deep in the
//! channel), `coast` (land with the sea beside it), `karst` (a tower) or any overworld biome by name (`desert`,
//! `sequoia_grove`, ...). Searched in rings outward from the middle every `step` blocks, the
//! world as the engine builds it (the overworld's fields, its continent scale).
//!
//! For each spot: the ground there, and a camera 30 blocks to its north (-z) facing it (+z,
//! yaw 180), high enough to clear the ground under it and the spot by 20 blocks, as testkit
//! lines.
//!
//!     cargo run -p base_api --example land_find -- <seed> <kind> [count] [step]

use base_api::SEA;
use base_api::land::{self as l, Water};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let kind = a.get(1).cloned().unwrap_or_else(|| "lake".into());
    let count: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let step: i32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(16);
    let f = l::fields(seed);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    let is = |x: i32, z: i32| -> bool {
        match kind.as_str() {
            "lake" | "river" => {
                let want = if kind == "lake" { Water::Lake } else { Water::River };
                l::bank(&f, x * 16, z * 16).is_some_and(|b| b.water == want && b.in_water) && h(x, z) <= SEA - 2
            }
            "coast" => {
                h(x, z) >= SEA
                    && [(4, 0), (-4, 0), (0, 4), (0, -4)]
                        .iter()
                        .any(|(dx, dz)| l::overworld_biome(&f, (x + dx) * 16, (z + dz) * 16) == "ocean")
            }
            "karst" => l::karst(&f, x * 16, z * 16) > 16 * 12,
            b => l::overworld_biome(&f, x * 16, z * 16) == b,
        }
    };
    let mut found: Vec<(i32, i32)> = Vec::new();
    let far = 12_000 / step;
    'rings: for r in 0..far {
        for i in -r..=r {
            for (x, z) in [(i, -r), (i, r), (-r, i), (r, i)] {
                let (x, z) = (x * step, z * step);
                if found
                    .iter()
                    .any(|&(fx, fz)| (fx - x).abs().max((fz - z).abs()) < 300)
                {
                    continue;
                }
                if is(x, z) {
                    found.push((x, z));
                    if found.len() >= count {
                        break 'rings;
                    }
                }
            }
        }
    }
    if found.is_empty() {
        println!("no {kind} within 12 km on seed {seed}");
    }
    for (x, z) in found {
        let ground = h(x, z);
        let (cx, cz) = (x, z - 30);
        let cam = ground.max(h(cx, cz)).max(SEA) + 20;
        let dist = ((x as f64).powi(2) + (z as f64).powi(2)).sqrt().round();
        let water = if ground < SEA { format!(", water {} deep", SEA - ground) } else { String::new() };
        println!("# {kind} at x {x} z {z} (ground {ground}{water}), {dist} blocks from spawn");
        println!("chat /tp {cx} {cam} {cz}\nidle\nlook 180 -30\nwait 2\nwhere\nshot {kind}_{}_{}.png", x, z);
    }
}

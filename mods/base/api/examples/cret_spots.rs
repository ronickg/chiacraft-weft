//! Spots in the Cretaceous (`mods/animals/src/cretaceous.rs`): the overworld's land and fields
//! with its own biome rules, mirrored here (keep in step): coral reef and lagoon in the warm
//! seas, ocean, beach, crags on the ranges, floodplain on low wet ground, red dunes, scrubland and
//! fern forest by humidity. The nearest of each to (`x`, `z`), searched in rings every 8 blocks,
//! with a camera 2 blocks over the ground (or over the sea).
//!
//!     cargo run -p base_api --example cret_spots -- <seed> <x> <z>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::q12;

fn main() {
    let a: Vec<i64> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let (cx, cz) = (a.get(1).copied().unwrap_or(0) as i32, a.get(2).copied().unwrap_or(0) as i32);
    let f = l::fields(seed);
    let biome = |x: i32, z: i32| -> (&'static str, i32) {
        let (xq, zq) = (x * 16, z * 16);
        let h = l::height(&f, xq, zq) >> 4;
        let (t, hu, c) = (f.temperature(xq, zq), f.humidity(xq, zq), f.continent(xq, zq));
        let warm = t >= q12(l::REEF_WARM_RULE) && c < q12(l::REEF_COAST);
        let name = if warm && h == SEA - 2 {
            "coral_reef"
        } else if warm && (SEA - l::REEF_LAGOON.1..=SEA - l::REEF_LAGOON.0).contains(&h) {
            "lagoon"
        } else if h <= SEA - 2 && c < q12(l::OCEAN_CONTINENT) {
            "ocean"
        } else if h <= SEA + 1 && c < q12(0.1) {
            "beach"
        } else if l::MOUNTAINS_AREA.value(&f, xq, zq) >= q12(l::MOUNTAIN_BIOME) {
            "crags"
        } else if h <= SEA + 6 && hu >= q12(0.05) {
            "floodplain"
        } else if hu < q12(-0.12) {
            "red_dunes"
        } else if hu < q12(0.05) {
            "scrubland"
        } else {
            "fern_forest"
        };
        (name, h)
    };
    for want in ["floodplain", "crags", "red_dunes", "fern_forest", "scrubland", "lagoon", "coral_reef"] {
        'search: for r in (0..6000).step_by(8) {
            for i in (-r..=r).step_by(8) {
                for (x, z) in [(cx + i, cz - r), (cx + i, cz + r), (cx - r, cz + i), (cx + r, cz + i)] {
                    let (name, h) = biome(x, z);
                    // Well inside: the same biome 12 blocks round.
                    if name == want && [(12, 0), (-12, 0), (0, 12), (0, -12)].iter().all(|&(dx, dz)| biome(x + dx, z + dz).0 == want) {
                        println!("{want:12} x {x} z {z} ground {h}, {r} blocks out: chat /tp {x}.5 {} {z}.5", h.max(SEA) + 2);
                        break 'search;
                    }
                }
            }
        }
    }
}

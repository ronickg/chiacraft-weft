//! Spots for a look at the air (`vitals::air`: its blend across borders, the forest's calm): the
//! nearest border to the spawn where the biomes' temperatures differ by `step_c` °C or more, with
//! the blended temperature every 4 blocks across it; and a forest's interior (all its ring 40 blocks
//! round the same forest) and its edge (another biome within 6 blocks).
//!
//!     cargo run -p base_api --example air_spots -- <seed> [step_c] [forest]

use base_api::land as l;
use vitals::air;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let step_c: f32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(8.0);
    let forest = a
        .get(2)
        .cloned()
        .unwrap_or_else(|| "tropical_forest".into());
    let f = l::fields(seed);
    let [sx, _, sz] = l::spawn(seed);
    let biome = |x: i32, z: i32| l::overworld_biome(&f, x * 16, z * 16);
    let celsius = |name: &str| vitals::climate::of(&format!("base:{name}")).map_or(f32::NAN, |c| c.celsius);
    let blended = |x: i32, z: i32| air::blend(air::ring(x, z, air::AIR_BLEND / 2).map(|(x, z)| celsius(biome(x, z))));
    // Rings out from the spawn, every 8 blocks: the first east-west border with that step.
    'border: for r in (8..3000).step_by(8) {
        for i in (-r..=r).step_by(8) {
            for (x, z) in [(sx + i, sz - r), (sx + i, sz + r), (sx - r, sz + i), (sx + r, sz + i)] {
                let (b0, b1) = (biome(x, z), biome(x + 8, z));
                if b0 != b1 && (celsius(b0) - celsius(b1)).abs() >= step_c && !celsius(b0).is_nan() && !celsius(b1).is_nan() {
                    println!("# border {b0} ({:.0} °C) | {b1} ({:.0} °C) at x {x}..{} z {z}", celsius(b0), celsius(b1), x + 8);
                    for dx in (-24..=32).step_by(4) {
                        let h = l::height(&f, (x + dx) * 16, z * 16) >> 4;
                        println!("x {:5} ground {h:3}: {:16} blended {:5.1} °C", x + dx, biome(x + dx, z), blended(x + dx, z));
                    }
                    break 'border;
                }
            }
        }
    }
    // The forest.
    let mut found = (None, None);
    'forest: for r in (8..4000).step_by(16) {
        for i in (-r..=r).step_by(16) {
            for (x, z) in [(sx + i, sz - r), (sx + i, sz + r), (sx - r, sz + i), (sx + r, sz + i)] {
                if biome(x, z) != forest {
                    continue;
                }
                let inside = air::ring(x, z, 40)
                    .iter()
                    .all(|&(x, z)| biome(x, z) == forest);
                let edge = air::ring(x, z, 6)
                    .iter()
                    .any(|&(x, z)| biome(x, z) != forest);
                if inside && found.0.is_none() {
                    found.0 = Some((x, z, l::height(&f, x * 16, z * 16) >> 4));
                }
                if edge && found.1.is_none() {
                    found.1 = Some((x, z, l::height(&f, x * 16, z * 16) >> 4));
                }
                if found.0.is_some() && found.1.is_some() {
                    break 'forest;
                }
            }
        }
    }
    println!("# {forest}: inside {:?}, edge {:?} (x, z, ground)", found.0, found.1);
}

//! The coasts (`base_api::land`: `COAST_CALM`, `COAST_CLIFF_AREA`, `OCEAN_DEEP_AREA`,
//! `ISLANDS_AREA`): their shares, spots to look at (a sea cliff, an island, a beach), and a
//! cross-section from the land out to the deep sea at each, as text (blocks over the sea every 4).
//!
//!     cargo run -p base_api --example land_coasts -- <seed> <radius> <step>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::q12;

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(4096) as i32;
    let step = a.get(2).copied().unwrap_or(8) as usize;
    let f = l::fields(seed);
    let (mut shore, mut cliff, mut beach, mut island, mut deep, mut sea) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut spots: Vec<(&str, i32, i32)> = Vec::new();
    let far = |s: &[(&str, i32, i32)], k: &str, x: i32, z: i32| {
        s.iter()
            .all(|p| p.0 != k || (p.1 - x).abs() + (p.2 - z).abs() > 1500)
    };
    for z in (-radius..=radius).step_by(step) {
        for x in (-radius..=radius).step_by(step) {
            let (qx, qz) = (x * 16, z * 16);
            let h = l::height(&f, qx, qz) >> 4;
            let c = f.continent(qx, qz);
            if h < SEA {
                sea += 1;
                if l::OCEAN_DEEP_AREA.value(&f, qx, qz) > q12(0.5) {
                    deep += 1;
                }
                continue;
            }
            // Land by the sea: a neighbour `step` away is under it.
            let wet = [(step as i32, 0), (-(step as i32), 0), (0, step as i32), (0, -(step as i32))]
                .iter()
                .any(|(dx, dz)| l::height(&f, (x + dx) * 16, (z + dz) * 16) >> 4 < SEA);
            if l::ISLANDS_AREA.value(&f, qx, qz) > q12(0.3) {
                island += 1;
                if far(&spots, "island", x, z) && spots.iter().filter(|p| p.0 == "island").count() < 3 {
                    spots.push(("island", x, z));
                }
            }
            if !wet {
                continue;
            }
            shore += 1;
            if l::COAST_CLIFF_AREA.value(&f, qx, qz) > q12(0.8) && h >= SEA + 8 {
                cliff += 1;
                if far(&spots, "cliff", x, z) && spots.iter().filter(|p| p.0 == "cliff").count() < 3 {
                    spots.push(("cliff", x, z));
                }
            } else if h <= SEA + 2 && c < q12(0.1) {
                beach += 1;
                if far(&spots, "beach", x, z) && spots.iter().filter(|p| p.0 == "beach").count() < 3 {
                    spots.push(("beach", x, z));
                }
            }
        }
    }
    let pc = |n: usize, of: usize| n as f64 * 100.0 / of.max(1) as f64;
    println!("seed {seed}, {radius} out every {step}");
    println!(
        "  shore: cliffs {:.0}%, beaches {:.0}%;  sea: deep {:.0}%;  islands' land {:.2}% of all",
        pc(cliff, shore),
        pc(beach, shore),
        pc(deep, sea),
        pc(island, ((2 * radius as usize / step) + 1).pow(2))
    );
    for (kind, x, z) in &spots {
        // Toward the lowest of 8 directions 64 blocks out: the sea.
        let (dx, dz) = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]
            .into_iter()
            .min_by_key(|(dx, dz)| l::height(&f, (x + dx * 64) * 16, (z + dz * 64) * 16))
            .unwrap();
        let line: Vec<String> = (-12..=40)
            .map(|i| (l::height(&f, (x + dx * i * 4) * 16, (z + dz * i * 4) * 16) >> 4) - SEA)
            .map(|h| h.to_string())
            .collect();
        println!("  {kind:6} x {x} z {z}, toward ({dx}, {dz}): {}", line.join(" "));
    }
}

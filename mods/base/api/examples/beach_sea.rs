//! How far the beaches are from the sea: every `step` blocks out to `radius`, each column the rules
//! call beach (or snowy coast: low, `continent < 0.1`), its distance to the nearest sea (ground
//! under the sea's level before the rivers and lakes) within 64 blocks, by the continent value. For
//! choosing a beach rule that keeps the sea's beaches and drops river flats inland.
//!
//!     cargo run -p base_api --example beach_sea -- <seed> <radius> <step>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::q12;

fn main() {
    let a: Vec<i32> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(4000);
    let step = a.get(2).copied().unwrap_or(16);
    let f = l::fields(seed);
    let sea = |x: i32, z: i32| l::land_height(&f, x * 16, z * 16) < (SEA << 4);
    // (continent bucket -0.02.., then by 0.01 to 0.05, then 0.05-0.10): [near (<=16), mid (<=40), far]
    let mut by_c = [[0u32; 3]; 8];
    let mut total = 0;
    for z in (-radius..=radius).step_by(step as usize) {
        for x in (-radius..=radius).step_by(step as usize) {
            let h = l::height(&f, x * 16, z * 16) >> 4;
            let c = f.continent(x * 16, z * 16);
            if !(h <= SEA + 1 && h > SEA - 2 && c < q12(0.1)) || sea(x, z) {
                continue;
            }
            total += 1;
            // Distance to the sea: rings of 4 blocks out to 64.
            let d = (4..=64)
                .step_by(4)
                .find(|&r| {
                    (0..16).any(|k| {
                        let a = k as f64 * std::f64::consts::TAU / 16.0;
                        sea(x + (r as f64 * a.cos()) as i32, z + (r as f64 * a.sin()) as i32)
                    })
                })
                .unwrap_or(999);
            let b = (((c as f64 / 4096.0) + 0.02) / 0.01).clamp(0.0, 7.0) as usize;
            by_c[b][if d <= 16 {
                0
            } else if d <= 40 {
                1
            } else {
                2
            }] += 1;
        }
    }
    println!("beach columns (low, continent < 0.1, not sea): {total}");
    // What they are now: a beach (under `BEACH_CONTINENT`), sand by the sea (`sea_sand`: a node
    // every 4 blocks within `SEA_SAND` under the sea), or the land's ground; by distance.
    let mut now = [[0u32; 3]; 3];
    for z in (-radius..=radius).step_by(step as usize) {
        for x in (-radius..=radius).step_by(step as usize) {
            let h = l::height(&f, x * 16, z * 16) >> 4;
            let c = f.continent(x * 16, z * 16);
            if !(h <= SEA + 1 && h > SEA - 2 && c < q12(0.1)) || sea(x, z) || f.temperature(x * 16, z * 16) < q12(l::SNOWY_BELOW) {
                continue;
            }
            let near = |r: i32| {
                (0..16).any(|k| {
                    let a = k as f64 * std::f64::consts::TAU / 16.0;
                    sea(x + (r as f64 * a.cos()) as i32, z + (r as f64 * a.sin()) as i32)
                })
            };
            let d = if (4..=16).step_by(4).any(near) {
                0
            } else if (20..=40).step_by(4).any(near) {
                1
            } else {
                2
            };
            let kind = if c < q12(l::BEACH_CONTINENT) {
                0
            } else {
                let n = l::SEA_SAND / 4;
                let (bx, bz) = (x.div_euclid(4) * 4, z.div_euclid(4) * 4);
                let sand = (-n..=n + 1).any(|i| {
                    (-n..=n + 1).any(|j| {
                        let (nx, nz) = (bx + i * 4, bz + j * 4);
                        let (dx, dz) = (nx - x, nz - z);
                        dx * dx + dz * dz <= l::SEA_SAND * l::SEA_SAND && sea(nx, nz)
                    })
                });
                if sand && h >= SEA - 1 { 1 } else { 2 }
            };
            now[d][kind] += 1;
        }
    }
    // The nearest spot to the spawn of each: beach, sand by the sea, and land ground on a low flat
    // with no sea within 40 (a river's flat inland).
    let [sx, _, sz] = l::spawn(seed);
    let mut spots: [Option<(i32, i32, i32)>; 3] = [None; 3];
    'rings: for r in (0..3000).step_by(8) {
        for i in (-r..=r).step_by(8) {
            for (x, z) in [(sx + i, sz - r), (sx + i, sz + r), (sx - r, sz + i), (sx + r, sz + i)] {
                let h = l::height(&f, x * 16, z * 16) >> 4;
                let c = f.continent(x * 16, z * 16);
                if !(h <= SEA + 1 && h >= SEA - 1 && c < q12(0.1)) || sea(x, z) || f.temperature(x * 16, z * 16) < q12(l::SNOWY_BELOW) {
                    continue;
                }
                let near = |r: i32| {
                    (0..16).any(|k| {
                        let a = k as f64 * std::f64::consts::TAU / 16.0;
                        sea(x + (r as f64 * a.cos()) as i32, z + (r as f64 * a.sin()) as i32)
                    })
                };
                let k = if c < q12(l::BEACH_CONTINENT) && near(8) {
                    0
                } else if c >= q12(l::BEACH_CONTINENT) && near(4) {
                    1
                } else if c >= q12(l::BEACH_CONTINENT) && !(4..=40).step_by(4).any(near) {
                    2
                } else {
                    continue;
                };
                spots[k].get_or_insert((x, z, h));
                if spots.iter().all(|s| s.is_some()) {
                    break 'rings;
                }
            }
        }
    }
    for (name, s) in ["a beach", "sand by the sea", "a low flat inland"]
        .iter()
        .zip(spots)
    {
        println!("# {name}: {s:?} (x, z, ground)");
    }
    println!("\nnow, not in the cold | beach | sand by the sea | the land's ground |\n|---|---|---|---|");
    for (d, name) in ["sea within 16", "17-40", "farther"].iter().enumerate() {
        println!("| {name} | {} | {} | {} |", now[d][0], now[d][1], now[d][2]);
    }
    println!("| continent | sea within 16 | 17-40 | farther |\n|---|---|---|---|");
    for (b, c) in by_c.iter().enumerate() {
        let lo = b as f64 * 0.01 - 0.02;
        println!(
            "| {:.2}-{} | {} | {} | {} |",
            lo,
            if b == 7 { "0.10".to_string() } else { format!("{:.2}", lo + 0.01) },
            c[0],
            c[1],
            c[2]
        );
    }
}

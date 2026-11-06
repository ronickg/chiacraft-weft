//! Camera spots in the Minecraft forest test world (`base:mc_forest`) whose view matches the
//! Minecraft comparison vista (seed 1, x 165.5 z 168.5, Minecraft yaw 300 pitch 12 down, FOV 70:
//! facing east-south-east, (0.87, 0.50)): the same facing, so the sun stands where it does there,
//! and the view's land in the same shares, read from its region files (MC shaders session, 11-05):
//! within 500 blocks woods two thirds, plains a fifth, the rest water; 500-1,500 mostly plains with
//! woods and a river; peaks on the horizon; a cherry grove ahead on the right. Each candidate's
//! view is sampled in three bands across the field of view and scored against those shares.
//!
//!     cargo run -p base_api --example mc_forest_find -- <seed> [reach] [step] [count]

use base_api::SEA;
use base_api::land::{self as l};
use vitals::wild::{CHERRY_GROVE_FROM, CHERRY_GROVE_SIZE};
use weft_sdk::noise::fixed::{q4, q12};

/// Our yaw for the Minecraft vista's facing (ours: 0 faces -z, -90 faces +x).
const YAW: f64 = -120.0;
/// Half the field of view across (FOV 70 at 16:9).
const HALF_FOV: f64 = 51.0;
/// The bands: (from, to blocks, wanted shares of woods, plains, water, mountains).
const BANDS: [(f64, f64, [f64; 4]); 3] = [
    (90.0, 500.0, [0.67, 0.20, 0.13, 0.0]),
    (500.0, 1500.0, [0.35, 0.50, 0.10, 0.05]),
    (1500.0, 4000.0, [0.40, 0.40, 0.10, 0.10]),
];

/// A candidate: its score (lower is nearer), where, each band's shares, a cherry grove in view.
type View = (f64, i32, i32, Vec<[f64; 4]>, bool);

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(0, 1) as u64;
    let reach = arg(1, 6000) as i32;
    let step = arg(2, 250).max(16) as usize;
    let count = arg(3, 8) as usize;
    let f = l::fields(seed);
    let (dx, dz) = (-YAW.to_radians().sin(), -YAW.to_radians().cos());
    let grove = |x: i32, z: i32| f.fbm(0xC4E77, x * 16, z * 16, q4(CHERRY_GROVE_SIZE), 1) >= q12(CHERRY_GROVE_FROM);
    let mut found: Vec<View> = Vec::new();
    for z in (-reach..=reach).step_by(step) {
        for x in (-reach..=reach).step_by(step) {
            // The camera over land that isn't mountains, so the near band is the low country.
            let h = l::height(&f, x * 16, z * 16) >> 4;
            if !(SEA..=SEA + 30).contains(&h) {
                continue;
            }
            let mut score = 0.0;
            let mut got = Vec::new();
            let mut cherry = false;
            for (bi, &(from, to, want)) in BANDS.iter().enumerate() {
                let mut n = [0.0f64; 4];
                let mut all = 0.0;
                for ai in 0..9 {
                    let ang = (-HALF_FOV + ai as f64 * HALF_FOV / 4.0).to_radians();
                    let (ex, ez) = (dx * ang.cos() - dz * ang.sin(), dz * ang.cos() + dx * ang.sin());
                    for di in 0..6 {
                        let d = from + (to - from) * (di as f64 + 0.5) / 6.0;
                        let (px, pz) = (x + (ex * d) as i32, z + (ez * d) as i32);
                        let ph = l::height(&f, px * 16, pz * 16) >> 4;
                        let biome = l::mc_forest_biome(&f, px * 16, pz * 16);
                        let k = if ph < SEA {
                            2
                        } else {
                            match biome {
                                "mountains" => 3,
                                "plains" | "beach" | "snowy_plains" => 1,
                                _ => 0,
                            }
                        };
                        n[k] += 1.0;
                        all += 1.0;
                        // A grove ahead on the right, in woods that grow one.
                        if bi < 2 && ai > 4 && k == 0 && biome != "mountains" && grove(px, pz) {
                            cherry = true;
                        }
                    }
                }
                let shares = n.map(|v| v / all);
                let w = if bi == 0 { 3.0 } else { 1.0 };
                score += w * shares.iter().zip(want).map(|(g, t)| (g - t) * (g - t)).sum::<f64>();
                got.push(shares);
            }
            if !cherry {
                score += 0.15;
            }
            found.push((score, x, z, got, cherry));
        }
    }
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!("# best views like the Minecraft vista (yaw {YAW}, shares woods/plains/water/mountains near | mid | far)");
    for (score, x, z, got, cherry) in found.into_iter().take(count) {
        let s = got
            .iter()
            .map(|g| format!("{:.2}/{:.2}/{:.2}/{:.2}", g[0], g[1], g[2], g[3]))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("{score:.3}  /tp {x} 200 {z}  look {YAW} -12   {s}   cherry {cherry}");
    }
}

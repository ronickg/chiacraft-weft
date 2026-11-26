//! The best old-growth vistas of a seed, for the look sessions' Bliss comparisons (Ronald, 11-05:
//! "a biome looking similar to the one shown in bliss, will make testing even easier then"): a
//! camera at `y` looking out over the old-growth forest, scored by what the view holds out to
//! 3.2 km (a 70° wedge, each distance by the share of the picture it fills): old growth and other woods are good, water
//! inland (rivers, lakes) good up to an eighth, mountains, snow, desert, open grass and the sea
//! bad. Also each seed's old-growth share of the land.
//!
//!     cargo run -p base_api --example old_growth_find -- <seed> [count] [reach] [y]
//!     cargo run -p base_api --example old_growth_find -- 0 shares   (each test seed's biome shares)
//!
//! Prints testkit lines for each: `chat /tp x y z` and `look yaw -10` (yaw 0 faces -z, 90 -x).

use base_api::SEA;
use base_api::land::{self as l};

const STEP: i32 = 32;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let count: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let reach: i32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(8000);
    let y: i32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(210);
    if a.get(1).is_some_and(|s| s == "shares") {
        for seed in [1, 2, 3, 7, 42, 1234] {
            let mut sh = l::biome_shares(seed, 6000, 48);
            sh.sort_by(|a, b| b.1.total_cmp(&a.1));
            let line: Vec<String> = sh
                .iter()
                .map(|(b, s, d)| format!("{b} {:.1}% ({d})", s * 100.0))
                .collect();
            println!("seed {seed}: {}", line.join(", "));
        }
        return;
    }
    let f = l::fields(seed);
    let far = reach + 3400;
    let n = (2 * far / STEP + 1) as usize;
    // Each 32-block cell: 0 old growth, 1 other woods, 2 inland water, 3 bad, 4 the rest; and
    // which bad (5 mountains, 6 plains, 7 the sea, 8 the rest).
    let mut grid = vec![4u8; n * n];
    let mut why = vec![0u8; n * n];
    let (mut land, mut old) = (0usize, 0usize);
    for j in 0..n {
        for i in 0..n {
            let (x, z) = (-far + i as i32 * STEP, -far + j as i32 * STEP);
            let h = l::height(&f, x * 16, z * 16) >> 4;
            let b = l::overworld_biome(&f, x * 16, z * 16);
            let k = match b {
                "old_growth_forest" if h >= SEA => 0,
                "forest" | "birch_forest" | "taiga" | "sequoia_grove" | "tropical_forest" if h >= SEA => 1,
                "ocean" | "lagoon" | "coral_reef" | "mountains" | "snowy_plains" | "desert" | "savanna" | "plains" | "beach" => 3,
                _ if h < SEA => 2,
                _ => 4,
            };
            grid[j * n + i] = k;
            why[j * n + i] = match b {
                "mountains" => 5,
                "plains" => 6,
                "ocean" | "lagoon" | "coral_reef" | "beach" => 7,
                _ => 8,
            };
            if b != "ocean" && (x.abs() <= reach && z.abs() <= reach) {
                land += 1;
                old += (b == "old_growth_forest") as usize;
            }
        }
    }
    println!("# seed {seed}: old growth is {:.2}% of the land within {reach}", old as f64 * 100.0 / land.max(1) as f64);
    let cell = |x: i32, z: i32| -> u8 {
        let (i, j) = ((x + far) / STEP, (z + far) / STEP);
        if i < 0 || j < 0 || i >= n as i32 || j >= n as i32 { 3 } else { grid[j as usize * n + i as usize] }
    };
    let bad = |x: i32, z: i32| -> u8 {
        let (i, j) = ((x + far) / STEP, (z + far) / STEP);
        if i < 0 || j < 0 || i >= n as i32 || j >= n as i32 { 8 } else { why[j as usize * n + i as usize] }
    };
    let mut best: Vec<(f64, i32, i32, i32, [f64; 4], [f64; 4])> = Vec::new();
    for cz in (-reach..=reach).step_by(128) {
        for cx in (-reach..=reach).step_by(128) {
            // The camera's own spot over old growth.
            if cell(cx, cz) != 0 {
                continue;
            }
            for yaw in (0..360).step_by(15) {
                let r = (yaw as f64).to_radians();
                let mut s = [0.0f64; 5];
                let mut b = [0.0f64; 4];
                let mut wsum = 0.0;
                for d in (120..=3200).step_by(80) {
                    for k in -7..=7 {
                        let a = r + (k as f64 * 5.0).to_radians();
                        let (dx, dz) = (-a.sin(), -a.cos());
                        let px = cx + (dx * d as f64) as i32;
                        let pz = cz + (dz * d as f64) as i32;
                        // By the share of the picture it fills: from y over the ground at a pitch of
                        // -10 a step of ground at distance d spans about h/d^2 of the frame's height,
                        // so the first 800 blocks fill its lower half and the far land a thin band
                        // (kept at a floor so the horizon still counts).
                        let w = (400.0 / d as f64).powi(2).min(1.0) + 0.02;
                        let c = cell(px, pz);
                        s[c as usize] += w;
                        if c == 3 {
                            b[bad(px, pz) as usize - 5] += w;
                        }
                        wsum += w;
                    }
                }
                let sh: Vec<f64> = s.iter().map(|v| v / wsum).collect();
                // The sea far off is in Bliss's view too: it counts a third as bad.
                let bad = sh[3] - b[2] / wsum * 2.0 / 3.0;
                let score = sh[0] + 0.6 * sh[1] + 3.0 * sh[2].min(0.12) - 1.5 * bad;
                best.push((score, cx, cz, yaw, [sh[0], sh[1], sh[2], sh[3]], b.map(|v| v / wsum)));
            }
        }
    }
    best.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut shown: Vec<(i32, i32)> = Vec::new();
    for (score, x, z, yaw, sh, b) in best {
        if shown
            .iter()
            .any(|&(sx, sz)| (sx - x).abs() < 1500 && (sz - z).abs() < 1500)
        {
            continue;
        }
        shown.push((x, z));
        println!(
            "# score {score:.2}: old growth {:.0}%, woods {:.0}%, water {:.0}%, bad {:.0}% (mountains {:.0}, plains {:.0}, sea {:.0}, other {:.0}) (ground {})",
            sh[0] * 100.0,
            sh[1] * 100.0,
            sh[2] * 100.0,
            sh[3] * 100.0,
            b[0] * 100.0,
            b[1] * 100.0,
            b[2] * 100.0,
            b[3] * 100.0,
            l::height(&f, x * 16, z * 16) >> 4
        );
        println!("chat /tp {x} {y} {z}\nlook {yaw} -10");
        if shown.len() >= count {
            break;
        }
    }
}

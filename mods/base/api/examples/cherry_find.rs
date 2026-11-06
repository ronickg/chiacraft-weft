//! Wild cherry groves near a spot (`vitals::wild::CHERRY_GROVES`): the grove patches (the same
//! noise and edge as the scatter's thin mask in `mods/base/src/worldgen.rs`) over a biome that
//! grows them, nearest first, with a camera 40 blocks over the ground 30 blocks south of each
//! (+z), looking north (yaw 0) and down at it, as testkit lines.
//!
//!     cargo run -p base_api --example cherry_find -- <seed> <x> <z> [count] [reach]

use base_api::land::{self as l};
use vitals::wild::{CHERRY_GROVE_FROM, CHERRY_GROVE_SIZE, CHERRY_GROVES};
use weft_sdk::noise::fixed::{q4, q12};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(0, 1) as u64;
    let (cx, cz) = (arg(1, 0) as i32, arg(2, 0) as i32);
    let count = arg(3, 5) as usize;
    let reach = arg(4, 1500) as i32;
    let f = l::fields(seed);
    let grove = |x: i32, z: i32| f.fbm(0xC4E77, x * 16, z * 16, q4(CHERRY_GROVE_SIZE), 1) >= q12(CHERRY_GROVE_FROM);
    // The share of the ground in grove patches, over 2 km round the spot.
    let (mut n, mut g) = (0, 0);
    for z in (cz - 1000..cz + 1000).step_by(4) {
        for x in (cx - 1000..cx + 1000).step_by(4) {
            n += 1;
            g += grove(x, z) as i32;
        }
    }
    println!("# grove patches cover {:.1}% of the ground", g as f64 * 100.0 / n as f64);
    let mut found: Vec<(i32, i32, i32, &str)> = Vec::new();
    for r in (0..=reach).step_by(8) {
        for k in 0..(r / 8).max(1) * 8 {
            let t = k as f64 / ((r / 8).max(1) * 8) as f64 * std::f64::consts::TAU;
            let (x, z) = (cx + (r as f64 * t.cos()) as i32, cz + (r as f64 * t.sin()) as i32);
            if !grove(x, z)
                || found
                    .iter()
                    .any(|&(fx, fz, _, _)| (fx - x).abs() < 120 && (fz - z).abs() < 120)
            {
                continue;
            }
            let b = l::overworld_biome(&f, x * 16, z * 16);
            if !CHERRY_GROVES
                .iter()
                .any(|(g, _)| g.trim_start_matches("base:") == b)
            {
                continue;
            }
            found.push((x, z, l::height(&f, x * 16, z * 16) >> 4, b));
            if found.len() >= count {
                break;
            }
        }
        if found.len() >= count {
            break;
        }
    }
    for (x, z, h, b) in found {
        println!(
            "# a {b} grove at x {x} z {z} (ground {h}), {} from the spot",
            (((x - cx).pow(2) + (z - cz).pow(2)) as f64).sqrt().round()
        );
        println!("chat /tp {x} {} {}\nlook 0 -35", h + 40, z + 30);
    }
}

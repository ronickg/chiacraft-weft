//! The Minecraft forest test world's biomes (`base_api::land::MC_MIX`, dimension `base:mc_forest`)
//! beside the overworld's on the same land: a map of each, lit by the ground's height, with the
//! water, and the test world's biome shares in the frame. For finding its test spots.
//!
//!     cargo run -p base_api --example mc_forest_map -- <seed> <x> <z> <blocks across> <blocks a pixel> <out.ppm>

use base_api::SEA;
use base_api::land::{self as l};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(1, 1) as u64;
    let (cx, cz) = (arg(2, 0) as i32, arg(3, 0) as i32);
    let across = arg(4, 3072) as i32;
    let step = arg(5, 4).max(1) as i32;
    let out = a.get(6).cloned().unwrap_or_else(|| "mc_forest_map.ppm".into());
    let f = l::fields(seed);
    let n = (across / step) as usize;
    let (x0, z0) = (cx - across / 2, cz - across / 2);
    let mut hs = vec![0i32; (n + 1) * (n + 1)];
    for j in 0..=n {
        for i in 0..=n {
            hs[j * (n + 1) + i] = l::height(&f, (x0 + i as i32 * step) * 16, (z0 + j as i32 * step) * 16) >> 4;
        }
    }
    let w = 2 * n + 8;
    let mut img = vec![20u8; w * n * 3];
    let mut shares: Vec<(&str, usize)> = Vec::new();
    let mut land = 0usize;
    for side in 0..2 {
        for j in 0..n {
            for i in 0..n {
                let (x, z) = ((x0 + i as i32 * step) * 16, (z0 + j as i32 * step) * 16);
                let h = hs[j * (n + 1) + i];
                let biome = if side == 0 { l::overworld_biome(&f, x, z) } else { l::mc_forest_biome(&f, x, z) };
                let c = if h < SEA { [40, 110, 220] } else { colour(biome) };
                if side == 1 && h >= SEA {
                    land += 1;
                    match shares.iter_mut().find(|e| e.0 == biome) {
                        Some(e) => e.1 += 1,
                        None => shares.push((biome, 1)),
                    }
                }
                let (east, south) = (hs[j * (n + 1) + i + 1], hs[(j + 1) * (n + 1) + i]);
                let lift = if h > SEA { ((h - SEA) / 3).min(40) } else { 0 };
                let shade = (((h - east) + (h - south)) * 10 / step).clamp(-60, 60) + lift - 10;
                let px = (j * w + i + side * (n + 8)) * 3;
                for k in 0..3 {
                    img[px + k] = (c[k] as i32 + shade).clamp(0, 255) as u8;
                }
            }
        }
    }
    let mut ppm = format!("P6\n{w} {n}\n255\n").into_bytes();
    ppm.extend_from_slice(&img);
    std::fs::write(&out, ppm).expect("write the map");
    shares.sort_by_key(|e| std::cmp::Reverse(e.1));
    println!("seed {seed} around {cx} {cz}, {across} blocks: left the overworld, right base:mc_forest");
    for (b, c) in shares {
        println!("  {b:<18} {:5.1} % of the land", 100.0 * c as f64 / land.max(1) as f64);
    }
}

fn colour(biome: &str) -> [u8; 3] {
    match biome {
        "ocean" => [30, 60, 170],
        "beach" => [220, 205, 150],
        "mountains" => [120, 120, 120],
        "snowy_plains" => [215, 225, 225],
        "taiga" => [60, 100, 70],
        "sequoia_grove" => [175, 70, 45],
        "desert" => [220, 200, 130],
        "savanna" => [170, 165, 80],
        "swamp" => [90, 105, 60],
        "forest" => [55, 125, 45],
        "old_growth_forest" => [25, 80, 30],
        "tropical_forest" => [25, 105, 55],
        "birch_forest" => [140, 185, 110],
        _ => [150, 190, 95],
    }
}

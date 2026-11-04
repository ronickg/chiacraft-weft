//! The nearest spots to spawn of a kind, with a camera for a look script: for testers, so a shot
//! of a lake or a river shows one. Kinds: `lake`, `river` (water at least two blocks deep in the
//! channel), `coast` (land with the sea beside it), `karst` (a tower), `canyon` (cut 24 or more), `crater`, `sinkhole` (karst), `arch`, `waterfall`, `spawn` (the fair first place) or any overworld biome by name (`desert`,
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
    if kind == "spawn" {
        let t = std::time::Instant::now();
        let [x, y, z] = l::spawn(seed);
        let ms = t.elapsed().as_millis();
        println!(
            "# spawn at x {x} z {z} (ground {y}), {} blocks from the middle, {} ({ms} ms to find)",
            ((x * x + z * z) as f64).sqrt().round(),
            l::overworld_biome(&f, x * 16, z * 16)
        );
        println!("chat /tp {x} {} {z}", y + 2);
        return;
    }
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    if kind == "arch" || kind == "waterfall" {
        // Sites, not a ring search: known by their cells.
        let mut sites: Vec<(i32, i32, String)> = if kind == "arch" {
            l::landmarks::arches_near(&f, 0, 0, 6000)
                .into_iter()
                .map(|a| {
                    (
                        a.x,
                        a.z,
                        format!(
                            "{} arch, {} across, ring from {}, along {}",
                            if a.base < SEA { "sea" } else { "land" },
                            2 * a.outer + 1,
                            a.base,
                            if a.along_x { "x" } else { "z" }
                        ),
                    )
                })
                .collect()
        } else {
            l::landmarks::waterfalls_near(&f, 0, 0, 6000)
                .into_iter()
                .map(|w| (w.spring[0], w.spring[2], format!("spring at y {}, falling at {} {}", w.spring[1], w.front[0], w.front[1])))
                .collect()
        };
        sites.sort_by_key(|&(x, z, _)| x as i64 * x as i64 + z as i64 * z as i64);
        if sites.is_empty() {
            println!("no {kind} near spawn on seed {seed}");
        }
        for (x, z, what) in sites.into_iter().take(count) {
            let dist = ((x as f64).powi(2) + (z as f64).powi(2)).sqrt().round();
            println!("# {kind} at x {x} z {z}: {what}, ground {}, {dist} blocks from spawn", h(x, z));
            println!("chat /tp {x} {} {}\nidle\nlook 180 -30\nwait 2\nwhere\nshot {kind}_{x}_{z}.png", h(x, z).max(SEA) + 20, z - 30);
        }
        return;
    }
    if kind == "crater" || kind == "sinkhole" {
        // Sites, not a ring search: they are rare and known by their cells.
        let mut sites = if kind == "crater" { l::craters_near(&f, 0, 0, 16_000) } else { l::sinks_near(&f, 0, 0, 6_000) };
        sites.sort_by_key(|&(x, z, _)| x as i64 * x as i64 + z as i64 * z as i64);
        if sites.is_empty() {
            println!("no {kind} near spawn on seed {seed}");
        }
        for (x, z, r) in sites.into_iter().take(count) {
            let (rim, floor) = (
                (0..16)
                    .map(|k| {
                        let a = k as f64 / 16.0 * std::f64::consts::TAU;
                        h(x + (a.cos() * r as f64) as i32, z + (a.sin() * r as f64) as i32)
                    })
                    .max()
                    .unwrap(),
                h(x, z),
            );
            let dist = ((x as f64).powi(2) + (z as f64).powi(2)).sqrt().round();
            println!("# {kind} at x {x} z {z}, radius {r}, floor {floor}, rim up to {rim}, {dist} blocks from spawn");
            let (cz, cam) = (z - r * 2 - 10, rim + 18);
            println!("chat /tp {x} {cam} {cz}\nidle\nlook 180 -30\nwait 2\nwhere\nshot {kind}_{x}_{z}.png");
        }
        return;
    }
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
            "canyon" => l::canyons(&f, x * 16, z * 16) < -16 * 24,
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

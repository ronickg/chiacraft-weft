//! How walkable mountain flanks are: over a square of the world, each column's steps to its four
//! neighbours, by the mountains' strength there (`MOUNTAINS_AREA`), counting only land below
//! the snow line less 20 (about where the trees stop) and above the sea. A column with a step of 2 or more to
//! a neighbour has a face a player can't jump up and has to go round or build past.
//! Also the ground round one spot, for the playtester's climb.
//!
//!     cargo run -p base_api --example flank_steps -- <seed> [radius] [step] [x z]

use base_api::SEA;
use base_api::land as l;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let radius: i32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let step: i32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);
    let f = l::fields(seed);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    let bands = [-1.0, 0.0001, 0.1, 0.25, 0.45, 0.7, 1.01];
    // Per band: steps ≥1, steps ≥2, steps ≥3, columns.
    let mut n = vec![[0u64; 4]; bands.len() - 1];
    // The flanks (strength 0..0.45) where a river cuts 1 block or more: the valleys' ends.
    let mut fade = [0u64; 4];
    let mut z = -radius;
    while z <= radius {
        let mut x = -radius;
        while x <= radius {
            let m = l::MOUNTAINS_AREA.value(&f, x * 16, z * 16) as f64 / 4096.0;
            {
                let here = h(x, z);
                let t = f.temperature(x * 16, z * 16);
                if here > SEA + 1 && here < l::snow_line(t) - 20 {
                    let b = bands
                        .windows(2)
                        .position(|w| m >= w[0] && m < w[1])
                        .unwrap();
                    let valley = m > 0.0 && m < 0.45 && l::rivers(&f, x * 16, z * 16) <= -16;
                    for c in [Some(&mut n[b]), valley.then_some(&mut fade)]
                        .into_iter()
                        .flatten()
                    {
                        c[3] += 1;
                        let d = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                            .iter()
                            .map(|&(dx, dz)| (h(x + dx, z + dz) - here).abs())
                            .max()
                            .unwrap();
                        c[0] += (d >= 1) as u64;
                        c[1] += (d >= 2) as u64;
                        c[2] += (d >= 3) as u64;
                    }
                }
            }
            x += step;
        }
        z += step;
    }
    println!("| mountains' strength | columns | sloped | with a face of 2+ | of 3+ |\n|---|---|---|---|---|");
    for (b, c) in n.iter().enumerate() {
        let name =
            if b == 0 { "none (the land outside)".to_string() } else { format!("{:.2}..{:.2}", bands[b].max(0.0), bands[b + 1].min(1.0)) };
        let s = c[3].max(1) as f64;
        println!(
            "| {name} | {} | {:.0}% | {:.1}% | {:.1}% |",
            c[3],
            c[0] as f64 / s * 100.0,
            c[1] as f64 / s * 100.0,
            c[2] as f64 / s * 100.0
        );
    }
    let s = fade[3].max(1) as f64;
    println!(
        "| river valleys in the flanks (0..0.45, cut 1+) | {} | {:.0}% | {:.1}% | {:.1}% |",
        fade[3],
        fade[0] as f64 / s * 100.0,
        fade[1] as f64 / s * 100.0,
        fade[2] as f64 / s * 100.0
    );
    if let (Some(x), Some(z)) = (a.get(3).and_then(|s| s.parse::<i32>().ok()), a.get(4).and_then(|s| s.parse::<i32>().ok())) {
        println!("\nground round {x} {z} (mountains {:.2}), north up:", l::MOUNTAINS_AREA.value(&f, x * 16, z * 16) as f64 / 4096.0);
        for dz in -6..=6 {
            let row: Vec<String> = (-6..=6)
                .map(|dx| format!("{:4}", h(x + dx, z + dz)))
                .collect();
            println!("{}", row.join(""));
        }
        parts(seed, x, z);
        climb(seed, x, z, 200);
        straight(seed, x as f64 + 0.5, z as f64 + 0.5);
    }
}

/// The spot's line north to south, by part: the land before the rivers, the rivers' cut or rise,
/// the mountains' share, the hills'.
#[allow(dead_code)]
pub fn parts(seed: u64, x: i32, z: i32) {
    use weft_sdk::noise::fixed::{q4, shape};
    let f = l::fields(seed);
    println!("\nz: height = land + rivers; mountains, hills, highlands (blocks)");
    for dz in -8..=8 {
        let (xq, zq) = (x * 16, (z + dz) * 16);
        let land = l::land_height(&f, xq, zq);
        let all = l::height(&f, xq, zq);
        let m = (l::MOUNTAINS_RISE.value(&f, xq, zq) * shape::mountains(&f, xq, zq, q4(l::MOUNTAINS.0), q4(l::MOUNTAINS.1))) >> 12;
        let hills = (l::HILLS_AREA.value(&f, xq, zq) * shape::hills(&f, xq, zq, q4(l::HILLS.0 * l::COAST_CALM), q4(l::HILLS.1))) >> 12;
        println!(
            "{:5}: {:6.2} = {:6.2} + {:5.2}; mountains {:5.2} hills {:5.2}",
            z + dz,
            all as f64 / 16.0,
            land as f64 / 16.0,
            (all - land) as f64 / 16.0,
            m as f64 / 16.0,
            hills as f64 / 16.0
        );
    }
}

/// How high a player gets walking from (`x`, `z`), never climbing more than one block a step
/// (any drop is fine), within `reach` blocks: the highest ground reached, and how far up the
/// mountains (their strength there), as a walk over the ground alone (no trees, no water).
#[allow(dead_code)]
pub fn climb(seed: u64, x: i32, z: i32, reach: i32) {
    let f = l::fields(seed);
    let side = (2 * reach + 1) as usize;
    let at = |dx: i32, dz: i32| ((dz + reach) as usize) * side + (dx + reach) as usize;
    let hs: Vec<i32> = (-reach..=reach)
        .flat_map(|dz| (-reach..=reach).map(move |dx| (dx, dz)))
        .map(|(dx, dz)| l::height(&f, (x + dx) * 16, (z + dz) * 16) >> 4)
        .collect();
    let mut seen = vec![false; side * side];
    let mut todo = vec![(0, 0)];
    seen[at(0, 0)] = true;
    let mut best = (hs[at(0, 0)], 0, 0);
    while let Some((dx, dz)) = todo.pop() {
        let here = hs[at(dx, dz)];
        if here > best.0 {
            best = (here, dx, dz);
        }
        for (ex, ez) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (dx + ex, dz + ez);
            if nx.abs() > reach || nz.abs() > reach || seen[at(nx, nz)] || hs[at(nx, nz)] > here + 1 || hs[at(nx, nz)] < SEA {
                continue;
            }
            seen[at(nx, nz)] = true;
            todo.push((nx, nz));
        }
    }
    let top = *hs.iter().max().unwrap();
    let (bx, bz) = (x + best.1, z + best.2);
    println!(
        "\nwalking from {x} {z} (ground {}): up to {} at {bx} {bz} (mountains {:.2}), {} blocks away; the highest ground within {reach} is {top}; {:.0}% of the square reached",
        hs[at(0, 0)],
        best.0,
        l::MOUNTAINS_AREA.value(&f, bx * 16, bz * 16) as f64 / 4096.0,
        ((best.1 * best.1 + best.2 * best.2) as f64).sqrt() as i32,
        seen.iter().filter(|&&s| s).count() as f64 / seen.len() as f64 * 100.0
    );
}

/// Walking straight from (`x`, `z`) on every heading (yaw, 0 north to -z, clockwise as the game
/// has it, every 5°), how far before a face of 2 or more stops the walk (250 at most), and the
/// height reached; the yaw-141 line (the playtester's climb from the spawn) on its own.
#[allow(dead_code)]
pub fn straight(seed: u64, x: f64, z: f64) {
    let f = l::fields(seed);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    let walk = |yaw: f64| {
        let (dx, dz) = (-yaw.to_radians().sin(), -yaw.to_radians().cos());
        let (mut cx, mut cz) = (x.floor() as i32, z.floor() as i32);
        let start = h(cx, cz);
        let mut here = start;
        let mut t = 0.0;
        while t < 250.0 {
            t += 0.25;
            let (nx, nz) = ((x + dx * t).floor() as i32, (z + dz * t).floor() as i32);
            if (nx, nz) == (cx, cz) {
                continue;
            }
            let next = h(nx, nz);
            if next >= here + 2 {
                return (t, here - start, nx, nz);
            }
            (cx, cz, here) = (nx, nz, next);
        }
        (t, here - start, cx, cz)
    };
    let (t, up, sx, sz) = walk(141.0);
    println!("\nstraight at yaw 141 from {x} {z}: {t:.0} blocks, {up} up, stopped at {sx} {sz}");
    let mut all: Vec<(f64, i32)> = (0..72)
        .map(|i| walk(i as f64 * 5.0))
        .map(|w| (w.0, w.1))
        .collect();
    all.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!(
        "every 5°: median {:.0} blocks; {} of 72 go 250; the uphill ones (20+ up): {}",
        all[36].0,
        all.iter().filter(|w| w.0 >= 250.0).count(),
        all.iter()
            .filter(|w| w.1 >= 20)
            .map(|w| format!("{:.0}/{}", w.0, w.1))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

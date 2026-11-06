//! A place to walk along one of the long rivers (`base_api::land`'s `line_rivers`): the nearest
//! river water to (`x`, `z`) that is a wide channel and not a lake, the way the channel runs there,
//! and a dry spot on its bank a few blocks from the water, as testkit lines: stand on the bank
//! facing downstream along it, and the yaw to face across it.
//!
//!     cargo run -p base_api --example river_walk -- <seed> <x> <z>

use base_api::SEA;
use base_api::land as l;

fn main() {
    let a: Vec<i64> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let (cx, cz) = (a.get(1).copied().unwrap_or(0) as i32, a.get(2).copied().unwrap_or(0) as i32);
    let f = l::fields(seed);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    let wet = |x: i32, z: i32| h(x, z) < SEA;
    let run = |x: i32, z: i32, dx: i32, dz: i32| (1..120).take_while(|&k| wet(x + dx * k, z + dz * k)).count() as i32;
    for r in (0..3000).step_by(8) {
        for k in 0..(r / 8).max(1) * 6 {
            let t = k as f64 / ((r / 8).max(1) * 6) as f64 * std::f64::consts::TAU;
            let (x, z) = (cx + (r as f64 * t.cos()) as i32, cz + (r as f64 * t.sin()) as i32);
            if !wet(x, z) || l::lakes(&f, x * 16, z * 16) < 0 {
                continue;
            }
            let Some(b) = l::bank(&f, x * 16, z * 16) else { continue };
            if b.water != l::Water::River || !b.in_water {
                continue;
            }
            // The channel's way: of 8 directions, the one the water runs furthest along.
            const DIRS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
            let (along, len) = DIRS
                .iter()
                .map(|&(dx, dz)| ((dx, dz), run(x, z, dx, dz) + run(x, z, -dx, -dz)))
                .max_by_key(|e| e.1)
                .unwrap();
            let across = (-along.1, along.0);
            let width = 1 + run(x, z, across.0, across.1) + run(x, z, -across.0, -across.1);
            if len < 60 || !(12..=60).contains(&width) {
                continue;
            }
            // The bank: out across from the water's edge, 4 blocks onto dry land.
            let edge = run(x, z, across.0, across.1);
            let (bx, bz) = (x + across.0 * (edge + 5), z + across.1 * (edge + 5));
            let ground = h(bx, bz);
            // Ours: yaw 0 faces -z, -90 faces +x.
            let yaw = |dx: i32, dz: i32| (-(dx as f64)).atan2(-(dz as f64)).to_degrees().round();
            println!("# river at {x} {z}: {width} blocks across, water runs {len} along ({}, {})", along.0, along.1);
            println!("chat /tp {bx} {} {bz}", ground + 1);
            println!("look {} -15   # along the bank", yaw(along.0, along.1));
            println!("look {} -20   # across the river", yaw(-across.0, -across.1));
            return;
        }
    }
    println!("no long river within 3 km of {cx} {cz}");
}

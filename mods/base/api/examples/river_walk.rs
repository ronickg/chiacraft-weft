//! Stations along one of the long rivers (`base_api::land`'s `line_rivers`): the nearest river water
//! to (`x`, `z`) that is a wide channel and not a lake, the way the channel runs there, and camera
//! stations at eye height (spectator: a walking player stuck in the bank's trees, river_walk1): on
//! each bank at the water's edge, looking along and across, and in the middle at the water's
//! surface, as testkit lines.
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
            // The water's edges either side, and the ground just past each.
            let (near, far) = (run(x, z, across.0, across.1), run(x, z, -across.0, -across.1));
            let (ax, az) = (x + across.0 * (near + 1), z + across.1 * (near + 1));
            let (fx, fz) = (x - across.0 * (far + 1), z - across.1 * (far + 1));
            // Ours: yaw 0 faces -z, -90 faces +x.
            let yaw = |dx: i32, dz: i32| (-(dx as f64)).atan2(-(dz as f64)).to_degrees().round();
            let eye = |gx: i32, gz: i32| h(gx, gz).max(SEA) as f64 + 1.6;
            println!("# river at {x} {z}: {width} blocks across, water runs {len} along ({}, {})", along.0, along.1);
            println!("near_bank: /tp {ax} {:.1} {az}  along {}  across {}", eye(ax, az), yaw(along.0, along.1), yaw(-across.0, -across.1));
            println!("middle:    /tp {x} {:.1} {z}  along {}", SEA as f64 + 0.6, yaw(along.0, along.1));
            println!("far_bank:  /tp {fx} {:.1} {fz}  along {}  across {}", eye(fx, fz), yaw(-along.0, -along.1), yaw(across.0, across.1));
            return;
        }
    }
    println!("no long river within 3 km of {cx} {cz}");
}

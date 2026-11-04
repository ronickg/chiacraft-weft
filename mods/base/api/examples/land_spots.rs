//! Finds spots that show the land's shapes (`base_api::land`) near the world's middle and writes
//! a testkit script that photographs each from a fixed eye, the same for the land before and
//! after: the eye stands above both versions' ground all the way to what it looks at.
//!
//!     cargo run -p base_api --example land_spots -- <seed> <radius> > terrain-shots.script

use base_api::SEA;
use base_api::land::{self as l};
use weft_sdk::noise::fixed::{Fields, q4, q12, shape};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let radius: i32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1500);
    let f = l::fields(seed);
    let after = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    let before = |x: i32, z: i32| old_height(&f, x * 16, z * 16) >> 4;
    let q = |x: i32, z: i32| (f.temperature(x * 16, z * 16), f.humidity(x * 16, z * 16), f.continent(x * 16, z * 16));
    let mountain = |x: i32, z: i32| l::MOUNTAINS_AREA.value(&f, x * 16, z * 16) > q12(l::MOUNTAIN_BIOME);
    // The best spot of each kind: (score, x, z), nearest the middle among equals.
    let mut best: [(i64, i32, i32); 10] = [(i64::MIN, 0, 0); 10];
    let step = 6;
    for z in (-radius..=radius).step_by(step) {
        for x in (-radius..=radius).step_by(step) {
            let (h, h0) = (after(x, z), before(x, z));
            let (t, hu, cont) = q(x, z);
            let near = -((x.abs() + z.abs()) as i64) / 64;
            let fall = [(4, 0), (-4, 0), (0, 4), (0, -4)]
                .iter()
                .map(|&(dx, dz)| h - after(x + dx, z + dz))
                .max()
                .unwrap();
            let mut put = |k: usize, score: i64| {
                if score > best[k].0 {
                    best[k] = (score, x, z);
                }
            };
            let temperate = t > q12(l::TAIGA_BELOW) && t < q12(l::SAVANNA.0) && !mountain(x, z);
            // 0: a river in a valley, inland, in the temperate woods and plains.
            if h < SEA && (SEA + 10..=SEA + 30).contains(&h0) && cont > q12(0.2) && temperate {
                put(0, (h0 - SEA) as i64 * 4 + near);
            }
            // 1: a river's mouth at the coast.
            if h < SEA && (SEA + 3..=SEA + 20).contains(&h0) && (q12(0.0)..q12(0.1)).contains(&cont) && t > q12(-0.3) && !mountain(x, z) {
                put(1, (h0 - h) as i64 * 4 + near);
            }
            // 2: a mesa's high tier in the desert.
            let m = l::mesa(&f, x * 16, z * 16) >> 4;
            if m >= 20 && l::overworld_biome(&f, x * 16, z * 16) == "desert" {
                put(2, m as i64 * 8 + fall as i64 * 4 + near);
            }
            // 3: a snow cap on a high peak.
            if mountain(x, z) && l::snowy(&f, x * 16, z * 16, h) {
                put(3, h as i64 * 4 + near);
            }
            // 4: a steep rock face under the snow line.
            if fall >= 12 && mountain(x, z) && h == h0 && !l::snowy(&f, x * 16, z * 16, h) && h > SEA + 10 {
                put(4, fall as i64 * 8 + near);
            }
            // 6, 7: a slate cliff in the cold, a limestone cliff by low land.
            if fall >= 8 && h > SEA + 4 {
                match l::rock_set(&f, x * 16, z * 16, h, false) {
                    l::RockSet::Slate => put(6, fall as i64 * 8 + near),
                    l::RockSet::Limestone => put(7, fall as i64 * 8 + near),
                    _ => {}
                }
            }
            // 8, 9: a lake, and an island in one (water a few blocks around it).
            if l::lakes(&f, x * 16, z * 16) < 0 {
                if h < SEA - 2 {
                    put(8, (SEA - h) as i64 * 4 + near);
                } else if h > SEA && ((after(x + 6, z) < SEA && after(x - 6, z) < SEA) || (after(x, z + 6) < SEA && after(x, z - 6) < SEA))
                {
                    put(9, h as i64 + near);
                }
            }
            // 5: highlands: rolling forest well over the sea, away from the mountains.
            if temperate && hu >= q12(l::FOREST_HUMIDITY) && h >= SEA + 24 && h - h0 >= 8 && cont > q12(0.2) {
                put(5, (h - h0) as i64 * 4 + h as i64 + near);
            }
        }
    }
    let names = ["river", "estuary", "mesa", "snow", "rock_face", "highlands", "slate_face", "limestone_face", "lake", "lake_island"];
    // How far back the eye stands and how high over the ground, per kind.
    let (back, up) = ([48, 70, 75, 110, 40, 60, 36, 36, 70, 40], [26, 30, 18, 25, 12, 30, 25, 25, 30, 25]);
    println!("# Terrain shapes before/after (reports/world-looks.md change 5), seed {seed}.");
    println!("# Run once on dev before claude/terrain-shape and once on the branch, each in a fresh");
    println!("# world with `--seed {seed}`. Each spot is a fixed eye looking at a fixed point.");
    println!("loaded");
    println!("chat /gamemode spectator");
    println!("chat /weather clear");
    println!("chat /time set 6000");
    for (k, &(score, x, z)) in best.iter().enumerate() {
        if score == i64::MIN {
            println!("# no {} spot within {radius} blocks", names[k]);
            continue;
        }
        // The eye: back from the spot to the south-west, over both grounds on the way.
        let (ex, ez) = (x - back[k] * 7 / 10, z - back[k] * 7 / 10);
        let mut top = i32::MIN;
        for i in 0..=20 {
            let (px, pz) = (ex + (x - ex) * i / 20, ez + (z - ez) * i / 20);
            top = top.max(after(px, pz)).max(before(px, pz));
        }
        let ey = top + up[k];
        let ty = after(x, z).max(SEA);
        println!("# --- {}: x {x} z {z}, ground {} before, {} after", names[k], before(x, z), after(x, z));
        println!("chat /tp {ex} {ey} {ez}");
        println!("loaded");
        println!("idle");
        println!("via {ex}.5 {ey}.5 {ez}.5 @ {x}.5 {ty} {z}.5");
        println!("travel 0.5");
        println!("wait 3");
        println!("shot terrain_{}.png", names[k]);
    }
    // One view from high over the middle of the spots, looking down on rivers and ranges.
    println!("# --- overview from high up");
    println!("chat /tp 0 260 -200");
    println!("loaded");
    println!("idle");
    println!("via 0.5 260.5 -199.5 @ 0.5 {SEA} 200.5");
    println!("travel 0.5");
    println!("wait 3");
    println!("shot terrain_overview.png");
    println!("quit");
}

/// The land as it was before `base_api::land` (as in `land_map`).
fn old_height(f: &Fields, x: i32, z: i32) -> i32 {
    let m = |a: &l::Area, h: i32| {
        let w = a.value(f, x, z);
        if w > 0 { (w * h) >> 12 } else { 0 }
    };
    q4(SEA as f64)
        + shape::continents(f, x, z, q4(l::CONTINENTS))
        + m(&l::HILLS_AREA, shape::hills(f, x, z, q4(l::HILLS.0), q4(l::HILLS.1)))
        + m(&l::DUNES_AREA, shape::dunes(f, x, z, q4(l::DUNES.0), q4(l::DUNES.1)))
        + m(&l::MOUNTAINS_AREA, shape::mountains(f, x, z, q4(l::MOUNTAINS.0), q4(l::MOUNTAINS.1)))
}

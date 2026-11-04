//! How steep the banks of lakes and rivers are: over a grid, every column on a bank
//! (`land::bank`, not in the water) by its steepest fall to a neighbour a block away, as shares,
//! and the share of the water's edge that is flush (a dry column at the water's level beside it).
//!
//!     cargo run -p base_api --example land_banks -- <seed> <radius> <step>

use base_api::SEA;
use base_api::land::{self as l, Water};

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(2048) as i32;
    let step = a.get(2).copied().unwrap_or(4) as usize;
    let f = l::fields(seed);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    for water in [Water::Lake, Water::River] {
        let mut falls = [0usize; 5];
        let (mut edge, mut flush) = (0usize, 0usize);
        for z in (-radius..=radius).step_by(step) {
            for x in (-radius..=radius).step_by(step) {
                let Some(b) = l::bank(&f, x * 16, z * 16) else { continue };
                if b.water != water || b.in_water {
                    continue;
                }
                let top = h(x, z);
                if top < SEA {
                    continue;
                }
                let around = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| h(x + dx, z + dz));
                let fall = around.iter().map(|n| (top - n).abs()).max().unwrap_or(0);
                falls[(fall as usize).min(4)] += 1;
                if around.iter().any(|&n| n < SEA) {
                    edge += 1;
                    if top == SEA {
                        flush += 1;
                    }
                }
            }
        }
        let all: usize = falls.iter().sum();
        let pc = |n: usize| n as f64 * 100.0 / all.max(1) as f64;
        println!(
            "{water:?} banks ({all} columns): fall 0 {:.0}%, 1 {:.0}%, 2 {:.0}%, 3 {:.0}%, 4+ {:.0}%;  water's edge flush {:.0}% of {edge}",
            pc(falls[0]),
            pc(falls[1]),
            pc(falls[2]),
            pc(falls[3]),
            pc(falls[4]),
            flush as f64 * 100.0 / edge.max(1) as f64
        );
    }
}

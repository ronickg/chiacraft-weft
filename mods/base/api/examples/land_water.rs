//! Where the overworld's water is: sea, rivers (carved), lakes (inland water at the sea's level
//! the rivers didn't make), and the land's height spread, over a grid.
//!
//!     cargo run -p base_api --example land_water -- <seed> <radius> <step>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::{Fields, q12};

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(8000) as i32;
    let step = a.get(2).copied().unwrap_or(16) as usize;
    let f = Fields { seed };
    let (mut sea, mut river, mut lake, mut land, mut all) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut heights = Vec::new();
    for z in (-radius..=radius).step_by(step) {
        for x in (-radius..=radius).step_by(step) {
            let (qx, qz) = (x * 16, z * 16);
            all += 1;
            let h = l::height(&f, qx, qz) >> 4;
            if h >= SEA {
                land += 1;
                heights.push(h);
                continue;
            }
            if f.continent(qx, qz) < q12(l::OCEAN_CONTINENT) {
                sea += 1;
            } else if l::rivers(&f, qx, qz) < 0 && l::land_height(&f, qx, qz) >> 4 >= SEA {
                river += 1;
            } else {
                lake += 1;
            }
        }
    }
    heights.sort();
    let pc = |n: usize| n as f64 * 100.0 / all as f64;
    println!(
        "seed {seed}, {radius} out every {step}: land {:.1}%, sea {:.1}%, rivers {:.2}%, lakes at the sea's level {:.2}%",
        pc(land),
        pc(sea),
        pc(river),
        pc(lake)
    );
    let q = |p: usize| heights[heights.len() * p / 100];
    println!("land heights: 10% {}  50% {}  90% {}  99% {}  max {}", q(10), q(50), q(90), q(99), heights[heights.len() - 1]);
}

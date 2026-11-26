//! Where the overworld's water is: sea, rivers (carved), lakes (inland water at the sea's level
//! the rivers didn't make), and the land's height spread, over a grid. A river's width is
//! measured at each river sample: the water's run across it, the narrower of east-west and
//! north-south (a river crossing the grid at a slant reads wider; the narrower run is near its
//! true width).
//!
//!     cargo run -p base_api --example land_water -- <seed> <radius> <step>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::q12;

fn main() {
    let a: Vec<i64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let seed = a.first().copied().unwrap_or(1) as u64;
    let radius = a.get(1).copied().unwrap_or(8000) as i32;
    let step = a.get(2).copied().unwrap_or(16) as usize;
    let f = l::fields(seed);
    let (mut sea, mut river, mut lake, mut land, mut all) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut heights = Vec::new();
    let mut widths = Vec::new();
    let wet = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4 < SEA;
    // The water's run through (`x`, `z`) along (`dx`, `dz`), up to 200 blocks.
    let run = |x: i32, z: i32, dx: i32, dz: i32| {
        let side = |s: i32| {
            (1..100)
                .take_while(|&k| wet(x + s * dx * k, z + s * dz * k))
                .count() as i32
        };
        1 + side(1) + side(-1)
    };
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
                widths.push(run(x, z, 1, 0).min(run(x, z, 0, 1)));
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
    widths.sort();
    if !widths.is_empty() {
        let w = |p: usize| widths[(widths.len() * p / 100).min(widths.len() - 1)];
        println!("river widths (blocks): 10% {}  50% {}  90% {}  max {}", w(10), w(50), w(90), widths[widths.len() - 1]);
    }
    let q = |p: usize| heights[heights.len() * p / 100];
    println!("land heights: 10% {}  50% {}  90% {}  99% {}  max {}", q(10), q(50), q(90), q(99), heights[heights.len() - 1]);
}

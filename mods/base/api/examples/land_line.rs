//! The ground along a line, as the engine should build it: for every `step` blocks from
//! (x0, z0) to (x1, z1), the land before the rivers, the rivers' cut and the ground (blocks), and
//! whether it's under the sea's level (water). For checking a game shot against the numbers.
//!
//!     cargo run -p base_api --example land_line -- <seed> <x0> <z0> <x1> <z1> <step>

use base_api::SEA;
use base_api::land as l;

fn main() {
    let a: Vec<i32> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let f = l::fields(a.first().copied().unwrap_or(1) as u64);
    let (x0, z0, x1, z1) = (a[1], a[2], a[3], a[4]);
    let step = a.get(5).copied().unwrap_or(2).max(1);
    let n = ((x1 - x0).abs().max((z1 - z0).abs()) / step).max(1);
    for i in 0..=n {
        let (x, z) = (x0 + (x1 - x0) * i / n, z0 + (z1 - z0) * i / n);
        let land = l::land_height(&f, x * 16, z * 16) >> 4;
        let cut = l::rivers(&f, x * 16, z * 16) >> 4;
        let h = l::height(&f, x * 16, z * 16) >> 4;
        let water = if h < SEA { format!("water {}", SEA - h) } else { String::new() };
        println!("x {x:6} z {z:6}  land {land:4}  rivers {cut:4}  ground {h:4}  {water}");
    }
}

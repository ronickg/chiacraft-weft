//! Trees by space colonization (`base_api::colonize`) in `tree_widths`' format, with the time each
//! took (stderr), for the algorithm study:
//!
//!     cargo run --release -p base_api --example tree_colonize -- [count] > colonize.json

use base_api::branchy::Part;
use base_api::colonize::{ALL, blocks};

fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let mut out = Vec::new();
    for c in ALL {
        let mut trees = Vec::new();
        let t0 = std::time::Instant::now();
        let mut n = 0;
        for seed in 0..count {
            let b = blocks(c, seed);
            n += b.len();
            let s: Vec<String> = b
                .iter()
                .map(|&(x, y, z, p, w)| match p {
                    Part::Leaves => format!("[{x},{y},{z},0,0]"),
                    Part::Log { axis } => format!("[{x},{y},{z},{},{w:.3}]", 1 + axis),
                })
                .collect();
            trees.push(format!("[{}]", s.join(",")));
        }
        eprintln!("{}: {:.2} ms a tree, {} blocks a tree", c.log, t0.elapsed().as_secs_f64() * 1000.0 / count as f64, n / count as usize);
        out.push(format!("\"{}\":[{}]", c.log, trees.join(",")));
    }
    // branchy's time for the same species, to compare
    for log in ["base:oak_log", "base:spruce_log", "base:birch_log"] {
        let r = base_api::tree_recipes::of(log).unwrap();
        let t0 = std::time::Instant::now();
        let mut n = 0;
        for seed in 0..count.max(50) {
            n += base_api::branchy::blocks(r, seed).len();
        }
        eprintln!(
            "branchy {log}: {:.3} ms a tree, {} blocks a tree",
            t0.elapsed().as_secs_f64() * 1000.0 / count.max(50) as f64,
            n / count.max(50) as usize
        );
    }
    println!("{{{}}}", out.join(","));
}

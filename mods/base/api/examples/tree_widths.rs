//! Prints trees grown from branches with each wood block's width (the widest limb through it),
//! for the thin-branch-block study (`mods/base/source/trees/branches.py`):
//! `{"base:oak_log": [[[x, y, z, part, width], ...], ...]}` (part 0 leaves; 1, 2, 3 wood along
//! x, y, z; width 0 for leaves), `count` seeds a species from seed 0.
//!
//!     cargo run -p base_api --example tree_widths -- [count] [log ...] > widths.json

use base_api::branchy::{Part, blocks, wood_widths};
use base_api::tree_recipes::RECIPES;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let count: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(3);
    let only: Vec<&String> = a.iter().skip(1).collect();
    let mut out = Vec::new();
    for r in RECIPES
        .iter()
        .filter(|r| only.is_empty() || only.iter().any(|o| r.log.contains(o.as_str())))
    {
        let mut trees = Vec::new();
        for seed in 0..count {
            let w: std::collections::HashMap<_, _> = wood_widths(r, seed)
                .into_iter()
                .map(|(x, y, z, w)| ((x, y, z), w))
                .collect();
            let b: Vec<String> = blocks(r, seed)
                .iter()
                .map(|&(x, y, z, p)| match p {
                    Part::Leaves => format!("[{x},{y},{z},0,0]"),
                    Part::Log { axis } => format!("[{x},{y},{z},{},{:.3}]", 1 + axis, w.get(&(x, y, z)).copied().unwrap_or(1.0)),
                })
                .collect();
            trees.push(format!("[{}]", b.join(",")));
        }
        out.push(format!("\"{}\":[{}]", r.log, trees.join(",")));
    }
    println!("{{{}}}", out.join(","));
}

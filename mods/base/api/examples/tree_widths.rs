//! Prints grown trees with each wood block's width (the widest limb through it), for the
//! branch-block sheets (`mods/base/source/trees/branches.py`): a colonized species' tree
//! (`colonize`) as worldgen grows it, the rest `branchy`'s.
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
            let grown: Vec<(i32, i32, i32, Part, f32)> = match base_api::tree_recipes::colonized(r.log) {
                Some(c) => base_api::colonize::blocks(c, seed),
                None => {
                    let w: std::collections::HashMap<_, _> = wood_widths(r, seed)
                        .into_iter()
                        .map(|(x, y, z, w)| ((x, y, z), w))
                        .collect();
                    blocks(r, seed)
                        .into_iter()
                        .map(|(x, y, z, p)| (x, y, z, p, w.get(&(x, y, z)).copied().unwrap_or(1.0)))
                        .collect()
                }
            };
            let b: Vec<String> = grown
                .iter()
                .map(|&(x, y, z, p, w)| match p {
                    Part::Leaves => format!("[{x},{y},{z},0,0]"),
                    Part::Log { axis } => format!("[{x},{y},{z},{},{w:.3}]", 1 + axis),
                })
                .collect();
            trees.push(format!("[{}]", b.join(",")));
        }
        out.push(format!("\"{}\":[{}]", r.log, trees.join(",")));
    }
    println!("{{{}}}", out.join(","));
}

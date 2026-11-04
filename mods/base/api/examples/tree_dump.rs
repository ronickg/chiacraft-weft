//! Prints trees grown from branches (`base_api::branchy`, `tree_recipes`) as JSON, for the
//! blocky-model skill's renderer (`mods/base/source/trees/sheet.py`):
//! `{"base:oak_log": [[[x, y, z, part], ...], ...], ...}`, `count` trees a species from seed 0
//! (part 0 leaves; 1, 2, 3 wood along x, y, z).
//!
//!     cargo run -p base_api --example tree_dump -- [count] > trees.json

use base_api::branchy::{Part, blocks};
use base_api::tree_recipes::RECIPES;

fn main() {
    let count: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let mut out = String::from("{");
    for (i, r) in RECIPES.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("\"{}\":[", r.log));
        for seed in 0..count {
            if seed > 0 {
                out.push(',');
            }
            let b: Vec<String> = blocks(r, seed)
                .iter()
                .map(|&(x, y, z, p)| {
                    let k = match p {
                        Part::Leaves => 0,
                        Part::Log { axis } => 1 + axis,
                    };
                    format!("[{x},{y},{z},{k}]")
                })
                .collect();
            out.push_str(&format!("[{}]", b.join(",")));
        }
        out.push(']');
    }
    out.push('}');
    println!("{out}");
}

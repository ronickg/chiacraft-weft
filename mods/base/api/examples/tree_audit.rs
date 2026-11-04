//! Prints every tree grown from branches (`base_api::branchy`): each species in `RECIPES`, and each
//! young recipe in `YOUNG` (keyed `young:<log>`) and each species' understory young
//! (`tree_recipes::understory`, keyed `understory:<log>`), `count` seeds from `first`, as JSON for the
//! tree-look audit sheets (`mods/base/source/trees/audit.py`):
//! `{"base:oak_log": [[[x, y, z, part], ...], ...], ...}` (part 0 leaves; 1, 2, 3 wood along x, y, z).
//!
//!     cargo run -p base_api --example tree_audit -- [count] [first] > trees.json

use base_api::branchy::{Part, Recipe, blocks};
use base_api::tree_recipes::{RECIPES, YOUNG, understory};

fn main() {
    let mut args = std::env::args().skip(1);
    let count: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);
    let first: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let under: Vec<Recipe> = RECIPES.iter().map(understory).collect();
    let all: Vec<(String, &Recipe)> = RECIPES
        .iter()
        .map(|r| (r.log.to_string(), r))
        .chain(YOUNG.iter().map(|r| (format!("young:{}", r.log), r)))
        .chain(under.iter().map(|r| (format!("understory:{}", r.log), r)))
        .collect();
    let mut out = String::from("{");
    for (i, (key, r)) in all.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("\"{key}\":["));
        for seed in first..first + count {
            if seed > first {
                out.push(',');
            }
            let grown =
                if key.starts_with("young:") || key.starts_with("understory:") { None } else { base_api::tree_recipes::grown(key, seed) };
            let b: Vec<String> = grown
                .unwrap_or_else(|| blocks(r, seed))
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

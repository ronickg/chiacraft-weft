//! Every species' grown tree in each form (`tree_recipes::Variant`: standing, big, crooked, twin),
//! `count` seeds each, in `tree_audit`'s format keyed `<log>|<form>`, for the variety sheet and
//! its checks.
//!
//!     cargo run -p base_api --example tree_forms -- [count] > forms.json

use base_api::branchy::{Part, blocks};
use base_api::tree_recipes::{RECIPES, Variant, colonized, varied, varied_colonized};

fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let mut out = Vec::new();
    for r in RECIPES {
        for v in Variant::ALL {
            let mut trees = Vec::new();
            for seed in 0..count {
                let b: Vec<(i32, i32, i32, Part)> = match colonized(r.log) {
                    Some(c) => base_api::colonize::blocks(&varied_colonized(c, v), seed)
                        .into_iter()
                        .map(|(x, y, z, p, _)| (x, y, z, p))
                        .collect(),
                    None => blocks(&varied(r, v), seed),
                };
                let s: Vec<String> = b
                    .iter()
                    .map(|&(x, y, z, p)| {
                        let k = match p {
                            Part::Leaves => 0,
                            Part::Log { axis } => 1 + axis,
                        };
                        format!("[{x},{y},{z},{k}]")
                    })
                    .collect();
                trees.push(format!("[{}]", s.join(",")));
            }
            out.push(format!("\"{}|{:?}\":[{}]", r.log, v, trees.join(",")));
        }
    }
    println!("{{{}}}", out.join(","));
}

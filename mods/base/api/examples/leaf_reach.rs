//! How far each species' leaves lie from their tree's nearest log (Chebyshev, the box a leaf
//! looks in: `mods/base/src/leaf_decay.rs`), over many grown trees: what leaf decay's reach must
//! cover so no living tree drops its own crown. `cargo run -p base_api --example leaf_reach`.

use std::collections::HashSet;

use base_api::branchy::{self, Part};
use base_api::tree_recipes::{RECIPES, SEQUOIA, YOUNG};

fn main() {
    println!("| tree | leaves | 99% within | farthest |");
    println!("|---|---|---|---|");
    for r in RECIPES.iter().chain([&SEQUOIA]).chain(YOUNG.iter()) {
        let mut far = Vec::new();
        for seed in 0..40u64 {
            let (mut logs, mut leaves) = (HashSet::new(), Vec::new());
            branchy::grow(r, 0, 0, 0, seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), |x, y, z, part| match part {
                Part::Log { .. } => {
                    logs.insert([x, y, z]);
                }
                Part::Leaves => leaves.push([x, y, z]),
            });
            for l in leaves {
                let d = (0..=12)
                    .find(|&d| (-d..=d).any(|dy| (-d..=d).any(|dz| (-d..=d).any(|dx| logs.contains(&[l[0] + dx, l[1] + dy, l[2] + dz])))))
                    .unwrap_or(13);
                far.push(d);
            }
        }
        far.sort_unstable();
        let p99 = far.get(far.len() * 99 / 100).copied().unwrap_or(0);
        println!("| {} | {} | {} | {} |", r.log, far.len(), p99, far.last().copied().unwrap_or(0));
    }
}

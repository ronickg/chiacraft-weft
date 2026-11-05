//! How much wood each species grows as felling counts it (a log one, a broadleaf tree's branch
//! block a quarter: `tree_recipes::felling_wood`), and how far across from its foot its wood
//! reaches, over many grown trees: what felling's chop time and axe tiers are sized on
//! (`vitals::felling`, `mods/base/src/felling.rs`). `cargo run -p base_api --example tree_logs`.

use base_api::branchy::{self, Part};
use base_api::tree_recipes::{self, RECIPES, SEQUOIA, YOUNG};

fn main() {
    println!("| tree | logs: least | median | most | reach across |");
    println!("|---|---|---|---|---|");
    for r in RECIPES.iter().chain([&SEQUOIA]).chain(YOUNG.iter()) {
        let young = YOUNG.iter().any(|y| std::ptr::eq(y, r));
        let (mut counts, mut reach) = (Vec::new(), 0);
        for seed in 0..40u64 {
            let s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            // A colonized species' grown tree; its young row (and the giants' young) `branchy`'s.
            let b = if young { None } else { tree_recipes::grown_widths(r.log, s) }.unwrap_or_else(|| {
                branchy::blocks(r, s)
                    .into_iter()
                    .map(|(x, y, z, p)| (x, y, z, p, 1.0))
                    .collect()
            });
            for &(x, _, z, part, _) in &b {
                if let Part::Log { .. } = part {
                    reach = reach.max(x.abs()).max(z.abs());
                }
            }
            counts.push(if young {
                b.iter().filter(|b| matches!(b.3, Part::Log { .. })).count()
            } else {
                tree_recipes::felling_wood(r.log, &b)
            });
        }
        counts.sort_unstable();
        println!("| {} | {} | {} | {} | {} |", r.log, counts[0], counts[counts.len() / 2], counts[counts.len() - 1], reach);
    }
}

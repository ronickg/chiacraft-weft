//! How many logs each species grows, and how far across from its foot they reach, over many grown
//! trees: what felling's chop time and axe tiers are sized on (`vitals::felling`,
//! `mods/base/src/felling.rs`). `cargo run -p base_api --example tree_logs`.

use base_api::branchy::{self, Part};
use base_api::tree_recipes::{RECIPES, SEQUOIA, YOUNG};

fn main() {
    println!("| tree | logs: least | median | most | reach across |");
    println!("|---|---|---|---|---|");
    for r in RECIPES.iter().chain([&SEQUOIA]).chain(YOUNG.iter()) {
        let (mut counts, mut reach) = (Vec::new(), 0);
        for seed in 0..40u64 {
            let mut n = 0;
            branchy::grow(r, 0, 0, 0, seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), |x, _, z, part| {
                if let Part::Log { .. } = part {
                    n += 1;
                    reach = reach.max(x.abs()).max(z.abs());
                }
            });
            counts.push(n);
        }
        counts.sort_unstable();
        println!("| {} | {} | {} | {} | {} |", r.log, counts[0], counts[counts.len() / 2], counts[counts.len() - 1], reach);
    }
}

//! Wood showing on top of a crown: per species, form and stand layer (`vitals::canopy`), how many
//! of `count` grown trees have a wood block with no leaves over it in the top third of the tree,
//! how wide their crowns are and how much trunk stands clear under them; and the simple style's
//! heights and clear trunks per layer.
//!
//!     cargo run -p base_api --example tree_tops -- [count]

use base_api::branchy::Part;
use base_api::tree_recipes::{RECIPES, Variant, colonized, tiered, tiered_colonized, varied, varied_colonized};
use base_api::trees::{Crown, simple_at, simple_standing};
use vitals::canopy::Tier;

fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);
    // Simple trees (`base:tree_style` simple): height and clear trunk (the lowest leaves) per layer.
    for crown in [Crown::Round, Crown::Cone, Crown::Slim] {
        for tier in [Tier::Under, Tier::Canopy, Tier::Emergent] {
            let (mut height, mut clear, mut lo, mut hi) = (0.0, 0.0, i32::MAX, i32::MIN);
            for seed in 0..count {
                let t = simple_at(crown, 1, 2, 0, 0, 0, weft_sdk::noise::hash(seed, 9, 9, 9) >> 32, false).with_tier(tier);
                let mut low = i32::MAX;
                simple_standing(&t, |_, y, _, s, _| {
                    if s == 2 {
                        low = low.min(y);
                    }
                });
                height += t.height() as f64;
                clear += low as f64;
                (lo, hi) = (lo.min(low), hi.max(low));
            }
            println!(
                "simple   {:16} {:9} height {:.1}  clear trunk {:.1} ({lo}-{hi})",
                format!("{crown:?}"),
                format!("{tier:?}"),
                height / count as f64,
                clear / count as f64
            );
        }
    }
    for r in RECIPES
        .iter()
        .filter(|r| ["base:oak_log", "base:birch_log", "base:spruce_log"].contains(&r.log))
    {
        for tier in [Tier::Under, Tier::Canopy, Tier::Emergent] {
            let (mut bare, mut width, mut height, mut clear) = (0, 0.0, 0.0, 0.0);
            for seed in 0..count {
                let v = if seed % 7 == 0 { Variant::Big } else { Variant::Standing };
                let b: Vec<(i32, i32, i32, Part)> = match colonized(r.log) {
                    Some(c) => {
                        base_api::tree_recipes::capped(&base_api::colonize::blocks(&tiered_colonized(&varied_colonized(c, v), tier), seed))
                            .into_iter()
                            .map(|(x, y, z, p, _)| (x, y, z, p))
                            .collect()
                    }
                    None => base_api::branchy::blocks(&tiered(&varied(r, v), tier), seed),
                };
                let top = b.iter().map(|e| e.1).max().unwrap_or(0);
                let leafy = |x: i32, y: i32, z: i32| {
                    b.iter()
                        .any(|e| e.0 == x && e.2 == z && e.1 > y && matches!(e.3, Part::Leaves))
                };
                let exposed: Vec<_> = b
                    .iter()
                    .filter(|e| matches!(e.3, Part::Log { .. }) && e.1 * 3 >= top * 2 && !leafy(e.0, e.1, e.2))
                    .collect();
                if !exposed.is_empty() {
                    bare += 1;
                    if std::env::var("SHOW").is_ok() && seed < 3 {
                        println!(
                            "  {} {tier:?} seed {seed} top {top}: {:?}",
                            r.log,
                            exposed
                                .iter()
                                .map(|e| (
                                    e.0,
                                    e.1,
                                    e.2,
                                    match e.3 {
                                        Part::Log { axis } => axis,
                                        _ => 9,
                                    }
                                ))
                                .collect::<Vec<_>>()
                        );
                    }
                }
                let w = b
                    .iter()
                    .filter(|e| matches!(e.3, Part::Leaves))
                    .map(|e| e.0.abs().max(e.2.abs()))
                    .max()
                    .unwrap_or(0);
                width += w as f64;
                height += top as f64;
                clear += b.iter().filter(|e| matches!(e.3, Part::Leaves)).map(|e| e.1).min().unwrap_or(0) as f64;
            }
            println!(
                "branched {:16} {:9} bare-topped {:3}/{count}  crown half-width {:.1}  height {:.1}  clear trunk {:.1}",
                r.log,
                format!("{tier:?}"),
                bare,
                width / count as f64,
                height / count as f64,
                clear / count as f64
            );
        }
    }
}

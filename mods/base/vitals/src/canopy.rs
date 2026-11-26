//! A forest's stand structure (`notes/ECOLOGY.md`, `notes/SYSTEMS.md`): the heights its crowns
//! grow to, so a canopy reads as separate crowns of different heights with shade and gaps between
//! them, not one flat mat of leaves (the Look session's tree diagnosis, 11-05: Bliss's canopy
//! measured 3-5x rougher than ours, and lighting, AO and textures barely changed it; the cause was
//! every crown alike, wide and flat, at one height).
//!
//! A real old-growth stand has three layers (Białowieża's strict reserve: hornbeam and young trees
//! under a 25-35 m canopy of oak, lime and ash, Norway spruces and old oaks rising through it to
//! 40-50 m). In blocks, scaled to the game's trees: suppressed and young trees (`Tier::Under`)
//! under the canopy, the canopy itself, and a few emergents over it. Each wild tree's site rolls
//! its tier from its biome's shares (`shares`); `base_api::trees` sizes its crown by the tier
//! (`crown`), simple and branched trees alike, and distant terrain samples the same shapes.

/// Which layer of the stand a tree grows into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// Under the canopy: suppressed or young, short with a small crown.
    Under,
    /// The canopy: most trees.
    Canopy,
    /// Over the canopy: a tall old one with a broad, lumpy crown.
    Emergent,
}

/// The crown shapes a simple tree grows (`base_api::trees`, by its log's tag).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A broadleaf's dome (oak).
    Round,
    /// A conifer's spire (spruce).
    Cone,
    /// A narrow, long crown (birch).
    Slim,
}

/// A crown's size: its trunk `height` from `low` to `low + spread - 1` (to the crown's top), its
/// `radius` at the widest (blocks from the trunk's middle), its `depth` in layers (a cone's: where
/// the needles start, as a share of the height), and how many side lobes it carries (an old oak's
/// big limbs each under a clump of leaves, like Minecraft's big oaks: a lumpy crown, not a ball).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crown {
    pub low: i32,
    pub spread: i32,
    pub radius: f32,
    pub depth: i32,
    pub lobes: u32,
}

const fn c(low: i32, spread: i32, radius: f32, depth: i32, lobes: u32) -> Crown {
    Crown { low, spread, radius, depth, lobes }
}

/// Each kind's crown in each tier. The crowns are about Minecraft's own trees' size (five to seven
/// across), lifted on clear trunks (Tester 2's walk at spot C, 11-06: the canopy came down to three
/// blocks over the floor, a walk was inside the leaves; Bliss's floor has clear trunks under a high
/// canopy, and a closed stand's oaks self-prune to a long clear bole): a canopy tree 11-14 high with
/// some 7-9 blocks bare, so a stand at Minecraft's density (about one tree in 25
/// square blocks, the forests' `TREES` and `STAND_TREES` together) closes most of the sky but
/// leaves pits between crowns; the crowns deep with steep sides (Bliss's shots: tall clumps whose
/// shaded walls and the pits between them go near black, lit tops at many heights), the emergents
/// half again to twice as high (the PC look, 11-06: one in five old-growth trees over the canopy
/// gave its skyline, fewer read flat). Cones (`depth` 0): `radius` at the base of the
/// needles, which start at `cone_bare` of the height.
pub fn crown(kind: Kind, tier: Tier) -> Crown {
    match (kind, tier) {
        (Kind::Round, Tier::Under) => c(4, 2, 1.8, 3, 0),
        (Kind::Round, Tier::Canopy) => c(11, 4, 3.0, 5, 1),
        (Kind::Round, Tier::Emergent) => c(16, 4, 3.6, 6, 3),
        (Kind::Cone, Tier::Under) => c(5, 3, 1.6, 0, 0),
        (Kind::Cone, Tier::Canopy) => c(11, 4, 2.4, 0, 0),
        (Kind::Cone, Tier::Emergent) => c(17, 4, 3.0, 0, 0),
        (Kind::Slim, Tier::Under) => c(5, 2, 1.4, 3, 0),
        (Kind::Slim, Tier::Canopy) => c(11, 3, 2.0, 5, 0),
        (Kind::Slim, Tier::Emergent) => c(15, 3, 2.5, 6, 1),
    }
}

/// The share of a cone tree's height bare under its needles, by layer: a young spruce keeps its
/// skirt to the ground, a closed stand's spruces lose their lower whorls (a third and more), the
/// tall old ones half.
pub fn cone_bare(tier: Tier) -> f32 {
    match tier {
        Tier::Under => 0.2,
        Tier::Canopy => 0.38,
        Tier::Emergent => 0.45,
    }
}

/// How much trunk a branched broadleaf keeps clear under its crown, by layer, at least (blocks):
/// the canopy's and emergents' grow up to the light and shed their lower limbs.
pub fn clear_trunk(tier: Tier) -> f32 {
    match tier {
        Tier::Under => 0.0,
        Tier::Canopy => 7.0,
        Tier::Emergent => 10.0,
    }
}

/// The chance a canopy tree carries a side lobe (an emergent always carries its `lobes`): some
/// crowns lopsided, so neighbours' outlines differ.
pub const CANOPY_LOBE: f32 = 0.4;

/// A stand's shares of trees under the canopy and over it, by biome; the rest form the canopy.
#[derive(Clone, Copy, Debug)]
pub struct Stand {
    pub biome: &'static str,
    pub under: f32,
    pub emergent: f32,
}

/// The stands. Old growth: the most of both (dense hornbeam and young growth under, old spruces
/// and oaks through the canopy); the managed forests fewer old ones; the taiga's spruces in the
/// even ranks of a stand regrown after fire, few towering.
pub const STANDS: &[Stand] = &[
    Stand { biome: "base:old_growth_forest", under: 0.25, emergent: 0.22 },
    Stand { biome: "base:forest", under: 0.25, emergent: 0.12 },
    Stand { biome: "base:birch_forest", under: 0.22, emergent: 0.08 },
    Stand { biome: "base:taiga", under: 0.3, emergent: 0.08 },
    Stand { biome: "base:tropical_forest", under: 0.25, emergent: 0.12 },
    Stand { biome: "base:swamp", under: 0.3, emergent: 0.05 },
    Stand { biome: "base:sequoia_grove", under: 0.3, emergent: 0.05 },
];

/// Anywhere else (open land's lone trees, the mountains' spruces): mostly canopy-sized.
pub const STAND_ELSE: Stand = Stand { biome: "", under: 0.2, emergent: 0.06 };

/// A biome's stand.
pub fn stand(biome: &str) -> &'static Stand {
    STANDS
        .iter()
        .find(|s| s.biome == biome)
        .unwrap_or(&STAND_ELSE)
}

/// The tier a tree at a site of `biome` grows into, `roll` (0..1) its site's own.
pub fn tier(biome: &str, roll: f32) -> Tier {
    let s = stand(biome);
    if roll < s.emergent {
        Tier::Emergent
    } else if roll < s.emergent + s.under {
        Tier::Under
    } else {
        Tier::Canopy
    }
}

/// How a branched tree's recipe grows in each tier (`base_api::tree_recipes::tiered`): its height
/// times this, its crown's reach kept within its scatter's.
pub fn branched_height(tier: Tier) -> f32 {
    match tier {
        Tier::Under => 0.65,
        Tier::Canopy => 1.0,
        Tier::Emergent => 1.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_heights_apart() {
        // Each tier's tallest stays under the next's shortest, so a stand shows three layers.
        for kind in [Kind::Round, Kind::Cone, Kind::Slim] {
            let [u, c, e] = [Tier::Under, Tier::Canopy, Tier::Emergent].map(|t| crown(kind, t));
            assert!(u.low + u.spread - 1 < c.low, "{kind:?}");
            assert!(c.low + c.spread - 1 + 2 <= e.low, "{kind:?}: emergents stand clear of the canopy");
            assert!(u.radius < c.radius && c.radius <= e.radius, "{kind:?}");
            // A walk under the canopy is under its leaves, not in them.
            if c.depth > 0 {
                assert!(c.low - c.depth >= 6, "{kind:?}: the canopy's trunks clear");
            }
        }
    }

    #[test]
    fn crowns_within_the_scatters_reach() {
        // The tree scatters reach 8 (`worldgen.rs`): a crown and its lobes stay inside.
        for kind in [Kind::Round, Kind::Cone, Kind::Slim] {
            for t in [Tier::Under, Tier::Canopy, Tier::Emergent] {
                let c = crown(kind, t);
                let lobe = if c.lobes > 0 { 2.5 + 2.2 } else { 0.0 };
                assert!(c.radius.max(lobe) + 1.0 < 8.0, "{kind:?} {t:?}");
                // and under the scatter's 32 up.
                assert!(c.low + c.spread + 2 < 32);
            }
        }
    }

    #[test]
    fn shares_leave_a_canopy() {
        for s in STANDS.iter().chain([&STAND_ELSE]) {
            assert!(s.under + s.emergent < 0.6, "{}", s.biome);
            assert!(s.emergent > 0.0, "{}: every forest has a few emergents", s.biome);
        }
        assert_eq!(tier("base:old_growth_forest", 0.0), Tier::Emergent);
        assert_eq!(tier("base:old_growth_forest", 0.99), Tier::Canopy);
    }
}

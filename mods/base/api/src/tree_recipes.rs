//! Each species' recipe for trees grown from branches (`branchy.rs`): its trunk, its branches
//! and its leaves, as numbers. A log with no recipe here keeps its plain crown (`trees.rs`).
//! Tuned by eye on `mods/base/source/trees/sheet.py` (today's crown next to grown ones, at 32
//! texels a block); the shapes after the world-looks round (`reports/world-looks.md`, item 1).

use crate::branchy::Recipe;

pub const RECIPES: &[Recipe] = &[
    // An oak (Quercus robur): a short stout trunk (2 x 2 on the bigger ones, up to its first
    // limbs), a few big limbs out and up from low on it, the leader ending inside the crown,
    // overlapping round clusters: a broad dome wider than it is tall.
    Recipe {
        log: "base:oak_log",
        height: 7.5,
        size_var: 0.25,
        width: 1.35,
        width_loss: 0.09,
        drift: 7.0,
        droop: 0.0,
        branch_start: 0.3,
        branch_every: 0.8,
        branch_angle: 48.0,
        branch_width: 0.7,
        branch_len: 0.7,
        taper: 0.45,
        leaf_r: 2.6,
        leaf_ry: 2.1,
        ragged: 0.45,
        ..Recipe::BASE
    },
    // A birch (Betula pendula): tall and slim, its white trunk straight up through the crown,
    // short thin branches steep up and drooping at their ends, leaves along them: a narrow oval
    // crown of fine twigs, no thick wood showing in it.
    Recipe {
        log: "base:birch_log",
        height: 10.0,
        width_loss: 0.07,
        drift: 4.0,
        droop: 0.3,
        branch_start: 0.42,
        branch_every: 0.9,
        branch_angle: 48.0,
        branch_width: 0.4,
        orders: 1,
        branch_len: 0.42,
        taper: 0.6,
        leaf_r: 1.3,
        leaf_ry: 1.2,
        ragged: 0.55,
        leaf_along: true,
        ..Recipe::BASE
    },
    // A spruce (Picea abies): a straight trunk to the tip, regular whorls of five nearly flat
    // drooping branches from near the ground, shorter up the tree, needles along their whole
    // length hiding the trunk: a clean, narrow cone. Big ones have a 2 x 2 foot.
    Recipe {
        log: "base:spruce_log",
        height: 11.0,
        size_var: 0.3,
        width: 1.25,
        width_loss: 0.075,
        leaf_width: 2.0,
        drift: 1.0,
        droop: 0.35,
        branch_start: 0.12,
        branch_every: 1.0,
        branch_count: 5,
        branch_angle: 82.0,
        branch_width: 0.32,
        orders: 1,
        branch_len: 0.34,
        taper: 0.95,
        branch_var: 0.35,
        leaf_r: 1.2,
        leaf_ry: 0.9,
        ragged: 0.35,
        leaf_along: true,
        ..Recipe::BASE
    },
    // An acacia: a trunk leaning a little, forking low into leaders, under one wide flat parasol.
    Recipe {
        log: "base:acacia_log",
        height: 7.0,
        width_loss: 0.1,
        drift: 8.0,
        lean: 4.0,
        branch_start: 0.4,
        branch_every: 0.6,
        branch_count: 3,
        branch_angle: 38.0,
        branch_width: 0.85,
        fork: true,
        leaf_r: 2.6,
        leaf_ry: 0.9,
        ragged: 0.3,
        ..Recipe::BASE
    },
    // A mesquite: low, wide and crooked, forking at about three blocks (lower read as a bush), a thin open flat crown.
    Recipe {
        log: "base:mesquite_log",
        height: 6.0,
        width_loss: 0.1,
        drift: 18.0,
        lean: 8.0,
        branch_start: 0.5,
        branch_every: 0.5,
        branch_count: 3,
        branch_angle: 55.0,
        branch_width: 0.85,
        fork: true,
        leaf_r: 1.9,
        leaf_ry: 0.8,
        ragged: 0.5,
        ..Recipe::BASE
    },
    // The acuri palm (Attalea phalerata): a short stout trunk, a little bent, a big crown of long
    // fronds sweeping out and down.
    Recipe {
        log: "base:acuri_palm_log",
        height: 6.0,
        width_loss: 0.06,
        drift: 3.0,
        lean: 4.0,
        rise: 0.9,
        orders: 0,
        fronds: 7,
        frond_len: 6.0,
        frond_droop: 0.5,
        leaf_r: 1.0,
        leaf_ry: 0.8,
        frond_angle: 32.0,
        frond_spread: 10.0,
        ..Recipe::BASE
    },
    // The moriche palm (Mauritia flexuosa): tall and dead straight (a slight lean shows as one jog), a round crown of fan leaves held
    // out and only a little drooping.
    Recipe {
        log: "base:moriche_palm_log",
        height: 14.0,
        width_loss: 0.05,
        drift: 0.0,
        lean: 0.0,
        rise: 1.0,
        orders: 0,
        fronds: 16,
        frond_len: 3.8,
        frond_droop: 0.3,
        leaf_r: 1.2,
        leaf_ry: 1.0,
        frond_angle: 64.0,
        frond_spread: 26.0,
        ..Recipe::BASE
    },
    // An araucaria: a tall straight bare trunk (2 x 2 at its foot), whorls of four near the top
    // held flat and turning up at their tips, tufts only at the tips: flat layered plates.
    Recipe {
        log: "animals:araucaria_log",
        height: 14.0,
        width: 1.6,
        width_loss: 0.08,
        drift: 1.5,
        droop: -0.12,
        branch_start: 0.62,
        branch_every: 2.2,
        branch_count: 4,
        branch_angle: 86.0,
        branch_width: 0.22,
        branch_len: 0.42,
        orders: 1,
        taper: 0.6,
        leaf_r: 2.1,
        leaf_ry: 0.8,
        ragged: 0.3,
        ..Recipe::BASE
    },
    // A ginkgo: a straight leader, short limbs held out nearly flat, sparse small clusters: a
    // narrow, open, uneven crown (an oak's is broad and full).
    Recipe {
        log: "animals:ginkgo_log",
        height: 10.0,
        width_loss: 0.07,
        drift: 4.0,
        branch_start: 0.3,
        branch_every: 1.3,
        branch_angle: 72.0,
        branch_width: 0.32,
        orders: 1,
        taper: 0.5,
        leaf_r: 1.4,
        leaf_ry: 1.1,
        ragged: 0.55,
        ..Recipe::BASE
    },
    // A swamp cypress (Taxodium): a buttressed 2 x 2 foot, a tall trunk, level branches with
    // feathery needles along them, a narrow crown flattening at the top.
    Recipe {
        log: "animals:swamp_cypress_log",
        height: 13.0,
        width: 1.9,
        width_loss: 0.12,
        drift: 2.5,
        droop: 0.15,
        branch_start: 0.35,
        branch_every: 1.0,
        branch_count: 2,
        branch_angle: 78.0,
        branch_width: 0.3,
        orders: 1,
        branch_len: 0.3,
        taper: 0.5,
        leaf_width: 2.0,
        leaf_r: 1.2,
        leaf_ry: 0.8,
        ragged: 0.35,
        leaf_along: true,
        ..Recipe::BASE
    },
    // A scale-leaved conifer (Cretaceous cupressaceae): dense and narrow, short branches steep up,
    // scale leaves along them: a tall column.
    Recipe {
        log: "animals:scale_conifer_log",
        height: 10.0,
        width_loss: 0.08,
        drift: 2.0,
        branch_start: 0.2,
        branch_every: 0.7,
        branch_count: 3,
        branch_angle: 45.0,
        branch_width: 0.3,
        orders: 1,
        branch_len: 0.22,
        taper: 0.7,
        leaf_width: 2.0,
        leaf_r: 1.1,
        leaf_ry: 1.2,
        ragged: 0.2,
        leaf_along: true,
        ..Recipe::BASE
    },
    // Tempskya, a tree fern: a short thick trunk of roots, a crown of fronds arching out.
    Recipe {
        log: "animals:tempskya_log",
        height: 5.0,
        size_var: 0.25,
        width_loss: 0.06,
        drift: 4.0,
        lean: 2.0,
        orders: 0,
        fronds: 6,
        frond_len: 3.2,
        frond_droop: 0.55,
        leaf_r: 1.0,
        leaf_ry: 0.9,
        frond_angle: 42.0,
        frond_spread: 12.0,
        ..Recipe::BASE
    },
    // A Pacific dogwood (Cornus nuttallii), the giant groves' small tree: a short trunk, its
    // branches in opposite pairs held out level, tips turning up, in flat layered sprays: an open
    // rounded crown about as wide as it is tall (it was a box of leaves, the tree-look audit).
    Recipe {
        log: "base:dogwood_log",
        height: 6.0,
        size_var: 0.25,
        width_loss: 0.1,
        drift: 6.0,
        droop: -0.08,
        branch_start: 0.3,
        branch_every: 1.1,
        branch_count: 2,
        branch_angle: 68.0,
        branch_width: 0.55,
        branch_len: 0.6,
        taper: 0.4,
        leaf_r: 2.0,
        leaf_ry: 1.0,
        ragged: 0.4,
        twig: 0.32,
        ..Recipe::BASE
    },
    // A giant sequoia (Sequoiadendron giganteum, `vitals::wild` for where): the real ones 80-95 m
    // tall on trunks 6-11 m across, here about 60 blocks on a round trunk 4 to 7 wide, flaring
    // into buttresses at the foot; bare for its lower half, then short branches all round,
    // shorter up the tree, in rounded clumps: a narrow, rounded spire.
    SEQUOIA,
    // The tropical forest's emergent, the Brazil nut (Bertholletia excelsa, Manaus's terra
    // firme): 30-50 m on a dead-straight trunk bare for two thirds, buttressed at the foot, its
    // umbrella crown above the canopy.
    BRAZIL_NUT,
    ANDIROBA,
    MATAMATA,
    IPE,
    // Cacao (Theobroma cacao), the Amazon's understorey tree: 4-8 m, slender, branching low
    // into a few limbs held out under a crown of big drooping leaves.
    Recipe {
        log: "base:cacao_log",
        height: 5.0,
        width: 0.9,
        width_loss: 0.1,
        drift: 4.0,
        droop: 0.3,
        branch_start: 0.3,
        branch_every: 0.6,
        branch_count: 3,
        branch_angle: 50.0,
        branch_width: 0.6,
        orders: 1,
        leaf_r: 1.8,
        leaf_ry: 1.2,
        ragged: 0.4,
        ..Recipe::BASE
    },
];

/// The tropical canopy (about 26-29 blocks, under its emergents): andiroba's broad dark crown on a tall clean trunk.
pub const ANDIROBA: Recipe = Recipe {
    log: "base:andiroba_log",
    height: 25.0,
    size_var: 0.12,
    width: 1.3,
    // Wood to its crown: width / loss (here 32 blocks at size 1) must cover its height, or its
    // limbs grow leaves with no wood (leaf reach); at 0.03 the world never settled round them.
    width_loss: 0.04,
    drift: 2.0,
    branch_start: 0.62,
    branch_every: 0.7,
    branch_count: 2,
    branch_angle: 55.0,
    branch_width: 0.6,
    orders: 2,
    branch_len: 0.3,
    leaf_r: 3.2,
    leaf_ry: 1.8,
    ragged: 0.3,
    reach: 8,
    max_height: 40,
    flare: 0.2,
    flare_up: 1.5,
    ..Recipe::BASE
};

/// Matamatá: Manaus's commonest canopy tree, a slimmer trunk and a narrower, rounder crown.
pub const MATAMATA: Recipe = Recipe {
    log: "base:matamata_log",
    height: 23.0,
    size_var: 0.15,
    width: 1.2,
    width_loss: 0.04,
    drift: 3.0,
    branch_start: 0.66,
    branch_every: 0.6,
    branch_count: 2,
    branch_angle: 45.0,
    branch_width: 0.55,
    orders: 2,
    branch_len: 0.25,
    leaf_r: 2.6,
    leaf_ry: 2.0,
    ragged: 0.35,
    reach: 8,
    max_height: 38,
    ..Recipe::BASE
};

/// The golden ipê: a tall straight trunk and a high, open, spreading crown.
pub const IPE: Recipe = Recipe {
    log: "base:ipe_log",
    height: 27.0,
    size_var: 0.12,
    width: 1.3,
    width_loss: 0.04,
    drift: 2.0,
    branch_start: 0.7,
    branch_every: 0.55,
    branch_count: 3,
    branch_angle: 50.0,
    // Limbs thick enough to stay wood to their tips (thin, their last third was leaves alone).
    branch_width: 0.8,
    orders: 2,
    branch_len: 0.3,
    leaf_r: 2.5,
    leaf_ry: 1.4,
    ragged: 0.45,
    reach: 8,
    max_height: 42,
    ..Recipe::BASE
};

/// A grown Brazil nut (`RECIPES`' own row), the world's (the giant scatter, `GIANT_KINDS`).
pub const BRAZIL_NUT: Recipe = Recipe {
    log: "base:brazil_nut_log",
    // Its crown clears the canopy (26-29) by 15-20 blocks, as a real emergent rises 10-20 m over a
    // 30-40 m canopy (Ronald, 11-04: "C's giants"; Planner: "well above the canopy").
    height: 58.0,
    size_var: 0.15,
    width: 3.0,
    // Thinning slowly enough to stay wood to its top: at 0.06 a smaller one's trunk ran out
    // under its crown, and its limbs grew leaves with no wood (the PC's measure: wood 2 out,
    // leaves 10).
    width_loss: 0.035,
    leaf_width: 0.7,
    drift: 1.0,
    rise: 1.0,
    droop: 0.1,
    // An umbrella of a few big limbs, not many small ones: a scatter's shape keeps 4,096 blocks
    // (the first try's limbs every 0.6 overran it, and the crown was dropped).
    branch_start: 0.75,
    branch_every: 1.6,
    branch_count: 3,
    branch_angle: 62.0,
    branch_width: 0.6,
    orders: 2,
    // Limbs scale with the height: short, so the umbrella stays inside the giant scatter's reach
    // and every leaf within LEAF_REACH of a log (at 0.3 and 0.22 a 58-high crown's limbs ran past
    // the reach, which cut their wood and left leaves with no log).
    branch_len: 0.18,
    taper: 0.4,
    leaf_r: 2.6,
    leaf_ry: 1.2,
    ragged: 0.3,
    // The giant scatter's reach (`worldgen.rs`, 12; at 14 the world never finished generating
    // round a giant). Its wood stays within 8 (the PC's measure), so the reach trims only leaves.
    reach: 12,
    // 58 high, a sixth more at most, and its crown over that; the scatter reaches 96 up.
    max_height: 80,
    flare: 0.55,
    flare_up: 3.0,
    lobes: 0.35,
    ..Recipe::BASE
};

/// A young Brazil nut, grown from a planted nut: a pole of a tree with a small crown, under the
/// canopy, waiting for a gap (`BRAZIL_NUT` is the grown one).
pub const YOUNG_BRAZIL_NUT: Recipe = Recipe {
    log: "base:brazil_nut_log",
    height: 14.0,
    width: 1.2,
    width_loss: 0.07,
    drift: 2.0,
    branch_start: 0.55,
    branch_every: 0.6,
    branch_count: 2,
    branch_angle: 60.0,
    branch_width: 0.55,
    orders: 1,
    branch_len: 0.3,
    leaf_r: 2.0,
    leaf_ry: 1.2,
    reach: 8,
    max_height: 22,
    flare: 0.2,
    flare_up: 1.5,
    ..Recipe::BASE
};

/// A grown giant sequoia (`RECIPES`' own row).
pub const SEQUOIA: Recipe = Recipe {
    log: "base:sequoia_log",
    height: 60.0,
    size_var: 0.18,
    width: 5.5,
    width_loss: 0.075,
    leaf_width: 0.9,
    drift: 1.2,
    rise: 1.0,
    droop: 0.25,
    branch_start: 0.5,
    branch_every: 0.5,
    branch_count: 1,
    branch_angle: 72.0,
    branch_width: 0.5,
    orders: 2,
    branch_len: 0.13,
    taper: 0.6,
    leaf_r: 2.4,
    leaf_ry: 1.7,
    ragged: 0.25,
    reach: 10,
    max_height: 80,
    flare: 0.45,
    flare_up: 6.0,
    lobes: 0.22,
    ..Recipe::BASE
};

/// A young giant sequoia, a few centuries old: conical, branched nearly to the ground, its trunk
/// 2 to 3 wide (for the sheet and a grove's young; `SEQUOIA` is the grown one).
pub const YOUNG_SEQUOIA: Recipe = Recipe {
    log: "base:sequoia_log",
    height: 26.0,
    size_var: 0.2,
    width: 2.6,
    width_loss: 0.085,
    leaf_width: 0.9,
    drift: 1.5,
    droop: 0.2,
    branch_start: 0.15,
    branch_every: 0.5,
    branch_angle: 70.0,
    branch_width: 0.5,
    orders: 2,
    branch_len: 0.2,
    taper: 0.9,
    leaf_r: 2.0,
    leaf_ry: 1.3,
    ragged: 0.22,
    reach: 8,
    max_height: 40,
    flare: 0.3,
    flare_up: 3.0,
    lobes: 0.1,
    ..Recipe::BASE
};

/// The young of the trees that grow too big to come from a sapling at once: a planted sequoia
/// grows into a young one (`YOUNG_SEQUOIA`), not a giant.
pub const YOUNG: &[Recipe] = &[YOUNG_SEQUOIA, YOUNG_BRAZIL_NUT];

/// How far a young tree of the understory reaches sideways (its scatter's reach in `worldgen.rs`).
pub const UNDERSTORY_REACH: i32 = 3;

/// A young tree of the understory (`trees::young_tree_blocks`): the species' own recipe at about
/// a third of its height on a trunk a block wide, so a young oak already reads as an oak and a
/// young birch as a birch (a ball on a stick read as every species at once, the tree-look audit).
/// Its branches come closer together, its clusters about half as wide (bigger ones closed the
/// forest floor, the PC look); one order of branches at most.
pub fn understory(r: &Recipe) -> Recipe {
    let height = (r.height * 0.36).clamp(3.0, 7.0);
    // The share of its width the grown trunk keeps at its top, kept for the young one.
    let left = (1.0 - r.width_loss * r.height / r.width).clamp(0.2, 0.8);
    Recipe {
        height,
        width: 1.0,
        width_loss: (1.0 - left) / height,
        branch_every: (r.branch_every * 0.6).max(0.5),
        // Young trees branch low and leafy (they shed their low limbs only as they grow): a
        // giant's young was a bare pole under a small crown, a lollipop.
        branch_start: (r.branch_start * 0.6).clamp(0.2, 0.4),
        branch_count: r.branch_count.min(4),
        orders: r.orders.min(1),
        leaf_r: (r.leaf_r * 0.55).max(0.9),
        leaf_ry: (r.leaf_ry * 0.7).max(0.9),
        // fronds (a young palm's) a little shorter
        frond_len: r.frond_len * 0.6,
        // Thin wood as leafy sprays, so a young tree is no knot of logs (the cacao's).
        twig: if r.leaf_along { r.twig } else { r.twig.max(0.3) },
        reach: UNDERSTORY_REACH,
        max_height: 12,
        flare: 0.0,
        lobes: 0.0,
        ..*r
    }
}

/// The recipe a sapling of this log grows by: its young one's, if it has one, else its own.
pub fn sapling_of(log: &str) -> Option<&'static Recipe> {
    YOUNG.iter().find(|r| r.log == log).or_else(|| of(log))
}

/// The recipe for a log, by its block name (`base:oak_log`).
pub fn of(log: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.log == log)
}

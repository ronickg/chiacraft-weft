//! Each species' recipe for trees grown from branches (`branchy.rs`): its trunk, its branches
//! and its leaves, as numbers. A log with no recipe here keeps its plain crown (`trees.rs`).
//! Tuned by eye on `mods/base/source/trees/sheet.py` (today's crown next to grown ones, at 32
//! texels a block); the shapes after the world-looks round (`reports/world-looks.md`, item 1).

use crate::branchy::Recipe;

pub const RECIPES: &[Recipe] = &[
    // An oak: a short trunk, a few big limbs out and up, lumpy round clusters, a broad crown.
    Recipe { log: "base:oak_log", height: 7.0, width: 1.1, width_loss: 0.1, branch_start: 0.4, branch_every: 1.1, branch_angle: 55.0, branch_width: 0.75, leaf_r: 2.3, leaf_ry: 1.8, ragged: 0.5, ..Recipe::BASE },
    // A birch: tall and slim, a pale trunk, short branches steep up, small clusters, narrow.
    Recipe {
        log: "base:birch_log",
        height: 9.0,
        width_loss: 0.085,
        branch_start: 0.3,
        branch_every: 0.7,
        branch_angle: 35.0,
        branch_width: 0.55,
        orders: 1,
        taper: 0.4,
        leaf_r: 1.5,
        leaf_ry: 1.5,
        ..Recipe::BASE
    },
    // A spruce: a straight trunk to the tip, whorls of branches nearly flat and drooping, shorter
    // up the tree, needles along their whole length: a ragged cone. Big ones have a 2 x 2 foot.
    Recipe { log: "base:spruce_log", height: 11.0, size_var: 0.3, width: 1.25, width_loss: 0.075, leaf_width: 2.0, drift: 2.0, droop: 0.4, branch_start: 0.22, branch_every: 0.9, branch_count: 3, branch_angle: 80.0, branch_width: 0.32, orders: 1, branch_len: 0.36, taper: 1.0, leaf_r: 1.1, leaf_ry: 0.9, ragged: 0.45, leaf_along: true, ..Recipe::BASE },
    // An acacia: a trunk leaning a little, forking low into leaders, under one wide flat parasol.
    Recipe { log: "base:acacia_log", height: 7.0, width_loss: 0.1, drift: 8.0, lean: 4.0, branch_start: 0.4, branch_every: 0.6, branch_count: 3, branch_angle: 38.0, branch_width: 0.85, fork: true, leaf_r: 2.6, leaf_ry: 0.9, ragged: 0.3, ..Recipe::BASE },
    // A mesquite: low, wide and crooked, forking at about three blocks (lower read as a bush), a thin open flat crown.
    Recipe { log: "base:mesquite_log", height: 6.0, width_loss: 0.1, drift: 18.0, lean: 8.0, branch_start: 0.5, branch_every: 0.5, branch_count: 3, branch_angle: 55.0, branch_width: 0.85, fork: true, leaf_r: 1.9, leaf_ry: 0.8, ragged: 0.5, ..Recipe::BASE },
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
        fronds: 10,
        frond_len: 5.0,
        frond_droop: 0.38,
        leaf_r: 1.0,
        leaf_ry: 0.8,
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
        fronds: 12,
        frond_len: 3.5,
        frond_droop: 0.25,
        leaf_r: 1.2,
        leaf_ry: 1.0,
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
        fronds: 8,
        frond_len: 3.2,
        frond_droop: 0.45,
        leaf_r: 1.0,
        leaf_ry: 0.9,
        ..Recipe::BASE
    },
];

/// The recipe for a log, by its block name (`base:oak_log`).
pub fn of(log: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.log == log)
}

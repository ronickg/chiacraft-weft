//! A big meteorite's live crater (Research round 17, addendum 2, approved by Ronald 11-06): the
//! shape the base mod carves where a big impact lands (`mods/base/src/impact.rs`). When and
//! where one falls is the sky events' schedule (`events`, First hours); this is the hole it makes.
//! Worldgen's ancient craters (`base_api::land`'s `CRATER_*`) are bigger, older and buried.

/// The crater's radius to its rim's crest (blocks), smallest and largest: kept small enough to
/// carve live (a radius-12 bowl moves some 5,000 blocks).
pub const IMPACT_RADIUS: (i32, i32) = (7, 12);

/// The bowl's depth at its middle, in radii (Barringer is about a fifth of its width deep).
pub const IMPACT_DEPTH: f64 = 0.6;

/// The rim's height over the ground round it (blocks), lowest and highest: thrown-out rock.
pub const IMPACT_RIM: (i32, i32) = (2, 4);

/// The rim's outer foot, in radii from the middle: where it has blended back into the land.
pub const IMPACT_RIM_OUT: f64 = 1.6;

/// Out to this many radii grass is scorched to coarse dirt, thinning outward, and the small
/// plants on it go.
pub const IMPACT_SCORCH: f64 = 2.0;

/// The carve is spread over this many ticks (one second): about 250 block edits a tick for the
/// largest, where a creeper's blast does 113 at once.
pub const IMPACT_CARVE_TICKS: u32 = 20;

/// Blocks over the ground the bowl clears at most (a tree standing in it goes with it).
pub const IMPACT_CLEAR_UP: i32 = 40;

/// Shares of the rim's thrown-out rock (out of 100): suevite (impact breccia; gravel until it
/// exists), then cobblestone; the rest dirt.
pub const IMPACT_RUBBLE: (u32, u32) = (45, 40);

/// Meteorites in the core at the bowl's bottom, under a block or two of broken rock (before
/// the sky events' share for more players nearby), fewest and most (addendum 2's core; the
/// star-steel mass takes their place once crafting has it).
pub const IMPACT_CORE: (i32, i32) = (3, 6);

/// Meteorites scattered in the rim's rubble, fewest and most (their drops are addendum 2's 6-12
/// rim lumps).
pub const IMPACT_RIM_METEORITES: (i32, i32) = (2, 4);

/// The core's meteorites lie in a 3 by 3 under the middle, a second layer under the first when
/// there are more than nine.
pub const IMPACT_CORE_LAYER: i32 = 9;

// The live crater stays small enough to carve in a second, and shallower than wide.
const _: () = {
    assert!(IMPACT_RADIUS.0 <= IMPACT_RADIUS.1 && IMPACT_RADIUS.1 <= 12);
    assert!(IMPACT_DEPTH > 0.0 && IMPACT_DEPTH < 1.0);
    assert!(IMPACT_RIM.0 <= IMPACT_RIM.1 && IMPACT_RIM.1 < IMPACT_RADIUS.0);
    assert!(IMPACT_RIM_OUT > 1.0 && IMPACT_SCORCH >= IMPACT_RIM_OUT);
    assert!(IMPACT_RUBBLE.0 + IMPACT_RUBBLE.1 <= 100);
    assert!(IMPACT_CORE.0 <= IMPACT_CORE.1 && IMPACT_CORE.1 <= IMPACT_CORE_LAYER);
    assert!(IMPACT_RIM_METEORITES.0 <= IMPACT_RIM_METEORITES.1);
};

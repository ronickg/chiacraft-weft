//! Worldgen's numbers: the sea, water tints, and the trees', clearings' and wild plants' measures.

pub const SEA: i32 = base_api::SEA;

/// Water's tint in base's biomes, muted toward the sky it mirrors (Bliss's lakes: saturation 0.10-0.19
/// at noon; the engine's default 0x3f76e4 is 0.73). The pack multiplies it into what the water
/// scatters back, 30% in the shallows to 70% deep; the cold biomes and the swamp keep their own.
pub(super) const WATER_TINT: u32 = 0x607ca0;

pub(super) const OCEAN_WATER_TINT: u32 = 0x557a9e;

/// The warm shallow seas' water: clearer and greener over the sand (Lighting may grade it).
pub(super) const LAGOON_WATER_TINT: u32 = 0x4f9aa6;

/// How far round a gap trees are kept back so their crowns don't close it over: a canopy tree's
/// crown reaches 8 from its trunk, a giant's 12 (the scatters' reaches); a little less, so the
/// edge's crowns overhang it as real ones do.
pub(super) const GAP_MARGIN: i32 = 6;

pub(super) const GAP_MARGIN_GIANT: i32 = 10;

pub(super) const STAND: i32 = 600;

/// Where the meadow noise is over this, flowers grow thick (a sixth of the ground or so).
pub(super) const FLOWER_MEADOW: f64 = 0.72;

pub(super) const CLEARING: i32 = 1600;

/// Water clay lies under is this deep at most.
pub(super) const CLAY_DEPTH: i32 = 4;

/// Water a floating wild plant grows on is this deep at most.
pub(super) const WILD_WATER_DEPTH: i32 = 3;

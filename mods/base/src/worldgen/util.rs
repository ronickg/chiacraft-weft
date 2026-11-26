//! Small helpers: the stand and cherry grove masks, flower patches, the wild plants' rolls.

use super::*;

/// The stand noise over the ground, about -4096 to 4096: a forest's stands where it's over
/// `STAND`, its clearings where it's under `-CLEARING`; one patch is some 40 blocks across. As a
/// mask that is on where the noise is `edge` or more (one octave of `fbm` is `perlin2` exactly),
/// so the engine thins sites by it near and in distant terrain alike.
pub(super) fn stand_over(edge: i32) -> Expr {
    let e = edge as f64 / fixed::ONE as f64;
    Expr::fbm(0x57A4D, 40.0, 1).ramp(e, e)
}

/// Where the wild cherry groves stand (`vitals::wild::CHERRY_GROVE_*`), as a mask: on in a grove
/// patch, in any biome (only those with `CHERRY_TREES` grow cherries there; the forests' own trees
/// thin there, so it's also a small glade in the other woods).
pub(super) fn cherry_grove() -> Expr {
    Expr::fbm(0xC4E77, vitals::wild::CHERRY_GROVE_SIZE, 1).ramp(vitals::wild::CHERRY_GROVE_FROM, vitals::wild::CHERRY_GROVE_FROM)
}

/// Which flower a column of a flower patch grows: one kind to a patch (its 16 x 16 cell), a few
/// strays of the others.
pub(super) fn flower_kind(seed: u64, wx: i32, wz: i32, h: u64) -> u64 {
    if h >> 60 == 0 { h >> 32 } else { hash(seed ^ 0xF70, (wx >> 4) as i64, 0, (wz >> 4) as i64) }
}

/// A column's roll for the wild plants, 0 to 1.
pub(super) fn wild_roll(seed: u64, wx: i32, wz: i32) -> f32 {
    unit(hash(seed ^ 0x3D1, wx as i64, 0, wz as i64)) as f32
}

/// The wild plant (its state) that grows on this column's `ground` in `biome`, if any
/// (`vitals::wild::pick`).
pub(super) fn wild_at(seed: u64, wx: i32, wz: i32, biome: u32, ground: wild::Ground) -> Option<u32> {
    let roll = wild_roll(seed, wx, wz);
    if roll >= wild::MOST {
        return None;
    }
    let i = wild::pick(name_of(abi::entry_kind::BIOME, biome), ground, roll)?;
    let s = crate::wild::state(i);
    (s != u32::MAX).then_some(s)
}

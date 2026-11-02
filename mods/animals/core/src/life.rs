//! Breeding and growing up as numbers (the mod's `breeding.rs` runs them in the world; the
//! website's simulator over time): how long love, rest and laying last, a young one's life stage
//! and size as it grows, how big a litter is and which parent each stat's wild points come from.

use crate::species::{ADOLESCENT, ADULT, BABY, JUVENILE, STATS, Species};

/// Thirty seconds in love; a male rests five minutes after mating.
pub const LOVE_TICKS: u32 = 600;
pub const MALE_REST: u32 = 6_000;
/// Five minutes after mating, an egg-layer lays.
pub const LAYING: u32 = 6_000;
/// Ticks an in-game day.
pub const DAY: u64 = 24_000;

/// A young one's life stage with `growing` ticks of growing up left (an adult stays one).
pub fn stage(s: &Species, growing: u32, was: u8) -> u8 {
    match growing * 3 / s.growth.max(1) {
        _ if growing == 0 => was.max(ADULT),
        2.. => BABY,
        1 => JUVENILE,
        _ => ADOLESCENT,
    }
}

/// How big it is against a grown one, with `growing` ticks left: from its birth size up.
pub fn size(s: &Species, growing: u32) -> f32 {
    if growing == 0 {
        return 1.0;
    }
    let grown = 1.0 - growing as f32 / s.growth.max(1) as f32;
    s.birth_size + (1.0 - s.birth_size) * grown
}

/// Ticks from mating to the young born, or the eggs laid.
pub fn carrying(s: &Species) -> u32 {
    if s.gestation > 0 { s.gestation } else { LAYING }
}

/// How many young a litter or clutch has, `roll` any random number.
pub fn litter(s: &Species, roll: u32) -> u32 {
    s.litter.0 + roll % (s.litter.1 - s.litter.0 + 1)
}

/// A young one's wild points: each stat's from one parent, the stronger 55% of the time
/// (`roll(100)` a random number below 100).
pub fn wild_points(mother: &[u8; STATS], father: &[u8; STATS], mut roll: impl FnMut(u32) -> u32) -> [u8; STATS] {
    std::array::from_fn(|i| {
        let (hi, lo) = (mother[i].max(father[i]), mother[i].min(father[i]));
        if roll(100) < 55 { hi } else { lo }
    })
}

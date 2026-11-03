//! Rot and fertile soil (`notes/ECOLOGY.md`, parts 8 and 9), from the agronomy research
//! (`reports/ecology/Plants and crops.md`, "Soil fertility"):
//! - farmland's fertility, 0 to 3, sets how fast crops grow on it, a fifth of the best when worn
//!   out (the Rothamsted wheat plot, unfertilised since 1843, still gives a sixth of a manured
//!   one's crop);
//! - a harvest of a crop that tires the soil takes a level;
//! - dung and compost give levels back, and so does a season left unplanted;
//! - a compost bin turns spoiled food, dung and plant waste into compost in the weeks hot
//!   compost takes, at the growing pace (`plants::GROWING`).

use crate::DAY_SECS;
use crate::plants::GROWING;

/// Fertility levels: new farmland is at the top.
pub const MAX: u8 = 3;

/// How fast crops grow on farmland of that fertility, against the best (`plants::rate`'s factor).
pub fn growth(fertility: u8) -> f32 {
    const WORN_OUT: f32 = 0.2;
    WORN_OUT + (1.0 - WORN_OUT) * fertility.min(MAX) as f32 / MAX as f32
}

/// The crops whose harvest (ripe) takes a level from the farmland under them: wheat takes about
/// 23 kg of nitrogen a tonne of grain.
pub const TIRES: [&str; 1] = ["base:wheat"];

/// What gives farmland levels back, used on it: a load of compost or of dung (about 6 kg of
/// nitrogen a tonne, a fifth of it there the first year).
pub const FEEDS: [(&str, u8); 2] = [("base:compost", 2), ("base:dung", 1)];

/// Game seconds a level comes back on farmland left unplanted, in a growing season (a fallow
/// season brings one back): the growth step's mean, at the plants' rate where it lies.
pub const FALLOW_SECS: f32 = 14.0 * DAY_SECS;

/// Steps a compost bin fills by; full, it rots into compost.
pub const BIN_FULL: u8 = 7;

/// What a compost bin takes, and how many steps one fills it by: spoiled food (any), dung, rotten
/// flesh, and plant waste.
pub const COMPOSTS: [(&str, u8); 9] = [
    ("base:dung", 2),
    ("base:rotten_flesh", 2),
    ("base:oak_leaves", 1),
    ("base:birch_leaves", 1),
    ("base:spruce_leaves", 1),
    ("base:acacia_leaves", 1),
    ("base:sapling", 1),
    ("base:wheat_seeds", 1),
    ("base:wheat", 1),
];

/// Steps spoiled food fills a bin by, whatever the food.
pub const SPOILED: u8 = 2;

/// Real days hot compost takes, turned and cured (3-6 weeks hot, 2-8 curing).
pub const COMPOST_DAYS: f32 = 60.0;

/// Game seconds a full bin takes to become compost, at the growing pace: about 23 minutes.
pub fn compost_secs() -> f32 {
    GROWING.0 * COMPOST_DAYS.powf(GROWING.1) * DAY_SECS
}

/// What `name` fills a compost bin by, if it takes it.
pub fn composts(name: &str) -> Option<u8> {
    COMPOSTS.iter().find(|(n, _)| *n == name).map(|&(_, v)| v)
}

/// Levels `name` gives farmland, if any.
pub fn feeds(name: &str) -> Option<u8> {
    FEEDS.iter().find(|(n, _)| *n == name).map(|&(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worn_soil_still_grows_a_fifth() {
        assert_eq!(growth(MAX), 1.0);
        assert!((growth(0) - 0.2).abs() < 1e-6);
        assert!(compost_secs() > 600.0 && compost_secs() < 3600.0);
    }
}

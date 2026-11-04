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

// Which crops' ripe harvest takes a level from the farmland under them and which give one back
// (beans): `crops::Crop::soil`.

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

/// Dung pats (`notes/ECOLOGY.md`, part 8): a grazer drops one every `MOUTHFULS_A_PAT` mouthfuls
/// where it stands on the ground (`mods/animals/src/forage.rs`), never within `PAT_SPACING`
/// blocks of another. It rots by the plant rule (`plants::PLANTS`, `base:dung_pat`), and lies at
/// most `PAT_MOST_DAYS` (frozen or dried out, frost and hooves break it up), then leaves a tuft of
/// fresh grass; while it lies, grazed grass within `PAT_REACH` grows back `PAT_GROWTH` times as
/// fast (a pat's ring of lush grass). Broken, it gives the dung item (`FEEDS`, `COMPOSTS`).
///
/// A block is far bigger than a pat (a cow's is about 0.07 m²), so pats are rarer than real
/// droppings: one every 20 mouthfuls keeps them to under 1% of a herd's ground at once, as in a
/// grazed pasture, in every season (`animals_core::web`'s check).
pub const MOUTHFULS_A_PAT: u32 = 20;
pub const PAT_SPACING: i32 = 2;
pub const PAT_REACH: i32 = 2;
pub const PAT_GROWTH: u32 = 2;
pub const PAT_MOST_DAYS: f32 = 3.0;

/// The chance a pat's growth step breaks it up though it didn't rot (`PAT_MOST_DAYS`).
pub fn pat_break_chance() -> f32 {
    crate::plants::of("base:dung_pat")
        .and_then(|p| crate::plants::step_secs(p, 1))
        .map_or(0.0, |step| (step / (PAT_MOST_DAYS * DAY_SECS)).min(1.0))
}

/// Game seconds a pat lies, on average, at a rot rate `rate` (`plants::rate`).
fn pat_lies(rate: f32) -> f32 {
    let step = crate::plants::of("base:dung_pat").and_then(|p| crate::plants::step_secs(p, 1)).unwrap_or(0.0);
    step / (rate + pat_break_chance() * (1.0 - rate)).max(1e-6)
}

/// Game seconds a pat lies, on average, in the day's mean air `air` (°C) and soil water `soil`
/// (0 to 1).
pub fn pat_rot_secs(air: f32, soil: f32) -> f32 {
    pat_lies(crate::plants::of("base:dung_pat").map_or(0.0, |p| crate::plants::rate(p, air, soil)))
}

/// Game seconds a pat lies, on average, through season `season` (0 spring) of a climate of that
/// mean, seasons and rain (`plants::season_rate_in`).
pub fn pat_season_secs(celsius: f32, seasons: f32, rain: [f32; 4], season: usize) -> f32 {
    pat_lies(crate::plants::of("base:dung_pat").map_or(0.0, |p| crate::plants::season_rate_in(p, celsius, seasons, rain, season)))
}

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

    /// Pats rot within two game days in a grass biome's best season, lie longer in a dry season
    /// (the llanos' winter: dried out) and in the cold, and never past `PAT_MOST_DAYS`.
    #[test]
    fn pats_rot_by_warmth_and_rain() {
        let lies = |c: &crate::climate::Climate, s: usize| pat_season_secs(c.celsius, c.seasons, c.rain, s);
        for c in crate::climate::BIOMES.iter().filter(|c| !c.underground && crate::forage::grass_grows(c.biome) > 0.0) {
            let best = (0..4).map(|s| lies(c, s)).fold(f32::MAX, f32::min);
            assert!(best <= 2.0 * DAY_SECS, "{}: a pat lies {:.1} game days in its best season", c.biome, best / DAY_SECS);
            for s in 0..4 {
                assert!(lies(c, s) <= PAT_MOST_DAYS * DAY_SECS * 1.01, "{}: a pat lies past PAT_MOST_DAYS in season {s}", c.biome);
            }
        }
        let savanna = crate::climate::of("base:savanna").unwrap();
        assert!(lies(savanna, 3) >= 2.0 * lies(savanna, 1), "the savanna's dry winter should keep pats twice as long as its rains");
        let plains = crate::climate::of("base:plains").unwrap();
        assert!(lies(plains, 3) >= 2.0 * lies(plains, 1), "a cold winter should keep pats twice as long as summer");
    }
}

//! Every growing plant's facts from real agronomy (`reports/ecology/Plants and crops.md`, its
//! numbers in `plants.json`), and one rule for all of them (`notes/ECOLOGY.md`, part 3): a growth
//! step goes ahead at the rate warmth × water, each 0 to 1, from the air and the ground where the
//! plant stands. So the season decides whether a plant grows (winter or a dry season rests it)
//! and the growing pace how fast (`GROWING`). The base mod asks `rate` at each growth step
//! (`mods/base/src/growth.rs`).

use crate::climate::{self, Climate};
use crate::{DAY_SECS, calendar, temperature};

/// How much water a plant needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Water {
    /// Desert plants: they grow at half speed on dry ground.
    Dry,
    Moderate,
    /// Water plants: only on wet ground or by water.
    Wet,
}

/// A plant that grows, by its block.
#[derive(Clone, Copy, Debug)]
pub struct Plant {
    /// The block's full name.
    pub block: &'static str,
    /// The real plant its numbers come from (`plants.json`).
    pub real: &'static str,
    /// The air it grows in, °C: from `base`, best between `best`, nothing past `max`.
    pub base: f32,
    pub best: (f32, f32),
    pub max: f32,
    pub water: Water,
    /// Real days from sowing to harvest (a crop), or from flower to ripe fruit (a bush), at a
    /// field's usual warmth and water; 0: the game's own time (trees and columns, whose real
    /// years would take hours of play).
    pub days: f32,
    /// The seasons its last stage (the ripe fruit) can come in (`calendar` order as bits, spring
    /// 1); all four: any.
    pub ripens: u8,
}

const fn p(block: &'static str, real: &'static str, warmth: (f32, f32, f32, f32), water: Water, days: f32, ripens: u8) -> Plant {
    Plant { block, real, base: warmth.0, best: (warmth.1, warmth.2), max: warmth.3, water, days, ripens }
}

const ANY: u8 = 15;
const SUMMER_AUTUMN: u8 = 2 | 4;

pub const PLANTS: [Plant; 5] = [
    p("base:wheat", "spring wheat", (0.0, 15.0, 23.0, 34.0), Water::Moderate, 110.0, ANY),
    p("base:berry_bush", "blueberry", (7.0, 20.0, 26.0, 32.0), Water::Moderate, 75.0, SUMMER_AUTUMN),
    p("base:sugar_cane", "sugar cane", (12.0, 22.0, 30.0, 38.0), Water::Wet, 0.0, ANY),
    p("base:cactus", "saguaro", (10.0, 25.0, 40.0, 50.0), Water::Dry, 0.0, ANY),
    p("base:sapling", "oak", (5.0, 18.0, 28.0, 38.0), Water::Moderate, 0.0, ANY),
];

/// The plant that grows as this block.
pub fn of(block: &str) -> Option<&'static Plant> {
    PLANTS.iter().find(|p| p.block == block)
}

/// The growing pace: game days = a · real days^k, the pace the species grow up by
/// (`animals_core::biology::GROWING`, held equal by `mods/animals/core/tests/places.rs`), so a
/// field and a litter keep their real proportions.
pub const GROWING: (f32, f32) = (0.0645, 0.7);

/// The real days are a field's in its season; at the best warmth and water a plant grows this
/// much faster.
pub const BEST_OVER_USUAL: f32 = 1.0 / 0.75;

/// Game seconds a growth step takes on average at the rate 1, over `steps` steps to harvest;
/// `None`: the game's own time.
pub fn step_secs(p: &Plant, steps: u32) -> Option<f32> {
    (p.days > 0.0).then(|| GROWING.0 * p.days.powf(GROWING.1) * DAY_SECS / steps as f32 / BEST_OVER_USUAL)
}

/// How warm enough it is, 0 to 1, in the day's mean air: rising from `base` to the best, falling
/// to nothing at `max`.
pub fn warmth(p: &Plant, air: f32) -> f32 {
    if air <= p.base || air >= p.max {
        0.0
    } else if air < p.best.0 {
        (air - p.base) / (p.best.0 - p.base)
    } else if air <= p.best.1 {
        1.0
    } else {
        (p.max - air) / (p.max - p.best.1)
    }
}

/// How well watered, 0 to 1, on ground that wet (`soil_water`).
pub fn watered(p: &Plant, soil: f32) -> f32 {
    let ramp = |from: f32, to: f32| ((soil - from) / (to - from)).clamp(0.0, 1.0);
    match p.water {
        Water::Dry => 0.5 + 0.5 * ramp(0.0, 0.2),
        Water::Moderate => ramp(0.05, 0.4),
        Water::Wet => ramp(0.3, 0.7),
    }
}

/// The chance a growth step goes ahead: warmth × water.
pub fn rate(p: &Plant, air: f32, soil: f32) -> f32 {
    warmth(p, air) * watered(p, soil)
}

/// Whether its last stage may come in `season` (0 spring; `None`: no seasons there).
pub fn ripens_in(p: &Plant, season: Option<usize>) -> bool {
    season.is_none_or(|s| p.ripens >> s & 1 != 0)
}

/// A day's mean air at sea level on day `day` of the year (from 0, with its fraction).
pub fn mean_air(c_celsius: f32, c_seasons: f32, day: f32) -> f32 {
    c_celsius + temperature::season(c_seasons, calendar::from_midwinter(day))
}

/// Days of rain the ground remembers.
pub const SOIL_DAYS: u64 = 8;

/// How wet a biome's ground is on `day` (days since the world began), 0 dry to 1 soaked: the last
/// `SOIL_DAYS` days' rain (`climate::spell`, a longer spell more), each day's drying faster in
/// warmer air. Worked out from the rain schedule, so nothing is stored. `calendar`: whether the
/// dimension has seasons. Ground by water is soaked whatever the weather (the game's side).
pub fn soil_water(rain: [f32; 4], celsius: f32, seasons: f32, calendar: bool, key: u64, day: u64) -> f32 {
    let mut wet = 0.0;
    let mut keep = 1.0;
    for back in 0..SOIL_DAYS.min(day + 1) {
        let d = day - back;
        let of_year = (d % calendar::YEAR_DAYS as u64) as f32;
        let season = calendar.then_some((of_year as u32 / calendar::SEASON_DAYS) as usize);
        if let Some((from, to)) = climate::spell(rain, season, key, d) {
            wet += keep * ((to - from) / climate::SPELL_HOURS.1).max(0.5);
        }
        // A day keeps 80% of the water at 0 °C, a third at 30 °C.
        let air = if calendar { mean_air(celsius, seasons, of_year + 0.5) } else { celsius };
        keep *= (0.8 - 0.015 * air.max(0.0)).clamp(0.33, 0.8);
    }
    (wet / 1.5).min(1.0)
}

/// A plant's rate there through season `season` (0 spring), on average: the middle day's air and
/// the season's soil, for the sheet and the checks.
pub fn season_rate(p: &Plant, c: &Climate, season: usize) -> f32 {
    let first = season as u64 * calendar::SEASON_DAYS as u64;
    let days = first + calendar::YEAR_DAYS as u64..first + calendar::YEAR_DAYS as u64 + calendar::SEASON_DAYS as u64;
    let n = days.end - days.start;
    days.map(|d| {
        let air = mean_air(c.celsius, c.seasons, (d % calendar::YEAR_DAYS as u64) as f32 + 0.5);
        let soil = if p.water == Water::Wet { 1.0 } else { soil_water(c.rain, c.celsius, c.seasons, true, 7, d) };
        rate(p, air, soil)
    })
    .sum::<f32>()
        / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmth_and_water_shape_the_rate() {
        let wheat = of("base:wheat").unwrap();
        assert_eq!(rate(wheat, 18.0, 0.8), 1.0);
        assert_eq!(rate(wheat, -2.0, 1.0), 0.0, "no growth in frost");
        assert_eq!(rate(wheat, 18.0, 0.0), 0.0, "no growth on dry ground");
        assert!(rate(wheat, 7.5, 1.0) > 0.4 && rate(wheat, 7.5, 1.0) < 0.6);
    }

    #[test]
    fn rain_wets_the_ground_and_heat_dries_it() {
        let wet = climate::of("base:savanna").unwrap();
        let dry = climate::of("base:desert").unwrap();
        let soil = |c: &Climate, day: u64| soil_water(c.rain, c.celsius, c.seasons, true, 3, day);
        // The llanos in their rains (summer) against the desert's dry spring.
        let summer = (calendar::YEAR_DAYS + calendar::SEASON_DAYS + 7) as u64;
        let spring = (calendar::YEAR_DAYS + 7) as u64;
        assert!(soil(wet, summer) > soil(dry, spring) + 0.2);
    }
}

//! Fireflies on warm nights (`notes/NIGHT_GLOW.md`, Research round 17's recommendation 1): how
//! many fly round a player, by the biome's real fireflies, the warmth, the wet ground, the rain
//! and the hour. The server works out the count; the client puts them about as particles.

use crate::{calendar, climate};

/// Each biome's fireflies, from its real place (0 where there are none: taiga, mountains, the
/// sequoia grove, desert, snow, beach, the seas).
pub const WEIGHTS: [(&str, f32); 6] = [
    // The Pantanal: swarms over the wet grass.
    ("base:swamp", 1.0),
    // Konza Prairie: Photinus pyralis.
    ("base:plains", 0.8),
    // Manaus: at gaps and along the water.
    ("base:tropical_forest", 0.8),
    // The llanos, in the rains (the wet ground below does that).
    ("base:savanna", 0.6),
    // Kassel: Lamprohiza and Lampyris at the edges and in meadows.
    ("base:forest", 0.6),
    // Moscow: Lampyris noctiluca, fewer.
    ("base:birch_forest", 0.3),
];

/// The night air they fly in, at least (°C): flashing stops in the cold.
pub const FLY_MIN_C: f32 = 12.0;
/// The day's mean air of a season they come out in, at least (°C): late spring to early autumn
/// in the temperate biomes, the whole year in the tropics.
pub const SEASON_MIN_C: f32 = 16.0;

/// How many round a player at the best place and hour, by `base:fx_fireflies` (off, few, many).
pub const COUNTS: [u32; 3] = [0, 24, 60];
/// How far from the player they are put (blocks).
pub const REACH: i32 = 24;
/// A cluster's size, fewest and most.
pub const CLUSTER: (u32, u32) = (3, 6);
/// Blocks between clusters, at least.
pub const CLUSTER_GAP: f32 = 4.0;
/// How high over the top block (blocks), lowest and highest.
pub const HOVER: (f32, f32) = (0.5, 3.0);
/// The block light where one is put, at most: none by a torch.
pub const TORCH_MAX: u32 = 7;
/// The sky light where one is put, at least: open ground or a forest's edge.
pub const SKY_MIN: u32 = 10;
/// A particle's life (s), shortest and longest.
pub const LIFE_SECS: (f32, f32) = (2.0, 4.0);
/// How fast one drifts (blocks a second), slowest and fastest.
pub const DRIFT: (f32, f32) = (0.05, 0.2);
/// Seconds between the server's counts for each player.
pub const REFRESH_SECS: f32 = 5.0;

/// The biome's weight (0: none there).
pub fn weight(biome: &str) -> f32 {
    WEIGHTS
        .iter()
        .find(|(b, _)| *b == biome)
        .map_or(0.0, |&(_, w)| w)
}

/// The share of the full count at `hour` (0-24) on a day whose sun sets at `sunset`: none before
/// sunset, all from half an hour after to two hours after, 0.3 by midnight, none from 02:00.
pub fn hour_share(hour: f32, sunset: f32) -> f32 {
    // Hours since sunset, the small hours after midnight counted on.
    let after = if hour < 12.0 { hour + 24.0 - sunset } else { hour - sunset };
    let midnight = 24.0 - sunset;
    if after <= 0.0 || after >= midnight + 2.0 {
        0.0
    } else if after < 0.5 {
        after / 0.5
    } else if after <= 2.0 {
        1.0
    } else if after <= midnight {
        // From all at two hours after sunset to 0.3 at midnight (a late sunset: straight to 0.3).
        let span = (midnight - 2.0).max(0.01);
        1.0 - 0.7 * ((after - 2.0) / span).min(1.0)
    } else {
        0.3 * (1.0 - (after - midnight) / 2.0)
    }
}

/// The hour the sun sets on day `day` of the year, where the engine's `.seasons(..)` swing is `swing`.
pub fn sunset(day: f32, swing: f32) -> f32 {
    12.0 + calendar::daylight_hours(day, swing) / 2.0
}

/// The share for the ground's water (`base_api::ground_at`'s soil, 0-1; soaked by water).
pub fn wet_share(soil: f32) -> f32 {
    0.3 + 0.7 * soil.clamp(0.0, 1.0)
}

/// The day's mean air in the open in `c`'s climate on day `day` (°C).
pub fn day_mean(c: &climate::Climate, day: f32) -> f32 {
    (0..24)
        .map(|h| climate::air(c, day, h as f32 + 0.5))
        .sum::<f32>()
        / 24.0
}

/// The share of a setting's full count (`COUNTS`) that fly round a player now, 0 to 1: in
/// `biome`, with the air `air` there, the day's mean `mean`, the soil water `soil`, rain or not,
/// at `hour` on a day whose sun sets at `sunset`. The server works it out; the client multiplies.
pub fn share(biome: &str, air: f32, mean: f32, soil: f32, raining: bool, hour: f32, sunset: f32) -> f32 {
    if raining || air < FLY_MIN_C || mean < SEASON_MIN_C {
        return 0.0;
    }
    weight(biome) * wet_share(soil) * hour_share(hour, sunset)
}

/// How many fly round a player now: `most` for the setting (`COUNTS`) times the `share`.
#[allow(clippy::too_many_arguments)]
pub fn count(most: u32, biome: &str, air: f32, mean: f32, soil: f32, raining: bool, hour: f32, sunset: f32) -> u32 {
    (most as f32 * share(biome, air, mean, soil, raining, hour, sunset)).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every biome with fireflies has a firefly season: at least a sixth of the year (two
    /// months) with a mean air of `SEASON_MIN_C` or more.
    #[test]
    fn every_firefly_biome_has_a_season() {
        for (biome, w) in WEIGHTS {
            assert!(w > 0.0 && w <= 1.0, "{biome}: weight {w}");
            let c = climate::of(biome).unwrap_or_else(|| panic!("{biome}: no climate"));
            let days = calendar::YEAR_DAYS;
            let warm = (0..days)
                .filter(|&d| day_mean(c, d as f32 + 0.5) >= SEASON_MIN_C)
                .count();
            assert!(warm * 6 >= days as usize, "{biome}: {warm} of {days} days warm enough");
        }
    }

    /// The night's curve: none by day, all early in the night, fading after midnight.
    #[test]
    fn the_hours() {
        let set = 20.0;
        assert_eq!(hour_share(12.0, set), 0.0);
        assert_eq!(hour_share(19.9, set), 0.0);
        assert_eq!(hour_share(21.0, set), 1.0);
        assert!((hour_share(0.0, set) - 0.3).abs() < 1e-4);
        assert!(hour_share(1.0, set) > 0.0 && hour_share(1.0, set) < 0.3);
        assert_eq!(hour_share(2.0, set), 0.0);
        assert_eq!(hour_share(6.0, set), 0.0);
        // Never more than the full count.
        for h in 0..96 {
            let s = hour_share(h as f32 / 4.0, set);
            assert!((0.0..=1.0).contains(&s), "{h}: {s}");
        }
    }

    /// The most a setting puts round a player stays its count; rain and cold put none.
    #[test]
    fn the_count() {
        let most = COUNTS[2];
        assert_eq!(count(most, "base:swamp", 25.0, 25.0, 1.0, false, 21.0, 20.0), most);
        assert_eq!(count(most, "base:swamp", 25.0, 25.0, 1.0, true, 21.0, 20.0), 0);
        assert_eq!(count(most, "base:swamp", 10.0, 25.0, 1.0, false, 21.0, 20.0), 0);
        assert_eq!(count(most, "base:taiga", 20.0, 20.0, 1.0, false, 21.0, 20.0), 0);
    }
}

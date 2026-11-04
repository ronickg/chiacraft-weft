//! Wind and fog by place, season and hour (`notes/ECOLOGY.md`): each biome's real place gives its
//! mean wind and its fog days per season (`climate::BIOMES`). A day is calm or windy, and foggy or
//! not, the same on every machine; the hour shapes both. The weather (`mods/base/src/weather.rs`)
//! tells the engine (`set_air`), shader packs draw it.

use crate::climate::{self, Fog};
use crate::{calendar, temperature};

/// How windy a day is against the season's mean: from 0.45 to 1.55 of it, even odds.
pub const DAY_SPREAD: (f32, f32) = (0.45, 1.55);

/// The wind over a day: calmest before dawn, strongest mid afternoon (the sun stirs the air), a
/// quarter either side of the day's mean.
pub const DAY_SWING: f32 = 0.25;
pub const WINDIEST_HOUR: f32 = 15.0;

/// A thunderstorm's gusts: the wind under one, against the day's.
pub const STORM: f32 = 1.8;

/// Fog that forms on still nights clears as the wind gets up, gone from `CLEARS.1` m/s. Sea fog
/// rides the wind and only a gale tears it (`SEA_CLEARS`).
pub const CLEARS: (f32, f32) = (3.5, 7.0);
pub const SEA_CLEARS: (f32, f32) = (9.0, 14.0);

/// The season's mean wind in m/s from a place's four (`Climate::wind`); without seasons, the
/// year's.
pub fn mean_wind(wind: [f32; 4], season: Option<usize>) -> f32 {
    match season {
        Some(s) => wind[s % 4],
        None => wind.iter().sum::<f32>() / 4.0,
    }
}

/// The share of the season's days with fog, from a place's fog days in its four (`Climate::fog`);
/// without seasons, the year's.
pub fn fog_share(fog: [f32; 4], season: Option<usize>) -> f32 {
    let days = |n: f32| (n / 91.25).clamp(0.0, 1.0);
    match season {
        Some(s) => days(fog[s % 4]),
        None => days(fog.iter().sum::<f32>() / 4.0),
    }
}

/// The wind in m/s at `hour` of `day` in the biome `key` stands for (its id and the world's seed),
/// stronger under a thunderstorm.
pub fn wind(mean: [f32; 4], season: Option<usize>, key: u64, day: u64, hour: f32, thunder: bool) -> f32 {
    let r = mix(key ^ day.wrapping_mul(0xA24B_AED4_963E_E407) ^ 0x5749_4E44);
    let daily = DAY_SPREAD.0 + (DAY_SPREAD.1 - DAY_SPREAD.0) * unit(r);
    let swing = 1.0 + DAY_SWING * ((hour - WINDIEST_HOUR) / 24.0 * std::f32::consts::TAU).cos();
    mean_wind(mean, season) * daily * swing * if thunder { STORM } else { 1.0 }
}

/// Where the wind blows to over the whole dimension, in degrees clockwise from north: each day
/// its own way, turning from one day's to the next's through the day.
pub fn heading(seed: u64, day: u64, hour: f32) -> f32 {
    let of = |d: u64| unit(mix(seed ^ d.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x4845_4144)) * 360.0;
    let (a, b) = (of(day), of(day + 1));
    let turn = (b - a + 540.0).rem_euclid(360.0) - 180.0;
    (a + turn * (hour / 24.0).clamp(0.0, 1.0)).rem_euclid(360.0)
}

/// Whether `day` is a foggy day there, and how thick its fog gets (0.55 to 1).
pub fn fog_day(fog: [f32; 4], season: Option<usize>, key: u64, day: u64) -> Option<f32> {
    let r = mix(key ^ day.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ 0x464F_4721);
    (unit(r) < fog_share(fog, season)).then(|| 0.55 + 0.45 * unit(r >> 24))
}

/// How the fog of that kind comes and goes over a day, 0 to 1 (06:00 is dawn on the calendar).
pub fn fog_hours(kind: Fog, hour: f32) -> f32 {
    let ramp = |from: f32, to: f32| ((hour - from) / (to - from)).clamp(0.0, 1.0);
    match kind {
        // Still clear nights cool the ground: it forms before dawn and the sun burns it off by
        // mid morning.
        Fog::Dawn => ramp(1.0, 4.5) * (1.0 - ramp(8.0, 10.5)),
        // Off the sea it comes and stays, thickest in the cool of the morning.
        Fog::Sea => 0.6 + 0.4 * (1.0 - ramp(9.0, 14.0)) * ramp(0.0, 4.0) + 0.4 * ramp(20.0, 24.0),
        // Ice fog of the far north: cold still nights, any hour in the dark of winter.
        Fog::Ice => 1.0 - 0.6 * ramp(10.0, 13.0) * (1.0 - ramp(16.0, 19.0)),
    }
}

/// The fog at `hour` of `day` there, 0 clear to 1 thick, with the wind blowing `wind` m/s: a place's
/// fog days (`Climate::fog`) and their kind.
pub fn fog(days: [f32; 4], kind: Fog, season: Option<usize>, key: u64, day: u64, hour: f32, wind: f32) -> f32 {
    let Some(thick) = fog_day(days, season, key, day) else { return 0.0 };
    let clears = if kind == Fog::Sea { SEA_CLEARS } else { CLEARS };
    let blown = ((wind - clears.0) / (clears.1 - clears.0)).clamp(0.0, 1.0);
    thick * fog_hours(kind, hour) * (1.0 - blown)
}

// ---- snow, blizzards and dust ----

/// Below this air (°C, out in the open, the rain's cooling counted) what falls is snow.
pub const SNOWS_BELOW: f32 = 1.0;

/// Snow lying, 0 bare to 1 deep: a day of snow adds `SNOWFALL`, and each day's mean above
/// freezing melts `MELT` a degree (a degree-day factor: about 4 mm of water, 1.2 cm of fresh
/// snow, a degree a day). Counted over the last `SNOW_DAYS` days.
pub const SNOWFALL: f32 = 0.4;
pub const MELT: f32 = 0.05;
pub const SNOW_DAYS: u64 = 12;

/// Snow in a wind is a blizzard: from `BLIZZARD.0` m/s the snow hides the land, all of it by
/// `BLIZZARD.1`.
pub const BLIZZARD: (f32, f32) = (6.0, 12.0);

/// Dry ground (humidity under `DRY`) lifts dust and sand from `DUST.0` m/s (loose sand's
/// threshold at 10 m), a storm of it by `DUST.1`.
pub const DRY: f32 = 0.35;
pub const DUST: (f32, f32) = (7.0, 11.0);

/// A place's air as the weather needs it: its climate (`Climate`'s numbers) and whether the world
/// has seasons.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub celsius: f32,
    pub humidity: f32,
    pub seasons: f32,
    pub rain: [f32; 4],
    pub calendar: bool,
}

impl Place {
    /// The air in the open at sea level at `hour` of `day` (days since the world began), raining
    /// or not.
    pub fn air(&self, day: u64, hour: f32, raining: bool) -> f32 {
        let year = if self.calendar { calendar::from_midwinter((day % calendar::YEAR_DAYS as u64) as f32 + hour / 24.0) } else { 0.25 };
        temperature::air(&temperature::Place {
            biome_c: self.celsius,
            humidity: self.humidity,
            altitude: 0.0,
            hour,
            open_sky: true,
            raining,
            snowing: false,
            in_water: false,
            fire: 0.0,
            season: temperature::season(self.seasons, year),
            underground: false,
            wind: 0.0,
        })
    }

    fn season(&self, day: u64) -> Option<usize> {
        self.calendar
            .then(|| ((day % calendar::YEAR_DAYS as u64) / calendar::SEASON_DAYS as u64) as usize)
    }

    /// Whether what falls at `hour` of `day` is snow (it rains or snows then, `climate::rains`).
    pub fn snows(&self, key: u64, day: u64, hour: f32) -> bool {
        climate::rains(self.rain, self.season(day), key, day, hour) && self.air(day, hour, true) < SNOWS_BELOW
    }

    /// The snow lying at `hour` of `day`: the last days' snowfalls, melted by the warm days since.
    pub fn snow_lying(&self, key: u64, day: u64, hour: f32) -> f32 {
        let mut snow: f32 = 0.0;
        for d in day.saturating_sub(SNOW_DAYS)..=day {
            let (fell, warmth) = if d == day {
                // Today so far: what has fallen by now, and the melt of the hours gone.
                let fell = climate::spell(self.rain, self.season(d), key, d)
                    .filter(|&(from, to)| self.air(d, (from + to) / 2.0, true) < SNOWS_BELOW)
                    .map_or(0.0, |(from, to)| ((hour - from) / (to - from)).clamp(0.0, 1.0));
                (fell, self.air(d, hour, false).max(0.0) * hour / 24.0)
            } else {
                let fell = climate::spell(self.rain, self.season(d), key, d)
                    .is_some_and(|(from, to)| self.air(d, (from + to) / 2.0, true) < SNOWS_BELOW);
                // The day's mean: before dawn and mid afternoon, halved.
                let mean = (self.air(d, 4.0, false) + self.air(d, 15.0, false)) / 2.0;
                (if fell { 1.0 } else { 0.0 }, mean.max(0.0))
            };
            snow = (snow + fell * SNOWFALL - warmth * MELT).clamp(0.0, 1.0);
        }
        snow
    }
}

/// How thick a blizzard is in falling snow with the wind at `wind` m/s, 0 to 1.
pub fn blizzard(wind: f32) -> f32 {
    ((wind - BLIZZARD.0) / (BLIZZARD.1 - BLIZZARD.0)).clamp(0.0, 1.0)
}

/// How much dust blows on ground of that `humidity`, dry (not raining), at `wind` m/s, 0 to 1.
pub fn dust(humidity: f32, wind: f32, raining: bool) -> f32 {
    if humidity >= DRY || raining {
        return 0.0;
    }
    ((wind - DUST.0) / (DUST.1 - DUST.0)).clamp(0.0, 1.0)
}

/// In thick fog an animal sees this share of what it sees in clear air (proposed: real fog under
/// 1 km is a fog day, the game's thick fog hides the land past about 20 blocks).
pub const FOG_SIGHT: f32 = 0.35;

/// How far one sees in fog `fog` (0..1) against clear air.
pub fn sight(fog: f32) -> f32 {
    1.0 - (1.0 - FOG_SIGHT) * fog.clamp(0.0, 1.0)
}

/// The wind chill (°C) in air `celsius` with the wind at `wind` m/s at 10 m: the North American
/// wind chill index (Environment Canada and the US National Weather Service, 2001), only in the cold
/// (10 °C and under) and a wind over 4.8 km/h; the air itself elsewhere.
pub fn feels(celsius: f32, wind: f32) -> f32 {
    let kmh = wind * 3.6;
    if celsius > 10.0 || kmh <= 4.8 {
        return celsius;
    }
    let v = kmh.powf(0.16);
    (13.12 + 0.6215 * celsius - 11.37 * v + 0.3965 * celsius * v).min(celsius)
}

/// SplitMix64: a hash every machine agrees on.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn unit(r: u64) -> f32 {
    (r & 0xF_FFFF) as f32 / 0x10_0000 as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::BIOMES;

    /// Over many years a biome has fog on about its share of each season's days, and its wind
    /// averages the season's mean.
    #[test]
    fn fog_and_wind_follow_the_places() {
        for c in BIOMES.iter().filter(|c| !c.underground) {
            for s in 0..4 {
                let days = 20_000u64;
                let foggy = (0..days)
                    .filter(|&d| fog_day(c.fog, Some(s), 777, d).is_some())
                    .count();
                let share = foggy as f32 / days as f32;
                assert!((share - fog_share(c.fog, Some(s))).abs() < 0.02, "{} season {s}: fog {share}", c.biome);
                let mean = (0..days)
                    .map(|d| wind(c.wind, Some(s), 777, d, 12.0 + (d % 24) as f32, false))
                    .sum::<f32>()
                    / days as f32;
                assert!((mean - c.wind[s]).abs() < c.wind[s] * 0.05 + 0.05, "{} season {s}: wind {mean} against {}", c.biome, c.wind[s]);
            }
        }
    }

    /// The wind chill index's own table: -10 °C in a 30 km/h wind feels -20.
    #[test]
    fn wind_chill_matches_the_table() {
        assert!((feels(-10.0, 30.0 / 3.6) - -19.5).abs() < 0.6, "{}", feels(-10.0, 30.0 / 3.6));
        assert_eq!(feels(20.0, 10.0), 20.0);
        assert_eq!(feels(-5.0, 1.0), -5.0);
    }

    /// Snow lies in a taiga winter and is gone by its summer.
    #[test]
    fn snow_lies_in_winter_and_melts_in_summer() {
        let c = BIOMES.iter().find(|c| c.biome == "base:taiga").unwrap();
        let p = Place { celsius: c.celsius, humidity: c.humidity, seasons: c.seasons, rain: c.rain, calendar: true };
        let winter = 3 * calendar::SEASON_DAYS as u64 + 7;
        let summer = calendar::SEASON_DAYS as u64 + 7;
        let years = 0..20u64;
        let lying = |day: u64| {
            years
                .clone()
                .map(|y| p.snow_lying(9, day + y * calendar::YEAR_DAYS as u64, 12.0))
                .sum::<f32>()
                / 20.0
        };
        assert!(lying(winter) > 0.2, "winter {}", lying(winter));
        assert!(lying(summer) < 0.01, "summer {}", lying(summer));
    }

    /// Dawn fog is gone by noon and the wind clears it.
    #[test]
    fn dawn_fog_burns_off_and_blows_away() {
        assert_eq!(fog_hours(Fog::Dawn, 12.0), 0.0);
        assert!(fog_hours(Fog::Dawn, 6.0) > 0.99);
        assert!(fog_hours(Fog::Sea, 15.0) >= 0.6);
        let c = BIOMES
            .iter()
            .find(|c| c.fog_kind == Fog::Dawn && c.fog.iter().any(|&f| f > 0.0))
            .unwrap();
        let day = (0..10_000)
            .find(|&d| fog_day(c.fog, Some(2), 5, d).is_some())
            .unwrap();
        assert!(fog(c.fog, c.fog_kind, Some(2), 5, day, 6.0, 1.0) > 0.5);
        assert_eq!(fog(c.fog, c.fog_kind, Some(2), 5, day, 6.0, CLEARS.1 + 1.0), 0.0);
    }
}

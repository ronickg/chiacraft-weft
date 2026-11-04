//! Wind and fog by place, season and hour (`notes/ECOLOGY.md`): each biome's real place gives its
//! mean wind and its fog days per season (`climate::BIOMES`). A day is calm or windy, and foggy or
//! not, the same on every machine; the hour shapes both. The weather (`mods/base/src/weather.rs`)
//! tells the engine (`set_air`), shader packs draw it.

use crate::climate::Fog;

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
                let foggy = (0..days).filter(|&d| fog_day(c.fog, Some(s), 777, d).is_some()).count();
                let share = foggy as f32 / days as f32;
                assert!((share - fog_share(c.fog, Some(s))).abs() < 0.02, "{} season {s}: fog {share}", c.biome);
                let mean = (0..days).map(|d| wind(c.wind, Some(s), 777, d, 12.0 + (d % 24) as f32, false)).sum::<f32>() / days as f32;
                assert!((mean - c.wind[s]).abs() < c.wind[s] * 0.05 + 0.05, "{} season {s}: wind {mean} against {}", c.biome, c.wind[s]);
            }
        }
    }

    /// Dawn fog is gone by noon and the wind clears it.
    #[test]
    fn dawn_fog_burns_off_and_blows_away() {
        assert_eq!(fog_hours(Fog::Dawn, 12.0), 0.0);
        assert!(fog_hours(Fog::Dawn, 6.0) > 0.99);
        assert!(fog_hours(Fog::Sea, 15.0) >= 0.6);
        let c = BIOMES.iter().find(|c| c.fog_kind == Fog::Dawn && c.fog.iter().any(|&f| f > 0.0)).unwrap();
        let day = (0..10_000).find(|&d| fog_day(c.fog, Some(2), 5, d).is_some()).unwrap();
        assert!(fog(c.fog, c.fog_kind, Some(2), 5, day, 6.0, 1.0) > 0.5);
        assert_eq!(fog(c.fog, c.fog_kind, Some(2), 5, day, 6.0, CLEARS.1 + 1.0), 0.0);
    }
}

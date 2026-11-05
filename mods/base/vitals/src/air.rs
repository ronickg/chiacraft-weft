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

/// How long wet ground takes to dry once the rain stops, in seconds of play (`DRYING`, as the
/// engine counts it): seven minutes in mild damp calm air (15 °C, humidity 0.6), twice as fast
/// for every ten degrees warmer, slower in damp air, faster in a wind; from half a minute (hot,
/// dry, windy) to an hour (frozen or close to it). Ronald's pick, 11-05: the ground and leaves
/// stay wet after rain and dry over minutes by the air's warmth and wind.
pub const DRYING: (f32, f32) = (30.0, 3600.0);

pub fn drying_secs(celsius: f32, humidity: f32, wind: f32) -> f32 {
    if celsius <= 0.5 {
        return DRYING.1;
    }
    let warmth = 2.0f32.powf(-(celsius - 15.0) / 10.0);
    let damp = (0.5 + humidity.clamp(0.0, 1.0)) / 1.1;
    let breeze = 1.0 + wind.max(0.0) / 4.0;
    // (Seven minutes: at three, a summer forest's ground was dry again within a minute, PC shw2.)
    (420.0 * warmth * damp / breeze).clamp(DRYING.0, DRYING.1)
}

/// `drying_secs` as the engine's byte (`air_flags::DRYING_SHIFT`): 1 an hour to 255 half a minute,
/// spaced evenly in the logarithm.
pub fn drying_byte(secs: f32) -> u32 {
    let t = (secs.clamp(DRYING.0, DRYING.1) / DRYING.1).ln() / (DRYING.0 / DRYING.1).ln();
    1 + (t * 254.0).round() as u32
}

/// A light mist on calm, clear mornings that are not fog days, by the air's humidity (Ronald's
/// pick, 11-05: no mist at 05:00-06:00 in a clear summer, Tester 2's mist shots, where `/air`'s
/// forced mist looked right): the ground cools through the night and the damp air above it
/// condenses in the last hours before sunrise, lying in valleys and over water (the shader pack
/// lays any fog low); the sun burns it off two to three hours after it rises. Dry air (humidity
/// under `DAWN_MIST_DAMP.0`) none, the dampest `DAWN_MIST` thick. Only on some mornings: each
/// day's, the same on every machine. `sunrise` the hour the sun rises that day.
// (0.1: at 0.42 a damp morning over the birch forest went a golden white-out, at 0.2 it still hid
// most of the land from 30 blocks up, PC shm_mist, shm_mist2.)
pub const DAWN_MIST: f32 = 0.1;
pub const DAWN_MIST_DAMP: (f32, f32) = (0.45, 0.8);
pub const DAWN_MIST_DAYS: f32 = 0.6;

pub fn dawn_mist(humidity: f32, sunrise: f32, hour: f32, wind: f32, raining: bool, key: u64, day: u64) -> f32 {
    if raining {
        return 0.0;
    }
    let damp = ((humidity - DAWN_MIST_DAMP.0) / (DAWN_MIST_DAMP.1 - DAWN_MIST_DAMP.0)).clamp(0.0, 1.0);
    if damp <= 0.0 || unit(mix(key ^ day.wrapping_mul(0x8CB9_2BA7_2F3D_8DD7) ^ 0x4D49_5354)) >= DAWN_MIST_DAYS {
        return 0.0;
    }
    let ramp = |from: f32, to: f32| ((hour - from) / (to - from)).clamp(0.0, 1.0);
    let hours = ramp(sunrise - 3.0, sunrise - 0.5) * (1.0 - ramp(sunrise + 1.5, sunrise + 3.5));
    let blown = ((wind - CLEARS.0) / (CLEARS.1 - CLEARS.0)).clamp(0.0, 1.0);
    DAWN_MIST * damp * hours * (1.0 - blown)
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

/// Snow lying as blocks (`base:snow_layer`, an eighth of a block each), near players on open
/// ground: none under `SNOW_LAYER_FROM` of a cover (the shader pack's dusting shows it, and on
/// far land all of it), one layer from there, `SNOW_LAYERS_DEEP` (half a block, about 6 cm of
/// water) in a deep cover. Over buried plants from `SNOW_BURIES` layers.
pub const SNOW_LAYER_FROM: f32 = 0.2;
pub const SNOW_LAYERS_DEEP: u8 = 4;
pub const SNOW_BURIES: u8 = 2;

/// How the layers follow the cover cheaply: each tick `SNOW_SAMPLES` columns at random within
/// `SNOW_REACH` blocks of each player move a layer toward it (a column every
/// `snow_visit_secs()` on average), and each layer block may step down on its own every
/// `SNOW_MELT_SECS` on average while there is less snow than it, also while nobody is near
/// (caught up when its land loads). A column is looked down `SNOW_SCAN` blocks from above
/// the player.
pub const SNOW_SAMPLES: u32 = 12;
pub const SNOW_REACH: i32 = 40;
pub const SNOW_SCAN: i32 = 48;
pub const SNOW_MELT_SECS: f32 = 20.0;
/// Snow melts with the warmth where it lies (World playtester, 11-04: with the cover gone,
/// layers still lay at dusk; Planner: gone within minutes on a warm day, kept only by a frost).
/// A step down comes only with the air there above freezing, every step at `SNOW_MELT_WARM` °C
/// and warmer (3 layers in about a minute), fewer the colder; at 0 °C and under none (and the
/// sampler near players melts it no faster).
pub const SNOW_MELT_WARM: f32 = 8.0;

/// Still water freezes (Planner, 11-04: tarns, lakes and ponds get a skin of ice and thaw in the
/// warmth, by the snow's rule): near players (the snow layers' sampler) the open top of still
/// water turns to lake ice (`base:lake_ice`) where the air there is under `FREEZE_BELOW` °C, every visit
/// from `FREEZE_HARD` and colder, fewer nearer freezing; it thaws back to water as snow melts
/// (`melt_chance`), on a visit or on its own every `ICE_THAW_SECS` on average (caught up when its
/// land loads).
pub const FREEZE_BELOW: f32 = 0.0;
pub const FREEZE_HARD: f32 = -6.0;
pub const ICE_THAW_SECS: f32 = 30.0;

/// The chance a visit freezes open still water at `celsius` (the air there): 0 from freezing up,
/// 1 from `FREEZE_HARD` down.
pub fn freeze_chance(celsius: f32) -> f32 {
    ((FREEZE_BELOW - celsius) / (FREEZE_BELOW - FREEZE_HARD)).clamp(0.0, 1.0)
}

/// The chance a melt step takes a layer at `celsius` (the air where it lies): 0 in a frost, 1 from
/// `SNOW_MELT_WARM`.
pub fn melt_chance(celsius: f32) -> f32 {
    (celsius / SNOW_MELT_WARM).clamp(0.0, 1.0)
}

/// How many snow layers a cover of `snow` (0..1, `Place::snow_lying`) lies as.
pub fn snow_layers(snow: f32) -> u8 {
    if snow < SNOW_LAYER_FROM {
        return 0;
    }
    let deep = ((snow.min(1.0) - SNOW_LAYER_FROM) / (1.0 - SNOW_LAYER_FROM) * (SNOW_LAYERS_DEEP - 1) as f32).round();
    1 + deep as u8
}

/// Seconds between the samples' visits to one column near a player, on average.
pub fn snow_visit_secs() -> f32 {
    let side = (2 * SNOW_REACH + 1) as f32;
    side * side / SNOW_SAMPLES as f32 / 20.0
}

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

/// A blizzard as fog: how much of the land it hides at its thickest.
pub const BLIZZARD_FOG: f32 = 0.9;

// ---- the air across borders, and in the forest ----

/// A place's air is its own and its neighbours' (Planner, 11-04: crossing a border shouldn't snap
/// the weather or the snow): the climate and the weather at a body's feet, counted twice, and at
/// the 8 points of `AIR_RING` `AIR_BLEND / 2` blocks round it, averaged, so the air turns from
/// one biome's to the next over a band `AIR_BLEND` blocks wide.
pub const AIR_BLEND: i32 = 32;
pub const AIR_RING: [(f32, f32); 8] =
    [(1.0, 0.0), (0.707, 0.707), (0.0, 1.0), (-0.707, 0.707), (-1.0, 0.0), (-0.707, -0.707), (0.0, -1.0), (0.707, -0.707)];

/// The 9 points of a place's air round (`x`, `z`) `radius` blocks out: the middle first.
pub fn ring(x: i32, z: i32, radius: i32) -> [(i32, i32); 9] {
    let mut out = [(x, z); 9];
    for (o, (dx, dz)) in out[1..].iter_mut().zip(AIR_RING) {
        *o = (x + (dx * radius as f32).round() as i32, z + (dz * radius as f32).round() as i32);
    }
    out
}

/// The average of `values` at the points of `ring` (the middle counted twice).
pub fn blend(values: [f32; 9]) -> f32 {
    (values[0] * 2.0 + values[1..].iter().sum::<f32>()) / 10.0
}

/// Inside a forest the wind drops: under a closed canopy, a tree height or two in from the edge,
/// to about a quarter of the open wind at a body's height (`FOREST_CALM` of it taken; forest
/// interiors keep some 10-40 % of the open wind near the ground, Raupach and Finnigan's canopy
/// flows), and with it the wind chill and the blizzard. How closed the canopy is comes from the
/// biome (`CANOPY`, full shelter from `CANOPY_SHELTERS`), how far in from how many of the 8
/// points `FOREST_EDGE` blocks round have leaves over them (`cover` 0..1): from `EDGE_COVER.0`
/// some shelter, all of it from `EDGE_COVER.1`. At the edge half the ring is open: no shelter.
pub const FOREST_CALM: f32 = 0.75;
pub const CANOPY_SHELTERS: f32 = 0.4;
pub const FOREST_EDGE: i32 = 12;
pub const EDGE_COVER: (f32, f32) = (0.5, 0.9);

/// How sheltered from the wind a place is, 0 open to 1 deep in a closed forest: its canopy
/// (`CANOPY`'s share, blended) and how much of the ring round it is under leaves.
pub fn shelter(canopy: f32, cover: f32) -> f32 {
    let closed = (canopy / CANOPY_SHELTERS).clamp(0.0, 1.0);
    let inside = ((cover - EDGE_COVER.0) / (EDGE_COVER.1 - EDGE_COVER.0)).clamp(0.0, 1.0);
    closed * inside
}

/// The wind `wind` m/s at a place that `shelter` (0..1) shelters.
pub fn sheltered_wind(wind: f32, shelter: f32) -> f32 {
    wind * (1.0 - FOREST_CALM * shelter.clamp(0.0, 1.0))
}

/// The fog `fog` (0..1, the weather's) where the wind drops from `wind` to `calm` m/s: in falling
/// snow the blizzard's part of it drops with the wind.
pub fn sheltered_fog(fog: f32, wind: f32, calm: f32, snowing: bool) -> f32 {
    if !snowing {
        return fog;
    }
    (fog - (blizzard(wind) - blizzard(calm)) * BLIZZARD_FOG).max(0.0)
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

    /// Drying: faster warm, dry and windy, an hour when it freezes; the byte runs 1 (an hour) to 255
    /// (half a minute) as the engine reads it.
    #[test]
    fn drying_by_warmth_damp_and_wind() {
        let mild = drying_secs(15.0, 0.6, 0.0);
        assert!((mild - 420.0).abs() < 1.0, "{mild}");
        assert!(drying_secs(30.0, 0.6, 0.0) < mild && drying_secs(15.0, 0.9, 0.0) > mild && drying_secs(15.0, 0.6, 8.0) < mild);
        assert_eq!(drying_secs(-5.0, 0.6, 0.0), DRYING.1);
        assert_eq!(drying_byte(DRYING.1), 1);
        assert_eq!(drying_byte(DRYING.0), 255);
    }

    /// Dawn mist: none in dry air, in rain, in a wind or at noon; some at sunrise in damp calm air
    /// on a misty morning, and never thicker than `DAWN_MIST`.
    #[test]
    fn dawn_mist_by_air_and_hour() {
        let misty = (0..64)
            .find(|&d| dawn_mist(1.0, 6.0, 6.0, 0.0, false, 7, d) > 0.0)
            .expect("a misty morning in 64 days");
        let m = dawn_mist(1.0, 6.0, 6.0, 0.0, false, 7, misty);
        assert!(m > 0.6 * DAWN_MIST && m <= DAWN_MIST, "{m}");
        assert_eq!(dawn_mist(0.3, 6.0, 6.0, 0.0, false, 7, misty), 0.0);
        assert_eq!(dawn_mist(1.0, 6.0, 6.0, 0.0, true, 7, misty), 0.0);
        assert_eq!(dawn_mist(1.0, 6.0, 6.0, CLEARS.1, false, 7, misty), 0.0);
        assert_eq!(dawn_mist(1.0, 6.0, 12.0, 0.0, false, 7, misty), 0.0);
        assert!(dawn_mist(1.0, 4.0, 4.0, 0.0, false, 7, misty) > 0.6 * DAWN_MIST, "it follows the season's sunrise");
    }

    /// Every biome the per-biome shares name (aurorae, canopies) has a climate row, so a renamed
    /// biome can't silently lose its aurora or its light under the leaves.
    #[test]
    fn shares_name_real_biomes() {
        for (name, share) in AURORA.iter().chain(CANOPY.iter()) {
            assert!(crate::climate::of(name).is_some(), "{name} has no climate row");
            assert!((0.0..=1.0).contains(share), "{name}: share {share}");
        }
    }

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

    /// Snow layers follow the cover: none for a dusting, one from `SNOW_LAYER_FROM`, half a
    /// block when deep, never more as it thins; a deep cover builds within a few minutes near
    /// a player, and a layer block melts within minutes too.
    #[test]
    fn snow_layers_follow_the_cover() {
        assert_eq!(snow_layers(0.0), 0);
        assert_eq!(snow_layers(SNOW_LAYER_FROM - 0.01), 0);
        assert_eq!(snow_layers(SNOW_LAYER_FROM), 1);
        assert_eq!(snow_layers(1.0), SNOW_LAYERS_DEEP);
        assert_eq!(snow_layers(2.0), SNOW_LAYERS_DEEP);
        let mut last = 0;
        for i in 0..=100 {
            let n = snow_layers(i as f32 / 100.0);
            assert!(n >= last && n <= 8, "{i}: {n}");
            last = n;
        }
        // A layer a visit: deep in under five minutes, gone in under five.
        assert!(snow_visit_secs() * SNOW_LAYERS_DEEP as f32 <= 300.0, "{}", snow_visit_secs());
        assert!(SNOW_MELT_SECS * SNOW_LAYERS_DEEP as f32 <= 300.0);
        // The warmth: none in a frost, all from a mild spring day, half at half of it; 3 layers
        // on a warm day in under 2 minutes on average.
        assert_eq!(melt_chance(-5.0), 0.0);
        assert_eq!(melt_chance(0.0), 0.0);
        assert_eq!(melt_chance(SNOW_MELT_WARM * 2.0), 1.0);
        assert!((melt_chance(SNOW_MELT_WARM / 2.0) - 0.5).abs() < 1e-6);
        assert!(SNOW_MELT_SECS * 3.0 / melt_chance(12.0) <= 120.0);
        // Ice: none forms at or over freezing, all of it in a hard frost; it never forms and
        // thaws at the same warmth.
        assert_eq!(freeze_chance(2.0), 0.0);
        assert_eq!(freeze_chance(FREEZE_BELOW), 0.0);
        assert_eq!(freeze_chance(FREEZE_HARD - 1.0), 1.0);
        for c in -20..=20 {
            let c = c as f32;
            assert!(freeze_chance(c) == 0.0 || melt_chance(c) == 0.0, "{c} °C both freezes and thaws");
        }
        // The scan reaches the ground below a player on a hill and above one in a valley; plants
        // are buried by a cover that has layers.
        const { assert!(SNOW_SCAN >= 32 && SNOW_BURIES >= 1 && SNOW_BURIES <= 8) };
    }

    /// Crossing a border the air turns over the band: a place half in each biome gets half of
    /// each. Deep in a closed forest the wind is a quarter of the open's and a blizzard thins;
    /// at the edge (half the ring open) and in open country nothing changes.
    #[test]
    fn air_blends_and_forests_calm_the_wind() {
        let r = ring(0, 0, AIR_BLEND / 2);
        assert_eq!(r[0], (0, 0));
        assert!(
            r[1..]
                .iter()
                .all(|&(x, z)| ((x * x + z * z) as f32).sqrt().round() as i32 == AIR_BLEND / 2)
        );
        // East of the border (x > 0) 10 °C, west 0: on the border, about half.
        let half: [f32; 9] = r.map(|(x, _)| if x >= 0 { 10.0 } else { 0.0 });
        assert!((4.0..=7.0).contains(&blend(half)), "{}", blend(half));
        assert_eq!(blend([3.0; 9]), 3.0);
        // The forest.
        assert_eq!(shelter(1.0, 0.5), 0.0, "the edge is open");
        assert_eq!(shelter(0.0, 1.0), 0.0, "no canopy, no shelter");
        assert_eq!(shelter(1.0, 1.0), 1.0);
        assert!((sheltered_wind(8.0, 1.0) - 2.0).abs() < 1e-4);
        assert_eq!(sheltered_wind(8.0, 0.0), 8.0);
        for &(name, share) in &CANOPY {
            let deep = sheltered_wind(10.0, shelter(share, 1.0));
            assert!((2.5..=5.0).contains(&deep), "{name}: {deep} m/s of 10 deep inside");
        }
        // A blizzard in 12 m/s, deep in a forest: the fog thins with the wind; no snow, no change.
        let fog = blizzard(12.0) * BLIZZARD_FOG;
        let calm = sheltered_wind(12.0, 1.0);
        assert!(sheltered_fog(fog, 12.0, calm, true) < 0.2, "{}", sheltered_fog(fog, 12.0, calm, true));
        assert_eq!(sheltered_fog(0.4, 12.0, calm, false), 0.4);
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

/// Aurorae on clear nights, by the real place's latitude: the auroral oval lies at about 60-75°
/// north, so the biomes whose place is under it get them (the taiga, Fairbanks at 64.8°, one of the
/// best aurora places on Earth; the snowy plains, Utqiaġvik at 71.3°), the birch forest (Moscow,
/// 55.8°) a rare faint one. How strong, 0..1 (`reports/sky-weather-life.md`).
pub const AURORA: [(&str, f32); 3] = [("base:taiga", 1.0), ("base:snowy_plains", 1.0), ("base:birch_forest", 0.2)];

/// How closed each forest's canopy is, 0..1, for the light under the leaves (the shader pack's
/// green shade and sunflecks): a rainforest's lets about 1-2 % of the sun reach the floor, much of
/// it as brief sunflecks (the central Amazon, `images/vegetation/2026-11-04/tropical_refs/real/NOTES.md`);
/// a temperate broadleaf wood in leaf 3-10 %, a conifer or birch wood more, and an open grove or a
/// swamp's scattered trees more again (Lighting's estimates from those shares). Biomes not listed
/// have open sky.
pub const CANOPY: [(&str, f32); 6] = [
    ("base:tropical_forest", 1.0),
    ("base:sequoia_grove", 0.6),
    ("base:forest", 0.5),
    ("base:taiga", 0.4),
    ("base:birch_forest", 0.35),
    ("base:swamp", 0.3),
];

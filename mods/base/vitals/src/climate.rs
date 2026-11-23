//! Every biome's climate in one table, the world's and other mods' (the Cretaceous's), each from
//! a real place of its kind (`reports/ecology/Biome climates.md`, its numbers in
//! `biome_climates.json`): the year's mean, how much the seasons and the day swing it, and how
//! often it rains in each season. The mods set these on their biomes (`base_api::climate`); the
//! checks across systems read them here (`mods/animals/core/tests/places.rs`): whether each
//! species is at home in its habitats and its eggs hatch there.

use crate::calendar;
use crate::temperature::{self, Place};

/// A biome's climate: the year's mean temperature in °C, humidity 0 dry to 1 wet (dry air swings
/// more between day and night, `temperature::air`), half the range between summer and winter, and
/// the share of days with rain in each season of the calendar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Its full name (`base:plains`).
    pub biome: &'static str,
    /// The real place its numbers come from.
    pub place: &'static str,
    pub celsius: f32,
    pub humidity: f32,
    pub seasons: f32,
    /// Days with rain (1 mm or more) in spring, summer, autumn and winter, as shares of the
    /// season's days (`rains`): the place's months three to a season, a southern place's six
    /// months on so its summer is the calendar's.
    pub rain: [f32; 4],
    /// Deep under ground: the year's mean, no day, sun or weather.
    pub underground: bool,
    /// The mean wind in m/s (at 10 m) in spring, summer, autumn and winter (`air::wind`).
    pub wind: [f32; 4],
    /// Days with fog (seeing under 1 km) in spring, summer, autumn and winter, of a real season's
    /// 91 (`air::fog`), and the kind of fog it is.
    pub fog: [f32; 4],
    pub fog_kind: Fog,
}

/// When in the day a place's fog lies (`air::fog_hours`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fog {
    /// Still, clear nights: it forms before dawn and burns off by mid morning.
    Dawn,
    /// Off the sea: any hour, thickest in the morning, and the wind does not clear it.
    Sea,
    /// Ice fog of the far north: cold still nights.
    Ice,
}

impl Fog {
    pub const fn index(self) -> u8 {
        self as u8
    }

    pub const fn from_index(i: u8) -> Fog {
        match i {
            1 => Fog::Sea,
            2 => Fog::Ice,
            _ => Fog::Dawn,
        }
    }
}

const fn c(biome: &'static str, place: &'static str, celsius: f32, humidity: f32, seasons: f32, rain: [f32; 4]) -> Climate {
    Climate { biome, place, celsius, humidity, seasons, rain, underground: false, wind: [0.0; 4], fog: [0.0; 4], fog_kind: Fog::Dawn }
}

impl Climate {
    /// Its wind and fog (`wind`, `fog`, `fog_kind`).
    const fn air(self, wind: [f32; 4], fog: [f32; 4], fog_kind: Fog) -> Climate {
        Climate { wind, fog, fog_kind, ..self }
    }
}

// Humidity is fitted to each place's day range (`temperature::air` swings 2.5 + 6.5 · (1 − humidity)
// a side). Rain days counted from 0.25 mm (NOAA) count a fifth fewer from 1 mm, and from 0.1 mm
// (Mexico, and Kassel's and Moscow's sources) three tenths fewer, the usual gap between the
// thresholds. The mountains' and crags' means are at sea level: the air takes 6.5 °C a km off
// with the height (`ALTITUDE` in `base_api`), so a mountain's slopes are Davos's.
// Wind and fog (`air`): each place's seasonal mean wind at 10 m and its days with fog, from airport
// records, the weather services and the Cretaceous places' modern stand-ins
// (`reports/ecology/Wind and fog.md`, with which are estimates).
pub const BIOMES: [Climate; 22] = [
    // The overworld (`mods/base/src/worldgen.rs`).
    c("base:plains", "Manhattan, Kansas (Konza Prairie)", 12.8, 0.42, 14.2, [0.27, 0.27, 0.2, 0.14]).air(
        [3.8, 3.0, 3.0, 3.1],
        [5.3, 5.8, 6.3, 8.4],
        Fog::Dawn,
    ),
    c("base:forest", "Kassel, Germany", 9.3, 0.77, 8.7, [0.34, 0.34, 0.37, 0.41]).air(
        [2.5, 2.0, 2.4, 3.0],
        [5.8, 4.9, 16.1, 11.3],
        Fog::Dawn,
    ),
    // Old-growth forest (Ronald, 11-05: the Bliss shots' forest): Białowieża's strict reserve, the
    // lowland oak-hornbeam and mixed old growth, cooler and more continental than Kassel's. Its
    // numbers are Białystok's 1991-2020 normals, 60 km off on the same plain: 7.7 °C, January
    // -2.9, July 18.4, about 106 days of rain a year, a little more in summer. Humidity fitted to
    // its 8.9 °C day range; wind and fog are estimates for the sheltered forest (no station
    // record): a light wind, dawn fog in autumn and winter along the river valleys.
    c("base:old_growth_forest", "Białowieża Forest, Poland", 7.7, 0.7, 10.7, [0.28, 0.3, 0.29, 0.29]).air(
        [2.8, 2.2, 2.4, 2.9],
        [6.0, 5.0, 16.0, 14.0],
        Fog::Dawn,
    ),
    c("base:birch_forest", "Moscow", 6.3, 0.8, 12.9, [0.29, 0.3, 0.32, 0.47]).air([1.5, 1.1, 1.4, 1.6], [1.5, 2.3, 3.0, 1.6], Fog::Dawn),
    c("base:taiga", "Fairbanks, Alaska", -2.1, 0.51, 19.8, [0.14, 0.32, 0.26, 0.22]).air(
        [2.3, 2.0, 1.5, 1.1],
        [2.0, 2.7, 7.3, 7.8],
        Fog::Ice,
    ),
    c("base:snowy_plains", "Utqiaġvik, Alaska", -10.2, 0.92, 14.9, [0.14, 0.24, 0.32, 0.15]).air(
        [5.3, 5.3, 6.1, 6.0],
        [26.9, 46.4, 29.1, 17.9],
        Fog::Sea,
    ),
    c("base:desert", "Hermosillo, Sonora", 25.7, 0.18, 8.1, [0.01, 0.13, 0.06, 0.05]).air(
        [2.3, 2.6, 1.6, 1.4],
        [0.2, 0.2, 0.2, 0.3],
        Fog::Dawn,
    ),
    c("base:savanna", "Arauca, the Colombian llanos", 27.0, 0.65, 1.6, [0.4, 0.7, 0.45, 0.11]).air(
        [1.5, 1.2, 1.2, 1.6],
        [1.2, 2.7, 1.4, 0.1],
        Fog::Dawn,
    ),
    c("base:swamp", "Nhumirim, the Pantanal", 25.1, 0.42, 3.4, [0.18, 0.33, 0.2, 0.07]).air(
        [3.5, 2.9, 2.9, 3.6],
        [3.0, 3.5, 5.5, 5.0],
        Fog::Dawn,
    ),
    // The tropical forest (Terrain's, for the hot, wet corner; Ronald 11-04): Manaus, the central
    // Amazon (INMET 1991-2020), hot and wet all year, the rains December to May and easing July to
    // October. Southern, so its months are shifted six: its autumn (March-May, 52 rain days) just
    // out-rains its summer (December-February, 51.6). Wind and fog are estimates for the forest's
    // still air: a light wind, dawn mist in the drier months.
    c("base:tropical_forest", "Manaus, the central Amazon", 27.4, 0.75, 1.0, [0.27, 0.56, 0.57, 0.25]).air(
        [1.8, 1.5, 1.4, 1.7],
        [3.0, 1.5, 1.5, 4.0],
        Fog::Dawn,
    ),
    c("base:mountains", "Davos, 1,594 m", 6.0, 0.63, 8.8, [0.32, 0.45, 0.31, 0.29]).air(
        [2.4, 2.7, 2.2, 1.7],
        [1.5, 1.0, 2.5, 2.5],
        Fog::Dawn,
    ),
    // Giant sequoia groves (Ronald, 11-04): the Sierra Nevada's west slope, cool snowy winters and
    // dry summers. Wind and fog are estimates for a sheltered montane forest (no station record).
    c("base:sequoia_grove", "Giant Forest, Sequoia National Park, 1,721 m", 10.4, 0.49, 8.4, [0.2, 0.05, 0.1, 0.24]).air(
        [2.0, 1.8, 1.6, 1.8],
        [3.0, 1.0, 2.0, 4.0],
        Fog::Dawn,
    ),
    c("base:ocean", "Ponta Delgada, the Azores", 18.2, 0.97, 4.1, [0.33, 0.19, 0.38, 0.43]).air(
        [3.1, 2.5, 2.7, 3.4],
        [1.4, 0.7, 0.3, 0.4],
        Fog::Sea,
    ),
    // Warm lagoons and coral reefs (Terrain's, Ronald 11-04): Key West on the Florida Reef Tract
    // (NOAA 1991-2020), the reef and its lagoon on one station. The sea's heat keeps the days and
    // nights within a few degrees (humidity from the 5.4 °C day range); rain days from 0.25 mm,
    // taken a fifth fewer as from 1 mm; the wet season June to October. Trade winds steady
    // through the dry season; sea fog is rare (estimates).
    c("base:coral_reef", "Key West, the Florida Reef Tract", 26.1, 0.97, 4.2, [0.15, 0.33, 0.3, 0.16]).air(
        [4.8, 4.0, 4.3, 4.8],
        [0.5, 0.2, 0.3, 0.8],
        Fog::Sea,
    ),
    c("base:lagoon", "Key West, the Florida Keys' back-reef lagoon", 26.1, 0.97, 4.2, [0.15, 0.33, 0.3, 0.16]).air(
        [4.5, 3.8, 4.0, 4.5],
        [0.5, 0.2, 0.3, 0.8],
        Fog::Sea,
    ),
    c("base:beach", "Faro, Portugal", 18.2, 0.74, 6.2, [0.16, 0.01, 0.16, 0.2]).air([4.1, 3.7, 3.4, 3.8], [1.0, 1.6, 2.1, 2.3], Fog::Dawn),
    Climate { underground: true, ..c("base:deep_caves", "any cave: the surface's year mean", 12.0, 0.95, 0.0, [0.0; 4]) },
    // The Cretaceous (`mods/animals/src/cretaceous.rs`), estimated from the formations' rocks
    // (`reports/ecology/Cretaceous food webs.md`; Ronald's choices in `notes/ECOLOGY.md`).
    c("animals:red_dunes", "the Djadokhta Formation, Mongolia", 17.0, 0.2, 12.0, [0.1, 0.02, 0.03, 0.12]).air(
        [4.5, 3.5, 3.3, 2.8],
        [0.3, 0.3, 0.3, 0.3],
        Fog::Dawn,
    ),
    c("animals:scrubland", "the Barun Goyot Formation, Mongolia", 17.0, 0.3, 11.0, [0.06, 0.25, 0.08, 0.04]).air(
        [3.8, 3.2, 2.9, 2.5],
        [0.5, 1.0, 0.5, 0.0],
        Fog::Dawn,
    ),
    c("animals:floodplain", "the Cedar Mountain Formation, Utah", 21.0, 0.5, 8.0, [0.08, 0.45, 0.25, 0.04]).air(
        [2.5, 2.5, 2.0, 1.8],
        [2.0, 1.0, 2.0, 5.0],
        Fog::Dawn,
    ),
    c("animals:fern_forest", "the Yixian Formation, Liaoning", 10.0, 0.8, 10.0, [0.25, 0.45, 0.3, 0.15]).air(
        [2.8, 2.0, 1.8, 1.8],
        [2.0, 4.0, 6.0, 2.0],
        Fog::Dawn,
    ),
    c("animals:crags", "the Sevier highlands, Utah", 15.0, 0.6, 8.0, [0.25, 0.4, 0.25, 0.15]).air(
        [4.5, 3.7, 3.7, 3.7],
        [5.0, 1.0, 3.0, 6.0],
        Fog::Dawn,
    ),
];

/// The climate of a biome by its full name.
pub fn of(biome: &str) -> Option<&'static Climate> {
    BIOMES.iter().find(|c| c.biome == biome)
}

/// The air at sea level under an open sky in dry weather, on day `day` of the year (from 0, with
/// its fraction) at `hour`: what `base_api::air_at` finds out there with nothing else around.
pub fn air(c: &Climate, day: f32, hour: f32) -> f32 {
    temperature::air(&Place {
        biome_c: c.celsius,
        humidity: c.humidity,
        altitude: 0.0,
        hour,
        open_sky: true,
        raining: false,
        snowing: false,
        in_water: false,
        fire: 0.0,
        season: temperature::season(c.seasons, calendar::from_midwinter(day)),
        underground: c.underground,
        wind: 0.0,
    })
}

/// The share of days with rain in season `season` (0 spring); a dimension without seasons has
/// the year's.
pub fn rain_share(rain: [f32; 4], season: Option<usize>) -> f32 {
    match season {
        Some(s) => rain[s % 4],
        None => rain.iter().sum::<f32>() / 4.0,
    }
}

/// A season is a place's dry season when it rains on at most this share of the days its wettest
/// season does: the llanos' winter (11 % of days against the summer's 70 %), the Pantanal's
/// (7 % against 33 %), the Cedar Mountain floodplain's winter and spring, a Mediterranean summer;
/// not a prairie winter (14 % against 27 %) or anywhere it rains much the same all year. Herds
/// leave a place in its dry season for wetter ground (`animals_core::groups`, migration).
pub const DRY_OF_WETTEST: f32 = 0.4;

/// Whether `season` (0 spring) is a dry season for a place with `rain` (`DRY_OF_WETTEST`); never
/// in a dimension without seasons, or where it never rains.
pub fn dry_season(rain: [f32; 4], season: Option<usize>) -> bool {
    let wettest = rain.iter().copied().fold(0.0f32, f32::max);
    season.is_some_and(|s| wettest > 0.0 && rain[s % 4] <= wettest * DRY_OF_WETTEST)
}

/// Waterholes shrink in the dry season (`notes/ECOLOGY.md`, "Later"; `base`'s `waterholes.rs`):
/// a pond's still water, a block deep, dries from its edges into a crust of cracked mud in its
/// biome's dry season (`dry_season`), and fills back once the season is over. The llanos'
/// lagoons and the Pantanal's bays shrink to pools by the end of the dry months, and the herds
/// crowd round what is left (Mourão et al. 2000 on the Pantanal's dry-season waterholes). Never
/// the sea's (these biomes' water is the sea, whatever their seasons).
pub const NEVER_DRIES: [&str; 4] = ["base:ocean", "base:beach", "base:coral_reef", "base:lagoon"];
/// An edge block of a waterhole dries in this many game days on average in the dry season (on
/// visits near players, `air::snow_visit_secs`): a pond seven across shrinks to its last pool in
/// about a week of the season's two.
pub const SHRINK_DAYS: f32 = 2.0;
/// A dried block fills back in this many seconds on average once its season is no longer dry,
/// also away from players (a growth step: caught up when its land loads).
pub const REFILL_SECS: f32 = 600.0;

/// Whether still water in `biome` (rain `rain`) shrinks now, in `season` (0 spring; None: no
/// seasons there).
pub fn waterholes_shrink(biome: &str, rain: [f32; 4], season: Option<usize>) -> bool {
    !NEVER_DRIES.contains(&biome) && dry_season(rain, season)
}

/// The chance a visit (every `visit_secs` on average) dries an edge block (`SHRINK_DAYS`).
pub fn shrink_chance(visit_secs: f32) -> f32 {
    (visit_secs / (SHRINK_DAYS * crate::DAY_SECS)).clamp(0.0, 1.0)
}

/// Whether a block of water with `water` of its four sides water and `land` of them dry ground
/// is an edge that dries: on the shore, and still part of the pool (two sides or more water), so
/// a pond shrinks from its rim to a last pool of a block or two, which stays.
pub fn edge_dries(water: u8, land: u8) -> bool {
    land >= 1 && water >= 2
}

/// A rain spell's hours: a rainy day's rain falls in one spell of 2 to 8 hours, as most of a real
/// rainy day's does.
pub const SPELL_HOURS: (f32, f32) = (2.0, 8.0);

/// When it rains on `day` (days since the world began) in the biome `key` stands for (its id and
/// the world's seed): `None` on a dry day; on a rainy day, the spell's start and end hour. A day
/// is rainy with the season's share, the same on every machine.
pub fn spell(rain: [f32; 4], season: Option<usize>, key: u64, day: u64) -> Option<(f32, f32)> {
    let r = mix(key ^ day.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    if unit(r) >= rain_share(rain, season) {
        return None;
    }
    let length = SPELL_HOURS.0 + (SPELL_HOURS.1 - SPELL_HOURS.0) * unit(r >> 20);
    let start = (24.0 - length) * unit(r >> 40);
    Some((start, start + length))
}

/// Whether it rains at `hour` of `day` there (`spell`).
pub fn rains(rain: [f32; 4], season: Option<usize>, key: u64, day: u64, hour: f32) -> bool {
    spell(rain, season, key, day).is_some_and(|(from, to)| (from..to).contains(&hour))
}

/// Whether a rain spell that day thunders, in a biome of that mean and seasons (`Climate`): a third
/// of the spells where the season's mean air is warm (over 15 °C), as summer storms are.
pub fn thunders(celsius: f32, seasons: f32, season: Option<usize>, key: u64, day: u64) -> bool {
    // The season's middle day; without seasons, a spring day.
    let middle = (season.unwrap_or(0) as u32 * calendar::SEASON_DAYS + calendar::SEASON_DAYS / 2) as f32;
    let mean = celsius + temperature::season(seasons, calendar::from_midwinter(middle));
    mean > 15.0 && mix(key ^ day ^ 0x7448_554E).is_multiple_of(3)
}

/// SplitMix64: a hash every machine agrees on.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 0..1 from the low 20 bits.
fn unit(r: u64) -> f32 {
    (r & 0xF_FFFF) as f32 / 0x10_0000 as f32
}

#[cfg(test)]
mod tests {
    #[test]
    fn waterholes_shrink_to_a_last_pool() {
        let rain = |b: &str| of(b).unwrap().rain;
        assert!(waterholes_shrink("base:savanna", rain("base:savanna"), Some(3)));
        assert!(!waterholes_shrink("base:savanna", rain("base:savanna"), Some(1)));
        // A beach's dry summer leaves the sea alone.
        assert!(dry_season(rain("base:beach"), Some(1)) && !waterholes_shrink("base:beach", rain("base:beach"), Some(1)));
        // Rim blocks dry; a pool's last block or the tip of a strip doesn't.
        assert!(edge_dries(3, 1) && edge_dries(2, 2));
        assert!(!edge_dries(1, 3) && !edge_dries(0, 4) && !edge_dries(4, 0));
        // An edge block dries in about SHRINK_DAYS of visits.
        let v = crate::air::snow_visit_secs();
        let days = v / shrink_chance(v) / crate::DAY_SECS;
        assert!((days - SHRINK_DAYS).abs() < 0.01);
    }

    #[test]
    fn dry_seasons_are_the_places_own() {
        let rain = |b: &str| of(b).unwrap().rain;
        // The llanos and the Pantanal are dry in winter, wet in summer.
        assert!(dry_season(rain("base:savanna"), Some(3)) && !dry_season(rain("base:savanna"), Some(1)));
        assert!(dry_season(rain("base:swamp"), Some(3)));
        assert!(dry_season(rain("animals:floodplain"), Some(0)));
        // A prairie winter and a German forest's year aren't.
        assert!(!dry_season(rain("base:plains"), Some(3)));
        assert!((0..4).all(|s| !dry_season(rain("base:forest"), Some(s))));
        // No seasons, no rain: no dry season.
        assert!(!dry_season(rain("base:savanna"), None));
        assert!(!dry_season([0.0; 4], Some(1)));
    }

    use super::*;

    /// Over many years a biome rains on about its share of each season's days.
    #[test]
    fn rain_follows_the_shares() {
        for c in &BIOMES {
            for s in 0..4 {
                let days = 20_000;
                let wet = (0..days)
                    .filter(|&d| spell(c.rain, Some(s), 12345, d).is_some())
                    .count();
                let share = wet as f32 / days as f32;
                assert!((share - c.rain[s]).abs() < 0.02, "{} season {s}: {share} against {}", c.biome, c.rain[s]);
            }
        }
    }
}

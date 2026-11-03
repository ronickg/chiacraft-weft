//! Every biome's climate in one table, the world's and other mods' (the Cretaceous's), each as a
//! real place of its kind has it. The mods set these on their biomes (`base_api::climate`); the
//! checks across systems read them here (`mods/animals/core/tests/places.rs`): whether each
//! species is at home in its habitats and its eggs hatch there.

use crate::calendar;
use crate::temperature::{self, Place};

/// A biome's climate: the year's mean temperature in °C, humidity 0 dry to 1 wet (dry air swings
/// more between day and night), and half the range between summer and winter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Its full name (`base:plains`).
    pub biome: &'static str,
    pub celsius: f32,
    pub humidity: f32,
    pub seasons: f32,
    /// Deep under ground: the year's mean, no day, sun or weather.
    pub underground: bool,
}

const fn c(biome: &'static str, celsius: f32, humidity: f32, seasons: f32) -> Climate {
    Climate { biome, celsius, humidity, seasons, underground: false }
}

pub const BIOMES: [Climate; 17] = [
    // The overworld (`mods/base/src/worldgen.rs`).
    c("base:plains", 12.0, 0.5, 12.0), // a prairie
    c("base:forest", 11.0, 0.65, 10.0),
    c("base:birch_forest", 9.0, 0.6, 12.0),
    c("base:taiga", 0.0, 0.55, 18.0),
    c("base:snowy_plains", -10.0, 0.4, 14.0), // tundra
    c("base:desert", 25.0, 0.1, 9.0),
    c("base:savanna", 26.0, 0.3, 4.0),
    c("base:swamp", 19.0, 0.9, 8.0),
    c("base:mountains", 6.0, 0.5, 12.0),
    c("base:ocean", 14.0, 0.8, 6.0),
    c("base:beach", 17.0, 0.7, 8.0),
    Climate { underground: true, ..c("base:deep_caves", 12.0, 0.6, 0.0) }, // keeps the year's mean
    // The Cretaceous (`mods/animals/src/cretaceous.rs`), a wet and a dry season.
    c("animals:red_dunes", 28.0, 0.1, 6.0), // the Djadokhta Formation, Mongolia
    c("animals:scrubland", 26.0, 0.25, 5.0),
    c("animals:floodplain", 24.0, 0.8, 4.0), // the Cedar Mountain Formation, Utah
    c("animals:fern_forest", 23.0, 0.85, 3.0),
    c("animals:crags", 15.0, 0.5, 6.0),
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
    })
}

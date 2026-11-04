//! Ambient life (`reports/sky-weather-life.md`, change 7): birds of prey circling on thermals by
//! day, gulls over the coast, an owl now and then crossing at night, fish jumping in lakes and
//! rivers, most at dusk and dawn. Not animals: sprites the client alone draws around its player,
//! a few at a time, so they cost no server work and no entities (`mods/animals/src/ambient.rs`).
//! Which of them a place has comes from the species that live there (their rows' habitats), so
//! the sky's birds are the ones the land's are.

use crate::species::{SPECIES, Species};

/// The species whose kind circles on thermals, gulls over the shore, owls at night.
pub const SOARING: &[&str] = &["golden_eagle", "savanna_hawk", "turkey_vulture"];
pub const COASTAL: &[&str] = &["herring_gull"];
pub const NIGHT: &[&str] = &["great_horned_owl", "snowy_owl"];

/// Soaring birds round a player at once, at most; their height above the player (blocks), the
/// circle's radius, and the seconds a lap takes (eagles and vultures circle a thermal in half a
/// minute or so).
pub const SOARERS: usize = 3;
pub const SOAR_HEIGHT: (f32, f32) = (30.0, 55.0);
pub const SOAR_RADIUS: (f32, f32) = (12.0, 26.0);
pub const SOAR_LAP: (f32, f32) = (25.0, 45.0);

/// Gulls over the shore: as many, lower, tighter and quicker.
pub const GULLS: usize = 4;
pub const GULL_HEIGHT: (f32, f32) = (8.0, 20.0);
pub const GULL_RADIUS: (f32, f32) = (6.0, 16.0);
pub const GULL_LAP: (f32, f32) = (10.0, 18.0);

/// An owl crossing at night: one at a time, every so many seconds on average, this high above
/// the player and this fast (blocks a second), from this far away.
pub const OWL_EVERY: f32 = 45.0;
pub const OWL_HEIGHT: (f32, f32) = (5.0, 10.0);
pub const OWL_SPEED: f32 = 7.0;
pub const OWL_FROM: f32 = 30.0;

/// A fish jumps somewhere on the water near the player every so many seconds on average (three
/// times as often at dusk and dawn), within this many blocks; a jump goes up so fast (blocks a
/// second).
pub const FISH_EVERY: f32 = 14.0;
pub const FISH_DUSK: f32 = 3.0;
pub const FISH_REACH: f32 = 20.0;
pub const FISH_LEAP: f32 = 5.0;

/// Ambient birds keep within this many blocks of the player; past it they're gone.
pub const KEEP: f32 = 64.0;
/// Ticks between the server's word to a player on what lives where they stand.
pub const TELL_EVERY: u64 = 40;

/// Day time (ticks of 24000, 0 sunrise): soarers and gulls by day, owls by night, dusk and dawn
/// around them.
pub const DAY: (u64, u64) = (1000, 11500);
pub const NIGHT_TIME: (u64, u64) = (13500, 22500);
pub fn dusk(day_time: u64) -> bool {
    let t = day_time % 24000;
    (11500..13500).contains(&t) || !(500..22500).contains(&t)
}

/// What lives in `biome` for the sky and the water (0: none; else the sprite, `SPRITES`).
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Life {
    pub soar: u8,
    pub coast: u8,
    pub owl: u8,
    pub fish: u8,
}

/// The sprites, by `Life`'s numbers: each a species' model seen from below in flight, or a fish
/// from the side (`source/ambient`).
pub const SPRITES: [&str; 5] = ["ambient_eagle", "ambient_vulture", "ambient_gull", "ambient_owl", "ambient_fish"];

fn lives(s: &Species, biome: &str) -> bool {
    s.habitats.iter().any(|&(b, share)| b == biome && share > 0.0)
}

fn sprite(model: &str) -> u8 {
    let name = match model {
        "vulture" => "ambient_vulture",
        "gull" => "ambient_gull",
        "owl" => "ambient_owl",
        "fish" => "ambient_fish",
        _ => "ambient_eagle",
    };
    SPRITES.iter().position(|&s| s == name).map_or(0, |i| i as u8 + 1)
}

/// The ambient life of `biome`, from the species that live there.
pub fn life_in(biome: &str) -> Life {
    let first = |names: &[&str]| {
        SPECIES
            .iter()
            .find(|s| names.contains(&s.name) && lives(s, biome))
            .map_or(0, |s| sprite(s.model))
    };
    let fish = SPECIES
        .iter()
        .any(|s| s.in_water() && lives(s, biome));
    Life { soar: first(SOARING), coast: first(COASTAL), owl: first(NIGHT), fish: if fish { sprite("fish") } else { 0 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ambient_birds_are_species() {
        for name in SOARING.iter().chain(COASTAL).chain(NIGHT) {
            assert!(SPECIES.iter().any(|s| s.name == *name), "{name} is no species");
        }
        assert_ne!(life_in("base:beach").coast, 0, "gulls over the beach");
    }
}

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

/// Ambient birds keep within this many blocks of the player; past it they're gone and others
/// come in round it.
pub const KEEP: f32 = 64.0;
/// Ticks between the server's word to a player on what lives where they stand, now.
pub const TELL_EVERY: u64 = 60;

/// Sprites' sides in blocks: each bird's wingspan, the fish's length (a golden eagle's span is
/// 1.8-2.3 m, a turkey vulture's 1.7, a herring gull's 1.4, a great horned owl's 1.2; a leaping
/// trout a big one's 0.5).
pub const SPAN: [f32; 5] = [2.0, 1.8, 1.4, 1.2, 0.5];
/// Gulls glide on and drift as a flock, this fast (blocks a second), turning back past this far
/// from the player.
pub const FLOCK_DRIFT: f32 = 1.2;
pub const FLOCK_REACH: f32 = 28.0;
/// Soarers' thermals drift with the air, this fast at most.
pub const THERMAL_DRIFT: f32 = 0.4;
/// The fall a leaping fish makes (blocks a second squared) and how far across it carries
/// (blocks a second); a jump's flight is `2 * FISH_LEAP / FISH_FALL` seconds.
pub const FISH_FALL: f32 = 18.0;
pub const FISH_ACROSS: f32 = 1.6;
/// Columns a client tries for water per jump, at most.
pub const FISH_TRIES: usize = 6;

/// The sprites' sounds (the sound worker's; `mods/animals/src/ambient.rs`), from where the bird
/// or fish is. An eagle on a thermal cries every so many seconds on average (a red-tailed hawk's
/// scream stands in for the golden eagle's and the savanna hawk's: thin, high, carrying);
/// vultures are silent. A gull flock calls every so many seconds; a crossing owl hoots once as it
/// passes nearest the player (the middle of its crossing) and, now and then, once more on its
/// way. A fish's leap splashes where it leaves the water (quieter) and where it falls back.
pub const EAGLE_CRY_EVERY: f32 = 20.0;
pub const GULL_CALL_EVERY: f32 = 5.0;
pub const OWL_SECOND_HOOT: f32 = 0.4;
pub const SPLASH_OUT: f32 = 0.45;
/// How far a cry and a splash are heard (blocks): a hawk's carries to the top of a thermal and
/// past. Gulls and owls use the land's calls (`base:ambient.call.*`, heard to 32).
pub const CRY_RANGE: f32 = 90.0;
pub const SPLASH_RANGE: f32 = 28.0;

/// Hours (0-24, from the sun): soarers ride thermals from this long after sunrise to this long
/// before sunset (the ground has to warm first; they come down before evening), strongest from 11
/// to 15; gulls are about from just before sunrise to just after sunset; owls hunt from this long
/// after sunset to before sunrise; fish jump most within this long of sunrise and sunset.
pub const THERMAL_AFTER_SUNRISE: f32 = 2.5;
pub const THERMAL_BEFORE_SUNSET: f32 = 1.5;
pub const THERMAL_PEAK: (f32, f32) = (11.0, 15.0);
pub const GULL_TWILIGHT: f32 = 0.5;
pub const OWL_AFTER_SUNSET: f32 = 0.75;
pub const FISH_TWILIGHT: f32 = 1.5;
/// Weak winter thermals: this share of the soarers. Cold water: this share of the jumps.
pub const WINTER_THERMALS: f32 = 0.34;
pub const WINTER_FISH: f32 = 0.3;
/// In rain: no thermals, no owls out, this share of the gulls.
pub const RAIN_GULLS: f32 = 0.5;

/// What flies and jumps round a player now: the sprites (0 none, else `SPRITES`' number + 1), how
/// many soarers and gulls, and how often fish jump (by `FISH_EVERY`, 1 a day's rate, `FISH_DUSK`
/// at dusk and dawn). The server works it out every `TELL_EVERY` ticks and puts it on the player
/// as one number (`pack`): the client knows neither biome, climate nor season.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Now {
    pub soar: u8,
    pub soarers: u8,
    pub gull: u8,
    pub gulls: u8,
    pub owl: u8,
    pub fish: u8,
    /// Jumps' rate against the day's, in tenths (10 = 1x).
    pub fish_rate: u8,
}

impl Now {
    pub fn pack(self) -> u32 {
        (self.soar as u32 & 7)
            | (self.soarers as u32 & 7) << 3
            | (self.gull as u32 & 7) << 6
            | (self.gulls as u32 & 7) << 9
            | (self.owl as u32 & 7) << 12
            | (self.fish as u32 & 7) << 15
            | (self.fish_rate as u32 & 63) << 18
    }
    pub fn unpack(v: u32) -> Now {
        let b = |at: u32, bits: u32| ((v >> at) & ((1 << bits) - 1)) as u8;
        Now { soar: b(0, 3), soarers: b(3, 3), gull: b(6, 3), gulls: b(9, 3), owl: b(12, 3), fish: b(15, 3), fish_rate: b(18, 6) }
    }
}

/// The hour the sun rises when it sets at `sunset` (the day is even about noon).
pub fn sunrise(sunset: f32) -> f32 {
    24.0 - sunset
}

/// Whether `hour` is within `span` hours of `at`, round midnight too.
fn near(hour: f32, at: f32, span: f32) -> bool {
    let d = (hour - at).rem_euclid(24.0);
    d.min(24.0 - d) <= span
}

/// How strong thermals are at `hour` (0 none, 1 the day's best), by the sun's day.
pub fn thermals(hour: f32, sunset: f32) -> f32 {
    let (from, to) = (sunrise(sunset) + THERMAL_AFTER_SUNRISE, sunset - THERMAL_BEFORE_SUNSET);
    if hour < from || hour > to {
        return 0.0;
    }
    let (a, b) = THERMAL_PEAK;
    if hour < a {
        0.35 + 0.65 * (hour - from) / (a - from).max(0.1)
    } else if hour > b {
        0.35 + 0.65 * (to - hour) / (to - b).max(0.1)
    } else {
        1.0
    }
}

/// What flies and jumps round a player in a place with `life`, at `hour` on a day whose sun sets
/// at `sunset`, in rain or not, in winter or not.
pub fn now(life: Life, hour: f32, sunset: f32, raining: bool, winter: bool) -> Now {
    let rise = sunrise(sunset);
    let mut n = Now { soar: life.soar, gull: life.coast, fish: life.fish, ..Now::default() };
    let thermal = if raining { 0.0 } else { thermals(hour, sunset) * if winter { WINTER_THERMALS } else { 1.0 } };
    if life.soar != 0 && thermal > 0.0 {
        n.soarers = ((SOARERS as f32 * thermal).round() as u8).max(1);
    }
    if life.coast != 0 && hour >= rise - GULL_TWILIGHT && hour <= sunset + GULL_TWILIGHT {
        let share = if raining { RAIN_GULLS } else { 1.0 };
        n.gulls = ((GULLS as f32 * share).round() as u8).max(1);
    }
    let night = hour >= sunset + OWL_AFTER_SUNSET || hour <= rise - OWL_AFTER_SUNSET;
    if life.owl != 0 && night && !raining {
        n.owl = life.owl;
    }
    if life.fish != 0 {
        let twilight = near(hour, rise, FISH_TWILIGHT) || near(hour, sunset, FISH_TWILIGHT);
        let rate = if twilight { FISH_DUSK } else { 1.0 } * if winter { WINTER_FISH } else { 1.0 };
        n.fish_rate = (rate * 10.0).round().clamp(1.0, 63.0) as u8;
    }
    if n.soarers == 0 {
        n.soar = 0;
    }
    if n.gulls == 0 {
        n.gull = 0;
    }
    n
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
    s.habitats
        .iter()
        .any(|&(b, share)| b == biome && share > 0.0)
}

fn sprite(model: &str) -> u8 {
    let name = match model {
        "vulture" => "ambient_vulture",
        "gull" => "ambient_gull",
        "owl" => "ambient_owl",
        "fish" => "ambient_fish",
        _ => "ambient_eagle",
    };
    SPRITES
        .iter()
        .position(|&s| s == name)
        .map_or(0, |i| i as u8 + 1)
}

/// The ambient life of `biome`, from the species that live there.
pub fn life_in(biome: &str) -> Life {
    let first = |names: &[&str]| {
        SPECIES
            .iter()
            .find(|s| names.contains(&s.name) && lives(s, biome))
            .map_or(0, |s| sprite(s.model))
    };
    let fish = SPECIES.iter().any(|s| s.in_water() && lives(s, biome));
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

    #[test]
    fn the_word_round_trips_and_follows_the_sun() {
        let life = Life { soar: 1, coast: 3, owl: 4, fish: 5 };
        let noon = now(life, 13.0, 19.0, false, false);
        assert_eq!(Now::unpack(noon.pack()), noon);
        assert_eq!(noon.soarers as usize, SOARERS, "full thermals at noon");
        assert_eq!(noon.owl, 0, "no owls by day");
        let night = now(life, 1.0, 19.0, false, false);
        assert_eq!((night.soarers, night.gulls), (0, 0), "nobody soars at night");
        assert_ne!(night.owl, 0, "an owl at night");
        assert!(now(life, 5.2, 19.0, false, false).fish_rate > noon.fish_rate, "fish jump most at dawn");
        assert_eq!(now(life, 13.0, 19.0, true, false).soarers, 0, "no thermals in rain");
        assert!(now(life, 13.0, 19.0, false, true).soarers < noon.soarers, "weak winter thermals");
    }
}

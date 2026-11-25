//! What the land sounds like where the player stands (`notes/SYSTEMS.md`, ambience), after
//! Hytale's AmbienceFX and AmbientSounds: a few looped beds whose volumes ease toward levels set
//! by the biomes around, the hour, the season and the weather, scattered one-shot calls whose
//! chances follow the same levels, and loops on the water and lava near the player. The cover
//! over the player (`weather_sound::Cover`) muffles them all as it does the weather; deep under
//! ground (no sky over the roof) the life of the surface is not heard at all, only drips.
//!
//! The server works out the beds' levels for each player (it knows biome, air and season); the
//! client eases them, rolls the calls and finds the water itself.

/// The beds, in the order of `Levels::beds`.
pub const BEDS: [&str; 6] = ["birds", "dawn", "night", "dry", "swamp", "surf"];
pub const BIRDS: usize = 0;
pub const DAWN: usize = 1;
pub const NIGHT: usize = 2;
pub const DRY: usize = 3;
pub const SWAMP: usize = 4;
pub const SURF: usize = 5;

/// How much of each kind of life a biome has, 0 to 1. Owls and ravens are calls, not beds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scape {
    pub biome: &'static str,
    /// Songbirds by day (the forest-day bed and its calls).
    pub birds: f32,
    /// Crickets and other insects at night, in warm air.
    pub night: f32,
    /// Hot dry wind and little else (desert, savanna's dry season).
    pub dry: f32,
    /// Frogs and wet air.
    pub swamp: f32,
    /// Waves on a shore.
    pub surf: f32,
    pub owls: f32,
    pub ravens: f32,
}

#[allow(clippy::too_many_arguments)]
const fn s(biome: &'static str, birds: f32, night: f32, dry: f32, swamp: f32, surf: f32, owls: f32, ravens: f32) -> Scape {
    Scape { biome, birds, night, dry, swamp, surf, owls, ravens }
}

/// Every biome with a climate (`climate::BIOMES`; the test ties them).
#[rustfmt::skip]
pub const SCAPES: [Scape; 22] = [
    //  biome                      birds night dry  swamp surf owls ravens
    s("base:plains",               0.55, 0.80, 0.0, 0.0,  0.0, 0.10, 0.20),
    s("base:forest",               1.00, 0.70, 0.0, 0.0,  0.0, 0.80, 0.30),
    s("base:birch_forest",         0.95, 0.70, 0.0, 0.0,  0.0, 0.60, 0.20),
    s("base:old_growth_forest",    0.90, 0.60, 0.0, 0.10, 0.0, 1.00, 0.40),
    s("base:sequoia_grove",        0.70, 0.50, 0.0, 0.0,  0.0, 0.90, 0.50),
    s("base:taiga",                0.45, 0.30, 0.0, 0.0,  0.0, 0.70, 0.90),
    s("base:snowy_plains",         0.10, 0.0,  0.0, 0.0,  0.0, 0.10, 0.60),
    s("base:mountains",            0.25, 0.20, 0.0, 0.0,  0.0, 0.20, 0.80),
    s("base:tropical_forest",      1.00, 1.00, 0.0, 0.30, 0.0, 0.30, 0.0),
    s("base:savanna",              0.50, 0.90, 0.60, 0.0, 0.0, 0.10, 0.10),
    s("base:desert",               0.05, 0.40, 1.00, 0.0, 0.0, 0.0,  0.20),
    s("base:swamp",                0.50, 0.90, 0.0, 1.00, 0.0, 0.40, 0.20),
    s("base:beach",                0.20, 0.50, 0.0, 0.0,  1.0, 0.0,  0.0),
    s("base:lagoon",               0.30, 0.60, 0.0, 0.20, 0.7, 0.0,  0.0),
    s("base:ocean",                0.0,  0.0,  0.0, 0.0,  1.0, 0.0,  0.0),
    s("base:coral_reef",           0.0,  0.0,  0.0, 0.0,  1.0, 0.0,  0.0),
    s("base:deep_caves",           0.0,  0.0,  0.0, 0.0,  0.0, 0.0,  0.0),
    // The animals mod's lands.
    s("animals:red_dunes",         0.05, 0.30, 1.00, 0.0, 0.0, 0.0,  0.30),
    s("animals:scrubland",         0.40, 0.80, 0.50, 0.0, 0.0, 0.10, 0.30),
    s("animals:floodplain",        0.70, 0.90, 0.0, 0.70, 0.0, 0.30, 0.20),
    s("animals:fern_forest",       0.90, 0.80, 0.0, 0.20, 0.0, 0.50, 0.10),
    s("animals:crags",             0.20, 0.20, 0.0, 0.0,  0.0, 0.20, 0.90),
];

/// The biome's scape; a biome without one is quiet (another mod's, till it adds one).
pub fn scape(biome: &str) -> Scape {
    SCAPES
        .iter()
        .copied()
        .find(|s| s.biome == biome)
        .unwrap_or(s("", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
}

/// Blocks from the player at which the biomes around are also sampled (four points on each
/// ring), so a forest's edge is heard from the field beside it; the player's own biome counts
/// as much as each whole ring.
pub const AROUND: [i32; 2] = [16, 40];

/// The mix of the scapes at the player (weight 1) and on the rings (`AROUND`, 1 each, shared by
/// their four points).
pub fn mix(here: Scape, rings: &[[Scape; 4]]) -> Scape {
    let mut out = here;
    let n = 1.0 + rings.len() as f32;
    let f = |a: f32, ring: &[Scape; 4], get: fn(&Scape) -> f32| a + ring.iter().map(get).sum::<f32>() / 4.0;
    for r in rings {
        out.birds = f(out.birds, r, |s| s.birds);
        out.night = f(out.night, r, |s| s.night);
        out.dry = f(out.dry, r, |s| s.dry);
        out.swamp = f(out.swamp, r, |s| s.swamp);
        out.surf = f(out.surf, r, |s| s.surf);
        out.owls = f(out.owls, r, |s| s.owls);
        out.ravens = f(out.ravens, r, |s| s.ravens);
    }
    for v in [&mut out.birds, &mut out.night, &mut out.dry, &mut out.swamp, &mut out.surf, &mut out.owls, &mut out.ravens] {
        *v /= n;
    }
    out
}

/// Waves are heard only near sea level: this many blocks above it they're gone.
pub const SURF_UP: f32 = 24.0;

/// Insects sing from this air (°C), fully from the second.
pub const INSECTS_C: (f32, f32) = (8.0, 18.0);
/// Birds sing less in the cold: none under the first air (°C), all over the second.
pub const BIRDS_C: (f32, f32) = (-15.0, 5.0);

/// Birdsong by season (spring, summer, autumn, winter): loudest when they court in spring.
pub const BIRD_SEASON: [f32; 4] = [1.0, 0.85, 0.6, 0.35];

/// How much rain hushes the birds and insects (1 = all of it at full rain); a storm hushes
/// everything alive.
pub const RAIN_HUSH: f32 = 0.85;

/// The dawn chorus: from this many hours before sunrise to this many after, loudest at sunrise.
pub const DAWN_HOURS: (f32, f32) = (1.0, 2.0);

/// Smoothstep from `a` to `b`.
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 0 at night, 1 in daylight, eased over an hour each side of sunrise and sunset.
pub fn daylight(hour: f32, sunrise: f32, sunset: f32) -> f32 {
    smooth(sunrise - 0.5, sunrise + 0.5, hour) * (1.0 - smooth(sunset - 0.5, sunset + 0.5, hour))
}

/// The dawn chorus' share at `hour`.
pub fn dawn(hour: f32, sunrise: f32) -> f32 {
    let (before, after) = DAWN_HOURS;
    if hour < sunrise {
        smooth(sunrise - before, sunrise, hour)
    } else {
        1.0 - smooth(sunrise, sunrise + after, hour)
    }
}

/// What the beds and calls are set to now, before the cover.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Levels {
    pub beds: [f32; 6],
    pub owls: f32,
    pub ravens: f32,
}

/// The levels for a mixed scape (`mix`) at `hour` (sunrise and sunset that day), in `air` °C,
/// in `season` (0 spring .. 3 winter, `None` without a calendar), with `rain` and `thunder`
/// (0 to 1, where the player is) and `height` blocks above the sea.
#[allow(clippy::too_many_arguments)]
pub fn levels(m: Scape, hour: f32, sunrise: f32, sunset: f32, air: f32, season: Option<usize>, rain: f32, thunder: f32, height: f32) -> Levels {
    let day = daylight(hour, sunrise, sunset);
    let calm = (1.0 - RAIN_HUSH * rain.clamp(0.0, 1.0)) * (1.0 - thunder.clamp(0.0, 1.0));
    let birds_warm = smooth(BIRDS_C.0, BIRDS_C.1, air) * season.map_or(0.8, |s| BIRD_SEASON[s.min(3)]);
    let insects = smooth(INSECTS_C.0, INSECTS_C.1, air);
    let birds = m.birds * birds_warm * calm;
    let mut beds = [0.0; 6];
    beds[BIRDS] = birds * day;
    beds[DAWN] = birds * dawn(hour, sunrise);
    beds[NIGHT] = m.night * insects * (1.0 - day) * calm;
    // The dry wind blows by day and night; rain lays it.
    beds[DRY] = m.dry * (1.0 - rain.clamp(0.0, 1.0));
    // Frogs call at dusk and through warm nights, a little by day.
    beds[SWAMP] = m.swamp * insects * (0.3 + 0.7 * (1.0 - day)) * calm.max(0.3);
    // Waves don't stop for the weather; they grow in a storm.
    beds[SURF] = m.surf * (1.0 - smooth(0.0, SURF_UP, height)) * (1.0 + 0.4 * thunder.clamp(0.0, 1.0)).min(1.0);
    Levels {
        beds,
        owls: m.owls * (1.0 - day) * calm * smooth(BIRDS_C.0, BIRDS_C.1, air),
        ravens: m.ravens * day * (1.0 - rain.clamp(0.0, 1.0) * 0.5) * (1.0 - thunder.clamp(0.0, 1.0)),
    }
}

/// A one-shot call: heard on average every `every` seconds at full level, placed `dist` blocks
/// away (min, max) and `up` blocks above the head.
#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub name: &'static str,
    pub every: f32,
    pub dist: (f32, f32),
    pub up: (f32, f32),
}

/// The calls: a songbird in the trees, an owl at night, a raven overhead, a gull on the shore,
/// a frog in the reeds, a drip in a cave, a pop of lava.
pub const CALLS: [Call; 7] = [
    Call { name: "bird", every: 7.0, dist: (6.0, 20.0), up: (3.0, 10.0) },
    Call { name: "owl", every: 25.0, dist: (12.0, 28.0), up: (4.0, 12.0) },
    Call { name: "raven", every: 30.0, dist: (10.0, 30.0), up: (8.0, 20.0) },
    Call { name: "gull", every: 12.0, dist: (8.0, 26.0), up: (4.0, 14.0) },
    Call { name: "frog", every: 4.0, dist: (5.0, 16.0), up: (-1.0, 0.5) },
    Call { name: "drip", every: 9.0, dist: (3.0, 12.0), up: (1.0, 5.0) },
    Call { name: "lava_pop", every: 3.0, dist: (0.0, 0.0), up: (0.5, 1.0) },
];

/// Never two calls of a kind closer than this many seconds (no machine-gun birds).
pub const CALL_GAP: f32 = 1.5;

/// The share of a call's chances its level gives: a bed level, or a count of things near.
pub fn call_chance(level: f32, every: f32, dt: f32) -> f32 {
    (level.clamp(0.0, 1.0) * dt / every.max(0.1)).clamp(0.0, 1.0)
}

/// Water and lava near the player: blocks of each kind found in the sampled cube
/// (`LIQUID_REACH`, every other block across, every block up and down) that make the loop full.
pub const FLOWING_FULL: f32 = 20.0;
pub const STILL_FULL: f32 = 80.0;
pub const LAVA_FULL: f32 = 12.0;
/// Half the cube's side, blocks.
pub const LIQUID_REACH: i32 = 10;

/// The loops' levels for counts of flowing water, still water open to the air, and lava.
pub fn liquids(flowing: u32, still: u32, lava: u32) -> [f32; 3] {
    let f = |n: u32, full: f32| (n as f32 / full).clamp(0.0, 1.0).sqrt();
    [f(flowing, FLOWING_FULL), f(still, STILL_FULL) * 0.6, f(lava, LAVA_FULL)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_climate_biome_has_a_scape() {
        for c in crate::climate::BIOMES.iter() {
            assert!(SCAPES.iter().any(|s| s.biome == c.biome), "{} has no scape", c.biome);
        }
        for s in SCAPES {
            assert!(crate::climate::of(s.biome).is_some(), "{} is no biome", s.biome);
        }
    }

    #[test]
    fn forest_morning_sings_and_night_chirps() {
        let f = scape("base:forest");
        let morning = levels(f, 6.0, 6.0, 20.0, 15.0, Some(0), 0.0, 0.0, 10.0);
        assert!(morning.beds[DAWN] > 0.9 && morning.beds[NIGHT] < 0.1);
        let noon = levels(f, 13.0, 6.0, 20.0, 22.0, Some(1), 0.0, 0.0, 10.0);
        assert!(noon.beds[BIRDS] > 0.7 && noon.beds[DAWN] < 0.01 && noon.owls < 0.01);
        let night = levels(f, 23.0, 6.0, 20.0, 18.0, Some(1), 0.0, 0.0, 10.0);
        assert!(night.beds[NIGHT] > 0.6 && night.beds[BIRDS] < 0.01 && night.owls > 0.5);
        // Cold nights are silent; rain hushes the birds; a storm everything alive.
        let cold = levels(f, 23.0, 6.0, 20.0, 2.0, Some(3), 0.0, 0.0, 10.0);
        assert!(cold.beds[NIGHT] < 0.01);
        let rain = levels(f, 13.0, 6.0, 20.0, 22.0, Some(1), 1.0, 0.0, 10.0);
        assert!(rain.beds[BIRDS] < noon.beds[BIRDS] * 0.2);
        let storm = levels(f, 13.0, 6.0, 20.0, 22.0, Some(1), 1.0, 1.0, 10.0);
        assert_eq!(storm.beds[BIRDS], 0.0);
    }

    #[test]
    fn surf_only_by_the_sea() {
        let b = scape("base:beach");
        assert!(levels(b, 12.0, 6.0, 20.0, 20.0, None, 0.0, 0.0, 2.0).beds[SURF] > 0.9);
        assert!(levels(b, 12.0, 6.0, 20.0, 20.0, None, 0.0, 0.0, SURF_UP + 1.0).beds[SURF] < 0.01);
        assert_eq!(levels(scape("base:plains"), 12.0, 6.0, 20.0, 20.0, None, 0.0, 0.0, 2.0).beds[SURF], 0.0);
    }

    #[test]
    fn a_forest_edge_is_heard_from_the_field() {
        let p = scape("base:plains");
        let f = scape("base:forest");
        let m = mix(p, &[[f, f, p, p], [f, f, f, p]]);
        assert!(m.birds > p.birds && m.birds < f.birds);
    }

    #[test]
    fn liquids_fill_up() {
        assert_eq!(liquids(0, 0, 0), [0.0; 3]);
        let full = liquids(100, 100, 100);
        assert_eq!(full[0], 1.0);
        assert!(full[1] <= 0.6 && full[2] == 1.0);
        assert!(liquids(2, 0, 0)[0] > 0.3, "a small stream is heard");
    }
}

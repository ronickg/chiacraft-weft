//! Sky events (reports/Research round 17.md, its addendum; Ronald: "Yes, as designed"): a blood
//! moon on some full moons, meteor showers on two dates a year with a lone fireball now and then,
//! and aurora storms (looks only). At most one a night, the same on every machine: a hash of the
//! world's seed and the day, as rain's `climate::spell` is.
//!
//! A game day starts at 06:00 (`calendar`), so a night, 18:00 to 06:00, lies inside one day:
//! `tonight(seed, day)` is the night at the end of `day` (days since the world began).

use crate::calendar::{SEASON_DAYS, YEAR_DAYS};

/// What the sky does tonight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A total eclipse of the full moon: more and bolder monsters in totality.
    BloodMoon,
    /// A meteor shower: its peak night (`peak`), or a night either side of it.
    Meteors { peak: bool },
    /// A lone fireball that lands, unannounced.
    Fireball,
    /// The aurora reaches south, brighter: looks only.
    AuroraStorm,
}

impl Event {
    /// Its name in keys and logs: `hint.base.<name>`, `/sky` and the like.
    pub fn name(self) -> &'static str {
        match self {
            Event::BloodMoon => "blood_moon",
            Event::Meteors { peak: true } => "meteor_shower",
            Event::Meteors { peak: false } => "meteor_side",
            Event::Fireball => "fireball",
            Event::AuroraStorm => "aurora_storm",
        }
    }

    /// Foretold at dusk (`DUSK_LINE_HOURS` before sunset): a fireball comes with no warning, and a
    /// shower's side nights aren't worth a line.
    pub fn announced(self) -> bool {
        matches!(self, Event::BloodMoon | Event::Meteors { peak: true } | Event::AuroraStorm)
    }
}

// ---------------------------------------------------------------- the blood moon

/// The moon's phases: full when the day is a multiple of it (`lib.rs`, the moon's path).
pub const MOON_DAYS: u64 = 8;
/// A full moon is a blood moon one time in three: 2-3 a year (7 full moons in a 56-day year),
/// about once every 24 days. Terraria's 1 in 9 nights would be every 3 hours of play.
pub const BLOOD_MOON_CHANCE: f32 = 1.0 / 3.0;
/// None before the second full moon after a new world's start (`calendar::START_DAY` 7).
pub const BLOOD_MOON_FROM_DAY: u64 = 16;
/// Totality, hours: the moon reddens over the hour before and clears over the hour after, so the
/// event runs 21:00-03:00.
pub const BLOOD_MOON_HOURS: (f32, f32) = (22.0, 2.0);
pub const BLOOD_MOON_FADE_HOURS: f32 = 1.0;
/// Its monsters, a category of their own on top of the hostile 8 within 64 blocks (normal's;
/// `difficulty::Difficulty::blood_moon_cap` for each).
pub const BLOOD_MOON_CAP: u32 = 6;
/// Bolder, not deadlier: max health and speed on spawning, and the reach they notice players at;
/// no extra damage.
pub const BLOOD_MOON_HEALTH: f32 = 1.2;
pub const BLOOD_MOON_SPEED: f32 = 1.1;
pub const BLOOD_MOON_REACH: f32 = 1.5;
/// XP for kills in totality.
pub const BLOOD_MOON_XP: f32 = 1.5;
/// The monsters it brings, by weight: pack hunters, few creepers (fewer craters in bases).
pub const BLOOD_MOON_WEIGHTS: [(&str, u32); 4] = [("zombie", 3), ("spider", 3), ("skeleton", 2), ("creeper", 1)];
/// One event monster in this many drops eclipse ash (a dye; cosmetic, so never a grind).
pub const ECLIPSE_ASH_ONE_IN: u32 = 8;

// ---------------------------------------------------------------- meteors

/// A meteor shower: its season (`calendar::SEASONS` index) and its peak's day of the season
/// (from 1), with a night either side.
#[derive(Clone, Copy, Debug)]
pub struct Shower {
    pub name: &'static str,
    pub season: u32,
    pub peak: u32,
}
/// The Perseids (mid-August) and the Geminids (mid-December), every year.
pub const METEOR_SHOWERS: [Shower; 2] = [Shower { name: "summer", season: 1, peak: 10 }, Shower { name: "winter", season: 3, peak: 3 }];
/// Shooting stars on a shower's peak and its side nights, times the usual (from 23:00 to dawn).
pub const SHOWER_PEAK_STREAKS: f32 = 20.0;
pub const SHOWER_SIDE_STREAKS: f32 = 6.0;
/// A night (that nothing else takes) brings a lone fireball that lands, one in this many.
pub const FIREBALL_CHANCE: f32 = 1.0 / 20.0;
/// Meteorites landing near each player, (least, most): a peak night, a side night, a fireball.
pub const METEOR_LANDINGS: [(u32, u32); 3] = [(1, 2), (0, 1), (1, 1)];
/// How far from the player one lands, blocks: under open sky, on land.
pub const METEOR_REACH: (i32, i32) = (48, 160);
/// Never within this many blocks of a player-placed block or a campfire.
pub const METEOR_CLEAR: i32 = 24;
/// A fallen meteorite glows hot for this many game hours (light 9 to 0).
pub const METEORITE_COOLS: f32 = 1.0;
/// Meteoric iron lumps a meteorite block drops (least, most).
pub const METEORITE_DROPS: (u32, u32) = (1, 3);

// ---------------------------------------------------------------- aurora storms

/// A night (that nothing else takes) is an aurora storm one in this many ...
pub const AURORA_STORM_CHANCE: f32 = 1.0 / 30.0;
/// ... and this many times as often within `EQUINOX_DAYS` of an equinox (spring's and autumn's
/// 7th day): the Russell-McPherron effect makes real geomagnetic storms likeliest then.
pub const EQUINOX_BOOST: f32 = 2.0;
pub const EQUINOX_DAYS: u32 = 4;
/// The equinoxes' days of the season (spring = season 0, autumn = 2).
pub const EQUINOX_DAY: u32 = 7;

// ---------------------------------------------------------------- the dusk line

/// Each announced event is foretold this many hours before sunset (the night hint's slot).
pub const DUSK_LINE_HOURS: f32 = 2.0;

// ---------------------------------------------------------------- big impacts (Addendum 2)

/// A big impact's schedule: the days from `BIG_IMPACT_FROM_DAY` cut in slots of this many, each
/// with one impact `BIG_IMPACT_SLOT_CHANCE` of the time on one of its first 10 days (the slot
/// less the gap, plus one), at any hour (big falls come by day too): about 2.65 a world a year
/// (the design's 3), at least `BIG_IMPACT_GAP` days apart, the same on every machine and with no
/// look back.
pub const BIG_IMPACT_SLOT_DAYS: u64 = 19;
pub const BIG_IMPACT_SLOT_CHANCE: f32 = 0.9;
pub const BIG_IMPACT_GAP: u64 = 10;
/// None in the first season: a player has a base to come back to first.
pub const BIG_IMPACT_FROM_DAY: u64 = 21;
/// Where one may land: at least this far from every player (no one is hurt), in loaded land out
/// to `IMPACT_REACH`, and this far from anything built (`BUILT_CELL` cells a player placed a
/// block in, beds and tombstones) and the world spawn.
pub const IMPACT_FROM_PLAYERS: i32 = 160;
pub const IMPACT_REACH: i32 = 200;
pub const IMPACT_FROM_BUILDS: i32 = 256;
/// The built-columns map: cells this many blocks across, and the most a player remembers.
pub const BUILT_CELL: i32 = 64;
pub const BUILT_CELLS: usize = 32;
/// Tries at a site before it falls in the sea instead.
pub const IMPACT_SITE_TRIES: u32 = 96;
/// The widest live crater (Terrain's `vitals::impact::IMPACT_RADIUS`): a site is level and dry
/// that far round.
pub const IMPACT_SITE_RADIUS: i32 = 12;
/// The land at a site: level within this many blocks over the crater's radius, and no water.
pub const IMPACT_LEVEL: i32 = 4;
/// The brightening and rumble before the streak, seconds.
pub const IMPACT_WARN_SECS: f32 = 20.0;
/// The pointer over the hotbar lasts a game day, or until within this many blocks.
pub const POINTER_DAYS: u64 = 1;
pub const POINTER_NEAR: f64 = 16.0;
/// The direction line's bands: near under the first, far under the second, very far past it.
pub const IMPACT_BANDS: (f64, f64) = (500.0, 1500.0);
/// The core's share for players near: 1 alone, `IMPACT_SHARE.0` more for each other player within
/// `IMPACT_SHARE.2` blocks, up to `IMPACT_SHARE.1`.
pub const IMPACT_SHARE: (f32, f32, f64) = (0.4, 2.2, 2000.0);

/// The hour a big impact falls on `day` in the world of `seed`, if one does.
pub fn impact_on(seed: u64, day: u64) -> Option<f32> {
    if day < BIG_IMPACT_FROM_DAY {
        return None;
    }
    let slot = (day - BIG_IMPACT_FROM_DAY) / BIG_IMPACT_SLOT_DAYS;
    let r = mix(mix(seed ^ 0x494D_5041_4354_5321) ^ slot);
    if unit(r) >= BIG_IMPACT_SLOT_CHANCE {
        return None;
    }
    let days = BIG_IMPACT_SLOT_DAYS - BIG_IMPACT_GAP + 1;
    let at = (mix(r) >> 44) % days;
    ((day - BIG_IMPACT_FROM_DAY) % BIG_IMPACT_SLOT_DAYS == at).then(|| 24.0 * unit(mix(mix(r))))
}

/// The core's share with `others` more players within reach.
pub fn impact_share(others: u32) -> f32 {
    (1.0 + IMPACT_SHARE.0 * others as f32).min(IMPACT_SHARE.1)
}

/// The 8-point direction (0 north, clockwise) from (`dx`, `dz`), north being -z.
pub fn compass(dx: f64, dz: f64) -> u32 {
    let angle = dx.atan2(-dz).to_degrees().rem_euclid(360.0);
    ((angle + 22.5) / 45.0) as u32 % 8
}

/// What the sky does on the night at the end of `day` (days since the world began), in the world
/// of `seed`. In order of priority: a blood moon, a shower's nights, an aurora storm, a fireball.
pub fn tonight(seed: u64, day: u64) -> Option<Event> {
    // A roll of its own for each kind of event, from the hash's best-mixed (top) bits: the low
    // bits of seeds a few bits apart gave three worlds the same first blood moons.
    let r = mix(mix(seed ^ 0x5348_5945_5645_4E54) ^ day);
    let (blood, aurora_roll, fireball) = (unit(r), unit(mix(r)), unit(mix(mix(r))));
    if day >= BLOOD_MOON_FROM_DAY && day.is_multiple_of(MOON_DAYS) && blood < BLOOD_MOON_CHANCE {
        return Some(Event::BloodMoon);
    }
    let of_year = (day % YEAR_DAYS as u64) as u32;
    for s in METEOR_SHOWERS {
        let peak = s.season * SEASON_DAYS + s.peak - 1;
        if of_year == peak {
            return Some(Event::Meteors { peak: true });
        }
        if of_year + 1 == peak || of_year == peak + 1 {
            return Some(Event::Meteors { peak: false });
        }
    }
    let near_equinox = [0, 2].iter().any(|&season| {
        let equinox = season * SEASON_DAYS + EQUINOX_DAY - 1;
        of_year.abs_diff(equinox) <= EQUINOX_DAYS
    });
    let aurora = AURORA_STORM_CHANCE * if near_equinox { EQUINOX_BOOST } else { 1.0 };
    if aurora_roll < aurora {
        return Some(Event::AuroraStorm);
    }
    if fireball < FIREBALL_CHANCE {
        return Some(Event::Fireball);
    }
    None
}

/// How far into a blood moon `hour` is: 0 outside it, rising to 1 over the hour before totality,
/// 1 in totality, falling to 0 over the hour after.
pub fn blood_moon_strength(hour: f32) -> f32 {
    let (from, to) = BLOOD_MOON_HOURS;
    // Hours after the event's start (21:00), through midnight.
    let start = from - BLOOD_MOON_FADE_HOURS;
    let since = (hour - start).rem_euclid(24.0);
    let length = (to - start).rem_euclid(24.0) + BLOOD_MOON_FADE_HOURS;
    if since >= length {
        return 0.0;
    }
    (since / BLOOD_MOON_FADE_HOURS)
        .min((length - since) / BLOOD_MOON_FADE_HOURS)
        .min(1.0)
}

/// Whether `hour` is in a blood moon's totality, when its monsters walk.
pub fn in_totality(hour: f32) -> bool {
    blood_moon_strength(hour) >= 1.0
}

/// The hour of day at `day_time` (ticks since the world began; 0 is 06:00).
pub fn hour_of(day_time: u64) -> f32 {
    ((day_time % crate::calendar::DAY_TICKS) as f32 / crate::calendar::DAY_TICKS as f32 * 24.0 + 6.0) % 24.0
}

/// SplitMix64: a hash every machine agrees on (as `climate`'s).
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 0..1 from the top 20 bits.
fn unit(r: u64) -> f32 {
    (r >> 44) as f32 / (1u64 << 20) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const YEARS: u64 = 100;
    const SEEDS: [u64; 5] = [1, 7, 42, 1234, 0xDEAD_BEEF];

    fn count(seed: u64, want: impl Fn(Event) -> bool) -> u32 {
        (0..YEARS * YEAR_DAYS as u64)
            .filter(|&d| tonight(seed, d).is_some_and(&want))
            .count() as u32
    }

    #[test]
    fn blood_moons_two_or_three_a_year_on_full_moons_only() {
        for seed in SEEDS {
            let n = count(seed, |e| e == Event::BloodMoon) as f32 / YEARS as f32;
            assert!((2.0..=3.0).contains(&n), "seed {seed}: {n} a year");
            for d in 0..YEARS * YEAR_DAYS as u64 {
                if tonight(seed, d) == Some(Event::BloodMoon) {
                    assert!(d.is_multiple_of(MOON_DAYS) && d >= BLOOD_MOON_FROM_DAY, "seed {seed}: day {d}");
                }
            }
        }
    }

    #[test]
    fn showers_peak_on_their_dates_every_year() {
        for seed in SEEDS {
            for year in 0..YEARS {
                for s in METEOR_SHOWERS {
                    let day = year * YEAR_DAYS as u64 + (s.season * SEASON_DAYS + s.peak - 1) as u64;
                    let e = tonight(seed, day);
                    // A blood moon on the same night would take it; the peaks are never full moons.
                    assert_eq!(e, Some(Event::Meteors { peak: true }), "seed {seed}: the {} shower, year {year}", s.name);
                }
            }
        }
    }

    #[test]
    fn aurora_storms_one_to_three_a_year() {
        for seed in SEEDS {
            let n = count(seed, |e| e == Event::AuroraStorm) as f32 / YEARS as f32;
            assert!((1.0..=3.0).contains(&n), "seed {seed}: {n} a year");
        }
    }

    #[test]
    fn fireballs_now_and_then() {
        for seed in SEEDS {
            let n = count(seed, |e| e == Event::Fireball) as f32 / YEARS as f32;
            assert!((1.5..=4.0).contains(&n), "seed {seed}: {n} a year");
        }
    }

    #[test]
    fn worlds_differ_and_each_is_the_same_every_time() {
        let a: Vec<_> = (0..560).map(|d| tonight(1, d)).collect();
        let b: Vec<_> = (0..560).map(|d| tonight(1, d)).collect();
        let c: Vec<_> = (0..560).map(|d| tonight(2, d)).collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn the_blood_moon_reddens_holds_and_clears() {
        assert_eq!(blood_moon_strength(20.0), 0.0);
        assert!((blood_moon_strength(21.5) - 0.5).abs() < 1e-4);
        assert!(in_totality(22.0) && in_totality(0.0) && in_totality(1.9));
        assert!((blood_moon_strength(2.5) - 0.5).abs() < 1e-4);
        assert_eq!(blood_moon_strength(3.0), 0.0);
        assert_eq!(blood_moon_strength(12.0), 0.0);
    }

    #[test]
    fn hours_from_day_time() {
        assert_eq!(hour_of(0), 6.0);
        assert_eq!(hour_of(crate::calendar::DAY_TICKS * 3 + 18000), 0.0);
        // The whole night, 18:00 to 06:00, is the same day's.
        assert!(hour_of(crate::calendar::DAY_TICKS - 1) > 5.9);
    }

    #[test]
    fn big_impacts_two_to_four_a_year_kept_apart() {
        for seed in SEEDS {
            let days: Vec<u64> = (0..YEARS * YEAR_DAYS as u64)
                .filter(|&d| impact_on(seed, d).is_some())
                .collect();
            let n = days.len() as f32 / YEARS as f32;
            assert!((2.0..=4.0).contains(&n), "seed {seed}: {n} a year");
            assert!(days[0] >= BIG_IMPACT_FROM_DAY, "seed {seed}: the first on day {}", days[0]);
            for pair in days.windows(2) {
                assert!(pair[1] - pair[0] >= BIG_IMPACT_GAP, "seed {seed}: days {pair:?}");
            }
            for d in days {
                let hour = impact_on(seed, d).unwrap();
                assert!((0.0..24.0).contains(&hour));
            }
        }
    }

    #[test]
    fn the_compass_points() {
        assert_eq!(compass(0.0, -10.0), 0, "north is -z");
        assert_eq!(compass(10.0, -10.0), 1);
        assert_eq!(compass(10.0, 0.0), 2);
        assert_eq!(compass(0.0, 10.0), 4);
        assert_eq!(compass(-10.0, 0.0), 6);
        assert_eq!(compass(-10.0, -9.0), 7);
    }

    #[test]
    fn the_core_grows_with_players_near_but_not_past_its_cap() {
        assert_eq!(impact_share(0), 1.0);
        assert!((impact_share(1) - 1.4).abs() < 1e-6);
        assert_eq!(impact_share(10), IMPACT_SHARE.1);
    }

    #[test]
    fn a_blood_moon_is_bolder_not_deadlier() {
        const {
            assert!(BLOOD_MOON_HEALTH > 1.0 && BLOOD_MOON_HEALTH < 1.5);
            assert!(BLOOD_MOON_SPEED <= 1.1);
            assert!(BLOOD_MOON_CAP <= 8);
        };
        let weights: u32 = BLOOD_MOON_WEIGHTS.iter().map(|w| w.1).sum();
        let creeper = BLOOD_MOON_WEIGHTS
            .iter()
            .find(|w| w.0 == "creeper")
            .map_or(0, |w| w.1);
        assert!(creeper * 5 <= weights, "few creepers");
    }
}

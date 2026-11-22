//! Day routines (`reports/Creature AI.md`, part 3; the crew's step 8): when in the day each species
//! is about (`Species::active`, from its research: the journal showed it, now it acts on it). The
//! numbers the mod reads (`mods/animals/src/choose.rs`, `wild.rs`, `rest.rs`, `drink.rs`,
//! `hunt.rs`).
//!
//! - Out of its hours it is sleepy: resting scores up to `SLEEPY` more in its needs, and it lies
//!   down more often.
//! - A plant eater goes to water at dawn and dusk (`DRINKS_AT` of its water, instead of
//!   `needs::THIRSTY`): herds at the river at dusk.
//! - A hunter hunts in its hours, out of them only when hunger is pressing (under `HUNTS_ANYWAY`
//!   of its food).

use crate::species::Active;

/// How about it is at `hour` (0..24, 6 dawn, 18 dusk), 0..1.
pub fn activity(active: Active, hour: f32) -> f32 {
    let h = hour.rem_euclid(24.0);
    // 1 at noon, 0 at midnight, smoothly.
    let day = (0.5 - 0.5 * ((h - 12.0) / 12.0 * std::f32::consts::PI).cos()).clamp(0.0, 1.0);
    // 1 at dawn and dusk, falling off over two hours either side.
    let twilight = [6.0f32, 18.0]
        .iter()
        .map(|&t| (1.0 - ((h - t).abs().min(24.0 - (h - t).abs())) / 2.5).max(0.0))
        .fold(0.0, f32::max);
    match active {
        Active::Day => 0.15 + 0.85 * day,
        Active::Night => 0.15 + 0.85 * (1.0 - day),
        Active::Twilight => (0.35 + 0.65 * twilight).max(0.35),
        Active::AnyTime => 0.75,
    }
}

/// How much more resting scores at no activity (`needs::REST` plus this times `1 - activity`).
pub const SLEEPY: f32 = 0.35;
/// The fewest of its usual rests it takes in its busiest hours, and the most out of them (times
/// its kind's chance to lie down, by `1 - activity`).
pub const REST_BUSY: f32 = 0.6;
pub const REST_SLEEPY: f32 = 4.0;
/// Hours of the dawn and dusk drink (from, to), and the share of its water under which a plant
/// eater goes to drink then.
pub const DAWN: (f32, f32) = (5.0, 8.0);
pub const DUSK: (f32, f32) = (17.0, 20.0);
pub const DRINKS_AT: f32 = 0.8;
/// Activity under which a hunter rests rather than hunts, unless its food is under this share.
pub const HUNTS_FROM: f32 = 0.45;
pub const HUNTS_ANYWAY: f32 = 0.35;
/// Lying down, it sleeps (eyes shut, head on its paws: `anim::Act::Sleep`) out of its hours (its
/// activity under this, as a hunter rests rather than hunts), or once it has lain this many ticks
/// (a minute: of a deer's 30 s to 2 min rests, most end asleep).
pub const ASLEEP_BELOW: f32 = HUNTS_FROM;
pub const DOZES_AFTER: u32 = 1200;

/// Whether one lying down `rested` ticks at `activity` is asleep.
pub fn asleep(activity: f32, rested: u32) -> bool {
    activity < ASLEEP_BELOW || rested >= DOZES_AFTER
}

/// Whether `hour` is dawn's or dusk's drinking time.
pub fn drinking_hour(hour: f32) -> bool {
    (DAWN.0..DAWN.1).contains(&hour) || (DUSK.0..DUSK.1).contains(&hour)
}

/// The hour (0..24) at `day_time` ticks (24,000 a day, 0 at 06:00).
pub fn hour(day_time: u64) -> f32 {
    ((day_time % 24_000) as f32 / 1000.0 + 6.0) % 24.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_in_its_hours() {
        assert!(activity(Active::Day, 12.0) > 0.9 && activity(Active::Day, 0.0) < 0.2);
        assert!(activity(Active::Night, 0.0) > 0.9 && activity(Active::Night, 12.0) < 0.2);
        assert!(activity(Active::Twilight, 18.0) > 0.95 && activity(Active::Twilight, 12.0) < 0.4);
        assert_eq!(hour(0), 6.0);
        assert!(drinking_hour(18.5) && !drinking_hour(12.0));
    }
}

// ------------------------------------------------------------------ AI level of detail

/// How much an animal's senses work by where the players are (`reports/Creature AI.md`, part 3,
/// step 8 of its plan; the crew's step 9): spend detail where a mistake would be seen (the LOD
/// trader). `Full` within `FULL_WITHIN` blocks of a player or in a player's view within
/// `SEEN_WITHIN` (`VIEW` degrees round where they look); `Coarse` out of view within
/// `COARSE_WITHIN` (the engine's activation distance): it looks round a quarter as often and
/// takes no line of sight; `Far` past that (the engine rests it): an eighth, coarse.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Detail {
    Full,
    Coarse,
    Far,
}

pub const FULL_WITHIN: f64 = 32.0;
pub const SEEN_WITHIN: f64 = 64.0;
pub const COARSE_WITHIN: f64 = 48.0;
pub const VIEW: f32 = 140.0;

impl Detail {
    /// Ticks between its looks round.
    pub fn every(self) -> u64 {
        match self {
            Detail::Full => 10,
            Detail::Coarse => 40,
            Detail::Far => 80,
        }
    }
}

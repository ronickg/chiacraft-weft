//! The animals under the sky's events (`vitals::events`, `reports/Research round 17.md` §1
//! "Animals"). In a blood moon's totality (22:00-02:00) herds bunch closer and watch farther for
//! hunters, and wolves howl once as it begins. On every difficulty: on gentle the blood moon brings
//! no monsters, but the animals still feel it.

use vitals::calendar::DAY_TICKS;
use vitals::events::{self, Event};

/// How much farther prey watch for hunters in totality (`hunt::wary`'s sight).
pub const BLOOD_MOON_WATCH: f32 = 1.5;
/// Blocks from the middle of its herd a herd animal comes back within in totality, and how far it
/// looks for its herd.
pub const HUDDLE: f32 = 4.0;
pub const HUDDLE_LOOK: f32 = 24.0;
/// Hours after totality begins in which a wolf still howls its one howl (one loaded later stays
/// quiet).
pub const HOWL_HOURS: f32 = 0.5;

/// Whether `day_time` (ticks since the world began) is in a blood moon's totality in world `seed`.
/// The whole night lies in one game day (06:00 to 06:00), so the day is the night's.
pub fn totality(seed: u64, day_time: u64) -> bool {
    events::in_totality(events::hour_of(day_time)) && events::tonight(seed, day_time / DAY_TICKS) == Some(Event::BloodMoon)
}

/// Whether a wolf howls now: in totality's first `HOWL_HOURS`.
pub fn howl_time(seed: u64, day_time: u64) -> bool {
    let into = (events::hour_of(day_time) - events::BLOOD_MOON_HOURS.0).rem_euclid(24.0);
    into < HOWL_HOURS && totality(seed, day_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first blood moon of a seed, its day.
    fn first_blood_moon(seed: u64) -> u64 {
        (0..2000)
            .find(|&d| events::tonight(seed, d) == Some(Event::BloodMoon))
            .expect("a blood moon in 2000 days")
    }

    #[test]
    fn totality_is_the_blood_moons_middle_hours() {
        let day = first_blood_moon(1);
        let at = |h: f32| day * DAY_TICKS + vitals::calendar::ticks_at_hour(h);
        assert!(!totality(1, at(21.5)), "before totality");
        assert!(totality(1, at(22.5)));
        assert!(totality(1, at(1.5)), "after midnight, the same night");
        assert!(!totality(1, at(2.5)));
        assert!(!totality(1, (day + 1) * DAY_TICKS + vitals::calendar::ticks_at_hour(23.0)), "the next night is no blood moon");
        assert!(howl_time(1, at(22.2)) && !howl_time(1, at(23.0)));
    }
}

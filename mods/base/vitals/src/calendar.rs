//! The calendar: a year of four seasons (spring first), from the world's day count. The engine
//! makes the days longer in summer by the same year (`.seasons(..)` on a dimension); here is what
//! day of which season it is, and how far the year is from midwinter for the temperature.

/// Days in a season and in a year (Ronald: fourteen days a season, a 56-day year).
pub const SEASON_DAYS: u32 = 14;
pub const YEAR_DAYS: u32 = SEASON_DAYS * 4;
/// The day a new world begins on: spring's 8th, from midwinter a quarter of the year (Research
/// round 3, B1: starting a week after midwinter froze new players in six biomes of ten). The world
/// simply begins later (`survival` moves a new world's clock there), so every calendar agrees.
pub const START_DAY: u32 = 7;
/// Ticks in a day, the game's.
pub const DAY_TICKS: u64 = 24000;
/// The hour a new world begins at (Ronald, 11-04: "10:00 start + slow 1st day"): at 07:00 the
/// first-day playtest's spawn was 3-7 °C; the day's swing and the sun give a birch forest about
/// 5 °C more by 10:00 (`temperature::air`).
pub const START_HOUR: f32 = 10.0;
/// How fast the first day's clock runs until its dusk (`FIRST_DAY_UNTIL`): half speed, so the
/// eight hours from 10:00 take about 13 real minutes, not 6.7 (a game hour is 50 s at 1).
pub const FIRST_DAY_RATE: f32 = 0.5;
pub const FIRST_DAY_UNTIL: f32 = 18.0;

/// Ticks into the day (0 at 06:00, the game's sunrise) of hour `h`.
pub fn ticks_at_hour(h: f32) -> u64 {
    (((h - 6.0).rem_euclid(24.0)) * DAY_TICKS as f32 / 24.0).round() as u64
}

/// How fast the clock runs at `day_time` (ticks since the world's day 0): `FIRST_DAY_RATE` on a
/// new world's first day (`START_DAY`) until `FIRST_DAY_UNTIL`, else 1.
pub fn day_rate(day_time: u64) -> f32 {
    let day = (day_time / DAY_TICKS) as u32;
    let hour = 6.0 + (day_time % DAY_TICKS) as f32 * 24.0 / DAY_TICKS as f32;
    if day == START_DAY && hour < FIRST_DAY_UNTIL { FIRST_DAY_RATE } else { 1.0 }
}
/// Midsummer, three eighths into the year (the middle of summer), as the engine's day length has
/// it; midwinter half a year later.
pub const MIDSUMMER: f32 = YEAR_DAYS as f32 * 3.0 / 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

pub const SEASONS: [Season; 4] = [Season::Spring, Season::Summer, Season::Autumn, Season::Winter];

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }

    pub fn index(self) -> u32 {
        self as u32
    }

    /// For a world with a wet and a dry season instead (the Cretaceous): the wet half is spring
    /// and summer.
    pub fn wet(self) -> bool {
        matches!(self, Season::Spring | Season::Summer)
    }
}

/// A day of the calendar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Date {
    /// Years since the world began, from 0.
    pub year: u32,
    /// Day of the year, from 0.
    pub day: u32,
    pub season: Season,
    /// Day of the season, from 1.
    pub day_of_season: u32,
    /// The day's fraction gone, 0 at 06:00 (the game's day starts at sunrise).
    pub fraction: f32,
}

/// The date at the world's day time (`World::day_time`, ticks since the world began).
pub fn date(day_time: u64) -> Date {
    let days = day_time / DAY_TICKS;
    let day = (days % YEAR_DAYS as u64) as u32;
    Date {
        year: (days / YEAR_DAYS as u64) as u32,
        day,
        season: SEASONS[(day / SEASON_DAYS) as usize],
        day_of_season: day % SEASON_DAYS + 1,
        fraction: (day_time % DAY_TICKS) as f32 / DAY_TICKS as f32,
    }
}

impl Date {
    /// How far the year has gone from midwinter, 0 to 1 (0.5 midsummer), for
    /// `temperature::season`.
    pub fn from_midwinter(&self) -> f32 {
        from_midwinter(self.day as f32 + self.fraction)
    }
}

/// How far day `day` of the year (from 0, with its fraction) is from midwinter, 0 to 1.
pub fn from_midwinter(day: f32) -> f32 {
    let midwinter = MIDSUMMER + YEAR_DAYS as f32 / 2.0;
    ((day - midwinter) / YEAR_DAYS as f32).rem_euclid(1.0)
}

/// Hours of daylight on day `day` of the year (with its fraction) where midsummer's daylight is
/// `swing` of the day longer than half (as the engine's `.seasons(..)`): sunrise is half of it
/// before noon.
pub fn daylight_hours(day: f32, swing: f32) -> f32 {
    24.0 * (0.5 + swing * (std::f32::consts::TAU * (day - MIDSUMMER) / YEAR_DAYS as f32).cos())
}

/// The day time on day `day` of the year (from 0), in the same year and at the same hour as
/// `now`: for jumping to a season.
pub fn on_day(now: u64, day: u32) -> u64 {
    let year = now / DAY_TICKS / YEAR_DAYS as u64;
    (year * YEAR_DAYS as u64 + (day % YEAR_DAYS) as u64) * DAY_TICKS + now % DAY_TICKS
}

#[cfg(test)]
mod first_day_tests {
    use super::*;

    #[test]
    fn a_new_world_starts_at_ten_and_its_first_day_runs_slow_until_dusk() {
        assert_eq!(ticks_at_hour(6.0), 0);
        assert_eq!(ticks_at_hour(START_HOUR), 4000, "10:00 is four hours after sunrise");
        let start = START_DAY as u64 * DAY_TICKS + ticks_at_hour(START_HOUR);
        assert_eq!(day_rate(start), FIRST_DAY_RATE);
        assert_eq!(day_rate(START_DAY as u64 * DAY_TICKS + ticks_at_hour(17.9)), FIRST_DAY_RATE);
        assert_eq!(day_rate(START_DAY as u64 * DAY_TICKS + ticks_at_hour(FIRST_DAY_UNTIL)), 1.0, "from dusk the usual pace");
        assert_eq!(day_rate(start + DAY_TICKS), 1.0, "the second day as any other");
        assert_eq!(day_rate(0), 1.0, "a world without the calendar's start");
        // About 13 real minutes from 10:00 to 18:00 (20 ticks a second).
        let real_min = (ticks_at_hour(FIRST_DAY_UNTIL) - ticks_at_hour(START_HOUR)) as f32 / FIRST_DAY_RATE / 20.0 / 60.0;
        assert!((13.0..14.0).contains(&real_min), "{real_min} min");
    }
}

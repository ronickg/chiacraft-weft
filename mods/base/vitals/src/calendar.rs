//! The calendar: a year of four seasons (spring first), from the world's day count. The engine
//! makes the days longer in summer by the same year (`.seasons(..)` on a dimension); here is what
//! day of which season it is, and how far the year is from midwinter for the temperature.

/// Days in a season and in a year (Ronald: fourteen days a season, a 56-day year).
pub const SEASON_DAYS: u32 = 14;
pub const YEAR_DAYS: u32 = SEASON_DAYS * 4;
/// Ticks in a day, the game's.
pub const DAY_TICKS: u64 = 24000;
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
        let midwinter = MIDSUMMER + YEAR_DAYS as f32 / 2.0;
        ((self.day as f32 + self.fraction - midwinter) / YEAR_DAYS as f32).rem_euclid(1.0)
    }
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

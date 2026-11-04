//! Crafts that wait (`notes/CONTENT.md` decision 6, taken overnight 11-04): what takes days in
//! real life takes its time in the game, on the growing pace crops and young already use
//! (`plants::GROWING`: game days = 0.0645 · real days^0.7). A brain-tanned hide is ready while
//! one eats; bark-tanned sole leather takes most of a game week. Each happens at a station that
//! holds the work and finishes it whether or not anyone watches (`mods/base/src/crafts.rs`).

use crate::DAY_SECS;
use crate::plants::GROWING;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Station {
    /// A pole frame a hide is laced into, scraped, brained and smoked.
    TanningFrame,
    /// A pit or vat of oak-bark liquor hides soak in for months.
    TanningPit,
    /// A mound of logs under earth, lit and left to char.
    CharcoalClamp,
}

pub const STATIONS: [Station; 3] = [Station::TanningFrame, Station::TanningPit, Station::CharcoalClamp];

impl Station {
    /// Its block (`base:tanning_frame`) and translations.
    pub fn name(self) -> &'static str {
        match self {
            Station::TanningFrame => "tanning_frame",
            Station::TanningPit => "tanning_pit",
            Station::CharcoalClamp => "charcoal_clamp",
        }
    }

    /// How many it works at once: a frame holds one hide, a pit a batch, a clamp a mound of logs.
    pub fn holds(self) -> u32 {
        match self {
            Station::TanningFrame => 1,
            Station::TanningPit => 8,
            Station::CharcoalClamp => 16,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Craft {
    pub station: Station,
    /// What goes in, an item or a `#tag`.
    pub input: &'static str,
    /// What it's worked with, one for each input (the pit's bark), if anything.
    pub with: Option<&'static str>,
    /// What comes out, one for each input.
    pub output: &'static str,
    /// How long it takes in real life, days (the content research's, `reports/content/Building and
    /// workstations.md` C and D: brain tan "days of hard work", a charcoal clamp 3 to 14 days, bark
    /// tanning 6 to 12 months).
    pub real_days: f32,
}

pub const CRAFTS: [Craft; 3] = [
    Craft { station: Station::TanningFrame, input: "#hides", with: None, output: "base:leather", real_days: 3.0 },
    Craft { station: Station::TanningPit, input: "#hides", with: Some("#bark_tannin"), output: "base:sole_leather", real_days: 365.0 },
    Craft { station: Station::CharcoalClamp, input: "#logs", with: None, output: "base:coal", real_days: 7.0 },
];

/// The craft a station does.
pub fn craft(station: Station) -> &'static Craft {
    CRAFTS
        .iter()
        .find(|c| c.station == station)
        .unwrap_or(&CRAFTS[0])
}

impl Craft {
    /// Seconds it takes in the game, on the growing pace.
    pub fn secs(&self) -> f32 {
        GROWING.0 * self.real_days.powf(GROWING.1) * DAY_SECS
    }
}

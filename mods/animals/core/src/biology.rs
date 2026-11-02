//! Species' numbers from a few real facts (`notes/ANIMALS.md`, "Balance"): a species is described
//! as life has it (its mass, how long it carries or incubates, grows up, lives and waits between
//! litters, in real days; its litter; whether it lives alone, in a pack or a herd; how rare it
//! is), and the game's numbers follow by one pacing for all of them. So a new animal is a row of
//! facts, its numbers fall in line with every other's, and the balance checks (`web/test.mjs`, the
//! lab's balance table) catch one that doesn't. A row may still set any number itself (`Tune`).
//!
//! The pacing: game days = a · real days^k for each kind of span. Life's spans scale with mass by
//! quarter powers (lifespan, maturity ~ M^1/4; Kleiber); games squash the gaps between species
//! (a dragon mustn't take a hundred times a dog's time) and compress short spans less than long
//! ones. The constants are fitted to what the balance research recommends for our first seven
//! species (pets carrying 1-1.5 days, growing up in 2.5-4, living 40-60; wyverns 5-7, 25-40, 400+).

use crate::life::DAY;

/// How an animal lives with its own kind.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Social {
    Solitary,
    Pack,
    Herd,
}

/// How rare a species is: what it may cost a player in time.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Social {
    /// How much more room its kind take near: a pack's and a herd's more.
    /// How many are met together in the wild.
    pub fn group(self) -> (u32, u32) {
        match self {
            Social::Solitary => (1, 1),
            Social::Pack => (2, 3),
            Social::Herd => (3, 5),
        }
    }
    pub fn room(self) -> f32 {
        match self {
            Social::Solitary => 1.0,
            Social::Pack => 1.2,
            Social::Herd => 2.0,
        }
    }
}

impl Rarity {
    /// The chance a natural spawn attempt that picked it brings it, where it is commonest.
    pub fn spawn_chance(self) -> f32 {
        match self {
            Rarity::Common => 0.02,
            Rarity::Uncommon => 0.008,
            Rarity::Rare => 0.004,
            Rarity::Legendary => 0.0006,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Legendary => "legendary",
        }
    }
}

/// A species as life has it (a fantasy one as it would be).
#[derive(Clone, Copy, Debug)]
pub struct Biology {
    /// A grown one's mass, kg.
    pub mass: f32,
    /// Real days it carries its young; 0: it lays eggs.
    pub gestation: f32,
    /// Real days its eggs take to hatch (an egg-layer's).
    pub incubation: f32,
    /// Real days to grow up (to breed).
    pub maturity: f32,
    /// Real days it lives, on average.
    pub lifespan: f32,
    /// Real days between one litter and the next.
    pub interval: f32,
    /// Young a litter or clutch, on average.
    pub litter: f32,
    pub social: Social,
    pub rarity: Rarity,
}

/// Numbers a species sets itself instead of deriving them (`None`: derived).
#[derive(Clone, Copy, Debug, Default)]
pub struct Tune {
    pub gestation: Option<u32>,
    pub incubation: Option<u32>,
    pub recovery: Option<u32>,
    pub growth: Option<u32>,
    pub lifespan: Option<u32>,
    pub crowd: Option<u32>,
    pub litter: Option<(u32, u32)>,
}

impl Tune {
    pub const NONE: Tune =
        Tune { gestation: None, incubation: None, recovery: None, growth: None, lifespan: None, crowd: None, litter: None };
}

/// The pacing: (a, k) of game days = a · real days^k, per kind of span.
pub const CARRYING: (f64, f64) = (0.056, 0.75);
pub const GROWING: (f64, f64) = (0.0645, 0.7);
pub const LIVING: (f64, f64) = (0.0105, 1.0);
pub const RESTING: (f64, f64) = (0.0139, 1.0);

/// The room near (`Biology::crowd`): for a 4 kg animal living alone, and how it goes with mass.
pub const ROOM: f32 = 13.0;
pub const ROOM_BY_MASS: f32 = -0.2;

/// A real span of `days` as the game's ticks, by `pace`.
pub fn span(pace: (f64, f64), days: f32) -> u32 {
    if days <= 0.0 {
        return 0;
    }
    (pace.0 * (days as f64).powf(pace.1) * DAY as f64).round() as u32
}

/// Its numbers as derived from its facts (each a row's own if it sets one).
impl Biology {
    pub fn gestation(&self) -> u32 {
        span(CARRYING, self.gestation)
    }
    pub fn incubation(&self) -> u32 {
        span(CARRYING, self.incubation)
    }
    pub fn recovery(&self) -> u32 {
        span(RESTING, self.interval)
    }
    pub fn growth(&self) -> u32 {
        span(GROWING, self.maturity)
    }
    pub fn lifespan(&self) -> u32 {
        span(LIVING, self.lifespan)
    }
    /// The fewest and most young a litter, round its mean.
    pub fn litter(&self) -> (u32, u32) {
        let lo = (self.litter * 0.4).round().max(1.0) as u32;
        let hi = (self.litter * 1.4).round().max(lo as f32) as u32;
        (lo, hi)
    }
    /// How many of its kind may live within `life::CROWD_RADIUS` blocks: fewer the bigger it is
    /// (land per animal grows with mass), more for packs and herds. No fewer than a group left to
    /// itself needs to last (a minimum viable population: run alone in the simulator, a cat's
    /// group of 8 died out within 150 days in half the runs, of 12 in none; a Utahraptor's of 4
    /// in half, of 6 in one in eight).
    pub fn crowd(&self) -> u32 {
        (ROOM * (self.mass / 4.0).powf(ROOM_BY_MASS) * self.social.room())
            .round()
            .max(3.0) as u32
    }
}

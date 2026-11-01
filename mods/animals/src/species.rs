//! Every species as data (`notes/ANIMALS.md`, "Species are data"): adding an animal is mostly a
//! row here. Only what the core uses is filled in so far; taming, breeding and riding add their
//! own fields when they are built.

use weft_sdk::Entity;

/// The nine stats, in the order of every `[_; STATS]` table (`STAT_NAMES`); those the core uses
/// by name.
pub const STATS: usize = 9;
pub const HEALTH: usize = 0;
pub const FOOD: usize = 2;
pub const DAMAGE: usize = 3;
pub const SPEED: usize = 4;
pub const TORPOR: usize = 7;
/// Their names in translations (`stat.animals.<name>`).
pub const STAT_NAMES: [&str; STATS] =
    ["health", "stamina", "food", "damage", "speed", "weight", "oxygen", "torpor", "fortitude"];

#[derive(Clone, Copy, PartialEq)]
pub enum Size {
    Small,
    Medium,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Diet {
    Carnivore,
    Herbivore,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Taming {
    Passive,
    Knockout,
}

pub struct Species {
    pub kind: &'static Entity,
    /// `entity.animals.<name>`, `species.animals.<name>.*`.
    pub name: &'static str,
    pub size: Size,
    pub diet: Diet,
    pub taming: Taming,
    /// Each stat's value at level 1 (damage and speed as the kind's own: 100%).
    pub base: [f32; STATS],
    /// What a point adds, as a share of the base: a wild point, a tamed point.
    pub wild_step: [f32; STATS],
    pub tamed_step: [f32; STATS],
    /// Seconds a tame takes to use one point of food.
    pub hunger: f32,
}

/// ARK's shape: health grows fastest, speed slowest, torpor only from wild points.
const WILD_STEP: [f32; STATS] = [0.2, 0.1, 0.1, 0.05, 0.01, 0.02, 0.1, 0.06, 0.05];
const TAMED_STEP: [f32; STATS] = [0.1, 0.1, 0.1, 0.04, 0.015, 0.04, 0.1, 0.0, 0.05];

pub static SPECIES: [Species; 4] = [
    Species {
        kind: &crate::CHIHUAHUA,
        name: "chihuahua",
        size: Size::Small,
        diet: Diet::Carnivore,
        taming: Taming::Passive,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [8.0, 100.0, 80.0, 2.0, 1.0, 40.0, 100.0, 60.0, 10.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
    },
    Species {
        kind: &crate::cat::CAT,
        name: "cat",
        size: Size::Small,
        diet: Diet::Carnivore,
        taming: Taming::Passive,
        base: [6.0, 80.0, 60.0, 1.0, 1.0, 30.0, 80.0, 40.0, 10.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 40.0,
    },
    Species {
        kind: &crate::capybara::CAPYBARA,
        name: "capybara",
        size: Size::Medium,
        diet: Diet::Herbivore,
        taming: Taming::Passive,
        // It holds its breath for five minutes.
        base: [12.0, 150.0, 200.0, 1.0, 1.0, 150.0, 400.0, 150.0, 20.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 25.0,
    },
    Species {
        kind: &crate::raptor::RAPTOR,
        name: "raptor",
        size: Size::Medium,
        diet: Diet::Carnivore,
        taming: Taming::Knockout,
        base: [14.0, 150.0, 150.0, 4.0, 1.0, 140.0, 150.0, 200.0, 15.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 20.0,
    },
];


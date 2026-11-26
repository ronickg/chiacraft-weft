//! Effects on a player or animal, what each changes, and the kit's icon for it
//! (`textures/gui/status/`).

use crate::temperature::Feeling;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    // Weather
    Wet,
    Chilly,
    Cold,
    Freezing,
    Warm,
    Hot,
    Heatstroke,
    // Body
    Poisoned,
    Bleeding,
    FoodPoisoning,
    Sedated,
    // Load
    Encumbered,
    Overloaded,
    // Camp
    NearFire,
    Sheltered,
    Rested,
    WellFed,
}

pub const ALL: [Effect; 17] = [
    Effect::Wet,
    Effect::Chilly,
    Effect::Cold,
    Effect::Freezing,
    Effect::Warm,
    Effect::Hot,
    Effect::Heatstroke,
    Effect::Poisoned,
    Effect::Bleeding,
    Effect::FoodPoisoning,
    Effect::Sedated,
    Effect::Encumbered,
    Effect::Overloaded,
    Effect::NearFire,
    Effect::Sheltered,
    Effect::Rested,
    Effect::WellFed,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Weather,
    Body,
    Load,
    Camp,
}

/// What effects change: multipliers (1 nothing) and steady losses as shares of the max a second.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Modifiers {
    pub health_regen: f32,
    pub stamina_regen: f32,
    /// Share of max health lost a second (0 none).
    pub health_loss: f32,
    pub stamina_loss: f32,
    pub speed: f32,
    /// Stamina every action costs.
    pub stamina_cost: f32,
    pub xp: f32,
    /// Water used (digested) faster.
    pub water_use: f32,
    pub can_sprint: bool,
}

pub const NONE: Modifiers = Modifiers {
    health_regen: 1.0,
    stamina_regen: 1.0,
    health_loss: 0.0,
    stamina_loss: 0.0,
    speed: 1.0,
    stamina_cost: 1.0,
    xp: 1.0,
    water_use: 1.0,
    can_sprint: true,
};

impl Effect {
    pub fn index(self) -> usize {
        ALL.iter().position(|&e| e == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Effect::Wet => "wet",
            Effect::Chilly => "chilly",
            Effect::Cold => "cold",
            Effect::Freezing => "freezing",
            Effect::Warm => "warm",
            Effect::Hot => "hot",
            Effect::Heatstroke => "heatstroke",
            Effect::Poisoned => "poisoned",
            Effect::Bleeding => "bleeding",
            Effect::FoodPoisoning => "food_poisoning",
            Effect::Sedated => "sedated",
            Effect::Encumbered => "encumbered",
            Effect::Overloaded => "overloaded",
            Effect::NearFire => "near_fire",
            Effect::Sheltered => "sheltered",
            Effect::Rested => "rested",
            Effect::WellFed => "well_fed",
        }
    }

    pub fn group(self) -> Group {
        match self {
            Effect::Wet | Effect::Chilly | Effect::Cold | Effect::Freezing | Effect::Warm | Effect::Hot | Effect::Heatstroke => {
                Group::Weather
            }
            Effect::Poisoned | Effect::Bleeding | Effect::FoodPoisoning | Effect::Sedated => Group::Body,
            Effect::Encumbered | Effect::Overloaded => Group::Load,
            Effect::NearFire | Effect::Sheltered | Effect::Rested | Effect::WellFed => Group::Camp,
        }
    }

    /// The kit's status icon (`kit/status/<icon>.png`, `kit/temperature/<icon>.png`).
    pub fn icon(self) -> &'static str {
        match self {
            Effect::Wet => "status/wet",
            Effect::Chilly => "temperature/cold",
            Effect::Cold => "temperature/cold",
            Effect::Freezing => "temperature/freezing",
            Effect::Warm => "temperature/warm",
            Effect::Hot => "temperature/hot",
            Effect::Heatstroke => "status/heat",
            Effect::Poisoned => "status/poison",
            Effect::Bleeding => "status/bleeding",
            Effect::FoodPoisoning => "status/poison",
            Effect::Sedated => "status/rested",
            Effect::Encumbered | Effect::Overloaded => "status/encumbered",
            Effect::NearFire => "status/heat",
            Effect::Sheltered => "status/wind",
            Effect::Rested => "status/rested",
            Effect::WellFed => "status/food",
        }
    }

    /// What it changes (proposed numbers; `notes/SURVIVAL.md`). Losses are a share of the max a
    /// second: freezing takes a full player in about three minutes.
    pub fn modifiers(self) -> Modifiers {
        let m = NONE;
        match self {
            Effect::Wet => Modifiers { health_regen: 0.75, stamina_regen: 0.75, ..m },
            Effect::Chilly => Modifiers { stamina_regen: 0.75, ..m },
            Effect::Cold => Modifiers { health_regen: 0.0, stamina_regen: 0.5, ..m },
            Effect::Freezing => Modifiers { health_regen: 0.0, stamina_regen: 0.5, health_loss: 0.006, speed: 0.85, ..m },
            Effect::Warm => Modifiers { water_use: 1.5, ..m },
            Effect::Hot => Modifiers { stamina_regen: 0.75, water_use: 2.0, ..m },
            Effect::Heatstroke => Modifiers { stamina_regen: 0.0, stamina_loss: 0.03, health_loss: 0.003, water_use: 3.0, ..m },
            Effect::Poisoned => Modifiers { health_regen: 0.0, health_loss: 0.004, ..m },
            Effect::Bleeding => Modifiers { health_regen: 0.0, health_loss: 0.006, ..m },
            Effect::FoodPoisoning => Modifiers { health_regen: 0.0, stamina_regen: 0.5, speed: 0.9, ..m },
            Effect::Sedated => Modifiers { stamina_regen: 0.5, speed: 0.5, can_sprint: false, ..m },
            Effect::Encumbered => Modifiers { speed: 0.75, stamina_cost: 1.5, ..m },
            Effect::Overloaded => Modifiers { speed: 0.5, stamina_cost: 2.0, can_sprint: false, ..m },
            Effect::NearFire => Modifiers { health_regen: 1.25, ..m },
            Effect::Sheltered => m,
            Effect::Rested => Modifiers { stamina_regen: 1.5, xp: 1.25, ..m },
            Effect::WellFed => Modifiers { health_regen: 1.1, stamina_regen: 1.1, ..m },
        }
    }
}

/// All effects together: multipliers multiply, losses add.
pub fn combine(effects: impl IntoIterator<Item = Effect>) -> Modifiers {
    effects
        .into_iter()
        .map(Effect::modifiers)
        .fold(NONE, |a, b| Modifiers {
            health_regen: a.health_regen * b.health_regen,
            stamina_regen: a.stamina_regen * b.stamina_regen,
            health_loss: a.health_loss + b.health_loss,
            stamina_loss: a.stamina_loss + b.stamina_loss,
            speed: a.speed * b.speed,
            stamina_cost: a.stamina_cost * b.stamina_cost,
            xp: a.xp * b.xp,
            water_use: a.water_use * b.water_use,
            can_sprint: a.can_sprint && b.can_sprint,
        })
}

/// The weather effect a core temperature brings.
pub fn from_feeling(f: Feeling) -> Option<Effect> {
    match f {
        Feeling::Freezing => Some(Effect::Freezing),
        Feeling::Cold => Some(Effect::Cold),
        Feeling::Chilly => Some(Effect::Chilly),
        Feeling::Comfortable => None,
        Feeling::Warm => Some(Effect::Warm),
        Feeling::Hot => Some(Effect::Hot),
        Feeling::Heatstroke => Some(Effect::Heatstroke),
    }
}

/// The load effect of carrying `kg` with that weight stat: encumbered over it, overloaded (no
/// sprinting, half speed) past half again.
pub fn load(kg: f32, weight: f32) -> Option<Effect> {
    if kg > weight * 1.5 {
        Some(Effect::Overloaded)
    } else if kg > weight {
        Some(Effect::Encumbered)
    } else {
        None
    }
}

/// Seconds rested lasts: eight minutes, and a minute more for each comfort level (Valheim's).
pub fn rested_secs(comfort: u32) -> f32 {
    480.0 + 60.0 * comfort as f32
}

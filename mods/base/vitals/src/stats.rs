//! The nine stats, players' and animals' alike (the order of the animals' `STAT_NAMES`), and what
//! points make of them.

pub const STATS: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    /// The ceiling of max health.
    Health,
    /// The ceiling of max stamina.
    Stamina,
    /// The stomach's size, in units (a unit is 10 g of food).
    Food,
    /// Melee and tool damage, 1 = 100%.
    Damage,
    /// Walking and running, 1 = 100%.
    Speed,
    /// Kilograms carried before encumbered.
    Weight,
    /// Breath under water.
    Oxygen,
    /// How much it takes to be knocked out (sedatives, blows).
    Torpor,
    /// Widens the comfort range (half a degree each way a point) and resists poison and
    /// sedation (2% a point).
    Fortitude,
}

pub const ALL: [Stat; STATS] =
    [Stat::Health, Stat::Stamina, Stat::Food, Stat::Damage, Stat::Speed, Stat::Weight, Stat::Oxygen, Stat::Torpor, Stat::Fortitude];
pub const NAMES: [&str; STATS] = ["health", "stamina", "food", "damage", "speed", "weight", "oxygen", "torpor", "fortitude"];

impl Stat {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        NAMES[self.index()]
    }
}

/// How points grow a stat: by a share of its base each, or by a fixed amount (fortitude, whose
/// base is 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Growth {
    Share(f32),
    Add(f32),
}

impl Growth {
    pub fn value(self, base: f32, points: u32) -> f32 {
        match self {
            Growth::Share(s) => base * (1.0 + points as f32 * s),
            Growth::Add(a) => base + points as f32 * a,
        }
    }
}

/// A person at level 1, and what a point adds (proposed: ARK's player, +10% health a point).
pub const PLAYER_BASE: [f32; STATS] = [100.0, 100.0, 100.0, 1.0, 1.0, 100.0, 100.0, 200.0, 0.0];
pub const PLAYER_GROWTH: [Growth; STATS] = [
    Growth::Share(0.10),
    Growth::Share(0.10),
    Growth::Share(0.10),
    Growth::Share(0.05),
    Growth::Share(0.02),
    Growth::Share(0.10),
    Growth::Share(0.20),
    Growth::Share(0.10),
    Growth::Add(1.0),
];

/// Points placed in each stat.
pub type Points = [u32; STATS];

/// Every stat's value from the points placed.
pub fn values(base: &[f32; STATS], growth: &[Growth; STATS], points: &Points) -> [f32; STATS] {
    std::array::from_fn(|i| growth[i].value(base[i], points[i]))
}

/// A player's stats.
pub fn player(points: &Points) -> [f32; STATS] {
    values(&PLAYER_BASE, &PLAYER_GROWTH, points)
}

/// Degrees the comfort range widens each way, from fortitude.
pub fn fortitude_degrees(fortitude: f32) -> f32 {
    fortitude * 0.5
}

/// The share of poison and sedation resisted, from fortitude (at most 60%).
pub fn fortitude_resists(fortitude: f32) -> f32 {
    (fortitude * 0.02).min(0.6)
}

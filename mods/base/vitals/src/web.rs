//! The perk web's budget (`notes/PROGRESSION.md`, "Skills and perks"; `reports/Balance model.md`,
//! 5.2): one Path of Exile-style web, priced in units. A small node is one unit (+2 % to one
//! family, like melee damage or taming speed), a notable three to four for its one point plus the
//! walk to it, a keystone nets nothing (its power is paid for by its cost). Bonuses add inside a
//! family and multiply between families; a fully specialised region is worth at most one age in
//! its speciality, and everything together at most `curve::TOTAL_CEILING`.
//!
//! No node exists yet (11-07): this is the price list new nodes are checked against.

use crate::curve::{GROWTH, TOTAL_CEILING};

/// What one unit adds to its family.
pub const UNIT: f32 = 0.02;
/// The most one family reaches from the web: one age's growth.
pub const FAMILY_CEILING: f32 = GROWTH;
/// Units that reach it (30: a region's depth of small nodes and notables).
pub fn units_to_ceiling() -> f32 {
    (FAMILY_CEILING - 1.0) / UNIT
}
/// Perk points a character level gives.
pub const POINTS_PER_LEVEL: u32 = 1;

/// A node's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    Small,
    Notable,
    /// Changes how you play, with a cost (your tames fight twice as hard, but no heavy armour).
    Keystone,
}

impl Node {
    /// The units it may give, fewest and most (a keystone's power minus its cost).
    pub const fn units(self) -> (f32, f32) {
        match self {
            Node::Small => (1.0, 1.0),
            Node::Notable => (3.0, 4.0),
            Node::Keystone => (0.0, 0.0),
        }
    }
}

/// The families bonuses stack in: additive inside one, multiplying between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    MeleeDamage,
    RangedDamage,
    SpellDamage,
    Defence,
    Health,
    TameDamage,
    TamingSpeed,
    CraftGrade,
    GatherSpeed,
    MineSpeed,
    MoveSpeed,
    ManaCost,
}

pub const FAMILIES: [Family; 12] = [
    Family::MeleeDamage,
    Family::RangedDamage,
    Family::SpellDamage,
    Family::Defence,
    Family::Health,
    Family::TameDamage,
    Family::TamingSpeed,
    Family::CraftGrade,
    Family::GatherSpeed,
    Family::MineSpeed,
    Family::MoveSpeed,
    Family::ManaCost,
];

impl Family {
    pub const fn name(self) -> &'static str {
        match self {
            Family::MeleeDamage => "melee damage",
            Family::RangedDamage => "ranged damage",
            Family::SpellDamage => "spell damage",
            Family::Defence => "defence",
            Family::Health => "health",
            Family::TameDamage => "tame damage",
            Family::TamingSpeed => "taming speed",
            Family::CraftGrade => "craft grade",
            Family::GatherSpeed => "gathering speed",
            Family::MineSpeed => "mining speed",
            Family::MoveSpeed => "move speed",
            Family::ManaCost => "mana cost",
        }
    }
}

/// What `units` in one family multiply it by, capped at one age.
pub fn family(units: f32) -> f32 {
    (1.0 + UNIT * units.max(0.0)).min(FAMILY_CEILING)
}

/// Multipliers from different sources (gear, web, a tame's bond, food) together, capped at
/// `TOTAL_CEILING`: they multiply, but never past two ages.
pub fn together(times: &[f32]) -> f32 {
    times.iter().product::<f32>().min(TOTAL_CEILING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_tops_out_at_one_age() {
        assert!((units_to_ceiling() - 30.0).abs() < 1e-4);
        assert!((family(30.0) - 1.6).abs() < 1e-5);
        assert_eq!(family(100.0), FAMILY_CEILING);
        assert_eq!(together(&[1.6, 1.6, 1.2]), TOTAL_CEILING);
    }
}

//! Gear's numbers from the curve (Ronald, 11-07: weapons and tools "from the curves"; player
//! health "grow with the ages"): a material's item level (its age's tier × 10 plus a depth), each
//! kind's spread (how fast it swings and what share of the reference's damage a second it deals),
//! and armour by slot, which carries both the armour points of `combat`'s formula and the health
//! the age's reference player has over the base 20 (`combat::Player`).
//!
//! The reference: same-age gear at grade 4 deals `combat::REF_HIT × g(L)` a second, and a full
//! same-age set gives `combat::REF_ARMOUR × g(L)` armour and lifts health to `REF_HEALTH × g(L)`
//! (about 25 in the Stone Age's middle, 65 in the Iron Age's). Food, levels and perks move a player
//! round it.

use crate::combat::{REF_ARMOUR, REF_HEALTH, REF_HIT, REF_RATE, Type};
use crate::curve::{LEVELS_PER_AGE, g};

/// A material's age and depth (its item level is tier × 10 + depth; depth 5 is the age's
/// reference). Tool parts' materials (`crafting::MATERIALS`) and today's tool and armour sets.
#[derive(Clone, Copy, Debug)]
pub struct MaterialLevel {
    pub id: &'static str,
    pub tier: u8,
    pub depth: u8,
}

impl MaterialLevel {
    pub fn level(&self) -> f32 {
        self.tier as f32 * LEVELS_PER_AGE + self.depth as f32
    }
}

const fn ml(id: &'static str, tier: u8, depth: u8) -> MaterialLevel {
    MaterialLevel { id, tier, depth }
}

/// Every material's level. Stone and bone, wood and cordage are the Stone Age's (wood its
/// weakest); copper, bronze and gold (cast, soft) the Bronze Age's, meteoric iron its lucky top;
/// wrought iron, steel and star steel the Iron Age's; diamond a tier above steel (`notes/CRAFTING.md`:
/// "a real tier above steel"). `iron` is today's iron set, `leather` the sole-leather armour.
pub const MATERIALS: &[MaterialLevel] = &[
    ml("wood", 0, 0),
    ml("branch", 0, 0),
    ml("oak", 0, 1),
    ml("mesquite", 0, 1),
    ml("birch", 0, 1),
    ml("spruce", 0, 1),
    ml("yew", 0, 1),
    ml("elm", 0, 1),
    ml("osage_orange", 0, 1),
    ml("bone", 0, 3),
    ml("antler", 0, 3),
    ml("chert", 0, 4),
    ml("flint", 0, 5),
    ml("stone", 0, 5),
    ml("tektite", 0, 6),
    ml("obsidian", 0, 6),
    ml("plant_cordage", 0, 2),
    ml("sinew", 0, 3),
    ml("rawhide", 0, 3),
    ml("birch_tar", 0, 4),
    ml("leather", 0, 5),
    ml("copper", 1, 2),
    ml("gold", 1, 3),
    ml("bronze", 1, 6),
    ml("meteoric_iron", 1, 8),
    ml("wrought_iron", 2, 3),
    ml("iron", 2, 5),
    ml("steel", 2, 7),
    ml("star_steel", 2, 9),
    ml("diamond", 3, 5),
];

pub fn material(id: &str) -> Option<&'static MaterialLevel> {
    MATERIALS.iter().find(|m| m.id == id)
}

/// A kind's spread: swings a second, the share of the reference's damage a second it deals (a
/// sword is the reference's weapon; a pickaxe is made for rock), and its damage type. Damage ×
/// speed stays the budget: a heavy axe trades speed for a bigger hit, which goes through armour
/// better (`combat::armour_cut`).
#[derive(Clone, Copy, Debug)]
pub struct Spread {
    pub kind: &'static str,
    pub speed: f32,
    pub share: f32,
    pub kind_type: Type,
}

const fn sp(kind: &'static str, speed: f32, share: f32, kind_type: Type) -> Spread {
    Spread { kind, speed, share, kind_type }
}

pub const SPREADS: &[Spread] = &[
    sp("sword", 1.6, 1.0, Type::Cut),
    sp("spear", 1.2, 1.0, Type::Pierce),
    sp("axe", 0.9, 0.9, Type::Cut),
    sp("knife", 2.0, 0.7, Type::Cut),
    sp("club", 1.0, 0.6, Type::Blunt),
    sp("pickaxe", 1.2, 0.5, Type::Pierce),
    sp("shovel", 1.0, 0.4, Type::Blunt),
    sp("hoe", 2.0, 0.3, Type::Cut),
    // A bow's arrow, at a shot a second.
    sp("bow", 1.0, 1.0, Type::Pierce),
];

pub fn spread(kind: &str) -> Option<&'static Spread> {
    SPREADS.iter().find(|s| s.kind == kind)
}

/// A weapon or tool of `kind` at item level `level`: one hit's damage and swings a second.
pub fn weapon(kind: &str, level: f32) -> Option<(f32, f32)> {
    let s = spread(kind)?;
    let dps = REF_HIT * REF_RATE * g(level) * s.share;
    Some((dps / s.speed, s.speed))
}

/// A weapon or tool of `kind` made of `material`.
pub fn weapon_of(kind: &str, of: &str) -> Option<(f32, f32)> {
    weapon(kind, material(of)?.level())
}

/// Armour slots, head to feet, and each one's share of a full set.
pub const SLOT_SHARE: [f32; 4] = [0.15, 0.4, 0.3, 0.15];

/// A piece in slot `slot` (0 head .. 3 feet) at item level `level`: its armour points (for
/// `combat::armour_cut`) and the health it adds over the base, engine points.
pub fn armour(slot: usize, level: f32) -> (f32, f32) {
    let share = SLOT_SHARE.get(slot).copied().unwrap_or(0.0);
    (REF_ARMOUR * g(level) * share, REF_HEALTH * (g(level) - 1.0) * share)
}

/// A piece of `material` in `slot`.
pub fn armour_of(slot: usize, of: &str) -> Option<(f32, f32)> {
    Some(armour(slot, material(of)?.level()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Player;
    use crate::curve::Age;

    #[test]
    fn a_sword_of_the_age_is_the_reference() {
        let (hit, speed) = weapon("sword", Age::Iron.reference()).unwrap();
        assert!((hit * speed - Player::of(Age::Iron).hit()).abs() < 1e-3);
    }

    #[test]
    fn a_full_set_is_the_reference() {
        let lv = Age::Iron.reference();
        let (a, v): (f32, f32) = (0..4)
            .map(|s| armour(s, lv))
            .fold((0.0, 0.0), |t, x| (t.0 + x.0, t.1 + x.1));
        assert!((a - Player::of(Age::Iron).armour()).abs() < 1e-3);
        assert!((REF_HEALTH + v - Player::of(Age::Iron).health()).abs() < 1e-3);
        assert!(((REF_HEALTH + v) - 65.5).abs() < 0.5, "{}", REF_HEALTH + v);
    }

    #[test]
    fn every_kind_and_material_is_known() {
        for k in ["sword", "axe", "pickaxe", "shovel", "hoe", "knife", "spear", "bow", "club"] {
            assert!(spread(k).is_some(), "{k}");
        }
        assert!(weapon_of("sword", "diamond").unwrap().0 > weapon_of("sword", "iron").unwrap().0);
    }
}

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
use crate::curve::{LEVELS_PER_AGE, REFERENCE_DEPTH, g};

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
    // Chert is flint's equal (its trait is the difference: repairs at half the stone).
    ml("chert", 0, 5),
    ml("flint", 0, 5),
    ml("stone", 0, 5),
    ml("tektite", 0, 6),
    ml("obsidian", 0, 6),
    ml("plant_cordage", 0, 2),
    ml("sinew", 0, 3),
    ml("rawhide", 0, 3),
    ml("birch_tar", 0, 4),
    ml("leather", 0, 5),
    // Armour linings (`crafting`: a lining is Survival's clothing in numbers).
    ml("linen", 0, 3),
    ml("wool", 0, 3),
    ml("fur", 0, 4),
    ml("copper", 1, 2),
    ml("gold", 1, 3),
    ml("bronze", 1, 6),
    ml("meteoric_iron", 1, 8),
    ml("wrought_iron", 2, 3),
    ml("iron", 2, 5),
    ml("steel", 2, 7),
    ml("star_steel", 2, 9),
    ml("diamond", 3, 5),
    // Gems only (`crafting::gems`): a pallasite's peridot (the fallen stars, cut like iron-age
    // star steel), the animals mod's amber (fossil resin, Stone Age).
    ml("peridot", 2, 8),
    ml("amber", 0, 4),
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

// ------------------------------------------------------------------ bows and arrows

/// A plain arrow's item level: flint-tipped, the Stone Age's reference. An arrow's head sets its
/// level (an arrowhead part's material will, when arrows are built from parts); the bow is its
/// times, as a blade's grade is: its stave's spring, its grade and its gems (`crafting::combine`).
pub const ARROW_LEVEL: f32 = REFERENCE_DEPTH;

/// What an arrow's head can be, in order (an arrow keeps its place here plus one; 0 is a plain
/// flint one): every material that makes an arrow's head (`crafting`'s working materials;
/// progression's ranged check holds them equal). Its item level is its material's.
pub const ARROWHEADS: &[&str] = &[
    "flint",
    "chert",
    "obsidian",
    "tektite",
    "bone",
    "antler",
    "copper",
    "bronze",
    "meteoric_iron",
    "wrought_iron",
    "steel",
    "star_steel",
    "diamond",
];

/// An arrowhead's place in `ARROWHEADS`.
pub fn arrowhead_of(material: &str) -> Option<u32> {
    ARROWHEADS
        .iter()
        .position(|m| *m == material)
        .map(|i| i as u32)
}

/// The item level of an arrow headed with `ARROWHEADS[head]`: its material's (a plain one
/// `ARROW_LEVEL`).
pub fn arrow_level(head: u32) -> f32 {
    ARROWHEADS
        .get(head as usize)
        .and_then(|m| material(m))
        .map_or(ARROW_LEVEL, |m| m.level())
}

/// A drawn bow's power after `secs` held, 0 to 1 (vanilla's draw: full at one second, under 0.1
/// it doesn't shoot). An arrow flies at its power times full speed and hits by its speed.
pub fn draw_power(secs: f32) -> f32 {
    ((secs * secs + secs * 2.0) / 3.0).min(1.0)
}

/// Seconds to a full draw: one shot a second, the bow's speed in `SPREADS`.
pub const FULL_DRAW_SECS: f32 = 1.0;

/// An arrow's hit at full draw: the bow's spread at the arrow's item level, times the bow's own
/// times (1 for a Sound yew or Osage bow). Less drawn, it hits by its speed (`draw_power`); in the
/// head, twice (a bow's critical hit, kept from vanilla).
pub fn arrow_hit(arrow_level: f32, bow_times: f32) -> f32 {
    weapon("bow", arrow_level).map_or(0.0, |(hit, _)| hit) * bow_times
}

// ------------------------------------------------------------------ saddles

/// A plain saddle's item level: leather, the Stone Age's reference.
pub const SADDLE_PLAIN_LEVEL: f32 = REFERENCE_DEPTH;

/// What a saddle may be fitted with, plain first (Planner 11-23, option A: its fittings set its
/// level, as a tool's working part does). A fitted saddle carries its fitting's index
/// (`base_api::SADDLE_FIT`); a saddle item per age (option B) would carry its own the same way.
/// Each is a crafting material with a fitting's role (checked); gold is too soft to hold a rider.
pub const SADDLE_FITTINGS: &[&str] = &["leather", "copper", "bronze", "meteoric_iron", "wrought_iron", "steel", "star_steel"];

/// The fitting at `fit`'s index.
pub fn saddle_fitting(fit: u32) -> Option<&'static str> {
    SADDLE_FITTINGS.get(fit as usize).copied()
}

/// The index of `material` as a saddle's fitting.
pub fn saddle_fit_of(material: &str) -> Option<u32> {
    SADDLE_FITTINGS
        .iter()
        .position(|m| *m == material)
        .map(|i| i as u32)
}

/// A saddle's item level: its fitting's, never under a plain one's.
pub fn saddle_level(fit: u32) -> f32 {
    saddle_fitting(fit)
        .and_then(material)
        .map_or(SADDLE_PLAIN_LEVEL, |m| m.level().max(SADDLE_PLAIN_LEVEL))
}

/// What a saddle at `saddle_level` does for a tame fighting at area level `own` (`notes/PROGRESSION.md`:
/// "a higher-age saddle ... raises the tame, capped like player gear"): its health, hit and armour
/// times the curve from its level to the saddle's, never below 1. The over-tier cap then reads
/// the higher of the two, as a player's gear (`animals_core::danger::tame`).
pub fn saddle_lift(saddle_level: f32, own: f32) -> f32 {
    (g(saddle_level) / g(own)).max(1.0)
}

/// A weapon or tool of `kind` made of `material`.
pub fn weapon_of(kind: &str, of: &str) -> Option<(f32, f32)> {
    weapon(kind, material(of)?.level())
}

/// What a tool's working part does by its material's level (`notes/CRAFTING.md`, "How stats
/// combine": the working part sets the numbers). Power `g(L)` splits two ways: uses grow as
/// `g^USES_POWER` (a better tool is also less upkeep), mining speed as `g^MINE_POWER` (the rest of
/// its power is the harder ground it opens, `harvest`). At Sound (grade 4), before shape, haft and
/// trait: flint about 75 uses at 3.0, wrought iron about 270 at 5.6, diamond about 630 at 8.6.
pub const REF_USES: f32 = 53.0;
pub const USES_POWER: f32 = 1.5;
pub const REF_MINE: f32 = 2.5;
pub const MINE_POWER: f32 = 0.75;
/// What a working part at the Stone Age's reference (flint, level 5) harvests: stone and iron ore
/// (`abi::tool_tier::STONE`); each level more opens ten (bronze 210 takes diamond ore, iron's 200;
/// steel 320 obsidian, diamond's 300; diamond itself 400, above steel).
pub const REF_HARVEST: f32 = 100.0;
pub const HARVEST_PER_LEVEL: f32 = 10.0;

/// Uses of a Sound working part at item level `level`.
pub fn tool_uses(level: f32) -> f32 {
    REF_USES * g(level).powf(USES_POWER)
}

/// Mining (chopping, digging) speed of a working part at `level`.
pub fn mining_speed(level: f32) -> f32 {
    REF_MINE * g(level).powf(MINE_POWER)
}

/// The harvest level (`abi::tool_tier`, a hundred an old tier) of a working part at `level`, in
/// steps of ten.
pub fn harvest(level: f32) -> u32 {
    let h = REF_HARVEST + HARVEST_PER_LEVEL * (level - REFERENCE_DEPTH);
    ((h / 10.0).round() * 10.0).max(0.0) as u32
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

    #[test]
    fn tools_by_level() {
        let flint = material("flint").unwrap().level();
        assert_eq!(harvest(flint), crate::gear::REF_HARVEST as u32);
        // The old gates: bronze takes what iron did (200), steel what diamond did (300), diamond
        // stays above steel; wood and bone stay under stone's 100.
        assert!(harvest(material("bronze").unwrap().level()) >= 200);
        assert!(harvest(material("steel").unwrap().level()) >= 300);
        assert!(harvest(material("diamond").unwrap().level()) > harvest(material("steel").unwrap().level()));
        assert!(harvest(material("bone").unwrap().level()) < 100 && harvest(material("oak").unwrap().level()) < 100);
        assert!((tool_uses(flint) - 75.0).abs() < 2.0 && (mining_speed(flint) - 3.0).abs() < 0.05);
        assert!(tool_uses(35.0) > tool_uses(25.0) && mining_speed(35.0) > mining_speed(25.0));
    }
}

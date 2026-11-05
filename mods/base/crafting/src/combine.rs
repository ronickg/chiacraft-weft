//! An item's numbers from its parts (`notes/CRAFTING.md`, "How stats combine"), one readable rule:
//! - the working part sets the numbers: its material × its shape × its grade
//! - the holding parts multiply them: × (1 + their materials' bonuses) × their shapes
//! - the fitting adds a trait and wears on its own
//! - weight is every part's mass
//!
//! Each part wears by itself: the tool is as good as its first part to give out, which for a
//! stone axe is the lashing, as it really was.

use crate::grades::grade;
use crate::kinds::{Role, Use, kind, shape};
use crate::materials::{fits, material};
use crate::traits::{Effect, Trait, When, of_material};

/// How many times longer a holding part lasts than a working part of the same material.
pub const HAFT_WEAR: f32 = 10.0;

/// One part as an item carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    pub slot: &'static str,
    pub shape: &'static str,
    pub material: &'static str,
    pub grade: u8,
}

/// How long one part lasts in the item, in uses at full health.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Life {
    pub slot: &'static str,
    pub uses: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolStats {
    pub kind: &'static str,
    /// The working part's: its harvest level, its grade (the item's), its modifier slots.
    pub harvest: u32,
    pub grade: u8,
    pub modifier_slots: u8,
    /// The main stat: chop, mine, dig or till speed, or for a blade or point its cutting speed.
    pub stat: f32,
    pub damage: f32,
    /// Blocks of reach beyond the hand's.
    pub reach: f32,
    pub mass_kg: f32,
    /// Each wearing part's life, the working part first.
    pub life: Vec<Life>,
    /// Every trait its parts bring, once each.
    pub traits: Vec<&'static Trait>,
}

impl ToolStats {
    /// The part that gives out first, and after how many uses.
    pub fn weakest(&self) -> Option<Life> {
        self.life
            .iter()
            .copied()
            .min_by(|a, b| a.uses.total_cmp(&b.uses))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Missing {
    Kind(String),
    /// A slot of the kind has no part.
    Slot(&'static str),
    /// A part names a shape the kind doesn't have in that slot.
    Shape(&'static str),
    Material(&'static str),
    /// A material that can't be that part (copper as a haft).
    Role(&'static str, &'static str),
}

/// Wear on a part from a trait that always works (×).
fn always_wear(traits: &[&Trait]) -> f32 {
    traits
        .iter()
        .filter(|t| t.when == When::Always)
        .map(|t| if let Effect::Wear(f) = t.effect { f } else { 1.0 })
        .product()
}

/// How long a part lasts by itself, in uses at full health: a working part its material's uses, a
/// holding part `HAFT_WEAR` times that, a fitting its fitting uses; all × its grade, ÷ its wear.
pub fn life_of(role: Role, id: &str, grade_n: u8) -> f32 {
    let Some(m) = material(id) else { return 0.0 };
    let g = grade(grade_n).mult;
    let wear = always_wear(&of_material(m.id));
    match role {
        Role::Working => m.uses() * g / wear,
        Role::Holding => m.uses() * g * HAFT_WEAR / wear,
        Role::Fitting => m.fitting_uses * g / wear,
        Role::Extra | Role::Lining => 0.0,
    }
}

/// Share of a part's life one unit of its material gives back in a repair: half, scaled by a
/// repair-cost trait (chert's half cost gives twice as much; copper's free hammering all of it).
pub const REPAIR_SHARE: f32 = 0.5;

pub fn repair_share(id: &str) -> f32 {
    let cost: f32 = of_material(id)
        .iter()
        .map(|t| if let Effect::RepairCost(f) = t.effect { f } else { 1.0 })
        .product();
    if cost <= 0.0 { 1.0 } else { (REPAIR_SHARE / cost).min(1.0) }
}

/// The numbers of a `kind` built from `parts`, one for each of its slots.
pub fn combine(kind_id: &str, parts: &[Part]) -> Result<ToolStats, Missing> {
    let k = kind(kind_id).ok_or_else(|| Missing::Kind(kind_id.to_string()))?;
    let mut working = None;
    let (mut hold_uses, mut hold_speed, mut hold_mul_speed, mut hold_mul_damage, mut reach) = (0.0, 0.0, 1.0, 1.0, 0.0);
    let mut mass = 0.0;
    let mut life = Vec::new();
    let mut traits: Vec<&'static Trait> = Vec::new();
    for slot in k.slots {
        let p = parts
            .iter()
            .find(|p| p.slot == slot.name)
            .ok_or(Missing::Slot(slot.name))?;
        let s = shape(k.id, p.shape)
            .filter(|s| s.slot == slot.name)
            .ok_or(Missing::Shape(p.shape))?;
        let m = material(p.material).ok_or(Missing::Material(p.material))?;
        if !fits(m, k.id, slot.name, slot.role) {
            return Err(Missing::Role(p.material, slot.name));
        }
        let g = grade(p.grade).mult;
        let bulk = if slot.role == Role::Working { m.bulk } else { 1.0 };
        mass += m.density * s.volume * bulk / 1000.0;
        reach += s.reach;
        let own = of_material(m.id);
        for t in &own {
            if !traits.iter().any(|x| x.id == t.id) {
                traits.push(t);
            }
        }
        match slot.role {
            Role::Working => working = Some((p, s, m, own.clone())),
            Role::Holding => {
                hold_uses += m.hold_uses;
                hold_speed += m.hold_speed;
                hold_mul_speed *= s.speed;
                hold_mul_damage *= s.damage;
                // A haft rarely wears: a tenth as fast as a head, so a good head outlives it and
                // a worn haft is replaced, not the tool.
                life.push(Life { slot: slot.name, uses: m.uses() * g * HAFT_WEAR / always_wear(&own) });
            }
            Role::Fitting => life.push(Life { slot: slot.name, uses: m.fitting_uses * g / always_wear(&own) }),
            Role::Extra | Role::Lining => {}
        }
    }
    let (p, s, m, own) = working.ok_or(Missing::Slot("working"))?;
    let g = grade(p.grade);
    let stat_mul: f32 = own
        .iter()
        .filter(|t| t.when == When::Always)
        .map(|t| match t.effect {
            Effect::Stat(f) | Effect::Speed(f) => f,
            _ => 1.0,
        })
        .product();
    let dmg_mul: f32 = own
        .iter()
        .filter(|t| t.when == When::Always)
        .map(|t| if let Effect::Damage(f) = t.effect { f } else { 1.0 })
        .product();
    // A blade's or point's main stat is its damage: the grade goes there, a tool's goes to speed.
    // A bow's power is its stave's spring. The rest hit as their kind does at their working part's
    // age (`vitals::gear`, from the curve; Ronald, 11-07).
    let weapon = matches!(k.use_, Use::Cut | Use::Thrust | Use::Shoot);
    let base_damage = if k.use_ == Use::Shoot {
        k.damage * crate::materials::spring(m.id)
    } else {
        vitals::gear::weapon_of(k.id, m.id).map_or(k.damage, |(hit, _)| hit)
    };
    let stat = m.speed() * s.stat * stat_mul * if weapon { 1.0 } else { g.mult } * (1.0 + hold_speed) * hold_mul_speed * s.speed;
    let damage = base_damage * s.damage * dmg_mul * if weapon { g.mult } else { 1.0 } * hold_mul_damage;
    let head = Life { slot: p.slot, uses: m.uses() * g.mult * (1.0 + hold_uses) / always_wear(&own) };
    life.insert(0, head);
    Ok(ToolStats {
        kind: k.id,
        harvest: m.harvest(),
        grade: g.n,
        modifier_slots: g.slots,
        stat,
        damage,
        reach,
        mass_kg: mass,
        life,
        traits,
    })
}

/// How much of a shell's uses each armour slot gets, head to feet (a coat takes the most blows;
/// vanilla's 11, 16, 15, 13 of 16).
pub const SLOT_WEAR: [f32; 4] = [0.69, 1.0, 0.94, 0.81];

/// A piece of armour built from parts (`notes/CRAFTING.md`, "Kinds and their parts").
#[derive(Clone, Debug, PartialEq)]
pub struct ArmourStats {
    pub kind: &'static str,
    /// Head to feet.
    pub slot: usize,
    /// Its shell's material's item level (`vitals::gear`): what over-tier caps read.
    pub level: f32,
    pub grade: u8,
    pub modifier_slots: u8,
    /// Armour points (`vitals::combat::armour_cut`) and the health it adds over the base, from the
    /// curve by its shell's level and slot, × the shell's grade.
    pub armour: f32,
    pub health: f32,
    pub mass_kg: f32,
    /// The shell's and fastening's lives in blows, the shell first.
    pub life: Vec<Life>,
    pub traits: Vec<&'static Trait>,
    /// The materials of its shell, fastening and lining (warmth: `vitals::clothing`).
    pub shell: &'static str,
    pub fastening: &'static str,
    pub lining: &'static str,
}

impl ArmourStats {
    pub fn weakest(&self) -> Option<Life> {
        self.life
            .iter()
            .copied()
            .min_by(|a, b| a.uses.total_cmp(&b.uses))
    }
}

/// The numbers of armour `kind_id` built from `parts`: the shell sets them (its material's level
/// on the curve, × its grade), the fastening wears first, the lining pads and warms.
pub fn armour(kind_id: &str, parts: &[Part]) -> Result<ArmourStats, Missing> {
    let k = kind(kind_id).ok_or_else(|| Missing::Kind(kind_id.to_string()))?;
    let slot_n = k
        .armour_slot()
        .ok_or_else(|| Missing::Kind(kind_id.to_string()))?;
    let mut mass = 0.0;
    let mut traits: Vec<&'static Trait> = Vec::new();
    let mut life = Vec::new();
    let (mut shell, mut fastening, mut lining) = (None, "", "");
    for slot in k.slots {
        let p = parts
            .iter()
            .find(|p| p.slot == slot.name)
            .ok_or(Missing::Slot(slot.name))?;
        let s = shape(k.id, p.shape)
            .filter(|s| s.slot == slot.name)
            .ok_or(Missing::Shape(p.shape))?;
        let m = material(p.material).ok_or(Missing::Material(p.material))?;
        if !fits(m, k.id, slot.name, slot.role) {
            return Err(Missing::Role(p.material, slot.name));
        }
        let bulk = if slot.role == Role::Working { m.bulk } else { 1.0 };
        mass += m.density * s.volume * bulk / 1000.0;
        let own = of_material(m.id);
        for t in &own {
            if !traits.iter().any(|x| x.id == t.id) {
                traits.push(t);
            }
        }
        let g = grade(p.grade).mult;
        match slot.role {
            Role::Working => {
                life.insert(0, Life { slot: slot.name, uses: m.uses() * SLOT_WEAR[slot_n] * g / always_wear(&own) });
                shell = Some((p, m));
            }
            Role::Fitting => {
                life.push(Life { slot: slot.name, uses: m.fitting_uses * g / always_wear(&own) });
                fastening = m.id;
            }
            _ => lining = m.id,
        }
    }
    let (p, m) = shell.ok_or(Missing::Slot("shell"))?;
    let g = grade(p.grade);
    let (armour, health) = vitals::gear::armour(slot_n, m.level());
    Ok(ArmourStats {
        kind: k.id,
        slot: slot_n,
        level: m.level(),
        grade: g.n,
        modifier_slots: g.slots,
        armour: armour * g.mult,
        health: health * g.mult,
        mass_kg: mass,
        life,
        traits,
        shell: m.id,
        fastening,
        lining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flint_axe(head: u8) -> ToolStats {
        combine(
            "axe",
            &[
                Part { slot: "head", shape: "felling_axe_head", material: "flint", grade: head },
                Part { slot: "haft", shape: "haft", material: "oak", grade: 5 },
                Part { slot: "fitting", shape: "lashing", material: "sinew", grade: 4 },
            ],
        )
        .unwrap()
    }

    #[test]
    fn the_mock_ups_flint_axe() {
        let a = flint_axe(7);
        // Flint's speed (about 3.0, from the curve) × Superior 1.20, oak's −3%.
        let flint = material("flint").unwrap().speed();
        assert!((a.stat - flint * 1.20 * 0.97).abs() < 1e-4, "{}", a.stat);
        assert_eq!((a.grade, a.modifier_slots, a.harvest), (7, 2, 100));
        // The lashing goes first, as it did.
        assert_eq!(a.weakest().unwrap().slot, "fitting");
        // A stone axe weighs about a kilogram.
        assert!((0.8..1.3).contains(&a.mass_kg), "{}", a.mass_kg);
        assert!(a.traits.iter().any(|t| t.id == "keen") && a.traits.iter().any(|t| t.id == "shrinks_tight"));
    }

    #[test]
    fn a_better_head_is_better() {
        let (low, high) = (flint_axe(2), flint_axe(9));
        assert!(high.stat > low.stat && high.life[0].uses > low.life[0].uses);
        assert!(high.modifier_slots > low.modifier_slots);
    }

    #[test]
    fn iron_is_heavier_and_lasts() {
        let iron = combine(
            "axe",
            &[
                Part { slot: "head", shape: "felling_axe_head", material: "wrought_iron", grade: 4 },
                Part { slot: "haft", shape: "haft", material: "oak", grade: 4 },
                Part { slot: "fitting", shape: "lashing", material: "wrought_iron", grade: 4 },
            ],
        )
        .unwrap();
        let flint = flint_axe(4);
        assert!(iron.mass_kg > flint.mass_kg && iron.weakest().unwrap().uses > flint.weakest().unwrap().uses);
        assert!(iron.harvest > flint.harvest);
    }

    #[test]
    fn a_swords_grade_is_its_damage() {
        let sword = |g| {
            combine(
                "sword",
                &[
                    Part { slot: "blade", shape: "straight_blade", material: "bronze", grade: g },
                    Part { slot: "guard", shape: "bar_guard", material: "bronze", grade: 4 },
                    Part { slot: "grip", shape: "grip", material: "leather", grade: 4 },
                    Part { slot: "pommel", shape: "round_pommel", material: "gold", grade: 4 },
                ],
            )
            .unwrap()
        };
        assert!(sword(9).damage > sword(4).damage);
        assert_eq!(sword(9).stat, sword(4).stat);
    }

    #[test]
    fn repairs_by_material() {
        assert_eq!(repair_share("wrought_iron"), 0.5);
        assert_eq!(repair_share("chert"), 1.0, "half the cost, twice the repair");
        assert_eq!(repair_share("copper"), 1.0, "hammered back for free");
        assert!(life_of(Role::Holding, "oak", 4) > life_of(Role::Working, "oak", 4), "hafts outlast heads");
        assert_eq!(life_of(Role::Extra, "gold", 4), 0.0);
    }

    #[test]
    fn yew_draws_hardest() {
        let bow = |wood| {
            combine(
                "bow",
                &[
                    Part { slot: "stave", shape: "self_stave", material: wood, grade: 4 },
                    Part { slot: "grip", shape: "grip", material: "leather", grade: 4 },
                    Part { slot: "string", shape: "bowstring", material: "sinew", grade: 4 },
                ],
            )
            .unwrap()
        };
        assert!(bow("yew").damage > bow("oak").damage && bow("oak").damage > bow("spruce").damage);
    }

    #[test]
    fn what_cant_be_built() {
        assert_eq!(combine("axe", &[]), Err(Missing::Slot("head")));
        let copper_haft = [
            Part { slot: "head", shape: "felling_axe_head", material: "flint", grade: 4 },
            Part { slot: "haft", shape: "haft", material: "copper", grade: 4 },
            Part { slot: "fitting", shape: "lashing", material: "sinew", grade: 4 },
        ];
        assert_eq!(combine("axe", &copper_haft), Err(Missing::Role("copper", "haft")));
        assert!(matches!(combine("mace", &copper_haft), Err(Missing::Kind(_))));
    }

    #[test]
    fn a_mail_coat() {
        let coat = |shell: &'static str, g| {
            armour(
                "chestplate",
                &[
                    Part { slot: "shell", shape: "chest_shell", material: shell, grade: g },
                    Part { slot: "fastening", shape: "straps", material: "leather", grade: 4 },
                    Part { slot: "lining", shape: "lining", material: "wool", grade: 4 },
                ],
            )
            .unwrap()
        };
        let (iron, leather) = (coat("wrought_iron", 4), coat("leather", 4));
        // The curve's coat: its slot's share of a same-age set.
        let (a, _) = vitals::gear::armour(1, vitals::gear::material("wrought_iron").unwrap().level());
        assert!((iron.armour - a).abs() < 1e-4);
        assert!(iron.armour > leather.armour && iron.mass_kg > 5.0 && leather.mass_kg < 3.0, "{} {}", iron.mass_kg, leather.mass_kg);
        assert!(coat("wrought_iron", 9).armour > iron.armour);
        assert_eq!((iron.slot, iron.lining), (1, "wool"));
        // No gold coat, no diamond straps.
        let gold = [
            Part { slot: "shell", shape: "chest_shell", material: "gold", grade: 4 },
            Part { slot: "fastening", shape: "straps", material: "leather", grade: 4 },
            Part { slot: "lining", shape: "lining", material: "wool", grade: 4 },
        ];
        assert_eq!(armour("chestplate", &gold), Err(Missing::Role("gold", "shell")));
    }

    #[test]
    fn numbers_from_the_curve() {
        // A Sound flint pick at its level's uses and speed; a diamond one above an iron one.
        let pick = |head: &'static str| {
            combine(
                "pickaxe",
                &[
                    Part { slot: "head", shape: "pick_head", material: head, grade: 4 },
                    Part { slot: "haft", shape: "haft", material: "birch", grade: 4 },
                    Part { slot: "fitting", shape: "lashing", material: "wrought_iron", grade: 4 },
                ],
            )
            .unwrap()
        };
        let flint = pick("flint");
        let lv = vitals::gear::material("flint").unwrap().level();
        assert!((flint.stat - vitals::gear::mining_speed(lv) * 1.05).abs() < 1e-3, "{}", flint.stat);
        assert_eq!(flint.harvest, vitals::gear::harvest(lv));
        let (iron, diamond) = (pick("wrought_iron"), pick("diamond"));
        assert!(diamond.harvest > iron.harvest && diamond.stat > iron.stat && diamond.life[0].uses > iron.life[0].uses);
    }
}

//! The whole tools and armour of before parts (`wood_pickaxe` ... `diamond_boots`). Ronald,
//! 11-05: "I actually don't want to have like diamond sword or something, because it doesn't
//! really make sense. I want ... more modular approach". Every tool, weapon and piece of armour is
//! now built from parts (`parts.rs`); these are no longer made, found or shown anywhere.
//!
//! They stay registered for one reason: an old save's stacks load by name, and the engine drops a
//! name it doesn't know (`persist.rs`: "items of removed mods are dropped"). So each still loads as
//! itself, with its old numbers, and turns into its closest part-built one the moment it's seen:
//! - a player's whole inventory (bag, hotbar, armour) when they join, and every second after;
//! - a chest's (or any block's) when it's opened.
//!
//! The closest one (`closest`): the same kind with a head, blade or shell of the same material
//! where that's real, at Sound (grade 4), with its wear carried over. Wrought iron for iron;
//! diamond for diamond; copper with gold fittings for the gold tools, bronze scales with gold
//! rivets for the gold armour (gold is a fitting or an ornament only); flint for the wooden axe,
//! pick and sword (no wood makes those), and a flint sword blade set in wood, a macuahuitl, for
//! the stone sword. Hafts and grips are oak, fittings cordage on stone, wrought iron on metal.

use weft_sdk::*;

use crate::items::{ARMOR, ARMOR_SLOTS, Armor};

/// The old tool sets: harvest level, mining speed and uses (still theirs until converted).
const TOOL_SETS: [(&str, u32, f32, u32); 5] = [
    ("wood", tool_tier::WOOD, 2.0, 59),
    ("stone", tool_tier::STONE, 4.0, 131),
    ("iron", tool_tier::IRON, 6.0, 250),
    ("gold", tool_tier::WOOD, 12.0, 32),
    ("diamond", tool_tier::DIAMOND, 8.0, 1561),
];
pub const TOOLS: [&str; 5] = ["pickaxe", "shovel", "sword", "axe", "hoe"];
/// Points per piece head to feet, toughness, and uses per slot unit, as they were.
const ARMOUR_SETS: [(&str, [u32; 4], f32, u32); 4] = [
    ("leather", [1, 3, 2, 1], 0.0, 5),
    ("iron", [2, 6, 5, 2], 0.0, 15),
    ("gold", [2, 5, 3, 1], 0.0, 7),
    ("diamond", [3, 8, 6, 3], 2.0, 33),
];
pub const PIECES: [&str; 4] = ["helmet", "chestplate", "leggings", "boots"];

/// Every old whole item's name (without `base:`).
#[cfg(test)]
pub fn names() -> impl Iterator<Item = String> {
    let tools = TOOL_SETS
        .iter()
        .flat_map(|(m, ..)| TOOLS.iter().map(move |k| format!("{m}_{k}")));
    let armour = ARMOUR_SETS
        .iter()
        .flat_map(|(m, ..)| PIECES.iter().map(move |p| format!("{m}_{p}")));
    tools.chain(armour)
}

/// What an old whole item is built as now: its kind and its parts' materials, slot by slot
/// (`crafting::KINDS`; a tool's head, haft and fitting, a sword's blade, guard, grip and pommel,
/// armour's shell, fastening and lining).
pub fn closest(name: &str) -> Option<(&'static str, &'static [&'static str])> {
    let name = name.strip_prefix("base:").unwrap_or(name);
    let (material, kind) = name.split_once('_')?;
    let parts: &'static [&'static str] = match (material, kind) {
        ("wood", "shovel" | "hoe") => &["oak", "oak", "plant_cordage"],
        ("wood", "axe" | "pickaxe") => &["flint", "oak", "plant_cordage"],
        ("wood" | "stone", "sword") => &["flint", "oak", "oak", "oak"],
        ("stone", "axe" | "pickaxe" | "shovel" | "hoe") => &["stone", "oak", "plant_cordage"],
        ("iron", "sword") => &["wrought_iron", "wrought_iron", "leather", "wrought_iron"],
        ("iron", "axe" | "pickaxe" | "shovel" | "hoe") => &["wrought_iron", "oak", "wrought_iron"],
        ("gold", "sword") => &["copper", "gold", "leather", "gold"],
        ("gold", "axe" | "pickaxe" | "shovel" | "hoe") => &["copper", "oak", "gold"],
        ("diamond", "sword") => &["diamond", "wrought_iron", "oak", "wrought_iron"],
        ("diamond", "axe" | "pickaxe" | "shovel" | "hoe") => &["diamond", "oak", "wrought_iron"],
        ("leather", "helmet" | "chestplate" | "leggings" | "boots") => &["leather", "leather", "linen"],
        ("iron", "helmet" | "chestplate" | "leggings" | "boots") => &["wrought_iron", "leather", "wool"],
        ("gold", "helmet" | "chestplate" | "leggings" | "boots") => &["bronze", "gold", "linen"],
        ("diamond", "helmet" | "chestplate" | "leggings" | "boots") => &["diamond", "wrought_iron", "wool"],
        _ => return None,
    };
    let kind = TOOLS.iter().chain(PIECES.iter()).find(|k| **k == kind)?;
    Some((kind, parts))
}

/// The part-built stack an old whole one becomes, its wear's share carried over; `None` for
/// anything else.
pub fn convert(w: &World, stack: &ItemStack) -> Option<ItemStack> {
    if stack.is_empty() || crate::parts::PARTS.on_stack_own(stack).is_some() {
        return None;
    }
    let (kind, parts) = closest(item_name(stack.item))?;
    let day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    let made = crate::parts::made_of(kind, parts, 4, day);
    if made.is_empty() {
        return None;
    }
    let old_max = DURABILITY.on_stack(stack).unwrap_or(0).max(1) as f32;
    let share = (DAMAGE.on_stack_own(stack).unwrap_or(0) as f32 / old_max).clamp(0.0, 1.0);
    let max = DURABILITY.on_stack_own(&made).unwrap_or(1) as f32;
    let worn = (share * max) as u32;
    Some(if worn > 0 { made.with(&DAMAGE, worn) } else { made })
}

/// Converts every old whole stack in an inventory; how many it changed.
pub fn convert_all(w: &World, inv: &Inventory) -> u32 {
    let mut n = 0;
    for i in 0..inv.len() {
        let s = inv.get(i);
        if let Some(new) = convert(w, &s) {
            weft_sdk::info!("old {} in slot {i} became a part-built {}", item_name(s.item), item_name(new.item));
            inv.set(i, new);
            n += 1;
        }
    }
    n
}

pub fn register(r: &mut Registry) {
    // As they were, minus their recipes, creative tabs and `<kind>s/<material>` tags.
    for (m, level, speed, durability) in TOOL_SETS {
        for kind in TOOLS {
            let (damage, swings) = vitals::gear::weapon_of(kind, m).unwrap_or((1.0, 1.0));
            let item_level = vitals::gear::material(m).map_or(0.0, |x| x.level());
            let b = r
                .item(Item::generated(format!("{m}_{kind}")))
                .tool(kind)
                .tool_level(level)
                .mining_speed(speed)
                .durability(durability)
                .attack_damage(damage)
                .attack_speed(swings)
                .set(&base_api::ITEM_LEVEL, item_level)
                .set(&WEAR_PER_HIT, if kind == "sword" { 1 } else { 2 });
            if kind == "sword" {
                b.sweeps().no_creative_break();
            }
        }
    }
    for (m, points, toughness, durability) in ARMOUR_SETS {
        for (slot, piece) in PIECES.iter().enumerate() {
            let dur = [11, 16, 15, 13][slot];
            let (inflate, layer) = if slot == 2 { (0.5, 2) } else { (1.0, 1) };
            r.item(Item::generated(format!("{m}_{piece}")))
                .durability(durability * dur)
                .set(&ARMOR, Armor { points: points[slot], toughness })
                .tag(ARMOR_SLOTS[slot])
                .wearable(crate::player::EQUIPMENT[slot], inflate, &format!("armor/{m}_{layer}"));
        }
    }
    r.on(&PLAYER_JOINED, |b| {
        let w = b.world();
        for p in b.iter() {
            convert_all(&w, &w.inventory(p.player));
        }
    });
    // Every second, so a stack from a command, a pickup or another mod never stays whole.
    r.world_tick(|w| {
        for id in w.players() {
            if w.entity(id).is_some_and(|e| e.age % 20 == 7) {
                convert_all(w, &w.inventory(id));
            }
        }
    });
    r.intercept(&actions::USE_BLOCK, |q| {
        let w = q.world();
        let be = w.block_entity(q.pos());
        if be != 0 {
            convert_all(&w, &w.inventory(be));
        }
        Verdict::Abstain
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_old_item_has_a_closest_one() {
        for name in names() {
            let (kind, parts) = closest(&name).unwrap_or_else(|| panic!("{name}"));
            let k = crafting::kinds::kind(kind).unwrap();
            assert_eq!(parts.len(), k.slots.len(), "{name}");
            for (slot, m) in k.slots.iter().zip(parts.iter()) {
                let mat = crafting::materials::material(m).unwrap();
                assert!(crafting::materials::fits(mat, kind, slot.name, slot.role), "{name}: {m} as {}", slot.name);
            }
        }
        assert_eq!(closest("base:diamond_sword").unwrap().1[0], "diamond");
        assert!(closest("base:stick").is_none() && closest("base:wool_coat").is_none());
    }
}

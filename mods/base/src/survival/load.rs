//! What one carries and how fast one walks. Every item has a game weight (`weight`): over the
//! weight stat one is encumbered, past half again overloaded. Walking speed is the speed stat
//! times what the effects leave of it, none when out cold, set on the engine's movement speed.

use base_api::Vitals;
use vitals::effects::{self, Effect, Modifiers};
use vitals::food;
use vitals::stats::{STATS, Stat};
use weft_sdk::*;

use super::set;

/// Once a second: what one carries, and encumbered or overloaded.
pub fn weigh(w: &World, id: u64, v: &mut Vitals, stats: &[f32; STATS]) {
    v.load = carried(w, id);
    let load = effects::load(v.load, stats[Stat::Weight.index()]);
    set(v, Effect::Encumbered, load == Some(Effect::Encumbered));
    set(v, Effect::Overloaded, load == Some(Effect::Overloaded));
}

/// Walking speed and jumping (none when out cold), set again now and then, as a respawn or a
/// rejoin drops it.
pub fn speed(w: &World, id: u64, v: &mut Vitals, stats: &[f32; STATS], mods: &Modifiers) {
    let speed = if v.out != 0 { 0.0 } else { stats[Stat::Speed.index()] * mods.speed };
    if (v.speed - speed).abs() > 0.001 || w.game_time() % 100 == 0 {
        v.speed = speed;
        w.attribute_add_modifier(id, &attribute::MOVEMENT_SPEED, "base:vitals", attr_op::MULTIPLY_TOTAL, (speed - 1.0) as f64, 0);
        w.attribute_add_modifier(id, &attribute::JUMP_STRENGTH, "base:vitals", attr_op::MULTIPLY_TOTAL, if v.out != 0 { -1.0 } else { 0.0 }, 0);
    }
}

/// In creative: the body's speed as the engine's.
pub fn reset_speed(w: &World, id: u64, v: &mut Vitals) {
    if v.speed != 1.0 {
        w.attribute_remove_modifier(id, &attribute::MOVEMENT_SPEED, "base:vitals");
        w.attribute_remove_modifier(id, &attribute::JUMP_STRENGTH, "base:vitals");
        v.speed = 1.0;
    }
}

/// Kilograms one carries, everything in the inventory and worn (`weight`).
fn carried(w: &World, player: u64) -> f32 {
    let inv = w.inventory(player);
    (0..inv.len())
        .map(|i| inv.get(i))
        .filter(|s| !s.is_empty())
        .map(|s| weight(item_name(s.item)) * s.count as f32)
        .sum()
}

/// What one of an item weighs, in kilograms: game weights, not real ones (a block of stone is a
/// tenth, so a hotbar of stone stacks is about all one carries freely; proposed). Food by its
/// portion, the rest by its kind of name.
pub fn weight(name: &str) -> f32 {
    if let Some(f) = food::find(name) {
        return food::FOODS[f].grams / 1000.0;
    }
    // A bowl of stew about half a kilo; clothes by their piece (a fur coat 2 kg).
    if name == "base:stew" {
        return 0.5;
    }
    if let Some((m, piece)) = vitals::clothing::piece_of(name) {
        let per = if m.name == "fur" { 1.0 } else { 0.5 };
        return per * [0.5, 2.0, 1.5, 1.0][piece];
    }
    let n = name.split_once(':').map_or(name, |(_, n)| n);
    let piece = |p: &str| n.ends_with(p);
    if piece("_helmet") || piece("_chestplate") || piece("_leggings") || piece("_boots") {
        let per = match n.split('_').next().unwrap_or("") {
            "leather" => 0.5,
            "chainmail" => 1.5,
            "iron" => 2.0,
            "gold" | "golden" => 2.5,
            "diamond" => 2.0,
            "netherite" => 3.0,
            _ => 1.0,
        };
        let share = if piece("_chestplate") {
            2.0
        } else if piece("_leggings") {
            1.5
        } else {
            1.0
        };
        return per * share;
    }
    if piece("_sword") || piece("_pickaxe") || piece("_axe") || piece("_shovel") || piece("_hoe") || n == "bow" || n == "shield" {
        return 1.0;
    }
    if piece("_ingot") || n.starts_with("raw_") || piece("_ore") || n == "coal" || n == "diamond" || n == "emerald" {
        return 0.2;
    }
    if piece("bucket") {
        return if n == "bucket" { 0.5 } else { 1.5 };
    }
    if piece("_log") || piece("_wood") {
        return 0.15;
    }
    if piece("_planks") || piece("_slab") || piece("_stairs") || n == "stick" {
        return 0.05;
    }
    if [
        "stone",
        "cobble",
        "brick",
        "deepslate",
        "granite",
        "diorite",
        "andesite",
        "sandstone",
        "obsidian",
        "dirt",
        "sand",
        "gravel",
        "clay",
        "block",
    ]
    .iter()
    .any(|m| n.contains(m))
    {
        return 0.1;
    }
    0.02
}

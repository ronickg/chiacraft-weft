//! Metals from the world (`notes/CONTENT.md`, "The ladder" and "Where things are"; the research's
//! `reports/content/Tools and materials.md`, C1-C3):
//! - **Copper ore** (malachite and native copper): in the taiga and snow ("cold": native copper),
//!   the desert ("dry": malachite) and on beaches, and a little deep everywhere.
//! - **Tin ore** (cassiterite): rare, about thirty times rarer than copper in the crust; only on
//!   beaches and in the mountains, and a very little deep down. "What isn't near you" is what
//!   travel and linked worlds are for.
//! - **Smelting:** raw copper and raw tin smelt into ingots in the furnace (the generic ore rules in
//!   `recipes.rs`), a stone pick mines both ores.
//! - **Bronze** is cast at a crucible: copper worked with tin, one ingot of bronze for each copper
//!   and its tin (the real alloy is 7:1; the scarcity is kept in tin's ore). The Bronze Age's.
//! - **Steel** is made at a forge: iron worked with charcoal (carburised), the Iron Age's.
//!
//! Both stations are crafts that wait (`crafts.rs`, `vitals::crafts`), in their real time.

use weft_sdk::*;

pub static COPPER_ORE: Block = Block::named("copper_ore");
pub static TIN_ORE: Block = Block::named("tin_ore");

/// The metals' items: (name, its tags).
const ITEMS: [(&str, &[&str]); 6] = [
    ("raw_copper", &["raw_ores/copper"]),
    ("raw_tin", &["raw_ores/tin"]),
    ("copper_ingot", &["ingots/copper", "materials/copper"]),
    ("tin_ingot", &["ingots/tin"]),
    ("bronze_ingot", &["ingots/bronze", "materials/bronze"]),
    ("steel_ingot", &["ingots/steel", "materials/steel"]),
];

pub fn register(r: &mut Registry) {
    for (name, tags) in ITEMS {
        let mut b = r.item(Item::generated(name.into()));
        for t in tags {
            b = b.tag(t);
        }
        drop(b.tab(&crate::tabs::INGREDIENTS));
    }
    // A stone pick for copper and tin ore (`notes/CONTENT.md`: "a stone pick for copper and tin
    // ore, bronze for iron ore"; iron ore stays at stone for now).
    for (block, metal) in [(&COPPER_ORE, "copper"), (&TIN_ORE, "tin")] {
        r.block(block)
            .sound_group("stone")
            .hardness(3.0)
            .mined_by("pickaxe", tool_tier::STONE)
            .drops(&format!("raw_{metal}"), 1)
            .tag("ores")
            .tag(&format!("ores/{metal}"))
            .tab(&crate::tabs::NATURAL);
    }
}

/// Where the ores lie: (ore, biome tag or "" for anywhere, deepest y to highest y, veins a cube,
/// vein size). Iron's 6 veins of 8 is the yardstick.
pub const VEINS: [(&str, &str, i32, u32, u32); 7] = [
    ("copper", "cold", 96, 6, 8),
    ("copper", "dry", 96, 6, 8),
    ("copper", "beach", 80, 5, 7),
    ("copper", "", 0, 2, 6),
    ("tin", "beach", 80, 2, 5),
    ("tin", "mountain", 120, 2, 5),
    ("tin", "", -16, 1, 4),
];

/// The ores in worldgen (`worldgen.rs`, with the others, left out where they'd touch a cave).
/// Each row is its own feature: a feature name is registered once, so the rows past an ore's
/// first are named `copper_ore/<biome>` (`copper_ore/deep` for the one anywhere).
pub fn ores(r: &mut Registry) {
    let mut seen: Vec<&str> = Vec::new();
    for (metal, biome, max_y, veins, size) in VEINS {
        let block = if metal == "copper" { &COPPER_ORE } else { &TIN_ORE };
        let first = !seen.contains(&metal);
        seen.push(metal);
        let o = r.ore(block);
        let o = if first {
            o
        } else {
            let place = if biome.is_empty() { "deep" } else { biome };
            o.named(&format!("{metal}_ore/{place}"))
        };
        let o = o
            .y(i32::MIN, max_y)
            .per_cube(veins)
            .vein_size(size)
            .replaces("#stone_ore_replaceables");
        // A biome tag is written `#cold`.
        drop(if biome.is_empty() { o } else { o.biome(&format!("#{biome}")) });
    }
}

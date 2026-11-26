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
/// Copper's shallow ore in the dry lands and on beaches (`vitals::smelt::MALACHITE_LANDS`): the
/// green carbonate, melted in a hot furnace as raw copper is (`casting.rs`).
pub static MALACHITE_ORE: Block = Block::named("malachite_ore");

/// The metals' items: (name, its tags).
const ITEMS: [(&str, &[&str]); 7] = [
    ("raw_copper", &["raw_ores/copper"]),
    ("malachite", &["raw_ores/copper", "malachite"]),
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
    for (block, metal, drop) in
        [(&COPPER_ORE, "copper", "raw_copper"), (&MALACHITE_ORE, "copper", "malachite"), (&TIN_ORE, "tin", "raw_tin")]
    {
        r.block(block)
            .sound_group("stone")
            .hardness(3.0)
            .mined_by("pickaxe", tool_tier::STONE)
            .drops(drop, 1)
            .tag("ores")
            .tag(&format!("ores/{metal}"))
            .tab(&crate::tabs::NATURAL);
    }
}

/// Where the ores lie: (ore block, biome tag or "" for anywhere, deepest y to highest y, veins a
/// cube, vein size). Iron's 6 veins of 8 is the yardstick. Copper near the surface of the dry lands
/// and beaches is malachite (`vitals::smelt::MALACHITE_LANDS`, `malachite_where_the_smelt_says`).
pub const VEINS: [(&str, &str, i32, u32, u32); 7] = [
    ("copper_ore", "cold", 96, 6, 8),
    ("malachite_ore", "dry", 96, 6, 8),
    ("malachite_ore", "beach", 80, 5, 7),
    ("copper_ore", "", 0, 2, 6),
    ("tin_ore", "beach", 80, 2, 5),
    ("tin_ore", "mountain", 120, 2, 5),
    ("tin_ore", "", -16, 1, 4),
];

/// The ores in worldgen (`worldgen.rs`, with the others, left out where they'd touch a cave).
/// Each row is its own feature: a feature name is registered once, so the rows past an ore's
/// first are named `copper_ore/<biome>` (`copper_ore/deep` for the one anywhere).
pub fn ores(r: &mut Registry) {
    let mut seen: Vec<&str> = Vec::new();
    for (ore, biome, max_y, veins, size) in VEINS {
        let block = match ore {
            "copper_ore" => &COPPER_ORE,
            "malachite_ore" => &MALACHITE_ORE,
            _ => &TIN_ORE,
        };
        let first = !seen.contains(&ore);
        seen.push(ore);
        let o = r.ore(block);
        let o = if first {
            o
        } else {
            let place = if biome.is_empty() { "deep" } else { biome };
            o.named(&format!("{ore}/{place}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malachite_where_the_smelt_says() {
        for (ore, biome, ..) in VEINS {
            if ore != "tin_ore" && !biome.is_empty() {
                let malachite = vitals::smelt::MALACHITE_LANDS.contains(&biome);
                assert_eq!(ore == "malachite_ore", malachite, "{ore} in {biome}");
            }
        }
    }
}

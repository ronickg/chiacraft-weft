//! The progression graph (`notes/PROGRESSION.md`, rule 3): every recipe the mods register, as
//! data (`RECIPES`; a check reads the mods' sources and fails on a recipe output missing here),
//! what each station and gathering needs, and the age keys' routes by path. From them: what each
//! thing costs in effort minutes the cheapest way, whether everything can be reached from the
//! start, and whether anything at tier N needs something above N.

use std::collections::HashMap;

use vitals::curve::{AGES, Age, Em, Path, ROUTES};

use crate::content::{self, AXE, CUTTER, Get, Needs, PICK, PICK_IRON, PICK_STEEL, PICK_STONE};

/// Where a recipe is made.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum At {
    /// In the hand's 2 × 2 grid.
    Hand,
    /// At a crafting table (a 3 × 3 recipe).
    Table,
    /// Over a fire: a campfire or a furnace (food).
    Fire,
    /// In a furnace only (ores, glass, bricks).
    Furnace,
    /// At a station block.
    Station(&'static str),
    /// Breaking the first input in the world with what `Needs` (its input is the world's, not
    /// carried, so it costs only the recipe's time).
    Break(Needs),
    /// A bucket filled at a lake.
    Fill,
}

impl At {
    /// The station blocks it needs, any one of them.
    pub fn stations(self) -> Vec<&'static str> {
        match self {
            At::Hand | At::Break(_) | At::Fill => Vec::new(),
            At::Table => vec!["base:crafting_table"],
            At::Fire => vec!["base:campfire_laid", "base:furnace"],
            At::Furnace => vec!["base:furnace"],
            At::Station(s) => vec![s],
        }
    }

    pub fn name(self) -> String {
        match self {
            At::Hand => "hand".into(),
            At::Table => "table".into(),
            At::Fire => "fire".into(),
            At::Furnace => "furnace".into(),
            At::Station(s) => s.trim_start_matches("base:").into(),
            At::Break(Needs::Hand) => "picked".into(),
            At::Break(Needs::Tool(k, l)) => format!("broken ({k} {l})"),
            At::Fill => "filled".into(),
        }
    }
}

/// A recipe: `n` of `out` (an id, or a pattern for one per material) from `ins` (ids or `#groups`,
/// with counts), at `at`, in `secs` of a player's work.
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub out: &'static str,
    pub n: f32,
    pub ins: &'static [(&'static str, f32)],
    pub at: At,
    pub secs: f32,
}

use At::{Break, Fill, Fire, Furnace, Hand, Station, Table};

const fn c(out: &'static str, n: f32, ins: &'static [(&'static str, f32)], at: At) -> Recipe {
    Recipe { out, n, ins, at, secs: 3.0 }
}

/// A furnace's 200 ticks.
const SMELT: f32 = 10.0;

const fn s(out: &'static str, ins: &'static [(&'static str, f32)], at: At) -> Recipe {
    Recipe { out, n: 1.0, ins, at, secs: SMELT }
}

/// Breaking or picking in the world: `secs` to find and break one (a meteorite's site is far).
const fn b(out: &'static str, n: f32, from: &'static [(&'static str, f32)], needs: Needs, secs: f32) -> Recipe {
    Recipe { out, n, ins: from, at: Break(needs), secs }
}

macro_rules! clothes {
    ($m:literal, $x:literal) => {
        [
            c(concat!("base:", $m, "_hat"), 1.0, &[($x, 2.0), ("#string", 1.0)], Table),
            c(concat!("base:", $m, "_coat"), 1.0, &[($x, 4.0), ("#string", 2.0)], Table),
            c(concat!("base:", $m, "_trousers"), 1.0, &[($x, 3.0), ("#string", 2.0)], Table),
            c(concat!("base:", $m, "_boots"), 1.0, &[($x, 2.0), ("#string", 1.0)], Table),
        ]
    };
}

const CLOTHES: [[Recipe; 4]; 4] =
    [clothes!("fur", "base:fur"), clothes!("hide", "base:hide"), clothes!("wool", "base:wool"), clothes!("linen", "base:cloth")];

/// Every other recipe, by the file that registers it.
const RECIPES_LIST: &[Recipe] = &[
    // the ground and the first hours (first.rs, firebed.rs, crafts.rs)
    b("base:rock", 1.0, &[("base:loose_rock", 1.0)], Needs::Hand, 6.0),
    b("base:flint", 1.0, &[("base:loose_flint", 1.0)], Needs::Hand, 12.0),
    b("base:tektite", 2.0, &[("base:loose_tektite", 1.0)], Needs::Hand, 600.0),
    b("base:stick", 2.0, &[("base:fallen_branch", 1.0)], Needs::Hand, 6.0),
    b("base:plant_fibre", 1.0, &[("base:tall_grass", 1.0)], CUTTER, 1.0),
    b("base:oak_bark", 1.0, &[("base:oak_log", 1.0)], AXE, 5.0),
    c("base:sharp_flake", 1.0, &[("base:flint", 1.0)], Hand),
    c("base:part_felling_axe_head", 1.0, &[("base:flint", 1.0)], Hand),
    c("base:part_pick_head", 1.0, &[("base:flint", 2.0)], Hand),
    c("base:part_shovel_blade", 1.0, &[("base:flint", 2.0)], Hand),
    c("base:axe", 1.0, &[("base:part_felling_axe_head", 1.0), ("#stick", 1.0), ("#string", 2.0)], Hand),
    c("base:pickaxe", 1.0, &[("base:part_pick_head", 1.0), ("#stick", 1.0), ("#string", 2.0)], Hand),
    c("base:shovel", 1.0, &[("base:part_shovel_blade", 1.0), ("#stick", 1.0), ("#string", 2.0)], Hand),
    c("base:knife", 1.0, &[("base:sharp_flake", 1.0), ("#stick", 1.0), ("#string", 1.0)], Hand),
    c("base:fire_bed", 1.0, &[("base:plant_fibre", 1.0), ("#stick", 3.0)], Hand),
    c("base:bow_drill", 1.0, &[("#stick", 2.0), ("#string", 1.0)], Hand),
    c("base:hand_drill", 1.0, &[("#stick", 2.0)], Hand),
    // the ley lines (ley.rs): the magic path's first step
    c("base:dowsing_rod", 1.0, &[("#stick", 2.0), ("#string", 1.0), ("base:tektite", 1.0)], Hand),
    c("base:campfire_laid", 1.0, &[("#stick", 2.0), ("#logs", 2.0)], Hand),
    c("base:tanning_frame", 1.0, &[("#stick", 4.0), ("#string", 4.0)], Table),
    c("base:tanning_pit", 1.0, &[("#planks", 7.0)], Table),
    c("base:charcoal_clamp", 1.0, &[("base:dirt", 8.0), ("#logs", 1.0)], Table),
    Recipe { out: "base:leather", n: 1.0, ins: &[("#hides", 1.0)], at: Station("base:tanning_frame"), secs: 30.0 },
    Recipe {
        out: "base:sole_leather",
        n: 1.0,
        ins: &[("#hides", 1.0), ("#bark_tannin", 1.0)],
        at: Station("base:tanning_pit"),
        secs: 10.0,
    },
    Recipe { out: "base:coal", n: 1.0, ins: &[("#logs", 1.0)], at: Station("base:charcoal_clamp"), secs: 2.0 },
    // parts from the recipe book (parts.rs): any part, of any material its shape takes
    c("base:part_*", 1.0, &[("#part_material", 2.0)], Table),
    // tools built from parts (the assembly)
    c("base:hoe", 1.0, &[("base:part_hoe_blade", 1.0), ("base:part_haft", 1.0), ("base:part_lashing", 1.0)], Table),
    c("base:spear", 1.0, &[("base:part_leaf_point", 1.0), ("base:part_shaft", 1.0), ("base:part_lashing", 1.0)], Table),
    c("base:sword", 1.0, &[("base:part_straight_blade", 1.0), ("base:part_grip", 1.0), ("base:part_bar_guard", 1.0)], Table),
    c("base:bow", 1.0, &[("base:part_self_stave", 1.0), ("base:part_bowstring", 1.0)], Table),
    c("base:axe", 1.0, &[("base:part_felling_axe_head", 1.0), ("base:part_haft", 1.0), ("base:part_lashing", 1.0)], Table),
    c("base:pickaxe", 1.0, &[("base:part_pick_head", 1.0), ("base:part_haft", 1.0), ("base:part_lashing", 1.0)], Table),
    c("base:shovel", 1.0, &[("base:part_shovel_blade", 1.0), ("base:part_haft", 1.0), ("base:part_lashing", 1.0)], Table),
    c("base:knife", 1.0, &[("base:part_knife_blade", 1.0), ("base:part_grip", 1.0), ("base:part_lashing", 1.0)], Table),
    // Armour: a shell, straps and a lining; a look family's item takes its shell's material
    // (`parts::ARMOUR_LOOKS`: an iron or steel shell is worn as mail, a diamond one as gems).
    c("base:helmet", 1.0, &[("base:part_helmet_shell", 1.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:chestplate", 1.0, &[("base:part_chest_shell", 1.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:leggings", 1.0, &[("base:part_leg_shell", 1.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:boots", 1.0, &[("base:part_boot_shell", 1.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:helmet_mail", 1.0, &[("base:iron_ingot", 2.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:chestplate_mail", 1.0, &[("base:iron_ingot", 3.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:leggings_mail", 1.0, &[("base:iron_ingot", 3.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:boots_mail", 1.0, &[("base:iron_ingot", 2.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:helmet_gem", 1.0, &[("base:diamond", 2.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:chestplate_gem", 1.0, &[("base:diamond", 3.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:leggings_gem", 1.0, &[("base:diamond", 3.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    c("base:boots_gem", 1.0, &[("base:diamond", 2.0), ("base:part_straps", 1.0), ("base:part_lining", 1.0)], Table),
    // recipes.rs
    c("base:*_planks", 4.0, &[("#logs", 1.0)], Hand),
    c("base:stick", 4.0, &[("#planks", 2.0)], Hand),
    c("base:crafting_table", 1.0, &[("#planks", 4.0)], Hand),
    c("base:furnace", 1.0, &[("#cobblestone", 8.0)], Table),
    c("base:chest", 1.0, &[("#planks", 8.0)], Table),
    c("base:torch", 4.0, &[("#coal", 1.0), ("#stick", 1.0)], Hand),
    c("base:ladder", 3.0, &[("#stick", 7.0)], Table),
    c("base:bread", 1.0, &[("#bread_grain", 3.0)], Table),
    c("base:sugar", 1.0, &[("base:sugar_cane", 1.0)], Hand),
    c("base:paper", 3.0, &[("base:sugar_cane", 3.0)], Table),
    c("base:book", 1.0, &[("base:paper", 3.0), ("base:leather", 1.0)], Hand),
    c("base:bookshelf", 1.0, &[("#planks", 6.0), ("base:book", 3.0)], Table),
    s("base:cooked_egg", &[("base:egg", 1.0)], Fire),
    s("base:cooked_wild_rice", &[("base:rice", 1.0)], Fire),
    c("base:apple_pie", 1.0, &[("base:apple", 1.0), ("base:sugar", 1.0), ("base:egg", 1.0), ("base:wheat", 1.0)], Hand),
    c("base:cacao_beans", 1.0, &[("base:cacao_pod", 1.0)], Hand),
    s("base:roasted_cacao", &[("base:cacao_beans", 1.0)], Fire),
    c("base:hay_bale", 1.0, &[("base:wheat", 9.0)], Table),
    c("base:wheat", 9.0, &[("base:hay_bale", 1.0)], Hand),
    c("base:stone_bricks", 4.0, &[("base:stone", 4.0)], Hand),
    c("base:wool", 1.0, &[("#string", 4.0)], Hand),
    c("base:bed", 1.0, &[("base:wool", 3.0), ("#planks", 3.0)], Table),
    c("base:bucket", 1.0, &[("base:iron_ingot", 3.0)], Table),
    c("base:glowstone", 1.0, &[("base:torch", 8.0), ("base:glass", 1.0)], Table),
    c("base:red_lamp", 1.0, &[("base:glowstone", 1.0), ("base:rose", 1.0)], Hand),
    c("base:green_lamp", 1.0, &[("base:glowstone", 1.0), ("base:cactus", 1.0)], Hand),
    c("base:blue_lamp", 1.0, &[("base:glowstone", 1.0), ("base:ice", 1.0)], Hand),
    c("base:red_stained_glass", 1.0, &[("base:glass", 1.0), ("base:rose", 1.0)], Hand),
    c("base:green_stained_glass", 1.0, &[("base:glass", 1.0), ("base:cactus", 1.0)], Hand),
    c("base:blue_stained_glass", 1.0, &[("base:glass", 1.0), ("base:ice", 1.0)], Hand),
    c("base:iron_block", 1.0, &[("base:iron_ingot", 9.0)], Table),
    c("base:gold_block", 1.0, &[("base:gold_ingot", 9.0)], Table),
    c("base:diamond_block", 1.0, &[("base:diamond", 9.0)], Table),
    c("base:coal_block", 1.0, &[("base:coal", 9.0)], Table),
    c("base:iron_ingot", 9.0, &[("base:iron_block", 1.0)], Hand),
    c("base:gold_ingot", 9.0, &[("base:gold_block", 1.0)], Hand),
    c("base:diamond", 9.0, &[("base:diamond_block", 1.0)], Hand),
    c("base:coal", 9.0, &[("base:coal_block", 1.0)], Hand),
    s("base:iron_ingot", &[("base:raw_iron", 1.0)], Furnace),
    s("base:gold_ingot", &[("base:raw_gold", 1.0)], Furnace),
    s("base:cooked_beef", &[("base:beef", 1.0)], Fire),
    s("base:cooked_porkchop", &[("base:porkchop", 1.0)], Fire),
    s("base:cooked_chicken", &[("base:chicken", 1.0)], Fire),
    s("base:cooked_mutton", &[("base:mutton", 1.0)], Fire),
    s("base:glass", &[("#sand", 1.0)], Furnace),
    s("base:stone", &[("#cobblestone", 1.0)], Furnace),
    s("base:coal", &[("#logs", 1.0)], Furnace),
    s("base:bricks", &[("base:clay", 1.0)], Furnace),
    // ores and the sky (blocks.rs, meteors.rs, stars.rs)
    b("base:coal", 1.0, &[("base:coal_ore", 1.0)], PICK, 30.0),
    b("base:raw_iron", 1.0, &[("base:iron_ore", 1.0)], PICK_STONE, 60.0),
    b("base:raw_gold", 1.0, &[("base:gold_ore", 1.0)], PICK_STONE, 180.0),
    b("base:diamond", 1.0, &[("base:diamond_ore", 1.0)], PICK_STEEL, 600.0),
    b("base:redstone", 4.0, &[("base:redstone_ore", 1.0)], PICK_IRON, 120.0),
    b("base:meteoric_iron", 2.0, &[("base:meteorite", 1.0)], PICK_STONE, 1200.0),
    b("base:star_steel_lump", 2.5, &[("base:star_steel_ore", 1.0)], PICK_IRON, 1800.0),
    b("base:starheart", 1.0, &[("base:star_steel_ore", 1.0)], PICK_IRON, 3600.0),
    c("base:peridot", 2.0, &[("base:pallasite", 1.0)], Hand),
    c("base:star_lantern", 1.0, &[("base:iron_ingot", 2.0), ("base:starheart", 1.0)], Table),
    c("base:unlearning_stone", 1.0, &[("base:peridot", 1.0), ("base:tektite", 2.0)], Table),
    Recipe { out: "base:star_steel_ingot", n: 1.0, ins: &[("base:star_steel_lump", 1.0)], at: Furnace, secs: 20.0 },
    Recipe { out: "base:water_bucket", n: 1.0, ins: &[("base:bucket", 1.0)], at: Fill, secs: 10.0 },
    Recipe { out: "base:lava_bucket", n: 1.0, ins: &[("base:bucket", 1.0)], at: Fill, secs: 30.0 },
    // redstone.rs
    c("base:redstone_block", 1.0, &[("base:redstone", 9.0)], Table),
    c("base:redstone", 9.0, &[("base:redstone_block", 1.0)], Hand),
    c("base:redstone_torch", 1.0, &[("base:redstone", 1.0), ("#stick", 1.0)], Hand),
    c("base:repeater", 1.0, &[("base:redstone_torch", 2.0), ("base:redstone", 1.0), ("base:stone", 3.0)], Table),
    c("base:comparator", 1.0, &[("base:redstone_torch", 3.0), ("base:redstone", 1.0), ("base:stone", 3.0)], Table),
    c("base:lever", 1.0, &[("#stick", 1.0), ("#cobblestone", 1.0)], Hand),
    c("base:stone_button", 1.0, &[("base:stone", 1.0)], Hand),
    c("base:redstone_lamp", 1.0, &[("base:redstone", 4.0), ("base:glowstone", 1.0)], Table),
    c("base:piston", 1.0, &[("#planks", 3.0), ("#cobblestone", 4.0), ("base:iron_ingot", 1.0), ("base:redstone", 1.0)], Table),
    c("base:slime_block", 1.0, &[("base:clay", 8.0), ("base:wheat", 1.0)], Table),
    c("base:sticky_piston", 1.0, &[("base:slime_block", 1.0), ("base:piston", 1.0)], Hand),
    c("base:floater", 1.0, &[("base:iron_ingot", 4.0), ("base:redstone", 1.0), ("base:piston", 2.0)], Table),
    c("base:golden_lamp", 1.0, &[("base:gold_ingot", 4.0), ("base:redstone", 1.0)], Table),
    c("base:anchor", 1.0, &[("base:iron_ingot", 8.0), ("base:redstone", 1.0)], Table),
    c("base:observer", 1.0, &[("#cobblestone", 6.0), ("base:redstone", 2.0), ("base:iron_ingot", 1.0)], Table),
    c("base:dispenser", 1.0, &[("#cobblestone", 7.0), ("#stick", 1.0), ("base:redstone", 1.0)], Table),
    // cooking.rs, crops.rs, farming.rs, survival/body.rs
    c("base:bowl", 4.0, &[("#planks", 3.0)], Table),
    s("base:salt", &[("base:water_bucket", 1.0)], Furnace),
    c("base:salted_meat", 1.0, &[("#raw_meat", 1.0), ("base:salt", 1.0)], Hand),
    c("base:cooking_pot", 1.0, &[("base:clay", 7.0)], Table),
    c("base:drying_rack", 1.0, &[("#stick", 5.0)], Table),
    Recipe {
        out: "base:stew",
        n: 1.0,
        ins: &[("base:bowl", 1.0), ("#raw_meat", 1.0), ("#veg", 1.0)],
        at: Station("base:cooking_pot"),
        secs: 10.0,
    },
    Recipe { out: "base:dried_meat", n: 1.0, ins: &[("#raw_meat", 1.0)], at: Station("base:drying_rack"), secs: 3.0 },
    Recipe { out: "base:dried_apple", n: 1.0, ins: &[("base:apple", 1.0)], at: Station("base:drying_rack"), secs: 3.0 },
    Recipe { out: "base:dried_berries", n: 1.0, ins: &[("base:berries", 1.0)], at: Station("base:drying_rack"), secs: 3.0 },
    s("base:smoked_meat", &[("#raw_meat", 1.0)], Fire),
    s("base:baked_potato", &[("base:potato", 1.0)], Fire),
    s("base:roasted_corn", &[("base:corn", 1.0)], Fire),
    s("base:cooked_rice", &[("base:rice_seeds", 1.0)], Fire),
    b("base:pumpkin_slice", 4.0, &[("base:pumpkin", 1.0)], Needs::Hand, 3.0),
    b("base:melon_slice", 4.0, &[("base:melon", 1.0)], Needs::Hand, 3.0),
    c("base:pumpkin_seeds", 2.0, &[("base:pumpkin_slice", 1.0)], Hand),
    c("base:melon_seeds", 2.0, &[("base:melon_slice", 1.0)], Hand),
    c("base:compost_bin", 1.0, &[("#planks", 7.0)], Table),
    Recipe { out: "base:compost", n: 1.0, ins: &[("#compostable", 4.0)], at: Station("base:compost_bin"), secs: 3.0 },
    c("base:bandage", 3.0, &[("base:wool", 1.0)], Hand),
    // clothing.rs
    c("base:string", 1.0, &[("base:plant_fibre", 2.0)], Hand),
    c("base:cloth", 1.0, &[("#string", 3.0)], Table),
    c("base:wool_cloak", 1.0, &[("base:wool", 3.0), ("#string", 1.0)], Table),
    c("base:fur_robe", 1.0, &[("base:fur", 5.0), ("#string", 1.0)], Table),
    c("base:rain_cape", 1.0, &[("base:leather", 5.0), ("#string", 1.0)], Table),
    // glider.rs
    c("base:glider", 1.0, &[("base:leather", 3.0), ("#stick", 4.0), ("#string", 1.0)], Table),
    // grapple.rs
    c("base:grappling_hook", 1.0, &[("base:iron_ingot", 3.0), ("#string", 3.0)], Table),
    // the animals mod (knockout.rs, screen.rs, gateway.rs, riding.rs, taming.rs, eggs.rs)
    c("animals:club", 1.0, &[("#logs", 1.0), ("base:stick", 1.0)], Hand),
    c("animals:sedative", 2.0, &[("base:spider_eye", 1.0), ("base:sugar", 1.0), ("base:wheat_seeds", 1.0)], Hand),
    c("animals:tranquilizer_dart", 4.0, &[("base:stick", 1.0), ("base:feather", 1.0), ("base:spider_eye", 1.0)], Hand),
    c("animals:spyglass", 1.0, &[("base:glass", 1.0), ("base:iron_ingot", 2.0)], Table),
    c("animals:amber_block", 1.0, &[("animals:amber", 4.0)], Hand),
    c("animals:amber", 4.0, &[("animals:amber_block", 1.0)], Hand),
    b("animals:amber", 1.0, &[("animals:amber_ore", 1.0)], PICK_STONE, 120.0),
    c("animals:utahraptor_saddle", 1.0, &[("base:leather", 3.0), ("base:string", 2.0)], Table),
    c("animals:wyvern_saddle", 1.0, &[("base:leather", 5.0), ("base:iron_ingot", 1.0), ("base:string", 2.0)], Table),
    Recipe {
        out: "animals:kibble",
        n: 4.0,
        ins: &[("base:egg", 1.0), ("#kibble_filling", 1.0), ("base:wheat", 1.0), ("base:plant_fibre", 1.0)],
        at: Hand,
        secs: 3.0,
    },
    c("animals:nest", 1.0, &[("base:stick", 4.0), ("base:wheat", 1.0)], Table),
    s("animals:cooked_venison", &[("animals:venison", 1.0)], Fire),
    s("animals:cooked_hare", &[("animals:hare", 1.0)], Fire),
    s("animals:cooked_trout", &[("animals:trout", 1.0)], Fire),
    s("animals:cooked_pacu", &[("animals:pacu", 1.0)], Fire),
];

/// Every recipe.
pub fn recipes() -> Vec<&'static Recipe> {
    CLOTHES.iter().flatten().chain(RECIPES_LIST).collect()
}

/// The `#groups` recipes take: any one of their members.
pub const GROUPS: &[(&str, &[&str])] = &[
    ("#logs", &["base:*_log", "animals:*_log"]),
    ("#planks", &["base:*_planks"]),
    ("#stick", &["base:stick"]),
    ("#string", &["base:string"]),
    ("#cobblestone", &["base:cobblestone", "base:mossy_cobblestone"]),
    ("#coal", &["base:coal"]),
    ("#sand", &["base:sand", "animals:red_sand"]),
    ("#hides", &["base:hide"]),
    ("#bark_tannin", &["base:oak_bark"]),
    ("#bread_grain", &["base:wheat", "base:rye", "base:barley", "base:sorghum", "base:millet"]),
    (
        "#raw_meat",
        &["base:beef", "base:porkchop", "base:chicken", "base:mutton", "animals:venison", "animals:hare", "animals:trout", "animals:pacu"],
    ),
    ("#veg", &["base:carrot", "base:potato", "base:beetroot", "base:beans", "base:cassava", "base:mushroom_brown"]),
    ("#kibble_filling", &["base:beef", "base:porkchop", "base:chicken", "base:mutton", "base:apple", "base:sugar_cane"]),
    ("#compostable", &["base:dung", "base:*_leaves"]),
    (
        "#part_material",
        &[
            "base:flint",
            "base:tektite",
            "base:cobblestone",
            "base:bone",
            "base:*_planks",
            "base:*_log",
            "base:string",
            "base:sole_leather",
            "base:iron_ingot",
            "base:meteoric_iron",
            "base:star_steel_ingot",
            "base:gold_ingot",
            "base:diamond",
            "base:wool",
            "base:cloth",
            "base:fur",
        ],
    ),
];

/// What can break what: a tool item, its kind and the level it harvests. Every tool is built from
/// parts, so a kind's item stands for its cheapest (a flint head, harvest 100); a better head's
/// material stands for the tool it makes (`vitals::gear::harvest` by its level: wrought iron 280,
/// star steel 340, diamond 400; a check keeps these equal to the curve's).
pub const TOOLS: &[(&str, &str, u32)] = &[
    ("base:axe", "axe", 100),
    ("base:pickaxe", "pickaxe", 100),
    ("base:iron_ingot", "pickaxe", 280),
    ("base:meteoric_iron", "pickaxe", 230),
    ("base:star_steel_ingot", "pickaxe", 340),
    ("base:diamond", "pickaxe", 400),
    ("base:shovel", "shovel", 100),
    ("base:sharp_flake", "cutter", 0),
    ("base:knife", "cutter", 0),
];

/// The material each stand-in in `TOOLS` is a head of.
pub const TOOL_HEADS: &[(&str, &str)] = &[
    ("base:axe", "flint"),
    ("base:pickaxe", "flint"),
    ("base:iron_ingot", "wrought_iron"),
    ("base:meteoric_iron", "meteoric_iron"),
    ("base:star_steel_ingot", "star_steel"),
    ("base:diamond", "diamond"),
    ("base:shovel", "flint"),
];

/// The tools that meet `needs`.
pub fn tools_for(needs: Needs) -> impl Iterator<Item = &'static str> {
    TOOLS
        .iter()
        .filter_map(move |&(id, kind, level)| match needs {
            Needs::Hand => None,
            Needs::Tool(k, l) => (k == kind && level >= l).then_some(id),
        })
}

/// An age's key (`notes/PROGRESSION.md`, "How an age is gated"): what moving on from it takes, by
/// each path's route, and the gates not built yet.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub age: Age,
    pub routes: &'static [(Path, &'static [&'static str])],
    /// The gates (and routes) that have nothing in the game yet.
    pub missing: &'static [&'static str],
}

/// The keys so far: only the Stone Age has parts of one (11-07).
pub const KEYS: &[Key] = &[Key {
    age: Age::Stone,
    routes: &[
        (Path::Tech, &["base:pickaxe", "base:furnace"]),
        (Path::Beasts, &["animals:club", "animals:tranquilizer_dart", "animals:nest"]),
        (Path::Magic, &["base:dowsing_rod"]),
    ],
    missing: &["a biome boss's part", "knowledge: a ruin's blueprint or lore page"],
}];

// ------------------------------------------------------------------ the graph, solved

/// What the graph comes to: each id's cheapest cost in effort minutes (missing: unreachable).
pub struct Solved {
    pub ids: Vec<String>,
    pub cost: HashMap<String, Em>,
    /// The recipe each was cheapest by (an index into `recipes()`), none for gathered ones.
    pub by: HashMap<String, usize>,
}

/// The ids of the universe a pattern or group names.
pub fn members<'a>(ids: &'a [String], name: &str) -> Vec<&'a String> {
    let pats: Vec<&str> = match GROUPS.iter().find(|(g, _)| *g == name) {
        Some((_, p)) => p.to_vec(),
        None if name.starts_with('#') => Vec::new(),
        None => vec![name],
    };
    ids.iter()
        .filter(|id| pats.iter().any(|p| content::matches(p, id)))
        .collect()
}

impl Solved {
    pub fn cost_of(&self, name: &str) -> Option<Em> {
        if name.starts_with('#') {
            members(&self.ids, name)
                .into_iter()
                .filter_map(|m| self.cost.get(m.as_str()).copied())
                .reduce(f32::min)
        } else {
            self.cost.get(name).copied()
        }
    }

    pub fn reachable(&self, name: &str) -> bool {
        self.cost_of(name).is_some()
    }

    fn can(&self, needs: Needs) -> bool {
        match needs {
            Needs::Hand => true,
            n => tools_for(n).any(|t| self.reachable(t)),
        }
    }

    fn at_ok(&self, at: At) -> bool {
        let stations = at.stations();
        (stations.is_empty() || stations.iter().any(|s| self.reachable(s))) && if let At::Break(n) = at { self.can(n) } else { true }
    }
}

/// Solves the graph: from the world's gatherings, every recipe relaxed until nothing gets cheaper
/// (a tool or station unlocks what needs it once it is reachable).
pub fn solve() -> Solved {
    let ids = content::universe();
    let mut g = Solved { ids: ids.clone(), cost: HashMap::new(), by: HashMap::new() };
    let all = recipes();
    let outs: Vec<Vec<String>> = all
        .iter()
        .map(|r| members(&ids, r.out).into_iter().cloned().collect())
        .collect();
    for _ in 0..200 {
        let mut changed = false;
        let mut relax = |g: &mut Solved, id: &str, v: Em, by: Option<usize>| {
            if g.cost.get(id).is_none_or(|&old| v < old - 1e-6) {
                g.cost.insert(id.to_string(), v);
                match by {
                    Some(i) => g.by.insert(id.to_string(), i),
                    None => g.by.remove(id),
                };
                changed = true;
            }
        };
        for id in &ids {
            match content::rule(id).map(|r| r.get) {
                Some(Get::World(needs, em)) if g.can(needs) => relax(&mut g, id, em, None),
                Some(Get::Drop(em)) => relax(&mut g, id, em, None),
                _ => {}
            }
        }
        for (i, r) in all.iter().enumerate() {
            if !g.at_ok(r.at) {
                continue;
            }
            let ins = if matches!(r.at, At::Break(_)) {
                Some(0.0)
            } else {
                r.ins
                    .iter()
                    .map(|&(name, n)| g.cost_of(name).map(|c| c * n))
                    .sum::<Option<Em>>()
            };
            let Some(ins) = ins else { continue };
            let unit = (ins + r.secs / 60.0) / r.n;
            for id in &outs[i] {
                relax(&mut g, id, unit, Some(i));
            }
        }
        if !changed {
            break;
        }
    }
    g
}

// ------------------------------------------------------------------ the checks

/// The lowest tier among what a name stands for (a group's cheapest member by tier).
fn tier_of(ids: &[String], name: &str) -> Option<u8> {
    members(ids, name)
        .into_iter()
        .filter_map(|m| content::tier(m))
        .min()
}

/// The lowest tier of a tool that meets `needs`.
fn needs_tier(needs: Needs) -> u8 {
    tools_for(needs)
        .filter_map(content::tier)
        .min()
        .unwrap_or(0)
}

/// Every id has a tag; every made one has a recipe; every recipe names real things.
pub fn tagging(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    let all = recipes();
    for id in &g.ids {
        match content::rule(id) {
            None => out.push(format!("{id} has no tag (add it to `content::RULES`)")),
            Some(r) if r.get == Get::Made && !all.iter().any(|x| content::matches(x.out, id)) => {
                out.push(format!("{id} is made, but no recipe in `graph::RECIPES` makes it"))
            }
            _ => {}
        }
    }
    for r in &all {
        if members(&g.ids, r.out).is_empty() {
            out.push(format!("a recipe makes {}, which has no name in the lang files", r.out));
        }
        for (name, _) in r.ins {
            if members(&g.ids, name).is_empty() {
                out.push(format!("{}'s recipe takes {name}, which names nothing", r.out));
            }
        }
    }
    out
}

/// Everything that can be had is reachable from the start.
pub fn reach(g: &Solved) -> Vec<String> {
    g.ids
        .iter()
        .filter(|id| content::rule(id).is_some_and(|r| r.get != Get::Not) && !g.reachable(id))
        .map(|id| format!("{id} can't be reached from the start"))
        .collect()
}

/// Nothing at tier N needs anything above N: a recipe's inputs and station, a gathering's tool.
pub fn tiers(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    for r in recipes() {
        for id in members(&g.ids, r.out) {
            let Some(t) = content::tier(id) else { continue };
            let mut need = Vec::new();
            if let At::Break(n) = r.at {
                need.push((format!("a {}", n_name(n)), needs_tier(n)));
            } else {
                for (name, _) in r.ins {
                    if let Some(x) = tier_of(&g.ids, name) {
                        need.push((name.to_string(), x));
                    }
                }
            }
            if let Some(x) = r.at.stations().into_iter().filter_map(content::tier).min() {
                need.push((r.at.name(), x));
            }
            for (what, x) in need {
                if x > t {
                    out.push(format!("{id} (tier {t}) needs {what} (tier {x})"));
                }
            }
        }
    }
    for id in &g.ids {
        if let Some(r) = content::rule(id)
            && let Get::World(n, _) = r.get
            && needs_tier(n) > r.tier
        {
            out.push(format!("{id} (tier {}) needs a {} (tier {})", r.tier, n_name(n), needs_tier(n)));
        }
    }
    out
}

fn n_name(n: Needs) -> String {
    match n {
        Needs::Hand => "hand".into(),
        Needs::Tool(k, l) => format!("{k} of level {l}"),
    }
}

/// Each age's key can be made by each route it declares, inside its age.
pub fn keys(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    for k in KEYS {
        for (path, items) in k.routes {
            for id in *items {
                match content::tier(id) {
                    None => out.push(format!("{}'s {} key takes {id}, which has no tag", k.age.name(), path.name())),
                    Some(t) if t > k.age.tier() => out.push(format!("{}'s {} key takes {id} of tier {t}", k.age.name(), path.name())),
                    _ if !g.reachable(id) => out.push(format!("{}'s {} key takes {id}, which can't be reached", k.age.name(), path.name())),
                    _ => {}
                }
            }
        }
    }
    out
}

/// The routes each age's key has, by path, for the sheet: `None` where a path has none yet.
pub fn routes(age: Age) -> Vec<(Path, Option<&'static [&'static str]>)> {
    let key = KEYS.iter().find(|k| k.age == age);
    ROUTES
        .iter()
        .map(|&p| (p, key.and_then(|k| k.routes.iter().find(|(q, _)| *q == p).map(|(_, i)| *i))))
        .collect()
}

/// Every crafting material has a tag, made from something of its tier or below.
pub fn materials(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    for m in crafting::MATERIALS {
        match content::material(m.id) {
            None => out.push(format!("material {} has no tag (add it to `content::MATERIALS`)", m.id)),
            Some(t) if t.tier().is_none() => out.push(format!("material {} has no age in `vitals::gear::MATERIALS`", m.id)),
            Some(t) if !t.from.is_empty() => match content::tier(t.from) {
                None => out.push(format!("material {} is made from {}, which has no tag", m.id, t.from)),
                Some(x) if x > t.tier().unwrap_or(0) => {
                    out.push(format!("material {} (tier {}) is made from {} (tier {x})", m.id, t.tier().unwrap_or(0), t.from))
                }
                _ if !g.reachable(t.from) => out.push(format!("material {}'s {} can't be reached", m.id, t.from)),
                _ => {}
            },
            _ => {}
        }
    }
    out
}

/// What making one of everything made at `age` costs, effort minutes.
pub fn making(g: &Solved, age: Age) -> (usize, Em) {
    let made: Vec<&String> = g
        .ids
        .iter()
        .filter(|id| content::rule(id).is_some_and(|r| r.get == Get::Made && r.tier == age.tier()))
        .collect();
    let em = made.iter().filter_map(|id| g.cost.get(id.as_str())).sum();
    (made.len(), em)
}

/// What an average player makes in an age, the first time through (one route's tools, one
/// armour or outfit, the stations and light it uses): the first crafting-time budget. Alternatives
/// (gold beside iron, four outfits) aren't in it; `making` sums those for the sheet.
pub const KITS: &[(Age, &[(&str, f32)])] = &[
    (
        Age::Stone,
        &[
            ("base:axe", 1.0),
            ("base:pickaxe", 1.0),
            ("base:shovel", 1.0),
            ("base:knife", 1.0),
            ("base:spear", 1.0),
            ("base:hand_drill", 1.0),
            ("base:campfire_laid", 2.0),
            ("base:crafting_table", 1.0),
            ("base:chest", 2.0),
            ("base:bed", 1.0),
            ("base:torch", 16.0),
            ("base:tanning_frame", 1.0),
            ("base:drying_rack", 1.0),
            ("base:cooking_pot", 1.0),
            ("base:hide_hat", 1.0),
            ("base:hide_coat", 1.0),
            ("base:hide_trousers", 1.0),
            ("base:hide_boots", 1.0),
            ("base:furnace", 1.0),
            ("animals:club", 1.0),
            ("animals:tranquilizer_dart", 8.0),
            ("animals:kibble", 8.0),
            ("animals:nest", 1.0),
        ],
    ),
    (
        Age::Bronze,
        &[
            ("base:glass", 8.0),
            ("base:stone_bricks", 32.0),
            ("base:bricks", 32.0),
            ("base:glowstone", 2.0),
            ("base:meteoric_iron", 4.0),
            ("base:gold_ingot", 4.0),
        ],
    ),
    (
        Age::Iron,
        &[
            // An iron pick, axe and sword and a mail set, built from parts: their iron (3 for a
            // pick or axe head, 2 for a blade, the shells 2, 3, 3, 2) and wool linings.
            ("base:pickaxe", 1.0),
            ("base:axe", 1.0),
            ("base:sword", 1.0),
            ("base:helmet_mail", 1.0),
            ("base:chestplate_mail", 1.0),
            ("base:leggings_mail", 1.0),
            ("base:boots_mail", 1.0),
            ("base:iron_ingot", 8.0),
            ("base:wool", 4.0),
            ("base:bucket", 2.0),
            ("animals:spyglass", 1.0),
            ("animals:wyvern_saddle", 1.0),
        ],
    ),
    (
        Age::Steam,
        &[
            // A diamond pick head and sword blade on the Iron Age's hafts and hilts.
            ("base:diamond", 5.0),
            ("base:piston", 4.0),
            ("base:observer", 2.0),
            ("base:repeater", 4.0),
            ("base:redstone_lamp", 4.0),
            ("base:dispenser", 2.0),
        ],
    ),
];

/// An age's kit: its cost in effort minutes, and what in it can't be had.
pub fn kit(g: &Solved, age: Age) -> (Em, Vec<&'static str>) {
    let mut em = 0.0;
    let mut missing = Vec::new();
    for (a, items) in KITS {
        if *a != age {
            continue;
        }
        for &(id, n) in *items {
            match g.cost.get(id) {
                Some(c) => em += c * n,
                None => missing.push(id),
            }
        }
    }
    (em, missing)
}

/// An age's kit stays inside its share of the age's hours (`curve::MAKING_SHARE`), is of its age
/// or below, and can be had.
pub fn budget(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    for &a in &AGES {
        let (em, missing) = kit(g, a);
        let cap = a.em() * vitals::curve::MAKING_SHARE;
        if em > cap {
            out.push(format!("{}: its kit takes {em:.0} EM, over its {cap:.0}", a.name()));
        }
        for id in missing {
            out.push(format!("{}'s kit has {id}, which can't be had", a.name()));
        }
    }
    for (a, items) in KITS {
        for (id, _) in *items {
            match content::tier(id) {
                Some(t) if t > a.tier() => out.push(format!("{}'s kit has {id} of tier {t}", a.name())),
                None => out.push(format!("{}'s kit has {id}, which has no tag", a.name())),
                _ => {}
            }
        }
    }
    out
}

/// The recipes whose outputs the mods' sources name as plain strings (`.shaped(KIND, "name"` and
/// `.shapeless(...)`), each needing a recipe here. Read from the sources at the time of the check.
pub fn declared(g: &Solved) -> Vec<String> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let mut out = Vec::new();
    let all = recipes();
    for ns in ["base", "animals"] {
        let mut files = Vec::new();
        collect_rs(&std::path::Path::new(root).join(ns).join("src"), &mut files);
        for f in files {
            let Ok(text) = std::fs::read_to_string(&f) else { continue };
            let f = f.strip_prefix(root).map(|p| p.to_path_buf()).unwrap_or(f);
            for name in literal_outputs(&text) {
                let id = format!("{ns}:{name}");
                if !all.iter().any(|r| content::matches(r.out, &id)) && g.ids.contains(&id) {
                    out.push(format!("{} registers a recipe for {id} that `graph::RECIPES` doesn't have", f.display()));
                } else if !g.ids.contains(&id) {
                    out.push(format!("{} registers a recipe for {id}, which has no name in the lang files", f.display()));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn collect_rs(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// Plain output names of `.shaped(` and `.shapeless(` calls: the second argument when it's a
/// string literal that isn't a `#tag` or a `{m}` pattern.
pub fn literal_outputs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for call in [".shaped(", ".shapeless("] {
        let mut rest = text;
        while let Some(i) = rest.find(call) {
            rest = &rest[i + call.len()..];
            let Some(comma) = rest.find(',') else { break };
            let arg = rest[comma + 1..].trim_start();
            if let Some(lit) = arg.strip_prefix('"')
                && let Some(end) = lit.find('"')
            {
                let name = &lit[..end];
                if !name.starts_with('#') && !name.contains('{') {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_stand_ins_harvest_as_their_heads() {
        for &(item, kind, level) in TOOLS.iter().filter(|t| t.1 != "cutter") {
            let (_, head) = TOOL_HEADS
                .iter()
                .find(|h| h.0 == item)
                .unwrap_or_else(|| panic!("{item}"));
            let m = crafting::materials::material(head).unwrap();
            assert_eq!(level, m.harvest(), "{item} as a {kind}");
        }
    }

    #[test]
    fn literal_outputs_are_found() {
        let src = "r.shaped(C, \"chest\", 1, &[\"# #\"], &[]); r.shapeless(&CRAFTING, \"#planks/{m}\", 4, &[], 0); r.shapeless(S, \"salt\", 1, &[], 2);";
        assert_eq!(literal_outputs(src), vec!["chest", "salt"]);
    }
}

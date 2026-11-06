//! The RTX showcase blocks (`notes/RTX_SHOWCASE.md`, Ronald 11-06: "some like cool glass or tile
//! things ... that's sort of where the RTX stuff really shine"): glass, polished stone and tiles,
//! glazed tiles, metal plates and a lantern. Their shine is in their textures' material maps
//! (LabPBR `_s`/`_n`, `notes/blocks/batches/matmaps.tsv`), which the shader pack and the traced
//! reflections read; as blocks they are plain building blocks.

use weft_sdk::*;

use base_api::CRAFTING;

const C: &RecipeKind = &CRAFTING;

/// The glazed tiles' colours: one per dye.
const GLAZES: [&str; 3] = ["blue", "green", "red"];

pub static VERDIGRIS_COPPER: Block = Block::named("verdigris_copper");

pub fn register(r: &mut Registry) {
    // ---- glass: see-through, dimming the light passing through a little (frosted), much (smoked),
    // or a little in its colours (leaded; traced shadows take each pane's colour)
    for (name, dim) in [("frosted_glass", 2u8), ("smoked_glass", 9), ("leaded_glass", 4)] {
        r.block(Block::generated(name.to_string()))
            .signal_blocker()
            .sound_group("glass")
            .translucent()
            .light_filter(dim, dim, dim)
            .cull_same()
            .hardness(0.3)
            .drops_nothing()
            .tab(&crate::tabs::BUILDING);
    }
    // ---- polished stone and tiles
    // In stairs and slabs too (`has_shapes`), for steps down into the pool and the hall's trims.
    for (name, hardness) in [("polished_limestone", 1.5), ("limestone_tiles", 1.5), ("slate_tiles", 1.5), ("checker_tiles", 1.5)] {
        r.block(Block::generated(name.to_string()))
            .sound_group("stone")
            .tag("has_shapes")
            .hardness(hardness)
            .mineable("pickaxe")
            .needs_tool_level(0)
            .tab(&crate::tabs::BUILDING);
    }
    r.block(Block::generated("polished_obsidian".to_string()))
        .immovable()
        .sound_group("stone")
        .hardness(50.0)
        .mineable("pickaxe")
        .needs_tool_level(tool_tier::DIAMOND)
        .tab(&crate::tabs::BUILDING);
    for m in GLAZES {
        r.block(Block::generated(format!("{m}_glazed_tiles")))
            .sound_group("stone")
            .tag(&format!("glazed_tiles/{m}"))
            .texture(&format!("glazed_tiles_{m}"))
            .hardness(1.4)
            .mineable("pickaxe")
            .needs_tool_level(0)
            .tab(&crate::tabs::COLORED);
    }
    // ---- metal
    // Bare copper weathers to verdigris in the open air over a few days (`weather_copper`).
    r.block(Block::generated("copper_block".to_string()))
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool_level(tool_tier::STONE)
        // Packs `ingots/copper` nine to one and back (`recipes.rs`).
        .tag("ore_blocks/copper")
        .grows(vitals::weathering::VERDIGRIS_SECS, 0)
        .on_grow(weather_copper)
        .tab(&crate::tabs::BUILDING);
    r.block(&VERDIGRIS_COPPER)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool_level(tool_tier::STONE)
        .drops("copper_block", 1)
        .tab(&crate::tabs::BUILDING);
    for name in ["bronze_plate", "steel_plate"] {
        r.block(Block::generated(name.to_string()))
            .sound_group("stone")
            .hardness(4.0)
            .mineable("pickaxe")
            .needs_tool_level(tool_tier::STONE)
            .tab(&crate::tabs::BUILDING);
    }
    // ---- light: a warm lamp of frosted glass in a bronze frame
    r.block(Block::generated("bronze_lantern".to_string()))
        .sound_group("glass")
        .textures("bronze_lantern_top", "bronze_lantern", "bronze_lantern_top")
        .light_rgb(15, 12, 8)
        .hardness(1.0)
        .mineable("pickaxe")
        .tab(&crate::tabs::FUNCTIONAL);

    // ---- recipes
    r.shapeless(C, "frosted_glass", 1, &["#glass", "#sand"], 0);
    r.shapeless(C, "smoked_glass", 1, &["#glass", "#gems/coal"], 0);
    // One pane of each colour set in tin cames.
    r.shaped(C, "leaded_glass", 3, &["rg", "tb"], &[('r', "#stained_glass/red"), ('g', "#stained_glass/green"), ('b', "#stained_glass/blue"), ('t', "#ingots/tin")]);
    // Sand polishes the stone.
    r.shaped(C, "polished_limestone", 4, &["ll", "ls", "l "], &[('l', "limestone"), ('s', "#sand")]);
    r.shaped(C, "limestone_tiles", 4, &["pp", "pp"], &[('p', "polished_limestone")]);
    r.shapeless(C, "polished_obsidian", 1, &["obsidian", "#sand"], 0);
    r.shaped(C, "slate_tiles", 4, &["ss", "ss"], &[('s', "slate")]);
    r.shaped(C, "checker_tiles", 4, &["ps", "sp"], &[('p', "polished_limestone"), ('s', "slate_tiles")]);
    r.shaped(C, "#glazed_tiles/{m}", 4, &["bb", "bb", "d "], &[('b', "bricks"), ('d', "#dyes/{m}")]);
    r.shaped(C, "bronze_plate", 4, &["bb", "bb"], &[('b', "#ingots/bronze")]);
    r.shaped(C, "steel_plate", 4, &["ss", "ss"], &[('s', "#ingots/steel")]);
    r.shaped(C, "bronze_lantern", 1, &[" b ", "bfb", " g "], &[('b', "#ingots/bronze"), ('f', "frosted_glass"), ('g', "#glowstone")]);
}

/// A copper block's turn to weather: green with verdigris while air is above it.
fn weather_copper(e: &mut BlockEvent) {
    let w = e.world();
    let p = e.pos();
    if w.block(p.above()) == 0 {
        w.set_block(p, VERDIGRIS_COPPER.id());
    }
}

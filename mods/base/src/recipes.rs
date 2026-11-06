//! Crafting and smelting recipes. Every ingredient is a `#tag`, so another mod's planks, coal or
//! dyes work in all of them without a recipe of its own.

use weft_sdk::*;

pub use base_api::{CRAFTING, SMELTING};
const C: &RecipeKind = &CRAFTING;
const S: &RecipeKind = &SMELTING;

pub fn register(r: &mut Registry) {
    // A crafting table's grid is 3 x 3 in the recipe viewer, whatever a recipe fills of it.
    r.recipe_kind(C).inputs(3, 3).outputs(1, 1);
    r.recipe_kind(S).inputs(1, 1).outputs(1, 1);
    r.shapeless(C, "#planks/{m}", 4, &["#logs/{m}"], 0);
    r.shaped(C, "stick", 4, &["#", "#"], &[('#', "#planks")]);
    r.shaped(C, "#stairs/{m}", 4, &["#  ", "## ", "###"], &[('#', "#stairs_of/{m}")]);
    r.shaped(C, "#slab/{m}", 6, &["###"], &[('#', "#slab_of/{m}")]);
    r.shapeless(C, "#vertical_slab/{m}", 1, &["#slab/{m}"], 0);
    r.shapeless(C, "#slab/{m}", 1, &["#vertical_slab/{m}"], 0);
    r.shaped(C, "crafting_table", 1, &["##", "##"], &[('#', "#planks")]);
    r.shaped(C, "furnace", 1, &["###", "# #", "###"], &[('#', "#cobblestone")]);
    r.shaped(C, "chest", 1, &["###", "# #", "###"], &[('#', "#planks")]);
    r.shaped(C, "torch", 4, &["c", "s"], &[('c', "#gems/coal"), ('s', "#stick")]);
    r.shaped(C, "ladder", 3, &["s s", "sss", "s s"], &[('s', "#stick")]);
    // Any grain bakes into bread (`vitals::crops::BREAD_GRAINS`): rye, barley, sorghum and millet
    // breads as wheat's.
    r.shaped(C, "bread", 1, &["www"], &[('w', "#bread_grain")]);
    // Sugar cane's uses: sugar, and paper for books, which make bookshelves (camp comfort).
    r.shapeless(C, "sugar", 1, &["#sugar_cane"], 0);
    r.shaped(C, "paper", 3, &["###"], &[('#', "#sugar_cane")]);
    r.shapeless(C, "book", 1, &["#paper", "#paper", "#paper", "#leather"], 0);
    r.shaped(C, "bookshelf", 1, &["###", "bbb", "###"], &[('#', "#planks"), ('b', "#book")]);
    // Eggs: fried in the furnace, or baked into a pie with an apple, sugar and wheat.
    r.shapeless(S, "cooked_egg", 1, &["#egg"], 200);
    // Wild rice's grain cooks into a bowl of wild rice (paddy rice, a crop to come, will cook
    // into its own white `cooked_rice`).
    r.shapeless(S, "cooked_wild_rice", 1, &["#rice"], 200);
    r.shapeless(C, "apple_pie", 1, &["#apple", "#sugar", "#egg", "#crops/wheat"], 0);
    // A cacao pod opens into its beans (the pulp eaten, `cacao_pod` the food), and the beans
    // roast in the furnace.
    r.shapeless(C, "cacao_beans", 1, &["cacao_pod"], 0);
    r.shapeless(S, "roasted_cacao", 1, &["cacao_beans"], 200);
    // Wheat packed nine to a bale and back.
    r.shaped(C, "hay_bale", 1, &["www", "www", "www"], &[('w', "#crops/wheat")]);
    r.shapeless(C, "wheat", 9, &["#hay_bale"], 0);
    r.shaped(C, "stone_bricks", 4, &["##", "##"], &[('#', "#smooth_stone")]);
    r.shaped(C, "wool", 1, &["ss", "ss"], &[('s', "#string")]);
    r.shaped(C, "bed", 1, &["www", "ppp"], &[('w', "wool"), ('p', "#planks")]);
    r.shaped(C, "bucket", 1, &["i i", " i "], &[('i', "#ingots/iron")]);
    r.shaped(C, "glowstone", 1, &["ttt", "tgt", "ttt"], &[('t', "#torches"), ('g', "#glass")]);
    r.shapeless(C, "#lamps/{m}", 1, &["#glowstone", "#dyes/{m}"], 0);
    r.shapeless(C, "#stained_glass/{m}", 1, &["#glass", "#dyes/{m}"], 0);

    // Every mod's `ore_blocks/<m>` packs its `ingots/<m>` or `gems/<m>`, and its ores smelt into them.
    for kind in ["#ingots/{m}", "#gems/{m}"] {
        r.shaped(C, "#ore_blocks/{m}", 1, &["###", "###", "###"], &[('#', kind)]);
        r.shapeless(C, kind, 9, &["#ore_blocks/{m}"], 0);
        r.shapeless(S, kind, 1, &["#ores/{m}"], 200);
    }
    r.shapeless(S, "#cooked_meat/{m}", 1, &["#raw_meat/{m}"], 200);
    r.shapeless(S, "#ingots/{m}", 1, &["#raw_ores/{m}"], 200);

    // Tools, weapons and armour have no recipe of their own: they're assembled from parts at the
    // crafting table (`parts.rs`, Ronald 11-05: "a more modular approach").

    for (input, output, ticks) in
        [("#sand", "glass", 200), ("#cobblestone", "stone", 200), ("#logs", "coal", 200), ("#clay", "bricks", 200)]
    {
        r.shapeless(S, output, 1, &[input], ticks);
    }
}

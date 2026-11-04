use weft_sdk::*;

use crate::items::{FLAMMABLE, FUEL};
use crate::worldgen;

pub static STONE: Block = Block::named("stone");
pub static COBBLESTONE: Block = Block::named("cobblestone");
pub static MOSSY_COBBLESTONE: Block = Block::named("mossy_cobblestone");
pub static DIRT: Block = Block::named("dirt");
pub static GRASS: Block = Block::named("grass");
pub static SNOWY_GRASS: Block = Block::named("snowy_grass");
pub static SAND: Block = Block::named("sand");
pub static GRAVEL: Block = Block::named("gravel");
pub static CLAY: Block = Block::named("clay");
pub static SANDSTONE: Block = Block::named("sandstone");
// The rock faces' stones (`land.rs`): granite on warm mountains, slate in the cold, limestone on
// temperate cliffs, ochre and red sandstone in the mesas' strata.
pub static GRANITE: Block = Block::named("granite");
pub static SLATE: Block = Block::named("slate");
pub static LIMESTONE: Block = Block::named("limestone");
pub static OCHRE_SANDSTONE: Block = Block::named("ochre_sandstone");
pub static RED_SANDSTONE: Block = Block::named("red_sandstone");
pub static CREAM_SANDSTONE: Block = Block::named("cream_sandstone");
pub static BANDED_CLAY: Block = Block::named("banded_clay");
pub static WEATHERED_LIMESTONE: Block = Block::named("weathered_limestone");
pub static SNOW: Block = Block::named("snow");
pub static ICE: Block = Block::named("ice");
pub static OBSIDIAN: Block = Block::named("obsidian");
pub static COAL_ORE: Block = Block::named("coal_ore");
pub static IRON_ORE: Block = Block::named("iron_ore");
pub static GOLD_ORE: Block = Block::named("gold_ore");
// Ores in the rock faces (`base_api::land::face_ore`; Art's textures, Ronald: "Yes, all four").
pub static BANDED_CLAY_COAL_ORE: Block = Block::named("banded_clay_coal_ore");
pub static RED_SANDSTONE_IRON_ORE: Block = Block::named("red_sandstone_iron_ore");
pub static GRANITE_GOLD_ORE: Block = Block::named("granite_gold_ore");
pub static GRANITE_IRON_ORE: Block = Block::named("granite_iron_ore");
pub static DIAMOND_ORE: Block = Block::named("diamond_ore");
pub static OAK_LOG: Block = Block::named("oak_log");
pub static BIRCH_LOG: Block = Block::named("birch_log");
/// The savanna's flat-topped tree.
pub static ACACIA_LOG: Block = Block::named("acacia_log");
pub static ACACIA_PLANKS: Block = Block::named("acacia_planks");
pub static ACACIA_LEAVES: Block = Block::named("acacia_leaves");
pub static SPRUCE_LOG: Block = Block::named("spruce_log");
pub static OAK_PLANKS: Block = Block::named("oak_planks");
pub static BIRCH_PLANKS: Block = Block::named("birch_planks");
pub static SPRUCE_PLANKS: Block = Block::named("spruce_planks");
pub static OAK_LEAVES: Block = Block::named("oak_leaves");
pub static BIRCH_LEAVES: Block = Block::named("birch_leaves");
pub static SPRUCE_LEAVES: Block = Block::named("spruce_leaves");
pub static WATER: Block = Block::named("water");
pub static LAVA: Block = Block::named("lava");
pub static TALL_GRASS: Block = Block::named("tall_grass");
pub static GRAZED_GRASS: Block = Block::named("grazed_grass");
pub static DANDELION: Block = Block::named("dandelion");
pub static ROSE: Block = Block::named("rose");
pub static SAPLING: Block = Block::named("sapling");
pub static SUGAR_CANE: Block = Block::named("sugar_cane");
pub static CACTUS: Block = Block::named("cactus");
pub static WHEAT: Block = Block::named("wheat");
pub static FARMLAND: Block = Block::named("farmland");
pub static COMPOST_BIN: Block = Block::named("compost_bin");
pub static GLASS: Block = Block::named("glass");
pub static CRAFTING_TABLE: Block = Block::named("crafting_table");
pub static FURNACE: Block = Block::named("furnace");
pub static CHEST: Block = Block::named("chest");
pub static TORCH: Block = Block::named("torch");
pub static LADDER: Block = Block::named("ladder");
pub static GLOWSTONE: Block = Block::named("glowstone");
pub static BRICKS: Block = Block::named("bricks");
pub static STONE_BRICKS: Block = Block::named("stone_bricks");
pub static IRON_BLOCK: Block = Block::named("iron_block");
pub static GOLD_BLOCK: Block = Block::named("gold_block");
pub static DIAMOND_BLOCK: Block = Block::named("diamond_block");
pub static COAL_BLOCK: Block = Block::named("coal_block");
pub static WOOL: Block = Block::named("wool");
/// Camp comfort (books, `survival::camp`), built from planks and books.
pub static BOOKSHELF: Block = Block::named("bookshelf");
/// Nine wheat packed (and back), soft to rest by.
pub static HAY_BALE: Block = Block::named("hay_bale");
/// Spruce forests' ground (`worldgen`), soil like dirt.
pub static PODZOL: Block = Block::named("podzol");
/// The desert's scrub: a few sticks where there is no wood.
pub static DEAD_BUSH: Block = Block::named("dead_bush");
/// Game seconds between a dogwood leaf's looks at the season, on average.
const DOGWOOD_BLOOM_SECS: f32 = 300.0;
/// The sequoia grove's understory (`grove`).
pub static SWORD_FERN: Block = Block::named("sword_fern");
pub static LUPINE: Block = Block::named("lupine");
pub static DOGWOOD_LOG: Block = Block::named("dogwood_log");
pub static DOGWOOD_LEAVES: Block = Block::named("dogwood_leaves");
pub static MOSSY_SEQUOIA_LOG: Block = Block::named("mossy_sequoia_log");
/// Bracken and the woods' ferns, under the trees (`vitals::layers`).
pub static FERN: Block = Block::named("fern");
/// The forest floor's blocks (`vitals::layers`): fallen leaves, a moss carpet, stony bare dirt.
pub static LEAF_LITTER: Block = Block::named("leaf_litter");
pub static MOSS_BLOCK: Block = Block::named("moss_block");
pub static COARSE_DIRT: Block = Block::named("coarse_dirt");
/// The shrubs' own leaves, in their own colours: willow, juniper, creosote bush.
pub static WILLOW_LEAVES: Block = Block::named("willow_leaves");
pub static JUNIPER_LEAVES: Block = Block::named("juniper_leaves");
pub static CREOSOTE_LEAVES: Block = Block::named("creosote_leaves");
/// Grass two blocks tall (the prairie's big bluestem, the llanos' grasses): `half` lower, upper.
pub static TALL_GRASS_DOUBLE: Block = Block::named("tall_grass_double");
/// Mushrooms at the foot of trunks (`vitals::layers`): honey fungus, chanterelles, ceps.
pub static HONEY_FUNGUS: Block = Block::named("honey_fungus");
pub static CHANTERELLE: Block = Block::named("chanterelle");
pub static PORCINI: Block = Block::named("porcini");
/// A shelf fungus on bark, on the north side of a dead trunk (`base_api::trees`).
pub static BRACKET_FUNGUS: Block = Block::named("bracket_fungus");

/// Log axis along the clicked face, like vanilla.
fn axis_placement(p: &Place, props: &mut Props) {
    props.set(
        "axis",
        match p.face() {
            0 | 1 => "x",
            4 | 5 => "z",
            _ => "y",
        },
    );
}

/// A log that grows into a tree of `leaves` (a cone, like a spruce, when `cone`).
/// `shape`: the tree's tag (`cone_tree`, `flat_tree`; none: round), `worldgen::tree_blocks`.
fn log(r: &mut Registry, handle: &'static Block, wood: &str, leaves: &str, shape: Option<&str>) {
    let side = format!("log_{wood}");
    let top = format!("log_{wood}_top");
    let b = r
        .block(handle)
        .set_text(&crate::worldgen::TREE_LEAVES, leaves);
    let b = match shape {
        Some(tag) => b.tag(tag),
        None => b,
    };
    b.sound_group("wood")
        .hardness(2.0)
        .mineable("axe")
        .tag("logs")
        .tag(&format!("logs/{wood}"))
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        // `bark`: bark on every face, for a trunk whose top shows (a giant's flared foot).
        .prop("axis", ["y", "x", "z", "bark"])
        .states(move |p, s| match p.get("axis") {
            "y" => s.textures(&top, &side, &top),
            "x" => s.faces([&top, &top, &side, &side, &side, &side]),
            "bark" => s.texture(&side),
            _ => s.faces([&side, &side, &side, &side, &top, &top]),
        })
        .placement(axis_placement)
        .tab(&crate::tabs::NATURAL);
}

/// Our trees (`notes/ECOLOGY.md`, part 3): mesquite in the desert's washes and the llanos, flat
/// topped like an acacia, its pods a summer food; the Pantanal's acuri palm and the llanos'
/// moriche palm, their fruit falling through their seasons. Wood, its tree's shape, and what its
/// leaves drop besides its sapling. Their saplings grow by the place (`vitals::plants`). The giant
/// sequoia of the Sierra Nevada's groves, cone-shaped young and grown from branches
/// (`tree_recipes::SEQUOIA`), its cones planting it.
const WOODS: [(&str, Option<&str>, &str); 4] = [
    ("mesquite", Some("flat_tree"), "mesquite_pod"),
    ("acuri_palm", Some("palm_tree"), "acuri_nut"),
    ("moriche_palm", Some("palm_tree"), "moriche_fruit"),
    ("sequoia", Some("cone_tree"), "sequoia_cone"),
];

/// Our trees' logs, planks, leaves and saplings (`WOODS`), and the fruit their leaves drop, which
/// plants a sapling too.
fn woods(r: &mut Registry) {
    for (wood, shape, fruit) in WOODS {
        let log_block = Block::generated(format!("{wood}_log"));
        log(r, log_block, wood, &format!("{wood}_leaves"), shape);
        planks(r, Block::generated(format!("{wood}_planks")), wood);
        let sapling = format!("{wood}_sapling");
        leaves(r, Block::generated(format!("{wood}_leaves")), &format!("leaves_{wood}"), &sapling, Some(fruit));
        let plant: &'static str = Box::leak(format!("base:{sapling}").into_boxed_str());
        let sapling_block = Block::generated(sapling);
        r.block(sapling_block)
            .sound_group("grass")
            .texture(&format!("sapling_{wood}"))
            .plant()
            .needs_below("#sapling_soil")
            .growth(crate::growth::step_secs(plant, 1, 470.0), 9)
            .on_grow(move |e| crate::growth::grow(e, plant, false, |e| worldgen::grow_tree(e.world(), e.pos(), log_block.id())))
            .tab(&crate::tabs::NATURAL);
        r.item(Item::generated(fruit.into()))
            .placed_block(sapling_block)
            .tab(&crate::tabs::NATURAL);
    }
    // Desert and beach trees take root in sand as well.
    r.tag("sapling_soil", &["#dirt", "#sand"]);
}

/// The sequoia grove's understory (Giant Forest): sword fern and lupine on its floor, the Pacific
/// dogwood (a small tree, its white bracts in spring, drawn in its own colours and so not tinted)
/// under the giants, and a fallen giant's mossy log. Their places are the grove's (`worldgen.rs`).
fn grove(r: &mut Registry) {
    r.block(&SWORD_FERN)
        .sound_group("grass")
        .texture("sword_fern")
        .plant()
        .needs_below("#plant_soil")
        .tint(&tints::GRASS)
        .tab(&crate::tabs::NATURAL);
    r.loot(&SWORD_FERN).pool(
        Pool::new()
            .when(Cond::Chance(vitals::clothing::FIBRE_CHANCE))
            .item("plant_fibre"),
    );
    plant(r, &LUPINE, "lupine");
    log(r, &DOGWOOD_LOG, "dogwood", "base:dogwood_leaves", None);
    // The dogwood is white with its bracts in spring and plain green the rest of the year: a
    // growth step every few minutes looks at the season, as the wild flowers do (`wild.rs`).
    r.block(&DOGWOOD_LEAVES)
        .sound_group("grass")
        .prop("bloom", 0..=1)
        .states(|p, s| s.texture(if p.get("bloom") == "1" { "leaves_dogwood" } else { "leaves_dogwood_green" }))
        .growth(DOGWOOD_BLOOM_SECS, 0)
        .on_grow(|e| {
            let spring = base_api::ground_at(&e.world(), e.pos()).season == Some(0);
            let want = if spring { "1" } else { "0" };
            if state_prop(e.state(), "bloom") != Some(want) {
                e.world().set_prop(e.pos(), "bloom", want);
            }
        })
        .hardness(0.2)
        .cutout()
        .cull_same()
        .opacity(1)
        .tag("leaves")
        .redstone_blocker()
        .mineable("shears")
        .tab(&crate::tabs::NATURAL);
    r.loot(&DOGWOOD_LEAVES)
        .pool(Pool::new().when(Cond::Chance(0.05)).item("stick"));
    // A fallen giant's bark under moss, its end mossy too.
    r.block(&MOSSY_SEQUOIA_LOG)
        .sound_group("wood")
        .hardness(2.0)
        .mineable("axe")
        .tag("logs")
        .tag("logs/sequoia")
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .prop("axis", ["y", "x", "z"])
        .states(|p, s| match p.get("axis") {
            "y" => s.textures("log_sequoia_mossy_top", "log_sequoia_mossy", "log_sequoia_mossy_top"),
            "x" => s.faces([
                "log_sequoia_mossy_top",
                "log_sequoia_mossy_top",
                "log_sequoia_mossy",
                "log_sequoia_mossy",
                "log_sequoia_mossy",
                "log_sequoia_mossy",
            ]),
            _ => s.faces([
                "log_sequoia_mossy",
                "log_sequoia_mossy",
                "log_sequoia_mossy",
                "log_sequoia_mossy",
                "log_sequoia_mossy_top",
                "log_sequoia_mossy_top",
            ]),
        })
        .placement(axis_placement)
        .tab(&crate::tabs::NATURAL);
}

fn planks(r: &mut Registry, h: &'static Block, wood: &str) {
    r.block(h)
        .sound_group("wood")
        .texture(&format!("planks_{wood}"))
        .hardness(2.0)
        .mineable("axe")
        .tag("planks")
        .tag(&format!("planks/{wood}"))
        .tag("materials/wood")
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .tab(&crate::tabs::BUILDING);
}

/// Leaves that drop `sapling` (a time in 20) and, for our trees, `fruit` (a time in 20).
fn leaves(r: &mut Registry, handle: &'static Block, tex: &str, sapling: &str, fruit: Option<&str>) {
    r.block(handle)
        .sound_group("grass")
        .texture(tex)
        .hardness(0.2)
        .cutout()
        // Faces between two leaves of one tree kind aren't drawn (each was drawn twice): a crown
        // keeps its outer shell and its holes (Researcher's performance report, item 9).
        .cull_same()
        .opacity(1)
        // Its tint by how it turns in autumn (`vitals::wild::LEAVES`).
        .tint(base_api::trees::leaf_tint(vitals::wild::turn(handle.name())))
        .tag("leaves")
        .redstone_blocker()
        .mineable("shears")
        .tab(&crate::tabs::NATURAL);
    // Vanilla: a sapling one time in 20, oak leaves an apple one time in 200.
    let table = r
        .loot(handle)
        .pool(Pool::new().when(Cond::Chance(0.05)).item(sapling));
    if let Some(fruit) = fruit {
        table.pool(Pool::new().when(Cond::Chance(0.05)).item(fruit));
        return;
    }
    // Ours: acorns from oak leaves, pine nuts from spruce cones (`vitals::forage::DROPPED`,
    // `food::FOODS`).
    if std::ptr::eq(handle, &OAK_LEAVES) {
        table
            .pool(Pool::new().when(Cond::Chance(0.005)).item("apple"))
            .pool(Pool::new().when(Cond::Chance(0.05)).item("acorn"));
    } else if std::ptr::eq(handle, &SPRUCE_LEAVES) {
        table.pool(Pool::new().when(Cond::Chance(0.03)).item("pine_nut"));
    }
}

fn plant(r: &mut Registry, handle: &'static Block, tex: &str) {
    r.block(handle)
        .sound_group("grass")
        .texture(tex)
        .plant()
        .needs_below("#plant_soil")
        .tab(&crate::tabs::NATURAL);
}

/// Sand and gravel: with nothing under them they fall as an entity and land as a block.
pub fn register(r: &mut Registry) {
    // The leaves' tint kinds besides foliage (`base_api::trees`), as green as foliage by default.
    r.tint_kind(&base_api::trees::EVERGREEN, 0x48b518);
    r.tint_kind(&base_api::trees::YELLOW_LEAVES, 0x48b518);
    r.block(&STONE)
        .sound_group("stone")
        .hardness(1.5)
        .mineable("pickaxe")
        .needs_tool(0)
        .drops("cobblestone", 1)
        .tag("stone")
        .tag("smooth_stone")
        .tag("stone_ore_replaceables")
        .tab(&crate::tabs::BUILDING);
    r.block(&COBBLESTONE)
        .sound_group("stone")
        .hardness(2.0)
        .mineable("pickaxe")
        .needs_tool(0)
        .tag("stone")
        .tag("cobblestone")
        .tag("materials/stone")
        .tab(&crate::tabs::BUILDING);
    r.block(&MOSSY_COBBLESTONE)
        .sound_group("stone")
        .hardness(2.0)
        .mineable("pickaxe")
        .needs_tool(0)
        .tab(&crate::tabs::BUILDING);
    r.block(&DIRT)
        .sound_group("gravel")
        .hardness(0.5)
        .mineable("shovel")
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    r.block(&GRASS)
        .sound_group("grass")
        .textures("grass_top", "grass_side", "dirt")
        .tint_top(&tints::GRASS)
        .hardness(0.6)
        .mineable("shovel")
        .drops("dirt", 1)
        .tag("dirt")
        // Vanilla's pace: a minute or so under a block, many minutes to cover bare dirt.
        .spreads("#grass_spreads_to", 765.0, 9)
        .smothered("dirt", 68.0)
        .tab(&crate::tabs::NATURAL);
    r.block(&SNOWY_GRASS)
        .sound_group("grass")
        .textures("snow", "grass_snow_side", "dirt")
        .hardness(0.6)
        .mineable("shovel")
        .drops("dirt", 1)
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    r.block(&SAND)
        .sound_group("sand")
        .hardness(0.5)
        .mineable("shovel")
        .tag("sand")
        .falling()
        .tab(&crate::tabs::NATURAL);
    r.block(&GRAVEL)
        .sound_group("gravel")
        .hardness(0.6)
        .mineable("shovel")
        .falling()
        .tab(&crate::tabs::NATURAL);
    r.block(&CLAY)
        .sound_group("gravel")
        .hardness(0.6)
        .mineable("shovel")
        .tag("clay")
        .tab(&crate::tabs::NATURAL);
    r.block(&SANDSTONE)
        .sound_group("stone")
        .textures("sandstone_top", "sandstone_side", "sandstone_bottom")
        .hardness(0.8)
        .mineable("pickaxe")
        .needs_tool(0)
        .tab(&crate::tabs::BUILDING);
    for (block, textures) in [
        (&GRANITE, None),
        (&SLATE, Some(("slate_top", "slate_side", "slate_top"))),
        (&LIMESTONE, Some(("limestone_top", "limestone_side", "limestone_top"))),
        (&OCHRE_SANDSTONE, Some(("ochre_sandstone_top", "ochre_sandstone_side", "ochre_sandstone_bottom"))),
        (&RED_SANDSTONE, Some(("red_sandstone_top", "red_sandstone_side", "red_sandstone_bottom"))),
        // The canyons' strata and the karst towers' faces (Art, 11-04; Ronald: "Yes, loosen streaks").
        (&CREAM_SANDSTONE, Some(("cream_sandstone_top", "cream_sandstone_side", "cream_sandstone_top"))),
        (&BANDED_CLAY, Some(("banded_clay_top", "banded_clay_side", "banded_clay_top"))),
        (&WEATHERED_LIMESTONE, Some(("limestone_top", "weathered_limestone_side", "limestone_top"))),
    ] {
        let b = r
            .block(block)
            .sound_group("stone")
            .hardness(1.2)
            .mineable("pickaxe")
            .needs_tool(0);
        match textures {
            Some((top, side, bottom)) => b.textures(top, side, bottom),
            None => b,
        }
        .tab(&crate::tabs::BUILDING);
    }
    r.block(&SNOW)
        .sound_group("sand")
        .hardness(0.2)
        .mineable("shovel")
        .tab(&crate::tabs::NATURAL);
    // Blue lamps and glass are made with it.
    r.block(&ICE)
        .redstone_blocker()
        .sound_group("glass")
        .tag("ice")
        .tag("dyes/blue")
        .hardness(0.5)
        .translucent()
        .opacity(2)
        .friction(0.98)
        .mineable("pickaxe")
        // Itself with a pickaxe (Research round 3, U1: it's the only blue dye).
        .needs_tool(0)
        .tab(&crate::tabs::NATURAL);
    r.block(&OBSIDIAN)
        .immovable()
        .sound_group("stone")
        .hardness(50.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::DIAMOND)
        .tab(&crate::tabs::NATURAL);
    r.block(&COAL_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool(0)
        .drops("coal", 1)
        .tag("ores")
        .tag("ores/coal")
        .tab(&crate::tabs::NATURAL);
    r.block(&IRON_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::STONE)
        .drops("raw_iron", 1)
        .tag("ores")
        .tag("ores/iron")
        .tab(&crate::tabs::NATURAL);
    r.block(&GOLD_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::IRON)
        .drops("raw_gold", 1)
        .tag("ores")
        .tag("ores/gold")
        .tab(&crate::tabs::NATURAL);
    // The faces' ores: like the ores underground (a pickaxe, the same tools and drops), each in
    // its host rock's own look.
    for (block, texture, drop, tier, kind) in [
        (&BANDED_CLAY_COAL_ORE, "banded_clay_coal_ore", "coal", 0, "coal"),
        (&RED_SANDSTONE_IRON_ORE, "red_sandstone_iron_ore", "raw_iron", tool_tier::STONE, "iron"),
        (&GRANITE_IRON_ORE, "granite_iron_ore", "raw_iron", tool_tier::STONE, "iron"),
        (&GRANITE_GOLD_ORE, "granite_gold_ore", "raw_gold", tool_tier::IRON, "gold"),
    ] {
        r.block(block)
            .sound_group("stone")
            .texture(texture)
            .hardness(3.0)
            .mineable("pickaxe")
            .needs_tool(tier)
            .drops(drop, 1)
            .tag("ores")
            .tag(&format!("ores/{kind}"))
            .tab(&crate::tabs::NATURAL);
    }
    r.block(&DIAMOND_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::IRON)
        .drops("diamond", 1)
        .tag("ores")
        .tag("ores/diamond")
        .tab(&crate::tabs::NATURAL);

    log(r, &OAK_LOG, "oak", "oak_leaves", None);
    log(r, &BIRCH_LOG, "birch", "birch_leaves", Some("slim_tree"));
    log(r, &SPRUCE_LOG, "spruce", "spruce_leaves", Some("cone_tree"));
    log(r, &ACACIA_LOG, "acacia", "acacia_leaves", Some("flat_tree"));
    for (h, wood) in [(&OAK_PLANKS, "oak"), (&BIRCH_PLANKS, "birch"), (&SPRUCE_PLANKS, "spruce"), (&ACACIA_PLANKS, "acacia")] {
        planks(r, h, wood);
    }
    leaves(r, &OAK_LEAVES, "leaves_oak", "sapling", None);
    leaves(r, &BIRCH_LEAVES, "leaves_birch", "sapling", None);
    leaves(r, &SPRUCE_LEAVES, "leaves_spruce", "sapling", None);
    leaves(r, &ACACIA_LEAVES, "leaves_acacia", "sapling", None);
    woods(r);
    grove(r);

    // Vanilla numbers: water steps every 5 ticks and reaches 7 blocks, lava every 30 and 3.
    r.liquid(&WATER)
        .tint(&tints::WATER)
        .translucent()
        .renewable(2)
        .fog(0x050533, 96.0)
        .tag("water")
        .bucket(Item::generated("water_bucket".into()), &crate::items::BUCKET, &crate::tabs::TOOLS);
    r.liquid(&LAVA)
        .light(15)
        .drag(0.4)
        .flow_delay(30)
        .drop_off(2)
        .slope_find(2)
        .temperature(1300)
        .not_swimmable()
        .fog(0x991A00, 1.0)
        .no_overlay()
        .tag("lava")
        .tag("hot")
        .bucket(Item::generated("lava_bucket".into()), &crate::items::BUCKET, &crate::tabs::TOOLS);
    r.liquid_reaction("#hot", "#water")
        .source_block("obsidian")
        .flowing_block("cobblestone")
        .turns_below_into("stone");

    r.block(&TALL_GRASS)
        .sound_group("grass")
        .texture("tallgrass")
        .plant()
        .needs_below("#plant_soil")
        .tint(&tints::GRASS)
        .no_item();
    r.loot(&TALL_GRASS)
        .pool(Pool::new().when(Cond::Chance(0.125)).item("wheat_seeds"))
        // Fibre for string and linen (`clothing.rs`).
        .pool(
            Pool::new()
                .when(Cond::Chance(vitals::clothing::FIBRE_CHANCE))
                .item("plant_fibre"),
        );
    // Tall grass an animal has grazed, growing back by warmth and rain (`growth.rs`).
    r.block(&GRAZED_GRASS)
        .sound_group("grass")
        .texture("grazed_grass")
        .plant()
        .needs_below("#plant_soil")
        .tint(&tints::GRASS)
        .no_item()
        .growth(crate::growth::step_secs("base:grazed_grass", 1, 600.0), 9)
        .on_grow(|e| {
            crate::growth::grow(e, "base:grazed_grass", false, |e| {
                e.world().set(e.pos(), TALL_GRASS.id());
            })
        });
    plant(r, &DANDELION, "flower_dandelion");
    plant(r, &ROSE, "flower_rose");
    // A wild rose gives itself and, a time in four, its hips.
    r.loot(&ROSE)
        .pool(Pool::new().entry(Entry::item("rose")))
        .pool(Pool::new().when(Cond::Chance(0.25)).item("rose_hip"));
    r.block(&SAPLING)
        .sound_group("grass")
        .texture("sapling_oak")
        .plant()
        .needs_below("#plant_soil")
        .growth(crate::growth::step_secs("base:sapling", 1, 470.0), 9)
        .on_grow(|e| crate::growth::grow(e, "base:sapling", false, |e| worldgen::grow_tree(e.world(), e.pos(), OAK_LOG.id())))
        .tab(&crate::tabs::NATURAL);
    r.block(&SUGAR_CANE)
        .sound_group("grass")
        .texture("reeds")
        .plant()
        .needs_below("#sugar_cane_soil")
        .needs_near_support("#water")
        .growth(crate::growth::step_secs("base:sugar_cane", 1, 270.0), 0)
        .on_grow(|e| crate::growth::grow(e, "base:sugar_cane", false, |e| grow_column(e, &SUGAR_CANE)))
        .tab(&crate::tabs::NATURAL);
    r.block(&CACTUS)
        .sound_group("cloth")
        .textures("cactus_top", "cactus_side", "cactus_top")
        .model_box([1.0, 0.0, 1.0], [15.0, 16.0, 15.0])
        .hardness(0.4)
        .needs_below("#cactus_soil")
        .growth(crate::growth::step_secs("base:cactus", 1, 270.0), 0)
        .on_grow(|e| crate::growth::grow(e, "base:cactus", false, |e| grow_column(e, &CACTUS)))
        .tab(&crate::tabs::NATURAL);
    r.block(&WHEAT)
        .sound_group("grass")
        .prop("age", 0..=7)
        .states(|p, s| s.texture(&format!("wheat_stage_{}", p.get("age"))))
        .plant()
        .needs_below("#crop_soil")
        .no_item()
        // By warmth and rain (`growth.rs`): seven stages over its real 110 days' pace.
        .growth(crate::growth::step_secs("base:wheat", 7, 205.0), 9)
        .on_grow(|e| crate::growth::grow(e, "base:wheat", false, |e| crate::growth::next_age(e, 7)));
    // Modern's: unripe gives its seed back, ripe gives the wheat and up to 3 more seeds.
    r.loot(&WHEAT)
        .pool(Pool::new().when(Cond::BlockState("age=7")).item("wheat"))
        .pool(Pool::new().entry(Entry::item("wheat_seeds")))
        .pool(
            Pool::new()
                .when(Cond::BlockState("age=7"))
                .entry(Entry::item("wheat_seeds").count(0, 3)),
        );
    r.block(&FARMLAND)
        .sound_group("gravel")
        // How worn its fertility is (`farming.rs`): 0 fresh, 3 worn out.
        .prop("worn", 0..=3)
        // Paler and cracked as it wears.
        .states(|p, s| {
            let top = match p.get("worn") {
                "0" => "farmland".to_string(),
                n => format!("farmland_worn_{n}"),
            };
            s.textures(&top, "dirt", "dirt")
        })
        .growth(vitals::soil::FALLOW_SECS, 0)
        .on_grow(crate::farming::fallow)
        .model_box([0.0, 0.0, 0.0], [16.0, 15.0, 16.0])
        .flags(0, 0)
        .hardness(0.6)
        .mineable("shovel")
        .drops("dirt", 1)
        .tab(&crate::tabs::NATURAL);

    r.block(&GLASS)
        .redstone_blocker()
        .sound_group("glass")
        .tag("glass")
        .cutout()
        .cull_same()
        .hardness(0.3)
        .drops_nothing()
        .tab(&crate::tabs::BUILDING);
    for (name, col) in [("red", [0u8, 12, 12]), ("green", [12, 0, 12]), ("blue", [12, 12, 0])] {
        r.block(Block::generated(format!("{name}_stained_glass")))
            .redstone_blocker()
            .sound_group("glass")
            .tag(&format!("stained_glass/{name}"))
            .texture(&format!("glass_{name}"))
            .translucent()
            .filter(col[0], col[1], col[2])
            .hardness(0.3)
            .tab(&crate::tabs::COLORED);
    }
    for (name, rgb) in [("red", [15u8, 3, 2]), ("green", [2, 15, 4]), ("blue", [3, 6, 15])] {
        r.block(Block::generated(format!("{name}_lamp")))
            .sound_group("glass")
            .tag(&format!("lamps/{name}"))
            .texture(&format!("lamp_{name}"))
            .light_rgb(rgb[0], rgb[1], rgb[2])
            .hardness(0.3)
            .tab(&crate::tabs::COLORED);
    }
    r.block(&CRAFTING_TABLE)
        .sound_group("wood")
        .faces([
            "crafting_table_side",
            "crafting_table_side",
            "planks_oak",
            "crafting_table_top",
            "crafting_table_front",
            "crafting_table_front",
        ])
        .hardness(2.5)
        .mineable("axe")
        .set(&FUEL, 300)
        .opens(&crate::player::CRAFTING)
        .tab(&crate::tabs::FUNCTIONAL);
    r.block(&TORCH)
        .sound_group("wood")
        .ambient_particles(&["core:flame", "core:smoke"])
        .facing(TORCH_FACINGS, Toward::Face)
        .states(|p, s| torch_state(p.facing(), s, "torch"))
        .item_texture("torch")
        .not_solid()
        .light_rgb(15, 13, 10)
        .hardness(0.0)
        .tab(&crate::tabs::FUNCTIONAL);
    // Facing away from the solid wall it hangs on.
    r.block(&LADDER)
        .sound_group("wood")
        .facing(Facings::HORIZONTAL, Toward::Face)
        .states(|p, s| {
            let d = p.facing();
            let (min, max) = match d {
                Dir::South => ([0.0, 0.0, 0.0], [16.0, 16.0, 1.0]),
                Dir::North => ([0.0, 0.0, 15.0], [16.0, 16.0, 16.0]),
                Dir::East => ([0.0, 0.0, 0.0], [1.0, 16.0, 16.0]),
                _ => ([15.0, 0.0, 0.0], [16.0, 16.0, 16.0]),
            };
            s.model_box(min, max).needs(d.opposite().face(), "")
        })
        .climbable()
        .not_solid()
        .hardness(0.4)
        .tab(&crate::tabs::FUNCTIONAL);
    r.block(&GLOWSTONE)
        .sound_group("glass")
        .tag("glowstone")
        .light(15)
        .hardness(0.3)
        .tab(&crate::tabs::NATURAL);
    r.block(&BRICKS)
        .sound_group("stone")
        .hardness(2.0)
        .mineable("pickaxe")
        .needs_tool(0)
        .tab(&crate::tabs::BUILDING);
    r.block(&STONE_BRICKS)
        .sound_group("stone")
        .texture("stonebrick")
        .hardness(1.5)
        .mineable("pickaxe")
        .needs_tool(0)
        .tab(&crate::tabs::BUILDING);
    r.block(&IRON_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::STONE)
        .tag("ore_blocks/iron")
        .tab(&crate::tabs::BUILDING);
    r.block(&GOLD_BLOCK)
        .sound_group("stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::IRON)
        .tag("ore_blocks/gold")
        .tab(&crate::tabs::BUILDING);
    r.block(&DIAMOND_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mineable("pickaxe")
        .needs_tool(tool_tier::IRON)
        .tag("ore_blocks/diamond")
        .tab(&crate::tabs::BUILDING);
    r.block(&COAL_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mineable("pickaxe")
        .needs_tool(0)
        .tag("ore_blocks/coal")
        .set(&FUEL, 16000)
        .tab(&crate::tabs::BUILDING);
    r.block(&WOOL)
        .sound_group("cloth")
        .texture("wool_white")
        .hardness(0.8)
        .set(&FLAMMABLE, 30)
        .tab(&crate::tabs::COLORED);
    r.block(&PODZOL)
        .sound_group("gravel")
        .textures("podzol", "dirt", "dirt")
        .hardness(0.5)
        .mineable("shovel")
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    // The forest floor (`vitals::layers`): soil like dirt, so plants grow on it.
    for (b, top, side, bottom, sound) in [
        (&LEAF_LITTER, "leaf_litter_top", "leaf_litter_side", "dirt", "grass"),
        (&MOSS_BLOCK, "moss_block_top", "moss_block_side", "dirt", "grass"),
        (&COARSE_DIRT, "coarse_dirt", "coarse_dirt", "coarse_dirt", "gravel"),
    ] {
        r.block(b)
            .sound_group(sound)
            .textures(top, side, bottom)
            .hardness(0.5)
            .mineable("shovel")
            .tag("dirt")
            .tab(&crate::tabs::NATURAL);
    }
    // The shrubs' leaves (`vitals::layers`), drawn in their own colours: not tinted.
    for (b, tex) in [(&WILLOW_LEAVES, "leaves_willow"), (&JUNIPER_LEAVES, "leaves_juniper"), (&CREOSOTE_LEAVES, "leaves_creosote")] {
        r.block(b)
            .sound_group("grass")
            .texture(tex)
            .hardness(0.2)
            .cutout()
            .cull_same()
            .opacity(1)
            .tag("leaves")
            .redstone_blocker()
            .mineable("shears")
            .tab(&crate::tabs::NATURAL);
        r.loot(b)
            .pool(Pool::new().when(Cond::Chance(0.05)).item("stick"));
    }
    // Grass two blocks tall: the lower half on soil, the upper on the lower; breaking the lower
    // takes the upper with it (it loses its support).
    r.tag("tall_grass_soil", &["#plant_soil", "tall_grass_double"]);
    r.block(&TALL_GRASS_DOUBLE)
        .sound_group("grass")
        .plant()
        .needs_below("#tall_grass_soil")
        .tint(&tints::GRASS)
        .prop("half", ["lower", "upper"])
        .states(|p, s| s.texture(if p.get("half") == "upper" { "tall_grass_upper" } else { "tall_grass_lower" }))
        .no_item();
    r.loot(&TALL_GRASS_DOUBLE)
        .pool(Pool::new().when(Cond::Chance(0.125)).item("wheat_seeds"))
        .pool(
            Pool::new()
                .when(Cond::Chance(vitals::clothing::FIBRE_CHANCE))
                .item("plant_fibre"),
        );
    // Mushrooms at the foot of trunks (`vitals::layers`); food rows for the ceps and chanterelles
    // are to come.
    for (b, tex) in [(&HONEY_FUNGUS, "honey_fungus"), (&CHANTERELLE, "chanterelle"), (&PORCINI, "porcini")] {
        plant(r, b, tex);
    }
    // A shelf on the north side of a trunk: the box against the cell's south face, where the bark
    // is; its top and bottom show the half disc, its front and ends the ridged bands.
    let (top, side, full) = ("bracket_fungus_top", "bracket_fungus_side", [0.0, 0.0, 16.0, 16.0]);
    r.block(&BRACKET_FUNGUS)
        .sound_group("wood")
        .hardness(0.3)
        .texture(side)
        .model_box_faces(
            [2.0, 6.0, 8.0],
            [14.0, 9.0, 16.0],
            [(side, full), (side, full), (top, full), (top, full), (side, full), (side, full)],
        )
        .tab(&crate::tabs::NATURAL);
    // Ferns on the forest floor (`vitals::layers`): bracken.
    r.block(&FERN)
        .sound_group("grass")
        .texture("bracken")
        .plant()
        .needs_below("#plant_soil")
        .tab(&crate::tabs::NATURAL);
    r.loot(&FERN).pool(
        Pool::new()
            .when(Cond::Chance(vitals::clothing::FIBRE_CHANCE))
            .item("plant_fibre"),
    );
    r.block(&DEAD_BUSH)
        .sound_group("grass")
        .texture("deadbush")
        .plant()
        .needs_below("#sand")
        .tab(&crate::tabs::NATURAL);
    r.loot(&DEAD_BUSH)
        .pool(Pool::new().when(Cond::Chance(0.7)).item("stick"))
        .pool(Pool::new().when(Cond::Chance(0.35)).item("stick"));
    // Spoiled food, dung and plant waste rot into compost (`farming.rs`): its level shows as it
    // fills, the top dark and rich when it's ready.
    r.block(&COMPOST_BIN)
        .sound_group("wood")
        .prop("level", 0..=8)
        .states(|p, s| {
            // Empty, filling, then full while it rots and when it's compost.
            let top = match p.get("level") {
                "0" => "compost_bin_top_empty",
                "7" | "8" => "compost_bin_top_full",
                _ => "compost_bin_top_filling",
            };
            s.textures(top, "compost_bin_side", "compost_bin_side")
        })
        .hardness(0.6)
        .mineable("axe")
        .growth(vitals::soil::compost_secs(), 0)
        .on_grow(crate::farming::rot)
        .on_use(crate::farming::bin)
        .tab(&crate::tabs::FUNCTIONAL);
    r.block(&BOOKSHELF)
        .sound_group("wood")
        .textures("planks_oak", "bookshelf", "planks_oak")
        .hardness(1.5)
        .mineable("axe")
        .set(&FLAMMABLE, 30)
        .tab(&crate::tabs::BUILDING);
    r.block(&HAY_BALE)
        .sound_group("grass")
        .textures("hay_bale_top", "hay_bale_side", "hay_bale_top")
        .hardness(0.5)
        .set(&FLAMMABLE, 60)
        .tab(&crate::tabs::BUILDING);

    // What may grow or stand where, from the common `dirt` and `sand` tags like modern vanilla:
    // a mod's sand tagged `sand` gets cactus and sugar cane, in the world and from players.
    r.tag("plant_soil", &["#dirt"]);
    r.tag("sugar_cane", &["sugar_cane"]);
    r.tag("hay_bale", &["hay_bale"]);
    r.tag("dyes/red", &["rose"]);
    r.tag("dyes/green", &["cactus"]);
    r.tag("torches", &["torch"]);
    r.tag("furnaces", &["furnace"]);
    r.tag("crafting_tables", &["crafting_table"]);
    r.tag("cobblestone", &["cobblestone"]);
    r.tag("plant_soil", &["farmland"]);
    r.tag("sugar_cane_soil", &["#dirt"]);
    r.tag("sugar_cane_soil", &["#sand"]);
    r.tag("sugar_cane_soil", &["sugar_cane"]);
    r.tag("cactus_soil", &["#sand"]);
    r.tag("cactus_soil", &["cactus"]);
    r.tag("crop_soil", &["farmland"]);
    r.tag("grass_spreads_to", &["dirt"]);
    r.tag("animal_spawn", &["grass"]);
}

/// Torches stand on the floor or hang on a wall, facing away from it.
pub const TORCH_FACINGS: Facings = Facings::only(&[Dir::Up, Dir::North, Dir::East, Dir::South, Dir::West]);

/// A torch facing `d`: on the floor (`Up`), hanging (`Down`), or leaning out from the wall behind
/// it, by vanilla's numbers.
pub fn torch_state<'a>(d: Dir, s: State<'a>, texture: &str) -> State<'a> {
    let side = (texture, [7.0, 6.0, 9.0, 16.0]);
    let faces = [side, side, (texture, [7.0, 13.0, 9.0, 15.0]), (texture, [7.0, 6.0, 9.0, 8.0]), side, side];
    let s = s.texture(texture).needs(d.opposite().face(), "");
    if d.horizontal() {
        let frame = weft_sdk::signal::Frame::leaning(d.opposite());
        s.framed_box_with(frame, [-1.0, 3.5, 7.0], [1.0, 13.5, 9.0], faces, [0; 6], Some(([0.0, 3.5, 8.0], Dir::South, -22.5)))
            .framed_selection(frame, [0.0, 3.0, 5.5], [5.0, 13.0, 10.5])
    } else {
        let frame = weft_sdk::signal::Frame::new(d.opposite(), Dir::North);
        s.framed_box(frame, [7.0, 0.0, 7.0], [9.0, 10.0, 9.0], faces)
            .framed_selection(frame, [6.0, 0.0, 6.0], [10.0, 10.0, 10.0])
    }
}

/// Sugar cane and cacti grow a block on top, up to three high.
fn grow_column(e: &mut BlockEvent, me: &'static Block) {
    let w = e.world();
    let p = e.pos();
    let above = [p[0], p[1] + 1, p[2]];
    if w.get(above) != 0 {
        return;
    }
    let mut height = 1;
    while height < 3 && me.is(w.get([p[0], p[1] - height, p[2]])) {
        height += 1;
    }
    if height < 3 {
        w.set(above, me.id());
    }
}

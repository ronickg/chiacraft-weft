use weft_sdk::*;

use crate::items::{FLAMMABLE, FUEL};
use crate::worldgen;

pub use base_api::blocks::*;

// Ours (the rest are in `base_api::blocks`).
/// A live impact crater's melt lining its bowl, and the broken rock (impact breccia) of its rim
/// (`impact.rs`; Art's, Research round 17).
pub static IMPACT_GLASS: Block = Block::named("impact_glass");
pub static SUEVITE: Block = Block::named("suevite");
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
pub static LAKE_ICE: Block = Block::named("lake_ice");
// Coral for the warm reefs and lagoons (Art's blocks; `land.rs` coral).
pub static BRAIN_CORAL_BLOCK: Block = Block::named("brain_coral_block");
pub static STAGHORN_CORAL_BLOCK: Block = Block::named("staghorn_coral_block");
pub static PORITES_CORAL_BLOCK: Block = Block::named("porites_coral_block");
pub static CORAL_RUBBLE: Block = Block::named("coral_rubble");
/// The floor of glaciers in the high valleys (`land.rs`, `base_api::land::glacier`; Art's tiles).
pub static GLACIER_ICE: Block = Block::named("glacier_ice");
// Ores in the rock faces (`base_api::land::face_ore`; Art's textures, Ronald: "Yes, all four").
pub static BANDED_CLAY_COAL_ORE: Block = Block::named("banded_clay_coal_ore");
pub static RED_SANDSTONE_IRON_ORE: Block = Block::named("red_sandstone_iron_ore");
pub static GRANITE_GOLD_ORE: Block = Block::named("granite_gold_ore");
pub static GRANITE_IRON_ORE: Block = Block::named("granite_iron_ore");
/// The savanna's flat-topped tree.
pub static ACACIA_LOG: Block = Block::named("acacia_log");
pub static ACACIA_PLANKS: Block = Block::named("acacia_planks");
pub static ACACIA_LEAVES: Block = Block::named("acacia_leaves");
pub static GRAZED_GRASS: Block = Block::named("grazed_grass");
pub static DUNG_PAT: Block = Block::named("dung_pat");
pub static COMPOST_BIN: Block = Block::named("compost_bin");
/// Camp comfort (books, `survival::camp`), built from planks and books.
pub static BOOKSHELF: Block = Block::named("bookshelf");
/// Nine wheat packed (and back), soft to rest by.
pub static HAY_BALE: Block = Block::named("hay_bale");
/// Spruce forests' ground (`worldgen`), soil like dirt.
pub static PODZOL: Block = Block::named("podzol");
/// The desert's scrub: a few sticks where there is no wood.
pub static DEAD_BUSH: Block = Block::named("dead_bush");
/// Game seconds between a blooming leaf's looks at the season, on average (`bloom`).
const BLOOM_SECS: f32 = 300.0;
/// The sequoia grove's understory (`grove`).
pub static SWORD_FERN: Block = Block::named("sword_fern");
pub static LUPINE: Block = Block::named("lupine");
pub static DOGWOOD_LOG: Block = Block::named("dogwood_log");
pub static DOGWOOD_LEAVES: Block = Block::named("dogwood_leaves");
/// The tropical forest's lianas, hanging from the crowns (`worldgen.rs`, `lianas`).
pub static LIANA: Block = Block::named("liana");
/// A Brazil nut's buttress fin, and the bromeliads on its limbs (`worldgen.rs`, `giant_foot`).
pub static BRAZIL_NUT_BUTTRESS: Block = Block::named("brazil_nut_buttress");
pub static BROMELIAD: Block = Block::named("bromeliad");
pub static MOSSY_SEQUOIA_LOG: Block = Block::named("mossy_sequoia_log");
/// The forest floor's blocks (`vitals::layers`): fallen leaves, a moss carpet, stony bare dirt.
pub static LEAF_LITTER: Block = Block::named("leaf_litter");
/// The tropical forest's deep litter (`vitals::layers`).
pub static DEEP_LEAF_LITTER: Block = Block::named("deep_leaf_litter");
pub static MOSS_BLOCK: Block = Block::named("moss_block");
/// The shrubs' own leaves, in their own colours: willow, juniper, creosote bush.
pub static WILLOW_LEAVES: Block = Block::named("willow_leaves");
pub static JUNIPER_LEAVES: Block = Block::named("juniper_leaves");
pub static CREOSOTE_LEAVES: Block = Block::named("creosote_leaves");
/// Grass two blocks tall (the prairie's big bluestem, the llanos' grasses): `half` lower, upper.
pub static TALL_GRASS_DOUBLE: Block = Block::named("tall_grass_double");
/// The tropical forest's big-leaved plants, two blocks tall (`half` lower, upper).
pub static BANANA: Block = Block::named("banana");
pub static PHILODENDRON: Block = Block::named("philodendron");
/// Mushrooms at the foot of trunks (`vitals::layers`): honey fungus, chanterelles, ceps.
pub static HONEY_FUNGUS: Block = Block::named("honey_fungus");
pub static CHANTERELLE: Block = Block::named("chanterelle");
pub static PORCINI: Block = Block::named("porcini");
/// A shelf fungus on bark, on the north side of a dead trunk (`base_api::trees`).
pub static BRACKET_FUNGUS: Block = Block::named("bracket_fungus");

/// What a cactus does to who touches it.
pub static PRICKED: DamageCause = DamageCause::named("cactus");

/// Log axis along the clicked face, like vanilla.
fn axis_placement(p: &Place, props: &mut Props) {
    props.set(
        "axis",
        match p.face() {
            Dir::West | Dir::East => "x",
            Dir::North | Dir::South => "z",
            _ => "y",
        },
    );
}

/// A log that grows into a tree of `leaves` (a cone, like a spruce, when `cone`).
/// `shape`: the tree's tag (`cone_tree`, `flat_tree`; none: round), `worldgen::tree_blocks`.
/// Each has a stripped twin, `stripped_<wood>_log` (`wood` `<wood>_stripped`, no `leaves`), its
/// wood's log for recipes, which an axe used on it makes (`crafts::strip`).
fn log(r: &mut Registry, handle: &'static Block, wood: &str, leaves: &str, shape: Option<&str>) {
    if !leaves.is_empty() {
        log(r, Block::generated(format!("stripped_{wood}_log")), &format!("{wood}_stripped"), "", None);
    }
    let side = format!("log_{wood}");
    let top = format!("log_{wood}_top");
    let b = r.block(handle);
    let b = if leaves.is_empty() { b } else { b.set_text(&crate::worldgen::TREE_LEAVES, leaves) };
    let b = match shape {
        Some(tag) => b.tag(tag),
        None => b,
    };
    let b = if leaves.is_empty() { b } else { b.on_use(crate::crafts::strip) };
    b.sound_group("wood")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tag("logs")
        .tag(&format!("logs/{}", wood.trim_end_matches("_stripped")))
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        // `bark`: bark on every face, for a trunk whose top shows (a giant's flared foot).
        .prop("axis", ["y", "x", "z", "bark"])
        // Lying after a felling, in alternating pieces (`felled`): chopped a piece at a time.
        .prop("felled", ["no", "a", "b"])
        .states(move |p, s| match p.get("axis") {
            "y" => s.texture(&side).top(&top).bottom(&top),
            // The bark runs along the log.
            "x" => s
                .west(&top)
                .east(&top)
                .bottom(&side)
                .top(&side)
                .north(&side)
                .south(&side)
                .face_turns([0, 0, 1, 1, 1, 1]),
            "bark" => s.texture(&side),
            _ => s
                .west(&side)
                .east(&side)
                .bottom(&side)
                .top(&side)
                .north(&top)
                .south(&top)
                .face_turns([1, 1, 0, 0, 0, 0]),
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
/// The tropical forest's (Manaus's terra firme): the Brazil nut, its emergent, grown from branches
/// (`tree_recipes::BRAZIL_NUT`), its nuts a food; cacao, its understorey tree, its pods a food.
/// The tropical forest's canopy, 28-32 blocks up, three of Manaus's commonest trees, each in its
/// own colours (`vitals::wild::OWN_COLOURS`): andiroba's dark glossy crown, matamatá's olive one
/// (the commonest tree there), ipê's light green that turns all golden yellow in its spring bloom.
/// The wild cherry (Prunus avium) of the old-growth and temperate forests, in groves, pink with
/// blossom in spring (the blossom in its own colours over its young leaves, tinted), green in
/// summer and russet in autumn as the forest's tint and season make it, its cherries a summer
/// fruit.
const WOODS: [(&str, Option<&str>, &str); 10] = [
    ("mesquite", Some("flat_tree"), "mesquite_pod"),
    ("acuri_palm", Some("palm_tree"), "acuri_nut"),
    ("moriche_palm", Some("palm_tree"), "moriche_fruit"),
    ("sequoia", Some("cone_tree"), "sequoia_cone"),
    ("brazil_nut", None, "brazil_nut"),
    ("cacao", None, "cacao_pod"),
    ("andiroba", None, "andiroba_seed"),
    ("matamata", None, "matamata_fruit"),
    ("ipe", None, "ipe_seed"),
    ("cherry", None, "cherry"),
];

/// The sapling a tree's fruit plants (`WOODS`), for fruit that's a food (`items.rs` registers
/// those).
pub fn sapling_planted_by(item: &str) -> Option<&'static Block> {
    WOODS
        .iter()
        .find(|(_, _, fruit)| *fruit == item)
        .map(|(wood, _, _)| Block::generated(format!("{wood}_sapling")))
}

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
            .grows(crate::growth::step_secs(plant, 1, 470.0), 9)
            .on_grow(move |e| crate::growth::grow(e, plant, false, |e| worldgen::grow_tree(e.world(), e.pos().to_array(), log_block.id())))
            .tab(&crate::tabs::NATURAL);
        // A fruit that's a food is registered with the foods (`items.rs`), which plants it too.
        if vitals::food::find(&format!("base:{fruit}")).is_none() {
            r.item(Item::generated(fruit.into()))
                .places(sapling_block)
                .tab(&crate::tabs::NATURAL);
        }
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
        .tint_kind(&tints::GRASS)
        .tab(&crate::tabs::NATURAL);
    r.loot(&SWORD_FERN).pool(
        Pool::new()
            .when(Cond::chance(vitals::clothing::FIBRE_CHANCE))
            .item("plant_fibre"),
    );
    plant(r, &LUPINE, "lupine");
    log(r, &DOGWOOD_LOG, "dogwood", "base:dogwood_leaves", None);
    // The dogwood is white with its bracts in spring and plain green the rest of the year
    // (`vitals::wild::BLOOMS`), both drawn in their own colours.
    let b = tree_leaves(r.block(&DOGWOOD_LEAVES).sound_group("grass"));
    bloom(b, DOGWOOD_LEAVES.name(), "leaves_dogwood", "leaves_dogwood_green", None)
        .hardness(0.2)
        .cutout()
        .cull_same()
        .light_opacity(1)
        .tag("leaves")
        .signal_blocker()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    r.loot(&DOGWOOD_LEAVES)
        .pool(Pool::new().when(Cond::chance(0.05)).item("stick"));
    // A fallen giant's bark under moss, its end mossy too.
    r.block(&MOSSY_SEQUOIA_LOG)
        .sound_group("wood")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tag("logs")
        .tag("logs/sequoia")
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .prop("axis", ["y", "x", "z"])
        .states(|p, s| match p.get("axis") {
            "y" => s
                .texture("log_sequoia_mossy")
                .top("log_sequoia_mossy_top")
                .bottom("log_sequoia_mossy_top"),
            "x" => s
                .west("log_sequoia_mossy_top")
                .east("log_sequoia_mossy_top")
                .bottom("log_sequoia_mossy")
                .top("log_sequoia_mossy")
                .north("log_sequoia_mossy")
                .south("log_sequoia_mossy"),
            _ => s
                .west("log_sequoia_mossy")
                .east("log_sequoia_mossy")
                .bottom("log_sequoia_mossy")
                .top("log_sequoia_mossy")
                .north("log_sequoia_mossy_top")
                .south("log_sequoia_mossy_top"),
        })
        .placement(axis_placement)
        .tab(&crate::tabs::NATURAL);
}

fn planks(r: &mut Registry, h: &'static Block, wood: &str) {
    r.block(h)
        .sound_group("wood")
        .texture(&format!("planks_{wood}"))
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tag("planks")
        .tag(&format!("planks/{wood}"))
        .tag("materials/wood")
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .tab(&crate::tabs::BUILDING);
}

/// Leaves that drop `sapling` (a time in 20) and, for our trees, `fruit` (a time in 20).
fn leaves(r: &mut Registry, handle: &'static Block, tex: &str, sapling: &str, fruit: Option<&str>) {
    let b = tree_leaves(r.block(handle).sound_group("grass").texture(tex));
    // Its tint by how it turns in autumn (`vitals::wild::LEAVES`), but for leaves drawn in their
    // own colours (the tropical canopy's, `vitals::wild::OWN_COLOURS`).
    let kind = (!vitals::wild::OWN_COLOURS.contains(&handle.name())).then(|| base_api::trees::leaf_tint(vitals::wild::turn(handle.name())));
    let b = bloom(b, handle.name(), leak(format!("{tex}_bloom")), leak(tex.to_string()), kind)
        .hardness(0.2)
        .cutout()
        // Faces between two leaves of one tree kind aren't drawn (each was drawn twice): a crown
        // keeps its outer shell and its holes (Researcher's performance report, item 9).
        .cull_same()
        .light_opacity(1);
    let b = match kind {
        Some(kind) => b.tint_kind(kind),
        None => b,
    };
    b.tag("leaves")
        .signal_blocker()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    // Vanilla: a sapling one time in 20, oak leaves an apple one time in 200.
    let table = r
        .loot(handle)
        .pool(Pool::new().when(Cond::chance(0.05)).item(sapling));
    if let Some(fruit) = fruit {
        table.pool(Pool::new().when(Cond::chance(0.05)).item(fruit));
        return;
    }
    // Ours: acorns from oak leaves, pine nuts from spruce cones (`vitals::forage::DROPPED`,
    // `food::FOODS`).
    if std::ptr::eq(handle, &OAK_LEAVES) {
        table
            .pool(Pool::new().when(Cond::chance(0.005)).item("apple"))
            .pool(Pool::new().when(Cond::chance(0.05)).item("acorn"));
    } else if std::ptr::eq(handle, &SPRUCE_LEAVES) {
        table.pool(Pool::new().when(Cond::chance(0.03)).item("pine_nut"));
    }
}

/// Leaves that flower (`vitals::wild::BLOOMS`): `bloom` 1 in their seasons, drawn as `open`, and
/// `shut` the rest of the year, a growth step every few minutes looking at the season as the wild
/// flowers do (`wild.rs`). Leaves that don't bloom are left as they are.
/// `tint`: the leaves' tint kind, put on each state's texture (upstream's `tint_kind` colours only
/// the faces set so far, and a state's texture replaces the block's).
fn bloom<'a>(
    b: BlockBuilder<'a>,
    leaves: &str,
    open: &'static str,
    shut: &'static str,
    tint: Option<&'static TintKind>,
) -> BlockBuilder<'a> {
    let Some(seasons) = vitals::wild::blooms(leaves) else { return b };
    b.prop("bloom", 0..=1)
        .states(move |p, s| {
            let s = s.texture(if p.get("bloom") == "1" { open } else { shut });
            match tint {
                Some(kind) => s.tint_kind(kind),
                None => s,
            }
        })
        .grows(BLOOM_SECS, 0)
        .on_grow(move |e| {
            let open = vitals::wild::in_bloom(seasons, base_api::ground_at(&e.world(), e.pos().to_array()).season);
            let want = if open { "1" } else { "0" };
            if state_prop(e.state(), "bloom") == Some(want) {
                return;
            }
            // The whole crown round it turns with it, in one go: leaf by leaf, each on its own
            // step a few minutes apart, a forest's thousands of leaves changed for a quarter of an
            // hour after a season turned, and the world round them never settled (the tropical
            // forest's stall, found on the PC).
            let w = e.world();
            let [x, y, z] = e.pos().to_array();
            let kind = block_name(e.state()).split('[').next().unwrap_or("");
            for dy in -BLOOM_REACH..=BLOOM_REACH {
                for dz in -BLOOM_REACH..=BLOOM_REACH {
                    for dx in -BLOOM_REACH..=BLOOM_REACH {
                        let p = [x + dx, y + dy, z + dz];
                        let st = w.block(p);
                        if block_name(st).split('[').next() == Some(kind) && state_prop(st, "bloom") != Some(want) {
                            w.prop_set(p, "bloom", want);
                        }
                    }
                }
            }
        })
}

/// How far round a blooming leaf its crown turns with it (`bloom`): a crown's reach.
const BLOOM_REACH: i32 = 4;

/// The state a tree's leaves are made in: in flower for the spring bloomers
/// (`vitals::wild::BLOOMS`), since every world starts in spring; others as they are.
pub fn made_leaves(state: u32) -> u32 {
    static OPEN: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    let open = OPEN.get_or_init(|| {
        vitals::wild::BLOOMS
            .iter()
            .filter(|(_, seasons)| vitals::wild::in_bloom(*seasons, Some(0)))
            .filter_map(|(name, _)| {
                let id = Block::lookup(name)?;
                Some((id, state_with(id, "bloom", "1")?))
            })
            .collect()
    });
    open.iter()
        .find(|(shut, _)| *shut == state)
        .map_or(state, |&(_, open)| open)
}

/// The state far crowns of `state` leaves are drawn in at day time `time` (distant terrain's
/// `far_leaves`): its crowns come from trees made once, in flower (`made_leaves`), so out of the
/// bloomer's seasons the bloom is taken off (the dense-forest session, 11-05: far wild cherries
/// stayed pink all year).
pub fn far_leaves(time: u64, state: u32) -> u32 {
    static OPEN: std::sync::OnceLock<Vec<(u32, u32, u8)>> = std::sync::OnceLock::new();
    let open = OPEN.get_or_init(|| {
        vitals::wild::BLOOMS
            .iter()
            .filter_map(|&(name, seasons)| {
                let id = Block::lookup(name)?;
                Some((state_with(id, "bloom", "1")?, id, seasons))
            })
            .collect()
    });
    let Some(&(_, shut, seasons)) = open.iter().find(|o| o.0 == state) else { return state };
    let season = vitals::calendar::date(time).season.index() as usize;
    if vitals::wild::in_bloom(seasons, Some(season)) { state } else { shut }
}

/// A tree's leaves: they decay once no log is near (`leaf_decay`), unless a player placed them
/// (`placed`, set as they are placed). The shrubs' leaves have no logs and keep.
fn tree_leaves(b: BlockBuilder<'_>) -> BlockBuilder<'_> {
    b.prop("placed", ["false", "true"]).tag("leaves/tree")
}

/// A name kept for the game's life (registration's textures).
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn plant(r: &mut Registry, handle: &'static Block, tex: &str) {
    r.block(handle)
        .sound_group("grass")
        .texture(tex)
        .plant()
        .replaceable()
        .hitbox([5.0, 0.0, 5.0], [11.0, 10.0, 11.0])
        .needs_below("#plant_soil")
        .tab(&crate::tabs::NATURAL);
}

/// Sand and gravel: with nothing under them they fall as an entity and land as a block.
pub fn register(r: &mut Registry) {
    // First: soil is any dirt. The engine sets a tag by tag in the order the calls come, reading
    // the tag as it is then, so `plant_soil` must hold the dirt before `tall_grass_soil` and
    // `big_leaf_soil` read it below. When it came after, double tall grass, bananas and
    // philodendrons on grass had no soil and broke at the first block update beside them (the
    // plant fibre round the grazing herds, 11-05).
    r.tag("plant_soil", &["#dirt"]);
    // The leaves' tint kinds besides foliage (`base_api::trees`), as green as foliage by default.
    r.tint_kind(&base_api::trees::EVERGREEN, vitals::foliage::muted(vitals::foliage::LEAVES));
    r.tint_kind(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(vitals::foliage::LEAVES));
    // Any block of any mod tagged so comes in these too.
    r.tag(
        "has_shapes",
        &[
            "base:stone",
            "base:cobblestone",
            "base:mossy_cobblestone",
            "base:sandstone",
            "base:bricks",
            "base:stone_bricks",
            "base:oak_planks",
            "base:birch_planks",
            "base:spruce_planks",
        ],
    );
    r.stairs("#has_shapes").tab(&crate::tabs::BUILDING);
    r.slabs("#has_shapes").tab(&crate::tabs::BUILDING);
    r.vertical_slabs("#has_shapes").tab(&crate::tabs::BUILDING);
    // Upstream's quarter and eighth blocks, cut at its woodcutter (`woodcutter.rs`).
    r.beams("#has_shapes").tab(&crate::tabs::BUILDING);
    r.eighths("#has_shapes").tab(&crate::tabs::BUILDING);
    r.block(&STONE)
        .sound_group("stone")
        .hardness(1.5)
        .mined_by("pickaxe", 0)
        .drops("cobblestone", 1)
        .tag("stone")
        .tag("smooth_stone")
        .tag("stone_ore_replaceables")
        .tab(&crate::tabs::BUILDING);
    r.block(&COBBLESTONE)
        .sound_group("stone")
        .hardness(2.0)
        .mined_by("pickaxe", 0)
        .tag("stone")
        .tag("cobblestone")
        .tag("materials/stone")
        .tab(&crate::tabs::BUILDING);
    r.block(&MOSSY_COBBLESTONE)
        .sound_group("stone")
        .hardness(2.0)
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::BUILDING);
    r.block(&DIRT)
        .sound_group("gravel")
        .hardness(0.5)
        .mined_by("shovel", tool_tier::HAND)
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    r.block(&GRASS)
        .sound_group("grass")
        .texture("grass_side")
        .bottom("dirt")
        .top("grass_top")
        .tint_kind(&tints::GRASS)
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .drops("dirt", 1)
        .tag("dirt")
        // Vanilla's pace: a minute or so under a block, many minutes to cover bare dirt.
        .spreads("#grass_spreads_to", 765.0, 9)
        .smothered_into("dirt", 68.0)
        .tab(&crate::tabs::NATURAL);
    r.block(&SNOWY_GRASS)
        .sound_group("grass")
        .texture("grass_snow_side")
        .top("snow")
        .bottom("dirt")
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .drops("dirt", 1)
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    r.block(&SAND)
        .sound_group("sand")
        .hardness(0.5)
        .mined_by("shovel", tool_tier::HAND)
        .tag("sand")
        .falling()
        .tab(&crate::tabs::NATURAL);
    r.block(&GRAVEL)
        .sound_group("gravel")
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .falling()
        .tab(&crate::tabs::NATURAL);
    r.block(&IMPACT_GLASS)
        .sound_group("glass")
        .hardness(1.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    r.block(&SUEVITE)
        .sound_group("stone")
        .hardness(1.2)
        .mined_by("pickaxe", tool_tier::HAND)
        .tag("rocks")
        .tab(&crate::tabs::NATURAL);
    r.block(&CLAY)
        .sound_group("gravel")
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .tag("clay")
        .tab(&crate::tabs::NATURAL);
    r.block(&SANDSTONE)
        .sound_group("stone")
        .texture("sandstone_side")
        .top("sandstone_top")
        .bottom("sandstone_bottom")
        .hardness(0.8)
        .mined_by("pickaxe", 0)
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
            .mined_by("pickaxe", 0);
        match textures {
            Some((top, side, bottom)) => b.texture(side).top(top).bottom(bottom),
            None => b,
        }
        .tab(&crate::tabs::BUILDING);
    }
    r.block(&SNOW)
        .sound_group("sand")
        .hardness(0.2)
        .mined_by("shovel", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    // Blue lamps and glass are made with it.
    r.block(&ICE)
        .signal_blocker()
        .sound_group("glass")
        .tag("ice")
        .tag("dyes/blue")
        .hardness(0.5)
        .translucent()
        .light_opacity(2)
        .friction(0.98)
        // Itself with a pickaxe (Research round 3, U1: it's the only blue dye).
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::NATURAL);
    // Water's skin of ice in the cold (`snow.rs`, `vitals::air::freeze_chance`), drawn as ice: it
    // thaws back to water in the warmth, also away from players (`snow::ice_step`). Broken, it's
    // gone (the water under it stays); ice set by a player is `ICE`, which keeps.
    r.block(&LAKE_ICE)
        .sound_group("glass")
        .texture("ice")
        .hardness(0.5)
        .translucent()
        .light_opacity(2)
        .friction(0.98)
        .mined_by("pickaxe", tool_tier::HAND)
        .drops_nothing()
        .no_item()
        .grows(vitals::air::ICE_THAW_SECS, 0)
        .on_grow(crate::snow::ice_step);
    // Packed by its own weight: opaque, slippery like ice, mined with a pickaxe.
    r.block(&GLACIER_ICE)
        .sound_group("glass")
        .texture("glacier_ice_side")
        .top("glacier_ice_top")
        .bottom("glacier_ice_top")
        .hardness(0.5)
        .friction(0.98)
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::NATURAL);
    // Coral: stony, mined with a pickaxe, dropping itself.
    for (block, texture) in [
        (&BRAIN_CORAL_BLOCK, "brain_coral_block"),
        (&STAGHORN_CORAL_BLOCK, "staghorn_coral_block"),
        (&PORITES_CORAL_BLOCK, "porites_coral_block"),
    ] {
        r.block(block)
            .sound_group("stone")
            .texture(texture)
            .hardness(1.0)
            .mined_by("pickaxe", 0)
            .tag("coral")
            .tab(&crate::tabs::NATURAL);
    }
    // Broken coral and sand: loose, a shovel's.
    r.block(&CORAL_RUBBLE)
        .sound_group("gravel")
        .texture("coral_rubble")
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .tag("coral")
        .tab(&crate::tabs::NATURAL);
    r.block(&OBSIDIAN)
        .immovable()
        .sound_group("stone")
        .hardness(50.0)
        .mined_by("pickaxe", tool_tier::DIAMOND)
        .tab(&crate::tabs::NATURAL);
    r.block(&COAL_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mined_by("pickaxe", 0)
        .drops("coal", 1)
        .tag("ores")
        .tag("ores/coal")
        .tab(&crate::tabs::NATURAL);
    r.block(&IRON_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::STONE)
        .drops("raw_iron", 1)
        .tag("ores")
        .tag("ores/iron")
        .tab(&crate::tabs::NATURAL);
    // Gold is the Bronze Age's (cast, soft; Ronald, 11-07: "move them to fit"): a stone pickaxe.
    r.block(&GOLD_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::STONE)
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
        (&GRANITE_GOLD_ORE, "granite_gold_ore", "raw_gold", tool_tier::STONE, "gold"),
    ] {
        r.block(block)
            .sound_group("stone")
            .texture(texture)
            .hardness(3.0)
            .mined_by("pickaxe", tier)
            .drops(drop, 1)
            .tag("ores")
            .tag(&format!("ores/{kind}"))
            .tab(&crate::tabs::NATURAL);
    }
    // Diamond is a tier above steel (`notes/CRAFTING.md`): a steel pick or better (300; star
    // steel's 350, diamond's own 400).
    r.block(&DIAMOND_ORE)
        .sound_group("stone")
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::DIAMOND)
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
    // The broadleaf trees' thin wood as branch blocks (Ronald, 11-05: "All broadleaf trees").
    for wood in base_api::trees::BRANCH_WOODS {
        branch(r, wood);
    }
    // Lianas: woody climbers hanging from the tropical forest's crowns, a rope one can climb;
    // cut, their stems are cordage now and then (`vitals::clothing`).
    r.block(&LIANA)
        .sound_group("grass")
        .texture("liana")
        .plant()
        .climbable()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    r.loot(&LIANA)
        .pool(Pool::new().when(Cond::chance(0.5)).item("plant_fibre"));
    // A giant's buttress: a plank of wood flaring from the trunk's foot, its own block so its
    // grain sweeps up; broken, it gives the log.
    r.block(&BRAZIL_NUT_BUTTRESS)
        .sound_group("wood")
        .texture("buttress_brazil_nut")
        .top("log_brazil_nut_top")
        .bottom("log_brazil_nut_top")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    r.loot(&BRAZIL_NUT_BUTTRESS)
        .pool(Pool::new().entry(Entry::item("brazil_nut_log")));
    // Bromeliads perched on the giants' limbs (epiphytes: rooted in the bark, not the ground).
    r.block(&BROMELIAD)
        .sound_group("grass")
        .texture("bromeliad")
        .plant()
        .tab(&crate::tabs::NATURAL);

    // Vanilla numbers: water steps every 5 ticks and reaches 7 blocks, lava every 30 and 3.
    // Players swim in water; what they wear may add more (`SWIMS_IN` on an item).
    r.edit(&PLAYER).set_text(&SWIMS_IN, "#water");
    // A light level lost per block of water, as vanilla's (and Distant Horizons' far floors): at the
    // engine's two, a lake's bed ten blocks down was black and its water a flat blue over nothing,
    // where vanilla's shows its sand at sky light five (Far view, stage 1, 11-06).
    r.liquid(&WATER)
        .block(|b| {
            b.tint_kind(&tints::WATER)
                .light_opacity(1)
                .translucent()
                .set_text(&DRIP, abi::particle::DRIP)
                .set_text(&SPLASH, abi::particle::SPLASH)
                .set_text(&BUBBLE, abi::particle::BUBBLE)
                .tag("water")
        })
        .renewable(2)
        .fog(0x050533, -8.0, 96.0)
        // Upstream's: under water sounds muffled.
        .environment(
            Environment::new("water_acoustics")
                .priority(-700)
                .acoustics(900.0, 0.0, 1.0),
        )
        .bucket(Item::generated("water_bucket".into()), &crate::items::BUCKET, &crate::tabs::TOOLS);
    r.liquid(&LAVA)
        .block(|b| {
            b.light_rgb(15, 10, 4)
                .drag(0.4)
                .tag("lava")
                .tag("cools_into_stone")
        })
        .flow_delay(30)
        .drop_off(2)
        .slope_find(2)
        .heat(1300)
        .fog(0x991A00, 0.25, 1.0)
        .no_overlay()
        .bucket(Item::generated("lava_bucket".into()), &crate::items::BUCKET, &crate::tabs::TOOLS);
    r.liquid_reaction("#cools_into_stone", "#water")
        .source_turns_into("obsidian")
        .flowing_turns_into("cobblestone")
        .below_turns_into("stone");

    r.block(&TALL_GRASS)
        .sound_group("grass")
        .texture("tallgrass")
        .plant()
        .replaceable()
        .needs_below("#plant_soil")
        .tint_kind(&tints::GRASS)
        .no_item();
    r.loot(&TALL_GRASS)
        .pool(
            Pool::new()
                .when(Cond::chance(vitals::crops::WHEAT_SEED_CHANCE))
                .item("wheat_seeds"),
        )
        // Fibre for string and linen (`clothing.rs`).
        .pool(
            Pool::new()
                .when(Cond::chance(vitals::clothing::FIBRE_CHANCE))
                .item("plant_fibre"),
        );
    // Tall grass an animal has grazed, growing back by warmth and rain (`growth.rs`).
    r.block(&GRAZED_GRASS)
        .sound_group("grass")
        .texture("grazed_grass")
        .plant()
        .needs_below("#plant_soil")
        .tint_kind(&tints::GRASS)
        .no_item()
        .grows(crate::growth::step_secs("base:grazed_grass", 1, 600.0), 9)
        .on_grow(|e| {
            // Beside a dung pat it grows back `PAT_GROWTH` times as fast: as many tries a step.
            let tries = if pat_near(&e.world(), e.pos().to_array()) { vitals::soil::PAT_GROWTH } else { 1 };
            for _ in 0..tries {
                match base_api::grow::step(e, "base:grazed_grass", false) {
                    base_api::grow::Step::Grow => {
                        e.world().set_block(e.pos(), TALL_GRASS.id());
                        return;
                    }
                    base_api::grow::Step::Frost => return,
                    base_api::grow::Step::Rest => {}
                }
            }
        });
    // A grazer's dung pat (`vitals::soil`, `mods/animals/src/forage.rs`): it rots by warmth and
    // rain into a tuft of fresh grass; broken, it gives the dung.
    r.block(&DUNG_PAT)
        .sound_group("mud")
        .texture("dung_pat_side")
        .top("dung_pat_top")
        .bottom("dung_pat_top")
        // A plant first: `.plant()` makes the model a cross, so the pat's box comes after it.
        .plant()
        .model_box([2.0, 0.0, 2.0], [14.0, 1.5, 14.0])
        // Its top's corners are clear (Art's tile), so the grass shows round the pat.
        .cutout()
        .needs_below("#plant_soil")
        .no_item()
        .grows(crate::growth::step_secs("base:dung_pat", 1, 1200.0), 0)
        .on_grow(|e| {
            // Rotted, or broken up at last (`vitals::soil::PAT_MOST_DAYS`): a tuft of fresh grass.
            let rots = matches!(base_api::grow::step(e, "base:dung_pat", false), base_api::grow::Step::Grow)
                || ((e.world().random() >> 11) as f32 / (1u64 << 53) as f32) < vitals::soil::pat_break_chance();
            if rots {
                e.world().set_block(e.pos(), TALL_GRASS.id());
            }
        });
    r.loot(&DUNG_PAT).pool(Pool::new().item("dung"));
    plant(r, &DANDELION, "flower_dandelion");
    plant(r, &ROSE, "flower_rose");
    // A wild rose gives itself and, a time in four, its hips.
    r.loot(&ROSE)
        .pool(Pool::new().entry(Entry::item("rose")))
        .pool(Pool::new().when(Cond::chance(0.25)).item("rose_hip"));
    r.block(&SAPLING)
        .sound_group("grass")
        .texture("sapling_oak")
        .plant()
        .needs_below("#plant_soil")
        .grows(crate::growth::step_secs("base:sapling", 1, 470.0), 9)
        .on_grow(|e| crate::growth::grow(e, "base:sapling", false, |e| worldgen::grow_tree(e.world(), e.pos().to_array(), OAK_LOG.id())))
        .tab(&crate::tabs::NATURAL);
    r.block(&SUGAR_CANE)
        .sound_group("grass")
        .texture("reeds")
        .item_texture("reeds")
        .plant()
        .needs_below("#sugar_cane_soil")
        .needs_near("#water")
        .grows(crate::growth::step_secs("base:sugar_cane", 1, 270.0), 0)
        .on_grow(|e| crate::growth::grow(e, "base:sugar_cane", false, |e| grow_column(e, &SUGAR_CANE)))
        .tab(&crate::tabs::NATURAL);
    // Upstream's: a cactus pricks who touches it.
    r.damage_cause(&PRICKED).death_message("death.base.cactus");
    r.block(&CACTUS)
        .grid_placement()
        .sound_group("cloth")
        .texture("cactus_side")
        .ends("cactus_top")
        .model_box([1.0, 0.0, 1.0], [15.0, 16.0, 15.0])
        .hitbox([1.0, 0.0, 1.0], [15.0, 16.0, 15.0])
        .hardness(0.4)
        .needs_below("#cactus_soil")
        .hazard(Hazard::touch(1.0, &PRICKED))
        .grows(crate::growth::step_secs("base:cactus", 1, 270.0), 0)
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
        .grows(crate::growth::step_secs("base:wheat", 7, 205.0), 9)
        .on_grow(|e| crate::growth::grow(e, "base:wheat", false, |e| crate::growth::next_age(e, 7)));
    // Modern's: unripe gives its seed back, ripe gives the wheat and up to 3 more seeds.
    r.loot(&WHEAT)
        .pool(Pool::new().when(Cond::state("age=7")).item("wheat"))
        .pool(Pool::new().entry(Entry::item("wheat_seeds")))
        .pool(
            Pool::new()
                .when(Cond::state("age=7"))
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
            s.texture("dirt").top(&top).bottom("dirt")
        })
        .grows(vitals::soil::FALLOW_SECS, 0)
        .on_grow(crate::farming::fallow)
        .model_box([0.0, 0.0, 0.0], [16.0, 15.0, 16.0])
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .drops("dirt", 1)
        .tab(&crate::tabs::NATURAL);

    r.block(&GLASS)
        .signal_blocker()
        .sound_group("glass")
        .tag("glass")
        .cutout()
        .cull_same()
        .hardness(0.3)
        .drops_nothing()
        .tab(&crate::tabs::BUILDING);
    for (name, col) in [("red", [0u8, 12, 12]), ("green", [12, 0, 12]), ("blue", [12, 12, 0])] {
        r.block(Block::generated(format!("{name}_stained_glass")))
            .signal_blocker()
            .sound_group("glass")
            .tag(&format!("stained_glass/{name}"))
            .texture(&format!("glass_{name}"))
            .translucent()
            .light_filter(col[0], col[1], col[2])
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
        .west("crafting_table_side")
        .east("crafting_table_side")
        .bottom("crafting_table_bottom")
        .top("crafting_table_top")
        .north("crafting_table_front")
        .south("crafting_table_front")
        .hardness(2.5)
        .mined_by("axe", tool_tier::HAND)
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
    // On walls only, up to four in one block (upstream's, Weft sync 11-22).
    r.block(&LADDER)
        .sound_group("wood")
        .on_sides(Facings::HORIZONTAL)
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
        .west("bricks_side_1")
        .east("bricks_side_4")
        .bottom("bricks_bottom")
        .top("bricks_top")
        .north("bricks_side_3")
        .south("bricks_side_2")
        .sound_group("stone")
        .hardness(2.0)
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::BUILDING);
    r.block(&STONE_BRICKS)
        .sound_group("stone")
        .west("stonebrick_side_1")
        .east("stonebrick_side_4")
        .bottom("stonebrick_bottom")
        .top("stonebrick_top")
        .north("stonebrick_side_3")
        .south("stonebrick_side_2")
        .hardness(1.5)
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::BUILDING);
    r.block(&IRON_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::STONE)
        .tag("ore_blocks/iron")
        .tab(&crate::tabs::BUILDING);
    r.block(&GOLD_BLOCK)
        .sound_group("stone")
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::STONE)
        .tag("ore_blocks/gold")
        .tab(&crate::tabs::BUILDING);
    r.block(&DIAMOND_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::IRON)
        .tag("ore_blocks/diamond")
        .tab(&crate::tabs::BUILDING);
    r.block(&COAL_BLOCK)
        .sound_group("stone")
        .hardness(5.0)
        .mined_by("pickaxe", 0)
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
        .texture("dirt")
        .top("podzol")
        .bottom("dirt")
        .hardness(0.5)
        .mined_by("shovel", tool_tier::HAND)
        .tag("dirt")
        .tab(&crate::tabs::NATURAL);
    // The forest floor (`vitals::layers`): soil like dirt, so plants grow on it.
    for (b, top, side, bottom, sound) in [
        (&LEAF_LITTER, "leaf_litter_top", "leaf_litter_side", "dirt", "grass"),
        (&DEEP_LEAF_LITTER, "leaf_litter_deep", "leaf_litter_side", "dirt", "grass"),
        (&MOSS_BLOCK, "moss_block_top", "moss_block_side", "dirt", "grass"),
        (&COARSE_DIRT, "coarse_dirt", "coarse_dirt", "coarse_dirt", "gravel"),
    ] {
        r.block(b)
            .sound_group(sound)
            .texture(side)
            .top(top)
            .bottom(bottom)
            .hardness(0.5)
            .mined_by("shovel", tool_tier::HAND)
            .tag("dirt")
            .tab(&crate::tabs::NATURAL);
    }
    // The shrubs' leaves (`vitals::layers`), drawn in their own colours: not tinted.
    for (b, tex) in [(&WILLOW_LEAVES, "leaves_willow"), (&JUNIPER_LEAVES, "leaves_juniper"), (&CREOSOTE_LEAVES, "leaves_creosote")] {
        let builder = r.block(b).sound_group("grass").texture(tex);
        bloom(builder, b.name(), leak(format!("{tex}_bloom")), tex, None)
            .hardness(0.2)
            .cutout()
            .cull_same()
            .light_opacity(1)
            .tag("leaves")
            .signal_blocker()
            .mined_by("shears", tool_tier::HAND)
            .tab(&crate::tabs::NATURAL);
        r.loot(b)
            .pool(Pool::new().when(Cond::chance(0.05)).item("stick"));
    }
    // Grass two blocks tall: the lower half on soil, the upper on the lower; breaking the lower
    // takes the upper with it (it loses its support).
    r.tag("tall_grass_soil", &["#plant_soil", "tall_grass_double"]);
    r.block(&TALL_GRASS_DOUBLE)
        .sound_group("grass")
        .plant()
        .needs_below("#tall_grass_soil")
        .prop("half", ["lower", "upper"])
        // The tint goes on the state's own texture: upstream's `tint_kind` colours only the faces
        // set so far, and a state's texture replaces the block's.
        .states(|p, s| {
            s.texture(if p.get("half") == "upper" { "tall_grass_upper" } else { "tall_grass_lower" })
                .tint_kind(&tints::GRASS)
        })
        .no_item();
    // Big-leaved plants two blocks tall, in their own colours: the upper half on the lower, which
    // takes it when broken.
    r.tag("big_leaf_soil", &["#plant_soil", "banana", "philodendron"]);
    for (b, name) in [(&BANANA, "banana"), (&PHILODENDRON, "philodendron")] {
        r.block(b)
            .sound_group("grass")
            .plant()
            .needs_below("#big_leaf_soil")
            .prop("half", ["lower", "upper"])
            .states(move |p, s| s.texture(&format!("{name}_{}", p.get("half"))))
            .no_item();
        let table = r.loot(b).pool(
            Pool::new()
                .when(Cond::chance(vitals::clothing::FIBRE_CHANCE))
                .item("plant_fibre"),
        );
        // A wild banana now and then carries a hand of fruit: its lower half gives it, once
        // a plant (`vitals::wild::BANANA_FRUIT`).
        if name == "banana" {
            table.pool(
                Pool::new()
                    .when(Cond::state("half=lower"))
                    .when(Cond::chance(vitals::wild::BANANA_FRUIT))
                    .item("bananas"),
            );
        }
    }
    r.loot(&TALL_GRASS_DOUBLE)
        .pool(
            Pool::new()
                .when(Cond::chance(vitals::crops::WHEAT_SEED_CHANCE))
                .item("wheat_seeds"),
        )
        .pool(
            Pool::new()
                .when(Cond::chance(vitals::clothing::FIBRE_CHANCE))
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
            .when(Cond::chance(vitals::clothing::FIBRE_CHANCE))
            .item("plant_fibre"),
    );
    r.block(&DEAD_BUSH)
        .sound_group("grass")
        .texture("deadbush")
        .plant()
        .needs_below("#sand")
        .tab(&crate::tabs::NATURAL);
    r.loot(&DEAD_BUSH)
        .pool(Pool::new().when(Cond::chance(0.7)).item("stick"))
        .pool(Pool::new().when(Cond::chance(0.35)).item("stick"));
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
            s.texture("compost_bin_side")
                .top(top)
                .bottom("compost_bin_side")
        })
        .hardness(0.6)
        .mined_by("axe", tool_tier::HAND)
        .grows(vitals::soil::compost_secs(), 0)
        .on_grow(crate::farming::rot)
        .on_use(crate::farming::bin)
        .tab(&crate::tabs::FUNCTIONAL);
    r.block(&BOOKSHELF)
        .sound_group("wood")
        .texture("bookshelf")
        .top("planks_oak")
        .bottom("planks_oak")
        .hardness(1.5)
        .mined_by("axe", tool_tier::HAND)
        .set(&FLAMMABLE, 30)
        .tab(&crate::tabs::BUILDING);
    r.block(&HAY_BALE)
        .sound_group("grass")
        .texture("hay_bale_side")
        .top("hay_bale_top")
        .bottom("hay_bale_top")
        .hardness(0.5)
        .set(&FLAMMABLE, 60)
        .tab(&crate::tabs::BUILDING);

    // What may grow or stand where, from the common `dirt` and `sand` tags like modern vanilla:
    // a mod's sand tagged `sand` gets cactus and sugar cane, in the world and from players.
    // (`plant_soil` from `#dirt` is set at the top of `register`.)
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
    upstream_blocks(r);
}

pub use weft_sdk::shapes::{TORCH_FACINGS, stick_faces, torch_state};

/// Sugar cane and cacti grow a block on top, up to three high.
fn grow_column(e: &mut BlockEvent, me: &'static Block) {
    let w = e.world();
    let p = e.pos();
    let above = [p.x, p.y + 1, p.z];
    if w.block(above) != 0 {
        return;
    }
    let mut height = 1;
    while height < 3 && me.is(w.block([p.x, p.y - height, p.z])) {
        height += 1;
    }
    if height < 3 {
        w.set_block(above, me.id());
    }
}

/// A thin branch of `wood` (the branch-block study, `base_api::trees::TREE_BRANCH`): a post 4 or 8
/// px thick in the block's middle with an arm to each side it joins (`n` `s` `e` `w` `u` `d`, set by
/// worldgen), in the log's bark cropped to the box, as Dynamic Trees draws its branches. Collides
/// as its boxes; an axe takes it; a stick for it.
/// The woods Art has drawn young bark for (`branch_<wood>`, `_across`, `_end`; all nine, Ronald:
/// "Approve all"); another wood wears its log's until then.
const BRANCH_BARK: &[&str] = &["oak", "birch", "acacia", "mesquite", "dogwood", "cherry", "ipe", "andiroba", "matamata", "brazil_nut"];

/// A thin branch of `wood` (`base:<wood>_branch`, named on its log as `TREE_BRANCH`): a post 4 or 8
/// px thick in the block's middle with an arm to each side it joins (`n` `s` `e` `w` `u` `d`, set by
/// worldgen), in Art's young bark (`branch_<wood>`: grain up; `_across`: along a level arm; `_end`:
/// its rings), the log's own until Art has drawn it. Faces an arm covers aren't drawn. Collides as
/// its boxes; an axe takes it; a stick for it, as Dynamic Trees gives sticks for thin wood.
fn branch(r: &mut Registry, wood: &str) {
    let name = format!("{wood}_branch");
    let art = BRANCH_BARK.contains(&wood);
    let bark = if art { format!("branch_{wood}") } else { format!("log_{wood}") };
    let across = if art { format!("branch_{wood}_across") } else { bark.clone() };
    let end = if art { format!("branch_{wood}_end") } else { format!("log_{wood}_top") };
    r.edit(Block::generated(format!("{wood}_log")))
        .set_text(&crate::worldgen::TREE_BRANCH, &format!("base:{name}"));
    let sides = ["n", "s", "e", "w", "u", "d"];
    let mut b = r
        .block(Block::generated(name))
        .sound_group("wood")
        .hardness(0.8)
        .mined_by("axe", tool_tier::HAND)
        .tag("logs")
        .tag("branches")
        .tag(&format!("logs/{wood}"))
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 100)
        .prop("size", ["4", "8"]);
    for side in sides {
        b = b.prop(side, ["0", "1"]);
    }
    b.states(move |p, s| {
        let h = if p.get("size") == "4" { 2.0 } else { 4.0 };
        let (lo, hi) = (8.0 - h, 8.0 + h);
        let on = |side: &str| p.get(side) == "1";
        // A box's faces in -X, +X, -Y, +Y, -Z, +Z order, the bark cropped to the box's own extent,
        // its grain along the box (`along`: 0 x, 1 y, 2 z), the rings across it; `hide` the faces
        // an arm or the post covers (they'd only add to what the GPU draws: the bench's +8%).
        let faces = |a: [f32; 3], c: [f32; 3], along: usize, hide: [bool; 6]| -> [(&str, [f32; 4]); 6] {
            let zy = [a[2], 16.0 - c[1], c[2], 16.0 - a[1]];
            let xz = [a[0], a[2], c[0], c[2]];
            let xy = [a[0], 16.0 - c[1], c[0], 16.0 - a[1]];
            let (x, y, z): (&str, &str, &str) = match along {
                0 => (&end, &across, &across),
                2 => (&across, &bark, &end),
                _ => (&bark, &end, &bark),
            };
            let f = [(x, zy), (x, zy), (y, xz), (y, xz), (z, xy), (z, xy)];
            std::array::from_fn(|i| if hide[i] { (f[i].0, abi::UV_NO_FACE) } else { f[i] })
        };
        let along = if on("u") || on("d") || !(on("e") || on("w") || on("n") || on("s")) {
            1
        } else if on("e") || on("w") {
            0
        } else {
            2
        };
        // The post: no face where an arm leaves it (the arm's end covers it exactly).
        let core = [on("w"), on("e"), on("d"), on("u"), on("n"), on("s")];
        let mut s = s.model_box_faces([lo, lo, lo], [hi, hi, hi], faces([lo, lo, lo], [hi, hi, hi], along, core));
        for (side, i) in [("w", 0), ("e", 1), ("d", 2), ("u", 3), ("n", 4), ("s", 5)] {
            if !on(side) {
                continue;
            }
            let (a, c, along) = match side {
                "n" => ([lo, lo, 0.0], [hi, hi, lo], 2),
                "s" => ([lo, lo, hi], [hi, hi, 16.0], 2),
                "w" => ([0.0, lo, lo], [lo, hi, hi], 0),
                "e" => ([hi, lo, lo], [16.0, hi, hi], 0),
                "d" => ([lo, 0.0, lo], [hi, lo, hi], 1),
                _ => ([lo, hi, lo], [hi, 16.0, hi], 1),
            };
            // An arm's end at the post is hidden; its end at the block's face stays (a thinner
            // neighbour's arm would leave a hole round it).
            let mut hide = [false; 6];
            hide[i ^ 1] = true;
            s = s.model_box_faces(a, c, faces(a, c, along, hide));
        }
        s
    })
    .drops("stick", 1)
    .tab(&crate::tabs::NATURAL);
}

/// Whether a dung pat lies within `vitals::soil::PAT_REACH` blocks of `pos` (a block up or down).
pub fn pat_near(w: &World, [x, y, z]: [i32; 3]) -> bool {
    let r = vitals::soil::PAT_REACH;
    (-1..=1).any(|dy| (-r..=r).any(|dz| (-r..=r).any(|dx| DUNG_PAT.is(w.block([x + dx, y + dy, z + dz])))))
}

// Upstream's new blocks (Weft sync 11-22), all here so a piece is cut by deleting its lines (and
// this function's call in `register` for all of them): the wood blocks with bark on every face,
// hollow logs, the berry bushes, the new grasses and flowers, moss and the forest mushrooms.
// Their statics are in `base_api::blocks`. Upstream's clover, fern and coarse dirt are left out
// (ours registers those), and so are its gravel flint, hoe tilling, tinted glass and wool and
// grass overlay (ours stay).
/// Upstream's "cut with shears" (`Cond::tool("#tools/shears")`), for its bushes and grasses
/// below. Our game has no shears, and a condition naming a tag nothing carries stops the game at
/// load, so it never holds: they drop what they drop unsheared (Weft sync 11-22, for Ronald: give
/// a tool of ours the `tools/shears` tag, or make shears, and put upstream's condition back).
fn sheared() -> Cond {
    Cond::chance(0.0)
}

fn upstream_blocks(r: &mut Registry) {
    wood_block(r, &OAK_WOOD, "oak");
    wood_block(r, &BIRCH_WOOD, "birch");
    wood_block(r, &SPRUCE_WOOD, "spruce");
    // Hollow logs only of the three woods whose inside upstream draws (`<wood>_log_inner`).
    r.tag("hollow_log_sources", &["oak_log", "birch_log", "spruce_log"]);
    r.shapes_of("hollow_log", "#hollow_log_sources", "_hollow")
        .sound_group("wood")
        .model_file("hollow_log")
        .model_texture("inside", "{source.mod}:block/{source.name}_inner")
        .axis()
        .signal_blocker()
        .shares_cell_with("#water")
        .liquid_walls(&[Dir::North, Dir::South, Dir::East, Dir::West])
        .outline([0.0; 3], [16.0; 3])
        .outline([2.0, 0.0, 2.0], [14.0, 16.0, 14.0])
        .tag("hollow_log")
        .tab(&crate::tabs::NATURAL);
    r.block(&BUSH_STEM)
        .sound_group("grass")
        .hardness(0.2)
        .cutout()
        .light_opacity(1)
        .signal_blocker()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    for tag in ["#materials/wood", "#planks", "#wood", "#logs"] {
        r.set(tag, &SOUND_ABSORPTION, 0.90);
    }
    // Upstream's bush leaves, untinted, a sapling a time in 20 unless sheared.
    r.block(&BUSH_LEAVES)
        .sound_group("grass")
        .texture("raspberry_leaves")
        .hardness(0.2)
        .cutout()
        .light_opacity(1)
        .tag("leaves")
        .signal_blocker()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL);
    let shears = sheared();
    r.loot(&BUSH_LEAVES)
        .pool(Pool::new().when(shears.clone()).item(BUSH_LEAVES.name()))
        .pool(
            Pool::new()
                .when(!shears.clone())
                .when(Cond::chance(0.05))
                .item("sapling"),
        );
    berry_leaves(r, &RASPBERRY_LEAVES, "raspberry_leaves", "raspberry_leaves_berried", &crate::items::RASPBERRY);
    berry_leaves(r, &SNOWBERRY_LEAVES, "raspberry_leaves", "snowberry_leaves_berried", &crate::items::SNOWBERRY);
    r.block(&SHORT_GRASS)
        .sound_group("grass")
        .texture("short_grass")
        .plant()
        .random_offset(0.25)
        .random_height_scale(0.85, 1.15)
        .random_yaw(0.0, 90.0)
        .replaceable()
        .needs_below("#plant_soil")
        .tint_kind(&tints::GRASS)
        .tab(&crate::tabs::NATURAL);
    r.block(&TALL_TALL_GRASS)
        .sound_group("grass")
        .texture("tall_tallgrass_bottom")
        .tall_plant("tall_tallgrass_bottom", "tall_tallgrass_top")
        .replaceable()
        // .two_part("half", ["lower", "upper"], |_| Dir::Up)
        // .state_where("half=upper", |s| s.texture("tall_tallgrass_top"))
        // .state_where("half=lower", |s| s.needs_below("#plant_6oil"))
        .tint_kind(&tints::GRASS)
        .tab(&crate::tabs::NATURAL);
    r.block(&BLUEGRASS)
        .sound_group("grass")
        .plant()
        .random_offset(0.25)
        .random_height_scale(0.85, 1.15)
        .random_yaw(0.0, 90.0)
        .replaceable()
        .needs_below("#plant_soil")
        .tint_kind(&tints::GRASS)
        .tab(&crate::tabs::NATURAL);
    for color in ["orange", "pink", "purple", "red", "white", "yellow"] {
        let name = format!("tulip_{color}");
        plant(r, Block::generated(name.clone()), name.as_str());
    }
    r.block(&INULA)
        .sound_group("grass")
        .tall_plant("inula_bottom", "inula_top")
        .replaceable()
        .needs_below("#plant_soil")
        .item_texture("inula")
        .tab(&crate::tabs::NATURAL);
    r.loot(&INULA)
        .pool(Pool::new().when(sheared()).item(INULA.name()));
    r.block(&VERBASCUM)
        .sound_group("grass")
        .tall_plant("verbascum_bottom", "verbascum_top")
        .replaceable()
        .needs_below("#plant_soil")
        .item_texture("verbascum")
        .tab(&crate::tabs::NATURAL);
    r.loot(&VERBASCUM)
        .pool(Pool::new().when(sheared()).item(VERBASCUM.name()));
    r.block(&LAVENDER)
        .sound_group("grass")
        .tall_plant("lavender_bottom", "lavender_top")
        .replaceable()
        .needs_below("#plant_soil")
        .item_texture("lavender")
        .tab(&crate::tabs::NATURAL);
    r.loot(&LAVENDER)
        .pool(Pool::new().when(sheared()).item(LAVENDER.name()));
    r.block(&MOSS)
        .grid_placement()
        .on_sides(Facings::ALL)
        .sound_group("grass")
        .shares_cell_with("#water")
        .hardness(0.1)
        .signal_blocker()
        .tab(&crate::tabs::NATURAL);
    r.block(&PORCINI_MUSHROOM)
        .sound_group("grass")
        .shares_cell_with("#water")
        .hardness(0.1)
        .needs_below("")
        .plant()
        .texture("porcini_mushroom")
        .drops("porcini", 1)
        .tab(&crate::tabs::NATURAL);
    r.block(&CHANTERELLE_MUSHROOM)
        .sound_group("grass")
        .shares_cell_with("#water")
        .hardness(0.1)
        .needs_below("")
        .plant()
        .texture("chanterelle_mushroom")
        .drops("chanterelle", 1)
        .tab(&crate::tabs::NATURAL);
    r.block(&TOADSTOOL_MUSHROOM)
        .sound_group("grass")
        .shares_cell_with("#water")
        .hardness(0.1)
        .needs_below("")
        .plant()
        .texture("toadstool_mushroom")
        .drops("toadstool", 1)
        .tab(&crate::tabs::NATURAL);
    let top_uv = [0.1, 0.1, 15.9, 7.9];
    let bottom_uv = [0.1, 8.1, 15.9, 15.9];
    fn flip_v(uv: [f32; 4]) -> [f32; 4] {
        [uv[0], uv[3], uv[2], uv[1]]
    }
    fn shelf_faces(top: [f32; 4], bottom: [f32; 4]) -> [(&'static str, [f32; 4]); 6] {
        let tex = "dryads_saddle_mushroom";
        let none = (tex, UV_NO_FACE);
        [none, none, (tex, bottom), (tex, top), none, none]
    }
    r.block(&DRYADS_SADDLE_MUSHROOM)
        .grid_placement()
        .sound_group("wood")
        .shares_cell_with("#water")
        .hardness(0.1)
        .texture("dryads_saddle_mushroom")
        .facing(Facings::HORIZONTAL, Toward::Face)
        .states(|_p, s| s.needs(Dir::South, ""))
        .model_box_faces(
            [0.0, 12.0, 8.0],
            [16.0, 12.2, 16.0],
            shelf_faces(top_uv, flip_v(top_uv)), // upper: top half both sides
        )
        .tilted([8.0, 12.0, 16.0], Dir::East, -18.0)
        .model_box_faces(
            [0.0, 7.0, 8.0],
            [16.0, 7.2, 16.0],
            shelf_faces(bottom_uv, flip_v(bottom_uv)), // lower: bottom half both sides
        )
        .tilted([8.0, 7.0, 16.0], Dir::East, -24.0)
        .not_solid()
        .hitbox([0.0, 6.0, 8.0], [16.0, 13.0, 16.0])
        .drops("dryads_saddle", 1)
        .tab(&crate::tabs::NATURAL);
    // Sheared, the grasses drop themselves; else now and then a wheat seed.
    for (grass, chance) in [(&SHORT_GRASS, 0.125), (&BLUEGRASS, 0.125), (&TALL_TALL_GRASS, 0.25)] {
        r.loot(grass)
            .pool(Pool::new().when(shears.clone()).item(grass.name()))
            .pool(
                Pool::new()
                    .when(!shears.clone())
                    .when(Cond::chance(chance))
                    .item(crate::items::WHEAT_SEEDS.name()),
            );
    }
}

/// Upstream's wood block: a log's bark on all six faces, turned to face the player.
fn wood_block(r: &mut Registry, handle: &'static Block, wood: &str) {
    r.block(handle)
        .sound_group("wood")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .tab(&crate::tabs::NATURAL)
        .tag(&format!("wood/{wood}"))
        .tag("wood")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .texture(&format!("log_{wood}"));
}

fn berry_leaves(r: &mut Registry, handle: &'static Block, texture: &'static str, texture_berried: &'static str, item: &'static Item) {
    r.block(handle)
        .sound_group("grass")
        .prop("berried", ["false", "true"])
        .hardness(0.2)
        .cutout()
        .light_opacity(1)
        .tag("leaves")
        .signal_blocker()
        .mined_by("shears", tool_tier::HAND)
        .tab(&crate::tabs::NATURAL)
        .texture(texture_berried)
        .states(|p, s| s.texture(if p.is("berried", "true") { texture_berried } else { texture }))
        .on_use(move |u| pick_berries(u, item));
    r.loot(handle)
        .pool(Pool::new().when(sheared()).item(handle.name()))
        .pool(
            Pool::new()
                .when(Cond::state("berried=true"))
                .items(item.name(), 1, 1),
        );
}

fn pick_berries(u: &mut Use, berry: &'static Item) -> bool {
    let w = u.world();
    let p = u.pos();
    if w.prop(p, "berried") != Some("true") {
        return false;
    }
    if !w.prop_set(p, "berried", "false") {
        return false;
    }
    w.spawn_item(block_center(p), berry.stack(1));
    true
}

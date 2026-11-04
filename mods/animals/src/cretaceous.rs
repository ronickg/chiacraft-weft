//! The Cretaceous (`notes/ANIMALS.md`, "Places, food and sedation"): a dimension for the
//! dinosaurs and wyverns, its land after the places they lived. Red dunes (Velociraptor: the
//! Djadokhta Formation, Mongolia, wind-blown dunes in a warm semi-arid climate), dry scrubland, a
//! fern floodplain (Utahraptor: the Cedar Mountain Formation, Utah, floodplains with soils that
//! swelled and cracked as wet and dry seasons came and went), conifer forests over ferns and
//! horsetails, high crags (wyverns), and the sea. Warm everywhere: no ice then. Mostly real: grass
//! as in the overworld (it was rare then), ferns and horsetails over it, conifers; no flowers
//! (where base's decoration puts one, a fern) and no cacti.
//!
//! Getting there: a gateway (`gateway.rs`).

use base_api::CLIMATE;
use base_api::{CLOUDS, MOON, STARS, SUN};
use weft_sdk::terrain::BiomeRule;
use weft_sdk::*;

pub static CRETACEOUS: Dimension = Dimension::named("cretaceous");

pub static RED_SAND: Block = Block::named("red_sand");
pub static RED_SANDSTONE: Block = Block::named("red_sandstone");
pub static FERN: Block = Block::named("fern");
pub static HORSETAIL: Block = Block::named("horsetail");

pub static DUNES: Biome = Biome::named("red_dunes");
pub static SCRUBLAND: Biome = Biome::named("scrubland");
pub static FLOODPLAIN: Biome = Biome::named("floodplain");
pub static FERN_FOREST: Biome = Biome::named("fern_forest");
pub static CRAGS: Biome = Biome::named("crags");

// Base's worldgen vocabulary, set on these biomes (`mods/base/src/worldgen.rs`).
static TREES: Component<f32> = Component::named("trees");
static GRASS_COVER: Component<f32> = Component::named("grass");
static TREE_KINDS: Component<BlockList> = Component::named("tree_kinds");
/// Extra trees where a forest stands thick, in stands with clearings between (base's worldgen).
static STAND_TREES: Component<f32> = Component::named("stand_trees");
/// Boulders, a heap of stone per 12 x 12 cell (base's worldgen).
static BOULDERS: Component<f32> = Component::named("boulders");
static FLOWERS: Component<BlockList> = Component::named("flowers");
/// On a log, the leaves of the tree base's worldgen grows from it; tags `cone_tree`, `flat_tree`,
/// `palm_tree` give its shape.
static TREE_LEAVES: Component<BlockRef> = Component::named("tree_leaves");
/// Share of the ground ferns and horsetails cover (this mod's decoration).
static FERNS: Component<f32> = Component::named("fern_cover");
static PLANT_SOIL: Tag = Tag::named("plant_soil");
static NATURAL: CreativeTab = CreativeTab::named("base:natural");
static BUILDING: CreativeTab = CreativeTab::named("base:building");

/// Sea level, as the overworld's.
const SEA: i32 = 63;
/// The floodplain: the low ground by the rivers, lakes and coasts, this high over the sea at most.
const FLOODPLAIN_RISE: i32 = 6;

pub fn init(r: &mut Registry) {
    r.block(&RED_SAND)
        .sound_group("base:block.sand")
        .hardness(0.5)
        .mineable("shovel")
        // Not `sand`: no cacti on it (they came 30 million years after).
        .tag("red_sand")
        .falling()
        .tab(&NATURAL);
    r.block(&RED_SANDSTONE)
        .sound_group("base:block.stone")
        .textures("red_sandstone_top", "red_sandstone_side", "red_sandstone_bottom")
        .hardness(0.8)
        .mineable("pickaxe")
        .needs_tool(0)
        .tab(&BUILDING);
    for plant in [&FERN, &HORSETAIL] {
        r.block(plant)
            .sound_group("base:block.grass")
            .plant()
            .needs_below("#plant_soil")
            .tab(&NATURAL);
    }
    // The Cretaceous's other wild plants (`vitals::wild`; base's decoration puts them, as it does
    // any mod's): cycads, early flowering shrubs, water lilies' kin floating on its ponds. They
    // stand as they are; a cycad gives, a time in two, its seed (poisonous to people), which
    // plants one.
    for p in vitals::wild::WILD
        .iter()
        .filter(|p| p.block.starts_with("animals:"))
    {
        let block = Block::generated(p.block.into());
        let name = p.block.split_once(':').map_or(p.block, |(_, n)| n);
        // A termite mound: hard-baked earth, a full block, and its crown (`<name>_upper`, which
        // the world puts on one in two, `wild::upper_of` in base); broken, the crown is a mound
        // block again.
        if p.form == vitals::wild::Form::Mound {
            let upper = Block::generated(format!("{name}_upper"));
            for (b, side) in [(block, format!("{name}_side")), (upper, format!("{name}_upper_side"))] {
                r.block(b)
                    .sound_group("base:block.gravel")
                    .textures(&format!("{name}_top"), &side, &format!("{name}_top"))
                    .hardness(0.9)
                    .mineable("shovel")
                    .tab(&NATURAL);
            }
            r.loot(upper).pool(Pool::new().entry(Entry::item(p.block)));
            continue;
        }
        // A rotten log lies on the ground: a soft, crumbling block, not a plant.
        if p.form == vitals::wild::Form::Log {
            r.block(block)
                .sound_group("base:block.wood")
                .textures(&format!("{name}_top"), name, &format!("{name}_top"))
                .hardness(1.0)
                .mineable("axe")
                .tab(&NATURAL);
            continue;
        }
        r.block(block)
            .sound_group("base:block.grass")
            .plant()
            .needs_below(if p.ground == vitals::wild::Ground::Water { "#water" } else { "#plant_soil" })
            .texture(name)
            .tab(&NATURAL);
        if let Some(seed) = p.gives {
            r.loot(block)
                .pool(Pool::new().entry(Entry::item(p.block)))
                .pool(Pool::new().when(Cond::Chance(0.5)).item(seed));
            r.item(Item::generated(seed.into()))
                .placed_block(block)
                .tab(&NATURAL);
        }
    }
    r.tag("plant_soil", &["red_sand"]);
    woods(r);

    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    // Wind-blown red dunes: Velociraptor's, and osmolskae's.
    r.biome(&DUNES)
        .tint(grass, vitals::foliage::muted(vitals::foliage::ENGINE_GRASS))
        .tint(foliage, vitals::foliage::muted(vitals::foliage::ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, vitals::foliage::muted(vitals::foliage::LEAVES))
        .tint(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(vitals::foliage::LEAVES))
        .set(&CLIMATE, base_api::climate("animals:red_dunes"))
        .humidity(..-0.12)
        .surface("red_sand", "red_sand", 4)
        .under("red_sandstone", 2)
        .tag("dry")
        .tag("hot")
        // A rare scale-leaved conifer, drought-hardy, on the dunes.
        .set(&TREES, 0.003)
        .set_text(&TREE_KINDS, "animals:scale_conifer_log")
        .set(&GRASS_COVER, vitals::forage::grass_cover("animals:red_dunes").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set(&FERNS, 0.004)
        .set_text(&FLOWERS, "animals:fern");
    // Dry, open land with a few conifers and ferns: between the dunes and the wet.
    r.biome(&SCRUBLAND)
        .set(&CLIMATE, base_api::climate("animals:scrubland"))
        .set(&BOULDERS, 0.08)
        .humidity(-0.12..0.05)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, vitals::foliage::muted(0xb5ad5a))
        .tint(foliage, vitals::foliage::muted(0x8f9a3c))
        .tint(&base_api::trees::EVERGREEN, vitals::foliage::muted(0x8f9a3c))
        .tint(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(0x8f9a3c))
        .tag("hot")
        .set(&TREES, 0.02)
        .set(&GRASS_COVER, vitals::forage::grass_cover("animals:scrubland").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "animals:scale_conifer_log:3 animals:araucaria_log:1")
        .set(&FERNS, 0.06)
        .set_text(&FLOWERS, "animals:fern");
    // Low, wet ground by the rivers: ferns and horsetails, scattered conifers (Utahraptor's).
    r.biome(&FLOODPLAIN)
        .set(&CLIMATE, base_api::climate("animals:floodplain"))
        .humidity(0.05..)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, vitals::foliage::muted(0x7fa64a))
        .tint(foliage, vitals::foliage::muted(0x5d8f35))
        .tint(&base_api::trees::EVERGREEN, vitals::foliage::muted(0x5d8f35))
        .tint(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(0x5d8f35))
        .tint(water, 0x4a6e5a)
        .tag("wet")
        .set(&TREES, 0.04)
        .set(&STAND_TREES, 0.1)
        .set(&GRASS_COVER, vitals::forage::grass_cover("animals:floodplain").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "animals:swamp_cypress_log:3 animals:araucaria_log:2 animals:tempskya_log:2")
        .set(&FERNS, 0.35)
        .set_text(&FLOWERS, "animals:fern");
    // Conifers over a floor of ferns.
    r.biome(&FERN_FOREST)
        .set(&CLIMATE, base_api::climate("animals:fern_forest"))
        .humidity(0.05..)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, vitals::foliage::muted(0x6f9c40))
        .tint(foliage, vitals::foliage::muted(0x4f8a2c))
        .tint(&base_api::trees::EVERGREEN, vitals::foliage::muted(0x4f8a2c))
        .tint(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(0x4f8a2c))
        .tag("forest")
        .set(&TREES, 0.35)
        .set(&STAND_TREES, 0.18)
        .set(&GRASS_COVER, vitals::forage::grass_cover("animals:fern_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "animals:araucaria_log:3 animals:ginkgo_log:2 base:spruce_log:2 animals:tempskya_log:1")
        .set(&FERNS, 0.3)
        .set_text(&FLOWERS, "animals:fern");
    // High rock: the wyverns'.
    r.biome(&CRAGS)
        .set(&CLIMATE, base_api::climate("animals:crags"))
        .set(&BOULDERS, 0.3)
        .surface("base:grass", "base:dirt", 3)
        .bare_above(115)
        .tint(grass, vitals::foliage::muted(0x8aa864))
        .tint(foliage, vitals::foliage::muted(vitals::foliage::ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, vitals::foliage::muted(vitals::foliage::LEAVES))
        .tint(&base_api::trees::YELLOW_LEAVES, vitals::foliage::muted(vitals::foliage::LEAVES))
        .tag("mountain")
        .set(&TREES, 0.04)
        .set_text(&TREE_KINDS, "animals:araucaria_log:1 animals:scale_conifer_log:2")
        .set(&FERNS, 0.05)
        .set_text(&FLOWERS, "animals:fern");

    // Warm all year, as the Cretaceous's greenhouse was, with a wet and a dry season instead of
    // four; near the tropics, the days barely change.
    r.dimension(&CRETACEOUS)
        .sky_above(16)
        .day_length(24000)
        .seasons(vitals::calendar::YEAR_DAYS, base_api::calendars::WET_AND_DRY_SWING)
        .set(&base_api::SEASONS, base_api::calendars::WET_AND_DRY)
        .weather();
    r.sky_stars(&CRETACEOUS, &STARS).count(1500);
    r.sky_clouds(&CRETACEOUS, &CLOUDS).texture("base:clouds");
    r.sky_body(&CRETACEOUS, &SUN)
        .texture("base:sun")
        .additive()
        .size(33.0)
        .glow(0xFFE633);
    r.sky_body(&CRETACEOUS, &MOON)
        .texture("base:moon_phases")
        .additive()
        .frames(4, 2)
        .size(23.0)
        .phase(0.5);

    // The overworld's land (Ronald, 11-04: "all three", Terrain's fourth sheet): its continents,
    // hills, mountain ranges, rivers by flow, lakes and coasts (`base_api::land`), the same seed,
    // so a gateway opens onto the same land a hundred million years earlier: the rivers and lakes
    // a player knows, the Cretaceous's own biomes on them. Crags on the ranges, floodplain on the
    // low ground by the water, red dunes and scrub where it's dry, fern forest where it's wet.
    let mut t = r
        .terrain(&CRETACEOUS)
        .stone("base:stone")
        .base_height(SEA as f64)
        .continent_scale(base_api::land::CONTINENT_SCALE)
        .sea(SEA, "base:water");
    for (name, field) in base_api::land::rule_fields() {
        t = t.field(name, field);
    }
    for f in base_api::land::features() {
        t = t.feature(f);
    }
    // The warm seas' lagoons and reefs as the overworld's (`base_api::land::reef_part`); the
    // coral blocks are the overworld's decoration, the Cretaceous's reefs are bare (its own reef
    // builders, rudist clams, if anyone draws them).
    t.biome(
        BiomeRule::new("base:coral_reef")
            .height(SEA - 2..=SEA - 2)
            .temperature(base_api::land::REEF_WARM_RULE..)
            .continent(..base_api::land::REEF_COAST),
    )
    .biome(
        BiomeRule::new("base:lagoon")
            .height(SEA - base_api::land::REEF_LAGOON.1..=SEA - base_api::land::REEF_LAGOON.0)
            .temperature(base_api::land::REEF_WARM_RULE..)
            .continent(..base_api::land::REEF_COAST),
    )
    .biome(
        BiomeRule::new("base:ocean")
            .height(..=SEA - 2)
            .continent(..base_api::land::OCEAN_CONTINENT),
    )
    .biome(
        BiomeRule::new("base:beach")
            .height(..=SEA + 1)
            .continent(..0.1),
    )
    .biome(BiomeRule::new("crags").field(base_api::land::MOUNTAINS_FIELD, base_api::land::MOUNTAIN_BIOME..))
    .biome(
        BiomeRule::new("floodplain")
            .height(..=SEA + FLOODPLAIN_RISE)
            .humidity(-0.12..),
    )
    .biome(BiomeRule::new("red_dunes"))
    .biome(BiomeRule::new("scrubland"))
    .biome(BiomeRule::new("fern_forest"));

    r.decorate(ferns).dimension(&CRETACEOUS);
}

/// The Cretaceous's trees (`reports/ecology/Cretaceous food webs.md`), which base's worldgen grows
/// where a biome's `tree_kinds` names them: the araucarias, tall with a flat crown of rich, slow
/// foliage and big cones; ginkgos, round, deciduous, with fleshy seeds; the floodplain's swamp
/// cypresses and the dry land's scale-leaved conifers (Cheirolepidiaceae), both cones; and the tree
/// fern Tempskya, a trunk of roots under a crown of fronds. Wood, shape, what the leaves drop.
/// Their saplings grow by the place (`base_api::grow`, `vitals::plants`); Tempskya, which spread by
/// its roots, has none.
const WOODS: [(&str, Option<&str>, Option<&str>); 5] = [
    ("araucaria", Some("flat_tree"), Some("araucaria_cone")),
    ("ginkgo", None, Some("ginkgo_seed")),
    ("swamp_cypress", Some("cone_tree"), None),
    ("scale_conifer", Some("cone_tree"), None),
    ("tempskya", Some("palm_tree"), None),
];

fn woods(r: &mut Registry) {
    for (wood, shape, fruit) in WOODS {
        let side = format!("log_{wood}");
        let top = format!("log_{wood}_top");
        let log = r
            .block(Block::generated(format!("{wood}_log")))
            .set_text(&TREE_LEAVES, &format!("animals:{wood}_leaves"));
        let log = match shape {
            Some(tag) => log.tag(tag),
            None => log,
        };
        log.sound_group("base:block.wood")
            .hardness(2.0)
            .mineable("axe")
            .tag("logs")
            .tag(&format!("logs/{wood}"))
            .prop("axis", ["y", "x", "z"])
            // Lying after a felling, in alternating pieces (`felled`): chopped a piece at a time.
            .prop("felled", ["no", "a", "b"])
            .states(move |p, s| match p.get("axis") {
                "y" => s.textures(&top, &side, &top),
                "x" => s.faces([&top, &top, &side, &side, &side, &side]),
                _ => s.faces([&side, &side, &side, &side, &top, &top]),
            })
            .placement(|p, props| {
                props.set(
                    "axis",
                    match p.face() {
                        0 | 1 => "x",
                        4 | 5 => "z",
                        _ => "y",
                    },
                )
            })
            .tab(&NATURAL);
        let leaves = Block::generated(format!("{wood}_leaves"));
        r.block(leaves)
            .sound_group("base:block.grass")
            .texture(&format!("leaves_{wood}"))
            .hardness(0.2)
            .cutout()
            // Faces between two leaves of one kind aren't drawn, as base's (`blocks.rs`).
            .cull_same()
            .opacity(1)
            .tint(base_api::trees::leaf_tint(vitals::wild::turn(&format!("animals:{wood}_leaves"))))
            .tag("leaves")
            .mineable("shears")
            .tab(&NATURAL);
        let plant: &'static str = Box::leak(format!("animals:{wood}_sapling").into_boxed_str());
        let has_sapling = vitals::plants::of(plant).is_some();
        let table = r.loot(leaves);
        let table = if has_sapling { table.pool(Pool::new().when(Cond::Chance(0.05)).item(plant)) } else { table };
        drop(match fruit {
            Some(fruit) => table.pool(Pool::new().when(Cond::Chance(0.05)).item(fruit)),
            None => table,
        });
        if let Some(fruit) = fruit {
            r.item(Item::generated(fruit.into())).tab(&NATURAL);
        }
        if has_sapling {
            let log: &'static str = Box::leak(format!("animals:{wood}_log").into_boxed_str());
            r.block(Block::generated(plant.into()))
                .sound_group("base:block.grass")
                .texture(&format!("sapling_{wood}"))
                .plant()
                .needs_below("#plant_soil")
                .growth(base_api::grow::step_secs(plant, 1, 470.0), 9)
                .on_grow(move |e| match base_api::grow::step(e, plant, false) {
                    base_api::grow::Step::Grow => {
                        if let Some(id) = Block::lookup(log) {
                            base_api::trees::grow_tree(e.world(), e.pos(), id);
                        }
                    }
                    // A frost kills a sapling of the warm Cretaceous.
                    base_api::grow::Step::Frost => {
                        e.world().set(e.pos(), 0);
                    }
                    base_api::grow::Step::Rest => {}
                })
                .tab(&NATURAL);
        }
    }
}

/// Ferns and horsetails over the ground, as many as the biome's `FERNS`; horsetails where it is
/// wet (low ground near the water).
fn ferns(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let [ox, oy, oz] = g.origin();
    for z in 0..16usize {
        for x in 0..16usize {
            let col = &cols[z * 16 + x];
            let y = col.height + 1 - oy;
            if !(1..16).contains(&y) || col.height < SEA {
                continue;
            }
            let y = y as usize;
            let Some(cover) = FERNS.on_biome(col.biome) else { continue };
            if g.get(x, y, z) != 0 || !PLANT_SOIL.on_block(g.get(x, y - 1, z)) {
                continue;
            }
            let h = hash(seed ^ 0xFE2, (ox + x as i32) as i64, (oz + z as i32) as i64);
            if (h & 0xFFFF) as f32 / 65536.0 >= cover {
                continue;
            }
            let wet = col.height <= SEA + 3;
            let plant = if wet && (h >> 20) % 3 == 0 { &HORSETAIL } else { &FERN };
            g.set(x, y, z, plant.id());
        }
    }
}

fn hash(seed: u64, x: i64, z: i64) -> u64 {
    let mut h = seed ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (z as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^ (h >> 33)
}

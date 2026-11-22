//! Base's worldgen as data: biomes, the overworld's terrain, ores, trees (features the engine
//! places in every dimension whose biomes ask for them) and plants (a decoration).
//!
//! Ours in upstream's layout (Weft sync 11-22): the numbers in `consts`, what biomes grow and the
//! biomes in `statics`, the trees' and boulders' plans in `tree_algorythms`, the decoration in
//! `decor`, small helpers in `util`. Upstream's own worldgen (its tree kinds, meadows, lavender
//! fields and hills) is not ours: `tree_kinds.rs` is upstream's and not compiled.

pub mod consts;
mod decor;
pub mod statics;
mod tree_algorythms;
mod util;

use base_api::CLIMATE;
use base_api::land;
use vitals::foliage::{ENGINE_FOLIAGE, ENGINE_GRASS, LEAVES as LEAF_TINT, muted};
use vitals::wild;
use weft_sdk::*;

use crate::blocks::*;

use weft_sdk::noise::*;
use weft_sdk::terrain::*;

pub use consts::*;
pub use statics::*;
// Ours, not `crate::blocks::MUSHROOMS` (upstream's mushroom list).
use statics::MUSHROOMS;
use decor::*;
use tree_algorythms::*;
use util::*;

// The trees' vocabulary and shapes are `base_api::trees`, so any mod's saplings grow them.
pub use base_api::trees::{TREE_BRANCH, TREE_LEAVES, grow_tree};
use base_api::trees::{tree_blocks, wild_tree_at};

pub fn register(r: &mut Registry) {
    // Simple trees for now (`base_api::trees::TREE_STYLE`; Ronald, 11-05); Minecraft's own for
    // like-for-like looks.
    r.config(&base_api::trees::TREE_STYLE)
        .choice(0, &["simple", "branched", "minecraft"])
        .world();
    // Tags say what a biome is (spawning, decoration and mods ask these, not names).
    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    r.biome(&PLAINS)
        .tint(grass, muted(ENGINE_GRASS))
        .tint(foliage, muted(ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, muted(LEAF_TINT))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(LEAF_TINT))
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:plains"))
        .set(&BOULDERS, 0.05)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tag("spawn_biome")
        .set(&TREES, 0.025)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:plains").unwrap_or(vitals::forage::GRASS_COVER_ELSE));
    r.biome(&FOREST)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:forest"))
        .set(&BOULDERS, 0.12)
        .humidity(land::FOREST_HUMIDITY..)
        .surface("grass", "dirt", 3)
        .tint(grass, muted(0x79c05a))
        .tint(foliage, muted(0x59ae30))
        .tint(&base_api::trees::EVERGREEN, muted(0x59ae30))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x59ae30))
        .tag("forest")
        .tag("animals")
        // A site in a 5 x 5 cell at 0.55 (220 a hectare, Research round 7's 170-244) and more in
        // the stands: the stand's layers' crowns (`vitals::canopy`) are Minecraft-sized, and at 0.4
        // the forest stood two fifths open (forest_canopy, 11-06).
        .set(&TREES, 0.55)
        .set(&STAND_TREES, 0.12)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:birch_log:1 base:oak_log:4");
    r.biome(&BIRCH_FOREST)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:birch_forest"))
        .set(&BOULDERS, 0.1)
        .temperature(..land::BIRCH.0)
        .humidity(land::BIRCH.1..)
        .surface("grass", "dirt", 3)
        .tint(grass, muted(0x88bb67))
        .tint(foliage, muted(0x6ba941))
        .tint(&base_api::trees::EVERGREEN, muted(0x6ba941))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x6ba941))
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.55)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:birch_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set(&STAND_TREES, 0.15)
        .set_text(&TREE_KINDS, "base:birch_log");
    r.biome(&TAIGA)
        .set(&CLIMATE, base_api::climate("base:taiga"))
        .set(&BOULDERS, 0.25)
        .temperature(..land::TAIGA_BELOW)
        .surface("podzol", "dirt", 3)
        .tint(grass, muted(0x86b783))
        .tint(foliage, muted(0x68a464))
        .tint(&base_api::trees::EVERGREEN, muted(0x68a464))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x68a464))
        .tint(water, 0x5c6c96)
        .tag("forest")
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.55)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:taiga").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set(&STAND_TREES, 0.12)
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&SNOWY)
        .set(&CLIMATE, base_api::climate("base:snowy_plains"))
        .set(&BOULDERS, 0.2)
        .snows()
        .temperature(..land::SNOWY_BELOW)
        .surface("snowy_grass", "dirt", 3)
        .frozen("ice")
        .tint(grass, muted(0x80b497))
        .tint(foliage, muted(0x60a17b))
        .tint(&base_api::trees::EVERGREEN, muted(0x60a17b))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x60a17b))
        .tint(water, 0x585c8c)
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.03)
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&DESERT)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:desert"))
        .set(&BOULDERS, 0.04)
        .temperature(land::DESERT.0..)
        .humidity(..land::DESERT.1)
        .surface("sand", "sand", 3)
        .under("sandstone", 1)
        .tint(grass, muted(0xbfb755))
        .tint(foliage, muted(0xaea42a))
        .tint(&base_api::trees::EVERGREEN, muted(0xaea42a))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0xaea42a))
        .tag("dry")
        .tag("hot")
        // Mesquite along the washes.
        .set(&TREES, 0.008)
        .set_text(&TREE_KINDS, "base:mesquite_log");
    // Hot grassland with few trees, between the desert and the forests (wet and dry seasons).
    r.biome(&SAVANNA)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:savanna"))
        .set(&BOULDERS, 0.04)
        .temperature(land::SAVANNA.0..)
        .humidity(..land::SAVANNA.1)
        .surface("grass", "dirt", 3)
        // A calmer grass yellow than the desert's: it greens in the rains and goes straw when dry
        // (`vitals::foliage::rain_tint`; the playtest read 0xbfb755 all year as neon).
        .tint(grass, muted(0xa9a565))
        .tint(foliage, muted(0xaea42a))
        .tint(&base_api::trees::EVERGREEN, muted(0xaea42a))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0xaea42a))
        .tag("hot")
        .tag("animals")
        .set(&TREES, 0.04)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:savanna").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        // Acacias and mesquite in the open, moriche palms along the streams (morichales).
        .set_text(&TREE_KINDS, "base:acacia_log:3 base:mesquite_log:1 base:moriche_palm_log:2");
    // Warm, wet lowland: still, dark water, low trees.
    r.biome(&SWAMP)
        .set(&CLIMATE, base_api::climate("base:swamp"))
        .temperature(land::SWAMP.0..)
        .humidity(land::SWAMP.1..)
        .surface("grass", "dirt", 3)
        .tint(grass, muted(0x6a7039))
        .tint(foliage, muted(0x6a7039))
        .tint(&base_api::trees::EVERGREEN, muted(0x6a7039))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x6a7039))
        .tint(water, 0x617b64)
        .tag("wet")
        .tag("animals")
        .set(&TREES, 0.12)
        // Palm groves on the higher ground (the Pantanal's capões).
        .set(&STAND_TREES, 0.1)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:swamp").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        // The Pantanal's acuri palms on the ground above the flood, a few moriches and oaks.
        .set_text(&TREE_KINDS, "base:acuri_palm_log:3 base:moriche_palm_log:1 base:oak_log:1");
    // Rainforest (Ronald, 11-04: "all four"; its look "A + C's giants + some B"), Manaus's terra
    // firme: a canopy at 28-32 of matamatá, andiroba and a few golden ipês, each crown its own
    // colour (`vitals::wild::OWN_COLOURS`); Brazil nuts as the buttressed emergents over it (the
    // giant scatter, about five a hectare), young ones, cacao and palms under it; its floor,
    // shrubs, big leaves and lianas are its `vitals::layers` row.
    r.biome(&TROPICAL_FOREST)
        .tint(water, 0x5d7f6a)
        .set(&CLIMATE, base_api::climate("base:tropical_forest"))
        .set(&BOULDERS, 0.01)
        .temperature(land::TROPICAL.0..)
        .humidity(land::TROPICAL.1..)
        .surface("grass", "dirt", 3)
        .tint(grass, muted(0x4f9e35))
        .tint(foliage, muted(0x3d8f28))
        .tint(&base_api::trees::EVERGREEN, muted(0x3d8f28))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x3d8f28))
        .tag("forest")
        .tag("wet")
        .tag("animals")
        .set(&TREES, 0.6)
        .set(&STAND_TREES, 0.15)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:tropical_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(
            &TREE_KINDS,
            "base:matamata_log:6 base:andiroba_log:5 base:ipe_log:1 base:cacao_log:3 base:brazil_nut_log:2 base:acuri_palm_log:1 base:moriche_palm_log:1",
        )
        .set(&GIANT_TREES, 0.3)
        .set_text(&GIANT_KINDS, "base:brazil_nut_log");
    r.biome(&MOUNTAINS)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:mountains"))
        .set(&BOULDERS, 0.4)
        .surface("grass", "dirt", 3)
        .bare_above(130)
        .tint(grass, muted(0x8ab689))
        .tint(foliage, muted(0x6da36b))
        .tint(&base_api::trees::EVERGREEN, muted(0x6da36b))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x6da36b))
        .tag("mountain")
        .tag("animals")
        .set(&TREES, 0.06)
        .set(&STAND_TREES, 0.08)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:mountains").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:spruce_log");
    // Giant sequoia groves on the mountains' front slopes (`base_api::land::SEQUOIA_AREA`, Giant
    // Forest's climate): the giants scattered among mixed conifers and black oaks over deep soil,
    // with clearings. Spruce and oak stand in for the mixed conifers and the sequoia until their
    // trees come (Voxel dog).
    r.biome(&SEQUOIA_GROVE)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:sequoia_grove"))
        .temperature(land::SEQUOIA_TEMPERATURE.0..land::SEQUOIA_TEMPERATURE.1)
        .humidity(land::SEQUOIA_HUMIDITY..)
        .set(&BOULDERS, 0.08)
        .surface("grass", "dirt", 5)
        .tint(grass, muted(0x7fa95a))
        .tint(foliage, muted(0x4d8a3c))
        .tint(&base_api::trees::EVERGREEN, muted(0x4d8a3c))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x6d9a3c))
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.12)
        .set(&STAND_TREES, 0.1)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:sequoia_grove").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:spruce_log:3 base:oak_log:1")
        // The giants among them (Plants): Giant Forest holds some 4-12 big sequoias a hectare;
        // a grove patch about 260 across at 0.45 per 24 x 24 cell is some 50, about 7 a hectare.
        .set(&GIANT_TREES, 0.45)
        .set_text(&GIANT_KINDS, "base:sequoia_log")
        // Lupine in its sunny gaps and meadows.
        .set_text(&FLOWERS, "base:lupine");
    // Old-growth forest (Ronald, 11-05: "a biome looking similar to the one shown in bliss ... with
    // all those trees"): Białowieża's strict reserve, old oaks with Norway spruce and a few
    // birches, the canopy closed over rolling hills to the horizon, rivers and lakes through it
    // with sandy beds. Nearly twice the forest's trees: a site in four 5 x 5 cells in five and more
    // in the stands, so the crowns touch, each its own (from branches), the spruces' dark spires
    // and the birches over the oaks' domes so the canopy has a skyline of crowns with shade between,
    // not a flat carpet (the first PC look, og_a1: at 0.95 + 0.3 of oaks alone it was one lime
    // mass); only the deepest clearings open, young trees and blossoming dogwoods in them
    // (`vitals::layers`). Deep greens (Bliss's forest, darker than the forest's), which the spring's
    // fresh green still lightens. Its own patches of the temperate wet land, not a climate range
    // (`land.rs`).
    r.biome(&OLD_GROWTH_FOREST)
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:old_growth_forest"))
        .set(&BOULDERS, 0.06)
        .surface("grass", "dirt", 4)
        .tint(grass, muted(0x5a9440))
        .tint(foliage, muted(0x346e1e))
        .tint(&base_api::trees::EVERGREEN, muted(0x2c5a22))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(0x4a8a28))
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.8)
        .set(&STAND_TREES, 0.15)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:old_growth_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:oak_log:6 base:spruce_log:2 base:birch_log:1");
    r.biome(&OCEAN)
        .tint(grass, muted(ENGINE_GRASS))
        .tint(foliage, muted(ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, muted(LEAF_TINT))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(LEAF_TINT))
        .tint(water, OCEAN_WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:ocean"))
        .surface("sand", "sand", 3)
        .deep_top("gravel", SEA - 8)
        .tag("ocean");
    // Warm shallow seas (Ronald, 11-04: "all three"): clear turquoise water over sand, and the
    // reef's coral (`land.rs` coral); Plants' coral plants and fans key to their names.
    for (biome, name) in [(&LAGOON, "base:lagoon"), (&CORAL_REEF, "base:coral_reef")] {
        r.biome(biome)
            .tint(grass, muted(ENGINE_GRASS))
            .tint(foliage, muted(ENGINE_FOLIAGE))
            .tint(&base_api::trees::EVERGREEN, muted(LEAF_TINT))
            .tint(&base_api::trees::YELLOW_LEAVES, muted(LEAF_TINT))
            .tint(water, LAGOON_WATER_TINT)
            .set(&CLIMATE, base_api::climate(name))
            .surface("sand", "sand", 3)
            .tag("ocean")
            .tag("warm_sea");
    }
    r.biome(&BEACH)
        .tint(grass, muted(ENGINE_GRASS))
        .tint(foliage, muted(ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, muted(LEAF_TINT))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(LEAF_TINT))
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:beach"))
        .surface("sand", "sand", 3)
        .tag("beach");
    // Underground (3D): whole cubes deep under the surface; spiders like it there.
    r.biome(&DEEP_CAVES)
        .tint(grass, muted(ENGINE_GRASS))
        .tint(foliage, muted(ENGINE_FOLIAGE))
        .tint(&base_api::trees::EVERGREEN, muted(LEAF_TINT))
        .tint(&base_api::trees::YELLOW_LEAVES, muted(LEAF_TINT))
        .tint(water, WATER_TINT)
        .set(&CLIMATE, base_api::climate("base:deep_caves"))
        .tag("cave")
        .tag("dark");

    // The last: share left out where it would touch a cave, vanilla's.
    for (ore, max_y, veins, size, exposed) in [
        (&COAL_ORE, 128, 8, 12, 0.0),
        (&IRON_ORE, 64, 6, 8, 0.0),
        (&GOLD_ORE, 32, 2, 7, 0.5),
        (&crate::redstone::REDSTONE_ORE, 16, 4, 8, 0.0),
        (&DIAMOND_ORE, 16, 1, 6, 0.5),
    ] {
        r.ore(ore)
            .y(i32::MIN, max_y)
            .per_cube(veins)
            .vein_size(size)
            .replaces("#stone_ore_replaceables")
            .exposed(exposed);
    }

    // Copper and tin by rock and biome (`metals.rs`).
    crate::metals::ores(r);

    // The layers under the trees, each biome's (`vitals::layers`).
    for l in vitals::layers::LAYERS
        .iter()
        .filter(|l| !vitals::layers::PENDING.contains(&l.biome))
    {
        let b = || Biome::generated(l.biome.to_string());
        if !l.floor.is_empty() {
            r.edit(b()).set_text(&FLOOR, l.floor);
            r.edit(b()).set(&FLOOR_COVER, l.floor_cover);
        }
        r.edit(b()).set(&FERNS, l.ferns);
        if l.young > 0.0 {
            r.edit(b()).set(&YOUNG_TREES, l.young);
        }
        if !l.young_kinds.is_empty() {
            r.edit(b()).set_text(&YOUNG_KINDS, l.young_kinds);
        }
        if !l.fern.is_empty() {
            r.edit(b()).set_text(&FERN_KINDS, l.fern);
        }
        if l.mushroom_cover > 0.0 {
            r.edit(b()).set_text(&MUSHROOMS, l.mushrooms);
            r.edit(b()).set(&MUSHROOM_COVER, l.mushroom_cover);
        }
        if l.tall_grass > 0.0 {
            r.edit(b()).set(&TALL_GRASS_SHARE, l.tall_grass);
        }
        if l.shrubs > 0.0 {
            r.edit(b()).set(&SHRUBS, l.shrubs);
            r.edit(b()).set_text(&SHRUB_KINDS, l.shrub_kinds);
        }
        if l.lianas > 0.0 {
            r.edit(b()).set(&LIANAS, l.lianas);
        }
        if l.gaps > 0.0 {
            r.edit(b()).set(&TREEFALL, l.gaps);
        }
        if l.big_leaves > 0.0 {
            r.edit(b()).set(&BIG_LEAVES, l.big_leaves);
            r.edit(b()).set_text(&BIG_LEAF_KINDS, l.big_leaf_kinds);
        }
    }

    // Wild cherry groves (`vitals::wild::CHERRY_GROVES`).
    for (biome, chance) in vitals::wild::CHERRY_GROVES {
        r.edit(Biome::generated(biome.to_string())).set(&CHERRY_TREES, chance);
    }

    // The overworld's shape, its snow and its bare rock (`land.rs`), before the plants below.
    crate::land::terrain(r);
    crate::land::mc_forest_terrain(r);

    // Trees wherever a biome asks for them, in any dimension.
    // Trees wherever a biome asks for them, in any dimension: a site per 5 x 5 cell, left open in
    // a forest's clearings; and where a forest stands thick (`STAND_TREES`), more in its stands, a
    // site per 3 x 3 cell. Stands and clearings follow one smooth noise (`stand`), so woods have
    // groves and glades instead of trees spread evenly.
    r.scatter(&TREE_SITES)
        .spacing(5)
        .reach(8)
        .ground("#plant_soil")
        .density(&TREES)
        // A forest's clearings and its cherry groves keep one tree in four (the groves' own trees,
        // `CHERRY_SITES`, fill them; among a full forest's crowns theirs only showed one or two).
        .thin_in(&STAND_TREES, stand_over(-CLEARING).and(cherry_grove().not()).not(), 4)
        .plan(tree);
    r.scatter(&GIANT_SITES)
        .spacing(24)
        .reach(12)
        // Giants rise higher than the default reach (Weft's 32 up).
        .reach_y(96, 16)
        .ground("#plant_soil")
        .density(&GIANT_TREES)
        .plan(giant);
    r.scatter(&STAND_SITES)
        .spacing(3)
        .reach(8)
        .ground("#plant_soil")
        .density(&STAND_TREES)
        .thin(stand_over(STAND + 1).and(cherry_grove().not()).not(), 0)
        .plan(tree);
    // Wild cherries in groves (`CHERRY_TREES`): a site per 4 x 4 cell only in the grove patches (a
    // noise some 320 blocks across, about 1.8% of the ground), among the forest's trees; a little taller than the oaks, so
    // their spring blossom shows over the canopy.
    r.scatter(&CHERRY_SITES)
        .spacing(4)
        .reach(8)
        .ground("#plant_soil")
        .density(&CHERRY_TREES)
        .thin(cherry_grove().not(), 0)
        .plan(cherry);
    // Boulders where a biome has them (`BOULDERS`): a heap of stone two to five blocks across, half
    // sunk in the ground, mossy in the woods and the wet.
    r.scatter(&BOULDER_SITES)
        .spacing(12)
        .reach(3)
        .density(&BOULDERS)
        .plan(boulder);
    // Shrubs where a biome has them (`SHRUBS`): a low clump of leaves per 4 x 4 cell, only in
    // thickets (a noise some 24 blocks across), so they stand in groups with open ground between.
    r.scatter(&SHRUB_SITES)
        .spacing(4)
        .reach(1)
        .density(&SHRUBS)
        .thin(Expr::fbm(0x5B2B, 24.0, 1).ramp(0.0, 0.0).not(), 0)
        .plan(shrub);
    // Young trees where light comes in: in clearings and at the stands' edges, none deep in a
    // stand (`stand_over`).
    r.scatter(&YOUNG_SITES)
        .spacing(5)
        .reach(base_api::tree_recipes::UNDERSTORY_REACH)
        .ground("#plant_soil")
        .density(&YOUNG_TREES)
        .thin(stand_over(STAND), 0)
        .plan(young_tree);
    r.decoration(decorate);
}

/// The test dimension: flat, bounded in height, nothing else.
pub fn flat(g: &mut Gen) {
    let BlockPos { y: oy, .. } = g.origin();
    for y in 0..16usize {
        let wy = oy + y as i32;
        let s = match wy {
            ..=-1 => STONE.id(),
            0..=2 => DIRT.id(),
            3 => GRASS.id(),
            _ => 0,
        };
        if s != 0 {
            for z in 0..16 {
                for x in 0..16 {
                    g.set(x, y, z, s);
                }
            }
        }
    }
}

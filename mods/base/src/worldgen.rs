//! Base's worldgen as data: biomes, the overworld's terrain, ores, trees (a scatter the engine
//! places in every dimension whose biomes ask for them) and plants (a decoration).

use base_api::CLIMATE;
use vitals::wild;
use weft_sdk::*;

use crate::blocks::*;

use weft_sdk::noise::*;
use weft_sdk::terrain::*;

pub const SEA: i32 = base_api::SEA;

static PLANT_SOIL: Tag = Tag::named("plant_soil");
static CACTUS_SOIL: Tag = Tag::named("cactus_soil");
static SUGAR_CANE_SOIL: Tag = Tag::named("sugar_cane_soil");
static SAND_TAG: Tag = Tag::named("sand");
/// What a biome grows, as data any mod can set on any biome: trees per 5x5 cell (0..1), share of
/// grass on plantable soil, which trees and flowers (block lists the engine checks), and tags `frozen` (nothing grows, set by
/// `.frozen(..)`), `dry` (cacti).
pub static TREES: Component<f32> = Component::named("trees");
pub static GRASS_COVER: Component<f32> = Component::named("grass");
/// The logs of the trees a biome grows, with weights: `"base:birch_log:1 base:oak_log:4"`
/// (oaks where unset).
pub static TREE_KINDS: Component<BlockList> = Component::named("tree_kinds");
/// Flowers a biome grows, picked evenly (`"base:dandelion base:rose"` where unset).
pub static FLOWERS: Component<BlockList> = Component::named("flowers");
// The trees' vocabulary and shapes are `base_api::trees`, so any mod's saplings grow them.
pub use base_api::trees::{TREE_LEAVES, grow_tree};
use base_api::trees::{tree_at, tree_blocks};
static FROZEN: Tag = Tag::named("frozen");
static DRY: Tag = Tag::named("dry");

pub static PLAINS: Biome = Biome::named("plains");
pub static FOREST: Biome = Biome::named("forest");
pub static BIRCH_FOREST: Biome = Biome::named("birch_forest");
pub static TAIGA: Biome = Biome::named("taiga");
pub static SNOWY: Biome = Biome::named("snowy_plains");
pub static DESERT: Biome = Biome::named("desert");
pub static SAVANNA: Biome = Biome::named("savanna");
pub static SWAMP: Biome = Biome::named("swamp");
pub static MOUNTAINS: Biome = Biome::named("mountains");
pub static OCEAN: Biome = Biome::named("ocean");
pub static BEACH: Biome = Biome::named("beach");
pub static DEEP_CAVES: Biome = Biome::named("deep_caves");

pub fn register(r: &mut Registry) {
    // Tags say what a biome is (spawning, decoration and mods ask these, not names).
    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    r.biome(&PLAINS)
        .set(&CLIMATE, base_api::climate("base:plains"))
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tag("spawn_biome")
        .set(&TREES, 0.025)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:plains").unwrap_or(vitals::forage::GRASS_COVER_ELSE));
    r.biome(&FOREST)
        .set(&CLIMATE, base_api::climate("base:forest"))
        .humidity(0.25..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x79c05a)
        .tint(foliage, 0x59ae30)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.4)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:birch_log:1 base:oak_log:4");
    r.biome(&BIRCH_FOREST)
        .set(&CLIMATE, base_api::climate("base:birch_forest"))
        .temperature(..0.1)
        .humidity(0.05..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x88bb67)
        .tint(foliage, 0x6ba941)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.45)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:birch_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:birch_log");
    r.biome(&TAIGA)
        .set(&CLIMATE, base_api::climate("base:taiga"))
        .temperature(..-0.15)
        .surface("podzol", "dirt", 3)
        .tint(grass, 0x86b783)
        .tint(foliage, 0x68a464)
        .tint(water, 0x3d57d6)
        .tag("forest")
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.45)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:taiga").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&SNOWY)
        .set(&CLIMATE, base_api::climate("base:snowy_plains"))
        .snows()
        .temperature(..-0.45)
        .surface("snowy_grass", "dirt", 3)
        .frozen("ice")
        .tint(grass, 0x80b497)
        .tint(foliage, 0x60a17b)
        .tint(water, 0x3938c9)
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.03)
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&DESERT)
        .set(&CLIMATE, base_api::climate("base:desert"))
        .temperature(0.35..)
        .humidity(..0.0)
        .surface("sand", "sand", 3)
        .under("sandstone", 1)
        .tint(grass, 0xbfb755)
        .tint(foliage, 0xaea42a)
        .tag("dry")
        .tag("hot")
        // Mesquite along the washes.
        .set(&TREES, 0.008)
        .set_text(&TREE_KINDS, "base:mesquite_log");
    // Hot grassland with few trees, between the desert and the forests (wet and dry seasons).
    r.biome(&SAVANNA)
        .set(&CLIMATE, base_api::climate("base:savanna"))
        .temperature(0.2..)
        .humidity(..0.2)
        .surface("grass", "dirt", 3)
        .tint(grass, 0xbfb755)
        .tint(foliage, 0xaea42a)
        .tag("hot")
        .tag("animals")
        .set(&TREES, 0.04)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:savanna").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        // Acacias and mesquite in the open, moriche palms along the streams (morichales).
        .set_text(&TREE_KINDS, "base:acacia_log:3 base:mesquite_log:1 base:moriche_palm_log:2");
    // Warm, wet lowland: still, dark water, low trees.
    r.biome(&SWAMP)
        .set(&CLIMATE, base_api::climate("base:swamp"))
        .temperature(-0.05..)
        .humidity(0.1..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x6a7039)
        .tint(foliage, 0x6a7039)
        .tint(water, 0x617b64)
        .tag("wet")
        .tag("animals")
        .set(&TREES, 0.12)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:swamp").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        // The Pantanal's acuri palms on the ground above the flood, a few moriches and oaks.
        .set_text(&TREE_KINDS, "base:acuri_palm_log:3 base:moriche_palm_log:1 base:oak_log:1");
    r.biome(&MOUNTAINS)
        .set(&CLIMATE, base_api::climate("base:mountains"))
        .surface("grass", "dirt", 3)
        .bare_above(130)
        .tint(grass, 0x8ab689)
        .tint(foliage, 0x6da36b)
        .tag("mountain")
        .tag("animals")
        .set(&TREES, 0.06)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:mountains").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&OCEAN)
        .set(&CLIMATE, base_api::climate("base:ocean"))
        .surface("sand", "sand", 3)
        .deep_top("gravel", SEA - 8)
        .tag("ocean");
    r.biome(&BEACH)
        .set(&CLIMATE, base_api::climate("base:beach"))
        .surface("sand", "sand", 3)
        .tag("beach");
    // Underground (3D): whole cubes deep under the surface; spiders like it there.
    r.biome(&DEEP_CAVES)
        .set(&CLIMATE, base_api::climate("base:deep_caves"))
        .tag("cave")
        .tag("dark");

    for (ore, max_y, veins, size) in [
        (&COAL_ORE, 128, 8, 12),
        (&IRON_ORE, 64, 6, 8),
        (&GOLD_ORE, 32, 2, 7),
        (&crate::redstone::REDSTONE_ORE, 16, 4, 8),
        (&DIAMOND_ORE, 16, 1, 6),
    ] {
        r.ore(ore)
            .y(i32::MIN, max_y)
            .veins(veins)
            .size(size)
            .replace("#stone_ore_replaceables");
    }

    // The overworld's shape. Other mods add to it by these names (`base:desert`, `base:mountains`).
    r.terrain(&crate::OVERWORLD)
        .stone("stone")
        .base_height(SEA as f64)
        .sea(SEA, "water")
        .feature(preset::continents(18.0))
        // Biomes shape the land through their climate, not their borders, so nothing seams:
        // the cold flattens hills into tundra, deserts get dunes.
        .feature(preset::hills(16.0, 110.0).mask(Mask::Temperature(-0.35, -0.6).not()))
        .feature(preset::dunes(4.0, 26.0).mask(Mask::Temperature(0.25, 0.45).and(Mask::Humidity(0.1, -0.05))))
        .feature(
            preset::mountains(150.0, 260.0)
                .mask(
                    Mask::noise(700.0, -0.1, 0.3)
                        .salt(0x44)
                        .and(Mask::Continent(0.0, 0.2)),
                )
                .named("mountains"),
        )
        // Biomes bring their climate and surface; rules only add where else they must hold.
        .biome(BiomeRule::new("ocean").height(..=SEA - 2))
        .biome(
            BiomeRule::new("snowy_plains")
                .named("snowy_coast")
                .height(..=SEA + 1)
                .continent(..0.1),
        )
        .biome(BiomeRule::new("beach").height(..=SEA + 1).continent(..0.1))
        .biome(BiomeRule::new("mountains").feature("mountains", 0.45..))
        .biome(BiomeRule::new("snowy_plains"))
        .biome(BiomeRule::new("taiga"))
        .biome(BiomeRule::new("desert"))
        .biome(BiomeRule::new("savanna"))
        // Swamps only on low land, where water stands.
        .biome(BiomeRule::new("swamp").height(..=SEA + 12).continent(0.1..))
        .biome(BiomeRule::new("forest"))
        .biome(BiomeRule::new("birch_forest"))
        .biome(BiomeRule::new("plains"))
        .cave_biome("deep_caves", 24, 96.0, 0.25)
        .caves(Caves::new("lava", i32::MIN, 11));

    // Trees wherever a biome asks for them, in any dimension.
    r.scatter(&TREE_SITES)
        .cell(5)
        .reach(3)
        .density(&TREES)
        .shape(|s| {
            let [x, y, z] = s.pos();
            let log = TREE_KINDS
                .on_biome(s.biome())
                .and_then(|l| l.pick(s.random() >> 40))
                .unwrap_or(OAK_LOG.id());
            let Some(t) = tree_at(log, x, y, z, s.random() >> 32) else { return };
            tree_blocks(&t, |x, y, z, state, force| if force { s.force([x, y, z], state) } else { s.put([x, y, z], state) });
        });
    r.decorate(decorate);
}

static TREE_SITES: Scatter = Scatter::named("trees");

/// Plants, after the terrain and trees filled the cube.
fn decorate(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let [ox, oy, oz] = g.origin();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);

    // Surface decoration only near the surface.
    if oy > max_h + 16 || oy + 15 < min_h - 1 {
        return;
    }
    for z in 0..16usize {
        for x in 0..16usize {
            let col = &cols[z * 16 + x];
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            // Floating plants on still, shallow water (`wild.rs`); frozen water is ice. Clay lies in
            // patches under shallow inland water, most in swamps, as real clay settles in still
            // water; the sea's floor is sand.
            if col.height < SEA {
                let shore = shore_biome(g, wx, wz, col.biome);
                let floor = col.height - oy;
                if col.height >= SEA - CLAY_DEPTH && (0..16).contains(&floor) {
                    let share = match name_of(abi::entry_kind::BIOME, shore) {
                        "base:swamp" => 0.5,
                        "base:ocean" | "base:beach" => 0.0,
                        _ => 0.2,
                    };
                    if unit(hash(seed ^ 0xC1A, (wx >> 2) as i64, 0, (wz >> 2) as i64)) < share {
                        g.set(x, floor as usize, z, CLAY.id());
                    }
                }
                if col.height >= SEA - WILD_WATER_DEPTH {
                    for top in [SEA, SEA - 1] {
                        let y = top + 1 - oy;
                        if (1..16).contains(&y) && g.get(x, y as usize, z) == 0 && WATER.is(g.get(x, y as usize - 1, z)) {
                            if let Some(s) = wild_at(seed, wx, wz, shore, wild::Ground::Water) {
                                g.set(x, y as usize, z, s);
                            }
                            break;
                        }
                    }
                }
                continue;
            }
            let y = col.height + 1 - oy;
            if !(0..16).contains(&y) {
                continue;
            }
            let y = y as usize;
            if g.get(x, y, z) != 0 {
                continue;
            }
            let h = hash(seed ^ 0xF10, wx as i64, 0, wz as i64);
            let roll = unit(h);
            // Plants go where they could be planted (soil tags); the biome decides how many.
            let b = col.biome;
            let grass_chance = GRASS_COVER
                .on_biome(b)
                .unwrap_or(vitals::forage::GRASS_COVER_ELSE) as f64;
            // What is under the lowest layer takes a look at the terrain: only if anything would grow.
            if y == 0 && roll >= grass_chance.max(0.08) && wild_roll(seed, wx, wz) >= wild::MOST {
                continue;
            }
            let below = if y == 0 { g.surface_block(wx, wz) } else { g.get(x, y - 1, z) };
            // Ponds in swamps and floodplains (`vitals::wild::POND_LANDS`): still water a block deep
            // where the ground is level, its floating plants on it.
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| (wx + dx, wz + dz, g.column(wx + dx, wz + dz)));
            if y >= 1
                && pond(seed, wx, wz, col.height, b)
                && PLANT_SOIL.on_block(below)
                && near.iter().all(|(_, _, c)| c.height >= col.height)
            {
                g.set(x, y - 1, z, WATER.id());
                if let Some(s) = wild_at(seed, wx, wz, b, wild::Ground::Water) {
                    g.set(x, y, z, s);
                }
                continue;
            }
            // The wild plants first, on their own roll (`wild.rs`), in the frozen north too; a shore
            // is the sea's edge or a pond's (a pond's neighbour at its own height).
            let shore = (col.height == SEA && near.iter().any(|(_, _, c)| c.height < SEA))
                || near
                    .iter()
                    .any(|&(nx, nz, c)| c.height == col.height && pond(seed, nx, nz, c.height, c.biome));
            let ground = if PLANT_SOIL.on_block(below) {
                Some(if shore { wild::Ground::Shore } else { wild::Ground::Soil })
            } else if SAND_TAG.on_block(below) {
                Some(wild::Ground::Sand)
            } else {
                None
            };
            if let Some(s) = ground.and_then(|gr| wild_at(seed, wx, wz, b, gr)) {
                g.set(x, y, z, s);
                continue;
            }
            if FROZEN.on_biome(b) {
                continue;
            }
            if PLANT_SOIL.on_block(below) {
                if roll < 0.012 {
                    let flower = FLOWERS
                        .on_biome(b)
                        .map_or_else(|| [DANDELION.id(), ROSE.id()][(h >> 32) as usize % 2], |l| l.pick(h >> 32).unwrap_or(0));
                    if flower != 0 {
                        g.set(x, y, z, flower);
                    }
                } else if roll < grass_chance {
                    g.set(x, y, z, TALL_GRASS.id());
                }
            } else if CACTUS_SOIL.on_block(below) && DRY.on_biome(b) && roll < 0.006 && y + 2 < 16 {
                let n = 1 + (h >> 40) % 3;
                for k in 0..n as usize {
                    g.set(x, y + k, z, CACTUS.id());
                }
            } else if CACTUS_SOIL.on_block(below) && DRY.on_biome(b) && roll < 0.02 {
                // Dead bushes: the desert's few sticks.
                g.set(x, y, z, DEAD_BUSH.id());
            } else if SUGAR_CANE_SOIL.on_block(below)
                && col.height == SEA
                && roll < 0.08
                && y + 2 < 16
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|&(dx, dz)| g.column(wx + dx, wz + dz).height < SEA)
            {
                let n = 1 + (h >> 40) % 3;
                for k in 0..n as usize {
                    g.set(x, y + k, z, SUGAR_CANE.id());
                }
            }
        }
    }
}

/// Water clay lies under is this deep at most.
const CLAY_DEPTH: i32 = 4;

/// Water a floating wild plant grows on is this deep at most.
const WILD_WATER_DEPTH: i32 = 3;

/// The biome water at a column belongs to for its plants: the land's beside it. Water deeper
/// than a block is the ocean to the world (`BiomeRule`), but a swamp's pond or a floodplain's
/// river is theirs; open water far from land stays its own.
fn shore_biome(g: &Gen, wx: i32, wz: i32, own: u32) -> u32 {
    [(3, 0), (-3, 0), (0, 3), (0, -3), (6, 0), (-6, 0), (0, 6), (0, -6)]
        .iter()
        .map(|&(dx, dz)| g.column(wx + dx, wz + dz))
        .find(|c| c.height >= SEA)
        .map_or(own, |c| c.biome)
}

/// Whether the column at `wx`, `wz` (ground at `height`, in `biome`) is a pond's: round ponds of
/// radius 2 to 4, one in the share of 8 x 8 cells `vitals::wild::POND_LANDS` gives its biome, on
/// ground above the sea (the swamps' own low land; where it isn't level, the caller leaves it dry).
fn pond(seed: u64, wx: i32, wz: i32, height: i32, biome: u32) -> bool {
    if height < SEA {
        return false;
    }
    let name = name_of(abi::entry_kind::BIOME, biome);
    let Some(&(_, share)) = wild::POND_LANDS.iter().find(|(b, _)| *b == name) else { return false };
    let (cx, cz) = (wx.div_euclid(8), wz.div_euclid(8));
    (-1..=1).any(|i| {
        (-1..=1).any(|k| {
            let (ox, oz) = (cx + i, cz + k);
            let h = hash(seed ^ 0x90D, ox as i64, 0, oz as i64);
            if unit(h) >= share as f64 {
                return false;
            }
            let (px, pz) = (ox * 8 + 2 + ((h >> 20) % 4) as i32, oz * 8 + 2 + ((h >> 28) % 4) as i32);
            let r = 2 + ((h >> 36) % 3) as i32;
            (wx - px).pow(2) + (wz - pz).pow(2) <= r * r
        })
    })
}

/// A column's roll for the wild plants, 0 to 1.
fn wild_roll(seed: u64, wx: i32, wz: i32) -> f32 {
    unit(hash(seed ^ 0x3D1, wx as i64, 0, wz as i64)) as f32
}

/// The wild plant (its state) that grows on this column's `ground` in `biome`, if any
/// (`vitals::wild::pick`).
fn wild_at(seed: u64, wx: i32, wz: i32, biome: u32, ground: wild::Ground) -> Option<u32> {
    let roll = wild_roll(seed, wx, wz);
    if roll >= wild::MOST {
        return None;
    }
    let i = wild::pick(name_of(abi::entry_kind::BIOME, biome), ground, roll)?;
    let s = crate::wild::state(i);
    (s != u32::MAX).then_some(s)
}

/// The test dimension: flat, bounded in height, nothing else.
pub fn flat(g: &mut Gen) {
    let [_, oy, _] = g.origin();
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

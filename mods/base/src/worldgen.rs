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
/// On a log, the tree it grows into: `round <leaves>` (like oaks, 4-6 high) or `cone <leaves>`
/// (like spruces, 6-9 high).
pub static TREE_LEAVES: Component<BlockRef> = Component::named("tree_leaves");
/// Logs with this tag grow into cones (spruces), the others round (oaks, birches).
pub static CONE_TREE: Tag = Tag::named("cone_tree");
/// A tall bare trunk under one wide, flat crown (the acacia).
pub static FLAT_TREE: Tag = Tag::named("flat_tree");
/// A taller bare trunk under a crown of fronds that droop at their tips (palms, tree ferns).
pub static PALM_TREE: Tag = Tag::named("palm_tree");
static FROZEN: Tag = Tag::named("frozen");
static DRY: Tag = Tag::named("dry");
static FOREST_TAG: Tag = Tag::named("forest");

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

struct TreeAt {
    x: i32,
    y: i32,
    z: i32,
    log: u32,
    leaves: u32,
    cone: bool,
    flat: bool,
    palm: bool,
    height: i32,
}

/// A tree growing from `log` (its `tree` component), `h` deciding the height.
fn tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    let leaves = TREE_LEAVES.on_block(log)?.0;
    let cone = CONE_TREE.on_block(log);
    let flat = FLAT_TREE.on_block(log);
    let palm = PALM_TREE.on_block(log);
    let height = if cone {
        6 + (h % 4) as i32
    } else if palm {
        6 + (h % 3) as i32
    } else if flat {
        5 + (h % 2) as i32
    } else {
        4 + (h % 3) as i32
    };
    Some(TreeAt { x, y, z, log, leaves, cone, flat, palm, height })
}

/// Calls `put(x, y, z, state, force)` for every block of the tree.
fn tree_blocks(t: &TreeAt, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let (log, leaves) = (t.log, t.leaves);
    let top = t.y + t.height;
    match t.cone {
        _ if t.palm => {
            // A tuft on top, fronds out two blocks each way, their tips a block lower.
            put(t.x, top, t.z, leaves, false);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                for k in 1..=2 {
                    put(t.x + dx * k, top - 1, t.z + dz * k, leaves, false);
                }
                put(t.x + dx * 3, top - 2, t.z + dz * 3, leaves, false);
            }
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                put(t.x + dx, top - 1, t.z + dz, leaves, false);
                put(t.x + dx * 2, top - 2, t.z + dz * 2, leaves, false);
            }
        }
        _ if t.flat => {
            // One wide flat layer on the trunk's top, a small one over its middle.
            for (y, r2) in [(top, 10), (top + 1, 2)] {
                for dz in -3i32..=3 {
                    for dx in -3i32..=3 {
                        let d = dx * dx + dz * dz;
                        if d <= r2 && (d < r2 || hash(1, (t.x + dx) as i64, y as i64, (t.z + dz) as i64) & 1 == 0) {
                            put(t.x + dx, y, t.z + dz, leaves, false);
                        }
                    }
                }
            }
        }
        true => {
            for dy in 2..=t.height + 1 {
                let y = t.y + dy;
                let r = if dy == t.height + 1 { 0 } else { 1 + ((t.height + 1 - dy) % 3).min(2) };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz <= r * r + 1 {
                            put(t.x + dx, y, t.z + dz, leaves, false);
                        }
                    }
                }
            }
        }
        false => {
            for y in top - 3..=top {
                let r: i32 = if y >= top - 1 { 1 } else { 2 };
                for dz in -r..=r {
                    for dx in -r..=r {
                        let corner = dx.abs() == r && dz.abs() == r;
                        if corner && (y == top || hash(0, (t.x + dx) as i64, y as i64, (t.z + dz) as i64) & 1 == 0) {
                            continue;
                        }
                        put(t.x + dx, y, t.z + dz, leaves, false);
                    }
                }
            }
        }
    }
    for y in t.y..top {
        put(t.x, y, t.z, log, true);
    }
    put(t.x, t.y - 1, t.z, DIRT.id(), true);
}

/// Places a tree of `log` into a live world (saplings). Checks for room first.
pub fn grow_tree(w: World, p: [i32; 3], log: u32) {
    let Some(t) = tree_at(log, p[0], p[1], p[2], w.random()) else { return };
    for y in t.y + 1..t.y + t.height {
        if w.get([t.x, y, t.z]) != 0 {
            return;
        }
    }
    tree_blocks(&t, |x, y, z, s, overwrite| {
        let cur = w.get([x, y, z]);
        if overwrite || cur == 0 || TALL_GRASS.is(cur) {
            w.set([x, y, z], s);
        }
    });
}

/// Plants, after the terrain and trees filled the cube.
fn decorate(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let [ox, oy, oz] = g.origin();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);

    // Surface decoration only near the surface, which under the sea is its water's (kelp).
    let top = if min_h < SEA { max_h.max(SEA) } else { max_h };
    if oy > top + 16 || oy + 15 < min_h - 1 {
        return;
    }
    for z in 0..16usize {
        for x in 0..16usize {
            let col = &cols[z * 16 + x];
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            // Floating plants on still, shallow water (`wild.rs`); frozen water is ice. Under it,
            // plants of the bed standing in the water.
            if col.height < SEA {
                bed(g, seed, [x, z], [wx, wz], col.height, col.biome);
                if col.height >= SEA - WILD_WATER_DEPTH {
                    for top in [SEA, SEA - 1] {
                        let y = top + 1 - oy;
                        if (1..16).contains(&y) && g.get(x, y as usize, z) == 0 && WATER.is(g.get(x, y as usize - 1, z)) {
                            if let Some(s) = wild_at(seed, wx, wz, col.biome, wild::Ground::Water) {
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
            // The wild plants first, on their own roll (`wild.rs`), in the frozen north too.
            let shore = col.height == SEA
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|&(dx, dz)| g.column(wx + dx, wz + dz).height < SEA);
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
                } else if roll < 0.016 && FOREST_TAG.on_biome(b) {
                    // Berry bushes in forests, unripe: found, then come back for.
                    g.set(x, y, z, BERRY_BUSH.id());
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

/// The plant of the bed (eelgrass, kelp; `vitals::wild::Ground::Bed`) on a column whose floor is
/// at `floor` under the sea, in this cube's part of it: in the water just over the floor, kelp on
/// up to its height, a block below the surface at most.
fn bed(g: &mut Gen, seed: u64, [x, z]: [usize; 2], [wx, wz]: [i32; 2], floor: i32, biome: u32) {
    let roll = wild_roll(seed, wx, wz);
    if roll >= wild::MOST {
        return;
    }
    let Some(i) = wild::pick(name_of(abi::entry_kind::BIOME, biome), wild::Ground::Bed, roll) else { return };
    let plant = &wild::WILD[i];
    let depth = SEA - floor;
    if !wild::depths(plant.block).contains(&depth) {
        return;
    }
    let s = crate::wild::state(i);
    if s == u32::MAX {
        return;
    }
    let tall =
        if plant.form == wild::Form::Column { 1 + (hash(seed ^ 0xCE1, wx as i64, 0, wz as i64) % (depth - 1) as u64) as i32 } else { 1 };
    let oy = g.origin()[1];
    for y in floor + 1..=floor + tall {
        let ly = y - oy;
        if (0..16).contains(&ly) && WATER.is(g.get(x, ly as usize, z)) {
            g.set(x, ly as usize, z, s);
        }
    }
}

/// Water a floating wild plant grows on is this deep at most.
const WILD_WATER_DEPTH: i32 = 3;

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

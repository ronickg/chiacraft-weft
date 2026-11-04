//! Base's worldgen as data: biomes, the overworld's terrain, ores, trees (a scatter the engine
//! places in every dimension whose biomes ask for them) and plants (a decoration).

use base_api::CLIMATE;
use base_api::land;
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
/// Boulders: the chance of a heap of stone per 12 x 12 cell (`boulder`).
pub static BOULDERS: Component<f32> = Component::named("boulders");
/// Where a forest stands thick: the chance of a tree per 3 x 3 cell in its stands, on top of
/// `TREES`, with clearings left open between them. Mature stands hold 170-244 trees a hectare
/// (Research round 7): a 5 x 5 site at 0.4 is 160, a 3 x 3 one at 0.12 adds 130 in the stands (the
/// trees grown from branches are wide: denser, the forest and taiga read as walls of leaves).
pub static STAND_TREES: Component<f32> = Component::named("stand_trees");
pub static GRASS_COVER: Component<f32> = Component::named("grass");
/// The logs of the trees a biome grows, with weights: `"base:birch_log:1 base:oak_log:4"`
/// (oaks where unset).
pub static TREE_KINDS: Component<BlockList> = Component::named("tree_kinds");
/// Flowers a biome grows, picked evenly (`"base:dandelion base:rose"` where unset).
pub static FLOWERS: Component<BlockList> = Component::named("flowers");
/// Giant trees (a sequoia grove's): the chance of one per 24 x 24 cell, and which (logs with a
/// recipe, `tree_recipes`), in a scatter of their own so distant terrain draws them at their own
/// height, not averaged with the small trees.
pub static GIANT_TREES: Component<f32> = Component::named("giant_trees");
pub static GIANT_KINDS: Component<BlockList> = Component::named("giant_kinds");
/// The layers under the trees (`vitals::layers`, `notes/VEGETATION.md`), as data on the biome:
/// floor blocks swapped in for the top block in patches, by weight, and the share they cover;
/// ferns on the bare soil; shrub clumps per 4 x 4 cell in thickets, made of these leaves.
pub static FLOOR: Component<BlockList> = Component::named("floor");
pub static FLOOR_COVER: Component<f32> = Component::named("floor_cover");
pub static FERNS: Component<f32> = Component::named("ferns");
pub static SHRUBS: Component<f32> = Component::named("shrubs");
pub static SHRUB_KINDS: Component<BlockList> = Component::named("shrub_kinds");
/// Young trees of the biome's `TREE_KINDS`, the chance per 5 x 5 cell, out of the thick stands.
pub static YOUNG_TREES: Component<f32> = Component::named("young_trees");
// The trees' vocabulary and shapes are `base_api::trees`, so any mod's saplings grow them.
pub use base_api::trees::{TREE_LEAVES, grow_tree};
use base_api::trees::{tree_blocks, wild_tree_at};
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
pub static SEQUOIA_GROVE: Biome = Biome::named("sequoia_grove");
pub static OCEAN: Biome = Biome::named("ocean");
pub static BEACH: Biome = Biome::named("beach");
pub static DEEP_CAVES: Biome = Biome::named("deep_caves");

pub fn register(r: &mut Registry) {
    // Tags say what a biome is (spawning, decoration and mods ask these, not names).
    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    r.biome(&PLAINS)
        .set(&CLIMATE, base_api::climate("base:plains"))
        .set(&BOULDERS, 0.05)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tag("spawn_biome")
        .set(&TREES, 0.025)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:plains").unwrap_or(vitals::forage::GRASS_COVER_ELSE));
    r.biome(&FOREST)
        .set(&CLIMATE, base_api::climate("base:forest"))
        .set(&BOULDERS, 0.12)
        .humidity(land::FOREST_HUMIDITY..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x79c05a)
        .tint(foliage, 0x59ae30)
        .tint(&base_api::trees::EVERGREEN, 0x59ae30)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x59ae30)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.4)
        .set(&STAND_TREES, 0.12)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:birch_log:1 base:oak_log:4");
    r.biome(&BIRCH_FOREST)
        .set(&CLIMATE, base_api::climate("base:birch_forest"))
        .set(&BOULDERS, 0.1)
        .temperature(..land::BIRCH.0)
        .humidity(land::BIRCH.1..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x88bb67)
        .tint(foliage, 0x6ba941)
        .tint(&base_api::trees::EVERGREEN, 0x6ba941)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x6ba941)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.45)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:birch_forest").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set(&STAND_TREES, 0.15)
        .set_text(&TREE_KINDS, "base:birch_log");
    r.biome(&TAIGA)
        .set(&CLIMATE, base_api::climate("base:taiga"))
        .set(&BOULDERS, 0.25)
        .temperature(..land::TAIGA_BELOW)
        .surface("podzol", "dirt", 3)
        .tint(grass, 0x86b783)
        .tint(foliage, 0x68a464)
        .tint(&base_api::trees::EVERGREEN, 0x68a464)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x68a464)
        .tint(water, 0x3d57d6)
        .tag("forest")
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.45)
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
        .tint(grass, 0x80b497)
        .tint(foliage, 0x60a17b)
        .tint(&base_api::trees::EVERGREEN, 0x60a17b)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x60a17b)
        .tint(water, 0x3938c9)
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.03)
        .set_text(&TREE_KINDS, "base:spruce_log");
    r.biome(&DESERT)
        .set(&CLIMATE, base_api::climate("base:desert"))
        .set(&BOULDERS, 0.04)
        .temperature(land::DESERT.0..)
        .humidity(..land::DESERT.1)
        .surface("sand", "sand", 3)
        .under("sandstone", 1)
        .tint(grass, 0xbfb755)
        .tint(foliage, 0xaea42a)
        .tint(&base_api::trees::EVERGREEN, 0xaea42a)
        .tint(&base_api::trees::YELLOW_LEAVES, 0xaea42a)
        .tag("dry")
        .tag("hot")
        // Mesquite along the washes.
        .set(&TREES, 0.008)
        .set_text(&TREE_KINDS, "base:mesquite_log");
    // Hot grassland with few trees, between the desert and the forests (wet and dry seasons).
    r.biome(&SAVANNA)
        .set(&CLIMATE, base_api::climate("base:savanna"))
        .set(&BOULDERS, 0.04)
        .temperature(land::SAVANNA.0..)
        .humidity(..land::SAVANNA.1)
        .surface("grass", "dirt", 3)
        .tint(grass, 0xbfb755)
        .tint(foliage, 0xaea42a)
        .tint(&base_api::trees::EVERGREEN, 0xaea42a)
        .tint(&base_api::trees::YELLOW_LEAVES, 0xaea42a)
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
        .tint(grass, 0x6a7039)
        .tint(foliage, 0x6a7039)
        .tint(&base_api::trees::EVERGREEN, 0x6a7039)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x6a7039)
        .tint(water, 0x617b64)
        .tag("wet")
        .tag("animals")
        .set(&TREES, 0.12)
        // Palm groves on the higher ground (the Pantanal's capões).
        .set(&STAND_TREES, 0.1)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:swamp").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        // The Pantanal's acuri palms on the ground above the flood, a few moriches and oaks.
        .set_text(&TREE_KINDS, "base:acuri_palm_log:3 base:moriche_palm_log:1 base:oak_log:1");
    r.biome(&MOUNTAINS)
        .set(&CLIMATE, base_api::climate("base:mountains"))
        .set(&BOULDERS, 0.4)
        .surface("grass", "dirt", 3)
        .bare_above(130)
        .tint(grass, 0x8ab689)
        .tint(foliage, 0x6da36b)
        .tint(&base_api::trees::EVERGREEN, 0x6da36b)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x6da36b)
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
        .set(&CLIMATE, base_api::climate("base:sequoia_grove"))
        .temperature(land::SEQUOIA_TEMPERATURE.0..land::SEQUOIA_TEMPERATURE.1)
        .humidity(land::SEQUOIA_HUMIDITY..)
        .set(&BOULDERS, 0.08)
        .surface("grass", "dirt", 5)
        .tint(grass, 0x7fa95a)
        .tint(foliage, 0x4d8a3c)
        .tint(&base_api::trees::EVERGREEN, 0x4d8a3c)
        .tint(&base_api::trees::YELLOW_LEAVES, 0x6d9a3c)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.12)
        .set(&STAND_TREES, 0.1)
        .set(&GRASS_COVER, vitals::forage::grass_cover("base:sequoia_grove").unwrap_or(vitals::forage::GRASS_COVER_ELSE))
        .set_text(&TREE_KINDS, "base:spruce_log:3 base:oak_log:1");
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

    // The layers under the trees, each biome's (`vitals::layers`).
    for l in &vitals::layers::LAYERS {
        let b = || Target::Biome(l.biome);
        if !l.floor.is_empty() {
            r.set_text_on(b(), &FLOOR, l.floor);
            r.set_on(b(), &FLOOR_COVER, l.floor_cover);
        }
        r.set_on(b(), &FERNS, l.ferns);
        if l.young > 0.0 {
            r.set_on(b(), &YOUNG_TREES, l.young);
        }
        if l.shrubs > 0.0 {
            r.set_on(b(), &SHRUBS, l.shrubs);
            r.set_text_on(b(), &SHRUB_KINDS, l.shrub_kinds);
        }
    }

    // The overworld's shape, its snow and its bare rock (`land.rs`), before the plants below.
    crate::land::terrain(r);

    // Trees wherever a biome asks for them, in any dimension.
    // Trees wherever a biome asks for them, in any dimension: a site per 5 x 5 cell, left open in
    // a forest's clearings; and where a forest stands thick (`STAND_TREES`), more in its stands, a
    // site per 3 x 3 cell. Stands and clearings follow one smooth noise (`stand`), so woods have
    // groves and glades instead of trees spread evenly.
    r.scatter(&TREE_SITES)
        .cell(5)
        .reach(8)
        .density(&TREES)
        // A forest's clearings keep one tree in four.
        .thin_in(&STAND_TREES, stand_over(-CLEARING).not(), 4)
        .shape(tree);
    r.scatter(&GIANT_SITES)
        .cell(24)
        .reach(12)
        .density(&GIANT_TREES)
        .shape(giant);
    r.scatter(&STAND_SITES)
        .cell(3)
        .reach(8)
        .density(&STAND_TREES)
        .thin(stand_over(STAND + 1).not(), 0)
        .shape(tree);
    // Boulders where a biome has them (`BOULDERS`): a heap of stone two to five blocks across, half
    // sunk in the ground, mossy in the woods and the wet.
    r.scatter(&BOULDER_SITES)
        .cell(12)
        .reach(3)
        .density(&BOULDERS)
        .shape(boulder);
    // Shrubs where a biome has them (`SHRUBS`): a low clump of leaves per 4 x 4 cell, only in
    // thickets (a noise some 24 blocks across), so they stand in groups with open ground between.
    r.scatter(&SHRUB_SITES)
        .cell(4)
        .reach(1)
        .density(&SHRUBS)
        .thin(Mask::Noise { salt: 0x5B2B, scale: 24.0, octaves: 1, from: 0.0, to: 0.0 }.not(), 0)
        .shape(shrub);
    // Young trees where light comes in: in clearings and at the stands' edges, none deep in a
    // stand (`stand_over`).
    r.scatter(&YOUNG_SITES)
        .cell(5)
        .reach(2)
        .density(&YOUNG_TREES)
        .thin(stand_over(STAND), 0)
        .shape(young_tree);
    r.decorate(decorate);
}

/// A young tree of the biome's kinds (`TREE_KINDS`) at a scatter's site; none on a bare rock face.
fn young_tree(s: &mut Site) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos()) {
        return;
    }
    let [x, y, z] = s.pos();
    let h = s.random();
    let log = TREE_KINDS.on_biome(s.biome()).and_then(|l| l.pick(h >> 40)).unwrap_or(OAK_LOG.id());
    base_api::trees::young_tree_blocks(log, x, y, z, h >> 8, |x, y, z, state, force| {
        if force { s.force([x, y, z], state) } else { s.put([x, y, z], state) }
    });
}

/// A shrub at a scatter's site: a clump of the biome's shrub leaves (`SHRUB_KINDS`), one or two
/// high (`vitals::layers::shrub_height`) and one to three across; none on a bare rock face.
fn shrub(s: &mut Site) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos()) {
        return;
    }
    let h = s.random();
    let Some(leaves) = SHRUB_KINDS.on_biome(s.biome()).and_then(|l| l.pick(h >> 40)) else { return };
    let tall = vitals::layers::shrub_height(name_of(abi::entry_kind::BIOME, s.biome()));
    let [x, y, z] = s.pos();
    for dy in 0..tall {
        for dz in -1..=1i32 {
            for dx in -1..=1i32 {
                let side = dx.abs() + dz.abs();
                // The middle always; its sides most of the time low down, less above; corners
                // seldom, and only low.
                let bit = hash(h, dx as i64, dy as i64, dz as i64) & 15;
                let keep = match (dy, side) {
                    (_, 0) => true,
                    (0, 1) => bit < 11,
                    (0, _) => bit < 4,
                    (_, 1) => bit < 5,
                    _ => false,
                };
                if keep {
                    s.put([x + dx, y + dy, z + dz], leaves);
                }
            }
        }
    }
}

/// A boulder at a scatter's site: a lumpy heap, cobbles and stone, some mossy where it's wet; none
/// on a bare rock face (`land::site_on_rock`), which is stone already.
fn boulder(s: &mut Site) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos()) {
        return;
    }
    let [x, y, z] = s.pos();
    let h = s.random();
    let mossy = WET.on_biome(s.biome()) || WOODS.on_biome(s.biome());
    let r = 1 + (h % 3) as i32;
    for dy in -1..=r {
        for dz in -r..=r {
            for dx in -r..=r {
                let bump = (hash(h, (x + dx) as i64, (y + dy) as i64, (z + dz) as i64) % 3) as i32;
                if dx * dx + 2 * dy * dy + dz * dz > r * r + bump {
                    continue;
                }
                let pick = hash(h ^ 0xB0, (x + dx) as i64, (y + dy) as i64, (z + dz) as i64) % 4;
                let block = match pick {
                    0 if mossy => MOSSY_COBBLESTONE.id(),
                    0 | 1 => COBBLESTONE.id(),
                    _ => STONE.id(),
                };
                s.force([x + dx, y + dy, z + dz], block);
            }
        }
    }
}

static TREE_SITES: Scatter = Scatter::named("trees");
static STAND_SITES: Scatter = Scatter::named("stand_trees");
static GIANT_SITES: Scatter = Scatter::named("giant_trees");

/// A giant of the biome's `GIANT_KINDS` at a scatter's site; none on a bare rock face.
fn giant(s: &mut Site) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos()) {
        return;
    }
    let h = s.random();
    let Some(log) = GIANT_KINDS.on_biome(s.biome()).and_then(|l| l.pick(h >> 40)) else { return };
    let [x, y, z] = s.pos();
    base_api::trees::giant_tree_blocks(log, x, y, z, h >> 8, |x, y, z, state, force| {
        if force { s.force([x, y, z], state) } else { s.put([x, y, z], state) }
    });
}
static BOULDER_SITES: Scatter = Scatter::named("boulders");
static SHRUB_SITES: Scatter = Scatter::named("shrubs");
static YOUNG_SITES: Scatter = Scatter::named("young_trees");
static LOGS: Tag = Tag::named("logs");
static LEAVES: Tag = Tag::named("leaves");
static WET: Tag = Tag::named("wet");
static WOODS: Tag = Tag::named("forest");

/// The stand noise over the ground, about -4096 to 4096: a forest's stands where it's over
/// `STAND`, its clearings where it's under `-CLEARING`; one patch is some 40 blocks across. As a
/// mask that is on where the noise is `edge` or more (one octave of `fbm` is `perlin2` exactly),
/// so the engine thins sites by it near and in distant terrain alike.
fn stand_over(edge: i32) -> Mask {
    let e = edge as f64 / fixed::ONE as f64;
    Mask::Noise { salt: 0x57A4D, scale: 40.0, octaves: 1, from: e, to: e }
}
const STAND: i32 = 600;
/// Which flower a column of a flower patch grows: one kind to a patch (its 16 x 16 cell), a few
/// strays of the others.
fn flower_kind(seed: u64, wx: i32, wz: i32, h: u64) -> u64 {
    if h >> 60 == 0 { h >> 32 } else { hash(seed ^ 0xF70, (wx >> 4) as i64, 0, (wz >> 4) as i64) }
}

/// Where the meadow noise is over this, flowers grow thick (a sixth of the ground or so).
const FLOWER_MEADOW: f64 = 0.72;
const CLEARING: i32 = 1600;

/// A tree of the biome's kinds (`TREE_KINDS`) at a scatter's site; none on a bare rock face
/// (`land::site_on_rock`).
fn tree(s: &mut Site) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos()) {
        return;
    }
    let [x, y, z] = s.pos();
    let log = TREE_KINDS
        .on_biome(s.biome())
        .and_then(|l| l.pick(s.random() >> 40))
        .unwrap_or(OAK_LOG.id());
    let Some(t) = wild_tree_at(log, x, y, z, s.random() >> 32) else { return };
    tree_blocks(&t, |x, y, z, state, force| if force { s.force([x, y, z], state) } else { s.put([x, y, z], state) });
}

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
                if col.height >= SEA - CLAY_DEPTH && (0..16).contains(&floor) && clay(seed, wx, wz, shore) {
                    g.set(x, floor as usize, z, CLAY.id());
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
            // Grass and flowers in patches, not spread evenly: a meadow noise (0 to 1, patches some
            // 14 blocks across) thickens the grass where it's high and thins it where it's low, the
            // biome's cover on average; flowers come in patches of one kind where it's highest.
            let meadow = ((fixed::perlin2(fixed::seed32(seed, 0x3EAD), wx * 16, wz * 16, 14 * 16) + 4096) as f64 / 8192.0).clamp(0.0, 1.0);
            let grass_chance = (GRASS_COVER
                .on_biome(b)
                .unwrap_or(vitals::forage::GRASS_COVER_ELSE) as f64
                * (0.35 + 1.3 * meadow))
                .min(0.9);
            let flower_chance = if meadow > FLOWER_MEADOW { 0.06 } else { 0.004 };
            // What is under the lowest layer takes a look at the terrain: only if anything would grow.
            if y == 0 && roll >= (flower_chance + grass_chance) && wild_roll(seed, wx, wz) >= wild::MOST {
                continue;
            }
            let below = if y == 0 { crate::land::surface_block(g, wx, wz) } else { g.get(x, y - 1, z) };
            // Ponds in swamps and floodplains (`vitals::wild::POND_LANDS`): still water a block deep
            // where the ground is level, its floating plants on it.
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| (wx + dx, wz + dz, g.column(wx + dx, wz + dz)));
            // Steep ground goes bare: gravel where it falls three blocks to a neighbour (sand and the
            // frozen north keep their own ground); from four, the terrain's rock faces have made it
            // stone already (`land.rs`), and it isn't plant soil here.
            let fall = near
                .iter()
                .map(|(_, _, c)| col.height - c.height)
                .max()
                .unwrap_or(0);
            if y >= 1 && fall >= 3 && PLANT_SOIL.on_block(below) && !FROZEN.on_biome(b) {
                g.set(x, y - 1, z, GRAVEL.id());
                continue;
            }
            if y >= 1
                && pond(seed, wx, wz, col.height, b)
                && PLANT_SOIL.on_block(below)
                && near.iter().all(|(_, _, c)| c.height >= col.height)
            {
                g.set(x, y - 1, z, WATER.id());
                if y >= 2 && clay(seed, wx, wz, b) {
                    g.set(x, y - 2, z, CLAY.id());
                }
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
            // Floor patches (`FLOOR`): a forest's leaf litter and bare dirt, the dry lands' gravel,
            // in clumps (a noise some 6 blocks across holds them, with holes), twice as thick under
            // the crowns and half in the open.
            let below = match FLOOR.on_biome(b) {
                Some(floor) if y >= 1 && !shore && (PLANT_SOIL.on_block(below) || SAND_TAG.on_block(below)) => {
                    let cover = FLOOR_COVER.on_biome(b).unwrap_or(0.0) as f64 * if under_crown(g, x, y, z) { 2.0 } else { 0.5 };
                    let patch = fixed::perlin2(fixed::seed32(seed, 0xF1C0), wx * 16, wz * 16, 6 * 16) > 0;
                    let fh = hash(seed ^ 0xF1C, wx as i64, 0, wz as i64);
                    match floor.pick(fh >> 32) {
                        Some(f) if patch && unit(fh) < 2.0 * cover && f != below => {
                            g.set(x, y - 1, z, f);
                            f
                        }
                        _ => below,
                    }
                }
                _ => below,
            };
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
                if roll < flower_chance {
                    // The biome's own flowers (`vitals::wild::FLOWERS`), else its `FLOWERS` (another
                    // mod's biome), else dandelions and roses.
                    let kind = flower_kind(seed, wx, wz, h);
                    let flower = crate::wild::flower(name_of(abi::entry_kind::BIOME, b), wild::Ground::Soil, kind).unwrap_or_else(|| {
                        FLOWERS
                            .on_biome(b)
                            .map_or_else(|| [DANDELION.id(), ROSE.id()][kind as usize % 2], |l| l.pick(kind).unwrap_or(0))
                    });
                    if flower != 0 {
                        g.set(x, y, z, flower);
                    }
                } else if roll < flower_chance + grass_chance {
                    g.set(x, y, z, TALL_GRASS.id());
                } else if let Some(ferns) = FERNS.on_biome(b).filter(|&f| f > 0.0) {
                    // Ferns on the bare soil (`FERNS`), three times as thick round a trunk.
                    let thick = if beside_log(g, x, y, z) { 3.0 } else { 1.0 };
                    if unit(hash(seed ^ 0xFE4, wx as i64, 0, wz as i64)) < (ferns * thick) as f64 {
                        g.set(x, y, z, FERN.id());
                    }
                }
            } else if SAND_TAG.on_block(below)
                && roll < flower_chance
                && let Some(flower) =
                    crate::wild::flower(name_of(abi::entry_kind::BIOME, b), wild::Ground::Sand, flower_kind(seed, wx, wz, h))
            {
                // The desert's and the beach's flowers, on their sand.
                g.set(x, y, z, flower);
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

/// Whether leaves hang over this spot of the cube (a tree's crown, a shrub), looking up to the
/// cube's top.
fn under_crown(g: &mut Gen, x: usize, y: usize, z: usize) -> bool {
    (y + 1..16).any(|k| LEAVES.on_block(g.get(x, k, z)))
}

/// Whether a trunk stands beside this spot of the cube (the four sides and the corners).
fn beside_log(g: &mut Gen, x: usize, y: usize, z: usize) -> bool {
    (-1..=1i32).any(|dz| {
        (-1..=1i32).any(|dx| {
            let (nx, nz) = (x as i32 + dx, z as i32 + dz);
            (0..16).contains(&nx) && (0..16).contains(&nz) && LOGS.on_block(g.get(nx as usize, y, nz as usize))
        })
    })
}

/// Whether clay lies under still water at this column of `biome`'s: in 4 x 4 patches, half the
/// swamp's floor, a fifth of other inland water's, none under the sea (sand there).
fn clay(seed: u64, wx: i32, wz: i32, biome: u32) -> bool {
    let share = match name_of(abi::entry_kind::BIOME, biome) {
        "base:swamp" => 0.5,
        "base:ocean" | "base:beach" => 0.0,
        _ => 0.2,
    };
    unit(hash(seed ^ 0xC1A, (wx >> 2) as i64, 0, (wz >> 2) as i64)) < share
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

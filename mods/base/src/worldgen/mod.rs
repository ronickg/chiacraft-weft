//! Base's worldgen as data: biomes, the overworld's terrain, ores, trees (a scatter the engine
//! places in every dimension whose biomes ask for them) and plants (a decoration).
pub mod consts;
mod decor;
pub mod statics;
pub mod tree_algorythms;
pub mod tree_kinds;
mod util;

use consts::*;
use decor::*;
use statics::*;
use tree_algorythms::register as register_trees;
use tree_kinds::*;
use util::gentle;

use weft_sdk::*;

use crate::blocks::*;

use weft_sdk::noise::*;
use weft_sdk::terrain::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Snow line in a temperate climate (temp = 0)
const SNOW_LINE: i32 = 190;
/// Air cools with height, so the warmer the climate the higher the snow line for this many
/// blocks for every step of temperature
const LAPSE: f64 = 100.0;
/// The lowest y the snow line can get in the coldest climates
const SNOW_FLOOR: i32 = 165;
/// Band size of snow line
const BAND: f64 = 0.1;
/// Afaik no mountain gets higher than this so a climate whose snow line is above it has none
const SNOW_TOP: i32 = SEA + 190;

fn snow_bands() -> Vec<(f64, f64, i32)> {
    let coldest = f64::from(SNOW_FLOOR - SNOW_LINE) / LAPSE;
    let edge = |i: usize| coldest + i as f64 * BAND;
    let mut bands = vec![(f64::NEG_INFINITY, coldest, SNOW_FLOOR)];
    for i in 0.. {
        let line = SNOW_LINE + (LAPSE * (edge(i) + BAND / 2.0)).round() as i32;
        if line > SNOW_TOP {
            break;
        }
        bands.push((edge(i), edge(i + 1), line));
    }
    bands
}

/// What a biome grows, as data any mod can set on any biome: trees per 5x5 cell (0..1), share of
/// grass on plantable soil, which trees and flowers (block lists the engine checks), and tags `frozen` (nothing grows, set by
/// `.frozen(..)`), `dry` (cacti).

pub fn register(r: &mut Registry) {
    // Tags say what a biome is (spawning, decoration and mods ask these, not names).
    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    register_trees();
    r.biome(&PLAINS)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tag("spawn_biome")
        .set(&TREES, 0.025)
        .set(&GRASS_COVER, 0.7)
        .set(&TALL_GRASS_COVER, 0.075)
        .set(&BOULDER_COVER, 0.03)
        .set(&BUSH_COVER, 0.03)
        .set(&RASPBERRY_COVER, 0.005)
        .set(&SNOWBERRY_COVER, 0.002)
        .set_text(&FLOWERS, "base:rose base:dandelion base:tulip_orange base:tulip_red base:tulip_yellow base:tulip_white")
        .set(&FLOWER_COVER, 0.02);
    r.biome(&LAVENDER_FIELDS)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .set(&GRASS_COVER, 0.7)
        .set(&TALL_GRASS_COVER, 0.25)
        .set(&BOULDER_COVER, 0.03)
        .set(&BUSH_COVER, 0.02)
        .set(&RASPBERRY_COVER, 0.005)
        .set(&SNOWBERRY_COVER, 0.002)
        .set_text(&FLOWERS, "base:lavender")
        .set(&FLOWER_COVER, 0.1);
    r.biome(&HILLS)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tint(grass, 0x88bb67)
        .tint(foliage, 0x6ba941)
        .set(&TREES, 0.025)
        .set(&TREE_KINDS, TreeKinds::from([("base:spruce", 1)]))
        .set(&GRASS_COVER, 0.5)
        .set(&TALL_GRASS_COVER, 0.05)
        .set(&BOULDER_COVER, 0.06)
        .set(&BLUEGRASS_COVER, 0.2)
        .set_text(&FLOWERS, "base:lavender base:dandelion base:tulip_pink base:tulip_purple base:tulip_white")
        .set(&FLOWER_COVER, 0.02);
    r.biome(&TALLGRASS_MEADOW)
        .surface("grass", "dirt", 3)
        .tag("animals")
        .tag("spawn_biome")
        .set(&TREES, 0.025)
        .set(&GRASS_COVER, 1.0)
        .set(&TALL_GRASS_COVER, 0.7)
        .set(&BOULDER_COVER, 0.03)
        .set(&BUSH_COVER, 0.03)
        .set(&RASPBERRY_COVER, 0.005)
        .set(&SNOWBERRY_COVER, 0.002)
        .set_text(&FLOWERS, "base:dandelion base:inula base:verbascum")
        .set(&FLOWER_COVER, 0.1);
    r.biome(&FOREST)
        .humidity(0.25..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x79c05a)
        .tint(foliage, 0x59ae30)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.4)
        .set(&FERN_COVER, 0.2)
        .set(&GRASS_COVER, 0.4)
        .set(&BUSH_COVER, 0.06)
        .set(&RASPBERRY_COVER, 0.002)
        .set(&FUNGI_COVER, 0.01)
        // .set_text(&FLOWERS, "base:rose base:dandelion base:fern base:fern")
        // .set(&FLOWER_COVER, 0.02)
        .set(&TREE_KINDS, TreeKinds::from([("base:oak", 4), ("base:birch", 1)]));
    r.biome(&BIRCH_FOREST)
        .temperature(..0.1)
        .humidity(0.05..)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x88bb67)
        .tint(foliage, 0x6ba941)
        .tag("forest")
        .tag("animals")
        .set(&TREES, 0.45)
        .set(&FERN_COVER, 0.2)
        .set(&GRASS_COVER, 0.4)
        .set(&BUSH_COVER, 0.06)
        .set(&RASPBERRY_COVER, 0.002)
        .set(&FUNGI_COVER, 0.01)
        // .set_text(&FLOWERS, "base:rose base:dandelion base:fern base:fern")
        // .set(&FLOWER_COVER, 0.02)
        .set(&TREE_KINDS, TreeKinds::from([("base:birch", 1)]));
    r.biome(&TAIGA)
        .temperature(..-0.15)
        .surface("grass", "dirt", 3)
        .tint(grass, 0x86b783)
        .tint(foliage, 0x68a464)
        .tint(water, 0x3d57d6)
        .tag("forest")
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.45)
        .set(&GRASS_COVER, 0.08)
        .set(&TREE_KINDS, TreeKinds::from([("base:spruce", 1)]));
    r.biome(&SNOWY)
        .snows()
        .temperature(..-0.25)
        .surface("snowy_grass", "dirt", 3)
        .frozen("ice")
        .tint(grass, 0x80b497)
        .tint(foliage, 0x60a17b)
        .tint(water, 0x3938c9)
        .tag("cold")
        .tag("animals")
        .set(&TREES, 0.03)
        .set(&TREE_KINDS, TreeKinds::from([("base:spruce", 1)]));
    r.biome(&DESERT)
        .temperature(0.25..)
        .humidity(..0.1)
        .surface("sand", "sand", 3)
        .under("sandstone", 1)
        .tint(grass, 0xbfb755)
        .tint(foliage, 0xaea42a)
        .tag("dry")
        .tag("hot");
    r.biome(&MOUNTAINS)
        .surface("grass", "dirt", 3)
        .bare_above(130)
        .tint(grass, 0x8ab689)
        .tint(foliage, 0x6da36b)
        .tag("mountain")
        .tag("animals")
        .set(&TREES, 0.06)
        .set(&GRASS_COVER, 0.05)
        .set(&TREE_KINDS, TreeKinds::from([("base:spruce", 1)]));
    r.biome(&SNOWY_PEAKS)
        .snows()
        .surface("snow", "snow", 3)
        .tint(grass, 0x80b497)
        .tint(foliage, 0x60a17b)
        .tint(water, 0x3938c9)
        .tag("mountain")
        .tag("cold");
    r.biome(&OCEAN)
        .surface("sand", "sand", 3)
        .deep_top("gravel", SEA - 8)
        .tag("ocean");
    r.biome(&BEACH).surface("sand", "sand", 3).tag("beach");
    // Underground (3D): whole cubes deep under the surface; spiders like it there.
    r.biome(&DEEP_CAVES).tag("cave").tag("dark");

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

    // The overworld's shape. Other mods add to it by these names (`base:desert`, `base:mountains`).
    let terrain = r
        .terrain(&crate::OVERWORLD)
        .stone("stone")
        .base_height(SEA as f64 - 8.0)
        .sea(SEA, "water")
        .field("temperature", Expr::fbm(0x22, 2200.0, 3) * Expr::value(1.4))
        .field("humidity", Expr::fbm(0x33, 1600.0, 3) * Expr::value(1.4))
        .field("base:weirdness", Expr::fbm(0x4A7, 700.0, 3).ramp(-0.2, 0.2))
        .field("base:randomness", Expr::fbm(0x4A7, 700.0, 3).ramp(-0.2, 0.2))
        .landform(preset::continents(18.0))
        // Biomes shape the land through their climate, not their borders, so nothing seams:
        // the cold flattens hills into tundra, deserts get dunes.
        .landform(preset::hills(16.0, 110.0).mask(Expr::temperature().ramp(-0.35, -0.6).not()))
        .landform(
            preset::dunes(4.0, 26.0).mask(
                Expr::temperature()
                    .ramp(0.25, 0.45)
                    .and(Expr::humidity().ramp(0.1, -0.05)),
            ),
        )
        // How mountainous a place is: the mountains' strength, and where the mountains biome goes.
        .field(
            "base:mountains",
            Expr::fbm(0x44, 700.0, 3)
                .ramp(-0.1, 0.3)
                .and(Expr::continent().ramp(0.0, 0.2)),
        )
        .landform(
            preset::mountains(150.0, 260.0)
                .mask(Expr::field("base:mountains"))
                .named("mountains"),
        )
        // Slight jittering so snow doesn't fall on visible contours
        .field(
            "base:alpine",
            Expr::temperature() + Expr::noise(400.0) * Expr::value(0.2) + Expr::noise(70.0) * Expr::value(0.1)
                - Expr::humidity() * Expr::value(0.1),
        )
        // Biomes bring their climate and surface; rules only add where else they must hold.
        .rule(BiomeRule::new("ocean").height(..=SEA - 2))
        .rule(
            BiomeRule::new("snowy_plains")
                .named("snowy_coast")
                .height(..=SEA + 1)
                .continent(..0.1),
        )
        .rule(BiomeRule::new("beach").height(..=SEA + 1).continent(..0.3));
    let mut terrain = terrain;
    for (i, (cold, warm, line)) in snow_bands().into_iter().enumerate() {
        terrain = terrain.rule(
            BiomeRule::new("snowy_peaks")
                .named(&format!("snow_line_{i}"))
                .field("base:mountains", 0.45..)
                .field("base:alpine", cold..warm)
                .height(line..),
        );
    }
    terrain
        .rule(BiomeRule::new("mountains").field("base:mountains", 0.45..))
        .rule(BiomeRule::new("snowy_plains"))
        .rule(BiomeRule::new("taiga"))
        .rule(BiomeRule::new("desert"))
        .rule(BiomeRule::new("forest"))
        .rule(BiomeRule::new("birch_forest"))
        .rule(
            BiomeRule::new("hills")
                .field("base:randomness", 0.5..)
                .height(SEA + 30..),
        )
        .rule(BiomeRule::new("lavender_fields").field("base:weirdness", 1.0..))
        .rule(BiomeRule::new("tallgrass_meadow").field("base:weirdness", 0.5..))
        .rule(BiomeRule::new("plains"))
        .cave_biome("deep_caves", 24, 96.0, 0.25)
        .caves(Caves::new("lava", i32::MIN, 11));

    // Trees wherever a biome asks for them, in any dimension.
    r.scatter(&TREE_SITES)
        .spacing(5)
        .reach(20)
        .density(&TREES)
        .ground("#plant_soil")
        .plan(|s| {
            let kinds = TREE_KINDS.on_biome(s.biome());
            let mut rng = Rng(s.random());
            // Biomes that ask for trees without naming kinds grow oaks, as they always did.
            let name = match &kinds {
                Some(kinds) => kinds.pick(rng.next()),
                None => Some("base:oak"),
            };
            let Some(name) = name else { return };
            let Some(place_tree) = tree_place(name) else { return };

            place_tree(&mut TreePlaceData::new(s));
        });
    r.scatter(&BOULDER_SITES)
        .spacing(5)
        .reach(3)
        .density(&BOULDER_COVER)
        .plan(|s| {
            let BlockPos { x, y, z } = s.pos();
            if y < 70 || !gentle(s, 2, 2) {
                return;
            }
            let mut rng = Rng(s.random());
            let mut cells = Vec::new();
            place_boulder(s, &mut rng, x, y, z, &mut cells);
            if rng.float() < 0.5 {
                let dx = x + (rng.float() * 2.0) as i32;
                let dy = y + (rng.float() * 2.0 - 0.5) as i32;
                let dz = z + (rng.float() * 2.0) as i32;
                place_boulder(s, &mut rng, dx, dy, dz, &mut cells);
            }
            let blocks: BTreeSet<[i32; 3]> = cells.into_iter().collect();
            let mut carpets: BTreeMap<[i32; 3], u8> = BTreeMap::new();
            for b in &blocks {
                for &d in &Dir::ALL {
                    if d == Dir::Down {
                        continue;
                    }
                    let o = d.offset();
                    let n = [b[0] + o.x, b[1] + o.y, b[2] + o.z];
                    if blocks.contains(&n) {
                        continue;
                    }
                    let face = d.opposite().face() as u8;
                    *carpets.entry(n).or_insert(0) |= 1u8 << face;
                }
            }
            for (p, mask) in carpets {
                if rng.float() > 0.2 {
                    continue;
                }
                let props: Vec<(&str, &str)> = Dir::ALL
                    .iter()
                    .map(|&d| {
                        let on = mask & (1u8 << d.face() as u8) != 0;
                        (d.name(), if on { "true" } else { "false" })
                    })
                    .collect();
                let state = MOSS.with(&props);
                if state != u32::MAX {
                    s.put(p, state);
                }
            }
        });
    r.scatter(&BUSH_SITES)
        .spacing(5)
        .reach(3)
        .density(&BUSH_COVER)
        .plan(|s| place_bush(s, |_r| BUSH_LEAVES.id(), BUSH_STEM.id()));
    r.scatter(&RASPBERRY_SITES)
        .spacing(5)
        .reach(3)
        .density(&RASPBERRY_COVER)
        .plan(|s| {
            place_bush(
                s,
                |r| if r.float() < 0.5 { RASPBERRY_LEAVES.id() } else { RASPBERRY_LEAVES.with(&[("berried", "true")]) },
                BUSH_STEM.id(),
            )
        });
    r.scatter(&SNOWBERRY_SITES)
        .spacing(5)
        .reach(3)
        .density(&SNOWBERRY_COVER)
        .plan(|s| {
            place_bush(
                s,
                |r| if r.float() < 0.5 { SNOWBERRY_LEAVES.id() } else { SNOWBERRY_LEAVES.with(&[("berried", "true")]) },
                BUSH_STEM.id(),
            )
        });
    r.scatter(&FUNGI_SITES)
        .spacing(3)
        .reach(1)
        .density(&FUNGI_COVER)
        .plan(|s| {
            let fungi = [&PORCINI_MUSHROOM, &CHANTERELLE_MUSHROOM, &TOADSTOOL_MUSHROOM];
            let mut rng = Rng(s.random());
            s.put(s.pos(), fungi[rng.next() as usize % fungi.len()].id());
        });
    r.decoration(decorate);
}

static FUNGI_SITES: Feature = Feature::named("fungis");
static BUSH_SITES: Feature = Feature::named("bushes");
static SNOWBERRY_SITES: Feature = Feature::named("snowberries");
static RASPBERRY_SITES: Feature = Feature::named("raspberries");
static BOULDER_SITES: Feature = Feature::named("boulders");
static TREE_SITES: Feature = Feature::named("trees");

struct TreeAt {
    x: i32,
    y: i32,
    z: i32,
    log: u32,
    leaves: u32,
    cone: bool,
    height: i32,
}

/// A tree growing from `log` (its `tree` component), `h` deciding the height.
fn tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    let leaves = TREE_LEAVES.on_block(log)?.0;
    let cone = CONE_TREE.on_block(log);
    let height = if cone { 6 + (h % 4) as i32 } else { 6 + (h % 3) as i32 };
    Some(TreeAt { x, y, z, log, leaves, cone, height })
}

/// Calls `put(x, y, z, state, force)` for every block of the tree.
fn tree_blocks(t: &TreeAt, rng: &mut Rng, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let (log, leaves) = (t.log, t.leaves);
    let top = t.y + t.height;
    match t.cone {
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
        let left = top - y;
        if rng.next() % (left as u64) <= 2 {
            if rng.float() < 0.5 {
                put(t.x, y, t.z, log, true);
            } else {
                put(t.x, y, t.z, log, true);
            }
        }
    }
    let mut grow_root = |dx: i32, dz: i32, tall: bool| {
        let x = t.x + dx;
        let z = t.z + dz;
        if tall {
            put(x, t.y + 1, z, log, true);
        }
        put(x, t.y, z, log, true);
        put(x, t.y - 1, z, log, true);
    };
    let root_positions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut root_count = 0;
    for dpos in root_positions {
        if rng.float() < 0.666 {
            grow_root(dpos.0, dpos.1, rng.float() < 0.5);
            root_count += 1;
        }
    }
    if root_count == 0 {
        let root_pos = root_positions[rng.next() as usize % root_positions.len()];
        grow_root(root_pos.0, root_pos.1, rng.float() < 0.5);
    }
    put(t.x, t.y - 1, t.z, log, true);
    put(t.x, t.y - 2, t.z, DIRT.id(), true);
}

/// Places a tree into a live world (saplings). Checks for room first.
pub fn grow_tree(w: World, p: [i32; 3]) {
    let Some(t) = tree_at(OAK_LOG.id(), p[0], p[1], p[2], w.random()) else { return };
    for y in t.y + 1..t.y + t.height {
        if w.block([t.x, y, t.z]) != 0 {
            return;
        }
    }
    let mut rng = Rng((p[0] ^ p[1] ^ p[2]) as u64);
    tree_blocks(&t, &mut rng, |x, y, z, s, overwrite| {
        let cur = w.block([x, y, z]);
        if overwrite || cur == 0 || TALL_GRASS.is(cur) || TALL_TALL_GRASS.is(cur) {
            w.set_block([x, y, z], s);
        }
    });
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

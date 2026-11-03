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

use base_api::{CLIMATE, Climate};
use base_api::{CLOUDS, MOON, STARS, SUN};
use weft_sdk::terrain::{BiomeRule, Mask, preset};
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
static FLOWERS: Component<BlockList> = Component::named("flowers");
/// Share of the ground ferns and horsetails cover (this mod's decoration).
static FERNS: Component<f32> = Component::named("fern_cover");
static PLANT_SOIL: Tag = Tag::named("plant_soil");
static NATURAL: CreativeTab = CreativeTab::named("base:natural");
static BUILDING: CreativeTab = CreativeTab::named("base:building");

/// Sea level, as the overworld's.
const SEA: i32 = 63;

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
    r.tag("plant_soil", &["red_sand"]);

    let (grass, foliage, water) = (&tints::GRASS, &tints::FOLIAGE, &tints::WATER);
    // Wind-blown red dunes: Velociraptor's, and osmolskae's.
    r.biome(&DUNES)
        .set(&CLIMATE, Climate { celsius: 28.0, humidity: 0.1, seasons: 6.0 })
        .humidity(..-0.12)
        .surface("red_sand", "red_sand", 4)
        .under("red_sandstone", 2)
        .tag("dry")
        .tag("hot")
        .set(&TREES, 0.0)
        .set(&GRASS_COVER, 0.0)
        .set(&FERNS, 0.004)
        .set_text(&FLOWERS, "animals:fern");
    // Dry, open land with a few conifers and ferns: between the dunes and the wet.
    r.biome(&SCRUBLAND)
        .set(&CLIMATE, Climate { celsius: 26.0, humidity: 0.25, seasons: 5.0 })
        .humidity(-0.12..0.05)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, 0xb5ad5a)
        .tint(foliage, 0x8f9a3c)
        .tag("hot")
        .set(&TREES, 0.02)
        .set(&GRASS_COVER, 0.1)
        .set_text(&TREE_KINDS, "base:spruce_log")
        .set(&FERNS, 0.06)
        .set_text(&FLOWERS, "animals:fern");
    // Low, wet ground by the rivers: ferns and horsetails, scattered conifers (Utahraptor's).
    r.biome(&FLOODPLAIN)
        .set(&CLIMATE, Climate { celsius: 24.0, humidity: 0.8, seasons: 4.0 })
        .humidity(0.05..)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, 0x7fa64a)
        .tint(foliage, 0x5d8f35)
        .tint(water, 0x4a6e5a)
        .tag("wet")
        .set(&TREES, 0.04)
        .set(&GRASS_COVER, 0.05)
        .set_text(&TREE_KINDS, "base:spruce_log")
        .set(&FERNS, 0.35)
        .set_text(&FLOWERS, "animals:fern");
    // Conifers over a floor of ferns.
    r.biome(&FERN_FOREST)
        .set(&CLIMATE, Climate { celsius: 23.0, humidity: 0.85, seasons: 3.0 })
        .humidity(0.05..)
        .surface("base:grass", "base:dirt", 3)
        .tint(grass, 0x6f9c40)
        .tint(foliage, 0x4f8a2c)
        .tag("forest")
        .set(&TREES, 0.35)
        .set(&GRASS_COVER, 0.03)
        .set_text(&TREE_KINDS, "base:spruce_log")
        .set(&FERNS, 0.3)
        .set_text(&FLOWERS, "animals:fern");
    // High rock: the wyverns'.
    r.biome(&CRAGS)
        .set(&CLIMATE, Climate { celsius: 15.0, humidity: 0.5, seasons: 6.0 })
        .surface("base:grass", "base:dirt", 3)
        .bare_above(115)
        .tint(grass, 0x8aa864)
        .tag("mountain")
        .set(&TREES, 0.04)
        .set_text(&TREE_KINDS, "base:spruce_log")
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

    r.terrain(&CRETACEOUS)
        .stone("base:stone")
        .base_height(SEA as f64)
        .sea(SEA, "base:water")
        .feature(preset::continents(18.0))
        .feature(preset::hills(14.0, 120.0))
        // The dunes where it is dry.
        .feature(preset::dunes(5.0, 24.0).mask(Mask::Humidity(-0.08, -0.16)))
        .feature(
            preset::mountains(140.0, 240.0)
                .mask(
                    Mask::noise(650.0, -0.05, 0.3)
                        .salt(0x5c)
                        .and(Mask::Continent(0.0, 0.2)),
                )
                .named("crags"),
        )
        .biome(BiomeRule::new("base:ocean").height(..=SEA - 2))
        .biome(
            BiomeRule::new("base:beach")
                .height(..=SEA + 1)
                .continent(..0.1),
        )
        .biome(BiomeRule::new("crags").feature("crags", 0.45..))
        .biome(BiomeRule::new("red_dunes"))
        .biome(BiomeRule::new("scrubland"))
        .biome(BiomeRule::new("floodplain").height(..=SEA + 10))
        .biome(BiomeRule::new("fern_forest"));

    r.decorate(ferns).dimension(&CRETACEOUS);
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

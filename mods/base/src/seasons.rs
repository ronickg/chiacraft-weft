//! Grass and leaves follow the calendar: every biome's grass and foliage tints by the time of
//! year and how strongly the place's year swings (`vitals::foliage`), told to the engine
//! (`set_biome_tints`) four times a day, in dimensions with four seasons. Leaves turn by kind
//! (`base_api::trees::leaf_tint`): birches gold, oaks russet, evergreens not at all.

use std::sync::{Mutex, OnceLock};

use base_api::{CLIMATE, Climate, SEASONS, calendars};
use vitals::wild::Turn;
use vitals::{calendar, foliage};
use weft_sdk::*;

/// The colors change a quarter of a day at a time (the client blends every column again on a
/// change).
const EVERY: u64 = calendar::DAY_TICKS / 4;

/// Test only, for comparing with Minecraft + Bliss + Distant Horizons (Ronald, 11-05: "lets make
/// sure our tree colors match the bliss/dh ... As i want the trees also to match ideally for
/// testing"): leaves tinted as Minecraft's (`/rule base:mc_leaf_colours 1`): each base biome's
/// broadleaf leaves its Minecraft biome's foliage colour (the colormap at its temperature and
/// downfall, read off Ronald's own client jar; not muted), spruce-like leaves Minecraft's fixed
/// spruce colour, birch-like its birch colour, and no seasonal turn (Bliss's seasons are off).
/// Water too, Minecraft's own colour (`mc_water`): base's is muted toward the sky it mirrors, for
/// the shader pack, so with no pack it stood grey-blue and flat beside vanilla's saturated blue
/// (Planner's plain-look sheet in Bliss's world, 11-06).
pub static MC_LEAF_COLOURS: Config<u32> = Config::named("mc_leaf_colours");

/// Minecraft's fixed leaf colours (`FoliageColor`): spruce's and birch's.
const MC_SPRUCE: u32 = 0x619961;
const MC_BIRCH: u32 = 0x80a755;

/// Minecraft's water colours (its biomes' `water_color`): most biomes', the swamp's, a warm ocean's.
const MC_WATER: u32 = 0x3f76e4;
const MC_SWAMP_WATER: u32 = 0x617b64;
const MC_WARM_WATER: u32 = 0x43d5ee;

/// Each base biome's Minecraft water colour (its nearest Minecraft biome's).
fn mc_water(id: u32) -> u32 {
    use crate::worldgen::*;
    if id == SWAMP.id() {
        MC_SWAMP_WATER
    } else if id == LAGOON.id() || id == CORAL_REEF.id() {
        MC_WARM_WATER
    } else {
        MC_WATER
    }
}

/// Each base biome's nearest Minecraft biome's foliage colour (its colormap at that biome's
/// temperature and downfall; the swamp's is fixed).
fn mc_foliage() -> Vec<(u32, u32)> {
    use crate::worldgen::*;
    [
        (&PLAINS, 0x77ab2f),            // plains
        (&FOREST, 0x59ae30),            // forest
        (&BIRCH_FOREST, 0x6ba941),      // birch forest
        (&TAIGA, 0x68a464),             // taiga
        (&SNOWY, 0x60a17b),             // snowy plains
        (&DESERT, 0xaea42a),            // desert
        (&SAVANNA, 0xaea42a),           // savanna
        (&SWAMP, 0x6a7039),             // swamp (fixed)
        (&TROPICAL_FOREST, 0x30bb0b),   // jungle
        (&MOUNTAINS, 0x6da36b),         // windswept hills
        (&SEQUOIA_GROVE, 0x68a464),     // old growth spruce taiga
        (&OLD_GROWTH_FOREST, 0x59ae30), // dark forest
        (&OCEAN, 0x71a74d),             // ocean
        (&BEACH, 0x77ab2f),             // beach
    ]
    .into_iter()
    .map(|(b, c)| (b.id(), c))
    .collect()
}

pub fn register(r: &mut Registry) {
    r.config(&MC_LEAF_COLOURS).choice(0, &["off", "on"]).world();
    r.world_tick(tick);
}

/// Every biome with a climate (any mod's), by id, found once.
fn biomes() -> &'static [(u32, Climate)] {
    static ALL: OnceLock<Vec<(u32, Climate)>> = OnceLock::new();
    ALL.get_or_init(|| {
        (0..4096)
            .filter_map(|id| CLIMATE.on_biome(id).map(|c| (id, c)))
            .collect()
    })
}

fn tick(w: &World) {
    // A second at a time, and only on a new quarter day (or a time set by hand), or when the
    // Minecraft leaf colours were turned on or off.
    let mc = MC_LEAF_COLOURS.get() == 1;
    if w.game_time() % 20 != 0 || (!mc && SEASONS.on_dimension(w.dimension()).unwrap_or(0) != calendars::FOUR) {
        return;
    }
    let time = w.day_time();
    let quarter = (time / EVERY) * 2 + mc as u64;
    static TOLD: Mutex<Vec<(u32, u64)>> = Mutex::new(Vec::new());
    {
        let mut told = TOLD.lock().unwrap();
        match told.iter_mut().find(|t| t.0 == w.dimension()) {
            Some(t) if t.1 == quarter => return,
            Some(t) => t.1 = quarter,
            None => told.push((w.dimension(), quarter)),
        }
    }
    let leaf_kinds = [&tints::FOLIAGE, &base_api::trees::EVERGREEN, &base_api::trees::YELLOW_LEAVES];
    if mc {
        let mut tints = Vec::new();
        for (id, colour) in mc_foliage() {
            tints.push((id, &tints::FOLIAGE, colour));
            tints.push((id, &base_api::trees::EVERGREEN, MC_SPRUCE));
            tints.push((id, &base_api::trees::YELLOW_LEAVES, MC_BIRCH));
        }
        for &(id, _) in biomes() {
            tints.push((id, &tints::WATER, mc_water(id)));
        }
        w.set_biome_tints(&tints);
        return;
    }
    let day = (time / EVERY) as f32 / 4.0 % calendar::YEAR_DAYS as f32;
    // Leaves and water as base has them first (after the Minecraft colours were on, they stayed).
    let mut tints: Vec<(u32, &TintKind, u32)> = Vec::new();
    for &(id, _) in biomes() {
        for kind in leaf_kinds.into_iter().chain([&tints::WATER]) {
            tints.push((id, kind, w.content_tint(id, kind)));
        }
    }
    for &(id, c) in biomes() {
        // Where the rain makes the year (the llanos, the Pantanal), the grass follows it.
        let rainy = foliage::rain_strength(c.seasons, c.rain);
        if rainy > 0.0 {
            let base = w.content_tint(id, &tints::GRASS);
            let now = foliage::rain_tint(base, day, c.rain, rainy);
            if now != base {
                tints.push((id, &tints::GRASS, now));
            }
        }
        let strength = foliage::strength(c.seasons);
        if strength <= 0.0 {
            continue;
        }
        // Grass, and the leaves that turn (evergreens keep the biome's color).
        let turning = [Turn::Yellow, Turn::Orange].into_iter().filter_map(|t| {
            t.autumn()
                .map(|a| (base_api::trees::leaf_tint(t), foliage::Part::Leaves { autumn: a }))
        });
        for (kind, part) in std::iter::once((&tints::GRASS, foliage::Part::Grass)).chain(turning) {
            let base = w.content_tint(id, kind);
            let now = foliage::tint(base, part, day, strength);
            if now != base {
                tints.push((id, kind, now));
            }
        }
    }
    w.set_biome_tints(&tints);
}

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

pub fn register(r: &mut Registry) {
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
    // A second at a time, and only on a new quarter day (or a time set by hand).
    if w.time() % 20 != 0 || SEASONS.on_dimension(w.dimension()).unwrap_or(0) != calendars::FOUR {
        return;
    }
    let time = w.day_time();
    let quarter = time / EVERY;
    static TOLD: Mutex<Vec<(u32, u64)>> = Mutex::new(Vec::new());
    {
        let mut told = TOLD.lock().unwrap();
        match told.iter_mut().find(|t| t.0 == w.dimension()) {
            Some(t) if t.1 == quarter => return,
            Some(t) => t.1 = quarter,
            None => told.push((w.dimension(), quarter)),
        }
    }
    let day = quarter as f32 / 4.0 % calendar::YEAR_DAYS as f32;
    let mut tints = Vec::new();
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

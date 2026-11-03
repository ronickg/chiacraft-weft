//! Weather by place and season (`notes/ECOLOGY.md`, part 1): each biome rains on its share of
//! the season's days (`vitals::climate::BIOMES`, from real places), in one spell of a few hours,
//! the same on every machine. A few times a minute this works out where it rains now and tells
//! the engine (`set_rain`), which draws rain only there; bodies, eggs and comfort read it through
//! `base_api::air_at` (`raining_at`). `/weather` still rains everywhere for its time; `/rain`
//! says when it rains today where you stand.

use std::sync::OnceLock;

use base_api::{CLIMATE, Climate, SEASONS};
use vitals::{calendar, climate};
use weft_sdk::*;

/// Ticks between looks: rain starts and stops within a few seconds of its hour.
const EVERY: u64 = 50;

pub fn register(r: &mut Registry) {
    r.world_tick(tick);
    // Today's rain where the player stands, and how often it rains there this season.
    r.command("rain", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let id = w.biome(st.pos.map(|v| v.floor() as i32));
        let Some(c) = CLIMATE.on_biome(id) else { return };
        let (day, _, season) = now(&w);
        let share = Arg::Int((climate::rain_share(c.rain, season) * 100.0).round() as i64);
        match climate::spell(c.rain, season, base_api::rain_key(&w, id), day) {
            Some((from, to)) => cmd.reply("command.base.rain.today", &[Arg::Int(from as i64), Arg::Int(to.ceil() as i64), share]),
            None => cmd.reply("command.base.rain.dry", &[share]),
        }
    });
}

/// The day since the world began, the hour (06:00 at the day's start) and the season (in a
/// dimension with a calendar).
fn now(w: &World) -> (u64, f32, Option<usize>) {
    let time = w.day_time();
    let hour = ((time % calendar::DAY_TICKS) as f32 / 1000.0 + 6.0) % 24.0;
    let season = (SEASONS.on_dimension(w.dimension()).unwrap_or(0) > 0).then(|| calendar::date(time).season.index() as usize);
    (time / calendar::DAY_TICKS, hour, season)
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
    if w.time() % EVERY != 0 {
        return;
    }
    // Without a calendar, the year's rain.
    let (day, hour, season) = now(w);
    let (mut wet, mut thunder) = (Vec::new(), false);
    for &(id, c) in biomes() {
        let key = base_api::rain_key(w, id);
        if climate::rains(c.rain, season, key, day, hour) {
            wet.push(id);
            thunder |= climate::thunders(c.celsius, c.seasons, season, key, day);
        }
    }
    w.set_rain(&wet, thunder);
}

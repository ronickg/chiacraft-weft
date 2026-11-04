//! Weather by place and season (`notes/ECOLOGY.md`, part 1): each biome rains on its share of
//! the season's days (`vitals::climate::BIOMES`, from real places), in one spell of a few hours,
//! the same on every machine. A few times a minute this works out where it rains now and tells
//! the engine (`set_rain`), which draws rain only there; bodies, eggs and comfort read it through
//! `base_api::air_at` (`raining_at`). `/weather` still rains everywhere for its time; `/rain`
//! says when it rains today where you stand.
//!
//! The air goes the same way (`vitals::air`): each biome's fog and wind now, and where the wind
//! blows, told to the engine (`set_air`) for shader packs to draw. `/air` says what it is where you
//! stand, and `/air <fog> <wind> [heading]` sets it everywhere for ten minutes.

use std::sync::{Mutex, OnceLock};

use base_api::{CLIMATE, Climate, SEASONS};
use vitals::{air, calendar, climate};
use weft_sdk::*;

/// Ticks between looks: rain starts and stops within a few seconds of its hour.
const EVERY: u64 = 50;

/// `/air <fog> <wind> [heading]`: fog 0..1 and wind m/s everywhere until the tick it ends.
static SET: Mutex<Option<(f32, f32, f32, u64)>> = Mutex::new(None);

/// How long `/air` holds: ten minutes.
const SET_TICKS: u64 = 12_000;

pub fn register(r: &mut Registry) {
    r.world_tick(tick);
    r.command("air", "[fog:float] [wind:float] [heading:float]", |cmd| {
        let w = cmd.world();
        if let Some(fog) = cmd.float("fog") {
            let wind = cmd.float("wind").unwrap_or(0.0) as f32;
            let heading = cmd.float("heading").map_or_else(|| now_heading(&w), |h| h as f32);
            *SET.lock().unwrap() = Some((fog.clamp(0.0, 1.0) as f32, wind.max(0.0), heading, w.time() + SET_TICKS));
            send(&w);
            cmd.reply("command.base.air.set", &[Arg::Int((fog.clamp(0.0, 1.0) * 100.0).round() as i64), Arg::Int(wind.round() as i64)]);
            return;
        }
        let Some(st) = w.entity(cmd.player()) else { return };
        let id = w.biome(st.pos.map(|v| v.floor() as i32));
        let Some(c) = CLIMATE.on_biome(id) else { return };
        let (fog, wind) = air_of(&w, id, &c);
        let share = Arg::Int((air::fog_share(c.fog, now(&w).2) * 100.0).round() as i64);
        cmd.reply(
            "command.base.air.now",
            &[Arg::Int((fog * 100.0).round() as i64), Arg::Int(wind.round() as i64), Arg::Int(now_heading(&w).round() as i64), share],
        );
    });
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
    send(w);
}

/// Where the wind blows to now, degrees clockwise from north.
fn now_heading(w: &World) -> f32 {
    let (day, hour, _) = now(w);
    air::heading(w.seed(), day, hour)
}

/// The fog (0..1) and the wind (m/s) in biome `id` now.
fn air_of(w: &World, id: u32, c: &Climate) -> (f32, f32) {
    if let Some((fog, wind, _, until)) = *SET.lock().unwrap() {
        if w.time() < until {
            return (fog, wind);
        }
    }
    let (day, hour, season) = now(w);
    let key = base_api::rain_key(w, id);
    let storm = climate::rains(c.rain, season, key, day, hour) && climate::thunders(c.celsius, c.seasons, season, key, day);
    let wind = air::wind(c.wind, season, key, day, hour, storm);
    (air::fog(c.fog, climate::Fog::from_index(c.fog_kind), season, key, day, hour, wind), wind)
}

/// Tells the engine the air in every biome.
fn send(w: &World) {
    let heading = match *SET.lock().unwrap() {
        Some((_, _, h, until)) if w.time() < until => h,
        _ => now_heading(w),
    };
    let list: Vec<(u32, f32, f32)> = biomes()
        .iter()
        .map(|&(id, c)| {
            let (fog, wind) = air_of(w, id, &c);
            (id, fog, wind)
        })
        .collect();
    w.set_air(&list, heading);
}

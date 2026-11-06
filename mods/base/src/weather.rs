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

/// `/air <fog> <wind> [heading] [kind]`: fog 0..1, wind m/s and `air_flags` (snow also lies)
/// everywhere until the tick it ends.
static SET: Mutex<Option<(f32, f32, f32, u32, u64)>> = Mutex::new(None);

/// How long `/air` holds: ten minutes.
const SET_TICKS: u64 = 12_000;

pub fn register(r: &mut Registry) {
    r.world_tick(tick);
    r.command("air", "[fog:float] [wind:float] [heading:float] [kind:word(mist,snow,dust)]", |cmd| {
        let w = cmd.world();
        if let Some(fog) = cmd.float("fog") {
            let wind = cmd.float("wind").unwrap_or(0.0) as f32;
            let heading = cmd
                .float("heading")
                .map_or_else(|| now_heading(&w), |h| h as f32);
            let flags = match cmd.text("kind") {
                Some("snow") => air_flags::SNOWING,
                Some("dust") => air_flags::DUST,
                _ => 0,
            };
            *SET.lock().unwrap() = Some((fog.clamp(0.0, 1.0) as f32, wind.max(0.0), heading, flags, w.game_time() + SET_TICKS));
            send(&w);
            cmd.reply("command.base.air.set", &[Arg::Int((fog.clamp(0.0, 1.0) * 100.0).round() as i64), Arg::Int(wind.round() as i64)]);
            return;
        }
        let Some(st) = w.entity(cmd.player()) else { return };
        let id = w.biome(block_pos_of(st.pos));
        let Some(c) = CLIMATE.on_biome(id) else { return };
        // Where the player stands: blended over the biomes round, calmer in a forest
        // (`base_api::air_at`).
        let here = base_api::air_at(&w, st.pos.into(), 1.8, false, 0, |_| {});
        let share = Arg::Int((air::fog_share(c.fog, now(&w).2) * 100.0).round() as i64);
        cmd.reply(
            "command.base.air.now",
            &[
                Arg::Int((here.fog * 100.0).round() as i64),
                Arg::Int(here.wind.round() as i64),
                Arg::Int(now_heading(&w).round() as i64),
                share,
                Arg::Int(here.celsius.round() as i64),
            ],
        );
    });
    // Today's rain where the player stands, and how often it rains there this season.
    r.command("rain", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let id = w.biome(block_pos_of(st.pos));
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
pub(crate) fn now(w: &World) -> (u64, f32, Option<usize>) {
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
    if w.game_time() % EVERY != 0 {
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
    w.set_rain(&wet, if thunder { &weathers::THUNDER } else { &weathers::RAIN });
    // `/weather rain` rains everywhere for its time, whatever the biomes' spells: seen as a player
    // rained on where their biome has no spell today. Then every biome's air counts as raining, so
    // the cold ones snow (a taiga winter drew rain under it).
    // Only a region with players tells (the others would say no, every region ticking this).
    let players = w.players();
    if !players.is_empty() {
        let manual = w.weather() != weathers::CLEAR.id()
            && players.into_iter().any(|p| {
                w.entity(p).is_some_and(|st| {
                    let at = block_pos_of(st.pos);
                    w.raining_at(at) && !wet.contains(&w.biome(at))
                })
            });
        MANUAL_RAIN.store(manual, std::sync::atomic::Ordering::Relaxed);
    }
    send(w);
}

/// A `/weather rain` is on: it rains in every biome (`tick`).
static MANUAL_RAIN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Where the wind blows to now, degrees clockwise from north.
fn now_heading(w: &World) -> f32 {
    let (day, hour, _) = now(w);
    air::heading(w.seed(), day, hour)
}

/// The wind at `pos` now, m/s, and where it blows to in degrees clockwise from north: its
/// biome's air as the engine is told it (`/air` included). The glider drifts with it.
pub(crate) fn wind_at(w: &World, pos: BlockPos) -> (f32, f32) {
    let heading = match *SET.lock().unwrap() {
        Some((_, _, h, _, until)) if w.game_time() < until => h,
        _ => now_heading(w),
    };
    let id = w.biome(pos);
    let speed = biomes()
        .iter()
        .find(|b| b.0 == id)
        .map_or(0.0, |&(id, c)| air_of(w, id, &c).wind);
    (speed, heading)
}

/// The air in biome `id` now: fog (blizzards and dust storms count as fog), wind, snow lying,
/// and whether it snows or dust blows.
fn air_of(w: &World, id: u32, c: &Climate) -> BiomeAir {
    let aurora = sky_flags(id);
    let (day, hour, season) = now(w);
    let key = base_api::rain_key(w, id);
    let place = air::Place { celsius: c.celsius, humidity: c.humidity, seasons: c.seasons, rain: c.rain, calendar: season.is_some() };
    let snow = place.snow_lying(key, day, hour);
    if let Some((fog, wind, _, flags, until)) = *SET.lock().unwrap() {
        if w.game_time() < until {
            let snow = if flags & air_flags::SNOWING != 0 { snow.max(0.8) } else { snow };
            return BiomeAir { biome: id, fog, wind, snow, flags: flags | aurora };
        }
    }
    let raining = climate::rains(c.rain, season, key, day, hour) || MANUAL_RAIN.load(std::sync::atomic::Ordering::Relaxed);
    // A thunderstorm's wind, and in dry country the hour before it: its gust front.
    let storm = climate::spell(c.rain, season, key, day)
        .is_some_and(|(from, to)| ((from - if c.humidity < air::DRY { 1.0 } else { 0.0 })..to).contains(&hour))
        && climate::thunders(c.celsius, c.seasons, season, key, day);
    let wind = air::wind(c.wind, season, key, day, hour, storm);
    let mut fog = air::fog(c.fog, climate::Fog::from_index(c.fog_kind), season, key, day, hour, wind);
    // A thin dawn mist on every calm, clear morning that is no fog day, at the season's sunrise.
    let sunrise = if season.is_some() {
        12.0 - calendar::daylight_hours((day % calendar::YEAR_DAYS as u64) as f32 + hour / 24.0, base_api::calendars::FOUR_SWING) / 2.0
    } else {
        6.0
    };
    fog = fog.max(air::dawn_mist(c.humidity, sunrise, hour, wind, raining));
    let mut flags = 0;
    if raining && place.air(day, hour, true) < air::SNOWS_BELOW {
        flags |= air_flags::SNOWING;
        fog = fog.max(air::blizzard(wind) * air::BLIZZARD_FOG);
    }
    let dust = air::dust(c.humidity, wind, raining);
    if dust > 0.0 {
        flags |= air_flags::DUST;
        fog = fog.max(dust * 0.95);
    }
    // How fast the ground dries here after rain (the engine's wet ground, `Scene::canopy.y`).
    let drying = air::drying_byte(air::drying_secs(place.air(day, hour, raining), c.humidity, wind));
    BiomeAir { biome: id, fog, wind, snow, flags: flags | aurora | drying << air_flags::DRYING_SHIFT }
}

/// Biome `id`'s share in a per-biome list (`air::AURORA`, `air::CANOPY`) as 0 to 255, 0 for one
/// not listed; `ids` caches the list's biome ids.
fn share_of(ids: &'static OnceLock<Vec<(u32, u32)>>, list: &[(&str, f32)], id: u32) -> u32 {
    ids.get_or_init(|| {
        list.iter()
            .map(|&(name, s)| (Biome::generated(name.to_string()).id(), (s.clamp(0.0, 1.0) * 255.0) as u32))
            .collect()
    })
    .iter()
    .find(|a| a.0 == id)
    .map_or(0, |a| a.1)
}

/// How strong an aurora is in biome `id` on clear nights (`air::AURORA`, by latitude), and how
/// closed its canopy is (`air::CANOPY`), as `air_flags` bits.
fn sky_flags(id: u32) -> u32 {
    static AURORA: OnceLock<Vec<(u32, u32)>> = OnceLock::new();
    static CANOPY: OnceLock<Vec<(u32, u32)>> = OnceLock::new();
    share_of(&AURORA, &air::AURORA, id) << air_flags::AURORA_SHIFT | share_of(&CANOPY, &air::CANOPY, id) << air_flags::CANOPY_SHIFT
}

/// Tells the engine the air in every biome.
fn send(w: &World) {
    let heading = match *SET.lock().unwrap() {
        Some((_, _, h, _, until)) if w.game_time() < until => h,
        _ => now_heading(w),
    };
    let list: Vec<BiomeAir> = biomes().iter().map(|&(id, c)| air_of(w, id, &c)).collect();
    *SNOW.lock().unwrap() = list.iter().map(|a| (a.biome, a.snow)).collect();
    w.set_air(&list, heading);
}

/// The snow lying in each biome as last told to the engine (`send`), 0 bare to 1 deep.
static SNOW: Mutex<Vec<(u32, f32)>> = Mutex::new(Vec::new());

/// The snow lying in biome `id` now (`vitals::air::Place::snow_lying`, or `/air`'s), 0 to 1;
/// `None` before the weather's first look or for a biome without a climate.
pub fn snow_in(id: u32) -> Option<f32> {
    SNOW.lock().unwrap().iter().find(|a| a.0 == id).map(|a| a.1)
}

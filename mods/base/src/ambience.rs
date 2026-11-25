//! What the land sounds like where the player stands (`notes/SYSTEMS.md`, ambience; the numbers in
//! `vitals::ambience`): six looped beds (birds by day, the dawn chorus, insects at night, dry wind,
//! a swamp, surf), calls scattered round the player (songbirds, owls, ravens, gulls, frogs, cave
//! drips, lava pops), and loops on the flowing water, still water and lava near the player.
//!
//! The server works out every `REFRESH` seconds what each player's beds are set to (it knows the
//! biomes around, the air, the hour, the season and the rain) and puts it on the player. The client
//! eases the beds there, through the same cover as the weather (`weather_sound::cover_now`): muffled
//! indoors, gone deep under ground, where only drips are heard. The water it finds itself.

use std::sync::Mutex;

use base_api::{SEA, SEASONS};
use vitals::ambience::{self as amb, CALLS};
use vitals::weather_sound as ws;
use vitals::{calendar, fireflies as ff};
use weft_sdk::*;

/// The beds' levels for this player, the server's (`vitals::ambience::Levels`): the six beds,
/// then owls and ravens.
static HERE: Component<[f32; 8]> = Component::named("ambience");

/// Seconds between the server's looks.
const REFRESH: f32 = 2.0;
/// Seconds between the client's looks at the water; each change eases over `EASE`.
const LOOK: f32 = 0.5;
const EASE: f32 = 1.5;
const FADE_IN: f32 = 3.0;
const FADE_OUT: f32 = 4.0;
/// Quieter than this a loop is stopped, giving its voice back.
const SILENT: f32 = 0.01;
/// Seconds between log lines while anything plays (`ambience:` in the log).
const LOG_EVERY: f32 = 10.0;
const EYE: f64 = 1.62;

static BEDS: [Sound; 6] = [
    Sound::named("ambient.birds"),
    Sound::named("ambient.dawn"),
    Sound::named("ambient.night"),
    Sound::named("ambient.dry"),
    Sound::named("ambient.swamp"),
    Sound::named("ambient.surf"),
];
/// Flowing water, still water, lava.
static LIQUIDS: [Sound; 3] = [Sound::named("ambient.water_flow"), Sound::named("ambient.water_still"), Sound::named("ambient.lava")];
/// One per `vitals::ambience::CALLS`, in its order.
static CALL_SOUNDS: [Sound; 7] = [
    Sound::named("ambient.call.bird"),
    Sound::named("ambient.call.owl"),
    Sound::named("ambient.call.raven"),
    Sound::named("ambient.call.gull"),
    Sound::named("ambient.call.frog"),
    Sound::named("ambient.call.drip"),
    Sound::named("ambient.call.lava_pop"),
];

pub fn register(r: &mut Registry) {
    use sound_categories::AMBIENT;
    r.entity_component(&HERE, [0.0; 8], component_flags::SYNC_OWNER);
    r.edit(&PLAYER).attach(&HERE);
    // Under the weather's beds (50): a storm is never crowded out by birds.
    for (s, file, volume) in [
        (&BEDS[amb::BIRDS], "ambient/birds1", 0.7),
        (&BEDS[amb::DAWN], "ambient/dawn1", 0.8),
        (&BEDS[amb::NIGHT], "ambient/night1", 0.7),
        (&BEDS[amb::DRY], "ambient/dry1", 0.7),
        (&BEDS[amb::SWAMP], "ambient/swamp1", 0.7),
        (&BEDS[amb::SURF], "ambient/surf1", 0.9),
    ] {
        r.sound(s)
            .files(&[file])
            .category(&AMBIENT)
            .volume(volume)
            .priority(40)
            .voices(2);
    }
    for (s, file, volume, range) in [
        (&LIQUIDS[0], "ambient/water_flow1", 0.8, 14.0),
        (&LIQUIDS[1], "ambient/water_still1", 0.6, 12.0),
        (&LIQUIDS[2], "ambient/lava1", 0.9, 12.0),
    ] {
        r.sound(s)
            .files(&[file])
            .category(&AMBIENT)
            .volume(volume)
            .range(range)
            .priority(40)
            .voices(2);
    }
    let calls: [(&[&str], f32, f32, f32); 7] = [
        (&["ambient/call/bird1", "ambient/call/bird2", "ambient/call/bird3", "ambient/call/bird4"], 0.5, 0.9, 1.15),
        (&["ambient/call/owl1", "ambient/call/owl2"], 0.6, 0.95, 1.05),
        (&["ambient/call/raven1", "ambient/call/raven2"], 0.6, 0.9, 1.1),
        (&["ambient/call/gull1"], 0.6, 0.85, 1.15),
        (&["ambient/call/frog1", "ambient/call/frog2", "ambient/call/frog3"], 0.5, 0.85, 1.15),
        (&["ambient/call/drip1", "ambient/call/drip2", "ambient/call/drip3"], 0.5, 0.8, 1.2),
        (&["ambient/call/lava_pop1", "ambient/call/lava_pop2"], 0.6, 0.8, 1.2),
    ];
    for (s, (files, volume, lo, hi)) in CALL_SOUNDS.iter().zip(calls) {
        r.sound(s)
            .files(files)
            .category(&AMBIENT)
            .volume(volume)
            .pitch(lo, hi)
            .range(32.0)
            .priority(20)
            .voices(3);
    }
    r.world_tick(tick);
    r.client_frame(frame);
    // What the land sounds like here, before the cover: the beds' levels and the biome (also a
    // log line, `ambience here:`, for scripts).
    r.command("ambience", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let l = levels_at(&w, st.pos);
        let biome = name_of(abi::entry_kind::BIOME, w.biome(st.pos.block().to_array()));
        let pc = |v: f32| Arg::Int((v * 100.0).round() as i64);
        info!(
            "ambience here: {biome} birds {:.2} dawn {:.2} night {:.2} dry {:.2} swamp {:.2} surf {:.2} owls {:.2} ravens {:.2}",
            l.beds[0], l.beds[1], l.beds[2], l.beds[3], l.beds[4], l.beds[5], l.owls, l.ravens
        );
        let mut args = vec![Arg::Text(biome)];
        args.extend(l.beds.iter().map(|&v| pc(v)));
        args.push(pc(l.owls));
        args.push(pc(l.ravens));
        cmd.reply("command.base.ambience", &args);
    });
}

/// Every `REFRESH` seconds, each player's beds (`vitals::ambience::levels`) on the player. Only
/// players the world shows: check in creative, not spectator.
fn tick(w: &World) {
    if !w.game_time().is_multiple_of((REFRESH * 20.0) as u64) {
        return;
    }
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let l = levels_at(w, st.pos);
        let mut out = [0.0; 8];
        out[..6].copy_from_slice(&l.beds);
        out[6] = l.owls;
        out[7] = l.ravens;
        let Some(here) = w.component_mut(player, &HERE) else { continue };
        if *here != out {
            *here = out;
        }
    }
}

fn levels_at(w: &World, pos: Vec3) -> amb::Levels {
    let block = pos.block().to_array();
    let scape_at = |dx: i32, dz: i32| {
        let at = [block[0] + dx, block[1], block[2] + dz];
        amb::scape(name_of(abi::entry_kind::BIOME, w.biome(at)))
    };
    let rings: Vec<[amb::Scape; 4]> = amb::AROUND
        .iter()
        .map(|&r| [scape_at(r, 0), scape_at(-r, 0), scape_at(0, r), scape_at(0, -r)])
        .collect();
    let m = amb::mix(scape_at(0, 0), &rings);
    let date = calendar::date(w.day_time());
    let hour = (date.fraction * 24.0 + 6.0) % 24.0;
    let swing = base_api::calendars::swing(SEASONS.on_dimension(w.dimension()).unwrap_or(0));
    let sunset = ff::sunset(date.day as f32 + date.fraction, swing);
    let sunrise = 24.0 - sunset;
    let season = (SEASONS.on_dimension(w.dimension()).unwrap_or(0) > 0).then(|| date.season.index() as usize);
    let air = base_api::air_at(w, pos.to_array(), 0.2, false, 0, |_| {}).celsius;
    let raining = w.raining_at(block);
    let rain = if raining { 1.0 } else { 0.0 };
    let thunder = if raining && w.weather() == weathers::THUNDER.id() { 1.0 } else { 0.0 };
    amb::levels(m, hour, sunrise, sunset, air, season, rain, thunder, (block[1] - SEA) as f32)
}

struct State {
    beds: [SoundHandle; 6],
    liquids: [SoundHandle; 3],
    /// Where the liquids' loops sit now, and their levels.
    liquid_at: [Option<[f64; 3]>; 3],
    liquid_levels: [f32; 3],
    /// A few lava blocks found, for the pops.
    lava_spots: Vec<[f64; 3]>,
    /// Seconds since each call was last heard.
    since: [f32; 7],
    next_look: f32,
    next_log: f32,
    logged_playing: bool,
    calls: u32,
    rng: u64,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn frame(c: &Client) {
    let st = c.state();
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if st.flags & client::client_flags::JOINED == 0 {
        *guard = None;
        return;
    }
    let s = guard.get_or_insert_with(|| State {
        beds: [SoundHandle::default(); 6],
        liquids: [SoundHandle::default(); 3],
        liquid_at: [None; 3],
        liquid_levels: [0.0; 3],
        lava_spots: Vec::new(),
        since: [99.0; 7],
        next_look: 0.0,
        next_log: 0.0,
        logged_playing: false,
        calls: 0,
        rng: 0xA5A5_5A5A_D1B5_4A33 ^ st.frame,
    });
    let dt = st.dt.clamp(0.0, 0.5);
    let eye = [st.pos.x, st.pos.y + EYE, st.pos.z];
    let here = c.view().component(&HERE).unwrap_or([0.0; 8]);
    let cover = crate::weather_sound::cover_now();
    // Deep under ground: no sky through any roof over the head.
    let deep = cover.open < 0.05 && cover.roof == 0.0;
    let heard = if deep { 0.0 } else { ws::THROUGH_WALLS + (1.0 - ws::THROUGH_WALLS) * cover.open };
    let muffle = ws::muffle(cover.open);
    for t in &mut s.since {
        *t += dt;
    }
    // The calls, rolled every frame.
    let lava_pop = if s.lava_spots.is_empty() { 0.0 } else { s.liquid_levels[2] };
    let call_levels = [
        ((here[amb::BIRDS] + here[amb::DAWN] * 1.5).min(1.0)) * heard,
        here[6] * heard,
        here[7] * heard,
        here[amb::SURF] * heard,
        here[amb::SWAMP] * heard,
        if deep { 1.0 } else { 0.0 },
        lava_pop,
    ];
    for (i, (call, level)) in CALLS.iter().zip(call_levels).enumerate() {
        if level < SILENT || s.since[i] < amb::CALL_GAP || random(&mut s.rng) >= amb::call_chance(level, call.every, dt) {
            continue;
        }
        s.since[i] = 0.0;
        s.calls += 1;
        let at = if call.name == "lava_pop" {
            let k = (random(&mut s.rng) * s.lava_spots.len() as f32) as usize;
            let p = s.lava_spots[k.min(s.lava_spots.len() - 1)];
            [p[0], p[1] + lerp(call.up, random(&mut s.rng)) as f64, p[2]]
        } else {
            let angle = random(&mut s.rng) * std::f32::consts::TAU;
            let d = lerp(call.dist, random(&mut s.rng));
            [eye[0] + (angle.cos() * d) as f64, eye[1] + lerp(call.up, random(&mut s.rng)) as f64, eye[2] + (angle.sin() * d) as f64]
        };
        let volume = if call.name == "drip" || call.name == "lava_pop" { 1.0 } else { level.sqrt() };
        let h = c.start_sound(&CALL_SOUNDS[i], Some(at), volume, false, 0.0);
        if !deep && call.name != "lava_pop" {
            h.muffle(c, muffle, 0.0);
        }
    }
    s.next_look -= dt;
    if s.next_look > 0.0 {
        return;
    }
    s.next_look = LOOK;
    // The beds, eased toward the server's levels through the cover.
    let mut playing = false;
    for (i, bed) in s.beds.iter_mut().enumerate() {
        let level = here[i] * heard;
        playing |= ease(c, bed, &BEDS[i], None, level, Some(muffle));
    }
    // The water and lava near, found every look.
    find_liquids(c, s, eye);
    for (i, (h, sound)) in s.liquids.iter_mut().zip(&LIQUIDS).enumerate() {
        playing |= ease(c, h, sound, s.liquid_at[i], s.liquid_levels[i], None);
    }
    s.next_log -= LOOK;
    if (playing || s.logged_playing) && s.next_log <= 0.0 {
        s.next_log = LOG_EVERY;
        s.logged_playing = playing;
        info!(
            "ambience: birds {:.2} dawn {:.2} night {:.2} dry {:.2} swamp {:.2} surf {:.2} owls {:.2} ravens {:.2} (heard {:.2}, deep {deep}, muffle {:.0} Hz); water flowing {:.2} still {:.2} lava {:.2}; {} calls{}",
            here[0] * heard,
            here[1] * heard,
            here[2] * heard,
            here[3] * heard,
            here[4] * heard,
            here[5] * heard,
            here[6] * heard,
            here[7] * heard,
            heard,
            muffle,
            s.liquid_levels[0],
            s.liquid_levels[1],
            s.liquid_levels[2],
            s.calls,
            if playing { "" } else { ", all quiet" }
        );
    }
}

/// Eases one loop to `level`: started when it should be heard, stopped when not, moved to `at`
/// (a positioned one). Says whether it plays.
fn ease(c: &Client, h: &mut SoundHandle, sound: &Sound, at: Option<[f64; 3]>, level: f32, muffle: Option<f32>) -> bool {
    let on = h.playing(c);
    if level < SILENT {
        if on {
            h.stop(c, FADE_OUT);
            *h = SoundHandle::default();
        }
        return false;
    }
    if !on {
        *h = c.start_sound(sound, at, level, true, FADE_IN);
    } else {
        h.volume(c, level, EASE);
        if let Some(p) = at {
            h.moved(c, p, EASE);
        }
    }
    if let Some(hz) = muffle {
        h.muffle(c, hz, EASE);
    }
    true
}

/// Counts flowing water, still water open to the air and lava round the head (every other block
/// across, every block up and down), and puts each loop at the middle of what it found.
fn find_liquids(c: &Client, s: &mut State, eye: [f64; 3]) {
    let head = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
    let r = amb::LIQUID_REACH;
    let mut n = [0u32; 3];
    let mut sum = [[0.0f64; 3]; 3];
    s.lava_spots.clear();
    for dx in (-r..=r).step_by(2) {
        // Every height: a pool or lava sunk in the floor is one block thick.
        for dy in -r / 2..=r / 2 {
            for dz in (-r..=r).step_by(2) {
                let p = [head[0] + dx, head[1] + dy, head[2] + dz];
                let Some(b) = c.block(p) else { continue };
                if b == 0 || liquid_of(b).is_none() {
                    continue;
                }
                let name = block_name(b);
                let kind = if name.contains("lava") {
                    2
                } else if name.contains("water") {
                    let source = block_info(b).is_some_and(|i| i.first_state == b);
                    if !source {
                        0
                    } else if c.block([p[0], p[1] + 1, p[2]]).is_some_and(|a| a == 0) {
                        1
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                n[kind] += 1;
                for k in 0..3 {
                    sum[kind][k] += p[k] as f64 + 0.5;
                }
                if kind == 2 && s.lava_spots.len() < 16 {
                    s.lava_spots
                        .push([p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5]);
                }
            }
        }
    }
    s.liquid_levels = amb::liquids(n[0], n[1], n[2]);
    for k in 0..3 {
        s.liquid_at[k] = (n[k] > 0).then(|| [sum[k][0] / n[k] as f64, sum[k][1] / n[k] as f64, sum[k][2] / n[k] as f64]);
    }
}

fn lerp(r: (f32, f32), t: f32) -> f32 {
    r.0 + (r.1 - r.0) * t
}

fn random(r: &mut u64) -> f32 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    (*r >> 40) as f32 / (1u64 << 24) as f32
}

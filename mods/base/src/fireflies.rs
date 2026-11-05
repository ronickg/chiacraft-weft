//! Fireflies on warm nights (`notes/NIGHT_GLOW.md`, Research round 17's recommendation 1; the
//! numbers in `vitals::fireflies`). The server works out every few seconds how many fly round each
//! player, as a share of the full count (the client knows neither biome, air nor soil), and puts it
//! on the player. The client keeps that share of `base:fx_fireflies`' count alive as particles, in
//! clusters over the ground near the player, away from torches and under open sky. They give no
//! light; the pack's particle shader makes them blink (Lighting).

use std::sync::Mutex;
use std::time::Instant;

use vitals::{calendar, fireflies as ff};
use weft_sdk::*;

/// Art's firefly: a speck of yellow-green light, three frames.
pub static FIREFLY: Particle = Particle::named("firefly");
/// Fireflies: 0 off, 1 few, 2 many (`vitals::fireflies::COUNTS`).
pub static FX_FIREFLIES: Config<i32> = Config::named("fx_fireflies");
/// The share of the full count that fly round this player now (0 to 1), the server's.
static HERE: Component<f32> = Component::named("fireflies");

pub fn register(r: &mut Registry) {
    r.config(&FX_FIREFLIES).int(1, 0, 2);
    r.entity_component(&HERE, 0.0, component_flags::SYNC_OWNER);
    r.attach(&PLAYER, &HERE);
    r.particle(&FIREFLY)
        .textures(&["firefly_0", "firefly_1", "firefly_2"])
        .life(ff::LIFE_SECS.0, ff::LIFE_SECS.1)
        .size(0.05, 0.08)
        .gravity(0.0)
        .drag(0.96)
        .fullbright();
    r.world_tick(count);
    r.client_frame(frame);
    // How many fly here now, and why: the inputs of `vitals::fireflies::share` where you stand.
    r.command("fireflies", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let n = Here::at(&w, st.pos);
        // What the server last put on you (`count`): the client's share.
        let on_you = w
            .component(Holder::entity(cmd.player()), &HERE)
            .map_or(-1, |v| (v * 100.0).round() as i64);
        cmd.reply(
            "command.base.fireflies",
            &[
                Arg::Int((n.share * 100.0).round() as i64),
                Arg::Text(n.biome),
                Arg::Float(n.air as f64, 1),
                Arg::Float(n.mean as f64, 1),
                Arg::Int((n.soil * 100.0).round() as i64),
                Arg::Float(n.hour as f64, 1),
                Arg::Float(n.sunset as f64, 1),
                Arg::Int(on_you),
            ],
        );
    });
}

/// What sets the fireflies where a player stands.
struct Here {
    biome: &'static str,
    air: f32,
    mean: f32,
    soil: f32,
    raining: bool,
    hour: f32,
    sunset: f32,
    share: f32,
}

impl Here {
    fn at(w: &World, pos: Vec3) -> Here {
        let date = calendar::date(w.day_time());
        let hour = (date.fraction * 24.0 + 6.0) % 24.0;
        let swing = base_api::calendars::swing(base_api::SEASONS.on_dimension(w.dimension()).unwrap_or(0));
        let sunset = ff::sunset(date.day as f32 + date.fraction, swing);
        let block = pos.block().to_array();
        let biome = name_of(abi::entry_kind::BIOME, w.biome(block));
        let ground = base_api::ground_at(w, block);
        let air = base_api::air_at(w, pos.to_array(), 0.2, false, 0, |_| {}).celsius;
        let raining = w.raining_at(block);
        let share = ff::share(biome, air, ground.mean_air, ground.soil, raining, hour, sunset);
        Here { biome, air, mean: ground.mean_air, soil: ground.soil, raining, hour, sunset, share }
    }
}

/// Every `REFRESH_SECS`, each player's share (`vitals::fireflies::share`). Only players the world
/// shows (`w.players()`): a spectator is hidden, so none fly round one (the sync looks were
/// spectators and saw none; check them in creative).
fn count(w: &World) {
    if !w
        .game_time()
        .is_multiple_of((ff::REFRESH_SECS * 20.0) as u64)
    {
        return;
    }
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let n = Here::at(w, st.pos);
        // In place, as survival keeps its vitals.
        let Some(here) = w.component_mut(player, &HERE) else { continue };
        if *here != n.share {
            weft_sdk::debug!(
                "fireflies: player {player} in {}: air {:.1}, mean {:.1}, soil {:.2}, rain {}, {:.2} h, sunset {:.2}: share {:.2}",
                n.biome,
                n.air,
                n.mean,
                n.soil,
                n.raining,
                n.hour,
                n.sunset,
                n.share
            );
            *here = n.share;
        }
    }
}

/// Where the client's fireflies are: clusters with the time they go, and how many it owes.
struct Swarm {
    last: Option<Instant>,
    owed: f32,
    clusters: Vec<([f64; 3], Instant)>,
    seed: u64,
    /// Why spots were turned down since the last log (no ground, not air, no light, dark or
    /// lit, too near another), the particles sent, and when it last logged.
    missed: [u32; 5],
    sent: u32,
    logged: Option<Instant>,
}

static SWARM: Mutex<Swarm> =
    Mutex::new(Swarm { last: None, owed: 0.0, clusters: Vec::new(), seed: 0x9E37_79B9_7F4A_7C15, missed: [0; 5], sent: 0, logged: None });

impl Swarm {
    /// 0 to 1.
    fn unit(&mut self) -> f64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Seconds a cluster stays before it moves on.
const CLUSTER_SECS: f32 = 30.0;

fn frame(c: &Client) {
    let mut s = SWARM.lock().unwrap();
    let now = Instant::now();
    let dt = s
        .last
        .map_or(0.0, |l| now.duration_since(l).as_secs_f32())
        .min(0.25);
    s.last = Some(now);
    let setting = FX_FIREFLIES.get().clamp(0, 2) as usize;
    let view = c.view();
    let (me, share) = (view.player_state(), view.component(&HERE));
    if s.logged
        .is_none_or(|l| now.duration_since(l).as_secs_f32() >= 5.0)
        && (me.is_none() || share.is_none())
    {
        weft_sdk::debug!("fireflies: no player state ({}) or share ({:?}) on the client", me.is_some(), share);
        s.logged = Some(now);
    }
    let (Some(me), Some(share)) = (me, share) else { return };
    let want = ff::COUNTS[setting] as f32 * share;
    if want < 0.5 {
        s.clusters.clear();
        s.owed = 0.0;
        return;
    }
    // Every 5 s a line for the logs: what it wants and what it found.
    if s.logged
        .is_none_or(|l| now.duration_since(l).as_secs_f32() >= 5.0)
    {
        weft_sdk::debug!(
            "fireflies: share {share:.2}, want {want:.0}, clusters {}, sent {}, spots turned down (no ground, not air, no light, sky or torch, crowded) {:?}",
            s.clusters.len(),
            s.sent,
            s.missed
        );
        s.logged = Some(now);
        s.missed = [0; 5];
        s.sent = 0;
    }
    let me = [me.pos.x, me.pos.y, me.pos.z];
    let reach = ff::REACH as f64;
    // Clusters gone by or left behind make room for new ones.
    s.clusters
        .retain(|(p, until)| *until > now && (p[0] - me[0]).hypot(p[2] - me[2]) <= reach);
    let mean_cluster = (ff::CLUSTER.0 + ff::CLUSTER.1) as f32 / 2.0;
    let clusters = (want / mean_cluster).ceil() as usize;
    for _ in 0..4 {
        if s.clusters.len() >= clusters {
            break;
        }
        if let Some(p) = spot(c, &mut s, me) {
            let stays = CLUSTER_SECS * (0.5 + s.unit() as f32);
            s.clusters
                .push((p, now + std::time::Duration::from_secs_f32(stays)));
        }
    }
    if s.clusters.is_empty() {
        return;
    }
    // As many born a second as die: the count over a particle's mean life.
    let life = (ff::LIFE_SECS.0 + ff::LIFE_SECS.1) / 2.0;
    s.owed += want * dt / life;
    while s.owed >= 1.0 {
        s.owed -= 1.0;
        let k = (s.unit() * s.clusters.len() as f64) as usize % s.clusters.len();
        let p = s.clusters[k].0;
        let at = [p[0] + (s.unit() - 0.5) * 3.0, p[1] + (s.unit() - 0.5) * 1.5, p[2] + (s.unit() - 0.5) * 3.0];
        let drift = ff::DRIFT.0 + (ff::DRIFT.1 - ff::DRIFT.0) * s.unit() as f32;
        let angle = s.unit() * std::f64::consts::TAU;
        let v = [angle.cos() * drift as f64, (s.unit() - 0.5) * drift as f64, angle.sin() * drift as f64];
        drop(c.spawn_particles(&FIREFLY, at).velocity(v));
        s.sent += 1;
    }
}

/// A new cluster's middle: over the top block of a random column within reach, in air, away from
/// torches and under open sky, and apart from the other clusters.
fn spot(c: &Client, s: &mut Swarm, me: [f64; 3]) -> Option<[f64; 3]> {
    let reach = ff::REACH as f64;
    let (dx, dz) = ((s.unit() * 2.0 - 1.0) * reach, (s.unit() * 2.0 - 1.0) * reach);
    let (x, z) = ((me[0] + dx).floor() as i32, (me[2] + dz).floor() as i32);
    let Some(top) = (-12..=8)
        .rev()
        .map(|dy| me[1].floor() as i32 + dy)
        .find(|&y| {
            c.block([x, y, z])
                .is_some_and(|b| b != 0 && liquid_of(b).is_none() && state_flags(b) & abi::block_flags::SOLID != 0)
        })
    else {
        s.missed[0] += 1;
        return None;
    };
    let hover = ff::HOVER.0 + (ff::HOVER.1 - ff::HOVER.0) * s.unit() as f32;
    let y = top as f64 + 1.0 + hover as f64;
    let at = [x, y.floor() as i32, z];
    if c.block(at) != Some(0) {
        s.missed[1] += 1;
        return None;
    }
    let Some(light) = c.light(at) else {
        s.missed[2] += 1;
        return None;
    };
    let (sky, torch) = (
        light & 0xFF,
        (light >> 8 & 0xFF)
            .max(light >> 16 & 0xFF)
            .max(light >> 24 & 0xFF),
    );
    if sky < ff::SKY_MIN || torch > ff::TORCH_MAX {
        s.missed[3] += 1;
        return None;
    }
    let p = [x as f64 + 0.5, y, z as f64 + 0.5];
    let apart = s
        .clusters
        .iter()
        .all(|(q, _)| (q[0] - p[0]).hypot(q[2] - p[2]) >= ff::CLUSTER_GAP as f64);
    if !apart {
        s.missed[4] += 1;
    }
    apart.then_some(p)
}

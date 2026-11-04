//! The wyvern's breath (`notes/ANIMALS.md`, "The wyvern"): what it breathes is in its genes
//! (`coats::breath`): fire, frost, lightning or venom. Its rider makes it breathe with a key (R by
//! default) where they look; a wild one breathes at what it is angry with. On the ground it rears,
//! draws breath and lunges (`breathe`, the stream from 0.95 to 2.6 s); in the air it breathes as it
//! flies. The stream reaches 14 blocks in a cone from its mouth:
//!
//! - **fire** burns what it touches (and sets it alight); scorches grass to dirt, burns leaves,
//!   plants and snow away, melts ice.
//! - **frost** hurts less, slows what it touches and puts its fire out; water freezes, grass
//!   frosts over.
//! - **lightning** strikes the nearest thing in it hard, and arcs on to two more near it.
//! - **venom** hurts and poisons (more hurt each second for five seconds); grass and leaves wither.

use std::collections::HashMap;
use std::sync::Mutex;

use weft_sdk::*;

use crate::coats::{self, GENOME};

value! {
    #[derive(Default)]
    /// A breath under way (not saved).
    pub struct Breath {
        /// Ticks since it began + 1 (0: not breathing).
        pub t: u32,
        /// Ticks before it can breathe again.
        pub cooldown: u32,
        /// It breathes flying (no rearing, the stream at once).
        pub flying: u32,
        /// What it breathes at, unridden (0: straight ahead).
        pub target: u64,
    }
}

pub static BREATH: Component<Breath> = Component::named("breath");
static KEY: Key = Key::named("breathe");
static FLAME: Particle = Particle::named("breath_fire");
static FROST: Particle = Particle::named("breath_frost");
static SPARK: Particle = Particle::named("breath_lightning");
static VENOM: Particle = Particle::named("breath_venom");

static AIR: Block = Block::named("core:air");
static DIRT: Block = Block::named("base:dirt");
static GRASS: Block = Block::named("base:grass");
static SNOWY_GRASS: Block = Block::named("base:snowy_grass");
static SNOW: Block = Block::named("base:snow");
static ICE: Block = Block::named("base:ice");
static WATER: Block = Block::named("base:water");
static TALL_GRASS: Block = Block::named("base:tall_grass");
static DANDELION: Block = Block::named("base:dandelion");
static ROSE: Block = Block::named("base:rose");
static LEAVES: [Block; 3] = [Block::named("base:oak_leaves"), Block::named("base:birch_leaves"), Block::named("base:spruce_leaves")];

/// How far the stream reaches, and how wide it is (cosine of its half angle).
const RANGE: f64 = 14.0;
const CONE: f64 = 0.93;
/// When the stream pours, in ticks of the breath: rearing first on the ground (`breathe`).
const GROUND: (u32, u32) = (19, 52);
const AIR_STREAM: (u32, u32) = (2, 40);
/// Ticks between breaths.
const COOLDOWN: u32 = 160;
/// Its mouth, ahead of its middle and up, in blocks: as the model has it rearing to breathe and
/// flying (`source/wyvern`).
const MOUTH_GROUND: (f64, f64) = (7.2, 3.7);
const MOUTH_AIR: (f64, f64) = (7.3, 4.2);

/// Poisoned: victim -> (who poisoned it, ticks left).
/// Seconds venom poisons a player.
const VENOM_SECS: f32 = 10.0;
static POISON: Mutex<Option<HashMap<u64, (u64, u32)>>> = Mutex::new(None);

pub fn init(r: &mut Registry) {
    r.entity_component(&BREATH, Breath::default(), 0);
    r.attach(&crate::wyvern::WYVERN, &BREATH);
    r.key(&KEY, "V")
        .on_press_server(|p| ridden(&p.world(), p.player()));
    // The streams: puffs that fly out of the mouth and fade (fire rising, frost and venom
    // sinking, sparks quick).
    r.particle(&FLAME)
        .textures(&["animals:breath0", "animals:breath1"])
        .life(0.6, 0.9)
        .size(0.4, 0.8)
        .gravity(-0.6)
        .drag(0.975)
        .color(0xffe08aff, 0x9a2a1200)
        .translucent()
        .shrinking()
        .fullbright();
    r.particle(&FROST)
        .textures(&["animals:breath0", "animals:breath1"])
        .life(0.6, 0.9)
        .size(0.3, 0.6)
        .gravity(0.4)
        .drag(0.975)
        .color(0xf2fbffff, 0x7fb8e800)
        .translucent();
    r.particle(&SPARK)
        .textures(&["animals:spark"])
        .life(0.15, 0.3)
        .size(0.2, 0.4)
        .gravity(0.0)
        .drag(0.9)
        .color(0xffffffff, 0x8ad8ff80)
        .translucent()
        .fullbright();
    r.particle(&VENOM)
        .textures(&["animals:breath0", "animals:breath1"])
        .life(0.6, 1.0)
        .size(0.3, 0.6)
        .gravity(1.2)
        .drag(0.975)
        .color(0xc8f05aff, 0x3c6a1c00)
        .translucent();
    r.world_tick(poison_tick);
}

/// Its rider pressed the key: the wyvern they ride breathes where they look.
fn ridden(w: &World, player: u64) {
    let Some((id, _)) = w.mount_of(player) else { return };
    if w.component(id, &BREATH).is_some() {
        start(w, id, 0);
    }
}

/// Begins a breath (if it isn't breathing or resting from one): at `target`, or ahead.
pub fn start(w: &World, id: u64, target: u64) -> bool {
    let flying = w
        .component(id, &crate::riding::FLIGHT)
        .is_some_and(|f| f.up != 0);
    let Some(b) = w.component_mut(id, &BREATH) else { return false };
    if b.t != 0 || b.cooldown != 0 {
        return false;
    }
    *b = Breath { t: 1, cooldown: COOLDOWN, flying: flying as u32, target };
    let kind = w.component(id, &GENOME).map_or("fire", coats::breath);
    weft_sdk::info!("wyvern {id} breathes {kind}");
    if !flying {
        w.play_animation(id, "breathe", false);
    }
    true
}

/// A goal for wyverns, early: a breath under way goes on (true while it holds it in place,
/// rearing on the ground).
pub fn behave(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(b) = w.component_mut(id, &BREATH) else { return false };
    b.cooldown = b.cooldown.saturating_sub(w.tick_elapsed(id) as _);
    if b.t == 0 {
        return false;
    }
    let breath = *b;
    b.t += 1;
    let (from, to) = if breath.flying != 0 { AIR_STREAM } else { GROUND };
    if breath.t > to {
        b.t = 0;
        return false;
    }
    let rider = w.rider_of(id);
    // Where it breathes: where its rider looks, at its target, or ahead.
    let (yaw, pitch) = match (rider.and_then(|r| w.entity(r)), w.entity(breath.target)) {
        (Some(r), _) => (r.yaw, r.pitch),
        (None, Some(t)) => {
            let d = [t.pos[0] - st.pos[0], t.pos[1] + t.size[1] as f64 * 0.5 - st.pos[1] - 3.0, t.pos[2] - st.pos[2]];
            let flat = (d[0] * d[0] + d[2] * d[2]).sqrt();
            ((-d[0]).atan2(-d[2]) as f32, d[1].atan2(flat) as f32)
        }
        _ => (st.yaw, -0.2),
    };
    if breath.flying == 0 {
        // Rearing on the ground it turns to face where it will breathe, and stands.
        let mut s = *st;
        s.yaw = yaw;
        w.set_entity(id, &s);
        if w.navigating(id) {
            w.navigate(id, st.pos, 0.0);
        }
    }
    if breath.t >= from {
        let kind = w.component(id, &GENOME).map_or("fire", coats::breath);
        let size = coats::size_of(w, id).unwrap_or(1.0) as f64;
        let (ahead, up) = if breath.flying != 0 { MOUTH_AIR } else { MOUTH_GROUND };
        let (sy, cy) = (st.yaw as f64).sin_cos();
        let mouth = [st.pos[0] - sy * ahead * size, st.pos[1] + up * size, st.pos[2] - cy * ahead * size];
        let (s2, c2) = (yaw as f64).sin_cos();
        let (sp, cp) = (pitch as f64).sin_cos();
        let dir = [-s2 * cp, sp, -c2 * cp];
        stream(w, id, rider, kind, mouth, dir, st.vel, breath.t);
    }
    breath.flying == 0
}

/// One tick of the stream from `mouth` along `dir`: its particles, and every few ticks what it
/// does to what it reaches.
/// The stream carries the wyvern's own speed (flying, it would outrun its fire).
fn stream(w: &World, id: u64, rider: Option<u64>, kind: &str, mouth: [f64; 3], dir: [f64; 3], carried: [f64; 3], t: u32) {
    let (particle, speed) = match kind {
        "frost" => (&FROST, 18.0),
        "lightning" => (&SPARK, 0.0),
        "venom" => (&VENOM, 15.0),
        _ => (&FLAME, 21.0),
    };
    if kind == "lightning" {
        // A jagged bolt to the first thing it hits, flickering.
        if t % 3 == 0 {
            let end = target_in_cone(w, id, rider, mouth, dir)
                .map_or_else(|| [mouth[0] + dir[0] * RANGE, mouth[1] + dir[1] * RANGE, mouth[2] + dir[2] * RANGE], |(_, p)| p);
            bolt(w, mouth, end, t);
        }
    } else {
        let v = [0, 1, 2].map(|k| (dir[k] * speed + carried[k]) as f32);
        drop(
            w.spawn_particles(particle, mouth)
                .count(6)
                .spread(0.25)
                .velocity(v),
        );
    }
    if t % 5 != 0 {
        return;
    }
    let source = rider.unwrap_or(id);
    let hit = in_cone(w, id, rider, mouth, dir);
    match kind {
        "frost" => {
            for &(e, _) in &hit {
                w.damage(e, 2.5, source, damage::GENERIC);
                w.add_modifier(e, &attribute::MOVEMENT_SPEED, "animals:frost", attr_op::MUL_BASE, -0.6, 80);
                w.ignite(e, 0);
            }
        }
        "lightning" => {
            if t % 10 == 0
                && let Some((first, p)) = target_in_cone(w, id, rider, mouth, dir)
            {
                w.damage(first, 9.0, source, damage::GENERIC);
                w.ignite(first, 40);
                // It arcs on to the two nearest beside it.
                let mut from = p;
                for other in w
                    .near(p, 6.0, None)
                    .into_iter()
                    .filter(|&o| o != first && o != id && Some(o) != rider)
                    .take(2)
                {
                    if let Some(o) = w.entity(other) {
                        let at = [o.pos[0], o.pos[1] + o.size[1] as f64 * 0.5, o.pos[2]];
                        bolt(w, from, at, t + 1);
                        w.damage(other, 5.0, source, damage::GENERIC);
                        from = at;
                    }
                }
            }
        }
        "venom" => {
            let mut poison = POISON.lock().unwrap();
            let poison = poison.get_or_insert_with(HashMap::new);
            for &(e, _) in &hit {
                w.damage(e, 2.0, source, damage::GENERIC);
                // A player is poisoned by their body (base's survival: longer, less with
                // fortitude, a death put down to the poison); an animal here.
                if w.players().contains(&e) {
                    base_api::poison(w, e, VENOM_SECS);
                } else {
                    poison.insert(e, (source, 100));
                }
            }
        }
        _ => {
            for &(e, _) in &hit {
                w.damage(e, 3.5, source, damage::FIRE);
                w.ignite(e, 120);
            }
        }
    }
    if let Some(p) = ground_hit(w, mouth, dir) {
        scorch(w, kind, p);
    }
}

/// Everything the stream reaches (not the wyvern, nor its rider): ids and where they are hit.
fn in_cone(w: &World, id: u64, rider: Option<u64>, mouth: [f64; 3], dir: [f64; 3]) -> Vec<(u64, [f64; 3])> {
    let mid = [mouth[0] + dir[0] * RANGE / 2.0, mouth[1] + dir[1] * RANGE / 2.0, mouth[2] + dir[2] * RANGE / 2.0];
    w.near(mid, RANGE / 2.0 + 2.0, None)
        .into_iter()
        .filter(|&e| e != id && Some(e) != rider)
        .filter_map(|e| {
            let s = w.entity(e)?;
            let p = [s.pos[0], s.pos[1] + s.size[1] as f64 * 0.5, s.pos[2]];
            let d = [p[0] - mouth[0], p[1] - mouth[1], p[2] - mouth[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let along = (d[0] * dir[0] + d[1] * dir[1] + d[2] * dir[2]) / len.max(1e-6);
            // Wide things count when their box is in it, not only their middle.
            let slack = (s.size[0] as f64 / 2.0 / len.max(1.0)).min(0.3);
            (len <= RANGE && along >= CONE - slack).then_some((e, p))
        })
        .collect()
}

/// The nearest thing the stream reaches.
fn target_in_cone(w: &World, id: u64, rider: Option<u64>, mouth: [f64; 3], dir: [f64; 3]) -> Option<(u64, [f64; 3])> {
    let d2 = |p: [f64; 3]| (0..3).map(|k| (p[k] - mouth[k]).powi(2)).sum::<f64>();
    in_cone(w, id, rider, mouth, dir)
        .into_iter()
        .min_by(|a, b| d2(a.1).total_cmp(&d2(b.1)))
}

/// A lightning bolt from `a` to `b`: sparks along a jagged line.
fn bolt(w: &World, a: [f64; 3], b: [f64; 3], seed: u32) {
    let n = 14;
    for k in 0..=n {
        let f = k as f64 / n as f64;
        let jag = |s: u32| ((((seed * 31 + k * 17 + s) % 97) as f64 / 97.0) - 0.5) * 0.8 * (1.0 - (2.0 * f - 1.0).powi(2));
        let p = [a[0] + (b[0] - a[0]) * f + jag(1), a[1] + (b[1] - a[1]) * f + jag(2), a[2] + (b[2] - a[2]) * f + jag(3)];
        drop(w.spawn_particles(&SPARK, p).count(2).spread(0.05));
    }
}

/// Where the stream meets the ground or a block, within its reach.
fn ground_hit(w: &World, mouth: [f64; 3], dir: [f64; 3]) -> Option<[i32; 3]> {
    let mut d = 0.0;
    while d < RANGE {
        let p = [mouth[0] + dir[0] * d, mouth[1] + dir[1] * d, mouth[2] + dir[2] * d].map(|v| v.floor() as i32);
        if w.get(p) != AIR.id() {
            return Some(p);
        }
        d += 0.5;
    }
    None
}

/// What the stream does to the blocks where it lands (a few around it).
fn scorch(w: &World, kind: &str, at: [i32; 3]) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                if w.random_below(3) != 0 {
                    continue;
                }
                let p = [at[0] + dx, at[1] + dy, at[2] + dz];
                let b = w.get(p);
                let plant = [TALL_GRASS.id(), DANDELION.id(), ROSE.id()].contains(&b);
                let leaves = LEAVES.iter().any(|l| l.id() == b);
                let to = match kind {
                    "fire" if b == GRASS.id() || b == SNOWY_GRASS.id() => Some(DIRT.id()),
                    "fire" if plant || leaves || b == SNOW.id() => Some(AIR.id()),
                    "fire" if b == ICE.id() => Some(WATER.id()),
                    "frost" if b == WATER.id() => Some(ICE.id()),
                    "frost" if b == GRASS.id() => Some(SNOWY_GRASS.id()),
                    "venom" if b == GRASS.id() => Some(DIRT.id()),
                    "venom" if plant || leaves => Some(AIR.id()),
                    _ => None,
                };
                if let Some(s) = to {
                    w.set(p, s);
                }
            }
        }
    }
}

/// Poison works on: each second a poisoned thing is hurt, for five seconds.
fn poison_tick(w: &World) {
    let mut poison = POISON.lock().unwrap();
    let Some(poison) = poison.as_mut() else { return };
    poison.retain(|&victim, (source, left)| {
        *left = left.saturating_sub(1);
        if w.entity(victim).is_none() {
            return *left > 0; // gone, or in another dimension: its time runs out all the same
        }
        if *left % 20 == 0 {
            w.damage(victim, 1.5, *source, damage::GENERIC);
            if let Some(s) = w.entity(victim) {
                drop(
                    w.spawn_particles(&VENOM, [s.pos[0], s.pos[1] + s.size[1] as f64 * 0.7, s.pos[2]])
                        .count(3)
                        .spread(0.3),
                );
            }
        }
        *left > 0
    });
}

//! An animal's body on the survival system the player has (`notes/SURVIVAL.md`, step 8): the air
//! where it stands (`base_api::air_at`, the same as a player's), its core temperature against its
//! species' comfortable air (`Biology::comfort`, facts), its condition from what it has eaten, and
//! seeking comfort: shade or water in the heat, warmth and shelter in the cold.
//!
//! Wild ones only move to where it's better: their places suit them. A tame one is cared for:
//! out of its comfort it eats faster, and freezing or overheating it loses health, down to a
//! tenth (a neglected tame suffers, it doesn't die of it). A cold-blooded one has no heat of its
//! own: it slows in the cold instead.

use vitals::effects::Effect;
use vitals::temperature::{self, Body as Comfort, Feeling};
use weft_sdk::*;

use crate::animal::{self, ANIMAL, Animal};
use crate::species::{self, SPECIES};

value! {
    #[derive(Default)]
    /// How its body is (once a second), sent to whoever has its screen open.
    pub struct Body {
        /// Core temperature, °C (0: not yet).
        pub core: f32,
        /// The air where it stands, °C.
        pub air: f32,
        /// How it feels (`FEELINGS`).
        pub feel: u8,
        /// Strength from what it has eaten, percent (`condition`).
        pub condition: u8,
        /// Its speed in this air if cold-blooded, percent (100 otherwise).
        pub warmth: u8,
    }
}

pub static BODY: Component<Body> = Component::named("animal_body");
/// Where it is heading for comfort, and until when (not saved).
static SEEK: Component<[f64; 4]> = Component::named("animal_seek");
pub(crate) static WATER: Tag = Tag::named("water");

/// The feelings in `Body::feel`'s order, their names in translations.
pub const FEELINGS: [Feeling; 7] =
    [Feeling::Comfortable, Feeling::Chilly, Feeling::Cold, Feeling::Freezing, Feeling::Warm, Feeling::Hot, Feeling::Heatstroke];
pub const FEELING_NAMES: [&str; 7] = ["comfortable", "chilly", "cold", "freezing", "warm", "hot", "heatstroke"];

/// Strength on an empty stomach, as the player's (`vitals::stomach`): the rest comes back as it
/// is fed, all of it from half full.
const FLOOR: f32 = 0.4;
/// Blocks searched for a fire around it, and for a better place.
const REACH: i32 = 3;
const LOOK: f64 = 10.0;
/// Blocks a cold one notices a fire from (its light and smell carry further).
const FIRE_LOOK: i32 = 16;
/// Places tried for a better one, and degrees better it must be to go.
const TRIES: u32 = 12;
const BETTER: f32 = 1.5;
/// A tame's health lost to the cold or heat stops at this share of its max.
const TAME_FLOOR: f32 = 0.1;

pub fn init(r: &mut Registry) {
    r.entity_component(&BODY, Body::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&SEEK, [0.0; 4], 0);
    for s in &SPECIES {
        r.attach(s.kind, &BODY);
        r.attach(s.kind, &SEEK);
    }
}

/// Its strength from its stomach and its water: full from half fed and half slaked, down to 40%
/// empty or parched (`drink.rs`), whichever is worse.
pub fn condition(a: &Animal) -> f32 {
    let max = a.stat(species::FOOD);
    if max <= 0.0 {
        return 1.0;
    }
    let fed = a.food as f32 / max;
    let watered = 1.0 - a.thirst.clamp(0.0, 1.0);
    FLOOR + (1.0 - FLOOR) * (fed.min(watered) * 2.0).min(1.0)
}

/// What its body makes of the air: its species' comfort, wider by its fortitude points.
fn comfort(a: &Animal) -> Option<Comfort> {
    let s = a.species()?;
    Some(Comfort {
        comfort: s.bio.comfort,
        // The facts are at the species' base: only points above it widen them.
        fortitude: (a.stat(species::FORTITUDE) - s.base[species::FORTITUDE]).max(0.0),
        fat: a.food as f32 / a.stat(species::FOOD).max(1.0) * 0.5,
        ..Comfort::default()
    })
}

/// Once a second, from `animal::upkeep`: the air, the core, how it feels, its condition; a tame
/// out of comfort loses health if freezing or overheating. Returns how many times faster a tame
/// eats now (out of comfort, and by the world's difficulty).
pub fn upkeep(w: &World, id: u64, a: &Animal, st: &EntityState) -> f32 {
    let Some(s) = a.species() else { return 1.0 };
    let Some(body) = comfort(a) else { return 1.0 };
    let air = base_api::air_at(w, st.pos, st.size[1], st.in_liquid != 0, REACH, |_| {}).celsius;
    let size = crate::coats::size_of(w, id).map_or(1.0, |g| g * crate::breeding::growth(w, id, (a.species - 1) as usize));
    let mass = s.bio.mass * size.powi(3);
    let old = w.component(id, &BODY).copied().unwrap_or_default();
    let (core, feeling, warmth) = if s.bio.cold_blooded {
        let mid = (s.bio.comfort.0 + s.bio.comfort.1) / 2.0;
        (air, Feeling::Comfortable, temperature::ectotherm_speed(air, mid))
    } else {
        let start = if old.core < 20.0 { temperature::CORE } else { old.core };
        let core = temperature::step_core(start, body.target(air), 1.0, mass);
        (core, temperature::feeling(core), 1.0)
    };
    let feel = FEELINGS.iter().position(|&f| f == feeling).unwrap_or(0);
    let condition = condition(a);
    if let Some(b) = w.component_mut(id, &BODY) {
        *b = Body { core, air, feel: feel as u8, condition: (condition * 100.0).round() as u8, warmth: (warmth * 100.0).round() as u8 };
    }
    if !animal::tamed(w, id) {
        return 1.0;
    }
    // A tame freezing or overheating loses health as a player does (as hard as the world's
    // difficulty says), down to a tenth; it eats as fast as the difficulty digests.
    let d = base_api::difficulty();
    let effect = match feeling {
        Feeling::Freezing => Some(Effect::Freezing),
        Feeling::Heatstroke => Some(Effect::Heatstroke),
        _ => None,
    };
    if let Some(e) = effect {
        let max = st.max_health;
        let loss = (e.modifiers().health_loss * max * d.harm).min((st.health - max * TAME_FLOOR).max(0.0));
        if loss > 0.0 {
            let cause = if e == Effect::Freezing { damage::FREEZE } else { damage::HEAT };
            w.damage(id, loss, 0, cause);
        }
    }
    d.digest
        * match feeling {
            Feeling::Comfortable | Feeling::Chilly | Feeling::Warm => 1.0,
            Feeling::Cold | Feeling::Hot => 1.5,
            Feeling::Freezing | Feeling::Heatstroke => 2.0,
        }
}

/// How fast it moves for its body: its condition, and a cold-blooded one's warmth.
pub fn speed(w: &World, id: u64) -> f32 {
    w.component(id, &BODY)
        .filter(|b| b.core > 0.0)
        .map_or(1.0, |b| b.condition as f32 / 100.0 * b.warmth as f32 / 100.0)
}

/// A goal: cold or hot (not just chilly or warm), it goes to a better place near by (in the
/// shade, into water, by a fire, under a roof) and rests there; it takes the turn while it does.
pub fn seek(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(b) = w.component(id, &BODY).copied() else { return false };
    let uneasy = matches!(FEELINGS.get(b.feel as usize), Some(Feeling::Cold | Feeling::Freezing | Feeling::Hot | Feeling::Heatstroke));
    let Some(seek) = w.component(id, &SEEK).copied() else { return false };
    if !uneasy {
        if seek[3] > 0.0 {
            w.navigate(id, st.pos, 0.0);
            if let Some(s) = w.component_mut(id, &SEEK) {
                *s = [0.0; 4];
            }
        }
        return false;
    }
    let now = w.time() as f64;
    // On its way, or resting where it went, until it looks again.
    if now < seek[3] {
        return true;
    }
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(body) = comfort(&a) else { return false };
    let (lo, hi) = body.comfort_range();
    let off = |t: f32| (lo - t).max(t - hi).max(0.0);
    let here = off(b.air);
    // Places to try: around it at random, and in the cold beside the nearest fire.
    let mut tries: Vec<(i32, i32)> = (0..TRIES)
        .map(|_| {
            let angle = w.random_below(3600) as f64 / 3600.0 * std::f64::consts::TAU;
            let dist = 2.0 + w.random_below(1000) as f64 / 1000.0 * (LOOK - 2.0);
            ((st.pos[0] + angle.cos() * dist).floor() as i32, (st.pos[2] + angle.sin() * dist).floor() as i32)
        })
        .collect();
    if b.air < lo
        && let Some(fire) = nearest_fire(w, st.pos)
    {
        for (dx, dz) in [(2, 0), (-2, 0), (0, 2), (0, -2), (2, 2), (-2, 2), (2, -2), (-2, -2)] {
            tries.push((fire[0] + dx, fire[2] + dz));
        }
    }
    let mut best: Option<([f64; 3], f32)> = None;
    for (x, z) in tries {
        let Some(y) = ground(w, x, st.pos[1].floor() as i32, z) else { continue };
        let at = [x as f64 + 0.5, y as f64, z as f64 + 0.5];
        let wet = w.in_liquid([at[0], at[1] + 0.3, at[2]]);
        let t = off(base_api::air_at(w, at, st.size[1], wet, REACH, |_| {}).celsius);
        if t + BETTER <= here && best.is_none_or(|(_, b)| t < b) {
            best = Some((at, t));
        }
    }
    let target = best.map_or(st.pos, |(p, _)| p);
    if let Some((p, t)) = best {
        weft_sdk::info!("animal {id} seeks comfort at {p:?}: {t:.1} °C off its comfort, {here:.1} here");
        w.navigate(id, target, 1.0);
    }
    if let Some(s) = w.component_mut(id, &SEEK) {
        // Look again in five seconds; resting where it is, in a minute.
        *s = [target[0], target[1], target[2], now + if best.is_some() { 100.0 } else { 1200.0 }];
    }
    true
}

/// The nearest block that heats the air (`base_api::heat_of`) within `FIRE_LOOK` blocks.
fn nearest_fire(w: &World, pos: [f64; 3]) -> Option<[i32; 3]> {
    let p = pos.map(|v| v.floor() as i32);
    let r = FIRE_LOOK;
    let mut best: Option<([i32; 3], i32)> = None;
    for dx in -r..=r {
        for dy in -3..=3 {
            for dz in -r..=r {
                let d = dx * dx + dy * dy + dz * dz;
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                if best.is_none_or(|(_, b)| d < b) && base_api::heat_of(w.get(at)) > 0.0 {
                    best = Some((at, d));
                }
            }
        }
    }
    best.map(|(at, _)| at)
}

/// Where one can stand in column `x, z` near height `y`: on solid ground (not lava or water), in
/// air or water.
pub(crate) fn ground(w: &World, x: i32, y: i32, z: i32) -> Option<i32> {
    (y - 4..=y + 3).rev().find(|&y| {
        let feet = w.get([x, y, z]);
        let below = w.get([x, y - 1, z]);
        let solid = below != 0 && !w.in_liquid([x as f64 + 0.5, y as f64 - 0.5, z as f64 + 0.5]);
        (feet == 0 || WATER.on_block(feet)) && solid && w.get([x, y + 1, z]) == 0
    })
}

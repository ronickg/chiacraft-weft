//! The player's survival (`notes/SURVIVAL.md`; the rules and numbers are the `vitals` crate's,
//! shared with the animals): one body whose stats are reached by what's eaten. No hunger: an empty
//! stomach makes one weak, never dead. Creative skips it all. One file per part:
//!
//! - `breath`: air under water.
//! - `food`: eating, digesting, freshness, food poisoning.
//! - `temperature`: the air, being wet, armor, the core temperature; the seasons command.
//! - `camp`: near a fire, under a roof, comfort and being rested.
//! - `stamina`: what actions cost and what brings it back.
//! - `body`: poison, bleeding, sedation, and the bandage.
//! - `load`: what one carries, and walking speed.
//! - `health`: max health from the stomach, healing, and what effects take.
//!
//! Here: the vitals component, the difficulty rule, dying, and the tick that runs the parts in
//! order.

mod body;
mod breath;
mod camp;
mod food;
mod health;
mod load;
mod stamina;
mod temperature;

use base_api::{VITALS, Vitals};
use vitals::effects::{self, Effect};
use vitals::stats::Stat;
use vitals::{difficulty, levels, stomach};
use weft_sdk::*;

pub use breath::MAX_AIR;
pub(crate) use food::{crafted, life_used, made_for, spoiled};
pub use camp::REST_SECS;

/// A tick, in seconds.
const DT: f32 = 1.0 / 20.0;

pub fn register(r: &mut Registry) {
    // How hard survival is, picked when the world is made (`vitals::difficulty`).
    r.config(&base_api::DIFFICULTY)
        .choice(1, &difficulty::ALL.map(|d| d.name))
        .world();
    r.entity_component(&VITALS, Vitals::new(MAX_AIR), component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP);
    r.attach(&PLAYER, &VITALS);
    r.world_tick(tick);
    food::register(r);
    temperature::register(r);
    stamina::register(r);
    body::register(r);
    // Dying empties the stomach and costs part of the XP into the level.
    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            let Some(v) = w.component_mut(d.entity, &VITALS) else { continue };
            v.set_stomach(&stomach::Stomach::default());
            v.stamina = 0.0;
            v.xp = levels::after_death(v.xp, base_api::difficulty().death_xp);
            v.air = MAX_AIR;
            v.wet = 0.0;
            v.core = vitals::temperature::CORE;
            (v.poison, v.bleed, v.torpor, v.out) = (0.0, 0.0, 0.0, 0);
            // Respawning drops the body's modifiers: the next tick sets max health and speed again.
            v.max_health = -1.0;
            v.speed = -1.0;
        }
    });
}

fn tick(w: &World) {
    // A brand-new world with seasons begins on spring's 8th day (`vitals::calendar::START_DAY`):
    // its clock is moved there in its first second, so the engine's day length and every calendar
    // agree.
    if w.time() < 20 && w.day_time() == w.time() && base_api::SEASONS.on_dimension(w.dimension()).unwrap_or(0) == base_api::calendars::FOUR {
        w.set_day_time(w.day_time() + vitals::calendar::START_DAY as u64 * vitals::calendar::DAY_TICKS);
    }
    let d = base_api::difficulty();
    let pulse = w.time() % 20 == 0;
    for id in w.players() {
        let Some(mut st) = w.entity(id) else { continue };
        let underwater = breath::underwater(w, &st);
        let creative = w.creative(id);
        let Some(v) = w.component_mut(id, &VITALS) else { continue };
        breath::breathe(w, id, v, underwater && !creative);
        if creative {
            v.stamina = v.max_stamina;
            load::reset_speed(w, id, v);
            continue;
        }

        let s = food::digest(v, d.digest);
        let stats = v.stats();
        let capacity = v.capacity();
        let full = s.fullness(capacity);
        let (max_health, max_stamina) = stomach::max_health_stamina(stats[0], stats[1], &s, capacity);
        // The place, a second at a time.
        if pulse {
            let fat = full[vitals::food::FAT];
            let (here, comfort) = temperature::feel(w, id, &st, v, &s, stats[Stat::Fortitude.index()], fat);
            camp::rest(v, &st, &here, comfort);
        }
        food::effects(v, &s, capacity);
        body::tick(v, &stats);
        if pulse {
            food::age(w, id);
            load::weigh(w, id, v, &stats);
        }
        let mods = effects::combine(v.effects());
        load::speed(w, id, v, &stats, &mods);
        stamina::tick(w, id, &st, v, &mods, &full, max_stamina, &d);
        health::tick(w, id, &mut st, v, &mods, &full, (max_health, max_stamina), &d, pulse);
    }
}

/// Turns an effect on or off.
fn set(v: &mut Vitals, e: Effect, on: bool) {
    v.effects = (v.effects & !(1 << e.index())) | if on { 1 << e.index() } else { 0 };
}

/// Turns off every effect in `group`, then on those of them in `on`.
fn set_group(v: &mut Vitals, group: &[Effect], on: &[Effect]) {
    for &e in group {
        set(v, e, on.contains(&e));
    }
}

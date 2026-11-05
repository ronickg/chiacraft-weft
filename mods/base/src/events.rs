//! Sky events in the world (`vitals::events` has the schedule and numbers; reports/Research round
//! 17.md's addendum, Ronald: "Yes, as designed"): the dusk line foretelling tonight's event, and
//! the blood moon's monsters.
//!
//! A blood moon's monsters come on top of the hostile ones (`BLOOD_MOON_CAP` within 64 blocks of
//! each player, `Difficulty::blood_moon_cap`), spawned here every two seconds in totality by the
//! engine's own rules for monsters: 20-45 blocks out, light 6 or less, on the ground, and through
//! `question::NATURAL_SPAWN`, so other mods' vetoes hold. They keep out of a campfire's ring and
//! leave a player in their respawn grace be, as every monster does (`mobs::keep_from_fire`): a
//! blood moon is a night to sit by the fire. They're bolder (more health and speed, a longer
//! reach), not deadlier, and at its end they walk off and are gone.

use std::collections::HashMap;
use std::sync::Mutex;

use vitals::events::{self as ev, Event};
use weft_sdk::*;

/// The blood moon's monsters alive: (entity, dimension).
static WALKING: Mutex<Vec<(u64, u32)>> = Mutex::new(Vec::new());
/// The day each player was last told tonight's event.
static TOLD: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);
/// Players out for a blood moon from its start, by day: "Red Night" if they see it through.
static WATCHING: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);

const MODIFIER: &str = "base:blood_moon";
/// Ticks between the spawner's rounds (the engine's natural spawner's), and tries a player a round.
const ROUND_TICKS: u64 = 40;
const TRIES: u32 = 4;
/// Blocks out from a player a monster may appear (the engine's), and how far its cap counts.
const SPAWN_REACH: (f64, f64) = (20.0, 45.0);
const CAP_REACH: f64 = 64.0;
/// Up and down from the player's feet a spot is looked for.
const SPAWN_UP: i32 = 16;
/// Monsters walking off at the end are gone this far from every player.
const GONE_AT: f64 = 40.0;

pub fn register(r: &mut Registry) {
    r.world_tick(tick);
    // Dying ends a player's watch of the blood moon.
    r.on(&actions::DIE, |b| {
        let mut watching = WATCHING.lock().unwrap();
        let watching = watching.get_or_insert_with(HashMap::new);
        for d in b.iter() {
            watching.remove(&d.entity);
        }
    });
}

/// Tonight's event in `w`, if any.
pub fn tonight(w: &World) -> Option<Event> {
    ev::tonight(w.seed(), w.day_time() / vitals::calendar::DAY_TICKS)
}

/// Whether a blood moon is in totality now, when its monsters walk and kills give more XP.
pub fn blood_totality(w: &World) -> bool {
    tonight(w) == Some(Event::BloodMoon) && ev::in_totality(ev::hour_of(w.day_time()))
}

/// Whether `id` is one of the blood moon's monsters (they see players further: `mobs.rs`).
pub fn is_blood(id: u64) -> bool {
    WALKING.lock().unwrap().iter().any(|e| e.0 == id)
}

/// The dusk line for `player`, once a day, `DUSK_LINE_HOURS` before `sunset` (`hints.rs` calls it
/// each second with the hour and the player's sunset).
pub fn dusk(w: &World, player: u64, hour: f32, sunset: f32) {
    let Some(e) = tonight(w).filter(|e| e.announced()) else { return };
    if !(sunset - ev::DUSK_LINE_HOURS..sunset + 1.0).contains(&hour) {
        return;
    }
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    let mut told = TOLD.lock().unwrap();
    let told = told.get_or_insert_with(HashMap::new);
    if told.get(&player) == Some(&day) {
        return;
    }
    told.insert(player, day);
    // On gentle the blood moon is a look only: no warning of monsters.
    let key = if e == Event::BloodMoon && !base_api::difficulty().blood_moon_gameplay {
        "hint.base.blood_moon_gentle".to_string()
    } else {
        format!("hint.base.{}", e.name())
    };
    weft_sdk::info!("events: tonight is {} (day {day}); told {player}", e.name());
    base_api::hint(w, player, &key, &[]);
}

fn tick(w: &World) {
    let now = w.game_time();
    if !now.is_multiple_of(ROUND_TICKS) {
        return;
    }
    let event = tonight(w);
    let strength = if event == Some(Event::BloodMoon) { ev::blood_moon_strength(ev::hour_of(w.day_time())) } else { 0.0 };
    watch(w, strength);
    let dim = w.dimension();
    WALKING
        .lock()
        .unwrap()
        .retain(|&(id, d)| d != dim || w.entity(id).is_some());
    if strength >= 1.0 {
        let diff = base_api::difficulty();
        if diff.blood_moon_gameplay && settings::MOB_SPAWNING.get() {
            for player in w.players() {
                spawn_near(w, player, diff.blood_moon_cap, diff.blood_moon_health);
            }
        }
    } else if strength == 0.0 {
        walk_off(w);
    }
}

/// Up to `TRIES` spots near `player` for a blood-moon monster, while fewer than `cap` are near.
fn spawn_near(w: &World, player: u64, cap: u32, health: f32) {
    if w.outside_survival(player) || crate::death::in_grace(w, player) {
        return;
    }
    let Some(p) = w.entity(player) else { return };
    let dim = w.dimension();
    let near = WALKING
        .lock()
        .unwrap()
        .iter()
        .filter(|&&(id, d)| {
            d == dim
                && w.entity(id)
                    .is_some_and(|e| (e.pos - p.pos).length_squared() < CAP_REACH * CAP_REACH)
        })
        .count() as u32;
    if near >= cap {
        return;
    }
    for _ in 0..TRIES {
        let angle = unit(w) as f64 * std::f64::consts::TAU;
        let reach = SPAWN_REACH.0 + (SPAWN_REACH.1 - SPAWN_REACH.0) * unit(w) as f64;
        let (x, z) = ((p.pos.x + angle.cos() * reach).floor() as i32, (p.pos.z + angle.sin() * reach).floor() as i32);
        let Some(spot) = ground_spot(w, x, p.pos.y.floor() as i32, z) else { continue };
        if abi::light::effective(w.light(spot), w.daylight()) > 6 || crate::mobs::campfire_near(w, spot.center()).is_some() {
            continue;
        }
        let kind = pick_kind(w);
        let mut q = abi::Query { pos: spot, other: kind.id() as u64, amount: 1.0, ..abi::Query::DEFAULT };
        if !w.ask(&question::NATURAL_SPAWN, &mut q) {
            continue;
        }
        let id = w.spawn(kind, [spot.x as f64 + 0.5, spot.y as f64, spot.z as f64 + 0.5]);
        if id == 0 {
            continue;
        }
        w.attribute_add_modifier(id, &attribute::MAX_HEALTH, MODIFIER, attr_op::MULTIPLY_BASE, (health - 1.0) as f64, 0);
        w.attribute_add_modifier(id, &attribute::MOVEMENT_SPEED, MODIFIER, attr_op::MULTIPLY_BASE, (ev::BLOOD_MOON_SPEED - 1.0) as f64, 0);
        w.heal(id, 1000.0);
        WALKING.lock().unwrap().push((id, dim));
        weft_sdk::info!(
            "events: a blood-moon {} at {spot:?} for {player} ({} near)",
            name_of(abi::entry_kind::ENTITY, kind.id()),
            near + 1
        );
        return;
    }
}

/// Where a monster may stand in column (`x`, `z`) near height `y`: on something solid, two free
/// above, the highest such within `SPAWN_UP`.
fn ground_spot(w: &World, x: i32, y: i32, z: i32) -> Option<BlockPos> {
    let solid = |p: BlockPos| w.block_flags(p) & abi::block_flags::SOLID != 0;
    (y - SPAWN_UP..=y + SPAWN_UP)
        .rev()
        .map(|y| BlockPos::new(x, y, z))
        .find(|&p| solid(p.below()) && !solid(p) && !solid(p.above()) && w.block_flags(p) & abi::block_flags::LIQUID == 0)
}

/// A monster by `BLOOD_MOON_WEIGHTS`.
fn pick_kind(w: &World) -> &'static EntityKind {
    let total: u32 = ev::BLOOD_MOON_WEIGHTS.iter().map(|k| k.1).sum();
    let mut pick = w.random_below(total);
    for (name, weight) in ev::BLOOD_MOON_WEIGHTS {
        if pick < weight {
            return match name {
                "zombie" => &crate::mobs::ZOMBIE,
                "spider" => &crate::mobs::SPIDER,
                "skeleton" => &crate::mobs::SKELETON,
                _ => &crate::mobs::CREEPER,
            };
        }
        pick -= weight;
    }
    &crate::mobs::ZOMBIE
}

/// The blood moon over: its monsters walk away from the nearest player and are gone out of sight.
fn walk_off(w: &World) {
    let dim = w.dimension();
    let players: Vec<Vec3> = w
        .players()
        .into_iter()
        .filter_map(|p| w.entity(p))
        .map(|e| e.pos)
        .collect();
    WALKING.lock().unwrap().retain(|&(id, d)| {
        if d != dim {
            return true;
        }
        let Some(e) = w.entity(id) else { return false };
        let nearest = players.iter().min_by(|a, b| {
            (**a - e.pos)
                .length_squared()
                .total_cmp(&(**b - e.pos).length_squared())
        });
        match nearest {
            Some(&p) if (p - e.pos).length_squared() < GONE_AT * GONE_AT => {
                let away = e.pos - p;
                let len = away.length_squared().sqrt().max(0.1);
                w.navigate(id, e.pos + away * (GONE_AT / len), 1.0);
                true
            }
            _ => {
                w.despawn(id);
                false
            }
        }
    });
}

/// "Red Night": players out when a blood moon starts are watched; those still alive when it has
/// cleared (`strength` 0 again, after 03:00) get the milestone.
fn watch(w: &World, strength: f32) {
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    let hour = ev::hour_of(w.day_time());
    let mut watching = WATCHING.lock().unwrap();
    let watching = watching.get_or_insert_with(HashMap::new);
    if strength > 0.0 && strength < 1.0 && hour > 12.0 {
        // Reddening: whoever is out now watches it.
        for p in w.players() {
            if !w.outside_survival(p) {
                watching.entry(p).or_insert(day);
            }
        }
    } else if strength == 0.0 && hour < 12.0 {
        watching.retain(|&p, &mut d| {
            if d == day && w.entity(p).is_some() {
                base_api::milestone(w, p, base_api::milestones::RED_NIGHT);
                weft_sdk::info!("events: {p} saw the blood moon through");
            }
            false
        });
    }
}

fn unit(w: &World) -> f32 {
    (w.random() >> 40) as f32 / (1u64 << 24) as f32
}

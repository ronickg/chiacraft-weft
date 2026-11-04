//! Eating from the world (`notes/ECOLOGY.md`, part 5): a hungry animal looks round for food of
//! its diet (`Species::eats`) that grows there (`vitals::forage::SOURCES`: tall grass, ripe
//! berries, leaves), walks to it and eats a mouthful at a time until it's full. What it eats
//! changes as eating it leaves it: grass is grazed short and grows back by warmth and rain
//! (`base:grazed_grass`), a berry bush starts its fruit again. So a lush place keeps its animals
//! fed and a dry season thins them, through their condition (`comfort::condition`).
//!
//! Its place feeds as many as its food grows: plant eaters come by natural spawning as often as
//! their food grows there this season (`season_lush`), and each mouthful feeds as well as the
//! plant grows today (dry-season grass is poor fodder).
//!
//! Wild plant eaters get hungry (`Species::forages_wild`, `animal::upkeep`); tames forage when
//! hungry and not told to stay (`commands::hold` comes first).

use vitals::forage::{Eaten, Source};
use weft_sdk::*;

use crate::animal::{self, ANIMAL};
use crate::species::{self, SPECIES, Species};

/// Where it is going to eat and until when (not saved): x, y, z of the food (NaN: none near),
/// and the time it looks again (0: not foraging).
static FORAGE: Component<[f64; 4]> = Component::named("animal_forage");
static GRAZED_GRASS: Block = Block::named("base:grazed_grass");
static DUNG: Item = Item::named("base:dung");

/// Blocks it looks round, and up and down.
const LOOK: i32 = 8;
const LOOK_UP: i32 = 3;
/// It starts foraging under this share of its food, and stops when this full.
const HUNGRY: f32 = 0.6;
const FULL: f32 = 0.95;
/// Ticks: chewing a mouthful, walking before it looks again, wandering when nothing is near.
const CHEW: f64 = 30.0;
const WALK: f64 = 160.0;
const NOTHING: f64 = 300.0;
/// A grazer leaves dung after about this many mouthfuls (`vitals::soil`: it feeds farmland and
/// compost).
const MOUTHFULS_A_DUNG: u32 = 6;

pub fn init(r: &mut Registry) {
    r.entity_component(&FORAGE, [0.0; 4], 0);
    for s in &SPECIES {
        r.attach(s.kind, &FORAGE);
    }
    // A place holds as many as it feeds (`notes/ECOLOGY.md`, part 7): plant eaters come as often
    // as their food grows there now, so fewer in a drought or a winter.
    r.intercept(&question::NATURAL_SPAWN, |q| {
        let kind = q.query().other as u32;
        let Some(s) = SPECIES
            .iter()
            .find(|s| s.kind.id() == kind && s.forages_wild())
        else {
            return Verdict::Abstain;
        };
        let w = q.world();
        let come = SPAWN_FLOOR + (1.0 - SPAWN_FLOOR) * season_lush(&w, s, q.pos());
        if (w.random_below(10_000) as f32) < come * 10_000.0 { Verdict::Abstain } else { Verdict::Deny }
    });
}

/// The share of natural spawns that still come where nothing grows this season: animals live
/// through a winter or a drought, fewer and thinner.
const SPAWN_FLOOR: f32 = 0.2;

/// How much of its food its biome grows through this season, 0 to 1
/// (`vitals::forage::season_lushness`); 1 for one that doesn't forage.
pub fn season_lush(w: &World, s: &Species, pos: [i32; 3]) -> f32 {
    let biome = w.biome(pos);
    let c = base_api::CLIMATE
        .on_biome(biome)
        .unwrap_or(base_api::TEMPERATE);
    let season = crate::breeding::season_bit(w).map(|bit| bit.trailing_zeros() as usize);
    let name = name_of(abi::entry_kind::BIOME, biome);
    vitals::forage::season_lushness(|k| s.eats(k), name, c.celsius, c.seasons, c.rain, season).unwrap_or(1.0)
}

/// How much of its food the place at `pos` grows now, 0 to 1 (`vitals::forage::lushness`); 1 for
/// one that doesn't forage.
pub fn lush(w: &World, s: &Species, pos: [i32; 3]) -> f32 {
    let g = base_api::ground_at(w, pos);
    vitals::forage::lushness(|c| s.eats(c), g.mean_air, g.soil).unwrap_or(1.0)
}

/// Whether it is foraging (a resting animal settles only when it isn't: hungry, it eats first).
pub fn busy(w: &World, id: u64) -> bool {
    w.component(id, &FORAGE)
        .is_some_and(|f| f[3] != 0.0 && !f[0].is_nan())
}

/// What it would eat of the block `state`, if anything.
fn food(s: &Species, state: u32, season: Option<usize>) -> Option<&'static Source> {
    let name = block_name(state).split('[').next()?;
    let age = state_prop(state, "age").and_then(|a| a.parse().ok());
    vitals::forage::of(name, age, |c| s.eats(c), season)
}

/// The season now (0 spring), where there are seasons.
fn season(w: &World) -> Option<usize> {
    crate::breeding::season_bit(w).map(|bit| bit.trailing_zeros() as usize)
}

/// The nearest food of its diet within `LOOK` blocks it can reach from the ground; the ground's
/// own roots, insects and lichen only when nothing richer is in sight (`Source::fallback`).
fn nearest(w: &World, s: &Species, pos: [f64; 3]) -> Option<[i32; 3]> {
    let p = pos.map(|v| v.floor() as i32);
    let now = season(w);
    let mut best: Option<([i32; 3], i32)> = None;
    let mut ground: Option<([i32; 3], i32)> = None;
    for dx in -LOOK..=LOOK {
        for dz in -LOOK..=LOOK {
            for dy in -2..=LOOK_UP {
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                let d = dx * dx + dz * dz + dy * dy * 4;
                if best.is_some_and(|(_, b)| d >= b) {
                    continue;
                }
                match food(s, w.get(at), now) {
                    Some(src) if !src.fallback() => best = Some((at, d)),
                    Some(_) if ground.is_none_or(|(_, b)| d < b) => ground = Some((at, d)),
                    _ => {}
                }
            }
        }
    }
    best.or(ground).map(|(at, _)| at)
}

/// A goal: hungry, it goes to the nearest food of its diet and eats there; it takes the turn
/// while it does. With nothing near it lets the next goal (wandering) take it elsewhere.
pub fn forage(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(s) = a.species() else { return false };
    let Some(mut f) = w.component(id, &FORAGE).copied() else { return false };
    let full = a.food as f32 / a.stat(species::FOOD).max(1.0);
    // Only an animal that gets hungry, and whose food grows in the world.
    let hungry = (s.hungry_in_the_wild() || animal::tamed(w, id)) && s.eats.iter().any(|&(c, _)| vitals::forage::grows(c));
    if !hungry || full >= FULL || (f[3] == 0.0 && full >= HUNGRY) {
        if f[3] != 0.0 {
            w.component_mut(id, &FORAGE).map(|f| *f = [0.0; 4]);
        }
        return false;
    }
    let now = w.time() as f64;
    let target = (!f[0].is_nan() && f[3] != 0.0).then(|| [f[0], f[1], f[2]]);
    let close = target.is_some_and(|t| {
        let (dx, dz) = (t[0] + 0.5 - st.pos[0], t[2] + 0.5 - st.pos[2]);
        // Within reach of its mouth: two blocks for even a small one standing beside it.
        dx * dx + dz * dz <= 4.0 && (t[1] - st.pos[1]).abs() <= 2.5
    });
    if close && let Some(t) = target {
        if now < f[3] {
            return true; // chewing
        }
        let at = t.map(|v| v as i32);
        if let Some(src) = food(s, w.get(at), season(w)) {
            // Growing food feeds as well as it grows now: dry-season grass is poor fodder.
            let fodder = if src.renews.is_some() { 0.4 + 0.6 * lush(w, s, at) } else { 1.0 };
            animal::feed(w, id, (src.food as f32 * fodder).round().max(1.0) as u16);
            weft_sdk::info!(
                "animal {id} eats {} at {at:?} ({:.0}% as good as at its best; food {} of {:.0})",
                src.block,
                fodder * 100.0,
                a.food,
                a.stat(species::FOOD)
            );
            match src.eaten {
                Eaten::Becomes(name) => {
                    if let Some(b) = becomes(name) {
                        w.set(at, b);
                    }
                }
                Eaten::Age(age) => {
                    w.set_prop(at, "age", &age.to_string());
                }
                Eaten::Stays => {}
            }
            if w.random_below(MOUTHFULS_A_DUNG) == 0 {
                w.spawn_item(st.pos, ItemStack::new(DUNG.id(), 1));
            }
            f[3] = now + CHEW;
            w.component_mut(id, &FORAGE).map(|c| *c = f);
            return true;
        }
    } else if now < f[3] {
        // Walking to it; or nothing near, wandering on.
        return target.is_some();
    }
    match nearest(w, s, st.pos) {
        Some(p) => {
            w.navigate(id, [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5], 1.0);
            f = [p[0] as f64, p[1] as f64, p[2] as f64, now + WALK];
        }
        None => f = [f64::NAN, 0.0, 0.0, now + NOTHING],
    }
    w.component_mut(id, &FORAGE).map(|c| *c = f);
    !f[0].is_nan()
}

/// The block a food becomes when eaten (`Eaten::Becomes`).
fn becomes(name: &str) -> Option<u32> {
    match name {
        "base:grazed_grass" => Some(GRAZED_GRASS.id()),
        _ => None,
    }
}

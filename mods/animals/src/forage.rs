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
//! Food on the ground comes first (`notes/ECOLOGY.md`, parts 5 and 8): dropped food of its diet
//! within `ITEM_LOOK` blocks (`vitals::forage::dropped`: seeds, fruit, meat, a kill's carcass;
//! spoiled food only to carrion eaters) is eaten an item a mouthful, each filling it by its weight
//! against its own mass (`animals_core::needs::fills`): a raven makes a meal of a chop, a deer
//! hardly notices a berry. A hunter with food lying near eats that before it hunts (`hunt.rs`).
//!
//! Wild plant eaters and hunters with prey get hungry (`Species::hungry_in_the_wild`,
//! `animal::upkeep`); tames forage when hungry and not told to stay (`commands::hold` comes
//! first).

use vitals::forage::{Eaten, Source};
use weft_sdk::*;

use crate::animal::{self, ANIMAL};
use crate::species::{self, SPECIES, Species};

/// Where it is going to eat and until when (not saved): x, y, z of the food (NaN: none near),
/// and the time it looks again (0: not foraging).
static FORAGE: Component<[f64; 4]> = Component::named("animal_forage");
/// The dropped item it is going to eat (0: a block, or nothing; not saved).
static ITEM: Component<u64> = Component::named("animal_forage_item");
static GRAZED_GRASS: Block = Block::named("base:grazed_grass");
static DUNG: Item = Item::named("base:dung");

/// Blocks it looks round, and up and down.
const LOOK: i32 = 8;
const LOOK_UP: i32 = 3;
/// Blocks it sees (and smells) food on the ground from.
pub const ITEM_LOOK: f64 = 12.0;
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
    r.entity_component(&ITEM, 0u64, 0);
    for s in &SPECIES {
        r.attach(s.kind, &FORAGE);
        r.attach(s.kind, &ITEM);
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
pub fn season_lush(w: &World, s: &animals_core::species::Species, pos: [i32; 3]) -> f32 {
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

/// The nearest food of its diet within `LOOK` blocks it can reach from the ground.
fn nearest(w: &World, s: &Species, pos: [f64; 3]) -> Option<[i32; 3]> {
    let p = pos.map(|v| v.floor() as i32);
    let now = season(w);
    let mut best: Option<([i32; 3], i32)> = None;
    for dx in -LOOK..=LOOK {
        for dz in -LOOK..=LOOK {
            for dy in -2..=LOOK_UP {
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                let d = dx * dx + dz * dz + dy * dy * 4;
                if best.is_none_or(|(_, b)| d < b) && food(s, w.get(at), now).is_some() {
                    best = Some((at, d));
                }
            }
        }
    }
    best.map(|(at, _)| at)
}

/// Grams of the dropped item `id` that `s` eats: food of its diet (spoiled food as carrion).
fn edible(w: &World, s: &animals_core::species::Species, id: u64) -> Option<f32> {
    let stack = w.dropped_item(id)?;
    let name = name_of(abi::entry_kind::ITEM, stack.item);
    let spoiled = vitals::food::FOODS
        .iter()
        .find(|f| f.name == name)
        .is_some_and(|f| {
            let (age, _) = base_api::food_age(&stack, w.day_time());
            vitals::food::freshness(f, age, base_api::difficulty().keep) == vitals::food::Freshness::Spoiled
        });
    let (c, grams) = vitals::forage::dropped(name, spoiled)?;
    s.eats(c).then_some(grams)
}

/// The nearest dropped food of its diet within `radius` blocks of `pos`: the item, where it is.
pub fn ground_food(w: &World, s: &animals_core::species::Species, pos: [f64; 3], radius: f64) -> Option<(u64, [f64; 3])> {
    w.near(pos, radius, None)
        .into_iter()
        .filter(|&t| edible(w, s, t).is_some())
        .filter_map(|t| Some((t, w.entity(t)?.pos)))
        .min_by(|a, b| dist2(a.1, pos).total_cmp(&dist2(b.1, pos)))
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// Within reach of its mouth: two blocks for even a small one standing beside it.
fn reach(at: [f64; 3], pos: [f64; 3]) -> bool {
    let (dx, dz) = (at[0] - pos[0], at[2] - pos[2]);
    dx * dx + dz * dz <= 4.0 && (at[1] - pos[1]).abs() <= 2.5
}

/// It eats one of the dropped item `item` (`grams` each): as much of its food as the item's
/// weight is of a meal of its mass (`needs::fills`).
fn eat_item(w: &World, id: u64, a: &animal::Animal, item: u64, grams: f32) {
    let Some(mut stack) = w.dropped_item(item) else { return };
    let own = crate::hunt::mass(w, id).unwrap_or_else(|| a.species().map_or(1.0, |s| s.bio.mass));
    let max = a.stat(species::FOOD);
    let points = (max * animals_core::needs::fills(own, grams / 1000.0)).round().max(1.0) as u16;
    animal::feed(w, id, points);
    stack.count -= 1;
    w.set_dropped_item(item, stack);
    weft_sdk::info!(
        "animal {id} eats {} off the ground ({grams} g, it {own:.3} kg): food {} of {max:.0}",
        name_of(abi::entry_kind::ITEM, stack.item),
        a.food.saturating_add(points).min(max as u16)
    );
}

/// A goal: hungry, it goes to the nearest food of its diet (on the ground first, then growing)
/// and eats there; it takes the turn while it does. With nothing near it lets the next goal
/// (wandering) take it elsewhere.
pub fn forage(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(s) = a.species() else { return false };
    let Some(mut f) = w.component(id, &FORAGE).copied() else { return false };
    let full = a.food as f32 / a.stat(species::FOOD).max(1.0);
    // Only an animal that gets hungry, and whose food the world grows or drops.
    let grows = s.eats.iter().any(|&(c, _)| vitals::forage::grows(c));
    let drops = s.eats.iter().any(|&(c, _)| vitals::forage::dropped_kind(c));
    let hungry = (s.hungry_in_the_wild() || animal::tamed(w, id)) && (grows || drops);
    if !hungry || full >= FULL || (f[3] == 0.0 && full >= HUNGRY) {
        if f[3] != 0.0 {
            w.component_mut(id, &FORAGE).map(|f| *f = [0.0; 4]);
            w.component_mut(id, &ITEM).map(|i| *i = 0);
        }
        return false;
    }
    let now = w.time() as f64;
    // Going to food on the ground, or eating it.
    let item = w.component(id, &ITEM).copied().unwrap_or(0);
    if item != 0 {
        if let (Some(e), Some(grams)) = (w.entity(item), edible(w, s, item)) {
            if reach(e.pos, st.pos) {
                if now < f[3] {
                    return true; // chewing
                }
                eat_item(w, id, &a, item, grams);
                f[3] = now + CHEW;
                w.component_mut(id, &FORAGE).map(|c| *c = f);
                return true;
            }
            if now < f[3] {
                return true; // walking to it
            }
        }
        // Gone (eaten, taken) or out of reach still: it looks again.
        w.component_mut(id, &ITEM).map(|i| *i = 0);
        f[3] = 0.0;
    }
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
    if let Some((t, p)) = ground_food(w, s, st.pos, ITEM_LOOK) {
        w.navigate(id, p, 1.0);
        w.component_mut(id, &ITEM).map(|i| *i = t);
        w.component_mut(id, &FORAGE)
            .map(|c| *c = [p[0], p[1], p[2], now + WALK]);
        return true;
    }
    match grows.then(|| nearest(w, s, st.pos)).flatten() {
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

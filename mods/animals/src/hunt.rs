//! Hunters and prey (`notes/ECOLOGY.md`, part 6). A hungry wild hunter looks round for prey of
//! the sizes it takes (`Species::prey`, from its research: as shares of its own mass), chases the
//! nearest and fights it (a knock-out kind through its anger, so a pack's kin join in); when it
//! kills, it eats the kill (`ENTITY_DIED`): it fills it by the prey's mass against its own
//! (`animals_core::needs::fills`: a mouse is a twentieth of a fox's meal, a hare all of it), and
//! what the kill drops is left as the carcass. It gives up a long chase. Animals of its own family
//! it leaves alone, and players for now.
//!
//! Its hunger is its prey's: it gets hungry in the wild where it has prey (`hungry_in_the_wild`),
//! arrives as fed as its prey are (`animal::arrive`), and a full one leaves them be.
//!
//! Prey keep watch (`wary`): a wild animal that sees a hunter able to take it runs off. In fog
//! both see less far (`base_api::sight`).

use weft_sdk::*;

use crate::dist2;

use crate::animal::{self, ANIMAL};
use crate::species::{self, OTHER_PREY, SPECIES, Species};

/// The prey it chases (0: none), since when, when it looks again, and whether it hunts with its
/// pack (1: from when the hunt began, so the pack strung out in the chase still runs it down; not
/// saved).
static HUNT: Component<[u64; 4]> = Component::named("animal_hunt");
/// Until when it runs from a hunter (not saved).
static WARY: Component<u64> = Component::named("animal_wary");

/// Blocks a hunter looks for prey and prey watches for hunters, in clear air: fog, a blizzard or
/// a dust storm close both in (`base_api::sight`).
const LOOK: f64 = 16.0;
const WATCH: f64 = 10.0;
/// It hunts under this share of its food, until this full (as foraging, `needs`).
use animals_core::needs::{FULL, HUNGRY};
/// Ticks: a chase it gives up, between looks, and a flight.
const GIVE_UP: u64 = 600;
const LOOK_EVERY: u64 = 60;
const RUN: u64 = 80;

pub fn init(r: &mut Registry) {
    r.entity_component(&HUNT, [0u64; 4], 0);
    r.entity_component(&WARY, 0u64, 0);
    for s in &SPECIES {
        r.attach(s.kind, &HUNT);
        r.attach(s.kind, &WARY);
    }
    // A kill feeds the hunter that made it, by the prey's mass against its own.
    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            let Some(a) = w.component(d.killer, &ANIMAL).copied() else { continue };
            let Some(s) = a.species().filter(|s| s.hunts()) else { continue };
            // The prey's own mass if it can still be read (its genes and growth), its kind's if not.
            let prey = mass(&w, d.entity).or_else(|| kind_mass(d.kind, None));
            let (Some(own), Some(prey)) = (mass(&w, d.killer), prey) else { continue };
            let max = a.stat(species::FOOD);
            let meal = (max * animals_core::needs::fills(own, prey))
                .round()
                .max(1.0) as u16;
            animal::feed(&w, d.killer, meal);
            weft_sdk::info!(
                "{} {} eats its kill ({:.1} kg of {}): food {} of {max:.0}",
                s.name,
                d.killer,
                prey,
                name_of(abi::entry_kind::ENTITY, d.kind),
                a.food.saturating_add(meal).min(max as u16)
            );
            w.component_mut(d.killer, &HUNT).map(|h| *h = [0; 4]);
        }
    });
}

/// How fed its prey are where `pos` is this season, 0 to 1: the best fed of its prey that live in
/// that biome (`needs::prey_in`), by how much of their own food grows there now
/// (`forage::season_lush`). `None`: none of its prey live there (only farm animals, which don't
/// thin with the seasons).
pub fn prey_lush(w: &World, s: &Species, pos: [i32; 3]) -> Option<f32> {
    let biome = name_of(abi::entry_kind::BIOME, w.biome(pos));
    animals_core::needs::prey_in(s, biome)
        .map(|o| crate::forage::season_lush(w, o, pos))
        .reduce(f32::max)
}

/// An egg's mass, kg, at a Velociraptor's egg's size (about a goose egg's; `eggs::egg_size` scales
/// the others): eggs lying in a nest are food for the egg eaters of their sizes (the report's tier
/// 3: "eggs are real food and nests are built already"), Citipati, the raptors, the foxes.
const EGG_KG: f32 = 0.2;

/// The species `index` an egg `id` holds, if it is one.
fn egg_of(w: &World, id: u64) -> Option<usize> {
    let e = w.component(id, &crate::eggs::EGG)?;
    (e.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
}

/// A player's egg (its nest's, or one laid by a tame): the wild's egg eaters leave those alone.
fn owned_egg(w: &World, id: u64) -> bool {
    egg_of(w, id).is_some() && w.component(id, &animal::OWNER).is_some_and(|o| o[0] != 0)
}

/// The mass in kg of an entity of `kind`: a species' grown mass (scaled by `size`, its genes and
/// growth), an egg's, or a farm animal's wild kind's (`OTHER_PREY`). `None`: not prey to anything.
fn kind_mass(kind: u32, size: Option<f32>) -> Option<f32> {
    if kind == crate::eggs::EGG_ENTITY.id() {
        return Some(EGG_KG * size.unwrap_or(1.0).powi(3));
    }
    if let Some(s) = SPECIES.iter().find(|s| s.kind.id() == kind) {
        return Some(s.bio.mass * size.unwrap_or(1.0).powi(3));
    }
    let name = name_of(abi::entry_kind::ENTITY, kind);
    OTHER_PREY
        .iter()
        .find(|&&(n, _)| n == name)
        .map(|&(_, kg)| kg)
}

/// What `id` weighs now.
pub(crate) fn mass(w: &World, id: u64) -> Option<f32> {
    let e = w.entity(id)?;
    if let Some(index) = egg_of(w, id) {
        return kind_mass(e.kind, Some(crate::eggs::egg_size(index)));
    }
    let size = w.component(id, &ANIMAL).and_then(|a| {
        let index = (a.species as usize).checked_sub(1)?;
        // Against its species' average size: its row's mass is an average grown one's.
        Some(crate::coats::size_of(w, id)? / crate::coats::mean_size(index) * crate::breeding::growth(w, id, index))
    });
    kind_mass(e.kind, size)
}

/// The family of `id`, if it's one of the species: by its animal record, or by its kind before it
/// has one (one spawned this tick, which a packmate otherwise took for prey).
fn family(w: &World, id: u64) -> Option<&'static str> {
    w.component(id, &ANIMAL)
        .and_then(|a| a.species())
        .map(|s| s.family.name)
        .or_else(|| {
            let kind = w.entity(id)?.kind;
            SPECIES
                .iter()
                .find(|s| s.kind.id() == kind)
                .map(|s| s.family.name)
        })
        // An egg is its species' family's: a hunter leaves its own kind's eggs.
        .or_else(|| egg_of(w, id).map(|i| SPECIES[i].family.name))
}

/// A goal for wild hunters: hungry, it goes after the nearest prey it can take; it takes the turn
/// while it chases (a knock-out kind fights through its anger, `knockout::behave`).
pub fn hunt(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(s) = a.species().filter(|s| s.hunts()) else { return false };
    let Some(mut h) = w.component(id, &HUNT).copied() else { return false };
    let full = a.food as f32 / a.stat(species::FOOD).max(1.0);
    if animal::tamed(w, id) || full >= FULL || (h[0] == 0 && full >= HUNGRY) {
        if h[0] != 0 {
            w.component_mut(id, &HUNT).map(|h| *h = [0; 4]);
        }
        return false;
    }
    let now = w.time();
    let knockout = w.component(id, &crate::knockout::KNOCKOUT).is_some();
    if h[0] != 0 {
        let alive = w
            .entity(h[0])
            .is_some_and(|e| e.health > 0.0 && dist2(e.pos, st.pos) < (LOOK * 1.5).powi(2));
        // A pack runs its prey down: they take turns at the front, faster than one alone could
        // keep up, and keep at it twice as long (a wolf pack's chase of an elk lasts minutes).
        let pack = h[3] == 1;
        if alive && now < h[1] + if pack { 2 * GIVE_UP } else { GIVE_UP } {
            if knockout {
                // Its anger does the chasing; kept up while the chase lasts.
                if w.component(id, &crate::knockout::KNOCKOUT)
                    .is_some_and(|k| k.angry == 0 && k.out == 0)
                {
                    crate::knockout::anger(w, id, h[0]);
                }
                return false;
            }
            crate::commands::fight_at(w, id, st, h[0], if pack { PACK_SPEED } else { 1.4 });
            return true;
        }
        weft_sdk::info!("{} {id} gives up on {} ({})", s.name, h[0], if alive { "too long" } else { "gone" });
        h = [0, 0, now + LOOK_EVERY, 0];
    }
    if now < h[2] {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        return false;
    }
    h[2] = now + LOOK_EVERY;
    // Food lying near (a carcass, dropped meat) is a meal without a chase: it eats that first
    // (`forage.rs`).
    if crate::forage::ground_food(w, s, st.pos, crate::forage::ITEM_LOOK).is_some() {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        return false;
    }
    let own = mass(w, id).unwrap_or(s.bio.mass);
    // A pack's kind with others of it near hunts together: bigger prey (`Species::takes_with`).
    let pack = if s.pack_prey().is_some() { pack_near(w, id, st) } else { 1 };
    let prey = w
        .near_bodies(st.pos, LOOK * base_api::sight(w, st.pos) as f64)
        .into_iter()
        .filter(|&t| t != id && family(w, t) != Some(s.family.name) && !owned_egg(w, t))
        .filter_map(|t| Some((t, w.entity(t)?, mass(w, t)?)))
        .filter(|(_, e, kg)| e.health > 0.0 && s.takes_with(own, *kg, pack))
        .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)));
    let Some((t, _, kg)) = prey else {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        return false;
    };
    weft_sdk::info!("{} {id} hunts {} ({kg:.1} kg, a pack of {pack})", s.name, t);
    if pack >= species::PACK && !s.takes(own, kg) {
        rally(w, id, st, t, now);
    }
    h = [t, now, now + LOOK_EVERY, (pack >= species::PACK) as u64];
    w.component_mut(id, &HUNT).map(|c| *c = h);
    if knockout {
        crate::knockout::anger(w, id, t);
        return false;
    }
    crate::commands::fight(w, id, st, t);
    true
}

/// Its kind's speed a pack chases at (one alone: 1.4, `commands::fight`): a wolf 5.5 blocks a
/// second, at this nearly 10, past a fleeing deer's 8.
const PACK_SPEED: f32 = 1.75;

/// Blocks within which a pack's kind hunt together.
const PACK_RANGE: f64 = 16.0;

/// How many wild ones of `id`'s kind are near it, itself among them.
fn pack_near(w: &World, id: u64, st: &EntityState) -> u32 {
    let kind = w.entity(id).map(|e| e.kind);
    w.near(st.pos, PACK_RANGE, None)
        .into_iter()
        .filter(|&o| o == id || (w.entity(o).map(|e| e.kind) == kind && !animal::tamed(w, o)))
        .count() as u32
}

/// Big prey only a pack takes: the others of the pack near that aren't hunting already go after it
/// too, so it isn't one wolf against a red deer.
fn rally(w: &World, id: u64, st: &EntityState, target: u64, now: u64) {
    let kind = w.entity(id).map(|e| e.kind);
    for o in w.near(st.pos, PACK_RANGE, None) {
        if o == id || w.entity(o).map(|e| e.kind) != kind || animal::tamed(w, o) {
            continue;
        }
        if let Some(c) = w.component_mut(o, &HUNT)
            && c[0] == 0
        {
            *c = [target, now, now + LOOK_EVERY, 1];
            weft_sdk::info!("animal {o} joins {id}'s hunt of {target}");
        }
    }
}

/// A goal for wild animals: a hunter near that could take it, and it runs the other way.
pub fn wary(w: &World, id: u64, st: &EntityState) -> bool {
    if animal::tamed(w, id) {
        return false;
    }
    let now = w.time();
    if w.component(id, &WARY).is_some_and(|&until| now < until) {
        return true;
    }
    if (now + id) % 10 != 0 {
        return false;
    }
    let Some(me) = mass(w, id) else { return false };
    let mine = family(w, id);
    let hunter = w
        .near_bodies(st.pos, WATCH * base_api::sight(w, st.pos) as f64)
        .into_iter()
        .filter(|&t| t != id && family(w, t) != mine && !animal::tamed(w, t))
        .filter_map(|t| {
            let s = w.component(t, &ANIMAL)?.species()?;
            let e = w.entity(t)?;
            s.takes_at_most(mass(w, t)?, me).then_some(e) // a pack's kind with its pack's reach
        })
        .min_by(|a, b| dist2(a.pos, st.pos).total_cmp(&dist2(b.pos, st.pos)));
    let Some(h) = hunter else { return false };
    let (x, z) = (st.pos[0] - h.pos[0], st.pos[2] - h.pos[2]);
    let n = (x * x + z * z).sqrt().max(0.01);
    w.navigate(id, [st.pos[0] + x / n * 10.0, st.pos[1], st.pos[2] + z / n * 10.0], 1.6);
    w.component_mut(id, &WARY).map(|u| *u = now + RUN);
    true
}

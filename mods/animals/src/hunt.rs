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
//! Prey keep watch (`wary`): a wild animal that sees a hunter able to take it runs off.

use weft_sdk::*;

use crate::animal::{self, ANIMAL};
use crate::species::{self, OTHER_PREY, SPECIES, Species};

/// The prey it chases (0: none), since when, and when it looks again (not saved).
static HUNT: Component<[u64; 3]> = Component::named("animal_hunt");
/// Until when it runs from a hunter (not saved).
static WARY: Component<u64> = Component::named("animal_wary");

/// Blocks a hunter looks for prey and prey watches for hunters.
const LOOK: f64 = 16.0;
const WATCH: f64 = 10.0;
/// It hunts under this share of its food, until this full (as foraging, `forage.rs`).
const HUNGRY: f32 = 0.6;
const FULL: f32 = 0.95;
/// Ticks: a chase it gives up, between looks, and a flight.
const GIVE_UP: u64 = 600;
const LOOK_EVERY: u64 = 60;
const RUN: u64 = 80;

pub fn init(r: &mut Registry) {
    r.entity_component(&HUNT, [0u64; 3], 0);
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
            let (Some(own), Some(prey)) = (mass(&w, d.killer), kind_mass(d.kind, None)) else { continue };
            let max = a.stat(species::FOOD);
            let meal = (max * animals_core::needs::fills(own, prey)).round().max(1.0) as u16;
            animal::feed(&w, d.killer, meal);
            weft_sdk::info!(
                "{} {} eats its kill ({:.1} kg of {}): food {} of {max:.0}",
                s.name,
                d.killer,
                prey,
                name_of(abi::entry_kind::ENTITY, d.kind),
                a.food.saturating_add(meal).min(max as u16)
            );
            w.component_mut(d.killer, &HUNT).map(|h| *h = [0; 3]);
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

/// The mass in kg of an entity of `kind`: a species' grown mass (scaled by `size`, its genes and
/// growth), or a farm animal's wild kind's (`OTHER_PREY`). `None`: not prey to anything.
fn kind_mass(kind: u32, size: Option<f32>) -> Option<f32> {
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
fn mass(w: &World, id: u64) -> Option<f32> {
    let e = w.entity(id)?;
    let size = w.component(id, &ANIMAL).and_then(|a| {
        let index = (a.species as usize).checked_sub(1)?;
        let g = w.component(id, &crate::coats::GENOME)?;
        Some(crate::coats::size(g) * crate::breeding::growth(w, id, index))
    });
    kind_mass(e.kind, size)
}

/// The family of `id`, if it's one of the species.
fn family(w: &World, id: u64) -> Option<&'static str> {
    w.component(id, &ANIMAL)
        .and_then(|a| a.species())
        .map(|s| s.family.name)
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
            w.component_mut(id, &HUNT).map(|h| *h = [0; 3]);
        }
        return false;
    }
    let now = w.time();
    let knockout = w.component(id, &crate::knockout::KNOCKOUT).is_some();
    if h[0] != 0 {
        let alive = w
            .entity(h[0])
            .is_some_and(|e| e.health > 0.0 && dist2(e.pos, st.pos) < (LOOK * 1.5).powi(2));
        if alive && now < h[1] + GIVE_UP {
            if knockout {
                // Its anger does the chasing; kept up while the chase lasts.
                if w.component(id, &crate::knockout::KNOCKOUT)
                    .is_some_and(|k| k.angry == 0 && k.out == 0)
                {
                    crate::knockout::anger(w, id, h[0]);
                }
                return false;
            }
            crate::commands::fight(w, id, st, h[0]);
            return true;
        }
        weft_sdk::info!("{} {id} gives up on {} ({})", s.name, h[0], if alive { "too long" } else { "gone" });
        h = [0, 0, now + LOOK_EVERY];
    }
    if now < h[2] {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        return false;
    }
    h[2] = now + LOOK_EVERY;
    let own = mass(w, id).unwrap_or(s.bio.mass);
    let prey = w
        .near_bodies(st.pos, LOOK)
        .into_iter()
        .filter(|&t| t != id && family(w, t) != Some(s.family.name))
        .filter_map(|t| Some((t, w.entity(t)?, mass(w, t)?)))
        .filter(|(_, e, kg)| e.health > 0.0 && s.takes(own, *kg))
        .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)));
    let Some((t, _, kg)) = prey else {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        return false;
    };
    weft_sdk::info!("{} {id} hunts {} ({kg:.1} kg)", s.name, t);
    h = [t, now, now + LOOK_EVERY];
    w.component_mut(id, &HUNT).map(|c| *c = h);
    if knockout {
        crate::knockout::anger(w, id, t);
        return false;
    }
    crate::commands::fight(w, id, st, t);
    true
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
        .near_bodies(st.pos, WATCH)
        .into_iter()
        .filter(|&t| t != id && family(w, t) != mine && !animal::tamed(w, t))
        .filter_map(|t| {
            let s = w.component(t, &ANIMAL)?.species()?;
            let e = w.entity(t)?;
            s.takes(mass(w, t)?, me).then_some(e)
        })
        .min_by(|a, b| dist2(a.pos, st.pos).total_cmp(&dist2(b.pos, st.pos)));
    let Some(h) = hunter else { return false };
    let (x, z) = (st.pos[0] - h.pos[0], st.pos[2] - h.pos[2]);
    let n = (x * x + z * z).sqrt().max(0.01);
    w.navigate(id, [st.pos[0] + x / n * 10.0, st.pos[1], st.pos[2] + z / n * 10.0], 1.6);
    w.component_mut(id, &WARY).map(|u| *u = now + RUN);
    true
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

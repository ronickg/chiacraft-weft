//! Drinking (`notes/ECOLOGY.md`, part 4): each species needs its own water, by its mass (a
//! mammal's `0.099 · kg^0.9` litres a day, a cold-blooded one's a fifth) and by how much of it it
//! drinks rather than gets from its food (`animals_core::needs::DRINKS`: a deer drinks most of
//! its, a vole hardly any, fish live in it). It gets thirstier once a second (`thirst`), faster out
//! of its comfort; rain on it and wading slake it a little. Thirsty, it walks to the nearest water
//! it can stand beside and drinks a gulp at a time until slaked (`drink`, before it forages).
//!
//! Thirst matters as hunger does: a parched animal is as weak as a starving one
//! (`comfort::condition`), and a wild one parched dies of it in a game day; a tame suffers.

use animals_core::needs::{self, GULP, GULP_TICKS, PARCHED_SECS, SLAKED, THIRSTY};
use weft_sdk::*;

use crate::animal::{self, ANIMAL, Animal};
use crate::species::SPECIES;

/// Where it stands to drink (x, y, z; NaN: no water near), and when it looks again or takes its
/// next gulp (0: not drinking). Not saved.
static DRINK: Component<[f64; 4]> = Component::named("animal_drink");

/// Blocks it looks round for water, and down and up.
const LOOK: i32 = 16;
const LOOK_DOWN: i32 = 4;
const LOOK_UP: i32 = 2;
/// Ticks: walking before it looks again, and waiting when no water is near.
const WALK: f64 = 200.0;
const NOTHING: f64 = 400.0;
/// Share of its water a second of rain on it, or of standing in water, gives.
const RAIN: f32 = 0.005;
const WADING: f32 = 0.05;

pub fn init(r: &mut Registry) {
    r.entity_component(&DRINK, [0.0; 4], 0);
    for s in &SPECIES {
        r.attach(s.kind, &DRINK);
    }
}

/// How long a full drink lasts `id` now, in seconds (`needs::drink_lasts`, by its mass now);
/// `None`: it never needs to drink.
fn lasts(w: &World, id: u64, a: &Animal) -> Option<f32> {
    let s = a.species()?;
    let kg = crate::hunt::mass(w, id).unwrap_or(s.bio.mass);
    needs::drink_lasts(s, kg)
}

/// Once a second, from `animal::upkeep` (for those that get hungry): it gets thirstier, `rate`
/// times as fast out of its comfort; rain on it or wading slakes it a little. Parched, a wild one
/// loses health (dead in `PARCHED_SECS` at full health).
pub fn thirst(w: &World, id: u64, a: &Animal, st: &EntityState, rate: f32) {
    let Some(secs) = lasts(w, id, a) else { return };
    let eye = [st.pos[0], st.pos[1] + st.size[2] as f64, st.pos[2]].map(|v| v.floor() as i32);
    let rained = w.sky_light(eye) >= 15 && w.raining_at(eye);
    let mut t = a.thirst + rate / secs;
    if rained {
        t -= RAIN;
    }
    if st.in_liquid != 0 {
        t -= WADING;
    }
    let t = t.clamp(0.0, 1.0);
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        a.thirst = t;
    }
    if t >= 1.0 && !animal::tamed(w, id) {
        w.damage(id, st.max_health / PARCHED_SECS, 0, damage::STARVE);
    }
}

/// Whether the block at `pos` is open water to drink from: its surface, air above it.
fn open_water(w: &World, pos: [i32; 3]) -> bool {
    crate::comfort::WATER.in_cell(w.get(pos)) && w.get([pos[0], pos[1] + 1, pos[2]]) == 0
}

/// The nearest place within `LOOK` blocks where it can stand on dry ground beside open water.
fn nearest_water(w: &World, pos: [f64; 3]) -> Option<[f64; 3]> {
    let p = pos.map(|v| v.floor() as i32);
    let mut best: Option<([f64; 3], f64)> = None;
    for dx in -LOOK..=LOOK {
        for dz in -LOOK..=LOOK {
            for dy in -LOOK_DOWN..=LOOK_UP {
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                let d = (dx * dx + dz * dz + dy * dy * 4) as f64;
                if best.as_ref().is_some_and(|&(_, b)| d >= b) || !open_water(w, at) {
                    continue;
                }
                // A shore beside it: dry ground at its surface or a block above.
                for (sx, sz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (x, z) = (at[0] + sx, at[2] + sz);
                    let Some(y) = crate::comfort::ground(w, x, at[1] + 1, z) else { continue };
                    if (y - (at[1] + 1)).abs() > 1 || crate::comfort::WATER.in_cell(w.get([x, y, z])) {
                        continue;
                    }
                    let spot = [x as f64 + 0.5, y as f64, z as f64 + 0.5];
                    let d = (spot[0] - pos[0]).powi(2) + (spot[2] - pos[2]).powi(2) + (spot[1] - pos[1]).powi(2) * 4.0;
                    if best.as_ref().is_none_or(|&(_, b)| d < b) {
                        best = Some((spot, d));
                    }
                }
            }
        }
    }
    best.map(|(spot, _)| spot)
}

/// Whether it is drinking or on its way to water (a resting animal settles only when it isn't).
pub fn busy(w: &World, id: u64) -> bool {
    w.component(id, &DRINK)
        .is_some_and(|d| d[3] != 0.0 && !d[0].is_nan())
}

/// A goal (run first by the forage goal, `forage::forage`): thirsty, it goes to the nearest water
/// it can stand beside and drinks a gulp at a time until slaked; it takes the turn while it does.
/// Standing in water it drinks where it is. With no water near it lets foraging and wandering
/// take it elsewhere.
pub fn drink(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(s) = a.species() else { return false };
    let Some(mut d) = w.component(id, &DRINK).copied() else { return false };
    let water = 1.0 - a.thirst;
    let gets_thirsty = (s.hungry_in_the_wild() || animal::tamed(w, id)) && lasts(w, id, &a).is_some();
    if !gets_thirsty || water >= SLAKED || (d[3] == 0.0 && water >= THIRSTY) {
        if d[3] != 0.0 {
            w.component_mut(id, &DRINK).map(|d| *d = [0.0; 4]);
        }
        return false;
    }
    let now = w.time() as f64;
    let spot = (!d[0].is_nan() && d[3] != 0.0).then(|| [d[0], d[1], d[2]]);
    let there = st.in_liquid != 0
        || spot.is_some_and(|p| {
            let (dx, dz) = (p[0] - st.pos[0], p[2] - st.pos[2]);
            dx * dx + dz * dz <= 2.25 && (p[1] - st.pos[1]).abs() <= 1.5
        });
    if there {
        if now < d[3] {
            return true; // swallowing
        }
        if let Some(a) = w.component_mut(id, &ANIMAL) {
            a.thirst = (a.thirst - GULP).max(0.0);
        }
        if w.navigating(id) {
            w.navigate(id, st.pos, 0.0);
        }
        weft_sdk::info!("{} {id} drinks: water {:.0}%", s.name, (water + GULP).min(1.0) * 100.0);
        let p = spot.unwrap_or(st.pos);
        d = [p[0], p[1], p[2], now + GULP_TICKS as f64];
        w.component_mut(id, &DRINK).map(|c| *c = d);
        return true;
    }
    if now < d[3] {
        // Walking to it; or no water near, foraging and wandering on.
        return spot.is_some();
    }
    match nearest_water(w, st.pos) {
        Some(p) => {
            weft_sdk::info!("{} {id} is thirsty (water {:.0}%): to water at {p:?}", s.name, water * 100.0);
            w.navigate(id, p, 1.0);
            d = [p[0], p[1], p[2], now + WALK];
        }
        None => d = [f64::NAN, 0.0, 0.0, now + NOTHING],
    }
    w.component_mut(id, &DRINK).map(|c| *c = d);
    !d[0].is_nan()
}

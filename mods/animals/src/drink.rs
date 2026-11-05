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
/// The last two places it drank from (x, y, z, and 1 where there is one), the newest first: a
/// thirsty animal goes back to one near before it looks over all the blocks round it again
/// (`reports/Creature AI.md`, step 1). Not saved.
static WATERS: Component<[[f32; 4]; 2]> = Component::named("animal_waters");

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
    r.entity_component(&WATERS, [[0.0f32; 4]; 2], 0);
    for s in &SPECIES {
        r.attach(s.kind, &DRINK);
        r.attach(s.kind, &WATERS);
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
pub fn thirst(w: &World, id: u64, a: &Animal, st: &EntityState, rate: f32, secs: f32) {
    let Some(lasts) = lasts(w, id, a) else { return };
    let eye = [st.pos.x, st.pos.y + st.size.eye as f64, st.pos.z].map(|v| v.floor() as i32);
    let rained = w.sky_light(eye) >= 15 && w.raining_at(eye);
    let mut t = a.thirst + rate / lasts * secs;
    if rained {
        t -= RAIN * secs;
    }
    if st.in_liquid != 0 {
        t -= WADING * secs;
    }
    let t = t.clamp(0.0, 1.0);
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        a.thirst = t;
    }
    if t >= 1.0 && !animal::tamed(w, id) {
        w.damage(id, st.max_health / PARCHED_SECS * secs, 0, damage_cause::STARVE);
    }
}

/// Whether the block at `pos` is open water to drink from: its surface, air above it.
fn open_water(w: &World, pos: [i32; 3]) -> bool {
    crate::comfort::WATER.at(w, pos) && w.block([pos[0], pos[1] + 1, pos[2]]) == 0
}

/// Blocks round a remembered place it looks for its shore again, and how far it goes back to one.
const AGAIN: i32 = 2;
const RECALL: f64 = 2.0 * LOOK as f64;

/// The nearest place to `pos` where it can stand on dry ground beside open water: at a place it
/// drank from before within `RECALL` blocks if that is still a shore, else within `LOOK` blocks.
fn find_water(w: &World, id: u64, pos: [f64; 3]) -> Option<[f64; 3]> {
    let mut known = w.component(id, &WATERS).unwrap_or_default();
    let near = |k: &[f32; 4]| k[3] != 0.0 && (k[0] as f64 - pos[0]).powi(2) + (k[2] as f64 - pos[2]).powi(2) <= RECALL * RECALL;
    let mut order: Vec<&[f32; 4]> = known.iter().filter(|k| near(k)).collect();
    order.sort_by(|a, b| {
        crate::dist2([a[0], a[1], a[2]].map(f64::from), pos).total_cmp(&crate::dist2([b[0], b[1], b[2]].map(f64::from), pos))
    });
    let again = order
        .into_iter()
        .find_map(|k| nearest_water(w, [k[0], k[1], k[2]].map(|v| v.floor() as i32), AGAIN, pos));
    let found = again.or_else(|| nearest_water(w, pos.map(|v| v.floor() as i32), LOOK, pos))?;
    let f = [found[0] as f32, found[1] as f32, found[2] as f32, 1.0];
    if known[0][..3] != f[..3] {
        known = [f, known[0]];
        if let Some(c) = w.component_mut(id, &WATERS) {
            *c = known;
        }
    }
    Some(found)
}

/// The nearest place to `pos` within `r` blocks of `p` where it can stand on dry ground beside
/// open water.
fn nearest_water(w: &World, p: [i32; 3], r: i32, pos: [f64; 3]) -> Option<[f64; 3]> {
    let mut best: Option<([f64; 3], f64)> = None;
    for dx in -r..=r {
        for dz in -r..=r {
            for dy in -LOOK_DOWN..=LOOK_UP {
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                let d =
                    (at[0] as f64 + 0.5 - pos[0]).powi(2) + (at[2] as f64 + 0.5 - pos[2]).powi(2) + (at[1] as f64 - pos[1]).powi(2) * 4.0;
                // Its shore is within a block of it: no nearer than this less a block and a half.
                if best
                    .as_ref()
                    .is_some_and(|&(_, b)| d.sqrt() - 1.5 >= b.sqrt())
                    || !open_water(w, at)
                {
                    continue;
                }
                // A shore beside it: dry ground at its surface or a block above.
                for (sx, sz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (x, z) = (at[0] + sx, at[2] + sz);
                    let Some(y) = crate::comfort::ground(w, x, at[1] + 1, z) else { continue };
                    if (y - (at[1] + 1)).abs() > 1 || crate::comfort::WATER.at(w, [x, y, z]) {
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
    // As foraging (`forage::busy`): a trip left from before it was slaked isn't one.
    let thirsty = w
        .component(id, &ANIMAL)
        .is_some_and(|a| a.thirst > 1.0 - animals_core::needs::SLAKED);
    thirsty
        && w.component(id, &DRINK)
            .is_some_and(|d| d[3] != 0.0 && !d[0].is_nan())
}

/// A goal (in the needs band, `choose.rs`, weighed against hunger): thirsty, it goes to the nearest water
/// it can stand beside and drinks a gulp at a time until slaked; it takes the turn while it does.
/// Standing in water it drinks where it is. With no water near it lets foraging and wandering
/// take it elsewhere.
pub fn drink(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    let Some(s) = a.species() else { return false };
    let Some(mut d) = w.component(id, &DRINK) else { return false };
    let water = 1.0 - a.thirst;
    // The cheap reasons first: most animals most of the time aren't thirsty.
    let calm = water >= SLAKED || (d[3] == 0.0 && water >= THIRSTY);
    if calm || !((s.hungry_in_the_wild() || animal::tamed(w, id)) && lasts(w, id, &a).is_some()) {
        if d[3] != 0.0 {
            w.component_mut(id, &DRINK).map(|d| *d = [0.0; 4]);
        }
        return false;
    }
    let now = w.game_time() as f64;
    let spot = (!d[0].is_nan() && d[3] != 0.0).then(|| [d[0], d[1], d[2]]);
    let there = st.in_liquid != 0
        || spot.is_some_and(|p| {
            let (dx, dz) = (p[0] - st.pos.x, p[2] - st.pos.z);
            dx * dx + dz * dz <= 2.25 && (p[1] - st.pos.y).abs() <= 1.5
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
        weft_sdk::debug!("{} {id} drinks: water {:.0}%", s.name, (water + GULP).min(1.0) * 100.0);
        let p = spot.unwrap_or(st.pos.to_array());
        d = [p[0], p[1], p[2], now + GULP_TICKS as f64];
        w.component_mut(id, &DRINK).map(|c| *c = d);
        return true;
    }
    if now < d[3] {
        // Walking to it; or no water near, foraging and wandering on.
        return spot.is_some();
    }
    match find_water(w, id, st.pos.to_array()) {
        Some(p) => {
            weft_sdk::debug!("{} {id} is thirsty (water {:.0}%): to water at {p:?}", s.name, water * 100.0);
            w.navigate(id, p, 1.0);
            d = [p[0], p[1], p[2], now + WALK];
        }
        None => d = [f64::NAN, 0.0, 0.0, now + NOTHING],
    }
    w.component_mut(id, &DRINK).map(|c| *c = d);
    !d[0].is_nan()
}

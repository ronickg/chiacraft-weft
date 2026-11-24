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
/// Seconds between its looks round for water it isn't after yet (`notice`).
const NOTICE_SECS: u64 = 30;
/// Share of its water a second of rain on it, or of standing in water, gives.
const RAIN: f32 = 0.005;
const WADING: f32 = 0.05;

/// Death by thirst (`thirst`): its own cause, so a run's log tells a parched animal from a starved
/// one (ai1124_savanna3's "starve" deaths were thirst).
pub(crate) static THIRST: DamageCause = DamageCause::named("thirst");

pub fn init(r: &mut Registry) {
    r.damage_cause(&THIRST)
        .bypasses_armor()
        .death_message("death.animals.thirst");
    r.entity_component(&DRINK, [0.0; 4], 0);
    r.entity_component(&WATERS, [[0.0f32; 4]; 2], 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&DRINK);
        r.edit(s.kind).attach(&WATERS);
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
    if (w.game_time() / 20 + id).is_multiple_of(NOTICE_SECS) && !animal::tamed(w, id) {
        notice(w, id, st.pos.to_array());
    }
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
        w.damage(id, Damage::new(st.max_health / PARCHED_SECS * secs, &THIRST));
    }
}

/// Whether open water lies beside `st`'s feet (the eight blocks round it, level with its feet or a
/// block down), so it can reach it with its mouth.
fn beside_water(w: &World, st: &EntityState) -> bool {
    let [x, y, z] = [st.pos.x, st.pos.y, st.pos.z].map(|v| v.floor() as i32);
    (-1..=1).any(|dx| (-1..=1).any(|dz| (dx, dz) != (0, 0) && (-1..=0).any(|dy| open_water(w, [x + dx, y + dy, z + dz]))))
}

/// Whether the block at `pos` is open water to drink from: its surface, air above it.
fn open_water(w: &World, pos: [i32; 3]) -> bool {
    crate::comfort::WATER.at(w, pos) && w.block([pos[0], pos[1] + 1, pos[2]]) == 0
}

/// Blocks round a remembered place it looks for its shore again, and how far it goes back to one.
const AGAIN: i32 = 2;
const RECALL: f64 = 2.0 * LOOK as f64;

/// The nearest place to `pos` where it can stand on dry ground beside open water: at a place it
/// drank from before within `RECALL` blocks if that is still a shore, else within `LOOK` blocks,
/// else the nearest one it or its kind near remember within `groups::WATER_TREK` (`trek`).
fn find_water(w: &World, id: u64, kind: &EntityKind, pos: [f64; 3], driven: bool) -> Option<[f64; 3]> {
    let mut known = w.component(id, &WATERS).unwrap_or_default();
    let near = |k: &[f32; 4]| k[3] != 0.0 && (k[0] as f64 - pos[0]).powi(2) + (k[2] as f64 - pos[2]).powi(2) <= RECALL * RECALL;
    let mut order: Vec<&[f32; 4]> = known.iter().filter(|k| near(k)).collect();
    order.sort_by(|a, b| {
        crate::dist2([a[0], a[1], a[2]].map(f64::from), pos).total_cmp(&crate::dist2([b[0], b[1], b[2]].map(f64::from), pos))
    });
    let again = order
        .into_iter()
        .find_map(|k| nearest_water(w, [k[0], k[1], k[2]].map(|v| v.floor() as i32), AGAIN, pos));
    let found = match again.or_else(|| nearest_water(w, pos.map(|v| v.floor() as i32), LOOK, pos)) {
        Some(f) => f,
        None => trek(w, id, kind, pos)?,
    };
    // A waterhole where its kind was killed it keeps off unless thirst drives it (`memory`).
    if !driven && crate::memory::fear_at(w, found) >= animals_core::memory::AVOID {
        weft_sdk::debug!("animal {id} keeps off feared water at {found:?}");
        return None;
    }
    let f = [found[0] as f32, found[1] as f32, found[2] as f32, 1.0];
    if known[0][..3] != f[..3] {
        known = [f, known[0]];
        if let Some(c) = w.component_mut(id, &WATERS) {
            *c = known;
        }
    }
    Some(found)
}

/// Nothing near: the nearest water that it (`known`) or a wild one of its kind within
/// `senses::KIN` remembers drinking at, up to `groups::WATER_TREK` blocks off, if it is still a
/// shore. Its herd goes with it (`group.rs`). Looked for only when nothing is near, at most every
/// `NOTHING` ticks.
fn trek(w: &World, id: u64, kind: &EntityKind, pos: [f64; 3]) -> Option<[f64; 3]> {
    let mut places = herd_waters(w, id, kind, pos);
    let far = animals_core::groups::WATER_TREK as f64;
    places.retain(|(p, _)| crate::dist2(*p, pos) <= far * far);
    places.sort_by(|a, b| crate::dist2(a.0, pos).total_cmp(&crate::dist2(b.0, pos)));
    places.into_iter().find_map(|(p, from)| {
        let found = nearest_water(w, p.map(|v| v.floor() as i32), AGAIN, pos)?;
        weft_sdk::info!(
            "animal {id} treks to water its herd remembers at {:.0} {:.0} {:.0} ({:.0} blocks, remembered by {from})",
            found[0],
            found[1],
            found[2],
            crate::dist2(found, pos).sqrt()
        );
        Some(found)
    })
}

/// A wild drinker new to the world notes the water near where it starts (`notice`), as it would
/// know its home range's: deer spawned 10-14 blocks from a pool went off before their first
/// 30-second look and never knew it (ai1124_graze7: one died parched 83 blocks off).
pub fn arrive(w: &World, id: u64, a: &Animal, pos: [f64; 3]) {
    if !animal::tamed(w, id) && lasts(w, id, a).is_some() {
        notice(w, id, pos);
    }
}

/// Water `id` sees in passing (within `LOOK` blocks) it remembers, as if it had drunk there,
/// when it knows none within `RECALL` already: a wild one knows the water in its range before it
/// is thirsty (ai1124_graze2: the deer grazed away from the pool they were spawned beside before
/// they ever drank, and never found it again).
fn notice(w: &World, id: u64, pos: [f64; 3]) {
    let mut known = w.component(id, &WATERS).unwrap_or_default();
    if known
        .iter()
        .any(|k| k[3] != 0.0 && (k[0] as f64 - pos[0]).powi(2) + (k[2] as f64 - pos[2]).powi(2) <= RECALL * RECALL)
    {
        return;
    }
    let Some(found) = nearest_water(w, pos.map(|v| v.floor() as i32), LOOK, pos) else { return };
    known = [[found[0] as f32, found[1] as f32, found[2] as f32, 1.0], known[0]];
    if let Some(c) = w.component_mut(id, &WATERS) {
        *c = known;
    }
    weft_sdk::debug!("animal {id} notes water at {found:?}");
}

/// The places with water `id` and the wild ones of its kind within `senses::KIN` of `pos`
/// remember (with who remembers each).
fn herd_waters(w: &World, id: u64, kind: &EntityKind, pos: [f64; 3]) -> Vec<([f64; 3], u64)> {
    let mut places: Vec<([f64; 3], u64)> = Vec::new();
    for o in std::iter::once(id).chain(
        w.entities_near(pos, crate::senses::KIN, Some(kind))
            .into_iter()
            .filter(|&o| o != id),
    ) {
        if o != id && crate::animal::tamed(w, o) {
            continue;
        }
        let theirs = w.component(o, &WATERS).unwrap_or_default();
        places.extend(
            theirs
                .iter()
                .filter(|k| k[3] != 0.0)
                .map(|k| ([k[0], k[1], k[2]].map(f64::from), o)),
        );
    }
    places
}

/// How far (x and z) the water nearest `at` that `id`'s herd knows lies from it; None: they know
/// none.
pub fn herd_water_from(w: &World, id: u64, kind: &EntityKind, pos: [f64; 3], at: [f64; 2]) -> Option<f64> {
    herd_waters(w, id, kind, pos)
        .into_iter()
        .map(|(p, _)| ((p[0] - at[0]).powi(2) + (p[2] - at[1]).powi(2)).sqrt())
        .min_by(f64::total_cmp)
}

/// Its range (`groups::WATER_RANGE`), in place of a wander: a wild drinker further than that from
/// the nearest water it or its herd knows walks back toward it, a leg at a time. None: it is
/// within it, knows no water, or doesn't drink (it wanders).
pub fn home(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    let a = w.component(id, &ANIMAL)?;
    let s = a.species()?;
    if animal::tamed(w, id) || !s.hungry_in_the_wild() || lasts(w, id, &a).is_none() {
        return None;
    }
    let pos = st.pos.to_array();
    let (p, _) = herd_waters(w, id, s.kind, pos)
        .into_iter()
        .min_by(|a, b| crate::dist2(a.0, pos).total_cmp(&crate::dist2(b.0, pos)))?;
    let d = ((p[0] - pos[0]).powi(2) + (p[2] - pos[2]).powi(2)).sqrt();
    let range = animals_core::groups::WATER_RANGE as f64;
    if d <= range {
        return None;
    }
    if w.navigating(id) {
        return Some(true);
    }
    // A leg back toward it, ending well inside its range.
    let step = (d - range / 2.0).min(LEG) / d;
    let (tx, tz) = (pos[0] + (p[0] - pos[0]) * step, pos[2] + (p[2] - pos[2]) * step);
    let y = crate::comfort::ground(w, tx.floor() as i32, st.pos.y.floor() as i32, tz.floor() as i32).map_or(st.pos.y, f64::from);
    weft_sdk::info!("animal {id} turns back toward water it knows at {:.0} {:.0} ({d:.0} blocks)", p[0], p[2]);
    w.navigate(id, [tx, y, tz], speed);
    Some(true)
}

/// Blocks a leg back toward its range goes at most (`home`).
const LEG: f64 = 12.0;

/// Testing (`/animalprof recall`): `id` remembers drinking at `p`.
pub fn remember(w: &World, id: u64, p: [f64; 3]) {
    if let Some(c) = w.component_mut(id, &WATERS) {
        *c = [[p[0] as f32, p[1] as f32, p[2] as f32, 1.0], [0.0; 4]];
    }
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
    // A plant eater goes to water at dawn and dusk (`routines::DRINKS_AT`): herds at the river.
    let thirsty = if s.forages_wild() && animals_core::routines::drinking_hour(crate::routines::hour(w)) {
        animals_core::routines::DRINKS_AT.max(THIRSTY)
    } else {
        THIRSTY
    };
    // A herd member drinks with its leader (`groups::JOIN_DRINK`).
    let thirsty = if crate::group::leader_doing(w, id) == Some(crate::choose::Need::Drink as u8) {
        thirsty.max(animals_core::groups::JOIN_DRINK)
    } else {
        thirsty
    };
    let calm = water >= SLAKED || (d[3] == 0.0 && water >= thirsty);
    if calm || !((s.hungry_in_the_wild() || animal::tamed(w, id)) && lasts(w, id, &a).is_some()) {
        if d[3] != 0.0 {
            w.component_mut(id, &DRINK).map(|d| *d = [0.0; 4]);
        }
        return false;
    }
    // Walking a way out of where it is stuck (`stuck`).
    if crate::stuck::going(w, id, st) {
        return true;
    }
    let now = w.game_time() as f64;
    let spot = (!d[0].is_nan() && d[3] != 0.0).then(|| [d[0], d[1], d[2]]);
    let there = st.in_liquid != 0
        || spot.is_some_and(|p| {
            let (dx, dz) = (p[0] - st.pos.x, p[2] - st.pos.z);
            dx * dx + dz * dz <= 2.25 && (p[1] - st.pos.y).abs() <= 1.5
        })
        // On its way to water and beside some, any of it: it drinks there (graze5: five deer round
        // a pool's last block, one 3 blocks off its shore spot and never on it, died parched).
        || (spot.is_some() && beside_water(w, st));
    if there {
        // Head down at the water, lapping, while it drinks.
        crate::anim::hold(w, id, crate::anim::Act::Drink);
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
    if spot.is_some() && now < d[3] && !w.navigating(id) {
        // Its walk ended short of the water: it can't get there from here. It gives the place up
        // for a while and lets foraging and wandering move it on (ai1124_savanna8: a deer in a
        // cave two blocks under the pool stood 'drinking' below it for most of a game day, the
        // same shore spot picked again each time its walk ran out, and died parched).
        weft_sdk::debug!("animal {id} can't reach the water at {spot:?}: looks elsewhere");
        if let Some(c) = w.component_mut(id, &DRINK) {
            *c = [f64::NAN, 0.0, 0.0, now + NOTHING];
        }
        // Ended short where the last did too: it is stuck, and looks for a way out (`stuck`).
        return crate::stuck::short(w, id, st);
    }
    let driven = needs::thirst_score(water, true) >= animals_core::memory::DRIVEN;
    if now < d[3] {
        // Walking to it; or no water near, foraging and wandering on: driven, it searches for some
        // (`stuck::search`).
        return spot.is_some() || (driven && crate::stuck::search(w, id, st, true));
    }
    // Its walk's time is up short of the water and it got nowhere (`stuck::ended_short`: herd
    // mates jostling keep the engine walking one that can't get there): as one ended short.
    if spot.is_some() && crate::stuck::ended_short(w, id, st) {
        weft_sdk::debug!("animal {id} got nowhere toward the water at {spot:?}: looks elsewhere");
        if let Some(c) = w.component_mut(id, &DRINK) {
            *c = [f64::NAN, 0.0, 0.0, now + NOTHING];
        }
        return crate::stuck::short(w, id, st);
    }
    match find_water(w, id, s.kind, st.pos.to_array(), driven) {
        Some(p) => {
            weft_sdk::debug!("{} {id} is thirsty (water {:.0}%): to water at {p:?}", s.name, water * 100.0);
            w.navigate(id, p, 1.0);
            crate::stuck::sets_off(w, id, st, p);
            d = [p[0], p[1], p[2], now + WALK];
        }
        None => d = [f64::NAN, 0.0, 0.0, now + NOTHING],
    }
    w.component_mut(id, &DRINK).map(|c| *c = d);
    // Driven by thirst and knowing no water it can find: it searches for some, downhill, or shut in
    // looks for a way out (`stuck::search`): deer in the cave under a pond, never having seen it,
    // stood doing nothing as their water fell from 60 % to 17 % (ai1124_cave2), and a deer on a
    // dune 24 blocks from the pond stared at the player till it died of thirst (graze15).
    if d[0].is_nan() && driven {
        return crate::stuck::search(w, id, st, true);
    }
    !d[0].is_nan()
}

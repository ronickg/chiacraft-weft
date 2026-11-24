//! A stuck animal finds its way out (`animals_core::needs`, "stuck, and the way out"): walks to
//! food or water that end where they began (`forage::eat`, `drink::drink`) are counted here; from
//! `STUCK_AFTER` it walks legs out along its own level, a different way each time, and from
//! `LEAP_AFTER` it leaps. Its need's goal carries the walk while it lasts (`going`).

use animals_core::needs::{
    FLEE_DROP, LEAP_AFTER, LEAP_MOST, LEAP_OVER, LEAP_REACH, LEAP_TRIES, STUCK_AFTER, STUCK_NEAR, TRAP_CELLS, WALK_MOVED, WAY_CLOSER,
    WAY_OUT_LEG, WAY_OUT_SECS, WAY_OUT_WAYS, WAY_UP_CELLS, WAY_UP_REACH,
};
use weft_sdk::*;

/// Where the first of its short walks ended (x, z), how many there have been, until when (game
/// ticks) it walks a leg out or waits after a leap (0: none), and the height it was at.
static STUCK: Component<[f64; 5]> = Component::named("animal_stuck");

/// Where a leap of its lands (x, y, z), whether it is on its way up (1) or across (2) (0: none),
/// and when it leapt (game ticks).
static LEAP_TO: Component<[f64; 5]> = Component::named("animal_leap_to");

/// Where its walk to food or water set off (x, z), and where it walked to (x, y, z; the last one,
/// kept as the goal of its way out: `way_out`), with 1 once it has walked to one.
static SET_OFF: Component<[f64; 6]> = Component::named("animal_set_off");

/// A search for water or food it knows none of (`search`): until when (game ticks) it walks the
/// leg, the way of 8 it walked, and when it set off.
static SEARCH: Component<[f64; 3]> = Component::named("animal_search");

/// When its drink and its eat goal last ran (game ticks), and whether the one running now was
/// kept from running in between (1): another goal took the turn, so a walk it had set off on was
/// cut short by that, not by the ground (`ran`).
static RAN: Component<[f64; 3]> = Component::named("animal_ran");

pub fn init(r: &mut Registry) {
    r.entity_component(&STUCK, [0.0; 5], 0);
    r.entity_component(&RAN, [0.0; 3], 0);
    r.entity_component(&SEARCH, [0.0; 3], 0);
    r.entity_component(&SET_OFF, [0.0; 6], 0);
    r.entity_component(&LEAP_TO, [0.0; 5], 0);
    for s in crate::species::SPECIES.iter() {
        r.edit(s.kind).attach(&STUCK);
        r.edit(s.kind).attach(&RAN);
        r.edit(s.kind).attach(&SEARCH);
        r.edit(s.kind).attach(&SET_OFF);
        r.edit(s.kind).attach(&LEAP_TO);
    }
}

/// `id` sets off on a walk to food or water at `to` (`ended_short` judges it).
pub fn sets_off(w: &World, id: u64, st: &EntityState, to: [f64; 3]) {
    if let Some(c) = w.component_mut(id, &SET_OFF) {
        *c = [st.pos.x, st.pos.z, to[0], to[1], to[2], 1.0];
    }
}

/// The food or water it last walked to, which its way out heads for.
fn goal(w: &World, id: u64) -> Option<[f64; 3]> {
    w.component(id, &SET_OFF)
        .filter(|c| c[5] != 0.0)
        .map(|c| [c[2], c[3], c[4]])
}

/// Whether its walk to food or water has failed: the engine stopped walking it, or it is less
/// than `WALK_MOVED` from where it set off (`needs::WALK_MOVED`).
pub fn ended_short(w: &World, id: u64, st: &EntityState) -> bool {
    !w.navigating(id)
        || w.component(id, &SET_OFF)
            .is_some_and(|o| (o[0] - st.pos.x).powi(2) + (o[1] - st.pos.z).powi(2) < WALK_MOVED * WALK_MOVED)
}

/// Its drink (`water`) or eat goal runs now: noted, with whether another goal took the turn in
/// between (more than `needs::RAN_GAP` ticks since it last ran), for `short`.
pub fn ran(w: &World, id: u64, water: bool) {
    let now = w.game_time() as f64;
    if let Some(r) = w.component_mut(id, &RAN) {
        let i = if water { 0 } else { 1 };
        r[2] = if now - r[i] > animals_core::needs::RAN_GAP { 1.0 } else { 0.0 };
        r[i] = now;
    }
}

/// A walk of `id`'s to food or water ended short of it: counted, and from `STUCK_AFTER` within
/// `STUCK_NEAR` of the first a leg out (true: it walks one, its need's goal keeps the turn).
pub fn short(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut k) = w.component(id, &STUCK) else { return false };
    // Cut short by another goal (lying down with its leader, a rest, a fright): not the ground, so
    // not counted (ai1124_savanna17: walks to water 2 blocks off ended each time it lay down with
    // its leader, and it was counted stuck and leapt 70 times).
    if w.component(id, &RAN).is_some_and(|r| r[2] != 0.0) {
        weft_sdk::debug!("animal {id}'s walk was cut short by another goal: not counted stuck");
        return false;
    }
    let (x, z) = (st.pos.x, st.pos.z);
    if k[2] == 0.0 || (k[0] - x).powi(2) + (k[1] - z).powi(2) > STUCK_NEAR * STUCK_NEAR {
        k = [x, z, 0.0, 0.0, st.pos.y];
    }
    k[2] += 1.0;
    let n = k[2] as u8;
    let now = w.game_time();
    let mut walks = false;
    // Only shut in: the ground it can walk to is a pit's or a pocket's, under `TRAP_CELLS` places.
    // On open ground a walk ends short for other reasons (food on a ledge, a herd mate lying down
    // in its way, its own rest), and a way out with leaps took deer up a hill away from the water
    // and hopped them in place a hundred times (ai1124_savanna16c, graze14, savanna17): there it
    // gives the place up, as before, and wanders on.
    if n >= STUCK_AFTER && enclosed(w, st) {
        // A way of 8, turned on each try from one of its own.
        let way = ((id % WAY_OUT_WAYS as u64) as u8 + n - STUCK_AFTER) % WAY_OUT_WAYS;
        let a = way as f64 * std::f64::consts::TAU / WAY_OUT_WAYS as f64;
        let (dx, dz) = (a.cos(), a.sin());
        // Up the way to the highest ground it can walk to; with none (a pit: nothing it can walk
        // to is higher, and a leg along its level gets it nowhere the search hasn't seen) it leaps
        // at once (ai1124_pit5d: four legs in the pit before each leap, 25 s to 60 s).
        let up = way_out(w, st, goal(w, id));
        let leap_on = match up {
            Some(Way::Leap(off)) => Some(off),
            _ => None,
        };
        let walk_to = match up {
            Some(Way::Walk(to)) => Some(to),
            _ => None,
        };
        // Up the way's leap, else at ground it can land on, up to `LEAP_MOST` above with open air
        // between. Never a leap at nothing: in a shaft a deer leapt at its walls and fell back
        // sixty times in 25 game seconds, and the falls killed it (ai1124_savanna15e).
        let aimed = if leap_on.is_some() || n >= LEAP_AFTER || up.is_none() { leap_on.or_else(|| landing(w, st)) } else { None };
        // At most `LEAP_TRIES` leaps from one stuck place that leave it there (ai1124_savanna17: a
        // deer leapt 70 times, 0.33 s apart, at the pond's rim it could step onto).
        let aimed = aimed.filter(|_| n < LEAP_AFTER + LEAP_TRIES);
        if aimed.is_none() && (n >= LEAP_AFTER || up.is_none()) {
            // Nothing to leap at: it stays, its need's goal letting the band move on.
            if let Some(c) = w.component_mut(id, &STUCK) {
                *c = k;
            }
            return false;
        }
        if let Some((lx, ly, lz)) = aimed {
            // Straight up; across once over the ledge (`going`, `needs::LEAP_OVER`).
            let v = animals_core::needs::leap(0.0, ly, 0.0);
            if let Some(c) = w.component_mut(id, &LEAP_TO) {
                *c = [st.pos.x + lx, st.pos.y + ly, st.pos.z + lz, 1.0, now as f64];
            }
            weft_sdk::info!("animal {id} is stuck at {x:.0} {:.0} {z:.0}: leaps out ({lx:.1} {ly:.0} {lz:.1} off)", st.pos.y);
            if w.navigating(id) {
                w.navigate(id, st.pos, 0.0);
            }
            w.push(id, v.map(|c| c as f32));
            // It waits the leap out (`going`), and leaps again if it fell back in.
            k[2] = k[2].max(LEAP_AFTER as f64);
            k[3] = (now + (WAY_OUT_SECS * 20.0) as u64) as f64;
            if let Some(c) = w.component_mut(id, &STUCK) {
                *c = k;
            }
            return true;
        }
        let to = walk_to.unwrap_or([x + dx * WAY_OUT_LEG, st.pos.y, z + dz * WAY_OUT_LEG]);
        weft_sdk::info!(
            "animal {id} is stuck at {x:.0} {:.0} {z:.0} ({n} walks ended there): looks for a way out toward {:.0} {:.0} {:.0}{}",
            st.pos.y,
            to[0],
            to[1],
            to[2],
            if walk_to.is_some() { " (way)" } else { "" }
        );
        w.navigate(id, to, 1.0);
        k[3] = (now + (WAY_OUT_SECS * 20.0) as u64) as f64;
        walks = true;
    }
    if let Some(c) = w.component_mut(id, &STUCK) {
        *c = k;
    }
    walks
}

/// Whether `id` walks a leg out now, or waits after a leap: its need's goal keeps the turn while
/// it does. A leg that ended where it began is the next short walk at once, so a pit, where every
/// leg ends at once, comes to the leap in seconds, not a give-up's wait apart (ai1124_pit2: three
/// in 50 s, no leap). Landed higher, or walked off, it is out.
pub fn going(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(k) = w.component(id, &STUCK) else { return false };
    if k[3] == 0.0 {
        return false;
    }
    let near = (k[0] - st.pos.x).powi(2) + (k[1] - st.pos.z).powi(2) <= STUCK_NEAR * STUCK_NEAR;
    // Higher only standing on ground: in a leap's arc, or on a herd mate's back, it is not out yet
    // (ai1124_pit5d: each leap and each climb on the other deer counted it out mid-air, and its
    // count began again from one in the pit, 90 s to its first leap).
    let airborne = st.on_ground == 0 && st.in_liquid == 0;
    let floor = [st.pos.x.floor() as i32, (st.pos.y - 0.05).floor() as i32, st.pos.z.floor() as i32];
    let high = !airborne && w.block_flags(floor) & abi::block_flags::SOLID != 0 && st.pos.y > k[4] + 1.0;
    let out = !near || high;
    if let Some(l) = w.component(id, &LEAP_TO).filter(|l| l[3] != 0.0) {
        // Back on the ground (not still at its take-off, before the push has lifted it).
        if !airborne && w.game_time() as f64 > l[4] + 4.0 {
            if let Some(c) = w.component_mut(id, &LEAP_TO) {
                c[3] = 0.0;
            }
        } else if l[3] == 1.0 && st.pos.y >= l[1] + 0.1 {
            // Over the ledge: across onto it.
            let (dx, dz) = (l[0] - st.pos.x, l[2] - st.pos.z);
            w.push(id, [(dx * LEAP_OVER) as f32, 0.0, (dz * LEAP_OVER) as f32]);
            if let Some(c) = w.component_mut(id, &LEAP_TO) {
                c[3] = 2.0;
            }
        }
    }
    if airborne && !out {
        return true;
    }
    let now = w.game_time() as f64;
    if !out && now < k[3] && (w.navigating(id) || k[2] as u8 >= LEAP_AFTER) {
        return true;
    }
    if let Some(c) = w.component_mut(id, &STUCK) {
        c[3] = 0.0;
        if out {
            c[2] = 0.0;
        }
    }
    !out && short(w, id, st)
}

/// The nearest ground it can leap onto: higher than its feet by at most `LEAP_MOST`, within
/// `LEAP_REACH` blocks; how far off (x, up, z).
fn landing(w: &World, st: &EntityState) -> Option<(f64, f64, f64)> {
    let feet = (st.pos.y + 0.05).floor() as i32;
    let (fx, fz) = (st.pos.x.floor() as i32, st.pos.z.floor() as i32);
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    // Room over it to rise, and open air over the landing up to the height it rises to.
    let top = feet + LEAP_MOST + 2;
    if (feet + 2..=top).any(|y| solid([fx, y, fz])) {
        return None;
    }
    let mut best: Option<(f64, f64, f64, f64)> = None;
    for dx in -LEAP_REACH..=LEAP_REACH {
        for dz in -LEAP_REACH..=LEAP_REACH {
            let (bx, bz) = (fx + dx, fz + dz);
            // `ground` looks from 3 above the height it is given down: its highest is the feet's limit.
            let Some(g) = crate::comfort::ground(w, bx, feet + LEAP_MOST - 3, bz) else { continue };
            // Higher than a step (one up it walks), at most `LEAP_MOST`, open over it.
            if g < feet + 2 || g - feet > LEAP_MOST || (g..=top).any(|y| solid([bx, y, bz])) {
                continue;
            }
            let (ox, oz) = (bx as f64 + 0.5 - st.pos.x, bz as f64 + 0.5 - st.pos.z);
            let d = ox * ox + oz * oz;
            if best.is_none_or(|b| d < b.3) {
                best = Some((ox, g as f64 - st.pos.y, oz, d));
            }
        }
    }
    best.map(|(x, y, z, _)| (x, y, z))
}

/// Driven by thirst or hunger and knowing none it can find: shut in, it looks for a way out
/// (`short`); else it walks a leg of `WAY_OUT_LEG` to search, for water the way whose ground is
/// lowest (water lies low; not back the way it came), for food a way turned on each leg. True while
/// it walks one (its need's goal keeps the turn).
pub fn search(w: &World, id: u64, st: &EntityState, water: bool) -> bool {
    if enclosed(w, st) {
        return short(w, id, st);
    }
    let Some(s) = w.component(id, &SEARCH) else { return false };
    let now = w.game_time() as f64;
    if now < s[0] && w.navigating(id) {
        return true;
    }
    // A leg that ended at once: the next a second on, not every tick.
    if now < s[2] + 20.0 {
        return false;
    }
    let last = s[1] as usize % 8;
    let way = |i: usize| {
        let a = i as f64 * std::f64::consts::TAU / 8.0;
        (a.cos(), a.sin())
    };
    let at = |i: usize| {
        let (dx, dz) = way(i);
        let (x, z) = ((st.pos.x + dx * WAY_OUT_LEG).floor() as i32, (st.pos.z + dz * WAY_OUT_LEG).floor() as i32);
        let y = st.pos.y.floor() as i32;
        // The ground there within what its paths go: one up, `FLEE_DROP` + 2 down (from 24
        // under, a search for water picked the floor of a shaft 9 deep as the lowest way,
        // ai1124_shaft1). A way with no such ground isn't taken.
        (y - (FLEE_DROP + 2)..=y + 1)
            .rev()
            .find(|&g| crate::comfort::ground(w, x, g, z) == Some(g))
    };
    let pick = if water {
        (0..8)
            .filter(|&i| i != (last + 4) % 8)
            .filter_map(|i| Some((i, at(i)?)))
            .min_by_key(|&(i, g)| (g, (i + 8 - last) % 8))
            .map(|(i, _)| i)
    } else {
        Some((last + 3) % 8)
    };
    let Some(i) = pick else { return false };
    let (dx, dz) = way(i);
    let to = [st.pos.x + dx * WAY_OUT_LEG, at(i).map_or(st.pos.y, f64::from), st.pos.z + dz * WAY_OUT_LEG];
    weft_sdk::info!("animal {id} searches for {} toward {:.0} {:.0} {:.0}", if water { "water" } else { "food" }, to[0], to[1], to[2]);
    w.navigate(id, to, 1.0);
    if let Some(c) = w.component_mut(id, &SEARCH) {
        *c = [now + (WAY_OUT_SECS * 20.0) as f64, i as f64, now];
    }
    true
}

/// Whether the ground `st` can walk to from its feet (up one, down at most `FLEE_DROP` + 2) is
/// fewer than `needs::TRAP_CELLS` places: a pit, a shaft's foot, a pocket of a cave.
fn enclosed(w: &World, st: &EntityState) -> bool {
    use std::collections::{HashSet, VecDeque};
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    let stand = |p: [i32; 3]| solid([p[0], p[1] - 1, p[2]]) && !solid(p) && !solid([p[0], p[1] + 1, p[2]]);
    let start = [st.pos.x.floor() as i32, (st.pos.y + 0.05).floor() as i32, st.pos.z.floor() as i32];
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if seen.len() >= TRAP_CELLS {
            return false;
        }
        let hop = !solid([p[0], p[1] + 2, p[2]]);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (p[0] + dx, p[2] + dz);
            for dy in (-(FLEE_DROP + 2)..=1).rev() {
                let n = [nx, p[1] + dy, nz];
                if dy == 1 && !hop {
                    continue;
                }
                if dy < 0 && (dy + 1..=0).any(|d| solid([nx, p[1] + d, nz]) || solid([nx, p[1] + d + 1, nz])) {
                    break;
                }
                if stand(n) {
                    if seen.insert(n) {
                        queue.push_back(n);
                    }
                    break;
                }
            }
        }
    }
    true
}

/// The first stretch of the way up: a walk to a place (its centre, at its feet), or a leap from
/// where it stands (how far off, x, up, z).
#[derive(Clone, Copy)]
enum Way {
    Walk([f64; 3]),
    Leap((f64, f64, f64)),
}

/// The way out of where it is (`needs::WAY_UP_*`): a search of the ground it can get to from its
/// feet, walking (up one, down at most `FLEE_DROP` + 2) and leaping (up to `LEAP_MOST`, to the
/// next column, with room overhead), for the place nearest the food or water it couldn't reach
/// (`goal`), or with none known, the highest. Its first stretch: a walk of up to `WAY_OUT_LEG`
/// steps, ending where the first leap begins; or that leap, when it begins where it stands. None
/// when nothing it can get to is nearer (by `WAY_CLOSER`) or higher than its feet. The highest
/// alone took a deer on a hill 10 over the pond further up and away from it, leaping from ledge to
/// ledge till it died of thirst at 142 (ai1124_savanna16c).
fn way_out(w: &World, st: &EntityState, goal: Option<[f64; 3]>) -> Option<Way> {
    way_out_counted(w, st, goal).0
}

/// `way_out`, and how many places it searched, the best and how many leaps it found.
fn way_out_counted(w: &World, st: &EntityState, goal: Option<[f64; 3]>) -> (Option<Way>, usize, [i32; 3], usize) {
    use std::collections::{HashMap, VecDeque};
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    let stand = |p: [i32; 3]| solid([p[0], p[1] - 1, p[2]]) && !solid(p) && !solid([p[0], p[1] + 1, p[2]]);
    let start = [st.pos.x.floor() as i32, (st.pos.y + 0.05).floor() as i32, st.pos.z.floor() as i32];
    // Each place, the one it was reached from, and whether by a leap.
    let mut from: HashMap<[i32; 3], ([i32; 3], bool)> = HashMap::new();
    let mut queue = VecDeque::from([start]);
    from.insert(start, (start, false));
    // Nearer the goal (its centre to the place's), else higher.
    let far =
        |p: [i32; 3], g: [f64; 3]| (p[0] as f64 + 0.5 - g[0]).powi(2) + (p[1] as f64 - g[1]).powi(2) + (p[2] as f64 + 0.5 - g[2]).powi(2);
    let better = |p: [i32; 3], than: [i32; 3]| match goal {
        Some(g) => far(p, g) < far(than, g),
        None => p[1] > than[1],
    };
    let mut best = start;
    let mut leaps = 0;
    while let Some(p) = queue.pop_front() {
        if from.len() > WAY_UP_CELLS {
            break;
        }
        if better(p, best) {
            best = p;
        }
        // Room over it to jump one, or to leap (its body clears `LEAP_MOST` plus its height).
        let room = |h: i32| (2..=h).all(|d| !solid([p[0], p[1] + d, p[2]]));
        let (hop, leap) = (room(2), room(LEAP_MOST + 2));
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (p[0] + dx, p[2] + dz);
            if (nx - start[0]).abs() > WAY_UP_REACH || (nz - start[2]).abs() > WAY_UP_REACH {
                continue;
            }
            let mut reach = |n: [i32; 3], by_leap: bool| {
                if let std::collections::hash_map::Entry::Vacant(e) = from.entry(n) {
                    e.insert((p, by_leap));
                    queue.push_back(n);
                }
            };
            // A leap onto a ledge higher than a step, with room over the landing.
            if let Some(dy) = (2..=LEAP_MOST).rev().find(|&dy| {
                let n = [nx, p[1] + dy, nz];
                leap && stand(n) && !solid([nx, n[1] + 2, nz])
            }) {
                reach([nx, p[1] + dy, nz], true);
                leaps += 1;
            }
            // Up one with room overhead to jump, along, or down as far as it may drop.
            for dy in (-(FLEE_DROP + 2)..=1).rev() {
                let n = [nx, p[1] + dy, nz];
                if dy == 1 && !hop {
                    continue;
                }
                // Down: the column it falls through is open.
                if dy < 0 && (dy + 1..=0).any(|d| solid([nx, p[1] + d, nz]) || solid([nx, p[1] + d + 1, nz])) {
                    break;
                }
                if stand(n) {
                    reach(n, false);
                    break;
                }
            }
        }
    }
    let searched = from.len();
    let gets_on = match goal {
        Some(g) => far(best, g).sqrt() < far(start, g).sqrt() - WAY_CLOSER,
        None => best[1] > start[1],
    };
    if !gets_on {
        return (None, searched, best, leaps);
    }
    // Back from the highest to its feet, then each place and whether it is left by a leap.
    let mut way = vec![best];
    while let Some(&(q, _)) = from
        .get(way.last().unwrap())
        .filter(|(q, _)| q != way.last().unwrap())
    {
        way.push(q);
    }
    way.reverse();
    let out: Vec<([i32; 3], bool)> = (0..way.len())
        .map(|i| (way[i], i + 1 < way.len() && from[&way[i + 1]].1))
        .collect();
    let centre = |c: [i32; 3]| [c[0] as f64 + 0.5, c[1] as f64, c[2] as f64 + 0.5];
    if out[0].1 {
        let to = centre(out[1].0);
        return (Some(Way::Leap((to[0] - st.pos.x, to[1] - st.pos.y, to[2] - st.pos.z))), searched, best, leaps);
    }
    let end = out
        .iter()
        .take(WAY_OUT_LEG as usize + 1)
        .position(|c| c.1)
        .unwrap_or((WAY_OUT_LEG as usize).min(out.len() - 1));
    (Some(Way::Walk(centre(out[end].0))), searched, best, leaps)
}

/// What the way-out search finds from where `id` stands, as a line (`/animalprof cave`).
pub fn way_out_report(w: &World, id: u64, st: &EntityState) -> String {
    let goal = goal(w, id);
    let (way, searched, best, leaps) = way_out_counted(w, st, goal);
    let way = match way {
        None => format!("no way out (toward {goal:?})"),
        Some(Way::Walk(p)) => format!("walk to {:.1} {:.0} {:.1}", p[0], p[1], p[2]),
        Some(Way::Leap(o)) => format!("leap {:.1} {:.0} {:.1} off", o.0, o.1, o.2),
    };
    format!("{way}; searched {searched} places, the best {} {} {}, {leaps} leaps", best[0], best[1], best[2])
}

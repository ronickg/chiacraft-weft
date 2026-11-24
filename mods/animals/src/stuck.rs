//! A stuck animal finds its way out (`animals_core::needs`, "stuck, and the way out"): walks to
//! food or water that end where they began (`forage::eat`, `drink::drink`) are counted here; from
//! `STUCK_AFTER` it walks legs out along its own level, a different way each time, and from
//! `LEAP_AFTER` it leaps. Its need's goal carries the walk while it lasts (`going`).

use animals_core::needs::{
    LEAP_AFTER, LEAP_MOST, LEAP_REACH, STUCK_AFTER, STUCK_NEAR, WALK_MOVED, WAY_OUT_LEG, WAY_OUT_SECS, WAY_OUT_WAYS,
};
use weft_sdk::*;

/// Where the first of its short walks ended (x, z), how many there have been, until when (game
/// ticks) it walks a leg out or waits after a leap (0: none), and the height it was at.
static STUCK: Component<[f64; 5]> = Component::named("animal_stuck");

/// Where its walk to food or water set off (x, z).
static SET_OFF: Component<[f64; 2]> = Component::named("animal_set_off");

pub fn init(r: &mut Registry) {
    r.entity_component(&STUCK, [0.0; 5], 0);
    r.entity_component(&SET_OFF, [0.0; 2], 0);
    for s in crate::species::SPECIES.iter() {
        r.edit(s.kind).attach(&STUCK);
        r.edit(s.kind).attach(&SET_OFF);
    }
}

/// `id` sets off on a walk to food or water (`ended_short` judges it).
pub fn sets_off(w: &World, id: u64, st: &EntityState) {
    if let Some(c) = w.component_mut(id, &SET_OFF) {
        *c = [st.pos.x, st.pos.z];
    }
}

/// Whether its walk to food or water has failed: the engine stopped walking it, or it is less
/// than `WALK_MOVED` from where it set off (`needs::WALK_MOVED`).
pub fn ended_short(w: &World, id: u64, st: &EntityState) -> bool {
    !w.navigating(id)
        || w.component(id, &SET_OFF)
            .is_some_and(|o| (o[0] - st.pos.x).powi(2) + (o[1] - st.pos.z).powi(2) < WALK_MOVED * WALK_MOVED)
}

/// A walk of `id`'s to food or water ended short of it: counted, and from `STUCK_AFTER` within
/// `STUCK_NEAR` of the first a leg out (true: it walks one, its need's goal keeps the turn).
pub fn short(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut k) = w.component(id, &STUCK) else { return false };
    let (x, z) = (st.pos.x, st.pos.z);
    if k[2] == 0.0 || (k[0] - x).powi(2) + (k[1] - z).powi(2) > STUCK_NEAR * STUCK_NEAR {
        k = [x, z, 0.0, 0.0, st.pos.y];
    }
    k[2] += 1.0;
    let n = k[2] as u8;
    let now = w.game_time();
    let mut walks = false;
    if n >= STUCK_AFTER {
        // A way of 8, turned on each try from one of its own.
        let way = ((id % WAY_OUT_WAYS as u64) as u8 + n - STUCK_AFTER) % WAY_OUT_WAYS;
        let a = way as f64 * std::f64::consts::TAU / WAY_OUT_WAYS as f64;
        let (dx, dz) = (a.cos(), a.sin());
        if n >= LEAP_AFTER {
            // At ground it can land on, up to `LEAP_MOST` above, else straight up along its way.
            let (lx, ly, lz) = landing(w, st).unwrap_or((dx, LEAP_MOST as f64, dz));
            let v = animals_core::needs::leap(lx, ly, lz);
            weft_sdk::info!("animal {id} is stuck at {x:.0} {:.0} {z:.0}: leaps out ({lx:.1} {ly:.0} {lz:.1} off)", st.pos.y);
            if w.navigating(id) {
                w.navigate(id, st.pos, 0.0);
            }
            w.push(id, v.map(|c| c as f32));
            k[3] = (now + (WAY_OUT_SECS * 20.0) as u64) as f64;
            if let Some(c) = w.component_mut(id, &STUCK) {
                *c = k;
            }
            return true;
        }
        let to = [x + dx * WAY_OUT_LEG, st.pos.y, z + dz * WAY_OUT_LEG];
        weft_sdk::info!(
            "animal {id} is stuck at {x:.0} {:.0} {z:.0} ({n} walks ended there): looks for a way out toward {:.0} {:.0}",
            st.pos.y,
            to[0],
            to[2]
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
    let out = !near || st.pos.y > k[4] + 1.0;
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
    let feet = st.pos.y.floor() as i32;
    let (fx, fz) = (st.pos.x.floor() as i32, st.pos.z.floor() as i32);
    let mut best: Option<(f64, f64, f64, f64)> = None;
    for dx in -LEAP_REACH..=LEAP_REACH {
        for dz in -LEAP_REACH..=LEAP_REACH {
            let (bx, bz) = (fx + dx, fz + dz);
            // `ground` looks from 3 above the height it is given down: its highest is the feet's limit.
            let Some(g) = crate::comfort::ground(w, bx, feet + LEAP_MOST - 3, bz) else { continue };
            if g <= feet || g - feet > LEAP_MOST {
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

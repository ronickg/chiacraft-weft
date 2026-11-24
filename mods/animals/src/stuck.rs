//! A stuck animal finds its way out (`animals_core::needs`, "stuck, and the way out"): walks to
//! food or water that end where they began (`forage::eat`, `drink::drink`) are counted here; from
//! `STUCK_AFTER` it walks legs out along its own level, a different way each time, and from
//! `LEAP_AFTER` it leaps. Its need's goal carries the walk while it lasts (`going`).

use animals_core::needs::{LEAP_AFTER, LEAP_BLOCKS, LEAP_SIDE, STUCK_AFTER, STUCK_NEAR, WAY_OUT_LEG, WAY_OUT_SECS, WAY_OUT_WAYS};
use weft_sdk::*;

/// Where the first of its short walks ended (x, z), how many there have been, and until when (game
/// ticks) it walks a leg out (0: none).
static STUCK: Component<[f64; 4]> = Component::named("animal_stuck");

pub fn init(r: &mut Registry) {
    r.entity_component(&STUCK, [0.0; 4], 0);
    for s in crate::species::SPECIES.iter() {
        r.edit(s.kind).attach(&STUCK);
    }
}

/// A walk of `id`'s to food or water ended short of it: counted, and from `STUCK_AFTER` within
/// `STUCK_NEAR` of the first a leg out (true: it walks one, its need's goal keeps the turn).
pub fn short(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut k) = w.component(id, &STUCK) else { return false };
    let (x, z) = (st.pos.x, st.pos.z);
    if k[2] == 0.0 || (k[0] - x).powi(2) + (k[1] - z).powi(2) > STUCK_NEAR * STUCK_NEAR {
        k = [x, z, 0.0, 0.0];
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
            let up = animals_core::needs::leap_up(LEAP_BLOCKS);
            weft_sdk::info!("animal {id} is stuck at {x:.0} {:.0} {z:.0}: leaps out", st.pos.y);
            w.push(id, [(dx * LEAP_SIDE) as f32, up as f32, (dz * LEAP_SIDE) as f32]);
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

/// Whether `id` walks a leg out now: its need's goal keeps the turn while it does.
pub fn going(w: &World, id: u64) -> bool {
    w.component(id, &STUCK)
        .is_some_and(|k| k[3] != 0.0 && (w.game_time() as f64) < k[3] && w.navigating(id))
}

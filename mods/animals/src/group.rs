//! Herds and packs that keep together (`animals_core::groups`; `reports/Creature AI.md`, part 3,
//! step 9). Its wander (the needs band's last goal) is the leader's alone: a wild member that isn't
//! its group's leader keeps to its own place round the leader instead, walking back once it is
//! `FOLLOW_FROM` off it, so the herd drifts where its leader goes. The leader is the lowest id of
//! its kind near, from each member's own look round (`senses::Senses::leader`): nothing to hand
//! over when a leader dies or wanders off, the next look finds the next one.
//!
//! A wander that gets nowhere (walking, but not a block in `STUCK_TICKS`: a target it can't reach)
//! is stopped here, so it picks again instead of standing in a walk for good, and logged with where
//! it stood (ai1122_herd2 and _herd3: leaders "walking" without moving for minutes).
//!
//! Only the wander: fear, hunts, water, food, rest and love go first as before (`choose.rs`), so a
//! thirsty member still goes to drink on its own, and comes back to its place after.

use weft_sdk::ai::Goal;
use weft_sdk::*;

use animals_core::groups::{self, CATCH_UP, FOLLOW_EVERY, FOLLOW_FROM, GROUP};

use crate::senses::SENSES;

/// Ticks a wander may walk without getting anywhere before it is stopped, and the blocks a tick
/// under which it counts as not moving (the engine's own stuck check is 0.02).
pub const STUCK_TICKS: u32 = 60;
const STILL: f32 = 0.02;

/// Ticks its wander has walked without moving (not saved).
static STUCK: Component<u32> = Component::named("animal_wander_stuck");

pub fn init(r: &mut Registry) {
    r.entity_component(&STUCK, 0u32, 0);
    for s in &crate::species::SPECIES {
        r.edit(s.kind).attach(&STUCK);
    }
}

/// `wander` (at `speed`) for one alone or a group's leader; a member keeps to its place round the
/// leader.
pub fn or_follow(speed: f32, wander: impl Goal) -> impl Goal {
    move |w: &World, id: u64, st: &EntityState| {
        if stuck(w, id, st) {
            return false;
        }
        match follow(w, id, st, speed) {
            Some(acted) => acted,
            None => wander.run(w, id, st),
        }
    }
}

/// Whether its walk got nowhere for `STUCK_TICKS`: then it is stopped (and logged).
fn stuck(w: &World, id: u64, st: &EntityState) -> bool {
    let walking = w.navigating(id) && st.moved < STILL;
    let Some(n) = w.component_mut(id, &STUCK) else { return false };
    if !walking {
        *n = 0;
        return false;
    }
    *n = n.saturating_add(w.tick_elapsed(id));
    if *n < STUCK_TICKS {
        return false;
    }
    *n = 0;
    let p = st.pos;
    let name = |dy: i32| {
        let b = [p.x.floor() as i32, p.y.floor() as i32 + dy, p.z.floor() as i32];
        name_of(abi::entry_kind::BLOCK, block_of(w.block(b)))
    };
    weft_sdk::info!(
        "animal {id} stuck walking at {:.1} {:.1} {:.1} (on {}, in {}, ground {}, flags {:#x}): stopped",
        p.x,
        p.y,
        p.z,
        name(-1),
        name(0),
        st.on_ground,
        st.flags,
    );
    w.navigate(id, st.pos, 0.0);
    true
}

/// What `id`'s leader is doing of its needs (`choose::doing`), if `id` is a wild member of a
/// group that isn't its leader (slice 2: it drinks and grazes when its leader does).
pub fn leader_doing(w: &World, id: u64) -> Option<u8> {
    let s = w.component(id, &SENSES)?;
    if s.kin < GROUP || s.leader == 0 || s.leader == id || crate::animal::tamed(w, id) {
        return None;
    }
    w.entity(s.leader).filter(|e| e.health > 0.0)?;
    Some(crate::choose::doing(w, s.leader))
}

/// Resting in step (slice 3): `Some(true)` when a member should lie down now (its leader rests),
/// `Some(false)` when it should get up now (lying while its leader is up and about); None: as it
/// likes. `resting`: whether it rests now.
pub fn rest_in_step(w: &World, id: u64, resting: bool) -> Option<bool> {
    let lead = leader_doing(w, id)?;
    let rests = lead == crate::choose::Need::Rest as u8;
    if !resting && rests && w.random_below(groups::JOIN_REST_IN) == 0 {
        weft_sdk::info!("animal {id} lies down with its leader");
        return Some(true);
    }
    // Up and about: a need of its own under way (not idle, which a resting one's next rest may be).
    if resting && !rests && lead != 0 && w.random_below(groups::JOIN_UP_IN) == 0 {
        weft_sdk::info!("animal {id} gets up with its leader");
        return Some(false);
    }
    None
}

/// For a wild member of a group that isn't its leader: whether it walks to its place now. None:
/// it wanders on its own (alone, the leader, tame, or its leader gone until its next look).
fn follow(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    let s = w.component(id, &SENSES)?;
    if s.kin < GROUP || s.leader == 0 || s.leader == id || crate::animal::tamed(w, id) {
        return None;
    }
    let lead = w.entity(s.leader).filter(|e| e.health > 0.0)?;
    if w.navigating(id) {
        return Some(true);
    }
    // Now and then, not every tick (and not in a rhythm the AI's level of detail could skip).
    if w.random_below(FOLLOW_EVERY as u32) != 0 {
        return Some(false);
    }
    let [ox, oz] = groups::slot(id, s.kin);
    let to = [lead.pos.x + ox as f64, lead.pos.y, lead.pos.z + oz as f64];
    let d2 = (to[0] - st.pos.x).powi(2) + (to[2] - st.pos.z).powi(2);
    let from = FOLLOW_FROM as f64;
    if d2 < from * from {
        // At its place: it stays (grazing, looking about: idle life).
        return Some(false);
    }
    let far = d2 > 4.0 * from * from;
    if far {
        weft_sdk::info!("animal {id} keeps up with its leader {} ({:.1} blocks off its place)", s.leader, d2.sqrt());
    }
    w.navigate(id, to, if far { speed * CATCH_UP } else { speed });
    Some(true)
}

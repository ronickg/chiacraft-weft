//! Herds and packs that keep together (`animals_core::groups`; `reports/Creature AI.md`, part 3,
//! step 9). Its wander (the needs band's last goal) is the leader's alone: a wild member that isn't
//! its group's leader keeps to its own place round the leader instead, walking back once it is
//! `FOLLOW_FROM` off it, so the herd drifts where its leader goes. The leader is the lowest id of
//! its kind near, from each member's own look round (`senses::Senses::leader`): nothing to hand
//! over when a leader dies or wanders off, the next look finds the next one.
//!
//! Only the wander: fear, hunts, water, food, rest and love go first as before (`choose.rs`), so a
//! thirsty member still goes to drink on its own, and comes back to its place after.

use weft_sdk::ai::Goal;
use weft_sdk::*;

use animals_core::groups::{self, CATCH_UP, FOLLOW_EVERY, FOLLOW_FROM, GROUP};

use crate::senses::SENSES;

/// `wander` (at `speed`) for one alone or a group's leader; a member keeps to its place round the
/// leader.
pub fn or_follow(speed: f32, wander: impl Goal) -> impl Goal {
    move |w: &World, id: u64, st: &EntityState| match follow(w, id, st, speed) {
        Some(acted) => acted,
        None => wander.run(w, id, st),
    }
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

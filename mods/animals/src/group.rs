//! Herds and packs that keep together (`animals_core::groups`; `reports/Creature AI.md`, part 3,
//! step 9). Its wander (the needs band's last goal) is the leader's alone: a wild member that isn't
//! its group's leader keeps to its own place round the leader instead, walking back once it is
//! `FOLLOW_FROM` off it, so the herd drifts where its leader goes. The leader stays leader
//! (`groups::keep_or_pick`, from each member's look round, `senses::Senses::leader`) until it dies,
//! is away too long or weakens; a leader whose group lies resting keeps by it (`by_resting_pack`).
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

use crate::dist2;
use crate::senses::SENSES;

/// Ticks a wander may walk without getting anywhere before it is stopped, and the blocks a tick
/// under which it counts as not moving (the engine's own stuck check is 0.02).
pub const STUCK_TICKS: u32 = 60;
const STILL: f32 = 0.02;

/// When a stray last howled for its pack (`regroup`; not saved).
static RALLIED: Component<u64> = Component::named("animal_rallied");

/// Ticks its wander has walked without moving (not saved).
static STUCK: Component<u32> = Component::named("animal_wander_stuck");

pub fn init(r: &mut Registry) {
    r.entity_component(&STUCK, 0u32, 0);
    r.entity_component(&RALLIED, 0u64, 0);
    for s in &crate::species::SPECIES {
        r.edit(s.kind).attach(&STUCK);
        r.edit(s.kind).attach(&RALLIED);
    }
}

/// `wander` (at `speed`) for one alone or a group's leader; a member keeps to its place round the
/// leader.
pub fn or_follow(speed: f32, wander: impl Goal) -> impl Goal {
    move |w: &World, id: u64, st: &EntityState| {
        if stuck(w, id, st) {
            return false;
        }
        // A member keeps to its place; a herd's leader in the dry season leads it to wetter ground
        // (`migrate`); one far from the water it knows turns back toward it (`drink::home`); else
        // it wanders.
        if let Some(acted) = follow(w, id, st, speed)
            .or_else(|| by_resting_pack(w, id, st, speed))
            .or_else(|| regroup(w, id, st, speed))
            .or_else(|| crate::migrate::lead(w, id, st, speed))
            .or_else(|| crate::drink::home(w, id, st, speed))
        {
            return acted;
        }
        wander.run(w, id, st)
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
    if s.leader == 0 || s.leader == id || crate::animal::tamed(w, id) {
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

/// A group's leader awake while `groups::PACK_RESTS` of the others lie resting keeps within
/// `groups::LEADER_TETHER` of their middle (it waits for them instead of wandering off): walking
/// back past it, else standing about. None: not leading a resting group.
fn by_resting_pack(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    let s = w.component(id, &SENSES)?;
    let others = s.kin.saturating_sub(1);
    if s.leader != id || s.kin < GROUP || (s.kin_resting as f32) < others as f32 * groups::PACK_RESTS {
        return None;
    }
    let mid = s.kin_mid.map(f64::from);
    let tether = groups::LEADER_TETHER as f64;
    let d2 = (mid[0] - st.pos.x).powi(2) + (mid[2] - st.pos.z).powi(2);
    if d2 <= tether * tether {
        return Some(w.navigating(id));
    }
    if !w.navigating(id) || w.random_below(FOLLOW_EVERY as u32) == 0 {
        weft_sdk::debug!("animal {id} keeps by its resting pack ({:.1} blocks off)", d2.sqrt());
        w.navigate(id, mid, speed);
    }
    Some(true)
}

/// For a wild member of a group that isn't its leader: whether it walks to its place now. None:
/// it wanders on its own (alone, the leader, tame, or its leader gone until its next look).
pub fn follow(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    let s = w.component(id, &SENSES)?;
    if s.leader == 0 || s.leader == id || crate::animal::tamed(w, id) {
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

/// Where a wild pack member stranded from its kin for `groups::STRAY_SECS` heads (a leg of
/// `groups::REGROUP_LEG` toward it): it howls now and then (`groups::RALLY_HOWL_EVERY_SECS`) and
/// the nearest pack of its kind within `groups::HOWL_HEARD` answers, or, with none answering,
/// where it last was with them. None: not stranded, or nowhere to go.
pub fn regroup_leg(w: &World, id: u64, st: &EntityState) -> Option<[f64; 3]> {
    let s = w.component(id, &SENSES)?;
    let now = w.game_time();
    if s.alone_since == 0 || (now.saturating_sub(s.alone_since) as f32) < groups::STRAY_SECS * 20.0 || crate::animal::tamed(w, id) {
        return None;
    }
    let mut to = (s.pack_at != [0.0; 3]).then(|| s.pack_at.map(f64::from));
    let last = w.component(id, &RALLIED).unwrap_or(0);
    if last == 0 || now >= last + (groups::RALLY_HOWL_EVERY_SECS * 20.0) as u64 {
        w.component_mut(id, &RALLIED).map(|r| *r = now);
        crate::voices::howl(w, id, st);
        // The nearest of its kind with kin about it answers (a pack), else any of its kind.
        let answer = w
            .entities_near(st.pos, groups::HOWL_HEARD, None)
            .into_iter()
            .filter(|&o| o != id && !crate::animal::tamed(w, o))
            .filter_map(|o| {
                w.entity(o)
                    .filter(|e| e.kind == st.kind && e.health > 0.0)
                    .map(|e| (o, e.pos))
            })
            .map(|(o, p)| {
                let pack = w.component(o, &SENSES).is_some_and(|k| k.kin >= GROUP);
                (!pack, dist2(p, st.pos), o, p)
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        match answer {
            Some((_, d2, o, p)) => {
                weft_sdk::info!("animal {id} howls for its pack: {o} answers {:.0} blocks off", d2.sqrt());
                if let Some(e) = w.entity(o) {
                    crate::voices::howl(w, o, &e);
                }
                if let Some(k) = w.component_mut(id, &SENSES) {
                    k.pack_at = [p.x as f32, p.y as f32, p.z as f32];
                }
                to = Some([p.x, p.y, p.z]);
            }
            None => weft_sdk::info!("animal {id} howls for its pack: none answers"),
        }
    }
    let t = to?;
    let (dx, dz) = (t[0] - st.pos.x, t[2] - st.pos.z);
    let d = dx.hypot(dz);
    if d < crate::senses::KIN * 0.5 {
        return None;
    }
    let k = (groups::REGROUP_LEG / d).min(1.0);
    Some([st.pos.x + dx * k, t[1], st.pos.z + dz * k])
}

/// `regroup_leg` walked at `speed` while it wanders: Some(true) while it goes.
fn regroup(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    if w.navigating(id) && w.component(id, &SENSES).is_some_and(|s| s.alone_since != 0) {
        return Some(true);
    }
    let to = regroup_leg(w, id, st)?;
    weft_sdk::debug!("animal {id} heads for its pack: a leg to {:.0} {:.0} {:.0}", to[0], to[1], to[2]);
    w.navigate(id, to, speed * CATCH_UP);
    Some(true)
}

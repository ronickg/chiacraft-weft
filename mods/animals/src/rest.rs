//! Settling down, for the full creatures (the cat, the capybara, the raptors, the Utahraptor, the
//! wyvern, the wolf and the jaguar): now and then one up and about rests for a while, sitting and
//! after a while lying down, or lying at once where it has one pose. A pose is a look
//! (`World::set_look`), a kind of its own drawn in place of the creature, so every player sees it;
//! knocked out it lies as it rests (`sleep`), and called to follow or fight it gets up (`wake`).
//! One component and one goal for all of them (Research round 1, G1).

use std::sync::Mutex;

use weft_sdk::*;

use crate::ModelFile;

value! {
    #[derive(Default)]
    /// What a resting creature is doing (not saved: it just settles down again).
    pub struct Resting {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static REST: Component<Resting> = Component::named("creature_rest");

/// How a kind settles down.
#[derive(Clone, Copy)]
pub struct Settle {
    /// One in this many ticks, up and about and not busy, it settles down...
    pub chance: u32,
    /// ...for the first of these ticks and up to the second more.
    pub ticks: (u32, u32),
    /// Rested this long, a sitting one lies down (its second pose); 0: one pose, lying.
    pub lie_after: u32,
}

/// Each kind's way of settling, and its poses' looks (`animals:<kind>`, first sitting or lying,
/// then lying), filled at registration.
static KINDS: Mutex<Vec<(&'static EntityKind, Settle, &'static [String])>> = Mutex::new(Vec::new());

pub fn init(r: &mut Registry) {
    r.entity_component(&REST, Resting::default(), 0);
}

/// Gives `kind` its poses: each (animation file's suffix, look's kind name) drawn on `model`'s
/// geometry with `texture`, `size` blocks wide and tall; the kind attaches its resting state.
pub fn register(
    r: &mut Registry,
    kind: &'static EntityKind,
    settle: Settle,
    model: &str,
    texture: &str,
    size: (f32, f32),
    poses: &[(&str, &str)],
) {
    for (suffix, look) in poses {
        drop(
            r.entity(EntityKind::generated(look.to_string()))
                .size(size.0, size.1)
                .passive()
                .model_file(model)
                .animations(&format!("{model}{suffix}"))
                .texture(texture),
        );
    }
    r.attach(kind, &REST);
    KINDS.lock().unwrap_or_else(|e| e.into_inner()).push((
        kind,
        settle,
        poses
            .iter()
            .map(|(_, look)| format!("animals:{look}"))
            .collect::<Vec<_>>()
            .leak(),
    ));
}

/// The way `id`'s kind settles and its looks.
fn kind_of(w: &World, id: u64) -> Option<(Settle, &'static [String])> {
    let kind = w.entity(id)?.kind;
    KINDS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, ..)| k.id() == kind)
        .map(|&(_, s, looks)| (s, looks))
}

/// A goal, every tick: now and then settle down for a while (sit, then lie), and show the pose;
/// it takes the turn while it rests.
pub fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some((how, looks)) = kind_of(w, id) else { return false };
    let Some(c) = w.component_mut(id, &REST) else { return false };
    let chance = crate::routines::rest_chance(how.chance, crate::routines::activity_of(w, id));
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(chance) == 0 {
        c.rest = how.ticks.0 + w.random_below(how.ticks.1);
        c.rested = 0;
    }
    let resting = c.rest > 0;
    if resting {
        // A resting one far from players ticks now and then: count the ticks it skipped.
        let e = w.tick_elapsed(id);
        c.rest = c.rest.saturating_sub(e);
        c.rested = c.rested.saturating_add(e);
    }
    let pose = match (resting, how.lie_after > 0 && c.rested > how.lie_after) {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    };
    show(w, id, c, pose, looks);
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

fn show(w: &World, id: u64, c: &mut Resting, pose: u32, looks: &[String]) {
    if c.look != pose + 1 {
        c.look = pose + 1;
        let look = pose.checked_sub(1).and_then(|i| looks.get(i as usize));
        w.set_appearance(id, look.map(String::as_str));
    }
}

/// Up from resting, to follow, fight or carry a rider.
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &REST)
        && (c.rest > 0 || c.look > 1)
    {
        c.rest = 0;
        c.look = 1;
        w.set_appearance(id, None);
    }
}

/// Hurt: it stops resting (the goal shows it up next tick).
pub fn stop(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &REST) {
        c.rest = 0;
    }
}

/// Knocked out: lying as it rests (its last pose).
pub fn sleep(w: &World, id: u64) {
    let Some((_, looks)) = kind_of(w, id) else { return };
    let pose = looks.len() as u32;
    if let Some(c) = w.component_mut(id, &REST)
        && c.look != pose + 1
    {
        c.rest = 0;
        show(w, id, c, pose, looks);
    }
}

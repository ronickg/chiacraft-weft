//! Dogs (Canis familiaris), by breed; one so far, the chihuahua: small long-haired dogs on grassland,
//! each with its own coat from its genes (`coats.rs`, dog coat genetics with the chihuahua's
//! frequencies). Tame one by feeding it when it asks (`taming.rs`; bones and cooked meat are its
//! favorites), make it sit or follow by right clicking it, breed a tamed male and female with
//! meat: the puppy gets one allele of each gene from each parent, so it looks like them (hidden
//! ones can surprise), and grows up in twenty minutes (meat speeds it along). Its signatures (an
//! alarm, tracking) are in `signatures.rs`.
//!
//! A breed is its own entity kind with its own model (`animals:chihuahua`); the species' biology,
//! genes and behavior are shared (`species.rs`: species "dog", breed "chihuahua"). The model is a
//! papillon-style long-haired chihuahua made of boxes and flat fur panels with a hand-painted
//! texture (Hytale's style), built in Blockbench: `models/chihuahua.geo.json`,
//! `animations/chihuahua[_pup][_sit|_lie].animation.json` and
//! `textures/entity/chihuahua/chihuahua.png` are built from its source in `source/dog/chihuahua`
//! (`build.py`; see `docs/creatures.md`); each dog's coat is painted over that texture. Age and
//! pose are looks (`World::set_look`): one entity kind per age and pose, drawn in place of the dog
//! with its own coat. Puppies have their own model file (half size, a bigger head) on the same
//! layout, so the same picture fits. A pose is the look's animation file, whose
//! `idle` holds it (sitting, lying down), so every player sees it whenever they come by.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::animal;
use crate::{commands, screen, signatures, species, taming};

pub(crate) static CHIHUAHUA: Entity = Entity::named("chihuahua");

/// The dog's texture, its coat painted over it (`coats.rs`).
const TEXTURE: &str = "chihuahua/chihuahua";
/// Sitting this long (thirty seconds), a dog lies down.
const LIE_TICKS: u32 = 600;

value! {
    #[derive(Default)]
    /// A dog's own state, saved with it.
    pub struct Dog {
        /// Unused: coats come from genes now (`coats.rs`); kept so saved dogs load.
        pub coat: u32,
        pub tamed: u32,
        /// Unused: love, rest and growing up are `breeding.rs`'s now; kept so saved dogs load.
        pub love: u32,
        pub cooldown: u32,
        pub age: u32,
        /// Ticks left running about after being hurt.
        pub panic: u32,
    }
}

value! {
    #[derive(Default)]
    /// What is not saved: the look applied, ticks sitting.
    pub struct Seen {
        pub look: u32,
        pub sat: u32,
    }
}

static DOG: Component<Dog> = Component::named("dog");
static SEEN: Component<Seen> = Component::named("seen");

// ------------------------------------------------------------------ looks

#[derive(Clone, Copy, PartialEq)]
enum Pose {
    Stand,
    Sit,
    Lie,
}

const POSES: [Pose; 3] = [Pose::Stand, Pose::Sit, Pose::Lie];

impl Pose {
    fn suffix(self) -> &'static str {
        match self {
            Pose::Stand => "",
            Pose::Sit => "_sit",
            Pose::Lie => "_lie",
        }
    }
}

/// The kind a dog is drawn as: grown or puppy, and pose.
fn look_name(puppy: bool, pose: Pose) -> String {
    format!("animals:chihuahua_{}{}", if puppy { "pup" } else { "adult" }, pose.suffix())
}

// ------------------------------------------------------------------ registration

pub fn init(r: &mut Registry) {
    r.entity_component(&DOG, Dog::default(), component_flags::SAVE);
    r.entity_component(&SEEN, Seen::default(), 0);
    // The looks, drawn in place of a dog of that age and pose.
    for puppy in [false, true] {
        for pose in POSES {
            let kind = Entity::generated(
                look_name(puppy, pose)
                    .trim_start_matches("animals:")
                    .to_string(),
            );
            let model = if puppy { "chihuahua_pup" } else { "chihuahua" };
            drop(
                r.entity(kind)
                    .size(0.25, 0.36)
                    .passive()
                    .model_file(model)
                    .animations(&format!("{model}{}", pose.suffix()))
                    .texture(TEXTURE),
            );
        }
    }
    r.entity(&CHIHUAHUA)
        .size(0.25, 0.36)
        .health(8.0)
        .speed(2.8)
        .attack_damage(2.0)
        .passive()
        .model_file("chihuahua")
        .animations("chihuahua")
        .texture(TEXTURE)
        .attach(&DOG)
        .attach(&SEEN)
        .goals(goals())
        .on_interact(interact)
        .on_hurt(|w, h| {
            taming::hurt(w, h.entity);
            if let Some(d) = w.component_mut(h.entity, &DOG)
                && d.tamed == 0
            {
                d.panic = 60;
            }
        });
    // Now and then a pair on grassland by day, rarer than pigs and cows.
    r.spawn(&CHIHUAHUA)
        .biome("#animals")
        .ground("#animal_spawn")
        .light(7, 15)
        .group(1, 2)
        .chance(0.02);
}

// ------------------------------------------------------------------ behavior

fn goals() -> Goals {
    Goals::new()
        .then(animal::upkeep(0))
        .then(upkeep)
        .then(taming::behave(0))
        .then(signatures::alarm)
        .then(signatures::tracking)
        .then(commands::act(0))
        .then(|w: &World, id: u64, st: &EntityState| {
            // Hurt: runs about for three seconds.
            if w.component(id, &DOG).is_none_or(|d| d.panic == 0) {
                return false;
            }
            if w.time() % 20 == id % 20 || !w.navigating(id) {
                let a = w.random_below(628) as f64 / 100.0;
                w.navigate(id, [st.pos[0] + a.cos() * 6.0, st.pos[1], st.pos[2] + a.sin() * 6.0], 1.8);
            }
            true
        })
        .then(crate::breeding::behave(0))
        .then(commands::hold)
        .then(ai::wander(0.45, 6.0))
}

/// Every tick, after its level and growing up (`animal::upkeep`): its look applied, time passing.
fn upkeep(w: &World, id: u64, _st: &EntityState) -> bool {
    let puppy = crate::breeding::young(w, id);
    let Some(d) = w.component_mut(id, &DOG) else { return false };
    d.panic = d.panic.saturating_sub(1);
    // A tame told to stay sits (once it is there).
    let sitting = d.tamed != 0 && commands::command(w, id) == commands::STAY && !w.navigating(id);
    let Some(s) = w.component_mut(id, &SEEN) else { return false };
    s.sat = if sitting { s.sat.saturating_add(1) } else { 0 };
    let pose = match (sitting, s.sat > LIE_TICKS) {
        (false, _) => Pose::Stand,
        (true, false) => Pose::Sit,
        (true, true) => Pose::Lie,
    };
    // The look in use, as 1 + puppy + 2 * pose (0: none yet).
    let look = 1 + puppy as u32 + 2 * pose as u32;
    if s.look != look {
        s.look = look;
        w.set_look(id, Some(&look_name(puppy, pose)));
        w.set_size(id, puppy.then_some([0.15, 0.18, 0.15]));
    }
    false
}

/// A player right clicks a dog: a wild one is fed toward taming (`taming.rs`), meat puts a
/// grown tamed one in love (or helps a puppy grow), anything else is as for every tame
/// (`commands::interact`: it eats, or sits or follows).
fn interact(e: &mut EntityEvent) {
    let (w, id, player) = (e.world(), e.entity(), e.other());
    if screen::interact(&w, id, player) {
        return;
    }
    let held = w.inventory(player).get(w.selected_slot(player));
    let Some(d) = w.component(id, &DOG).copied() else { return };
    if d.tamed == 0 {
        taming::feed(&w, id, player);
        return;
    }
    if !commands::can_command(&w, id, player) {
        return;
    }
    // Something an animal drops (and the dog doesn't eat): it tracks that animal down.
    if species::SPECIES[0].food(held.item).is_none() && signatures::track(&w, id, held.item) {
        return;
    }
    commands::interact(&w, id, player);
}

/// `id` was let go: a wild dog again.
pub(crate) fn on_released(w: &World, id: u64) {
    if let Some(d) = w.component_mut(id, &DOG) {
        (d.tamed, d.love) = (0, 0);
    }
}

/// `id` was just tamed: a dog keeps its own record of that too.
pub(crate) fn on_tamed(w: &World, id: u64) {
    if let Some(d) = w.component_mut(id, &DOG) {
        d.tamed = 1;
    }
}

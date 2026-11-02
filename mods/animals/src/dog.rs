//! Dogs (Canis familiaris), by breed; one so far, the chihuahua: small long-haired dogs on grassland,
//! each with its own coat (8 colors, 6 patterns, 2 marking layouts: 96 coats, to be replaced by the
//! dog coat genes of `genetics/dog.json`). Tame one by feeding it when it asks (`taming.rs`; bones
//! and cooked meat are its favorites), make it sit or follow by right clicking it, breed two tamed
//! ones with meat: the puppy takes its color from one parent and its pattern from either, now and
//! then something new, and grows up in twenty minutes (meat speeds it along). Its signatures (an
//! alarm, tracking) are in `signatures.rs`.
//!
//! A breed is its own entity kind with its own model (`animals:chihuahua`); the species' biology,
//! genes and behavior are shared (`species.rs`: species "dog", breed "chihuahua"). The model is a
//! papillon-style long-haired chihuahua made of boxes and flat fur panels with a hand-painted
//! texture (Hytale's style), built in Blockbench: `models/chihuahua.geo.json`,
//! `animations/chihuahua[_pup][_sit|_lie].animation.json` and
//! `textures/entity/chihuahua/chihuahua.png` are built from its source in `source/dog/chihuahua`
//! (`build.py`; see `docs/creatures.md`). All coats draw with that one texture for now; the coat
//! is still chosen and inherited. A coat is a look (`World::set_look`): one entity kind per coat,
//! age and pose, drawn in place of the dog. Puppies have their own model file (half size, a bigger
//! head) on the same layout, so the same picture fits. A pose is the look's animation file, whose
//! `idle` holds it (sitting, lying down), so every player sees it whenever they come by.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::animal::{self, OWNER};
use crate::{commands, screen, signatures, species, taming};

pub(crate) static CHIHUAHUA: Entity = Entity::named("chihuahua");
static HEART: Particle = Particle::named("heart");
static MEATS: [Item; 4] =
    [Item::named("base:beef"), Item::named("base:cooked_beef"), Item::named("base:porkchop"), Item::named("base:cooked_porkchop")];

const COLORS: u32 = 8;
const PATTERNS: u32 = 6;
const MARKS: u32 = 2;
const COATS: u32 = COLORS * PATTERNS * MARKS;
/// The dog's texture (every coat, for now).
const TEXTURE: &str = "chihuahua/chihuahua";
/// Twenty minutes to grow up.
const GROW_TICKS: u32 = 24_000;
/// Sitting this long (thirty seconds), a dog lies down.
const LIE_TICKS: u32 = 600;
/// Thirty seconds in love, five minutes before the next litter.
const LOVE_TICKS: u32 = 600;
const COOLDOWN_TICKS: u32 = 6_000;

value! {
    #[derive(Default)]
    /// A dog's own state, saved with it.
    pub struct Dog {
        /// Coat + 1 (0: not chosen yet).
        pub coat: u32,
        pub tamed: u32,
        /// Ticks left in love, before breeding again, and before growing up (0: grown).
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

/// The kind a dog is drawn as: one per coat, grown or puppy, and pose.
fn look_name(coat: u32, puppy: bool, pose: Pose) -> String {
    format!("animals:chihuahua_{}{}_{coat}", if puppy { "pup" } else { "coat" }, pose.suffix())
}

// ------------------------------------------------------------------ registration

pub fn init(r: &mut Registry) {
    r.entity_component(&DOG, Dog::default(), component_flags::SAVE);
    r.entity_component(&SEEN, Seen::default(), 0);
    // The looks, drawn in place of a dog with that coat, age and pose.
    for coat in 0..COATS {
        for puppy in [false, true] {
            for pose in POSES {
                let kind = Entity::generated(
                    look_name(coat, puppy, pose)
                        .trim_start_matches("animals:")
                        .to_string(),
                );
                let model = if puppy { "chihuahua_pup" } else { "chihuahua" };
                drop(
                    r.entity(kind)
                        .size(0.35, 0.5)
                        .passive()
                        .model_file(model)
                        .animations(&format!("{model}{}", pose.suffix()))
                        .texture(TEXTURE),
                );
            }
        }
    }
    r.entity(&CHIHUAHUA)
        .size(0.35, 0.5)
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
        .then(breed)
        .then(commands::hold)
        .then(ai::wander(0.45, 6.0))
}

/// Every tick, before anything else: a coat for a new dog, its look applied, time passing.
fn upkeep(w: &World, id: u64, _st: &EntityState) -> bool {
    let Some(d) = w.component_mut(id, &DOG) else { return false };
    if d.coat == 0 {
        d.coat = 1 + w.random_below(COATS);
    }
    d.love = d.love.saturating_sub(1);
    d.cooldown = d.cooldown.saturating_sub(1);
    d.panic = d.panic.saturating_sub(1);
    d.age = d.age.saturating_sub(1);
    // A tame told to stay sits (once it is there).
    let staying = d.tamed != 0 && commands::command(w, id) == commands::STAY && !w.navigating(id);
    let (coat, puppy, sitting) = (d.coat - 1, d.age > 0, staying);
    let stage = match d.age * 3 / GROW_TICKS.max(1) {
        _ if d.age == 0 => animal::ADULT,
        2.. => animal::BABY,
        1 => animal::JUVENILE,
        _ => animal::ADOLESCENT,
    };
    if let Some(a) = w.component_mut(id, &animal::ANIMAL) {
        a.stage = stage;
    }
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
        w.set_look(id, Some(&look_name(coat, puppy, pose)));
        w.set_size(id, puppy.then_some([0.2, 0.25, 0.2]));
    }
    false
}

/// In love: goes to another dog in love nearby, of any breed (or waits for it, sitting); when
/// they meet, a puppy of the lower id's breed.
fn breed(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(me) = w.component(id, &DOG).copied().filter(|d| d.love > 0) else { return false };
    let species = |o: u64| w.component(o, &animal::ANIMAL).and_then(|a| a.species());
    let Some(mine) = species(id) else { return false };
    let partner = w
        .near(st.pos, 8.0, None)
        .into_iter()
        .find(|&o| {
            o != id
                && species(o).is_some_and(|s| species::same_species(s, mine))
                && w.component(o, &DOG)
                    .is_some_and(|d| d.love > 0 && d.age == 0)
        });
    let Some(partner) = partner else { return false };
    let Some(ps) = w.entity(partner) else { return false };
    let d2 = (ps.pos[0] - st.pos[0]).powi(2) + (ps.pos[2] - st.pos[2]).powi(2);
    if d2 > 1.2 * 1.2 {
        if me.tamed != 0 && commands::command(w, id) == commands::STAY {
            return false;
        }
        w.navigate(id, ps.pos, 1.0);
        return true;
    }
    // The pair meets once: the lower id has the puppy.
    if id > partner {
        return true;
    }
    let (Some(a), Some(b)) = (w.component(id, &DOG).copied(), w.component(partner, &DOG).copied()) else { return true };
    let coat = inherit(w, a.coat.max(1) - 1, b.coat.max(1) - 1);
    let at = [(st.pos[0] + ps.pos[0]) / 2.0, st.pos[1], (st.pos[2] + ps.pos[2]) / 2.0];
    let pup = w.spawn(mine.kind, at);
    if pup != 0 {
        let owner = w.component(id, &OWNER).copied().unwrap_or([0; 64]);
        if let Some(d) = w.component_mut(pup, &DOG) {
            *d = Dog { coat: coat + 1, tamed: (owner[0] != 0) as u32, age: GROW_TICKS, ..Dog::default() };
        }
        if let Some(o) = w.component_mut(pup, &OWNER) {
            *o = owner;
        }
        weft_sdk::info!("chihuahua {pup} born to {id} and {partner}, coat {coat}");
    }
    for p in [id, partner] {
        if let Some(d) = w.component_mut(p, &DOG) {
            d.love = 0;
            d.cooldown = COOLDOWN_TICKS;
        }
    }
    hearts(w, at);
    true
}

/// A puppy's coat: its color from one parent, its pattern from either, its markings at random,
/// and now and then a color or pattern neither parent has.
fn inherit(w: &World, a: u32, b: u32) -> u32 {
    let split = |c: u32| (c / (PATTERNS * MARKS), (c / MARKS) % PATTERNS);
    let ((ca, pa), (cb, pb)) = (split(a), split(b));
    let pick = |x: u32, y: u32| if w.random_below(2) == 0 { x } else { y };
    let color = if w.random_below(8) == 0 { w.random_below(COLORS) } else { pick(ca, cb) };
    let pattern = if w.random_below(12) == 0 { w.random_below(PATTERNS) } else { pick(pa, pb) };
    (color * PATTERNS + pattern) * MARKS + w.random_below(MARKS)
}

fn hearts(w: &World, at: [f64; 3]) {
    drop(
        w.spawn_particles(&HEART, [at[0], at[1] + 0.6, at[2]])
            .count(5)
            .spread(0.3)
            .velocity([0.0, 0.6, 0.0]),
    );
}

/// A player right clicks a dog: a wild one is fed toward taming (`taming.rs`), meat puts a
/// grown tamed one in love (or helps a puppy grow), anything else is as for every tame
/// (`commands::interact`: it eats, or sits or follows).
fn interact(e: &mut EntityEvent) {
    let (w, id, player) = (e.world(), e.entity(), e.other());
    if screen::interact(&w, id, player) {
        return;
    }
    let Some(st) = w.entity(id) else { return };
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
    if MEATS.iter().any(|m| m.id() == held.item) {
        animal::feed(&w, id, 20);
        if d.age > 0 {
            taming::take_one(&w, player);
            if let Some(d) = w.component_mut(id, &DOG) {
                d.age = d.age * 9 / 10;
            }
            hearts(&w, st.pos);
        } else if d.love == 0 && d.cooldown == 0 {
            taming::take_one(&w, player);
            if let Some(d) = w.component_mut(id, &DOG) {
                d.love = LOVE_TICKS;
            }
            weft_sdk::info!("chihuahua {id} in love");
            hearts(&w, st.pos);
        }
        return;
    }
    commands::interact(&w, id, player);
}

/// `id` was just tamed: a dog keeps its own record of that too.
pub(crate) fn on_tamed(w: &World, id: u64) {
    if let Some(d) = w.component_mut(id, &DOG) {
        d.tamed = 1;
    }
}


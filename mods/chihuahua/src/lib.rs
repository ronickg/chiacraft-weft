//! Chihuahuas: small long-haired dogs on grassland, each with its own coat (8 colors, 6 patterns,
//! 2 marking layouts: 96 coats). Tame one with bones (a third of the time it
//! takes), make it sit or follow by right clicking it, breed two tamed ones with meat: the puppy
//! takes its color from one parent and its pattern from either, now and then something new, and
//! grows up in twenty minutes (meat speeds it along).
//!
//! The model, its animations and the coats are files `tools/chihuahua.py` writes: boxes for the
//! body and flat fur panels, cut out where the coat is clear, for ears, ruff, tail and feathering.
//! A coat is a look (`World::set_look`): one entity kind per coat, age and pose, drawn in place of
//! the dog. Puppies have their own model file (half size, a bigger head) on the same layout, so
//! the same pictures fit. A pose is the look's animation file, whose `idle` holds it (sitting,
//! lying down), so every player sees it whenever they come by. The owner is kept by player key, which the engine's bridge gives
//! (`player_key`), so a dog knows its owner after a restart.

use std::sync::OnceLock;

use weft_sdk::abi::{ItemStack, Str};
use weft_sdk::ai::Goals;
use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

static CHIHUAHUA: Entity = Entity::named("chihuahua");
static HEART: Particle = Particle::named("heart");
static POOF: Particle = Particle::named("core:poof");
static BONE: Item = Item::named("base:bone");
static MEATS: [Item; 4] = [Item::named("base:beef"), Item::named("base:cooked_beef"), Item::named("base:porkchop"), Item::named("base:cooked_porkchop")];

const COLORS: u32 = 8;
const PATTERNS: u32 = 6;
const MARKS: u32 = 2;
const COATS: u32 = COLORS * PATTERNS * MARKS;
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
        pub sitting: u32,
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
    /// What is not saved: the look applied, the owner's entity as last found, ticks sitting.
    pub struct Seen {
        pub look: u32,
        pub owner: u64,
        pub sat: u32,
    }
}

static DOG: Component<Dog> = Component::named("dog");
static SEEN: Component<Seen> = Component::named("seen");
/// The owner's player key (64 hex digits), zeros while wild.
static OWNER: Component<[u8; 64]> = Component::named("owner");

// ------------------------------------------------------------------ the engine's bridge

type Query = unsafe extern "C" fn(query: Str, arg: Str) -> Str;
static BRIDGE: OnceLock<Query> = OnceLock::new();

/// The engine hands the mod its bridge when the mod loads.
///
/// # Safety
/// `q` must be the engine's `bridge::query`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chiacraft_bridge(q: Query) {
    let _ = BRIDGE.set(q);
}

/// A player's key: what their saves are keyed by, the same after a restart.
fn player_key(player: u64) -> String {
    BRIDGE.get().map(|f| unsafe { f(Str::new("player_key"), Str::new(&player.to_string())).as_str().to_string() }).unwrap_or_default()
}

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
    format!("chihuahua:{}{}_{coat}", if puppy { "pup" } else { "coat" }, pose.suffix())
}

// ------------------------------------------------------------------ registration

fn init(r: &mut Registry) {
    r.entity_component(&DOG, Dog::default(), component_flags::SAVE);
    r.entity_component(&SEEN, Seen::default(), 0);
    r.entity_component(&OWNER, [0u8; 64], component_flags::SAVE);
    r.particle(&HEART).textures(&["chihuahua:heart"]).life(0.8, 1.2).size(0.12, 0.16).gravity(-0.4).drag(0.9).fullbright();
    // The looks, drawn in place of a dog with that coat, age and pose.
    for coat in 0..COATS {
        for puppy in [false, true] {
            for pose in POSES {
                let kind = Entity::generated(look_name(coat, puppy, pose).trim_start_matches("chihuahua:").to_string());
                let model = if puppy { "chihuahua_pup" } else { "chihuahua" };
                drop(r.entity(kind).size(0.35, 0.5).passive().model_file(model).animations(&format!("{model}{}", pose.suffix())).texture(&format!("chihuahua/coat_{coat}")));
            }
        }
    }
    r.entity(&CHIHUAHUA)
        .size(0.35, 0.5)
        .health(8.0)
        .speed(2.8)
        .passive()
        .model_file("chihuahua")
        .animations("chihuahua")
        .texture("chihuahua/coat_0")
        .attach(&DOG)
        .attach(&SEEN)
        .attach(&OWNER)
        .goals(goals())
        .on_interact(interact)
        .on_hurt(|w, h| {
            if let Some(d) = w.component_mut(h.entity, &DOG) {
                d.sitting = 0;
                d.panic = 60;
            }
        });
    // Now and then a pair on grassland by day, rarer than pigs and cows.
    r.spawn(&CHIHUAHUA).biome("#animals").ground("#animal_spawn").light(7, 15).group(1, 2).chance(0.02);
}

// ------------------------------------------------------------------ behavior

fn goals() -> Goals {
    Goals::new()
        .then(upkeep)
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
        .then(|w: &World, id: u64, st: &EntityState| {
            // Sitting: stays put.
            let sitting = w.component(id, &DOG).is_some_and(|d| d.tamed != 0 && d.sitting != 0);
            if sitting && w.navigating(id) {
                w.navigate(id, st.pos, 0.0);
            }
            sitting
        })
        .then(follow_owner)
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
    let (coat, puppy, sitting) = (d.coat - 1, d.age > 0, d.tamed != 0 && d.sitting != 0);
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

/// The owner's entity, if they are here (looked up by key now and then).
fn owner_of(w: &World, id: u64) -> Option<u64> {
    let key = w.component(id, &OWNER).filter(|k| k[0] != 0).map(|k| String::from_utf8_lossy(k).into_owned())?;
    let cached = w.component(id, &SEEN).map_or(0, |s| s.owner);
    if cached != 0 && w.entity(cached).is_some() && (w.time() + id) % 40 != 0 {
        return Some(cached);
    }
    let found = w.players().into_iter().find(|&p| player_key(p) == key);
    if let Some(s) = w.component_mut(id, &SEEN) {
        s.owner = found.unwrap_or(0);
    }
    found
}

/// Tamed and not sitting: keeps near its owner, catching up when left behind.
fn follow_owner(w: &World, id: u64, st: &EntityState) -> bool {
    if w.component(id, &DOG).is_none_or(|d| d.tamed == 0) {
        return false;
    }
    let Some(owner) = owner_of(w, id) else { return false };
    let Some(o) = w.entity(owner) else { return false };
    let d2 = (o.pos[0] - st.pos[0]).powi(2) + (o.pos[2] - st.pos[2]).powi(2);
    if d2 > 24.0 * 24.0 || (o.pos[1] - st.pos[1]).abs() > 12.0 {
        // Far behind: there in a moment, beside the owner.
        let mut s = *st;
        s.pos = [o.pos[0] + 1.0, o.pos[1], o.pos[2] + 1.0];
        w.set_entity(id, &s);
        return true;
    }
    if d2 > 3.5 * 3.5 {
        w.navigate(id, o.pos, 1.3);
        return true;
    }
    false
}

/// In love: goes to another dog in love nearby (or waits for it, sitting); when they meet, a
/// puppy.
fn breed(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(me) = w.component(id, &DOG).copied().filter(|d| d.love > 0) else { return false };
    let partner = w.near(st.pos, 8.0, Some(&CHIHUAHUA)).into_iter().find(|&o| o != id && w.component(o, &DOG).is_some_and(|d| d.love > 0 && d.age == 0));
    let Some(partner) = partner else { return false };
    let Some(ps) = w.entity(partner) else { return false };
    let d2 = (ps.pos[0] - st.pos[0]).powi(2) + (ps.pos[2] - st.pos[2]).powi(2);
    if d2 > 1.2 * 1.2 {
        if me.tamed != 0 && me.sitting != 0 {
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
    let pup = w.spawn(&CHIHUAHUA, at);
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
    drop(w.spawn_particles(&HEART, [at[0], at[1] + 0.6, at[2]]).count(5).spread(0.3).velocity([0.0, 0.6, 0.0]));
}

/// A held item from the player's hand, used up (unless in creative).
fn take_one(w: &World, player: u64) {
    if w.creative(player) {
        return;
    }
    let inv = w.inventory(player);
    let slot = w.selected_slot(player);
    let mut s = inv.get(slot);
    s.count = s.count.saturating_sub(1);
    inv.set(slot, if s.count == 0 { ItemStack::EMPTY } else { s });
}

/// A player right clicks a dog: a bone tames a wild one (a third of the time), meat puts a
/// grown tamed one in love (or helps a puppy grow), anything else makes its owner's dog sit or
/// follow.
fn interact(e: &mut EntityEvent) {
    let (w, id, player) = (e.world(), e.entity(), e.other());
    let Some(st) = w.entity(id) else { return };
    let held = w.inventory(player).get(w.selected_slot(player));
    let Some(d) = w.component(id, &DOG).copied() else { return };
    let key = player_key(player);
    let mine = w.component(id, &OWNER).is_some_and(|k| k[0] != 0 && String::from_utf8_lossy(k) == key);
    if d.tamed == 0 {
        if held.item != BONE.id() || key.len() != 64 {
            return;
        }
        take_one(&w, player);
        if w.random_below(3) == 0 {
            if let Some(o) = w.component_mut(id, &OWNER) {
                o.copy_from_slice(key.as_bytes());
            }
            if let Some(d) = w.component_mut(id, &DOG) {
                d.tamed = 1;
                d.sitting = 1;
            }
            weft_sdk::info!("chihuahua {id} tamed");
            if let Some(s) = w.component_mut(id, &SEEN) {
                s.owner = player;
            }
            hearts(&w, st.pos);
        } else {
            drop(w.spawn_particles(&POOF, [st.pos[0], st.pos[1] + 0.5, st.pos[2]]).count(4).spread(0.2));
        }
        return;
    }
    if !mine {
        return;
    }
    if MEATS.iter().any(|m| m.id() == held.item) {
        if d.age > 0 {
            take_one(&w, player);
            if let Some(d) = w.component_mut(id, &DOG) {
                d.age = d.age * 9 / 10;
            }
            hearts(&w, st.pos);
        } else if d.love == 0 && d.cooldown == 0 {
            take_one(&w, player);
            if let Some(d) = w.component_mut(id, &DOG) {
                d.love = LOVE_TICKS;
            }
            weft_sdk::info!("chihuahua {id} in love");
            hearts(&w, st.pos);
        }
        return;
    }
    if let Some(d) = w.component_mut(id, &DOG) {
        d.sitting = 1 - d.sitting;
        weft_sdk::info!("chihuahua {id} {}", if d.sitting != 0 { "sits" } else { "stands" });
    }
    if w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
}

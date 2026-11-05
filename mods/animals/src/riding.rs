//! Riding (`notes/ANIMALS.md`, "Riding"): a tame of a rideable species, saddled with its saddle
//! (right click it with one), grown up, with a bond of 25 or more, is ridden by its owner or a
//! tribe member: an empty hand gets on. The engine's riding (Weft's seats and steering) does
//! the rest: the rider sits in the kind's seat (`seat`: `Species::seat` above its feet,
//! `seat_forward` ahead), and their keys move it on their own client (`steering`): walking it
//! faces where the rider looks, it runs while sprinting and jumps half again as high; the
//! dismount key gets off beside it.
//!
//! One that flies (`Species::flight`, the wyvern) takes off when its rider holds jump. In the
//! air it flies where the rider looks while they go forward (look up to climb, down to dive),
//! sideways with the side keys, climbs with jump and comes down with sneak, faster sprinting,
//! and hovers when no key is held; it beats its wings, gliding as it dives (`ridden`, from how it
//! moves). It lands when it touches the ground and isn't climbing.

use weft_sdk::abi::Steer;
use weft_sdk::riding::{Exit, Seat, Steering};
use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::species::SPECIES;

/// A tame's saddle (1: saddled), saved.
pub static SADDLED: Component<u32> = Component::named("saddled");
/// Being ridden (1), to tidy up once the rider is gone (not saved).
static RIDDEN: Component<u32> = Component::named("ridden");

value! {
    #[derive(Default)]
    /// A flying mount in the air (not saved: after a restart it's on foot, and falls if it was up).
    pub struct Flight {
        /// Ticks it has been in the air ridden + 1 (0: on the ground).
        pub up: u32,
        /// The animation playing: 0 none, 1 flying, 2 gliding.
        pub anim: u32,
    }
}

pub(crate) static FLIGHT: Component<Flight> = Component::named("flight");
/// Flying faster sprinting, and climbing or coming down at this share of the speed.
const FLY_SPRINT: f32 = 1.5;
const FLY_VERTICAL: f32 = 0.7;
/// Climbing and coming down, blocks a second for each unit of `Species::flight`: `rise` is an
/// upward speed the engine sets outright, while going ahead builds up against the air's drag, so
/// at the walking-speed scale a wyvern (flight 8) shot up 28 blocks a second (PC run
/// aw_sync_wyvern6). About 7 for a held jump now.
const CLIMB_PER_FLIGHT: f32 = 1.25;
/// Diving faster than this, blocks a second, with its rider going forward, it glides.
const GLIDE_BELOW: f32 = -2.0;
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static POOF: Particle = Particle::named("core:poof");
/// Each rideable species' saddle, by species name (made once).
static SADDLES: std::sync::OnceLock<Vec<(&'static str, &'static Item)>> = std::sync::OnceLock::new();

/// The bond riding needs.
const BOND: u8 = 25;
/// Running speed over walking.
const SPRINT: f32 = 1.7;
/// A ridden jump, blocks a second up: half again a mob's (the engine's 0.42 a tick).
const RIDE_JUMP: f32 = 0.42 * 20.0 * 1.5;

pub fn init(r: &mut Registry) {
    r.entity_component(&SADDLED, 0u32, component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&RIDDEN, 0u32, 0);
    r.entity_component(&FLIGHT, Flight::default(), 0);
    let mut saddles = Vec::new();
    for s in SPECIES.iter().filter(|s| s.seat > 0.0) {
        r.attach(s.kind, &SADDLED);
        r.attach(s.kind, &RIDDEN);
        if s.flight > 0.0 {
            r.attach(s.kind, &FLIGHT);
        }
        let name = format!("{}_saddle", s.kind_name());
        let saddle = Item::generated(name.clone());
        r.item(saddle).max_stack(1).tab(&TOOLS);
        if s.seat > 3.0 {
            // A saddle for a dragon-sized back: more leather, an iron frame.
            r.shaped(&CRAFTING, &name, 1, &["lll", "lil", "s s"], &[('l', "base:leather"), ('i', "base:iron_ingot"), ('s', "base:string")]);
        } else {
            r.shaped(&CRAFTING, &name, 1, &["lll", "s s"], &[('l', "base:leather"), ('s', "base:string")]);
        }
        saddles.push((s.kind_name(), saddle));
    }
    let _ = SADDLES.set(saddles);
    // A flyer coming down under its rider lands; it doesn't fall. The engine counts a steered
    // mount's way down as a fall (it doesn't know it flies), and a wyvern hurt itself landing.
    r.intercept(&actions::HURT, |q| {
        let w = q.world();
        let id = q.entity();
        let flies = w
            .component(id, &ANIMAL)
            .and_then(|a| a.species())
            .is_some_and(|s| s.flight > 0.0);
        if q.query().cause == damage_cause::FALL && flies && rider_of(&w, id).is_some() { Verdict::Deny } else { Verdict::Abstain }
    });
}

/// The seat of rideable species `index`, for its kind (`EntityBuilder::seat`): `Species::seat`
/// above its feet and `seat_forward` ahead; its rider steers, and gets off beside it.
// The engine scales the seat with the animal (`set_entity_scale`: genes and growth).
pub fn seat(index: usize) -> Seat {
    let s = &SPECIES[index];
    Seat::at([0.0, s.seat as f64, s.seat_forward as f64])
        .steers()
        .exit(Exit::nearest(2.0))
}

/// How a rider's keys move rideable species `index`
/// (`EntityBuilder::steering`): on the client, from the keys and the mount as it is.
pub fn steering(index: usize) -> Steering {
    let flight = SPECIES[index].flight;
    Steering::new(move |i, m| {
        let sprint = i.sprint != 0 && i.forward > 0.0;
        let moving = i.forward != 0.0 || i.strafe != 0.0;
        // Walking: it faces where its rider looks (standing, the rider looks about freely).
        let on_foot = Steer {
            forward: i.forward,
            strafe: i.strafe,
            speed: if sprint { SPRINT } else { 1.0 },
            yaw: if moving { i.yaw } else { f32::NAN },
            jump: if i.jump_held == 1 && m.on_ground != 0 { RIDE_JUMP } else { 0.0 },
            ..Steer::DEFAULT
        };
        if flight <= 0.0 || (m.on_ground != 0 && i.up == 0) {
            return on_foot;
        }
        // In the air, or taking off: where the rider looks, climbing with the look and jump,
        // coming down with sneak; hovering with no key held.
        let fast = if sprint { FLY_SPRINT } else { 1.0 };
        let speed = flight * fast;
        let climb = i.forward * i.pitch.sin() + (i.up as f32 - i.down as f32) * FLY_VERTICAL;
        Steer {
            forward: i.forward * i.pitch.cos(),
            strafe: i.strafe,
            speed,
            yaw: if moving { i.yaw } else { f32::NAN },
            flies: 1,
            rise: climb * CLIMB_PER_FLIGHT * speed,
            ..Steer::DEFAULT
        }
    })
}

/// Its species' saddle (an item id), if it can be ridden.
fn saddle_of(w: &World, id: u64) -> Option<u32> {
    let s = w.component(id, &ANIMAL)?.species()?;
    SADDLES
        .get()?
        .iter()
        .find(|(n, _)| *n == s.kind_name())
        .map(|(_, i)| i.id())
}

/// Takes its saddle off and drops it beside it (it is let go).
pub fn unsaddle(w: &World, id: u64) {
    let (Some(saddle), Some(st)) = (saddle_of(w, id), w.entity(id)) else { return };
    if let Some(s) = w.component_mut(id, &SADDLED)
        && *s != 0
    {
        *s = 0;
        w.spawn_item([st.pos.x, st.pos.y + 0.5, st.pos.z], ItemStack::new(saddle, 1));
    }
}

/// A player right clicks a tame: its saddle saddles it; an empty hand on a saddled one gets on.
/// True if the click was used.
pub fn interact(w: &World, id: u64, player: u64) -> bool {
    let Some(saddle) = saddle_of(w, id) else { return false };
    if !crate::animal::tamed(w, id) || !crate::commands::can_command(w, id, player) {
        return false;
    }
    let held = w.inventory(player).get(w.selected_slot(player));
    let saddled = w.component(id, &SADDLED).is_some_and(|s| s != 0);
    if !saddled {
        if held.item != saddle {
            return false;
        }
        crate::taming::take_one(w, player);
        if let Some(s) = w.component_mut(id, &SADDLED) {
            *s = 1;
        }
        return true;
    }
    if !held.is_empty() || rider_of(w, id).is_some() {
        return false;
    }
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    let Some(st) = w.entity(id) else { return false };
    let out = w
        .component(id, &crate::knockout::KNOCKOUT)
        .is_some_and(|k| k.out > 0);
    if a.bond < BOND || a.stage < crate::animal::ADULT || out {
        // Not ready to carry anyone: it shakes them off.
        drop(
            w.spawn_particles(&POOF, [st.pos.x, st.pos.y + 0.6, st.pos.z])
                .count(4)
                .spread(0.2),
        );
        return true;
    }
    if w.mount(player, id) {
        crate::journal::note_animal(w, player, id, animals_core::journal::RIDDEN);
        if let Some(r) = w.component_mut(id, &RIDDEN) {
            *r = 1;
        }
        weft_sdk::info!("{player} rides {id}");
    }
    true
}

/// Who rides `id` (its seat's rider), if anyone.
pub fn rider_of(w: &World, id: u64) -> Option<u64> {
    w.riders(id).into_iter().flatten().next()
}

/// A goal for rideable species, first: ridden, its rider's client moves it (`steering`), so it
/// does nothing else; a flying one beats its wings or glides by how it moves. Once its rider is
/// off, it is tidied up.
pub fn ridden(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        if rider_of(w, id).is_none() {
            if w.component(id, &RIDDEN).is_some_and(|r| r != 0) {
                end_ride(w, id);
            }
            return false;
        }
        if let Some(r) = w.component_mut(id, &RIDDEN) {
            *r = 1;
        }
        crate::wake(w, id, index);
        if w.navigating(id) {
            w.navigate(id, st.pos, 0.0);
        }
        if SPECIES[index].flight > 0.0 {
            wings(w, id, st);
        }
        true
    }
}

/// A ridden flyer's wings: beating in the air, gliding as it dives going forward, folded on the
/// ground.
fn wings(w: &World, id: u64, st: &EntityState) {
    let Some(f) = w.component_mut(id, &FLIGHT) else { return };
    if st.on_ground != 0 {
        if f.up != 0 {
            *f = Flight::default();
            w.action_stop(id);
        }
        return;
    }
    f.up = f.up.saturating_add(1).max(2);
    let ahead = st.vel.x.hypot(st.vel.z) > 1.0;
    let anim = if ahead && (st.vel.y as f32) < GLIDE_BELOW { 2 } else { 1 };
    if f.anim != anim {
        f.anim = anim;
        w.action_play(id, if anim == 2 { "glide" } else { "fly" }, true);
    }
}

fn end_ride(w: &World, id: u64) {
    // Left in the air without its rider, it stops flying (and comes down).
    if let Some(f) = w.component_mut(id, &FLIGHT)
        && f.up != 0
    {
        *f = Flight::default();
        w.action_stop(id);
    }
    if let Some(r) = w.component_mut(id, &RIDDEN) {
        *r = 0;
    }
}

/// Kind `b` of species `index` made rideable when its species
/// is (`Species::seat`): its seat and steering.
pub fn rideable(b: EntityBuilder<'_>, index: usize) -> EntityBuilder<'_> {
    if SPECIES[index].seat > 0.0 { b.seat(seat(index)).steering(steering(index)) } else { b }
}

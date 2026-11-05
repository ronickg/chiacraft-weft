//! The living world's simple creatures (`notes/ECOLOGY.md`, "The food webs and the species to
//! add"): wild, never tamed (`Taming::Wild`), one model and a few animations each. What they
//! share is here, so each animal's own file is its kinds and its body: they forage, bolt from a
//! walker (`taming::behave`) and from hunters that could take them (`hunt::wary`), hunt if they're
//! hunters (`hunt::hunt`: a fox), breed in their season, and now and then settle into a resting pose (a hare in its form, a mouse sitting up to
//! nibble), a look of its own (`World::set_look`), as the capybara's poses.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;
use crate::choose::{Band, Need, Stops, WithBand};
use crate::prof::Timed;

/// One simple creature's body, for every species on its model.
pub struct Body {
    /// Its model and animation files (`<model>.geo.json`, `<model>.animation.json`), and texture.
    pub model: &'static str,
    pub texture: &'static str,
    /// Its full speed (fleeing), and the share of it it wanders at.
    pub speed: f32,
    pub wander: f32,
    /// Its resting pose: the look kind's name (`<model>_<pose>`, its own animation file) and its
    /// box.
    pub rest: (&'static str, (f32, f32)),
    /// Whether it bites (a hunter's); how hard, and its health, come from its species' size, role
    /// and level (`animals_core::danger::body`; its box is its model's, `animals_core::bodies`).
    pub bites: bool,
}

value! {
    #[derive(Default)]
    /// What a simple creature is doing (not saved: it just settles down again).
    pub struct Wild {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Whether the resting look is on.
        pub resting: bool,
        /// Ticks rested so far.
        pub rested: u32,
    }
}

static STATE: Component<Wild> = Component::named("wild_state");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Wild::default(), 0);
}

/// Registers `kinds` (each an entity and its row in `species::SPECIES`) on one `body`.
pub fn register(r: &mut Registry, kinds: &[&'static EntityKind], body: &'static Body) {
    let (look, (w, h)) = body.rest;
    drop(
        r.entity(EntityKind::generated(look.to_string()))
            .size(w, h)
            .passive()
            .model_file(body.model)
            .animations(look)
            .texture(body.texture),
    );
    let (bw, bh) = animals_core::bodies::of(body.model);
    for &kind in kinds {
        let index = crate::species::of(kind);
        r.entity(kind)
            .size(bw, bh)
            .max_health(crate::animal::health(index))
            .walk_speed(body.speed)
            .attack_damage(if body.bites { crate::animal::hit(index) } else { 0.0 })
            .passive()
            .model_file(body.model)
            .animations(body.model)
            .texture(body.texture)
            .attach(&STATE)
            .on_interact(|e| {
                let (w, id, player) = (e.world(), e.entity(), e.other());
                crate::screen::interact(&w, id, player);
                false
            })
            .goals(
                Goals::new()
                    .timed("upkeep", crate::animal::upkeep(index))
                    .timed("startle", startle)
                    .timed("taming", crate::taming::behave(index))
                    .timed("wary", |w: &World, id: u64, st: &EntityState| crate::hunt::wary(w, id, st) && woken(w, id))
                    .timed("huddle", |w: &World, id: u64, st: &EntityState| crate::hunt::huddle(w, id, st) && woken(w, id))
                    .timed("hunt", |w: &World, id: u64, st: &EntityState| crate::hunt::hunt(w, id, st) && woken(w, id))
                    // What it does when nothing is after it, the most pressing first (`choose.rs`).
                    .band(
                        "needs",
                        Band::new()
                            .with(Need::Love, crate::breeding::behave(index))
                            .with(
                                Need::Rest,
                                Stops { goal: move |w: &World, id: u64, st: &EntityState| settle(w, id, st, look), stop: wake },
                            )
                            .with(Need::Watch, crate::senses::watch_goal)
                            .with(Need::Investigate, crate::senses::investigate)
                            .with(Need::Comfort, crate::comfort::seek)
                            .with(Need::Drink, crate::drink::drink)
                            .with(Need::Eat, crate::forage::eat)
                            .with(Need::Wander, ai::wander(body.wander, 8.0)),
                    ),
            )
            .on_hurt(|w, h| {
                crate::taming::hurt(w, h.entity, h.source);
                wake(w, h.entity);
            });
    }
}

/// Bolting: up out of its rest (`taming::behave` scared it).
fn startle(w: &World, id: u64, _: &EntityState) -> bool {
    if w.component(id, &crate::taming::TAMING)
        .is_some_and(|t| t.scared > 0)
    {
        wake(w, id);
    }
    false
}

/// Every tick: now and then rest for a while, showing the resting look.
fn settle(w: &World, id: u64, st: &EntityState, look: &str) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    let chance = crate::routines::rest_chance(400, crate::routines::activity_of(w, id));
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(chance) == 0 {
        c.rest = 600 + w.random_below(1800);
    }
    let resting = c.rest > 0;
    if resting {
        let e = w.tick_elapsed(id);
        c.rest = c.rest.saturating_sub(e as _);
        c.rested = c.rested.saturating_add(e);
    }
    let rested = c.rested;
    if c.resting != resting {
        c.resting = resting;
        c.rested = 0;
        w.set_appearance(id, resting.then(|| format!("animals:{look}")).as_deref());
        if !resting {
            // Rested enough: up, and a stretch.
            crate::anim::clear(w, id);
            crate::anim::act(w, id, crate::anim::Act::Stretch);
        }
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    if resting && animals_core::routines::asleep(crate::routines::activity_of(w, id), rested) {
        crate::anim::hold(w, id, crate::anim::Act::Sleep);
    }
    resting
}

/// Up and running from a hunter (`hunt::wary`): out of its rest.
fn woken(w: &World, id: u64) -> bool {
    wake(w, id);
    true
}

/// Up out of its rest (scared, hurt).
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.resting)
    {
        c.rest = 0;
        c.resting = false;
        w.set_appearance(id, None);
        // Up at once (scared, hurt): no stretch, and not asleep standing.
        crate::anim::clear(w, id);
    }
}

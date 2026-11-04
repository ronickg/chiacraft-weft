//! The living world's simple creatures (`notes/ECOLOGY.md`, "The food webs and the species to
//! add"): wild, never tamed (`Taming::Wild`), one model and a few animations each. What they
//! share is here, so each animal's own file is its kinds and its body: they forage, bolt from a
//! walker (`taming::behave`) and from hunters that could take them (`hunt::wary`), hunt if they're
//! hunters (`hunt::hunt`: a fox), breed in their season, and now and then settle into a resting pose (a hare in its form, a mouse sitting up to
//! nibble), a look of its own (`World::set_look`), as the capybara's poses.

use weft_sdk::ai::Goals;
use weft_sdk::*;

/// One simple creature's body, for every species on its model.
pub struct Body {
    /// Its model and animation files (`<model>.geo.json`, `<model>.animation.json`), and texture.
    pub model: &'static str,
    pub texture: &'static str,
    /// Its box, width and height in blocks, and its health.
    pub size: (f32, f32),
    pub health: f32,
    /// Its full speed (fleeing), and the share of it it wanders at.
    pub speed: f32,
    pub wander: f32,
    /// Its resting pose: the look kind's name (`<model>_<pose>`, its own animation file) and its
    /// box.
    pub rest: (&'static str, (f32, f32)),
    /// What its bite does (a hunter's; 0: it never fights).
    pub damage: f32,
}

value! {
    #[derive(Default)]
    /// What a simple creature is doing (not saved: it just settles down again).
    pub struct Wild {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Whether the resting look is on.
        pub resting: bool,
    }
}

static STATE: Component<Wild> = Component::named("wild_state");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Wild::default(), 0);
}

/// Registers `kinds` (each an entity and its row in `species::SPECIES`) on one `body`.
pub fn register(r: &mut Registry, kinds: &[&'static Entity], body: &'static Body) {
    let (look, (w, h)) = body.rest;
    drop(
        r.entity(Entity::generated(look.to_string()))
            .size(w, h)
            .passive()
            .model_file(body.model)
            .animations(look)
            .texture(body.texture),
    );
    for &kind in kinds {
        let index = crate::species::of(kind);
        r.entity(kind)
            .size(body.size.0, body.size.1)
            .health(body.health)
            .speed(body.speed)
            .attack_damage(body.damage)
            .passive()
            .model_file(body.model)
            .animations(body.model)
            .texture(body.texture)
            .attach(&STATE)
            .on_interact(|e| {
                let (w, id, player) = (e.world(), e.entity(), e.other());
                crate::screen::interact(&w, id, player);
            })
            .goals(
                Goals::new()
                    .then(crate::animal::upkeep(index))
                    .then(startle)
                    .then(crate::taming::behave(index))
                    .then(|w: &World, id: u64, st: &EntityState| crate::hunt::wary(w, id, st) && woken(w, id))
                    .then(|w: &World, id: u64, st: &EntityState| crate::hunt::hunt(w, id, st) && woken(w, id))
                    .then(crate::breeding::behave(index))
                    .then(move |w: &World, id: u64, st: &EntityState| settle(w, id, st, look))
                    .then(crate::comfort::seek)
                    .then(crate::forage::forage)
                    .then(ai::wander(body.wander, 8.0)),
            )
            .on_hurt(|w, h| {
                crate::taming::hurt(w, h.entity);
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
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(400) == 0 {
        c.rest = 600 + w.random_below(1800);
    }
    let resting = c.rest > 0;
    if resting {
        c.rest = c.rest.saturating_sub(w.tick_elapsed(id) as _);
    }
    if c.resting != resting {
        c.resting = resting;
        w.set_look(id, resting.then(|| format!("animals:{look}")).as_deref());
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
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
        w.set_look(id, None);
    }
}

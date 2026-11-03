//! Hares: the brown hare (`HARE`) of open country and the mountain hare (`MOUNTAIN_HARE`) of the
//! north, two species on one model (`source/hare`), each its own coat from its genes. Wild, never
//! tamed (`Taming::Wild`): they graze and browse, bolt from a walker (`taming::behave`) and from
//! hunters that could take them (`hunt::wary`), breed in spring and summer, and now and then
//! settle into a form: pressed low with the ears laid flat, a look of its own
//! (`World::set_look`), as the capybara's poses.
use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static HARE: Entity = Entity::named("hare");
pub(crate) static MOUNTAIN_HARE: Entity = Entity::named("mountain_hare");
/// Each kind, its row in `species::SPECIES`, and its texture.
static KINDS: [(&Entity, usize); 2] = [(&HARE, 7), (&MOUNTAIN_HARE, 8)];
const TEXTURE: &str = "hare/hare";
const FORM: &str = "hare_lie";

value! {
    #[derive(Default)]
    /// What a hare is doing (not saved: it just settles down again).
    pub struct Hare {
        /// Ticks left in its form (0: up and about).
        pub rest: u32,
        /// Whether the form's look is on.
        pub lying: bool,
    }
}

static STATE: Component<Hare> = Component::named("hare_state");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Hare::default(), 0);
    // Lying in its form, drawn in place of a hare at rest.
    drop(
        r.entity(Entity::generated(FORM.to_string()))
            .size(0.4, 0.3)
            .passive()
            .model_file("hare")
            .animations("hare_lie")
            .texture(TEXTURE),
    );
    for (kind, index) in KINDS {
        r.entity(kind)
            .size(0.4, 0.45)
            .health(5.0)
            // Lolloping about slowly (`wander`), flat out when it bolts: faster than a running player.
            .speed(4.0)
            .passive()
            .model_file("hare")
            .animations("hare")
            .texture(TEXTURE)
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
                    .then(|w: &World, id: u64, st: &EntityState| crate::hunt::wary(w, id, st) && wake_up(w, id))
                    .then(crate::breeding::behave(index))
                    .then(settle)
                    .then(crate::comfort::seek)
                    .then(crate::forage::forage)
                    .then(ai::wander(0.3, 8.0)),
            )
            .on_hurt(|w, h| {
                crate::taming::hurt(w, h.entity);
                wake(w, h.entity);
            });
    }
}

/// Bolting: up out of its form (`taming::behave` scared it).
fn startle(w: &World, id: u64, _: &EntityState) -> bool {
    if w.component(id, &crate::taming::TAMING)
        .is_some_and(|t| t.scared > 0)
    {
        wake(w, id);
    }
    false
}

/// Every tick: now and then sit tight in a form for a while, and show it.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(400) == 0 {
        c.rest = 600 + w.random_below(1800);
    }
    let resting = c.rest > 0;
    if resting {
        c.rest -= 1;
    }
    if c.lying != resting {
        c.lying = resting;
        w.set_look(id, resting.then(|| format!("animals:{FORM}")).as_deref());
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

/// Up and running from a hunter (`hunt::wary`): out of its form.
fn wake_up(w: &World, id: u64) -> bool {
    wake(w, id);
    true
}

/// Up out of its form (scared, hurt).
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.lying)
    {
        c.rest = 0;
        c.lying = false;
        w.set_look(id, None);
    }
}

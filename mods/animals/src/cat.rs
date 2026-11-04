//! Cats: orange tabbies on grassland that wander about and now and then settle down: they sit,
//! and after half a minute curl up into a loaf, then get up and go on. A pose is a look
//! (`World::set_look`), as the chihuahua's, so every player sees it. The model, its animations
//! and texture are built from `source/cat` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static CAT: Entity = Entity::named("cat");
const TEXTURE: &str = "cat/cat";
/// Resting this long (thirty seconds), a sitting cat lies down.
const LIE_TICKS: u32 = 600;

value! {
    #[derive(Default)]
    /// What a cat is doing (not saved: it just settles down again).
    pub struct Cat {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Cat> = Component::named("cat_state");

const LOOKS: [(&str, &str); 3] = [("", ""), ("_sit", "cat_sit"), ("_lie", "cat_lie")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Cat::default(), 0);
    // The poses, drawn in place of a cat sitting or lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(0.35, 0.44)
                .passive()
                .model_file("cat")
                .animations(&format!("cat{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&CAT)
        .size(0.35, 0.44)
        .health(6.0)
        .speed(3.0)
        .attack_damage(1.0)
        .passive()
        .model_file("cat")
        .animations("cat")
        .texture(TEXTURE)
        .attach(&STATE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::taming::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(1))
                .then(crate::taming::behave(1))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::signatures::gifts)
                .then(crate::commands::act(1))
                .then(crate::breeding::behave(1))
                .then(settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.5, 7.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
}

/// Every tick: now and then settle down for a while (sit, then lie), and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(300) == 0 {
        c.rest = 400 + w.random_below(1200);
        c.rested = 0;
    }
    let resting = c.rest > 0;
    if resting {
        c.rest -= 1;
        c.rested += 1;
    }
    let pose = match (resting, c.rested > LIE_TICKS) {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    };
    if c.look != pose + 1 {
        c.look = pose + 1;
        let kind = LOOKS[pose as usize].1;
        w.set_look(
            id,
            (!kind.is_empty())
                .then(|| format!("animals:{kind}"))
                .as_deref(),
        );
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

/// Up from resting, to follow or fight.
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.look > 1)
    {
        c.rest = 0;
        c.look = 1;
        w.set_look(id, None);
    }
}

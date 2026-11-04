//! Jaguars (`JAGUAR`): the top hunter of the Pantanal's marshes and the llanos, a full creature on
//! its own model built on the cat's (`source/jaguar`). Wild, it hunts capybaras, caimans, deer and
//! fish of the sizes it takes (`hunt.rs`), with a swipe and a bite (its attacks), calls now and then
//! with its sawing roar, and settles down: it sits, then lies down. Too dangerous to feed from the
//! hand, it is tamed as the raptors are: knocked out and fed while it sleeps (`knockout.rs`); tamed,
//! it follows, sits and fights on command (`commands.rs`). A pose is a look (`World::set_look`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static JAGUAR: Entity = Entity::named("jaguar");
const INDEX: usize = 34;
const TEXTURE: &str = "jaguar/jaguar";
/// Resting this long (half a minute), a sitting jaguar lies down.
const LIE_TICKS: u32 = 600;

value! {
    #[derive(Default)]
    /// What a jaguar is doing (not saved: it just settles down again).
    pub struct Jaguar {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Jaguar> = Component::named("jaguar_state");

const LOOKS: [(&str, &str); 3] = [("", ""), ("_sit", "jaguar_sit"), ("_lie", "jaguar_lie")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Jaguar::default(), 0);
    // The poses, drawn in place of a jaguar sitting or lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(0.8, 0.8)
                .passive()
                .model_file("jaguar")
                .animations(&format!("jaguar{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&JAGUAR)
        .size(0.8, 0.85)
        .health(45.0)
        .speed(5.0)
        .attack_damage(9.0)
        .passive()
        .model_file("jaguar")
        .animations("jaguar")
        .texture(TEXTURE)
        .attach(&STATE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::knockout::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(INDEX))
                .then(crate::knockout::behave(INDEX))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(INDEX))
                .then(crate::breeding::behave(INDEX))
                .then(settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.4, 10.0)),
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
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(400) == 0 {
        c.rest = 400 + w.random_below(1600);
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
    show(w, id, c, pose);
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

fn show(w: &World, id: u64, c: &mut Jaguar, pose: u32) {
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

/// Knocked out: lying down.
pub fn sleep(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE) {
        c.rest = 0;
        show(w, id, c, 2);
    }
}

//! Wolves: the grey wolf (`WOLF`) of the taiga, the forests, the mountains and the tundra, a full
//! creature on its own model (`source/wolf`), and the dog's ancestor, so a natural tame: fed from
//! the hand it comes round (`taming.rs`), then follows, sits and fights on command
//! (`commands.rs`). Wild, it hunts hares, deer and reindeer of the sizes it takes (`hunt.rs`),
//! pouncing and biting (its attacks), howls now and then (its idle), and settles down: it sits,
//! then after a while lies down, the head on its paws. A pose is a look (`World::set_look`), as the
//! cat's.

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static WOLF: Entity = Entity::named("grey_wolf");
const INDEX: usize = 33;
const TEXTURE: &str = "wolf/wolf";
/// Resting this long (half a minute), a sitting wolf lies down.
const LIE_TICKS: u32 = 600;

value! {
    #[derive(Default)]
    /// What a wolf is doing (not saved: it just settles down again).
    pub struct Wolf {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Wolf> = Component::named("wolf_state");

const LOOKS: [(&str, &str); 3] = [("", ""), ("_sit", "wolf_sit"), ("_lie", "wolf_lie")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Wolf::default(), 0);
    // The poses, drawn in place of a wolf sitting or lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(0.6, 0.8)
                .passive()
                .model_file("wolf")
                .animations(&format!("wolf{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&WOLF)
        .size(0.6, 0.85)
        .health(30.0)
        .speed(5.5)
        .attack_damage(6.0)
        .passive()
        .model_file("wolf")
        .animations("wolf")
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
                .then(crate::animal::upkeep(INDEX))
                .then(crate::taming::behave(INDEX))
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
        c.rest = c.rest.saturating_sub(w.tick_elapsed(id) as _);
        c.rested = c.rested.saturating_add(w.tick_elapsed(id) as _);
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

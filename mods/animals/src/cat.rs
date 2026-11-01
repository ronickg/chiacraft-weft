//! Cats: orange tabbies on grassland that wander about and now and then settle down: they sit,
//! and after half a minute curl up into a loaf, then get up and go on. A pose is a look
//! (`World::set_look`), as the chihuahua's, so every player sees it. The model, its animations
//! and texture are built from `source/cat` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

static CAT: Entity = Entity::named("cat");
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
                .size(0.4, 0.5)
                .passive()
                .model_file("cat")
                .animations(&format!("cat{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&CAT)
        .size(0.4, 0.5)
        .health(6.0)
        .speed(3.0)
        .passive()
        .model_file("cat")
        .animations("cat")
        .texture(TEXTURE)
        .attach(&STATE)
        .goals(Goals::new().then(settle).then(ai::wander(0.5, 7.0)))
        .on_hurt(|w, h| {
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
    r.spawn(&CAT)
        .biome("#animals")
        .ground("#animal_spawn")
        .light(7, 15)
        .group(1, 2)
        .chance(0.02);
}

/// Every tick: now and then settle down for a while (sit, then lie), and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && w.random_below(300) == 0 {
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
        w.set_look(id, (!kind.is_empty()).then(|| format!("animals:{kind}")).as_deref());
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

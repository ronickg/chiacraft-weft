//! Velociraptors: feathered, turkey-sized hunters (the real animal, not the film's), in small
//! groups on grassland, rarer than cats. For now they only wander and now and then lie down to
//! rest like birds, legs folded under; hunting is for later. A pose is a look (`World::set_look`),
//! as the chihuahua's. The model, its animations and texture are built from `source/raptor` (see
//! `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static RAPTOR: Entity = Entity::named("raptor");
const TEXTURE: &str = "raptor/raptor";

value! {
    #[derive(Default)]
    /// What a raptor is doing (not saved: it just settles down again).
    pub struct Raptor {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Raptor> = Component::named("raptor_state");

const LOOKS: [(&str, &str); 2] = [("", ""), ("_rest", "raptor_rest")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Raptor::default(), 0);
    // The resting pose, drawn in place of a raptor lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(0.6, 0.9)
                .passive()
                .model_file("raptor")
                .animations(&format!("raptor{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&RAPTOR)
        .size(0.6, 0.9)
        .health(14.0)
        .speed(4.0)
        .passive()
        .model_file("raptor")
        .animations("raptor")
        .texture(TEXTURE)
        .attach(&STATE)
        .on_interact(|e| {
            crate::screen::interact(&e.world(), e.entity(), e.other());
        })
        .goals(Goals::new().then(crate::animal::upkeep(3)).then(settle).then(ai::wander(0.6, 9.0)))
        .on_hurt(|w, h| {
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
    // A small pack now and then, rarer than cats.
    r.spawn(&RAPTOR)
        .biome("#animals")
        .ground("#animal_spawn")
        .light(7, 15)
        .group(2, 3)
        .chance(0.008);
}

/// Every tick: now and then lie down to rest for a while, and show the pose.
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
    let pose = resting as u32;
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

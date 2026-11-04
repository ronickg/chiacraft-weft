//! Psittacosaurus (`notes/ECOLOGY.md`, the Cretaceous's tier 2: the fern forest's plant eater and
//! the Utahraptor's prey there): the Jehol's small parrot-beaked dinosaur, on its hind legs
//! (`source/psittacosaurus`). A simple wild creature (`wild.rs`) in herds: it crops ferns, runs from
//! walkers and hunters, lays its eggs in the wet season, and now and then lies down.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static PSITTACOSAURUS: Entity = Entity::named("psittacosaurus");

static BODY: Body = Body {
    model: "psittacosaurus",
    texture: "psittacosaurus/psittacosaurus",
    size: (0.6, 0.8),
    health: 22.0,
    // A quick runner on its hind legs, about 5 m/s at its fastest (est., from its long shins).
    speed: 5.0,
    wander: 0.3,
    damage: 0.0,
    rest: ("psittacosaurus_lie", (0.6, 0.45)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&PSITTACOSAURUS], &BODY);
}

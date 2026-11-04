//! Ravens (`notes/ECOLOGY.md`, wave one: something for the dead in the cold biomes, where there are
//! no vultures): the common raven of the north's woods, mountains, tundra, desert and coast
//! (`source/raven`). A bird on the ground and in the air (`birds.rs`): it hops and walks about,
//! scavenging and now and then taking a mouse or a vole, and takes off from walkers and hunters.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static RAVEN: Entity = Entity::named("raven");

static WINGS: Wings = Wings {
    model: "raven",
    texture: "raven/raven",
    fly_size: (0.4, 0.3),
    health: 6.0,
    // On the ground it walks and hops at about 2.5 m/s; it flies at about 9.
    speed: 2.5,
    air: 3.5,
    wander: 0.35,
    damage: 1.0,
    height: 6.0,
    reach: 16.0,
    flight: 400,
    restless: 2400,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&RAVEN], &WINGS);
}

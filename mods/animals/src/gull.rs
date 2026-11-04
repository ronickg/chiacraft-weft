//! Gulls: the herring gull (`HERRING_GULL`) of the beaches and the sea's shore (`source/gull`). A bird
//! on the ground and in the air (`birds.rs`): it walks the strandline taking small fish, crabs, the
//! dead and whatever people leave, a mouse now and then (`hunt.rs`), and takes off on long pointed
//! wings, beating and gliding.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static HERRING_GULL: Entity = Entity::named("herring_gull");

static WINGS: Wings = Wings {
    model: "gull",
    texture: "gull/gull",
    size: (0.4, 0.45),
    fly_size: (0.9, 0.3),
    health: 8.0,
    // It walks at about 1.5 m/s; it flies at about 11.
    speed: 2.0,
    air: 5.0,
    wander: 0.35,
    damage: 1.5,
    height: 6.0,
    reach: 20.0,
    flight: 500,
    restless: 2000,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[(&HERRING_GULL, 30)], &WINGS);
}

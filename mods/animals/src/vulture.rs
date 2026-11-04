//! Vultures: the turkey vulture (`TURKEY_VULTURE`) of the llanos, the Pantanal, the desert and the
//! prairie (`source/vulture`). A bird on the ground and in the air (`birds.rs`): it walks about
//! slowly, takes off at a walker, and soars a long way on long, still wings, held open most of the
//! cycle. It hunts nothing: the dead it eats come into the world with the web's third tier.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static TURKEY_VULTURE: Entity = Entity::named("turkey_vulture");

static WINGS: Wings = Wings {
    model: "vulture",
    texture: "vulture/vulture",
    size: (0.45, 0.55),
    fly_size: (1.1, 0.35),
    health: 10.0,
    // It walks little; it soars at about 12 m/s.
    speed: 1.5,
    air: 5.5,
    wander: 0.2,
    damage: 0.0,
    height: 12.0,
    reach: 32.0,
    flight: 800,
    restless: 2400,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&TURKEY_VULTURE], &WINGS);
}

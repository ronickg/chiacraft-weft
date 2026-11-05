//! The early birds (`notes/ECOLOGY.md`, the Cretaceous's tier 2): Confuciusornis (`CONFUCIUSORNIS`)
//! of the Jehol's forests and lakes, and the enantiornithine Gobipteryx (`GOBIPTERYX`) of the Gobi's
//! crags and scrub, two species on one model (`source/early_bird`), Gobipteryx sandier and smaller
//! by its genes. Birds on the ground and in the air (`birds.rs`): they hop about taking insects and
//! seeds (Confuciusornis small fish too, `hunt.rs`), and take off on broad wings.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static CONFUCIUSORNIS: EntityKind = EntityKind::named("confuciusornis");
pub(crate) static GOBIPTERYX: EntityKind = EntityKind::named("gobipteryx");

static WINGS: Wings = Wings {
    model: "early_bird",
    texture: "early_bird/early_bird",
    fly_size: (0.7, 0.25),
    health: 6.0,
    // No record: a crow-sized bird's walk and flight (est.).
    speed: 2.0,
    air: 4.5,
    wander: 0.35,
    damage: 1.0,
    height: 5.0,
    reach: 14.0,
    flight: 300,
    restless: 1500,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&CONFUCIUSORNIS, &GOBIPTERYX], &WINGS);
}

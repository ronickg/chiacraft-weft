//! Owls: the great horned owl (`GREAT_HORNED_OWL`) of every land but the high Arctic, and the snowy
//! owl (`SNOWY_OWL`) of the tundra, two species on one model (`source/owl`), the snowy owl white and
//! nearly without ear tufts by its genes. Birds on the ground and in the air (`birds.rs`): a hunter
//! of mice, voles, hares and birds (`hunt.rs`), it sits upright turning its head right round, and
//! flies off low on broad silent wings.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static GREAT_HORNED_OWL: Entity = Entity::named("great_horned_owl");
pub(crate) static SNOWY_OWL: Entity = Entity::named("snowy_owl");

static WINGS: Wings = Wings {
    model: "owl",
    texture: "owl/owl",
    fly_size: (0.6, 0.35),
    health: 8.0,
    // It walks little; it flies at about 10 m/s.
    speed: 1.5,
    air: 5.0,
    wander: 0.2,
    damage: 2.5,
    height: 4.0,
    reach: 14.0,
    flight: 400,
    restless: 3600,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&GREAT_HORNED_OWL, &SNOWY_OWL], &WINGS);
}

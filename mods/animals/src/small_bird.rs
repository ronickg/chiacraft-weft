//! Small birds: the house sparrow (`HOUSE_SPARROW`) wherever people farm and build, and the snow
//! bunting (`SNOW_BUNTING`) of the tundra and the fells, two songbirds on one model
//! (`source/small_bird`), the bunting in its own colors by its genes. Birds on the ground and in
//! the air (`birds.rs`): they hop about in flocks after seeds and insects, and are up and away at a
//! walker or a hunter, a short dash low over the ground.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static HOUSE_SPARROW: Entity = Entity::named("house_sparrow");
pub(crate) static SNOW_BUNTING: Entity = Entity::named("snow_bunting");

static WINGS: Wings = Wings {
    model: "small_bird",
    texture: "small_bird/small_bird",
    fly_size: (0.25, 0.12),
    health: 2.0,
    // Hopping at about 1.5 m/s; it flies at about 10.
    speed: 1.5,
    air: 5.0,
    wander: 0.4,
    damage: 0.0,
    height: 2.0,
    reach: 8.0,
    flight: 200,
    restless: 1200,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&HOUSE_SPARROW, &SNOW_BUNTING], &WINGS);
}

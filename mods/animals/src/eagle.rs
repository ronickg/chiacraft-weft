//! Eagles: the golden eagle (`GOLDEN_EAGLE`) of the mountains, the prairie, the desert and the
//! north, and the savanna hawk (`SAVANNA_HAWK`) of the llanos and the Pantanal, two species on one
//! model (`source/eagle`), the hawk smaller and rufous by its genes. Birds on the ground and in the
//! air (`birds.rs`): hunters of hares, foxes and birds (`hunt.rs`), shy of walkers from far off;
//! they take off high and soar a long way on broad, fingered wings.

use weft_sdk::*;

use crate::birds::{self, Wings};

pub(crate) static GOLDEN_EAGLE: EntityKind = EntityKind::named("golden_eagle");
pub(crate) static SAVANNA_HAWK: EntityKind = EntityKind::named("savanna_hawk");

static WINGS: Wings = Wings {
    model: "eagle",
    texture: "eagle/eagle",
    fly_size: (1.2, 0.4),
    health: 16.0,
    // It walks little; it soars at about 14 m/s.
    speed: 1.5,
    air: 6.5,
    wander: 0.2,
    damage: 4.0,
    height: 10.0,
    reach: 28.0,
    flight: 700,
    restless: 3000,
};

pub fn init(r: &mut Registry) {
    birds::register(r, &[&GOLDEN_EAGLE, &SAVANNA_HAWK], &WINGS);
}

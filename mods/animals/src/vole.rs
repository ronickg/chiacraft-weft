//! Field voles (`notes/ECOLOGY.md`, wave one: the north's rodent): grass eaters of the taiga,
//! birch woods, tundra, mountains and marshes (`source/vole`), the cold biomes' owls' and foxes'
//! main food. A simple wild creature (`wild.rs`): it grazes, darts off, breeds all year (even under
//! the snow), and now and then sits up to nibble.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static FIELD_VOLE: Entity = Entity::named("field_vole");

static BODY: Body = Body {
    model: "vole",
    texture: "vole/vole",
    health: 2.0,
    // Busier by day than a mouse, darting along its runs through the grass.
    speed: 2.8,
    wander: 0.3,
    damage: 0.0,
    rest: ("vole_sit", (0.15, 0.2)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&FIELD_VOLE], &BODY);
}

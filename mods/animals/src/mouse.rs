//! Wood mice (`notes/ECOLOGY.md`, wave one: the rodent, the small hunters' main food): seed
//! eaters of fields, woods, the desert's scrub and the dunes (`source/mouse`). A simple wild
//! creature (`wild.rs`): it forages, darts off, breeds from spring to autumn, and now and then
//! sits up on its haunches to nibble.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static WOOD_MOUSE: Entity = Entity::named("wood_mouse");

static BODY: Body = Body {
    model: "mouse",
    texture: "mouse/mouse",
    size: (0.2, 0.15),
    health: 2.0,
    // Scurrying about, darting off at about 3 m/s.
    speed: 3.0,
    wander: 0.35,
    damage: 0.0,
    rest: ("mouse_sit", (0.15, 0.2)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&WOOD_MOUSE], &BODY);
}

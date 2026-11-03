//! Deer: the white-tailed deer (`WHITE_TAILED_DEER`) of the Americas' prairie, llanos, desert edge
//! and marshes and the red deer (`RED_DEER`) of Europe's forests and mountains, two species on one
//! model (`source/deer`), the red deer drawn bigger by its size genes. Simple wild creatures
//! (`wild.rs`) in herds: they browse and graze, bolt with the white tail raised, rut in autumn, and
//! now and then lie down to chew the cud. A buck's antlers are its build (`genetics/*deer.json`).

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static WHITE_TAILED_DEER: Entity = Entity::named("white_tailed_deer");
pub(crate) static RED_DEER: Entity = Entity::named("red_deer");

static BODY: Body = Body {
    model: "deer",
    texture: "deer/deer",
    size: (0.9, 1.3),
    health: 30.0,
    // Ambling while it feeds, bounding off at up to 60 km/h.
    speed: 5.0,
    wander: 0.25,
    damage: 0.0,
    rest: ("deer_lie", (0.9, 0.7)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[(&WHITE_TAILED_DEER, 11), (&RED_DEER, 12)], &BODY);
}

//! Hares: the brown hare (`HARE`) of open country and the mountain hare (`MOUNTAIN_HARE`) of the
//! north, two species on one model (`source/hare`), each its own coat from its genes. Simple wild
//! creatures (`wild.rs`): they graze and browse, bolt, breed in spring and summer, and now and
//! then settle into a form, pressed low with the ears laid flat.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static HARE: Entity = Entity::named("hare");
pub(crate) static MOUNTAIN_HARE: Entity = Entity::named("mountain_hare");

static BODY: Body = Body {
    model: "hare",
    texture: "hare/hare",
    size: (0.4, 0.45),
    health: 5.0,
    // Lolloping about slowly, flat out when it bolts: faster than a running player.
    speed: 4.0,
    wander: 0.3,
    damage: 0.0,
    rest: ("hare_lie", (0.4, 0.3)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&HARE, &MOUNTAIN_HARE], &BODY);
}

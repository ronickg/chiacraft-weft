//! Lizards (`notes/ECOLOGY.md`, wave one: the Cretaceous's lizard): Priscagama, an agama's kin of
//! the red dunes and the scrubland (`source/lizard`), an insect eater the raptors take. A simple
//! wild creature (`wild.rs`): it dashes off, lays its eggs in the wet season, and now and then lies
//! flat basking, a look of its own; standing about it bobs its head and pushes up on its front legs.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static PRISCAGAMA: EntityKind = EntityKind::named("priscagama");

static BODY: Body = Body {
    model: "lizard",
    texture: "lizard/lizard",
    // A dash of a few metres a second (est., as agamas').
    speed: 3.5,
    wander: 0.3,
    bites: false,
    rest: ("lizard_bask", (0.2, 0.07)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&PRISCAGAMA], &BODY);
}

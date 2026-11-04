//! Kryptobaatar (`notes/ECOLOGY.md`, wave one: the Cretaceous's small mammal): a multituberculate
//! of the red dunes and the scrubland (`source/kryptobaatar`), the raptors' small food. A simple
//! wild creature (`wild.rs`): it darts off, breeds in the wet season, and now and then sits up on
//! its haunches.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static KRYPTOBAATAR: Entity = Entity::named("kryptobaatar");

static BODY: Body = Body {
    model: "kryptobaatar",
    texture: "kryptobaatar/kryptobaatar",
    size: (0.2, 0.15),
    health: 2.0,
    // Scurrying, darting off at about 3 m/s (est., as a mouse's).
    speed: 3.0,
    wander: 0.35,
    damage: 0.0,
    rest: ("kryptobaatar_sit", (0.15, 0.2)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[(&KRYPTOBAATAR, 19)], &BODY);
}

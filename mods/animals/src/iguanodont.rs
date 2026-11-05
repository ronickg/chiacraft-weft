//! The iguanodont (`notes/ECOLOGY.md`, the Cretaceous's tier 1, starting as tier 2): Iguanacolossus,
//! the floodplain's and the fern forest's big plant eater, on all fours (`source/iguanodont`). A
//! simple wild creature (`wild.rs`) in herds for now: it browses, lumbers off from walkers and
//! hunters, lays its eggs in the wet season, and now and then lies down. The Utahraptor's prey
//! (the Stikes Quarry block); tamed and loaded as a beast of burden later.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static IGUANODONT: EntityKind = EntityKind::named("iguanodont");

static BODY: Body = Body {
    model: "iguanodont",
    texture: "iguanodont/iguanodont",
    health: 150.0,
    // A heavy trot on all fours at most (est.).
    speed: 4.5,
    wander: 0.25,
    damage: 0.0,
    rest: ("iguanodont_lie", (1.8, 1.5)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&IGUANODONT], &BODY);
}

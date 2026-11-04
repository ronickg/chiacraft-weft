//! Citipati (`notes/ECOLOGY.md`, the Cretaceous's tier 2): the oviraptorid of the Gobi's red dunes
//! and scrub, feathered, on two long legs (`source/citipati`). A simple wild creature (`wild.rs`):
//! it crops plants and takes lizards and Kryptobaatar of the sizes it takes (`hunt.rs`), runs from
//! walkers and raptors, lays its eggs in the wet season, and lies down as it broods, the wings spread.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static CITIPATI: Entity = Entity::named("citipati");

static BODY: Body = Body {
    model: "citipati",
    texture: "citipati/citipati",
    health: 40.0,
    // A long-legged runner, as an emu (est.).
    speed: 6.0,
    wander: 0.3,
    damage: 3.0,
    rest: ("citipati_lie", (1.2, 1.15)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&CITIPATI], &BODY);
}

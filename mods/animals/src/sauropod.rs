//! The sauropod (`notes/ECOLOGY.md`, the Cretaceous's tier 2): Cedarosaurus (`CEDAROSAURUS`), the
//! floodplain's high browser, a brachiosaurid about 15 m long, the neck raised high
//! (`source/sauropod`). A simple wild creature (`wild.rs`) in herds: it browses the conifers,
//! lumbers off from walkers, lays its eggs in the wet season, and now and then lies down. Grown, it
//! has no hunter; dead, it is a giant carcass for the scavengers.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static CEDAROSAURUS: EntityKind = EntityKind::named("cedarosaurus");

static BODY: Body = Body {
    model: "sauropod",
    texture: "sauropod/sauropod",
    // The body's box (its neck reaches far above it; a box to the head would stand on everything).
    // A slow amble at most (est.).
    speed: 4.0,
    wander: 0.2,
    bites: false,
    rest: ("sauropod_lie", (2.4, 2.6)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&CEDAROSAURUS], &BODY);
}

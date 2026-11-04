//! Protoceratops (`notes/ECOLOGY.md`, wave one: the Cretaceous's plant eater and the
//! Velociraptors' prey): the Djadokhta's small horned dinosaur of the red dunes and the scrubland
//! (`source/protoceratops`). A simple wild creature (`wild.rs`) in herds of mixed ages: it crops
//! ferns, runs from walkers and raptors, lays its eggs in a nest in the wet season, and now and
//! then lies down on its belly.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static PROTOCERATOPS: Entity = Entity::named("protoceratops");

static BODY: Body = Body {
    model: "protoceratops",
    texture: "protoceratops/protoceratops",
    size: (0.7, 0.75),
    health: 35.0,
    // A heavy trot, about 4 m/s at its fastest (est., from its build).
    speed: 4.0,
    wander: 0.3,
    damage: 0.0,
    rest: ("protoceratops_lie", (0.7, 0.45)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&PROTOCERATOPS], &BODY);
}

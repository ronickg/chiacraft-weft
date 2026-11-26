//! Reindeer (`REINDEER`): the tundra's herds, wintering into the taiga's edge and on the
//! mountains' fells, on a model of their own built on the deer's (`source/reindeer`). Simple wild
//! creatures (`wild.rs`) in herds: they graze lichen, sedges and leaves, gallop off from wolves and
//! walkers, rut in autumn and lie down to chew the cud. Both sexes carry antlers, the cows' smaller
//! (their build, `genetics/reindeer.json`).

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static REINDEER: EntityKind = EntityKind::named("reindeer");

static BODY: Body = Body {
    model: "reindeer",
    texture: "reindeer/reindeer",
    // Walking far while it feeds, galloping off at up to 60-80 km/h.
    speed: 5.0,
    wander: 0.3,
    bites: false,
    rest: ("reindeer_lie", (1.0, 0.8)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&REINDEER], &BODY);
}

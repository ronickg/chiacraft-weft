//! Coyotes (`COYOTE`): the prairie's and the desert's middle hunter, into the woods and the
//! mountains, on a model of their own built on the fox's (`source/coyote`). Simple wild creatures
//! (`wild.rs`) that hunt: hungry, a coyote goes after mice, voles, hares and birds of the sizes it
//! takes (`hunt.rs`), pouncing and biting (its attacks), and eats its kill; it runs from wolves,
//! breeds in midwinter, howls at the sky now and then (its idle), and lies down, the head on its
//! paws.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static COYOTE: EntityKind = EntityKind::named("coyote");

static BODY: Body = Body {
    model: "coyote",
    texture: "coyote/coyote",
    health: 18.0,
    // Trotting far, running at up to 65 km/h.
    speed: 5.0,
    wander: 0.3,
    damage: 3.0,
    rest: ("coyote_lie", (0.55, 0.45)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&COYOTE], &BODY);
}

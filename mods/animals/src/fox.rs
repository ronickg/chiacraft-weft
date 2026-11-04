//! Foxes: the red fox (`RED_FOX`) of the north's woods, fields and moors and the Arctic fox
//! (`ARCTIC_FOX`) of the tundra, two species on one model (`source/fox`), the Arctic fox drawn
//! smaller with shorter ears by its genes. Simple wild creatures (`wild.rs`) that hunt: hungry, a fox
//! goes after voles, mice, hares and birds of the sizes it takes (`hunt.rs`), pouncing and biting
//! (its attacks), and eats its kill; it runs from wolves and eagles, breeds in winter (the red) or
//! spring (the Arctic), and now and then lies down, the head on its paws.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static RED_FOX: Entity = Entity::named("red_fox");
pub(crate) static ARCTIC_FOX: Entity = Entity::named("arctic_fox");

static BODY: Body = Body {
    model: "fox",
    texture: "fox/fox",
    health: 10.0,
    // Trotting about, galloping after prey or away from a wolf.
    speed: 4.5,
    wander: 0.3,
    damage: 2.0,
    rest: ("fox_lie", (0.45, 0.35)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&RED_FOX, &ARCTIC_FOX], &BODY);
}

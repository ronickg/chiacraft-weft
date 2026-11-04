//! Caimans: the yacare caiman (`YACARE_CAIMAN`) of the Pantanal and the spectacled caiman
//! (`SPECTACLED_CAIMAN`) of the llanos, two species on one model (`source/caiman`), the spectacled
//! drawn smaller and paler by its genes. Simple wild creatures (`wild.rs`) that hunt: fish, snails,
//! water birds, rodents and the young of capybaras of the sizes they take (`hunt.rs`), biting (their
//! attack); cold-blooded, they lie gaping in the sun to cool and bask flat on the bank (a look of
//! its own), and lay their eggs in mounds in the rains.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static YACARE_CAIMAN: Entity = Entity::named("yacare_caiman");
pub(crate) static SPECTACLED_CAIMAN: Entity = Entity::named("spectacled_caiman");

static BODY: Body = Body {
    model: "caiman",
    texture: "caiman/caiman",
    size: (0.9, 0.45),
    health: 30.0,
    // A slow walk on land, a short rush at prey.
    speed: 3.0,
    wander: 0.15,
    damage: 5.0,
    rest: ("caiman_bask", (0.9, 0.3)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[(&YACARE_CAIMAN, 31), (&SPECTACLED_CAIMAN, 32)], &BODY);
}

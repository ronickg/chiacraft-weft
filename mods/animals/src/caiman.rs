//! Caimans: the yacare caiman (`YACARE_CAIMAN`) of the Pantanal and the spectacled caiman
//! (`SPECTACLED_CAIMAN`) of the llanos, two species on one model (`source/caiman`), the spectacled
//! drawn smaller and paler by its genes, and the Cretaceous's crocodyliform (`CROCODYLIFORM`), drawn
//! bigger and darker. Simple wild creatures (`wild.rs`) that hunt: fish, snails,
//! water birds, rodents and the young of capybaras of the sizes they take (`hunt.rs`), biting (their
//! attack); cold-blooded, they lie gaping in the sun to cool and bask flat on the bank (a look of
//! its own), and lay their eggs in mounds in the rains.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static YACARE_CAIMAN: EntityKind = EntityKind::named("yacare_caiman");
pub(crate) static SPECTACLED_CAIMAN: EntityKind = EntityKind::named("spectacled_caiman");
/// The Cretaceous's crocodyliform, on the same model drawn bigger by its genes.
pub(crate) static CROCODYLIFORM: EntityKind = EntityKind::named("crocodyliform");

static BODY: Body = Body {
    model: "caiman",
    texture: "caiman/caiman",
    // A slow walk on land, a short rush at prey.
    speed: 3.0,
    wander: 0.15,
    bites: true,
    rest: ("caiman_bask", (0.9, 0.3)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&YACARE_CAIMAN, &SPECTACLED_CAIMAN, &CROCODYLIFORM], &BODY);
}

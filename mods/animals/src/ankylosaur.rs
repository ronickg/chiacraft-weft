//! The ankylosaurs (`notes/ECOLOGY.md`, the Cretaceous's tier 2): Gastonia (`GASTONIA`), the
//! floodplain's commonest animal, and Pinacosaurus (`PINACOSAURUS`) of the Gobi's scrub, two
//! species on one armoured model in their own colors (`source/ankylosaur`). Simple wild creatures
//! (`wild.rs`) in herds: they crop low plants, lumber off from walkers and hunters, lay their eggs
//! in the wet season, and now and then lie down. Their armour makes a grown one hard prey: the
//! hunters take their young.

use weft_sdk::*;

use crate::wild::{self, Body};

pub(crate) static GASTONIA: EntityKind = EntityKind::named("gastonia");
pub(crate) static PINACOSAURUS: EntityKind = EntityKind::named("pinacosaurus");

static BODY: Body = Body {
    model: "ankylosaur",
    texture: "ankylosaur/ankylosaur",
    health: 120.0,
    // Slow and heavy: a walk, and a lumbering trot at most (est.).
    speed: 2.8,
    wander: 0.25,
    damage: 0.0,
    rest: ("ankylosaur_lie", (1.8, 0.95)),
};

pub fn init(r: &mut Registry) {
    wild::register(r, &[&GASTONIA, &PINACOSAURUS], &BODY);
}

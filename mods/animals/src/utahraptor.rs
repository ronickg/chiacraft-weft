//! Utahraptors: the biggest raptors, 1.5 m at the hip and nearly 5 m long, feathered, heavily
//! built ambush hunters, alone or in pairs on grassland and rarer than anything else here. They
//! wander, now and then lie down like birds, and sometimes go after a sheep, pig or cow; hurt one
//! and the others of its kind nearby join in. Tamed by knocking out (`knockout.rs`) and the mount
//! of this mod: saddled, it is ridden (`riding.rs`). A pose is a look (`World::set_look`), as the
//! chihuahua's. The model, its animations and texture are built from `source/utahraptor` (see
//! `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static UTAHRAPTOR: Entity = Entity::named("utahraptor");
const TEXTURE: &str = "utahraptor/utahraptor";
/// Its box: wide and tall enough for the body and the raised head.
const SIZE: (f32, f32) = (1.2, 2.4);

/// What it hunts.

pub fn init(r: &mut Registry) {
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &UTAHRAPTOR,
        crate::rest::Settle { chance: 400, ticks: (600, 1600), lie_after: 0 },
        "utahraptor",
        TEXTURE,
        (SIZE.0, SIZE.1),
        &[("_rest", "utahraptor_rest")],
    );
    r.entity(&UTAHRAPTOR)
        .size(SIZE.0, SIZE.1)
        .health(60.0)
        .speed(3.6)
        .attack_damage(10.0)
        .passive()
        .model_file("utahraptor")
        .animations("utahraptor")
        .texture(TEXTURE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player)
                && !crate::knockout::feed(&w, id, player)
                && !crate::riding::interact(&w, id, player)
            {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(4))
                .then(crate::riding::ridden(4))
                .then(crate::knockout::behave(4))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(4))
                .then(crate::breeding::behave(4))
                .then(crate::eggs::nest(4))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.5, 12.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            crate::rest::stop(w, h.entity);
        });
}

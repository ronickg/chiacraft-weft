//! Utahraptors: the biggest raptors, 1.5 m at the hip and nearly 5 m long, feathered, heavily
//! built ambush hunters, alone or in pairs on grassland and rarer than anything else here. They
//! wander, now and then lie down like birds, and sometimes go after a sheep, pig or cow; hurt one
//! and the others of its kind nearby join in. Tamed by knocking out (`knockout.rs`) and the mount
//! of this mod: saddled, it is ridden (`riding.rs`). A pose is a look (`World::set_look`), as the
//! chihuahua's. The model, its animations and texture are built from `source/utahraptor` (see
//! `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub(crate) static UTAHRAPTOR: EntityKind = EntityKind::named("utahraptor");
const TEXTURE: &str = "utahraptor/utahraptor";

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&UTAHRAPTOR);
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &UTAHRAPTOR,
        crate::rest::Settle { chance: 400, ticks: (600, 1600), lie_after: 0 },
        "utahraptor",
        TEXTURE,
        animals_core::bodies::of("utahraptor"),
        &[("_rest", "utahraptor_rest")],
    );
    crate::riding::rideable(r.entity(&UTAHRAPTOR), index, 3.6)
        .size(animals_core::bodies::of("utahraptor").0, animals_core::bodies::of("utahraptor").1)
        .max_health(60.0)
        .walk_speed(3.6)
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
            false
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(index))
                .then(crate::riding::ridden(index))
                .then(crate::knockout::behave(index))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::eggs::nest(index))
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

//! Jaguars (`JAGUAR`): the top hunter of the Pantanal's marshes and the llanos, a full creature on
//! its own model built on the cat's (`source/jaguar`). Wild, it hunts capybaras, caimans, deer and
//! fish of the sizes it takes (`hunt.rs`), with a swipe and a bite (its attacks), calls now and then
//! with its sawing roar, and settles down: it sits, then lies down. Too dangerous to feed from the
//! hand, it is tamed as the raptors are: knocked out and fed while it sleeps (`knockout.rs`); tamed,
//! it follows, sits and fights on command (`commands.rs`). A pose is a look (`World::set_look`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static JAGUAR: Entity = Entity::named("jaguar");
const INDEX: usize = 34;
const TEXTURE: &str = "jaguar/jaguar";

pub fn init(r: &mut Registry) {
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &JAGUAR,
        crate::rest::Settle { chance: 400, ticks: (400, 1600), lie_after: 600 },
        "jaguar",
        TEXTURE,
        (0.8, 0.8),
        &[("_sit", "jaguar_sit"), ("_lie", "jaguar_lie")],
    );
    r.entity(&JAGUAR)
        .size(0.8, 0.85)
        .health(45.0)
        .speed(5.0)
        .attack_damage(9.0)
        .passive()
        .model_file("jaguar")
        .animations("jaguar")
        .texture(TEXTURE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::knockout::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(INDEX))
                .then(crate::knockout::behave(INDEX))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(INDEX))
                .then(crate::breeding::behave(INDEX))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.4, 10.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            crate::rest::stop(w, h.entity);
        });
}

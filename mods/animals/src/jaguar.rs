//! Jaguars (`JAGUAR`): the top hunter of the Pantanal's marshes and the llanos, a full creature on
//! its own model built on the cat's (`source/jaguar`). Wild, it hunts capybaras, caimans, deer and
//! fish of the sizes it takes (`hunt.rs`), with a swipe and a bite (its attacks), calls now and then
//! with its sawing roar, and settles down: it sits, then lies down. Too dangerous to feed from the
//! hand, it is tamed as the raptors are: knocked out and fed while it sleeps (`knockout.rs`); tamed,
//! it follows, sits and fights on command (`commands.rs`). A pose is a look (`World::set_look`).

use crate::choose::{Band, Need, Stops, WithBand};
use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub(crate) static JAGUAR: EntityKind = EntityKind::named("jaguar");
const TEXTURE: &str = "jaguar/jaguar";

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&JAGUAR);
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
        .size(animals_core::bodies::of("jaguar").0, animals_core::bodies::of("jaguar").1)
        .max_health(crate::animal::health(index))
        .walk_speed(5.0)
        .attack_damage(crate::animal::hit(index))
        .passive()
        .model_file("jaguar")
        .animations("jaguar")
        .texture(TEXTURE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::knockout::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
            false
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(index))
                .then(crate::knockout::behave(index))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(index))
                // What it does when nothing is after it or ordered, the most pressing first (`choose.rs`).
                .band(
                    "needs",
                    Band::new()
                        .with(Need::Love, crate::breeding::behave(index))
                        .with(Need::Rest, Stops { goal: crate::rest::settle, stop: crate::rest::stop })
                        .with(Need::Hold, crate::commands::hold)
                        .with(Need::Comfort, crate::comfort::seek)
                        .with(Need::Drink, crate::drink::drink)
                        .with(Need::Eat, crate::forage::eat)
                        .with(Need::Wander, ai::wander(0.4, 10.0)),
                ),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity, h.source, h.cause);
            crate::rest::stop(w, h.entity);
        });
}

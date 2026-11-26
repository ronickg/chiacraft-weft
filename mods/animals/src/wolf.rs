//! Wolves: the grey wolf (`WOLF`) of the taiga, the forests, the mountains and the tundra, a full
//! creature on its own model (`source/wolf`), and the dog's ancestor, so a natural tame: fed from
//! the hand it comes round (`taming.rs`), then follows, sits and fights on command
//! (`commands.rs`). Wild, it hunts hares, deer and reindeer of the sizes it takes (`hunt.rs`),
//! pouncing and biting (its attacks), howls now and then (its idle), and settles down: it sits,
//! then after a while lies down, the head on its paws. A pose is a look (`World::set_look`), as the
//! cat's.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;
use crate::choose::{Band, Need, Stops, WithBand};
use crate::prof::Timed;

pub(crate) static WOLF: EntityKind = EntityKind::named("grey_wolf");
const TEXTURE: &str = "wolf/wolf";

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&WOLF);
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &WOLF,
        crate::rest::Settle { chance: 400, ticks: (400, 1600), lie_after: 600 },
        "wolf",
        TEXTURE,
        (0.6, 0.8),
        &[("_sit", "wolf_sit"), ("_lie", "wolf_lie")],
    );
    r.entity(&WOLF)
        .size(animals_core::bodies::of("wolf").0, animals_core::bodies::of("wolf").1)
        .max_health(crate::animal::health(index))
        .walk_speed(5.5)
        .attack_damage(crate::animal::hit(index))
        .passive()
        .model_file("wolf")
        .animations("wolf")
        .texture(TEXTURE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::taming::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
            false
        })
        .goals(
            Goals::new()
                .timed("upkeep", crate::animal::upkeep(index))
                .timed("howl", crate::voices::howl_at_blood_moon)
                .timed("taming", crate::taming::behave(index))
                .timed("wary", crate::hunt::wary)
                .timed("hunt", crate::hunt::hunt)
                .timed("commands", crate::commands::act(index))
                // What it does when nothing is after it or ordered, the most pressing first
                // (`choose.rs`).
                .band(
                    "needs",
                    Band::new()
                        .with(Need::Love, crate::breeding::behave(index))
                        .with(Need::Rest, Stops { goal: crate::rest::settle, stop: crate::rest::stop })
                        .with(Need::Hold, crate::commands::hold)
                        .with(Need::Comfort, crate::comfort::seek)
                        .with(Need::Drink, crate::drink::drink)
                        .with(Need::Eat, crate::forage::eat)
                        .with(Need::Wander, crate::group::or_follow(0.4, ai::wander(0.4, 10.0))),
                ),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity, h.source, h.cause);
            crate::rest::stop(w, h.entity);
        });
}

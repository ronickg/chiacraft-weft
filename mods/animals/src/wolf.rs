//! Wolves: the grey wolf (`WOLF`) of the taiga, the forests, the mountains and the tundra, a full
//! creature on its own model (`source/wolf`), and the dog's ancestor, so a natural tame: fed from
//! the hand it comes round (`taming.rs`), then follows, sits and fights on command
//! (`commands.rs`). Wild, it hunts hares, deer and reindeer of the sizes it takes (`hunt.rs`),
//! pouncing and biting (its attacks), howls now and then (its idle), and settles down: it sits,
//! then after a while lies down, the head on its paws. A pose is a look (`World::set_look`), as the
//! cat's.

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static WOLF: Entity = Entity::named("grey_wolf");
const INDEX: usize = 33;
const TEXTURE: &str = "wolf/wolf";

pub fn init(r: &mut Registry) {
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
        .size(0.6, 0.85)
        .health(30.0)
        .speed(5.5)
        .attack_damage(6.0)
        .passive()
        .model_file("wolf")
        .animations("wolf")
        .texture(TEXTURE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::taming::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(INDEX))
                .then(crate::taming::behave(INDEX))
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

//! Cats: orange tabbies on grassland that wander about and now and then settle down: they sit,
//! and after half a minute curl up into a loaf, then get up and go on. A pose is a look
//! (`World::set_look`), as the chihuahua's, so every player sees it. The model, its animations
//! and texture are built from `source/cat` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub(crate) static CAT: EntityKind = EntityKind::named("cat");
const TEXTURE: &str = "cat/cat";

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&CAT);
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &CAT,
        crate::rest::Settle { chance: 300, ticks: (400, 1200), lie_after: 600 },
        "cat",
        TEXTURE,
        (0.35, 0.44),
        &[("_sit", "cat_sit"), ("_lie", "cat_lie")],
    );
    r.entity(&CAT)
        .size(animals_core::bodies::of("cat").0, animals_core::bodies::of("cat").1)
        .max_health(6.0)
        .walk_speed(3.0)
        .attack_damage(1.0)
        .passive()
        .model_file("cat")
        .animations("cat")
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
                .then(crate::animal::upkeep(index))
                .then(crate::taming::behave(index))
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::signatures::gifts)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.5, 7.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity, h.source);
            crate::rest::stop(w, h.entity);
        });
}

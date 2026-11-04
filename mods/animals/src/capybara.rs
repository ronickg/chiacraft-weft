//! Capybaras: the biggest rodents, calm and always chewing, in small groups on grassland. They
//! wander slowly, graze (their idle puts the head down to the grass now and then), and settle
//! down: they sit, and after half a minute flop onto their bellies with the head held up. A pose
//! is a look (`World::set_look`), as the chihuahua's. The model, its animations and texture are
//! built from `source/capybara` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub(crate) static CAPYBARA: EntityKind = EntityKind::named("capybara");
const TEXTURE: &str = "capybara/capybara";

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&CAPYBARA);
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &CAPYBARA,
        crate::rest::Settle { chance: 300, ticks: (400, 1200), lie_after: 600 },
        "capybara",
        TEXTURE,
        (0.7, 0.6),
        &[("_sit", "capybara_sit"), ("_lie", "capybara_lie")],
    );
    r.entity(&CAPYBARA)
        .size(animals_core::bodies::of("capybara").0, animals_core::bodies::of("capybara").1)
        .max_health(12.0)
        .walk_speed(2.5)
        .passive()
        .model_file("capybara")
        .animations("capybara")
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
                .then(crate::signatures::calm_aura)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.35, 6.0)), // slow and calm
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            crate::rest::stop(w, h.entity);
        });
}

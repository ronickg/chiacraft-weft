//! Wyverns: a fantasy animal, dragons with two legs and bat wings for arms (as the dragons of
//! Game of Thrones), about 3.9 m at the shoulder and 16 m long, rare and alone, most in the mountains.
//! On the ground they go on four, on their hind feet and the wrists of their folded wings. Wild
//! ones are dangerous: one goes for any player who comes near (nearer its nest, sooner), and
//! hurt, it fights back, breathing at its foe from a distance; now and then one hunts a sheep, pig
//! or cow. What one breathes (fire, frost, lightning or venom) is in its genes
//! (`coats::breath`), inherited like a coat. Tamed by knocking out (`knockout.rs`), and ridden
//! saddled (`riding.rs`). Lying down (resting, asleep, knocked out) is a look
//! (`World::set_look`), as the Utahraptor's. The model, its animations and texture are built from
//! `source/wyvern` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub(crate) static WYVERN: EntityKind = EntityKind::named("wyvern");
const TEXTURE: &str = "wyvern/wyvern";
/// How near a player can come before a wild one goes for them, in blocks; near its nest, further.
const TERRITORY: f64 = 14.0;
const NEST_TERRITORY: f64 = 24.0;

pub fn init(r: &mut Registry) {
    let index = crate::species::of(&WYVERN);
    // Settling down: its poses, drawn in place of it (`rest.rs`).
    crate::rest::register(
        r,
        &WYVERN,
        crate::rest::Settle { chance: 500, ticks: (800, 2400), lie_after: 0 },
        "wyvern",
        TEXTURE,
        (animals_core::bodies::of("wyvern").0, animals_core::bodies::of("wyvern").1 * 0.5),
        &[("_rest", "wyvern_rest")],
    );
    crate::riding::rideable(r.entity(&WYVERN), index)
        .size(animals_core::bodies::of("wyvern").0, animals_core::bodies::of("wyvern").1)
        .max_health(400.0)
        .walk_speed(5.0)
        .attack_damage(30.0)
        .attack_speed(0.8)
        .passive()
        .model_file("wyvern")
        .animations("wyvern")
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
                .then(crate::breath::behave)
                .then(crate::riding::ridden(index))
                .then(breathe_at_foe)
                .then(crate::knockout::behave(index))
                .then(guard)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::eggs::nest(index))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.4, 20.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            crate::rest::stop(w, h.entity);
        });
}

/// A goal for wild wyverns: a calm one goes for the nearest player within its territory (wider
/// near a nest), and ends its rest to do it.
fn guard(w: &World, id: u64, st: &EntityState) -> bool {
    if crate::animal::tamed(w, id) || w.random_below(10) != 0 {
        return false;
    }
    let calm = w
        .component(id, &crate::knockout::KNOCKOUT)
        .is_some_and(|k| k.out == 0 && k.angry == 0);
    if !calm {
        return false;
    }
    let nest = !w
        .entities_near(st.pos, NEST_TERRITORY, Some(&crate::eggs::NEST))
        .is_empty();
    if let Some((p, _)) = ai::nearest_player(w, st, if nest { NEST_TERRITORY } else { TERRITORY }) {
        weft_sdk::info!("wyvern {id} goes for player {p}");
        crate::rest::wake(w, id);
        crate::knockout::anger(w, id, p);
    }
    false
}

/// A goal for wild wyverns: angry at something 5 to 16 blocks off, now and then it breathes at
/// it (what its genes give it: `breath.rs`).
fn breathe_at_foe(w: &World, id: u64, st: &EntityState) -> bool {
    if crate::animal::tamed(w, id) || w.random_below(30) != 0 {
        return false;
    }
    let Some(foe) = w
        .component(id, &crate::knockout::KNOCKOUT)
        .filter(|k| k.out == 0 && k.angry > 0)
        .map(|k| k.angry_at)
    else {
        return false;
    };
    let Some(f) = w.entity(foe) else { return false };
    let d2 = (f.pos.x - st.pos.x).powi(2) + (f.pos.z - st.pos.z).powi(2);
    if (25.0..256.0).contains(&d2) {
        crate::breath::start(w, id, foe);
    }
    false
}

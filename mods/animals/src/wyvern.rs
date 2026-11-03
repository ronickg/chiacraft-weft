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

pub(crate) static WYVERN: Entity = Entity::named("wyvern");
const TEXTURE: &str = "wyvern/wyvern";
/// Its box: about its body (the neck and tail reach well out of it), as tall as its shoulders.
const SIZE: (f32, f32) = (3.4, 4.0);
/// Its row in `species::SPECIES`.
const INDEX: usize = 6;

value! {
    #[derive(Default)]
    /// What a wyvern is doing (not saved: it just settles down again).
    pub struct Wyvern {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Wyvern> = Component::named("wyvern_state");

const LOOKS: [(&str, &str); 2] = [("", ""), ("_rest", "wyvern_rest")];
/// How near a player can come before a wild one goes for them, in blocks; near its nest, further.
const TERRITORY: f64 = 14.0;
const NEST_TERRITORY: f64 = 24.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Wyvern::default(), 0);
    // Lying down, drawn in place of one resting.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(SIZE.0, SIZE.1 * 0.5)
                .passive()
                .model_file("wyvern")
                .animations(&format!("wyvern{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&WYVERN)
        .size(SIZE.0, SIZE.1)
        .health(400.0)
        .speed(5.0)
        .attack_damage(30.0)
        .attack_speed(0.8)
        .passive()
        .model_file("wyvern")
        .animations("wyvern")
        .texture(TEXTURE)
        .attach(&STATE)
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
                .then(crate::animal::upkeep(INDEX))
                .then(crate::breath::behave)
                .then(crate::riding::ridden(INDEX))
                .then(breathe_at_foe)
                .then(crate::knockout::behave(INDEX))
                .then(guard)
                .then(crate::utahraptor::hunt)
                .then(crate::commands::act(INDEX))
                .then(crate::breeding::behave(INDEX))
                .then(crate::eggs::nest(INDEX))
                .then(settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.4, 20.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
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
        .near(st.pos, NEST_TERRITORY, Some(&crate::eggs::NEST))
        .is_empty();
    if let Some((p, _)) = ai::nearest_player(w, st, if nest { NEST_TERRITORY } else { TERRITORY }) {
        weft_sdk::info!("wyvern {id} goes for player {p}");
        wake(w, id);
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
    let d2 = (f.pos[0] - st.pos[0]).powi(2) + (f.pos[2] - st.pos[2]).powi(2);
    if (25.0..256.0).contains(&d2) {
        crate::breath::start(w, id, foe);
    }
    false
}

/// Every tick: now and then lie down to rest for a while, and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(500) == 0 {
        c.rest = 800 + w.random_below(2400);
    }
    let resting = c.rest > 0;
    if resting {
        c.rest -= 1;
    }
    let pose = resting as u32;
    if c.look != pose + 1 {
        c.look = pose + 1;
        let kind = LOOKS[pose as usize].1;
        w.set_look(
            id,
            (!kind.is_empty())
                .then(|| format!("animals:{kind}"))
                .as_deref(),
        );
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

/// Up from resting, to follow, fight or carry a rider.
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.look > 1)
    {
        c.rest = 0;
        c.look = 1;
        w.set_look(id, None);
    }
}

/// Knocked out: lying down.
pub fn sleep(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && c.look != 2
    {
        c.rest = 0;
        c.look = 2;
        w.set_look(id, Some("animals:wyvern_rest"));
    }
}

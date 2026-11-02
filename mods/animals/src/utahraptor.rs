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

value! {
    #[derive(Default)]
    /// What a Utahraptor is doing (not saved: it just settles down again).
    pub struct Utahraptor {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Utahraptor> = Component::named("utahraptor_state");

const LOOKS: [(&str, &str); 2] = [("", ""), ("_rest", "utahraptor_rest")];
/// What it hunts.
static PREY: [Entity; 3] = [Entity::named("base:sheep"), Entity::named("base:pig"), Entity::named("base:cow")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Utahraptor::default(), 0);
    // The resting pose, drawn in place of one lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(SIZE.0, SIZE.1)
                .passive()
                .model_file("utahraptor")
                .animations(&format!("utahraptor{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&UTAHRAPTOR)
        .size(SIZE.0, SIZE.1)
        .health(60.0)
        .speed(3.6)
        .attack_damage(10.0)
        .passive()
        .model_file("utahraptor")
        .animations("utahraptor")
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
                .then(crate::animal::upkeep(4))
                .then(crate::riding::ridden(4))
                .then(crate::knockout::behave(4))
                .then(hunt)
                .then(crate::commands::act(4))
                .then(crate::breeding::behave(4))
                .then(crate::eggs::nest(4))
                .then(settle)
                .then(crate::commands::hold)
                .then(ai::wander(0.5, 12.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
}

/// Every tick: now and then lie down to rest for a while, and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && w.random_below(400) == 0 {
        c.rest = 600 + w.random_below(1600);
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

/// Knocked out: lying as it rests.
pub fn sleep(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && c.look != 2
    {
        c.rest = 0;
        c.look = 2;
        w.set_look(id, Some("animals:utahraptor_rest"));
    }
}

/// A goal for wild Utahraptors (and wyverns): now and then (about every ten minutes) one goes
/// after a sheep, pig or cow nearby, and the others of its kind near it join in.
pub(crate) fn hunt(w: &World, id: u64, st: &EntityState) -> bool {
    if crate::animal::tamed(w, id) || w.random_below(12_000) != 0 {
        return false;
    }
    let calm = w
        .component(id, &crate::knockout::KNOCKOUT)
        .is_some_and(|k| k.out == 0 && k.angry == 0);
    if !calm {
        return false;
    }
    let prey = PREY
        .iter()
        .flat_map(|p| w.near(st.pos, 24.0, Some(p)))
        .next();
    if let Some(p) = prey {
        weft_sdk::info!("{id} hunts {p}");
        crate::knockout::anger(w, id, p);
    }
    false
}

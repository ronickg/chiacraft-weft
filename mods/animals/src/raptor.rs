//! Velociraptors: feathered, turkey-sized hunters (the real animal, not the film's), in small
//! groups on grassland, rarer than cats. Two species, as paleontologists name them: Velociraptor
//! mongoliensis (`RAPTOR`) on grassland, and V. osmolskae (`OSMOLSKAE`), paler, on desert sand
//! (it lived among dunes); the same body and behavior, two species that don't breed together.
//! They wander and now and then lie down to rest like birds, legs folded under; a pack now and then hunts a chicken, sheep or pig together, and
//! comes to the aid of any of them that is hurt. Tamed by knocking out (`knockout.rs`); tamed
//! ones near each other fight better. Too small to ride (the Utahraptor is the mount). A pose is a look (`World::set_look`), as the chihuahua's. The model, its animations and texture are built from `source/raptor` (see
//! `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static RAPTOR: Entity = Entity::named("raptor");
pub(crate) static OSMOLSKAE: Entity = Entity::named("velociraptor_osmolskae");
/// Each species: its kind, its row in `species::SPECIES`, its texture.
static KINDS: [(&Entity, usize, &str); 2] = [(&RAPTOR, 3, "raptor/raptor"), (&OSMOLSKAE, 5, "raptor/osmolskae")];

value! {
    #[derive(Default)]
    /// What a raptor is doing (not saved: it just settles down again).
    pub struct Raptor {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Raptor> = Component::named("raptor_state");

/// How near a pack keeps together: helping, hunting, fighting better.
const PACK_RANGE: f64 = 8.0;
const AID_RANGE: f64 = 16.0;
/// What a pack hunts.
static PREY: [Entity; 3] = [Entity::named("base:chicken"), Entity::named("base:sheep"), Entity::named("base:pig")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Raptor::default(), 0);
    // Tamed raptors fight better together: a tenth more damage for each other tame raptor of
    // its species within 8 blocks, up to three.
    r.modify(&chain::ATTACK_DAMAGE, stage::MULTIPLY, |m| {
        let (w, id) = (m.world(), m.query().entity);
        if !is_tame_raptor(&w, id) {
            return;
        }
        let Some(st) = w.entity(id) else { return };
        let pack = w
            .near(st.pos, PACK_RANGE, kind_of(&w, id))
            .into_iter()
            .filter(|&o| o != id && is_tame_raptor(&w, o))
            .count()
            .min(3);
        m.value *= 1.0 + 0.1 * pack as f64;
    });
    for (kind, index, texture) in KINDS {
        register(r, kind, index, texture);
    }
    // Small packs now and then, rarer than cats: mongoliensis on grassland, osmolskae on sand.
    r.spawn(&RAPTOR)
        .biome("#animals")
        .ground("#animal_spawn")
        .light(7, 15)
        .group(2, 3)
        .chance(0.008);
    r.spawn(&OSMOLSKAE)
        .biome("#dry")
        .ground("#sand")
        .light(7, 15)
        .group(2, 3)
        .chance(0.008);
}

/// One species: the kind, and its resting pose (a look) drawn in place of it lying down.
fn register(r: &mut Registry, kind: &'static Entity, index: usize, texture: &str) {
    drop(
        r.entity(Entity::generated(rest_look(kind)))
            .size(0.6, 0.9)
            .passive()
            .model_file("raptor")
            .animations("raptor_rest")
            .texture(texture),
    );
    r.entity(kind)
        .size(0.6, 0.9)
        .health(14.0)
        .speed(4.0)
        .attack_damage(4.0)
        .passive()
        .model_file("raptor")
        .animations("raptor")
        .texture(texture)
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
                .then(crate::animal::upkeep(index))
                .then(crate::riding::ridden(index))
                .then(crate::knockout::behave(index))
                .then(hunt)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::eggs::nest(index))
                .then(settle)
                .then(crate::commands::hold)
                .then(ai::wander(0.6, 9.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
}

/// A species' resting look (`raptor_rest`, `velociraptor_osmolskae_rest`).
fn rest_look(kind: &Entity) -> String {
    format!("{}_rest", kind.path())
}

/// The kind of raptor `id` is.
fn kind_of(w: &World, id: u64) -> Option<&'static Entity> {
    let k = w.entity(id)?.kind;
    KINDS.iter().map(|(e, _, _)| *e).find(|e| e.id() == k)
}

/// Every tick: now and then lie down to rest for a while, and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && w.random_below(300) == 0 {
        c.rest = 400 + w.random_below(1200);
        c.rested = 0;
    }
    let resting = c.rest > 0;
    if resting {
        c.rest -= 1;
        c.rested += 1;
    }
    let pose = resting as u32;
    if c.look != pose + 1 {
        c.look = pose + 1;
        let look = kind_of(w, id)
            .filter(|_| resting)
            .map(|k| format!("animals:{}", rest_look(k)));
        w.set_look(id, look.as_deref());
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

/// Up from resting, to follow or fight.
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.look > 1)
    {
        c.rest = 0;
        c.look = 1;
        w.set_look(id, None);
    }
}

fn is_tame_raptor(w: &World, id: u64) -> bool {
    kind_of(w, id).is_some() && crate::animal::tamed(w, id)
}

/// Knocked out: lying as it rests.
pub fn sleep(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && c.look != 2
    {
        c.rest = 0;
        c.look = 2;
        let look = kind_of(w, id).map(|k| format!("animals:{}", rest_look(k)));
        w.set_look(id, look.as_deref());
    }
}

/// `id` (wild) was hurt by `at`, or goes after it: the wild ones of its kind near it join in
/// (velociraptors and Utahraptors alike).
pub fn aid(w: &World, id: u64, at: u64) {
    if crate::animal::tamed(w, id) {
        return;
    }
    let Some(st) = w.entity(id) else { return };
    for other in w.near(st.pos, AID_RANGE, None) {
        if other == id || crate::animal::tamed(w, other) || w.entity(other).is_none_or(|e| e.kind != st.kind) {
            continue;
        }
        if let Some(k) = w.component_mut(other, &crate::knockout::KNOCKOUT)
            && k.out == 0
            && k.angry == 0
        {
            k.angry = 600;
            k.angry_at = at;
        }
    }
}

/// A goal for wild raptors: now and then (about every five minutes) a pack goes after a
/// chicken, sheep or pig nearby.
fn hunt(w: &World, id: u64, st: &EntityState) -> bool {
    if crate::animal::tamed(w, id) || w.random_below(6000) != 0 {
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
        .flat_map(|p| w.near(st.pos, AID_RANGE, Some(p)))
        .next();
    if let Some(p) = prey {
        weft_sdk::info!("raptor {id} hunts {p}");
        crate::knockout::anger(w, id, p);
    }
    false
}

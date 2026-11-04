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
/// Each species: its kind and its texture.
static KINDS: [(&Entity, &str); 2] = [(&RAPTOR, "raptor/raptor"), (&OSMOLSKAE, "raptor/osmolskae")];

/// How near a pack keeps together: helping, hunting, fighting better.
const PACK_RANGE: f64 = 8.0;
const AID_RANGE: f64 = 16.0;
/// What a pack hunts.

pub fn init(r: &mut Registry) {
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
    for (kind, texture) in KINDS {
        register(r, kind, crate::species::of(kind), texture);
    }
}

/// One species: the kind, and its resting pose (a look) drawn in place of it lying down
/// (`rest.rs`).
fn register(r: &mut Registry, kind: &'static Entity, index: usize, texture: &str) {
    let settle = crate::rest::Settle { chance: 300, ticks: (400, 1200), lie_after: 0 };
    crate::rest::register(r, kind, settle, "raptor", texture, (0.6, 0.9), &[("_rest", &rest_look(kind))]);
    r.entity(kind)
        .size(animals_core::bodies::of("raptor").0, animals_core::bodies::of("raptor").1)
        .health(14.0)
        .speed(4.0)
        .attack_damage(4.0)
        .passive()
        .model_file("raptor")
        .animations("raptor")
        .texture(texture)
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
                .then(crate::hunt::wary)
                .then(crate::hunt::hunt)
                .then(crate::commands::act(index))
                .then(crate::breeding::behave(index))
                .then(crate::eggs::nest(index))
                .then(crate::rest::settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.6, 9.0)),
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            crate::rest::stop(w, h.entity);
        });
}

/// A species' resting look (`raptor_rest`, `velociraptor_osmolskae_rest`).
fn rest_look(kind: &Entity) -> String {
    format!("{}_rest", kind.path())
}

/// The kind of raptor `id` is.
fn kind_of(w: &World, id: u64) -> Option<&'static Entity> {
    let k = w.entity(id)?.kind;
    KINDS.iter().map(|(e, _)| *e).find(|e| e.id() == k)
}

fn is_tame_raptor(w: &World, id: u64) -> bool {
    kind_of(w, id).is_some() && crate::animal::tamed(w, id)
}

/// `id` (wild) was hurt by `at`, or goes after it: the wild ones of its kind near it join in
/// (velociraptors and Utahraptors alike).
pub fn aid(w: &World, id: u64, at: u64) {
    // Only a pack's kind (Utahraptors); a lone hunter's kin don't join in.
    let pack = w
        .component(id, &crate::animal::ANIMAL)
        .and_then(|a| a.species())
        .is_some_and(|s| s.bio.social == crate::species::Social::Pack);
    if crate::animal::tamed(w, id) || !pack {
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

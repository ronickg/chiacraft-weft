//! A dead animal's body (`animals_core::carcass`, `notes/crew/plans/carcass-and-skinning.md`, slice 1). The engine
//! removes a creature the tick it dies, so its body is read at the `DIE` question (before removal: its species,
//! mass, where it faces) and a carcass entity is laid where it fell. Its look steps down as it is eaten and as it rots:
//! whole and opened drawn as the animal itself lying dead (its death clip held), then a ribcage, then a pile of bones
//! (placeholders until the blocky-model looks per body plan), then it is gone.
//!
//! Who takes from it: the killer and its pack (`hunt.rs`: their meal's kilograms, `taken`), scavengers (`forage.rs`:
//! a bite at a time, smelled from afar as carrion is), and later a player's knife. A player's kill still drops its
//! loot as before, so its carcass starts opened (skinned, as RDR2's).

use std::sync::Mutex;

use animals_core::carcass::{self as core, Look};
use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

use crate::species::SPECIES;

/// The body: its kind (`EntityKind`); the looks it is drawn as past opened.
pub static CARCASS: EntityKind = EntityKind::named("carcass_body");
static BONES: EntityKind = EntityKind::named("carcass_bones");
static PILE: EntityKind = EntityKind::named("carcass_pile");

value! {
    #[derive(Default)]
    /// What a carcass was and what's left; saved.
    pub struct Body {
        /// The species + 1.
        pub species: u32,
        /// Its live mass, kg, and the kg eaten off it.
        pub mass: f32,
        pub eaten: f32,
        /// Days aged at larder pace (`vitals::rot`), and the tick it was last aged at.
        pub aged: f32,
        pub at: u64,
        /// The scale the animal was drawn at (its genes and growth).
        pub scale: f32,
        /// Skinned (1) or not.
        pub skinned: u32,
        /// The look it is drawn as now, + 1 (0: not set since it loaded).
        pub look: u32,
        /// The animal it was.
        pub of: u64,
    }
}
pub static BODY: Component<Body> = Component::named("carcass");
/// The look it is drawn as on this run, + 1 (not saved: the engine forgets appearance and scale on loading).
static SHOWN: Component<u32> = Component::named("carcass_shown");

/// The dying read at the `DIE` question: the animal, its species index, mass, yaw, drawn scale.
type Dying = (u64, usize, f32, f32, f32);
static DYING: Mutex<Vec<Dying>> = Mutex::new(Vec::new());
/// Kilograms taken off a body before its carcass was laid (the hunters' `DIE` handler may run first).
static TAKEN: Mutex<Vec<(u64, f32)>> = Mutex::new(Vec::new());

/// How often a carcass ages and looks again, ticks.
const EVERY: u64 = 100;

pub fn init(r: &mut Registry) {
    r.entity_component(&BODY, Body::default(), component_flags::SAVE);
    r.entity_component(&SHOWN, 0u32, 0);
    let (bw, bh) = animals_core::bodies::of("carcass_bones");
    r.entity(&CARCASS)
        .no_egg()
        .size(bw, bh)
        .max_health(20.0)
        .walk_speed(0.0)
        .passive()
        .unpushable()
        .model_file("carcass_bones")
        .texture("carcass/bones")
        .attach(&BODY)
        .attach(&SHOWN)
        .goals(Goals::new().then(rot));
    for (kind, model) in [(&BONES, "carcass_bones"), (&PILE, "carcass_pile")] {
        let (w, h) = animals_core::bodies::of(model);
        r.entity(kind)
            .no_egg()
            .size(w, h)
            .walk_speed(0.0)
            .passive()
            .unpushable()
            .model_file(model)
            .texture("carcass/bones");
    }
    // Read before the body goes (`DIE`'s question comes while it is still there).
    r.before_event(&actions::DIE, |q| {
        let w = q.world();
        let id = q.entity();
        if let (Some(a), Some(e)) = (w.component(id, &crate::animal::ANIMAL), w.entity(id))
            && let Some(index) = (a.species as usize)
                .checked_sub(1)
                .filter(|&i| i < SPECIES.len())
            && let Some(mass) = crate::hunt::mass(&w, id)
        {
            let scale = crate::coats::size_of(&w, id).unwrap_or(1.0) * crate::breeding::growth(&w, id, index);
            let mut d = DYING.lock().unwrap();
            d.retain(|r| r.0 != id);
            d.push((id, index, mass, e.yaw, scale));
            if d.len() > 64 {
                d.remove(0);
            }
        }
        Verdict::Abstain
    });
    r.on_event(&actions::DIE, |b| {
        let w = b.world();
        for d in b.iter() {
            let read = {
                let mut dying = DYING.lock().unwrap();
                let i = dying.iter().position(|r| r.0 == d.entity);
                i.map(|i| dying.swap_remove(i))
            };
            // Died some other way (a command, the void): its kind's own size, facing north.
            let Some((_, index, mass, yaw, scale)) = read.or_else(|| {
                let index = SPECIES.iter().position(|s| s.kind.id() == d.kind)?;
                Some((d.entity, index, SPECIES[index].bio.mass, 0.0, 1.0))
            }) else {
                continue;
            };
            if mass < core::FROM_KG || crate::hunt::egg_of(&w, d.entity).is_some() {
                continue;
            }
            let by_player = w.entity(d.source).is_some_and(|e| e.kind == PLAYER.id());
            lay(&w, d.entity, index, (mass, scale), yaw, [d.pos.x, d.pos.y, d.pos.z], by_player);
        }
    });
}

/// A carcass of `of` (species `index`, `mass` kg drawn at `scale`) laid at `pos` facing `yaw`; skinned if a player's kill (its loot
/// dropped).
fn lay(w: &World, of: u64, index: usize, (mass, scale): (f32, f32), yaw: f32, pos: [f64; 3], skinned: bool) -> u64 {
    let id = w.spawn_with(&CARCASS, pos, &Spawn::new().yaw(yaw));
    if id == 0 {
        return 0;
    }
    let eaten = {
        let mut t = TAKEN.lock().unwrap();
        let kg = t.iter().filter(|r| r.0 == of).map(|r| r.1).sum::<f32>();
        t.retain(|r| r.0 != of);
        kg
    };
    if let Some(b) = w.component_mut(id, &BODY) {
        *b = Body { species: index as u32 + 1, mass, eaten, aged: 0.0, at: w.game_time(), scale, skinned: skinned as u32, look: 0, of };
    }
    drop(w.entity_modify(id).persistent(true));
    weft_sdk::info!(
        "carcass {id} of {} {of} ({mass:.1} kg){} at {:.0} {:.0} {:.0}",
        SPECIES[index].name,
        if skinned { ", skinned" } else { "" },
        pos[0],
        pos[1],
        pos[2]
    );
    show(w, id);
    id
}

/// `kg` taken off the body of `of` (a hunter's meal), lying near `at`: its carcass, or kept for it until it is laid.
pub fn taken(w: &World, of: u64, at: Vec3, kg: f32) {
    match find(w, of, at) {
        Some(id) => {
            take(w, id, kg);
        }
        None => TAKEN.lock().unwrap().push((of, kg)),
    }
}

/// The carcass of the animal `of` near `at`.
fn find(w: &World, of: u64, at: Vec3) -> Option<u64> {
    w.entities_near(at, 4.0, Some(&CARCASS))
        .into_iter()
        .find(|&c| w.component(c, &BODY).is_some_and(|b| b.of == of))
}

/// `kg` eaten off carcass `id`; its look again. What it gave (less when little is left).
pub fn take(w: &World, id: u64, kg: f32) -> f32 {
    let Some(b) = w.component_mut(id, &BODY) else { return 0.0 };
    let left = (b.mass * core::EDIBLE - b.eaten).max(0.0);
    let got = kg.min(left);
    b.eaten += got;
    show(w, id);
    got
}

/// What can still be eaten off carcass `id`, kg (`None`: not a carcass).
pub fn left(w: &World, id: u64) -> Option<f32> {
    let b = w.component(id, &BODY)?;
    Some((b.mass * core::EDIBLE - b.eaten).max(0.0))
}

/// Its look by what's been eaten and its rot; gone when it has rotted away or nothing is left.
fn look(b: &Body) -> Option<Look> {
    let rot = vitals::rot::carcass_stage(core::keeps(), b.aged);
    core::look(b.eaten, b.mass, rot, b.skinned != 0)
}

/// Draws carcass `id` as its look (when that changed), or takes it away when gone.
fn show(w: &World, id: u64) {
    let Some(b) = w.component(id, &BODY) else { return };
    let Some(l) = look(&b) else {
        weft_sdk::info!("carcass {id} gone ({:.1} of {:.1} kg eaten, {:.1} days aged)", b.eaten, b.mass * core::EDIBLE, b.aged);
        w.despawn(id);
        return;
    };
    let n = l as u32 + 1;
    if w.component(id, &SHOWN) == Some(n) {
        return;
    }
    let Some(s) = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return;
    };
    match l {
        // The animal itself lying dead: its death clip, held on its last frame.
        Look::Whole | Look::Opened => {
            w.set_appearance(id, Some(name_of(abi::entry_kind::ENTITY, s.kind.id())));
            w.set_scale(id, if b.scale > 0.0 { b.scale } else { 1.0 });
            w.action_play(id, "death", false);
        }
        Look::Bones | Look::Pile => {
            let kind = if l == Look::Bones { &BONES } else { &PILE };
            w.set_appearance(id, Some(name_of(abi::entry_kind::ENTITY, kind.id())));
            w.set_scale(id, core::scale(b.mass));
            w.action_stop(id);
        }
    }
    if let Some(v) = w.component_mut(id, &SHOWN) {
        *v = n;
    }
    if b.look != 0 && b.look != n {
        weft_sdk::info!(
            "carcass {id} of {} now {l:?} ({:.1} of {:.1} kg eaten, {:.1} days aged)",
            s.name,
            b.eaten,
            b.mass * core::EDIBLE,
            b.aged
        );
    }
    if let Some(b) = w.component_mut(id, &BODY) {
        b.look = n;
    }
}

/// A goal: it ages as its raw meat would at its spot's air (`vitals::rot::rate`), and looks again.
fn rot(w: &World, id: u64, st: &EntityState) -> bool {
    let now = w.game_time();
    if !(now + id).is_multiple_of(EVERY) {
        return true;
    }
    let celsius = base_api::air_at(w, st.pos.to_array(), 0.5, st.in_liquid != 0, 0, |_| {}).celsius;
    if let Some(b) = w.component_mut(id, &BODY) {
        let ticks = now.saturating_sub(if b.at == 0 { now } else { b.at });
        b.aged += ticks as f32 / 24_000.0 * vitals::rot::rate(celsius);
        b.at = now;
    }
    show(w, id);
    true
}

/// `/animalprof carcass [days]`: every carcass within 64 blocks of `at` (aged `days` more first): a line each.
pub fn report(w: &World, at: Vec3, days: f32) -> String {
    let mut rows = Vec::new();
    for id in w.entities_near(at, 64.0, Some(&CARCASS)) {
        if days > 0.0 {
            if let Some(b) = w.component_mut(id, &BODY) {
                b.aged += days;
            }
            show(w, id);
        }
        let (Some(b), Some(e)) = (w.component(id, &BODY), w.entity(id)) else { continue };
        let name = (b.species as usize)
            .checked_sub(1)
            .and_then(|i| SPECIES.get(i))
            .map_or("?", |s| s.name);
        rows.push(format!(
            "carcass {id} {name} {:?} {:.1}/{:.1} kg eaten, {:.2} d aged{} at {:.0} {:.0} {:.0}",
            look(&b),
            b.eaten,
            b.mass * core::EDIBLE,
            b.aged,
            if b.skinned != 0 { ", skinned" } else { "" },
            e.pos.x,
            e.pos.y,
            e.pos.z
        ));
    }
    if rows.is_empty() { "no carcasses near".to_string() } else { rows.join(" | ") }
}

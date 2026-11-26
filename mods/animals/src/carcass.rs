//! A dead animal's body (`animals_core::carcass`, `notes/crew/plans/carcass-and-skinning.md`, slice 1). The engine
//! removes a creature the tick it dies, so its body is read at the `DIE` question (before removal: its species,
//! mass, where it faces) and a carcass entity is laid where it fell. Its look steps down as it is eaten and as it rots:
//! whole and opened drawn as the animal itself laid on its side, then a ribcage, then a pile of bones
//! (placeholders until the blocky-model looks per body plan), then it is gone.
//!
//! Who takes from it: the killer and its pack (`hunt.rs`: their meal's kilograms, `taken`), scavengers (`forage.rs`:
//! a bite at a time, smelled from afar as carrion is), and a player's knife (slice 2, `skin`).
//!
//! Skinning (Ronald 11-25: "skinning only"): an animal that leaves a carcass drops no hide or meat when it dies
//! (`lib.rs`); a player with a knife (or a flake) in hand uses the carcass, kneels beside it (`SKINNING`, frozen and
//! turned to it), the camera eases in low beside them (`SKIN_CAMERA`), and after the knife's time
//! (`animals_core::carcass::skin_secs`) the hide (poor, good or perfect: `quality`) and what meat is left come off and
//! the carcass is left skinned and stripped. Walking off (more than `SKIN_REACH`) stops it.

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
        /// Its wild level, the hits it took and what killed it (`damage_cause`): its hide's quality.
        pub level: u32,
        pub hits: u32,
        pub cause: u32,
    }
}
pub static BODY: Component<Body> = Component::named("carcass");
/// The look it is drawn as on this run, + 1 (not saved: the engine forgets appearance and scale on loading); its
/// watchers have it, to lay a whole or opened body on its side (`client_entity_frame`).
static SHOWN: Component<u32> = Component::named("carcass_shown");

/// Hits taken in its life (not saved): a hide's quality (`animals_core::carcass::kill`).
static HITS: Component<u32> = Component::named("carcass_hits");
/// Being skinned: by whom, from and until which tick, the knife's tier + 1 (0: a flake); not saved.
static SKIN: Component<[u64; 4]> = Component::named("carcass_skin");
/// The player kneeling at a carcass, and the camera beside them.
static SKINNING: Pose = Pose::named("skinning");
static SKIN_CAMERA: CameraMode = CameraMode::named("skinning");

/// The dying read at the `DIE` question: the animal, its species index, mass, yaw, drawn scale, level, hits.
type Dying = (u64, usize, f32, f32, f32, u32, u32);
static DYING: Mutex<Vec<Dying>> = Mutex::new(Vec::new());
/// Kilograms taken off a body before its carcass was laid (the hunters' `DIE` handler may run first).
static TAKEN: Mutex<Vec<(u64, f32)>> = Mutex::new(Vec::new());

/// Model pixels a body laid on its side is lifted by: about half a deer's body width, so its flank lies on the ground.
const LIE_LIFT: f32 = 4.0;

/// How often a carcass ages and looks again, ticks.
const EVERY: u64 = 100;

pub fn init(r: &mut Registry) {
    r.entity_component(&BODY, Body::default(), component_flags::SAVE);
    r.entity_component(&SHOWN, 0u32, component_flags::SYNC_WATCHERS);
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
        .attach(&SKIN)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            start_skinning(&w, id, player)
        })
        .goals(Goals::new().then(rot));
    r.entity_component(&SKIN, [0u64; 4], 0);
    // A whole or opened body is the animal's own model, laid on its side: its root rolled a quarter turn and lifted by
    // half a body's width (r4: drawn with no clip it stood, legs down, beside the skinner).
    r.client_entity_frame(&CARCASS, |c, e| {
        let shown = c.view().component_of(e.id(), &SHOWN).unwrap_or(0);
        if shown == Look::Whole as u32 + 1 || shown == Look::Opened as u32 + 1 {
            c.bone_turn(e.id(), "root", [0.0, 0.0, 90.0]);
            c.bone_move(e.id(), "root", [0.0, LIE_LIFT, 0.0]);
        }
    });
    // A knife used at the ground beside a carcass skins it too (a body lying flat is easy to aim past: r3's use hit
    // the grass by it). The nearest carcass within `SKIN_REACH` of the block aimed at, or of the player.
    r.before_event(&actions::USE_ITEM, |q| {
        let (w, player) = (q.world(), q.entity());
        if player == 0 || !matches!(item_name(q.item()), "base:knife" | "base:sharp_flake") {
            return Verdict::Abstain;
        }
        let Some(me) = w.entity(player) else { return Verdict::Abstain };
        let p = q.pos();
        let at = if q.state() != 0 { Vec3 { x: p.x as f64 + 0.5, y: p.y as f64 + 1.0, z: p.z as f64 + 0.5 } } else { me.pos };
        let near = w
            .entities_near(at, core::SKIN_REACH, Some(&CARCASS))
            .into_iter()
            .filter(|&c| crate::dist2(w.entity(c).map_or(at, |e| e.pos), me.pos) <= core::SKIN_REACH * core::SKIN_REACH)
            .min_by(|&a, &b| {
                let d = |c: u64| crate::dist2(w.entity(c).map_or(at, |e| e.pos), at);
                d(a).total_cmp(&d(b))
            });
        match near {
            Some(c) if start_skinning(&w, c, player) => Verdict::Deny,
            _ => Verdict::Abstain,
        }
    });
    r.stack_component(&PELT_QUALITY, 0);
    r.entity_component(&HITS, 0u32, 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&HITS);
    }
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            if let Some(n) = w.component_mut(h.entity, &HITS) {
                *n += 1;
            }
        }
    });
    // Kneeling, still, at eye height a little over half a standing one's (`player.animation.json`'s `skin` clip).
    r.pose(&SKINNING)
        .height(0.6)
        .eye(0.55)
        .speed(0.0)
        .action("skin")
        .no_bob();
    // Low beside the skinner's right shoulder, looking down at the body before them.
    r.camera(&SKIN_CAMERA)
        .shot(camera::Shot::new(&[abi::comp::camera::BODY, abi::comp::camera::HUD, abi::comp::camera::CLIP], |a| {
            let (yaw, eye) = (a.eye_yaw as f64, a.eye);
            let (f, right) = ([-yaw.sin(), -yaw.cos()], [yaw.cos(), -yaw.sin()]);
            // Out past the arm (r4's, 1.1 blocks out, sat inside it), a little above the eyes.
            let pos = Vec3 { x: eye.x + right[0] * 1.9 - f[0] * 1.3, y: eye.y + 0.7, z: eye.z + right[1] * 1.9 - f[1] * 1.3 };
            let at = Vec3 { x: eye.x + f[0] * 1.5, y: eye.y - 0.9, z: eye.z + f[1] * 1.5 };
            let (yaw, pitch) = camera::look_at(pos, at);
            a.out = abi::CameraPose { pos, yaw, pitch, ..abi::CameraPose::DEFAULT };
        }))
        .transition(0.6, camera::easing::IN_OUT);
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
            // The killing blow is no `HURT` (a lethal hit fires `DIE` alone): one more.
            let hits = w.component(id, &HITS).unwrap_or(0) + 1;
            let mut d = DYING.lock().unwrap();
            d.retain(|r| r.0 != id);
            d.push((id, index, mass, e.yaw, scale, a.level(), hits));
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
            let Some((_, index, mass, yaw, scale, level, hits)) = read.or_else(|| {
                let index = SPECIES.iter().position(|s| s.kind.id() == d.kind)?;
                Some((d.entity, index, SPECIES[index].bio.mass, 0.0, 1.0, 0, 1))
            }) else {
                continue;
            };
            // By its species' size, as its loot goes (`lib.rs`): a young deer leaves a carcass too.
            if core::drops_on_death(SPECIES[index].bio.mass) || crate::hunt::egg_of(&w, d.entity).is_some() {
                continue;
            }
            let id = lay(&w, d.entity, index, (mass, scale), yaw, [d.pos.x, d.pos.y, d.pos.z], false);
            if let Some(b) = w.component_mut(id, &BODY) {
                (b.level, b.hits, b.cause) = (level, hits, d.cause);
            }
        }
    });
}

/// A carcass of `of` (species `index`, `mass` kg drawn at `scale`) laid at `pos` facing `yaw`.
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
        *b = Body {
            species: index as u32 + 1,
            mass,
            eaten,
            aged: 0.0,
            at: w.game_time(),
            scale,
            skinned: skinned as u32,
            look: 0,
            of,
            ..Body::default()
        };
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
        // The animal itself lying dead, laid on its side on the client (`client_entity_frame`).
        Look::Whole | Look::Opened => {
            w.set_appearance(id, Some(name_of(abi::entry_kind::ENTITY, s.kind.id())));
            w.set_scale(id, if b.scale > 0.0 { b.scale } else { 1.0 });
            w.action_stop(id);
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

/// A goal: it ages as its raw meat would at its spot's air (`vitals::rot::rate`), and looks again; a skinning goes on.
fn rot(w: &World, id: u64, st: &EntityState) -> bool {
    let now = w.game_time();
    skinning(w, id, st, now);
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

/// `/animalprof carcass [days]`: every carcass within 128 blocks (a chase ends far off) of `at` (aged `days` more first): a line each.
pub fn report(w: &World, at: Vec3, days: f32) -> String {
    let mut rows = Vec::new();
    for id in w.entities_near(at, 128.0, Some(&CARCASS)) {
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

// ---------------------------------------------------------------- skinning

/// A knife's tier in `player`'s hand (`base_api::ITEM_LEVEL` / 10), `Some(None)` for a flake, `None` for neither.
fn knife(w: &World, player: u64) -> Option<Option<u32>> {
    let stack = w.inventory(player).get(w.selected_slot(player));
    match item_name(stack.item) {
        "base:knife" => Some(Some((base_api::ITEM_LEVEL.on_stack(&stack).unwrap_or(5.0) / 10.0) as u32)),
        "base:sharp_flake" | "base:flake" => Some(None),
        _ => None,
    }
}

/// `player` used carcass `id`: with a knife, they kneel and start skinning it. Whether it took the use.
fn start_skinning(w: &World, id: u64, player: u64) -> bool {
    let (Some(b), Some(c), Some(p)) = (w.component(id, &BODY), w.entity(id), w.entity(player)) else { return false };
    let Some(tier) = knife(w, player) else {
        base_api::hint(w, player, "hint.animals.skin_knife", &[]);
        return false;
    };
    if b.skinned != 0 || w.component(id, &SKIN).is_some_and(|k| k[0] != 0) {
        base_api::hint(w, player, "hint.animals.skinned_already", &[]);
        return true;
    }
    let now = w.game_time();
    let ticks = (core::skin_secs(b.mass, tier) * 20.0).round() as u64;
    if let Some(k) = w.component_mut(id, &SKIN) {
        *k = [player, now, now + ticks, tier.map_or(0, |t| t as u64 + 1)];
    }
    // Turned to the body, kneeling beside it; the camera eases in.
    let (dx, dz) = (c.pos.x - p.pos.x, c.pos.z - p.pos.z);
    let facing = ((-dx).atan2(-dz) as f32).to_degrees();
    w.set_pose(player, Some(&SKINNING), Some(facing));
    w.force_camera(player, &SKIN_CAMERA, Some((0.6, camera::easing::IN_OUT)));
    weft_sdk::info!("player {player} skins carcass {id} ({:.1} kg, {:.1} s, knife tier {tier:?})", b.mass, ticks as f32 / 20.0);
    true
}

/// The player is let go: standing again, their own camera back.
fn stand_up(w: &World, player: u64) {
    w.set_pose(player, None, None);
    w.release_camera(player, Some((0.5, camera::easing::IN_OUT)));
}

/// A skinning of carcass `id` goes on: halfway the body shows opened, at the end its hide and meat come off; it stops
/// if the skinner is gone or walked off.
fn skinning(w: &World, id: u64, st: &EntityState, now: u64) {
    let Some([player, from, until, tier]) = w.component(id, &SKIN).filter(|k| k[0] != 0) else { return };
    let gone = w
        .entity(player)
        .is_none_or(|p| p.health <= 0.0 || crate::dist2(p.pos, st.pos) > core::SKIN_REACH * core::SKIN_REACH);
    if gone {
        if let Some(k) = w.component_mut(id, &SKIN) {
            *k = [0; 4];
        }
        stand_up(w, player);
        weft_sdk::info!("player {player} stops skinning carcass {id}");
        return;
    }
    if now < until {
        // Halfway: the hide is off, the body opened.
        if now == from + (until - from) / 2
            && let Some(b) = w.component_mut(id, &BODY)
        {
            b.skinned = 1;
            show(w, id);
        }
        return;
    }
    if let Some(k) = w.component_mut(id, &SKIN) {
        *k = [0; 4];
    }
    stand_up(w, player);
    let Some(b) = w.component(id, &BODY) else { return };
    let Some(s) = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return;
    };
    let rot = vitals::rot::carcass_stage(core::keeps(), b.aged);
    let knife = (tier != 0).then(|| tier as u32 - 1);
    let quality = core::quality(b.level, core::kill(b.hits, b.cause), knife, rot);
    let at = [st.pos.x, st.pos.y + 0.4, st.pos.z];
    let mut got = Vec::new();
    // The hide (none once rotted to remains), its quality on it.
    let hide_share = vitals::rot::CARCASS
        .iter()
        .find(|c| c.0 == rot)
        .map_or(0.0, |c| c.3);
    if hide_share > 0.0
        && let Some(&(_, item, lo, hi)) = animals_core::hides::HIDES.iter().find(|h| h.0 == s.name)
    {
        let n = core::hide_count(lo, hi, quality);
        if n > 0 {
            w.spawn_item(at, ItemStack::new(Item::generated(item.to_string()).id(), n).with(&PELT_QUALITY, quality as u32));
            got.push(format!("{n} {item} (quality {quality})"));
        }
    }
    // The meat left on it: what's not been eaten, less as it rots. It comes off the body.
    if let Some((_, meat, lo, hi)) = animals_core::meat::drops().find(|d| d.0 == s.name) {
        let n = core::meat_count(lo, hi, b.eaten, b.mass, rot);
        if n > 0 {
            w.spawn_item(at, ItemStack::new(Item::generated(format!("animals:{meat}")).id(), n));
            got.push(format!("{n} animals:{meat}"));
        }
    }
    // One with no meat of its own gives carcass pieces (`meat::carcass`: carrion, bait), as many as its meat would be.
    if let Some((lo, hi)) = animals_core::meat::carcass(s.name) {
        let n = core::meat_count(lo, hi, b.eaten, b.mass, rot);
        if n > 0 {
            w.spawn_item(at, ItemStack::new(Item::generated(animals_core::meat::CARCASS.to_string()).id(), n));
            got.push(format!("{n} {}", animals_core::meat::CARCASS));
        }
    }
    let meat_kg = (b.mass * animals_core::meat::YIELD).min((b.mass * core::EDIBLE - b.eaten).max(0.0));
    if let Some(b) = w.component_mut(id, &BODY) {
        b.skinned = 1;
        b.eaten += meat_kg;
    }
    weft_sdk::info!(
        "player {player} skinned carcass {id} of {}: {} (level {}, {} hits, cause {}, knife {knife:?}, {:?})",
        s.name,
        if got.is_empty() { "nothing".to_string() } else { got.join(", ") },
        b.level,
        b.hits,
        damage_cause::NAMES.get(b.cause as usize).unwrap_or(&"?"),
        rot
    );
    base_api::hint(w, player, &format!("hint.animals.pelt_{quality}"), &[]);
    show(w, id);
}

/// A hide's quality from skinning, 1 poor, 2 good, 3 perfect (`animals_core::carcass::quality`); none: an old one.
pub static PELT_QUALITY: Component<u32> = Component::named("pelt_quality");

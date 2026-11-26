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

/// The body: its kind (`EntityKind`); the looks it is drawn as: its body plan's whole and opened (slice 3, at the
/// plans' order in `animals_core::carcass::Plan::ALL`), its bones and its pile of bones.
pub static CARCASS: EntityKind = EntityKind::named("carcass_body");
static WHOLE: [EntityKind; 5] = [
    EntityKind::named("carcass_quadruped"),
    EntityKind::named("carcass_bird"),
    EntityKind::named("carcass_biped"),
    EntityKind::named("carcass_reptile"),
    EntityKind::named("carcass_fish"),
];
static OPENED: [EntityKind; 5] = [
    EntityKind::named("carcass_quadruped_open"),
    EntityKind::named("carcass_bird_open"),
    EntityKind::named("carcass_biped_open"),
    EntityKind::named("carcass_reptile_open"),
    EntityKind::named("carcass_fish_open"),
];
/// What its watchers draw it from (`Dead`), saved.
static GENES: Component<Dead> = Component::named("carcass_genome");
/// Flies over a rotting body (`animals_core::carcass::flies`), drawn on each client.
static FLY: Particle = Particle::named("carcass_fly");
static BONES: [EntityKind; 5] = [
    EntityKind::named("carcass_quadruped_bones"),
    EntityKind::named("carcass_bird_bones"),
    EntityKind::named("carcass_biped_bones"),
    EntityKind::named("carcass_reptile_bones"),
    EntityKind::named("carcass_fish_bones"),
];
static PILE: [EntityKind; 5] = [
    EntityKind::named("carcass_quadruped_pile"),
    EntityKind::named("carcass_bird_pile"),
    EntityKind::named("carcass_biped_pile"),
    EntityKind::named("carcass_reptile_pile"),
    EntityKind::named("carcass_fish_pile"),
];
/// The blood and guts under an opened body: a second entity at its place (the animal's own model can't be painted per
/// carcass), so "opened" reads apart from "whole" until the art slice draws the opened body itself.
static GORE: EntityKind = EntityKind::named("carcass_gore");
/// The blood pool's rot step (`animals_core::carcass::gore_tint`), its watchers repainting it when it changes.
static GORE_ROT: Component<u32> = Component::named("carcass_gore_rot");

value! {
    #[derive(Default)]
    /// What a body's watchers paint and draw it from: the animal's genes (species 0: none, drawn in the texture's own
    /// deer coat), painted on the body as on the living animal (`paint_dead`), its rot step
    /// (`animals_core::carcass::rot_step`, tinting the coat) and the flies over it.
    pub struct Dead {
        pub genes: crate::coats::Genome,
        pub rot: u8,
        pub flies: u8,
    }
}

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
/// The look it is drawn as on this run, + 1 (not saved: the engine forgets appearance and scale on loading).
static SHOWN: Component<u32> = Component::named("carcass_shown");

/// Hits taken in its life (not saved): a hide's quality (`animals_core::carcass::kill`).
static HITS: Component<u32> = Component::named("carcass_hits");
/// Being skinned: by whom, from and until which tick, the knife's tier + 1 (0: a flake); not saved.
static SKIN: Component<[u64; 4]> = Component::named("carcass_skin");
/// The player kneeling at a carcass, and the camera beside them.
static SKINNING: Pose = Pose::named("skinning");
static SKIN_CAMERA: CameraMode = CameraMode::named("skinning");

/// The dying read at the `DIE` question.
#[derive(Clone, Copy)]
struct Dying {
    id: u64,
    index: usize,
    mass: f32,
    yaw: f32,
    /// Its drawn scale (genes and growth), its wild level, the hits it took, its genes (`GENES`).
    scale: f32,
    level: u32,
    hits: u32,
    genes: crate::coats::Genome,
}
static DYING: Mutex<Vec<Dying>> = Mutex::new(Vec::new());
/// Kilograms taken off a body before its carcass was laid (the hunters' `DIE` handler may run first).
static TAKEN: Mutex<Vec<(u64, f32)>> = Mutex::new(Vec::new());

/// How often a carcass ages and looks again, ticks.
const EVERY: u64 = 100;

pub fn init(r: &mut Registry) {
    r.entity_component(&BODY, Body::default(), component_flags::SAVE);
    r.entity_component(&SHOWN, 0u32, 0);
    r.entity_component(&GENES, Dead::default(), component_flags::SAVE | component_flags::SYNC_WATCHERS);
    // Its box the lying deer's (scaled with it, `set_scale`), so a click on the body hits it (r7's knife use landed on
    // the grass beside one boxed as the ribcage).
    let (bw, bh) = animals_core::bodies::of("carcass_quadruped");
    r.entity(&CARCASS)
        .no_egg()
        .size(bw, bh)
        .max_health(20.0)
        .walk_speed(0.0)
        .passive()
        .unpushable()
        .model_file("carcass_quadruped")
        .texture("carcass/body")
        .attach(&BODY)
        .attach(&SHOWN)
        .attach(&GENES)
        .attach(&SKIN)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            start_skinning(&w, id, player)
        })
        .goals(Goals::new().then(rot));
    r.entity_component(&SKIN, [0u64; 4], 0);
    // Its coat painted over whichever body plan it is drawn as (they share the texture), keeping the light painted in.
    r.entity_skin(&CARCASS, &GENES, paint_dead);
    // The pool under an opened body rots with it: browner, then a dry dark stain (`core::gore_tint`).
    r.entity_component(&GORE_ROT, 0u32, component_flags::SAVE | component_flags::SYNC_WATCHERS);
    r.particle(&FLY)
        .textures(&["carcass_fly"])
        .life(FLY_LIFE * 0.7, FLY_LIFE * 1.3)
        .size(0.035, 0.05)
        .gravity(0.0)
        .drag(0.92);
    r.client_entity_frame(&CARCASS, flies);
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
        // A block's breadth past the reach round the spot aimed at (r7's deer lay 2.5-3 blocks off it); the player
        // still within reach of the body.
        let near = w
            .entities_near(at, core::SKIN_REACH + 1.0, Some(&CARCASS))
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
    r.entity_component(&HITS, 0u32, 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&HITS);
    }
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        // Blows only: not its bleeding, hunger or cold.
        for h in b.iter().filter(|h| !crate::comfort::bodily(h.cause)) {
            if let Some(n) = w.component_mut(h.entity, &HITS) {
                *n += 1;
            }
        }
    });
    // Kneeling forward over the body, still (`player.animation.json`'s `skin` clip: the knees down behind, the torso
    // leaning 30 degrees about the hips with the head and arms carried along, its eye at 1.33 blocks).
    r.pose(&SKINNING)
        .height(0.93)
        .eye(0.8)
        .speed(0.0)
        .action("skin")
        .no_bob();
    // Beside the skinner, three-quarters from the right, the body before them in view.
    r.camera(&SKIN_CAMERA)
        .shot(camera::Shot::new(&[abi::comp::camera::BODY, abi::comp::camera::HUD, abi::comp::camera::CLIP], |a| {
            let (yaw, eye) = (a.eye_yaw as f64, a.eye);
            let (f, right) = ([-yaw.sin(), -yaw.cos()], [yaw.cos(), -yaw.sin()]);
            // A three-quarter side view from the right, level with the body before them and looking between the
            // skinner and it, so both the knife and the body are in frame (r4's, 1.1 blocks out, sat inside the
            // arm; r5's, behind the shoulder, had the head filling the near side). r9's, level with the skinner and
            // above, showed the back of a tipped head bigger than the body: now out in front on the right and lower,
            // the body nearer the camera than the skinner, who shows face and chest.
            // r1b's, from above, saw only the top of the bowed head: now at the eyes' height.
            let pos = Vec3 { x: eye.x + right[0] * 2.4 + f[0] * 2.2, y: eye.y + 0.05, z: eye.z + right[1] * 2.4 + f[1] * 2.2 };
            let at = Vec3 { x: eye.x + f[0] * 0.8, y: eye.y - 0.55, z: eye.z + f[1] * 0.8 };
            let (yaw, pitch) = camera::look_at(pos, at);
            a.out = abi::CameraPose { pos, yaw, pitch, ..abi::CameraPose::DEFAULT };
        }))
        .transition(0.6, camera::easing::IN_OUT);
    let (gw, gh) = animals_core::bodies::of("carcass_gore");
    r.entity(&GORE)
        .no_egg()
        .size(gw, gh)
        .walk_speed(0.0)
        .passive()
        .invulnerable()
        .unpushable()
        .model_file("carcass_gore")
        .texture("carcass/bones")
        .attach(&GORE_ROT)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            let Some(at) = w.entity(id).map(|g| g.pos) else { return false };
            let near = w.entities_near(at, 1.0, Some(&CARCASS));
            near.first().is_some_and(|&c| start_skinning(&w, c, player))
        });
    r.entity_skin(&GORE, &GORE_ROT, |step, skin| core::gore_tint(skin.rgba, *step as u8));
    for (i, plan) in core::Plan::ALL.into_iter().enumerate() {
        let looks = [(&WHOLE[i], ""), (&OPENED[i], "_open"), (&BONES[i], "_bones"), (&PILE[i], "_pile")];
        for (kind, model) in looks.map(|(k, s)| (k, format!("carcass_{}{s}", plan.name()))) {
            let (w, h) = animals_core::bodies::of(&model);
            r.entity(kind)
                .no_egg()
                .size(w, h)
                .walk_speed(0.0)
                .passive()
                // Arrows and blows don't kill a body (pt_rain_8: 11 arrows after the kill shot it to nothing in 20 s,
                // with no line logged); it goes by skinning, rot and scavengers.
                .invulnerable()
                .unpushable()
                .model_file(&model)
                .texture("carcass/body");
        }
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
            let genes = w.component(id, &crate::coats::GENOME).unwrap_or_default();
            let mut d = DYING.lock().unwrap();
            d.retain(|r| r.id != id);
            d.push(Dying { id, index, mass, yaw: e.yaw, scale, level: a.level(), hits, genes });
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
                let i = dying.iter().position(|r| r.id == d.entity);
                i.map(|i| dying.swap_remove(i))
            };
            // Died some other way (a command, the void): its kind's own size, facing north.
            let Some(Dying { index, mass, yaw, scale, level, hits, genes, .. }) = read.or_else(|| {
                let index = SPECIES.iter().position(|s| s.kind.id() == d.kind)?;
                Some(Dying {
                    id: d.entity,
                    index,
                    mass: SPECIES[index].bio.mass,
                    yaw: 0.0,
                    scale: 1.0,
                    level: 0,
                    hits: 1,
                    genes: Default::default(),
                })
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
            if let Some(g) = w.component_mut(id, &GENES) {
                g.genes = genes;
            }
        }
    });
}

/// A carcass of `of` (species `index`, `mass` kg drawn at `scale`) laid at `pos` facing `yaw`.
fn lay(w: &World, of: u64, index: usize, (mass, scale): (f32, f32), yaw: f32, pos: [f64; 3], skinned: bool) -> u64 {
    // Out of any wall or cliff it died pressed against (`core::clear_spot`), so a knife can reach it.
    let plan = core::plan_of(SPECIES[index].model);
    let (bw, bh) = animals_core::bodies::of(&format!("carcass_{}", plan.name()));
    let size = core::body_scale(mass, plan) as f64;
    let at = core::clear_spot(pos, bw as f64 * size / 2.0, bh as f64 * size, |p| crate::boss::solid(w, p));
    if at != pos {
        weft_sdk::info!(
            "carcass of {of} moved out of the blocks at {:.1} {:.1} {:.1} to {:.1} {:.1}",
            pos[0],
            pos[1],
            pos[2],
            at[0],
            at[2]
        );
    }
    let pos = at;
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
        // What's left of it rots into the ground: richer farmland, grass grown back round it (`vitals::soil::RICH_KG`).
        if let Some(e) = w.entity(id) {
            let at = e.pos.to_array().map(|v| v.floor() as i32);
            let levels = vitals::soil::rotted_levels(b.mass - b.eaten);
            let (fed, grown) = base_api::grow::enrich(w, at, levels, vitals::soil::RICH_REACH, levels > 0);
            weft_sdk::info!(
                "carcass {id} rots into the ground at {at:?}: {:.1} kg, {levels} levels to {fed} farmland, {grown} grass grown back",
                b.mass - b.eaten
            );
        }
        gore(w, id, &b, false, 0);
        w.despawn(id);
        return;
    };
    // Its rot drawn (a repaint when the step changes) and the flies over it.
    let plan_scale = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
        .map_or(1.0, |s| core::body_scale(b.mass, core::plan_of(s.model)));
    let step = core::rot_step(core::keeps(), b.aged);
    let n_flies = core::flies(step, l, plan_scale);
    let stepped = w.component(id, &GENES).is_some_and(|g| g.rot != step);
    if w.component(id, &GENES)
        .is_some_and(|g| (g.rot, g.flies) != (step, n_flies))
        && let Some(g) = w.component_mut(id, &GENES)
    {
        (g.rot, g.flies) = (step, n_flies);
    }
    let n = l as u32 + 1;
    if w.component(id, &SHOWN) == Some(n) {
        if stepped {
            gore(w, id, &b, core::has_gore(l, step), step);
        }
        return;
    }
    let Some(s) = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return;
    };
    // Its body plan's lying model (or its bones, or the pile they end in), at its size, in its own coat (`paint_dead`:
    // the hide scraps and feathers left by the bones too).
    let plan = core::plan_of(s.model);
    let i = core::Plan::ALL.iter().position(|&p| p == plan).unwrap_or(0);
    let kind = match l {
        Look::Whole => &WHOLE[i],
        Look::Opened => &OPENED[i],
        Look::Bones => &BONES[i],
        Look::Pile => &PILE[i],
    };
    w.set_appearance(id, Some(name_of(abi::entry_kind::ENTITY, kind.id())));
    w.set_scale(id, core::body_scale(b.mass, plan));
    w.action_stop(id);
    gore(w, id, &b, core::has_gore(l, step), step);
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

/// Lays the blood and guts under carcass `id` (`on`), at its size and rot `step` (`GORE_ROT`), or takes them up; one at
/// most.
fn gore(w: &World, id: u64, b: &Body, on: bool, step: u8) {
    let Some(e) = w.entity(id) else { return };
    let near = w.entities_near(e.pos, 0.5, Some(&GORE));
    if !on {
        for g in near {
            w.despawn(g);
        }
        return;
    }
    let g = match near.first() {
        Some(&g) => g,
        None => {
            let g = w.spawn_with(&GORE, [e.pos.x, e.pos.y, e.pos.z], &Spawn::new().yaw(e.yaw));
            if g == 0 {
                return;
            }
            drop(w.entity_modify(g).persistent(true));
            g
        }
    };
    // Its scale isn't saved: set again whenever the carcass is shown afresh (after a load too). The pool is built at a
    // deer's size, under its body plan's.
    let plan = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
        .map_or(core::Plan::Quadruped, |s| core::plan_of(s.model));
    w.set_scale(g, core::body_scale(b.mass, plan) * plan.pool());
    if w.component(g, &GORE_ROT) != Some(step as u32)
        && let Some(r) = w.component_mut(g, &GORE_ROT)
    {
        *r = step as u32;
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
        let dead = w.component(id, &GENES).unwrap_or_default();
        let name = (b.species as usize)
            .checked_sub(1)
            .and_then(|i| SPECIES.get(i))
            .map_or("?", |s| s.name);
        rows.push(format!(
            "carcass {id} {name} {:?} {:.1}/{:.1} kg eaten, {:.2} d aged, rot step {}, {} flies{} at {:.0} {:.0} {:.0}",
            look(&b),
            b.eaten,
            b.mass * core::EDIBLE,
            b.aged,
            dead.rot,
            dead.flies,
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
    // Kneeling beside the body, on the side they came from, its hands at it, turned to it; the camera eases in.
    let (dx, dz) = (c.pos.x - p.pos.x, c.pos.z - p.pos.z);
    let len = (dx * dx + dz * dz).sqrt().max(1e-3);
    let plan = (b.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
        .map_or(core::Plan::Quadruped, |s| core::plan_of(s.model));
    let from = (core::KNEEL_FROM * core::body_scale(b.mass, plan) as f64).max(core::KNEEL_MIN);
    w.teleport(player, None, Vec3 { x: c.pos.x - dx / len * from, y: p.pos.y, z: c.pos.z - dz / len * from });
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
    // What comes off is tossed from the body toward the skinner (`off`), to be picked up a moment later: spawned on
    // the skinner, it went into the inventory with nothing seen (Planner 11-26).
    // (`st` is the carcass's own: skin r4b's toss, aimed from it to itself, went straight up and fell into the ribs.)
    let from = st.pos;
    let to = w.entity(player).map_or(from, |p| p.pos);
    let mut n_off = 0;
    let mut off = |stack: ItemStack| {
        let (dx, dz) = (to.x - from.x, to.z - from.z);
        let d = (dx * dx + dz * dz).sqrt().max(0.1);
        let spread = (n_off as f64 - 1.0) * 0.6;
        let v = [
            ((dx / d) * core::off_speed(d) - dz / d * spread) as f32,
            core::OFF_UP,
            ((dz / d) * core::off_speed(d) + dx / d * spread) as f32,
        ];
        n_off += 1;
        w.spawn_with(
            &ITEM,
            [from.x, from.y + 0.5, from.z],
            &Spawn::new()
                .stack(stack)
                .velocity(v)
                .pickup_delay(core::OFF_PICKUP_TICKS),
        );
    };
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
            off(ItemStack::new(Item::generated(item.to_string()).id(), n).with(&base_api::HIDE_QUALITY, quality as u32));
            got.push(format!("{n} {item} (quality {quality})"));
        }
    }
    // The meat left on it: what's not been eaten, less as it rots. It comes off the body.
    if let Some((_, meat, lo, hi)) = animals_core::meat::drops().find(|d| d.0 == s.name) {
        let n = core::meat_count(lo, hi, b.eaten, b.mass, rot);
        if n > 0 {
            off(ItemStack::new(Item::generated(format!("animals:{meat}")).id(), n));
            got.push(format!("{n} animals:{meat}"));
        }
    }
    // One with no meat of its own gives carcass pieces (`meat::carcass`: carrion, bait), as many as its meat would be.
    if let Some((lo, hi)) = animals_core::meat::carcass(s.name) {
        let n = core::meat_count(lo, hi, b.eaten, b.mass, rot);
        if n > 0 {
            off(ItemStack::new(Item::generated(animals_core::meat::CARCASS.to_string()).id(), n));
            got.push(format!("{n} {}", animals_core::meat::CARCASS));
        }
    }
    // A buck's antlers in their season (`animals_core::carcass::antlers`): bone, they keep whatever its rot.
    let female = w.component(id, &GENES).is_some_and(|g| g.genes.female != 0);
    let grown = b.mass / s.bio.mass.max(0.001);
    let n = core::antlers(s.name, female, crate::breeding::season_bit(w), grown);
    if n > 0 {
        off(ItemStack::new(Item::generated("base:antler".to_string()).id(), n));
        got.push(format!("{n} base:antler"));
    }
    let meat_kg = (b.mass * animals_core::meat::YIELD).min((b.mass * core::EDIBLE - b.eaten).max(0.0));
    if let Some(b) = w.component_mut(id, &BODY) {
        b.skinned = 1;
        b.eaten += meat_kg;
    }
    weft_sdk::info!(
        "player {player} skinned carcass {id} of {} ({}): {} (level {}, {} hits, cause {}, knife {knife:?}, {:?})",
        s.name,
        if female { "female" } else { "male" },
        if got.is_empty() { "nothing".to_string() } else { got.join(", ") },
        b.level,
        b.hits,
        damage_cause::NAMES.get(b.cause as usize).unwrap_or(&"?"),
        rot
    );
    base_api::hint(w, player, &format!("hint.animals.pelt_{quality}"), &[]);
    show(w, id);
}

/// Paints a body's coat from its genes (`GENES`) over the bodies' texture (`animals_core::coats::paint_dead`), at its
/// species' usual size for its plan (the client has its genes, not its own mass).
fn paint_dead(dead: &Dead, skin: &mut Skin) {
    let genes = &dead.genes;
    let Some(s) = (genes.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return;
    };
    let scale = core::body_scale(s.bio.mass, core::plan_of(s.model));
    let (w, h) = (skin.width as usize, skin.height as usize);
    crate::coats::paint_dead(genes, skin.rgba, w, h, scale, core::COAT_RGB, core::UNDER_RGB);
    core::rot_tint(skin.rgba, dead.rot);
}

/// A fly lives this long, seconds: a short dart, born again near the body.
const FLY_LIFE: f32 = 0.45;

/// Each frame on a client: keeps a carcass's flies (`Dead::flies`) darting over it, each a short-lived particle in a box
/// round the body.
fn flies(c: &Client, e: &weft_sdk::client::ClientEntity) {
    let Some(dead) = c.component_of(e.id(), &GENES) else { return };
    if dead.flies == 0 {
        return;
    }
    // As many born a second as die, so `flies` live at once; a frame's share, the fraction by chance.
    let born = dead.flies as f32 * e.dt() / FLY_LIFE;
    let mut seed = (e.id() as u32).wrapping_mul(2_654_435_761) ^ (e.age() * 1000.0) as u32;
    let mut rng = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 10_000) as f32 / 10_000.0
    };
    let n = born as u32 + (rng() < born.fract()) as u32;
    let spread = 0.25 + 0.06 * dead.flies as f32;
    let p = e.pos();
    for _ in 0..n {
        let at =
            [p.x + ((rng() - 0.5) * 2.0 * spread) as f64, p.y + 0.15 + (rng() * 0.6) as f64, p.z + ((rng() - 0.5) * 2.0 * spread) as f64];
        let v = [(rng() - 0.5) * 2.4, (rng() - 0.5) * 1.0, (rng() - 0.5) * 2.4];
        drop(c.spawn_particles(&FLY, at).velocity(v));
    }
}

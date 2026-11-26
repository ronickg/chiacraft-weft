//! Wounds and blood trails (`animals_core::blood`): a heavy blow opens a wound that bleeds a while (`tick`, once a
//! second from `animal::upkeep`), taking health as `base:bleed`; bleeding as it goes, it leaves spots of blood
//! (`blood_spot`, unsaved entities that shrink as they dry and go). A hungry hunter with no prey in sight follows the
//! freshest spots it finds (`trail`, from `hunt.rs`): to the bleeding animal, or to its carcass if it bled out.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

use crate::animal::ANIMAL;
use crate::dist2;
use crate::species::SPECIES;
use animals_core::blood as core;

/// Base's bleeding (`survival::body::BLEED`): its death message, past armour.
pub static BLEED: DamageCause = DamageCause::named("base:bleed");
/// How long it still bleeds (seconds), where it left its last spot (x, z) and when (game tick): not saved.
static WOUND: Component<[f64; 4]> = Component::named("animal_wound");
/// A spot of blood on the ground.
static SPOT: EntityKind = EntityKind::named("blood_spot");
/// Whose blood a spot is and when it fell (game tick): not saved, nor are the spots.
static SPOT_OF: Component<[u64; 2]> = Component::named("blood_spot_of");

pub fn init(r: &mut Registry) {
    r.entity_component(&WOUND, [0.0; 4], 0);
    r.entity_component(&SPOT_OF, [0u64; 2], 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&WOUND);
    }
    let (sw, sh) = animals_core::bodies::of("blood_spot");
    r.entity(&SPOT)
        .no_egg()
        .size(sw, sh)
        .walk_speed(0.0)
        .passive()
        .unpushable()
        .model_file("blood_spot")
        .texture("carcass/bones")
        .attach(&SPOT_OF)
        .goals(Goals::new().then(dry));
    // A blow struck by something (not its own bleeding, hunger or cold, nor a fall) of `core::WOUND` of its max health or
    // more opens a wound.
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            // Something struck it (a fall, a test's sourceless hurt: no wound).
            if h.amount <= 0.0 || crate::comfort::bodily(h.cause) || (h.source == 0 && h.cause != damage_cause::ATTACK) {
                continue;
            }
            // Wild ones only: a tame's wounds wait for its keeper's care (bandages: later), as a tame never starves.
            if crate::animal::tamed(&w, h.entity) {
                continue;
            }
            let Some(st) = w
                .entity(h.entity)
                .filter(|st| st.health > 0.0 && st.max_health > 0.0)
            else {
                continue;
            };
            let secs = core::bleed_secs(h.amount / st.max_health) as f64;
            if secs <= 0.0 {
                continue;
            }
            if let Some(wd) = w.component_mut(h.entity, &WOUND) {
                if wd[0] <= 0.0 {
                    // A fresh wound: its first spot where it was hit.
                    *wd = [secs, f64::MAX, f64::MAX, 0.0];
                } else {
                    wd[0] = wd[0].max(secs);
                }
                weft_sdk::debug!("animal {} wounded ({:.2} of {:.1}): bleeds {:.0} s", h.entity, h.amount, st.max_health, wd[0]);
            }
        }
    });
}

/// Once a second (`secs` of them): a wound bleeds, taking health, and drops a spot where it has moved to (only while
/// ticking every second: far from players it bleeds unseen).
pub fn tick(w: &World, id: u64, st: &EntityState, secs: f32) {
    let Some(mut wd) = w.component(id, &WOUND).filter(|wd| wd[0] > 0.0) else { return };
    let bled = (secs as f64).min(wd[0]);
    wd[0] -= bled;
    let now = w.game_time();
    let moved = (st.pos.x - wd[1]).powi(2) + (st.pos.z - wd[2]).powi(2);
    let due = now as f64 >= wd[3] + (core::SPOT_EVERY_SECS * 20.0) as f64;
    // On the ground only (a bird in the air, a swimmer: none).
    if secs <= 2.0 && due && moved >= core::SPOT_STEP * core::SPOT_STEP && st.in_liquid == 0 && st.on_ground != 0 {
        let at = st.pos.y;
        let yaw = (now.wrapping_mul(2_654_435_761) % 360) as f32;
        let spot = w.spawn_with(&SPOT, [st.pos.x, at + 0.01, st.pos.z], &Spawn::new().yaw(yaw));
        if spot != 0 {
            w.set_scale(spot, core::SPOT_SCALE);
            if let Some(o) = w.component_mut(spot, &SPOT_OF) {
                *o = [id, now];
            }
        }
        (wd[1], wd[2], wd[3]) = (st.pos.x, st.pos.z, now as f64);
    }
    if let Some(c) = w.component_mut(id, &WOUND) {
        *c = wd;
    }
    if wd[0] <= 0.0 {
        weft_sdk::debug!("animal {id} stops bleeding at {:.1} of {:.1}", st.health, st.max_health);
    }
    w.damage(id, Damage::new(core::bleed_loss(st.max_health, bled as f32), BLEED.id()));
}

/// A spot's goal: it shrinks as it dries (`core::spot_size`) and goes, looked at every second.
fn dry(w: &World, id: u64, _st: &EntityState) -> bool {
    let now = w.game_time();
    if !(now + id).is_multiple_of(20) {
        return true;
    }
    let Some(o) = w.component(id, &SPOT_OF) else { return true };
    match core::spot_size(now.saturating_sub(o[1]) as f32 / 20.0) {
        // One the world kept past a reload has no maker: it goes.
        Some(s) if o[1] != 0 => w.set_scale(id, s),
        _ => w.despawn(id),
    }
    true
}

/// For hungry hunter `id` of `own` kg at `pos`: the next spot of a blood trail it follows, fresher than the last one
/// it went to (`after`, a game tick), and when that fell. A trail of prey it takes (`takes`), or of one that bled out
/// (its carcass at the end); never its own kind's.
pub fn trail(w: &World, id: u64, pos: Vec3, after: u64, takes: impl Fn(u64) -> bool) -> Option<([f64; 3], u64)> {
    let kind = w.entity(id)?.kind;
    let mut best: Option<([f64; 3], u64)> = None;
    for s in w.entities_near(pos, core::TRAIL_SMELL, Some(&SPOT)) {
        let (Some(o), Some(e)) = (w.component(s, &SPOT_OF), w.entity(s)) else { continue };
        if o[1] <= after || best.is_some_and(|b| b.1 >= o[1]) {
            continue;
        }
        let follows = match w.entity(o[0]) {
            Some(b) => b.kind != kind && b.health > 0.0 && dist2(b.pos, pos) < core::TRAIL_REACH.powi(2) && takes(o[0]),
            None => true,
        };
        if follows {
            best = Some(([e.pos.x, e.pos.y, e.pos.z], o[1]));
        }
    }
    best
}

/// `/animalprof blood`: every wound and spot within 64 blocks of `at`.
pub fn report(w: &World, at: Vec3) -> String {
    let mut rows = Vec::new();
    for id in w.entities_near(at, 64.0, None) {
        if let Some(wd) = w.component(id, &WOUND).filter(|wd| wd[0] > 0.0)
            && let (Some(a), Some(e)) = (w.component(id, &ANIMAL), w.entity(id))
        {
            let name = a.species().map_or("?", |s| s.name);
            rows.push(format!("{name} {id} bleeds {:.0} s more, health {:.1}/{:.1}", wd[0], e.health, e.max_health));
        }
    }
    let spots = w.entities_near(at, 64.0, Some(&SPOT)).len();
    rows.push(format!("{spots} spots of blood"));
    rows.join(" | ")
}

//! Birds on the ground and in the air (`notes/ECOLOGY.md`, the living world's simple creatures):
//! what the raven (`raven.rs`), the small birds (`small_bird.rs`), the owls (`owl.rs`), the eagle
//! (`eagle.rs`), the vulture (`vulture.rs`) and the gull (`gull.rs`) share. Wild, never tamed. On
//! the ground a bird walks or hops about, foraging, and a hunter or a scavenger takes what it can
//! (`hunt::hunt`); a walker or a hunter near, or just now and then, it takes off (`fly`): up a few
//! blocks, a stretch through the air with its wings open (its flying model, a look of its own),
//! and down again somewhere else. They lay in nests at their kind of site (`eggs::lay_in_nest`,
//! `animals_core::nests`), and their young hatch there.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

use crate::dist2;

use crate::taming::TAMING;

/// One bird's body, for every species on its model.
pub struct Wings {
    /// Its model and animation files on the ground (`<model>`) and in the air (`<model>_fly`), and
    /// its texture.
    pub model: &'static str,
    pub texture: &'static str,
    /// Its box in the air, width and height in blocks (on the ground its model's:
    /// `animals_core::bodies`).
    pub fly_size: (f32, f32),
    pub health: f32,
    /// Its speed on the ground, and in the air as a multiple of it.
    pub speed: f32,
    pub air: f32,
    pub wander: f32,
    /// What its bite or talons do (0: it never fights).
    pub damage: f32,
    /// How high it climbs (blocks above where it took off, and a random share as much again), how
    /// far it flies (blocks, and as much again at random) and the ticks a flight may last.
    pub height: f64,
    pub reach: f64,
    pub flight: u32,
    /// One in how many ticks it takes off unprovoked.
    pub restless: u32,
}

value! {
    #[derive(Default)]
    /// A bird's flight (not saved: one loaded in the air falls and lands).
    pub struct Flight {
        pub flying: bool,
        /// Coming down (it lands when it touches the ground).
        pub landing: bool,
        /// Where it flies to, and the height it keeps until it gets there.
        pub to: [f32; 3],
        pub cruise: f32,
        /// Ticks left in the air.
        pub left: u32,
    }
}

static STATE: Component<Flight> = Component::named("bird_flight");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Flight::default(), 0);
}

fn fly_look(wings: &Wings) -> String {
    format!("{}_fly", wings.model)
}

/// Registers `kinds` (each a species' entity kind) on one body.
pub fn register(r: &mut Registry, kinds: &[&'static EntityKind], wings: &'static Wings) {
    // In the air: the wings open and beating.
    let look = fly_look(wings);
    drop(
        r.entity(EntityKind::generated(look.clone()))
            .size(wings.fly_size.0, wings.fly_size.1)
            .passive()
            .model_file(&look)
            .animations(&look)
            .texture(wings.texture),
    );
    let (bw, bh) = animals_core::bodies::of(wings.model);
    for &kind in kinds {
        let index = crate::species::of(kind);
        r.entity(kind)
            .size(bw, bh)
            .max_health(wings.health)
            .walk_speed(wings.speed)
            .attack_damage(wings.damage)
            .passive()
            .model_file(wings.model)
            .animations(wings.model)
            .texture(wings.texture)
            .attach(&STATE)
            .on_interact(|e| {
                let (w, id, player) = (e.world(), e.entity(), e.other());
                crate::screen::interact(&w, id, player);
                false
            })
            .goals(
                Goals::new()
                    .then(crate::animal::upkeep(index))
                    .then(move |w: &World, id: u64, st: &EntityState| fly(w, id, st, wings))
                    .then(crate::taming::behave(index))
                    .then(move |w: &World, id: u64, st: &EntityState| crate::hunt::wary(w, id, st) && take_off(w, id, st, wings))
                    .then(crate::hunt::hunt)
                    .then(crate::breeding::behave(index))
                    .then(crate::comfort::seek)
                    .then(crate::forage::forage)
                    .then(ai::wander(wings.wander, 8.0)),
            )
            .on_hurt(|w, h| crate::taming::hurt(w, h.entity));
    }
}

fn random(w: &World) -> f64 {
    w.random_below(10_000) as f64 / 10_000.0
}

/// Up into the air, away from the nearest player (if one is near, else anywhere).
fn take_off(w: &World, id: u64, st: &EntityState, wings: &Wings) -> bool {
    let from = w
        .players()
        .into_iter()
        .filter_map(|p| w.entity(p))
        .map(|p| p.pos)
        .min_by(|a, b| dist2(*a, st.pos).total_cmp(&dist2(*b, st.pos)))
        .filter(|p| dist2(*p, st.pos) < 24.0 * 24.0);
    let a = match from {
        Some(p) if dist2(p, st.pos) > 0.01 => (st.pos.z - p.z).atan2(st.pos.x - p.x) + (random(w) - 0.5),
        _ => random(w) * std::f64::consts::TAU,
    };
    let r = wings.reach * (1.0 + random(w));
    let to = [st.pos.x + a.cos() * r, st.pos.y, st.pos.z + a.sin() * r];
    set(
        w,
        id,
        Flight {
            flying: true,
            landing: false,
            to: to.map(|v| v as f32),
            cruise: (st.pos.y + wings.height * (1.0 + random(w))) as f32,
            left: wings.flight,
        },
    );
    w.set_appearance(id, Some(&format!("animals:{}", fly_look(wings))));
    if let Some(t) = w.component_mut(id, &TAMING) {
        t.scared = 0;
    }
    w.navigate(id, st.pos, 0.0);
    true
}

/// Every tick: take off when scared or now and then; in the air, fly to where it's going at its
/// height, then come down and land.
fn fly(w: &World, id: u64, st: &EntityState, wings: &Wings) -> bool {
    let Some(mut f) = w.component(id, &STATE) else { return false };
    if !f.flying {
        let scared = w.component(id, &TAMING).is_some_and(|t| t.scared > 0);
        if scared || (!w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(wings.restless) == 0) {
            return take_off(w, id, st, wings);
        }
        return false;
    }
    if f.landing && st.on_ground != 0 {
        // Down; in water it takes off again (only the gull swims, and not here).
        if st.in_liquid != 0 {
            return take_off(w, id, st, wings);
        }
        set(w, id, Flight::default());
        w.set_appearance(id, None);
        return false;
    }
    f.left = f.left.saturating_sub(w.tick_elapsed(id));
    let (dx, dz) = (f.to[0] as f64 - st.pos.x, f.to[2] as f64 - st.pos.z);
    let flat = (dx * dx + dz * dz).sqrt();
    if f.left == 0 || flat < 2.0 {
        f.landing = true;
    }
    let dir = if f.landing {
        // Gliding down, a little forward.
        let n = flat.max(1.0);
        [(dx / n * 0.3) as f32, -0.7, (dz / n * 0.3) as f32]
    } else {
        // Climbing to its height (straight up where something holds it back), then level.
        let blocked = f.left < wings.flight.saturating_sub(20) && st.vel.x.hypot(st.vel.z) < 0.05;
        let climb = if blocked { 1.5 } else { ((f.cruise as f64 - st.pos.y) / 4.0).clamp(-0.4, 0.8) };
        let n = (1.0 + climb * climb).sqrt();
        [(dx / flat / n) as f32, (climb / n) as f32, (dz / flat / n) as f32]
    };
    w.fly(id, dir, if f.landing { wings.air * 0.5 } else { wings.air });
    set(w, id, f);
    true
}

fn set(w: &World, id: u64, f: Flight) {
    if let Some(c) = w.component_mut(id, &STATE) {
        *c = f;
    }
}

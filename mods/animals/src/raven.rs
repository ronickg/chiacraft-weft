//! Ravens (`notes/ECOLOGY.md`, wave one: something for the dead in the cold biomes, where there are
//! no vultures): the common raven of the north's woods, mountains, tundra, desert and coast
//! (`source/raven`). Wild, never tamed. On the ground it hops and walks about, scavenging and now
//! and then taking a mouse or a vole (`hunt::hunt`); a walker or a hunter near, or just now and
//! then, it takes off (`fly`): up a few blocks, a stretch through the air with its wings open (its
//! flying model, a look of its own), and down again somewhere else. No nests yet (Ronald,
//! 2026-11-03): its young come out as fledglings (`Tune::gestation` in its row).

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::taming::TAMING;

pub(crate) static RAVEN: Entity = Entity::named("raven");
const INDEX: usize = 17;
const FLY: &str = "raven_fly";
/// Its speed in the air, as a multiple of its kind's (on the ground it walks and hops at about
/// 2.5 m/s; it flies at about 9).
const AIR_SPEED: f32 = 3.5;
/// Ticks a flight may last before it comes down where it is.
const FLIGHT: u32 = 400;

value! {
    #[derive(Default)]
    /// A raven's flight (not saved: one loaded in the air falls and lands).
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

static STATE: Component<Flight> = Component::named("raven_state");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Flight::default(), 0);
    // In the air: the wings open and beating.
    drop(
        r.entity(Entity::generated(FLY.to_string()))
            .size(0.4, 0.3)
            .passive()
            .model_file(FLY)
            .animations(FLY)
            .texture("raven/raven"),
    );
    r.entity(&RAVEN)
        .size(0.35, 0.35)
        .health(6.0)
        .speed(2.5)
        .attack_damage(1.0)
        .passive()
        .model_file("raven")
        .animations("raven")
        .texture("raven/raven")
        .attach(&STATE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            crate::screen::interact(&w, id, player);
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(INDEX))
                .then(fly)
                .then(crate::taming::behave(INDEX))
                .then(|w: &World, id: u64, st: &EntityState| crate::hunt::wary(w, id, st) && take_off(w, id, st, None))
                .then(crate::hunt::hunt)
                .then(crate::breeding::behave(INDEX))
                .then(crate::comfort::seek)
                .then(crate::forage::forage)
                .then(ai::wander(0.35, 8.0)),
        )
        .on_hurt(|w, h| crate::taming::hurt(w, h.entity));
}

fn random(w: &World) -> f64 {
    w.random_below(10_000) as f64 / 10_000.0
}

/// Up into the air, away from `from` (the nearest player when none is given, else anywhere).
fn take_off(w: &World, id: u64, st: &EntityState, from: Option<[f64; 3]>) -> bool {
    let from = from.or_else(|| {
        w.players()
            .into_iter()
            .filter_map(|p| w.entity(p))
            .map(|p| p.pos)
            .min_by(|a, b| dist2(*a, st.pos).total_cmp(&dist2(*b, st.pos)))
            .filter(|p| dist2(*p, st.pos) < 24.0 * 24.0)
    });
    let a = match from {
        Some(p) if dist2(p, st.pos) > 0.01 => (st.pos[2] - p[2]).atan2(st.pos[0] - p[0]) + (random(w) - 0.5),
        _ => random(w) * std::f64::consts::TAU,
    };
    let r = 16.0 + random(w) * 16.0;
    let to = [st.pos[0] + a.cos() * r, st.pos[1], st.pos[2] + a.sin() * r];
    set(
        w,
        id,
        Flight { flying: true, landing: false, to: to.map(|v| v as f32), cruise: (st.pos[1] + 6.0 + random(w) * 6.0) as f32, left: FLIGHT },
    );
    w.set_look(id, Some(&format!("animals:{FLY}")));
    if let Some(t) = w.component_mut(id, &TAMING) {
        t.scared = 0;
    }
    w.navigate(id, st.pos, 0.0);
    true
}

/// Every tick: take off when scared or now and then; in the air, fly to where it's going at its
/// height, then come down and land.
fn fly(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut f) = w.component(id, &STATE).copied() else { return false };
    if !f.flying {
        let scared = w.component(id, &TAMING).is_some_and(|t| t.scared > 0);
        if scared || (!w.navigating(id) && !crate::forage::busy(w, id) && w.random_below(2400) == 0) {
            return take_off(w, id, st, None);
        }
        return false;
    }
    if f.landing && st.on_ground != 0 {
        // Down; in water it takes off again (ravens don't swim).
        if st.in_liquid != 0 {
            return take_off(w, id, st, None);
        }
        set(w, id, Flight::default());
        w.set_look(id, None);
        return false;
    }
    f.left = f.left.saturating_sub(1);
    let (dx, dz) = (f.to[0] as f64 - st.pos[0], f.to[2] as f64 - st.pos[2]);
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
        let blocked = f.left < FLIGHT - 20 && st.vel[0].hypot(st.vel[2]) < 0.05;
        let climb = if blocked { 1.5 } else { ((f.cruise as f64 - st.pos[1]) / 4.0).clamp(-0.4, 0.8) };
        let n = (1.0 + climb * climb).sqrt();
        [(dx / flat / n) as f32, (climb / n) as f32, (dz / flat / n) as f32]
    };
    w.fly(id, dir, if f.landing { AIR_SPEED * 0.5 } else { AIR_SPEED });
    set(w, id, f);
    true
}

fn set(w: &World, id: u64, f: Flight) {
    if let Some(c) = w.component_mut(id, &STATE) {
        *c = f;
    }
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

//! Climbing any wall on stamina (`vitals::climb`, `notes/SYSTEMS.md`). The engine climbs
//! (`core:climb_speed` times the wall's `grip`, the `CLIMBING` pose while it holds on); here: each
//! wall's grip, the speed the body allows (what one carries, wet, cold), stamina by the second on
//! a wall, and the hands letting go when it runs out until some has come back.

use std::collections::HashMap;
use std::sync::Mutex;

use base_api::VITALS;
use vitals::climb::{self, GRIP_LOOSE, GRIP_NONE, GRIP_ROUGH, GRIP_SMOOTH};
use vitals::effects::{self, Effect};
use weft_sdk::*;

/// Held on to a wall (the engine's `pose_when::CLIMBING`): arms reaching, no bob.
pub static CLIMB: Pose = Pose::named("climb");

/// Each wall's grip (`vitals::climb`), by name or tag; the rest keep the engine's 1 (stone, dirt
/// that holds, bricks).
const GRIPS: &[(&str, f32)] = &[
    // Rough rock, raw faces and bark: holds everywhere.
    ("#logs", GRIP_ROUGH),
    ("#ores", GRIP_ROUGH),
    ("base:cobblestone", GRIP_ROUGH),
    ("base:mossy_cobblestone", GRIP_ROUGH),
    ("base:granite", GRIP_ROUGH),
    ("base:slate", GRIP_ROUGH),
    ("base:limestone", GRIP_ROUGH),
    ("base:weathered_limestone", GRIP_ROUGH),
    ("base:sandstone", GRIP_ROUGH),
    ("base:ochre_sandstone", GRIP_ROUGH),
    ("base:red_sandstone", GRIP_ROUGH),
    ("base:cream_sandstone", GRIP_ROUGH),
    ("base:banded_clay", GRIP_ROUGH),
    ("base:loose_rock", GRIP_ROUGH),
    // Dressed and planed faces: few holds.
    ("base:stone_bricks", GRIP_SMOOTH),
    ("#planks", GRIP_SMOOTH),
    // Loose ground crumbles under the hands.
    ("#sand", GRIP_LOOSE),
    ("#dirt", GRIP_LOOSE),
    ("base:gravel", GRIP_LOOSE),
    ("base:clay", GRIP_LOOSE),
    ("base:snow", GRIP_LOOSE),
    // Nothing to hold on to.
    ("#ice", GRIP_NONE),
    ("base:ice", GRIP_NONE),
    ("base:lake_ice", GRIP_NONE),
    ("base:glacier_ice", GRIP_NONE),
    ("base:glass", GRIP_NONE),
    ("base:red_stained_glass", GRIP_NONE),
    ("base:green_stained_glass", GRIP_NONE),
    ("base:blue_stained_glass", GRIP_NONE),
    ("base:impact_glass", GRIP_NONE),
];

pub fn register(r: &mut Registry) {
    r.pose(&CLIMB)
        .action("climb")
        .bob(false)
        .priority(40)
        .when(abi::pose_when::CLIMBING | abi::pose_when::NOT_FLYING);
    for &(what, grip) in GRIPS {
        r.set(what, &GRIP, grip);
    }
    r.world_tick(tick);
    // Stamina under the crosshair while on a wall: the HUD's own stamina bar (kit 2's, at its
    // size so its pixels stay whole), gone the moment the hands let go.
    r.hud_element(&HUD_CLIMB, hud_anchor::UNDER_CROSSHAIR, 5, |ui| {
        ui.progress_bind("hud_stamina", S::default().size(HINT.0, HINT.1), |c| {
            let on = c.player_state().is_some_and(|me| me.pose == CLIMB.id() && me.health > 0.0);
            let v = c.component(&VITALS).filter(|v| v.max_stamina > 0.0);
            match v {
                Some(v) if on && !c.outside_survival() => (3.0 + (v.stamina / v.max_stamina).clamp(0.0, 1.0) * 186.0) / 192.0,
                _ => -1.0,
            }
        });
    });
}

static HUD_CLIMB: HudElement = HudElement::named("climb_stamina");

/// The hint's size, GUI px: the HUD's stamina bar (192 x 19 texels, two a GUI pixel; its rim 3).
const HINT: (f32, f32) = (96.0, 9.5);

/// A climber as last seen: where (whether it moved on the wall), run out (until `climb::AGAIN`),
/// and the speed last set on the engine.
struct Hold {
    last: Vec3,
    spent: bool,
    speed: f32,
}

static HOLDS: Mutex<Option<HashMap<u64, Hold>>> = Mutex::new(None);

const DT: f32 = 1.0 / 20.0;

/// Whether the climber on the wall is moving (climbing up or down, along it), not hanging.
fn moving(a: Vec3, b: Vec3) -> bool {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);
    dx * dx + dy * dy + dz * dz > 0.02 * 0.02
}

fn tick(w: &World) {
    let mut holds = HOLDS.lock().unwrap();
    let holds = holds.get_or_insert_with(HashMap::new);
    let d = base_api::difficulty();
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        let h = holds.entry(id).or_insert(Hold { last: st.pos, spent: false, speed: f32::NAN });
        let on_wall = st.pose == CLIMB.id();
        let moved = moving(st.pos, h.last);
        h.last = st.pos;
        let speed = if w.outside_survival(id) {
            climb::SPEED
        } else {
            let Some(v) = w.component_mut(id, &VITALS) else { continue };
            let on: Vec<Effect> = v.effects().collect();
            let b = climb::burden(&on);
            if on_wall && st.health > 0.0 {
                let mods = effects::combine(v.effects());
                v.stamina = (v.stamina - climb::cost(moved, b) * DT * d.stamina_cost * mods.stamina_cost).max(0.0);
                v.rest_wait = (vitals::stamina::REGEN_DELAY * 20.0) as u32;
            }
            if v.stamina <= 0.0 {
                if !h.spent && on_wall {
                    weft_sdk::info!("climbing: {id} ran out of stamina at {:.1} {:.1} {:.1}, letting go", st.pos.x, st.pos.y, st.pos.z);
                }
                h.spent = true;
            } else if v.stamina >= climb::AGAIN {
                h.spent = false;
            }
            if h.spent || v.out != 0 || st.health <= 0.0 { 0.0 } else { climb::speed(b) }
        };
        // Respawning drops modifiers: said again now and then.
        if (speed - h.speed).abs() > 1e-3 || w.game_time().is_multiple_of(100) {
            h.speed = speed;
            w.attribute_add_modifier(id, &attribute::CLIMB_SPEED, "base:climbing", attr_op::ADD, speed as f64, 0);
        }
    }
}

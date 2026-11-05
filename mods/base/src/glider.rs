//! The glider (`vitals::glide`, `notes/SYSTEMS.md`): carried anywhere in the inventory, it opens
//! with jump in the air once falling and closes with jump again, on landing or in water. The
//! engine flies it (`core:glide_speed` and the rest, the `GLIDING` pose); here: the wing's
//! numbers, the air round it a few times a second (the day's wind from the weather, thermals over
//! sun-warmed ground, wind up a slope), its wear by the second aloft, and the open wing drawn over
//! the player (a model on the body, hidden on everyone not gliding) and from one's own eyes (the
//! frame across the top of the view).

use std::collections::HashMap;
use std::sync::Mutex;

use vitals::glide::{self, Ground};
use weft_sdk::*;

pub static GLIDER: Item = Item::named("glider");

/// Gliding (the engine's `pose_when::GLIDING`): upright, arms up on the bar, no bob.
pub static GLIDE: Pose = Pose::named("glide");

/// Jump pressed in the air, from the client: open or close it.
static TOGGLE: Channel<()> = Channel::named("glider_toggle");

/// The open wing's model on every player's body (`models/glider_wing.geo.json`), its root bone.
const WING_MODEL: &str = "base:glider_wing";
const WING_BONE: &str = "glider";

/// Blocks fallen before jump opens it: a jump off the ground doesn't.
const OPENS_AFTER: f32 = 0.75;

/// The modifiers' id on the engine's attributes. They last `HOLD` ticks unless said again (every
/// `glide::AIR_EVERY`), so a flight its region no longer sees (the player left) ends by itself.
const MOD: &str = "base:glider";
const HOLD: u64 = 40;

pub fn register(r: &mut Registry) {
    let wing = glide::HIDE;
    r.item(&GLIDER).max_stack(1).durability(wing.durability()).tab(&crate::tabs::TOOLS);
    // Leather stretched on a frame of sticks, lashed with string.
    r.shaped(&base_api::CRAFTING, "glider", 1, &["LLL", "sTs", "s s"], &[('L', "base:leather"), ('s', "#stick"), ('T', "#string")]);
    r.pose(&GLIDE)
        .action("glide")
        .bob(false)
        .priority(45)
        .when(abi::pose_when::GLIDING | abi::pose_when::NOT_FLYING);
    r.set_animations(&PLAYER, "base:player");
    r.bone_model(&PLAYER, WING_MODEL, "body");
    r.packet(&TOGGLE, |p, ()| toggle(&p.world(), p.player()));
    // Jump pressed in the world: the server decides whether it opens or closes anything.
    let was = std::sync::atomic::AtomicBool::new(false);
    r.client_frame(move |c| {
        let now = c.key_held(&client::keys::JUMP) && c.screen().is_empty();
        if now && !was.swap(true, std::sync::atomic::Ordering::Relaxed) {
            c.send(&TOGGLE, &());
        } else if !now {
            was.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        // The wing shows only on whoever glides.
        let glide = GLIDE.id();
        for id in c.entities() {
            if c.entity(id).is_some_and(|e| e.pose != glide) {
                c.bone_scale(id, WING_BONE, [0.0; 3]);
            }
        }
    });
    r.world_tick(tick);
}

/// One flight: where and when it began (for the log), ticks aloft toward the next wear.
struct Flight {
    speed: f32,
    sink: f32,
    from: Vec3,
    began: u64,
    aloft: u32,
    wind: (f32, f32),
}

static FLIGHTS: Mutex<Option<HashMap<u64, Flight>>> = Mutex::new(None);

fn flights<R>(f: impl FnOnce(&mut HashMap<u64, Flight>) -> R) -> R {
    f(FLIGHTS.lock().unwrap().get_or_insert_with(HashMap::new))
}

/// The first glider in `id`'s inventory that isn't worn out: its slot and wing.
fn carried(w: &World, id: u64) -> Option<(u32, &'static glide::Wing)> {
    let inv = w.inventory(id);
    (0..inv.len()).find_map(|i| {
        let s = inv.get(i);
        if s.is_empty() {
            return None;
        }
        let wing = glide::wing(item_name(s.item))?;
        let worn = DAMAGE.on_stack_own(&s).unwrap_or(0);
        (worn < wing.durability()).then_some((i, wing))
    })
}

fn toggle(w: &World, id: u64) {
    let Some(st) = w.entity(id) else { return };
    if flights(|f| f.contains_key(&id)) {
        close(w, id, &st, "closed");
        return;
    }
    if st.on_ground != 0 || st.in_liquid != 0 || st.flying != 0 || st.health <= 0.0 || st.fall_distance < OPENS_AFTER {
        return;
    }
    let Some((_, wing)) = carried(w, id) else { return };
    let wind = crate::weather::wind_at(w, block_pos_of(st.pos));
    weft_sdk::info!(
        "glider: {id} opens at {:.1} {:.1} {:.1}, wind {:.1} m/s to {:.0} degrees",
        st.pos.x,
        st.pos.y,
        st.pos.z,
        wind.0,
        wind.1
    );
    flights(|f| f.insert(id, Flight { speed: wing.speed, sink: wing.sink, from: st.pos, began: w.game_time(), aloft: 0, wind }));
    air(w, id, &st);
}

/// Closes `id`'s glider, logging the flight.
fn close(w: &World, id: u64, st: &EntityState, why: &str) {
    let Some(f) = flights(|f| f.remove(&id)) else { return };
    for a in [&attribute::GLIDE_SPEED, &attribute::GLIDE_SINK, &attribute::WIND_X, &attribute::WIND_Z, &attribute::LIFT] {
        w.attribute_remove_modifier(id, a, MOD);
    }
    let (dx, dz) = (st.pos.x - f.from.x, st.pos.z - f.from.z);
    let along = |deg: f32| {
        let (s, c) = (deg as f64).to_radians().sin_cos();
        dx * s - dz * c
    };
    weft_sdk::info!(
        "glider: {id} {why} after {:.1} blocks ({:.1} with the wind) and {:.1} down in {:.1} s, wind {:.1} m/s to {:.0} degrees, ratio {:.1}",
        (dx * dx + dz * dz).sqrt(),
        along(f.wind.1),
        f.from.y - st.pos.y,
        (w.game_time() - f.began) as f32 / 20.0,
        f.wind.0,
        f.wind.1,
        (dx * dx + dz * dz).sqrt() / (f.from.y - st.pos.y).max(0.1),
    );
}

fn tick(w: &World) {
    let open: Vec<u64> = flights(|f| f.keys().copied().collect());
    for id in open {
        // In another region (each ticks this), or gone: its modifiers run out by themselves.
        let Some(st) = w.entity(id) else { continue };
        if st.on_ground != 0 || st.in_liquid != 0 || st.flying != 0 || st.health <= 0.0 {
            close(w, id, &st, if st.on_ground != 0 { "landed" } else { "stopped" });
            continue;
        }
        let Some((slot, _)) = carried(w, id) else {
            close(w, id, &st, "lost its glider");
            continue;
        };
        // A second's wear a second aloft; worn out, it closes (the stack stays, broken).
        let wear = flights(|f| {
            let fl = f.get_mut(&id)?;
            fl.aloft += 1;
            (fl.aloft % 20 == 0).then_some(())
        });
        if wear.is_some() && !w.outside_survival(id) {
            let inv = w.inventory(id);
            let s = inv.get(slot);
            let worn = DAMAGE.on_stack_own(&s).unwrap_or(0) + 1;
            inv.set(slot, s.with(&DAMAGE, worn));
        }
        if w.game_time() % glide::AIR_EVERY == 0 {
            air(w, id, &st);
        }
    }
}

/// The air round a glider now: the wind it drifts with, and rising air under it.
fn air(w: &World, id: u64, st: &EntityState) {
    let at = block_pos_of(st.pos);
    let (speed, heading) = crate::weather::wind_at(w, at);
    let Some((wing_speed, sink)) = flights(|f| {
        f.get_mut(&id).map(|f| {
            f.wind = (speed, heading);
            (f.speed, f.sink)
        })
    }) else {
        return;
    };
    let blow = speed * glide::WIND_SHARE;
    let (s, c) = heading.to_radians().sin_cos();
    let (wx, wz) = (blow * s, -blow * c);
    let lift = rising(w, st.pos, (s, -c), blow).min(glide::LIFT_MAX);
    for (a, v) in [
        (&attribute::GLIDE_SPEED, wing_speed),
        (&attribute::GLIDE_SINK, sink),
        (&attribute::WIND_X, wx),
        (&attribute::WIND_Z, wz),
        (&attribute::LIFT, lift),
    ] {
        w.attribute_add_modifier(id, a, MOD, attr_op::ADD, v as f64, HOLD);
    }
}

/// The ground under a point: its height and kind, or `None` past `reach`.
fn ground(w: &World, from: Vec3, reach: f64) -> Option<(f64, Ground)> {
    let hit = w.raycast(from, Vec3::new(0.0, -1.0, 0.0), reach, true)?;
    let name = block_name(hit.state);
    let n = name.split_once(':').map_or(name, |(_, n)| n);
    let has = |k: &str| n.contains(k);
    let kind = if has("water") || has("ice") || has("snow") || has("leaves") {
        Ground::Cool
    } else if has("sand") || has("stone") || has("gravel") || has("granite") || has("slate") || has("rock") || has("clay") {
        Ground::Bare
    } else {
        Ground::Grass
    };
    Some((hit.pos.y as f64 + 1.0, kind))
}

/// Rising air at `pos`, blocks a second: a thermal over warm ground in the day's sun, and the
/// wind (`blow` blocks a second toward `dir`) forced up a slope ahead.
fn rising(w: &World, pos: Vec3, dir: (f32, f32), blow: f32) -> f32 {
    let Some((below, kind)) = ground(w, pos, 160.0) else { return 0.0 };
    let over = (pos.y - below) as f32;
    let mut lift = 0.0;
    // Ridge: the ground downwind rises toward the glider's height (a cliff in front counts whole).
    let look = glide::RIDGE_LOOK as f64;
    let ahead = Vec3::new(pos.x + dir.0 as f64 * look, pos.y, pos.z + dir.1 as f64 * look);
    let rise = if w.block_flags(block_pos_of(ahead)) & abi::block_flags::SOLID != 0 {
        glide::RIDGE_LOOK
    } else {
        ground(w, ahead, 160.0).map_or(0.0, |(h, _)| (h - below) as f32)
    };
    if rise > 0.0 && over < rise.max(0.0) + glide::RIDGE_REACH {
        lift += glide::ridge(blow, rise);
    }
    // Thermal: one cell in a few, its core near the cell's middle, by day and in no rain.
    let raining = w.raining_at(block_pos_of(pos));
    let (_, hour, _) = crate::weather::now(w);
    let sun = if raining { 0.0 } else { glide::sun(hour) };
    if sun > 0.0 && over < glide::THERMAL_REACH {
        let cell = glide::THERMAL_CELL;
        let (cx, cz) = ((pos.x as i32).div_euclid(cell), (pos.z as i32).div_euclid(cell));
        let day = w.day_time() / vitals::calendar::DAY_TICKS;
        let r = hash(cx, cz, day ^ w.seed());
        if (r & 0xFFFF) as f32 / 65536.0 < glide::THERMAL_SHARE {
            let mid = |c: i32, bits: u64| (c * cell) as f64 + cell as f64 * (0.3 + 0.4 * ((bits & 0xFF) as f64 / 255.0));
            let (mx, mz) = (mid(cx, r >> 16), mid(cz, r >> 24));
            let d = ((pos.x - mx).powi(2) + (pos.z - mz).powi(2)).sqrt() as f32;
            let core = (1.0 - d / (cell as f32 * 0.35)).max(0.0);
            lift += glide::THERMAL * sun * glide::warms(kind) * core;
        }
    }
    lift
}

fn hash(x: i32, z: i32, k: u64) -> u64 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (z as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ k;
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 29)
}

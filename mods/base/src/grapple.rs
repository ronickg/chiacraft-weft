//! The grappling hook (`vitals::grapple`, `notes/SYSTEMS.md`): use throws it along the look; it
//! catches on a solid block that gives a hold (its `grip` over 0) within reach, or falls back. The
//! engine holds the rope (`core:rope_length` and the rest, the `ROPE` pose: jump pulls in, sneak
//! lets out, gravity swings); here: the throw, climbing's stamina while hanging, letting go (use
//! again, out of stamina, the block gone), its wear by the throw, and the rope as every player sees
//! it (`ROPE`, synced to whoever sees the thrower; each client draws every rope as a cord of small
//! cubes with the hook at its end, and a missed throw going out and coming back).

use std::collections::HashMap;
use std::sync::Mutex;

use base_api::VITALS;
use vitals::climb;
use vitals::effects::{self, Effect};
use vitals::grapple;
use weft_sdk::*;

pub static HOOK: Item = Item::named("grappling_hook");

/// Hanging on the rope (the engine's `pose_when::ROPE`): arms up on it, no bob.
pub static HANG: Pose = Pose::named("hang");

value! {
    /// A player's rope as everyone sees it: on (1) with its anchor, or off; `throws` counts every
    /// throw so each client sees a new one, and `miss` is the last missed throw's way.
    pub struct Rope {
        pub on: u32,
        pub throws: u32,
        pub anchor: [f64; 3],
        pub miss: [f32; 3],
    }
}
pub static ROPE: Component<Rope> = Component::named("base:rope");

/// The engine's rope attributes last `HOLD` ticks unless said again (every `REFRESH`), so a rope
/// its region no longer sees (the player left) lets go by itself.
const MOD: &str = "base:grapple";
const HOLD: u64 = 40;
const REFRESH: u64 = 10;
/// Where the rope leaves the hands, over the feet (the engine's `ROPE_HOLD` × 1.8).
const HANDS: f64 = 1.9;
/// Let go when this much farther from the anchor than the rope is long (a teleport).
const SNAPS: f64 = 8.0;

pub fn register(r: &mut Registry) {
    r.entity_component(
        &ROPE,
        Rope { on: 0, throws: 0, anchor: [0.0; 3], miss: [0.0; 3] },
        component_flags::SYNC_OWNER | component_flags::SYNC_WATCHERS,
    );
    r.attach(&PLAYER, &ROPE);
    let hook = grapple::IRON;
    r.item(&HOOK).max_stack(1).durability(hook.durability()).tab(&crate::tabs::TOOLS).on_use(throw);
    // Three iron flukes on a shank, a cord of string.
    r.shaped(&base_api::CRAFTING, "grappling_hook", 1, &["I I", "SIS", " S "], &[('I', "base:iron_ingot"), ('S', "#string")]);
    r.pose(&HANG)
        .action("hang")
        .bob(false)
        .priority(42)
        .when(abi::pose_when::ROPE | abi::pose_when::NOT_FLYING);
    r.world_tick(tick);
    r.client_frame(draw);
}

/// A rope made fast: its anchor's block (it lets go when that is gone), anchor, length and pace.
#[derive(Clone, Copy)]
struct Line {
    block: BlockPos,
    anchor: Vec3,
    length: f64,
    reel: f32,
    last: Vec3,
}

#[derive(Default)]
struct State {
    lines: HashMap<u64, Line>,
    /// Ran out of stamina on a rope: no new hold until `grapple::AGAIN` is back.
    spent: Vec<u64>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    f(STATE.lock().unwrap().get_or_insert_with(State::default))
}

/// Whether `id` hangs on a rope now (the glider doesn't open on one).
pub fn attached(id: u64) -> bool {
    state(|s| s.lines.contains_key(&id))
}

const FACES: [[f64; 3]; 6] = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0, 1.0]];

fn throw(u: &mut ItemUse) -> bool {
    let w = u.world();
    let id = u.player();
    if attached(id) {
        let st = w.entity(id);
        release(&w, id, st.as_ref(), "let go");
        return true;
    }
    let Some(st) = w.entity(id) else { return false };
    let Some(hook) = grapple::hook(item_name(u.stack().item)) else { return false };
    if state(|s| s.spent.contains(&id)) {
        return true;
    }
    let dir = look_dir(st.yaw, st.pitch);
    let hit = w.raycast(st.eye(), dir, hook.reach as f64, false);
    let holds = |state: u32| state_flags(state) & abi::block_flags::SOLID != 0 && GRIP.on_block(state).unwrap_or(1.0) > 0.0;
    if !w.outside_survival(id) {
        u.set_stack(u.stack().add_wear(1));
    }
    let throws = w.component(id, &ROPE).map_or(0, |r| r.throws) + 1;
    match hit.filter(|h| holds(h.state)) {
        Some(h) => {
            let n = FACES[h.face.min(5) as usize];
            let anchor = Vec3::new(
                h.pos.x as f64 + 0.5 + n[0] * 0.52,
                h.pos.y as f64 + 0.5 + n[1] * 0.52,
                h.pos.z as f64 + 0.5 + n[2] * 0.52,
            );
            let hands = st.pos.offset(0.0, HANDS, 0.0);
            let length = ((anchor - hands).length_squared().sqrt() + 0.3).min(hook.reach as f64 + 2.0);
            weft_sdk::info!(
                "grapple: {id} caught on {} at {:.1} {:.1} {:.1}, {length:.1} blocks of rope",
                block_name(h.state),
                anchor.x,
                anchor.y,
                anchor.z
            );
            state(|s| {
                s.lines.insert(id, Line { block: h.pos, anchor, length, reel: hook.reel, last: st.pos });
            });
            say(&w, id, anchor, length, hook.reel);
            if let Some(r) = w.component_mut(id, &ROPE) {
                *r = Rope { on: 1, throws, anchor: [anchor.x, anchor.y, anchor.z], miss: [0.0; 3] };
            }
        }
        None => {
            let why = if hit.is_some() { "found no hold" } else { "reached nothing" };
            weft_sdk::info!("grapple: {id} missed ({why} within {} blocks)", hook.reach);
            if let Some(r) = w.component_mut(id, &ROPE) {
                *r = Rope { on: 0, throws, anchor: [0.0; 3], miss: [dir.x as f32, dir.y as f32, dir.z as f32] };
            }
        }
    }
    true
}

/// Tells the engine the rope (and again now and then, so it holds).
fn say(w: &World, id: u64, anchor: Vec3, length: f64, reel: f32) {
    for (a, v) in [
        (&attribute::ROPE_X, anchor.x),
        (&attribute::ROPE_Y, anchor.y),
        (&attribute::ROPE_Z, anchor.z),
        (&attribute::ROPE_LENGTH, length),
        (&attribute::ROPE_REEL, reel as f64),
    ] {
        w.attribute_add_modifier(id, a, MOD, attr_op::ADD, v, HOLD);
    }
}

fn release(w: &World, id: u64, st: Option<&EntityState>, why: &str) {
    let Some(line) = state(|s| s.lines.remove(&id)) else { return };
    for a in [&attribute::ROPE_X, &attribute::ROPE_Y, &attribute::ROPE_Z, &attribute::ROPE_LENGTH, &attribute::ROPE_REEL] {
        w.attribute_remove_modifier(id, a, MOD);
    }
    if let Some(r) = w.component_mut(id, &ROPE) {
        r.on = 0;
    }
    let at = st.map_or(line.last, |s| s.pos);
    weft_sdk::info!(
        "grapple: {id} {why} at {:.1} {:.1} {:.1}, {:.1} under the anchor",
        at.x,
        at.y,
        at.z,
        line.anchor.y - at.y
    );
}

const DT: f32 = 1.0 / 20.0;

fn tick(w: &World) {
    let d = base_api::difficulty();
    let ids: Vec<u64> = state(|s| s.lines.keys().copied().collect());
    for id in ids {
        // In another region (each ticks this), or gone: its attributes run out by themselves.
        let Some(st) = w.entity(id) else { continue };
        let Some(Line { block, anchor, length, reel, last }) = state(|s| s.lines.get(&id).copied()) else {
            continue;
        };
        if st.health <= 0.0 {
            release(w, id, Some(&st), "died");
            continue;
        }
        if state_flags(w.block(block)) & abi::block_flags::SOLID == 0 {
            release(w, id, Some(&st), "lost its hold (the block is gone)");
            continue;
        }
        if (st.pos.offset(0.0, HANDS, 0.0) - anchor).length_squared().sqrt() > length + SNAPS {
            release(w, id, Some(&st), "was pulled off the rope");
            continue;
        }
        // Climbing's stamina while hanging off the ground: dearer pulling in or out.
        if st.pose == HANG.id() && !w.outside_survival(id) {
            let moved = (st.pos - last).length_squared().sqrt() > 0.02;
            let Some(v) = w.component_mut(id, &VITALS) else { continue };
            let on: Vec<Effect> = v.effects().collect();
            let mods = effects::combine(v.effects());
            v.stamina = (v.stamina - climb::cost(moved, climb::burden(&on)) * DT * d.stamina_cost * mods.stamina_cost).max(0.0);
            v.rest_wait = (vitals::stamina::REGEN_DELAY * 20.0) as u32;
            if v.stamina <= 0.0 {
                state(|s| s.spent.push(id));
                release(w, id, Some(&st), "ran out of stamina");
                continue;
            }
        }
        state(|s| {
            if let Some(l) = s.lines.get_mut(&id) {
                l.last = st.pos;
            }
        });
        if w.game_time().is_multiple_of(REFRESH) {
            say(w, id, anchor, length, reel);
        }
    }
    // Hands back once some stamina is.
    let spent: Vec<u64> = state(|s| s.spent.clone());
    for id in spent {
        let back = w
            .component(id, &VITALS)
            .is_none_or(|v| v.stamina >= grapple::AGAIN);
        if back && w.entity(id).is_some() {
            state(|s| s.spent.retain(|&x| x != id));
        }
    }
}

// ------------------------------------------------------------------ every rope, on every client

/// The cord's cubes: their size and spacing (blocks), and its colours (twisted string, a darker
/// strand every fourth), the hook's iron.
const CORD: f64 = 0.0625;
const CORD_COLORS: [u32; 2] = [0x9C7A4CFF, 0x74583AFF];
const IRON: u32 = 0x5E6068FF;
const IRON_LIT: u32 = 0x8A8C94FF;

/// Throws seen per entity and when a missed one began (client side).
type Seen = HashMap<u64, (u32, Option<std::time::Instant>)>;
static SEEN: Mutex<Option<Seen>> = Mutex::new(None);

fn draw(c: &Client) {
    let st = c.state();
    let me = st.player;
    let first = st.camera == cameras::FIRST_PERSON.camera_id();
    let mut ids = c.entities();
    if !ids.contains(&me) {
        ids.push(me);
    }
    let mut seen = SEEN.lock().unwrap();
    let seen = seen.get_or_insert_with(HashMap::new);
    for id in ids {
        let rope = if id == me { c.component(&ROPE) } else { c.component_of(id, &ROPE) };
        let Some(rope) = rope else { continue };
        let Some(e) = c.entity(id) else { continue };
        // The hands: from one's own eyes low and to the right of the view; on a body over the
        // head while hanging, else at the shoulder.
        let hands = if id == me && first {
            let (s, co) = (st.yaw as f64).sin_cos();
            let right = Vec3::new(co, 0.0, -s);
            st.camera_pos.offset(right.x * 0.3, -0.35, right.z * 0.3)
        } else {
            let pos = if id == me { st.shown } else { e.pos };
            pos.offset(0.0, if e.pose == HANG.id() { HANDS } else { 1.4 }, 0.0)
        };
        let entry = seen.entry(id).or_insert((rope.throws, None));
        if entry.0 != rope.throws {
            *entry = (rope.throws, (rope.on == 0).then(std::time::Instant::now));
        }
        if rope.on != 0 {
            let anchor = Vec3::new(rope.anchor[0], rope.anchor[1], rope.anchor[2]);
            cord(c, hands, anchor);
            hook_at(c, anchor, anchor - hands);
        } else if let Some(t0) = entry.1 {
            // A miss: out to the hook's reach and back.
            let t = t0.elapsed().as_secs_f32() / grapple::MISS_SECS;
            if t >= 1.0 {
                entry.1 = None;
                continue;
            }
            let out = (1.0 - (2.0 * t - 1.0).abs()) as f64 * grapple::IRON.reach as f64;
            let dir = Vec3::new(rope.miss[0] as f64, rope.miss[1] as f64 - t as f64 * 0.6, rope.miss[2] as f64);
            let end = hands + dir * (out / dir.length_squared().sqrt().max(1e-6));
            cord(c, hands, end);
            hook_at(c, end, end - hands);
        }
    }
}

/// A cord of small cubes from `a` to `b`.
fn cord(c: &Client, a: Vec3, b: Vec3) {
    let d = b - a;
    let n = (d.length_squared().sqrt() / CORD).ceil().max(1.0) as usize;
    let h = CORD * 0.5;
    for i in 0..n {
        let p = a + d * ((i as f64 + 0.5) / n as f64);
        let color = CORD_COLORS[(i / 4) % 2];
        c.draw_fill(None, p.offset(-h, -h, -h), p.offset(h, h, h), color);
    }
}

/// The grapnel at the end: a shank along the rope and three flukes out round it.
fn hook_at(c: &Client, at: Vec3, along: Vec3) {
    let k = along * (1.0 / along.length_squared().sqrt().max(1e-6));
    let cube = |p: Vec3, r: f64, color: u32| c.draw_fill(None, p.offset(-r, -r, -r), p.offset(r, r, r), color);
    cube(at, 0.07, IRON_LIT);
    cube(at - k * 0.12, 0.05, IRON);
    // Flukes: three ways round the rope's line, bent back toward the thrower.
    let side = if k.y.abs() < 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let u = cross(k, side);
    let u = u * (1.0 / u.length_squared().sqrt().max(1e-6));
    let v = cross(k, u);
    for i in 0..3 {
        let a = i as f64 * std::f64::consts::TAU / 3.0;
        let out = u * a.cos() + v * a.sin();
        cube(at + out * 0.12 - k * 0.04, 0.04, IRON);
        cube(at + out * 0.2 - k * 0.12, 0.035, IRON_LIT);
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

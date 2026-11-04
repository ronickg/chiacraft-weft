//! A felled tree falls (Ronald, 11-05: "I want them to like fall down sideways and like the
//! leaves to sort of start breaking while it's falling. So like have an animation"). The tree's
//! own blocks are lifted out of the world into a moving structure (`World::lift`), which leans
//! over the stump's far edge away from the player (`World::body_turn`, the engine's pose) on
//! Panda's Falling Trees' curve: a creak, then an ease-in fall that starts slow and hits the
//! ground fastest, and a small bounce. As it goes its leaves break off, the fastest-moving (the
//! highest) first, as pieces of their own texture, some of them dropping their loot (saplings,
//! sticks, fruit); what is left bursts when it lands. Then the structure goes and the trunk lies
//! on the ground in pieces (`felled::lay`), or, past the axe's tier, shatters.
//!
//! The numbers are `vitals::felling`'s (`FALL_*`, `CREAK_*`, `BOUNCE_*`, `SHED_*`).

use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;
use std::sync::Mutex;

use vitals::felling as n;
use weft_sdk::*;

static BLOCK_BITS: Particle = Particle::named("core:block");
/// Its sounds in three beats (`reports/sound-files/fall_sounds.py`): the hinge creaking as it
/// starts to lean, the rush of its crown, the thud just before it lands.
static CREAK: Sound = Sound::named("tree.creak");
static RUSH: Sound = Sound::named("tree.fall");
static THUD: Sound = Sound::named("tree.thud");
static LOGS: Tag = Tag::named("logs");

/// A leaf's loot, by its block's name.
struct LootOf(&'static str);
impl LootSource for LootOf {
    fn table_name(&self) -> &str {
        self.0
    }
}

/// A tree on its way down.
struct Fall {
    dim: u32,
    body: u64,
    /// When it was lifted (world ticks).
    start: u64,
    foot: [i32; 3],
    d: [i32; 2],
    /// The point it turns about (world), and the axis (`up × d`).
    pivot: [f64; 3],
    axis: [f32; 3],
    /// Its logs and their states where they stood, for `felled::lay`.
    logs: Vec<[i32; 3]>,
    states: Vec<u32>,
    /// Its leaves still on it, the lowest first (shed from the end): where they stood, their state.
    leaves: Vec<([i32; 3], u32)>,
    leaves_at_start: usize,
    /// It lies down for its wood (within the axe's tier, in survival).
    lays: bool,
    /// Ticks of the fall itself, by its height.
    fall: u32,
    /// The pose last sent: how far over (degrees) it is told to be, and at which tick.
    sent_at: u64,
    /// Cells of the world its trunk has gone through, so each is struck once.
    struck: HashSet<[i32; 3]>,
    /// Creatures it has hit, each once.
    hurt: HashSet<u64>,
    /// Caught on a trunk it can't knock down: the angle it rests at, when, and the log holding it.
    hung: Option<(f32, u64, [i32; 3])>,
    /// Its rush and thud have sounded.
    rushed: bool,
    thudded: bool,
}

/// A tree that hung up on a trunk and settled there as blocks, leaning (`settle`): what holds it
/// up, its lowest log, the way it fell and whether it gives wood. When the log holding it goes
/// (chopped, or that tree falls), it comes down the rest of the way (`release`). Chopping it
/// at its lowest log fells it like any tree. Not saved: after a restart it is only a leaning tree.
struct Hung {
    dim: u32,
    holder: [i32; 3],
    base: [i32; 3],
    d: [i32; 2],
    lays: bool,
}

static HUNG: Mutex<Vec<Hung>> = Mutex::new(Vec::new());

static FALLING: Mutex<Vec<Fall>> = Mutex::new(Vec::new());
/// Falls started since the last tick (a tree struck by a falling one starts while the falls
/// are stepped): joined to `FALLING` at the next.
static STARTING: Mutex<Vec<Fall>> = Mutex::new(Vec::new());

/// The angle (degrees over) `t` ticks after the lift, for a fall of `fall` ticks.
pub fn angle(t: f32, fall: u32) -> f32 {
    let creak = n::CREAK_TICKS as f32;
    let fall = fall as f32;
    let bounce = n::BOUNCE_TICKS as f32;
    if t <= creak {
        n::CREAK_DEGREES * t / creak
    } else if t <= creak + fall {
        let s = (t - creak) / fall;
        n::CREAK_DEGREES + (90.0 - n::CREAK_DEGREES) * (1.0 - (s * FRAC_PI_2).cos())
    } else {
        // Up and back down once.
        let s = ((t - creak - fall) / bounce).min(1.0);
        90.0 - n::BOUNCE_DEGREES * (s * std::f32::consts::PI).sin()
    }
}

/// When the fall is over and the tree lies (ticks after the lift).
pub fn done_at(fall: u32) -> u32 {
    n::CREAK_TICKS + fall + n::BOUNCE_TICKS
}

/// The pivot (world) and the axis a tree falling from `foot` toward `d` turns about: the stump's
/// far edge, at the bottom of its base (its other logs at the foot's height), or at the cut's
/// top for a trunk of one (`felled::turned` lays it the same way).
pub fn pivot(foot: [i32; 3], logs: &[[i32; 3]], d: [i32; 2]) -> ([f64; 3], [f32; 3]) {
    let along = |p: &[i32; 3]| (p[0] - foot[0]) * d[0] + (p[2] - foot[2]) * d[1];
    let base: Vec<_> = logs.iter().filter(|p| p[1] == foot[1]).collect();
    let far = base.iter().map(|p| along(p)).max().unwrap_or(0).max(0) as f64;
    let y = foot[1] as f64 + if base.is_empty() { 1.0 } else { 0.0 };
    // The far edge of block `far` along d, from the foot block's middle.
    let edge = far + 0.5;
    let pivot = [foot[0] as f64 + 0.5 + d[0] as f64 * edge, y, foot[2] as f64 + 0.5 + d[1] as f64 * edge];
    (pivot, [d[1] as f32, 0.0, -d[0] as f32])
}

/// A quaternion (x, y, z, w) turning `degrees` about `axis` (a unit vector).
pub fn quat(axis: [f32; 3], degrees: f32) -> [f32; 4] {
    let h = degrees.to_radians() / 2.0;
    let s = h.sin();
    [axis[0] * s, axis[1] * s, axis[2] * s, h.cos()]
}

/// `p` (world) turned `degrees` about `axis` through `pivot`.
pub fn turn(p: [f64; 3], pivot: [f64; 3], axis: [f32; 3], degrees: f32) -> [f64; 3] {
    let (s, c) = (degrees.to_radians() as f64).sin_cos();
    let a = axis.map(|v| v as f64);
    let v = [p[0] - pivot[0], p[1] - pivot[1], p[2] - pivot[2]];
    let dot = a[0] * v[0] + a[1] * v[1] + a[2] * v[2];
    let cross = [a[1] * v[2] - a[2] * v[1], a[2] * v[0] - a[0] * v[2], a[0] * v[1] - a[1] * v[0]];
    // Rodrigues.
    std::array::from_fn(|i| pivot[i] + v[i] * c + cross[i] * s + a[i] * dot * (1.0 - c))
}

/// Starts a tree's fall: `logs` (highest first) and `leaves` are lifted out of the world. False
/// if nothing could be lifted (the caller then breaks the tree as before).
#[allow(clippy::too_many_arguments)]
pub fn start(
    w: &World,
    foot: [i32; 3],
    logs: Vec<[i32; 3]>,
    states: Vec<u32>,
    mut leaves: Vec<([i32; 3], u32)>,
    d: [i32; 2],
    lays: bool,
) -> bool {
    let mut cells: Vec<[i32; 3]> = logs.clone();
    cells.extend(leaves.iter().map(|l| l.0));
    let body = w.lift(foot, &cells);
    if body == 0 {
        return false;
    }
    release(w, &logs);
    let height = logs
        .iter()
        .map(|p| p[1] - foot[1])
        .max()
        .unwrap_or(1)
        .max(1);
    let fall = n::fall_ticks(height as f32);
    let (pivot, axis) = pivot(foot, &logs, d);
    // Shed from the end: the highest last in the list.
    leaves.sort_by_key(|l| l.0[1]);
    let leaves_at_start = leaves.len();
    let now = w.time();
    w.body_turn(body, pivot, quat(axis, n::CREAK_DEGREES), n::CREAK_TICKS);
    STARTING.lock().unwrap().push(Fall {
        dim: w.dimension(),
        body,
        start: now,
        foot,
        d,
        pivot,
        axis,
        logs,
        states,
        leaves,
        leaves_at_start,
        lays,
        fall,
        sent_at: now,
        struck: HashSet::new(),
        hurt: HashSet::new(),
        hung: None,
        rushed: false,
        thudded: false,
    });
    w.play_sound(&CREAK, pivot, n::sound_volume(height as f32));
    true
}

pub fn register(r: &mut Registry) {
    r.sound(&CREAK)
        .files(&["tree/creak1", "tree/creak2"])
        .category(&sound_categories::BLOCKS)
        .pitch(0.85, 1.1)
        .range(48.0);
    r.sound(&RUSH)
        .files(&["tree/fall1", "tree/fall2"])
        .category(&sound_categories::BLOCKS)
        .pitch(0.9, 1.1)
        .range(48.0);
    r.sound(&THUD)
        .files(&["tree/thud1", "tree/thud2"])
        .category(&sound_categories::BLOCKS)
        .pitch(0.85, 1.05)
        .range(64.0);
    r.world_tick(|w| {
        let (dim, now) = (w.dimension(), w.time());
        let mut falling = FALLING.lock().unwrap();
        falling.append(&mut STARTING.lock().unwrap());
        if falling.is_empty() {
            return;
        }
        falling.retain_mut(|f| f.dim != dim || step(w, f, now));
    });
}

/// One tick of a fall; false once it lies.
fn step(w: &World, f: &mut Fall, now: u64) -> bool {
    // Caught on a trunk: it rests there a moment, then settles as blocks, leaning.
    if let Some((deg, at, holder)) = f.hung {
        if now < at + n::HANG_SETTLE_TICKS as u64 {
            return true;
        }
        settle(w, f, deg, holder);
        return false;
    }
    let t = now.saturating_sub(f.start) as f32;
    let end = done_at(f.fall) as f32;
    // The pose, a few ticks ahead, every few ticks (clients ease between).
    if now >= f.sent_at + n::POSE_EVERY as u64 && t < end {
        let ahead = n::POSE_EVERY;
        w.body_turn(f.body, f.pivot, quat(f.axis, angle(t + ahead as f32, f.fall)), ahead);
        f.sent_at = now;
    }
    let deg = angle(t, f.fall);
    // The rush as it goes over, from its crown; the thud a little before it lands, where it does.
    let height = f.logs.iter().map(|p| p[1] - f.foot[1]).max().unwrap_or(1) as f32;
    let crown = [f.foot[0], f.foot[1] + height as i32, f.foot[2]];
    if !f.rushed && t >= n::CREAK_TICKS as f32 {
        f.rushed = true;
        w.play_sound(&RUSH, turn(center(crown), f.pivot, f.axis, deg), n::sound_volume(height));
    }
    if !f.thudded && t >= (n::CREAK_TICKS + f.fall) as f32 - n::THUD_EARLY_TICKS as f32 {
        f.thudded = true;
        w.play_sound(&THUD, turn(center(crown), f.pivot, f.axis, 90.0), n::sound_volume(height));
    }
    // Leaves break off as it goes: the share shed grows with the angle past SHED_FROM.
    let past = ((deg - n::SHED_FROM) / (90.0 - n::SHED_FROM)).clamp(0.0, 1.0);
    let shed_by_now = (f.leaves_at_start as f32 * n::SHED_SHARE * past) as usize;
    let mut shed = f.leaves_at_start - f.leaves.len();
    let speed = (angle(t + 1.0, f.fall) - deg).to_radians() * vitals::TICKS as f32;
    while shed < shed_by_now {
        let Some((p, state)) = f.leaves.pop() else { break };
        shed += 1;
        let local = [p[0] - f.foot[0], p[1] - f.foot[1], p[2] - f.foot[2]];
        w.set_structure_block(f.body, local, 0);
        leaf_off(w, f, p, state, deg, speed);
    }
    if deg > n::KNOCK_FROM && deg < 90.0 {
        hurt(w, f, deg);
    }
    if deg > n::KNOCK_FROM {
        if let Some(holder) = knock(w, f, deg) {
            // It leans back a little onto what caught it and rests there.
            let rest = (deg - n::HANG_BACK).max(n::KNOCK_FROM);
            w.body_turn(f.body, f.pivot, quat(f.axis, rest), n::HANG_SETTLE_TICKS / 2);
            f.hung = Some((rest, now, holder));
            return true;
        }
    }
    if t < end {
        return true;
    }
    // Down: what is left of the crown bursts, the structure goes, the trunk lies.
    for &(p, state) in &f.leaves {
        if w.random_below(n::SHED_BURST_ONE_IN) == 0 {
            leaf_off(w, f, p, state, 90.0, 0.0);
        }
    }
    w.remove(f.body);
    if f.lays {
        crate::felled::lay(w, f.foot, &f.logs, &f.states, f.d);
    } else {
        // Past the axe's tier it shatters where it lands.
        for (p, &state) in f.logs.iter().zip(&f.states).step_by(2) {
            let at = turn(center(*p), f.pivot, f.axis, 90.0);
            w.spawn_particles(&BLOCK_BITS, at)
                .block(state)
                .count(6)
                .spread(0.4)
                .outward();
        }
    }
    false
}

/// What the falling tree strikes now, `deg` over (Ronald, 11-05: "even if the tree falls, it
/// would be nice if it can break some other trees or partially"): each of its logs (up to
/// `KNOCK_SAMPLES` of them) is where it has turned to, and what stands in that cell of the world
/// takes the blow once: leaves and plants break, a branch log (one over air) breaks off, and
/// another tree's trunk falls too, the same way, if this tree has at least `KNOCK_FELLS` times
/// its logs (Valheim's chain felling). A trunk too big for that is passed through.
fn knock(w: &World, f: &mut Fall, deg: f32) -> Option<[i32; 3]> {
    let step = (f.logs.len() / n::KNOCK_SAMPLES).max(1);
    let mut hit = Vec::new();
    for p in f.logs.iter().step_by(step) {
        let at = turn(center(*p), f.pivot, f.axis, deg);
        let cell = at.map(|v| v.floor() as i32);
        if f.struck.insert(cell) {
            hit.push(cell);
        }
    }
    for cell in hit {
        let state = w.get(cell);
        if state == 0 || crate::felled::piece_of(state).is_some() {
            continue;
        }
        if crate::leaf_decay::decays(w, cell) || HARDNESS.on_block(state).is_some_and(|h| h <= 0.0) {
            w.break_block(cell, ItemStack::EMPTY, 0, true);
            continue;
        }
        if !LOGS.on_block(state) {
            continue;
        }
        // Down its column to where it stands: on the ground (a trunk) or on air (a branch).
        let mut foot = cell;
        while LOGS.on_block(w.get([foot[0], foot[1] - 1, foot[2]])) {
            foot[1] -= 1;
        }
        if w.get([foot[0], foot[1] - 1, foot[2]]) == 0 {
            w.break_block(cell, ItemStack::EMPTY, 0, true);
            continue;
        }
        let Some(logs) = crate::felling::tree_on(w, foot, w.get(foot)) else { continue };
        // Too big to knock down: it catches there.
        if f.logs.len() < logs.len() * n::KNOCK_FELLS {
            return Some(cell);
        }
        let states: Vec<u32> = logs.iter().map(|&p| w.get(p)).collect();
        let leaves = crate::felling::crown(w, &logs);
        if start(w, foot, logs.clone(), states, leaves, f.d, f.lays) {
            crate::leaf_decay::logs_gone(w, logs.iter().copied());
        }
    }
    None
}

/// Creatures in the falling trunk's way (Planner's numbers: 2 + a tenth of its logs, up to 20),
/// each hit once while it falls.
fn hurt(w: &World, f: &mut Fall, deg: f32) {
    let step = (f.logs.len() / n::KNOCK_SAMPLES).max(1);
    let harm = n::hurt(f.logs.len() as u32);
    for p in f.logs.iter().step_by(step) {
        let at = turn(center(*p), f.pivot, f.axis, deg);
        for id in w.near_bodies(at, n::HURT_REACH) {
            if id != f.body && f.hurt.insert(id) {
                w.damage(id, harm, 0, abi::damage::CRUSH);
            }
        }
    }
}

/// A caught tree settles where it rests, `deg` over: the structure goes and its logs and what is
/// left of its crown are put back into the world where they are now, a block each (a leaning
/// trunk of blocks, corners joined), wherever there is room. It is then a tree like any other,
/// chopped at its lowest log; and `holder` going brings it down (`release`).
fn settle(w: &World, f: &Fall, deg: f32, holder: [i32; 3]) {
    w.remove(f.body);
    // Upright under 45 degrees, lying along the fall past it.
    let axis = if deg < 45.0 {
        "y"
    } else if f.d[0] != 0 {
        "x"
    } else {
        "z"
    };
    let mut base: Option<[i32; 3]> = None;
    let mut put = |p: [i32; 3], state: u32| {
        let cell = turn(center(p), f.pivot, f.axis, deg).map(|v| v.floor() as i32);
        let here = w.get(cell);
        if here == 0 || crate::leaf_decay::decays(w, cell) || HARDNESS.on_block(here).is_some_and(|h| h <= 0.0) {
            w.set(cell, state);
            Some(cell)
        } else {
            None
        }
    };
    for (&p, &state) in f.logs.iter().zip(&f.states) {
        let state = state_with(state, "axis", axis).unwrap_or(state);
        if let Some(cell) = put(p, state) {
            if base.is_none_or(|b| cell[1] < b[1]) {
                base = Some(cell);
            }
        }
    }
    for &(p, state) in &f.leaves {
        put(p, state);
    }
    if let Some(base) = base {
        HUNG.lock()
            .unwrap()
            .push(Hung { dim: w.dimension(), holder, base, d: f.d, lays: f.lays });
    }
}

/// The logs at `gone` went (chopped, or lifted by a fall): a hung tree held by one of them comes
/// down the rest of the way, from its lowest log, the way it was falling.
pub fn release(w: &World, gone: &[[i32; 3]]) {
    let dim = w.dimension();
    let freed: Vec<Hung> = {
        let mut hung = HUNG.lock().unwrap();
        if hung.is_empty() {
            return;
        }
        let gone: HashSet<[i32; 3]> = gone.iter().copied().collect();
        let (freed, kept) = std::mem::take(&mut *hung)
            .into_iter()
            .partition(|h| h.dim == dim && gone.contains(&h.holder));
        *hung = kept;
        freed
    };
    for h in freed {
        let state = w.get(h.base);
        let Some(mut logs) = crate::felling::loose(w, h.base, state) else { continue };
        logs.push(h.base);
        let states: Vec<u32> = logs.iter().map(|&p| w.get(p)).collect();
        let leaves = crate::felling::crown(w, &logs);
        if start(w, h.base, logs.clone(), states, leaves, h.d, h.lays) {
            crate::leaf_decay::logs_gone(w, logs.iter().copied());
        }
    }
}

fn center(p: [i32; 3]) -> [f64; 3] {
    [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5]
}

/// A leaf that stood at `p` breaks off the tree now `deg` over, turning at `speed` (radians a
/// second): pieces of it fly on along the fall, and sometimes its loot drops there.
fn leaf_off(w: &World, f: &Fall, p: [i32; 3], state: u32, deg: f32, speed: f32) {
    let at = turn(center(p), f.pivot, f.axis, deg);
    // Moving along the arc: the speed times how far it is from the pivot, toward the fall.
    let r = ((at[0] - f.pivot[0]).powi(2) + (at[1] - f.pivot[1]).powi(2) + (at[2] - f.pivot[2]).powi(2)).sqrt() as f32;
    let ahead = turn(center(p), f.pivot, f.axis, deg + 1.0);
    let len = (((ahead[0] - at[0]).powi(2) + (ahead[1] - at[1]).powi(2) + (ahead[2] - at[2]).powi(2)).sqrt() as f32).max(1e-3);
    let v = speed * r * 0.5;
    let dir = [(ahead[0] - at[0]) as f32 / len, (ahead[1] - at[1]) as f32 / len, (ahead[2] - at[2]) as f32 / len];
    w.spawn_particles(&BLOCK_BITS, at)
        .block(state)
        .count(n::SHED_BITS)
        .spread(0.45)
        .velocity(dir.map(|c| c * v))
        .speed(1.0);
    if w.random_below(n::SHED_LOOT_ONE_IN) == 0 {
        // The block's name, without the state's props: its loot table.
        let name = name_of(abi::entry_kind::BLOCK, block_of(state))
            .split('[')
            .next()
            .unwrap_or("");
        for stack in w.roll_loot(&LootOf(name), &LootCtx { mined: state, ..LootCtx::default() }) {
            w.spawn_item(at, stack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creaks_falls_ever_faster_and_bounces_once() {
        let fall = n::fall_ticks(10.0);
        assert_eq!(angle(0.0, fall), 0.0);
        assert!((angle(n::CREAK_TICKS as f32, fall) - n::CREAK_DEGREES).abs() < 1e-4);
        let end_fall = (n::CREAK_TICKS + fall) as f32;
        assert!((angle(end_fall, fall) - 90.0).abs() < 1e-3);
        // Faster at the end than at the start.
        let early = angle(n::CREAK_TICKS as f32 + 2.0, fall) - angle(n::CREAK_TICKS as f32, fall);
        let late = angle(end_fall, fall) - angle(end_fall - 2.0, fall);
        assert!(late > early * 4.0);
        // The bounce lifts it and lays it back.
        let mid = end_fall + n::BOUNCE_TICKS as f32 / 2.0;
        assert!((angle(mid, fall) - (90.0 - n::BOUNCE_DEGREES)).abs() < 1e-3);
        assert!((angle(done_at(fall) as f32, fall) - 90.0).abs() < 1e-3);
    }

    #[test]
    fn the_pivot_turns_the_trunk_onto_where_it_is_laid() {
        // A trunk of one falling toward +x turns about the cut's top at the foot's far edge, and
        // each log comes down where `felled::turned` lays it (a block up, before it settles).
        let foot = [0, 0, 0];
        let logs = [[0, 1, 0], [0, 2, 0], [0, 3, 0]];
        let (pv, axis) = pivot(foot, &logs, [1, 0]);
        assert_eq!(pv, [1.0, 1.0, 0.5]);
        let laid = crate::felled::turned(foot, &logs, [1, 0]);
        for (p, l) in logs.iter().zip(&laid) {
            let at = turn(center(*p), pv, axis, 90.0);
            let (spot, _) = l.unwrap();
            assert!((at[0] - (spot[0] as f64 + 0.5)).abs() < 1e-9, "{at:?} {spot:?}");
            assert!((at[1] - (spot[1] as f64 + 1.5)).abs() < 1e-9, "a block up: {at:?} {spot:?}");
            assert!((at[2] - (spot[2] as f64 + 0.5)).abs() < 1e-9);
        }
        // A 2 x 2 falling toward -z turns about the bottom of its base's far edge, and lands
        // exactly where it is laid.
        let logs = [[0, 0, -1], [0, 1, 0], [0, 1, -1], [1, 1, 0]];
        let (pv, axis) = pivot(foot, &logs, [0, -1]);
        assert_eq!(pv, [0.5, 0.0, -1.0]);
        let laid = crate::felled::turned(foot, &logs, [0, -1]);
        for (p, l) in logs.iter().zip(&laid) {
            let at = turn(center(*p), pv, axis, 90.0);
            let (spot, _) = l.unwrap();
            let want = [spot[0] as f64 + 0.5, spot[1] as f64 + 0.5, spot[2] as f64 + 0.5];
            assert!((0..3).all(|i| (at[i] - want[i]).abs() < 1e-9), "{p:?}: {at:?} vs {want:?}");
        }
    }
}

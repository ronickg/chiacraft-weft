//! A big meteorite's live crater (Research round 17, addendum 2): a bowl carved into the land
//! where it lands, a rim of thrown-out rock round it, the bowl lined with melt and the grass
//! round scorched. Spread over `IMPACT_CARVE_TICKS` ticks, a column at a time from the middle
//! out, each column read and set in the tick it is carved. The shape's numbers are
//! `vitals::impact`'s; when and where one falls is the sky events' (First hours).
//!
//! `/impact [radius] [ticks] [x] [z]` carves one now (200 blocks north of the player when no
//! place is given), for the looks and the bench.

use std::sync::Mutex;

use vitals::impact as n;
use weft_sdk::*;

use crate::blocks::{COBBLESTONE, DIRT, GRASS, IMPACT_GLASS, SNOWY_GRASS, SUEVITE};

static COARSE_DIRT: Block = Block::named("coarse_dirt");
/// First hours' block (the sky events); a crater holds none until it is there.
static METEORITE: Block = Block::named("base:meteorite");
static LOGS: Tag = Tag::named("logs");
static LEAVES: Tag = Tag::named("leaves");

/// A crater being carved: its middle (blocks, the ground's top there), its shape, and the
/// columns still to carve, nearest the middle first.
struct Carve {
    dim: u32,
    at: [i32; 3],
    radius: i32,
    rim: i32,
    seed: u64,
    /// (dx, dz) still to carve, farthest first (taken from the back).
    columns: Vec<(i32, i32)>,
    /// Block edits a tick.
    budget: usize,
    /// Edits made so far, columns skipped (not loaded or water).
    edits: usize,
    skipped: usize,
    /// The core: (dx, dz, blocks under the bowl's lining) and its block; the rim's meteorites:
    /// (dx, dz) and theirs. No block (`u32::MAX`), none placed.
    core: Vec<(i32, i32, i32)>,
    core_state: u32,
    rim_spots: Vec<(i32, i32)>,
    rim_state: u32,
    /// Called on each core block once placed (First hours' `meteors::hot`: heat 3, cooling).
    hot: Option<fn(&World, [i32; 3])>,
}

/// What a crater holds: the core's block (full name) and how many, and the meteorites in the
/// rim. A block that isn't registered is left out.
#[derive(Clone, Copy, Debug)]
pub struct Loot {
    pub core: &'static str,
    pub core_count: i32,
    pub rim_count: i32,
    /// Called on each core block once it is placed: First hours' `meteors::hot` sets it hot and
    /// cooling. None leaves the block as placed.
    pub hot: Option<fn(&World, [i32; 3])>,
}

impl Loot {
    /// Meteorites only, `IMPACT_CORE` in the core times `share` (the sky events' share for the
    /// players nearby, 1 alone) and `IMPACT_RIM_METEORITES` in the rim, picked by `seed`.
    pub fn meteorites(seed: u64, share: f64) -> Loot {
        let pick = |v: u64, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u64) as i32;
        Loot {
            core: "base:meteorite",
            core_count: (pick(seed >> 8, n::IMPACT_CORE) as f64 * share.max(1.0)).round() as i32,
            rim_count: pick(seed >> 24, n::IMPACT_RIM_METEORITES),
            hot: None,
        }
    }
}

/// The state of block `name` (full name), `u32::MAX` while nothing registers it.
fn state_named(name: &'static str) -> u32 {
    if name == METEORITE.name() { METEORITE.id() } else { Block::generated(name.to_string()).id() }
}

static CARVES: Mutex<Vec<Carve>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    r.world_tick(tick);
    r.command("impact", "[radius:int] [ticks:int] [x:int] [z:int]", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let radius = cmd
            .int("radius")
            .map_or(n::IMPACT_RADIUS.1, |r| (r as i32).clamp(n::IMPACT_RADIUS.0, n::IMPACT_RADIUS.1));
        let ticks = cmd
            .int("ticks")
            .map_or(n::IMPACT_CARVE_TICKS, |t| (t as u32).clamp(1, 200));
        let (px, pz) = (st.pos.x.floor() as i32, st.pos.z.floor() as i32);
        let x = cmd.int("x").map_or(px, |v| v as i32);
        let z = cmd.int("z").map_or(pz - 200, |v| v as i32);
        match start(&w, x, z, radius, ticks) {
            Some(columns) => cmd.reply("command.base.impact.started", &[Arg::Int(radius as i64), Arg::Int(x as i64), Arg::Int(z as i64), Arg::Int(columns as i64), Arg::Int(ticks as i64)]),
            None => cmd.reply("command.base.impact.no_ground", &[Arg::Int(x as i64), Arg::Int(z as i64)]),
        }
    });
}

/// Starts a crater of `radius` at (`x`, `z`), carved over `ticks` ticks, holding meteorites for
/// one player (`Loot::meteorites`); the number of columns it will carve, or None where the
/// middle isn't loaded land.
pub fn start(w: &World, x: i32, z: i32, radius: i32, ticks: u32) -> Option<usize> {
    start_with(w, x, z, radius, ticks, None)
}

/// `start` with what the crater holds (`loot`; None for `Loot::meteorites` alone).
pub fn start_with(w: &World, x: i32, z: i32, radius: i32, ticks: u32, loot: Option<Loot>) -> Option<usize> {
    let guess = base_api::land::height(&base_api::land::fields(w.seed()), x * 16, z * 16) >> 4;
    let y = ground(w, x, z, guess)?;
    let seed = w.seed_at([x, y, z]);
    let rim = n::IMPACT_RIM.0 + (seed % (n::IMPACT_RIM.1 - n::IMPACT_RIM.0 + 1) as u64) as i32;
    let out = (radius as f64 * n::IMPACT_SCORCH).ceil() as i32;
    let mut columns: Vec<(i32, i32)> = (-out..=out)
        .flat_map(|dz| (-out..=out).map(move |dx| (dx, dz)))
        .filter(|&(dx, dz)| dx * dx + dz * dz <= out * out)
        .collect();
    // Farthest first, so the middle comes off the back first.
    columns.sort_by_key(|&(dx, dz)| std::cmp::Reverse(dx * dx + dz * dz));
    // Edits it will make on level ground: the bowl's depth at each column, the rim's height,
    // a lining or top each; spread evenly.
    let total: f64 = columns
        .iter()
        .map(|&(dx, dz)| {
            let u = ((dx * dx + dz * dz) as f64).sqrt() / radius as f64;
            (surface(u, radius, rim) as f64).abs() + 1.0
        })
        .sum();
    let budget = (total / ticks as f64).ceil() as usize;
    let count = columns.len();
    let loot = loot.unwrap_or_else(|| Loot::meteorites(seed, 1.0));
    // The core in a 3 by 3 under the middle, layer by layer; the rim's meteorites round the
    // crest, a little outside it, in its rubble.
    const ROUND: [(i32, i32); 9] = [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)];
    let core = (0..loot.core_count.max(0))
        .map(|i| {
            let (dx, dz) = ROUND[(i % n::IMPACT_CORE_LAYER) as usize % ROUND.len()];
            (dx, dz, 2 + i / n::IMPACT_CORE_LAYER)
        })
        .collect();
    let mut rim_spots: Vec<(i32, i32)> = Vec::new();
    for i in 0..loot.rim_count.max(0) as i64 {
        let h = weft_sdk::noise::hash(seed, i, 7, 0);
        let a = (h % 3600) as f64 / 3600.0 * std::f64::consts::TAU;
        let u = 1.0 + ((h >> 16) % 30) as f64 / 100.0;
        let spot = ((a.cos() * u * radius as f64).round() as i32, (a.sin() * u * radius as f64).round() as i32);
        if !rim_spots.contains(&spot) {
            rim_spots.push(spot);
        }
    }
    let (core_state, rim_state) = (state_named(loot.core), METEORITE.id());
    CARVES.lock().unwrap().push(Carve {
        dim: w.dimension(),
        at: [x, y, z],
        radius,
        rim,
        seed,
        columns,
        budget,
        edits: 0,
        skipped: 0,
        core,
        core_state,
        rim_spots,
        rim_state,
        hot: loot.hot,
    });
    Some(count)
}

/// The crater's ground at `u` radii from its middle, over (+) or under (-) the land there
/// (blocks): the bowl down to `IMPACT_DEPTH` radii at the middle rising steeply into the rim's
/// crest at one radius, then the rim falling back into the land by `IMPACT_RIM_OUT`; worldgen's
/// craters' shape (`base_api::land`), smaller.
fn surface(u: f64, radius: i32, rim: i32) -> i32 {
    let depth = radius as f64 * n::IMPACT_DEPTH;
    let out = n::IMPACT_RIM_OUT;
    let h = if u < 1.0 {
        -depth * (1.0 - u * u) + rim as f64 * u.powi(6)
    } else if u < out {
        let v = (out - u) / (out - 1.0);
        rim as f64 * v * v
    } else {
        0.0
    };
    h.round() as i32
}

/// The top of the ground at (`x`, `z`) near `guess`: the highest full block that isn't a tree.
/// None where it isn't loaded, or water lies on it.
fn ground(w: &World, x: i32, z: i32, guess: i32) -> Option<i32> {
    for y in (guess - 24..=guess + 24).rev() {
        let s = w.block_loaded([x, y, z])?;
        if s == 0 {
            continue;
        }
        let flags = state_flags(s);
        if flags & abi::block_flags::LIQUID != 0 {
            return None;
        }
        if flags & abi::block_flags::OPAQUE != 0 && !LOGS.on_block(s) && !LEAVES.on_block(s) {
            return Some(y);
        }
    }
    None
}

fn tick(w: &World) {
    let dim = w.dimension();
    let mut carves = CARVES.lock().unwrap();
    if carves.is_empty() {
        return;
    }
    carves.retain_mut(|c| {
        // The region whose land holds the middle carves it.
        if c.dim != dim || w.block_loaded(c.at).is_none() {
            return true;
        }
        let mut edits = 0;
        let mut gone = Vec::new();
        while edits < c.budget {
            let Some((dx, dz)) = c.columns.pop() else { break };
            match column(w, c, dx, dz, &mut gone) {
                Some(e) => edits += e,
                None => c.skipped += 1,
            }
        }
        c.edits += edits;
        if !gone.is_empty() {
            crate::leaf_decay::logs_gone(w, gone.into_iter());
        }
        if c.columns.is_empty() {
            weft_sdk::info!(
                "impact at {} {} {}: radius {}, rim {}, {} block edits, {} columns skipped",
                c.at[0], c.at[1], c.at[2], c.radius, c.rim, c.edits, c.skipped
            );
            return false;
        }
        true
    });
}

/// Carves one column; the blocks it changed, or None where it was left (not loaded, water).
fn column(w: &World, c: &Carve, dx: i32, dz: i32, gone: &mut Vec<[i32; 3]>) -> Option<usize> {
    let [cx, cy, cz] = c.at;
    let (x, z) = (cx + dx, cz + dz);
    let g = ground(w, x, z, cy)?;
    let d = ((dx * dx + dz * dz) as f64).sqrt();
    let u = d / c.radius as f64;
    let hash = |y: i32, k: i64| weft_sdk::noise::hash(c.seed, x as i64, y as i64 * 7 + k, z as i64);
    let mut edits = 0;
    let mut set = |p: [i32; 3], s: u32| {
        if w.block(p) != s && w.set_block(p, s) {
            edits += 1;
        }
    };
    if u < 1.0 {
        // The bowl: measured from the middle's ground, so it sits level; everything over its
        // floor goes, a tree in it too.
        let floor = cy + surface(u, c.radius, c.rim);
        for y in (floor + 1..=g.max(floor) + n::IMPACT_CLEAR_UP).rev() {
            let p = [x, y, z];
            let s = w.block(p);
            if s != 0 && state_flags(s) & abi::block_flags::LIQUID == 0 {
                if LOGS.on_block(s) {
                    gone.push(p);
                }
                set(p, 0);
            }
        }
        // Built up where the land round was lower than the bowl's wall.
        for y in g + 1..floor {
            set([x, y, z], rubble(hash(y, 1)));
        }
        // Lined with melt, broken rock under it.
        set([x, floor, z], IMPACT_GLASS.id());
        set([x, floor - 1, z], rubble(hash(floor - 1, 2)));
        // The core, under a block or two of broken rock.
        if c.core_state != u32::MAX {
            for &(_, _, below) in c.core.iter().filter(|m| (m.0, m.1) == (dx, dz)) {
                for y in floor - below + 1..floor - 1 {
                    set([x, y, z], rubble(hash(y, 5)));
                }
                set([x, floor - below, z], c.core_state);
                if let Some(hot) = c.hot {
                    hot(w, [x, floor - below, z]);
                }
            }
        }
    } else if u < n::IMPACT_RIM_OUT {
        // The rim: thrown-out rock over the land, scorched on top.
        let top = g + surface(u, c.radius, c.rim);
        for y in g + 1..top {
            set([x, y, z], rubble(hash(y, 3)));
        }
        if top > g {
            set([x, top, z], COARSE_DIRT.id());
            clear_plant(w, [x, top + 1, z], &mut set);
        } else {
            scorch(w, [x, g, z], &mut set);
        }
        // A meteorite in the rubble, under its top.
        if c.rim_state != u32::MAX && c.rim_spots.contains(&(dx, dz)) {
            set([x, (top - 1).max(g), z], c.rim_state);
        }
    } else {
        // Round it: the grass scorched, thinning outward.
        let v = (n::IMPACT_SCORCH - u) / (n::IMPACT_SCORCH - n::IMPACT_RIM_OUT);
        if (hash(g, 4) % 100) < (v * 100.0) as u64 {
            scorch(w, [x, g, z], &mut set);
        }
    }
    Some(edits)
}

/// Thrown-out rock: suevite, cobblestone or dirt, by `IMPACT_RUBBLE`.
fn rubble(h: u64) -> u32 {
    let (gravel, cobble) = n::IMPACT_RUBBLE;
    match (h % 100) as u32 {
        r if r < gravel => SUEVITE.id(),
        r if r < gravel + cobble => COBBLESTONE.id(),
        _ => DIRT.id(),
    }
}

/// Grass at `p` turns to coarse dirt, and the plant on it goes.
fn scorch(w: &World, [x, y, z]: [i32; 3], set: &mut impl FnMut([i32; 3], u32)) {
    let s = w.block([x, y, z]);
    if GRASS.is(s) || SNOWY_GRASS.is(s) || DIRT.is(s) {
        set([x, y, z], COARSE_DIRT.id());
        clear_plant(w, [x, y + 1, z], set);
    }
}

/// A small plant (grass, flowers, snow: anything replaceable but water) at `p` goes.
fn clear_plant(w: &World, p: [i32; 3], set: &mut impl FnMut([i32; 3], u32)) {
    let s = w.block(p);
    let flags = state_flags(s);
    if s != 0 && flags & abi::block_flags::REPLACEABLE != 0 && flags & abi::block_flags::LIQUID == 0 {
        set(p, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crater_is_a_bowl_under_a_rim_that_fades_into_the_land() {
        let (r, rim) = (12, 3);
        assert_eq!(surface(0.0, r, rim), -(r as f64 * n::IMPACT_DEPTH).round() as i32, "the middle");
        assert_eq!(surface(1.0, r, rim), rim, "the crest");
        assert_eq!(surface(n::IMPACT_RIM_OUT, r, rim), 0, "the rim's foot");
        let mut last = i32::MIN;
        for i in 0..=10 {
            let s = surface(i as f64 / 10.0, r, rim);
            assert!(s >= last, "the bowl rises from the middle to the crest");
            last = s;
        }
    }
}

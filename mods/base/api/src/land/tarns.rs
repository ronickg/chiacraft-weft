//! Cirque tarns (Planner, 11-04: mountain lakes, water above the sea's level): small round lakes
//! in the ranges' high hollows, under the glaciers. At most one per `TARN_CELL` square, kept with
//! `TARN_CHANCE`: up to `TARN_TRIES` points of the cell are tried for a hollow in the mountains'
//! band `TARN_BAND` under the snow line. Its water stands at one block under the lowest ground
//! of a closed ring just outside it, so the rim holds it all round, and inside that its ground is
//! cut to a bowl (the last feature: trees and plants see it); a decoration fills it
//! (`tarn_water`). Worked out once per cell and kept; the site stays far enough inside its cell
//! that a column only asks its own cell.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, div_floor, q4, q12};

use super::{
    Here, MOUNTAIN_BIOME, MOUNTAINS_AREA, STREAM_BEND, STREAM_MAX, STREAM_MIN, STREAM_WANDER, TARN_BAND, TARN_CARVE, TARN_CELL,
    TARN_CHANCE, TARN_DEPTH, TARN_RADIUS, TARN_TRIES, salt, snow_line,
};
use crate::SEA;
use weft_sdk::noise::fixed::Fields;

/// A tarn: its middle and radius (blocks), its water's level (the top water block, y) and its
/// depth in the middle (blocks).
#[derive(Clone, Copy, Debug)]
pub struct Tarn {
    pub x: i32,
    pub z: i32,
    pub radius: i32,
    pub level: i32,
    pub depth: i32,
}

impl Tarn {
    /// The top of the bed at `d` (Q4, from the middle) blocks: `depth` under the water in the
    /// middle, one at the edge; `None` outside it.
    fn bed(&self, d: i32) -> Option<i32> {
        let r = q4(self.radius as f64);
        if d > r {
            return None;
        }
        let u = (d as i64 * 4096 / r as i64) as i32;
        let deep = (((self.depth - 1) as i64 * (4096 - ((u as i64 * u as i64) >> 12))) >> 12) as i32;
        Some(self.level - 1 - deep)
    }
}

thread_local! {
    static SITES: RefCell<HashMap<(u64, i32, i32), Option<Tarn>>> = RefCell::new(HashMap::new());
}

/// How far a tarn reaches past its middle (blocks): its ring.
fn reach() -> i32 {
    TARN_RADIUS.1 + 2
}

/// The columns of the closed ring round a tarn of radius `r` at (`x`, `z`): from `r` out to a
/// block and a half past it, so no gap lets the water out sideways.
fn ring(x: i32, z: i32, r: i32) -> impl Iterator<Item = (i32, i32)> {
    let (r2, out2) = (r * r, (2 * r + 3) * (2 * r + 3) / 4);
    (-r - 2..=r + 2).flat_map(move |dz| {
        (-r - 2..=r + 2).filter_map(move |dx| {
            let d2 = dx * dx + dz * dz;
            (d2 > r2 && d2 <= out2).then_some((x + dx, z + dz))
        })
    })
}

/// Cell (`cx`, `cz`)'s tarn, if it has one. `ground(x, z)`: the ground (blocks) before the tarns.
fn tarn(f: &Fields, cx: i32, cz: i32, ground: &dyn Fn(i32, i32) -> i32) -> Option<Tarn> {
    let key = (f.seed, cx, cz);
    if let Some(hit) = SITES.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let hash = fixed::hash(fixed::seed32(f.seed, salt::TARNS), cx, 0, cz);
    let site = ((fixed::fmix(hash) & 4095) < q12(TARN_CHANCE) as u32)
        .then(|| {
            let span = (TARN_CELL - 2 * reach()) as u32;
            (0..TARN_TRIES).find_map(|k| {
                let h = fixed::fmix(hash ^ k.wrapping_mul(0x9E37_79B9));
                let x = cx * TARN_CELL + reach() + (h % span) as i32;
                let z = cz * TARN_CELL + reach() + ((h >> 11) % span) as i32;
                let here = Here::new(f, x * 16, z * 16);
                if MOUNTAINS_AREA.at(&here) <= q12(MOUNTAIN_BIOME) {
                    return None;
                }
                let line = snow_line(here.temperature());
                let mid = ground(x, z);
                if mid < line - TARN_BAND.0 || mid > line - TARN_BAND.1 {
                    return None;
                }
                let radius = TARN_RADIUS.0 + ((h >> 22) % (TARN_RADIUS.1 - TARN_RADIUS.0 + 1) as u32) as i32;
                // A hollow: the lowest of the ring over the middle's ground or nearly, and the
                // ground inside not much over the water (a little cut, not a notch in a slope).
                let level = ring(x, z, radius).map(|(x, z)| ground(x, z)).min()? - 1;
                if level < mid - 2 {
                    return None;
                }
                let r2 = radius * radius;
                let mut inside = (-radius..=radius)
                    .flat_map(|dz| (-radius..=radius).map(move |dx| (dx, dz)))
                    .filter(|&(dx, dz)| dx * dx + dz * dz <= r2)
                    .map(|(dx, dz)| ground(x + dx, z + dz));
                if inside.any(|g| g > level + TARN_CARVE) {
                    return None;
                }
                let depth = TARN_DEPTH.0 + ((h >> 27) % (TARN_DEPTH.1 - TARN_DEPTH.0 + 1) as u32) as i32;
                Some(Tarn { x, z, radius, level, depth })
            })
        })
        .flatten();
    SITES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4096 {
            c.clear();
        }
        c.insert(key, site);
    });
    site
}

/// The tarn whose bowl column (`x`, `z`) (blocks) is in, if any.
pub(super) fn at(f: &Fields, x: i32, z: i32, ground: &dyn Fn(i32, i32) -> i32) -> Option<Tarn> {
    let (cx, cz) = (div_floor(x, TARN_CELL).0, div_floor(z, TARN_CELL).0);
    let t = tarn(f, cx, cz, ground)?;
    let (dx, dz) = (x - t.x, z - t.z);
    (dx * dx + dz * dz <= t.radius * t.radius).then_some(t)
}

/// What a tarn takes off the ground (Q4, zero or less) at (`x`, `z`) (Q4), the ground there
/// before it being `base` (Q4; asked only inside a tarn's bowl): down to its bowl's bed.
pub(super) fn cut(f: &Fields, x: i32, z: i32, base: &dyn Fn() -> i32, ground: &dyn Fn(i32, i32) -> i32) -> i32 {
    let (bx, bz) = (x >> 4, z >> 4);
    let Some(t) = at(f, bx, bz, ground) else { return 0 };
    let (ex, ez) = ((x - t.x * 16) as i64, (z - t.z * 16) as i64);
    let d = ((ex * ex + ez * ez) as u64).isqrt() as i32;
    let Some(bed) = t.bed(d) else { return 0 };
    // The bed's top block at `bed`: the ground's sum somewhere in that block.
    (q4(bed as f64) + 8 - base()).min(0)
}

/// The tarns (middle x, z, radius, level) within `r` blocks of (`x`, `z`) (blocks).
pub(super) fn near(f: &Fields, x: i32, z: i32, r: i32, ground: &dyn Fn(i32, i32) -> i32) -> Vec<Tarn> {
    let (c0, c1) = (div_floor(x - r, TARN_CELL).0, div_floor(x + r, TARN_CELL).0);
    let (d0, d1) = (div_floor(z - r, TARN_CELL).0, div_floor(z + r, TARN_CELL).0);
    (d0..=d1)
        .flat_map(|cz| (c0..=c1).map(move |cx| (cx, cz)))
        .filter_map(|(cx, cz)| tarn(f, cx, cz, ground))
        .filter(|t| (t.x - x).abs() <= r && (t.z - z).abs() <= r)
        .collect()
}

// ---- the tarns' streams ----

/// A tarn's stream (Planner, 11-04: springs on the range flanks): from the lowest column of its
/// ring (the outlet, cut to the water's level) down the steepest way, a channel whose bed never
/// rises and lies a block under the ground and under the ground on both sides (so the water stays
/// in it on a slope: no sheets over the terraces), still water a block deep in it, falling where
/// the bed drops. Kept only if it ends in a hollow or at the sea's level within `STREAM_MAX`
/// columns: never running out onto open ground.
#[derive(Default)]
pub(super) struct Stream {
    /// Each column of it: its bed's top (blocks).
    beds: HashMap<(i32, i32), i32>,
    /// Its columns in order from the outlet, with their beds.
    pub path: Vec<(i32, i32, i32)>,
}

thread_local! {
    static STREAMS: RefCell<HashMap<(u64, i32, i32), std::rc::Rc<Stream>>> = RefCell::new(HashMap::new());
}

/// Cell (`cx`, `cz`)'s tarn's stream (empty: none). `ground(x, z)`: the ground (blocks) with the
/// tarns, before the streams.
fn stream(f: &Fields, cx: i32, cz: i32, before: &dyn Fn(i32, i32) -> i32, ground: &dyn Fn(i32, i32) -> i32) -> std::rc::Rc<Stream> {
    let key = (f.seed, cx, cz);
    if let Some(hit) = STREAMS.with(|c| c.borrow().get(&key).cloned()) {
        return hit;
    }
    let made = std::rc::Rc::new(
        tarn(f, cx, cz, before)
            .and_then(|t| trace(f, &t, ground))
            .unwrap_or_default(),
    );
    STREAMS.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4096 {
            c.clear();
        }
        c.insert(key, made.clone());
    });
    made
}

/// The way down from tarn `t`'s outlet, if it ends well.
fn trace(f: &Fields, t: &Tarn, ground: &dyn Fn(i32, i32) -> i32) -> Option<Stream> {
    let r2 = t.radius * t.radius;
    let inside = |x: i32, z: i32| (x - t.x) * (x - t.x) + (z - t.z) * (z - t.z) <= r2;
    // The outlet: the ring's lowest column, the nearest to the middle of those.
    let (ox, oz) = ring(t.x, t.z, t.radius).min_by_key(|&(x, z)| (ground(x, z), (x - t.x).pow(2) + (z - t.z).pow(2)))?;
    let mut path = vec![(ox, oz, t.level - 1)];
    let mut seen: std::collections::HashSet<(i32, i32)> = [(ox, oz)].into();
    let (mut x, mut z, mut bed) = (ox, oz, t.level - 1);
    let mut here = ground(x, z);
    let ended = loop {
        if path.len() > STREAM_MAX as usize {
            break false;
        }
        let round: Vec<(i32, i32, i32)> = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .map(|&(dx, dz)| (x + dx, z + dz))
            .filter(|&(nx, nz)| !inside(nx, nz) && !seen.contains(&(nx, nz)))
            .map(|(nx, nz)| (ground(nx, nz), nx, nz))
            .collect();
        let Some(&(lowest, ..)) = round.iter().min() else { break true };
        // It winds (Planner, 11-04: they ran like canals): of the ways nearly as low as the lowest
        // (`STREAM_WANDER`) and not uphill, a noise `STREAM_BEND` blocks across picks.
        let (g, nx, nz) = *round
            .iter()
            .filter(|&&(g, ..)| g <= lowest + STREAM_WANDER && (g <= here || g == lowest))
            .min_by_key(|&&(g, nx, nz)| (g - lowest) * 4096 + f.fbm(salt::STREAMS, nx * 16, nz * 16, q4(STREAM_BEND), 2))
            .unwrap_or(&round.iter().min().copied().unwrap());
        // A hollow: nothing lower round it (the water gathers there), or the sea's level.
        if g > here {
            break true;
        }
        if g <= SEA + 1 {
            path.push((nx, nz, (g - 1).min(bed)));
            break true;
        }
        // Under the ground and under its sides (the columns beside it that aren't the way).
        let sides = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .map(|&(dx, dz)| (nx + dx, nz + dz))
            .filter(|&(sx, sz)| (sx, sz) != (x, z) && !seen.contains(&(sx, sz)) && !inside(sx, sz))
            .map(|(sx, sz)| ground(sx, sz))
            .min()
            .unwrap_or(g);
        bed = bed.min(g - 1).min(sides - 1);
        path.push((nx, nz, bed));
        seen.insert((nx, nz));
        (x, z, here) = (nx, nz, g);
    };
    (ended && path.len() >= STREAM_MIN as usize).then(|| Stream { beds: path.iter().map(|&(x, z, b)| ((x, z), b)).collect(), path })
}

/// The bed (blocks) of a tarn's stream at column (`x`, `z`) (blocks), if one runs there.
pub(super) fn stream_bed(f: &Fields, x: i32, z: i32, before: &dyn Fn(i32, i32) -> i32, ground: &dyn Fn(i32, i32) -> i32) -> Option<i32> {
    let (cx, cz) = (div_floor(x, TARN_CELL).0, div_floor(z, TARN_CELL).0);
    (cz - 1..=cz + 1)
        .flat_map(|cz| (cx - 1..=cx + 1).map(move |cx| (cx, cz)))
        .filter_map(|(cx, cz)| stream(f, cx, cz, before, ground).beds.get(&(x, z)).copied())
        .min()
}

/// Cell (`cx`, `cz`)'s stream's path (x, z, bed; blocks), empty if none.
pub(super) fn stream_path(
    f: &Fields,
    cx: i32,
    cz: i32,
    before: &dyn Fn(i32, i32) -> i32,
    ground: &dyn Fn(i32, i32) -> i32,
) -> Vec<(i32, i32, i32)> {
    stream(f, cx, cz, before, ground).path.clone()
}

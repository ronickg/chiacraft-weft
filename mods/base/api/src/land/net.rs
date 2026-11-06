//! The river network: rain runs downhill over a grid of nodes `RIVER_NODE` blocks apart, and
//! where enough of it gathers the water carves a valley (`super::rivers`).
//!
//! - Each node stands somewhere in its cell (jittered by the seed) at the ground's height there
//!   before the lakes and rivers (`ground_at`).
//! - It drains to the neighbour (of 8) it falls to most steeply. A node lower than all of them (a
//!   pit) spills over the lowest neighbour that doesn't drain back into it, so rivers cut through
//!   hills instead of ending in every hollow. Nodes under the sea drain nowhere.
//! - Every node's rain (`RIVER_RAIN`, less where it's dry) is carried `RIVER_REACH` steps down,
//!   and a node's flow is the rain of every node whose water passes it within that many steps.
//!   That keeps a node's flow a pure function of where it is, so the network is the same however
//!   the world is cut into regions; rivers longer than the reach just stop growing.
//!
//! The network is worked out a region (`RIVER_REGION` nodes square) at a time, with a margin, and
//! kept: the river segments that could reach each cell are listed, so a column only looks at
//! those (none, in most of the world).
//!
//! Where a river ends inland, in a pit nothing spills from, it fills a lake (`RIVER_SINK_FLOW`,
//! `RIVER_SINK_RADIUS`), cut like the basins' lakes (`lake_cut`).

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use weft_sdk::noise::fixed::{self, Fields, ONE, div_floor, q4, q12, ramp, shape};

use super::{
    CONTINENTS, HIGHLANDS, HIGHLANDS_AREA, HILLS, HILLS_AREA, Here, LAKE_DEPTH, LAKE_MAX_RISE, LAKE_STRETCH, Lake, MOUNTAINS_AREA,
    RIVER_BANKS, RIVER_MOUNTAINS, RIVER_NODE, RIVER_RAIN, RIVER_REACH,
    RIVER_REGION, RIVER_ROUTE_HILLS, RIVER_ROUTE_MOUNTAINS, RIVER_SINK_RADIUS, RIVER_TERRACE, RIVER_TERRACE_RISER,
    RIVER_VALLEY_PER_BLOCK, RIVER_VALLEY_RISE, RIVER_WARP_SCALE, Rivers, ground_at, rivers_style, lake_reach_most, masked, mesa_at, salt,
};
use crate::SEA;

/// One stretch of river, from a node to the one it drains to: its ends (Q4), its flow at each
/// (rain, `ONE` a wet node), and the most its valley's sides climb (blocks over the sea): the
/// higher end's big shape plus `RIVER_RISE_SPARE` for the hills, at most `RIVER_VALLEY_RISE`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Seg {
    pub a: [i32; 2],
    pub b: [i32; 2],
    pub flow: [i32; 2],
    pub rise: i32,
}

/// A region's river segments, and for each of its cells the ones whose valley could reach it.
pub(super) struct Region {
    pub segs: Vec<Seg>,
    /// `cells[i]..cells[i + 1]` index `near` for cell `i` (row by row).
    cells: Vec<u32>,
    near: Vec<u32>,
    /// Lakes where a river ends inland (a pit nothing spills from), and per cell the ones that
    /// could reach it, like `cells` and `near`.
    pub lakes: Vec<Lake>,
    lake_cells: Vec<u32>,
    lake_near: Vec<u32>,
    /// Node counts, for the map tools: nodes, pits spilled, segments.
    pub stats: [usize; 3],
}

/// How far a river's valley can reach from its middle line (blocks) with its sides climbing
/// `rise` blocks: the widest channel and floodplain, the sides, and the meander.
pub(super) fn valley_reach(rise: i32) -> i32 {
    let r = rivers_style();
    r.half_width.1 + r.floodplain.1 + RIVER_BANKS + ((RIVER_VALLEY_PER_BLOCK * rise) >> 4) + warp_reach(r)
}

/// A stretch's valley sides climb at most this much more than its ends' big shape (blocks): the
/// hills and dunes the routing leaves out.
const RIVER_RISE_SPARE: i32 = 24;

/// The meander's furthest push (blocks): the warp noise stays well inside ±0.5.
fn warp_reach(r: &Rivers) -> i32 {
    (r.warp * 0.6) as i32
}

/// A node's place (blocks): its cell's corner plus a jitter in the middle half of the cell.
fn node_pos(seed: u64, i: i32, j: i32) -> [i32; 2] {
    let h = fixed::hash(fixed::seed32(seed, salt::RIVER_NODES), i, 0, j);
    let n = RIVER_NODE;
    let span = (n / 2).max(1) as u32;
    [i * n + n / 4 + (h % span) as i32, j * n + n / 4 + ((h >> 16) % span) as i32]
}

/// The height rain runs down (Q4): the land's big shape, without the hills' hollows that would
/// stop it every few nodes. The continents unclamped (so inland still falls toward the sea),
/// the highlands and mesas, the mountains as a dome over their area (`RIVER_ROUTE_MOUNTAINS`
/// blocks at full strength) and a share of the hills (`RIVER_ROUTE_HILLS`), so streams wind and
/// meet.
fn route_height(h: &Here) -> i32 {
    let (f, x, z) = (h.f, h.x, h.z);
    let continents = (h.continent() * 5 / 2 * q4(CONTINENTS)) >> 12;
    let hills = masked(h, &HILLS_AREA, || shape::hills(f, x, z, q4(HILLS.0), q4(HILLS.1)));
    q4(crate::SEA as f64)
        + continents
        + ((hills * q12(RIVER_ROUTE_HILLS)) >> 12)
        + masked(h, &HIGHLANDS_AREA, || q4(HIGHLANDS))
        + ((MOUNTAINS_AREA.at(h) * q4(RIVER_ROUTE_MOUNTAINS)) >> 12)
        + mesa_at(h)
}

const AROUND: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

impl Region {
    fn build(f: &Fields, rx: i32, rz: i32) -> Region {
        let r = RIVER_REGION;
        let border = div_floor(valley_reach(RIVER_VALLEY_RISE) + RIVER_NODE - 1, RIVER_NODE).0 + 1;
        // Sources out to the reach beyond the cells whose flow counts, two more rings for the
        // neighbours' own drains.
        let margin = border + RIVER_REACH + 1;
        let side = r + 2 * margin + 4;
        let (i0, j0) = (rx * r - margin - 2, rz * r - margin - 2);
        let at = |i: i32, j: i32| (j * side + i) as usize;
        let n = (side * side) as usize;
        let mut pos = vec![[0i32; 2]; n];
        let mut height = vec![0i32; n];
        let mut rain = vec![0i32; n];
        let sea = q4(SEA as f64);
        // The nodes from the tiles every region shares (`tile_at`): a region's grid reaches 40
        // nodes past its own 32 each way, so its neighbours sample most of the same ones.
        let (t0, t1) = (div_floor(i0, TILE).0, div_floor(i0 + side - 1, TILE).0);
        let (u0, u1) = (div_floor(j0, TILE).0, div_floor(j0 + side - 1, TILE).0);
        for tj in u0..=u1 {
            for ti in t0..=t1 {
                let tile = tile_at(f, ti, tj);
                for b in 0..TILE {
                    for a in 0..TILE {
                        let (i, j) = (ti * TILE + a - i0, tj * TILE + b - j0);
                        if (0..side).contains(&i) && (0..side).contains(&j) {
                            let (k, t) = (at(i, j), (b * TILE + a) as usize);
                            pos[k] = tile.pos[t];
                            height[k] = tile.height[t];
                            rain[k] = tile.rain[t];
                        }
                    }
                }
            }
        }
        // Steepest descent, then pits spill over their lowest neighbour that doesn't drain back.
        const NONE: u32 = u32::MAX;
        let mut down = vec![NONE; n];
        let dist = |a: [i32; 2], b: [i32; 2]| {
            let (dx, dz) = ((a[0] - b[0]) as i64, (a[1] - b[1]) as i64);
            ((dx * dx + dz * dz) as u64).isqrt().max(1) as i64
        };
        for j in 1..side - 1 {
            for i in 1..side - 1 {
                let k = at(i, j);
                if height[k] < sea {
                    continue;
                }
                let mut best = (0i64, NONE);
                for (di, dj) in AROUND {
                    let o = at(i + di, j + dj);
                    let fall = (height[k] - height[o]) as i64 * 1024 / dist(pos[k], pos[o]);
                    if fall > best.0 {
                        best = (fall, o as u32);
                    }
                }
                down[k] = best.1;
            }
        }
        let mut spilled = 0;
        for j in 2..side - 2 {
            for i in 2..side - 2 {
                let k = at(i, j);
                if height[k] < sea || down[k] != NONE {
                    continue;
                }
                let mut best = (i32::MAX, NONE);
                for (di, dj) in AROUND {
                    let o = at(i + di, j + dj);
                    // Ties go by the node's index, so two flat neighbours can't spill into each other.
                    if down[o] != k as u32 && (height[o], o as u32) > (height[k], k as u32) && height[o] < best.0 {
                        best = (height[o], o as u32);
                    }
                }
                if best.1 != NONE {
                    down[k] = best.1;
                    spilled += 1;
                }
            }
        }
        // Short loops the spills made end where they close: a node whose water comes back to it
        // within a few steps drains nowhere.
        for k in 0..n {
            let mut o = down[k];
            for _ in 0..6 {
                if o == NONE {
                    break;
                }
                if o as usize == k {
                    down[k] = NONE;
                    break;
                }
                o = down[o as usize];
            }
        }
        // Flow: each node's rain carried `RIVER_REACH` steps down (a step back onto a node already
        // passed ends it: spills can make loops).
        let mut flow = vec![0i32; n];
        let mut path = Vec::with_capacity(RIVER_REACH as usize + 1);
        for j in 2..side - 2 {
            for i in 2..side - 2 {
                let mut k = at(i, j);
                path.clear();
                for _ in 0..=RIVER_REACH {
                    if path.contains(&k) {
                        break;
                    }
                    flow[k] += rain[at(i, j)];
                    path.push(k);
                    if down[k] == NONE {
                        break;
                    }
                    k = down[k] as usize;
                }
            }
        }
        // Segments from the nodes whose flow is complete (within `border` cells of the region),
        // listed in every cell their valley could reach.
        let mut segs = Vec::new();
        let mut lists: Vec<Vec<u32>> = vec![Vec::new(); (r * r) as usize];
        let lo = margin + 2 - border;
        for j in lo..side - lo {
            for i in lo..side - lo {
                let k = at(i, j);
                if down[k] == NONE || flow[k] < q12(rivers_style().flow.0) {
                    continue;
                }
                let d = down[k] as usize;
                let id = segs.len() as u32;
                let rise = (((height[k].max(height[d]) - sea) >> 4) + RIVER_RISE_SPARE).clamp(0, RIVER_VALLEY_RISE);
                let reach = valley_reach(rise);
                segs.push(Seg {
                    a: [pos[k][0] * 16, pos[k][1] * 16],
                    b: [pos[d][0] * 16, pos[d][1] * 16],
                    flow: [flow[k], flow[d].max(flow[k])],
                    rise,
                });
                let (x0, x1) = (pos[k][0].min(pos[d][0]) - reach, pos[k][0].max(pos[d][0]) + reach);
                let (z0, z1) = (pos[k][1].min(pos[d][1]) - reach, pos[k][1].max(pos[d][1]) + reach);
                let cell = |b: i32, o: i32| div_floor(b, RIVER_NODE).0 - o;
                let (ci0, ci1) = (cell(x0, rx * r).max(0), cell(x1, rx * r).min(r - 1));
                let (cj0, cj1) = (cell(z0, rz * r).max(0), cell(z1, rz * r).min(r - 1));
                for cj in cj0..=cj1 {
                    for ci in ci0..=ci1 {
                        lists[(cj * r + ci) as usize].push(id);
                    }
                }
            }
        }
        // Lakes at the ends: low ground (as `lake_site` asks), out of the mountains.
        let mut lakes = Vec::new();
        let mut lake_lists: Vec<Vec<u32>> = vec![Vec::new(); (r * r) as usize];
        for j in lo..side - lo {
            for i in lo..side - lo {
                let k = at(i, j);
                if down[k] != NONE || height[k] < sea || flow[k] < q12(rivers_style().sink_flow) {
                    continue;
                }
                let p = pos[k];
                let here = Here::new(f, p[0] * 16, p[1] * 16);
                let rise = (ground_at(&here) >> 4) - SEA;
                if !(1..=LAKE_MAX_RISE).contains(&rise) || MOUNTAINS_AREA.at(&here) > q12(0.3) {
                    continue;
                }
                let h = fixed::hash(fixed::seed32(f.seed, salt::RIVER_NODES), p[0], 1, p[1]);
                let h2 = fixed::fmix(h);
                let size = ramp(flow[k], q12(rivers_style().sink_flow), q12(rivers_style().flow.2));
                let lake = Lake {
                    x: p[0],
                    z: p[1],
                    radius: RIVER_SINK_RADIUS.0 + (((RIVER_SINK_RADIUS.1 - RIVER_SINK_RADIUS.0) * size) >> 12),
                    depth: LAKE_DEPTH.0 + (h % (LAKE_DEPTH.1 - LAKE_DEPTH.0 + 1) as u32) as i32,
                    axis: (h2 % 8) as u8,
                    stretch: LAKE_STRETCH.0 + ((h2 >> 3) % (LAKE_STRETCH.1 - LAKE_STRETCH.0 + 1) as u32) as i32,
                };
                let reach = lake_reach_most(lake.radius);
                let id = lakes.len() as u32;
                lakes.push(lake);
                let cell = |b: i32, o: i32| div_floor(b, RIVER_NODE).0 - o;
                for cj in cell(p[1] - reach, rz * r).max(0)..=cell(p[1] + reach, rz * r).min(r - 1) {
                    for ci in cell(p[0] - reach, rx * r).max(0)..=cell(p[0] + reach, rx * r).min(r - 1) {
                        lake_lists[(cj * r + ci) as usize].push(id);
                    }
                }
            }
        }
        let flatten = |lists: &[Vec<u32>]| {
            let mut cells = Vec::with_capacity(lists.len() + 1);
            let mut near = Vec::new();
            cells.push(0);
            for l in lists {
                near.extend_from_slice(l);
                cells.push(near.len() as u32);
            }
            (cells, near)
        };
        let (cells, near) = flatten(&lists);
        let (lake_cells, lake_near) = flatten(&lake_lists);
        let stats = [n, spilled, segs.len()];
        Region { segs, cells, near, lakes, lake_cells, lake_near, stats }
    }

    fn cell(bx: i32, bz: i32) -> usize {
        let r = RIVER_REGION;
        let ci = div_floor(bx, RIVER_NODE).0;
        let cj = div_floor(bz, RIVER_NODE).0;
        let (li, lj) = (ci - div_floor(ci, r).0 * r, cj - div_floor(cj, r).0 * r);
        (lj * r + li) as usize
    }

    /// The lakes at rivers' ends that could reach block (`bx`, `bz`) (in this region).
    pub fn lakes_near(&self, bx: i32, bz: i32) -> impl Iterator<Item = Lake> + '_ {
        let c = Self::cell(bx, bz);
        self.lake_near[self.lake_cells[c] as usize..self.lake_cells[c + 1] as usize]
            .iter()
            .map(|&l| self.lakes[l as usize])
    }

    /// The segments whose valley could reach block (`bx`, `bz`) (in this region).
    pub fn near(&self, bx: i32, bz: i32) -> impl Iterator<Item = &Seg> {
        let c = Self::cell(bx, bz);
        self.near[self.cells[c] as usize..self.cells[c + 1] as usize]
            .iter()
            .map(|&s| &self.segs[s as usize])
    }
}

type Key = (u64, i32, i32);
/// Something being worked out or done: the first thread to ask builds it, the rest wait for it.
type Slot<V> = Arc<OnceLock<Arc<V>>>;

/// Things every thread shares, each with when it was last asked for. Past `keep` the least
/// recently asked quarter goes, never the one being asked for.
struct Shared<V> {
    map: HashMap<Key, (Slot<V>, u64)>,
    clock: u64,
}

impl<V> Shared<V> {
    fn new() -> Self {
        Shared { map: HashMap::new(), clock: 0 }
    }

    fn slot(&mut self, key: Key, keep: usize) -> Slot<V> {
        self.clock += 1;
        let now = self.clock;
        if self.map.len() > keep {
            let mut ages: Vec<u64> = self.map.values().map(|v| v.1).collect();
            let (_, &mut cut, _) = ages.select_nth_unstable(keep / 4);
            self.map.retain(|k, v| *k == key || v.1 > cut);
        }
        let e = self
            .map
            .entry(key)
            .or_insert_with(|| (Slot::default(), now));
        e.1 = now;
        e.0.clone()
    }
}

/// Nodes sampled for the routing (`route_height` and the rain), `TILE` by `TILE` to a tile.
struct Tile {
    pos: Vec<[i32; 2]>,
    height: Vec<i32>,
    rain: Vec<i32>,
}

/// Nodes to a tile's side: small, so a lone region samples few it doesn't need (a 130-node
/// grid takes 17 by 17 tiles, 136 for 130), large enough that a region asks only some 300.
const TILE: i32 = 8;

impl Tile {
    fn build(f: &Fields, ti: i32, tj: i32) -> Tile {
        let n = (TILE * TILE) as usize;
        let mut t = Tile { pos: Vec::with_capacity(n), height: Vec::with_capacity(n), rain: Vec::with_capacity(n) };
        let dry = q12(RIVER_RAIN.2);
        for b in 0..TILE {
            for a in 0..TILE {
                let p = node_pos(f.seed, ti * TILE + a, tj * TILE + b);
                let here = Here::new(f, p[0] * 16, p[1] * 16);
                t.pos.push(p);
                t.height.push(route_height(&here));
                t.rain
                    .push(dry + (((ONE - dry) * ramp(here.humidity(), q12(RIVER_RAIN.0), q12(RIVER_RAIN.1))) >> 12));
            }
        }
        t
    }
}

/// Regions kept for every thread (`region_at`): some 30 cover distant terrain's reach, a few
/// the cubes. Least recently asked go first (it was every region more than 3 from the last
/// asker).
const KEEP: usize = 160;
/// Tiles kept: a region's grid takes some 300, and a cluster of regions shares most of theirs.
const KEEP_TILES: usize = 6000;
/// Regions each thread keeps at hand, most recently asked first: a column's features ask the
/// same one or its neighbours over and over, without taking the lock.
const AT_HAND: usize = 8;

static REGIONS: LazyLock<Mutex<Shared<Region>>> = LazyLock::new(|| Mutex::new(Shared::new()));
static TILES: LazyLock<Mutex<Shared<Tile>>> = LazyLock::new(|| Mutex::new(Shared::new()));

thread_local! {
    static AT: std::cell::RefCell<Vec<(Key, Arc<Region>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Tile (`ti`, `tj`) of the routing's nodes, sampled once for every region and thread.
fn tile_at(f: &Fields, ti: i32, tj: i32) -> Arc<Tile> {
    let key = (f.seed, ti, tj);
    let slot = TILES
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .slot(key, KEEP_TILES);
    slot.get_or_init(|| Arc::new(Tile::build(f, ti, tj)))
        .clone()
}

/// The region holding block (`bx`, `bz`), worked out once for every thread.
pub(super) fn region_at(f: &Fields, bx: i32, bz: i32) -> Arc<Region> {
    let r = RIVER_REGION * RIVER_NODE;
    let key = (super::cache_seed(f.seed), div_floor(bx, r).0, div_floor(bz, r).0);
    if let Some(hit) = AT.with(|a| {
        let mut a = a.borrow_mut();
        let i = a.iter().position(|(k, _)| *k == key)?;
        let e = a.remove(i);
        let v = e.1.clone();
        a.insert(0, e);
        Some(v)
    }) {
        return hit;
    }
    let slot = REGIONS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .slot(key, KEEP);
    let region = slot
        .get_or_init(|| Arc::new(Region::build(f, key.1, key.2)))
        .clone();
    AT.with(|a| {
        let mut a = a.borrow_mut();
        a.insert(0, (key, region.clone()));
        a.truncate(AT_HAND);
    });
    region
}

/// The meandered point a column measures its distance to the rivers from (Q4).
pub(super) fn meander(f: &Fields, x: i32, z: i32) -> [i64; 2] {
    let (warp, ws) = (q4(rivers_style().warp), q4(RIVER_WARP_SCALE));
    [
        (x + ((f.fbm(salt::RIVER_WARP_X, x, z, ws, 2) * warp) >> 12)) as i64,
        (z + ((f.fbm(salt::RIVER_WARP_Z, x, z, ws, 2) * warp) >> 12)) as i64,
    ]
}

impl Seg {
    /// Point `p`'s (Q4) distance from this stretch's middle line (Q4) and the flow there.
    pub fn place(&self, [px, pz]: [i64; 2]) -> (i32, i32) {
        let (ax, az) = (self.a[0] as i64, self.a[1] as i64);
        let (ex, ez) = (self.b[0] as i64 - ax, self.b[1] as i64 - az);
        let len2 = (ex * ex + ez * ez).max(1);
        let t = (((px - ax) * ex + (pz - az) * ez) * 4096 / len2).clamp(0, 4096);
        let (cx, cz) = (ax + ex * t / 4096 - px, az + ez * t / 4096 - pz);
        let d = ((cx * cx + cz * cz) as u64).isqrt() as i32;
        let flow = self.flow[0] + (((self.flow[1] - self.flow[0]) as i64 * t) >> 12) as i32;
        (d, flow)
    }
}

/// A river's shape at a flow: its channel's half width and depth, and its floodplain (all Q4),
/// and how strongly it runs (0 to `ONE`: under `ONE` it's a dry gully, rising out of the water).
pub(super) struct Shape {
    pub half: i32,
    pub depth: i32,
    pub floodplain: i32,
    pub strength: i32,
}

/// How much the mountains leave of a river at a column (`RIVER_MOUNTAINS`): `ONE` clear of them.
pub(super) fn mountains(h: &Here) -> i32 {
    ONE - ramp(MOUNTAINS_AREA.at(h), q12(RIVER_MOUNTAINS.0), q12(RIVER_MOUNTAINS.1))
}

pub(super) fn shape(flow: i32, mountains: i32) -> Shape {
    let r = rivers_style();
    let size = ramp(flow, q12(r.flow.1), q12(r.flow.2));
    let lerp = |(a, b): (i32, i32)| q4(a as f64) + (((q4(b as f64) - q4(a as f64)) * size) >> 12);
    let strength = ramp(flow, q12(r.flow.0), q12(r.flow.1));
    Shape {
        half: lerp(r.half_width).max(16),
        depth: lerp(r.depth),
        floodplain: lerp(r.floodplain),
        strength: (strength * mountains) >> 12,
    }
}

/// A valley side's rise (Q4) in benches: flat for all but the last `RIVER_TERRACE_RISER` of each
/// `RIVER_TERRACE` blocks, which climbs to the next (`super::rivers`).
pub(super) fn terraced(raise: i32) -> i32 {
    let step = q4(RIVER_TERRACE as f64);
    let (k, frac) = div_floor(raise.max(0), step);
    let bench = step - ((step * q12(RIVER_TERRACE_RISER)) >> 12);
    let u = ramp(frac, bench, step);
    let s = (((u * u) >> 12) * (3 * ONE - 2 * u)) >> 12;
    k * step + ((step * s) >> 12)
}

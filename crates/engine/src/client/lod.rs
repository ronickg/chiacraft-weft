//! Distant terrain beyond the view: made on the GPU straight from the dimension's terrain at a
//! coarser step (`shaders/terrain.wgsl`'s `lod`), never from cubes, so it reaches as far as wanted
//! for a few kilobytes a tile. A tile is 32x32 cells of 2^level blocks; each cell is the column
//! at its middle: its top block, the one under it, its biome, and tree canopies over it (no
//! caves or edits).
//! Tiles are picked by distance, nearer ones split into the four of the level below; one is
//! shown once made, its parent or children standing in until then. Where a column of real cubes
//! shows its ground the tiles' pixels are dropped (`fs_lod`, `Lod::mask`), so they fill in for
//! cubes not made or meshed yet (flying fast), and nowhere stand through them.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_channel::{Receiver, Sender, unbounded};
use glam::{DVec3, IVec2};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::content::{Content, content};
use crate::gpu_layout::face;

/// Cells a tile side.
pub const CELLS: i32 = 32;
/// Columns a tile is made from: its cells and one around them (for the walls at its edges).
pub const SAMPLES: usize = crate::gpu_layout::terrain::lod::SAMPLES as usize;
const SIDE: i32 = crate::gpu_layout::terrain::lod::SIDE as i32;
/// Words per tile coming back from the GPU.
pub const LOD_WORDS_PER_TILE: usize = SAMPLES * 2;
pub const MAX_LEVEL: u32 = 5;
/// From this level on tiles are smooth ground, morphing into the next level with distance: at
/// 8-block cells and more (from about 1000 blocks away), boxes only drew stairs.
pub const SMOOTH_LEVEL: u32 = 3;
/// A tile splits into the level below while nearer than this many of its widths: a cell stays
/// about the same size on screen (a 32-block cell at 4096 blocks).
const SPLIT: f64 = 4.0;
/// A split tile joins again only once farther than this many of its widths, so walking to and
/// fro over the line doesn't swap levels each time.
const MERGE: f64 = 4.5;
/// Made tiles no longer wanted kept a while (not drawn), for coming back.
const CACHE: usize = 192;
/// Blocks under the land from where distant terrain is left out.
const UNDERGROUND: i32 = 32;
/// Tiles asked for and not back yet, at most.
const IN_FLIGHT: usize = 256;

/// A tile: its level and place in the grid of its level's tiles.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TileKey {
    pub level: u32,
    pub x: i32,
    pub z: i32,
}

impl TileKey {
    /// Width in blocks.
    pub fn size(self) -> i32 {
        CELLS << self.level
    }

    /// Whether it is drawn as smooth ground (`mesh_smooth`) rather than boxes.
    pub fn smooth(self) -> bool {
        self.level >= SMOOTH_LEVEL
    }

    /// Lowest block corner (x, z).
    pub fn origin(self) -> IVec2 {
        IVec2::new(self.x, self.z) * self.size()
    }

    fn parent(self) -> TileKey {
        TileKey { level: self.level + 1, x: self.x.div_euclid(2), z: self.z.div_euclid(2) }
    }

    fn children(self) -> [TileKey; 4] {
        std::array::from_fn(|i| TileKey { level: self.level - 1, x: self.x * 2 + (i as i32 & 1), z: self.z * 2 + (i as i32 >> 1) })
    }

    /// Nearest and farthest horizontal distance from `p` (x, z) to it.
    fn distance(self, p: glam::DVec2) -> (f64, f64) {
        let lo = self.origin().as_dvec2();
        let hi = lo + glam::DVec2::splat(self.size() as f64);
        let near = (lo - p).max(p - hi).max(glam::DVec2::ZERO).length();
        let far = (p - lo).abs().max((p - hi).abs()).length();
        (near, far)
    }
}

/// A tile for the GPU to make, with the heights of mods' own features at its columns.
pub struct LodJob {
    pub dim: u32,
    pub seed: u64,
    pub key: TileKey,
    pub custom: Vec<i32>,
    pub done: Sender<LodMesh>,
    /// Cleared when the tile is no longer wanted: the work left is skipped.
    pub live: Arc<AtomicBool>,
}

impl LodJob {
    pub fn live(&self) -> bool {
        self.live.load(Ordering::Relaxed)
    }
}

/// LOD's own threads, at low priority: its mods' heights and meshing never hold up the pool that
/// finishes the near cubes (joining and flying wait on those).
pub fn pool() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(3)
            .thread_name(|i| format!("distant terrain {i}"))
            .start_handler(|_| crate::lower_priority())
            .build()
            .expect("distant terrain pool")
    })
}

/// A made tile's quads (`mesh`), how many records come before them (`mesh_smooth`'s
/// corners) and the heights they span.
pub struct LodMesh {
    pub dim: u32,
    pub seed: u64,
    pub key: TileKey,
    pub quads: Vec<[u32; 4]>,
    pub skip: u32,
    pub y: (i32, i32),
}

/// What came back for `job`, meshed on a worker.
pub fn made(job: LodJob, words: Vec<u32>) {
    if !job.live() {
        return;
    }
    pool().spawn(move || {
        if !job.live() {
            return;
        }
        let (quads, skip, y) = if job.key.smooth() {
            mesh_smooth(content(), job.dim, job.seed, job.key, &words)
        } else {
            let (quads, y) = mesh(content(), job.dim, job.seed, job.key, &words);
            (quads, 0, y)
        };
        let _ = job
            .done
            .send(LodMesh { dim: job.dim, seed: job.seed, key: job.key, quads, skip, y });
    });
}

/// A record of `world.wgsl`'s `vs_lod`: the `dir` face of a box at (x, y, z) in the tile, size
/// (sx, sy, sz) blocks, drawn in the average color of texture `tex` (its smallest mip), tinted
/// for `biome` when `tint`.
fn quad(dir: u32, x: i32, y: i32, z: i32, size: (i32, i32, i32), tex: u32, tint: u32, biome: u32) -> [u32; 4] {
    crate::gpu_layout::lod_record(dir, [x, y, z], [size.0, size.1, size.2], tex, tint, biome)
}

/// The texture (its average color) and tint kind a distant face of `state` in direction `dir`
/// is drawn in. A box model's faces are its boxes' own (its block's textures may be unset, and
/// came out as the missing texture's magenta): its largest box's. A plant's is its picture.
fn face_of(c: &Content, looks: &super::looks::Looks, state: u32, dir: usize) -> (u32, u32) {
    let info = &c.states[state as usize];
    let tint = if info.tint_faces & (1 << dir) != 0 { info.tint as u32 } else { 0 };
    let drawn = looks.drawn(state, info);
    let volume = |b: &crate::content::ModelBox| (0..3).map(|a| (b.max[a] - b.min[a]).abs()).product::<f32>();
    let tex = match drawn.model {
        weft_abi::Model::Boxes => drawn
            .boxes
            .iter()
            .max_by(|a, b| volume(a).total_cmp(&volume(b)))
            .map_or(drawn.textures[dir], |b| b.textures[dir]),
        weft_abi::Model::Cross | weft_abi::Model::Wire => drawn.textures[0],
        _ => drawn.textures[dir],
    };
    (tex & face::TEX_MASK, tint)
}

/// How far the ground falls per block to its lowest neighbour, from a `drop` between samples `s`
/// blocks apart: a little more than the mean, as the fall between blocks gathers in steps (the
/// same as `terrain.wgsl`'s `lod_fall`).
pub fn fall(drop: i32, s: i32) -> i32 {
    (drop.max(0) * 3 + 2 * s - 1) / (2 * s)
}

/// Blocks of soil under a column's top before the stone, as most surface rules have it.
const FILLER: i32 = 3;
/// Bands a steep wall is drawn in at most, each asking the face what it shows there.
const BANDS: i32 = 8;

/// The tile's top faces (runs along x merged) and the walls down to lower neighbors: the top
/// block's top color for its own height (a grass side averages to the dirt under its rim, and
/// every slope came out striped brown), then the block under it for the rest. Where the ground
/// falls steeply the dimension's faces (`Terrain::faces`) say what shows instead (bare rock), in
/// bands down the wall.
pub fn mesh(c: &Content, dim: u32, seed: u64, key: TileKey, words: &[u32]) -> (Vec<[u32; 4]>, (i32, i32)) {
    let s = 1 << key.level;
    let at = |i: i32, j: i32| {
        let k = ((j + 1) * SIDE + i + 1) as usize * 2;
        let (a, b) = (words[k], words[k + 1]);
        ((a & 0xFFFF) as u16 as i16 as i32, a >> 16, b & 0xFFFF, b >> 16)
    };
    let looks = super::looks::current();
    let face_of = |state: u32, dir: usize| face_of(c, &looks, state, dir);
    let terrain = c.dims.get(dim as usize).and_then(|d| d.terrain.as_ref());
    let faces = terrain.and_then(|t| t.faces);
    let stone = terrain.map_or(0, |t| t.stone);
    let o = key.origin();
    // Per cell: its top as shown, and for a steep one its fall and wall bands from the top down
    // (bottom, block), down to its lowest neighbour.
    let n = CELLS as usize;
    let mut tops = vec![0u32; n * n];
    let mut walls: Vec<Option<Vec<(i32, u32)>>> = vec![None; n * n];
    for j in 0..CELLS {
        for i in 0..CELLS {
            let (y, top, biome, under) = at(i, j);
            let k = (j * CELLS + i) as usize;
            tops[k] = top;
            // Water, ice and canopies have the same block under: no ground face there.
            let Some(f) = faces.filter(|_| top != under) else { continue };
            let low = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .map(|&(di, dj)| at(i + di, j + dj).0)
                .min()
                .unwrap_or(y);
            let fall = fall(y - low, s);
            if fall < f.fall {
                continue;
            }
            let (x, z) = (o.x + i * s + s / 2, o.y + j * s + s / 2);
            let shown = |wy: i32, before: u32| f.block(seed, dim, [x, wy, z], y, biome, fall, before);
            tops[k] = shown(y, top);
            // The wall below the top block, to the skirt's depth at most.
            let bottom = low.min(y - 1) - 2 * s;
            let step = ((y - 1 - bottom) + BANDS - 1) / BANDS;
            let mut bands: Vec<(i32, u32)> = Vec::new();
            let mut wy = y - 1;
            while wy > bottom && step > 0 {
                let before = if y - wy <= FILLER { under } else { stone };
                let block = shown(wy, before);
                let lo = (wy - step + 1).max(bottom + 1);
                match bands.last_mut() {
                    Some(last) if last.1 == block => last.0 = lo,
                    _ => bands.push((lo, block)),
                }
                wy = lo - 1;
            }
            walls[k] = Some(bands);
        }
    }
    let mut out = Vec::new();
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    let top_at = |i: i32, j: i32| tops[(j * CELLS + i) as usize];
    for j in 0..CELLS {
        let mut i = 0;
        while i < CELLS {
            let (y, _, biome, _) = at(i, j);
            let top = top_at(i, j);
            let mut end = i + 1;
            while end < CELLS && at(end, j).0 == y && top_at(end, j) == top && at(end, j).2 == biome {
                end += 1;
            }
            let (tex, tint) = face_of(top, 3);
            out.push(quad(3, i * s, y, j * s, ((end - i) * s, 1, s), tex, tint, biome));
            (lo, hi) = (lo.min(y), hi.max(y + 1));
            i = end;
        }
    }
    for j in 0..CELLS {
        for i in 0..CELLS {
            let (y, _, biome, under) = at(i, j);
            let k = (j * CELLS + i) as usize;
            let top = tops[k];
            for (dir, (di, dj)) in [(0usize, (-1, 0)), (1, (1, 0)), (4, (0, -1)), (5, (0, 1))] {
                let mut ny = at(i + di, j + dj).0;
                // At the tile's edge the tile beside may be of another level, its ground not
                // where this one's sample says: the wall goes on down below it (a skirt), or
                // the sky shows through the crack.
                if !(0..CELLS).contains(&(i + di)) || !(0..CELLS).contains(&(j + dj)) {
                    ny = ny.min(y - 1) - 2 * s;
                }
                if ny >= y {
                    continue;
                }
                let (tex, tint) = face_of(top, 3);
                out.push(quad(dir as u32, i * s, y, j * s, (s, 1, s), tex, tint, biome));
                lo = lo.min(ny + 1);
                if y - 1 <= ny {
                    continue;
                }
                match &walls[k] {
                    Some(bands) => {
                        let mut from = y - 1;
                        for &(band_lo, block) in bands {
                            let b = band_lo.max(ny + 1);
                            if b > from {
                                break;
                            }
                            let (tex, tint) = face_of(block, dir);
                            out.push(quad(dir as u32, i * s, b, j * s, (s, from + 1 - b, s), tex, tint, biome));
                            from = b - 1;
                        }
                    }
                    None => {
                        let (tex, tint) = face_of(under, dir);
                        out.push(quad(dir as u32, i * s, ny + 1, j * s, (s, y - 1 - ny, s), tex, tint, biome));
                    }
                }
            }
        }
    }
    (out, (lo, hi))
}

/// A smooth tile (`TileKey::smooth`): its corners' heights and biomes first (`gpu_layout::lod_grid`,
/// `SIDE` x `SIDE` from one before its first corner, not drawn), then a cell per 32 x 32
/// (`lod_grid_cell`) in the block most of its corners have on top (bare rock where the ground
/// falls steeply, `Terrain::faces`), and short skirts under its edges against cracks where a tile
/// of another level is beside it. The shader morphs its corners into the next level's.
pub fn mesh_smooth(c: &Content, dim: u32, seed: u64, key: TileKey, words: &[u32]) -> (Vec<[u32; 4]>, u32, (i32, i32)) {
    use crate::gpu_layout::{lod_face, lod_grid, lod_grid_cell};
    let s = 1 << key.level;
    // Corner (i, j), from -1 to 32.
    let at = |i: i32, j: i32| {
        let k = ((j + 1) * SIDE + i + 1) as usize * 2;
        let (a, b) = (words[k], words[k + 1]);
        ((a & 0xFFFF) as u16 as i16 as i32, a >> 16, b & 0xFFFF, b >> 16)
    };
    let looks = super::looks::current();
    let face_of = |state: u32, dir: usize| face_of(c, &looks, state, dir);
    let faces = c
        .dims
        .get(dim as usize)
        .and_then(|d| d.terrain.as_ref())
        .and_then(|t| t.faces);
    let grid: Vec<u32> = (0..SAMPLES as i32)
        .map(|k| {
            let (y, _, biome, _) = at(k % SIDE - 1, k / SIDE - 1);
            lod_grid(y, biome)
        })
        .collect();
    debug_assert_eq!(grid.len(), lod_face::GRID_WORDS as usize);
    let mut out: Vec<[u32; 4]> = grid.chunks(4).map(|w| [w[0], w[1], w[2], w[3]]).collect();
    let skip = out.len() as u32;
    let o = key.origin();
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    for j in 0..CELLS {
        for i in 0..CELLS {
            let corners = [at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1)];
            // The top most of its corners have (the first of them on a tie).
            let pick = (0..4)
                .max_by_key(|&a| (corners.iter().filter(|c| c.1 == corners[a].1).count(), std::cmp::Reverse(a)))
                .unwrap_or(0);
            let (_, mut top, biome, under) = corners[pick];
            let low = corners.iter().map(|c| c.0).min().unwrap_or(0);
            let high = corners.iter().map(|c| c.0).max().unwrap_or(0);
            (lo, hi) = (lo.min(low + 1 - 2 * s), hi.max(high + 1));
            if let Some(f) = faces.filter(|_| top != under) {
                let fall = fall(high - low, s);
                if fall >= f.fall {
                    let (x, z) = (o.x + i * s + s / 2, o.y + j * s + s / 2);
                    top = f.block(seed, dim, [x, high, z], high, biome, fall, top);
                }
            }
            let (tex, tint) = face_of(top, 3);
            let k = out.len() as u32;
            out.push(lod_grid_cell(i as u32, j as u32, key.level, k, tex, tint, biome));
            // Skirts: as deep as two cells, from under the edge however it morphs (an odd corner
            // goes to the middle of its neighbours, so from the lowest of the four along it).
            for (dir, edge) in [(0u32, i == 0), (1, i == CELLS - 1), (4, j == 0), (5, j == CELLS - 1)] {
                if !edge {
                    continue;
                }
                let along = |d: i32| match dir {
                    0 => at(i, (j + d).clamp(-1, CELLS)),
                    1 => at(i + 1, (j + d).clamp(-1, CELLS)),
                    4 => at((i + d).clamp(-1, CELLS), j),
                    _ => at((i + d).clamp(-1, CELLS), j + 1),
                };
                let bottom = (-1..=2).map(|d| along(d).0).min().unwrap_or(0);
                let y = bottom + 1 - 2 * s;
                lo = lo.min(y);
                let (tex, tint) = face_of(top, dir as usize);
                out.push(quad(dir, i * s, y, j * s, (s, 2 * s, s), tex, tint, biome));
            }
        }
    }
    (out, skip, (lo, hi))
}

/// The client's distant terrain: which tiles to show, asking for missing ones, taking made ones.
pub struct Lod {
    /// The land's height at the camera's column, kept while it stays in it.
    ground: Option<((u32, IVec2), i32)>,
    jobs: Sender<LodJob>,
    done: (Sender<LodMesh>, Receiver<LodMesh>),
    dim: u32,
    seed: u64,
    /// Indices of the dimension's features whose heights run mods' code (`terrain_gpu`);
    /// `None`: no distant terrain here.
    custom: Option<Arc<Vec<usize>>>,
    ready: FxHashSet<TileKey>,
    /// Asked for, and their jobs' flags (cleared to call them off).
    pending: FxHashMap<TileKey, Arc<AtomicBool>>,
    /// Made tiles not wanted now, oldest first, kept until `CACHE` of them.
    cached: FxHashSet<TileKey>,
    dropped: VecDeque<TileKey>,
    /// Tiles split in the last pick (`MERGE`).
    split: FxHashSet<TileKey>,
    /// Tiles wanted, nearest first, and where the camera was then.
    wanted: Vec<TileKey>,
    wanted_at: Option<(glam::DVec2, i32, i32)>,
    /// A bit per cube column around the camera's (`MASK_MARGIN` past the view, addressed
    /// modulo the window's width so it does not move with the camera): its cubes show its
    /// ground. Its half width and center, and whether it changed since uploaded.
    mask: Vec<u32>,
    half: i32,
    center: IVec2,
    mask_changed: bool,
}

/// Columns the mask reaches past the view radius (cubes are kept that far).
const MASK_MARGIN: i32 = 3;

impl Lod {
    pub fn new(jobs: Sender<LodJob>) -> Lod {
        Lod {
            ground: None,
            jobs,
            done: unbounded(),
            dim: u32::MAX,
            seed: 0,
            custom: None,
            ready: FxHashSet::default(),
            pending: FxHashMap::default(),
            cached: FxHashSet::default(),
            dropped: VecDeque::new(),
            split: FxHashSet::default(),
            wanted: Vec::new(),
            wanted_at: None,
            mask: Vec::new(),
            half: 0,
            center: IVec2::ZERO,
            mask_changed: false,
        }
    }

    /// Tiles asked for and not back yet.
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// Distance in blocks it reaches (`core:lod_distance` cubes), 0 when off.
    pub fn distance() -> i32 {
        let c = content();
        c.game
            .config_names
            .get("core:lod_distance")
            .map_or(0, |&id| crate::config::get(id) as i32)
            * 16
    }

    /// Picks, asks for and hands the renderer this frame's tiles. `view`: the view radius in
    /// cubes (inside it the real cubes are drawn).
    pub fn update(
        &mut self,
        gpu: &mut dyn weft_render::Backend,
        render: &mut super::world_render::WorldRenderer,
        world: &mut super::world::ClientWorld,
        textures: &super::textures::GameTextures,
        seed: u64,
        cam: DVec3,
        view: i32,
    ) {
        let c = content();
        let dim = world.dim;
        self.update_mask(gpu, render, world, seed, cam, view);
        let reach = Self::distance();
        if (dim, seed) != (self.dim, self.seed) || reach == 0 {
            for key in self.ready.drain() {
                render.remove_lod(key);
            }
            for (_, live) in self.pending.drain() {
                live.store(false, Ordering::Relaxed);
            }
            self.cached.clear();
            self.dropped.clear();
            self.split.clear();
            self.wanted.clear();
            self.wanted_at = None;
        }
        if (dim, seed) != (self.dim, self.seed) {
            (self.dim, self.seed) = (dim, seed);
            self.custom = super::terrain_gpu::describe(c, dim, seed).map(|(_, custom)| Arc::new(custom));
        }
        // Tiles stand in for cubes not shown yet; the inner half of the view is shown long
        // before the edge, where flying outruns the cubes.
        let mut hide = (view / 2 * 16).max(0);
        // High up, the view's cubes don't reach the ground: tiles have to fill that in too (where
        // cubes do show, the mask leaves them out).
        if self
            .land_height(c, cam)
            .is_some_and(|land| cam.y > (land + view * 8) as f64)
        {
            hide = 0;
        }
        while let Ok(m) = self.done.1.try_recv() {
            if (m.dim, m.seed) != (self.dim, self.seed) || self.pending.remove(&m.key).is_none() {
                continue;
            }
            render.upload_lod(gpu, textures, m.key, &m.quads, m.skip, m.y);
            self.ready.insert(m.key);
            crate::diag::LOD_MADE.add(1);
        }
        if reach == 0 || self.custom.is_none() {
            render.lod_draw.clear();
            return;
        }
        let p = glam::DVec2::new(cam.x, cam.z);
        if self
            .wanted_at
            .is_none_or(|(at, h, r)| (at - p).length() > 16.0 || h != hide || r != reach)
        {
            self.wanted_at = Some((p, hide, reach));
            self.pick(p, hide as f64, reach as f64);
        }
        // Shown: each wanted tile when made, else its children when all four are, else its
        // nearest made ancestor, which then stands in for every wanted tile under it: a child
        // shows only with its three siblings, never over its parent.
        let mut keep = FxHashSet::default();
        let mut made = Vec::new();
        let mut standing = FxHashSet::default();
        for &key in &self.wanted {
            keep.insert(key);
            if self.ready.contains(&key) {
                made.push(key);
                continue;
            }
            if key.level > 1 && key.children().iter().all(|k| self.ready.contains(k)) {
                made.extend(key.children());
                keep.extend(key.children());
                continue;
            }
            let mut a = key;
            while a.level < MAX_LEVEL {
                a = a.parent();
                if self.ready.contains(&a) {
                    standing.insert(a);
                    keep.insert(a);
                    break;
                }
            }
        }
        let under_standing = |mut k: TileKey| {
            while k.level < MAX_LEVEL {
                k = k.parent();
                if standing.contains(&k) {
                    return true;
                }
            }
            false
        };
        let mut shown: FxHashSet<TileKey> = made.into_iter().filter(|&k| !under_standing(k)).collect();
        shown.extend(standing.iter().copied());
        // It is a shell over the land with nothing under it: from deep below, its walls would hang
        // in the sky. Only rock is that way anyway.
        if self.deep_underground(c, cam) {
            shown.clear();
        }
        // Other tiles shown (a cached one back, a level swapped) change what casts into the far
        // shadow map, which is kept while the terrain stays (`terrain_epoch`).
        if shown.len() != render.lod_draw.len() || render.lod_draw.iter().any(|k| !shown.contains(k)) {
            render.terrain_epoch += 1;
        }
        render.lod_draw.clear();
        render.lod_draw.extend(shown);
        // Tiles no longer wanted wait in the cache, the oldest freed past `CACHE`.
        for &key in &self.ready {
            if !keep.contains(&key) && self.cached.insert(key) {
                self.dropped.push_back(key);
            }
        }
        self.cached.retain(|k| !keep.contains(k));
        while self.cached.len() > CACHE {
            let Some(key) = self.dropped.pop_front() else { break };
            if self.cached.remove(&key) {
                self.ready.remove(&key);
                render.remove_lod(key);
            }
        }
        if self.dropped.len() > 4 * CACHE {
            let cached = &self.cached;
            self.dropped.retain(|k| cached.contains(k));
        }
        // Work for tiles no longer wanted is called off.
        self.pending.retain(|k, live| {
            let on = keep.contains(k);
            if !on {
                live.store(false, Ordering::Relaxed);
                crate::diag::LOD_CALLED_OFF.add(1);
            }
            on
        });
        self.request();
    }

    /// Well under the land at the camera's column, so its sky can't be seen. Near the surface a cave
    /// mouth or a ravine still looks out over the distance.
    fn deep_underground(&mut self, c: &Content, cam: DVec3) -> bool {
        self.land_height(c, cam)
            .is_some_and(|land| cam.y < (land - UNDERGROUND) as f64)
    }

    /// How high the land is at the camera's column.
    fn land_height(&mut self, c: &Content, cam: DVec3) -> Option<i32> {
        let at = IVec2::new(cam.x.floor() as i32, cam.z.floor() as i32);
        if self.ground.is_none_or(|g| g.0 != (self.dim, at)) {
            let height = c.dims[self.dim as usize]
                .column(self.seed, at.x, at.y)
                .map_or(i32::MIN, |col| col.height);
            self.ground = Some(((self.dim, at), height));
        }
        self.ground.map(|g| g.1).filter(|&h| h != i32::MIN)
    }

    /// Keeps the mask of columns shown by cubes current: columns that changed, and the ones
    /// coming into the window when the camera moved to another column.
    fn update_mask(
        &mut self,
        gpu: &mut dyn weft_render::Backend,
        render: &mut super::world_render::WorldRenderer,
        world: &mut super::world::ClientWorld,
        seed: u64,
        cam: DVec3,
        view: i32,
    ) {
        let half = view + MASK_MARGIN;
        let w = 2 * half + 1;
        let center = IVec2::new((cam.x / 16.0).floor() as i32, (cam.z / 16.0).floor() as i32);
        let slot = |col: IVec2| (col.y.rem_euclid(w) * w + col.x.rem_euclid(w)) as usize;
        let set = |mask: &mut Vec<u32>, col: IVec2, on: bool| {
            let i = slot(col);
            let (word, bit) = (i / 32, 1u32 << (i % 32));
            let old = mask[word];
            mask[word] = if on { old | bit } else { old & !bit };
            old != mask[word]
        };
        let changed = world.take_changed_columns();
        let inside = |col: IVec2, at: IVec2| (col - at).abs().max_element() <= half;
        if half != self.half || self.mask.is_empty() {
            self.half = half;
            self.mask = vec![0; ((w * w) as usize).div_ceil(32)];
            self.center = center;
            for z in -half..=half {
                for x in -half..=half {
                    let col = center + IVec2::new(x, z);
                    set(&mut self.mask, col, world.column_shown(col, seed));
                }
            }
            self.mask_changed = true;
        } else if center != self.center {
            let old = self.center;
            self.center = center;
            for z in -half..=half {
                for x in -half..=half {
                    let col = center + IVec2::new(x, z);
                    if !inside(col, old) {
                        self.mask_changed |= set(&mut self.mask, col, world.column_shown(col, seed));
                    }
                }
            }
        }
        for col in changed {
            if inside(col, center) {
                self.mask_changed |= set(&mut self.mask, col, world.column_shown(col, seed));
            }
        }
        render.lod_half = half as u32;
        if self.mask_changed {
            self.mask_changed = false;
            render.set_lod_mask(gpu, &self.mask);
        }
    }

    /// The tiles to show: level 5 tiles within `reach`, split while near, leaving out what
    /// lies inside `hide` (where the view's cubes are always shown).
    fn pick(&mut self, p: glam::DVec2, hide: f64, reach: f64) {
        let top = CELLS << MAX_LEVEL;
        let (lo, hi) = (((p - reach) / top as f64).floor().as_ivec2(), ((p + reach) / top as f64).floor().as_ivec2());
        let mut out = Vec::new();
        let mut stack: Vec<TileKey> = Vec::new();
        for z in lo.y..=hi.y {
            for x in lo.x..=hi.x {
                stack.push(TileKey { level: MAX_LEVEL, x, z });
            }
        }
        let mut split = FxHashSet::default();
        while let Some(key) = stack.pop() {
            let (near, far) = key.distance(p);
            if near > reach || far < hide {
                continue;
            }
            let edge = if self.split.contains(&key) { MERGE } else { SPLIT };
            if key.level > 1 && near < edge * key.size() as f64 {
                split.insert(key);
                stack.extend(key.children());
            } else {
                out.push((near, key));
            }
        }
        out.sort_by(|a, b| a.0.total_cmp(&b.0));
        self.wanted = out.into_iter().map(|t| t.1).collect();
        self.split = split;
    }

    /// Asks for the nearest wanted tiles not made yet.
    fn request(&mut self) {
        let c = content();
        let custom = self.custom.clone().unwrap();
        for &key in &self.wanted {
            if self.pending.len() >= IN_FLIGHT {
                break;
            }
            if self.ready.contains(&key) || self.pending.contains_key(&key) {
                continue;
            }
            let live = Arc::new(AtomicBool::new(true));
            self.pending.insert(key, live.clone());
            let (dim, seed, done, jobs) = (self.dim, self.seed, self.done.0.clone(), self.jobs.clone());
            if custom.is_empty() {
                let _ = jobs.send(LodJob { dim, seed, key, custom: Vec::new(), done, live });
                continue;
            }
            let custom = custom.clone();
            // Mods' features run their code: on LOD's own threads.
            pool().spawn(move || {
                if !live.load(Ordering::Relaxed) {
                    return;
                }
                let heights = custom_heights(c, dim, seed, key, &custom);
                let _ = jobs.send(LodJob { dim, seed, key, custom: heights, done, live });
            });
        }
    }
}

/// Heights of the mod features `custom` at the tile's columns, for its job.
fn custom_heights(c: &Content, dim: u32, seed: u64, key: TileKey, custom: &[usize]) -> Vec<i32> {
    let Some(t) = c.dims[dim as usize].terrain.as_ref() else { return Vec::new() };
    let f = t.fields(seed);
    let (o, s) = (key.origin(), 1 << key.level);
    // At the cells' middles, or a smooth tile's corners (as `terrain.wgsl` samples them).
    let mid = if key.smooth() { 0 } else { s / 2 };
    let mut out = Vec::with_capacity(custom.len() * SAMPLES);
    for &i in custom {
        for k in 0..SAMPLES as i32 {
            let (x, z) = (o.x + (k % SIDE - 1) * s + mid, o.y + (k / SIDE - 1) * s + mid);
            out.push(t.features[i].height_at(&f, x * 16, z * 16));
        }
    }
    out
}

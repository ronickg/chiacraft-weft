//! Distant terrain beyond the view: made on the GPU straight from the dimension's terrain at a
//! coarser step (`shaders/terrain.wgsl`'s `lod`), so it reaches as far as wanted for a few
//! kilobytes a tile. A tile is 32x32 cells of 2^level blocks; each cell is the column at its
//! middle: its top block, the one under it, its biome, and tree crowns over it. Where saved
//! cubes made a column's top other than the terrain's (a town, a crater), the server says so
//! (`server::lod_tops`) and that column is taken instead.
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

/// Columns whose top isn't the terrain's: height, top block, the one under it.
pub type Columns = FxHashMap<IVec2, (i32, u32, u32)>;

use crate::content::{Content, content};
use crate::gpu_layout::face;

/// Cells a tile side.
pub const CELLS: i32 = 32;
/// Columns a tile is made from: its cells and one around them (for the walls at its edges).
pub const SAMPLES: usize = crate::gpu_layout::terrain::lod::SAMPLES as usize;
const SIDE: i32 = crate::gpu_layout::terrain::lod::SIDE as i32;
/// Words per sample and per tile coming back from the GPU.
const WORDS: usize = crate::gpu_layout::terrain::lod::WORDS as usize;
pub const LOD_WORDS_PER_TILE: usize = SAMPLES * WORDS;
pub const MAX_LEVEL: u32 = 5;
/// From this level on tiles are smooth ground, morphing into the next level with distance
/// (`mesh_smooth`). chiacraft: past `MAX_LEVEL`, so every level is boxes. Far land keeps the block
/// steps the near land has, as Distant Horizons' does: the smooth facets from about 500 blocks out
/// read as another game, faceted snow peaks and curtains along far shores (Ronald, 11-05, after the
/// Distant Horizons + Bliss comparison: "I want to be better").
pub const SMOOTH_LEVEL: u32 = MAX_LEVEL + 1;
/// From this level on (8-block cells, about 500 blocks out) a crown is drawn down to the ground, a
/// forest's canopy as one block over the land as from afar it is: a trunk a cell's eighth wide or
/// less is under a pixel there, and without it the crown hung in the air.
const CANOPY_LEVEL: u32 = 3;
/// From this level on (8-block cells) a cell's walls are drawn as the slope it stands for: its top
/// block (bare rock where the dimension's faces say), steepness its mean drop per block.
const SLOPE_LEVEL: u32 = 3;
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
    /// What the server said of columns, taken over the terrain's where the tile samples them.
    pub columns: Arc<Columns>,
    /// When it was asked for (`diag::LOD_AGE_NS`).
    pub asked: std::time::Instant,
}

impl LodJob {
    pub fn live(&self) -> bool {
        self.live.load(Ordering::Relaxed)
    }
}

/// LOD's own threads, apart from the pool that finishes the near cubes: a third of the cores (3 to
/// 6), at that pool's low priority, so they share the CPU with it by count. At 3 threads its
/// meshing waited on base's steep faces (fixed in `Terrain::surface_block`); at half the cores and
/// normal priority (Distant Horizons gives its LOD that) the near cubes took 7 s to come in after
/// a teleport instead of 1.5 (PC, 11-05), worse than the far boxes it cleared.
pub fn pool() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        rayon::ThreadPoolBuilder::new()
            .num_threads((cores / 3).clamp(3, 6))
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
    pub asked: std::time::Instant,
}

/// What came back for `job`, meshed on a worker.
pub fn made(job: LodJob, mut words: Vec<u32>) {
    if !job.live() {
        return;
    }
    pool().spawn(move || {
        if !job.live() {
            return;
        }
        let start = std::time::Instant::now();
        if !job.columns.is_empty() {
            take_columns(&job, &mut words);
        }
        let (quads, skip, y) = if job.key.smooth() {
            mesh_smooth(content(), job.dim, job.seed, job.key, &words)
        } else {
            let (quads, y) = mesh(content(), job.dim, job.seed, job.key, &words);
            (quads, 0, y)
        };
        crate::diag::LOD_MESHED.add(1);
        crate::diag::LOD_MESH_NS.add(start.elapsed().as_nanos() as u64);
        let _ = job
            .done
            .send(LodMesh { dim: job.dim, seed: job.seed, key: job.key, quads, skip, y, asked: job.asked });
    });
}

/// Saved columns over the terrain's where the tile samples them (at the cells' middles, or a
/// smooth tile's corners): their ground, and no crown (whatever stands there now is in them).
fn take_columns(job: &LodJob, words: &mut [u32]) {
    let (s, o) = (1 << job.key.level, job.key.origin());
    let mid = if job.key.smooth() { 0 } else { s / 2 };
    for k in 0..SAMPLES {
        let (i, j) = ((k as i32 % SIDE) - 1, (k as i32 / SIDE) - 1);
        let col = o + IVec2::new(i * s + mid, j * s + mid);
        if let Some(&(y, top, under)) = job.columns.get(&col) {
            let w = k * WORDS;
            words[w] = (y as u32 & 0xFFFF) | (top << 16);
            words[w + 1] = (words[w + 1] & 0xFFFF) | (under << 16);
            words[w + 2] = 0;
            words[w + 3] = 0;
        }
    }
}

/// What a steep face shows (`Faces::block`, the dimension's mod), its surface lookups one column
/// each (`terrain::one_column`), timed for `diag`.
#[allow(clippy::too_many_arguments)]
fn face_block(f: crate::server::terrain::Faces, seed: u64, dim: u32, pos: [i32; 3], height: i32, biome: u32, fall: i32, block: u32) -> u32 {
    let start = std::time::Instant::now();
    let b = crate::server::terrain::one_column(|| f.block(seed, dim, pos, height, biome, fall, block));
    crate::diag::LOD_FACES.add(1);
    crate::diag::LOD_FACES_NS.add(start.elapsed().as_nanos() as u64);
    b
}

/// Sample (i, j)'s ground (from -1 to 32): its height, top block, biome and the block under.
fn ground(words: &[u32], i: i32, j: i32) -> (i32, u32, u32, u32) {
    let k = ((j + 1) * SIDE + i + 1) as usize * WORDS;
    let (a, b) = (words[k], words[k + 1]);
    ((a & 0xFFFF) as u16 as i16 as i32, a >> 16, b & 0xFFFF, b >> 16)
}

/// Trees' crowns over a sample, from its top block down to `bottom`, and the trunk standing in
/// its cell.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Crown {
    top: i32,
    bottom: i32,
    leaves: u32,
    trunk: Option<Trunk>,
}

/// A trunk: its place in its cell (x, z), width and block.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Trunk {
    x: i32,
    z: i32,
    width: i32,
    block: u32,
}

/// Sample (i, j)'s crown (`gpu_layout::terrain::lod_out`), if trees cover it.
fn crown(words: &[u32], i: i32, j: i32) -> Option<Crown> {
    use crate::gpu_layout::terrain::lod_out as o;
    let k = ((j + 1) * SIDE + i + 1) as usize * WORDS;
    let (a, b) = (words[k + 2], words[k + 3]);
    let leaves = a >> o::HALF_SHIFT;
    if leaves == 0 {
        return None;
    }
    let top = (a & o::HALF_MASK) as u16 as i16 as i32;
    let width = ((b >> o::TRUNK_W_SHIFT) & o::TRUNK_W_MASK) as i32;
    let block = b >> o::HALF_SHIFT;
    let trunk = (width > 0 && block != 0).then_some(Trunk {
        x: ((b >> o::TRUNK_DX_SHIFT) & o::PLACE_MASK) as i32,
        z: ((b >> o::TRUNK_DZ_SHIFT) & o::PLACE_MASK) as i32,
        width,
        block,
    });
    Some(Crown { top, bottom: top - (b & o::DEPTH_MASK) as i32, leaves, trunk })
}

/// What of `lo..=hi` stays open beside a neighbour whose ground reaches `ground` and whose crown
/// fills `filled`: at most two spans, above its ground and around its crown.
fn open_spans(lo: i32, hi: i32, ground: i32, filled: Option<(i32, i32)>) -> impl Iterator<Item = (i32, i32)> {
    let lo = lo.max(ground + 1);
    let (a, b) = match filled {
        Some((f0, f1)) if f0 <= hi && f1 >= lo => ((lo, f0 - 1), (f1 + 1, hi)),
        _ => ((lo, hi), (1, 0)),
    };
    [a, b].into_iter().filter(|(a, b)| a <= b)
}

/// A record of `world.wgsl`'s `vs_lod`: the `dir` face of a box at (x, y, z) in the tile, size
/// (sx, sy, sz) blocks, drawn in the average color of texture `tex` (its smallest mip), tinted
/// for `biome` when `tint`.
fn quad(dir: u32, x: i32, y: i32, z: i32, size: (i32, i32, i32), tex: u32, tint: u32, biome: u32) -> [u32; 4] {
    crate::gpu_layout::lod_record(dir, [x, y, z], [size.0, size.1, size.2], tex, tint, biome)
}

/// chiacraft: marks a record as a tree crown's face (`lod_face::CROWN_SHIFT`): packs shade far
/// canopies as crowns, lit on top and dark in the gaps and under them.
fn crowned(mut r: [u32; 4]) -> [u32; 4] {
    r[0] |= 1 << crate::gpu_layout::lod_face::CROWN_SHIFT;
    r
}

/// chiacraft: sets how much the land round shuts a record's face's sky out (`lod_face::OCC_SHIFT`, 0-3).
fn occluded(mut r: [u32; 4], occ: u32) -> [u32; 4] {
    r[0] |= occ.min(crate::gpu_layout::lod_face::OCC_MASK) << crate::gpu_layout::lod_face::OCC_SHIFT;
    r
}

/// chiacraft: how much of the sky the land round a point `h` high in sample (i, j) of a tile of
/// `s`-block cells hides, 0 (open) to 3, from `high` (its samples' ground or crowns, the highest):
/// the mean over eight ways of the steepest rise within three cells. Valley floors and the ground in
/// a forest's gaps go dark, ridges and canopy tops stay open (Ronald, 11-05, comparing with Distant
/// Horizons + Bliss: far land evenly lit; Planner: "a cheap far shadow or horizon-occlusion term").
fn openness(high: &dyn Fn(i32, i32) -> i32, i: i32, j: i32, h: i32, s: i32) -> u32 {
    const WAYS: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut shut = 0.0f32;
    for (dx, dz) in WAYS {
        let step = ((dx * dx + dz * dz) as f32).sqrt() * s as f32;
        let mut most = 0.0f32;
        for d in 1..=3 {
            let rise = (high((i + dx * d).clamp(-1, CELLS), (j + dz * d).clamp(-1, CELLS)) - h) as f32;
            if rise > 0.0 {
                let run = step * d as f32;
                most = most.max(rise / (rise * rise + run * run).sqrt());
            }
        }
        shut += most;
    }
    ((shut / 8.0 / 0.15).round() as u32).min(3)
}

/// The texture (its average color) and tint kind a distant face of `state` in direction `dir`
/// is drawn in. A box model's faces are its boxes' own (its block's textures may be unset, and
/// came out as the missing texture's magenta): its largest box's. A plant's is its picture.
fn face_of(c: &Content, looks: &super::looks::Looks, state: u32, dir: usize) -> (u32, u32) {
    let info = &c.states[state as usize];
    // No overlays out there.
    let tint = if crate::content::face_paint(info, dir, false).base_tinted { info.tint as u32 } else { 0 };
    let drawn = looks.drawn(state, info);
    let volume = |b: &crate::content::ModelBox| (0..3).map(|a| (b.max[a] - b.min[a]).abs()).product::<f32>();
    let tex = match drawn.model {
        weft_abi::BlockModel::Boxes => drawn
            .boxes
            .iter()
            .max_by(|a, b| volume(a).total_cmp(&volume(b)))
            .map_or(drawn.textures[dir], |b| b.textures[dir]),
        weft_abi::BlockModel::Cross | weft_abi::BlockModel::Wire => drawn.textures[0],
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
/// block's top color down a cell's height (a grass side averages to the dirt under its rim, and
/// every slope came out striped brown; one block of it still left rows of dirt dashes on far
/// terraces), then the block under it for the rest. Where the ground
/// falls steeply the dimension's faces (`Terrain::faces`) say what shows instead (bare rock), in
/// bands down the wall. Over the ground stand trees' crowns (`crown`): a box per cell from its top
/// down to its lowest leaves, rounded or tapering over the cells as the trees are, its sides where
/// the next cell's crown or ground doesn't hide them, and a trunk under it.
pub fn mesh(c: &Content, dim: u32, seed: u64, key: TileKey, words: &[u32]) -> (Vec<[u32; 4]>, (i32, i32)) {
    let s = 1 << key.level;
    let at = |i: i32, j: i32| ground(words, i, j);
    let canopy = key.level >= CANOPY_LEVEL;
    let slope = key.level >= SLOPE_LEVEL;
    let crown_at = |i: i32, j: i32| {
        crown(words, i, j).map(|c| if canopy { Crown { bottom: c.bottom.min(at(i, j).0 + 1), trunk: None, ..c } } else { c })
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
            // chiacraft: from 8-block cells (`SLOPE_LEVEL`) the mean drop per block: a cell that wide
            // stands for a slope, and the stepped fall's half again (`fall`) made whole mountainsides
            // cliffs, far snow ranges bare rock.
            let fall = if slope { (y - low).max(0) / s } else { fall(y - low, s) };
            if fall < f.fall {
                continue;
            }
            let (x, z) = (o.x + i * s + s / 2, o.y + j * s + s / 2);
            let shown = |wy: i32, before: u32| face_block(f, seed, dim, [x, wy, z], y, biome, fall, before);
            tops[k] = shown(y, top);
            // The wall below the top block, to the skirt's depth at most.
            let bottom = low.min(y - 1) - 2 * s;
            let step = ((y - 1 - bottom) + BANDS - 1) / BANDS;
            let mut bands: Vec<(i32, u32)> = Vec::new();
            let mut wy = y - 1;
            while wy > bottom && step > 0 {
                // A coarse cell's wall is the slope's face, not a cut through the ground: its top
                // block, bare where the face says (far snow ranges were stone walls under thin
                // white lines).
                let before = if slope {
                    top
                } else if y - wy <= FILLER {
                    under
                } else {
                    stone
                };
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
    // Per sample its highest point, ground or crown, and per cell how shut in its ground is.
    let high = |i: i32, j: i32| {
        let g = at(i, j).0;
        crown_at(i, j).map_or(g, |c| c.top.max(g))
    };
    let ground_occ: Vec<u32> = (0..CELLS * CELLS)
        .map(|k| openness(&high, k % CELLS, k / CELLS, at(k % CELLS, k / CELLS).0 + 1, s))
        .collect();
    let occ_at = |i: i32, j: i32| ground_occ[(j * CELLS + i) as usize];
    // A crown sitting right on the ground hides its top.
    let covered = |i: i32, j: i32| crown_at(i, j).is_some_and(|c| c.bottom <= at(i, j).0 + 1);
    for j in 0..CELLS {
        let mut i = 0;
        while i < CELLS {
            let (y, _, biome, _) = at(i, j);
            let top = top_at(i, j);
            let hidden = covered(i, j);
            let mut end = i + 1;
            let occ = occ_at(i, j);
            while end < CELLS
                && at(end, j).0 == y
                && top_at(end, j) == top
                && at(end, j).2 == biome
                && covered(end, j) == hidden
                && occ_at(end, j) == occ
            {
                end += 1;
            }
            if !hidden {
                let (tex, tint) = face_of(top, 3);
                out.push(occluded(quad(3, i * s, y, j * s, ((end - i) * s, 1, s), tex, tint, biome), occ));
            }
            (lo, hi) = (lo.min(y), hi.max(y + 1));
            i = end;
        }
    }
    let n_tops = out.len();
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
                    // Not under water beside water, which lies at the sea's height at every level:
                    // there is no crack to close, and a sheet of water hung down the tile's edge
                    // showed as a curtain along far shores (Far view, 11-05).
                    if under == top && ny >= y {
                        continue;
                    }
                    ny = ny.min(y - 1) - 2 * s;
                }
                if ny >= y {
                    continue;
                }
                // The top block's own color down the step, a cell's height at most: a gentle slope's
                // steps read as its grass from afar, not as rows of dirt dashes (bare faces keep
                // their bands under a block of it).
                // A fine cell's riser as tall as the faces' fall is a bare face in the cubes too (each
                // column at its edge falls that far): the faces say what it shows. Drawn in the top
                // block over the one under it, a mesa's sand terraces read as one cream slope where the
                // cubes show their banded risers, and where finer tiles met coarser ones (whose walls
                // ask the faces) a straight edge ran across the slope (Far view, 11-05: the "glass
                // slabs" of the forest vista and Tester 2's far snow shot).
                let riser = (!slope && walls[k].is_none() && top != under)
                    .then_some(faces)
                    .flatten()
                    .filter(|f| y - ny >= f.fall)
                    .map(|f| {
                        let (x, z) = (o.x + i * s + s / 2, o.y + j * s + s / 2);
                        face_block(f, seed, dim, [x, y - 1, z], y, biome, y - ny, under)
                    })
                    .filter(|&b| b != under);
                // (A coarse cell's whole wall: it stands for a slope of its top.)
                let band = if walls[k].is_some() || riser.is_some() {
                    1
                } else if slope {
                    y - ny
                } else {
                    s.min(y - ny)
                };
                let (tex, tint) = face_of(top, 3);
                out.push(occluded(quad(dir as u32, i * s, y + 1 - band, j * s, (s, band, s), tex, tint, biome), occ_at(i, j)));
                lo = lo.min(ny + 1);
                if y - band <= ny {
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
                            out.push(occluded(quad(dir as u32, i * s, b, j * s, (s, from + 1 - b, s), tex, tint, biome), occ_at(i, j)));
                            from = b - 1;
                        }
                    }
                    None => {
                        let (tex, tint) = face_of(riser.unwrap_or(under), dir);
                        out.push(occluded(quad(dir as u32, i * s, ny + 1, j * s, (s, y - band - ny, s), tex, tint, biome), occ_at(i, j)));
                    }
                }
            }
        }
    }
    let n_walls = out.len() - n_tops;
    let side = CELLS * s;
    // Crowns: tops and undersides in runs along x, as the ground's tops.
    for (dir, face_y) in [(3u32, true), (2, false)] {
        for j in 0..CELLS {
            let mut i = 0;
            while i < CELLS {
                let Some(c) = crown_at(i, j) else {
                    i += 1;
                    continue;
                };
                let biome = at(i, j).2;
                let y = if face_y { c.top } else { c.bottom };
                let shows = face_y || c.bottom > at(i, j).0 + 1;
                let crown_occ = |e: i32| if face_y { openness(&high, e, j, y + 1, s) } else { 0 };
                let occ = crown_occ(i);
                let same = |e: i32| {
                    crown_at(e, j).is_some_and(|d| {
                        d.leaves == c.leaves
                            && at(e, j).2 == biome
                            && (if face_y { d.top } else { d.bottom }) == y
                            && (face_y || d.bottom > at(e, j).0 + 1) == shows
                    }) && crown_occ(e) == occ
                };
                let mut end = i + 1;
                while end < CELLS && same(end) {
                    end += 1;
                }
                if shows {
                    let (tex, tint) = face_of(c.leaves, dir as usize);
                    out.push(occluded(crowned(quad(dir, i * s, y, j * s, ((end - i) * s, 1, s), tex, tint, biome)), occ));
                }
                (lo, hi) = (lo.min(c.bottom), hi.max(c.top + 1));
                i = end;
            }
        }
    }
    let n_flat = out.len() - n_tops - n_walls;
    for j in 0..CELLS {
        for i in 0..CELLS {
            let Some(c) = crown_at(i, j) else { continue };
            let (y, _, biome, _) = at(i, j);
            // Its sides, where the cell beside shows neither ground nor crown (at the tile's edge
            // the tile beside may be another level's: there the side is whole).
            for (dir, (di, dj)) in [(0usize, (-1, 0)), (1, (1, 0)), (4, (0, -1)), (5, (0, 1))] {
                let inside = (0..CELLS).contains(&(i + di)) && (0..CELLS).contains(&(j + dj));
                let (ny, filled) =
                    if inside { (at(i + di, j + dj).0, crown_at(i + di, j + dj).map(|n| (n.bottom, n.top))) } else { (y, None) };
                let (tex, tint) = face_of(c.leaves, dir);
                for (a, b) in open_spans(c.bottom, c.top, ny, filled) {
                    out.push(crowned(quad(dir as u32, i * s, a, j * s, (s, b + 1 - a, s), tex, tint, biome)));
                }
            }
            // Its trunk, from the ground to the crown, in the tile.
            let Some(t) = c.trunk else { continue };
            if c.bottom <= y + 1 {
                continue;
            }
            let w = t.width.min(side);
            let x = (i * s + t.x - (w - 1) / 2).clamp(0, side - w);
            let z = (j * s + t.z - (w - 1) / 2).clamp(0, side - w);
            for dir in [0u32, 1, 4, 5] {
                let (tex, tint) = face_of(t.block, dir as usize);
                out.push(quad(dir, x, y + 1, z, (w, c.bottom - 1 - y, w), tex, tint, biome));
            }
        }
    }
    crate::diag::lod_box_quads([n_tops, n_walls, n_flat, out.len() - n_tops - n_walls - n_flat]);
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
    // Corner (i, j), from -1 to 32: its ground, or a forest's crowns over it as a canopy (its top
    // block over the same, so no rock face).
    let at = |i: i32, j: i32| {
        let g = ground(words, i, j);
        crown(words, i, j).map_or(g, |c| (c.top, c.leaves, g.2, c.leaves))
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
                    top = face_block(f, seed, dim, [x, high, z], high, biome, fall, top);
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
    /// Tiles whose features' heights are being worked out (`request` keeps it under
    /// `heights_in_flight`).
    heights: Arc<std::sync::atomic::AtomicUsize>,
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
    /// What the server said of columns of `columns_dim`.
    columns: Arc<Columns>,
    columns_dim: u32,
    /// Made tiles to make again for columns that changed, shown as they are till then.
    stale: FxHashSet<TileKey>,
    /// Since when some wanted tiles are not made, and the most missing since (`diag`).
    missing_since: Option<(std::time::Instant, usize)>,
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
            heights: Arc::default(),
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
            columns: Arc::default(),
            columns_dim: u32::MAX,
            stale: FxHashSet::default(),
            missing_since: None,
        }
    }

    /// `ToClient::LodColumns`: tiles with those columns are made again.
    pub fn set_columns(&mut self, dim: u32, all: bool, cols: &[(i32, i32, i32, u32, u32)]) {
        if all || dim != self.columns_dim {
            self.columns = Arc::default();
            self.columns_dim = dim;
            self.stale.extend(self.ready.iter().copied());
        }
        let map = Arc::make_mut(&mut self.columns);
        for &(x, z, y, top, under) in cols {
            let p = IVec2::new(x, z);
            if top == u32::MAX {
                map.remove(&p);
            } else {
                map.insert(p, (y, top, under));
            }
            if !all {
                // Every level's tile with it, and the ones beside that sample it for their walls.
                for level in 0..=MAX_LEVEL {
                    let size = CELLS << level;
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            let key = TileKey {
                                level,
                                x: (x + dx * (size / CELLS)).div_euclid(size),
                                z: (z + dz * (size / CELLS)).div_euclid(size),
                            };
                            if self.ready.contains(&key) {
                                self.stale.insert(key);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn columns_len(&self) -> usize {
        self.columns.len()
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
            // Its description runs mods' shapes for a while: started now, ahead of the tiles.
            if self.custom.is_some() {
                let _ = super::terrain_gpu::lod_desc(dim, seed);
            }
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
            crate::diag::LOD_AGE_NS.add(m.asked.elapsed().as_nanos() as u64);
        }
        if reach == 0 || self.custom.is_none() {
            render.lod_draw.clear();
            render.lod_fade = 0;
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
        // shows only with its three siblings, never over its parent. A missing tile's parent is
        // asked for first (`request`), so the stand-in is one level coarser and hides at most its
        // three siblings, not every finished tile of a far coarser one.
        let mut keep = FxHashSet::default();
        let mut made = Vec::new();
        let mut standing = FxHashSet::default();
        let mut missing = 0;
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
            missing += 1;
            if key.level < MAX_LEVEL {
                keep.insert(key.parent());
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
        self.count_missing(missing);
        let mut shown: FxHashSet<TileKey> = made.into_iter().filter(|&k| !under_standing(k)).collect();
        shown.extend(standing.iter().copied());
        // It is a shell over the land with nothing under it: from deep below, its walls would hang
        // in the sky. Only rock is that way anyway.
        if self.deep_underground(c, cam) {
            shown.clear();
        }
        // The cubes fade into the tiles over the last blocks of the view (`WorldRenderer::lod_fade`),
        // a little inside its edge, where every cube is kept.
        render.lod_fade = if shown.is_empty() { 0 } else { (view * 16 - 8).max(0) as u32 };
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

    /// Keeps `diag`'s gauges, and says how long it took until every wanted tile was made again.
    fn count_missing(&mut self, missing: usize) {
        use std::sync::atomic::Ordering::Relaxed;
        crate::diag::LOD_WANTED.store(self.wanted.len() as u64, Relaxed);
        crate::diag::LOD_MISSING.store(missing as u64, Relaxed);
        crate::diag::LOD_PENDING.store(self.pending.len() as u64, Relaxed);
        match (&mut self.missing_since, missing) {
            (None, 0) => {}
            (None, n) => self.missing_since = Some((std::time::Instant::now(), n)),
            (Some((_, most)), n) if n > 0 => *most = (*most).max(n),
            (Some((since, most)), _) => {
                log::debug!(
                    target: "load",
                    "distant terrain: every wanted tile made ({} of them), {:.1} s after some were missing (at most {most})",
                    self.wanted.len(),
                    since.elapsed().as_secs_f64()
                );
                self.missing_since = None;
            }
        }
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

    /// Asks for the wanted tiles not made yet, or made before columns changed, nearest first in
    /// tile widths (distance over size): coarse before fine where they are as near, as a tile
    /// costs the same at every level (Distant Horizons loads coarse first), while the land by the
    /// camera still comes before the horizon (strictly coarse first left holes beside the player
    /// for the first seconds). A missing tile's parent goes first too, being half as many widths
    /// away: it stands in, one level coarser, until the tile is made, rather than a far coarser
    /// tile that hides every finished one under it, or nothing.
    fn request(&mut self) {
        let c = content();
        let custom = self.custom.clone().unwrap();
        let columns = if self.columns_dim == self.dim { self.columns.clone() } else { Arc::default() };
        let p = self.wanted_at.map_or(glam::DVec2::ZERO, |w| w.0);
        let widths = |k: TileKey| k.distance(p).0 / k.size() as f64;
        let mut order: Vec<(f64, TileKey)> = Vec::new();
        let mut parents = FxHashSet::default();
        for &key in &self.wanted {
            if self.ready.contains(&key) && !self.stale.contains(&key) {
                continue;
            }
            order.push((widths(key), key));
            // A stale tile still shows as it was: no stand-in for it.
            let up = key.parent();
            if !self.ready.contains(&key)
                && key.level < MAX_LEVEL
                && !self.ready.contains(&up)
                && !key.children().iter().all(|k| self.ready.contains(k))
                && parents.insert(up)
            {
                order.push((widths(up), up));
            }
        }
        order.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.level.cmp(&a.1.level)));
        let busy = self.heights.load(Ordering::Relaxed);
        let mut started = 0;
        for (_, key) in order {
            if self.pending.len() >= IN_FLIGHT {
                break;
            }
            // A few tiles' heights at a time: meshes and the description share the pool's queue,
            // and behind 256 heights jobs the first tiles came 3 s after a join.
            if !custom.is_empty() && busy + started >= heights_in_flight() {
                break;
            }
            let again = self.stale.remove(&key);
            if (self.ready.contains(&key) && !again) || self.pending.contains_key(&key) {
                continue;
            }
            let live = Arc::new(AtomicBool::new(true));
            self.pending.insert(key, live.clone());
            let (dim, seed, done, jobs, columns) = (self.dim, self.seed, self.done.0.clone(), self.jobs.clone(), columns.clone());
            let asked = std::time::Instant::now();
            if custom.is_empty() {
                let _ = jobs.send(LodJob { dim, seed, key, custom: Vec::new(), done, live, columns, asked });
                continue;
            }
            let custom = custom.clone();
            started += 1;
            let heights = self.heights.clone();
            heights.fetch_add(1, Ordering::Relaxed);
            // Mods' features run their code: on LOD's own threads.
            pool().spawn(move || {
                struct Done(Arc<std::sync::atomic::AtomicUsize>);
                impl Drop for Done {
                    fn drop(&mut self) {
                        self.0.fetch_sub(1, Ordering::Relaxed);
                    }
                }
                let _done = Done(heights);
                if !live.load(Ordering::Relaxed) {
                    return;
                }
                let start = std::time::Instant::now();
                let heights = custom_heights(c, dim, seed, key, &custom);
                crate::diag::LOD_HEIGHTS.add(1);
                crate::diag::LOD_HEIGHTS_NS.add(start.elapsed().as_nanos() as u64);
                let _ = jobs.send(LodJob { dim, seed, key, custom: heights, done, live, columns, asked });
            });
        }
        self.stale.retain(|k| self.ready.contains(k));
    }
}

/// Tiles' heights worked out at once at most: eight per LOD thread, about a fifth of a second of
/// them ahead of a made tile's meshing. Three per thread starved a fast flight (PC, 11-05: 100
/// tiles a second, 340 missing, against 120-300 and 250 with no cap); 256 kept the first tiles
/// back 1.5 s.
fn heights_in_flight() -> usize {
    pool().current_num_threads() * 8
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

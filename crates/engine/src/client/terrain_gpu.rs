//! Terrain made on the GPU (`shaders/terrain.wgsl`, its layout in `gpu_layout::terrain`): the dimension's terrain description and a
//! batch of cube positions go up, blocks, biomes and columns come back a couple of frames later,
//! and the rest of generation (the dimension's generator, trees, ores, structures, light) is
//! finished on the CPU workers. Integer math only on both sides, so a cube is the same bits
//! whoever made it (`core:terrain_check` compares every one).

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use weft_abi::entry_kind as entries;

use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;
use weft_abi::{TerrainColumn, expr as ek};
use weft_noise::fixed::seed32;
use weft_render::{Backend, BufferId, Cmd, CommandList, ComputeDesc, PipelineId, usage};

use crate::content::{Content, content};
use crate::server::area::{self, AreaFill};
use crate::server::terrain::Expr;
use crate::world::Cube;

use crate::gpu_layout::terrain::{self as layout, binding, d, words};

const OUT_WORDS: usize = words::OUT as usize;
const CUSTOM_WORDS: usize = words::CUSTOM as usize;
/// Cubes per frame at most.
const BATCH: usize = 4096;
/// Distant terrain tiles per frame at most (`lod`).
const LOD_BATCH: usize = 64;

pub static GPU_CUBES: AtomicU64 = AtomicU64::new(0);
pub static GPU_MISMATCHES: AtomicU64 = AtomicU64::new(0);

/// A cube for the GPU, with the heights of the features made on the CPU (mods' code runs here).
pub struct Job {
    pub epoch: u64,
    pub dim: u32,
    pub seed: u64,
    pub pos: IVec3,
    pub custom: Arc<Vec<i32>>,
    pub edits: Arc<Vec<AreaFill>>,
    pub done: Sender<super::terrain::Made>,
}

/// What `terrain.wgsl` reads for dimension `dim`, and the indices of the features made on the
/// CPU (`Feature::custom`: mods' code; their heights go up with each job); `None` when the GPU
/// cannot make it.
pub fn describe(c: &Content, dim: u32, seed: u64) -> Option<(Vec<i32>, Vec<usize>)> {
    let def = &c.dims[dim as usize];
    let t = def.terrain.as_ref().filter(|_| def.builtin.is_none())?;
    if c.states.len() > 65536 {
        return None;
    }
    let mut w = vec![0i32; d::LEN as usize];
    let mut put = |at: u32, v: i32| w[at as usize] = v;
    put(d::STONE, t.stone as i32);
    put(d::BASE, t.base);
    put(d::CONT_SCALE, t.continent_scale);
    if let Some((level, water)) = t.sea {
        put(d::HAS_SEA, 1);
        put(d::SEA, level);
        put(d::WATER, water as i32);
    }
    if let Some(caves) = &t.caves {
        put(d::HAS_CAVES, 1);
        put(d::LAVA, caves.lava as i32);
        put(d::LAVA_FROM, caves.lava_from);
        put(d::LAVA_TO, caves.lava_to);
    }
    put(d::FIELDS, t.fields.len() as i32);
    put(d::FEATURES, t.features.len() as i32);
    put(d::RULES, t.rules.len() as i32);
    put(d::CAVE_BIOMES, t.cave_biomes.len() as i32);
    for (i, (_, salt)) in layout::SEEDS.iter().enumerate() {
        put(d::SEEDS + i as u32, seed32(seed, *salt) as i32);
    }
    // The records of fields, features and rules; their steps and conditions go after them.
    w[d::FIELD_OFF as usize] = w.len() as i32;
    let at = w.len();
    let fw = words::FIELD as usize;
    w.resize(at + t.fields.len() * fw, 0);
    for (i, f) in t.fields.iter().enumerate() {
        let (start, len) = steps(&mut w, &f.expr, seed);
        w[at + i * fw..at + (i + 1) * fw].copy_from_slice(&[start, len]);
    }
    w[d::FEAT_OFF as usize] = w.len() as i32;
    let at = w.len();
    let fw = words::FEATURE as usize;
    w.resize(at + t.features.len() * fw, 0);
    let mut custom = Vec::new();
    for (i, f) in t.features.iter().enumerate() {
        let (height, mask) = (steps(&mut w, &f.height, seed), f.mask.as_ref().map_or((0, 0), |m| steps(&mut w, m, seed)));
        let slot = if f.custom.is_some() {
            custom.push(i);
            custom.len() as i32 - 1
        } else {
            -1
        };
        w[at + i * fw..at + (i + 1) * fw].copy_from_slice(&[height.0, height.1, mask.0, mask.1, slot]);
    }
    w[d::RULE_OFF as usize] = w.len() as i32;
    let at = w.len();
    let rw = words::RULE as usize;
    w.resize(at + t.rules.len() * rw, 0);
    let opt = |v: Option<u32>| v.map_or(-1, |v| v as i32);
    for (i, r) in t.rules.iter().enumerate() {
        let conditions = w.len() as i32;
        for &(field, from, to) in &r.ranges {
            w.extend_from_slice(&[field as i32, from, to]);
        }
        let under = r.under.map_or((-1, 0), |(b, n)| (b as i32, n));
        let deep = r.deep_top.map_or((-1, 0), |(b, y)| (b as i32, y));
        let bare = r.bare_above.map_or((0, 0), |y| (1, y));
        let rule: [i32; words::RULE as usize] = [
            r.biome as i32,
            r.height.0,
            r.height.1,
            conditions,
            r.ranges.len() as i32,
            opt(r.top),
            opt(r.filler),
            r.depth,
            under.0,
            under.1,
            deep.0,
            deep.1,
            opt(r.ice),
            bare.0,
            bare.1,
        ];
        w[at + i * rw..at + (i + 1) * rw].copy_from_slice(&rule);
    }
    w[d::CAVE_OFF as usize] = w.len() as i32;
    for (k, b) in t.cave_biomes.iter().enumerate() {
        let cave: [i32; words::CAVE as usize] =
            [b.biome as i32, b.depth, b.size, b.threshold, seed32(seed, weft_noise::fixed::salt::CAVE_BIOME ^ k as u64) as i32];
        w.extend_from_slice(&cave);
    }
    Some((w, custom))
}

/// Appends the steps of `e` (the noise seeds made from `seed`); returns where they start and how many.
fn steps(w: &mut Vec<i32>, e: &Expr, seed: u64) -> (i32, i32) {
    let start = w.len() as i32;
    for s in &e.0 {
        let noisy = matches!(s.kind, ek::NOISE | ek::SITE);
        w.extend_from_slice(&[s.kind as i32, if noisy { seed32(seed, s.salt) as i32 } else { 0 }, s.a, s.b, s.c]);
    }
    (start, e.0.len() as i32)
}

/// `describe` for distant terrain, with what steep faces need (`d::FACE_FALL`) and the
/// dimension's scattered things (trees) after it (`d::SCATTER_OFF` points there): per scatter
/// its cell, reach, hash seed, biome count and thinning (`ScatterThin`: its mask's place and
/// length, keep), the mask's steps, then per biome the chance as a threshold on the site's bits
/// (`dim::scatter_site`'s test, exactly), whether the thinning applies there and the crowns its
/// shape callback makes there (`crown_of`), each site drawn with one of them; then the ground it
/// stands on (`ScatterDef::ground`) as state ranges (count, then first and count each), none for
/// any.
///
/// The crowns come from sample runs of the shape on real ground spread over the world: a run at
/// one spot only said what that spot allowed (a clearing, a rock face), for every site.
// SYNC: upstream replaced the dimension's scatters (`DimensionDef::scatters`, `ScatterDef`,
// `abi::ScatterArgs`) with features (`features::FeatureDef`: `Spread::Grid` with
// `Chance::Density`, `Shape::Plan`). Until the far trees read them (grid spacing as the cell,
// `reach[0]`, the density, sample runs of the plan callback through `crown_of`, our thinning) and
// the sites' hash in `terrain.wgsl`'s `lod` is checked against `features`' grid starts, no
// scatters are described: distant terrain has no far trees.
pub fn describe_lod(c: &Content, dim: u32, seed: u64) -> Option<(Vec<i32>, Vec<usize>)> {
    let (mut w, custom) = describe(c, dim, seed)?;
    let def = &c.dims[dim as usize];
    w[d::FACE_FALL as usize] = def
        .terrain
        .as_ref()
        .and_then(|t| t.faces)
        .map_or(i32::MAX, |f| f.fall);
    w[d::SCATTER_OFF as usize] = w.len() as i32;
    w.push(0);
    Some((w, custom))
}

/// A block a sample run put (`crown_of`): where, relative to the site, which state, and whether
/// it was forced over what was there.
#[allow(dead_code)]
struct RunBlock {
    offset: glam::IVec3,
    state: u32,
    force: u32,
}

/// Sample runs of a scatter's shape per biome for its crowns, and the spots tried for them.
#[allow(dead_code)]
const RUNS: usize = 16;
#[allow(dead_code)]
const RUN_SPOTS: usize = 48;

/// What a run of a scatter's shape looks like from afar (`gpu_layout::terrain::crown`), and its
/// top: its crown, of the block it puts most into air (a tree's leaves; else the one it forces
/// most, a boulder's stone, as none at all was air, drawn violet), and the trunk under it. None
/// when it put nothing.
#[allow(dead_code)]
fn crown_of(blocks: &[RunBlock]) -> Option<(i32, [u32; 2])> {
    use layout::crown as cr;
    let commonest = |forced: bool, keep: &dyn Fn(&RunBlock) -> bool| {
        let mut n = rustc_hash::FxHashMap::<u32, u32>::default();
        for b in blocks
            .iter()
            .filter(|b| (b.force != 0) == forced && keep(b))
        {
            *n.entry(b.state).or_default() += 1;
        }
        n.into_iter().max_by_key(|e| (e.1, e.0)).map(|e| e.0)
    };
    // chiacraft: counted in the run's upper half, where a crown is: over the whole run a tall tree's
    // logs (a 2x2 trunk 30 high is 120 of them) outnumbered its leaves, and far off it stood as a
    // crown of logs from the ground up, a bare pillar (Far view, 11-05: the far forests' pillars).
    let high = blocks.iter().map(|b| b.offset.y).max()?;
    let upper = |b: &RunBlock| 2 * b.offset.y >= high;
    let leaves = commonest(false, &upper).or_else(|| commonest(true, &|_| true))?;
    let crown: Vec<_> = blocks.iter().filter(|b| b.state == leaves).collect();
    let top = crown.iter().map(|b| b.offset.y).max()?;
    let bottom = crown.iter().map(|b| b.offset.y).min()?.clamp(0, top);
    // Out from the trunk at a layer, and at all.
    let out = |b: &&RunBlock| ((b.offset.x * b.offset.x + b.offset.z * b.offset.z) as f32).sqrt();
    let width = |y: i32| {
        crown
            .iter()
            .filter(|b| b.offset.y == y)
            .map(out)
            .fold(0.0f32, f32::max)
    };
    let radius = crown.iter().map(out).fold(0.0f32, f32::max);
    // The widest layer, the middle one of a tie.
    let layers: Vec<f32> = (bottom..=top).map(width).collect();
    let most = layers.iter().copied().fold(0.0f32, f32::max);
    let wide: Vec<i32> = (bottom..=top)
        .filter(|&y| layers[(y - bottom) as usize] >= most - 0.01)
        .collect();
    let widest = (wide[0] + wide[wide.len() - 1]) / 2;
    let fifteenths = |part: i32, whole: i32| if whole <= 0 { 0 } else { ((part * 15 + whole / 2) / whole).clamp(0, 15) };
    let round =
        if radius < 0.5 || top == widest { 15 } else { ((width((widest + top + 1) / 2) / radius * 15.0).round() as i32).clamp(1, 15) };
    // The trunk: what stands under the crown at its middle, as wide as it is at the bottom.
    let under = |b: &RunBlock| b.state != leaves && b.offset.x.abs() <= 3 && b.offset.z.abs() <= 3 && (0..bottom).contains(&b.offset.y);
    let trunk = commonest(false, &under).or_else(|| commonest(true, &under));
    let trunk_width = trunk.map_or(0, |t| {
        let low = blocks
            .iter()
            .filter(|b| b.state == t && under(b))
            .map(|b| b.offset.y)
            .min()
            .unwrap_or(0);
        let n = blocks
            .iter()
            .filter(|b| b.state == t && under(b) && b.offset.y == low)
            .count();
        ((n as f32).sqrt().round() as u32).clamp(1, cr::TRUNK_MASK)
    });
    let shape = (radius.round() as u32).min(cr::RADIUS_MASK)
        | (top as u32).min(cr::TOP_MASK) << cr::TOP_SHIFT
        | (bottom as u32).min(cr::BOTTOM_MASK) << cr::BOTTOM_SHIFT
        | (fifteenths(widest - bottom, top - bottom) as u32) << cr::WIDEST_SHIFT
        | (round as u32) << cr::ROUND_SHIFT
        | trunk_width << cr::TRUNK_SHIFT;
    // Round 15 is never 0: a crown's first word always says it is there.
    Some((top, [shape, (leaves & cr::BLOCK_MASK) | (trunk.unwrap_or(0) & cr::BLOCK_MASK) << cr::TRUNK_BLOCK_SHIFT]))
}

/// Heights of the CPU-made features `custom` at the columns and corners of cube column (`x`, `z`),
/// for its jobs.
pub fn custom_heights(c: &Content, dim: u32, seed: u64, x: i32, z: i32, custom: &[usize]) -> Arc<Vec<i32>> {
    let def = &c.dims[dim as usize];
    def.custom_cache.get(seed, x, z, || {
        let Some(t) = def.terrain.as_ref() else { return Vec::new() };
        let f = t.fields(seed);
        let (ox, oz) = (x * 16, z * 16);
        let mut out = Vec::with_capacity(custom.len() * CUSTOM_WORDS);
        for &i in custom {
            for ci in 0..CUSTOM_WORDS as i32 {
                let (x, z) = if ci < 256 { (ox + ci % 16, oz + ci / 16) } else { (ox + (ci - 256) % 5 * 4, oz + (ci - 256) / 5 * 4) };
                out.push(t.features[i].height_at(&f, x * 16, z * 16));
            }
        }
        out
    })
}

#[derive(Default)]
struct Slot {
    desc: Option<(BufferId, usize)>,
    desc_key: Option<(u32, u64)>,
    jobs: Option<(BufferId, usize)>,
    /// Two, so the next batch goes into one while what came back in the other is read.
    out: [Option<(BufferId, usize)>; 2],
    /// Which `out` the batch in flight writes.
    flight_out: usize,
    flight: Vec<Job>,
    /// What came back, while workers still read it.
    reading: [Option<Arc<Readback>>; 2],
    /// The same for distant terrain tiles (`lod`), read back right away (a few kilobytes each).
    lod_desc: Option<(BufferId, usize)>,
    lod_desc_key: Option<(u32, u64)>,
    lod_jobs: Option<(BufferId, usize)>,
    lod_out: Option<(BufferId, usize)>,
    lod_flight: Vec<super::lod::LodJob>,
}

type LodDesc = ((u32, u64), Arc<std::sync::OnceLock<Option<(Vec<i32>, usize)>>>);

/// Distant terrain's description of (`dim`, `seed`) and its count of features made on the CPU,
/// worked out once on LOD's threads (it runs mods' shapes) by whoever asks first: `Lod` asks when
/// the dimension changes, so it is done by the time the first tile's job is in, not started then
/// behind every tile's heights. Inside: `None` while not done, then `None` when the GPU can't
/// make it.
pub fn lod_desc(dim: u32, seed: u64) -> LodDesc {
    static LAST: std::sync::Mutex<Option<LodDesc>> = std::sync::Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(d) = last.as_ref().filter(|d| d.0 == (dim, seed)) {
        return d.clone();
    }
    let cell = Arc::new(std::sync::OnceLock::new());
    let out = cell.clone();
    super::lod::pool().spawn(move || {
        let _ = out.set(describe_lod(content(), dim, seed).map(|(w, custom)| (w, custom.len())));
    });
    let d = ((dim, seed), cell);
    *last = Some(d.clone());
    d
}

/// A frame slot's output buffer as workers read it, straight from where the GPU wrote it.
struct Readback(*const u32, usize);

// The buffer is neither written nor freed while anyone holds this (`Slot::reading`).
unsafe impl Send for Readback {}
unsafe impl Sync for Readback {}

impl Readback {
    fn cube(&self, i: usize) -> &[u32] {
        assert!((i + 1) * OUT_WORDS <= self.1);
        unsafe { std::slice::from_raw_parts(self.0.add(i * OUT_WORDS), OUT_WORDS) }
    }
}

/// The main thread's end: dispatches batches into the frame and hands what came back to the
/// workers.
pub struct GpuTerrain {
    pipeline: Option<PipelineId>,
    lod_pipeline: Option<PipelineId>,
    pub far_lod_pipeline: Option<PipelineId>,
    rx: Receiver<Job>,
    queue: VecDeque<Job>,
    /// Distant terrain tiles asked for (`lod::Lod`).
    pub lod_tx: Sender<super::lod::LodJob>,
    pub lod_shader: Option<u64>,
    lod_rx: Receiver<super::lod::LodJob>,
    lod_queue: VecDeque<super::lod::LodJob>,
    slots: Vec<Slot>,
    /// (dim, seed) and its description and mod features, for cubes and for tiles.
    desc: Option<((u32, u64), Vec<i32>, usize)>,
    /// The same for tiles, worked out on LOD's threads (it runs mods' shapes): `None` inside
    /// while not done, then `None` when the GPU can't make it.
    lod_desc: Option<LodDesc>,
    /// Set once the pipeline works: the terrain thread sends jobs here instead of to the CPU.
    pub on: Arc<AtomicBool>,
}

/// The terrain's pipelines: label and entry point.
const PIPELINES: [(&str, &str); 2] = [("terrain", "generate"), ("terrain lod", "lod")];
const FAR_LOD_PIPELINE: (&str, &str) = ("far terrain lod", "lod_far");

// chiacraft: our far terrain (`client::lod`) stays ours until an A/B against upstream's; its
// `lod_far` entry (GPU coarse terrain) isn't in our `terrain.wgsl`, so the pipeline isn't made.
// SYNC: upstream's far LOD (`lod_far`, `world_render::far_lod`) comes back with the LOD A/B.
fn far_lod_enabled() -> bool {
    false
}

impl GpuTerrain {
    /// Its pipelines compiled ahead on another thread (`Backend::precompile`), from the title
    /// screen on: a phone's driver needs most of a minute for them the first time.
    pub fn precompile(gpu: &mut dyn Backend, assets: &crate::assets::Assets) {
        if !enabled() {
            return;
        }
        match Self::shader(assets) {
            Ok(s) => {
                for (label, entry) in PIPELINES
                    .into_iter()
                    .chain(far_lod_enabled().then_some(FAR_LOD_PIPELINE))
                {
                    gpu.precompile(&ComputeDesc { label, shader: &s, entry });
                }
            }
            Err(e) => log::error!("terrain shader: {e}"),
        }
    }

    // Not a shader packs may replace: it has to make the CPU's blocks exactly.
    fn shader(assets: &crate::assets::Assets) -> Result<weft_render::Shader, String> {
        crate::packs::engine_shader(assets, "terrain.wgsl", &layout::prelude()).map_err(|e| e.0)
    }

    pub fn new(gpu: &mut dyn Backend, assets: &crate::assets::Assets, rx: Receiver<Job>, on: Arc<AtomicBool>) -> GpuTerrain {
        let pipelines = Self::shader(assets).and_then(|s| {
            let [terrain, lod] = PIPELINES.map(|(label, entry)| {
                gpu.create_compute(&ComputeDesc { label, shader: &s, entry })
                    .map_err(|e| e.0)
            });
            let (terrain, lod) = (terrain?, lod?);
            let far = far_lod_enabled()
                .then(|| {
                    let (label, entry) = FAR_LOD_PIPELINE;
                    gpu.create_compute(&ComputeDesc { label, shader: &s, entry })
                        .map_err(|e| log::warn!("far LOD compute: {e}"))
                        .ok()
                })
                .flatten();
            Ok((terrain, lod, far, s.hash))
        });
        let (pipeline, lod_pipeline, far_lod_pipeline, lod_shader) = match pipelines {
            Ok((p, l, far, hash)) => (Some(p), Some(l), far, Some(hash)),
            Err(e) => {
                log::error!("terrain on the GPU unavailable, generating on the CPU: {e}");
                (None, None, None, None)
            }
        };
        on.store(pipeline.is_some() && enabled(), Ordering::Relaxed);
        let (lod_tx, lod_rx) = crossbeam_channel::unbounded();
        GpuTerrain {
            pipeline,
            lod_pipeline,
            far_lod_pipeline,
            rx,
            queue: VecDeque::new(),
            lod_tx,
            lod_shader,
            lod_rx,
            lod_queue: VecDeque::new(),
            slots: Vec::new(),
            desc: None,
            lod_desc: None,
            on,
        }
    }

    /// Frees its buffers, once the workers reading them are done.
    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        for s in self.slots.drain(..) {
            while s.reading.iter().flatten().any(|r| Arc::strong_count(r) > 1) {
                std::thread::yield_now();
            }
            for (b, _) in [s.desc, s.jobs, s.out[0], s.out[1], s.lod_desc, s.lod_jobs, s.lod_out]
                .into_iter()
                .flatten()
            {
                gpu.destroy_buffer(b);
            }
        }
        for p in [self.pipeline.take(), self.lod_pipeline.take(), self.far_lod_pipeline.take()]
            .into_iter()
            .flatten()
        {
            gpu.destroy_pipeline(p);
        }
    }

    /// Takes in what the GPU finished in this slot's last frame and records the next batches.
    pub fn frame(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, slot: usize) {
        let Some(pipeline) = self.pipeline else { return };
        self.on.store(enabled(), Ordering::Relaxed);
        while self.slots.len() < gpu.frames_in_flight() {
            self.slots.push(Slot::default());
        }
        self.cubes(gpu, list, slot, pipeline);
        self.tiles(gpu, list, slot);
    }

    /// Distant terrain tiles: what came back goes to the workers to mesh, the next ones go up.
    fn tiles(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, slot: usize) {
        use super::lod::{LOD_WORDS_PER_TILE, SAMPLES};
        let Some(pipeline) = self.lod_pipeline else { return };
        let s = &mut self.slots[slot];
        if !s.lod_flight.is_empty() {
            let (out, _) = s.lod_out.unwrap();
            let words: &[u32] = bytemuck::cast_slice(&gpu.mapped(out)[..s.lod_flight.len() * LOD_WORDS_PER_TILE * 4]);
            for (i, job) in s.lod_flight.drain(..).enumerate() {
                super::lod::made(job, words[i * LOD_WORDS_PER_TILE..(i + 1) * LOD_WORDS_PER_TILE].to_vec());
            }
        }
        self.lod_queue.extend(self.lod_rx.try_iter());
        // Tiles called off meanwhile (`LodJob::live`) aren't made.
        self.lod_queue.retain(super::lod::LodJob::live);
        let Some(first) = self.lod_queue.front() else { return };
        let key = (first.dim, first.seed);
        if self.lod_desc.as_ref().is_none_or(|d| d.0 != key) {
            self.lod_desc = Some(lod_desc(key.0, key.1));
        }
        let cell = self.lod_desc.as_ref().unwrap().1.clone();
        let Some(made) = cell.get() else { return };
        let Some((words, custom)) = made else {
            self.lod_queue.clear();
            return;
        };
        let mut batch = Vec::new();
        while batch.len() < LOD_BATCH
            && self
                .lod_queue
                .front()
                .is_some_and(|j| (j.dim, j.seed) == key)
        {
            batch.push(self.lod_queue.pop_front().unwrap());
        }
        let stride = 4 + custom * SAMPLES;
        let mut jobs = vec![0i32; batch.len() * stride];
        log::debug!(target: "load", "lod dispatch {} tiles", batch.len());
        for (i, j) in batch.iter().enumerate() {
            let r = i * stride;
            let o = j.key.origin();
            jobs[r..r + 4].copy_from_slice(&[o.x, o.y, j.key.level as i32, j.key.smooth() as i32]);
            if j.custom.len() == custom * SAMPLES {
                jobs[r + 4..r + stride].copy_from_slice(&j.custom);
            }
        }
        let desc = sized(gpu, &mut s.lod_desc, words.len() * 4, usage::STORAGE | usage::HOST, "terrain lod desc");
        if s.lod_desc_key != Some(key) {
            gpu.mapped(desc)[..words.len() * 4].copy_from_slice(bytemuck::cast_slice(words));
            s.lod_desc_key = Some(key);
        }
        let jobs_buf = sized(gpu, &mut s.lod_jobs, LOD_BATCH * stride * 4, usage::STORAGE | usage::HOST, "terrain lod jobs");
        gpu.mapped(jobs_buf)[..jobs.len() * 4].copy_from_slice(bytemuck::cast_slice(&jobs));
        let out = sized(gpu, &mut s.lod_out, LOD_BATCH * LOD_WORDS_PER_TILE * 4, usage::STORAGE | usage::READBACK, "terrain lod out");
        list.push(Cmd::SetPipeline(pipeline));
        list.bind(binding::DESC, desc);
        list.bind(binding::JOBS, jobs_buf);
        list.bind(binding::OUT, out);
        list.push_constants(&[batch.len() as u32, *custom as u32]);
        list.push(Cmd::Dispatch { x: batch.len() as u32, y: 1, z: 1 });
        list.push(Cmd::Barrier);
        s.lod_flight = batch;
    }

    /// Cubes: what came back goes to the workers to finish, the next batch goes up.
    fn cubes(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, slot: usize, pipeline: PipelineId) {
        let s = &mut self.slots[slot];
        if !s.flight.is_empty() {
            let (out, _) = s.out[s.flight_out].unwrap();
            let words: &[u32] = bytemuck::cast_slice(&gpu.mapped(out)[..s.flight.len() * OUT_WORDS * 4]);
            let back = Arc::new(Readback(words.as_ptr(), words.len()));
            // A column's cubes come together: what they share is worked out first, once.
            let mut columns: rustc_hash::FxHashMap<(i32, i32), Vec<(usize, Job)>> = Default::default();
            for (i, job) in s.flight.drain(..).enumerate() {
                columns
                    .entry((job.pos.x, job.pos.z))
                    .or_default()
                    .push((i, job));
            }
            // Unpacked on threads of their own (the other pools are busy while loading): the
            // slot waits for it.
            for ((x, z), jobs) in columns {
                let back = back.clone();
                unpacking().spawn(move || {
                    let made: Vec<(Job, Vec<u32>)> = jobs
                        .into_iter()
                        .map(|(i, job)| (job, back.cube(i).to_vec()))
                        .collect();
                    drop(back);
                    crate::features::warm(content(), made[0].0.dim, made[0].0.seed, x, z);
                    for (job, words) in made {
                        crate::client_terrain().spawn(move || finish(job, unpack(&words)));
                    }
                });
            }
            s.reading[s.flight_out] = Some(back);
        }
        for r in &mut s.reading {
            if r.as_ref().is_some_and(|r| Arc::strong_count(r) == 1) {
                *r = None;
            }
        }
        let Some(next) = (0..2).find(|&k| s.reading[k].is_none()) else { return };
        self.queue.extend(self.rx.try_iter());
        let Some(first) = self.queue.front() else { return };
        let key = (first.dim, first.seed);
        if self.desc.as_ref().is_none_or(|d| d.0 != key) {
            let Some((words, custom)) = describe(content(), key.0, key.1) else {
                // The terrain thread asked for a dimension the GPU cannot make: done by the CPU.
                for job in self.queue.drain(..) {
                    crate::client_terrain().spawn(move || cpu(job));
                }
                return;
            };
            self.desc = Some((key, words, custom.len()));
        }
        let (_, words, custom) = self.desc.as_ref().unwrap();
        let mut batch = Vec::new();
        while batch.len() < BATCH && self.queue.front().is_some_and(|j| (j.dim, j.seed) == key) {
            batch.push(self.queue.pop_front().unwrap());
        }
        let stride = 4 + custom * CUSTOM_WORDS;
        let mut jobs = vec![0i32; batch.len() * stride];
        for (i, j) in batch.iter().enumerate() {
            let r = i * stride;
            jobs[r..r + 3].copy_from_slice(&j.pos.to_array());
            if j.custom.len() == custom * CUSTOM_WORDS {
                jobs[r + 4..r + stride].copy_from_slice(&j.custom);
            }
        }
        let desc = sized(gpu, &mut s.desc, words.len() * 4, usage::STORAGE | usage::HOST, "terrain desc");
        if s.desc_key != Some(key) {
            gpu.mapped(desc)[..words.len() * 4].copy_from_slice(bytemuck::cast_slice(words));
            s.desc_key = Some(key);
        }
        let jobs_buf = sized(gpu, &mut s.jobs, BATCH * stride * 4, usage::STORAGE | usage::HOST, "terrain jobs");
        gpu.mapped(jobs_buf)[..jobs.len() * 4].copy_from_slice(bytemuck::cast_slice(&jobs));
        let out = sized(gpu, &mut s.out[next], BATCH * OUT_WORDS * 4, usage::STORAGE | usage::READBACK, "terrain out");
        s.flight_out = next;
        list.push(Cmd::SetPipeline(pipeline));
        list.bind(binding::DESC, desc);
        list.bind(binding::JOBS, jobs_buf);
        list.bind(binding::OUT, out);
        list.push_constants(&[batch.len() as u32, *custom as u32]);
        list.push(Cmd::Dispatch { x: batch.len() as u32, y: 1, z: 1 });
        crate::diag::TERRAIN_GPU.add(batch.len() as u64);
        list.push(Cmd::Barrier);
        s.flight = batch;
    }
}

fn unpacking() -> &'static rayon::ThreadPool {
    static POOL: crate::Pool = crate::Pool::new();
    crate::pool(&POOL, "terrain unpack", 4, true)
}

fn enabled() -> bool {
    let c = content();
    c.game
        .catalog
        .ids(entries::CONFIG)
        .get("core:terrain_gpu")
        .is_none_or(|&id| crate::config::get(id) != 0.0)
}

fn checking() -> bool {
    let c = content();
    c.game
        .catalog
        .ids(entries::CONFIG)
        .get("core:terrain_check")
        .is_some_and(|&id| crate::config::get(id) != 0.0)
}

/// A buffer of at least `bytes`, made again when too small.
fn sized(gpu: &mut dyn Backend, slot: &mut Option<(BufferId, usize)>, bytes: usize, usage: u32, label: &str) -> BufferId {
    match *slot {
        Some((b, size)) if size >= bytes => b,
        old => {
            if let Some((b, _)) = old {
                gpu.destroy_buffer(b);
            }
            let b = gpu.create_buffer(bytes.max(4) as u64, usage, label);
            *slot = Some((b, bytes));
            b
        }
    }
}

/// The CPU's way, for jobs the GPU cannot take.
fn cpu(job: Job) {
    let c = content();
    let mut cube = crate::server::dim::generate(c, job.dim, job.seed, job.pos);
    done(c, job, &mut cube);
}

/// Unpacks what the GPU made and finishes the cube like the CPU would.
/// A cube as the GPU made it: blocks, biomes, columns.
type Unpacked = (Vec<u32>, [u16; 256], [TerrainColumn; 256]);

fn unpack(w: &[u32]) -> Unpacked {
    let blocks: Vec<u32> = (0..4096)
        .map(|i| (w[i / 2] >> ((i & 1) * 16)) & 0xFFFF)
        .collect();
    let half = |at: usize, i: usize| ((w[at + i / 2] >> ((i & 1) * 16)) & 0xFFFF) as u16;
    use crate::gpu_layout::terrain::out;
    let biomes: [u16; 256] = std::array::from_fn(|i| half(out::BIOMES as usize, i));
    let cols: [TerrainColumn; 256] = std::array::from_fn(|i| TerrainColumn {
        height: w[out::HEIGHTS as usize + i] as i32,
        biome: half(out::COLUMN_BIOMES as usize, i) as u32,
    });
    (blocks, biomes, cols)
}

fn finish(job: Job, (blocks, biomes, cols): Unpacked) {
    let t = std::time::Instant::now();
    let c = content();
    let mut cube = crate::server::dim::finish_generated(c, job.dim, job.seed, job.pos, blocks, biomes, Some(Arc::new(cols)), false);
    GPU_CUBES.fetch_add(1, Ordering::Relaxed);
    if checking() {
        let want = crate::server::dim::generate(c, job.dim, job.seed, job.pos);
        let differs = (0..4096).find(|&i| cube.get(i) != want.get(i));
        let biome_differs = (0..256).find(|&i| cube.biome(i % 16, i / 16) != want.biome(i % 16, i / 16));
        // Light too, against the way it was done before (lit as it will be below).
        let mut lit = cube.clone();
        area::catch_up(&job.edits, job.pos, &mut lit);
        let mut old = lit.clone();
        crate::world::light::prelight(c, &mut lit, c.dims[job.dim as usize].sky_light);
        crate::world::light::prelight_reference(c, &mut old, c.dims[job.dim as usize].sky_light);
        let light_differs = (0..4096).find(|&i| lit.light(i) != old.light(i));
        if let Some(i) = light_differs {
            if GPU_MISMATCHES.fetch_add(1, Ordering::Relaxed) < 8 {
                log::warn!("prelight differs at cube {} block {i}: {:#x} instead of {:#x}", job.pos, lit.light(i), old.light(i));
            }
        }
        if differs.is_some() || biome_differs.is_some() {
            if GPU_MISMATCHES.fetch_add(1, Ordering::Relaxed) < 8 {
                match differs {
                    Some(i) => log::warn!("GPU terrain differs at cube {} block {i}: {} instead of {}", job.pos, cube.get(i), want.get(i)),
                    None => log::warn!("GPU terrain differs at cube {} in biomes", job.pos),
                }
            }
        }
    }
    done(c, job, &mut cube);
    crate::diag::TERRAIN_FINISHED.add(1);
    crate::diag::TERRAIN_FINISH_NS.add(t.elapsed().as_nanos() as u64);
}

fn done(c: &Content, job: Job, cube: &mut Cube) {
    crate::server::dim::prepare(c, job.dim, &job.edits, job.pos, cube);
    let _ = job
        .done
        .send((job.epoch, job.pos, std::mem::replace(cube, Cube::uniform(0, 0))));
}

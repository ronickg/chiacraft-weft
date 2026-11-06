//! Ray tracing's scene (`core:rt_shadows`, where the GPU has ray queries): a bottom-level
//! acceleration structure per meshed cube (its faces and its free quads apart, so a ray can skip
//! the plants) and per distant terrain tile, built from the records already on the GPU by a small
//! compute pass (`rt_expand.wgsl`) and kept until the mesh changes; and a top-level one over them
//! all, made again each frame about the camera.
//!
//! Each BLAS has up to two geometries: the opaque records, and the see-through ones (cutout, then
//! translucent), whose records are also copied into `records` so a ray's hit there can look up
//! its texel (a leaf's holes, a pane's colour). A TLAS instance's custom index picks its row of
//! the frame's instance table: x the copy's first word (`u32::MAX`: none), y the record where the
//! translucent ones start in it, z the kind (0 faces, 1 quads, 2 distant terrain).

use super::*;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// Records expanded per frame at most, and BLASes built: a world loading spreads its builds over
/// frames (a frame of a full load at 32 chunks would otherwise build thousands at once).
const FRAME_RECORDS: u32 = 1 << 19;
const FRAME_BUILDS: usize = 384;
/// Bytes for the see-through records' copies (16-byte units): about 16 million faces' worth.
const RECORDS_BYTES: u64 = 256 << 20;
const UNIT: u64 = 16;
const NONE: u32 = u32::MAX;
/// TLAS instance masks: a cube's faces, its free quads (plants, models), distant terrain.
pub const MASK_FACES: u8 = 1;
pub const MASK_QUADS: u8 = 2;
pub const MASK_LOD: u8 = 4;
const KIND_FACES: u32 = 0;
const KIND_QUADS: u32 = 1;
const KIND_LOD: u32 = 2;

/// `rt_expand.wgsl`'s push constants.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Job {
    src: u32,
    count: u32,
    dst: u32,
    kind: u32,
    copy: u32,
    copy_from: u32,
    pad: [u32; 2],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Item {
    Faces(IVec3),
    Quads(IVec3),
    Lod(super::super::lod::TileKey),
}

struct Built {
    accel: AccelId,
    /// Its see-through records' copy in `records`: first unit and units.
    copy: Option<(u32, u32)>,
    /// Its row of the instance table.
    info: [u32; 4],
}

pub(super) struct RtScene {
    expand: PipelineId,
    tlas: AccelId,
    /// (0 1 2)(0 2 3) for this many records.
    index: (BufferId, u32),
    /// Per frame slot: the corners the frame's expansions write, bytes.
    scratch: Vec<(BufferId, u64)>,
    pub records: BufferId,
    free: FreeList,
    full_warned: bool,
    built: FxHashMap<Item, Built>,
    /// What to build: changed meshes first (they cast already), then new ones.
    urgent: VecDeque<Item>,
    queue: VecDeque<Item>,
    queued: FxHashSet<Item>,
    /// Copies to free once no frame in flight reads them: (frame they went in, first unit, units).
    later: Vec<(u64, u32, u32)>,
    frame_no: u64,
    /// Per frame slot: the instance table.
    table: Vec<(BufferId, u32)>,
    instances: Vec<TlasInstance>,
    rows: Vec<[u32; 4]>,
    /// Of the last frame: BLASes built, records expanded, TLAS instances.
    pub last: (u32, u32, u32),
    /// The frame slot of the last `rt_frame`, whose instance table the draws bind.
    slot: usize,
    /// Since the last log line (every 5 s while anything was built): BLASes built, records.
    since: (Option<std::time::Instant>, u32, u64),
}

/// `core:rt_shadows`, or off where the engine's configs are not loaded.
pub fn rt_setting() -> bool {
    crate::content::content()
        .game
        .config_names
        .get("core:rt_shadows")
        .is_some_and(|&id| crate::config::get(id) > 0.5)
}

impl RtScene {
    /// The expansion's pipeline (`expand_pipeline`), made with the world's other pipelines.
    pub(super) fn expand_pipeline(gpu: &mut dyn Backend, assets: &crate::assets::Assets) -> Option<PipelineId> {
        if !gpu.ray_query() {
            return None;
        }
        let shader = crate::packs::engine_shader(assets, "rt_expand.wgsl", &crate::gpu_layout::rt_expand_prelude())
            .map_err(|e| log::error!("rt_expand.wgsl: {}", e.0))
            .ok()?;
        gpu.create_compute(&ComputeDesc { label: "rt expand", shader: &shader, entry: "cs_expand" })
            .map_err(|e| log::error!("rt expand: {}", e.0))
            .ok()
    }

    pub(super) fn new(gpu: &mut dyn Backend, expand: PipelineId) -> RtScene {
        let records = gpu.create_buffer(RECORDS_BYTES, usage::STORAGE, "rt records");
        let mut free = FreeList::default();
        free.add(0, (RECORDS_BYTES / UNIT) as u32);
        RtScene {
            expand,
            tlas: gpu.create_tlas(4096, "scene TLAS"),
            index: (gpu.create_buffer(16, usage::STORAGE, "rt index"), 0),
            scratch: Vec::new(),
            records,
            free,
            full_warned: false,
            built: FxHashMap::default(),
            urgent: VecDeque::new(),
            queue: VecDeque::new(),
            queued: FxHashSet::default(),
            later: Vec::new(),
            frame_no: 0,
            table: Vec::new(),
            instances: Vec::new(),
            rows: Vec::new(),
            last: (0, 0, 0),
            slot: 0,
            since: (None, 0, 0),
        }
    }

    /// The see-through records' copies and the last frame's instance table.
    pub(super) fn tables(&self) -> Option<(BufferId, BufferId)> {
        Some((self.records, self.table.get(self.slot)?.0))
    }

    pub(super) fn tlas(&self) -> AccelId {
        self.tlas
    }

    pub(super) fn destroy(mut self, gpu: &mut dyn Backend) {
        for (_, b) in self.built.drain() {
            gpu.destroy_accel(b.accel);
        }
        gpu.destroy_accel(self.tlas);
        for b in [self.index.0, self.records]
            .into_iter()
            .chain(self.scratch.drain(..).map(|s| s.0))
            .chain(self.table.drain(..).map(|t| t.0))
        {
            gpu.destroy_buffer(b);
        }
    }

    fn want(&mut self, item: Item) {
        if self.queued.insert(item) {
            if self.built.contains_key(&item) {
                self.urgent.push_back(item);
            } else {
                self.queue.push_back(item);
            }
        }
    }

    /// A cube's mesh changed or went: built again, or dropped, at the next frame (until then the
    /// old one still casts, so an edit never leaves a frame without its shadow).
    pub(super) fn mesh_changed(&mut self, key: MeshKey) {
        if key.0 != 0 {
            return;
        }
        self.want(Item::Faces(key.1));
        self.want(Item::Quads(key.1));
    }

    pub(super) fn lod_changed(&mut self, key: super::super::lod::TileKey) {
        self.want(Item::Lod(key));
    }

    fn drop_item(&mut self, gpu: &mut dyn Backend, item: Item) {
        if let Some(b) = self.built.remove(&item) {
            gpu.destroy_accel(b.accel);
            if let Some((start, len)) = b.copy {
                self.later.push((self.frame_no, start, len));
            }
        }
    }

    fn ensure_index(&mut self, gpu: &mut dyn Backend, records: u32) {
        if self.index.1 >= records {
            return;
        }
        let n = records.next_power_of_two().max(4096);
        gpu.destroy_buffer(self.index.0);
        let b = gpu.create_buffer(n as u64 * 24, usage::STORAGE | usage::ACCEL_INPUT, "rt index");
        let idx: Vec<u32> = (0..n)
            .flat_map(|q| {
                let v = q * 4;
                [v, v + 1, v + 2, v, v + 2, v + 3]
            })
            .collect();
        gpu.write_buffer(b, 0, bytemuck::cast_slice(&idx));
        self.index = (b, n);
    }

    fn ensure_scratch(&mut self, gpu: &mut dyn Backend, slot: usize, bytes: u64) -> BufferId {
        while self.scratch.len() < gpu.frames_in_flight() {
            let size = FRAME_RECORDS as u64 * 48;
            self.scratch
                .push((gpu.create_buffer(size, usage::STORAGE | usage::ACCEL_INPUT, "rt corners"), size));
        }
        if self.scratch[slot].1 < bytes {
            gpu.destroy_buffer(self.scratch[slot].0);
            let size = bytes.next_power_of_two();
            self.scratch[slot] = (gpu.create_buffer(size, usage::STORAGE | usage::ACCEL_INPUT, "rt corners"), size);
        }
        self.scratch[slot].0
    }

    fn alloc_copy(&mut self, units: u32) -> Option<u32> {
        let &(len, start) = self.free.by_len.range((units, 0)..).next()?;
        self.free.take(start, len);
        if len > units {
            self.free.add(start + units, len - units);
        }
        Some(start)
    }

    fn free_copy(&mut self, start: u32, len: u32) {
        let free = &mut self.free;
        let (mut start, mut len) = (start, len);
        if let Some((&s, &l)) = free
            .by_start
            .range(..start)
            .next_back()
            .filter(|(s, l)| **s + **l == start)
        {
            free.take(s, l);
            (start, len) = (s, l + len);
        }
        if let Some(&l) = free.by_start.get(&(start + len)) {
            free.take(start + len, l);
            len += l;
        }
        free.add(start, len);
    }
}

/// One BLAS to build this frame: what to expand and its geometries' record counts.
struct Plan {
    item: Item,
    page: BufferId,
    src: u32,
    kind: u32,
    /// Opaque records, then see-through ones (cutout, then translucent from `translucent`).
    opaque: u32,
    clear: u32,
    translucent: u32,
}

impl WorldRenderer {
    /// Ray tracing's scene on or off (`core:rt_shadows` where the GPU has ray queries). Turned
    /// on, every mesh and distant tile held is queued for its BLAS, and the world shader is
    /// composed again to trace (`rt_reload`); turned off, the scene stays until the shader that
    /// binds it is gone.
    pub fn set_rt(&mut self, gpu: &mut dyn Backend, on: bool) {
        let on = on && self.rt_expand.is_some();
        self.rt_wanted = on;
        if (on && self.rt_pack) != self.rt_compiled {
            self.rt_reload = true;
        }
        if !on {
            if !self.rt_compiled
                && let Some(rt) = self.rt.take()
            {
                rt.destroy(gpu);
                log::info!("ray-traced scene off");
            }
            return;
        }
        let (None, Some(expand)) = (&self.rt, self.rt_expand) else { return };
        let mut rt = RtScene::new(gpu, expand);
        let keys: Vec<MeshKey> = self
            .table_only
            .keys()
            .copied()
            .chain(self.meshes.keys().copied())
            .collect();
        for k in keys {
            rt.mesh_changed(k);
        }
        for &k in self.lod.keys() {
            rt.lod_changed(k);
        }
        log::info!("ray-traced scene on");
        self.rt = Some(rt);
    }

    /// Whether the world's shader traces its sun shadows this frame (`Scene::rt`): composed with
    /// the scene, and a shader pack whose `world.wgsl` calls `rt_sun`. Then the near shadow map
    /// isn't drawn.
    pub fn rt_traced(&self) -> bool {
        self.rt_compiled && self.rt.is_some()
    }

    /// Of the last frame: BLASes built, records expanded, TLAS instances (diagnostics).
    pub fn rt_stats(&self) -> Option<(u32, u32, u32)> {
        self.rt.as_ref().map(|r| r.last)
    }

    fn mesh_of(&self, key: MeshKey) -> Option<&GpuMesh> {
        if let Some(m) = self.table_only.get(&key) {
            return Some(m);
        }
        let i = *self.meshes.get(&key)?;
        self.regions
            .get(&region_of(key))
            .and_then(|l| l.get(i))
            .map(|e| &e.1)
    }

    /// What a queued item is now: its plan, or None when it has nothing to cast (gone, empty).
    fn rt_plan(&self, item: Item) -> Option<Plan> {
        match item {
            Item::Faces(pos) => {
                let m = self.mesh_of((0, pos))?;
                let a = m.faces?;
                let n = m.face_counts.map(|l| l.iter().sum::<u32>());
                Some(Plan {
                    item,
                    page: self.faces.pages[a.page as usize].0,
                    src: a.offset,
                    kind: KIND_FACES,
                    opaque: n[0],
                    clear: n[1] + n[2],
                    translucent: n[1],
                })
            }
            Item::Quads(pos) => {
                let m = self.mesh_of((0, pos))?;
                let a = m.quads?;
                let n = m.quad_counts;
                Some(Plan {
                    item,
                    page: self.quads.pages[a.page as usize].0,
                    src: a.offset,
                    kind: KIND_QUADS,
                    opaque: n[0],
                    clear: n[1] + n[2],
                    translucent: n[1],
                })
            }
            Item::Lod(key) => {
                let &(a, count, _, skip) = self.lod.get(&key)?;
                Some(Plan {
                    item,
                    page: self.faces.pages[a.page as usize].0,
                    src: a.offset + skip,
                    kind: KIND_LOD,
                    // All of it see-through to a ray: where cubes stand in its columns the tile
                    // isn't there (`Lod::mask`), which the ray's shader decides.
                    opaque: 0,
                    clear: count - skip,
                    translucent: count - skip,
                })
            }
        }
    }

    /// The frame's part of the ray-traced scene, before anything traces: the BLASes of meshes
    /// queued (within the frame's budget), then the TLAS about the camera.
    pub fn rt_frame(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, slot: usize, cam: &Camera) {
        let Some(mut rt) = self.rt.take() else { return };
        rt.frame_no += 1;
        rt.slot = slot;
        let frames = gpu.frames_in_flight() as u64;
        let (done, keep): (Vec<_>, Vec<_>) = rt
            .later
            .drain(..)
            .partition(|l| l.0 + frames < rt.frame_no);
        rt.later = keep;
        for (_, start, len) in done {
            rt.free_copy(start, len);
        }

        // Which to build this frame.
        let mut plans: Vec<Plan> = Vec::new();
        let mut records = 0u32;
        while plans.len() < FRAME_BUILDS {
            let Some(item) = rt.urgent.pop_front().or_else(|| rt.queue.pop_front()) else { break };
            match self.rt_plan(item) {
                Some(p) if p.opaque + p.clear > 0 => {
                    let n = p.opaque + p.clear;
                    if !plans.is_empty() && records + n > FRAME_RECORDS {
                        rt.urgent.push_front(item);
                        break;
                    }
                    rt.queued.remove(&item);
                    records += n;
                    plans.push(p);
                }
                _ => {
                    rt.queued.remove(&item);
                    rt.drop_item(gpu, item);
                }
            }
        }

        if !plans.is_empty() {
            let max = plans
                .iter()
                .map(|p| p.opaque.max(p.clear))
                .max()
                .unwrap_or(0);
            rt.ensure_index(gpu, max);
            let scratch = rt.ensure_scratch(gpu, slot, records as u64 * 48);
            let mut at = 0u32;
            let mut builds = Vec::with_capacity(plans.len());
            for p in &plans {
                let words = if p.kind == KIND_QUADS { 8 } else { 4 };
                // The see-through records' copy, but distant terrain's (its hits need only where
                // they are). Out of room, they cast as if solid.
                let mut copy = None;
                if p.clear > 0 && p.kind != KIND_LOD {
                    let units = p.clear * words / 4;
                    copy = rt.alloc_copy(units).map(|s| (s, units));
                    if copy.is_none() && !rt.full_warned {
                        rt.full_warned = true;
                        log::warn!("see-through records full ({} MB): more leaves and glass cast as solid", RECORDS_BYTES >> 20);
                    }
                }
                let job = Job {
                    src: p.src,
                    count: p.opaque + p.clear,
                    dst: at * 4,
                    kind: p.kind,
                    copy: copy.map_or(NONE, |c| c.0 * 4),
                    copy_from: p.opaque,
                    pad: [0; 2],
                };
                list.pipeline(rt.expand);
                list.bind(0, p.page);
                list.bind(1, scratch);
                list.bind(2, rt.records);
                list.push_constants(&job);
                let groups = job.count.div_ceil(64);
                list.push(Cmd::Dispatch { x: groups.min(65535), y: groups.div_ceil(65535), z: 1 });
                let geometry = |first: u32, n: u32, opaque: bool| BlasGeometry {
                    vertices: scratch,
                    vertex_offset: (at + first) as u64 * 48,
                    vertex_stride: 12,
                    vertex_count: n * 4,
                    indices: rt.index.0,
                    index_offset: 0,
                    triangles: n * 2,
                    opaque,
                };
                let mut geometries = Vec::with_capacity(2);
                if p.opaque > 0 {
                    geometries.push(geometry(0, p.opaque, true));
                }
                if p.clear > 0 {
                    // Distant terrain is see-through to rays only for its hidden columns; without a
                    // copy, cutout and translucent records cast as solid.
                    geometries.push(geometry(p.opaque, p.clear, copy.is_none() && p.kind != KIND_LOD));
                }
                let accel = gpu.create_blas(&geometries, "BLAS");
                builds.push(accel);
                at += p.opaque + p.clear;
                let info = [copy.map_or(NONE, |c| c.0 * 4), p.translucent, p.kind, 0];
                // The old one goes once this one is in (the frames in flight still hold it).
                rt.drop_item(gpu, p.item);
                rt.built.insert(p.item, Built { accel, copy, info });
            }
            list.push(Cmd::Barrier);
            for a in builds {
                list.push(Cmd::BuildBlas(a));
            }
        }
        rt.last = (plans.len() as u32, records, 0);

        // The TLAS, about the camera: whole cubes and the distant tiles drawn now.
        let cam_block = cam.pos.floor();
        let cam_cube: IVec3 = cam_block.as_ivec3() >> 4i32;
        let cam_frac = (cam.pos - (cam_cube * 16).as_dvec3()).as_vec3();
        let lod_draw: FxHashSet<super::super::lod::TileKey> = self.lod_draw.iter().copied().collect();
        rt.instances.clear();
        rt.rows.clear();
        for (item, b) in &rt.built {
            let (origin, mask) = match *item {
                Item::Faces(p) => (((p - cam_cube) * 16).as_vec3() - cam_frac, MASK_FACES),
                Item::Quads(p) => (((p - cam_cube) * 16).as_vec3() - cam_frac, MASK_QUADS),
                Item::Lod(k) => {
                    if !lod_draw.contains(&k) {
                        continue;
                    }
                    let o = k.origin();
                    (IVec3::new(o.x, 0, o.y).as_vec3() - (cam_cube * 16).as_vec3() - cam_frac, MASK_LOD)
                }
            };
            let custom = rt.rows.len() as u32;
            rt.rows.push(b.info);
            rt.instances.push(TlasInstance {
                transform: [[1.0, 0.0, 0.0, origin.x], [0.0, 1.0, 0.0, origin.y], [0.0, 0.0, 1.0, origin.z]],
                custom,
                mask,
                blas: b.accel,
            });
        }
        let n = rt.rows.len().max(1) as u32;
        let table = crate::gpu_layout::frame_buffer(gpu, &mut rt.table, slot, n, 16, usage::STORAGE, "rt instances");
        gpu.mapped(table)[..rt.rows.len() * 16].copy_from_slice(bytemuck::cast_slice(&rt.rows));
        gpu.set_tlas_instances(rt.tlas, &rt.instances);
        list.push(Cmd::BuildTlas(rt.tlas));
        rt.last.2 = rt.instances.len() as u32;
        rt.since.1 += rt.last.0;
        rt.since.2 += rt.last.1 as u64;
        let now = std::time::Instant::now();
        let since = *rt.since.0.get_or_insert(now);
        if now - since >= std::time::Duration::from_secs(5) {
            if rt.since.1 > 0 {
                log::info!(
                    "ray-traced scene: {} BLAS ({} built, {} records in {:.1} s), {} queued, {} TLAS instances, {:.0} MB",
                    rt.built.len(),
                    rt.since.1,
                    rt.since.2,
                    (now - since).as_secs_f32(),
                    rt.urgent.len() + rt.queue.len(),
                    rt.last.2,
                    gpu.accel_memory() as f64 / 1048576.0
                );
            }
            rt.since = (Some(now), 0, 0);
        }
        self.rt = Some(rt);
    }
}

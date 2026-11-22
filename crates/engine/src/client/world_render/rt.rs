//! Ray tracing's scene (`core:rt_shadows`, where the GPU has ray queries): a bottom-level
//! acceleration structure per meshed cube (its faces and its free quads apart, so a ray can skip
//! the plants) and per distant terrain tile, built from the records already on the GPU by a small
//! compute pass (`rt_expand.wgsl`) and kept until the mesh changes; and a top-level one over them
//! all, made again each frame about the camera.
//!
//! Each BLAS has up to two geometries: the opaque records, and the see-through ones (cutout, then
//! translucent). All its records are also copied into `records`, so a ray's hit can look up its
//! texel (a leaf's holes, a pane's colour), light and facing (a reflection's hit). A TLAS
//! instance's custom index picks its row of the frame's instance table: x the copy's first word
//! (`u32::MAX`: none), y the record where the translucent ones start, z the kind (0 faces, 1 quads,
//! 2 distant terrain), w the records before the see-through geometry's (its primitive `p` is record
//! `w + p / 2`).

use super::*;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// Records expanded per frame at most, and BLASes built: a world loading spreads its builds over
/// frames (a frame of a full load at 32 chunks would otherwise build thousands at once).
const FRAME_RECORDS: u32 = 1 << 19;
const FRAME_BUILDS: usize = 384;
/// Bytes for the records' copies (16-byte units): about 32 million faces' worth (spot C at view 32
/// held about 7 million records).
const RECORDS_BYTES: u64 = 512 << 20;
const UNIT: u64 = 16;
const NONE: u32 = u32::MAX;
/// TLAS instance masks: a cube's faces, its free quads (plants, models), distant terrain.
pub const MASK_FACES: u8 = 1;
pub const MASK_QUADS: u8 = 2;
pub const MASK_LOD: u8 = 4;
/// Blocks from the eye past which no traced ray reaches (packs trace within a few hundred blocks,
/// `rt_sun`'s rays end at 768): distant tiles further off stay out of the TLAS.
const RAY_REACH: f32 = 1100.0;
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
    /// The TLAS is made again only when what it holds changed (`version`): a BLAS built or
    /// dropped, the camera into another cube (its space is the camera's cube's, the shader adding
    /// the camera's place in it), the distant tiles drawn or the cubes' mask, a BLAS moved by
    /// compaction. Each frame slot's TLAS and table are made again once per version.
    dirty: bool,
    seen: Option<(IVec3, Vec<super::super::lod::TileKey>, u32, u64)>,
    version: u64,
    slot_version: Vec<u64>,
}

/// `core:rt_shadows`, or off where the engine's configs are not loaded.
pub fn rt_setting() -> bool {
    let on = |name: &str| crate::config::by_name(name).is_some_and(|id| crate::config::get(id) > 0.5);
    on("core:rt_shadows") || on("core:rt_reflections") || on("core:path_traced")
}

/// The path tracer's pipelines (`pathtrace.wgsl` in the shader pack, with the world's traced
/// prelude): its compute, and the composite drawing its pixels over the world's image.
pub(super) fn pt_pipelines(gpu: &mut dyn Backend, assets: &crate::assets::Assets) -> Option<[PipelineId; 4]> {
    let shader = match crate::packs::world_side_shader(assets, "pathtrace.wgsl", crate::gpu_layout::PT_DECLS)? {
        Ok(s) => s,
        Err(e) => {
            log::error!("path tracer: {}", e.0);
            return None;
        }
    };
    let cs = gpu
        .create_compute(&ComputeDesc { label: "path trace", shader: &shader, entry: "cs_trace" })
        .map_err(|e| log::error!("path tracer: {}", e.0))
        .ok()?;
    // The world's image, then the light for the denoisers: diffuse (the motion slot), specular.
    let fs = gpu
        .create_pipeline_mrt(
            &PipelineDesc {
                label: "path trace composite",
                shader: &shader,
                vertex: "vs_pt",
                fragment: Some("fs_pt"),
                color: Some(crate::post::world_format()),
                depth: None,
                blend: Blend::None,
                cull_back: false,
                topology: Topology::Triangles,
                motion: true,
            },
            &[Format::Rgba16Float],
        )
        .map_err(|e| log::error!("path tracer composite: {}", e.0))
        .ok();
    let resolve = gpu
        .create_compute(&ComputeDesc { label: "radiance cache", shader: &shader, entry: "cs_cache" })
        .map_err(|e| log::error!("path tracer cache: {}", e.0))
        .ok();
    let finish = gpu
        .create_pipeline(&PipelineDesc {
            label: "path trace finish",
            shader: &shader,
            vertex: "vs_pt",
            fragment: Some("fs_pt_finish"),
            color: Some(crate::post::world_format()),
            depth: None,
            blend: Blend::None,
            cull_back: false,
            topology: Topology::Triangles,
            motion: false,
        })
        .map_err(|e| log::error!("path tracer finish: {}", e.0))
        .ok();
    match (fs, resolve, finish) {
        (Some(fs), Some(resolve), Some(finish)) => Some([cs, fs, resolve, finish]),
        (fs, resolve, finish) => {
            for p in [Some(cs), fs, resolve, finish].into_iter().flatten() {
                gpu.destroy_pipeline(p);
            }
            None
        }
    }
}

/// The path tracer's G-buffer pipelines from the world shader (its `fs_gb_opaque`, `fs_gb_cutout`):
/// faces opaque, faces cutout, quads opaque, quads cutout, each drawing where the world's depth is
/// its own (Equal, no write) into the albedo (Rgba8 sRGB), normal (Rgba16F, the motion slot) and
/// spec (Rgba8 sRGB) images. None without those entries.
pub(super) fn gb_pipelines(gpu: &mut dyn Backend, shader: &Shader) -> Option<[PipelineId; 4]> {
    if !crate::packs::pack_uses("world.wgsl", "fs_gb_opaque") {
        return None;
    }
    let mut made = Vec::with_capacity(4);
    for (label, vs, fs) in [
        ("faces G-buffer", "vs_face", "fs_gb_opaque"),
        ("faces cutout G-buffer", "vs_face", "fs_gb_cutout"),
        ("quads G-buffer", "vs_quad", "fs_gb_opaque"),
        ("quads cutout G-buffer", "vs_quad", "fs_gb_cutout"),
    ] {
        let desc = PipelineDesc {
            label,
            shader,
            vertex: vs,
            fragment: Some(fs),
            color: Some(Format::Rgba8Srgb),
            depth: Some(DepthState { write: false, compare: Compare::Equal }),
            blend: Blend::None,
            cull_back: true,
            topology: Topology::Triangles,
            motion: true,
        };
        match gpu.create_pipeline_mrt(&desc, &[Format::Rgba8Srgb]) {
            Ok(p) => made.push(p),
            Err(e) => {
                log::error!("path tracer G-buffer: {}", e.0);
                for p in made {
                    gpu.destroy_pipeline(p);
                }
                return None;
            }
        }
    }
    Some([made[0], made[1], made[2], made[3]])
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
            dirty: true,
            seen: None,
            version: 0,
            slot_version: Vec::new(),
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
            self.dirty = true;
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

    /// Path traced lighting (`core:path_traced`): after the world's pass and its G-buffer
    /// (`gbuffer_pass`), before the effects, the path tracer lights each terrain pixel again
    /// (`pathtrace.wgsl`) into a buffer, and its composite draws them over the world's image `target`
    /// (the world's size `size`, its depth `depth`) and writes their light into `pt_images` for the
    /// denoisers. With `denoised` the composite leaves the light out of `target` (what lies under it,
    /// the air's share, is written): a denoiser adds it. Nothing without the traced scene, the pack's
    /// path tracer and the G-buffer. `debug`: 0 the picture, 1 albedo, 2 normals, 3 direct light,
    /// 4 the bounce, 5 the specular light, 6 the radiance cache, 7 the picture with a line where
    /// tracing ends.
    #[allow(clippy::too_many_arguments)]
    pub fn path_trace(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        slot: usize,
        depth: TextureId,
        target: Target,
        size: (u32, u32),
        debug: u32,
        denoised: bool,
    ) {
        let (Some([cs, fs, resolve, _]), true, Some(page), Some((albedo, normal, spec, gb_size))) = (
            self.pt_pipes,
            self.rt_traced() || self.rt_reflects() || self.rt.is_some(),
            self.faces.pages.first().map(|p| p.0),
            self.gb_images,
        ) else {
            return;
        };
        if !self.rt_compiled || self.buffers.is_none() || gb_size != size {
            return;
        }
        let n = size.0 * size.1;
        // On the GPU only, one a frame slot, grown to the world's size: three texels a pixel.
        const STRIDE: u64 = 48;
        while self.pt_out.len() < gpu.frames_in_flight() {
            self.pt_out
                .push((gpu.create_buffer(STRIDE, usage::STORAGE, "path traced"), 1));
        }
        if self.pt_out[slot].1 < n {
            gpu.destroy_buffer(self.pt_out[slot].0);
            self.pt_out[slot] = (gpu.create_buffer(n as u64 * STRIDE, usage::STORAGE, "path traced"), n);
        }
        if self.pt_images.is_none_or(|p| p.2 != size) {
            if let Some((a, b, _)) = self.pt_images.take() {
                gpu.destroy_texture(a);
                gpu.destroy_texture(b);
            }
            let make = |gpu: &mut dyn Backend, label| {
                gpu.create_texture(
                    &TextureDesc { width: size.0, height: size.1, mips: 1, format: Format::Rgba16Float, render_target: true },
                    label,
                )
            };
            self.pt_images = Some((make(gpu, "path traced diffuse"), make(gpu, "path traced specular"), size));
        }
        let (diffuse, specular, _) = self.pt_images.unwrap();
        use crate::gpu_layout::{PT_CACHE, PT_CACHE_BYTES, pt_bind};
        let cache = match self.pt_cache {
            Some(c) => c,
            None => {
                let c = gpu.create_buffer(PT_CACHE_BYTES, usage::STORAGE, "radiance cache");
                list.push(Cmd::FillBuffer { buffer: c, offset: 0, size: PT_CACHE_BYTES, value: 0 });
                list.push(Cmd::Barrier);
                self.pt_cache = Some(c);
                c
            }
        };
        let out = self.pt_out[slot].0;
        let args = crate::gpu_layout::PtArgs {
            size: [size.0, size.1],
            depth: gpu.texture_index(depth),
            debug,
            albedo: gpu.texture_index(albedo),
            normal: gpu.texture_index(normal),
            spec: gpu.texture_index(spec),
            mode: u32::from(denoised && debug == 0),
            light: 0,
            pad: [0; 3],
        };
        let origins = page;
        self.bind_run(list, cs, page, origins);
        list.bind(pt_bind::OUT, out);
        list.bind(pt_bind::CACHE, cache);
        list.push_constants(&args);
        list.push(Cmd::Dispatch { x: size.0.div_ceil(8), y: size.1.div_ceil(8), z: 1 });
        list.push(Cmd::Barrier);
        // The cache takes in what the paths left in it (read by the next frame's).
        self.bind_run(list, resolve, page, origins);
        list.bind(pt_bind::CACHE, cache);
        list.push_constants(&args);
        list.push(Cmd::Dispatch { x: PT_CACHE / 64, y: 1, z: 1 });
        list.push(Cmd::Barrier);
        let none = [0.0; 4];
        list.push(Cmd::PassExtras([Some((specular, Some(none))), None, None]));
        list.push(Cmd::BeginPass { color: Some((target, None)), depth: None, motion: Some((diffuse, Some(none))) });
        self.bind_run(list, fs, page, origins);
        list.bind(pt_bind::IN, out);
        list.push_constants(&args);
        list.draw(3, 1);
        list.push(Cmd::EndPass);
    }

    /// The image a denoiser writes its denoised light into (diffuse x albedo + specular x
    /// specular albedo, the air's transmittance already in; 0 where untraced) when `path_trace`
    /// ran `denoised`: Rgba16F at the world's `size`, made here. `pt_finish` then puts it in.
    pub fn pt_light(&mut self, gpu: &mut dyn Backend, size: (u32, u32)) -> TextureId {
        if self.pt_light_image.is_none_or(|l| l.1 != size) {
            if let Some((t, _)) = self.pt_light_image.take() {
                gpu.destroy_texture(t);
            }
            let t = gpu.create_texture(
                &TextureDesc { width: size.0, height: size.1, mips: 1, format: Format::Rgba16Float, render_target: true },
                "path traced light",
            );
            self.pt_light_image = Some((t, size));
        }
        self.pt_light_image.unwrap().0
    }

    /// After a denoiser wrote `pt_light` (a `denoised` `path_trace` this frame, slot `slot`): each
    /// traced pixel of the world's image `target` becomes the air's light plus the denoised light,
    /// finished as the world's pass finishes its pixels (the pack's `display`: the night's blue).
    pub fn pt_finish(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, slot: usize, target: Target, size: (u32, u32)) {
        let (Some([_, _, _, finish]), Some(page), Some((light, lsize))) =
            (self.pt_pipes, self.faces.pages.first().map(|p| p.0), self.pt_light_image)
        else {
            return;
        };
        if lsize != size || self.pt_out.len() <= slot || self.buffers.is_none() {
            return;
        }
        let args = crate::gpu_layout::PtArgs {
            size: [size.0, size.1],
            depth: 0,
            debug: 0,
            albedo: 0,
            normal: 0,
            spec: 0,
            mode: 1,
            light: gpu.texture_index(light),
            pad: [0; 3],
        };
        list.push(Cmd::BeginPass { color: Some((target, None)), depth: None, motion: None });
        self.bind_run(list, finish, page, page);
        list.bind(crate::gpu_layout::pt_bind::IN, self.pt_out[slot].0);
        list.push_constants(&args);
        list.draw(3, 1);
        list.push(Cmd::EndPass);
    }

    /// The path tracer's G-buffer (`core:path_traced`): after the world's pass, its opaque and cutout
    /// terrain drawn again where its depth (`depth`, the world's size `size`) is theirs, into the
    /// albedo, normal and spec images (`gb_images`, made at that size). Nothing without the traced
    /// world shader's G-buffer pipelines.
    pub fn gbuffer_pass(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, depth: TextureId, size: (u32, u32)) {
        let Some(pipes) = self.gb_pipes else { return };
        if self.buffers.is_none() || self.gb_draws.is_empty() && self.translucent == 0 {
            return;
        }
        if self.gb_images.is_none_or(|g| g.3 != size) {
            if let Some((a, b, c, _)) = self.gb_images.take() {
                for t in [a, b, c] {
                    gpu.destroy_texture(t);
                }
            }
            let make = |gpu: &mut dyn Backend, format, label| {
                gpu.create_texture(&TextureDesc { width: size.0, height: size.1, mips: 1, format, render_target: true }, label)
            };
            self.gb_images = Some((
                make(gpu, Format::Rgba8Srgb, "G-buffer albedo"),
                make(gpu, Format::Rgba16Float, "G-buffer normal"),
                make(gpu, Format::Rgba8Srgb, "G-buffer spec"),
                size,
            ));
        }
        let (albedo, normal, spec, _) = self.gb_images.unwrap();
        let none = [0.0; 4];
        list.push(Cmd::PassExtras([Some((spec, Some(none))), None, None]));
        list.push(Cmd::BeginPass {
            color: Some((Target::Texture(albedo), Some(none))),
            depth: Some((depth, None)),
            motion: Some((normal, Some(none))),
        });
        let pipe = |kind: u8, layer: u8| {
            let quads = matches!(kind, 1 | 5);
            pipes[usize::from(quads) * 2 + usize::from(layer == 1)]
        };
        let Some((_, obuf, ibuf, ..)) = self.buffers else { return };
        // The table's draws, as `draw` made them this frame.
        if let Some(o) = self.table.out.as_ref() {
            let (draws, origins, slot) = (o.draws, o.origins, self.gb_draws_slot);
            for &(kind, layer, page, id, base, n) in &self.gb_draws {
                let buf = if matches!(kind, 1 | 5) { self.quads.pages[page as usize].0 } else { self.faces.pages[page as usize].0 };
                self.bind_run(list, pipe(kind, layer), buf, origins);
                list.push(Cmd::DrawIndexedIndirectCount {
                    buffer: draws,
                    offset: base as u64 * 20,
                    count_buffer: self.table.buckets[slot].0,
                    count_offset: id as u64 * 8,
                    max: n,
                });
            }
        }
        // Every opaque and cutout run the CPU made (structures, the nearest cubes): the plain ones
        // `draw` puts first and the rest before the translucent ones. With only the first, the
        // storeroom's gold and birch blocks had no G-buffer and came out black (pt1_rtxd_pt).
        for &(kind, layer, page, start, count) in &self.runs[..self.translucent.min(self.runs.len())] {
            if layer >= 2 || !matches!(kind, 0 | 1 | 4 | 5) {
                continue;
            }
            let buf = if matches!(kind, 1 | 5) { self.quads.pages[page as usize].0 } else { self.faces.pages[page as usize].0 };
            self.bind_run(list, pipe(kind, layer), buf, obuf);
            list.push(Cmd::DrawIndexedIndirect { buffer: ibuf, offset: start as u64 * 20, count });
        }
        list.push(Cmd::EndPass);
    }

    /// Frames the ray-traced scene has been kept (`Scene::rt.y`, for noise that moves every frame).
    pub fn rt_frame_no(&self) -> u64 {
        self.rt.as_ref().map_or(0, |r| r.frame_no)
    }

    /// Whether the world's shader traces its sun shadows this frame (`Scene::rt`): composed with
    /// the scene, and a shader pack whose `world.wgsl` calls `rt_sun`. Then the near shadow map
    /// isn't drawn.
    pub fn rt_traced(&self) -> bool {
        self.rt_compiled && self.rt.is_some() && self.rt_modes.0
    }

    /// Whether the world's shader traces its reflections this frame (`Scene::rt.w`).
    pub fn rt_reflects(&self) -> bool {
        self.rt_compiled && self.rt.is_some() && self.rt_modes.1
    }

    /// What the scene is for (`core:rt_shadows`, `core:rt_reflections`): shadows, reflections.
    pub fn set_rt_modes(&mut self, shadows: bool, reflections: bool) {
        self.rt_modes = (shadows, reflections);
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
        let (done, keep): (Vec<_>, Vec<_>) = rt.later.drain(..).partition(|l| l.0 + frames < rt.frame_no);
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
                // The records' copy, for the hits' lookups. Out of room, see-through ones cast as
                // if solid and reflections see nothing there.
                let units = (p.opaque + p.clear) * words / 4;
                let copy = rt.alloc_copy(units).map(|s| (s, units));
                if copy.is_none() && !rt.full_warned {
                    rt.full_warned = true;
                    log::warn!("ray-traced records full ({} MB): more leaves and glass cast as solid", RECORDS_BYTES >> 20);
                }
                let job = Job {
                    src: p.src,
                    count: p.opaque + p.clear,
                    dst: at * 4,
                    kind: p.kind,
                    copy: copy.map_or(NONE, |c| c.0 * 4),
                    copy_from: 0,
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
                let info = [copy.map_or(NONE, |c| c.0 * 4), p.opaque + p.translucent, p.kind, p.opaque];
                // The old one goes once this one is in (the frames in flight still hold it).
                rt.drop_item(gpu, p.item);
                rt.built.insert(p.item, Built { accel, copy, info });
                rt.dirty = true;
            }
            list.push(Cmd::Barrier);
            for a in builds {
                list.push(Cmd::BuildBlas(a));
            }
        }
        rt.last = (plans.len() as u32, records, 0);

        // The TLAS, in the space of the camera's cube (the shader adds the camera's place in it):
        // whole cubes and the distant tiles drawn now, made again only when something changed.
        let cam_cube: IVec3 = cam.pos.floor().as_ivec3() >> 4i32;
        let seen = (cam_cube, self.lod_draw.clone(), self.lod_half, gpu.accel_generation());
        if rt.dirty || rt.seen.as_ref() != Some(&seen) {
            rt.dirty = false;
            rt.version += 1;
            let lod_draw: FxHashSet<super::super::lod::TileKey> = self.lod_draw.iter().copied().collect();
            let half = self.lod_half as i32;
            rt.instances.clear();
            rt.rows.clear();
            for (item, b) in &rt.built {
                let (origin, mask, opaque) = match *item {
                    Item::Faces(p) => ((p - cam_cube) * 16, MASK_FACES, false),
                    Item::Quads(p) => ((p - cam_cube) * 16, MASK_QUADS, false),
                    Item::Lod(k) => {
                        if !lod_draw.contains(&k) {
                            continue;
                        }
                        let o = k.origin();
                        // Wholly outside the cubes' mask (`Lod::mask`), no column of it is left to
                        // cubes: every hit counts, without asking the shader.
                        let (lo, hi) = (o.div_euclid(glam::IVec2::splat(16)), (o + k.size() - 1).div_euclid(glam::IVec2::splat(16)));
                        let cxz = glam::IVec2::new(cam_cube.x, cam_cube.z);
                        let (c0, c1) = (cxz - half, cxz + half);
                        let outside = hi.x < c0.x || lo.x > c1.x || hi.y < c0.y || lo.y > c1.y;
                        // Wholly inside the mask, where cubes stand in every column, it never casts;
                        // beyond any ray's reach (a pack traces from within a few hundred blocks, rays
                        // of 768) it can't either. Both only cost the TLAS and the rays' walk.
                        let inside = lo.x >= c0.x && hi.x <= c1.x && lo.y >= c0.y && hi.y <= c1.y;
                        let near = (glam::IVec2::new(o.x, o.y) + k.size() / 2 - cxz * 16)
                            .as_vec2()
                            .length()
                            - k.size() as f32 * 0.71;
                        if inside || near > RAY_REACH {
                            continue;
                        }
                        (IVec3::new(o.x, 0, o.y) - cam_cube * 16, MASK_LOD, outside)
                    }
                };
                let origin = origin.as_vec3();
                let custom = rt.rows.len() as u32;
                rt.rows.push(b.info);
                rt.instances.push(TlasInstance {
                    transform: [[1.0, 0.0, 0.0, origin.x], [0.0, 1.0, 0.0, origin.y], [0.0, 0.0, 1.0, origin.z]],
                    custom,
                    mask,
                    opaque,
                    blas: b.accel,
                });
            }
            rt.seen = Some(seen);
        }
        rt.slot_version.resize(gpu.frames_in_flight(), 0);
        if rt.slot_version[slot] != rt.version {
            rt.slot_version[slot] = rt.version;
            let n = rt.rows.len().max(1) as u32;
            let table = crate::gpu_layout::frame_buffer(gpu, &mut rt.table, slot, n, 16, usage::STORAGE, "rt instances");
            gpu.mapped(table)[..rt.rows.len() * 16].copy_from_slice(bytemuck::cast_slice(&rt.rows));
            gpu.set_tlas_instances(rt.tlas, &rt.instances);
            list.push(Cmd::BuildTlas(rt.tlas));
        }
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

//! Draws meshed cubes. All face records live in a few big storage-buffer pages; each frame the CPU
//! culls cubes and writes one indirect draw per (cube, layer) plus the cube origin it uses.
//! Cubes of moving structures are meshed in the structure's own coordinates and drawn moved
//! by its pose (an offset per structure, `origin.w` in the shader): a block placed on one
//! remeshes only its cube.

use glam::{DVec3, IVec2, IVec3, Mat4, Vec3, Vec4};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use weft_render::*;

use super::mesher::Mesh;
use super::textures::GameTextures;
use crate::content::Content;

mod rt;
pub use crate::gpu_layout::Globals;

const FACE_WORDS: u32 = 4;
const QUAD_WORDS: u32 = 8;
const PAGE_BYTES: u64 = 64 << 20;
/// How far from the camera, in blocks, translucent quads are kept sorted far to near. Past that
/// overlaps are too small to tell.
const SORT_REACH: f64 = 48.0;
/// Translucent quads re-sorted in one go up to which the frame thread sorts them itself
/// (`sort_translucent`), not the pool.
const PARALLEL_SORT: usize = 16_384;
/// A cube holds at most 6 * 4096 faces; plants and boxes add a few quads per block.
const MAX_QUADS_PER_DRAW: u32 = 6 * 4096 * 2;

#[derive(Clone, Copy)]
struct Alloc {
    page: u32,
    offset: u32,
    len: u32,
}

struct Pool {
    words: u32,
    pages: Vec<(BufferId, FreeList)>,
    label: &'static str,
}

/// Free ranges of a page by start and by (length, start): best fit and merging with the
/// neighbors in log time. A big edit frees and allocates tens of thousands of ranges at once,
/// which a scanned list took seconds for.
#[derive(Default)]
struct FreeList {
    by_start: std::collections::BTreeMap<u32, u32>,
    by_len: std::collections::BTreeSet<(u32, u32)>,
}

impl FreeList {
    fn add(&mut self, start: u32, len: u32) {
        self.by_start.insert(start, len);
        self.by_len.insert((len, start));
    }

    fn take(&mut self, start: u32, len: u32) {
        self.by_start.remove(&start);
        self.by_len.remove(&(len, start));
    }
}

impl Pool {
    fn records_per_page(&self) -> u32 {
        (PAGE_BYTES / (self.words as u64 * 4)) as u32
    }

    fn alloc(&mut self, gpu: &mut dyn Backend, n: u32) -> Alloc {
        let per = self.records_per_page();
        assert!(n <= per, "mesh larger than a page");
        for (pi, (_, free)) in self.pages.iter_mut().enumerate() {
            if let Some(&(len, start)) = free.by_len.range((n, 0)..).next() {
                free.take(start, len);
                if len > n {
                    free.add(start + n, len - n);
                }
                return Alloc { page: pi as u32, offset: start, len: n };
            }
        }
        let buf = gpu.create_buffer(PAGE_BYTES, usage::STORAGE, self.label);
        let mut free = FreeList::default();
        if per > n {
            free.add(n, per - n);
        }
        self.pages.push((buf, free));
        Alloc { page: self.pages.len() as u32 - 1, offset: 0, len: n }
    }

    fn free(&mut self, a: Alloc) {
        let free = &mut self.pages[a.page as usize].1;
        let (mut start, mut len) = (a.offset, a.len);
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

#[derive(Clone, Copy)]
struct GpuMesh {
    /// Its places in `CubeTable`, none for `u32::MAX`.
    slots: [u32; 4],
    faces: Option<Alloc>,
    /// Per layer, per direction rank (see `mesher::DIR_RANK`).
    face_counts: [[u32; 6]; 3],
    quads: Option<Alloc>,
    quad_counts: [u32; 3],
}

#[derive(Clone, Copy)]
pub struct Camera {
    pub pos: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    /// Walking sway in view space: offset, roll and extra pitch (vanilla's view bobbing).
    pub bob: (Vec3, f32, f32),
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        crate::physics::look_dir(self.yaw, self.pitch).as_vec3()
    }

    /// The walking sway alone, in view space: the world moves by it, and so does the hand.
    pub fn bobbing(&self) -> Mat4 {
        let (offset, roll, pitch) = self.bob;
        Mat4::from_translation(offset) * Mat4::from_rotation_z(roll) * Mat4::from_rotation_x(pitch)
    }

    pub fn view(&self) -> Mat4 {
        self.bobbing() * glam::camera::rh::view::look_to_mat4(Vec3::ZERO, self.forward(), Vec3::Y)
    }

    /// The projection the world is drawn with: `proj_plain` moved by this frame's jitter
    /// (`post::jitter`, a fraction of a pixel; none without temporal anti-aliasing).
    pub fn proj(&self, aspect: f32) -> Mat4 {
        let mut m = self.proj_plain(aspect);
        let j = crate::post::jitter();
        if j != glam::Vec2::ZERO {
            // Clip x and y moved by j times w: the whole image moved by j in NDC.
            for c in [&mut m.x_axis, &mut m.y_axis, &mut m.z_axis, &mut m.w_axis] {
                c.x += j.x * c.w;
                c.y += j.y * c.w;
            }
        }
        m
    }

    /// The projection without the jitter.
    pub fn proj_plain(&self, aspect: f32) -> Mat4 {
        // Y-up clip space: naga already flips Y for Vulkan.
        glam::camera::rh::proj::directx::perspective_infinite_reverse(self.fov.to_radians(), aspect, 0.05)
    }
}

/// One indirect draw: kind (0 faces, 1 quads, 3 distant terrain), layer, page, arguments, cube origin (w: offset
/// index), cube center relative to the camera.
type DrawRecord = (u8, u8, u32, DrawIndexedArgs, [i32; 4], Vec3);

/// A mapped buffer written from every core, each record by one.
#[derive(Clone, Copy)]
struct Mapped(*mut u8);

unsafe impl Send for Mapped {}
unsafe impl Sync for Mapped {}

impl Mapped {
    /// SAFETY: `off` + what is written there stays inside the buffer.
    unsafe fn at(&self, off: usize) -> *mut u8 {
        unsafe { self.0.add(off) }
    }
}

/// Blocks over which the cubes fade into distant terrain, before `lod_fade`.
const FADE_BAND: f32 = 24.0;
/// The share of distant terrain's reach past which its tiles take `lod_end_pipe` (chiacraft): the
/// pack fades the land out from there (`fs_lod_end`).
pub const END_FADE: f32 = 0.85;

/// Meshes are kept in regions of this many cubes a side (log2), culled as a whole first.
const REGION: i32 = 3;
/// Meshes in view up to which the frame thread culls and writes the draws itself (`draw`): about
/// a millisecond; a view of 12 on the Mac held about 9,000 draws.
const SERIAL_CULL: usize = 24_576;

fn region_of(key: MeshKey) -> (u64, IVec3) {
    (key.0, key.1 >> REGION)
}

/// A structure's lean (chiacraft): its pivot (own coordinates) and rotation.
type Lean = (Vec3, glam::Quat);
/// Where each moving structure was drawn last frame (world position) and how it leaned, by id.
type StructuresBefore = FxHashMap<u64, (DVec3, Option<Lean>)>;

/// The draws of one visible mesh (its corner `lo` relative to the camera, `k` its structure or
/// 0), returning the faces drawn. A face direction can be seen only from the side its normal
/// points to; contiguous visible runs become one draw.
/// Which way faces a mesh at `lo` (camera relative) shows the camera: by rank +X +Y +Z -X -Y -Z.
fn camera_sides(lo: Vec3) -> [bool; 6] {
    [lo.x < 0.0, lo.y < 0.0, lo.z < 0.0, lo.x > -16.0, lo.y > -16.0, lo.z > -16.0]
}

/// A mesh's draws from layer `first_layer` on, with only the faces whose direction `seen` keeps (by
/// rank, see `camera_sides`). The view's draws of the world start at the translucent layer (2): its
/// opaque and cutout layers are `CubeTable`'s; a structure's, and the sun-shadow passes, all layers
/// (0), or terrain would cast no shadows (chiacraft).
fn mesh_draws(
    m: &GpuMesh,
    key: MeshKey,
    lo: Vec3,
    k: i32,
    cam_cube: IVec3,
    seen: [bool; 6],
    first_layer: usize,
    out: &mut Vec<DrawRecord>,
) -> u64 {
    let rel = if k == 0 { key.1 - cam_cube } else { key.1 };
    let origin = [rel.x, rel.y, rel.z, k];
    let center = lo + Vec3::splat(8.0);
    let mut faces = 0u64;
    let args = |s: u32, len: u32| DrawIndexedArgs {
        indices: len * 6,
        instances: 1,
        first_index: 0,
        vertex_offset: (s * 4) as i32,
        first_instance: 0,
    };
    if let Some(a) = m.faces {
        let mut off = a.offset;
        for layer in 0..3 {
            let counts = m.face_counts[layer];
            if layer < first_layer {
                off += counts.iter().sum::<u32>();
                continue;
            }
            let mut run: Option<(u32, u32)> = None;
            for rank in 0..6 {
                let n = counts[rank];
                if n > 0 && seen[rank] {
                    faces += n as u64;
                    run = Some(match run {
                        Some((s, len)) => (s, len + n),
                        None => (off, n),
                    });
                } else if n > 0 {
                    if let Some((s, len)) = run.take() {
                        out.push((0, layer as u8, a.page, args(s, len), origin, center));
                    }
                }
                off += n;
            }
            if let Some((s, len)) = run {
                out.push((0, layer as u8, a.page, args(s, len), origin, center));
            }
        }
    }
    if let Some(a) = m.quads {
        let mut off = a.offset;
        for layer in 0..3 {
            let n = m.quad_counts[layer];
            if n > 0 && layer >= first_layer {
                out.push((1, layer as u8, a.page, args(off, n), origin, center));
            }
            off += n;
        }
    }
    faces
}

/// A mesh's place: the structure (entity) it belongs to, 0 for the world, and its cube.
pub type MeshKey = (u64, IVec3);

/// A cube's translucent quads and where they are on the GPU.
struct SortedQuads {
    page: u32,
    offset: u32,
    quads: Vec<[u32; 8]>,
    /// The block the camera was in when they were put in order, in the cube's own coordinates.
    seen_from: Option<IVec3>,
}

/// Which blocks around the camera cast shadows, kept on the GPU for shader packs (`solid_at`
/// in the world shader's prelude); only while the chosen pack's world shader uses it. 16^3 cube
/// slots addressed by cube position modulo 16: first a tag per slot (cube x y z, 1 when
/// filled), then 128 words of bits per slot in `cube::index` order.
struct Solid {
    buf: BufferId,
    on: bool,
    /// Per slot: the cube it holds and the `cube_rev` of its blocks then.
    held: Vec<Option<(IVec3, u32)>>,
    /// The tags as uploaded: sent whole when one changes, since copies within a frame have no
    /// order between them and a slot's tag must never lose to an older write.
    tags: Vec<[u32; 4]>,
    /// Per block state: casts a shadow.
    casts: Vec<bool>,
    /// The center and the world's `revision` of the last pass that finished: while both stay,
    /// nothing is looked at (the 15³ cubes' lookups were a cost every frame).
    done: Option<(IVec3, u32)>,
}

const SOLID_SLOTS: usize = 4096;
/// Cubes rebuilt per frame at most; the rest follow in the next frames, nearest first. At 192 a
/// step into the next cube rebuilt up to 192 x 4096 blocks in one frame, a burst of 1-3 ms
/// (reports/performance.md); 32 spreads a step's new shell over a few frames.
const SOLID_PER_FRAME: usize = 32;

impl Solid {
    fn new(gpu: &mut dyn Backend, c: &Content) -> Solid {
        let casts = c
            .states
            .iter()
            .map(|st| st.opacity > 0 && st.liquid_in().is_none())
            .collect();
        let mut s =
            Solid { buf: gpu.create_buffer(4, usage::STORAGE, "solid"), on: false, held: Vec::new(), tags: Vec::new(), casts, done: None };
        s.set_on(gpu, crate::packs::pack_uses("world.wgsl", "solid_at"));
        s
    }

    fn set_on(&mut self, gpu: &mut dyn Backend, on: bool) {
        if on == self.on {
            return;
        }
        gpu.destroy_buffer(self.buf);
        self.on = on;
        self.held = vec![None; if on { SOLID_SLOTS } else { 0 }];
        self.done = None;
        self.tags = vec![[0; 4]; if on { SOLID_SLOTS } else { 0 }];
        let words = if on { SOLID_SLOTS * (4 + 128) } else { 1 };
        self.buf = gpu.create_buffer(words as u64 * 4, usage::STORAGE, "solid");
        gpu.write_buffer(self.buf, 0, &vec![0u8; words * 4]);
    }
}

/// The world's colour pipelines from one shader: faces and quads (opaque, cutout, translucent),
/// distant terrain, and when the pack has them its open tiles, split cutouts and fading cubes.
struct ColorPipes {
    face: [PipelineId; 3],
    quad: [PipelineId; 3],
    lod: PipelineId,
    lod_open: Option<PipelineId>,
    lod_end: Option<PipelineId>,
    cutout_split: Option<[PipelineId; 4]>,
    fade: Option<[PipelineId; 6]>,
}

pub struct WorldRenderer {
    /// Moving structures as drawn last frame, for their motion (the offsets' second part).
    structures_before: StructuresBefore,
    /// Depth only, from the sun, for `sun_shadows` packs: faces solid, faces cutout, quads solid,
    /// quads cutout. Solid blocks have no fragment shader (nothing to leave out, and a pass without
    /// `discard` keeps Apple GPUs' hidden-surface removal on); cutout ones the pack's `fs_shadow`.
    shadow_pipes: Option<[PipelineId; 4]>,
    /// Distant terrain into the far cascade, when the pack's `world.wgsl` has `fs_lod_shadow` (it
    /// leaves out the columns cubes show, as `fs_lod` does): hills and canopies past the render
    /// distance cast shadows too.
    lod_shadow_pipe: Option<PipelineId>,
    /// Counts every change to the terrain drawn (cubes or distant tiles uploaded or removed): a
    /// cached shadow map is drawn again when it moves (`Client::render_shadows`).
    pub terrain_epoch: u64,
    /// The shadow pass's own per-frame draws (`draw_shadow`), apart from the camera's.
    /// Per shadow cascade (near, far): each pass writes its own draws.
    shadow_origins: [Vec<(BufferId, u32)>; 2],
    shadow_indirect: [Vec<(BufferId, u32)>; 2],
    shadow_offsets: [Vec<(BufferId, u32)>; 2],
    face_pipes: [PipelineId; 3],
    quad_pipes: [PipelineId; 3],
    lod_pipe: PipelineId,
    /// Distant terrain's tiles that lie wholly outside the cubes' mask (`lod_half`), when the pack's
    /// `world.wgsl` has `fs_lod_open`: no `discard`, which can't happen there anyway, so a tiled GPU
    /// keeps its hidden-surface removal for them (Android port: on the Pixel 10's PowerVR the
    /// discard alone cost a third of the world pass at the vista).
    lod_open_pipe: Option<PipelineId>,
    /// Those of them that reach toward where distant terrain ends (past `END_FADE` of its reach),
    /// when the pack's `world.wgsl` has `fs_lod_end` (chiacraft, Far view): faded out there by an
    /// ordered dither, so what is drawn behind (the sky, the cloud layer seen edge on) shows
    /// through, as Distant Horizons fades its far edge (`dh_fade`). Only that last ring takes the
    /// `discard` (on a phone's tiled GPU it costs: Android keeps the plain open path there).
    lod_end_pipe: Option<PipelineId>,
    /// Cutout blocks (leaves) in two passes, when the pack has `fs_cutout_depth` and
    /// `fs_cutout_shade` and `cutout_prepass()`: their depth with the alpha test alone (no color),
    /// then their color where the depth is theirs, with no `discard`. Faces depth, faces shade,
    /// quads depth, quads shade. The same picture; a tiled GPU then keeps hidden-surface removal
    /// for the shading (Android port: on the Pixel 10 the leaves' discard cost the forest about a
    /// third of its world pass).
    cutout_split: Option<[PipelineId; 4]>,
    /// Faces opaque, cutout, translucent, then quads the same, for the cubes near where they end,
    /// when the pack's `world.wgsl` has `fs_opaque_fade`: dithered away toward `lod_fade`, distant
    /// terrain taking the pixels they leave (Distant Horizons' fade), so the edge isn't a line.
    /// Only those meshes, so the rest keep early depth (no `discard`).
    fade_pipes: Option<[PipelineId; 6]>,
    /// The calm colour pipelines (`calm_pipes`): calm, then calm and dry. None where the pack
    /// doesn't use `CALM` (or `DRY`). The set of level `calm_level` (`set_calm`) is in use, and its
    /// place here holds the plain ones meanwhile.
    calm_sets: [Option<ColorPipes>; 2],
    calm_level: u8,
    /// How far from the camera (blocks, across) the cubes end while distant terrain is shown; 0:
    /// no fade. Cubes past it aren't drawn.
    pub lod_fade: u32,
    /// Distant terrain tiles (`lod`): their quads (in `faces`, four words like a face), count,
    /// the heights they span and the records before its faces; the ones to draw this frame; and the columns whose cubes show
    /// their ground (`Lod::mask`, half its width), where their pixels are left to the cubes.
    lod: FxHashMap<super::lod::TileKey, (Alloc, u32, (i32, i32), u32)>,
    pub lod_draw: Vec<super::lod::TileKey>,
    pub lod_half: u32,
    lod_mask: (BufferId, usize),
    faces: Pool,
    quads: Pool,
    /// Index into its region's meshes by key. A frame culls the regions, then goes through the
    /// meshes of the ones seen, nearest first, in parallel.
    meshes: FxHashMap<MeshKey, usize>,
    regions: FxHashMap<(u64, IVec3), Vec<(MeshKey, GpuMesh)>>,
    /// Meshes the CPU has nothing to do for each frame, there being no translucent draws: they are
    /// in `table` and not in `regions`.
    table_only: FxHashMap<MeshKey, GpuMesh>,
    table: CubeTable,
    globals: Vec<BufferId>,
    /// The sun-shadow passes' own (`draw_shadow`, no fade): they are recorded after the world's,
    /// and writing the world's buffer took its fade end and the rest away for the whole frame.
    shadow_globals: Vec<BufferId>,
    origins: Vec<(BufferId, u32)>,
    /// Per frame: camera-relative positions of the structures drawn (index 0: none, the world).
    offsets: Vec<(BufferId, u32)>,
    indirect: Vec<(BufferId, u32)>,
    /// Not drawing what the last frame shows to be behind something.
    pub occlusion: Occlusion,
    /// Of the last `draw`: its runs (kind, layer, page, first record, records), and its
    /// translucent draws, sorted far to near.
    runs: Vec<(u8, u8, u32, u32, u32)>,
    trans: Vec<DrawRecord>,
    /// Translucent quads by cube, to keep them in order.
    sorted: FxHashMap<MeshKey, SortedQuads>,
    /// The block the camera was in at the last sort, unless there is more to sort since.
    sorted_from: Option<IVec3>,
    /// Draws of each region seen, kept between frames.
    parts: Vec<Vec<DrawRecord>>,
    /// Of the last `draw`: where its translucent draws start in `scratch.2`, and the buffers
    /// they use (globals, origins, indirect, offsets, the frame's scene).
    translucent: usize,
    buffers: Option<(BufferId, BufferId, BufferId, BufferId, BufferId)>,
    /// Biome table, then per block column of the loaded area its blended colors: tiles of
    /// 16x16 blocks addressed by column position modulo `tint_tiles`.
    biomes: BufferId,
    biome_table: Vec<u32>,
    tint_tiles: u32,
    tex_info: BufferId,
    anim: BufferId,
    /// Start of the overlay table in `anim`.
    overlays: u32,
    /// Block shader slot per texture id (`shader_table`).
    tex_shader: BufferId,
    /// `GameTextures::glow_buffer`, bound as it is.
    glow: BufferId,
    /// `GameTextures::mat_buffer`, the same.
    mat: BufferId,
    /// `GameTextures::generation` the tables were built from.
    tex_generation: u64,
    /// (0 1 2)(0 2 3) for as many quads as one draw can hold; every face and quad draw shares it
    /// so each corner is shaded once instead of once per triangle.
    quad_index: BufferId,
    solid: Solid,
    pub drawn: u32,
    pub faces_drawn: u64,
    /// Ray tracing's scene (`rt.rs`, `core:rt_shadows`), and its compute pipeline where the GPU
    /// has ray queries.
    rt: Option<rt::RtScene>,
    rt_expand: Option<PipelineId>,
    /// The world shader traces (`gpu_layout::set_rt_prelude`; the shader pack's `world.wgsl` calls
    /// `rt_sun`): every draw then binds the scene, which is kept while it does.
    rt_compiled: bool,
    rt_wanted: bool,
    /// The shader pack's `world.wgsl` calls `rt_sun` (as of the last composing).
    rt_pack: bool,
    /// What the scene is traced for: sun shadows, reflections (`set_rt_modes`).
    rt_modes: (bool, bool),
    /// The path tracer (`rt.rs` `path_trace`, the pack's `pathtrace.wgsl`): its compute and its
    /// composite, made with the traced world shader; its output per frame slot.
    /// The path tracer's pipelines: the trace, the composite, the radiance cache's resolve, the
    /// finish after a denoiser (`pt_finish`).
    pt_pipes: Option<[PipelineId; 4]>,
    pt_out: Vec<(BufferId, u32)>,
    /// The radiance cache (`pathtrace.wgsl`'s `pt_cache`, `gpu_layout::PT_CACHE` cells), made zeroed
    /// with the first traced frame.
    pt_cache: Option<BufferId>,
    /// The path tracer's G-buffer (`rt.rs` `gbuffer_pass`): its pipelines (faces opaque, faces
    /// cutout, quads opaque, quads cutout), the last `draw`'s opaque and cutout table draws (kind,
    /// layer, page, group, first, draws) with its frame slot (the CPU-made runs before
    /// `translucent` are drawn again too), and its images (albedo, normal, spec) at the world's size.
    gb_pipes: Option<[PipelineId; 4]>,
    gb_draws: Vec<(u8, u8, u32, u32, u32, u32)>,
    gb_draws_slot: usize,
    pub gb_images: Option<(TextureId, TextureId, TextureId, (u32, u32))>,
    /// The path tracer's light (`path_trace`), at the world's size, Rgba16F: diffuse (rgb divided
    /// by the albedo; a the bounce's distance) and specular (rgb divided by `pt_spec_albedo`; a the
    /// reflection's distance, 65504 the sky), for the denoisers.
    pub pt_images: Option<(TextureId, TextureId, (u32, u32))>,
    /// The denoised light a denoiser writes for `pt_finish` (Rgba16F, the world's size; `pt_light`).
    pt_light_image: Option<(TextureId, (u32, u32))>,
    /// The setting changed: the app composes the world shader again (`reload_shaders`).
    pub rt_reload: bool,
}

/// Block shaders: `fn shade(p: BlockPixel) -> vec4<f32>` in `shaders/block/<name>.wgsl`, for
/// blocks with a `shader` component.
pub const HOOK: crate::shaders::Hook = crate::shaders::Hook {
    kind: "block",
    entry: "shade",
    dispatch: "block_shade",
    params: "p: BlockPixel",
    args: "p",
    ret: "vec4<f32>",
    fallback: "return vec4(p.albedo.rgb * p.light, p.albedo.a);",
};

pub fn world_shader(assets: &crate::assets::Assets) -> Result<(Shader, crate::shaders::Slots), ShaderError> {
    let snippets = crate::shaders::snippets(assets, &HOOK);
    let (s, mut slots) = crate::packs::compose(assets, "world.wgsl", &[(&HOOK, &snippets)])?;
    Ok((s, slots.pop().unwrap_or_default()))
}

/// Block shader slot per texture id: a block's `shader` goes with its textures.
fn shader_table(gpu: &mut dyn Backend, c: &Content, textures: &GameTextures, slots: &crate::shaders::Slots) -> BufferId {
    let mut table = vec![0u32; textures.count().max(1)];
    if let Some(comp) = c.components.id(weft_abi::comp::SHADER) {
        for (i, s) in c.states.iter().enumerate() {
            let Some(name) = c.components.text(crate::components::BLOCK, i as u32, comp) else { continue };
            let slot = slots.slot(name);
            if slot == 0 {
                log::warn!("block {}: no block shader {name}", c.block_of(i as u32).name);
                continue;
            }
            for t in s
                .textures
                .iter()
                .chain(s.boxes.iter().flat_map(|b| b.textures.iter()))
            {
                if let Some(e) = table.get_mut(*t as usize) {
                    *e = slot;
                }
            }
        }
    }
    let buf = gpu.create_buffer((table.len() * 4) as u64, usage::STORAGE, "texture shaders");
    gpu.write_buffer(buf, 0, bytemuck::cast_slice(&table));
    buf
}

impl WorldRenderer {
    pub fn new(
        gpu: &mut dyn Backend,
        c: &Content,
        textures: &GameTextures,
        assets: &crate::assets::Assets,
    ) -> Result<WorldRenderer, String> {
        // Traced sun shadows (`core:rt_shadows` where the GPU has ray queries): the world shader
        // with the scene's bindings; should that fail, the shader without (`rt.rs`).
        let rt_expand = rt::RtScene::expand_pipeline(gpu, assets);
        let rt_pack = crate::packs::pack_uses("world.wgsl", "rt_sun");
        let mut rt_compiled = rt_expand.is_some() && rt::rt_setting() && rt_pack;
        crate::gpu_layout::set_rt_prelude(rt_compiled);
        let made = world_shader(assets)
            .map_err(|e| e.0)
            .and_then(|(s, slots)| Ok((Self::pipelines(gpu, &s)?, s, slots)));
        let made = match made {
            Err(e) if rt_compiled => {
                log::error!("world shader with traced shadows: {e}; without them");
                rt_compiled = false;
                crate::gpu_layout::set_rt_prelude(false);
                world_shader(assets)
                    .map_err(|e| e.0)
                    .and_then(|(s, slots)| Ok((Self::pipelines(gpu, &s)?, s, slots)))
            }
            m => m,
        };
        let ((face_pipes, quad_pipes, lod_pipe), shader, slots) = made?;
        let fade_pipes = Self::fade_pipelines(gpu, &shader);
        let lod_open_pipe = Self::lod_open_pipeline(gpu, &shader, "fs_lod_open");
        let lod_end_pipe = Self::lod_open_pipeline(gpu, &shader, "fs_lod_end");
        let cutout_split = Self::cutout_split_pipelines(gpu, &shader);
        let shadow_pipes = Self::shadow_pipelines(gpu, &shader);
        let lod_shadow_pipe = Self::lod_shadow_pipeline(gpu, &shader, shadow_pipes.is_some());
        let tex_shader = shader_table(gpu, c, textures, &slots);

        let (tex_info, anim, overlays) = Self::texture_info(gpu, textures);

        // Tint kind colors per biome, then the blended columns (`set_tint_tiles`).
        let biome_data: Vec<u32> = if c.biomes.is_empty() {
            c.tint_kinds.iter().map(|k| k.1).collect()
        } else {
            c.biomes
                .iter()
                .flat_map(|b| b.tints.iter().copied())
                .collect()
        };
        let biomes = gpu.create_buffer((biome_data.len() * 4) as u64, usage::STORAGE, "biomes");
        gpu.write_buffer(biomes, 0, bytemuck::cast_slice(&biome_data));

        let indices: Vec<u32> = (0..MAX_QUADS_PER_DRAW)
            .flat_map(|q| [0, 1, 2, 0, 2, 3].map(|i| q * 4 + i))
            .collect();
        let quad_index = gpu.create_buffer((indices.len() * 4) as u64, usage::INDEX, "quad indices");
        gpu.write_buffer(quad_index, 0, bytemuck::cast_slice(&indices));

        let frames = gpu.frames_in_flight();
        let globals = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Globals>() as u64, usage::STORAGE | usage::HOST, "globals"))
            .collect();
        let shadow_globals = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Globals>() as u64, usage::STORAGE | usage::HOST, "shadow globals"))
            .collect();
        Ok(WorldRenderer {
            shadow_pipes,
            lod_shadow_pipe,
            terrain_epoch: 0,
            shadow_origins: Default::default(),
            shadow_indirect: Default::default(),
            shadow_offsets: Default::default(),
            face_pipes,
            quad_pipes,
            lod_pipe,
            lod_open_pipe,
            lod_end_pipe,
            cutout_split,
            fade_pipes,
            lod_fade: 0,
            lod: FxHashMap::default(),
            lod_draw: Vec::new(),
            lod_half: 0,
            lod_mask: (gpu.create_buffer(4, usage::STORAGE, "lod mask"), 1),
            faces: Pool { words: FACE_WORDS, pages: Vec::new(), label: "face page" },
            quads: Pool { words: QUAD_WORDS, pages: Vec::new(), label: "quad page" },
            meshes: FxHashMap::default(),
            regions: FxHashMap::default(),
            table_only: FxHashMap::default(),
            table: CubeTable::new(gpu),
            parts: Vec::new(),
            offsets: Vec::new(),
            structures_before: StructuresBefore::default(),
            globals,
            shadow_globals,
            origins: Vec::new(),
            indirect: Vec::new(),
            occlusion: Occlusion::new(gpu, assets)?,
            runs: Vec::new(),
            trans: Vec::new(),
            sorted: FxHashMap::default(),
            sorted_from: None,
            translucent: 0,
            buffers: None,
            biomes,
            biome_table: biome_data,
            tint_tiles: 0,
            tex_info,
            anim,
            overlays,
            glow: textures.glow_buffer(),
            mat: textures.mat_buffer(),
            tex_shader,
            tex_generation: textures.generation(),
            quad_index,
            solid: Solid::new(gpu, c),
            drawn: 0,
            faces_drawn: 0,
            rt: match (rt_expand, rt::rt_setting()) {
                (Some(p), true) => Some(rt::RtScene::new(gpu, p)),
                _ => None,
            },
            rt_expand,
            rt_compiled,
            rt_wanted: rt_expand.is_some() && rt::rt_setting(),
            rt_pack,
            rt_modes: (false, false),
            pt_pipes: if rt_compiled { rt::pt_pipelines(gpu, assets) } else { None },
            pt_out: Vec::new(),
            pt_cache: None,
            gb_pipes: if rt_compiled { rt::gb_pipelines(gpu, &shader) } else { None },
            gb_draws: Vec::new(),
            gb_draws_slot: 0,
            gb_images: None,
            pt_images: None,
            pt_light_image: None,
            rt_reload: false,
            calm_sets: [Self::calm_pipes(gpu, assets, false), Self::calm_pipes(gpu, assets, true)],
            calm_level: 0,
        })
    }

    /// The colour pipelines from the world shader with `CALM` true (`packs::compose_calm`), when
    /// the pack uses it: the same draws, the shader without the paths that do nothing in a calm
    /// scene. Shadows and depth-only passes keep the plain ones.
    /// `dry`: with `DRY` true as well (`gpu_layout::DRY_OFF`).
    fn calm_pipes(gpu: &mut dyn Backend, assets: &crate::assets::Assets, dry: bool) -> Option<ColorPipes> {
        let snippets = crate::shaders::snippets(assets, &HOOK);
        let s = crate::packs::compose_calm(assets, "world.wgsl", &[(&HOOK, &snippets)], dry)?;
        let (face, quad, lod) = Self::pipelines(gpu, &s)
            .map_err(|e| log::error!("calm world pipelines: {e}"))
            .ok()?;
        Some(ColorPipes {
            face,
            quad,
            lod,
            lod_open: Self::lod_open_pipeline(gpu, &s, "fs_lod_open"),
            lod_end: Self::lod_open_pipeline(gpu, &s, "fs_lod_end"),
            cutout_split: Self::cutout_split_pipelines(gpu, &s),
            fade: Self::fade_pipelines(gpu, &s),
        })
    }

    /// Trades the colour pipelines in use with `set`'s.
    fn swap_pipes(&mut self, set: &mut ColorPipes) {
        std::mem::swap(&mut self.face_pipes, &mut set.face);
        std::mem::swap(&mut self.quad_pipes, &mut set.quad);
        std::mem::swap(&mut self.lod_pipe, &mut set.lod);
        std::mem::swap(&mut self.lod_open_pipe, &mut set.lod_open);
        std::mem::swap(&mut self.lod_end_pipe, &mut set.lod_end);
        std::mem::swap(&mut self.cutout_split, &mut set.cutout_split);
        std::mem::swap(&mut self.fade_pipes, &mut set.fade);
    }

    /// Draws with the calm pipelines (`gpu_layout::CALM_OFF`) at `level` 1, the calm-and-dry ones
    /// (`DRY_OFF`) at 2, the plain ones at 0; a level the pack has no set for takes the one below.
    /// Call before the frame's draws.
    pub fn set_calm(&mut self, level: u8) {
        let want = (1..=level.min(2))
            .rev()
            .find(|&l| self.calm_sets[l as usize - 1].is_some())
            .unwrap_or(0);
        if want == self.calm_level {
            return;
        }
        for l in [self.calm_level, want] {
            if l > 0
                && let Some(mut set) = self.calm_sets[l as usize - 1].take()
            {
                self.swap_pipes(&mut set);
                self.calm_sets[l as usize - 1] = Some(set);
            }
        }
        self.calm_level = want;
    }

    /// Puts the plain pipelines back in use and destroys the calm ones (before a reload or drop).
    fn drop_calm(&mut self, gpu: &mut dyn Backend) {
        self.set_calm(0);
        for set in self
            .calm_sets
            .iter_mut()
            .filter_map(Option::take)
            .collect::<Vec<_>>()
        {
            for p in set
                .face
                .into_iter()
                .chain(set.quad)
                .chain([set.lod])
                .chain(set.lod_open)
                .chain(set.lod_end)
                .chain(set.cutout_split.into_iter().flatten())
                .chain(set.fade.into_iter().flatten())
            {
                gpu.destroy_pipeline(p);
            }
        }
    }

    /// The block texture tables (`GameTextures::world_tables`) on the GPU.
    fn texture_info(gpu: &mut dyn Backend, textures: &GameTextures) -> (BufferId, BufferId, u32) {
        let (info, frames, overlays) = textures.world_tables();
        let tex_info = gpu.create_buffer((info.len().max(1) * 4) as u64, usage::STORAGE, "texture info");
        gpu.write_buffer(tex_info, 0, bytemuck::cast_slice(&info));
        let anim = gpu.create_buffer((frames.len() * 4) as u64, usage::STORAGE, "texture frames");
        gpu.write_buffer(anim, 0, bytemuck::cast_slice(&frames));
        (tex_info, anim, overlays)
    }

    /// Builds the texture tables again when textures were loaded since (cheap: a few words per
    /// block texture).
    pub fn set_textures(&mut self, gpu: &mut dyn Backend, textures: &GameTextures) {
        self.glow = textures.glow_buffer();
        self.mat = textures.mat_buffer();
        if self.tex_generation == textures.generation() {
            return;
        }
        gpu.destroy_buffer(self.tex_info);
        gpu.destroy_buffer(self.anim);
        (self.tex_info, self.anim, self.overlays) = Self::texture_info(gpu, textures);
        self.tex_generation = textures.generation();
    }

    fn pipelines(gpu: &mut dyn Backend, shader: &Shader) -> Result<([PipelineId; 3], [PipelineId; 3], PipelineId), String> {
        let depth_write = Some(DepthState { write: true, compare: Compare::GreaterEqual });
        // See-through blocks only test depth, unless the shader pack wants their surface in it.
        let depth_read = Some(DepthState { write: crate::packs::translucent_depth(), compare: Compare::GreaterEqual });
        let mut make = |label: &str, vs: &str, fs: &str, depth, blend, cull| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader,
                vertex: vs,
                fragment: Some(fs),
                color: Some(crate::post::world_format()),
                depth,
                blend,
                cull_back: cull,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            })
            .map_err(|e| e.0)
        };
        // `WEFT_NO_ALPHA_TEST=1`: cutout (leaves, plants) drawn as opaque, no discard, to measure
        // what alpha testing costs (a Mac row; leaves then show as solid boxes).
        let cutout = if std::env::var_os("WEFT_NO_ALPHA_TEST").is_some() { "fs_opaque" } else { "fs_cutout" };
        let faces = [
            make("faces opaque", "vs_face", "fs_opaque", depth_write, Blend::None, true)?,
            make("faces cutout", "vs_face", cutout, depth_write, Blend::None, true)?,
            make("faces translucent", "vs_face", "fs_translucent", depth_read, Blend::Alpha, true)?,
        ];
        let quads = [
            make("quads opaque", "vs_quad", "fs_opaque", depth_write, Blend::None, true)?,
            make("quads cutout", "vs_quad", cutout, depth_write, Blend::None, true)?,
            make("quads translucent", "vs_quad", "fs_translucent", depth_read, Blend::Alpha, true)?,
        ];
        let lod = make("distant terrain", "vs_lod", "fs_lod", depth_write, Blend::None, true)?;
        Ok((faces, quads, lod))
    }

    /// `lod_open_pipe` (`entry` "fs_lod_open") or `lod_end_pipe` ("fs_lod_end"): None when the
    /// pack lacks the entry, or it fails.
    fn lod_open_pipeline(gpu: &mut dyn Backend, shader: &Shader, entry: &str) -> Option<PipelineId> {
        if !crate::packs::pack_uses("world.wgsl", entry) {
            return None;
        }
        // Not on a phone until measured there: its `discard` costs a tiled GPU's hidden-surface
        // removal (the Pixel 10: a third of the world pass for `fs_lod`'s).
        if entry == "fs_lod_end" && cfg!(target_os = "android") {
            return None;
        }
        gpu.create_pipeline(&PipelineDesc {
            label: if entry == "fs_lod_end" { "distant terrain end" } else { "distant terrain open" },
            shader,
            vertex: "vs_lod",
            fragment: Some(entry),
            color: Some(crate::post::world_format()),
            depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
            blend: Blend::None,
            cull_back: true,
            topology: Topology::Triangles,
            motion: crate::post::motion(),
        })
        .map_err(|e| log::error!("world shader {entry}: {}", e.0))
        .ok()
    }

    /// `cutout_split`: None when `cutout_prepass()` is off, the pack lacks `fs_cutout_depth` or
    /// `fs_cutout_shade`, or one fails.
    fn cutout_split_pipelines(gpu: &mut dyn Backend, shader: &Shader) -> Option<[PipelineId; 4]> {
        if !cutout_prepass()
            || !["fs_cutout_depth", "fs_cutout_shade"]
                .iter()
                .all(|e| crate::packs::pack_uses("world.wgsl", e))
        {
            return None;
        }
        let mut made = Vec::new();
        for (label, vs, fs, depth, blend) in [
            (
                "faces cutout depth",
                "vs_face",
                "fs_cutout_depth",
                DepthState { write: true, compare: Compare::GreaterEqual },
                Blend::NoColor,
            ),
            ("faces cutout shade", "vs_face", "fs_cutout_shade", DepthState { write: false, compare: Compare::Equal }, Blend::None),
            (
                "quads cutout depth",
                "vs_quad",
                "fs_cutout_depth",
                DepthState { write: true, compare: Compare::GreaterEqual },
                Blend::NoColor,
            ),
            ("quads cutout shade", "vs_quad", "fs_cutout_shade", DepthState { write: false, compare: Compare::Equal }, Blend::None),
        ] {
            match gpu.create_pipeline(&PipelineDesc {
                label,
                shader,
                vertex: vs,
                fragment: Some(fs),
                color: Some(crate::post::world_format()),
                depth: Some(depth),
                blend,
                cull_back: true,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            }) {
                Ok(p) => made.push(p),
                Err(e) => {
                    log::error!("world shader {fs}: {}", e.0);
                    for p in made {
                        gpu.destroy_pipeline(p);
                    }
                    return None;
                }
            }
        }
        made.try_into().ok()
    }

    /// The two passes of a run of `kind` (0 faces, 1 quads) and `layer` (1 cutout) when cutout
    /// blocks are split (`cutout_split`): depth, then shade.
    fn split_of(&self, kind: u8, layer: u8) -> Option<(PipelineId, PipelineId)> {
        let s = self.cutout_split?;
        (layer == 1 && kind < 2).then(|| (s[kind as usize * 2], s[kind as usize * 2 + 1]))
    }

    /// `fade_pipes`: None when the pack lacks `fs_opaque_fade`, `fs_cutout_fade` or
    /// `fs_translucent_fade`, or one fails.
    fn fade_pipelines(gpu: &mut dyn Backend, shader: &Shader) -> Option<[PipelineId; 6]> {
        if !["fs_opaque_fade", "fs_cutout_fade", "fs_translucent_fade"]
            .iter()
            .all(|e| crate::packs::pack_uses("world.wgsl", e))
        {
            return None;
        }
        let solid = Some(DepthState { write: true, compare: Compare::GreaterEqual });
        let see_through = Some(DepthState { write: crate::packs::translucent_depth(), compare: Compare::GreaterEqual });
        let mut made = Vec::new();
        for (label, vs, fs, depth, blend) in [
            ("faces opaque fading", "vs_face", "fs_opaque_fade", solid, Blend::None),
            ("faces cutout fading", "vs_face", "fs_cutout_fade", solid, Blend::None),
            ("faces translucent fading", "vs_face", "fs_translucent_fade", see_through, Blend::Alpha),
            ("quads opaque fading", "vs_quad", "fs_opaque_fade", solid, Blend::None),
            ("quads cutout fading", "vs_quad", "fs_cutout_fade", solid, Blend::None),
            ("quads translucent fading", "vs_quad", "fs_translucent_fade", see_through, Blend::Alpha),
        ] {
            match gpu.create_pipeline(&PipelineDesc {
                label,
                shader,
                vertex: vs,
                fragment: Some(fs),
                color: Some(crate::post::world_format()),
                depth,
                blend,
                cull_back: true,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            }) {
                Ok(p) => made.push(p),
                Err(e) => {
                    log::error!("world shader {fs}: {}", e.0);
                    for p in made {
                        gpu.destroy_pipeline(p);
                    }
                    return None;
                }
            }
        }
        made.try_into().ok()
    }

    /// The shadow pass's pipelines when the shader pack asks for sun shadows: depth only, the
    /// pack's `fs_shadow` leaving out the see-through texels of cutout blocks (leaves cast dappled
    /// shadows). Both sides drawn: what faces away from the sun casts as well.
    fn shadow_pipelines(gpu: &mut dyn Backend, shader: &Shader) -> Option<[PipelineId; 4]> {
        if !crate::packs::sun_shadows() {
            return None;
        }
        let depth = Some(DepthState { write: true, compare: Compare::GreaterEqual });
        let mut made: Vec<PipelineId> = Vec::new();
        for (label, vs, fs) in [
            ("faces shadow", "vs_face", None),
            ("faces shadow cutout", "vs_face", Some("fs_shadow")),
            ("quads shadow", "vs_quad", None),
            ("quads shadow cutout", "vs_quad", Some("fs_shadow")),
        ] {
            match gpu.create_pipeline(&PipelineDesc {
                label,
                shader,
                vertex: vs,
                fragment: fs,
                color: None,
                depth,
                blend: Blend::None,
                cull_back: false,
                topology: Topology::Triangles,
                motion: false,
            }) {
                Ok(p) => made.push(p),
                Err(e) => {
                    log::error!("sun shadows: {}", e.0);
                    for p in made {
                        gpu.destroy_pipeline(p);
                    }
                    return None;
                }
            }
        }
        Some([made[0], made[1], made[2], made[3]])
    }

    /// The far cascade's pipeline for distant terrain (`lod_shadow_pipe`): depth only, the pack's
    /// `fs_lod_shadow` dropping the columns cubes show. None without sun shadows or that entry.
    fn lod_shadow_pipeline(gpu: &mut dyn Backend, shader: &Shader, shadows: bool) -> Option<PipelineId> {
        if !shadows || !crate::packs::pack_uses("world.wgsl", "fs_lod_shadow") {
            return None;
        }
        gpu.create_pipeline(&PipelineDesc {
            label: "distant terrain shadow",
            shader,
            vertex: "vs_lod",
            fragment: Some("fs_lod_shadow"),
            color: None,
            depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
            blend: Blend::None,
            cull_back: false,
            topology: Topology::Triangles,
            motion: false,
        })
        .map_err(|e| log::error!("distant terrain shadows: {}", e.0))
        .ok()
    }

    /// Whether the shader pack's sun shadows can be drawn (`draw_shadow`).
    pub fn has_shadows(&self) -> bool {
        self.shadow_pipes.is_some()
    }

    /// The terrain's solid and cutout blocks within the shadow box `light` (camera-relative to its
    /// clip space, an orthographic box along the light) into the bound depth target: the world's
    /// cubes only (structures and distant terrain cast none), with `scene` the shadow pass's.
    /// `leaning`: structures that lean (chiacraft, a falling tree), their block (0, 0, 0)'s place
    /// in the world, pivot and rotation: they cast shadows, turned; other structures cast none.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_shadow(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        light: Mat4,
        scene: BufferId,
        cascade: usize,
        leaning: &[(u64, DVec3, Lean)],
    ) {
        let Some(pipes) = self.shadow_pipes else { return };
        let cam_block = cam.pos.floor();
        let cam_cube: IVec3 = cam_block.as_ivec3() >> 4i32;
        let cam_frac = (cam.pos - (cam_cube * 16).as_dvec3()).as_vec3();
        let globals = Globals {
            cam_frac: [cam_frac.x, cam_frac.y, cam_frac.z, 0.0],
            ids: [0, 0, (self.biome_table.len() / crate::content::content().tint_kinds.len()) as u32, self.tint_tiles],
            cam_cube: cam_cube.extend(0).to_array(),
            extra: [self.overlays, 0, crate::content::content().tint_kinds.len() as u32, self.lod_half],
        };
        let gbuf = self.shadow_globals[frame.slot];
        gpu.mapped(gbuf)[..size_of::<Globals>()].copy_from_slice(bytemuck::bytes_of(&globals));
        // The light's box on all six sides: along the light too, so what lies beyond its depth (the
        // ground far behind, caves deep under) is not drawn into the map for nothing.
        let planes = box_planes(light);
        // The faces the light falls on: the same for every mesh (it comes from one direction), not
        // the ones the camera sees. Its depth row points toward it (reverse depth). Faces nearly
        // edge-on to it are kept either way.
        let to_light = light.row(2).truncate().normalize_or_zero();
        let sides = [to_light.x > -0.05, to_light.y > -0.05, to_light.z > -0.05, to_light.x < 0.05, to_light.y < 0.05, to_light.z < 0.05];
        let side = (16 << REGION) as f32;
        let mut draws: Vec<DrawRecord> = Vec::new();
        for (&(f, r), meshes) in &self.regions {
            if f != 0 {
                continue;
            }
            let center = ((r << REGION) - cam_cube).as_vec3() * 16.0 - cam_frac + Vec3::splat(side / 2.0);
            if !planes
                .iter()
                .all(|pl| pl.dot(center.extend(1.0)) > -side * 0.8661)
            {
                continue;
            }
            for &(key, m) in meshes {
                if key.0 != 0 {
                    continue;
                }
                let lo = (key.1 - cam_cube).as_vec3() * 16.0 - cam_frac;
                let c = lo + Vec3::splat(8.0);
                if planes.iter().all(|pl| pl.dot(c.extend(1.0)) > -13.86) {
                    mesh_draws(&m, key, lo, 0, cam_cube, sides, 0, &mut draws);
                }
            }
        }
        // The world's meshes with nothing translucent are kept only for `CubeTable` (`table_only`),
        // not in `regions`: they cast too (chiacraft's sun shadows), or most terrain would cast none.
        for (&key, m) in &self.table_only {
            let lo = (key.1 - cam_cube).as_vec3() * 16.0 - cam_frac;
            let c = lo + Vec3::splat(8.0);
            if planes.iter().all(|pl| pl.dot(c.extend(1.0)) > -13.86) {
                mesh_draws(m, key, lo, 0, cam_cube, sides, 0, &mut draws);
            }
        }
        // Leaning structures (a falling tree), every face (the light's side turns with them).
        for (i, &(id, origin, _)) in leaning.iter().enumerate() {
            let at = (origin - (cam_cube * 16).as_dvec3()).as_vec3();
            for (&(f, _), meshes) in &self.regions {
                if f != id {
                    continue;
                }
                for &(key, m) in meshes {
                    if key.0 == id {
                        let lo = at + key.1.as_vec3() * 16.0 - cam_frac;
                        mesh_draws(&m, key, lo, i as i32 + 1, cam_cube, [true; 6], 0, &mut draws);
                    }
                }
            }
        }
        // Solid and cutout layers only, in runs of one pipeline and page.
        draws.retain(|d| d.1 < 2);
        // The far cascade: the distant terrain's tiles shown (`lod_draw`) within the light's box,
        // behind the camera too (their shadows fall into view), as `draw` makes their records.
        if cascade == 1 && self.lod_shadow_pipe.is_some() {
            for key in &self.lod_draw {
                let Some(&(a, count, (y0, y1), skip)) = self.lod.get(key) else { continue };
                let o = key.origin();
                let half = Vec3::new(key.size() as f32, (y1 - y0) as f32, key.size() as f32) / 2.0;
                let center = Vec3::new(o.x as f32, y0 as f32, o.y as f32) + half - (cam_cube * 16).as_vec3() - cam_frac;
                if !planes
                    .iter()
                    .all(|pl| pl.dot(center.extend(1.0)) > -half.length())
                {
                    continue;
                }
                let args = DrawIndexedArgs {
                    indices: (count - skip) * 6,
                    instances: 1,
                    first_index: 0,
                    vertex_offset: ((a.offset + skip) * 4) as i32,
                    first_instance: 0,
                };
                draws.push((3, 0, a.page, args, [o.x / 16 - cam_cube.x, -cam_cube.y, o.y / 16 - cam_cube.z, 0], center));
            }
        }
        draws.sort_unstable_by_key(|d| (d.0, d.1, d.2));
        let n = draws.len() as u32;
        if n == 0 {
            return;
        }
        let obuf =
            crate::gpu_layout::frame_buffer(gpu, &mut self.shadow_origins[cascade], frame.slot, n, 16, usage::STORAGE, "shadow origins");
        let ibuf =
            crate::gpu_layout::frame_buffer(gpu, &mut self.shadow_indirect[cascade], frame.slot, n, 20, usage::INDIRECT, "shadow indirect");
        // As `draw`'s: three vec4 a structure (offset and light, rotation, pivot and 1 for leaning).
        let entries = (leaning.len() as u32 + 1) * 3;
        let fbuf = crate::gpu_layout::frame_buffer(
            gpu,
            &mut self.shadow_offsets[cascade],
            frame.slot,
            entries,
            16,
            usage::STORAGE,
            "shadow offsets",
        );
        {
            let offs = gpu.mapped(fbuf);
            offs[..48].fill(0);
            for (i, &(_, origin, (pivot, rot))) in leaning.iter().enumerate() {
                let at = (i + 1) * 48;
                let o = (origin - (cam_cube * 16).as_dvec3()).as_vec3();
                offs[at..at + 12].copy_from_slice(bytemuck::cast_slice(&o.to_array()));
                offs[at + 12..at + 16].copy_from_slice(&(crate::world::light::SKY_FULL as u32).to_le_bytes());
                offs[at + 16..at + 32].copy_from_slice(bytemuck::cast_slice(&rot.to_array()));
                offs[at + 32..at + 44].copy_from_slice(bytemuck::cast_slice(&pivot.to_array()));
                offs[at + 44..at + 48].copy_from_slice(&1.0f32.to_le_bytes());
            }
        }
        {
            let origins = gpu.mapped(obuf);
            for (k, d) in draws.iter().enumerate() {
                origins[k * 16..k * 16 + 16].copy_from_slice(bytemuck::bytes_of(&d.4));
            }
        }
        {
            let indirect = gpu.mapped(ibuf);
            for (k, d) in draws.iter().enumerate() {
                let mut a = d.3;
                a.first_instance = k as u32;
                indirect[k * 20..k * 20 + 20].copy_from_slice(bytemuck::bytes_of(&a));
            }
        }
        list.push(Cmd::IndexBuffer(self.quad_index));
        let mut start = 0usize;
        while start < draws.len() {
            let (kind, layer, page) = (draws[start].0, draws[start].1, draws[start].2);
            let mut end = start;
            while end < draws.len() && draws[end].0 == kind && draws[end].1 == layer && draws[end].2 == page {
                end += 1;
            }
            let cutout = layer as usize;
            let (pipe, buf) = if kind == 3 {
                // Only pushed with the pipeline there (above).
                (self.lod_shadow_pipe.unwrap_or(pipes[0]), self.faces.pages[page as usize].0)
            } else if kind == 0 {
                (pipes[cutout], self.faces.pages[page as usize].0)
            } else {
                (pipes[2 + cutout], self.quads.pages[page as usize].0)
            };
            list.pipeline(pipe);
            use crate::gpu_layout::bind::world as b;
            list.bind(b::G, gbuf);
            list.bind(b::ORIGINS, obuf);
            list.bind(b::RECORDS, buf);
            list.bind(b::BIOMES, self.biomes);
            list.bind(b::TEX_INFO, self.tex_info);
            list.bind(b::FRAMES, self.anim);
            list.bind(b::OFFSETS, fbuf);
            list.bind(b::GLOW_MAP, self.glow);
            list.bind(b::TEX_SHADER, self.tex_shader);
            list.bind(b::SCENE, scene);
            list.bind(b::LOD_MASK, self.lod_mask.0);
            list.bind(b::SOLID, self.solid.buf);
            list.bind(b::MAT_MAP, self.mat);
            list.push(Cmd::DrawIndexedIndirect { buffer: ibuf, offset: start as u64 * 20, count: (end - start) as u32 });
            start = end;
        }
    }

    /// Recompiles the world shader; keeps the old pipelines when the new source has errors.
    pub fn reload_shaders(&mut self, gpu: &mut dyn Backend, textures: &GameTextures, assets: &crate::assets::Assets) {
        self.occlusion.reload(gpu, assets);
        // Traced only with the scene there to bind (`set_rt` makes it first).
        self.rt_pack = crate::packs::pack_uses("world.wgsl", "rt_sun");
        let rt = self.rt_wanted && self.rt.is_some() && self.rt_pack;
        crate::gpu_layout::set_rt_prelude(rt);
        self.rt_reload = false;
        match world_shader(assets)
            .map_err(|e| e.0)
            .and_then(|(s, slots)| Ok((Self::pipelines(gpu, &s)?, slots)))
        {
            Ok(((f, q, l), slots)) => {
                self.drop_calm(gpu);
                for p in self
                    .face_pipes
                    .into_iter()
                    .chain(self.quad_pipes)
                    .chain([self.lod_pipe])
                {
                    gpu.destroy_pipeline(p);
                }
                self.face_pipes = f;
                self.quad_pipes = q;
                self.lod_pipe = l;
                self.rt_compiled = rt;
                for p in self.pt_pipes.take().into_iter().flatten() {
                    gpu.destroy_pipeline(p);
                }
                for p in self.gb_pipes.take().into_iter().flatten() {
                    gpu.destroy_pipeline(p);
                }
                if rt {
                    self.pt_pipes = rt::pt_pipelines(gpu, assets);
                    if let Ok((s, _)) = world_shader(assets) {
                        self.gb_pipes = rt::gb_pipelines(gpu, &s);
                    }
                }
                for p in self
                    .shadow_pipes
                    .take()
                    .into_iter()
                    .flatten()
                    .chain(self.lod_shadow_pipe.take())
                    .chain(self.lod_open_pipe.take())
                    .chain(self.lod_end_pipe.take())
                    .chain(self.cutout_split.take().into_iter().flatten())
                    .chain(self.fade_pipes.take().into_iter().flatten())
                {
                    gpu.destroy_pipeline(p);
                }
                if let Ok((s, _)) = world_shader(assets) {
                    self.fade_pipes = Self::fade_pipelines(gpu, &s);
                    self.lod_open_pipe = Self::lod_open_pipeline(gpu, &s, "fs_lod_open");
                    self.lod_end_pipe = Self::lod_open_pipeline(gpu, &s, "fs_lod_end");
                    self.cutout_split = Self::cutout_split_pipelines(gpu, &s);
                    self.shadow_pipes = Self::shadow_pipelines(gpu, &s);
                    self.lod_shadow_pipe = Self::lod_shadow_pipeline(gpu, &s, self.shadow_pipes.is_some());
                    self.terrain_epoch += 1;
                }
                gpu.destroy_buffer(self.tex_shader);
                self.tex_shader = shader_table(gpu, crate::content::content(), textures, &slots);
                self.solid
                    .set_on(gpu, crate::packs::pack_uses("world.wgsl", "solid_at"));
                self.calm_sets = [Self::calm_pipes(gpu, assets, false), Self::calm_pipes(gpu, assets, true)];
                log::info!("world shader reloaded");
            }
            Err(e) => log::error!("world shader: {e}"),
        }
    }

    /// Sizes the blended color area to `tiles` x `tiles` columns; true when it was reallocated
    /// and every column has to be uploaded again.
    pub fn set_tint_tiles(&mut self, gpu: &mut dyn Backend, tiles: u32) -> bool {
        if tiles == self.tint_tiles {
            return false;
        }
        gpu.destroy_buffer(self.biomes);
        let size = (self.biome_table.len() + (tiles * tiles) as usize * 256 * crate::content::content().tint_kinds.len()) * 4;
        self.biomes = gpu.create_buffer(size as u64, usage::STORAGE, "biomes");
        gpu.write_buffer(self.biomes, 0, bytemuck::cast_slice(&self.biome_table));
        // Zero marks a block without blended colors yet.
        gpu.write_buffer(self.biomes, self.biome_table.len() as u64 * 4, &vec![0u8; size - self.biome_table.len() * 4]);
        self.tint_tiles = tiles;
        true
    }

    /// Every biome's tint colors from now on (`ClientWorld::biome_tints`), for blocks whose
    /// column has no blended colors yet and for distant terrain.
    pub fn set_biome_table(&mut self, gpu: &mut dyn Backend, table: Vec<u32>) {
        if table.len() != self.biome_table.len() || table == self.biome_table {
            return;
        }
        gpu.write_buffer(self.biomes, 0, bytemuck::cast_slice(&table));
        self.biome_table = table;
    }

    pub fn upload_tints(&mut self, gpu: &mut dyn Backend, col: glam::IVec2, tints: &super::world::ColumnTints) {
        let t = self.tint_tiles as i32;
        let slot = (col.x.rem_euclid(t) + col.y.rem_euclid(t) * t) as usize;
        gpu.write_buffer(self.biomes, ((self.biome_table.len() + slot * tints.len()) * 4) as u64, bytemuck::cast_slice(tints));
    }

    pub fn remove(&mut self, key: MeshKey) {
        self.terrain_epoch += 1;
        if let Some(rt) = &mut self.rt {
            rt.mesh_changed(key);
        }
        let m = if let Some(m) = self.table_only.remove(&key) {
            m
        } else {
            let Some(i) = self.meshes.remove(&key) else { return };
            self.sorted.remove(&key);
            let r = region_of(key);
            let list = self.regions.get_mut(&r).unwrap();
            let (_, m) = list.swap_remove(i);
            if let Some(&(moved, _)) = list.get(i) {
                self.meshes.insert(moved, i);
            }
            if list.is_empty() {
                self.regions.remove(&r);
            }
            m
        };
        self.table.remove(m.slots);
        if let Some(a) = m.faces {
            self.faces.free(a);
        }
        if let Some(a) = m.quads {
            self.quads.free(a);
        }
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some(rt) = self.rt.take() {
            rt.destroy(gpu);
        }
        if let Some(p) = self.rt_expand.take() {
            gpu.destroy_pipeline(p);
        }
        for p in self.pt_pipes.take().into_iter().flatten() {
            gpu.destroy_pipeline(p);
        }
        for (b, _) in self.pt_out.drain(..) {
            gpu.destroy_buffer(b);
        }
        if let Some(b) = self.pt_cache.take() {
            gpu.destroy_buffer(b);
        }
        for p in self.gb_pipes.take().into_iter().flatten() {
            gpu.destroy_pipeline(p);
        }
        if let Some((a, b, c, _)) = self.gb_images.take() {
            for t in [a, b, c] {
                gpu.destroy_texture(t);
            }
        }
        if let Some((t, _)) = self.pt_light_image.take() {
            gpu.destroy_texture(t);
        }
        if let Some((a, b, _)) = self.pt_images.take() {
            gpu.destroy_texture(a);
            gpu.destroy_texture(b);
        }
        self.drop_calm(gpu);
        self.meshes.clear();
        self.regions.clear();
        self.table_only.clear();
        self.table.destroy(gpu);
        self.lod.clear();
        for (b, _) in self.faces.pages.drain(..).chain(self.quads.pages.drain(..)) {
            gpu.destroy_buffer(b);
        }
        let shadow_globals: Vec<BufferId> = self.shadow_globals.drain(..).collect();
        for b in self.globals.drain(..).chain(shadow_globals).chain([
            self.biomes,
            self.tex_info,
            self.anim,
            self.quad_index,
            self.tex_shader,
            self.lod_mask.0,
            self.solid.buf,
        ]) {
            gpu.destroy_buffer(b);
        }
        for (b, _) in self
            .origins
            .drain(..)
            .chain(self.indirect.drain(..))
            .chain(self.offsets.drain(..))
            .chain(self.shadow_origins.iter_mut().flat_map(|v| v.drain(..)))
            .chain(self.shadow_indirect.iter_mut().flat_map(|v| v.drain(..)))
            .chain(self.shadow_offsets.iter_mut().flat_map(|v| v.drain(..)))
        {
            gpu.destroy_buffer(b);
        }
        self.occlusion.destroy(gpu);
        for p in self
            .face_pipes
            .into_iter()
            .chain(self.quad_pipes)
            .chain([self.lod_pipe])
            .chain(self.shadow_pipes.take().into_iter().flatten())
            .chain(self.lod_shadow_pipe.take())
            .chain(self.lod_open_pipe.take())
            .chain(self.lod_end_pipe.take())
            .chain(self.cutout_split.take().into_iter().flatten())
            .chain(self.fade_pipes.take().into_iter().flatten())
        {
            gpu.destroy_pipeline(p);
        }
    }

    /// Brings `Solid` up to date for the cubes within 7 of `center` (their slots are distinct).
    pub fn update_solid(&mut self, gpu: &mut dyn Backend, world: &super::world::ClientWorld, center: IVec3) {
        let s = &mut self.solid;
        if !s.on {
            return;
        }
        let now_seen = (center, world.revision());
        if s.done == Some(now_seen) {
            return;
        }
        s.done = Some(now_seen);
        let mut rebuilt = 0;
        let mut bits = [0u32; 128];
        // Nearest first, so a frame out of budget has fixed what is close.
        'shells: for r in 0i32..=7 {
            for dy in -r..=r {
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx.abs().max(dy.abs()).max(dz.abs()) != r {
                            continue;
                        }
                        let p = center + IVec3::new(dx, dy, dz);
                        let slot = ((p.y & 15) * 256 + (p.z & 15) * 16 + (p.x & 15)) as usize;
                        let now = world.cube_rev(p);
                        let key = now.map(|(_, v)| (p, v));
                        if s.held[slot] == key {
                            continue;
                        }
                        if rebuilt >= SOLID_PER_FRAME {
                            // More next frame.
                            s.done = None;
                            break 'shells;
                        }
                        rebuilt += 1;
                        s.held[slot] = key;
                        let Some((cube, _)) = now else {
                            s.tags[slot] = [0; 4];
                            continue;
                        };
                        let casts = |st: u32| s.casts.get(st as usize).copied().unwrap_or(false);
                        match cube.uniform_state() {
                            Some(st) => bits.fill(if casts(st) { !0 } else { 0 }),
                            None => {
                                bits.fill(0);
                                for i in 0..crate::world::cube::VOLUME {
                                    if casts(cube.get(i)) {
                                        bits[i / 32] |= 1 << (i % 32);
                                    }
                                }
                            }
                        }
                        gpu.write_buffer(s.buf, ((SOLID_SLOTS * 4 + slot * 128) * 4) as u64, bytemuck::cast_slice(&bits));
                        s.tags[slot] = [p.x as u32, p.y as u32, p.z as u32, 1];
                    }
                }
            }
        }
        if rebuilt > 0 {
            gpu.write_buffer(s.buf, 0, bytemuck::cast_slice(&s.tags));
        }
    }

    /// A distant terrain tile's quads (`lod::mesh`), spanning heights `y`; the first `skip`
    /// records aren't drawn (a smooth tile's corners, which its cells read).
    pub fn upload_lod(
        &mut self,
        gpu: &mut dyn Backend,
        textures: &GameTextures,
        key: super::lod::TileKey,
        quads: &[[u32; 4]],
        skip: u32,
        y: (i32, i32),
    ) {
        self.remove_lod(key);
        if quads.len() <= skip as usize {
            return;
        }
        let mut used: Vec<u32> = quads[skip as usize..]
            .iter()
            .map(|q| q[1] & crate::gpu_layout::face::TEX_MASK)
            .collect();
        used.sort_unstable();
        used.dedup();
        textures.ensure_blocks(gpu, used);
        let a = self.faces.alloc(gpu, quads.len() as u32);
        gpu.write_buffer(self.faces.pages[a.page as usize].0, a.offset as u64 * 16, bytemuck::cast_slice(quads));
        self.lod.insert(key, (a, quads.len() as u32, y, skip));
        self.terrain_epoch += 1;
        if let Some(rt) = &mut self.rt {
            rt.lod_changed(key);
        }
    }

    /// The columns shown by cubes (`Lod::mask`).
    pub fn set_lod_mask(&mut self, gpu: &mut dyn Backend, mask: &[u32]) {
        if self.lod_mask.1 < mask.len() {
            gpu.destroy_buffer(self.lod_mask.0);
            self.lod_mask = (gpu.create_buffer((mask.len() * 4) as u64, usage::STORAGE, "lod mask"), mask.len());
        }
        gpu.write_buffer(self.lod_mask.0, 0, bytemuck::cast_slice(mask));
    }

    /// Distant terrain tiles held and their quads.
    pub fn lod_counts(&self) -> (usize, usize) {
        (self.lod.len(), self.lod.values().map(|t| (t.1 - t.3) as usize).sum())
    }

    pub fn remove_lod(&mut self, key: super::lod::TileKey) {
        if let Some((a, ..)) = self.lod.remove(&key) {
            self.faces.free(a);
            self.terrain_epoch += 1;
            if let Some(rt) = &mut self.rt {
                rt.lod_changed(key);
            }
        }
    }

    /// Every cube of structure `frame` goes (it left).
    pub fn remove_frame(&mut self, frame: u64) {
        let keys: Vec<MeshKey> = self
            .regions
            .iter()
            .filter(|(r, _)| r.0 == frame)
            .flat_map(|(_, l)| l.iter().map(|e| e.0))
            .collect();
        for k in keys {
            self.remove(k);
        }
    }

    /// Translucent quads near the camera go far to near again when it has come into another block
    /// of their cube, as vanilla does. `frames`: where the structures drawn are.
    fn sort_translucent(&mut self, gpu: &mut dyn Backend, cam: DVec3, frames: &[(u64, DVec3)]) {
        let at = cam.floor().as_ivec3();
        if self.sorted_from == Some(at) && frames.is_empty() {
            return;
        }
        self.sorted_from = Some(at);
        let began = std::time::Instant::now();
        // Which cubes to redo, picked here (a distance each), then sorted.
        let mut work: Vec<(MeshKey, &mut SortedQuads, IVec3)> = self
            .sorted
            .iter_mut()
            .filter_map(|(key, s)| {
                let origin = if key.0 == 0 { DVec3::ZERO } else { frames.iter().find(|f| f.0 == key.0)?.1 };
                let here = cam - origin - (key.1 * 16).as_dvec3();
                if (here - DVec3::splat(8.0)).length() > SORT_REACH + 14.0 {
                    return None;
                }
                let block = here.floor().as_ivec3();
                if s.seen_from == Some(block) {
                    return None;
                }
                s.seen_from = Some(block);
                Some((*key, s, (here * 32.0).as_ivec3()))
            })
            .collect();
        let sort = |(_, s, eye): &mut (MeshKey, &mut SortedQuads, IVec3)| {
            let eye = *eye;
            s.quads.sort_by_cached_key(|q| {
                let m = crate::gpu_layout::quad_middle(q);
                let d = [m[0] - eye.x, m[1] - eye.y, m[2] - eye.z].map(|v| v as i64);
                std::cmp::Reverse(d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
            });
        };
        // A few thousand quads (a lake, a window) sort here in a fraction of a millisecond; handed
        // to the pool they waited for a worker to wake and get a core, up to 15 ms on a busy Mac
        // (the lag runs, 11-04). Only a sea's worth goes to every core.
        if work.iter().map(|w| w.1.quads.len()).sum::<usize>() < PARALLEL_SORT {
            work.iter_mut().for_each(sort);
        } else {
            work.par_iter_mut().for_each(sort);
        }
        let redone: Vec<MeshKey> = work.into_iter().map(|w| w.0).collect();
        let mut quads = 0;
        for key in &redone {
            let s = &self.sorted[key];
            quads += s.quads.len();
            gpu.write_buffer(self.quads.pages[s.page as usize].0, s.offset as u64 * 32, bytemuck::cast_slice(&s.quads));
        }
        if !redone.is_empty() {
            log::debug!(target: "frame", "translucent: {} cubes, {quads} quads sorted in {:.2} ms", redone.len(), began.elapsed().as_secs_f64() * 1000.0);
        }
    }

    pub fn upload(&mut self, gpu: &mut dyn Backend, textures: &GameTextures, key: MeshKey, mesh: &Mesh) {
        self.remove(key);
        if mesh.is_empty() {
            return;
        }
        self.terrain_epoch += 1;
        let began = std::time::Instant::now();
        // The textures it shows are loaded before it is drawn: a handful of distinct ones over
        // thousands of records, neighbours mostly the same (collecting one a record and sorting
        // them was a good part of an upload, perf review client #6).
        let mut used: Vec<u32> = Vec::with_capacity(16);
        let mut last = u32::MAX;
        let ids = mesh
            .faces
            .iter()
            .flatten()
            .map(|f| f[1] & crate::gpu_layout::face::TEX_MASK)
            .chain(
                mesh.quads
                    .iter()
                    .flatten()
                    .map(|q| q[4] & crate::gpu_layout::face::TEX_MASK),
            );
        for id in ids {
            if id != last && !used.contains(&id) {
                used.push(id);
            }
            last = id;
        }
        textures.ensure_blocks(gpu, used);
        let face_counts = mesh.faces.each_ref().map(|v| {
            let mut n = [0u32; 6];
            for f in v {
                n[super::mesher::DIR_RANK[crate::gpu_layout::face_dir(f)] as usize] += 1;
            }
            n
        });
        let quad_counts = mesh.quads.each_ref().map(|v| v.len() as u32);
        let total_f: u32 = face_counts.iter().flatten().sum();
        let total_q: u32 = quad_counts.iter().sum();
        let faces = (total_f > 0).then(|| {
            let a = self.faces.alloc(gpu, total_f);
            // Each layer in its place, one after the other: no joined copy first (staging copies anyway).
            let page = self.faces.pages[a.page as usize].0;
            let mut at = a.offset as u64;
            for layer in &mesh.faces {
                gpu.write_buffer(page, at * 16, bytemuck::cast_slice(layer));
                at += layer.len() as u64;
            }
            a
        });
        let quads = (total_q > 0).then(|| {
            let a = self.quads.alloc(gpu, total_q);
            let page = self.quads.pages[a.page as usize].0;
            let mut at = a.offset as u64;
            for layer in &mesh.quads {
                gpu.write_buffer(page, at * 32, bytemuck::cast_slice(layer));
                at += layer.len() as u64;
            }
            if !mesh.quads[2].is_empty() {
                self.sorted.insert(
                    key,
                    SortedQuads {
                        page: a.page,
                        offset: a.offset + quad_counts[0] + quad_counts[1],
                        quads: mesh.quads[2].clone(),
                        seen_from: None,
                    },
                );
                self.sorted_from = None;
            }
            a
        });
        let slots = if key.0 == 0 {
            self.table
                .add(key.1, faces, &face_counts, quads, &quad_counts)
        } else {
            [u32::MAX; 4]
        };
        let mesh = GpuMesh { slots, faces, face_counts, quads, quad_counts };
        crate::diag::UPLOADS.add(1);
        crate::diag::UPLOAD_NS.add(began.elapsed().as_nanos() as u64);
        if key.0 == 0 && face_counts[2].iter().sum::<u32>() + quad_counts[2] == 0 {
            self.table_only.insert(key, mesh);
            return;
        }
        let list = self.regions.entry(region_of(key)).or_default();
        self.meshes.insert(key, list.len());
        list.push((key, mesh));
    }

    /// `frames`: structures to draw and where their block (0, 0, 0) is in the world; `leans`:
    /// those that lean (chiacraft, a falling tree), their pivot (own coordinates) and rotation.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        early: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        scene: BufferId,
        frames: &[(u64, DVec3)],
        lights: &FxHashMap<MeshKey, u32>,
        leans: &FxHashMap<u64, (Vec3, glam::Quat)>,
    ) {
        self.sort_translucent(gpu, cam.pos, frames);
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let view_proj = cam.proj(aspect) * cam.view();
        let last_frame = self.occlusion.next_frame(view_proj, cam.pos);
        let cam_block = cam.pos.floor();
        let cam_cube: IVec3 = cam_block.as_ivec3() >> 4i32;
        let cam_frac = (cam.pos - (cam_cube * 16).as_dvec3()).as_vec3();
        let fade = if self.fade_pipes.is_some() { self.lod_fade } else { 0 };
        let globals = Globals {
            cam_frac: [cam_frac.x, cam_frac.y, cam_frac.z, 0.0],
            ids: [0, 0, (self.biome_table.len() / crate::content::content().tint_kinds.len()) as u32, self.tint_tiles],
            cam_cube: cam_cube.extend(0).to_array(),
            extra: [
                self.overlays,
                fade,
                crate::content::content().tint_kinds.len() as u32,
                self.lod_half
                    | (crate::content::content()
                        .game
                        .config_names
                        .get("core:far_look")
                        .map_or(7, |&id| crate::config::get(id) as u32)
                        & 255)
                        << 24,
            ],
        };
        let gbuf = self.globals[frame.slot];
        gpu.mapped(gbuf)[..size_of::<Globals>()].copy_from_slice(bytemuck::bytes_of(&globals));

        let planes = frustum_planes(view_proj);
        // Camera-relative offset of each structure drawn (k + 1 its index; 0 the world).
        let frame_offsets: Vec<Vec3> = frames
            .iter()
            .map(|f| (f.1 - (cam_cube * 16).as_dvec3()).as_vec3())
            .collect();
        // What a structure's meshes pick: its place and the world's light on that cube of it,
        // full sky until it is known.
        // And how it leans (chiacraft): its pivot and rotation, upright for most.
        let lean_of = |k: usize| leans.get(&frames[k].0).copied();
        // (Its frame's index too, for where it was last frame: `structures_before`.)
        let mut picks: Vec<(Vec3, u32, Option<Lean>, usize)> = frame_offsets
            .iter()
            .enumerate()
            .map(|(k, o)| (*o, crate::world::light::SKY_FULL as u32, lean_of(k), k))
            .collect();
        let mut cube_pick: FxHashMap<MeshKey, i32> = FxHashMap::default();
        for (&key, &light) in lights {
            if let Some(k) = frames.iter().position(|f| f.0 == key.0) {
                // A leaning one's cubes are elsewhere than where the world's light on them was
                // sampled (a falling trunk's lowest cube is centred in the ground): its own light.
                let light = if lean_of(k).is_some() { crate::world::light::SKY_FULL as u32 } else { light };
                picks.push((frame_offsets[k], light, lean_of(k), k));
                cube_pick.insert(key, picks.len() as i32);
            }
        }
        // Regions in view, nearest first: what is drawn first then hides what is behind it.
        let side = (16 << REGION) as f32;
        let mut seen: Vec<(f32, &Vec<(MeshKey, GpuMesh)>)> = self
            .regions
            .iter()
            .filter_map(|(&(f, r), list)| {
                if f != 0 {
                    return Some((0.0, list));
                }
                let center = ((r << REGION) - cam_cube).as_vec3() * 16.0 - cam_frac + Vec3::splat(side / 2.0);
                planes
                    .iter()
                    .all(|pl| pl.dot(center.extend(1.0)) > -side * 0.8661)
                    .then(|| (center.length_squared(), list))
            })
            .collect();
        seen.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));

        // A list per region seen, kept between frames: allocating them anew on every core each
        // frame cost more than the culling.
        let mut parts = std::mem::take(&mut self.parts);
        parts.resize_with(seen.len().max(parts.len()), Vec::new);
        // The frame thread culls and writes the draws itself up to `SERIAL_CULL` meshes (about a
        // millisecond): handed to the pool, it waited for all of its parts, and a worker put
        // aside for loading's threads held it for 20-70 ms (Ronald's window, standing while
        // distant terrain came in, 11-04). Every region a thread of its own past that.
        let meshes: usize = seen.iter().map(|s| s.1.len()).sum();
        let split = if meshes <= SERIAL_CULL { usize::MAX } else { 1 };
        let span = crate::diag::span("world/cull");
        let faces_drawn: u64 = seen
            .par_iter()
            .zip(parts.par_iter_mut())
            .with_min_len(split)
            .map(|((_, list), out)| {
                out.clear();
                let mut faces = 0u64;
                for &(key, m) in list.iter() {
                    let (lo, k) = if key.0 == 0 {
                        ((key.1 - cam_cube).as_vec3() * 16.0 - cam_frac, 0)
                    } else {
                        let Some(k) = frames.iter().position(|f| f.0 == key.0) else { continue };
                        // A leaning structure's cubes are elsewhere than their place: drawn
                        // whole, every face (chiacraft).
                        if leans.contains_key(&key.0) {
                            let lo = frame_offsets[k] + key.1.as_vec3() * 16.0 - cam_frac;
                            let pick = cube_pick.get(&key).copied().unwrap_or(k as i32 + 1);
                            faces += mesh_draws(&m, key, lo, pick, cam_cube, [true; 6], 0, out);
                            continue;
                        }
                        (frame_offsets[k] + key.1.as_vec3() * 16.0 - cam_frac, cube_pick.get(&key).copied().unwrap_or(k as i32 + 1))
                    };
                    let center = lo + Vec3::splat(8.0);
                    // Near where the cubes end (`lod_fade`): past it left to distant terrain, in
                    // the band before it drawn fading.
                    // SYNC: the world's opaque and cutout layers are `CubeTable`'s now (drawn by
                    // `cull` on the GPU), so only their translucent layer passes here: the cull must
                    // also skip cubes past `lod_fade` and send the fading band's draws to the fade
                    // pipelines (kinds 4 and 5), or the solid cubes draw past the end and don't fade
                    // (Distant terrain's fade, chiacraft).
                    let mut fading = false;
                    if fade > 0 && key.0 == 0 {
                        let (a, b) = (glam::Vec2::new(lo.x, lo.z), glam::Vec2::new(lo.x + 16.0, lo.z + 16.0));
                        let near = glam::Vec2::ZERO.clamp(a, b).length();
                        let far = a.abs().max(b.abs()).length();
                        if near > fade as f32 {
                            continue;
                        }
                        fading = far > fade as f32 - FADE_BAND;
                    }
                    if planes.iter().all(|pl| pl.dot(center.extend(1.0)) > -13.86) {
                        let from = out.len();
                        faces += mesh_draws(&m, key, lo, k, cam_cube, camera_sides(lo), if k == 0 { 2 } else { 0 }, out);
                        if fading {
                            for d in out[from..].iter_mut() {
                                d.0 |= 2;
                            }
                        }
                    }
                }
                faces
            })
            .sum();
        drop(span);
        // Into runs of one pipeline and page (layer, kind, page; translucent last), written
        // straight into the mapped buffers from every core: each region knows where its draws
        // of each run go, so the near-to-far order holds inside runs without a sort.
        let seen_parts = &parts[..seen.len()];
        let pages = self.faces.pages.len().max(self.quads.pages.len()).max(1);
        // Opaque and cutout runs, then the same fading (kind 2 and 3, `fade_pipes`); translucent
        // ones are sorted far to near over all of them.
        let nb = 8 * pages;
        let bucket = |d: &DrawRecord| ((d.0 as usize >> 1) * 4 + d.1 as usize * 2 + (d.0 & 1) as usize) * pages + d.2 as usize;
        let counts: Vec<Vec<u32>> = seen_parts
            .par_iter()
            .with_min_len(split)
            .map(|p| {
                let mut c = vec![0u32; nb];
                for d in p.iter().filter(|d| d.1 < 2) {
                    c[bucket(d)] += 1;
                }
                c
            })
            .collect();
        let mut totals = vec![0u32; nb];
        for c in &counts {
            for (t, v) in totals.iter_mut().zip(c) {
                *t += v;
            }
        }
        let mut next = Vec::with_capacity(nb);
        let mut opaque = 0u32;
        for t in &totals {
            next.push(opaque);
            opaque += t;
        }
        let mut starts: Vec<Vec<u32>> = Vec::with_capacity(counts.len());
        for c in &counts {
            starts.push(next.clone());
            for (n, v) in next.iter_mut().zip(c) {
                *n += v;
            }
        }
        let mut trans = std::mem::take(&mut self.trans);
        trans.clear();
        for p in seen_parts {
            trans.extend(p.iter().filter(|d| d.1 == 2));
        }
        // Far first; distances are positive, so their bits order like them. Inside a cube
        // `sort_translucent` has seen to it.
        trans.par_sort_unstable_by_key(|d| std::cmp::Reverse(d.5.length_squared().to_bits()));

        // Distant terrain after the cubes (nearer, they hide most of it), a run per page: the tiles
        // that reach into the cubes' mask first (kind 3), then the rest without its discard (kind 6,
        // `lod_open_pipe`), and last those reaching toward its end, faded there (kind 7, `lod_end_pipe`). A column a tile's edge walls stand on may be the next one: a column's
        // margin.
        let open = self.lod_open_pipe.is_some();
        let end = self.lod_end_pipe.is_some();
        let end_from = END_FADE * crate::client::lod::Lod::distance() as f32;
        let mask_half = self.lod_half as i32 + 1;
        let cam_col = IVec2::new(cam_cube.x, cam_cube.z);
        let mut lod: Vec<DrawRecord> = self
            .lod_draw
            .iter()
            .filter_map(|key| {
                let &(a, count, (y0, y1), skip) = self.lod.get(key)?;
                let o = key.origin();
                let half = Vec3::new(key.size() as f32, (y1 - y0) as f32, key.size() as f32) / 2.0;
                let center = Vec3::new(o.x as f32, y0 as f32, o.y as f32) + half - (cam_cube * 16).as_vec3() - cam_frac;
                if !planes
                    .iter()
                    .all(|pl| pl.dot(center.extend(1.0)) > -half.length())
                {
                    return None;
                }
                let args = DrawIndexedArgs {
                    indices: (count - skip) * 6,
                    instances: 1,
                    first_index: 0,
                    vertex_offset: ((a.offset + skip) * 4) as i32,
                    first_instance: 0,
                };
                let cols =
                    (o.div_euclid(IVec2::splat(16)) - cam_col, (o + IVec2::splat(key.size() - 1)).div_euclid(IVec2::splat(16)) - cam_col);
                let masked = cols.0.x <= mask_half && cols.1.x >= -mask_half && cols.0.y <= mask_half && cols.1.y >= -mask_half;
                // Its corner farthest across the ground, toward where distant terrain ends.
                let far = glam::Vec2::new(center.x.abs() + half.x, center.z.abs() + half.z).length();
                let kind = if masked || !open {
                    3
                } else if end && far > end_from {
                    7
                } else {
                    6
                };
                Some((kind, 0, a.page, args, [o.x / 16 - cam_cube.x, -cam_cube.y, o.y / 16 - cam_cube.z, 0], center))
            })
            .collect();
        lod.sort_by_key(|d| (d.0, d.2, d.5.length_squared().to_bits()));
        let lods = lod.len() as u32;
        let n = opaque + lods + trans.len() as u32;
        let (gpu_draws, gpu_faces) = self.table.drawn_last(gpu, frame.slot);
        self.drawn = n + gpu_draws;
        self.faces_drawn = faces_drawn + gpu_faces;
        let mut runs = std::mem::take(&mut self.runs);
        runs.clear();
        let obuf = crate::gpu_layout::frame_buffer(gpu, &mut self.origins, frame.slot, n, 16, usage::STORAGE, "origins");
        // Then, for temporal anti-aliasing's motion, two vec4 a structure after them all: where it
        // was last frame (camera relative as now) and how it was turned; their count in entry 0's
        // last slot, which the world's own cubes never read.
        let fbuf = crate::gpu_layout::frame_buffer(
            gpu,
            &mut self.offsets,
            frame.slot,
            (picks.len() as u32 + 1) * 5,
            16,
            usage::STORAGE,
            "frame offsets",
        );
        // Three vec4 a structure: its offset and light, its rotation (x, y, z, w) and its pivot
        // (chiacraft: `world.wgsl`'s `finish` turns a leaning one's vertices about it).
        let offs = gpu.mapped(fbuf);
        let put = |offs: &mut [u8], k: usize, o: Vec3, light: u32, lean: Option<Lean>| {
            let (pivot, rot) = lean.unwrap_or((Vec3::ZERO, glam::Quat::IDENTITY));
            let at = k * 48;
            offs[at..at + 12].copy_from_slice(bytemuck::cast_slice(&o.to_array()));
            offs[at + 12..at + 16].copy_from_slice(&light.to_le_bytes());
            offs[at + 16..at + 32].copy_from_slice(bytemuck::cast_slice(&rot.to_array()));
            offs[at + 32..at + 44].copy_from_slice(bytemuck::cast_slice(&pivot.to_array()));
            // 1 for a leaning one: lit as in the open (`on_body`).
            let leaning: f32 = if lean.is_some() { 1.0 } else { 0.0 };
            offs[at + 44..at + 48].copy_from_slice(&leaning.to_le_bytes());
        };
        put(offs, 0, Vec3::ZERO, 0, None);
        for (k, &(o, light, lean, _)) in picks.iter().enumerate() {
            put(offs, k + 1, o, light, lean);
        }
        offs[44..48].copy_from_slice(&(picks.len() as f32).to_le_bytes());
        let base = (picks.len() + 1) * 48;
        offs[base..base + 32].copy_from_slice(bytemuck::cast_slice(&[0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]));
        let origin = (cam_cube * 16).as_dvec3();
        for (k, &(o, _, lean, f)) in picks.iter().enumerate() {
            let (was, was_lean) = self
                .structures_before
                .get(&frames[f].0)
                .copied()
                .unwrap_or((frames[f].1, lean));
            let o = if self.structures_before.contains_key(&frames[f].0) { (was - origin).as_vec3() } else { o };
            let rot = was_lean.map_or(glam::Quat::IDENTITY, |l| l.1);
            let at = base + (k + 1) * 32;
            offs[at..at + 16].copy_from_slice(bytemuck::cast_slice(&o.extend(0.0).to_array()));
            offs[at + 16..at + 32].copy_from_slice(bytemuck::cast_slice(&rot.to_array()));
        }
        self.structures_before.clear();
        self.structures_before.extend(
            frames
                .iter()
                .enumerate()
                .map(|(k, f)| (f.0, (f.1, lean_of(k)))),
        );
        let ibuf =
            crate::gpu_layout::frame_buffer(gpu, &mut self.indirect, frame.slot, n, 20, usage::INDIRECT | usage::STORAGE, "indirect");
        let origins = Mapped(gpu.mapped(obuf)[..n as usize * 16].as_mut_ptr());
        let indirect = Mapped(gpu.mapped(ibuf)[..n as usize * 20].as_mut_ptr());
        let put = |k: u32, d: &DrawRecord| {
            let mut a = d.3;
            a.first_instance = k;
            // SAFETY: both buffers hold `n` records and every k below `n` is written once.
            unsafe {
                std::ptr::copy_nonoverlapping(bytemuck::bytes_of(&d.4).as_ptr(), origins.at(k as usize * 16), 16);
                std::ptr::copy_nonoverlapping(bytemuck::bytes_of(&a).as_ptr(), indirect.at(k as usize * 20), 20);
            }
        };
        let span = crate::diag::span("world/write");
        seen_parts
            .par_iter()
            .zip(starts.par_iter_mut())
            .with_min_len(split)
            .for_each(|(p, at)| {
                for d in p.iter().filter(|d| d.1 < 2) {
                    let b = bucket(d);
                    put(at[b], d);
                    at[b] += 1;
                }
            });
        drop(span);
        for (j, d) in lod.iter().enumerate() {
            put(opaque + j as u32, d);
        }
        for (j, d) in trans.iter().enumerate() {
            put(opaque + lods + j as u32, d);
        }
        let mut start = 0;
        for (b, &t) in totals.iter().enumerate() {
            if t > 0 {
                // Kinds 4 and 5: faces and quads fading.
                let fading = b / pages / 4;
                runs.push(((b / pages % 2 + 4 * fading) as u8, (b / pages / 2 % 2) as u8, (b % pages) as u32, start, t));
            }
            start += t;
        }
        for (j, d) in lod.iter().enumerate() {
            match runs.last_mut() {
                Some(r) if r.0 == d.0 && r.2 == d.2 => r.4 += 1,
                _ => runs.push((d.0, 0, d.2, opaque + j as u32, 1)),
            }
        }
        self.translucent = runs.len();
        for (j, d) in trans.iter().enumerate() {
            // Fading ones (kind 2 and 3) run as kinds 4 and 5, like the opaque ones.
            let kind = if d.0 & 2 != 0 { 4 + (d.0 & 1) } else { d.0 };
            match runs.last_mut() {
                Some(r) if r.1 == 2 && (r.0, r.1, r.2) == (kind, d.1, d.2) => r.4 += 1,
                _ => runs.push((kind, d.1, d.2, opaque + lods + j as u32, 1)),
            }
        }
        (self.parts, self.trans, self.runs) = (parts, trans, runs);
        self.buffers = Some((gbuf, obuf, ibuf, fbuf, scene));
        let moved = last_frame.map_or(Vec3::ZERO, |l| (cam.pos - l.1).as_vec3());
        let (size, levels) = self.occlusion.depth_size();
        let view = crate::gpu_layout::OcclusionView {
            prev: last_frame
                .map_or(Mat4::IDENTITY, |l| l.0)
                .to_cols_array_2d(),
            planes: planes.map(|p| p.to_array()),
            // What goes from there to here is bigger by how far it came, what it might have been behind.
            moved: moved.extend(moved.length() + 0.5).to_array(),
            cam_frac: cam_frac.extend(0.0).to_array(),
            cam_cube: cam_cube.extend(0).to_array(),
            size,
            skip: [opaque, opaque + lods],
            levels,
            hiz: last_frame.is_some() as u32,
            pad: [0; 2],
        };
        let vbuf = crate::gpu_layout::frame_buffer(
            gpu,
            &mut self.table.views,
            frame.slot,
            1,
            size_of::<crate::gpu_layout::OcclusionView>() as u64,
            usage::STORAGE,
            "occlusion view",
        );
        gpu.mapped(vbuf)[..size_of::<crate::gpu_layout::OcclusionView>()].copy_from_slice(bytemuck::bytes_of(&view));
        let depths = self.occlusion.depths();
        if last_frame.is_some() && n > 0 {
            self.occlusion.test(early, vbuf, obuf, ibuf, n);
        }
        if let Some(pipe) = self.occlusion.cull_pipe() {
            self.table.cull(gpu, early, frame.slot, pipe, vbuf, depths);
        }
        let groups = self.table.groups();
        // The counts are two frames old: after a big turn or move since, they may be far too few,
        // so that frame draws every record (as without the cap).
        let now = (cam.forward(), cam.pos);
        let turned = self.table.looks[1].is_some_and(|(f, p)| f.dot(now.0) < 0.985 || p.distance(now.1) > 4.0);
        self.table.looks = [Some(now), self.table.looks[0]];
        let capped = !gpu.indirect_count() && cap_draws() && !turned;
        let mut issued = (0u64, 0u64);
        // Everything opaque that never discards first, then all that may (cutout, the fade band,
        // distant terrain): a discarding fragment can't hide what is drawn after it, so Apple's
        // hidden-surface removal stopped there, and other GPUs' early depth lost the occluders.
        // The table's groups are layer first, so its opaque ones lead; the CPU's plain opaque runs
        // (meshes with something translucent too) came after the table's cutout before (perf
        // review: the Mac's world pass is 77 % fragment work). `WEFT_OLD_ORDER=1` to compare.
        let plain = |r: &(u8, u8, u32, u32, u32)| r.1 == 0 && r.0 < 2;
        let firsts = if old_order() {
            0
        } else {
            self.runs[..self.translucent]
                .iter()
                .take_while(|r| plain(r))
                .count()
        };
        let mut cpu_opaque_drawn = old_order();
        // Each group's pipeline; split cutout groups (`cutout_split`) all lay their depth first,
        // then shade, so each pixel of leaves is shaded once.
        let mut steps: Vec<(PipelineId, BufferId, usize, bool)> = Vec::with_capacity(groups.len() + 4);
        let mut depth_laid = false;
        for (i, &(kind, layer, page, ..)) in groups.iter().enumerate() {
            let (pipe, buf) = self.pipe_of(kind, layer, page);
            match self.split_of(kind, layer) {
                Some((_, shade)) => {
                    if !depth_laid {
                        depth_laid = true;
                        for (j, &(k, l, pg, ..)) in groups.iter().enumerate().skip(i) {
                            if let Some((depth, _)) = self.split_of(k, l) {
                                steps.push((depth, self.pipe_of(k, l, pg).1, j, false));
                            }
                        }
                    }
                    steps.push((shade, buf, i, true));
                }
                None => steps.push((pipe, buf, i, true)),
            }
        }
        // The path tracer's G-buffer replays the opaque and cutout terrain's draws (`draw_gbuffer`).
        self.gb_draws.clear();
        self.gb_draws_slot = frame.slot;
        for (pipe, buf, i, counted) in steps {
            let (_, layer, _, id, base, max) = groups[i];
            if layer > 0 && !cpu_opaque_drawn {
                self.emit(list, 0..firsts);
                cpu_opaque_drawn = true;
            }
            self.bind_run(list, pipe, buf, self.table.out.as_ref().map_or(obuf, |o| o.origins));
            let Some(o) = self.table.out.as_ref() else { continue };
            let seen = self
                .table
                .seen
                .get(id as usize)
                .copied()
                .flatten()
                .map(|n| n.max(self.table.peak.get(id as usize).copied().unwrap_or(0)));
            let draws = if capped { capped_draws(seen, max) } else { max };
            if counted {
                let (kind, layer, page, ..) = groups[i];
                if layer < 2 && matches!(kind, 0 | 1 | 4 | 5) {
                    self.gb_draws
                        .push((kind as u8, layer as u8, page as u32, id as u32, base as u32, draws as u32));
                }
            }
            if counted {
                issued = (issued.0 + draws as u64, issued.1 + max as u64);
            }
            list.push(Cmd::DrawIndexedIndirectCount {
                buffer: o.draws,
                offset: base as u64 * 20,
                count_buffer: self.table.buckets[frame.slot].0,
                count_offset: id as u64 * 8,
                max: draws,
            });
        }
        self.table.log_draws(gpu.indirect_count(), capped, issued);
        if !cpu_opaque_drawn {
            self.emit(list, 0..firsts);
        }
        self.emit(list, firsts..self.translucent);
    }

    /// The translucent part of the last `draw`, after everything that may stand behind it.
    pub fn draw_translucent(&self, list: &mut CommandList) {
        let n = self.runs.len();
        if self.buffers.is_some() && self.translucent < n {
            self.emit(list, self.translucent..n);
        }
    }

    /// Runs `range` of the last `draw`.
    fn emit(&self, list: &mut CommandList, range: std::ops::Range<usize>) {
        let Some((_, obuf, ibuf, _, _)) = self.buffers else { return };
        for &(kind, layer, page, start, count) in &self.runs[range] {
            let (pipe, buf) = self.pipe_of(kind, layer, page);
            if let Some((depth, _)) = self.split_of(kind, layer) {
                self.bind_run(list, depth, buf, obuf);
                list.push(Cmd::DrawIndexedIndirect { buffer: ibuf, offset: start as u64 * 20, count });
            }
            let pipe = self.split_of(kind, layer).map_or(pipe, |s| s.1);
            self.bind_run(list, pipe, buf, obuf);
            list.push(Cmd::DrawIndexedIndirect { buffer: ibuf, offset: start as u64 * 20, count });
        }
    }

    /// The pipeline and records of a run: kind 0 faces, 1 quads, 3 distant terrain; chiacraft's 4 and
    /// 5 faces and quads fading where the cubes end (`fade_pipes`, the plain ones without them).
    fn pipe_of(&self, kind: u8, layer: u8, page: u32) -> (PipelineId, BufferId) {
        match (kind, self.fade_pipes) {
            (0, _) => (self.face_pipes[layer as usize], self.faces.pages[page as usize].0),
            (3, _) => (self.lod_pipe, self.faces.pages[page as usize].0),
            (6, _) => (self.lod_open_pipe.unwrap_or(self.lod_pipe), self.faces.pages[page as usize].0),
            (7, _) => (
                self.lod_end_pipe
                    .or(self.lod_open_pipe)
                    .unwrap_or(self.lod_pipe),
                self.faces.pages[page as usize].0,
            ),
            (4, Some(f)) => (f[layer as usize], self.faces.pages[page as usize].0),
            (5, Some(f)) => (f[3 + layer as usize], self.quads.pages[page as usize].0),
            (4, None) => (self.face_pipes[layer as usize], self.faces.pages[page as usize].0),
            _ => (self.quad_pipes[layer as usize], self.quads.pages[page as usize].0),
        }
    }

    fn bind_run(&self, list: &mut CommandList, pipe: PipelineId, records: BufferId, origins: BufferId) {
        let Some((gbuf, _, _, fbuf, scene)) = self.buffers else { return };
        list.push(Cmd::IndexBuffer(self.quad_index));
        list.pipeline(pipe);
        use crate::gpu_layout::bind::world as b;
        list.bind(b::G, gbuf);
        list.bind(b::ORIGINS, origins);
        list.bind(b::RECORDS, records);
        list.bind(b::BIOMES, self.biomes);
        list.bind(b::TEX_INFO, self.tex_info);
        list.bind(b::FRAMES, self.anim);
        list.bind(b::OFFSETS, fbuf);
        list.bind(b::GLOW_MAP, self.glow);
        list.bind(b::TEX_SHADER, self.tex_shader);
        list.bind(b::SCENE, scene);
        list.bind(b::LOD_MASK, self.lod_mask.0);
        list.bind(b::SOLID, self.solid.buf);
        list.bind(b::MAT_MAP, self.mat);
        if self.rt_compiled
            && let Some(rt) = &self.rt
        {
            use crate::gpu_layout::rt_bind as r;
            list.push(Cmd::BindAccel { binding: r::TLAS, accel: rt.tlas() });
            if let Some((records, table)) = rt.tables() {
                list.bind(r::INSTANCES, table);
                list.bind(r::RECORDS, records);
            }
        }
    }
}

/// Every cube of the world with something opaque or cutout to draw, kept on the GPU and changed only
/// when meshes do: a compute shader makes each frame's draws from it, so the CPU's cost doesn't grow
/// with how many cubes are in view.
struct CubeTable {
    entries: Vec<crate::gpu_layout::CubeEntry>,
    free: Vec<u32>,
    /// Places changed since the last upload.
    dirty: Vec<u32>,
    /// On the GPU, and how many places it has.
    buffer: BufferId,
    capacity: u32,
    /// How many places each bucket (`CubeEntry::bucket`) has.
    per_bucket: Vec<u32>,
    /// Per frame: per bucket how many draws were written and where they start, then the faces.
    buckets: Vec<(BufferId, u32)>,
    views: Vec<(BufferId, u32)>,
    out: Option<TableOut>,
    /// Without `drawIndirectCount` (`Backend::indirect_count`): per bucket how many draws the
    /// culling wrote the last time this frame slot ran, read back before it is reset; `None`
    /// while unknown (a slot's buffer just made). A draw then issues about that many, not all.
    seen: Vec<Option<u32>>,
    /// Per bucket the most `seen` has been lately, falling by 1/64 a frame (about a second to
    /// halve): a count drawn from two frames back was too few after staring into the ground and
    /// looking up, and cubes went missing for a frame or two, the world black behind them (Ronald,
    /// 11-06).
    peak: Vec<u32>,
    /// The camera's look and place the last two frames: a big change since the counts were made
    /// draws every record that frame.
    looks: [Option<(Vec3, DVec3)>; 2],
    /// What the last frame issued and the culling had written: (draws issued, all records,
    /// written), for the once-a-second log line.
    issued: (u64, u64, u64, u32),
    logged: Option<std::time::Instant>,
}

/// What `cull` writes: draws and where each one's cube is. One of each serves all frames in flight,
/// they run in order on the GPU.
struct TableOut {
    draws: BufferId,
    origins: BufferId,
    capacity: u32,
}

/// Direction runs of a cube: as many sides as face a camera outside it, in order.
const DRAWS_PER_FACES: u32 = 3;

/// Where the faces' counter is in the buckets' buffer, in 32-bit words: after the `buckets`
/// buckets' 8 bytes each, at a 256-byte boundary. It is bound on its own, and a storage buffer's
/// offset must be a multiple of the GPU's `minStorageBufferOffsetAlignment` (at most 256): right
/// after the buckets it was only when their count happened to suit (a validation error otherwise).
fn faces_word(buckets: usize) -> usize {
    (buckets * 8).next_multiple_of(256) / 4
}

/// How many draw records `cull` may write for buckets with `per_bucket` places: what `TableOut`
/// holds (at least 1).
fn records_of(per_bucket: &[u32]) -> u32 {
    per_bucket
        .iter()
        .enumerate()
        .map(|(id, &n)| n * if id & 2 == 0 { DRAWS_PER_FACES } else { 1 })
        .sum::<u32>()
        .max(1)
}

/// The draw groups of buckets with `per_bucket` places: (kind, layer, page, bucket, first record,
/// most records), in drawing order. Each one's records are the bucket's share of `records_of`.
fn groups_of(per_bucket: &[u32]) -> Vec<(u8, u8, u32, u32, u32, u32)> {
    let mut base = 0;
    let mut out = Vec::new();
    for (id, &n) in per_bucket.iter().enumerate() {
        let id = id as u32;
        let (page, kind, layer) = (id / 4, (id >> 1 & 1) as u8, (id & 1) as u8);
        let max = n * if kind == 0 { DRAWS_PER_FACES } else { 1 };
        if n > 0 {
            out.push((kind, layer, page, id, base, max));
        }
        base += max;
    }
    out.sort_by_key(|g| (g.1, g.0, g.2));
    out
}

/// Whether a GPU without `drawIndirectCount` draws about as many records as the culling wrote
/// last time (`capped_draws`), not all of them: on by default, off with `--no-indirect-cap`
/// (`WEFT_NO_INDIRECT_CAP`), to compare.
fn cap_draws() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WEFT_NO_INDIRECT_CAP").is_none())
}

/// How many of a group's `max` records to draw when the GPU can't read the count: what the
/// culling wrote the last time this frame slot ran, `seen`, with room for half again as many and
/// 256 more (turning, or coming nearer to more), all while it isn't known. A record past the count
/// is zeroed and draws nothing, so drawing more is only slower; drawing fewer would leave cubes
/// out for a frame, so the room is generous.
fn capped_draws(seen: Option<u32>, max: u32) -> u32 {
    seen.map_or(max, |n| n.saturating_add(n / 2).saturating_add(256).min(max))
}

impl CubeTable {
    fn new(gpu: &mut dyn Backend) -> CubeTable {
        CubeTable {
            entries: Vec::new(),
            free: Vec::new(),
            dirty: Vec::new(),
            buffer: gpu.create_buffer(32 * 1024, usage::STORAGE, "cube table"),
            capacity: 1024,
            per_bucket: Vec::new(),
            buckets: Vec::new(),
            views: Vec::new(),
            out: None,
            seen: Vec::new(),
            peak: Vec::new(),
            looks: [None; 2],
            issued: (0, 0, 0, 0),
            logged: None,
        }
    }

    fn put(&mut self, cube: IVec3, bucket: u32, start: u32, counts: [u32; 3]) -> u32 {
        let e = crate::gpu_layout::CubeEntry { cube: cube.to_array(), bucket, start, counts };
        let slot = self.free.pop().unwrap_or_else(|| {
            self.entries.push(e);
            self.entries.len() as u32 - 1
        });
        self.entries[slot as usize] = e;
        self.dirty.push(slot);
        if self.per_bucket.len() <= bucket as usize {
            self.per_bucket.resize(bucket as usize + 1, 0);
        }
        self.per_bucket[bucket as usize] += 1;
        slot
    }

    /// A mesh's opaque and cutout layers: their places.
    fn add(
        &mut self,
        cube: IVec3,
        faces: Option<Alloc>,
        face_counts: &[[u32; 6]; 3],
        quads: Option<Alloc>,
        quad_counts: &[u32; 3],
    ) -> [u32; 4] {
        let mut slots = [u32::MAX; 4];
        if let Some(a) = faces {
            let mut start = a.offset;
            for layer in 0..2 {
                let c = face_counts[layer];
                let total: u32 = c.iter().sum();
                if total > 0 {
                    let counts = [c[0] | c[1] << 16, c[2] | c[3] << 16, c[4] | c[5] << 16];
                    slots[layer] = self.put(cube, a.page * 4 + layer as u32, start, counts);
                }
                start += total;
            }
        }
        if let Some(a) = quads {
            let mut start = a.offset;
            for layer in 0..2 {
                let n = quad_counts[layer];
                if n > 0 {
                    slots[2 + layer] = self.put(cube, a.page * 4 + 2 + layer as u32, start, [n, 0, 0]);
                }
                start += n;
            }
        }
        slots
    }

    fn remove(&mut self, slots: [u32; 4]) {
        for slot in slots.into_iter().filter(|s| *s != u32::MAX) {
            let e = &mut self.entries[slot as usize];
            self.per_bucket[e.bucket as usize] -= 1;
            e.bucket = u32::MAX;
            self.free.push(slot);
            self.dirty.push(slot);
        }
    }

    /// Brings the GPU's copy up to date, in runs of neighbours.
    fn upload(&mut self, gpu: &mut dyn Backend) {
        let n = self.entries.len() as u32;
        if n > self.capacity {
            gpu.destroy_buffer(self.buffer);
            self.capacity = n.next_power_of_two();
            self.buffer = gpu.create_buffer(self.capacity as u64 * 32, usage::STORAGE, "cube table");
            gpu.write_buffer(self.buffer, 0, bytemuck::cast_slice(&self.entries));
            self.dirty.clear();
            return;
        }
        self.dirty.sort_unstable();
        self.dirty.dedup();
        let mut i = 0;
        while i < self.dirty.len() {
            let first = self.dirty[i];
            let mut end = i + 1;
            while end < self.dirty.len() && self.dirty[end] == first + (end - i) as u32 {
                end += 1;
            }
            let run = &self.entries[first as usize..first as usize + (end - i)];
            gpu.write_buffer(self.buffer, first as u64 * 32, bytemuck::cast_slice(run));
            i = end;
        }
        self.dirty.clear();
    }

    /// What this frame slot's last use drew, for showing: draws and faces.
    fn drawn_last(&mut self, gpu: &mut dyn Backend, slot: usize) -> (u32, u64) {
        let Some(&(b, _)) = self.buckets.get(slot) else { return (0, 0) };
        let words: &[u32] = bytemuck::cast_slice(gpu.mapped(b));
        let n = self.per_bucket.len().min(words.len() / 2);
        // The GPU's counts, read back: a frame's worth may be stale or half written.
        let draws = words[..n * 2]
            .chunks_exact(2)
            .fold(0u32, |sum, c| sum.saturating_add(c[0]));
        (
            draws,
            words
                .get(faces_word(self.per_bucket.len()))
                .copied()
                .unwrap_or(0) as u64,
        )
    }

    /// Makes this frame's draws. Before the world's pass.
    fn cull(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        slot: usize,
        pipe: PipelineId,
        view: BufferId,
        depths: Option<BufferId>,
    ) {
        self.upload(gpu);
        let needed = records_of(&self.per_bucket);
        if self.out.as_ref().is_none_or(|o| o.capacity < needed) {
            if let Some(o) = self.out.take() {
                gpu.destroy_buffer(o.draws);
                gpu.destroy_buffer(o.origins);
            }
            let capacity = needed.next_power_of_two().max(4096);
            self.out = Some(TableOut {
                draws: gpu.create_buffer(capacity as u64 * 20, usage::INDIRECT | usage::STORAGE, "table draws"),
                origins: gpu.create_buffer(capacity as u64 * 16, usage::STORAGE, "table origins"),
                capacity,
            });
        }
        let Some(o) = &self.out else { return };
        let before = self.buckets.get(slot).map(|b| b.0);
        let buckets = crate::gpu_layout::frame_buffer(
            gpu,
            &mut self.buckets,
            slot,
            faces_word(self.per_bucket.len()) as u32 / 2 + 1,
            8,
            usage::STORAGE | usage::INDIRECT,
            "table buckets",
        );
        {
            let words: &mut [u32] = bytemuck::cast_slice_mut(gpu.mapped(buckets));
            // What this slot's culling wrote last time (its frame is done on the GPU by now): a
            // buffer made just now holds nothing yet.
            let same = before == Some(buckets);
            self.seen.resize(self.per_bucket.len(), None);
            self.peak.resize(self.per_bucket.len(), 0);
            for (id, (seen, peak)) in self.seen.iter_mut().zip(self.peak.iter_mut()).enumerate() {
                *seen = same.then(|| words[id * 2]);
                *peak = (*peak - *peak / 64).max(seen.unwrap_or(0));
            }
            let mut base = 0;
            for (id, &n) in self.per_bucket.iter().enumerate() {
                words[id * 2] = 0;
                words[id * 2 + 1] = base;
                base += n * if id & 2 == 0 { DRAWS_PER_FACES } else { 1 };
            }
            words[faces_word(self.per_bucket.len())] = 0;
        }
        use crate::gpu_layout::bind::occlusion as b;
        if !gpu.indirect_count() {
            // Its draws then draw every record (`Backend::indirect_count`): the ones the culling
            // doesn't write this frame have to draw nothing, so all start zeroed. After the last
            // frame's draws read them.
            list.push(Cmd::Barrier);
            list.push(Cmd::FillBuffer { buffer: o.draws, offset: 0, size: o.capacity as u64 * 20, value: 0 });
        }
        list.push(Cmd::Barrier);
        list.pipeline(pipe);
        list.bind(b::VIEW, view);
        list.bind(b::TABLE, self.buffer);
        list.bind(b::BUCKETS, buckets);
        list.bind(b::OUT_DRAWS, o.draws);
        list.bind(b::OUT_ORIGINS, o.origins);
        // The faces' counter is the one after the last bucket.
        list.bind_range(b::FACES_DRAWN, buckets, faces_word(self.per_bucket.len()) as u64 * 4, 8);
        // Its use is decided in the shader, but all a shader mentions has to be bound.
        list.bind(b::DEPTHS, depths.unwrap_or(self.buffer));
        let count = self.entries.len() as u32;
        list.push_constants(&crate::gpu_layout::OcclusionArgs { size: [0; 2], src: 0, dst: 0, count, pad: 0 });
        list.push(Cmd::Dispatch { x: count.div_ceil(256), y: 1, z: 1 });
        list.push(Cmd::Barrier);
    }

    /// The buckets with anything in them, in drawing order, opaque before cutout: kind, layer, page,
    /// bucket, where its draws start and how many it may have.
    fn groups(&self) -> Vec<(u8, u8, u32, u32, u32, u32)> {
        groups_of(&self.per_bucket)
    }

    /// Adds a frame's indirect draws, and once a second (`--log frame=debug`) says how many
    /// records the terrain's draws issued, of all there are and of what the culling wrote.
    fn log_draws(&mut self, count: bool, capped: bool, (issued, all): (u64, u64)) {
        let written: u64 = self.seen.iter().map(|s| s.unwrap_or(0) as u64).sum();
        let t = &mut self.issued;
        (t.0, t.1, t.2, t.3) = (t.0 + issued, t.1 + all, t.2 + written, t.3 + 1);
        let now = std::time::Instant::now();
        let since = *self.logged.get_or_insert(now);
        if now - since < std::time::Duration::from_secs(1) {
            return;
        }
        let (i, a, w, n) = std::mem::take(&mut self.issued);
        let n = n.max(1) as u64;
        let how = if count {
            "count from the GPU"
        } else if capped {
            "capped at last time's"
        } else {
            "every record"
        };
        log::debug!(target: "frame", "terrain draws: {} records issued a frame of {}, {} written ({how})", i / n, a / n, w / n);
        self.logged = Some(now);
    }

    fn destroy(&mut self, gpu: &mut dyn Backend) {
        gpu.destroy_buffer(self.buffer);
        for (b, _) in self.buckets.drain(..).chain(self.views.drain(..)) {
            gpu.destroy_buffer(b);
        }
        if let Some(o) = self.out.take() {
            gpu.destroy_buffer(o.draws);
            gpu.destroy_buffer(o.origins);
        }
    }
}

/// How far the camera may go in a frame for what the last one saw to still say anything.
const OCCLUSION_MOVE: f64 = 8.0;

/// Occlusion culling on the GPU, by the depth of the frame before: the cubes it shows to be behind
/// something get no draw. A cube wrongly left out is back the frame after, since it then isn't in
/// that depth and nothing there hides it any longer.
pub struct Occlusion {
    /// The setting.
    pub on: bool,
    pipes: Option<[PipelineId; 3]>,
    /// Depth and its smaller copies; its size; how many levels.
    depths: Option<(BufferId, [u32; 2], u32)>,
    /// This frame's camera, and the last one's if `depths` has its depth.
    now: Option<(Mat4, DVec3)>,
    last: Option<(Mat4, DVec3)>,
}

impl Occlusion {
    /// Without its shader nothing opaque would get drawn, so it is a reason not to start.
    fn new(gpu: &mut dyn Backend, assets: &crate::assets::Assets) -> Result<Occlusion, String> {
        Ok(Occlusion { on: true, pipes: Some(Self::pipes(gpu, assets)?), depths: None, now: None, last: None })
    }

    fn pipes(gpu: &mut dyn Backend, assets: &crate::assets::Assets) -> Result<[PipelineId; 3], String> {
        // Not a shader packs may replace: it decides what is drawn, not how it looks.
        let s = crate::packs::engine_shader(assets, "occlusion.wgsl", &crate::gpu_layout::occlusion_prelude());
        s.map_err(|e| e.0).and_then(|s| {
            let mut pipe = |entry| {
                gpu.create_compute(&ComputeDesc { label: "occlusion", shader: &s, entry })
                    .map_err(|e| e.0)
            };
            Ok([pipe("down")?, pipe("test")?, pipe("cull")?])
        })
    }

    fn reload(&mut self, gpu: &mut dyn Backend, assets: &crate::assets::Assets) {
        match Self::pipes(gpu, assets) {
            Ok(new) => {
                for p in self.pipes.replace(new).into_iter().flatten() {
                    gpu.destroy_pipeline(p);
                }
            }
            Err(e) => log::error!("occlusion shader: {e}"),
        }
    }

    fn destroy(&mut self, gpu: &mut dyn Backend) {
        for p in self.pipes.take().into_iter().flatten() {
            gpu.destroy_pipeline(p);
        }
        if let Some((b, ..)) = self.depths.take() {
            gpu.destroy_buffer(b);
        }
    }

    /// What this frame may go by: the last frame's camera, unless it went too far since.
    fn next_frame(&mut self, view_proj: Mat4, pos: DVec3) -> Option<(Mat4, DVec3)> {
        self.now = Some((view_proj, pos));
        self.last
            .filter(|l| self.on && (l.1 - pos).length() < OCCLUSION_MOVE)
    }

    /// The size of the depth kept and how many levels it has.
    fn depth_size(&self) -> ([u32; 2], u32) {
        self.depths.map_or(([1; 2], 1), |d| (d.1, d.2))
    }

    fn depths(&self) -> Option<BufferId> {
        self.depths.map(|d| d.0)
    }

    fn cull_pipe(&self) -> Option<PipelineId> {
        self.pipes.map(|p| p[2])
    }

    /// Takes what the last frame shows to be hidden out of the draws the CPU made.
    fn test(&self, list: &mut CommandList, view: BufferId, origins: BufferId, draws: BufferId, count: u32) {
        let (Some([_, pipe, _]), Some((depths, ..))) = (self.pipes, self.depths) else { return };
        use crate::gpu_layout::bind::occlusion as b;
        list.push(Cmd::Barrier);
        list.pipeline(pipe);
        list.bind(b::VIEW, view);
        list.bind(b::DEPTHS, depths);
        list.bind(b::ORIGINS, origins);
        list.bind(b::DRAWS, draws);
        list.push_constants(&crate::gpu_layout::OcclusionArgs { size: [0; 2], src: 0, dst: 0, count, pad: 0 });
        list.push(Cmd::Dispatch { x: count.div_ceil(256), y: 1, z: 1 });
        list.push(Cmd::Barrier);
    }

    /// Takes this frame's depth for the next frame to go by. After the world's pass.
    pub fn take_depth(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, depth: TextureId, size: [u32; 2]) {
        let Some([down, ..]) = self.pipes.filter(|_| self.on) else {
            self.last = None;
            return;
        };
        // The levels under the depth (level 0, the depth texture, is read where it is: copying it,
        // 31 MB at 3456 x 2234, and a dispatch behind a full barrier per level were a dozen Metal
        // encoders a frame on the Mac; perf review #14).
        let (texels, levels) = pyramid(size);
        let buffer = match self.depths.filter(|d| d.1 == size) {
            Some(d) => d.0,
            None => {
                if let Some((old, ..)) = self.depths.take() {
                    gpu.destroy_buffer(old);
                }
                let b = gpu.create_buffer(texels.max(1) * 4, usage::STORAGE, "depth levels");
                self.depths = Some((b, size, levels));
                b
            }
        };
        list.push(Cmd::Barrier);
        list.pipeline(down);
        list.bind(crate::gpu_layout::bind::occlusion::DEPTHS, buffer);
        // Each dispatch makes up to `PER_DISPATCH` levels from the last one made (the depth for the
        // first), a 32 x 32 tile halving in workgroup memory.
        let (mut s, mut src, mut from_texture, mut made) = (size, 0u32, true, 1u32);
        while made < levels {
            let count = PER_DISPATCH.min(levels - made);
            let d = s.map(|v| v.div_ceil(2));
            let dst = if from_texture { 0 } else { src + s[0] * s[1] };
            let pad = if from_texture { gpu.texture_index(depth) + 1 } else { 0 };
            list.push_constants(&crate::gpu_layout::OcclusionArgs { size: s, src, dst, count, pad });
            list.push(Cmd::Dispatch { x: d[0].div_ceil(32), y: d[1].div_ceil(32), z: 1 });
            list.push(Cmd::Barrier);
            // The last level this one made, and where it is.
            let (mut at, mut ls) = (dst, d);
            for _ in 1..count {
                at += ls[0] * ls[1];
                ls = ls.map(|v| v.div_ceil(2));
            }
            (s, src, from_texture, made) = (ls, at, false, made + count);
        }
        self.last = self.now.take();
    }
}

/// `WEFT_OLD_ORDER=1`: the world pass in the order before opaque-first (the table's groups, then
/// the CPU's runs), to compare.
/// Cutout blocks drawn depth first, then shaded with no discard (`cutout_split`): on Android, where
/// GPUs are tiled (`WEFT_CUTOUT_PREPASS=0/1` to compare or to try it elsewhere).
fn cutout_prepass() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match std::env::var("WEFT_CUTOUT_PREPASS").as_deref() {
        Ok("0") => false,
        Ok(_) => true,
        Err(_) => cfg!(target_os = "android"),
    })
}

fn old_order() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WEFT_OLD_ORDER").is_some())
}

/// Levels a dispatch of `occlusion.wgsl`'s `down` makes: a 32 x 32 tile halved down to 1.
const PER_DISPATCH: u32 = 6;

/// The depth pyramid of a depth of `size`: the texels of levels 1 and on (what `depths` holds), and
/// how many levels there are with level 0 (down to 1 x 1).
fn pyramid(size: [u32; 2]) -> (u64, u32) {
    let (mut s, mut texels, mut levels) = (size, 0u64, 1u32);
    while s[0] > 1 || s[1] > 1 {
        s = s.map(|v| v.div_ceil(2));
        texels += s[0] as u64 * s[1] as u64;
        levels += 1;
    }
    (texels, levels)
}

/// Left, right, bottom, top planes; the reverse-Z infinite projection has no useful far plane.
pub fn frustum_planes(m: Mat4) -> [Vec4; 4] {
    let r = |i: usize| m.row(i);
    let norm = |p: Vec4| p / p.truncate().length();
    [norm(r(3) + r(0)), norm(r(3) - r(0)), norm(r(3) + r(1)), norm(r(3) - r(1))]
}

/// `frustum_planes` and the depth range's two (reversed depth, 0 to 1): a closed box, for the
/// shadow maps' orthographic light.
fn box_planes(m: Mat4) -> [Vec4; 6] {
    let r = |i: usize| m.row(i);
    let norm = |p: Vec4| p / p.truncate().length();
    let [a, b, c, d] = frustum_planes(m);
    [a, b, c, d, norm(r(2)), norm(r(3) - r(2))]
}

#[cfg(test)]
mod indirect_count_tests {
    use super::*;

    #[test]
    fn the_faces_counter_is_aligned_after_the_buckets() {
        for n in [0, 1, 4, 9, 31, 32, 33, 100] {
            let at = faces_word(n) * 4;
            assert_eq!(at % 256, 0, "{n} buckets");
            assert!(at >= n * 8, "{n} buckets: it would overlap them");
        }
    }

    #[test]
    fn the_pyramid_has_every_level_but_the_depth() {
        // 4 x 4: levels 2 x 2 and 1 x 1 kept, three levels with the depth.
        assert_eq!(pyramid([4, 4]), (5, 3));
        assert_eq!(pyramid([1, 1]), (0, 1));
        // Odd sizes round up, as the shader's (s + 1) / 2.
        assert_eq!(pyramid([5, 3]), (3 * 2 + 2 + 1, 4));
        let (_, levels) = pyramid([3456, 2234]);
        assert_eq!(levels, 13);
    }

    #[test]
    fn capped_draws_keep_room_and_stay_in_the_group() {
        assert_eq!(capped_draws(None, 5000), 5000, "unknown: all");
        assert_eq!(capped_draws(Some(0), 5000), 256);
        assert_eq!(capped_draws(Some(1000), 5000), 1756);
        assert_eq!(capped_draws(Some(4000), 5000), 5000, "never past the group");
        assert_eq!(capped_draws(Some(u32::MAX), 7), 7);
    }

    /// Without `drawIndirectCount` every group draws all its records, so each must lie in the
    /// zeroed `TableOut` (`records_of`) and no two may overlap.
    #[test]
    fn every_drawn_record_is_in_the_zeroed_buffer() {
        for per_bucket in [vec![0], vec![5], vec![3, 0, 7, 2, 0, 0, 9, 1], vec![1; 13]] {
            let all = records_of(&per_bucket);
            let mut groups = groups_of(&per_bucket);
            groups.sort_by_key(|g| g.4);
            let mut end = 0;
            for &(.., base, max) in &groups {
                assert!(base >= end, "{per_bucket:?}: groups overlap");
                end = base + max;
                assert!(end <= all, "{per_bucket:?}: a group draws past the buffer ({end} > {all})");
            }
        }
    }
}

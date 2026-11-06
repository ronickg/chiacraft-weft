//! Backend-neutral rendering interface. Only crates under `backends/` talk to a graphics API.
//!
//! Model: no vertex buffers (shaders pull from storage buffers), all sampled textures live in a
//! global bindless table (group 0), per-draw buffers are group 1, small per-draw data may go
//! through push constants (`var<immediate>`). Shaders are WGSL, prefixed with [`PRELUDE`].

mod shader;

pub use shader::{BufferKind, Shader, ShaderError};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineId(pub u32);
/// A ray-tracing acceleration structure ([`Backend::ray_query`]): a bottom-level one over
/// triangles ([`Backend::create_blas`]) or a top-level one over instances of those
/// ([`Backend::create_tlas`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccelId(pub u32);

/// Global WGSL declarations every shader can use. Texture handles index into these arrays.
/// The `mipped` samplers clamp too: textures are never tiled, and wrapping would blend the edge
/// texels of a face with the opposite edge (seams between overlays and cutout blocks).
pub const PRELUDE: &str = r#"enable wgpu_binding_array;
@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 65536>;
@group(0) @binding(1) var nearest_clamp: sampler;
@group(0) @binding(2) var linear_clamp: sampler;
@group(0) @binding(3) var nearest_mipped: sampler;
@group(0) @binding(4) var linear_mipped: sampler;
"#;

/// Every texture of every kind is its own entry (blocks, items, particles, skins, GUI, each
/// animation frame): big packs have thousands; face records keep 16-bit indices.
pub const MAX_TEXTURES: u32 = 65536;

pub mod usage {
    pub const STORAGE: u32 = 1;
    pub const INDIRECT: u32 = 2;
    pub const UNIFORM: u32 = 4;
    /// Persistently mapped and written directly by the CPU. The caller must not overwrite a range the
    /// GPU may still read (use one range per frame in flight).
    pub const HOST: u32 = 8;
    pub const INDEX: u32 = 16;
    /// Written by the GPU and read by the CPU through `mapped`, once the frame that wrote it is
    /// done (the next time its frame slot comes round).
    pub const READBACK: u32 = 32;
    /// Read by acceleration structure builds ([`BlasGeometry`]). Only with [`Backend::ray_query`].
    pub const ACCEL_INPUT: u32 = 64;
}

/// One set of triangles of a bottom-level acceleration structure.
#[derive(Clone, Copy, Debug)]
pub struct BlasGeometry {
    /// Positions, three `f32` each, `vertex_stride` bytes apart from `vertex_offset`
    /// (a buffer made with [`usage::ACCEL_INPUT`]); `vertices` of them.
    pub vertices: BufferId,
    pub vertex_offset: u64,
    pub vertex_stride: u32,
    pub vertex_count: u32,
    /// 32-bit indices, three a triangle, into those positions, from `index_offset`.
    pub indices: BufferId,
    pub index_offset: u64,
    pub triangles: u32,
    /// false: a ray query hands each hit to the shader as a candidate, to accept or pass through
    /// (see-through texels); true: every hit counts.
    pub opaque: bool,
}

/// One instance of a bottom-level acceleration structure in a top-level one.
#[derive(Clone, Copy, Debug)]
pub struct TlasInstance {
    /// Row-major 3x4, from the BLAS's space to the TLAS's.
    pub transform: [[f32; 4]; 3],
    /// What a hit reports as `instance_custom_data` (24 bits).
    pub custom: u32,
    /// A ray skips it unless its cull mask shares a bit with this.
    pub mask: u8,
    /// Every hit on it counts, its see-through geometry's too, without asking the shader.
    pub opaque: bool,
    pub blas: AccelId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Rgba8Srgb,
    Rgba8Unorm,
    R8Unorm,
    Rgba16Float,
    Depth32,
    /// Whatever the presentation surface uses.
    Surface,
}

impl Format {
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            Format::R8Unorm => 1,
            Format::Rgba16Float => 8,
            _ => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub mips: u32,
    pub format: Format,
    pub render_target: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    None,
    Alpha,
    Premultiplied,
    Additive,
    /// The color attachment left as it is (write mask 0): a depth pass inside a pass with color.
    NoColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compare {
    Always,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    /// Only where the depth is already this fragment's (a second pass over a depth pre-pass).
    Equal,
}

#[derive(Clone, Copy, Debug)]
pub struct DepthState {
    pub write: bool,
    pub compare: Compare,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topology {
    Triangles,
    Lines,
}

pub struct PipelineDesc<'a> {
    pub label: &'a str,
    pub shader: &'a Shader,
    pub vertex: &'a str,
    pub fragment: Option<&'a str>,
    pub color: Option<Format>,
    pub depth: Option<DepthState>,
    pub blend: Blend,
    pub cull_back: bool,
    pub topology: Topology,
    /// Drawn in a pass with a motion image as well ([`Cmd::BeginPass`]'s `motion`,
    /// [`MOTION_FORMAT`]): the fragment entry writes it at `@location(1)` if it has that output,
    /// and leaves it as it is (write mask 0) if not, so shaders without motion keep working.
    pub motion: bool,
}

/// The motion image's format (`PipelineDesc::motion`): xy where the pixel was last frame minus
/// where it is now, in UV (0..1 across the image, y down), z the depth of the surface that wrote
/// it (reverse Z), w the share of the pixel that is not to be gathered over frames (reactive).
pub const MOTION_FORMAT: Format = Format::Rgba16Float;

pub struct ComputeDesc<'a> {
    pub label: &'a str,
    pub shader: &'a Shader,
    pub entry: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Surface,
    Texture(TextureId),
}

#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    BeginPass {
        color: Option<(Target, Option<[f32; 4]>)>,
        depth: Option<(TextureId, Option<f32>)>,
        /// A second color image, at `@location(1)` ([`MOTION_FORMAT`]); every pipeline drawn in
        /// the pass must be made with `PipelineDesc::motion`.
        motion: Option<(TextureId, Option<[f32; 4]>)>,
    },
    EndPass,
    SetPipeline(PipelineId),
    BindBuffer {
        binding: u32,
        buffer: BufferId,
        offset: u64,
        size: u64,
    },
    /// Range inside [`CommandList::data`].
    PushConstants {
        start: u32,
        len: u32,
    },
    Scissor {
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    },
    Draw {
        vertices: u32,
        instances: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndirect {
        buffer: BufferId,
        offset: u64,
        count: u32,
    },
    /// 32-bit indices for the following indexed draws.
    IndexBuffer(BufferId),
    /// `count` [`DrawIndexedArgs`] records.
    DrawIndexedIndirect {
        buffer: BufferId,
        offset: u64,
        count: u32,
    },
    /// Up to `max` [`DrawIndexedArgs`] records, how many a shader wrote at `count_offset`.
    /// Without [`Backend::indirect_count`] all `max` are drawn, so records past the count must
    /// draw nothing (zeroed with [`Cmd::FillBuffer`] before the shader writes them).
    DrawIndexedIndirectCount {
        buffer: BufferId,
        offset: u64,
        count_buffer: BufferId,
        count_offset: u64,
        max: u32,
    },
    DrawIndirectCount {
        buffer: BufferId,
        offset: u64,
        count_buffer: BufferId,
        count_offset: u64,
        max: u32,
    },
    Dispatch {
        x: u32,
        y: u32,
        z: u32,
    },
    /// Makes shader writes visible to later commands. Only needed between compute and its consumers.
    Barrier,
    /// Sets `size` bytes of `buffer` from `offset` to the 32-bit `value`, on the GPU. Outside a
    /// pass; offset and size multiples of 4. Ordered with later commands by a [`Cmd::Barrier`].
    FillBuffer {
        buffer: BufferId,
        offset: u64,
        size: u64,
        value: u32,
    },
    /// Ends the pass in progress and begins it again on the same images, keeping what it drew:
    /// what follows is drawn in a pass of its own. See `Backend::hand_own_pass`.
    PassBreak,
    /// Copies a depth texture to the start of a buffer, 32-bit floats row by row. Outside a pass.
    CopyDepthToBuffer {
        depth: TextureId,
        buffer: BufferId,
    },
    /// GPU timestamp, see [`Backend::gpu_times`].
    Mark,
    /// Up to three more color images for the [`Cmd::BeginPass`] that follows at once, at
    /// `@location(2)` on, after its motion image (each with its clear, as `motion`); their
    /// pipelines are made by [`Backend::create_pipeline_mrt`]. The path tracer's G-buffer.
    PassExtras([Option<(TextureId, Option<[f32; 4]>)>; 3]),
    /// Builds a BLAS made by [`Backend::create_blas`] from its inputs as they are now (written
    /// before, ordered by a [`Cmd::Barrier`] when a shader wrote them). Outside a pass. What reads
    /// it after (a TLAS build, a shader) waits for it.
    BuildBlas(AccelId),
    /// Builds a TLAS from the instances [`Backend::set_tlas_instances`] gave it this frame.
    /// Outside a pass, after the builds of the BLASes it holds; shaders after it see it.
    BuildTlas(AccelId),
    /// A TLAS for the current pipeline's group 1 `binding` (a `var x: acceleration_structure`).
    BindAccel {
        binding: u32,
        accel: AccelId,
    },
    /// The backend's temporal upscaler ([`Backend::upscaler`]: DLSS through NVIDIA's Streamline
    /// on Windows) makes `output` from this frame's jittered world. Outside a pass. Inputs are
    /// read as textures; `output` comes from [`Backend::create_upscale_target`]. The guides
    /// (Ray Reconstruction's) are `NO_TEXTURE` when not given. Pipelines and bindings are gone
    /// after it: set them again.
    Upscale {
        color: TextureId,
        depth: TextureId,
        motion: TextureId,
        output: TextureId,
        /// xyz the surface's normal (camera space), w its roughness.
        normals: TextureId,
        /// The diffuse albedo, linear.
        albedo: TextureId,
        /// The specular albedo, linear.
        specular: TextureId,
        /// An [`UpscaleParams`] at this offset in [`CommandList::data`].
        start: u32,
    },
}

/// A texture that is not there (an [`Cmd::Upscale`] guide not given).
pub const NO_TEXTURE: TextureId = TextureId(u32::MAX);

/// How much of the screen a temporal upscaler draws the world at ([`Cmd::Upscale`]). The numbers
/// are the `core:dlss` setting's. Ray Reconstruction is a flag beside it
/// ([`UpscaleParams::reconstruct`]), at the same sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum UpscaleMode {
    Off = 0,
    /// Anti-aliasing at the screen's size (DLAA).
    Native = 1,
    /// From about 2/3 of the screen each way.
    Quality = 2,
    /// From about 0.58.
    Balanced = 3,
    /// From half.
    Performance = 4,
    /// From a third.
    UltraPerformance = 5,
}

impl UpscaleMode {
    pub const ALL: [UpscaleMode; 6] = [
        UpscaleMode::Off,
        UpscaleMode::Native,
        UpscaleMode::Quality,
        UpscaleMode::Balanced,
        UpscaleMode::Performance,
        UpscaleMode::UltraPerformance,
    ];

    pub fn from_number(n: u32) -> UpscaleMode {
        Self::ALL
            .get(n as usize)
            .copied()
            .unwrap_or(UpscaleMode::Off)
    }

    /// The share of the screen each way the world is drawn at, NVIDIA's numbers; a backend that
    /// knows better says so in [`Backend::upscale_render_size`].
    pub fn scale(self) -> f32 {
        match self {
            UpscaleMode::Quality => 2.0 / 3.0,
            UpscaleMode::Balanced => 0.58,
            UpscaleMode::Performance => 0.5,
            UpscaleMode::UltraPerformance => 1.0 / 3.0,
            _ => 1.0,
        }
    }

    /// For logs, config key suffixes (`core:dlss_model_<name>`) and the Video menu's words.
    pub fn name(self) -> &'static str {
        match self {
            UpscaleMode::Off => "off",
            UpscaleMode::Native => "dlaa",
            UpscaleMode::Quality => "quality",
            UpscaleMode::Balanced => "balanced",
            UpscaleMode::Performance => "performance",
            UpscaleMode::UltraPerformance => "ultra_performance",
        }
    }
}

/// One of the upscaler's own models ([`UpscalerInfo::models`]): its number as the upscaler takes
/// it (0 its default) and a short key for the Video menu's words (`video.base.value.model_<key>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpscaleModel {
    pub id: u32,
    pub key: &'static str,
}

/// What [`Cmd::Upscale`] needs besides the images, in [`CommandList::data`]. Matrices are glam's
/// (column-major, column vectors), camera-relative (the camera at the origin), without the jitter.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UpscaleParams {
    /// [`UpscaleMode`] as its number.
    pub mode: u32,
    /// 1: no last frame to gather from (a teleport, a resize, a new world).
    pub reset: u32,
    /// 1: the colour is linear HDR (`core:hdr_world`), 0 linear 0..1.
    pub hdr: u32,
    /// 1: Ray Reconstruction (with the guides), at `mode`'s sizes.
    pub reconstruct: u32,
    /// The world's size (the inputs') and the output's.
    pub render: [u32; 2],
    pub output: [u32; 2],
    /// This frame's jitter in pixels of the world, x right, y down.
    pub jitter: [f32; 2],
    /// What turns the motion image's xy into -1..1 (1, 1 for ours, which are in UV).
    pub mv_scale: [f32; 2],
    /// DLSS 5's strength, 0..1.
    pub intensity: f32,
    /// Vertical field of view (radians), width over height, the near plane (blocks); the far
    /// plane is infinitely far (reverse Z).
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub view_to_clip: [f32; 16],
    pub clip_to_view: [f32; 16],
    pub clip_to_prev_clip: [f32; 16],
    pub prev_clip_to_clip: [f32; 16],
    pub world_to_view: [f32; 16],
    pub view_to_world: [f32; 16],
    /// The camera's right, up and forward in the world, w unused.
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub forward: [f32; 4],
    /// The model for this mode ([`UpscaleModel::id`], 0 the upscaler's default), and Ray
    /// Reconstruction's.
    pub model: u32,
    pub rr_model: u32,
    /// 1: DLSS 5's neural rendering over the result, at `intensity`.
    pub neural: u32,
    /// Generated frames between drawn ones (frame generation, later): 0 none.
    pub frame_gen: u32,
}

/// The temporal upscaler a backend has ([`Backend::upscaler`]), and what of it runs here: the
/// Video menu shows only that.
#[derive(Clone, Debug, Default)]
pub struct UpscalerInfo {
    /// For logs and the debug screen, e.g. "DLSS 310.4 (Streamline 2.14.16)"; empty with none.
    pub name: String,
    /// Which [`UpscaleMode`]s it runs, a bit per mode number.
    pub modes: u32,
    /// Ray Reconstruction.
    pub reconstruct: bool,
    /// DLSS 5's neural rendering.
    pub neural: bool,
    /// The most frames it generates between drawn ones (frame generation): 0 none.
    pub frame_gen: u32,
    /// Its models for the modes, and Ray Reconstruction's, the default first.
    pub models: Vec<UpscaleModel>,
    pub rr_models: Vec<UpscaleModel>,
}

impl UpscalerInfo {
    pub fn has(&self, mode: UpscaleMode) -> bool {
        mode != UpscaleMode::Off && self.modes & (1 << mode as u32) != 0
    }

    /// The model numbers as bits (`core:gpu_dlss_models`).
    pub fn model_bits(models: &[UpscaleModel]) -> u32 {
        models.iter().fold(0, |b, m| b | 1 << m.id.min(31))
    }
}

/// Layout of one indexed indirect draw, matches every API we care about.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedArgs {
    pub indices: u32,
    pub instances: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

/// Layout of one non-indexed indirect draw, matches every API we care about.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawArgs {
    pub vertices: u32,
    pub instances: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[derive(Default)]
pub struct CommandList {
    pub cmds: Vec<Cmd>,
    pub data: Vec<u8>,
}

impl CommandList {
    pub fn clear(&mut self) {
        self.cmds.clear();
        self.data.clear();
    }

    /// Moves `other`'s commands to the end of this list.
    pub fn append(&mut self, other: &mut CommandList) {
        let base = self.data.len() as u32;
        self.cmds.extend(other.cmds.drain(..).map(|c| match c {
            Cmd::PushConstants { start, len } => Cmd::PushConstants { start: start + base, len },
            c => c,
        }));
        self.data.append(&mut other.data);
    }

    pub fn push(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }

    pub fn push_constants<T: bytemuck::Pod>(&mut self, value: &T) {
        let start = self.data.len() as u32;
        self.data.extend_from_slice(bytemuck::bytes_of(value));
        self.cmds
            .push(Cmd::PushConstants { start, len: size_of::<T>() as u32 });
    }

    pub fn pipeline(&mut self, p: PipelineId) {
        self.cmds.push(Cmd::SetPipeline(p));
    }

    pub fn bind(&mut self, binding: u32, buffer: BufferId) {
        self.cmds
            .push(Cmd::BindBuffer { binding, buffer, offset: 0, size: u64::MAX });
    }

    pub fn bind_range(&mut self, binding: u32, buffer: BufferId, offset: u64, size: u64) {
        self.cmds
            .push(Cmd::BindBuffer { binding, buffer, offset, size });
    }

    pub fn draw(&mut self, vertices: u32, instances: u32) {
        self.cmds
            .push(Cmd::Draw { vertices, instances, first_vertex: 0, first_instance: 0 });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FrameInfo {
    pub width: u32,
    pub height: u32,
    /// Index of the frame-in-flight slot, for callers that keep per-frame host buffers.
    pub slot: usize,
}

pub struct Capture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub trait Window: HasWindowHandle + HasDisplayHandle {}
impl<T: HasWindowHandle + HasDisplayHandle> Window for T {}

pub struct BackendDesc<'a> {
    pub window: &'a dyn Window,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub debug: bool,
    /// A stand-in upscaler that only scales the picture up (`Backend::upscaler`), for checking
    /// everything around one where there is none (`core:upscale_standin`).
    pub upscale_standin: bool,
}

pub trait Backend: Send {
    fn name(&self) -> &str;
    /// The GPU in use, for diagnostics.
    fn device_name(&self) -> &str {
        ""
    }
    /// What kind of GPU it is: 0 not known, 1 integrated, 2 discrete, 3 another (virtual, CPU).
    /// The base game picks its first quality preset by it (Research round 19, 7).
    fn device_class(&self) -> u32 {
        0
    }
    /// Its own memory (the largest device-local heap), MB; 0 not known.
    fn device_memory_mb(&self) -> u64 {
        0
    }
    fn frames_in_flight(&self) -> usize;
    /// Whether the `*IndirectCount` draws take their count from the GPU (Vulkan 1.2's
    /// `drawIndirectCount`). Without it (MoltenVK) they draw all `max` records.
    fn indirect_count(&self) -> bool {
        true
    }
    /// Whether the first-person hand goes in a render pass of its own (`Cmd::PassBreak`). On
    /// Imagination's GPUs (the Pixel 10) the hand, which writes depth and discards the texels of
    /// the body parts it hides, wiped out blended water drawn before it in the same pass in whole
    /// 32-pixel tiles: squares at the bottom of the screen where the sand under the water showed.
    fn hand_own_pass(&self) -> bool {
        false
    }
    fn surface_size(&self) -> (u32, u32);
    /// The window's surface is going away (Android sends the app to the background): drop it and
    /// its swapchain, keeping everything else. No frames until `surface_restored`.
    fn surface_lost(&mut self) {}
    /// The window has a surface again (Android brought the app back): make one for it.
    fn surface_restored(&mut self, _window: &dyn Window) -> Result<(), Error> {
        Ok(())
    }
    fn resize(&mut self, width: u32, height: u32);
    fn set_vsync(&mut self, vsync: bool);

    fn create_buffer(&mut self, size: u64, usage: u32, label: &str) -> BufferId;
    /// Only valid for `usage::HOST` buffers.
    fn mapped(&mut self, buffer: BufferId) -> &mut [u8];
    /// Copied on the GPU timeline at the start of the next submitted frame, before any pass.
    fn write_buffer(&mut self, buffer: BufferId, offset: u64, data: &[u8]);
    fn destroy_buffer(&mut self, buffer: BufferId);

    /// Returns the texture and its bindless index (into `textures`).
    fn create_texture(&mut self, desc: &TextureDesc, label: &str) -> TextureId;
    /// `data` holds one mip level, tightly packed.
    fn write_texture(&mut self, texture: TextureId, mip: u32, data: &[u8]);
    fn texture_index(&self, texture: TextureId) -> u32;
    fn destroy_texture(&mut self, texture: TextureId);

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineId, Error>;
    /// A pipeline drawing into `extra` color images as well, at `@location(2)` on (after the
    /// motion image, which it must have: `desc.motion`), in a pass begun after a
    /// [`Cmd::PassExtras`] naming them. Without such targets (the default), `create_pipeline`.
    fn create_pipeline_mrt(&mut self, desc: &PipelineDesc, extra: &[Format]) -> Result<PipelineId, Error> {
        let _ = extra;
        self.create_pipeline(desc)
    }
    fn create_compute(&mut self, desc: &ComputeDesc) -> Result<PipelineId, Error>;
    fn destroy_pipeline(&mut self, pipeline: PipelineId);

    /// `None` when the surface cannot be drawn to right now (minimized).
    fn begin_frame(&mut self) -> Option<FrameInfo>;
    fn end_frame(&mut self, cmds: &CommandList);
    /// The next `end_frame` copies the surface image back to the CPU.
    fn request_capture(&mut self);
    /// Stalls until the requested capture is ready. Debug and screenshots only.
    fn take_capture(&mut self) -> Option<Capture>;
    /// GPU milliseconds of the last finished frame, split at [`Cmd::Mark`]s: uploads, then one
    /// entry per mark, then the rest of the frame. Empty if the backend cannot measure.
    fn gpu_times(&self) -> &[f32] {
        &[]
    }
    /// The driver's compiled pipelines, to keep on disk and hand back to `load_pipeline_cache`
    /// next start, when pipelines made again skip most of their compiling. Empty if it keeps none.
    fn pipeline_cache_data(&self) -> Vec<u8> {
        Vec::new()
    }
    /// What an earlier `pipeline_cache_data` gave, before any pipeline is made. Data from another
    /// driver or device is ignored.
    fn load_pipeline_cache(&mut self, _data: &[u8]) {}
    /// The temporal upscaler [`Cmd::Upscale`] runs, if any (none by default: the engine's own
    /// temporal anti-aliasing then).
    fn upscaler(&self) -> UpscalerInfo {
        UpscalerInfo::default()
    }
    /// The size to draw the world at for `mode` (with Ray Reconstruction: `reconstruct`) with the
    /// screen at `output`.
    fn upscale_render_size(&mut self, mode: UpscaleMode, reconstruct: bool, output: (u32, u32)) -> (u32, u32) {
        let _ = reconstruct;
        let s = mode.scale();
        (((output.0 as f32 * s).round() as u32).max(1), ((output.1 as f32 * s).round() as u32).max(1))
    }
    /// An image [`Cmd::Upscale`] can write its result to (Rgba16Float, read as a texture after).
    fn create_upscale_target(&mut self, width: u32, height: u32, label: &str) -> TextureId {
        self.create_texture(&TextureDesc { width, height, mips: 1, format: Format::Rgba16Float, render_target: true }, label)
    }

    /// Whether shaders can trace rays (`enable wgpu_ray_query;`) through acceleration structures:
    /// a desktop GPU with Vulkan's ray queries. Without it the calls below must not be made.
    fn ray_query(&self) -> bool {
        false
    }
    /// A bottom-level acceleration structure over `geometries`, built by the next
    /// [`Cmd::BuildBlas`] naming it (its inputs must hold until that frame is done), then compacted
    /// on its own. Built once: to change it, make another.
    fn create_blas(&mut self, _geometries: &[BlasGeometry], _label: &str) -> AccelId {
        panic!("no ray tracing on this backend")
    }
    /// A top-level acceleration structure for up to `max_instances` (it grows past them).
    fn create_tlas(&mut self, _max_instances: u32, _label: &str) -> AccelId {
        panic!("no ray tracing on this backend")
    }
    /// This frame's instances of `tlas`, built by its [`Cmd::BuildTlas`].
    fn set_tlas_instances(&mut self, _tlas: AccelId, _instances: &[TlasInstance]) {}
    /// Once the GPU is done with it.
    fn destroy_accel(&mut self, _accel: AccelId) {}
    /// Memory held by acceleration structures, bytes (diagnostics).
    fn accel_memory(&self) -> u64 {
        0
    }
    /// Goes up whenever a BLAS moves in memory (compaction): a TLAS built before must be built
    /// again before it is used once more.
    fn accel_generation(&self) -> u64 {
        0
    }
    fn wait_idle(&mut self);
}

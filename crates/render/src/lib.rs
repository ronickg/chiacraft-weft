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
}

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

//! Screen effects selected by the resolved environment: `shaders/post/<name>.wgsl` snippets in
//! one pipeline. With none on, the world is drawn straight to the screen; with some, or drawn
//! smaller than the screen (`core:render_scale`), it goes into an image the effects read one
//! after another, the last one onto the screen, stretched (with none, a pass without an effect).

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use weft_render::*;

use crate::assets::Assets;
use crate::gpu_layout::Pass;
use crate::shaders::{Hook, Slots};

pub const HOOK: Hook = Hook {
    kind: "post",
    entry: "apply",
    dispatch: "post_apply",
    params: "p: PostPixel",
    args: "p",
    ret: "vec4<f32>",
    fallback: "return p.color;",
};

/// An effect's half-size part (chiacraft): `shaders/post_low/<name>.wgsl`, `fn low(p: PostPixel) ->
/// vec4<f32>`, drawn before the effects into an image of its own a quarter the pixels, which the
/// effect of the same name reads (`p.low`, `post_low_upsampled`).
pub const HOOK_LOW: Hook = Hook {
    kind: "post_low",
    entry: "low",
    dispatch: "post_low",
    params: "p: PostPixel",
    args: "p",
    ret: "vec4<f32>",
    fallback: "return vec4(0.0);",
};

static HDR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The HDR world (chiacraft, `core:hdr_world`): set once at start, before any pipeline is made, as
/// every pipeline drawing into the world must have its image's format.
pub fn set_hdr(on: bool) {
    HDR.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether the world is drawn in 16-bit float, the screen effects after it and the picture finished
/// in the last pass (`Pass::spare` 1 there, for a pack's tone curve).
pub fn hdr() -> bool {
    HDR.load(std::sync::atomic::Ordering::Relaxed)
}

static MOTION: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static TAA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Temporal anti-aliasing at all (`core:aa` 1 or 2), set once at start: its pipelines are made.
pub fn set_taa(on: bool) {
    TAA.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether temporal anti-aliasing's pipelines are made (`set_taa`): with the motion image
/// (`motion`), or from the camera alone (`core:aa` 2: what moves by itself is left to the
/// neighbourhood's clipping; no second image in the world's pass, which cost a phone's tiled GPU
/// a third of its world pass).
pub fn taa() -> bool {
    TAA.load(std::sync::atomic::Ordering::Relaxed)
}

/// Temporal anti-aliasing (`core:aa`): set once at start, before any pipeline is made, as every
/// pipeline drawing into the world must then have the motion image too (`PipelineDesc::motion`).
pub fn set_motion(on: bool) {
    MOTION.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether the world's pass has a motion image (`weft_render::MOTION_FORMAT`), which shaders
/// with a `@location(1)` output write: where each pixel's surface was last frame.
pub fn motion() -> bool {
    MOTION.load(std::sync::atomic::Ordering::Relaxed)
}

/// This frame's jitter in NDC (`jitter`), as bits, and the frame count it is picked by.
static JITTER: [AtomicU32; 2] = [AtomicU32::new(0), AtomicU32::new(0)];
static JITTER_FRAME: AtomicU32 = AtomicU32::new(0);

/// How far this frame's world is shifted on the screen, in NDC (`Camera::proj`): a fraction of
/// one of its pixels with temporal anti-aliasing on, else none.
pub fn jitter() -> glam::Vec2 {
    glam::Vec2::new(f32::from_bits(JITTER[0].load(Relaxed)), f32::from_bits(JITTER[1].load(Relaxed)))
}

static UPSCALING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether this frame goes to the backend's upscaler (`Post::set_upscale`), set by the app before
/// the world is drawn: per-frame dithers then hold still (`Scene::frame.w`), as DLSS takes a
/// pattern that changes every frame for noise and can't settle it (speckled canopy in the fade
/// band, SR Quality under Proton, 11-06). `WEFT_UPSCALE_MOVING_DITHER=1` keeps them moving.
pub fn set_upscaling(on: bool) {
    UPSCALING.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn upscaling() -> bool {
    UPSCALING.load(std::sync::atomic::Ordering::Relaxed)
}

static MIP_BIAS: AtomicU32 = AtomicU32::new(0);

/// The texture mip bias for this frame's world (`Scene::dlss.x`): set by the app under an
/// upscaler to NVIDIA's log2(render / output) - 1 (DLSS's programming guide: textures as sharp as
/// at the output's size), 0 without. `WEFT_UPSCALE_MIP_BIAS=<levels>` puts another under an upscaler.
pub fn set_mip_bias(render_width: u32, output_width: u32) {
    let bias = if upscaling() {
        static OVERRIDE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
        OVERRIDE
            .get_or_init(|| {
                std::env::var("WEFT_UPSCALE_MIP_BIAS")
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
            })
            .unwrap_or_else(|| (render_width.max(1) as f32 / output_width.max(1) as f32).log2() - 1.0)
    } else {
        0.0
    };
    MIP_BIAS.store(bias.to_bits(), Relaxed);
}

pub fn mip_bias() -> f32 {
    f32::from_bits(MIP_BIAS.load(Relaxed))
}

/// Whether per-frame dithers move this frame (all but under an upscaler, see `set_upscaling`).
pub fn dither_moves() -> bool {
    static MOVING: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !upscaling() || *MOVING.get_or_init(|| std::env::var_os("WEFT_UPSCALE_MOVING_DITHER").is_some())
}

static TAA_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether this frame is gathered over frames (`next_jitter`'s `on`): the shaders then write
/// motion (`Scene::taa.z`), and skip it otherwise.
pub fn taa_active() -> bool {
    TAA_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The jitter sequence's length: Halton's points in bases 2 and 3 spread evenly over any run of
/// them, and 8 cover a pixel well (Bliss's eight, UE's default); an upscaler may want more
/// (DLSS asks 8 x its scale squared), `set_jitter_phases`.
static JITTER_PHASES: AtomicU32 = AtomicU32::new(8);

/// How many points the jitter goes through before it starts again (8; an upscaler's own count).
pub fn set_jitter_phases(n: u32) {
    JITTER_PHASES.store(n.clamp(1, 1024), Relaxed);
}

/// Picks this frame's jitter, before the world is drawn: with `on`, the next point within one of
/// the world's pixels (`w` x `h`) in Halton's sequence; else none.
pub fn next_jitter(on: bool, w: u32, h: u32) {
    TAA_ACTIVE.store(on, std::sync::atomic::Ordering::Relaxed);
    let j = if on {
        let n = JITTER_PHASES.load(Relaxed).max(1);
        let i = JITTER_FRAME.fetch_add(1, Relaxed) % n + 1;
        let (x, y) = jitter_pixels(i);
        // Pixels run down the screen, NDC up.
        glam::Vec2::new(2.0 * x / w.max(1) as f32, -2.0 * y / h.max(1) as f32)
    } else {
        glam::Vec2::ZERO
    };
    JITTER[0].store(j.x.to_bits(), Relaxed);
    JITTER[1].store(j.y.to_bits(), Relaxed);
}

/// The `i`th jitter (from 1) in pixels, -0.5..0.5 each way, x right and y down.
pub fn jitter_pixels(i: u32) -> (f32, f32) {
    (halton(i, 2) - 0.5, halton(i, 3) - 0.5)
}

fn halton(mut i: u32, base: u32) -> f32 {
    let (mut f, mut r) = (1.0, 0.0);
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

/// A frame drawn for the backend's temporal upscaler (`Backend::upscaler`, DLSS on Windows):
/// what the app hands `Post::set_upscale`. `params` carries the camera (the rest `apply` fills).
#[derive(Clone, Copy, Debug)]
pub struct UpscaleFrame {
    pub mode: UpscaleMode,
    /// Ray Reconstruction (and DLSS 5, which reads the same guides): the guides are drawn.
    pub reconstruct: bool,
    /// The output's size: the screen's.
    pub output: (u32, u32),
    pub params: UpscaleParams,
}

/// What a temporal upscaler (DLSS/DLAA on the PC, Arm ASR on Android) needs of this frame, as the
/// engine's own temporal anti-aliasing takes it (`Post::temporal_inputs`), all at the world's size:
/// - `color`: the world as drawn, jittered, linear light (`world_format`), before any screen effect;
/// - `depth`: its depth, reverse Z (1 near, 0 the sky), infinite far plane;
/// - `motion`: the whole motion image (`fs_taa_motion`, `Format::Rgba16Float`): xy where each
///   pixel's surface was last frame minus where it is now, in UV (multiply by the size for pixels;
///   both without the jitter), z its depth, w 1 where a surface wrote its own motion (entities,
///   swaying leaves and grass), 0 where it came from the camera. With `core:aa` 2 (no motion
///   image) all of it comes from the camera, and only with `export_motion` is there an image at
///   all (`TextureId(u32::MAX)` otherwise);
/// - `jitter`: this frame's offset in pixels (x right, y down), as `jitter_pixels`;
/// - `reset`: no last frame to gather from (a teleport, a resize, a new world);
/// - `view_proj` and `prev_view_proj` (`Scene::taa_prev`) without the jitter.
/// Its result goes back through `Post::set_upscaled` (instead of the engine's own resolve).
#[derive(Clone, Copy, Debug)]
pub struct TemporalInputs {
    pub color: TextureId,
    pub depth: TextureId,
    pub motion: TextureId,
    pub size: (u32, u32),
    pub jitter: [f32; 2],
    pub reset: bool,
}

/// The path tracer's G-buffer and specular light (`Post::set_pt_guides`), all at the world's size:
/// Ray Reconstruction's guides come from them instead of `upscale_guides.wgsl`'s estimates from
/// the picture and depth (the layout agreed with the RT session, `notes/` "pt-denoisers").
#[derive(Clone, Copy, Debug)]
pub struct PtGuides {
    /// xyz the shading normal (world axes, camera-relative), w its perceptual roughness.
    pub normal: TextureId,
    /// rgb the diffuse albedo, linear (metals 0).
    pub albedo: TextureId,
    /// rgb the reflectance at normal incidence (F0).
    pub spec: TextureId,
    /// The path tracer's specular light; a how far its ray went (blocks). None before it has one:
    /// no hit distance then.
    pub specular: Option<TextureId>,
}

static BLOOM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the last frame drew the HDR world's bloom pyramid (`Pass::bloom`): a pack's own bloom
/// effect then leaves the glow to its last pass (`Scene::frame` y).
pub fn bloom() -> bool {
    BLOOM.load(std::sync::atomic::Ordering::Relaxed)
}

/// The HDR world's bloom pyramid and exposure (chiacraft): the images the pack's `fs_bloom_down`
/// draws, each half the one before, from half the world's size; those `fs_bloom_up` draws back up,
/// each the level below it blended with the wider one over it; and two 1x1 exposure texels
/// (`fs_exposure`), this frame's from last frame's.
struct Pyramid {
    size: (u32, u32),
    down: Vec<(TextureId, u32, (u32, u32))>,
    up: Vec<(TextureId, u32, (u32, u32))>,
    exposure: [(TextureId, u32); 2],
    /// Which exposure texel this frame writes.
    flip: usize,
    /// The levels asked for (fewer are made for a small world), and whether the exposure has no
    /// last frame yet (its pass is told, `spare` 1, and starts from this frame's alone).
    levels: usize,
    fresh: bool,
}

/// The format of the image the world is drawn into: the screen's, or 16-bit float.
pub fn world_format() -> Format {
    if hdr() { Format::Rgba16Float } else { Format::Surface }
}

pub struct Post {
    pipeline: Option<PipelineId>,
    /// With the HDR world: the same shader drawing the last pass onto the screen (its format).
    final_pipe: Option<PipelineId>,
    slots: Slots,
    /// The half-size parts' pipeline and slots (`HOOK_LOW`), and each effect's image for it:
    /// (name, texture, bindless index, size).
    low_pipe: Option<PipelineId>,
    low_slots: Slots,
    low_targets: Vec<(String, TextureId, u32, (u32, u32))>,
    /// Whether the half-size parts run (`core:post_low`); off, the effects do their full work.
    pub use_low: bool,
    /// Effects on, in the order they were turned on: (name, strength).
    pub active: Vec<(String, f32)>,
    /// Two images the size of the screen (scene and the one after each pass), with their
    /// bindless indices.
    targets: Option<([TextureId; 2], [u32; 2], (u32, u32))>,
    params: Vec<BufferId>,
    /// A scene of zeros, bound when there is no world's (`apply`).
    no_scene: Option<BufferId>,
    /// The world before what is seen through it is drawn (`copy_behind`): the copy pipeline, the
    /// color and depth images with their bindless indices and size, and the copy's pass entries.
    copy_pipe: Option<PipelineId>,
    behind: Option<([TextureId; 2], [u32; 2], (u32, u32))>,
    copy_params: Vec<BufferId>,
    /// The HDR world's bloom pyramid and exposure: the pack's three passes (down, up, exposure),
    /// their images, how many levels to draw (`core:bloom`: 0 none, 4 or 6) and when the last
    /// frame was, for the exposure's easing.
    bloom_pipes: Option<[PipelineId; 3]>,
    pyramid: Option<Pyramid>,
    pub bloom_levels: u32,
    last_frame: Option<std::time::Instant>,
    /// Temporal anti-aliasing (`taa.wgsl`): its two pipelines (`fs_taa` gathering, `fs_taa_motion`
    /// the whole motion image for an upscaler); the motion image the world's pass draws into
    /// (`motion_target`) with its bindless index and size; the two pictures so far (each frame's
    /// drawn from the other's), which this frame draws, and whether the other holds last
    /// frame's; and the whole motion image when an upscaler asked for it.
    taa_pipes: Option<(PipelineId, PipelineId)>,
    motion_tex: Option<(TextureId, u32, (u32, u32))>,
    history: Option<([TextureId; 2], [u32; 2], (u32, u32))>,
    /// Beside each picture so far, the frames gathered in each pixel (`fs_taa`'s second output).
    counts: Option<([TextureId; 2], [u32; 2], (u32, u32))>,
    taa_cur: usize,
    taa_held: bool,
    full_motion: Option<(TextureId, u32, (u32, u32))>,
    /// Whether temporal anti-aliasing is wanted (`core:aa`), its least share of a new frame
    /// (`core:taa_blend`), set by the app each frame, and whether an upscaler wants the whole
    /// motion image drawn (`temporal_inputs`).
    pub use_taa: bool,
    pub taa_blend: f32,
    pub export_motion: bool,
    /// This frame's inputs, as `apply` drew them (`temporal_inputs`), and the world's depth image
    /// (`set_depth`).
    inputs: Option<TemporalInputs>,
    depth_tex: Option<TextureId>,
    /// An upscaler's picture for this frame in place of the engine's resolve (`set_upscaled`).
    upscaled: Option<u32>,
    /// The backend's upscaler for this frame (`set_upscale`), in the resolve's place: its output
    /// at the screen's size, the effects' two images at that size, Ray Reconstruction's guides
    /// (`upscale_guides.wgsl`: pipelines and images, normals + roughness, albedo, specular) and
    /// what the last upscaled frame was (mode, world size, output size), else it starts anew.
    upscale: Option<UpscaleFrame>,
    up_out: Option<(TextureId, u32, (u32, u32))>,
    up_targets: Option<([TextureId; 2], [u32; 2], (u32, u32))>,
    guide_pipes: Option<(PipelineId, PipelineId)>,
    guides: Option<([TextureId; 3], (u32, u32))>,
    /// From the path tracer's G-buffer (`set_pt_guides`): this frame's, the pipeline drawing the
    /// specular albedo and hit distance from it (`fs_pt_guides`), and those two images.
    pt_guides: Option<PtGuides>,
    /// `fs_pt_guides_a` (normals + roughness and albedo from the G-buffer, estimated where it is
    /// empty) and `fs_pt_guides` (specular albedo and hit distance).
    pt_pipe_a: Option<PipelineId>,
    pt_pipe: Option<PipelineId>,
    pt_targets: Option<([TextureId; 2], (u32, u32))>,
    up_held: Option<(UpscaleMode, bool, (u32, u32), (u32, u32))>,
    /// The path tracer's denoiser (`denoise.rs`), which the path tracer runs after its pass.
    pub denoiser: crate::denoise::Denoiser,
}

const MAX_PASSES: usize = 40;

impl Post {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Post {
        let mut p = Post {
            pipeline: None,
            final_pipe: None,
            slots: Slots::default(),
            low_pipe: None,
            low_slots: Slots::default(),
            low_targets: Vec::new(),
            use_low: true,
            active: Vec::new(),
            targets: None,
            params: Vec::new(),
            no_scene: None,
            copy_pipe: None,
            behind: None,
            copy_params: Vec::new(),
            bloom_pipes: None,
            pyramid: None,
            bloom_levels: 6,
            last_frame: None,
            taa_pipes: None,
            motion_tex: None,
            history: None,
            counts: None,
            taa_cur: 0,
            taa_held: false,
            full_motion: None,
            use_taa: false,
            taa_blend: 0.1,
            export_motion: false,
            inputs: None,
            depth_tex: None,
            upscaled: None,
            upscale: None,
            denoiser: crate::denoise::Denoiser::new(gpu, assets),
            pt_guides: None,
            pt_pipe_a: None,
            pt_pipe: None,
            pt_targets: None,
            up_out: None,
            up_targets: None,
            guide_pipes: None,
            guides: None,
            up_held: None,
        };
        p.reload(gpu, assets);
        p
    }

    /// Temporal anti-aliasing's pipelines from `taa.wgsl` (a pack's, else the engine's), made only
    /// with the motion image (`motion`).
    fn reload_taa(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        if let Some((a, b)) = self.taa_pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        self.taa_held = false;
        if !taa() {
            return;
        }
        let made = crate::packs::shader(assets, "taa.wgsl")
            .map_err(|e| e.0)
            .and_then(|s| {
                // The gathering writes its count beside the picture (`motion`: a second image).
                let mut make = |label: &str, fragment: &str, motion: bool| {
                    gpu.create_pipeline(&PipelineDesc {
                        label,
                        shader: &s,
                        vertex: "vs",
                        fragment: Some(fragment),
                        color: Some(Format::Rgba16Float),
                        depth: None,
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion,
                    })
                    .map_err(|e| e.0)
                };
                let resolve = make("temporal aa", "fs_taa", true)?;
                match make("temporal aa motion", "fs_taa_motion", false) {
                    Ok(m) => Ok((resolve, m)),
                    Err(e) => {
                        gpu.destroy_pipeline(resolve);
                        Err(e)
                    }
                }
            });
        match made {
            Ok(p) => self.taa_pipes = Some(p),
            Err(e) => log::error!("temporal anti-aliasing: {e}"),
        }
        self.reload_guides(gpu, assets);
    }

    /// Ray Reconstruction's guides (`upscale_guides.wgsl`), made with the motion image too: only
    /// for a backend with an upscaler that takes them.
    fn reload_guides(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        if let Some((a, b)) = self.guide_pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        for p in [self.pt_pipe.take(), self.pt_pipe_a.take()]
            .into_iter()
            .flatten()
        {
            gpu.destroy_pipeline(p);
        }
        let info = gpu.upscaler();
        if !motion() || !(info.reconstruct || info.neural) {
            return;
        }
        let made = crate::packs::shader(assets, "upscale_guides.wgsl")
            .map_err(|e| e.0)
            .and_then(|s| {
                let mut make = |label: &str, fragment: &str, motion: bool| {
                    gpu.create_pipeline(&PipelineDesc {
                        label,
                        shader: &s,
                        vertex: "vs",
                        fragment: Some(fragment),
                        color: Some(Format::Rgba16Float),
                        depth: None,
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion,
                    })
                    .map_err(|e| e.0)
                };
                // Normals and roughness, with the albedo beside them (the second image).
                let guides = make("upscale guides", "fs_guides", true)?;
                match make("upscale specular", "fs_specular", false) {
                    Ok(sp) => Ok((
                        guides,
                        sp,
                        make("upscale pt guides", "fs_pt_guides", true)
                            .and_then(|b| make("upscale pt guides a", "fs_pt_guides_a", true).map(|a| (a, b))),
                    )),
                    Err(e) => {
                        gpu.destroy_pipeline(guides);
                        Err(e)
                    }
                }
            });
        match made {
            Ok((g, sp, pt)) => {
                self.guide_pipes = Some((g, sp));
                match pt {
                    Ok((a, b)) => {
                        self.pt_pipe_a = Some(a);
                        self.pt_pipe = Some(b);
                    }
                    Err(e) => log::error!("upscaler guides from the G-buffer: {e}"),
                }
            }
            Err(e) => log::error!("upscaler guides: {e}"),
        }
    }

    /// Whether a frame can go to the backend's upscaler (`set_upscale`): the world's pass draws
    /// the motion image and the pipelines that finish it are there.
    pub fn can_upscale(&self) -> bool {
        motion() && self.taa_pipes.is_some() && self.pipeline.is_some()
    }

    /// This frame goes to the backend's upscaler (`Cmd::Upscale`) in the place of the engine's
    /// temporal anti-aliasing, drawn jittered at the size it asked for; the effects then run at
    /// `frame.output`, the screen's size.
    pub fn set_upscale(&mut self, frame: UpscaleFrame) {
        self.upscale = Some(frame);
    }

    /// This frame was path traced: Ray Reconstruction's guides come from its G-buffer
    /// (`PtGuides`), with its specular hit distance. Set each frame, before `apply`.
    pub fn set_pt_guides(&mut self, guides: PtGuides) {
        self.pt_guides = Some(guides);
    }

    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        self.reload_taa(gpu, assets);
        self.denoiser.reload(gpu, assets);
        let snippets = crate::shaders::snippets(assets, &HOOK);
        let low_snippets = crate::shaders::snippets(assets, &HOOK_LOW);
        let made = crate::packs::compose(assets, "post.wgsl", &[(&HOOK, &snippets), (&HOOK_LOW, &low_snippets)])
            .map_err(|e| e.0)
            .and_then(|(s, mut slots)| {
                // The half-size parts' pipeline (`fs_low`), when there are any.
                let low_slots = slots.pop().unwrap_or_default();
                let low = (!low_slots.names.is_empty())
                    .then(|| {
                        gpu.create_pipeline(&PipelineDesc {
                            label: "post low",
                            shader: &s,
                            vertex: "vs",
                            fragment: Some("fs_low"),
                            color: Some(Format::Rgba16Float),
                            depth: None,
                            blend: Blend::None,
                            cull_back: false,
                            topology: Topology::Triangles,
                            motion: false,
                        })
                        .map_err(|e| log::warn!("post: no half-size parts: {}", e.0))
                        .ok()
                    })
                    .flatten();
                let p = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "post",
                        shader: &s,
                        vertex: "vs",
                        fragment: Some("fs"),
                        color: Some(world_format()),
                        depth: None,
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: false,
                    })
                    .map_err(|e| e.0)?;
                // The copy for `copy_behind`; a pack's own post.wgsl may lack it.
                let copy = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "post copy behind",
                        shader: &s,
                        vertex: "vs",
                        fragment: Some("fs_copy_behind"),
                        color: Some(world_format()),
                        depth: Some(DepthState { write: true, compare: Compare::Always }),
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: false,
                    })
                    .map_err(|e| log::warn!("post: no copy of the world for water: {}", e.0))
                    .ok();
                // The last pass onto the screen, from the HDR world.
                let last = if hdr() {
                    Some(
                        gpu.create_pipeline(&PipelineDesc {
                            label: "post last",
                            shader: &s,
                            vertex: "vs",
                            fragment: Some("fs"),
                            color: Some(Format::Surface),
                            depth: None,
                            blend: Blend::None,
                            cull_back: false,
                            topology: Topology::Triangles,
                            motion: false,
                        })
                        .map_err(|e| e.0)?,
                    )
                } else {
                    None
                };
                // The HDR world's bloom pyramid and exposure, when the pack's post.wgsl has them.
                let bloom = if hdr() {
                    // Said only when a pack's post.wgsl is in use and lacks them: the built-in one
                    // never has them, and its lines at every start read as a fault (Tester's Mac run).
                    let from_pack = crate::packs::pack_source("post.wgsl").is_some();
                    let mut make = |label: &'static str, fragment: &'static str| {
                        gpu.create_pipeline(&PipelineDesc {
                            label,
                            shader: &s,
                            vertex: "vs",
                            fragment: Some(fragment),
                            color: Some(Format::Rgba16Float),
                            depth: None,
                            blend: Blend::None,
                            cull_back: false,
                            topology: Topology::Triangles,
                            motion: false,
                        })
                        .map_err(|e| {
                            if from_pack {
                                log::info!("post: no bloom pyramid ({fragment}): {}", e.0)
                            }
                        })
                        .ok()
                    };
                    match (make("bloom down", "fs_bloom_down"), make("bloom up", "fs_bloom_up"), make("exposure", "fs_exposure")) {
                        (Some(d), Some(u), Some(e)) => Some([d, u, e]),
                        (d, u, e) => {
                            [d, u, e]
                                .into_iter()
                                .flatten()
                                .for_each(|p| gpu.destroy_pipeline(p));
                            None
                        }
                    }
                } else {
                    None
                };
                Ok((p, copy, slots.pop().unwrap_or_default(), low, low_slots, last, bloom))
            });
        match made {
            Ok((p, copy, slots, low, low_slots, last, bloom)) => {
                if let Some(old) = std::mem::replace(&mut self.bloom_pipes, bloom) {
                    old.into_iter().for_each(|p| gpu.destroy_pipeline(p));
                }
                if let Some(old) = std::mem::replace(&mut self.final_pipe, last) {
                    gpu.destroy_pipeline(old);
                }
                if let Some(old) = std::mem::replace(&mut self.low_pipe, low) {
                    gpu.destroy_pipeline(old);
                }
                self.low_slots = low_slots;
                if let Some(old) = self.pipeline.replace(p) {
                    gpu.destroy_pipeline(old);
                }
                if let Some(old) = std::mem::replace(&mut self.copy_pipe, copy) {
                    gpu.destroy_pipeline(old);
                }
                self.slots = slots;
            }
            Err(e) => log::error!("post shader: {e}"),
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.slots.slot(name) != 0
    }

    /// Whether this frame needs the scene drawn into an image first.
    pub fn on(&self) -> bool {
        self.pipeline.is_some() && !self.active.is_empty()
    }

    /// Whether it can put a world drawn smaller than the screen onto it (`apply` with no effect
    /// on draws it across, filtered).
    pub fn can_scale(&self) -> bool {
        self.pipeline.is_some()
    }

    /// Whether a scene drawn smaller can be stretched over the screen (upstream's name for
    /// `can_scale`).
    pub fn stretches(&self) -> bool {
        self.can_scale()
    }

    /// Whether this frame is gathered over frames (`use_taa`, the motion image, and the pipelines):
    /// the app asks before drawing the world, which is then jittered (`next_jitter`).
    pub fn taa_on(&self) -> bool {
        self.use_taa && taa() && self.taa_pipes.is_some() && self.pipeline.is_some()
    }

    /// The motion image the world's pass draws into, sized to the frame (`Cmd::BeginPass`'s
    /// `motion`); none without temporal anti-aliasing's pipelines (`set_motion`).
    pub fn motion_target(&mut self, gpu: &mut dyn Backend, w: u32, h: u32) -> Option<TextureId> {
        if !motion() {
            return None;
        }
        let (w, h) = (w.max(1), h.max(1));
        if self.motion_tex.is_none_or(|t| t.2 != (w, h)) {
            if let Some((old, ..)) = self.motion_tex.take() {
                gpu.destroy_texture(old);
            }
            let t = gpu.create_texture(&TextureDesc { width: w, height: h, mips: 1, format: MOTION_FORMAT, render_target: true }, "motion");
            self.motion_tex = Some((t, gpu.texture_index(t), (w, h)));
        }
        self.motion_tex.map(|t| t.0)
    }

    /// The count image this frame's gathering will write (bindless index + 1, `Scene::taa_frame`),
    /// asked before the world is drawn: 0 without temporal anti-aliasing, or on a frame whose size
    /// differs from the images' (they are made anew in `apply`).
    pub fn taa_out(&self, w: u32, h: u32) -> u32 {
        match self.counts {
            Some((_, idx, size)) if self.taa_on() && size == (w, h) && self.upscaled.is_none() => idx[self.taa_cur] + 1,
            _ => 0,
        }
    }

    /// What a temporal upscaler needs of the frame `apply` last drew (`TemporalInputs`), with
    /// `export_motion` set (else the motion is not drawn whole); none without temporal
    /// anti-aliasing.
    pub fn temporal_inputs(&self) -> Option<TemporalInputs> {
        self.inputs
    }

    /// The world's depth image, for `TemporalInputs` (its bindless index goes to `apply`).
    pub fn set_depth(&mut self, depth: TextureId) {
        self.depth_tex = Some(depth);
    }

    /// For this frame, the bindless index of a picture an upscaler made from `temporal_inputs`
    /// (the world's size, linear light, not jittered): everything after reads it in place of the
    /// engine's own resolve, which is skipped.
    pub fn set_upscaled(&mut self, picture: u32) {
        self.upscaled = Some(picture);
    }

    /// One of a pair of images sized to the frame, made anew (and their bindless indices) when the
    /// size changed: whether they were.
    fn ensure_pair(gpu: &mut dyn Backend, pair: &mut Option<([TextureId; 2], [u32; 2], (u32, u32))>, w: u32, h: u32, label: &str) -> bool {
        if pair.is_some_and(|t| t.2 == (w, h)) {
            return false;
        }
        if let Some((old, ..)) = pair.take() {
            old.into_iter().for_each(|t| gpu.destroy_texture(t));
        }
        let desc = TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: Format::Rgba16Float, render_target: true };
        let t = [gpu.create_texture(&desc, label), gpu.create_texture(&desc, label)];
        *pair = Some((t, t.map(|t| gpu.texture_index(t)), (w, h)));
        true
    }

    /// The image the scene is drawn into, sized to the frame.
    pub fn scene_target(&mut self, gpu: &mut dyn Backend, w: u32, h: u32) -> TextureId {
        if self.targets.is_none_or(|t| t.2 != (w, h)) {
            if let Some((old, ..)) = self.targets.take() {
                old.into_iter().for_each(|t| gpu.destroy_texture(t));
            }
            let desc = TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: world_format(), render_target: true };
            let t = [gpu.create_texture(&desc, "post a"), gpu.create_texture(&desc, "post b")];
            self.targets = Some((t, t.map(|t| gpu.texture_index(t)), (w, h)));
        }
        self.targets.unwrap().0[0]
    }

    /// The images `copy_behind` copies the world into, sized to the frame: their bindless indices
    /// (color, depth); none without the copy pipeline.
    pub fn behind_targets(&mut self, gpu: &mut dyn Backend, w: u32, h: u32) -> Option<[u32; 2]> {
        self.copy_pipe?;
        if self.behind.is_none_or(|b| b.2 != (w, h)) {
            if let Some((old, ..)) = self.behind.take() {
                old.into_iter().for_each(|t| gpu.destroy_texture(t));
            }
            let (w, h) = (w.max(1), h.max(1));
            let color = gpu
                .create_texture(&TextureDesc { width: w, height: h, mips: 1, format: world_format(), render_target: true }, "behind color");
            let depth = gpu.create_texture(
                &TextureDesc { width: w, height: h, mips: 1, format: Format::Depth32, render_target: true },
                "behind depth",
            );
            self.behind = Some(([color, depth], [gpu.texture_index(color), gpu.texture_index(depth)], (w, h)));
        }
        self.behind.map(|b| b.1)
    }

    /// Copies the world drawn so far (the scene image and `depth`, its depth's bindless index) into
    /// the `behind_targets`, in a pass of its own: what is seen through (water) is drawn after, and
    /// reads them (`scene.behind`). The world's pass must be ended before and begun again after.
    pub fn copy_behind(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, frame: &FrameInfo, scene: BufferId, depth: u32) {
        let (Some(pipe), Some((_, idx, _)), Some((behind, ..))) = (self.copy_pipe, self.targets, self.behind) else { return };
        while self.copy_params.len() < gpu.frames_in_flight() {
            self.copy_params
                .push(gpu.create_buffer(size_of::<Pass>() as u64, usage::STORAGE | usage::HOST, "post copy pass"));
        }
        let pass = Pass {
            scene: idx[0],
            slot: 0,
            strength: 1.0,
            time: crate::shaders::time(),
            size: [frame.width as f32, frame.height as f32],
            depth,
            pad: 0,
            low: 0,
            spare: 0,
            ..Pass::default()
        };
        let buf = self.copy_params[frame.slot];
        gpu.mapped(buf)[..size_of::<Pass>()].copy_from_slice(bytemuck::bytes_of(&pass));
        list.push(Cmd::BeginPass {
            motion: None,
            color: Some((Target::Texture(behind[0]), Some([0.0; 4]))),
            depth: Some((behind[1], Some(0.0))),
        });
        list.pipeline(pipe);
        list.bind(crate::gpu_layout::bind::post::PASSES, buf);
        list.bind(crate::gpu_layout::bind::post::SCENE, scene);
        list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: 0 });
        list.push(Cmd::EndPass);
    }

    /// The bloom pyramid's images for a world of `w` x `h` with `levels` levels down (each half the
    /// one before, from half the world's size, none under 2 pixels), made again when either changes;
    /// the exposure texels with them.
    fn ensure_pyramid(&mut self, gpu: &mut dyn Backend, w: u32, h: u32, levels: usize) {
        if self
            .pyramid
            .as_ref()
            .is_some_and(|p| p.size == (w, h) && p.levels == levels)
        {
            return;
        }
        if let Some(old) = self.pyramid.take() {
            old.down
                .iter()
                .chain(old.up.iter())
                .for_each(|t| gpu.destroy_texture(t.0));
            old.exposure.iter().for_each(|t| gpu.destroy_texture(t.0));
        }
        let mut make = |size: (u32, u32), label: &str| {
            let t = gpu.create_texture(
                &TextureDesc { width: size.0, height: size.1, mips: 1, format: Format::Rgba16Float, render_target: true },
                label,
            );
            (t, gpu.texture_index(t), size)
        };
        let mut down = Vec::new();
        let mut size = (w.div_ceil(2).max(1), h.div_ceil(2).max(1));
        while down.len() < levels.max(1) {
            down.push(make(size, "bloom down"));
            if size.0.min(size.1) < 4 {
                break;
            }
            size = (size.0.div_ceil(2), size.1.div_ceil(2));
        }
        let up = down[..down.len() - 1]
            .iter()
            .map(|d| make(d.2, "bloom up"))
            .collect();
        let e = [make((1, 1), "exposure a"), make((1, 1), "exposure b")];
        self.pyramid = Some(Pyramid { size: (w, h), down, up, exposure: e.map(|t| (t.0, t.1)), flip: 0, levels, fresh: true });
        self.last_frame = None;
    }

    /// Runs the effects over the scene; the last one draws onto the screen and leaves that pass
    /// open for the UI. `frame` is the world's size: the screen's, or less (`render_scale`), when
    /// the last pass also scales it up. With no effect on, that is one plain pass.
    /// `scene` is the buffer the world was drawn with (`gpu_layout::Scene`); `scaled`, that the
    /// world is drawn smaller than the screen.
    pub fn apply(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        scene: Option<BufferId>,
        depth: u32,
        scaled: bool,
    ) {
        let (Some(pipe), Some((tex, idx, _))) = (self.pipeline, self.targets) else { return };
        let scene = match scene {
            Some(b) => b,
            None => *self.no_scene.get_or_insert_with(|| {
                let size = size_of::<crate::gpu_layout::Scene>();
                let b = gpu.create_buffer(size as u64, usage::STORAGE | usage::HOST, "post no scene");
                gpu.mapped(b)[..size].fill(0);
                b
            }),
        };
        while self.params.len() < gpu.frames_in_flight() {
            self.params
                .push(gpu.create_buffer((size_of::<Pass>() * MAX_PASSES) as u64, usage::STORAGE | usage::HOST, "post passes"));
        }
        let time = crate::shaders::time();
        // The denoiser's check before the path tracer exists (`denoise::test_on`): its result is
        // this frame's world from here on.
        let (mut world_tex, mut world_idx) = (tex[0], idx[0]);
        if crate::denoise::test_on() && self.denoiser.ready() {
            if let Some(d) = self.depth_tex {
                let motion = self
                    .motion_tex
                    .filter(|m| m.2 == (frame.width, frame.height))
                    .map(|m| m.0);
                if let Some(out) = self
                    .denoiser
                    .run_test(gpu, list, frame.slot, scene, tex[0], d, motion, (frame.width, frame.height))
                {
                    (world_tex, world_idx) = (out, gpu.texture_index(out));
                }
            }
        }
        // Temporal anti-aliasing first: this frame's world gathered into the picture so far, which
        // all after it read in the world's place (`src`): the bloom, the half-size parts and the
        // effects, so their sharpening (chiacraft's `sharpen_delta`) works on the gathered picture,
        // not on this frame's aliasing. An upscaler's result (`set_upscaled`) takes its place.
        let mut src = world_idx;
        let mut taa_entries: Vec<Pass> = Vec::new();
        // (target, pipeline, entry among `taa_entries`)
        let mut taa_draws: Vec<(TextureId, Option<TextureId>, PipelineId, usize)> = Vec::new();
        self.inputs = None;
        // The size the effects run at (the screen's after an upscaler), whether they still scale
        // the world up, and their two images.
        let mut fx = *frame;
        let mut scaled = scaled;
        let (mut tex, mut idx) = (tex, idx);
        let mut up_cmd: Option<Cmd> = None;
        let motion_now = self
            .motion_tex
            .filter(|m| m.2 == (frame.width, frame.height));
        if let (Some(u), Some((_, fill)), Some((_, motion_idx, _)), Some(depth_tex)) =
            (self.upscale.take(), self.taa_pipes, motion_now, self.depth_tex)
        {
            self.upscaled = None;
            self.taa_held = false;
            let (w, h) = (frame.width.max(1), frame.height.max(1));
            let rt = |gpu: &mut dyn Backend, w: u32, h: u32, label: &str| {
                gpu.create_texture(&TextureDesc { width: w, height: h, mips: 1, format: Format::Rgba16Float, render_target: true }, label)
            };
            // The whole motion image at the world's size (`fs_taa_motion`), as DLSS reads it.
            if self.full_motion.is_none_or(|t| t.2 != (w, h)) {
                if let Some((old, ..)) = self.full_motion.take() {
                    gpu.destroy_texture(old);
                }
                let t = rt(gpu, w, h, "full motion");
                self.full_motion = Some((t, gpu.texture_index(t), (w, h)));
            }
            let (full, ..) = self.full_motion.unwrap();
            let entry =
                Pass { scene: world_idx, slot: 0, time, size: [w as f32, h as f32], depth, bloom: motion_idx + 1, ..Pass::default() };
            taa_entries.push(entry);
            taa_draws.push((full, None, fill, taa_entries.len() - 1));
            // Ray Reconstruction's guides: from the path tracer's G-buffer when it drew this frame
            // (the normals and albedo as they are, the specular albedo and hit distance drawn
            // from it), else estimated from the world's picture and depth.
            let mut g = [NO_TEXTURE; 3];
            let mut hit = NO_TEXTURE;
            let pt = self.pt_guides.take();
            if let (true, Some(pt), Some(pt_pipe), Some(pt_pipe_a)) = (u.reconstruct, pt, self.pt_pipe, self.pt_pipe_a) {
                if self.guides.is_none_or(|t| t.1 != (w, h)) {
                    if let Some((old, _)) = self.guides.take() {
                        old.into_iter().for_each(|t| gpu.destroy_texture(t));
                    }
                    let t = [rt(gpu, w, h, "guide normals"), rt(gpu, w, h, "guide albedo"), rt(gpu, w, h, "guide specular")];
                    self.guides = Some((t, (w, h)));
                }
                let ga = self.guides.unwrap().0;
                // Normals and albedo: the G-buffer's, estimated from the picture where it drew nothing.
                taa_entries.push(Pass {
                    scene: idx[0],
                    size: [w as f32, h as f32],
                    depth,
                    low: gpu.texture_index(pt.normal) + 1,
                    bloom: gpu.texture_index(pt.albedo) + 1,
                    ..Pass::default()
                });
                taa_draws.push((ga[0], Some(ga[1]), pt_pipe_a, taa_entries.len() - 1));
                if self.pt_targets.is_none_or(|t| t.1 != (w, h)) {
                    if let Some((old, _)) = self.pt_targets.take() {
                        old.into_iter().for_each(|t| gpu.destroy_texture(t));
                    }
                    self.pt_targets = Some(([rt(gpu, w, h, "guide pt specular"), rt(gpu, w, h, "guide pt hit distance")], (w, h)));
                }
                let t = self.pt_targets.unwrap().0;
                let pt_entry = Pass {
                    scene: gpu.texture_index(pt.spec),
                    size: [w as f32, h as f32],
                    depth,
                    low: gpu.texture_index(ga[0]) + 1,
                    bloom: pt.specular.map_or(0, |s| gpu.texture_index(s) + 1),
                    exposure: gpu.texture_index(pt.normal) + 1,
                    ..Pass::default()
                };
                taa_entries.push(pt_entry);
                taa_draws.push((t[0], Some(t[1]), pt_pipe, taa_entries.len() - 1));
                g = [ga[0], ga[1], t[0]];
                hit = if pt.specular.is_some() { t[1] } else { NO_TEXTURE };
            } else if let (true, Some((guide_pipe, spec_pipe))) = (u.reconstruct, self.guide_pipes) {
                if self.guides.is_none_or(|t| t.1 != (w, h)) {
                    if let Some((old, _)) = self.guides.take() {
                        old.into_iter().for_each(|t| gpu.destroy_texture(t));
                    }
                    let t = [rt(gpu, w, h, "guide normals"), rt(gpu, w, h, "guide albedo"), rt(gpu, w, h, "guide specular")];
                    self.guides = Some((t, (w, h)));
                }
                g = self.guides.unwrap().0;
                taa_draws.push((g[0], Some(g[1]), guide_pipe, taa_entries.len() - 1));
                taa_draws.push((g[2], None, spec_pipe, taa_entries.len() - 1));
            }
            let (ow, oh) = (u.output.0.max(1), u.output.1.max(1));
            if self.up_out.is_none_or(|t| t.2 != (ow, oh)) {
                if let Some((old, ..)) = self.up_out.take() {
                    gpu.destroy_texture(old);
                }
                let t = gpu.create_upscale_target(ow, oh, "upscaled");
                self.up_out = Some((t, gpu.texture_index(t), (ow, oh)));
            }
            let (out, out_idx, _) = self.up_out.unwrap();
            if self.up_targets.is_none_or(|t| t.2 != (ow, oh)) {
                if let Some((old, ..)) = self.up_targets.take() {
                    old.into_iter().for_each(|t| gpu.destroy_texture(t));
                }
                let desc = TextureDesc { width: ow, height: oh, mips: 1, format: world_format(), render_target: true };
                let t = [gpu.create_texture(&desc, "post up a"), gpu.create_texture(&desc, "post up b")];
                self.up_targets = Some((t, t.map(|t| gpu.texture_index(t)), (ow, oh)));
            }
            let held = (u.mode, u.reconstruct, (w, h), (ow, oh));
            let fresh = self.up_held != Some(held);
            self.up_held = Some(held);
            let j = jitter();
            let mut params = u.params;
            params.mode = u.mode as u32;
            params.reset = (params.reset != 0 || fresh) as u32;
            params.hdr = hdr() as u32;
            params.render = [w, h];
            params.output = [ow, oh];
            params.jitter = [j.x * w as f32 * 0.5, -j.y * h as f32 * 0.5];
            params.mv_scale = [1.0, 1.0];
            let start = list.data.len() as u32;
            list.data.extend_from_slice(bytemuck::bytes_of(&params));
            up_cmd = Some(Cmd::Upscale {
                color: world_tex,
                depth: depth_tex,
                motion: full,
                output: out,
                normals: g[0],
                albedo: g[1],
                specular: g[2],
                hit_distance: hit,
                start,
            });
            src = out_idx;
            fx = FrameInfo { width: ow, height: oh, slot: frame.slot };
            scaled = false;
            let t = self.up_targets.unwrap();
            (tex, idx) = (t.0, t.1);
        } else if let Some(up) = self.upscaled.take() {
            self.up_held = None;
            src = up;
            self.taa_held = false;
        } else if let (true, Some((resolve, fill))) = (self.taa_on(), self.taa_pipes) {
            // Without the motion image (`core:aa` 2): all from the depth (`bloom` 0).
            let (motion, motion_idx) = self
                .motion_tex
                .map_or((TextureId(u32::MAX), u32::MAX), |m| (m.0, m.1));
            let anew = Self::ensure_pair(gpu, &mut self.history, frame.width, frame.height, "temporal aa");
            if Self::ensure_pair(gpu, &mut self.counts, frame.width, frame.height, "temporal aa count") || anew {
                self.taa_held = false;
            }
            let (hist, hidx, _) = self.history.unwrap();
            let (counts, cidx, _) = self.counts.unwrap();
            let (cur, last) = (self.taa_cur, 1 - self.taa_cur);
            let entry = Pass {
                scene: world_idx,
                slot: 0,
                strength: self.taa_blend,
                time,
                size: [frame.width as f32, frame.height as f32],
                depth,
                low: if self.taa_held { hidx[last] + 1 } else { 0 },
                bloom: motion_idx.wrapping_add(1),
                exposure: if self.taa_held { cidx[last] + 1 } else { 0 },
                ..Pass::default()
            };
            taa_entries.push(entry);
            taa_draws.push((hist[cur], Some(counts[cur]), resolve, 0));
            src = hidx[cur];
            self.taa_cur = last;
            self.taa_held = true;
            let mut full = motion;
            if self.export_motion {
                if self
                    .full_motion
                    .is_none_or(|t| t.2 != (frame.width, frame.height))
                {
                    if let Some((old, ..)) = self.full_motion.take() {
                        gpu.destroy_texture(old);
                    }
                    let t = gpu.create_texture(
                        &TextureDesc {
                            width: frame.width.max(1),
                            height: frame.height.max(1),
                            mips: 1,
                            format: Format::Rgba16Float,
                            render_target: true,
                        },
                        "full motion",
                    );
                    self.full_motion = Some((t, gpu.texture_index(t), (frame.width, frame.height)));
                }
                let (t, ..) = self.full_motion.unwrap();
                taa_entries.push(entry);
                taa_draws.push((t, None, fill, 1));
                full = t;
            }
            let j = jitter();
            self.inputs = Some(TemporalInputs {
                color: world_tex,
                depth: self.depth_tex.unwrap_or(TextureId(u32::MAX)),
                motion: full,
                size: (frame.width, frame.height),
                jitter: [j.x * frame.width as f32 * 0.5, -j.y * frame.height as f32 * 0.5],
                reset: entry.low == 0,
            });
        } else {
            self.taa_held = false;
            self.up_held = None;
        }
        // No effect on: one pass that only draws the scene across the screen (slot 0 returns it).
        let plain = [(String::new(), 1.0)];
        let active = if self.active.is_empty() { &plain[..] } else { &self.active[..] };
        let n = active.len().min(MAX_PASSES);
        let passes: Vec<Pass> = active[..n]
            .iter()
            .enumerate()
            .map(|(i, (name, strength))| Pass {
                scene: if i == 0 { src } else { idx[i % 2] },
                slot: self.slots.slot(name),
                strength: *strength,
                time,
                size: [fx.width as f32, fx.height as f32],
                depth,
                pad: 0,
                low: 0,
                spare: 0,
                ..Pass::default()
            })
            .collect();
        let mut passes = passes;
        // The half-size parts first: each into its effect's own image, which that effect reads.
        let mut lows: Vec<TextureId> = Vec::new();
        if self.low_pipe.is_some() && self.use_low {
            let (w, h) = (fx.width.div_ceil(2).max(1), fx.height.div_ceil(2).max(1));
            for (i, (name, _)) in active[..n].iter().enumerate() {
                let slot = self.low_slots.slot(name);
                if slot == 0 {
                    continue;
                }
                let at = match self.low_targets.iter().position(|t| t.0 == *name) {
                    Some(at) => at,
                    None => {
                        self.low_targets
                            .push((name.clone(), TextureId(u32::MAX), 0, (0, 0)));
                        self.low_targets.len() - 1
                    }
                };
                if self.low_targets[at].3 != (w, h) {
                    // The GPU frees the old image once no frame in flight still draws with it.
                    if self.low_targets[at].1 != TextureId(u32::MAX) {
                        gpu.destroy_texture(self.low_targets[at].1);
                    }
                    let t = gpu.create_texture(
                        &TextureDesc { width: w, height: h, mips: 1, format: Format::Rgba16Float, render_target: true },
                        "post low",
                    );
                    self.low_targets[at] = (name.clone(), t, gpu.texture_index(t), (w, h));
                }
                passes[i].low = self.low_targets[at].2 + 1;
                lows.push(self.low_targets[at].1);
            }
        }
        let low_entries: Vec<Pass> = active[..n]
            .iter()
            .filter(|(name, _)| self.low_pipe.is_some() && self.use_low && self.low_slots.slot(name) != 0)
            .map(|(name, strength)| Pass {
                scene: src,
                slot: self.low_slots.slot(name),
                strength: *strength,
                time,
                size: [fx.width as f32, fx.height as f32],
                depth,
                pad: 0,
                low: 0,
                spare: 0,
                ..Pass::default()
            })
            .collect();
        // Merged (`packs::merge_effects`): one pass runs them all, the first's `pad` their count,
        // reading the entries after it. `draws`: the entry each pass starts from.
        let mut draws: Vec<u32> = if n > 1 && crate::packs::merge_effects() {
            passes[0].pad = n as u32;
            vec![0]
        } else {
            (0..n as u32).collect()
        };
        // With the world drawn smaller than the screen, the effects stay at the world's size and a
        // plain pass (slot 0) scales the last one's image up. As the last pass, an effect drew
        // straight onto the screen and ran per screen pixel: with the effects merged into one, all
        // of them, 2.25 times the work at a 0.67 render scale (the Mac's effects after a flight over
        // water took 6-7.8 ms, Planner 11-04). Its entry goes after all the effects': in the place
        // of the second (Perf 7), the merged pass read it and the half-size parts' entries as its
        // other effects, so under a render scale (the Mac) only the first effect ran as itself.
        if scaled && !self.active.is_empty() && passes.len() < MAX_PASSES {
            passes.push(Pass { scene: idx[draws.len() % 2], slot: 0, strength: 1.0, pad: 0, low: 0, ..passes[0] });
            draws.push((passes.len() - 1) as u32);
        }
        let buf = self.params[frame.slot];
        // The main passes, then the half-size parts' entries after them (`lows` holds their index).
        // The last pass finishes the picture in the HDR world (a pack's tone curve, `spare` 1).
        if hdr() && n > 0 {
            passes[n - 1].spare = 1;
        }
        let first_low = passes.len();
        passes.extend(low_entries);
        // The HDR world's bloom pyramid and exposure, after the half-size parts' entries: their
        // images, pipelines and entries (`pyramid_draws`), and the effects told where to find them.
        let mut pyramid_draws: Vec<(TextureId, PipelineId, u32)> = Vec::new();
        let pipes = self.bloom_pipes.filter(|_| hdr() && self.bloom_levels > 0);
        BLOOM.store(pipes.is_some(), std::sync::atomic::Ordering::Relaxed);
        if let Some(pipes) = pipes {
            self.ensure_pyramid(gpu, fx.width, fx.height, self.bloom_levels as usize);
            let now = std::time::Instant::now();
            let dt = self
                .last_frame
                .map_or(0.0, |t| (now - t).as_secs_f32().min(0.5));
            self.last_frame = Some(now);
            let py = self.pyramid.as_mut().unwrap();
            py.flip ^= 1;
            let entry = |scene: u32, low: u32, level: usize, size: (u32, u32)| Pass {
                scene,
                slot: 0,
                strength: dt,
                time,
                size: [size.0 as f32, size.1 as f32],
                depth,
                pad: level as u32,
                low,
                ..Pass::default()
            };
            let levels = py.down.len();
            for i in 0..levels {
                let from = if i == 0 { src } else { py.down[i - 1].1 };
                passes.push(entry(from, 0, i, py.down[i].2));
                pyramid_draws.push((py.down[i].0, pipes[0], (passes.len() - 1) as u32));
            }
            // Back up: each level below blended with the wider one over it (the narrowest is its
            // own), so the widest holds every scale.
            for i in (0..py.up.len()).rev() {
                let wider = if i + 1 == py.up.len() { py.down[i + 1].1 } else { py.up[i + 1].1 };
                passes.push(entry(py.down[i].1, wider + 1, i, py.up[i].2));
                pyramid_draws.push((py.up[i].0, pipes[1], (passes.len() - 1) as u32));
            }
            // The exposure from the narrowest level and last frame's texel.
            let (now_tex, now_idx) = py.exposure[py.flip];
            let before = py.exposure[1 - py.flip].1;
            passes.push(Pass { spare: py.fresh as u32, ..entry(py.down[levels - 1].1, before + 1, levels - 1, (1, 1)) });
            py.fresh = false;
            pyramid_draws.push((now_tex, pipes[2], (passes.len() - 1) as u32));
            let bloom = py.up.first().map_or(py.down[0].1, |u| u.1) + 1;
            for p in &mut passes[..n] {
                p.bloom = bloom;
                p.exposure = now_idx + 1;
            }
        }
        // Temporal anti-aliasing's entries last, kept whatever else is cut.
        passes.truncate(MAX_PASSES - taa_entries.len());
        let first_taa = passes.len();
        passes.extend(taa_entries);
        gpu.mapped(buf)[..passes.len() * size_of::<Pass>()].copy_from_slice(bytemuck::cast_slice(&passes));
        for (target, beside, pipe, k) in taa_draws {
            list.push(Cmd::BeginPass {
                motion: beside.map(|t| (t, Some([0.0; 4]))),
                color: Some((Target::Texture(target), Some([0.0; 4]))),
                depth: None,
            });
            list.pipeline(pipe);
            list.bind(crate::gpu_layout::bind::post::PASSES, buf);
            list.bind(crate::gpu_layout::bind::post::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: (first_taa + k) as u32 });
            list.push(Cmd::EndPass);
        }
        // The upscaler, once its motion image and guides are drawn.
        if let Some(c) = up_cmd {
            list.push(c);
        }
        if let Some(low_pipe) = self.low_pipe {
            for (k, target) in lows.iter().enumerate() {
                let entry = first_low + k;
                if entry >= passes.len() {
                    break;
                }
                list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(*target), Some([0.0; 4]))), depth: None });
                list.pipeline(low_pipe);
                list.bind(crate::gpu_layout::bind::post::PASSES, buf);
                list.bind(crate::gpu_layout::bind::post::SCENE, scene);
                list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: entry as u32 });
                list.push(Cmd::EndPass);
            }
        }
        for (target, pipe, entry) in pyramid_draws {
            if entry as usize >= passes.len() {
                break;
            }
            list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(target), Some([0.0; 4]))), depth: None });
            list.pipeline(pipe);
            list.bind(crate::gpu_layout::bind::post::PASSES, buf);
            list.bind(crate::gpu_layout::bind::post::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: entry });
            list.push(Cmd::EndPass);
        }
        let n = draws.len();
        for (i, entry) in draws.into_iter().enumerate() {
            let target = if i + 1 == n { Target::Surface } else { Target::Texture(tex[(i + 1) % 2]) };
            list.push(Cmd::BeginPass { motion: None, color: Some((target, Some([0.0; 4]))), depth: None });
            // Onto the screen from the HDR world: the pipeline of the screen's format.
            list.pipeline(if i + 1 == n { self.final_pipe.unwrap_or(pipe) } else { pipe });
            list.bind(crate::gpu_layout::bind::post::PASSES, buf);
            list.bind(crate::gpu_layout::bind::post::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: entry });
            if i + 1 < n {
                list.push(Cmd::EndPass);
            }
        }
    }
}

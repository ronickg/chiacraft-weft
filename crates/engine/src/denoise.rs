//! Our own denoiser for the path tracer (chiacraft, `reports/Path tracing plan.md`): SVGF-style
//! (`denoise.wgsl`) on any GPU with ray queries, where DLSS Ray Reconstruction isn't used.
//!
//! The path tracer (the RT session's) draws, at the world's size, its diffuse and specular light
//! with the albedo divided out, beside the G-buffer the world's pass wrote (albedo, normal and
//! roughness, F0), then calls [`Denoiser::run`] before what is seen through the world (water,
//! glass) is drawn. `run` gathers each light over frames along TAA's motion, filters it by its
//! variance in a few à-trous steps that keep edges, multiplies the albedo back in and adds it onto
//! the scene image, which holds the emission and the sky. With `core:pt_denoiser` 0 (or under Ray
//! Reconstruction, which denoises the whole frame itself) it only does that last step on the raw light.
//!
//! The layout was agreed with the RT session (shared notes, "pt-denoisers"): every pass is a
//! fullscreen draw into Rgba16Float images, two at a time (the colour and the motion slot), so it
//! needs nothing past what the render API has.

use crate::assets::Assets;
use crate::gpu_layout::DenoisePass;
use weft_render::{
    Backend, Blend, BufferId, Cmd, CommandList, Format, PipelineDesc, PipelineId, Target, TextureDesc, TextureId, Topology, usage,
};

/// What the path tracer drew this frame, for [`Denoiser::run`]. All at `size` (the world's).
#[derive(Clone, Copy, Debug)]
pub struct PtFrame {
    pub size: (u32, u32),
    /// The frame-in-flight slot (`FrameInfo::slot`).
    pub slot: usize,
    /// The world's `Scene` buffer (its matrices, last frame's camera, the jitter).
    pub scene: BufferId,
    /// The world's depth (reverse Z, 0 the sky) and its raw motion image (`MOTION_FORMAT`).
    pub depth: TextureId,
    pub motion: Option<TextureId>,
    /// The G-buffer: rgb diffuse albedo (metals 0); xyz the shading normal (world axes) and w
    /// perceptual roughness; rgb F0.
    pub albedo: TextureId,
    pub normal: TextureId,
    pub spec: TextureId,
    /// The path tracer's light: diffuse over the albedo, specular over `pt_spec_albedo` (alpha its
    /// hit distance).
    pub diffuse: TextureId,
    pub specular: TextureId,
    /// The scene image the light is added onto (it holds the emission and the sky).
    pub target: TextureId,
    /// Nothing of last frame is to be used (a teleport, a new world).
    pub reset: bool,
}

/// `core:pt_denoiser`: 0 the raw light, 1 ours.
pub const RAW: u32 = 0;
pub const SVGF: u32 = 1;

/// The à-trous steps (1, 2, 4, 8, 16 texels apart): 5 as SVGF's paper.
const STEPS: u32 = 5;

struct Pipes {
    temporal: PipelineId,
    moments: PipelineId,
    variance: PipelineId,
    atrous: PipelineId,
    composite: PipelineId,
}

type Image = (TextureId, u32);

/// Its images at one size: the gathered light, moments and guide (this frame's and last frame's,
/// in turn), and the à-trous steps' pair.
struct Images {
    size: (u32, u32),
    diffuse: [Image; 2],
    specular: [Image; 2],
    moments: [Image; 2],
    guide: [Image; 2],
    step_diffuse: [Image; 2],
    step_specular: [Image; 2],
}

pub struct Denoiser {
    pipes: Option<Pipes>,
    images: Option<Images>,
    /// Which of each pair this frame writes, and whether the other holds last frame's.
    cur: usize,
    held: bool,
    params: Vec<BufferId>,
    /// `core:pt_denoiser`, set by the app.
    pub mode: u32,
    /// Ray Reconstruction denoises this frame (`core:dlss_rr` under DLSS): the raw light then.
    pub reconstruct: bool,
}

const MAX_PASSES: usize = 4 + STEPS as usize;

impl Denoiser {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Denoiser {
        let mut d = Denoiser { pipes: None, images: None, cur: 0, held: false, params: Vec::new(), mode: SVGF, reconstruct: false };
        d.reload(gpu, assets);
        d
    }

    /// Its pipelines from `denoise.wgsl` (a pack's, else the engine's).
    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        if let Some(p) = self.pipes.take() {
            for id in [p.temporal, p.moments, p.variance, p.atrous, p.composite] {
                gpu.destroy_pipeline(id);
            }
        }
        self.held = false;
        let made = crate::packs::shader(assets, "denoise.wgsl")
            .map_err(|e| e.0)
            .and_then(|s| {
                let mut ids = Vec::new();
                let mut make = |label: &str, fragment: &str, two: bool, blend: Blend, color: Format| -> Result<PipelineId, String> {
                    gpu.create_pipeline(&PipelineDesc {
                        label,
                        shader: &s,
                        vertex: "vs",
                        fragment: Some(fragment),
                        color: Some(color),
                        depth: None,
                        blend,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: two,
                    })
                    .map_err(|e| e.0)
                };
                let f = Format::Rgba16Float;
                for (label, entry, two, blend, color) in [
                    ("denoise temporal", "fs_temporal", true, Blend::None, f),
                    ("denoise moments", "fs_moments", true, Blend::None, f),
                    ("denoise variance", "fs_variance", true, Blend::None, f),
                    ("denoise atrous", "fs_atrous", true, Blend::None, f),
                    ("denoise composite", "fs_composite", false, Blend::Additive, crate::post::world_format()),
                ] {
                    ids.push(make(label, entry, two, blend, color)?);
                }
                Ok(ids)
            });
        match made {
            Ok(ids) => {
                self.pipes = Some(Pipes { temporal: ids[0], moments: ids[1], variance: ids[2], atrous: ids[3], composite: ids[4] });
            }
            Err(e) => log::error!("path tracing denoiser: {e}"),
        }
    }

    /// Whether `run` can draw (its pipelines were made).
    pub fn ready(&self) -> bool {
        self.pipes.is_some()
    }

    fn ensure_images(&mut self, gpu: &mut dyn Backend, (w, h): (u32, u32)) {
        if self.images.as_ref().is_some_and(|i| i.size == (w, h)) {
            return;
        }
        if let Some(old) = self.images.take() {
            for (t, _) in [old.diffuse, old.specular, old.moments, old.guide, old.step_diffuse, old.step_specular]
                .into_iter()
                .flatten()
            {
                gpu.destroy_texture(t);
            }
        }
        let mut pair = |label: &str| -> [Image; 2] {
            [0, 1].map(|k| {
                let t = gpu.create_texture(
                    &TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: Format::Rgba16Float, render_target: true },
                    &format!("{label} {k}"),
                );
                (t, gpu.texture_index(t))
            })
        };
        self.images = Some(Images {
            size: (w, h),
            diffuse: pair("denoise diffuse"),
            specular: pair("denoise specular"),
            moments: pair("denoise moments"),
            guide: pair("denoise guide"),
            step_diffuse: pair("denoise step diffuse"),
            step_specular: pair("denoise step specular"),
        });
        self.held = false;
    }

    /// Denoises `f`'s light (or not: `mode`, `reconstruct`) and adds it onto `f.target`. Outside a
    /// pass; leaves none open.
    pub fn run(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, f: &PtFrame) {
        let Some(p) = &self.pipes else { return };
        let (temporal, moments_pipe, variance, atrous, composite) = (p.temporal, p.moments, p.variance, p.atrous, p.composite);
        while self.params.len() < gpu.frames_in_flight() {
            let size = (size_of::<DenoisePass>() * MAX_PASSES) as u64;
            self.params
                .push(gpu.create_buffer(size, usage::STORAGE | usage::HOST, "denoise passes"));
        }
        let buf = self.params[f.slot % self.params.len()];
        let ix = |t: TextureId| gpu.texture_index(t);
        let base = DenoisePass {
            albedo: ix(f.albedo),
            normal: ix(f.normal),
            spec: ix(f.spec),
            depth: ix(f.depth),
            motion: f.motion.map_or(0, |m| ix(m) + 1),
            size: [f.size.0 as f32, f.size.1 as f32],
            alpha: 0.05,
            max_len: 32.0,
            phi_color: 4.0,
            phi_normal: 128.0,
            ..DenoisePass::default()
        };
        let (raw_d, raw_s) = (ix(f.diffuse), ix(f.specular));
        // (target, second target, pipeline, entry)
        let mut draws: Vec<(TextureId, Option<TextureId>, PipelineId, usize)> = Vec::new();
        let mut entries: Vec<DenoisePass> = Vec::new();
        let mut light = (raw_d, raw_s);
        if self.mode == SVGF && !self.reconstruct {
            self.ensure_images(gpu, f.size);
            let im = self.images.as_ref().unwrap();
            let (cur, last) = (self.cur, 1 - self.cur);
            let held = self.held && !f.reset;
            let prev = DenoisePass {
                prev_diffuse: im.diffuse[last].1,
                prev_specular: im.specular[last].1,
                prev_moments: im.moments[last].1,
                prev_guide: if held { im.guide[last].1 + 1 } else { 0 },
                reset: !held as u32,
                ..base
            };
            entries.push(DenoisePass { diffuse: raw_d, specular: raw_s, ..prev });
            draws.push((im.diffuse[cur].0, Some(im.specular[cur].0), temporal, entries.len() - 1));
            entries.push(DenoisePass { diffuse: raw_d, specular: raw_s, ..prev });
            draws.push((im.moments[cur].0, Some(im.guide[cur].0), moments_pipe, entries.len() - 1));
            entries.push(DenoisePass { diffuse: im.diffuse[cur].1, specular: im.specular[cur].1, moments: im.moments[cur].1, ..base });
            draws.push((im.step_diffuse[0].0, Some(im.step_specular[0].0), variance, entries.len() - 1));
            let mut from = 0;
            for k in 0..STEPS {
                let to = 1 - from;
                entries.push(DenoisePass { diffuse: im.step_diffuse[from].1, specular: im.step_specular[from].1, step: 1 << k, ..base });
                draws.push((im.step_diffuse[to].0, Some(im.step_specular[to].0), atrous, entries.len() - 1));
                from = to;
            }
            light = (im.step_diffuse[from].1, im.step_specular[from].1);
            self.cur = last;
            self.held = true;
        } else {
            self.held = false;
        }
        entries.push(DenoisePass { diffuse: light.0, specular: light.1, ..base });
        let comp_entry = entries.len() - 1;
        gpu.mapped(buf)[..entries.len() * size_of::<DenoisePass>()].copy_from_slice(bytemuck::cast_slice(&entries));
        let bind = |list: &mut CommandList, pipe: PipelineId| {
            list.pipeline(pipe);
            list.bind(crate::gpu_layout::bind::denoise::PASSES, buf);
            list.bind(crate::gpu_layout::bind::denoise::SCENE, f.scene);
        };
        for (target, beside, pipe, k) in draws {
            list.push(Cmd::BeginPass {
                motion: beside.map(|t| (t, Some([0.0; 4]))),
                color: Some((Target::Texture(target), Some([0.0; 4]))),
                depth: None,
            });
            bind(list, pipe);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: k as u32 });
            list.push(Cmd::EndPass);
        }
        // Onto the scene image as it is (loaded, not cleared).
        list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(f.target), None)), depth: None });
        bind(list, composite);
        list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: comp_entry as u32 });
        list.push(Cmd::EndPass);
    }
}

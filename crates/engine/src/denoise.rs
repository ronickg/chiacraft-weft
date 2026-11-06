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
    /// `target` is the path tracer's own light image (Rgba16Float, `WorldRenderer::pt_light`):
    /// cleared and written with the light alone (0 where nothing was traced), which its finish pass
    /// then puts into the scene with the air's light and the pack's `display`. Else the light is
    /// added onto `target` as it is.
    pub write: bool,
}

/// `core:pt_denoiser`: 0 the raw light, 1 ours.
pub const RAW: u32 = 0;
pub const SVGF: u32 = 1;

/// The most à-trous steps (1, 2, 4, 8, 16 texels apart), as SVGF's paper; 4 by default (`steps`).
const STEPS: u32 = 5;

struct Pipes {
    temporal: PipelineId,
    moments: PipelineId,
    variance: PipelineId,
    atrous: PipelineId,
    composite: PipelineId,
    /// The same into a cleared Rgba16Float image (`PtFrame::write`).
    composite_write: PipelineId,
    /// The test's noisy light and normals from today's picture (`run_test`).
    test: PipelineId,
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
    /// The first à-trous step, kept as next frame's history (SVGF's feedback).
    feed_diffuse: [Image; 2],
    feed_specular: [Image; 2],
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
    /// How far the light may differ between texels the à-trous steps join (standard deviations of
    /// its noise) and how many steps run (1-5, each twice as wide): `core:pt_dn_phi`, `core:pt_dn_steps`.
    pub phi_color: f32,
    pub steps: u32,
    /// `core:pt_dn_debug` (checks only): what the composite shows instead of the light (1 frames
    /// gathered, 2 the raw diffuse light, 3 gathered, 4 filtered, 5 its noise, 6 what was lit).
    pub debug: u32,
    /// The test's images (`run_test`): noisy light, normals, zero specular and F0, the result.
    test: Option<((u32, u32), [TextureId; 4])>,
    test_frame: u32,
    /// `run_test` is running: the sky (depth 0) comes through from the picture.
    test_sky: bool,
}

static TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `core:pt_denoise_test` (live) or `WEFT_PT_DENOISE_TEST=1`: the denoiser on today's frame
/// before the path tracer exists (`run_test`), set by the app.
pub fn set_test(on: bool) {
    let on = on || std::env::var_os("WEFT_PT_DENOISE_TEST").is_some_and(|v| v != "0");
    if TEST.swap(on, std::sync::atomic::Ordering::Relaxed) != on {
        log::info!("denoise: test on today's frame {}", if on { "on" } else { "off" });
    }
}

pub fn test_on() -> bool {
    TEST.load(std::sync::atomic::Ordering::Relaxed)
}

const MAX_PASSES: usize = 4 + STEPS as usize;

impl Denoiser {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Denoiser {
        let mut d = Denoiser {
            pipes: None,
            images: None,
            cur: 0,
            held: false,
            params: Vec::new(),
            mode: SVGF,
            reconstruct: false,
            // Chosen at the PC (ptd_rrt5, 11-06): the hill's block shadows kept best without noise.
            phi_color: 1.0,
            steps: 4,
            debug: 0,
            test: None,
            test_frame: 0,
            test_sky: false,
        };
        d.reload(gpu, assets);
        d
    }

    /// Its pipelines from `denoise.wgsl` (a pack's, else the engine's).
    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        if let Some(p) = self.pipes.take() {
            for id in [p.temporal, p.moments, p.variance, p.atrous, p.composite, p.test, p.composite_write] {
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
                    ("denoise composite write", "fs_composite", false, Blend::None, f),
                    ("denoise test", "fs_test", true, Blend::None, f),
                ] {
                    ids.push(make(label, entry, two, blend, color)?);
                }
                Ok(ids)
            });
        match made {
            Ok(ids) => {
                self.pipes = Some(Pipes {
                    temporal: ids[0],
                    moments: ids[1],
                    variance: ids[2],
                    atrous: ids[3],
                    composite: ids[4],
                    composite_write: ids[5],
                    test: ids[6],
                });
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
            for (t, _) in [
                old.diffuse,
                old.specular,
                old.moments,
                old.guide,
                old.step_diffuse,
                old.step_specular,
                old.feed_diffuse,
                old.feed_specular,
            ]
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
            feed_diffuse: pair("denoise feed diffuse"),
            feed_specular: pair("denoise feed specular"),
        });
        self.held = false;
    }

    /// Denoises `f`'s light (or not: `mode`, `reconstruct`) and adds it onto `f.target`. Outside a
    /// pass; leaves none open.
    pub fn run(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, f: &PtFrame) {
        let Some(p) = &self.pipes else { return };
        let (temporal, moments_pipe, variance, atrous) = (p.temporal, p.moments, p.variance, p.atrous);
        let composite = if f.write { p.composite_write } else { p.composite };
        while self.params.len() < gpu.frames_in_flight() {
            // One more for `run_test`'s own entry.
            let size = (size_of::<DenoisePass>() * (MAX_PASSES + 1)) as u64;
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
            phi_color: self.phi_color,
            phi_normal: 128.0,
            ..DenoisePass::default()
        };
        let (raw_d, raw_s) = (ix(f.diffuse), ix(f.specular));
        // (target, second target, pipeline, entry)
        let mut draws: Vec<(TextureId, Option<TextureId>, PipelineId, usize)> = Vec::new();
        let mut entries: Vec<DenoisePass> = Vec::new();
        let mut light = (raw_d, raw_s);
        // This frame's guide + 1 for the composite (what was lit), 0: from the raw light.
        let mut guide = 0;
        let mut gathered = 0;
        if self.mode == SVGF && !self.reconstruct {
            self.ensure_images(gpu, f.size);
            let im = self.images.as_ref().unwrap();
            let (cur, last) = (self.cur, 1 - self.cur);
            let held = self.held && !f.reset;
            let prev = DenoisePass {
                prev_diffuse: im.feed_diffuse[last].1,
                prev_specular: im.feed_specular[last].1,
                prev_len_diffuse: im.diffuse[last].1,
                prev_len_specular: im.specular[last].1,
                prev_moments: im.moments[last].1,
                prev_guide: if held { im.guide[last].1 + 1 } else { 0 },
                reset: !held as u32,
                ..base
            };
            entries.push(DenoisePass { diffuse: raw_d, specular: raw_s, ..prev });
            draws.push((im.diffuse[cur].0, Some(im.specular[cur].0), temporal, entries.len() - 1));
            entries.push(DenoisePass { diffuse: raw_d, specular: raw_s, ..prev });
            draws.push((im.moments[cur].0, Some(im.guide[cur].0), moments_pipe, entries.len() - 1));
            // From here on `prev_guide` is this frame's guide: what the path tracer lit.
            let lit = DenoisePass { prev_guide: im.guide[cur].1 + 1, ..base };
            entries.push(DenoisePass { diffuse: im.diffuse[cur].1, specular: im.specular[cur].1, moments: im.moments[cur].1, ..lit });
            draws.push((im.step_diffuse[0].0, Some(im.step_specular[0].0), variance, entries.len() - 1));
            // The first step into this frame's feedback pair (next frame's history), the rest in turn
            // between the step pair.
            let mut src = (im.step_diffuse[0].1, im.step_specular[0].1);
            let mut next = 1;
            for k in 0..self.steps.clamp(1, STEPS) {
                let (td, ts) =
                    if k == 0 { (im.feed_diffuse[cur], im.feed_specular[cur]) } else { (im.step_diffuse[next], im.step_specular[next]) };
                entries.push(DenoisePass { diffuse: src.0, specular: src.1, step: 1 << k, ..lit });
                draws.push((td.0, Some(ts.0), atrous, entries.len() - 1));
                src = (td.1, ts.1);
                if k > 0 {
                    next = 1 - next;
                }
            }
            light = src;
            guide = im.guide[cur].1 + 1;
            gathered = im.diffuse[cur].1 + 1;
            self.cur = last;
            self.held = true;
        } else {
            self.held = false;
        }
        // `step` 1 in the composite: the test's sky from the picture (the albedo there).
        entries.push(DenoisePass {
            diffuse: light.0,
            specular: light.1,
            step: self.test_sky as u32,
            prev_guide: guide,
            debug: self.debug,
            gathered,
            raw: raw_d + 1,
            ..base
        });
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
        // Onto the scene image as it is (loaded), or into the path tracer's light image (cleared).
        let clear = f.write.then_some([0.0; 4]);
        list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(f.target), clear)), depth: None });
        bind(list, composite);
        list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: comp_entry as u32 });
        list.push(Cmd::EndPass);
    }

    /// A check of the denoiser before the path tracer is there (`test_on`): today's picture
    /// `picture` is taken as the albedo, the light as a noise of mean 1 a pixel (each frame anew,
    /// heavy-tailed like one path a pixel), the normals from `depth`, no specular; the denoised light
    /// times the picture is drawn into an image of its own, which it returns (the frame's world in
    /// the picture's place). With `core:pt_denoiser` 0 that is the raw noise times the picture.
    #[allow(clippy::too_many_arguments)]
    pub fn run_test(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        slot: usize,
        scene: BufferId,
        picture: TextureId,
        depth: TextureId,
        motion: Option<TextureId>,
        size: (u32, u32),
    ) -> Option<TextureId> {
        let pipe = self.pipes.as_ref()?.test;
        if self.test.is_none_or(|t| t.0 != size) {
            if let Some((_, old)) = self.test.take() {
                old.into_iter().for_each(|t| gpu.destroy_texture(t));
            }
            let desc =
                TextureDesc { width: size.0.max(1), height: size.1.max(1), mips: 1, format: Format::Rgba16Float, render_target: true };
            let out_desc = TextureDesc { format: crate::post::world_format(), ..desc.clone() };
            self.test = Some((
                size,
                [
                    gpu.create_texture(&desc, "denoise test light"),
                    gpu.create_texture(&desc, "denoise test normals"),
                    gpu.create_texture(&desc, "denoise test zero"),
                    gpu.create_texture(&out_desc, "denoise test result"),
                ],
            ));
        }
        let [light, normal, zero, out] = self.test.unwrap().1;
        self.test_frame = self.test_frame.wrapping_add(1);
        // Its own entry, after the denoiser's (`run` writes from the start of the buffer).
        while self.params.len() < gpu.frames_in_flight() {
            let bytes = (size_of::<DenoisePass>() * (MAX_PASSES + 1)) as u64;
            self.params
                .push(gpu.create_buffer(bytes, usage::STORAGE | usage::HOST, "denoise passes"));
        }
        let buf = self.params[slot % self.params.len()];
        let entry = DenoisePass {
            diffuse: gpu.texture_index(picture),
            depth: gpu.texture_index(depth),
            step: self.test_frame,
            size: [size.0 as f32, size.1 as f32],
            ..DenoisePass::default()
        };
        let at = MAX_PASSES * size_of::<DenoisePass>();
        gpu.mapped(buf)[at..at + size_of::<DenoisePass>()].copy_from_slice(bytemuck::bytes_of(&entry));
        list.push(Cmd::BeginPass {
            motion: Some((normal, Some([0.0; 4]))),
            color: Some((Target::Texture(light), Some([0.0; 4]))),
            depth: None,
        });
        list.pipeline(pipe);
        list.bind(crate::gpu_layout::bind::denoise::PASSES, buf);
        list.bind(crate::gpu_layout::bind::denoise::SCENE, scene);
        list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: MAX_PASSES as u32 });
        list.push(Cmd::EndPass);
        for t in [zero, out] {
            list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(t), Some([0.0; 4]))), depth: None });
            list.push(Cmd::EndPass);
        }
        self.test_sky = true;
        self.run(
            gpu,
            list,
            &PtFrame {
                size,
                slot,
                scene,
                depth,
                motion,
                albedo: picture,
                normal,
                spec: zero,
                diffuse: light,
                specular: zero,
                target: out,
                reset: false,
                write: false,
            },
        );
        self.test_sky = false;
        Some(out)
    }
}

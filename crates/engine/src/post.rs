//! Screen effects mods turn on (`client_op::POST_EFFECT`): `shaders/post/<name>.wgsl` snippets in
//! one pipeline. With none on, the world is drawn straight to the screen; with some,
//! it goes into an image the effects read one after another, the last one onto the screen.

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

pub struct Post {
    pipeline: Option<PipelineId>,
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
}

const MAX_PASSES: usize = 16;

impl Post {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Post {
        let mut p = Post {
            pipeline: None,
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
        };
        p.reload(gpu, assets);
        p
    }

    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
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
                        color: Some(Format::Surface),
                        depth: None,
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                    })
                    .map_err(|e| e.0)?;
                // The copy for `copy_behind`; a pack's own post.wgsl may lack it.
                let copy = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "post copy behind",
                        shader: &s,
                        vertex: "vs",
                        fragment: Some("fs_copy_behind"),
                        color: Some(Format::Surface),
                        depth: Some(DepthState { write: true, compare: Compare::Always }),
                        blend: Blend::None,
                        cull_back: false,
                        topology: Topology::Triangles,
                    })
                    .map_err(|e| log::warn!("post: no copy of the world for water: {}", e.0))
                    .ok();
                Ok((p, copy, slots.pop().unwrap_or_default(), low, low_slots))
            });
        match made {
            Ok((p, copy, slots, low, low_slots)) => {
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

    /// Turns `name` on at `strength` (0 = off). False if no such effect exists.
    pub fn set(&mut self, name: &str, strength: f32) -> bool {
        let known = self.slots.slot(name) != 0;
        self.active.retain(|(n, _)| n != name);
        if strength > 0.0 && known {
            self.active.push((name.to_string(), strength));
        }
        known || strength <= 0.0
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

    /// The image the scene is drawn into, sized to the frame.
    pub fn scene_target(&mut self, gpu: &mut dyn Backend, w: u32, h: u32) -> TextureId {
        if self.targets.is_none_or(|t| t.2 != (w, h)) {
            if let Some((old, ..)) = self.targets.take() {
                old.into_iter().for_each(|t| gpu.destroy_texture(t));
            }
            let desc = TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: Format::Surface, render_target: true };
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
            let color = gpu.create_texture(
                &TextureDesc { width: w, height: h, mips: 1, format: Format::Surface, render_target: true },
                "behind color",
            );
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
        };
        let buf = self.copy_params[frame.slot];
        gpu.mapped(buf)[..size_of::<Pass>()].copy_from_slice(bytemuck::bytes_of(&pass));
        list.push(Cmd::BeginPass { color: Some((Target::Texture(behind[0]), Some([0.0; 4]))), depth: Some((behind[1], Some(0.0))) });
        list.pipeline(pipe);
        list.bind(crate::gpu_layout::bind::post::PASSES, buf);
        list.bind(crate::gpu_layout::bind::post::SCENE, scene);
        list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: 0 });
        list.push(Cmd::EndPass);
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
        // No effect on: one pass that only draws the scene across the screen (slot 0 returns it).
        let plain = [(String::new(), 1.0)];
        let active = if self.active.is_empty() { &plain[..] } else { &self.active[..] };
        let n = active.len().min(MAX_PASSES);
        let passes: Vec<Pass> = active[..n]
            .iter()
            .enumerate()
            .map(|(i, (name, strength))| Pass {
                scene: idx[i % 2],
                slot: self.slots.slot(name),
                strength: *strength,
                time,
                size: [frame.width as f32, frame.height as f32],
                depth,
                pad: 0,
                low: 0,
                spare: 0,
            })
            .collect();
        let mut passes = passes;
        // The half-size parts first: each into its effect's own image, which that effect reads.
        let mut lows: Vec<TextureId> = Vec::new();
        if self.low_pipe.is_some() && self.use_low {
            let (w, h) = (frame.width.div_ceil(2).max(1), frame.height.div_ceil(2).max(1));
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
                scene: idx[0],
                slot: self.low_slots.slot(name),
                strength: *strength,
                time,
                size: [frame.width as f32, frame.height as f32],
                depth,
                pad: 0,
                low: 0,
                spare: 0,
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
        let first_low = passes.len();
        passes.extend(low_entries);
        passes.truncate(MAX_PASSES);
        gpu.mapped(buf)[..passes.len() * size_of::<Pass>()].copy_from_slice(bytemuck::cast_slice(&passes));
        if let Some(low_pipe) = self.low_pipe {
            for (k, target) in lows.iter().enumerate() {
                let entry = first_low + k;
                if entry >= passes.len() {
                    break;
                }
                list.push(Cmd::BeginPass { color: Some((Target::Texture(*target), Some([0.0; 4]))), depth: None });
                list.pipeline(low_pipe);
                list.bind(crate::gpu_layout::bind::post::PASSES, buf);
                list.bind(crate::gpu_layout::bind::post::SCENE, scene);
                list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: entry as u32 });
                list.push(Cmd::EndPass);
            }
        }
        let n = draws.len();
        for (i, entry) in draws.into_iter().enumerate() {
            let target = if i + 1 == n { Target::Surface } else { Target::Texture(tex[(i + 1) % 2]) };
            list.push(Cmd::BeginPass { color: Some((target, Some([0.0; 4]))), depth: None });
            list.pipeline(pipe);
            list.bind(crate::gpu_layout::bind::post::PASSES, buf);
            list.bind(crate::gpu_layout::bind::post::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: entry });
            if i + 1 < n {
                list.push(Cmd::EndPass);
            }
        }
    }
}

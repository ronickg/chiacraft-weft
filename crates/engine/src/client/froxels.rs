//! The mist and the air's light as a volume (chiacraft's sky, `core:froxels`): the view cut into
//! cells an eighth of the world's pixels each way and 32 slices deep (closer together near the eye,
//! out to 1500 blocks), which the pack's sky fills before the world is drawn: `fs_froxel_light`
//! each cell's own mist and the light it sends toward the eye, `fs_froxel_sum` all of it from the
//! eye to each cell's far end. Every surface then reads how much mist lies before it and what light
//! it adds there (`Scene::froxels`) in two filtered taps, where it walked the mist in 8 steps of its
//! own: the mist was one of the Mac's dearest parts (Research rounds 15 and 19).
//!
//! The backend has no 3D images, so the slices lie side by side in one image, eight to a row
//! (like the soft clouds' noise). Both passes are drawn with the sky's `vs_clouds`.

use weft_render::*;

use crate::gpu_layout::bind;

/// Slices along the view (the pack's `FROXEL_SLICES`), eight to a row of the image.
pub const SLICES: u32 = 32;
const ROW: u32 = 8;

/// The cells across the view for a world `w` × `h` pixels: one per `cell` pixels each way (8, or
/// 4 at `core:froxels` 2, ultra).
fn grid(w: u32, h: u32, cell: u32) -> (u32, u32) {
    let cell = cell.max(1);
    (w.div_ceil(cell).max(1), h.div_ceil(cell).max(1))
}

struct Targets {
    /// Each cell's own mist and light, and the sums from the eye.
    light: TextureId,
    sum: TextureId,
    index: [u32; 2],
    grid: (u32, u32),
}

#[derive(Default)]
pub struct Froxels {
    /// The light and sum pipelines, when the pack's sky has both entries.
    pipes: Option<(PipelineId, PipelineId)>,
    targets: Option<Targets>,
    /// What the scene gets this frame (`Scene::froxels`): x the sums' bindless index + 1 (0: none,
    /// the surfaces walk the mist themselves), y the cells' own image's index, zw the cells across.
    pub scene: [f32; 4],
    /// Pixels a cell covers each way (`core:froxels`: 8, or 4 for ultra; 0 reads as 8).
    pub cell: u32,
    /// The two passes, for the app to run once the shadow maps are drawn and before the world.
    pub passes: CommandList,
}

impl Froxels {
    /// The pipelines from the sky's `shader` anew (a reload), none if the pack has no volume.
    pub fn set_shader(&mut self, gpu: &mut dyn Backend, shader: &Shader) {
        if let Some((a, b)) = self.pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        let has = |name: &str| shader.module.entry_points.iter().any(|e| e.name == name);
        if !has("fs_froxel_light") || !has("fs_froxel_sum") {
            return;
        }
        let mut pipe = |label, fragment| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader,
                vertex: "vs_clouds",
                fragment: Some(fragment),
                color: Some(Format::Rgba16Float),
                depth: None,
                blend: Blend::None,
                cull_back: false,
                topology: Topology::Triangles,
                motion: false,
            })
            .map_err(|e| e.0)
        };
        match (pipe("froxels light", "fs_froxel_light"), pipe("froxels sum", "fs_froxel_sum")) {
            (Ok(a), Ok(b)) => self.pipes = Some((a, b)),
            (a, b) => {
                for p in [a.as_ref().ok(), b.as_ref().ok()].into_iter().flatten() {
                    gpu.destroy_pipeline(*p);
                }
                if let Err(e) = a.and(b) {
                    log::error!("froxels shader: {e}");
                }
            }
        }
    }

    /// Everything, at shutdown.
    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some((a, b)) = self.pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        if let Some(t) = self.targets.take() {
            gpu.destroy_texture(t.light);
            gpu.destroy_texture(t.sum);
        }
    }

    /// Readies this frame for a world `w` × `h` pixels and records its passes with the sky's
    /// buffers (`objects`, `scene`). `on`: wanted and of use (mist in the air, the eye out of
    /// liquids); off, the scene's fields are zeros and the surfaces walk the mist themselves.
    pub fn frame(&mut self, gpu: &mut dyn Backend, on: bool, w: u32, h: u32, objects: BufferId, scene: BufferId) {
        self.passes.clear();
        let Some((light_pipe, sum_pipe)) = self.pipes.filter(|_| on) else {
            self.scene = [0.0; 4];
            return;
        };
        let cells = grid(w, h, if self.cell == 0 { 8 } else { self.cell });
        if self.targets.as_ref().is_none_or(|t| t.grid != cells) {
            if let Some(old) = self.targets.take() {
                // The GPU frees them once no frame in flight still draws with them.
                gpu.destroy_texture(old.light);
                gpu.destroy_texture(old.sum);
            }
            let desc = TextureDesc {
                width: cells.0 * ROW,
                height: cells.1 * SLICES.div_ceil(ROW),
                mips: 1,
                format: Format::Rgba16Float,
                render_target: true,
            };
            let light = gpu.create_texture(&desc, "froxels light");
            let sum = gpu.create_texture(&desc, "froxels sum");
            let index = [gpu.texture_index(light), gpu.texture_index(sum)];
            self.targets = Some(Targets { light, sum, index, grid: cells });
        }
        let t = self.targets.as_ref().unwrap();
        self.scene = [t.index[1] as f32 + 1.0, t.index[0] as f32, cells.0 as f32, cells.1 as f32];
        for (target, pipe) in [(t.light, light_pipe), (t.sum, sum_pipe)] {
            self.passes.push(Cmd::BeginPass {
                motion: None,
                color: Some((Target::Texture(target), Some([0.0, 0.0, 0.0, 1.0]))),
                depth: None,
            });
            self.passes.pipeline(pipe);
            self.passes.bind(bind::sky::OBJECTS, objects);
            self.passes.bind(bind::sky::SCENE, scene);
            self.passes
                .push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: 0 });
            self.passes.push(Cmd::EndPass);
        }
    }
}

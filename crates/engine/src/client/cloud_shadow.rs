//! The soft clouds' shadow on the land (chiacraft's sky, with the HDR world): a map seen from above
//! of how much of the sun gets through the clouds, which the pack's sky fills before the world is
//! drawn (`fs_cloud_shadow`, drawn with the clouds' own sky entry, so it reads their wind and
//! height), and every surface and the mist read (`Scene::cloud_shadow`). The soft clouds are a
//! volume the world's shaders can't see; their shadows came from the blocky clouds' picture at the
//! blocky clouds' height and matched nothing in the sky (Planner's gap breakdown, 11-05: Bliss's
//! cloud masses throw shadow patches across the land).

use weft_render::*;

use crate::gpu_layout::bind;

/// Texels each way.
pub const SIZE: u32 = 512;

#[derive(Default)]
pub struct CloudShadow {
    pipe: Option<PipelineId>,
    target: Option<(TextureId, u32)>,
    /// What the scene gets this frame (`Scene::cloud_shadow`): x the map's bindless index + 1 (0:
    /// none this frame), yzw unused.
    pub scene: [f32; 4],
    /// Its pass, for the app to run before the world.
    pub passes: CommandList,
}

impl CloudShadow {
    /// The pipeline from the sky's `shader` anew (a reload), none if the pack has no cloud shadow.
    pub fn set_shader(&mut self, gpu: &mut dyn Backend, shader: &Shader) {
        if let Some(p) = self.pipe.take() {
            gpu.destroy_pipeline(p);
        }
        if !shader.module.entry_points.iter().any(|e| e.name == "fs_cloud_shadow") {
            return;
        }
        match gpu.create_pipeline(&PipelineDesc {
            label: "cloud shadow",
            shader,
            vertex: "vs_clouds",
            fragment: Some("fs_cloud_shadow"),
            color: Some(Format::Rgba16Float),
            depth: None,
            blend: Blend::None,
            cull_back: false,
            topology: Topology::Triangles,
        }) {
            Ok(p) => self.pipe = Some(p),
            Err(e) => log::error!("cloud shadow shader: {}", e.0),
        }
    }

    /// Everything, at shutdown.
    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some(p) = self.pipe.take() {
            gpu.destroy_pipeline(p);
        }
        if let Some((t, _)) = self.target.take() {
            gpu.destroy_texture(t);
        }
    }

    /// Records this frame's pass with the sky's buffers (`objects`, `scene`) and the clouds' entry
    /// (`clouds`, none without soft clouds); `on`: wanted (the HDR world, soft clouds drawn).
    pub fn frame(&mut self, gpu: &mut dyn Backend, on: bool, objects: BufferId, scene: BufferId, clouds: Option<u32>) {
        self.passes.clear();
        let (Some(pipe), Some(clouds), true) = (self.pipe, clouds, on) else {
            self.scene = [0.0; 4];
            return;
        };
        let (t, index) = *self.target.get_or_insert_with(|| {
            let t = gpu.create_texture(
                &TextureDesc { width: SIZE, height: SIZE, mips: 1, format: Format::Rgba16Float, render_target: true },
                "cloud shadow",
            );
            (t, gpu.texture_index(t))
        });
        self.scene = [index as f32 + 1.0, SIZE as f32, 0.0, 0.0];
        self.passes.push(Cmd::BeginPass { color: Some((Target::Texture(t), Some([1.0, 1.0, 1.0, 0.0]))), depth: None });
        self.passes.pipeline(pipe);
        self.passes.bind(bind::sky::OBJECTS, objects);
        self.passes.bind(bind::sky::SCENE, scene);
        self.passes.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: clouds });
        self.passes.push(Cmd::EndPass);
    }
}

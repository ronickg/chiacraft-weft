//! What a pack's look works out once a frame (chiacraft's sky, `fs_frame_look`): a row of a few
//! texels the sky draws before the world, which every shader then reads (`Scene::frame_look`)
//! instead of working the same values out again for every pixel. chiacraft's palette by the hour
//! (sky, sun, fill light) was worked out in up to ten places per pixel, and the Mac's world pass
//! spilled registers (Tester's trace, 11-05: 480 bytes, the pass 77% fragment).

use weft_render::*;

use crate::gpu_layout::bind;

/// Texels in the row (the pack fills as many as it needs).
const TEXELS: u32 = 8;

#[derive(Default)]
pub struct FrameLook {
    pipe: Option<PipelineId>,
    target: Option<(TextureId, u32)>,
    /// What the scene gets this frame (`Scene::frame_look`): x the row's bindless index + 1 (0:
    /// none, each shader works the values out itself), yzw unused.
    pub scene: [f32; 4],
    /// Its pass, for the app to run before the world.
    pub passes: CommandList,
}

impl FrameLook {
    /// The pipeline from the sky's `shader` anew (a reload), none if the pack has no `fs_frame_look`.
    pub fn set_shader(&mut self, gpu: &mut dyn Backend, shader: &Shader) {
        if let Some(p) = self.pipe.take() {
            gpu.destroy_pipeline(p);
        }
        if !shader.module.entry_points.iter().any(|e| e.name == "fs_frame_look") {
            return;
        }
        match gpu.create_pipeline(&PipelineDesc {
            label: "frame look",
            shader,
            vertex: "vs_clouds",
            fragment: Some("fs_frame_look"),
            color: Some(Format::Rgba16Float),
            depth: None,
            blend: Blend::None,
            cull_back: false,
            topology: Topology::Triangles,
        }) {
            Ok(p) => self.pipe = Some(p),
            Err(e) => log::error!("frame look shader: {}", e.0),
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

    /// Records this frame's pass with the sky's buffers (`objects`, `scene`).
    pub fn frame(&mut self, gpu: &mut dyn Backend, objects: BufferId, scene: BufferId) {
        self.passes.clear();
        let Some(pipe) = self.pipe else {
            self.scene = [0.0; 4];
            return;
        };
        let (t, index) = *self.target.get_or_insert_with(|| {
            let t = gpu.create_texture(
                &TextureDesc { width: TEXELS, height: 1, mips: 1, format: Format::Rgba16Float, render_target: true },
                "frame look",
            );
            (t, gpu.texture_index(t))
        });
        self.scene = [index as f32 + 1.0, 0.0, 0.0, 0.0];
        self.passes.push(Cmd::BeginPass { color: Some((Target::Texture(t), Some([0.0; 4]))), depth: None });
        self.passes.pipeline(pipe);
        self.passes.bind(bind::sky::OBJECTS, objects);
        self.passes.bind(bind::sky::SCENE, scene);
        self.passes.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: 0 });
        self.passes.push(Cmd::EndPass);
    }
}

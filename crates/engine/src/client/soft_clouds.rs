//! Soft clouds (chiacraft): a pack's volumetric clouds traced at a quarter of the pixels each way
//! and blended with last frame's, as Photon and Horizon Zero Dawn do, so a soft cloud of many
//! steps costs what a blocky one does. Off (`core:soft_clouds`), or with a pack without the
//! entries, the sky's clouds draw as they always did.
//!
//! Each frame, after the opaque world is copied (`post::copy_behind`, whose depth the trace stops
//! at): the pack's `fs_cloud_trace` fills the quarter target, then `fs_cloud_resolve` blends it
//! into one of two histories, reprojecting last frame's (`Scene::prev_view_proj`,
//! `Scene::cloud_moved`). The sky's `fs_clouds` then draws the full picture from that history
//! (`Scene::soft_clouds`).

use glam::{DVec2, DVec3, Mat4};
use weft_render::*;

use crate::gpu_layout::bind;

/// The quarter target's size for a world of `w` × `h` pixels.
fn quarter(w: u32, h: u32) -> (u32, u32) {
    (w.div_ceil(4).max(1), h.div_ceil(4).max(1))
}

/// The noise volume's side, its slices' cells (a one-texel wrapped border) and the atlas's side.
const SIDE: usize = 64;
const CELL: usize = SIDE + 2;
const ATLAS: usize = CELL * 8;

fn hash(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(73856093)
        ^ (y as u32).wrapping_mul(19349663)
        ^ (z as u32).wrapping_mul(83492791)
        ^ seed.wrapping_mul(2654435761);
    h = (h ^ (h >> 16)).wrapping_mul(0x45d9f3b);
    h = (h ^ (h >> 16)).wrapping_mul(0x45d9f3b);
    h ^ (h >> 16)
}

fn unit(h: u32) -> f32 {
    (h & 0xffffff) as f32 / 16777215.0
}

/// Gradient noise that tiles every `period` lattice cells, about -1..1.
fn perlin(p: [f32; 3], period: i32, seed: u32) -> f32 {
    let i = p.map(|v| v.floor() as i32);
    let f = [p[0] - i[0] as f32, p[1] - i[1] as f32, p[2] - i[2] as f32];
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let u = f.map(fade);
    let grad = |dx: i32, dy: i32, dz: i32| {
        let h = hash((i[0] + dx).rem_euclid(period), (i[1] + dy).rem_euclid(period), (i[2] + dz).rem_euclid(period), seed);
        // One of the twelve edge directions of a cube.
        const G: [[f32; 3]; 12] = [
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [1.0, 0.0, -1.0],
            [-1.0, 0.0, -1.0],
            [0.0, 1.0, 1.0],
            [0.0, -1.0, 1.0],
            [0.0, 1.0, -1.0],
            [0.0, -1.0, -1.0],
        ];
        let g = G[(h % 12) as usize];
        g[0] * (f[0] - dx as f32) + g[1] * (f[1] - dy as f32) + g[2] * (f[2] - dz as f32)
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(grad(0, 0, 0), grad(1, 0, 0), u[0]);
    let x10 = lerp(grad(0, 1, 0), grad(1, 1, 0), u[0]);
    let x01 = lerp(grad(0, 0, 1), grad(1, 0, 1), u[0]);
    let x11 = lerp(grad(0, 1, 1), grad(1, 1, 1), u[0]);
    lerp(lerp(x00, x10, u[1]), lerp(x01, x11, u[1]), u[2])
}

/// Cellular noise that tiles every `cells` cells: 1 at a feature point, falling to 0 a cell away.
fn worley(p: [f32; 3], cells: i32, seed: u32) -> f32 {
    let i = p.map(|v| v.floor() as i32);
    let mut best = f32::MAX;
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let c = [i[0] + dx, i[1] + dy, i[2] + dz];
                let h = [c[0].rem_euclid(cells), c[1].rem_euclid(cells), c[2].rem_euclid(cells)];
                let at = [
                    c[0] as f32 + unit(hash(h[0], h[1], h[2], seed)),
                    c[1] as f32 + unit(hash(h[0], h[1], h[2], seed ^ 0x5bd1e995)),
                    c[2] as f32 + unit(hash(h[0], h[1], h[2], seed ^ 0x27d4eb2f)),
                ];
                let d = (at[0] - p[0]).powi(2) + (at[1] - p[1]).powi(2) + (at[2] - p[2]).powi(2);
                best = best.min(d);
            }
        }
    }
    1.0 - best.sqrt().min(1.0)
}

/// The noise volume the soft clouds are shaped from, tiling every 64 texels each way: r Perlin-
/// Worley (the clouds' billows: Perlin's broad shapes eaten into rounded cells), g, b and a Worley
/// at 4, 8 and 16 cells (the detail that erodes their edges). As RGBA8 in an atlas of 64 slices,
/// eight to a row, each in a 66² cell with a one-texel wrapped border.
pub fn noise_atlas() -> Vec<u8> {
    let mut out = vec![0u8; ATLAS * ATLAS * 4];
    let mut raw = vec![0f32; ATLAS * ATLAS * 4];
    let s = SIDE as f32;
    for z in 0..SIDE {
        let (cx, cy) = ((z % 8) * CELL, (z / 8) * CELL);
        for y in 0..CELL {
            for x in 0..CELL {
                // The border wraps to the opposite edge of the slice.
                let (vx, vy) = ((x + SIDE - 1) % SIDE, (y + SIDE - 1) % SIDE);
                let q = [vx as f32 / s, vy as f32 / s, z as f32 / s];
                let at = |n: f32| [q[0] * n, q[1] * n, q[2] * n];
                let fbm = perlin(at(4.0), 4, 1) * 0.6 + perlin(at(8.0), 8, 2) * 0.3 + perlin(at(16.0), 16, 3) * 0.1;
                let base = (fbm * 0.5 + 0.5).clamp(0.0, 1.0);
                let w4 = worley(at(4.0), 4, 11);
                let w8 = worley(at(8.0), 8, 12);
                let w16 = worley(at(16.0), 16, 13);
                let cells = w4 * 0.625 + w8 * 0.25 + w16 * 0.125;
                // Perlin remapped by the cells: billows with rounded tops.
                let billow = ((base - (1.0 - cells)) / (1.0 - (1.0 - cells)).max(1e-3)).clamp(0.0, 1.0) * 0.5 + base * 0.5;
                let o = ((cy + y) * ATLAS + cx + x) * 4;
                raw[o..o + 4].copy_from_slice(&[billow, w4, w8, w16]);
            }
        }
    }
    // Each channel stretched to the whole 0..1: as made, the billows spanned about 0.1-0.6, and
    // a clear day's cover let almost none of them through (the first soft-cloud runs, 11-05).
    let mut lo = [f32::MAX; 4];
    let mut hi = [f32::MIN; 4];
    for px in raw.as_chunks::<4>().0 {
        for c in 0..4 {
            lo[c] = lo[c].min(px[c]);
            hi[c] = hi[c].max(px[c]);
        }
    }
    for (o, v) in out.iter_mut().zip(raw.iter().enumerate()) {
        let c = v.0 % 4;
        *o = ((v.1 - lo[c]) / (hi[c] - lo[c]).max(1e-6) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    // The billows (r) equalized: each value by its rank, so a cover of 0.5 lets half of them
    // through. Stretched alone they still sat mostly low, and a clear day showed none (11-05).
    let mut order: Vec<usize> = (0..ATLAS * ATLAS).collect();
    order.sort_by(|&a, &b| raw[a * 4].total_cmp(&raw[b * 4]));
    let last = (order.len() - 1).max(1) as f32;
    for (rank, &i) in order.iter().enumerate() {
        out[i * 4] = (rank as f32 / last * 255.0).round() as u8;
    }
    out
}

struct Targets {
    trace: TextureId,
    history: [TextureId; 2],
    index: [u32; 3],
    size: (u32, u32),
}

#[derive(Default)]
pub struct SoftClouds {
    /// The trace's and the resolve's pipelines, from the sky's shader when the pack draws soft
    /// clouds; kept apart from the targets, which a shader reload keeps.
    pipes: Option<(PipelineId, PipelineId)>,
    targets: Option<Targets>,
    noise: Option<(TextureId, u32)>,
    /// Textures no longer drawn to, kept until the frames that may still use them are done: a
    /// target destroyed while the GPU still drew to it hung the GPU on a resize (render scale).
    retired: Vec<(u64, TextureId)>,
    frame: u64,
    /// Frames the history holds (0: start anew).
    age: u32,
    /// Last frame's camera: its view_proj (camera relative), where it stood, and the clouds'
    /// offset (`prepare`).
    prev: Option<(Mat4, DVec3, DVec2)>,
    /// What the scene gets this frame (`Scene::soft_clouds`, `prev_view_proj`, `cloud_moved`).
    pub scene: ([f32; 4], [[f32; 4]; 4], [f32; 4]),
    /// The trace and resolve passes, for the app to run after the world is copied.
    pub passes: CommandList,
}

impl SoftClouds {
    /// The pipelines from the sky's `shader` anew (a reload), none if the pack draws no soft
    /// clouds. The old ones go; the targets and the history stay.
    pub fn set_shader(&mut self, gpu: &mut dyn Backend, shader: &Shader) {
        if let Some((a, b)) = self.pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        let has = |name: &str| shader.module.entry_points.iter().any(|e| e.name == name);
        if !has("fs_cloud_trace") || !has("fs_cloud_resolve") {
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
            })
            .map_err(|e| e.0)
        };
        match (pipe("soft clouds trace", "fs_cloud_trace"), pipe("soft clouds resolve", "fs_cloud_resolve")) {
            (Ok(a), Ok(b)) => self.pipes = Some((a, b)),
            (a, b) => {
                for p in [a.as_ref().ok(), b.as_ref().ok()].into_iter().flatten() {
                    gpu.destroy_pipeline(*p);
                }
                if let Err(e) = a.and(b) {
                    log::error!("soft clouds shader: {e}");
                }
            }
        }
        self.age = 0;
    }

    /// Whether the pack draws soft clouds.
    pub fn available(&self) -> bool {
        self.pipes.is_some()
    }

    /// Everything, at shutdown.
    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some((a, b)) = self.pipes.take() {
            gpu.destroy_pipeline(a);
            gpu.destroy_pipeline(b);
        }
        if let Some(t) = self.targets.take() {
            for id in [t.trace, t.history[0], t.history[1]] {
                gpu.destroy_texture(id);
            }
        }
        if let Some((n, _)) = self.noise.take() {
            gpu.destroy_texture(n);
        }
        for (_, t) in self.retired.drain(..) {
            gpu.destroy_texture(t);
        }
    }

    /// Readies this frame: targets sized to the world (`w` × `h`), the scene's fields. `on`:
    /// wanted and possible (the world copied for the trace's depth, the eye out of liquids).
    /// `offset`: what the sky adds to a place (x and z, blocks) to find it in the cloud picture
    /// (its own drift less the wind's): a cloud keeps its place + offset.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(&mut self, gpu: &mut dyn Backend, on: bool, w: u32, h: u32, view_proj: Mat4, cam: DVec3, offset: DVec2) {
        self.passes.clear();
        let in_flight = gpu.frames_in_flight() as u64 + 1;
        let frame = self.frame;
        self.retired.retain(|&(at, t)| {
            let done = frame >= at + in_flight;
            if done {
                gpu.destroy_texture(t);
            }
            !done
        });
        if !on || self.pipes.is_none() {
            self.age = 0;
            self.prev = None;
            self.scene = ([0.0; 4], [[0.0; 4]; 4], [0.0; 4]);
            return;
        }
        if self.noise.is_none() {
            let t = gpu.create_texture(
                &TextureDesc { width: ATLAS as u32, height: ATLAS as u32, mips: 1, format: Format::Rgba8Unorm, render_target: false },
                "soft clouds noise",
            );
            gpu.write_texture(t, 0, &noise_atlas());
            self.noise = Some((t, gpu.texture_index(t)));
        }
        let size = quarter(w, h);
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            if let Some(old) = self.targets.take() {
                for id in [old.trace, old.history[0], old.history[1]] {
                    self.retired.push((frame, id));
                }
            }
            let desc = TextureDesc { width: size.0, height: size.1, mips: 1, format: Format::Rgba16Float, render_target: true };
            let trace = gpu.create_texture(&desc, "soft clouds trace");
            let history = [gpu.create_texture(&desc, "soft clouds a"), gpu.create_texture(&desc, "soft clouds b")];
            let index = [gpu.texture_index(trace), gpu.texture_index(history[0]), gpu.texture_index(history[1])];
            self.targets = Some(Targets { trace, history, index, size });
            self.age = 0;
        }
        let t = self.targets.as_ref().unwrap();
        let (now, before) = ((frame % 2) as usize, ((frame + 1) % 2) as usize);
        let (prev_vp, moved) = match self.prev {
            Some((vp, at, was)) if self.age > 0 => {
                // A cloud at camera-relative p now stands at cam + p, and keeps place + offset: last
                // frame it stood at cam + p + (offset - was), from that camera p + (cam - at) +
                // (offset - was).
                let d = cam - at;
                let o = offset - was;
                (vp, [(d.x + o.x) as f32, d.y as f32, (d.z + o.y) as f32])
            }
            _ => (view_proj, [0.0; 3]),
        };
        self.scene = (
            [t.index[1 + now] as f32 + 1.0, t.index[1 + before] as f32 + 1.0, self.noise.unwrap().1 as f32, self.age as f32],
            prev_vp.to_cols_array_2d(),
            [moved[0], moved[1], moved[2], (frame % 4096) as f32],
        );
        self.prev = Some((view_proj, cam, offset));
    }

    /// Records the trace and the resolve, after `prepare`, with the sky's buffer and its clouds
    /// entry; then the next frame.
    pub fn record(&mut self, objects: BufferId, scene: BufferId, clouds: u32) {
        let (Some(t), Some((trace_pipe, resolve_pipe))) = (&self.targets, self.pipes) else { return };
        if self.scene.0[0] == 0.0 {
            return;
        }
        let now = (self.frame % 2) as usize;
        let draw = |list: &mut CommandList, pipe| {
            list.pipeline(pipe);
            list.bind(bind::sky::OBJECTS, objects);
            list.bind(bind::sky::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: clouds });
        };
        self.passes
            .push(Cmd::BeginPass { color: Some((Target::Texture(t.trace), Some([0.0, 0.0, 0.0, 1.0]))), depth: None });
        draw(&mut self.passes, trace_pipe);
        self.passes.push(Cmd::EndPass);
        // The resolve reads the trace by the index in the clouds entry (`trace_index`).
        self.passes
            .push(Cmd::BeginPass { color: Some((Target::Texture(t.history[now]), None)), depth: None });
        draw(&mut self.passes, resolve_pipe);
        self.passes.push(Cmd::EndPass);
        self.frame += 1;
        self.age = (self.age + 1).min(1 << 20);
    }

    /// The bindless index of the trace target, for the resolve (`Sky` puts it in the clouds entry).
    pub fn trace_index(&self) -> Option<u32> {
        self.targets.as_ref().map(|t| t.index[0])
    }

    /// No clouds this frame (none in the sky, or the eye in a liquid): the history starts anew.
    pub fn skip(&mut self) {
        self.scene.0 = [0.0; 4];
        self.age = 0;
    }
}

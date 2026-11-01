//! Entity models on the GPU: every kind's boxes and triangles in two static tables; per frame
//! the bones of every visible entity are posed on the CPU (parents, keyframes, vanilla's limb
//! swing) and each layer (skin or something worn) is one instance, drawn instanced per kind.
//! Models and animations come from code or from files (`model.rs`); files win, so resource packs
//! can change any entity's model.

use std::collections::HashMap;

use glam::Mat4;
use rayon::prelude::*;
use weft_abi::comp;
use weft_render::*;

use crate::assets::Assets;
use crate::components::ENTITY;
use crate::content::content;
pub use crate::gpu_layout::Instance;
use crate::gpu_layout::{GpuCorner, GpuCuboid};
use crate::model::{self, Bound, Motion};
use crate::registry::Part;

/// A kind's model and animations.
pub struct KindModel {
    pub parts: Vec<Part>,
    cuboids: (u32, u32),
    corners: (u32, u32),
    pub anims: Vec<Bound>,
    pub idle: Option<usize>,
    pub walk: Option<usize>,
    /// Texture size the model file was made for (it overrides the kind's).
    pub texture: Option<[u32; 2]>,
}

impl KindModel {
    pub fn anim(&self, name: &str) -> Option<usize> {
        self.anims.iter().position(|a| a.anim.name == name)
    }

    /// The animation started by the server (name, start age, looping) if it still runs at `age`,
    /// with its time.
    pub fn playing(&self, anim: Option<&(String, f32, bool)>, age: f32) -> Option<(usize, f32)> {
        let (name, start, looping) = anim?;
        let i = self.anim(name)?;
        let b = &self.anims[i];
        (!b.finished(age - start, *looping)).then(|| (i, b.at(age - start, *looping)))
    }
}

/// Layers one entity draws at most: its skin and what it wears.
pub const MAX_LAYERS: usize = 8;

/// One entity to pose this frame: kind, root transform, look pitch, limb swing, crouch, swim, up to
/// two keyframe animations (index, time, weight) and its layers.
#[derive(Clone, Copy)]
pub struct Posed {
    pub kind: u32,
    pub root: Mat4,
    pub pitch: f32,
    pub swing: f32,
    pub crouch: bool,
    pub head_yaw: f32,
    pub swim: Option<f32>,
    pub attack: f32,
    pub holding: bool,
    pub anims: [(usize, f32, f32); 2],
    pub anim_count: usize,
    pub layers: [Instance; MAX_LAYERS],
    pub layer_count: usize,
}

#[derive(Clone, Copy)]
pub enum Batch {
    World,
    Preview,
}

pub struct Models {
    pub kinds: Vec<KindModel>,
    cuboids: BufferId,
    corners: BufferId,
    /// Per frame slot, one set per [`Batch`]: the GPU reads them only when the frame's list
    /// runs, so two batches drawn in one frame can't share them.
    instances: [Vec<(BufferId, u32)>; 2],
    bones: [Vec<(BufferId, u32)>; 2],
    /// This frame: (kind, instance) and bone rows.
    queued: Vec<(u32, Instance)>,
    rows: Vec<[[f32; 4]; 3]>,
    scratch: Vec<Mat4>,
    /// Bones per entity in `add_many`.
    counts: Vec<usize>,
}

/// A model file read once: its parts, texture size and where its boxes and triangles sit in the
/// tables. Kinds naming the same file share it (a mod's coats and poses as kinds, say).
struct Shared {
    parts: Vec<Part>,
    texture: Option<[u32; 2]>,
    cuboids: (u32, u32),
    corners: (u32, u32),
}

fn read(assets: &Assets, name: &str, kind: &str, ext: &str) -> Option<String> {
    assets
        .read(name, kind, ext)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

fn geo_file(assets: &Assets, name: &str) -> Option<model::Geo> {
    model::load_geo(&read(assets, name, "models", "geo.json")?)
        .map_err(|e| log::error!("model {name}: {e}"))
        .ok()
}

fn anim_file(assets: &Assets, name: &str) -> Vec<model::Animation> {
    match read(assets, name, "animations", "animation.json") {
        Some(src) => model::load_animations(&src).unwrap_or_else(|e| {
            log::error!("animations {name}: {e}");
            Vec::new()
        }),
        None => Vec::new(),
    }
}

impl Models {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Models {
        let mut m = Models {
            kinds: Vec::new(),
            cuboids: gpu.create_buffer(16, usage::STORAGE, "model cuboids"),
            corners: gpu.create_buffer(16, usage::STORAGE, "model corners"),
            instances: Default::default(),
            bones: Default::default(),
            queued: Vec::new(),
            rows: Vec::new(),
            scratch: Vec::new(),
            counts: Vec::new(),
        };
        m.reload(gpu, assets);
        m
    }

    /// Reads every kind's model and animations again (files may have changed).
    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        let mut cuboids: Vec<GpuCuboid> = Vec::new();
        let mut corners: Vec<GpuCorner> = Vec::new();
        self.kinds.clear();
        // A kind's model is the file named by its `model` component, else `<ns>:<name>` if a
        // model file of that name exists, else its parts from code; its animations likewise.
        let mut shared: HashMap<String, Shared> = HashMap::new();
        let mut anim_files: HashMap<String, Vec<model::Animation>> = HashMap::new();
        for k in 0..content().game.entities.len() as u32 {
            let c = content();
            let kind = &c.game.entities[k as usize];
            let named = |key: &str| {
                c.components
                    .id(key)
                    .and_then(|id| c.components.text(ENTITY, k, id))
                    .map(str::to_string)
            };
            let model_name = named(comp::MODEL).unwrap_or_else(|| kind.name.clone());
            let anim_name = named(comp::ANIMATIONS).unwrap_or_else(|| kind.name.clone());
            let mut push = |parts: &[Part]| {
                let (c0, v0) = (cuboids.len() as u32, corners.len() as u32);
                for (b, p) in parts.iter().enumerate() {
                    for cu in &p.cuboids {
                        let origin = [p.pivot[0] + cu.origin[0], p.pivot[1] + cu.origin[1], p.pivot[2] + cu.origin[2]];
                        cuboids.push(GpuCuboid {
                            origin,
                            bone: b as u32,
                            size: cu.size,
                            inflate: cu.inflate,
                            faces: cu.face_rects(),
                            role: p.role,
                            pad0: 0,
                            pad1: 0,
                            pad2: 0,
                        });
                    }
                    corners.extend(
                        p.mesh
                            .iter()
                            .map(|v| GpuCorner { pos: v.pos, bone: b as u32, uv: v.uv, role: p.role, pad: 0 }),
                    );
                }
                ((c0, cuboids.len() as u32 - c0), (v0, corners.len() as u32 - v0))
            };
            if !shared.contains_key(&model_name)
                && let Some(mut g) = geo_file(assets, &model_name)
            {
                let (cu, co) = push(&g.parts);
                // The triangles are on the GPU now; every kind sharing the file would copy them.
                g.parts.iter_mut().for_each(|p| p.mesh = Vec::new());
                shared.insert(model_name.clone(), Shared { parts: g.parts, texture: g.texture, cuboids: cu, corners: co });
            }
            let (parts, texture, cu, co) = match shared.get(&model_name) {
                Some(s) => (s.parts.clone(), s.texture, s.cuboids, s.corners),
                None => {
                    let (cu, co) = push(&kind.parts);
                    (kind.parts.clone(), None, cu, co)
                }
            };
            let anims = anim_files
                .entry(anim_name.clone())
                .or_insert_with(|| anim_file(assets, &anim_name))
                .clone();
            let anims: Vec<Bound> = anims.into_iter().map(|a| Bound::new(a, &parts)).collect();
            let find = |n: &str| anims.iter().position(|a| a.anim.name == n);
            let (idle, walk) = (find("idle"), find("walk"));
            self.kinds
                .push(KindModel { cuboids: cu, corners: co, parts, anims, idle, walk, texture });
        }
        // Buffers may not be empty.
        cuboids.push(GpuCuboid::default());
        corners.push(GpuCorner::default());
        gpu.destroy_buffer(self.cuboids);
        gpu.destroy_buffer(self.corners);
        self.cuboids = gpu.create_buffer((cuboids.len() * size_of::<GpuCuboid>()) as u64, usage::STORAGE, "model cuboids");
        gpu.write_buffer(self.cuboids, 0, bytemuck::cast_slice(&cuboids));
        self.corners = gpu.create_buffer((corners.len() * size_of::<GpuCorner>()) as u64, usage::STORAGE, "model corners");
        gpu.write_buffer(self.corners, 0, bytemuck::cast_slice(&corners));
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        for b in [self.cuboids, self.corners].into_iter().chain(
            self.instances
                .iter_mut()
                .chain(&mut self.bones)
                .flat_map(|v| v.drain(..))
                .map(|b| b.0),
        ) {
            gpu.destroy_buffer(b);
        }
    }

    pub fn begin(&mut self) {
        self.queued.clear();
        self.rows.clear();
    }

    /// Poses a model of `kind` (model pixels to `root`) and queues its layers.
    pub fn add(&mut self, kind: u32, root: Mat4, motion: &Motion, layers: impl IntoIterator<Item = Instance>) {
        let Some(m) = self.kinds.get(kind as usize) else { return };
        model::pose(&m.parts, root, motion, &mut self.scratch);
        let first = self.rows.len() as u32;
        self.rows.extend(
            self.scratch
                .iter()
                .map(|b| [b.row(0).to_array(), b.row(1).to_array(), b.row(2).to_array()]),
        );
        for mut inst in layers {
            (inst.bones, inst.cuboids, inst.cuboid_count, inst.vertices) = (first, m.cuboids.0, m.cuboids.1, m.corners.0);
            self.queued.push((kind, inst));
        }
    }

    /// How many bones are posed so far: where what comes next starts in `bone`.
    pub fn bones_so_far(&self) -> usize {
        self.rows.len()
    }

    /// A posed bone's transform.
    pub fn bone(&self, at: usize) -> Mat4 {
        let [a, b, c] = self.rows[at];
        Mat4::from_cols_array_2d(&[a, b, c, [0.0, 0.0, 0.0, 1.0]]).transpose()
    }

    /// Poses many models at once, in parallel, straight into the bone table: per entity its kind,
    /// root, pitch, swing, crouch, keyframes and layers. Nothing is allocated per entity (a
    /// thousand mobs are a thousand of each every frame).
    pub fn add_many(&mut self, entities: &[Posed]) {
        let kinds = &self.kinds;
        let base = self.rows.len();
        let mut counts = std::mem::take(&mut self.counts);
        counts.clear();
        counts.extend(
            entities
                .iter()
                .map(|e| kinds.get(e.kind as usize).map_or(0, |m| m.parts.len())),
        );
        self.rows
            .resize(base + counts.iter().sum::<usize>(), [[0.0; 4]; 3]);
        let mut slices: Vec<&mut [[[f32; 4]; 3]]> = Vec::with_capacity(entities.len());
        let mut rest = &mut self.rows[base..];
        for &n in &counts {
            let (a, b) = rest.split_at_mut(n);
            slices.push(a);
            rest = b;
        }
        entities
            .par_iter()
            .zip(slices)
            .with_min_len(64)
            .for_each_init(Vec::new, |scratch, (e, out)| {
                let Some(m) = kinds.get(e.kind as usize) else { return };
                let a = |i: usize| {
                    e.anims[..e.anim_count]
                        .get(i)
                        .and_then(|&(a, t, w)| m.anims.get(a).map(|b| (b, t, w)))
                };
                let (single, both);
                let anims: &[(&Bound, f32, f32)] = match (a(0), a(1)) {
                    (Some(x), Some(y)) => {
                        both = [x, y];
                        &both
                    }
                    (Some(x), None) | (None, Some(x)) => {
                        single = [x];
                        &single
                    }
                    (None, None) => &[],
                };
                model::pose(
                    &m.parts,
                    e.root,
                    &Motion {
                        pitch: e.pitch,
                        head_yaw: e.head_yaw,
                        swing: e.swing,
                        swim: e.swim,
                        attack: e.attack,
                        holding: e.holding,
                        anims,
                        crouch: e.crouch,
                    },
                    scratch,
                );
                for (o, b) in out.iter_mut().zip(scratch.iter()) {
                    *o = [b.row(0).to_array(), b.row(1).to_array(), b.row(2).to_array()];
                }
            });
        let mut first = base as u32;
        for (e, &n) in entities.iter().zip(&counts) {
            if let Some(m) = self.kinds.get(e.kind as usize) {
                for mut inst in e.layers[..e.layer_count].iter().copied() {
                    (inst.bones, inst.cuboids, inst.cuboid_count, inst.vertices) = (first, m.cuboids.0, m.cuboids.1, m.corners.0);
                    self.queued.push((e.kind, inst));
                }
            }
            first += n as u32;
        }
        self.counts = counts;
    }

    /// Draws what was queued since `begin`, one instanced draw per kind, with `pipe` and the
    /// view in `view`.
    pub fn draw(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        batch: Batch,
        pipe: PipelineId,
        view: BufferId,
    ) {
        if self.queued.is_empty() {
            return;
        }
        self.queued.sort_by_key(|q| q.0);
        let grow = crate::gpu_layout::frame_buffer;
        let ib = grow(
            gpu,
            &mut self.instances[batch as usize],
            frame.slot,
            self.queued.len() as u32,
            size_of::<Instance>() as u64,
            usage::STORAGE,
            "model instances",
        );
        let bb = grow(gpu, &mut self.bones[batch as usize], frame.slot, self.rows.len().max(1) as u32, 48, usage::STORAGE, "model bones");
        let insts: Vec<Instance> = self.queued.iter().map(|q| q.1).collect();
        gpu.mapped(ib)[..insts.len() * size_of::<Instance>()].copy_from_slice(bytemuck::cast_slice(&insts));
        gpu.mapped(bb)[..self.rows.len() * 48].copy_from_slice(bytemuck::cast_slice(&self.rows));
        list.pipeline(pipe);
        list.bind(crate::gpu_layout::bind::model::INSTANCES, ib);
        list.bind(crate::gpu_layout::bind::model::SCENE, view);
        list.bind(crate::gpu_layout::bind::model::CUBOIDS, self.cuboids);
        list.bind(crate::gpu_layout::bind::model::CORNERS, self.corners);
        list.bind(crate::gpu_layout::bind::model::BONES, bb);
        let mut i = 0;
        while i < self.queued.len() {
            let kind = self.queued[i].0;
            let end = i + self.queued[i..].iter().take_while(|q| q.0 == kind).count();
            let m = &self.kinds[kind as usize];
            let vertices = m.cuboids.1 * 36 + m.corners.1;
            list.push(Cmd::Draw { vertices, instances: (end - i) as u32, first_vertex: 0, first_instance: i as u32 });
            i = end;
        }
    }
}

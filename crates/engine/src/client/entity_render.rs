//! Entities: their models (`models.rs`, posed per frame and drawn instanced per kind), dropped
//! items, the held item, flames, the block selection outline, break progress and gizmo boxes
//! (textured boxes built on the CPU each frame, a few hundred at most).

use super::looks::ItemDraw;
use super::models::{MAX_LAYERS, Posed};
use crate::gpu_layout::box_mode;
use glam::{DVec3, IVec3, Mat4, Quat, Vec3, Vec4};
use rustc_hash::FxHashMap;
use weft_render::*;

use super::ClientEntity;
use super::models::{Instance, Models};
use super::textures::GameTextures;
use super::world_render::Camera;
use crate::assets::{Assets, decode_png};
use crate::content::content;
use crate::gpu_layout::{GpuBox, Scene};
use crate::physics::Hit;

/// A skin a mod painted: the component it was painted from, the texture of the kind it was
/// painted over (the looks drawn with it), and the GPU copy.
struct Painted {
    from: Vec<u8>,
    texture: String,
    tex: (TextureId, u32, [u32; 2]),
}

/// An entity's build its painter set (`SkinArgs::bones`): bones by name and their scales, and
/// for the kind it was last drawn as, the scale of each of its parts and how far that lifts the
/// model (model pixels: longer legs stand taller).
struct Build {
    bones: Vec<(String, [f32; 3])>,
    resolved: Option<(u32, Vec<[f32; 3]>, f32)>,
}

/// Bone scales a painter may set on one entity.
const MAX_BUILD_BONES: usize = 64;

/// A kind's painter: its component, the mod and the mod's callback.
type Painter = (u32, weft_abi::ModId, weft_abi::Callback);

/// Skins being painted at once (`paint_skins`): many animals coming into view at once queue up
/// behind them.
const PAINTS_IN_FLIGHT: usize = 8;

/// A texture painted over, as read: RGBA (shared with the painters) and size; none if unreadable.
type SkinBase = Option<(std::sync::Arc<Vec<u8>>, [u32; 2])>;

/// A skin a painter finished on `skin_painters()`: `bones` none if its callback failed.
struct PaintDone {
    id: u64,
    generation: u32,
    from: Vec<u8>,
    rgba: Vec<u8>,
    size: [u32; 2],
    bones: Option<Vec<(String, [f32; 3])>>,
}

/// Where mods' skin painters run: two threads of normal priority (what comes into view), apart
/// from the loading queues.
fn skin_painters() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .thread_name(|i| format!("skin paint {i}"))
            .build()
            .expect("skin paint pool")
    })
}

/// What the first person view holds: the item (or the bare arm) and how it moves.
/// The block outline's width on the screen (pixels) and how dark it is (its black's alpha).
const OUTLINE_PX: f32 = 1.25;
const OUTLINE_DARK: f32 = 0.4;

pub struct Hand {
    pub stack: weft_abi::ItemStack,
    pub swing: Option<f32>,
    /// How far it is lowered, 0..1: while changing items and just after a use.
    pub lowered: f32,
    /// Radians the hand lags behind a turning camera: pitch, yaw.
    pub lag: (f32, f32),
    /// Holding the use button on it: ticks left, ticks in all.
    pub using: Option<(f32, f32)>,
    /// The local player's animation started by the server and the player's age, for the `hand`
    /// bone of its keyframes.
    pub anim: Option<(String, f32, bool)>,
    pub age: f32,
}

/// Opaque pixels of a sprite, for drawing it as a 1/16 thick solid like vanilla's held items.
struct Sprite {
    size: u32,
    /// Opaque pixels and which of their sides touch another one (left, right, below, above).
    pixels: Vec<(u8, u8, u32)>,
}

pub struct EntityRenderer {
    hand_pipe: PipelineId,
    pipeline: PipelineId,
    overlay: PipelineId,
    /// Model pipelines for the screen and for the inventory figure's image.
    model_pipe: PipelineId,
    preview_pipe: PipelineId,
    /// Models into the sun's shadow map, when the shader pack draws one and its model shader has
    /// `fs_shadow`.
    shadow_pipe: Option<PipelineId>,
    pub models: Models,
    buffers: Vec<(BufferId, u32)>,
    /// The held item's projection: a fixed field of view, like vanilla, so it keeps its shape
    /// whatever the player's setting.
    hand_views: Vec<BufferId>,
    /// The inventory figure's view; it is drawn into its own texture so the UI can layer it like
    /// any image.
    preview: Vec<BufferId>,
    /// Color, depth, size, bindless index.
    preview_target: Option<(TextureId, TextureId, [u32; 2], u32)>,
    /// Texture, bindless index and size, by name; and the glow masks of those that have one.
    skins: FxHashMap<String, Option<(TextureId, u32, [u32; 2])>>,
    /// Players' own skins by entity, replacing the player kind's texture on them.
    player_skins: FxHashMap<u64, (TextureId, u32, [u32; 2])>,
    /// Skins received and not uploaded yet.
    pub skin_pending: Vec<(u64, Vec<u8>)>,
    /// Skins mods painted (`Host::reg_entity_skin`), by entity.
    painted: FxHashMap<u64, Painted>,
    /// Builds their painters set, by entity; and this frame's parts' scales (`Posed::build`).
    builds: FxHashMap<u64, Build>,
    frame_builds: Vec<[f32; 3]>,
    /// Each kind with a painter: its component, the mod and its callback (made on first use).
    painters: Option<FxHashMap<u32, Painter>>,
    /// Textures painted over, as read: RGBA and size, by name.
    skin_bases: FxHashMap<String, SkinBase>,
    /// Entities whose skin is being painted, what finishes (`paint_skins`), and which reading of
    /// the textures they were painted over (`reload` counts up; older ones are dropped).
    painting: rustc_hash::FxHashSet<u64>,
    paint_done: (std::sync::mpsc::Sender<PaintDone>, std::sync::mpsc::Receiver<PaintDone>),
    paint_generation: u32,
    /// The local player's entity.
    pub local: u64,
    glows: FxHashMap<String, (TextureId, u32)>,
    /// Shader slot of each entity kind.
    kind_shaders: Vec<u32>,
    sprites: FxHashMap<String, Option<Sprite>>,
    /// Entities to pose this frame, kept between frames.
    posed: Vec<Posed>,
    /// `preload`'s kinds and items already looked at this frame.
    seen_kinds: Vec<bool>,
    seen_items: Vec<bool>,
    /// Item models drawn instanced: pipelines for the world and the hand, each item's boxes
    /// (first, count) in `item_boxes` by `model_key`, the GPU copy (buffer, capacity, boxes
    /// uploaded) and the texture generation they were made with.
    item_pipe: PipelineId,
    hand_item_pipe: PipelineId,
    item_models: FxHashMap<u64, (u32, u32)>,
    item_boxes: Vec<GpuBox>,
    item_buf: Option<(BufferId, u32, usize)>,
    item_generation: u64,
    /// This frame's item instances: (model key, instance).
    instances: Vec<(u64, GpuBox)>,
}

fn shader(assets: &Assets) -> Result<Shader, ShaderError> {
    crate::packs::shader(assets, "entity.wgsl")
}

pub const HOOK: crate::shaders::Hook = crate::shaders::Hook {
    kind: "entity",
    entry: "shade",
    dispatch: "entity_shade",
    params: "p: EntityPixel",
    args: "p",
    ret: "vec4<f32>",
    fallback: "return vec4(p.albedo.rgb * p.light, p.albedo.a);",
};

/// Light on boxes and model layers drawn without any (the inventory figure, flames).
const FULL_LIGHT: u32 = u32::MAX;

/// Gives boxes pushed since `from` the light they stand in (the shader turns it into color).
fn light_boxes(boxes: &mut [GpuBox], from: usize, light: u32) {
    for b in &mut boxes[from..] {
        b.ids2[1] = light;
    }
}

impl EntityRenderer {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Result<EntityRenderer, String> {
        let (pipeline, overlay) = Self::pipelines(gpu, assets)?;
        let ([model_pipe, preview_pipe], shadow_pipe, kind_shaders) = Self::model_pipelines(gpu, assets)?;
        let models = Models::new(gpu, assets);
        let frames = gpu.frames_in_flight();
        let hand_views = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Scene>() as u64, usage::STORAGE | usage::HOST, "hand scene"))
            .collect();
        let preview = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Scene>() as u64, usage::STORAGE | usage::HOST, "preview scene"))
            .collect();
        let hand_pipe = Self::hand_pipeline(gpu, assets)?;
        let (item_pipe, hand_item_pipe) = Self::item_pipelines(gpu, assets)?;
        Ok(EntityRenderer {
            item_pipe,
            hand_item_pipe,
            item_models: FxHashMap::default(),
            item_boxes: Vec::new(),
            item_buf: None,
            item_generation: 0,
            instances: Vec::new(),
            seen_items: Vec::new(),
            pipeline,
            overlay,
            hand_pipe,
            model_pipe,
            shadow_pipe,
            models,
            buffers: Vec::new(),
            hand_views,
            preview,
            preview_pipe,
            preview_target: None,
            skins: FxHashMap::default(),
            player_skins: FxHashMap::default(),
            skin_pending: Vec::new(),
            painted: FxHashMap::default(),
            builds: FxHashMap::default(),
            frame_builds: Vec::new(),
            painters: None,
            skin_bases: FxHashMap::default(),
            painting: Default::default(),
            paint_done: std::sync::mpsc::channel(),
            paint_generation: 0,
            local: 0,
            glows: FxHashMap::default(),
            kind_shaders,
            sprites: FxHashMap::default(),
            posed: Vec::new(),
            seen_kinds: Vec::new(),
        })
    }

    /// The first person hand: drawn last with its depth squeezed to the front, so it never sinks
    /// into walls.
    fn hand_pipeline(gpu: &mut dyn Backend, assets: &Assets) -> Result<PipelineId, String> {
        let s = shader(assets).map_err(|e| e.0)?;
        gpu.create_pipeline(&PipelineDesc {
            label: "hand",
            shader: &s,
            vertex: "vs_hand",
            fragment: Some("fs"),
            color: Some(Format::Surface),
            depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
            blend: Blend::None,
            cull_back: true,
            topology: Topology::Triangles,
        })
        .map_err(|e| e.0)
    }

    /// Item models instanced in the world and in the hand.
    fn item_pipelines(gpu: &mut dyn Backend, assets: &Assets) -> Result<(PipelineId, PipelineId), String> {
        let s = shader(assets).map_err(|e| e.0)?;
        let mut pipe = |label, vertex| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader: &s,
                vertex,
                fragment: Some("fs"),
                color: Some(Format::Surface),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
            })
            .map_err(|e| e.0)
        };
        Ok((pipe("items", "vs_item")?, pipe("hand item", "vs_hand_item")?))
    }

    fn pipelines(gpu: &mut dyn Backend, assets: &Assets) -> Result<(PipelineId, PipelineId), String> {
        let s = shader(assets).map_err(|e| e.0)?;
        let main = gpu
            .create_pipeline(&PipelineDesc {
                label: "entities",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs"),
                color: Some(Format::Surface),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
            })
            .map_err(|e| e.0)?;
        let overlay = gpu
            .create_pipeline(&PipelineDesc {
                label: "overlay",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs_overlay"),
                color: Some(Format::Surface),
                depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                blend: Blend::Alpha,
                cull_back: true,
                topology: Topology::Triangles,
            })
            .map_err(|e| e.0)?;
        Ok((main, overlay))
    }

    /// The model pipelines (screen, inventory figure image) with every entity shader snippet, the
    /// sun's shadow map's (with the pack's sun shadows) and each kind's slot.
    #[allow(clippy::type_complexity)]
    fn model_pipelines(gpu: &mut dyn Backend, assets: &Assets) -> Result<([PipelineId; 2], Option<PipelineId>, Vec<u32>), String> {
        let snippets = crate::shaders::snippets(assets, &HOOK);
        let (s, mut slots) = crate::packs::compose(assets, "model.wgsl", &[(&HOOK, &snippets)]).map_err(|e| e.0)?;
        let slots = slots.pop().unwrap_or_default();
        let c = content();
        let comp = c.components.id(weft_abi::comp::SHADER);
        let kinds = (0..c.game.entities.len() as u32)
            .map(|k| {
                let name = comp.and_then(|id| c.components.text(crate::components::ENTITY, k, id));
                let slot = name.map_or(0, |n| slots.slot(n));
                if slot == 0 && name.is_some() {
                    log::warn!("entity {}: no entity shader {}", c.game.entities[k as usize].name, name.unwrap_or(""));
                }
                slot
            })
            .collect();
        let mut make = |label: &str, color: Format| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader: &s,
                vertex: "vs",
                fragment: Some("fs"),
                color: Some(color),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
            })
            .map_err(|e| e.0)
        };
        let pipes = [make("models", Format::Surface)?, make("entity preview", Format::Rgba8Srgb)?];
        // Depth only, both sides (a model's thin parts are open boxes), its see-through texels left out.
        let shadow = crate::packs::sun_shadows()
            .then(|| {
                gpu.create_pipeline(&PipelineDesc {
                    label: "models shadow",
                    shader: &s,
                    vertex: "vs",
                    fragment: Some("fs_shadow"),
                    color: None,
                    depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                    blend: Blend::None,
                    cull_back: false,
                    topology: Topology::Triangles,
                })
                .map_err(|e| log::warn!("models cast no sun shadows: {}", e.0))
                .ok()
            })
            .flatten();
        Ok((pipes, shadow, kinds))
    }

    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        match Self::model_pipelines(gpu, assets) {
            Ok(([p, q], shadow, kinds)) => {
                gpu.destroy_pipeline(self.model_pipe);
                gpu.destroy_pipeline(self.preview_pipe);
                if let Some(s) = self.shadow_pipe.take() {
                    gpu.destroy_pipeline(s);
                }
                (self.model_pipe, self.preview_pipe, self.shadow_pipe) = (p, q, shadow);
                self.kind_shaders = kinds;
            }
            Err(e) => log::error!("model shader: {e}"),
        }
        self.models.reload(gpu, assets);
        // Painted skins are painted again over the textures as read now.
        self.skin_bases.clear();
        self.painting.clear();
        self.paint_generation = self.paint_generation.wrapping_add(1);
        for (_, p) in self.painted.drain() {
            gpu.destroy_texture(p.tex.0);
        }
        // Skins and masks are read again the next time they are used.
        for t in self
            .skins
            .drain()
            .filter_map(|(_, v)| v)
            .map(|v| v.0)
            .chain(self.glows.drain().map(|(_, g)| g.0))
        {
            gpu.destroy_texture(t);
        }
        if let Ok(h) = Self::hand_pipeline(gpu, assets) {
            gpu.destroy_pipeline(self.hand_pipe);
            self.hand_pipe = h;
        }
        if let Ok((a, b)) = Self::item_pipelines(gpu, assets) {
            gpu.destroy_pipeline(self.item_pipe);
            gpu.destroy_pipeline(self.hand_item_pipe);
            (self.item_pipe, self.hand_item_pipe) = (a, b);
        }
        // Sprites and the models made of them are read again.
        self.sprites.clear();
        self.item_models.clear();
        self.item_boxes.clear();
        match Self::pipelines(gpu, assets) {
            Ok((a, b)) => {
                gpu.destroy_pipeline(self.pipeline);
                gpu.destroy_pipeline(self.overlay);
                self.pipeline = a;
                self.overlay = b;
            }
            Err(e) => log::error!("entity shader: {e}"),
        }
    }

    /// Uploads player `entity`'s own skin.
    pub fn set_skin(&mut self, gpu: &mut dyn Backend, entity: u64, png: &[u8]) {
        let Some(img) = crate::assets::decode_png(png) else { return };
        let t = gpu.create_texture(
            &TextureDesc { width: img.width, height: img.height, mips: 1, format: Format::Rgba8Srgb, render_target: false },
            "player skin",
        );
        gpu.write_texture(t, 0, &img.rgba);
        if let Some(old) = self
            .player_skins
            .insert(entity, (t, gpu.texture_index(t), [img.width, img.height]))
        {
            gpu.destroy_texture(old.0);
        }
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        for (t, ..) in self
            .player_skins
            .drain()
            .map(|(_, v)| v)
            .chain(self.painted.drain().map(|(_, p)| p.tex))
        {
            gpu.destroy_texture(t);
        }
        for (b, _) in self.buffers.drain(..) {
            gpu.destroy_buffer(b);
        }
        self.models.destroy(gpu);
        gpu.destroy_pipeline(self.model_pipe);
        gpu.destroy_pipeline(self.hand_pipe);
        gpu.destroy_pipeline(self.preview_pipe);
        if let Some(s) = self.shadow_pipe.take() {
            gpu.destroy_pipeline(s);
        }
        if let Some((c, d, ..)) = self.preview_target.take() {
            gpu.destroy_texture(c);
            gpu.destroy_texture(d);
        }
        let preview: Vec<BufferId> = self.preview.drain(..).collect();
        for b in self.hand_views.drain(..).chain(preview) {
            gpu.destroy_buffer(b);
        }
        for t in self
            .skins
            .drain()
            .filter_map(|(_, v)| v)
            .map(|v| v.0)
            .chain(self.glows.drain().map(|(_, g)| g.0))
        {
            gpu.destroy_texture(t);
        }
        gpu.destroy_pipeline(self.pipeline);
        gpu.destroy_pipeline(self.overlay);
        gpu.destroy_pipeline(self.item_pipe);
        gpu.destroy_pipeline(self.hand_item_pipe);
        if let Some((b, ..)) = self.item_buf.take() {
            gpu.destroy_buffer(b);
        }
    }

    /// Loads skins and worn textures the first time something visible uses them; `worn` is what
    /// the local player wears.
    pub fn preload(
        &mut self,
        gpu: &mut dyn Backend,
        assets: &Assets,
        entities: &FxHashMap<u64, ClientEntity>,
        comps: &FxHashMap<(u64, u32), Vec<u8>>,
        worn: &[u32],
        held: u32,
    ) {
        let c = content();
        let s = crate::diag::span("preload/skins");
        self.paint_skins(gpu, assets, entities, comps);
        drop(s);
        self.load_sprite(assets, held);
        // Each kind and item looked at once per frame, without listing thousands of entities.
        let mut seen = std::mem::take(&mut self.seen_kinds);
        seen.clear();
        seen.resize(c.game.entities.len(), false);
        let mut seen_items = std::mem::take(&mut self.seen_items);
        seen_items.clear();
        seen_items.resize(c.items.len(), false);
        for e in entities.values() {
            if !e.item.is_empty() && seen_items.get(e.item.item as usize).is_some_and(|s| !s) {
                seen_items[e.item.item as usize] = true;
                self.load_sprite(assets, e.item.item);
            }
            if !std::mem::replace(&mut seen[e.look() as usize], true) {
                self.load(gpu, assets, &c.game.entities[e.look() as usize].texture);
            }
            for &i in &e.worn {
                if let Some((_, t)) = c.worn(i) {
                    self.load(gpu, assets, t);
                }
            }
            if e.held != 0 && seen_items.get(e.held as usize).is_some_and(|s| !s) {
                seen_items[e.held as usize] = true;
                self.load_sprite(assets, e.held);
            }
        }
        self.seen_kinds = seen;
        self.seen_items = seen_items;
        self.load(gpu, assets, &c.game.entities[c.game.player_kind as usize].texture);
        for &i in worn {
            if let Some((_, t)) = c.worn(i) {
                self.load(gpu, assets, t);
            }
        }
    }

    /// Paints the skins mods paint (`Host::reg_entity_skin`) of entities whose component is new
    /// or changed, and forgets those of entities gone. The painters run on `skin_painters()` (the
    /// SDK has them `Send + Sync`): a coat took 0.3-2.5 ms, a big model's 18-42 ms (the animals'
    /// wyvern, 1024 x 544), and eight a frame on this thread made 20-60 ms frames whenever animals
    /// came into view (the Mac lag runs, 11-04). Each is shown when done, over the plain texture
    /// until then.
    fn paint_skins(
        &mut self,
        gpu: &mut dyn Backend,
        assets: &Assets,
        entities: &FxHashMap<u64, ClientEntity>,
        comps: &FxHashMap<(u64, u32), Vec<u8>>,
    ) {
        let c = content();
        let painters = self.painters.get_or_insert_with(|| {
            c.game
                .skin_painters
                .iter()
                .filter_map(|p| {
                    let kind = *c.game.entity_names.get(&p.kind)?;
                    let comp = c.components.id(&p.comp)?;
                    Some((kind, (comp, p.owner, p.cb)))
                })
                .collect()
        });
        if painters.is_empty() {
            return;
        }
        self.painted.retain(|id, p| {
            let keep = entities.contains_key(id);
            if !keep {
                gpu.destroy_texture(p.tex.0);
            }
            keep
        });
        self.builds.retain(|id, _| entities.contains_key(id));
        // What the painters finished: shown if the entity is still here and the textures weren't
        // read again since (`reload`).
        while let Ok(done) = self.paint_done.1.try_recv() {
            if done.generation != self.paint_generation {
                continue;
            }
            self.painting.remove(&done.id);
            let (Some(set), Some(e)) = (done.bones, entities.get(&done.id)) else { continue };
            if set.is_empty() {
                self.builds.remove(&done.id);
            } else {
                self.builds
                    .insert(done.id, Build { bones: set, resolved: None });
            }
            let size = done.size;
            let tex = match self.painted.remove(&done.id) {
                Some(p) if p.tex.2 == size => p.tex,
                old => {
                    if let Some(p) = old {
                        gpu.destroy_texture(p.tex.0);
                    }
                    let t = gpu.create_texture(
                        &TextureDesc { width: size[0], height: size[1], mips: 1, format: Format::Rgba8Srgb, render_target: false },
                        "painted skin",
                    );
                    (t, gpu.texture_index(t), size)
                }
            };
            gpu.write_texture(tex.0, 0, &done.rgba);
            let texture = c.game.entities[e.kind as usize].texture.clone();
            self.painted
                .insert(done.id, Painted { from: done.from, texture, tex });
        }
        for (&id, e) in entities {
            if self.painting.len() >= PAINTS_IN_FLIGHT {
                break;
            }
            let Some(&(comp, owner, cb)) = painters.get(&e.kind) else { continue };
            let Some(from) = comps.get(&(id, comp)) else { continue };
            // One at a time each: a value that changed while it was painted is painted again
            // once that one is shown.
            if self.painting.contains(&id) || self.painted.get(&id).is_some_and(|p| p.from == *from) {
                continue;
            }
            let texture = &c.game.entities[e.kind as usize].texture;
            let base = self.skin_bases.entry(texture.clone()).or_insert_with(|| {
                let img = assets
                    .read(texture, "textures", "png")
                    .and_then(|b| decode_png(&b))?;
                Some((std::sync::Arc::new(img.rgba), [img.width, img.height]))
            });
            let Some((rgba, size)) = base.clone() else { continue };
            self.painting.insert(id);
            let (from, tx, generation) = (from.clone(), self.paint_done.0.clone(), self.paint_generation);
            skin_painters().spawn(move || {
                let mut rgba = rgba.as_ref().clone();
                let mut bones = [weft_abi::BoneScale { name: [0; 32], scale: [1.0; 3] }; MAX_BUILD_BONES];
                let mut args = weft_abi::SkinArgs {
                    data: weft_abi::Slice::new(&from),
                    rgba: rgba.as_mut_ptr(),
                    width: size[0],
                    height: size[1],
                    bones: bones.as_mut_ptr(),
                    bone_cap: MAX_BUILD_BONES as u32,
                    bone_count: 0,
                };
                let painted = crate::host::invoke(owner, cb, &mut args);
                let set = painted.then(|| {
                    bones[..(args.bone_count as usize).min(MAX_BUILD_BONES)]
                        .iter()
                        .filter(|b| b.scale.iter().all(|v| v.is_finite()))
                        .map(|b| {
                            let end = b.name.iter().position(|&c| c == 0).unwrap_or(b.name.len());
                            (String::from_utf8_lossy(&b.name[..end]).into_owned(), b.scale.map(|v| v.clamp(0.2, 5.0)))
                        })
                        .collect()
                });
                let _ = tx.send(PaintDone { id, generation, from, rgba, size, bones: set });
            });
        }
    }

    /// The opaque pixels of an item's sprite, read the first time it is held or dropped.
    fn load_sprite(&mut self, assets: &Assets, item: u32) {
        let Some(name) = sprite_name(item) else { return };
        if self.sprites.contains_key(&name) {
            return;
        }
        let img = assets
            .read(&name, "textures", "png")
            .and_then(|b| decode_png(&b));
        let sprite = img.map(|img| sprite_of(img.width, &img.rgba));
        self.sprites.insert(name, sprite);
    }

    /// An item's model as boxes in the unit cube, made once: a block item's shape, else its
    /// sprite one texel thick (flat when the sprite could not be read); a stack with a look of
    /// its own (`abi::comp::LOOK`) is that picture. (model key, (first box, count)).
    fn item_model(&mut self, textures: &GameTextures, item: u32, look: Option<&str>) -> Option<(u64, (u32, u32))> {
        let look = look.and_then(|l| textures.look(l));
        let key = model_key(item, look);
        if let Some(&m) = self.item_models.get(&key) {
            return Some((key, m));
        }
        let c = content();
        let first = self.item_boxes.len();
        let white = Vec4::new(1.0, 1.0, 1.0, 0.0);
        let looks = super::looks::current();
        let sprite = match looks.item_draw(c, item) {
            _ if c.items.get(item as usize).is_none() || item == 0 => return None,
            _ if look.is_some() => look,
            ItemDraw::Model(model) => {
                boxes_shape(&mut self.item_boxes, textures, Mat4::IDENTITY, model, &c.states[0], white);
                None
            }
            ItemDraw::Icon(t) => Some(t),
            ItemDraw::Block(state) => {
                let s = &c.states[state as usize];
                let sprite = looks.block_sprite(state, s);
                if sprite.is_none() {
                    boxes_shape(&mut self.item_boxes, textures, Mat4::IDENTITY, &looks.block_shape(state, s), s, white);
                }
                sprite
            }
            ItemDraw::Nothing => return None,
        };
        if let Some(t) = sprite {
            let l = textures.texture(t);
            // A look's pixels come with its texture, once it has loaded (a load remakes models).
            let own = look
                .and_then(|t| textures.look_pixels(t))
                .map(|(size, rgba)| sprite_of(size, &rgba));
            let from_file = || {
                sprite_name(item)
                    .and_then(|n| self.sprites.get(&n))
                    .and_then(|s| s.as_ref())
            };
            match own
                .as_ref()
                .or_else(|| look.is_none().then(from_file).flatten())
            {
                Some(sp) => {
                    let n = sp.size as f32;
                    // Only the outline: a side face against a neighboring pixel would fight
                    // with it. Mode 4: every face takes the color of this one texel.
                    for &(x, y, hidden) in &sp.pixels {
                        let m = Mat4::from_translation(Vec3::new(x as f32 / n, 1.0 - (y as f32 + 1.0) / n, 7.5 / 16.0))
                            * Mat4::from_scale(Vec3::new(1.0 / n, 1.0 / n, 1.0 / 16.0));
                        self.item_boxes.push(GpuBox {
                            rows: rows(m),
                            uvo: [(x as f32 + 0.5) / n, (y as f32 + 0.5) / n, 0.0, 0.0],
                            color: white.to_array(),
                            ids: [0, box_mode::TEXEL, l, hidden],
                            ..Default::default()
                        });
                    }
                }
                None => {
                    let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.5)) * Mat4::from_scale(Vec3::new(1.0, 1.0, 0.0));
                    self.item_boxes.push(GpuBox {
                        rows: rows(m),
                        color: white.to_array(),
                        ids: [0, box_mode::FACES, l | l << 16, l | l << 16],
                        ids2: [l | l << 16, 0, 0, 0],
                        ..Default::default()
                    });
                }
            }
        }
        let m = (first as u32, (self.item_boxes.len() - first) as u32);
        self.item_models.insert(key, m);
        Some((key, m))
    }

    /// Item models made since the last upload go to the GPU; a new texture generation remakes
    /// them all.
    fn upload_items(&mut self, gpu: &mut dyn Backend) {
        let n = self.item_boxes.len();
        let (buf, cap, done) = match self.item_buf {
            Some((b, cap, done)) if (n as u32) <= cap => (b, cap, done.min(n)),
            old => {
                if let Some((b, ..)) = old {
                    gpu.destroy_buffer(b);
                }
                let cap = (n as u32).max(1024).next_power_of_two();
                (gpu.create_buffer(cap as u64 * 128, usage::STORAGE, "item models"), cap, 0)
            }
        };
        if done < n {
            gpu.write_buffer(buf, done as u64 * 128, bytemuck::cast_slice(&self.item_boxes[done..]));
        }
        self.item_buf = Some((buf, cap, n));
    }

    fn load(&mut self, gpu: &mut dyn Backend, assets: &Assets, name: &str) {
        if name.is_empty() || self.skins.contains_key(name) {
            return;
        }
        let loaded = assets
            .read(name, "textures", "png")
            .and_then(|b| decode_png(&b))
            .map(|img| {
                let t = gpu.create_texture(
                    &TextureDesc { width: img.width, height: img.height, mips: 1, format: Format::Rgba8Srgb, render_target: false },
                    name,
                );
                gpu.write_texture(t, 0, &img.rgba);
                (t, gpu.texture_index(t), [img.width, img.height])
            });
        if loaded.is_none() {
            log::warn!("missing entity texture {name}");
        }
        if let Some(mask) = crate::assets::emission_mask(assets, name) {
            let t = gpu.create_texture(
                &TextureDesc { width: mask.width, height: mask.height, mips: 1, format: Format::R8Unorm, render_target: false },
                name,
            );
            gpu.write_texture(t, 0, &mask.rgba);
            self.glows
                .insert(name.to_string(), (t, gpu.texture_index(t)));
        }
        self.skins.insert(name.to_string(), loaded);
    }

    /// Model instances of `entity`, of `kind`: its skin (a player's own if they sent one) and what
    /// it wears (how many in `.1`).
    fn layers(&self, kind: u32, entity: u64, worn: &[u32], light: u32, hurt: f32) -> ([Instance; MAX_LAYERS], usize) {
        let c = content();
        let k = &c.game.entities[kind as usize];
        let mut out = ([Instance::default(); MAX_LAYERS], 0);
        let Some(Some((_, skin, _))) = self.skins.get(&k.texture) else { return out };
        let glow = |name: &str| self.glows.get(name).map_or(u32::MAX, |g| g.1);
        let size = self
            .models
            .kinds
            .get(kind as usize)
            .and_then(|m| m.texture)
            .unwrap_or(k.texture_size);
        // A player's own skin; a painted one on any look drawn with the texture it was painted
        // over (an animal sitting, lying, a puppy).
        let own = self
            .player_skins
            .get(&entity)
            .filter(|_| kind == c.game.player_kind)
            .or_else(|| {
                self.painted
                    .get(&entity)
                    .filter(|p| p.texture == k.texture)
                    .map(|p| &p.tex)
            });
        let layout = size;
        let (skin, size) = own.map_or((skin, size), |o| (&o.1, o.2));
        let base = Instance {
            color: [1.0; 3],
            light,
            hurt,
            tex: *skin,
            tex_size: [size[0] as f32, size[1] as f32],
            roles: u32::MAX,
            shader: self.kind_shaders.get(kind as usize).copied().unwrap_or(0),
            glow: glow(&k.texture),
            ..Default::default()
        };
        out.0[0] = base;
        out.1 = 1;
        for &i in worn {
            if let Some((w, t)) = c.worn(i) {
                if let Some(Some((_, idx, size))) = self.skins.get(t).filter(|_| out.1 < MAX_LAYERS) {
                    out.0[out.1] = Instance {
                        tex: *idx,
                        tex_size: worn_tex_size(layout, *size),
                        roles: crate::model::role_mask(w.roles),
                        inflate: w.inflate,
                        glow: glow(t),
                        ..base
                    };
                    out.1 += 1;
                }
            }
        }
        out
    }

    /// Renders the local player's model into a `size` texture (physical pixels), turned towards
    /// `look` (relative to the texture's top left) like the inventory figure. Returns the
    /// texture's bindless index for the UI to draw.
    pub fn draw_preview(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        size: [u32; 2],
        look: glam::Vec2,
        worn: &[u32],
        subject: Option<(u64, u32)>,
    ) -> Option<u32> {
        let c = content();
        if size[0] == 0 || size[1] == 0 {
            return None;
        }
        // The local player (the inventory's figure), or another entity (a menu's own, such as an
        // animal's screen), fitted to the frame by its size and seen from three quarters.
        let (entity, kind) = subject.unwrap_or((self.local, c.game.player_kind));
        let Some(def) = c.game.entities.get(kind as usize) else { return None };
        if self.preview_target.is_none_or(|t| t.2 != size) {
            if let Some((col, dep, ..)) = self.preview_target.take() {
                gpu.destroy_texture(col);
                gpu.destroy_texture(dep);
            }
            let desc = |format| TextureDesc { width: size[0], height: size[1], mips: 1, format, render_target: true };
            let col = gpu.create_texture(&desc(Format::Rgba8Srgb), "preview color");
            let dep = gpu.create_texture(&desc(Format::Depth32), "preview depth");
            self.preview_target = Some((col, dep, size, gpu.texture_index(col)));
        }
        let (col, dep, _, index) = self.preview_target?;
        let (x, y, w, h) = (0.0, 0.0, size[0] as f32, size[1] as f32);
        let (sw, sh) = (w, h);
        let player = subject.is_none();
        // Pixels a block: the player as before; another kind by its height and its length, about
        // twice its width for a four-legged one, with room around.
        let s = if player {
            h / 2.4
        } else {
            let (bw, bh) = (def.width.max(0.1), def.height.max(0.1));
            (h * 0.8 / bh).min(w * 0.8 / (bw * 2.2).max(bh))
        };
        let (cx, feet) = (x + w / 2.0, y + h - h * 0.08);
        let head = feet - if player { 1.5 } else { def.height * 0.8 } * s;
        let turn = if player { 0.0 } else { 0.7 };
        let yaw = std::f32::consts::PI + turn + ((look.x - cx) / (s * 1.6)).atan() * 0.7;
        let pitch = ((head - look.y) / (s * 1.6)).atan() * 0.5;
        let proj = Mat4::from_cols(
            Vec4::new(2.0 * s / sw, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 * s / sh, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 0.1, 0.0),
            Vec4::new(2.0 * cx / sw - 1.0, 1.0 - 2.0 * feet / sh, 0.5, 1.0),
        );
        let layers = self.layers(kind, entity, if player { worn } else { &[] }, FULL_LIGHT, 0.0);
        self.models.begin();
        let root = Mat4::from_rotation_y(yaw) * Mat4::from_scale(Vec3::splat(1.0 / 16.0));
        self.models.add(
            kind,
            root,
            &crate::model::Motion {
                pitch,
                head_yaw: 0.0,
                swing: 0.0,
                swim: None,
                holding: false,
                attack: 0.0,
                anims: &[],
                crouch: false,
                sit: false,
                build: &[],
            },
            layers.0[..layers.1].iter().copied(),
        );
        let vb = self.preview[frame.slot];
        // Full daylight, no fog.
        let v = Scene { view_proj: proj.to_cols_array_2d(), params: [1e9, 2e9, 1.0, crate::shaders::time()], ..Default::default() };
        gpu.mapped(vb)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&v));
        list.push(Cmd::BeginPass { color: Some((Target::Texture(col), Some([0.0; 4]))), depth: Some((dep, Some(0.0))) });
        self.models
            .draw(gpu, list, frame, super::models::Batch::Preview, self.preview_pipe, vb);
        list.push(Cmd::EndPass);
        Some(index)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        entities: &FxHashMap<u64, ClientEntity>,
        me: Option<&ClientEntity>,
        shadow_me: Option<&ClientEntity>,
        hand: Option<Hand>,
        textures: &GameTextures,
        light: &impl Fn(IVec3) -> u32,
        scene: &Scene,
        scene_buf: BufferId,
        target: Option<(Hit, DVec3)>,
        breaking: &[(IVec3, f32, DVec3)],
        gizmo_lines: &[super::gizmo::Line],
        gizmo_fills: &[super::gizmo::Fill],
        late: &mut CommandList,
    ) {
        let c = content();
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let mut boxes: Vec<GpuBox> = Vec::new();
        if textures.generation() != self.item_generation {
            self.item_generation = textures.generation();
            self.item_models.clear();
            self.item_boxes.clear();
            if let Some(b) = &mut self.item_buf {
                b.2 = 0;
            }
        }
        let mut instances = std::mem::take(&mut self.instances);
        instances.clear();
        let rel = |p: DVec3| (p - cam.pos).as_vec3();
        let shade = |p: DVec3| light(p.floor().as_ivec3());

        // Models: bones posed here (in parallel when many), drawn instanced per kind.
        let mut posed = std::mem::take(&mut self.posed);
        posed.clear();
        let mut frame_builds = std::mem::take(&mut self.frame_builds);
        frame_builds.clear();
        // Who holds something: its place in `posed`, the item, its look and its light.
        let mut holders: Vec<(usize, u32, Option<&str>, u32)> = Vec::new();
        // The player's own body in first person only casts its shadow (`shadow_me`).
        for (id, e, shadow_only) in entities
            .iter()
            .map(|(i, e)| (*i, e, false))
            .chain(me.map(|m| (self.local, m, false)))
            .chain(
                shadow_me
                    .filter(|_| self.shadow_pipe.is_some())
                    .map(|m| (self.local, m, true)),
            )
        {
            let base = rel(e.pos);
            if base.length_squared() > 160.0 * 160.0 {
                continue;
            }
            let lit = shade(e.pos + DVec3::Y * 0.5);
            // Below zero the model's shader flashes white instead of red.
            let hurt = if e.flash > 0.0 { -1.0 } else { (e.hurt / 0.5).clamp(0.0, 1.0) };
            if !e.item.is_empty() {
                if let Some((key, m)) = self.item_model(textures, e.item.item, crate::stack::look(&e.item)) {
                    instances.push((key, item_instance(ground_transform(base, e.age, e.item.item), m.0, hurt, lit)));
                }
                continue;
            }
            let kind = e.look();
            let (layers, layer_count) = self.layers(kind, id, &e.worn, lit, hurt);
            if layer_count == 0 {
                continue;
            }
            if e.burning && !shadow_only {
                let (hw, h, _) = e.dims();
                let from = boxes.len();
                fire_boxes(&mut boxes, textures, base, hw as f32 * 2.0, h as f32, cam.yaw, e.age);
                light_boxes(&mut boxes, from, FULL_LIGHT);
            }
            // Vanilla's limb swing: 0.6662 rad per tick of limbSwing, which grows 4 per block;
            // models with their own walk animation use that instead.
            let mut swing = (e.walk * 2.66).sin() * 1.2 * e.swing;
            let (mut anims, mut anim_count) = ([(0, 0.0, 0.0); crate::client::models::MAX_ANIMS], 0);
            let mut add = |a| {
                anims[anim_count] = a;
                anim_count += 1;
            };
            if let Some(m) = self.models.kinds.get(kind as usize) {
                match m.playing(e.anim.as_ref(), e.age) {
                    Some((i, t)) => {
                        add((i, t, 1.0));
                        swing = 0.0;
                    }
                    _ => {
                        if let Some(i) = m.idle {
                            add((i, m.anims[i].at(e.age, true), 1.0));
                        }
                        if let Some(i) = m.walk {
                            // Smaller steps, quicker: a small one's legs go round faster. Both
                            // gaits are timed by the distance moved, so their feet keep pace.
                            let at = e.walk * 0.6 / e.scale;
                            let moving = e.swing.min(1.0);
                            // Running takes over between 3 and 4.5 blocks a second (`swing` is
                            // the speed over 5, eased).
                            let fast = ((e.swing - 0.6) / 0.3).clamp(0.0, 1.0);
                            let run = if m.run.is_some() { fast } else { 0.0 };
                            add((i, m.anims[i].at(at, true), moving * (1.0 - run)));
                            if let Some(r) = m.run.filter(|_| run > 0.0) {
                                add((r, m.anims[r].at(at, true), moving * run));
                            }
                            swing = 0.0;
                        }
                    }
                }
            }
            // Vanilla: sqrt of the death time over a second, up to a quarter turn.
            let roll = e
                .dying
                .map_or(0.0, |d| (d * 1.6).sqrt().min(1.0) * std::f32::consts::FRAC_PI_2);
            // Lying down in its low box: head first where it looks when swimming (vanilla's
            // setupRotations), flat when crawling, turned around the middle of the body.
            let lying = match e.pose {
                crate::world::Pose::Swim => Some(e.pitch - std::f32::consts::FRAC_PI_2),
                crate::world::Pose::Crawl => Some(-std::f32::consts::FRAC_PI_2),
                // On its back, the head toward its back's side.
                crate::world::Pose::Sleep => Some(std::f32::consts::FRAC_PI_2),
                _ => None,
            };
            let sleeping = e.pose == crate::world::Pose::Sleep;
            let body_yaw = if sleeping { e.yaw } else { e.body_yaw };
            let swim = lying.map_or(Mat4::IDENTITY, |a| {
                Mat4::from_translation(Vec3::new(0.0, 0.3, 0.0))
                    * Mat4::from_rotation_x(a)
                    * Mat4::from_translation(Vec3::new(0.0, -0.9, 0.0))
            });
            // Seated, its position is the seat: the hips rest a little above it.
            let seated = e.pose == crate::world::Pose::Sit;
            let base = if seated { base - Vec3::Y * 0.65 } else { base };
            // Its build (its painter's bone scales) for this kind's parts, and how far that lifts it.
            let mut build = (0, 0);
            let mut lift = 0.0;
            if let (Some(b), Some(m)) = (self.builds.get_mut(&id), self.models.kinds.get(kind as usize)) {
                if b.resolved.as_ref().is_none_or(|r| r.0 != kind) {
                    let scales: Vec<[f32; 3]> = m
                        .parts
                        .iter()
                        .map(|p| {
                            b.bones
                                .iter()
                                .find(|(n, _)| n.eq_ignore_ascii_case(&p.name))
                                .map_or([1.0; 3], |(_, s)| *s)
                        })
                        .collect();
                    let lift = crate::model::lowest(&m.parts, &[]) - crate::model::lowest(&m.parts, &scales);
                    b.resolved = Some((kind, scales, lift));
                }
                if let Some((_, scales, l)) = &b.resolved {
                    build = (frame_builds.len() as u32, scales.len() as u32);
                    frame_builds.extend_from_slice(scales);
                    lift = *l;
                }
            }
            let root = Mat4::from_translation(base)
                * Mat4::from_rotation_y(body_yaw)
                * Mat4::from_rotation_z(roll)
                * swim
                * Mat4::from_scale(Vec3::splat(e.scale / 16.0))
                * Mat4::from_translation(Vec3::Y * lift);
            // Its head up to look ahead, and limbs by the swimming stroke.
            let swim_limbs = lying.filter(|_| !sleeping).map(|_| e.walk * 4.0);
            let (pitch, head_yaw) = if sleeping {
                (0.0, 0.0)
            } else if lying.is_some() {
                (std::f32::consts::FRAC_PI_4, 0.0)
            } else {
                (e.pitch, e.yaw - e.body_yaw)
            };
            posed.push(Posed {
                kind,
                root,
                pitch,
                head_yaw,
                swing,
                crouch: e.pose == crate::world::Pose::Crouch,
                sit: seated,
                swim: swim_limbs,
                attack: e.arm_swing.unwrap_or(0.0),
                holding: e.held != 0 && !sleeping,
                anims,
                anim_count,
                layers,
                layer_count,
                build,
                shadow_only,
            });
            if e.held != 0 && !sleeping && !shadow_only {
                holders.push((posed.len() - 1, e.held, e.held_look.as_deref(), lit));
            }
        }
        let vb = scene_buf;
        self.models.begin();
        let first_bone = self.models.bones_so_far();
        self.models.add_many(&posed, &frame_builds);
        let (mut at, mut next) = (first_bone, 0);
        for (holder, item, look, lit) in holders {
            at += posed[next..holder]
                .iter()
                .filter_map(|p| self.models.kinds.get(p.kind as usize))
                .map(|m| m.parts.len())
                .sum::<usize>();
            next = holder;
            let parts = self
                .models
                .kinds
                .get(posed[holder].kind as usize)
                .map(|m| m.parts.as_slice())
                .unwrap_or_default();
            let Some(i) = parts
                .iter()
                .position(|p| p.role == crate::model::role::ARM_RIGHT)
            else {
                continue;
            };
            let arm = self.models.bone(at + i) * Mat4::from_translation(Vec3::from_array(parts[i].pivot));
            if let Some((key, m)) = self.item_model(textures, item, look) {
                instances.push((key, item_instance(held_transform(arm, item), m.0, 0.0, lit)));
            }
        }
        self.posed = posed;
        self.frame_builds = frame_builds;
        self.models
            .draw(gpu, list, frame, super::models::Batch::World, self.model_pipe, vb);

        // Gizmo lines: thin boxes like the outline, a little thicker so they read from afar.
        for &(a, b, color) in gizmo_lines {
            let (a, b) = (rel(a), rel(b));
            let d = b - a;
            let len = d.length();
            if len < 1e-4 || a.length_squared().min(b.length_squared()) > 160.0 * 160.0 {
                continue;
            }
            let t = 0.012 + 0.0012 * a.length().min(b.length());
            let rot = Quat::from_rotation_arc(Vec3::X, d / len);
            let m = Mat4::from_rotation_translation(rot, a)
                * Mat4::from_translation(Vec3::new(0.0, -t / 2.0, -t / 2.0))
                * Mat4::from_scale(Vec3::new(len, t, t));
            let rgb = |s: u32| ((color >> s) & 255) as f32 / 255.0;
            boxes.push(GpuBox {
                rows: rows(m),
                color: [rgb(24), rgb(16), rgb(8), 0.0],
                ids: [0, box_mode::SOLID, 0, 0],
                ..Default::default()
            });
        }

        let overlay_start = boxes.len();
        // Selection outline: twelve thin boxes along the edges of each selection box, see-through
        // and about a pixel wide wherever the edge is (vanilla's faint thin lines). Ronald: our
        // solid black ones, a fixed 0.012 blocks thick, were 3 px up close and too heavy; he picked
        // "thin and faint like vanilla", about 1 px at about 40 % dark.
        let px_at_1 = 2.0 * (cam.fov.to_radians() * 0.5).tan() / frame.height.max(1) as f32;
        let outlined = target.map(|(hit, at)| (&c.states[hit.state as usize].selection, rel(hit.block.as_dvec3() + at)));
        for (sel, o) in outlined
            .iter()
            .flat_map(|(sel, o)| sel.iter().map(move |s| (s, *o)))
        {
            let (mn, mx) = (Vec3::new(sel[0], sel[1], sel[2]), Vec3::new(sel[3], sel[4], sel[5]));
            for axis in 0..3 {
                for corner in 0..4 {
                    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                    // The edge's middle, and its half width there: OUTLINE_PX pixels across.
                    let mut mid = (o + mn + o + mx) * 0.5;
                    mid[u] = if corner & 1 == 0 { o[u] + mn[u] } else { o[u] + mx[u] };
                    mid[v] = if corner & 2 == 0 { o[v] + mn[v] } else { o[v] + mx[v] };
                    let t = (mid.length() * px_at_1 * OUTLINE_PX * 0.5).max(0.0015);
                    let mut lo = mid - Vec3::splat(t);
                    let mut size = Vec3::splat(t * 2.0);
                    lo[axis] = o[axis] + mn[axis] - t;
                    size[axis] = mx[axis] - mn[axis] + t * 2.0;
                    let m = Mat4::from_translation(lo) * Mat4::from_scale(size);
                    boxes.push(GpuBox {
                        rows: rows(m),
                        color: [0.0, 0.0, 0.0, 0.0],
                        ids: [0, box_mode::FILLED, OUTLINE_DARK.to_bits(), 0],
                        ..Default::default()
                    });
                }
            }
        }
        // Filled gizmo boxes: see-through by their own alpha (mode 5), slightly grown so they do
        // not fight with the faces of the blocks they cover.
        for &(a, b, color) in gizmo_fills {
            let (a, b) = (rel(a) - Vec3::splat(0.002), rel(b) + Vec3::splat(0.002));
            if a.length_squared().min(b.length_squared()) > 160.0 * 160.0 {
                continue;
            }
            let m = Mat4::from_translation(a) * Mat4::from_scale(b - a);
            let ch = |s: u32| ((color >> s) & 255) as f32 / 255.0;
            boxes.push(GpuBox {
                rows: rows(m),
                color: [ch(24), ch(16), ch(8), 0.0],
                ids: [0, box_mode::FILLED, ch(0).to_bits(), 0],
                ..Default::default()
            });
        }
        for &(p, progress, at) in breaking {
            if progress > 0.0 {
                // Over the faces of the block's model (a ladder's board, not its whole cube),
                // textured like the block's own faces there.
                let stage = ((progress * 10.0) as usize).min(9);
                let l = textures.texture(c.crack_textures[stage]);
                let lit = shade(p.as_dvec3() + at + DVec3::splat(0.5));
                let state = target
                    .filter(|h| h.0.block == p)
                    .map(|h| &c.states[h.0.state as usize]);
                let shape: Vec<(Vec3, Vec3)> = match state {
                    Some(s) => s
                        .selection
                        .iter()
                        .map(|b| (Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
                        .collect(),
                    None => vec![(Vec3::ZERO, Vec3::ONE)],
                };
                for (mn, mx) in shape {
                    let (a, b) = (mn - Vec3::splat(0.004), mx + Vec3::splat(0.004));
                    let m = Mat4::from_translation(rel(p.as_dvec3() + at) + a) * Mat4::from_scale(b - a);
                    boxes.push(GpuBox {
                        rows: rows(m),
                        dims: mn.extend(1.0).to_array(),
                        uvo: mx.extend(0.0).to_array(),
                        color: [1.0, 1.0, 1.0, 0.0],
                        ids: [0, box_mode::FACES, l | l << 16, l | l << 16],
                        ids2: [l | l << 16, lit, 0, 0],
                    });
                }
            }
        }
        let hand_start = boxes.len();
        let mut hand_item: Option<(GpuBox, u32)> = None;
        if let Some(h) = hand {
            let k = &c.game.entities[c.game.player_kind as usize];
            let player_skin = match (self.player_skins.get(&self.local), self.skins.get(&k.texture)) {
                (Some((_, idx, size)), _) => Some((*idx, [size[0] as f32, size[1] as f32])),
                (None, Some(Some((_, idx, _)))) => Some((*idx, [k.texture_size[0] as f32, k.texture_size[1] as f32])),
                _ => None,
            };
            // Keyframes may move the hand too: the `hand` bone of what plays (else of idle).
            let pose = self
                .models
                .kinds
                .get(c.game.player_kind as usize)
                .and_then(|m| {
                    let (i, t) = m
                        .playing(h.anim.as_ref(), h.age)
                        .or_else(|| m.idle.map(|i| (i, m.anims[i].at(h.age, true))))?;
                    m.anims[i].anim.free_bone("hand", t)
                });
            let from = boxes.len();
            match hand_transform(cam, &h, pose.unwrap_or(Mat4::IDENTITY)) {
                Some(m) => {
                    hand_item = self
                        .item_model(textures, h.stack.item, crate::stack::look(&h.stack))
                        .map(|(_, (first, n))| (item_instance(m, first, 0.0, shade(cam.pos)), n))
                }
                None => {
                    // The arm is the player model's own right arm bone.
                    let parts = self
                        .models
                        .kinds
                        .get(c.game.player_kind as usize)
                        .map_or(&k.parts, |m| &m.parts);
                    let arm = parts
                        .iter()
                        .find(|p| p.name.eq_ignore_ascii_case("rightArm"));
                    hand_boxes(&mut boxes, cam, &h, player_skin, arm, pose.unwrap_or(Mat4::IDENTITY));
                    light_boxes(&mut boxes, from, shade(cam.pos));
                }
            }
        }
        // Item instances after the hand, grouped by item: one instanced draw each.
        let hand_end = boxes.len();
        instances.sort_unstable_by_key(|i| i.0);
        boxes.extend(instances.iter().map(|i| i.1));
        if let Some((inst, _)) = hand_item {
            boxes.push(inst);
        }
        if boxes.is_empty() {
            self.instances = instances;
            return;
        }
        if !instances.is_empty() || hand_item.is_some() {
            self.upload_items(gpu);
        }

        let buf =
            crate::gpu_layout::frame_buffer(gpu, &mut self.buffers, frame.slot, boxes.len() as u32, 128, usage::STORAGE, "entity boxes");
        gpu.mapped(buf)[..boxes.len() * 128].copy_from_slice(bytemuck::cast_slice(&boxes));
        let glow = textures.glow_buffer();

        if overlay_start > 0 {
            list.pipeline(self.pipeline);
            list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
            list.bind(crate::gpu_layout::bind::entity::SCENE, vb);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
            list.draw(overlay_start as u32 * 36, 1);
        }
        let item_buf = self.item_buf.map(|b| b.0);
        if let (Some(ib), false) = (item_buf, instances.is_empty()) {
            list.pipeline(self.item_pipe);
            list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
            list.bind(crate::gpu_layout::bind::entity::SCENE, vb);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
            list.bind(crate::gpu_layout::bind::entity::ITEM_BOXES, ib);
            let mut i = 0;
            while i < instances.len() {
                let key = instances[i].0;
                let run = instances[i..].iter().take_while(|x| x.0 == key).count();
                let boxes_per = self.item_models.get(&key).map_or(0, |m| m.1);
                list.push(Cmd::Draw {
                    vertices: boxes_per * 36,
                    instances: run as u32,
                    first_vertex: 0,
                    first_instance: (hand_end + i) as u32,
                });
                i += run;
            }
        }
        // Cracks, filled gizmos and the hand come after translucent blocks.
        let list = late;
        if hand_start > overlay_start {
            list.pipeline(self.overlay);
            list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
            list.bind(crate::gpu_layout::bind::entity::SCENE, vb);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
            list.push(Cmd::Draw {
                vertices: (hand_start - overlay_start) as u32 * 36,
                instances: 1,
                first_vertex: overlay_start as u32 * 36,
                first_instance: 0,
            });
        }
        if hand_end > hand_start || hand_item.is_some() {
            let hand_cam = Camera { fov: 70.0, ..*cam };
            let hv = Scene { view_proj: (hand_cam.proj(aspect) * cam.view()).to_cols_array_2d(), ..*scene };
            let hb = self.hand_views[frame.slot];
            gpu.mapped(hb)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&hv));
            if hand_end > hand_start {
                list.pipeline(self.hand_pipe);
                list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
                list.bind(crate::gpu_layout::bind::entity::SCENE, hb);
                list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
                list.push(Cmd::Draw {
                    vertices: (hand_end - hand_start) as u32 * 36,
                    instances: 1,
                    first_vertex: hand_start as u32 * 36,
                    first_instance: 0,
                });
            }
            if let (Some((_, n)), Some(ib)) = (hand_item, item_buf) {
                list.pipeline(self.hand_item_pipe);
                list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
                list.bind(crate::gpu_layout::bind::entity::SCENE, hb);
                list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
                list.bind(crate::gpu_layout::bind::entity::ITEM_BOXES, ib);
                list.push(Cmd::Draw { vertices: n * 36, instances: 1, first_vertex: 0, first_instance: boxes.len() as u32 - 1 });
            }
        }
        self.instances = instances;
    }

    /// The models posed by this frame's `draw` (and the player's own body in first person) into
    /// the bound sun's shadow map, with `view` the shadow pass's scene.
    pub fn draw_shadow(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, frame: &FrameInfo, view: BufferId) {
        if let Some(pipe) = self.shadow_pipe {
            self.models
                .draw(gpu, list, frame, super::models::Batch::Shadow, pipe, view);
        }
    }
}

/// Two layers of flames facing the camera, the inner one smaller, like vanilla's burning mobs.
fn fire_boxes(out: &mut Vec<GpuBox>, textures: &GameTextures, base: Vec3, width: f32, height: f32, cam_yaw: f32, age: f32) {
    let c = content();
    let frame = |offset: f32| textures.frame_of(c.fire_texture, (age * crate::world::TPS as f32 + offset) as u32);
    for (i, (scale, back)) in [(1.4, 0.0), (1.0, 0.2)].into_iter().enumerate() {
        let l = frame(i as f32 * 11.0);
        let w = width * scale;
        let m = Mat4::from_translation(base)
            * Mat4::from_rotation_y(cam_yaw)
            * Mat4::from_translation(Vec3::new(-w / 2.0, 0.0, back * width))
            * Mat4::from_scale(Vec3::new(w, height * scale * 0.95, 0.0));
        out.push(GpuBox {
            rows: rows(m),
            color: [1.0, 1.0, 1.0, 0.0],
            ids: [0, box_mode::FLAMES, l | l << 16, l | l << 16],
            ids2: [l | l << 16, 0, 0, 0],
            ..Default::default()
        });
    }
}

/// Opaque pixels of a picture (a strip's first frame) and their neighbors.
fn sprite_of(size: u32, rgba: &[u8]) -> Sprite {
    let solid = |x: i64, y: i64| {
        (0..size as i64).contains(&x)
            && (0..size as i64).contains(&y)
            && rgba
                .get((y as usize * size as usize + x as usize) * 4 + 3)
                .is_some_and(|&a| a >= 128)
    };
    let pixels = (0..size * size)
        .map(|i| ((i % size) as i64, (i / size) as i64))
        .filter(|&(x, y)| solid(x, y))
        .map(|(x, y)| {
            (
                x as u8,
                y as u8,
                solid(x - 1, y) as u32 | (solid(x + 1, y) as u32) << 1 | (solid(x, y + 1) as u32) << 2 | (solid(x, y - 1) as u32) << 3,
            )
        })
        .collect();
    Sprite { size, pixels }
}

/// Item models by item and, for a stack with a look of its own, that look's texture id: one
/// item drawn from many looks has a model per look.
fn model_key(item: u32, look: Option<u32>) -> u64 {
    item as u64 | look.map_or(0, |t| t as u64 + 1) << 32
}

/// Texture a held item is drawn from as a solid sprite: its icon, or the texture of a plant block.
fn sprite_name(item: u32) -> Option<String> {
    let c = content();
    let looks = super::looks::current();
    let t = match looks.item_draw(c, item) {
        ItemDraw::Icon(t) => t,
        ItemDraw::Block(state) => looks.block_sprite(state, &c.states[state as usize])?,
        _ => return None,
    };
    looks.texture_name(c, t).map(str::to_string)
}

/// From view space to the camera-relative world for what the first person view holds: it bobs
/// with the world and lags behind a turning camera.
fn hand_space(cam: &Camera, h: &Hand, pose: Mat4) -> Mat4 {
    cam.view().inverse() * cam.bobbing() * Mat4::from_rotation_x(-h.lag.0 * 0.1) * Mat4::from_rotation_y(-h.lag.1 * 0.1) * pose
}

/// Where the held item's model goes in front of the camera, posed like vanilla's
/// ItemInHandRenderer: swing translation, arm offset, attack rotation, then the item model's
/// first person display transform. `None` for the bare arm.
fn hand_transform(cam: &Camera, h: &Hand, pose: Mat4) -> Option<Mat4> {
    use std::f32::consts::PI;
    let c = content();
    c.items
        .get(h.stack.item as usize)
        .filter(|_| !h.stack.is_empty())?;
    let p = h.swing.unwrap_or(0.0);
    let (s, sr) = ((p * PI).sin(), (p.sqrt() * PI).sin());
    let view = hand_space(cam, h, pose);
    let deg = |d: f32| d.to_radians();
    let swing_sin = (p * p * PI).sin();
    // Vanilla's applyEatTransform: up to the mouth, bobbing once it is there.
    let view = h.using.map_or(view, |(left, all)| {
        let done = left / all;
        let bob = if done < 0.8 { (left / 4.0 * PI).cos().abs() * 0.1 } else { 0.0 };
        let up = 1.0 - done.powi(27);
        view * Mat4::from_translation(Vec3::new(up * 0.6, bob - up * 0.5, 0.0))
            * Mat4::from_rotation_y(deg(up * 90.0))
            * Mat4::from_rotation_x(deg(up * 10.0))
            * Mat4::from_rotation_z(deg(up * 30.0))
    });
    let arm = view
        * Mat4::from_translation(Vec3::new(-0.4 * sr, 0.2 * (p.sqrt() * PI * 2.0).sin(), -0.2 * s))
        * Mat4::from_translation(Vec3::new(0.56, -0.52 - h.lowered * 0.6, -0.72))
        * Mat4::from_rotation_y(deg(45.0 - swing_sin * 20.0))
        * Mat4::from_rotation_z(deg(-sr * 20.0))
        * Mat4::from_rotation_x(deg(-sr * 80.0))
        * Mat4::from_rotation_y(deg(-45.0));
    Some(if shows_block(h.stack.item) {
        // block.json: rotated 45 degrees, scale 0.4.
        arm * Mat4::from_rotation_y(deg(45.0)) * Mat4::from_scale(Vec3::splat(0.4)) * Mat4::from_translation(Vec3::splat(-0.5))
    } else {
        // generated.json / handheld.json first person: rotation 0 -90 25, translation 1.13 3.2
        // 1.13 (pixels), scale 0.68.
        arm * Mat4::from_translation(Vec3::new(1.13, 3.2, 1.13) / 16.0)
            * Mat4::from_rotation_y(deg(-90.0))
            * Mat4::from_rotation_z(deg(25.0))
            * Mat4::from_scale(Vec3::splat(0.68))
            * Mat4::from_translation(Vec3::splat(-0.5))
    })
}

/// The bare right arm (the model's `rightArm` bone) in front of the camera, with the numbers of
/// vanilla's ItemInHandRenderer.renderPlayerArm.
fn hand_boxes(
    out: &mut Vec<GpuBox>,
    cam: &Camera,
    h: &Hand,
    skin: Option<(u32, [f32; 2])>,
    arm: Option<&crate::registry::Part>,
    pose: Mat4,
) {
    use std::f32::consts::{PI, TAU};
    let (Some((tex, size)), Some(arm)) = (skin, arm) else { return };
    let p = h.swing.unwrap_or(0.0);
    let (s, sr) = ((p * PI).sin(), (p.sqrt() * PI).sin());
    let deg = |d: f32| d.to_radians();
    let swing = Vec3::new(-0.3 * sr + 0.64, 0.4 * (p.sqrt() * TAU).sin() - 0.6 - h.lowered * 0.6, -0.4 * s - 0.72);
    let shoulder = hand_space(cam, h, pose)
        * Mat4::from_translation(swing)
        * Mat4::from_rotation_y(deg(45.0 + sr * 70.0))
        * Mat4::from_rotation_z(deg((p * p * PI).sin() * -20.0))
        * Mat4::from_translation(Vec3::new(-1.0, 3.6, 3.5))
        * Mat4::from_rotation_z(deg(120.0))
        * Mat4::from_rotation_x(deg(200.0))
        * Mat4::from_rotation_y(deg(-135.0))
        * Mat4::from_translation(Vec3::new(5.6, 0.0, 0.0))
        * Mat4::from_scale(Vec3::splat(1.0 / 16.0))
        // Vanilla's model space hangs down from y 24 and faces -z; ours stands up and faces +z.
        * Mat4::from_rotation_x(PI)
        * Mat4::from_translation(Vec3::from_array(arm.pivot) - Vec3::Y * 24.0);
    for c in &arm.cuboids {
        let m = shoulder * Mat4::from_translation(Vec3::from_array(c.origin)) * Mat4::from_scale(Vec3::from_array(c.size));
        out.push(GpuBox {
            rows: rows(m),
            dims: [c.size[0], c.size[1], c.size[2], 0.0],
            uvo: [c.uv[0], c.uv[1], size[0], size[1]],
            color: [1.0, 1.0, 1.0, 0.0],
            ids: [tex, box_mode::SKIN, 0, 0],
            ids2: [0; 4],
        });
    }
}

/// Model boxes (a block's, or an item's from its file) in the unit cube `m` maps, tinted like
/// `s` (an item's own boxes pass air: no tint).
fn boxes_shape(
    out: &mut Vec<GpuBox>,
    textures: &GameTextures,
    m: Mat4,
    boxes: &[crate::content::ModelBox],
    s: &crate::content::StateInfo,
    color: Vec4,
) {
    let tint = crate::content::item_tint(s.tint) & 0xff_ffff;
    for b in boxes {
        for d in 0..6 {
            let Some((corners, uv)) = super::mesher::box_face(b, d) else { continue };
            let (n, shade) = super::mesher::face_look(&corners);
            let [p0, p1, _, p3] = corners.map(Vec3::from_array);
            let overlay = textures
                .overlays
                .get(b.textures[d] as usize)
                .filter(|&&o| o != 0)
                .map(|o| o - 1);
            let mut face = |texture: u32, tinted: bool, lift: f32| {
                let place = Mat4::from_cols((p1 - p0).extend(0.0), (p3 - p0).extend(0.0), Vec4::ZERO, (p0 + n * lift).extend(1.0));
                out.push(GpuBox {
                    rows: rows(m * place),
                    dims: [shade, 0.0, 0.0, 0.0],
                    uvo: uv.map(|v| v / 16.0),
                    color: color.to_array(),
                    ids: [0, box_mode::QUAD, textures.texture(texture), 0],
                    ids2: [0, 0, tinted as u32, tint],
                });
            };
            // Faces the block tints take the tint here, not from a tinted copy drawn on top (the
            // untinted faces would show through cutout holes and at the edges).
            face(b.textures[d], s.tint != 0 && s.tint_faces & (1 << d) != 0 && overlay.is_none(), 0.0);
            // The tinted part of a face (a grass side), just outside it.
            if let Some(o) = overlay.filter(|_| s.tint != 0) {
                face(o, true, 0.002);
            }
        }
    }
}

fn rows(m: Mat4) -> [[f32; 4]; 3] {
    [m.row(0).to_array(), m.row(1).to_array(), m.row(2).to_array()]
}

/// An item model instance: `m` places the model's unit cube.
fn item_instance(m: Mat4, first: u32, hurt: f32, light: u32) -> GpuBox {
    GpuBox { rows: rows(m), color: [1.0, 1.0, 1.0, hurt], ids: [0, 0, first, 0], ids2: [0, light, 0, 0], ..Default::default() }
}

/// Whether the item is posed as a block (else as a sprite). A handheld item's model (a sword,
/// a pickaxe) lies in its square as its sprite would, so it is held like the sprite.
fn shows_block(item: u32) -> bool {
    let c = content();
    let looks = super::looks::current();
    match looks.item_draw(c, item) {
        ItemDraw::Model(_) => !c.items.get(item as usize).is_some_and(|d| d.handheld),
        ItemDraw::Block(state) => looks
            .block_sprite(state, &c.states[state as usize])
            .is_none(),
        _ => false,
    }
}

/// Where an item goes in the hand of a model, from its arm at the shoulder (pixels): vanilla's
/// ItemInHandLayer, then the item model's third person display transform. Vanilla's model space is
/// ours turned around z, and it measures from a shoulder one pixel closer to the body.
fn held_transform(arm: Mat4, item: u32) -> Mat4 {
    use std::f32::consts::{FRAC_PI_2, PI};
    let deg = |d: f32| d.to_radians();
    let hand = arm
        * Mat4::from_scale(Vec3::splat(16.0))
        * Mat4::from_rotation_z(PI)
        * Mat4::from_rotation_x(-FRAC_PI_2)
        * Mat4::from_rotation_y(PI)
        * Mat4::from_translation(Vec3::new(0.0, 0.125, -0.625));
    // block.json, handheld.json and generated.json: rotation, translation, scale.
    let ([rx, ry, rz], move_by, scale) = if shows_block(item) {
        ([75.0, 45.0, 0.0], [0.0, 2.5, 0.0], 0.375)
    } else if content()
        .items
        .get(item as usize)
        .is_some_and(|d| d.handheld)
    {
        ([0.0, -90.0, 55.0], [0.0, 4.0, 0.5], 0.85)
    } else {
        ([0.0, 0.0, 0.0], [0.0, 3.0, 1.0], 0.55)
    };
    hand * Mat4::from_translation(Vec3::from_array(move_by) / 16.0)
        * Mat4::from_rotation_x(deg(rx))
        * Mat4::from_rotation_y(deg(ry))
        * Mat4::from_rotation_z(deg(rz))
        * Mat4::from_scale(Vec3::splat(scale))
        * Mat4::from_translation(Vec3::splat(-0.5))
}

/// Dropped items spin and bob: a block a quarter of a block across, a sprite half a block
/// (vanilla's ground transforms).
fn ground_transform(base: Vec3, age: f32, item: u32) -> Mat4 {
    let bob = (age * 2.5).sin() * 0.06 + 0.15;
    let spin = Mat4::from_translation(base + Vec3::Y * bob) * Mat4::from_rotation_y(age * 1.5);
    if shows_block(item) {
        spin * Mat4::from_translation(Vec3::splat(-0.125)) * Mat4::from_scale(Vec3::splat(0.25))
    } else {
        spin * Mat4::from_translation(Vec3::new(-0.25, 0.0, -0.25)) * Mat4::from_scale(Vec3::splat(0.5))
    }
}

/// What a worn layer's pixels are read against: the wearer's layout width, at the layer's own
/// shape. A layer drawn at a multiple of the layout (a 128 x 64 armour or clothes layer on a
/// 64 x 32 skin: 32 texels a block on a 16 one) spreads the layout's pixels over all of it, as
/// Minecraft's armour layers do in high-resolution packs; one at the layout's width reads as before.
pub(crate) fn worn_tex_size(layout: [u32; 2], worn: [u32; 2]) -> [f32; 2] {
    if layout[0] == 0 || worn[0] == 0 {
        return [worn[0] as f32, worn[1] as f32];
    }
    let w = layout[0] as f32;
    [w, worn[1] as f32 * w / worn[0] as f32]
}

#[cfg(test)]
mod worn_tests {
    use super::worn_tex_size;

    #[test]
    fn worn_layers_map_by_the_skins_layout() {
        // As before: a 64 x 32 layer on a 64 x 32 skin, or on a 64 x 64 one (its top half).
        assert_eq!(worn_tex_size([64, 32], [64, 32]), [64.0, 32.0]);
        assert_eq!(worn_tex_size([64, 64], [64, 32]), [64.0, 32.0]);
        // Twice the layout (32 a block on a 16 skin) and four times: the same layout pixels.
        assert_eq!(worn_tex_size([64, 32], [128, 64]), [64.0, 32.0]);
        assert_eq!(worn_tex_size([64, 32], [256, 128]), [64.0, 32.0]);
        // No layout known: the layer's own size.
        assert_eq!(worn_tex_size([0, 0], [128, 64]), [128.0, 64.0]);
    }
}

//! Entities: their models (`models.rs`, posed per frame and drawn instanced per kind), dropped
//! items, the held item, flames, the block selection outline, break progress and gizmo boxes
//! (textured boxes built on the CPU each frame, a few hundred at most).

use super::looks::ItemDraw;
use super::models::{MAX_LAYERS, PoseLod, Posed};
use crate::gpu_layout::box_mode;
use glam::{DVec3, IVec3, Mat4, Quat, Vec3, Vec4};
use rustc_hash::FxHashMap;
use weft_abi::entry_kind as ek;

use super::models::{Act, Clips, Fade, Play};
use crate::block_model::Display;
use crate::gpu_layout::rows;
use crate::model::{self, Drive, role};
use weft_abi::ItemStack;
use weft_render::*;

use super::ClientEntity;
use super::models::{Instance, Models};
use super::textures::GameTextures;
use super::world_render::Camera;
use crate::assets::{Assets, decode_png};
use crate::content::content;
use crate::gpu_layout::{GpuBox, Scene};
use crate::physics::Hit;
use crate::physics::Transform;

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

/// The block outline's width on the screen (pixels) and how dark it is (its black's alpha).
const OUTLINE_PX: f32 = 1.25;
const OUTLINE_DARK: f32 = 0.4;

/// What the first person view holds: the item (or the bare arm) and how it moves.
/// Cracks to draw: how far along, on `state`'s shape at `pos`, of what is at `at`.
#[derive(Clone, Copy)]
pub struct Crack {
    pub pos: IVec3,
    pub progress: f32,
    pub place: Transform,
    pub part: crate::world::placement::Part,
}

pub struct Hand {
    /// How far it is lowered, 0..1: while changing items and just after a swing.
    pub lowered: f32,
    /// Radians the hand lags behind a turning camera: pitch, yaw.
    pub lag: (f32, f32),
    /// The bob to draw it with when the camera's own leaves walking out (`core:camera_bob` off).
    pub bob: Option<(Vec3, f32, f32)>,
    /// The body shows too, not only what the clips bring up.
    pub body: bool,
}

/// A player's avatar: its model and what it is painted with.
struct AvatarLook {
    kind: u32,
    tex: TextureId,
    index: u32,
    size: [u32; 2],
    glow: Option<(TextureId, u32)>,
}

impl AvatarLook {
    fn destroy(self, gpu: &mut dyn Backend) {
        gpu.destroy_texture(self.tex);
        if let Some((g, _)) = self.glow {
            gpu.destroy_texture(g);
        }
    }
}

/// Biggest avatar texture, each way.
const AVATAR_TEXTURE: u32 = 1024;

/// A sheet of pictures of items' models, `ICON_CELLS` a side, each `ICON_PIXELS` big.
struct IconSheet {
    color: TextureId,
    depth: TextureId,
    index: u32,
    used: u32,
}

const ICON_PIXELS: u32 = 64;
const ICON_CELLS: u32 = 16;

/// What something on an entity is.
#[derive(Clone, Copy)]
enum Carrying {
    /// In a hand: a model, or a sprite or block when it is none.
    Held {
        stack: ItemStack,
        model: Option<u32>,
        off: bool,
    },
    /// A model on one of its bones.
    On {
        model: u32,
        bone: usize,
    },
    Embedded {
        stack: ItemStack,
    },
}

/// Something on an entity this frame: its place in `posed`, what, its light, its age and who.
#[derive(Clone, Copy)]
struct Carry {
    holder: usize,
    what: Carrying,
    lit: u32,
    /// The biome its holder is in.
    biome: Option<u16>,
    age: f32,
    id: u64,
}

/// Opaque pixels of a sprite, for drawing it as a 1/16 thick solid like vanilla's held items.
struct Sprite {
    size: u32,
    /// Opaque pixels and which of their sides touch another one (left, right, below, above).
    pixels: Vec<(u8, u8, u32)>,
}

#[derive(Clone, Copy)]
struct ItemModel {
    boxes: (u32, u32),
    solid: bool,
}

struct PlayerSkin {
    texture: TextureId,
    index: u32,
    size: [u32; 2],
    slim: bool,
}

pub struct EntityRenderer {
    /// What the head's move in an action does to the camera: offset in view space, roll, pitch.
    pub camera_kick: (Vec3, f32, f32),
    /// An entity kind a UI preview shows alone (`Ui::kind_model`): its skin is loaded although
    /// none of it is in the world (the Field Journal's picture was empty).
    pub preview_kind: Option<u32>,
    pipeline: PipelineId,
    overlay: PipelineId,
    /// Model pipelines for the screen and for the inventory figure's image.
    model_pipe: PipelineId,
    /// The first person's arms: fixed field of view, depth squeezed so they never sink into walls.
    model_hand_pipe: PipelineId,
    preview_pipe: PipelineId,
    /// Models into the sun's shadow map, when the shader pack draws one and its model shader has
    /// `fs_shadow`.
    shadow_pipe: Option<PipelineId>,
    box_preview_pipe: PipelineId,
    translucent_preview_pipe: PipelineId,
    box_preview_buffers: Vec<(BufferId, u32)>,
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
    /// Players' own skins by entity, replacing the player kind's texture on them, and if their arms are slim.
    player_skins: FxHashMap<u64, PlayerSkin>,
    /// Skins received and not uploaded yet.
    pub skin_pending: Vec<(u64, crate::net::Skin)>,
    /// Skins mods painted (`Host::reg_entity_skin`), by entity.
    painted: FxHashMap<u64, Painted>,
    /// Builds their painters set, by entity (this frame's parts' scales go to `Models::builds`).
    builds: FxHashMap<u64, Build>,
    /// Each kind with a painter: its component, the mod and its callback (made on first use).
    painters: Option<FxHashMap<u32, Painter>>,
    /// Textures painted over, as read: RGBA and size, by name.
    skin_bases: FxHashMap<String, SkinBase>,
    /// Entities whose skin is being painted, what finishes (`paint_skins`), and which reading of
    /// the textures they were painted over (`reload` counts up; older ones are dropped).
    painting: rustc_hash::FxHashSet<u64>,
    paint_done: (std::sync::mpsc::Sender<PaintDone>, std::sync::mpsc::Receiver<PaintDone>),
    paint_generation: u32,
    /// Players' avatars as they sent them, as uploaded, and received but not looked at yet.
    avatar_sent: FxHashMap<u64, std::sync::Arc<crate::net::AvatarData>>,
    avatars: FxHashMap<u64, AvatarLook>,
    pub avatar_pending: Vec<(u64, std::sync::Arc<crate::net::AvatarData>)>,
    /// Players who left, their skins and avatars to go.
    pub gone_pending: Vec<u64>,
    /// The setting: other than the kind's own model, a player's avatar shows.
    show_avatars: bool,
    /// What plays on who coming in and going out, by entity and model: clips are the model's.
    fades: FxHashMap<(u64, u32), Vec<Fade>>,
    /// Actions asked for with no clip on a model, once said so.
    warned_actions: rustc_hash::FxHashSet<(u32, String)>,
    /// When the last frame was, and from where.
    last_draw: std::time::Instant,
    last_cam: DVec3,
    icon_views: Vec<BufferId>,
    icon_sheets: Vec<IconSheet>,
    /// Frames drawn, for pose LOD; assets are prepared on changes.
    frame_count: u32,
    preloaded: bool,
    pub changed_items: rustc_hash::FxHashSet<u64>,
    /// Stacks whose picture comes next, and the model it is of.
    icons_todo: Vec<(ItemStack, u32)>,
    /// Pictures by model: however many stacks show as one have the one.
    icon_of_model: FxHashMap<u32, (u32, [f32; 4])>,
    /// The local player's entity.
    pub local: u64,
    glows: FxHashMap<String, (TextureId, u32)>,
    /// Shader slot of each entity kind.
    kind_shaders: Vec<u32>,
    /// Shader slot of each entity effect, by effect id.
    effect_slots: Vec<u32>,
    sprites: FxHashMap<String, Option<Sprite>>,
    /// Entities to pose this frame, kept between frames.
    posed: Vec<Posed>,
    /// Item models drawn instanced: pipelines for the world and the hand, each item's boxes
    /// (first, count) in `item_boxes`, the GPU copy (buffer, capacity, boxes uploaded) and the
    /// texture generation they were made with.
    item_pipe: PipelineId,
    hand_item_pipe: PipelineId,
    /// Boxes of each item's model, (first, count), by item and its stack's `layers` if any.
    item_models: FxHashMap<(u32, Option<&'static str>, Option<u16>), ItemModel>,
    item_boxes: Vec<GpuBox>,
    item_buf: Option<(BufferId, u32, usize)>,
    item_generation: u64,
    /// This frame's item instances: (item, instance).
    instances: Vec<(u64, GpuBox)>,
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

/// Entity effects (`World::entity_effect`) over whatever shaded the pixel.
pub const EFFECT_HOOK: crate::shaders::Hook = crate::shaders::Hook {
    kind: "entity_effect",
    entry: "effect",
    dispatch: "entity_effect",
    params: "p: EntityPixel, c: vec4<f32>, strength: f32",
    args: "p, c, strength",
    ret: "vec4<f32>",
    fallback: "return c;",
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
        let (pipeline, overlay, box_preview_pipe, translucent_preview_pipe) = Self::pipelines(gpu, assets)?;
        let ([model_pipe, preview_pipe, model_hand_pipe], shadow_pipe, kind_shaders, effect_slots) = Self::model_pipelines(gpu, assets)?;
        let models = Models::new(gpu, assets);
        let frames = gpu.frames_in_flight();
        let hand_views = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Scene>() as u64, usage::STORAGE | usage::HOST, "hand scene"))
            .collect();
        let preview = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Scene>() as u64, usage::STORAGE | usage::HOST, "preview scene"))
            .collect();
        let (item_pipe, hand_item_pipe) = Self::item_pipelines(gpu, assets)?;
        let icon_views = (0..frames)
            .map(|_| gpu.create_buffer(size_of::<Scene>() as u64, usage::STORAGE | usage::HOST, "icon scene"))
            .collect();
        Ok(EntityRenderer {
            avatar_sent: FxHashMap::default(),
            avatars: FxHashMap::default(),
            avatar_pending: Vec::new(),
            gone_pending: Vec::new(),
            frame_count: 0,
            preloaded: false,
            changed_items: Default::default(),
            icon_of_model: FxHashMap::default(),
            show_avatars: true,
            fades: FxHashMap::default(),
            warned_actions: Default::default(),
            last_draw: std::time::Instant::now(),
            last_cam: DVec3::ZERO,
            icon_views,
            icon_sheets: Vec::new(),
            icons_todo: Vec::new(),
            item_pipe,
            hand_item_pipe,
            item_models: FxHashMap::default(),
            item_boxes: Vec::new(),
            item_buf: None,
            item_generation: 0,
            instances: Vec::new(),
            pipeline,
            overlay,
            camera_kick: (Vec3::ZERO, 0.0, 0.0),
            model_pipe,
            model_hand_pipe,
            shadow_pipe,
            models,
            buffers: Vec::new(),
            hand_views,
            preview,
            preview_pipe,
            box_preview_pipe,
            translucent_preview_pipe,
            box_preview_buffers: Vec::new(),
            preview_target: None,
            preview_kind: None,
            skins: FxHashMap::default(),
            player_skins: FxHashMap::default(),
            skin_pending: Vec::new(),
            painted: FxHashMap::default(),
            builds: FxHashMap::default(),
            painters: None,
            skin_bases: FxHashMap::default(),
            painting: Default::default(),
            paint_done: std::sync::mpsc::channel(),
            paint_generation: 0,
            local: 0,
            glows: FxHashMap::default(),
            kind_shaders,
            effect_slots,
            sprites: FxHashMap::default(),
            posed: Vec::new(),
        })
    }

    /// Item models instanced in the world and in the hand.
    fn item_pipelines(gpu: &mut dyn Backend, assets: &Assets) -> Result<(PipelineId, PipelineId), String> {
        let s = crate::packs::shader(assets, "entity.wgsl").map_err(|e| e.0)?;
        let mut pipe = |label, vertex| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader: &s,
                vertex,
                fragment: Some("fs"),
                color: Some(crate::post::world_format()),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            })
            .map_err(|e| e.0)
        };
        Ok((pipe("items", "vs_item")?, pipe("hand item", "vs_hand_item")?))
    }

    fn pipelines(gpu: &mut dyn Backend, assets: &Assets) -> Result<(PipelineId, PipelineId, PipelineId, PipelineId), String> {
        let s = crate::packs::shader(assets, "entity.wgsl").map_err(|e| e.0)?;
        let main = gpu
            .create_pipeline(&PipelineDesc {
                label: "entities",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs"),
                color: Some(crate::post::world_format()),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            })
            .map_err(|e| e.0)?;
        let overlay = gpu
            .create_pipeline(&PipelineDesc {
                label: "overlay",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs_overlay"),
                color: Some(crate::post::world_format()),
                depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                blend: Blend::Alpha,
                cull_back: true,
                topology: Topology::Triangles,
                motion: crate::post::motion(),
            })
            .map_err(|e| e.0)?;
        let preview = gpu
            .create_pipeline(&PipelineDesc {
                label: "block preview",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs"),
                color: Some(Format::Rgba8Srgb),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
                motion: false,
            })
            .map_err(|e| e.0)?;
        let translucent = gpu
            .create_pipeline(&PipelineDesc {
                label: "translucent block preview",
                shader: &s,
                vertex: "vs",
                fragment: Some("fs_overlay"),
                color: Some(Format::Rgba8Srgb),
                depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                blend: Blend::Alpha,
                cull_back: true,
                topology: Topology::Triangles,
                motion: false,
            })
            .map_err(|e| e.0)?;
        Ok((main, overlay, preview, translucent))
    }

    /// The model pipelines (screen, inventory figure image, first person hand) with every entity
    /// shader snippet, the sun's shadow map's (with the pack's sun shadows), each kind's slot and
    /// each effect's.
    #[allow(clippy::type_complexity)]
    fn model_pipelines(
        gpu: &mut dyn Backend,
        assets: &Assets,
    ) -> Result<([PipelineId; 3], Option<PipelineId>, Vec<u32>, Vec<u32>), String> {
        let snippets = crate::shaders::snippets(assets, &HOOK);
        let effects = crate::shaders::snippets(assets, &EFFECT_HOOK);
        let (s, mut slots) =
            crate::packs::compose(assets, "model.wgsl", &[(&HOOK, &snippets), (&EFFECT_HOOK, &effects)]).map_err(|e| e.0)?;
        let effect_slots = slots.pop().unwrap_or_default();
        let slots = slots.pop().unwrap_or_default();
        let c = content();
        let effect_slots = c
            .game
            .entity_effects
            .iter()
            .map(|d| {
                let slot = effect_slots.slot(&d.shader);
                if slot == 0 && d.shader_given && !d.shader.is_empty() {
                    log::warn!("entity effect {}: no shader entity_effect/{}", d.name, d.shader);
                }
                slot
            })
            .collect();
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
        let mut make = |label: &str, vertex: &str, color: Format| {
            gpu.create_pipeline(&PipelineDesc {
                label,
                shader: &s,
                vertex,
                fragment: Some("fs"),
                color: Some(color),
                depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                blend: Blend::None,
                cull_back: true,
                topology: Topology::Triangles,
                motion: color == crate::post::world_format() && crate::post::motion(),
            })
            .map_err(|e| e.0)
        };
        let (world, preview) = (make("models", "vs", crate::post::world_format())?, make("entity preview", "vs", Format::Rgba8Srgb)?);
        // A shader pack's older copy of the file has no `vs_hand`.
        let hand = make("model hand", "vs_hand", crate::post::world_format())
            .or_else(|_| make("model hand", "vs", crate::post::world_format()))?;
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
                    motion: false,
                })
                .map_err(|e| log::warn!("models cast no sun shadows: {}", e.0))
                .ok()
            })
            .flatten();
        Ok(([world, preview, hand], shadow, kinds, effect_slots))
    }

    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        match Self::model_pipelines(gpu, assets) {
            Ok(([p, q, h], shadow, kinds, effects)) => {
                self.effect_slots = effects;
                gpu.destroy_pipeline(self.model_pipe);
                gpu.destroy_pipeline(self.model_hand_pipe);
                gpu.destroy_pipeline(self.preview_pipe);
                if let Some(s) = self.shadow_pipe.take() {
                    gpu.destroy_pipeline(s);
                }
                (self.model_pipe, self.preview_pipe, self.model_hand_pipe, self.shadow_pipe) = (p, q, h, shadow);
                self.kind_shaders = kinds;
            }
            Err(e) => log::error!("model shader: {e}"),
        }
        self.models.reload(gpu, assets);
        self.preloaded = false;
        // Avatars are models, gone with the others: they come back from what was sent.
        for (id, a) in self.avatars.drain() {
            a.destroy(gpu);
            if let Some(sent) = self.avatar_sent.get(&id) {
                self.avatar_pending.push((id, sent.clone()));
            }
        }
        // Painted skins are painted again over the textures as read now; builds are resolved
        // again against the new models.
        self.skin_bases.clear();
        self.painting.clear();
        self.paint_generation = self.paint_generation.wrapping_add(1);
        for (_, p) in self.painted.drain() {
            gpu.destroy_texture(p.tex.0);
        }
        for b in self.builds.values_mut() {
            b.resolved = None;
        }
        // Clips and models have new numbers.
        self.fades.clear();
        self.icon_of_model.clear();
        self.icons_todo.clear();
        // Skins, masks and icons are made again the next time they are used.
        self.free_loaded(gpu);
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
            Ok((a, b, preview, translucent)) => {
                gpu.destroy_pipeline(self.box_preview_pipe);
                gpu.destroy_pipeline(self.translucent_preview_pipe);
                self.box_preview_pipe = preview;
                self.translucent_preview_pipe = translucent;
                gpu.destroy_pipeline(self.pipeline);
                gpu.destroy_pipeline(self.overlay);
                self.pipeline = a;
                self.overlay = b;
            }
            Err(e) => log::error!("entity shader: {e}"),
        }
    }

    pub fn set_show_avatars(&mut self, show: bool) {
        if self.show_avatars != show {
            self.show_avatars = show;
            self.changed_items.extend(self.avatar_sent.keys().copied());
        }
    }

    /// Uploads the player's skin with the model chosen in their profile.
    pub fn set_skin(&mut self, gpu: &mut dyn Backend, entity: u64, skin: &crate::net::Skin) {
        let Some(img) = crate::assets::decode_png(&skin.png) else { return };
        let t = gpu.create_texture(
            &TextureDesc { width: img.width, height: img.height, mips: 1, format: Format::Rgba8Srgb, render_target: false },
            "player skin",
        );
        gpu.write_texture(t, 0, &img.rgba);
        let skin = PlayerSkin { texture: t, index: gpu.texture_index(t), size: [img.width, img.height], slim: skin.slim };
        if let Some(old) = self.player_skins.insert(entity, skin) {
            gpu.destroy_texture(old.texture);
        }
        self.changed_items.insert(entity);
    }

    /// Takes in player `entity`'s avatar: a model made of what it sent, painted with its picture.
    fn set_avatar(&mut self, gpu: &mut dyn Backend, entity: u64, sent: std::sync::Arc<crate::net::AvatarData>) {
        let fits = |png: &[u8]| crate::assets::png_size(png).is_some_and(|(w, h)| w <= AVATAR_TEXTURE && h <= AVATAR_TEXTURE);
        let img = Some(&sent.png)
            .filter(|p| fits(p))
            .and_then(|p| crate::assets::decode_png(p));
        let Some(img) = img else {
            log::warn!("avatar of {entity}: no usable picture");
            return;
        };
        let kind = match self.models.add_avatar(&sent) {
            Ok(k) => k,
            Err(e) => {
                log::warn!("avatar of {entity}: {e}");
                return;
            }
        };
        let tex = gpu.create_texture(
            &TextureDesc { width: img.width, height: img.height, mips: 1, format: Format::Rgba8Srgb, render_target: false },
            "avatar",
        );
        gpu.write_texture(tex, 0, &img.rgba);
        let glow = Some(&sent.glow)
            .filter(|p| fits(p))
            .and_then(|p| crate::assets::decode_png(p))
            .filter(|g| (g.width, g.height) == (img.width, img.height))
            .map(|g| {
                let mask = crate::assets::glow_mask(&g.rgba);
                let t = gpu.create_texture(
                    &TextureDesc { width: g.width, height: g.height, mips: 1, format: Format::R8Unorm, render_target: false },
                    "avatar glow",
                );
                gpu.write_texture(t, 0, &mask);
                (t, gpu.texture_index(t))
            });
        let look = AvatarLook { kind, tex, index: gpu.texture_index(tex), size: [img.width, img.height], glow };
        if let Some(old) = self.avatars.insert(entity, look) {
            old.destroy(gpu);
        }
        self.avatar_sent.insert(entity, sent);
        self.changed_items.insert(entity);
    }

    /// Skins, glow masks and icon sheets: loaded when first used, freed on a reload and at the end.
    fn free_loaded(&mut self, gpu: &mut dyn Backend) {
        for t in self
            .skins
            .drain()
            .filter_map(|(_, v)| v)
            .map(|v| v.0)
            .chain(self.glows.drain().map(|(_, g)| g.0))
        {
            gpu.destroy_texture(t);
        }
        for s in self.icon_sheets.drain(..) {
            gpu.destroy_texture(s.color);
            gpu.destroy_texture(s.depth);
        }
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        for skin in self.player_skins.drain().map(|(_, v)| v) {
            gpu.destroy_texture(skin.texture);
        }
        for p in self.painted.drain().map(|(_, p)| p) {
            gpu.destroy_texture(p.tex.0);
        }
        for a in self.avatars.drain().map(|(_, v)| v) {
            a.destroy(gpu);
        }
        for b in self.icon_views.drain(..) {
            gpu.destroy_buffer(b);
        }
        for (b, _) in self.buffers.drain(..) {
            gpu.destroy_buffer(b);
        }
        self.models.destroy(gpu);
        gpu.destroy_pipeline(self.model_pipe);
        gpu.destroy_pipeline(self.model_hand_pipe);
        gpu.destroy_pipeline(self.preview_pipe);
        if let Some(s) = self.shadow_pipe.take() {
            gpu.destroy_pipeline(s);
        }
        gpu.destroy_pipeline(self.box_preview_pipe);
        gpu.destroy_pipeline(self.translucent_preview_pipe);
        for (buffer, _) in self.box_preview_buffers.drain(..) {
            gpu.destroy_buffer(buffer);
        }
        if let Some((c, d, ..)) = self.preview_target.take() {
            gpu.destroy_texture(c);
            gpu.destroy_texture(d);
        }
        let preview: Vec<BufferId> = self.preview.drain(..).collect();
        for b in self.hand_views.drain(..).chain(preview) {
            gpu.destroy_buffer(b);
        }
        self.free_loaded(gpu);
        gpu.destroy_pipeline(self.pipeline);
        gpu.destroy_pipeline(self.overlay);
        gpu.destroy_pipeline(self.item_pipe);
        gpu.destroy_pipeline(self.hand_item_pipe);
        if let Some((b, ..)) = self.item_buf.take() {
            gpu.destroy_buffer(b);
        }
    }

    /// Prepares changed displayed stacks before recording draws, including models and textures
    /// supplied by stack data. The local player's presentation follows prediction each frame.
    /// `comps`: the entities' components, for the skins mods paint (chiacraft).
    pub fn preload(
        &mut self,
        gpu: &mut dyn Backend,
        assets: &Assets,
        entities: &FxHashMap<u64, ClientEntity>,
        comps: &FxHashMap<(u64, u32), Vec<u8>>,
        me: &ClientEntity,
    ) {
        let c = content();
        self.frame_count = self.frame_count.wrapping_add(1);
        let s = crate::diag::span("preload/skins");
        self.paint_skins(gpu, assets, entities, comps);
        drop(s);
        for (id, sent) in std::mem::take(&mut self.avatar_pending) {
            self.set_avatar(gpu, id, sent);
        }
        for id in std::mem::take(&mut self.gone_pending) {
            self.avatar_sent.remove(&id);
            if let Some(a) = self.avatars.remove(&id) {
                a.destroy(gpu);
            }
            if let Some(skin) = self.player_skins.remove(&id) {
                gpu.destroy_texture(skin.texture);
            }
        }
        let mut changed = std::mem::take(&mut self.changed_items);
        if !self.preloaded {
            changed.extend(entities.keys());
            self.preloaded = true;
        }
        for (id, e) in changed
            .iter()
            .filter_map(|id| entities.get(id).map(|e| (*id, e)))
            .chain(std::iter::once((self.local, me)))
        {
            self.load(gpu, assets, &c.game.entities[e.look() as usize].texture);
            for stack in e.displayed() {
                self.load_sprite(assets, &stack);
                self.models.model_for(assets, stack);
                for embedded in crate::stack::embedded(&stack) {
                    if let Some(kind) = c.game.catalog.find(ek::ENTITY, &embedded.kind) {
                        self.load(gpu, assets, &c.game.entities[kind as usize].texture);
                    }
                }
            }
            let drawn = self.model_of(id, e.look());
            for &stack in &e.worn {
                if let Some((_, texture)) = c.worn(&stack) {
                    self.load(gpu, assets, texture);
                }
                self.models.prepare_worn(assets, drawn, stack);
            }
        }
        changed.clear();
        self.changed_items = changed;
        let wanted = std::mem::take(&mut super::models::ICONS.lock().unwrap().wanted);
        for stack in wanted {
            match self.models.model_for(assets, stack) {
                Some(k) => match self.icon_of_model.get(&k) {
                    Some(&icon) => super::models::icon_done(stack, Some(icon)),
                    None => self.icons_todo.push((stack, k)),
                },
                None => super::models::icon_done(stack, None),
            }
        }
        for skin in std::mem::take(&mut self.models.new_skins) {
            self.load(gpu, assets, &skin);
        }
        self.models.sync(gpu);
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
                    let kind = *c
                        .game
                        .catalog
                        .ids(weft_abi::entry_kind::ENTITY)
                        .get(&p.kind)?;
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

    /// Entity `id`'s build (its painter's bone scales) for the parts of `kind`, into
    /// `Models::builds`: where (`Posed::build`), and how far it lifts the model (model pixels).
    fn build_of(&mut self, id: u64, kind: u32) -> ((u32, u32), f32) {
        let from = self.models.builds.len() as u32;
        let (Some(b), Some(m)) = (self.builds.get_mut(&id), self.models.kinds.get(kind as usize)) else {
            return ((from, from), 0.0);
        };
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
        let Some((_, scales, lift)) = &b.resolved else { return ((from, from), 0.0) };
        self.models.builds.extend_from_slice(scales);
        ((from, self.models.builds.len() as u32), *lift)
    }

    /// The instance of a model that isn't an entity kind's, painted with its own texture.
    fn model_layer(&self, kind: u32, light: u32, effect: (u32, f32)) -> Option<Instance> {
        let m = self.models.kinds.get(kind as usize)?;
        let skin = m.skin.as_ref()?;
        let (_, tex, size) = self.skins.get(skin)?.as_ref()?;
        let size = m.texture.unwrap_or(*size);
        Some(Instance {
            color: [1.0; 3],
            light,
            effect: effect.0,
            strength: effect.1,
            tex: *tex,
            tex_size: [size[0] as f32, size[1] as f32],
            roles: u32::MAX,
            glow: self.glows.get(skin).map_or(u32::MAX, |g| g.1),
            ..Default::default()
        })
    }

    /// The opaque pixels of a stack's sprites, read the first time it is held or dropped.
    fn load_sprite(&mut self, assets: &Assets, stack: &ItemStack) {
        let looks = super::looks::current();
        let c = content();
        let draw = looks.item_draw(c, stack, None, "west");
        for name in draw.sprites().iter().filter_map(|s| s.outline(&looks, c)) {
            self.load_sprite_named(assets, name);
        }
    }

    fn load_sprite_named(&mut self, assets: &Assets, name: String) {
        if self.sprites.contains_key(&name) {
            return;
        }
        // A stack's own pictures one over the other, as its icon shows them.
        let img = match name.strip_prefix("layers:") {
            // Chiacraft's look: its layers composed as its icon is.
            Some(spec) if crate::stack::is_look(spec) => super::stack_look::compose(spec, |n| crate::assets::read_image(assets, n))
                .map(|(size, rgba)| crate::assets::Image { width: size, height: size, rgba }),
            Some(spec) => {
                let parts: Vec<(String, u32)> = crate::stack::parse_layers(spec)
                    .into_iter()
                    .map(|(n, t)| (n.to_string(), t))
                    .collect();
                let t = super::textures::layered(assets, &parts);
                t.frames
                    .first()
                    .and_then(|f| f.first())
                    .map(|rgba| crate::assets::Image { width: t.size, height: t.size, rgba: rgba.clone() })
            }
            None => assets
                .read(&assets.picture(&name), "textures", "png")
                .and_then(|b| decode_png(&b)),
        };
        let sprite = img.map(|img| {
            let size = img.width;
            // Animated strips: the first frame.
            let solid = |x: i64, y: i64| {
                (0..size as i64).contains(&x)
                    && (0..size as i64).contains(&y)
                    && img
                        .rgba
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
                        solid(x - 1, y) as u32
                            | (solid(x + 1, y) as u32) << 1
                            | (solid(x, y + 1) as u32) << 2
                            | (solid(x, y - 1) as u32) << 3,
                    )
                })
                .collect();
            Sprite { size, pixels }
        });
        self.sprites.insert(name, sprite);
    }

    /// An item's model as boxes in the unit cube, made once: a block item's shape, else its
    /// sprite one texel thick (flat when the sprite could not be read). (first box, count). A
    /// block's layers tinted by a kind take its colors in `biome`, where the item is.
    fn item_model(&mut self, textures: &GameTextures, stack: &ItemStack, biome: Option<u16>) -> Option<ItemModel> {
        let c = content();
        let looks = super::looks::current();
        let (item, layers) = (stack.item, crate::stack::layers(stack));
        if let Some(&m) = self.item_models.get(&(item, layers, biome)) {
            return Some(m);
        }
        let draw = looks.item_draw(c, stack, biome, "west");
        let first = self.item_boxes.len();
        let white = Vec4::new(1.0, 1.0, 1.0, 0.0);
        if let ItemDraw::Model { boxes, state } = &draw {
            boxes_shape(&mut self.item_boxes, textures, Mat4::IDENTITY, boxes, &c.states[*state as usize], biome, white);
        }
        if matches!(draw, ItemDraw::Nothing) {
            return None;
        }
        for (k, sprite) in draw.sprites().iter().enumerate() {
            let t = sprite.texture(textures);
            let [r, g, b, _] = (sprite.color << 8).to_be_bytes().map(|v| v as f32 / 255.0);
            let color = Vec4::new(r, g, b, 0.0);
            let l = textures.texture(t);
            // A layer over another is a hair thicker, so it covers it on both sides.
            let more = k as f32 * 0.002;
            match sprite
                .outline(&looks, c)
                .and_then(|name| self.sprites.get(&name))
                .and_then(|s| s.as_ref())
            {
                Some(sp) => {
                    let n = sp.size as f32;
                    // Only the outline: a side face against a neighboring pixel would fight
                    // with it. Mode 4: every face takes the color of this one texel.
                    for &(x, y, hidden) in &sp.pixels {
                        let m = Mat4::from_translation(Vec3::new(x as f32 / n, 1.0 - (y as f32 + 1.0) / n, 7.5 / 16.0 - more))
                            * Mat4::from_scale(Vec3::new(1.0 / n, 1.0 / n, 1.0 / 16.0 + 2.0 * more));
                        self.item_boxes.push(GpuBox {
                            rows: rows(m),
                            uvo: [(x as f32 + 0.5) / n, (y as f32 + 0.5) / n, 0.0, 0.0],
                            color: color.to_array(),
                            ids: [0, box_mode::TEXEL, l, hidden],
                            ..Default::default()
                        });
                    }
                }
                None => {
                    let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.5 + more)) * Mat4::from_scale(Vec3::new(1.0, 1.0, 0.0));
                    self.item_boxes.push(GpuBox {
                        rows: rows(m),
                        color: color.to_array(),
                        ids: [0, box_mode::FACES, l | l << 16, l | l << 16],
                        ids2: [l | l << 16, 0, 0, 0],
                        ..Default::default()
                    });
                }
            }
        }
        let m = ItemModel { boxes: (first as u32, (self.item_boxes.len() - first) as u32), solid: draw.solid() };
        self.item_models.insert((item, layers, biome), m);
        Some(m)
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
        // A worn look (`abi::comp::WORN_LOOK`) is composed from its layers, the rest read.
        let image = if crate::stack::is_look(name) {
            super::stack_look::compose_rect(name, |n| crate::assets::read_image(assets, n))
                .map(|(width, height, rgba)| crate::assets::Image { width, height, rgba })
        } else {
            assets
                .read(name, "textures", "png")
                .and_then(|b| decode_png(&b))
        };
        let loaded = image.map(|img| {
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
        // Two channels (`emission_mask_rg`): r the glow, g a pack's own (the blood moon's eyes).
        if let Some(mask) = crate::assets::emission_mask_rg(assets, name) {
            let t = gpu.create_texture(
                &TextureDesc { width: mask.width, height: mask.height, mips: 1, format: Format::Rgba8Unorm, render_target: false },
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
    fn layers(&self, kind: u32, entity: u64, worn: &[ItemStack], light: u32, effect: (u32, f32)) -> ([Instance; MAX_LAYERS], usize) {
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
            .map(|o| (o.index, o.size))
            .or_else(|| {
                self.painted
                    .get(&entity)
                    .filter(|p| p.texture == k.texture)
                    .map(|p| (p.tex.1, p.tex.2))
            });
        // Worn textures are laid out on the model's own texture size (`worn_tex_size`).
        let layout = size;
        let (skin, size) = own.map_or((*skin, size), |o| o);
        let skin = &skin;
        let base = Instance {
            color: [1.0; 3],
            light,
            effect: effect.0,
            strength: effect.1,
            tex: *skin,
            tex_size: [size[0] as f32, size[1] as f32],
            roles: u32::MAX,
            shader: self.kind_shaders.get(kind as usize).copied().unwrap_or(0),
            glow: glow(&k.texture),
            ..Default::default()
        };
        // An avatar is painted its own way; what it wears goes on its own worn geometry.
        let avatar = self.avatar_of(entity, kind);
        out.0[0] = match avatar {
            Some(a) => {
                let size = self
                    .models
                    .kinds
                    .get(a.kind as usize)
                    .and_then(|m| m.texture)
                    .unwrap_or(a.size);
                Instance { tex: a.index, tex_size: size.map(|v| v as f32), glow: a.glow.map_or(u32::MAX, |g| g.1), ..base }
            }
            None => base,
        };
        out.1 = 1;
        let model = self.model_of(entity, kind);
        for stack in worn {
            let i = stack.item;
            if let Some((w, t)) = c.worn(stack) {
                if let Some(Some((_, idx, size))) = self.skins.get(t).filter(|_| out.1 < MAX_LAYERS) {
                    let names = c.worn_bones(i);
                    let bone_mask = self.models.kinds.get(model as usize).map_or(0, |m| {
                        m.parts
                            .iter()
                            .take(32)
                            .enumerate()
                            .filter(|(_, p)| names.iter().any(|n| n.eq_ignore_ascii_case(&p.name)))
                            .fold(0u32, |mask, (b, _)| mask | 1 << b)
                    });
                    out.0[out.1] = Instance {
                        tex: *idx,
                        tex_size: worn_tex_size(layout, *size),
                        roles: c.worn_roles(i),
                        bone_mask,
                        inflate: w.inflate,
                        glow: glow(t),
                        worn: 1,
                        ..base
                    };
                    out.1 += 1;
                }
            }
        }
        out
    }

    /// The bones (the first 32) of kind `kind`'s model whose role is in `roles` (a mask of
    /// `1 << role`): what a layer's `bone_mask` keeps when its `roles` are cut to these.
    fn bones_with(&self, kind: u32, roles: u32) -> u32 {
        self.models.kinds.get(kind as usize).map_or(0, |m| {
            m.parts
                .iter()
                .take(32)
                .enumerate()
                .filter(|(_, p)| p.role < 32 && roles >> p.role & 1 != 0)
                .fold(0u32, |mask, (b, _)| mask | 1 << b)
        })
    }

    /// The action entity `id` is playing now, and how long a clip of `name` is on it, if it has one.
    pub fn action_of<'a>(&self, id: u64, e: &'a ClientEntity) -> Option<&'a str> {
        let (name, start, looping) = e.action.as_ref()?;
        let m = self.models.kinds.get(self.model_kind(id, e) as usize)?;
        let last = *m.action(e.held.item, name).last()?;
        let done = m
            .anims
            .get(last as usize)?
            .finished(e.age - start, *looping);
        (!done).then_some(name.as_str())
    }

    pub fn clip_length(&self, id: u64, e: &ClientEntity, name: &str) -> Option<f32> {
        let m = self.models.kinds.get(self.model_kind(id, e) as usize)?;
        let last = *m.action(e.held.item, name).last()?;
        Some(m.anims.get(last as usize)?.anim.length)
    }

    fn model_kind(&self, id: u64, e: &ClientEntity) -> u32 {
        self.model_of(id, e.look())
    }

    /// The model `entity` of `kind` is drawn with: its avatar's,
    /// else the player's for a skin of 64 x 64, else the kind's own.
    fn model_of(&self, entity: u64, kind: u32) -> u32 {
        if kind != content().game.player_kind {
            return kind;
        }
        if let Some(a) = self.avatar_of(entity, kind) {
            return a.kind;
        }
        let skin = self
            .player_skins
            .get(&entity)
            .filter(|s| s.size == [64, 64]);
        match (skin, self.models.skinned) {
            (Some(skin), Some(models)) => {
                if skin.slim {
                    models.slim
                } else {
                    models.classic
                }
            }
            _ => kind,
        }
    }

    fn avatar_of(&self, entity: u64, kind: u32) -> Option<&AvatarLook> {
        if self.avatars.is_empty() || (!self.show_avatars && entity != self.local) || kind != content().game.player_kind {
            return None;
        }
        self.avatars.get(&entity)
    }

    /// Draws the pictures of items' models that slots asked for; whether there are new ones.
    pub fn draw_icons(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, frame: &FrameInfo) -> bool {
        if self.icons_todo.is_empty() {
            return false;
        }
        let side = ICON_PIXELS * ICON_CELLS;
        if self
            .icon_sheets
            .last()
            .is_none_or(|s| s.used == ICON_CELLS * ICON_CELLS)
        {
            let desc = |format| TextureDesc { width: side, height: side, mips: 1, format, render_target: true };
            let color = gpu.create_texture(&desc(Format::Rgba8Srgb), "item pictures");
            let depth = gpu.create_texture(&desc(Format::Depth32), "item pictures depth");
            self.icon_sheets
                .push(IconSheet { color, depth, index: gpu.texture_index(color), used: 0 });
            let unit = 2.0 * ICON_PIXELS as f32 / side as f32;
            let proj = Mat4::from_cols(
                Vec4::new(unit, 0.0, 0.0, 0.0),
                Vec4::new(0.0, unit, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 0.1, 0.0),
                Vec4::new(-1.0, 1.0, 0.5, 1.0),
            );
            for &vb in &self.icon_views {
                let v = Scene {
                    view_proj: proj.to_cols_array_2d(),
                    params: [1e9, 2e9, 1.0, crate::shaders::time()],
                    environment: [1.0, 0.0, 0.0, 0.0],
                    light: [0.04, 0.0, 0.0, 0.0],
                    ..Default::default()
                };
                gpu.mapped(vb)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&v));
            }
        }
        let new_sheet = self.icon_sheets.last().is_some_and(|s| s.used == 0);
        let Some(sheet) = self.icon_sheets.last() else { return false };
        let (color, depth, index, mut used) = (sheet.color, sheet.depth, sheet.index, sheet.used);
        self.models.begin();
        // As block.json turns a block in a slot.
        let turn = Mat4::from_rotation_x(30f32.to_radians()) * Mat4::from_rotation_y(225f32.to_radians());
        while used < ICON_CELLS * ICON_CELLS {
            let Some((stack, kind)) = self.icons_todo.pop() else { break };
            if let Some(&icon) = self.icon_of_model.get(&kind) {
                super::models::icon_done(stack, Some(icon));
                continue;
            }
            let Some(layer) = self.model_layer(kind, FULL_LIGHT, (0, 0.0)) else {
                super::models::icon_done(stack, None);
                continue;
            };
            let Some(m) = self.models.kinds.get(kind as usize) else { continue };
            let mut clips = Clips::default();
            m.play(&mut clips, m.act(Act::Idle), &Play { looping: true, ..Play::default() });
            m.play(&mut clips, m.action(0, "gui"), &Play::default());
            let bones = self.models.pose_only(kind, turn, clips.all());
            let Some(m) = self.models.kinds.get(kind as usize) else { continue };
            // However it is shaped and turned, it fills the slot.
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for (p, at) in m.parts.iter().zip(&bones) {
                crate::model::part_points(p, |v| {
                    let v = at.transform_point3(v);
                    lo = lo.min(v);
                    hi = hi.max(v);
                });
            }
            let across = (hi.x - lo.x).max(hi.y - lo.y);
            if !across.is_finite() || across <= 0.0 {
                super::models::icon_done(stack, None);
                continue;
            }
            let (col, row) = (used % ICON_CELLS, used / ICON_CELLS);
            let cell = Vec3::new(col as f32 + 0.5, -(row as f32 + 0.5), 0.0);
            let root = Mat4::from_translation(cell)
                * Mat4::from_scale(Vec3::splat(0.92 / across))
                * Mat4::from_translation(-(lo + hi) / 2.0)
                * turn;
            let first = self.models.pose_clips(kind, root, clips.all());
            self.models.queue(kind, first, [layer]);
            let f = 1.0 / ICON_CELLS as f32;
            let icon = (index, [col as f32 * f, row as f32 * f, (col + 1) as f32 * f, (row + 1) as f32 * f]);
            self.icon_of_model.insert(kind, icon);
            super::models::icon_done(stack, Some(icon));
            used += 1;
        }
        if let Some(s) = self.icon_sheets.last_mut() {
            s.used = used;
        }
        let keep = if new_sheet { Some([0.0; 4]) } else { None };
        list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(color), keep)), depth: Some((depth, Some(0.0))) });
        let vb = self.icon_views[frame.slot];
        self.models
            .draw(gpu, list, frame, super::models::Batch::Icon, self.preview_pipe, vb);
        list.push(Cmd::EndPass);
        true
    }

    /// Prepares a local appearance without a world, using the inventory figure's renderer.
    pub fn prepare_preview(
        &mut self,
        gpu: &mut dyn Backend,
        assets: &Assets,
        skin: Option<&crate::net::Skin>,
        avatar: Option<crate::net::AvatarData>,
    ) {
        self.load(gpu, assets, &content().game.entities[content().game.player_kind as usize].texture);
        if let Some(skin) = skin {
            self.set_skin(gpu, self.local, skin);
        }
        if let Some(avatar) = avatar {
            self.set_avatar(gpu, self.local, std::sync::Arc::new(avatar));
        }
        for skin in std::mem::take(&mut self.models.new_skins) {
            self.load(gpu, assets, &skin);
        }
        self.models.sync(gpu);
    }

    pub fn set_preview_model(&mut self, slim: bool) {
        if let Some(skin) = self.player_skins.get_mut(&self.local) {
            skin.slim = slim;
        }
    }

    fn preview_surface(&mut self, gpu: &mut dyn Backend, size: [u32; 2]) -> Option<(TextureId, TextureId, u32)> {
        if size[0] == 0 || size[1] == 0 {
            return None;
        }
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
        Some((col, dep, index))
    }

    /// Unregistered block boxes use the same depth-tested faces as held block models.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_box_preview(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        size: [u32; 2],
        textures: &GameTextures,
        boxes: &[crate::content::ModelBox],
        view: crate::ui::render::ModelView,
        translucent: bool,
    ) -> Option<u32> {
        let (col, dep, index) = self.preview_surface(gpu, size)?;
        let mut faces = Vec::new();
        boxes_shape(&mut faces, textures, Mat4::IDENTITY, boxes, &content().states[0], None, Vec4::new(1.0, 1.0, 1.0, 0.0));
        light_boxes(&mut faces, 0, FULL_LIGHT);
        let projection = view.projection([size[0] as f32, size[1] as f32], boxes);
        if translucent {
            let depth = |quad: &GpuBox| {
                let point = |row: [f32; 4]| row[3] + (row[0] + row[1]) * 0.5;
                projection
                    .project_point3(Vec3::from(quad.rows.map(point)))
                    .z
            };
            faces.sort_by(|a, b| depth(a).total_cmp(&depth(b)));
        }

        let scene = Scene {
            view_proj: projection.to_cols_array_2d(),
            params: [1e9, 2e9, 1.0, crate::shaders::time()],
            environment: [1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };
        let scene_buffer = self.preview[frame.slot];
        gpu.mapped(scene_buffer)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&scene));
        let buffer = crate::gpu_layout::frame_buffer(
            gpu,
            &mut self.box_preview_buffers,
            frame.slot,
            faces.len().max(1) as u32,
            size_of::<GpuBox>() as u64,
            usage::STORAGE,
            "preview boxes",
        );
        gpu.mapped(buffer)[..faces.len() * size_of::<GpuBox>()].copy_from_slice(bytemuck::cast_slice(&faces));
        list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(col), Some([0.0; 4]))), depth: Some((dep, Some(0.0))) });
        if !faces.is_empty() {
            list.pipeline(if translucent { self.translucent_preview_pipe } else { self.box_preview_pipe });
            list.bind(crate::gpu_layout::bind::entity::BOXES, buffer);
            list.bind(crate::gpu_layout::bind::entity::SCENE, scene_buffer);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, textures.glow_buffer());
            list.draw(faces.len() as u32 * 36, 1);
        }
        list.push(Cmd::EndPass);
        Some(index)
    }

    /// Renders the local player's model into a `size` texture (physical pixels), turned towards
    /// `look` (relative to the texture's top left) like the inventory figure; or `subject`'s
    /// (entity, kind), such as an animal's in its own screen. Returns the texture's bindless
    /// index for the UI to draw.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_preview(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        size: [u32; 2],
        look: glam::Vec2,
        worn: &[ItemStack],
        subject: Option<(u64, u32)>,
    ) -> Option<u32> {
        let c = content();
        // The local player (the inventory's figure), or another entity (a menu's own, such as an
        // animal's screen), fitted to the frame by its size and seen from three quarters.
        let (entity, look_kind) = subject.unwrap_or((self.local, c.game.player_kind));
        let def = c.game.entities.get(look_kind as usize)?;
        let player = subject.is_none();
        let worn: &[ItemStack] = if player { worn } else { &[] };
        let (col, dep, index) = self.preview_surface(gpu, size)?;
        let (x, y, w, h) = (0.0, 0.0, size[0] as f32, size[1] as f32);
        let (sw, sh) = (w, h);
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
        let layers = self.layers(look_kind, entity, worn, FULL_LIGHT, (0, 0.0));
        self.models.begin();
        let root = Mat4::from_rotation_y(yaw) * Mat4::from_scale(Vec3::splat(1.0 / 16.0));
        let kind = self.model_of(entity, look_kind);
        let clock = crate::shaders::time();
        let idle = |m: &super::models::KindModel, t: f32| {
            let mut clips = Clips::default();
            m.play(&mut clips, m.act(Act::Idle), &Play { clock: t, looping: true, ..Play::default() });
            clips
        };
        let clips = self
            .models
            .kinds
            .get(kind as usize)
            .map_or(Clips::default(), |m| idle(m, clock));
        let first = self
            .models
            .add_clips(kind, root, (pitch, 0.0), clips.all(), layers.0[..layers.1].iter().copied());
        let mut on: Vec<(u32, usize)> = Vec::new();
        self.models
            .carried_by(kind, worn, |model, bone| on.push((model, bone)));
        for (model, bone) in on {
            let Some(layer) = self.model_layer(model, FULL_LIGHT, (0, 0.0)) else { continue };
            let at = self.models.bone(first as usize + bone);
            let clips = self
                .models
                .kinds
                .get(model as usize)
                .map_or(Clips::default(), |m| idle(m, clock));
            let bones = self.models.pose_clips(model, at, clips.all());
            self.models.queue(model, bones, [layer]);
        }
        let vb = self.preview[frame.slot];
        // Full daylight, no fog.
        let v = Scene {
            view_proj: proj.to_cols_array_2d(),
            params: [1e9, 2e9, 1.0, crate::shaders::time()],
            environment: [1.0, 0.0, 0.0, 0.0],
            light: [0.04, 0.0, 0.0, 0.0],
            ..Default::default()
        };
        gpu.mapped(vb)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&v));
        list.push(Cmd::BeginPass { motion: None, color: Some((Target::Texture(col), Some([0.0; 4]))), depth: Some((dep, Some(0.0))) });
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
        now: u64,
        entities: &FxHashMap<u64, ClientEntity>,
        me: Option<&ClientEntity>,
        // The local player's body where it isn't shown (the first person without `BODY`): it still
        // casts its sun shadow. It is `me`, posed once.
        shadow_me: Option<&ClientEntity>,
        hand: Option<Hand>,
        textures: &GameTextures,
        light: &impl Fn(IVec3) -> u32,
        biome: &impl Fn(IVec3) -> Option<u16>,
        scene: &Scene,
        scene_buf: BufferId,
        target: Option<(Hit, Transform)>,
        placement: Option<(IVec3, crate::world::placement::Part)>,
        breaking: &[Crack],
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
        let dt = std::mem::replace(&mut self.last_draw, std::time::Instant::now())
            .elapsed()
            .as_secs_f32();
        let moved = (cam.pos - std::mem::replace(&mut self.last_cam, cam.pos)).as_vec3();

        // Models: bones posed here (in parallel when many), drawn instanced per kind.
        let mut posed = std::mem::take(&mut self.posed);
        posed.clear();
        self.models.begin();
        let steady = Camera { bob: (Vec3::ZERO, 0.0, 0.0), ..*cam }.view();
        let mut aim_root = None;
        if hand.is_none() {
            self.camera_kick = (Vec3::ZERO, 0.0, 0.0);
        }
        let mut carried: Vec<Carry> = Vec::new();
        let planes = super::world_render::frustum_planes(cam.proj(aspect) * cam.view());
        let frame_count = self.frame_count;
        // The first person: its place in `posed`, its layers before the arms were left out of
        // them and which arms the clips bring up.
        let mut own_hand: Option<(usize, [Instance; super::models::MAX_LAYERS], usize, u32)> = None;
        // The first person's whole body for the sun's shadow map alone (`shadow_me`): its place
        // in `posed` and its layers before any were left out.
        let mut own_shadow: Option<(usize, [Instance; MAX_LAYERS], usize)> = None;
        let hidden = |e: &ClientEntity| {
            e.effects.iter().any(|f| {
                f.until > now
                    && c.game
                        .entity_effects
                        .get(f.id as usize)
                        .is_some_and(|d| d.hides)
            })
        };
        let own_body = me.filter(|_| hand.as_ref().is_none_or(|h| h.body));
        for (id, e) in entities
            .iter()
            .map(|(i, e)| (*i, e))
            .chain(me.map(|m| (self.local, m)))
        {
            let base = rel(e.pos);
            let far2 = base.length_squared();
            if far2 > 160.0 * 160.0 {
                continue;
            }
            // Invisible to everyone else.
            if hidden(e) && id != self.local {
                continue;
            }
            if e.stuck.is_some_and(|s| {
                let host = if s.host == self.local { own_body } else { entities.get(&s.host).filter(|h| !hidden(h)) };
                host.is_none()
            }) {
                continue;
            }
            // Out of the view it has nothing to show. What it may have on goes by its model's reach.
            if id != self.local && !e.item.is_empty() {
                if !planes.iter().all(|p| p.dot(base.extend(1.0)) > -1.5) {
                    continue;
                }
            } else if id != self.local {
                let reach = self
                    .models
                    .kinds
                    .get(self.model_kind(id, e) as usize)
                    .map_or(4.0, |m| m.reach * e.scale.max(1.0) + 2.0);
                if !planes.iter().all(|p| p.dot(base.extend(1.0)) > -reach) {
                    continue;
                }
            }
            let lit = shade(e.pos + DVec3::Y * 0.5);
            let here = biome(e.pos.floor().as_ivec3());
            // The effect a mod put on it, else the red of a recent hit; boxes only show the red.
            let hurt = (e.hurt / 0.5).clamp(0.0, 1.0);
            let shader_effect = e.shader_effect(now, |id| self.effect_slots.get(id as usize).is_some_and(|&s| s != 0));
            let effect = match shader_effect {
                Some((id, s)) => (self.effect_slots.get(id as usize).copied().unwrap_or(0), s),
                None if hurt > 0.0 => (self.effect_slots[weft_abi::entity_effect::HURT as usize], hurt),
                None => (0, 0.0),
            };
            if !e.item.is_empty() {
                let holder = posed.len();
                self.pose_ground_item(id, e, base, lit, here, effect, hurt, textures, &mut posed, &mut instances);
                if posed.len() > holder && e.item.data != 0 {
                    carried.push(Carry { holder, what: Carrying::Embedded { stack: e.item }, lit, biome: here, age: e.age, id });
                }
                continue;
            }
            let look = e.look();
            let kind = self.model_kind(id, e);
            let own_view = id == self.local && hand.is_some();
            // With its body showing the player sees its real arms, moving as anyone else sees them.
            // Without it there is nothing for them to be attached to: they are held in the view.
            let body = own_view && hand.as_ref().is_some_and(|h| h.body);
            let in_view = own_view && !body;
            let effect = if own_view { (0, 0.0) } else { effect };
            // Arms held in the view are bare, as vanilla's: armor is for a body.
            let worn: &[ItemStack] = if in_view { &[] } else { &e.worn };
            let (mut layers, mut layer_count) = self.layers(look, id, worn, lit, effect);
            if layer_count == 0 {
                continue;
            }
            if e.burning && !own_view {
                let crate::physics::Dims { half_width: hw, height: h, .. } = e.dims();
                let from = boxes.len();
                fire_boxes(&mut boxes, textures, base, hw as f32 * 2.0, h as f32, cam.yaw, e.age);
                light_boxes(&mut boxes, from, FULL_LIGHT);
            }
            // Lying down in its low box: head first where it looks when swimming (vanilla's
            // setupRotations), flat when crawling, turned around the middle of the body.
            let pose = e.pose.def();
            let lying = model::body_tilt(e.pose, e.pitch);
            let sleeping = pose.tilt == weft_abi::pose_tilt::ON_BACK;
            let body_yaw = if sleeping { e.yaw } else { e.body_yaw };
            // A projectile points all of itself where it flies, not just a head.
            let flies = c.game.entities[e.kind as usize].projectile.is_some();
            let swim = match lying {
                _ if flies => Mat4::from_rotation_x(e.pitch),
                Some(a) => model::tilted_body(a),
                None => Mat4::IDENTITY,
            };
            // Its build (its painter's bone scales) for this model's parts, and how far that lifts it.
            let (build, lift) = self.build_of(id, kind);
            let mut root = Mat4::from_translation(base)
                * Mat4::from_rotation_y(body_yaw)
                * swim
                * Mat4::from_scale(Vec3::splat(e.scale / 16.0))
                * Mat4::from_translation(Vec3::Y * lift);
            // Its head up to look ahead when lying down.
            let (pitch, head_yaw) = model::head_look(sleeping || flies, lying.is_some(), e.pitch, e.yaw - e.body_yaw);
            // Far away a pose lasts a few frames, each entity at its own turn. Who is dying or was
            // just hit still shows it.
            let every = if id == self.local || e.dying.is_some() || shader_effect.is_some() || e.hurt > 0.0 {
                1
            } else if far2 < 32.0 * 32.0 {
                1
            } else if far2 < 64.0 * 64.0 {
                2
            } else if far2 < 96.0 * 96.0 {
                4
            } else {
                8
            };
            let lod = if every == 1 {
                PoseLod::Direct
            } else if (frame_count.wrapping_add(id as u32)) % every != 0 && self.models.has_kept(id, kind) {
                PoseLod::Kept
            } else {
                PoseLod::Fresh
            };
            let (clips, standing) = if lod == PoseLod::Kept {
                (Clips::default(), 0)
            } else {
                self.clips_of(id, e, kind, look, lying, in_view, hand.as_ref().filter(|_| in_view).map(|h| h.lowered))
            };
            if own_view {
                // Its model goes where the camera is: the camera doesn't sit in the torso, whatever pose it's in.
                let eye = c.game.entities[look as usize].eye;
                if let Some(at) = self
                    .models
                    .eye_point(kind, root, &clips.all()[..standing], eye)
                {
                    root = Mat4::from_translation(-at) * root;
                }
                aim_root = self
                    .models
                    .view_point(kind, eye)
                    .filter(|_| in_view)
                    .map(|p| {
                        let view_root = Mat4::from_rotation_y(e.yaw)
                            * Mat4::from_rotation_x(e.pitch)
                            * Mat4::from_scale(Vec3::splat(1.0 / 16.0))
                            * Mat4::from_translation(-p);
                        (posed.len(), view_root)
                    });
                let (arms_up, kick) = self.view_motion(kind, root, &clips, standing, in_view, steady, eye);
                self.camera_kick = kick;
                if shadow_me.is_some() && self.shadow_pipe.is_some() {
                    own_shadow = Some((posed.len(), layers, layer_count));
                }
                own_hand = in_view.then_some((posed.len(), layers, layer_count, arms_up));
                // Cut by role and by named bone alike: a worn layer drawn by its bones' names
                // (`bone_mask`, our clothes) kept its head and sleeves at the camera otherwise.
                let cut = 1 << role::HEAD | 1 << role::ON_HEAD | arms_up;
                let cut_bones = self.bones_with(kind, cut);
                for l in &mut layers[..layer_count] {
                    l.roles &= !cut;
                    l.bone_mask &= !cut_bones;
                }
                if !body {
                    layer_count = 0;
                }
            }
            let from = self.models.clips.len() as u32;
            self.models.clips.extend_from_slice(clips.all());
            posed.push(Posed {
                id,
                kind,
                lod,
                root,
                pitch,
                head_yaw,
                clips: (from, self.models.clips.len() as u32),
                build,
                layers,
                layer_count,
            });
            let carry = |what| Carry { holder: posed.len() - 1, what, lit, biome: here, age: e.age, id };
            if !sleeping {
                for (stack, off) in [(e.held, false), (e.off_held, true)] {
                    if !stack.is_empty() {
                        let model = self.models.model_seen(stack);
                        carried.push(carry(Carrying::Held { stack, model, off }));
                    }
                }
            }
            // Held in the view, there is no body to have anything on.
            if !in_view {
                let parts = self
                    .models
                    .kinds
                    .get(kind as usize)
                    .map_or(&[][..], |m| m.parts.as_slice());
                self.models.carried_by(kind, &e.worn, |model, bone| {
                    let on_head = matches!(parts[bone].role, role::HEAD | role::ON_HEAD);
                    if !(own_view && on_head) {
                        carried.push(carry(Carrying::On { model, bone }));
                    }
                });
            }
        }
        let vb = scene_buf;
        let own_index = own_hand.as_ref().map(|o| o.0);
        let (firsts, own_first) = self.pose_bones(&posed, aim_root, own_index, hand.as_ref(), steady, dt, moved);
        if let Some((i, layers, count)) = own_shadow {
            self.models
                .queue_shadow(posed[i].kind, firsts[i] as u32, layers[..count].iter().copied());
        }
        let (hand_items, own_items) = self.place_carried(&posed, &firsts, carried, own_index, textures, &mut instances, dt, moved);
        self.models.end_located(cam.pos);
        if self.models.frame_number() % 256 == 0 {
            let alive = |id: &u64| *id == self.local || entities.contains_key(id);
            self.fades.retain(|id, _| alive(&id.0));
            let local = self.local;
            self.models
                .retain_kept(|id| id == local || entities.contains_key(&id));
        }
        self.posed = posed;
        self.models
            .draw(gpu, list, frame, super::models::Batch::World, self.model_pipe, vb);

        gizmo_line_boxes(&mut boxes, gizmo_lines, cam.pos);
        let overlay_start = boxes.len();
        if let Some((hit, place)) = target {
            let px_at_1 = 2.0 * (cam.fov.to_radians() * 0.5).tan() / frame.height.max(1) as f32;
            let origin = hit.origin.as_dvec3() + hit.part().shift();
            let root = Mat4::from_rotation_translation(place.rotation().as_quat(), rel(place.to_world(origin)));
            let selection = (c.states[hit.state as usize].outlined(), Vec3::ZERO);
            let preview = placement.map(|(pos, part)| {
                let offset = pos.as_dvec3() + part.shift() - origin;
                (c.states[part.state as usize].outlined(), offset.as_vec3())
            });
            outline_boxes(&mut boxes, std::iter::once(selection).chain(preview), root, px_at_1);
        }
        gizmo_fill_boxes(&mut boxes, gizmo_fills, cam.pos);
        crack_boxes(&mut boxes, breaking, textures, cam.pos, light);
        // Item instances after the rest, grouped by item: one instanced draw each.
        let overlay_end = boxes.len();
        instances.sort_unstable_by_key(|i| i.0);
        boxes.extend(instances.iter().map(|i| i.1));
        boxes.extend(hand_items.iter().map(|h| h.0));
        // The buffer for them may not be empty, and arms alone have no boxes.
        if boxes.is_empty() {
            boxes.push(GpuBox::default());
        }
        if !instances.is_empty() || !hand_items.is_empty() {
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
            self.draw_item_runs(list, &instances, overlay_end);
        }
        // Cracks, filled gizmos and the own hands come after translucent blocks.
        let list = late;
        if overlay_end > overlay_start {
            list.pipeline(self.overlay);
            list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
            list.bind(crate::gpu_layout::bind::entity::SCENE, vb);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
            list.push(Cmd::Draw {
                vertices: (overlay_end - overlay_start) as u32 * 36,
                instances: 1,
                first_vertex: overlay_start as u32 * 36,
                first_instance: 0,
            });
        }
        if let Some(own) = own_hand {
            // (Our Android hand pass, `Cmd::PassBreak`, went with our Android backend: upstream's
            // came in, Weft sync 11-22.)
            let hand_bob = hand.as_ref().and_then(|h| h.bob);
            self.draw_own_hand(gpu, list, frame, cam, hand_bob, scene, own, own_first, own_items, &hand_items, boxes.len(), buf, glow);
        }
        self.instances = instances;
    }

    /// A dropped item at `base`: a model of bones posed with its feet off the ground, or the
    /// item's boxes spinning.
    #[allow(clippy::too_many_arguments)]
    fn pose_ground_item(
        &mut self,
        id: u64,
        e: &ClientEntity,
        base: Vec3,
        lit: u32,
        biome: Option<u16>,
        effect: (u32, f32),
        hurt: f32,
        textures: &GameTextures,
        posed: &mut Vec<Posed>,
        instances: &mut Vec<(u64, GpuBox)>,
    ) {
        // Made of bones: posed like anything else that is, its feet off the ground.
        let bones = self
            .models
            .model_seen(e.item)
            .and_then(|k| Some((k, self.model_layer(k, lit, effect)?)));
        if let Some((kind, layer)) = bones {
            let m = &self.models.kinds[kind as usize];
            let mut clips = Clips::default();
            let how = Play { clock: e.age, looping: true, ..Play::default() };
            m.play(&mut clips, m.act(Act::Idle), &how);
            m.play(&mut clips, m.act(Act::Ground), &how);
            let (lo, hi) = m.bounds;
            let root = ground_spin(base, e.age)
                * Mat4::from_scale(Vec3::splat(0.5 / 16.0))
                * Mat4::from_translation(Vec3::new(-(lo.x + hi.x) / 2.0, -lo.y, -(lo.z + hi.z) / 2.0));
            let from = self.models.clips.len() as u32;
            self.models.clips.extend_from_slice(clips.all());
            let mut layers = [Instance::default(); MAX_LAYERS];
            layers[0] = layer;
            posed.push(Posed {
                id,
                kind,
                lod: PoseLod::Direct,
                root,
                pitch: 0.0,
                head_yaw: 0.0,
                clips: (from, self.models.clips.len() as u32),
                build: (0, 0),
                layers,
                layer_count: 1,
            });
        } else if let Some(ItemModel { boxes: m, solid }) = self.item_model(textures, &e.item, biome) {
            instances.push((run_key(m), item_instance(ground_transform(base, e.age, solid), m.0, hurt, lit)));
        }
    }

    /// One instanced draw per run of the same model in `instances` (sorted by `run_key`), their
    /// boxes from `first` on.
    fn draw_item_runs(&self, list: &mut CommandList, instances: &[(u64, GpuBox)], first: usize) {
        let mut i = 0;
        while i < instances.len() {
            let key = instances[i].0;
            let run = instances[i..].iter().take_while(|x| x.0 == key).count();
            let boxes_per = key as u32;
            list.push(Cmd::Draw { vertices: boxes_per * 36, instances: run as u32, first_vertex: 0, first_instance: (first + i) as u32 });
            i += run;
        }
    }

    /// The first person's arms and what they hold, in a pass of their own with vanilla's fixed
    /// 70 degree view; `boxes` holds the held items' instances at its end.
    #[allow(clippy::too_many_arguments)]
    fn draw_own_hand(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        bob: Option<(Vec3, f32, f32)>,
        scene: &Scene,
        (i, own, count, arms_up): (usize, [Instance; MAX_LAYERS], usize, u32),
        own_first: usize,
        own_items: Vec<(u32, u32, Instance)>,
        hand_items: &[(GpuBox, u32)],
        boxes: usize,
        buf: BufferId,
        glow: BufferId,
    ) {
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let item_buf = self.item_buf.map(|b| b.0);
        // The hand bobs walking even where the view doesn't (`Hand::bob`).
        let hand_cam = Camera { fov: 70.0, bob: bob.unwrap_or(cam.bob), ..*cam };
        let hv = Scene { view_proj: (hand_cam.proj(aspect) * hand_cam.view()).to_cols_array_2d(), ..*scene };
        let hb = self.hand_views[frame.slot];
        gpu.mapped(hb)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&hv));
        self.models.begin_batch();
        let kind = self.posed[i].kind;
        // Only the arms, by role and by named bone: a shirt drawn by its bones' names
        // (`bone_mask`) came along whole, a slab of cloth across the view.
        let arm_bones = self.bones_with(kind, arms_up);
        let arms = own[..count].iter().copied().map(|mut l| {
            l.roles &= arms_up;
            l.bone_mask &= arm_bones;
            l
        });
        self.models.queue(kind, own_first as u32, arms);
        for (kind, first, l) in own_items {
            self.models.queue(kind, first, [l]);
        }
        self.models
            .draw(gpu, list, frame, super::models::Batch::Hand, self.model_hand_pipe, hb);
        if let Some(ib) = item_buf.filter(|_| !hand_items.is_empty()) {
            list.pipeline(self.hand_item_pipe);
            list.bind(crate::gpu_layout::bind::entity::BOXES, buf);
            list.bind(crate::gpu_layout::bind::entity::SCENE, hb);
            list.bind(crate::gpu_layout::bind::entity::GLOW_MAP, glow);
            list.bind(crate::gpu_layout::bind::entity::ITEM_BOXES, ib);
            let first = boxes - hand_items.len();
            for (k, (_, n)) in hand_items.iter().enumerate() {
                list.push(Cmd::Draw { vertices: n * 36, instances: 1, first_vertex: 0, first_instance: (first + k) as u32 });
            }
        }
    }

    /// What entity `id` (of model `kind`, looking like `look`) plays this frame: its clips, and
    /// how many of them come first as its standing pose (what the eyes follow; actions after).
    /// `lying`: how far its pose tilts it; `lowered`: the first person's hand, when held in view.
    #[allow(clippy::too_many_arguments)]
    fn clips_of(
        &mut self,
        id: u64,
        e: &ClientEntity,
        kind: u32,
        look: u32,
        lying: Option<f32>,
        in_view: bool,
        lowered: Option<f32>,
    ) -> (Clips, usize) {
        let c = content();
        let pose = e.pose.def();
        let sleeping = pose.tilt == weft_abi::pose_tilt::ON_BACK;
        let (mut clips, mut standing) = (Clips::default(), 0);
        let Some(m) = self.models.kinds.get(kind as usize) else { return (clips, standing) };
        // Smaller steps, quicker: a small one's legs go round faster (`ClientEntity::scale`).
        let how = Play { clock: e.age, distance: e.walk * 0.6 / e.scale.max(0.05), speed: e.swing, ..Play::default() };
        // Stands for what it says unless a mod swapped it for now, and has a clip for that.
        let replacement = |item: u32, name: &str| -> Option<&str> {
            match e.swaps.iter().find(|s| s.0 == name) {
                Some(s) if !m.action_seen(item, &s.1, in_view).is_empty() => Some(&s.1),
                _ => None,
            }
        };
        let swapped = |item, name: &'static str| replacement(item, name).unwrap_or(name);
        let act = |a: Act| if e.swaps.is_empty() { m.act(a) } else { m.action(0, swapped(0, a.name())) };
        // Upstream plays these through `model::Posture`; chiacraft keeps its own for the run gait
        // and the smaller steps of a small one (`how`).
        m.play(&mut clips, act(Act::Idle), &Play { looping: true, ..how });
        // A rider sits, whatever its mount's walking does to where it is.
        if e.vehicle.is_some() {
            m.play(&mut clips, m.action(0, swapped(0, "ride")), &Play { looping: true, ..how });
        } else if lying.is_none() {
            // Running takes over from walking between 3 and 4.5 blocks a second (`swing` is the
            // speed over 5, eased), for a model with a `run` clip. Both gaits go by the distance
            // moved, so their feet keep pace.
            let run = act(Act::Run);
            let fast = if run.is_empty() { 0.0 } else { ((e.swing - 0.6) / 0.3).clamp(0.0, 1.0) };
            let gait = Play { drive: Drive::Distance, by_speed: true, ..how };
            m.play(&mut clips, act(Act::Walk), &Play { weight: 1.0 - fast, ..gait });
            if fast > 0.0 {
                m.play(&mut clips, run, &Play { weight: fast, ..gait });
            }
        }
        if e.crouched > 0.0 {
            m.play(&mut clips, act(Act::Sneak), &Play { looping: true, weight: e.crouched, ..how });
        }
        // Lying, its pose's own clip; one the model hasn't got swims.
        if !sleeping && lying.is_some() {
            let own = m.action(0, &pose.action);
            let limbs = if own.is_empty() { act(Act::Swim) } else { own };
            m.play(&mut clips, limbs, &Play { drive: Drive::Distance, ..how });
        } else if lying.is_none() && !pose.action.is_empty() && e.pose != crate::world::Pose::CROUCH {
            m.play(&mut clips, m.action(0, &pose.action), &Play { looping: true, ..how });
        }
        standing = clips.n;
        // From the eyes it stays in view whatever the body does.
        if lying.is_none() || (in_view && !sleeping) {
            // What it holds, and what it does with that. An item for both hands in the
            // other hand, or with something there, plays what it has for one.
            let hold_in = |item: u32, busy: bool| -> &str {
                let one = if busy && crate::content::two_handed(c, item) { crate::content::one_handed_action(c, item) } else { None };
                one.unwrap_or_else(|| swapped(item, hold_of(c, m, item)))
            };
            let held_how = Play { looping: true, aim: true, hand: true, ..how };
            // Hands on a wall or a glider's bar: the pose's clip has the arms, not what is held.
            let hands_busy = crate::conditions::asks_hold(c, pose.when);
            if hands_busy {
                // Nothing of the hands: the item rides along on the raised arm.
            } else if e.held.item != 0 {
                m.play(&mut clips, m.action_seen(e.held.item, hold_in(e.held.item, e.off_held.item != 0), in_view), &held_how);
            } else if in_view {
                m.play(&mut clips, m.action_seen(0, "hold_empty", true), &held_how);
            }
            if e.off_held.item != 0 && !hands_busy {
                m.play(
                    &mut clips,
                    m.action_seen(e.off_held.item, hold_in(e.off_held.item, true), in_view),
                    &Play { mirror: true, ..held_how },
                );
            }
            let progress = Play { drive: Drive::Progress, aim: in_view, hand: true, ..how };
            if let Some(lowered) = lowered {
                m.play(
                    &mut clips,
                    m.action_seen(e.held.item, swapped(e.held.item, "equip"), true),
                    &Play { progress: lowered, ..progress },
                );
            }
            if let Some(p) = e.arm_swing {
                let item = if e.swing_off { e.off_held.item } else { e.held.item };
                let swing =
                    if item == 0 && !in_view { act(Act::Swing) } else { m.action_seen(item, swapped(item, m.swing_of(item)), in_view) };
                m.play(&mut clips, swing, &Play { progress: p, mirror: e.swing_off, ..progress });
            }
            if let Some((p, off)) = e.using {
                let item = if off { e.off_held.item } else { e.held.item };
                // Used until let go (a bow drawn), it plays by the seconds held and stays at its end.
                let how = if super::endless_use(e.use_ticks) {
                    Play { drive: Drive::Clock, clock: p, mirror: off, ..progress }
                } else {
                    Play { progress: p, mirror: off, ..progress }
                };
                m.play(&mut clips, m.action_seen(item, swapped(item, "use"), in_view), &how);
            }
        }
        if let Some(d) = e.dying {
            m.play(&mut clips, act(Act::Death), &Play { clock: d, ..how });
        }
        if let Some((name, start, looping)) = &e.action {
            let secs = e.age - start;
            let list = m.action_seen(e.held.item, name, in_view);
            if list.is_empty() && !self.warned_actions.contains(&(kind, name.clone())) {
                log::warn!("no clip for action {name} on {}", c.game.entities[look as usize].name);
                self.warned_actions.insert((kind, name.clone()));
            }
            let running = list
                .last()
                .and_then(|&i| m.anims.get(i as usize))
                .is_some_and(|b| !b.finished(secs, *looping));
            if running {
                m.play(&mut clips, list, &Play { clock: secs, looping: *looping, aim: in_view, ..how });
            }
        }
        // Coming and going smoothly. Nearly all of a crowd has nothing here to look at.
        if clips.n > standing || (!self.fades.is_empty() && self.fades.contains_key(&(id, kind))) {
            let fades = self.fades.entry((id, kind)).or_default();
            super::models::fade_clips(fades, m, &mut clips, standing, e.age);
            if fades.is_empty() {
                self.fades.remove(&(id, kind));
            }
        }
        (clips, standing)
    }

    /// What the first person's actions (`clips` after its `standing` ones) do: which arms they
    /// bring into view, and how they move the camera (a bone named `camera`, and the head).
    #[allow(clippy::too_many_arguments)]
    fn view_motion(
        &mut self,
        kind: u32,
        root: Mat4,
        clips: &Clips,
        standing: usize,
        in_view: bool,
        steady: Mat4,
        eye: f32,
    ) -> (u32, (Vec3, f32, f32)) {
        let acting = &clips.all()[standing..];
        let mut arms_up = 0u32;
        let mut kick = (Vec3::ZERO, 0.0, 0.0);
        let mut head_moves = false;
        if let Some(m) = self.models.kinds.get(kind as usize) {
            let names = |i: usize| {
                acting.iter().any(|c| {
                    m.anims
                        .get(c.anim as usize)
                        .is_some_and(|b| b.names(i, c.mirror))
                })
            };
            for (i, p) in m.parts.iter().enumerate() {
                if in_view && (p.role == role::ARM_LEFT || p.role == role::ARM_RIGHT) && names(i) {
                    arms_up |= 1 << p.role;
                }
            }
            for c in acting {
                let Some((pos, rot)) = m
                    .anims
                    .get(c.anim as usize)
                    .and_then(|b| b.anim.free_bone("camera", c.t))
                else {
                    continue;
                };
                kick.0 -= steady.transform_vector3(root.transform_vector3(pos * c.w));
                kick.1 -= rot.z * c.w;
                kick.2 -= rot.x * c.w;
            }
            head_moves = m
                .parts
                .iter()
                .position(|p| p.role == role::HEAD)
                .is_some_and(names);
        }
        if head_moves {
            // The pose has its eyes at the camera; what the action does on top of it, the camera does too.
            if let Some(at) = self.models.eye_point(kind, root, clips.all(), eye) {
                kick.0 -= steady.transform_vector3(at);
            }
        }
        (arms_up, kick)
    }

    /// Poses the bones of `posed`; returns where each one's start and where the first person's
    /// do (0 without one), whose arms lag behind a turning camera.
    #[allow(clippy::too_many_arguments)]
    fn pose_bones(
        &mut self,
        posed: &[Posed],
        aim_root: Option<(usize, Mat4)>,
        own_hand: Option<usize>,
        hand: Option<&Hand>,
        steady: Mat4,
        dt: f32,
        moved: Vec3,
    ) -> (Vec<usize>, usize) {
        let first_bone = self.models.bones_so_far();
        self.models.add_many(posed, aim_root);
        // Where each one's bones start.
        let mut firsts = Vec::with_capacity(posed.len());
        let mut at = first_bone;
        for p in posed {
            firsts.push(at);
            at += self
                .models
                .kinds
                .get(p.kind as usize)
                .map_or(0, |m| m.parts.len());
        }
        // The first person's arms lag behind a turning camera. Its bones are the last ones.
        let mut own_first = 0;
        if let (Some(i), Some(h)) = (own_hand, hand) {
            let parts = self
                .models
                .kinds
                .get(posed[i].kind as usize)
                .map_or(&[][..], |m| m.parts.as_slice());
            own_first = firsts[i];
            let arms: Vec<usize> = parts
                .iter()
                .enumerate()
                .filter(|(_, p)| p.role == role::ARM_LEFT || p.role == role::ARM_RIGHT)
                .map(|(i, _)| i)
                .collect();
            let lag = steady.inverse() * Mat4::from_rotation_x(-h.lag.0 * 0.1) * Mat4::from_rotation_y(-h.lag.1 * 0.1) * steady;
            let kind = posed[i].kind;
            for a in arms {
                self.models.move_subtree(kind, own_first, a, lag);
            }
        }
        // What moves by itself, once clips and the rest are done.
        for (p, &at) in posed.iter().zip(&firsts) {
            self.models.located_at(p.id, p.kind, at);
            self.models.move_live(p.id, p.kind, 0, at, dt, moved);
        }
        (firsts, own_first)
    }

    /// Places what posed entities hold and have on: items in the hand as instances (`instances`,
    /// the first person's apart), models posed by the holder's clips. Returns the first person's
    /// held items and models.
    #[allow(clippy::too_many_arguments)]
    fn place_carried(
        &mut self,
        posed: &[Posed],
        firsts: &[usize],
        carried: Vec<Carry>,
        own_holder: Option<usize>,
        textures: &GameTextures,
        instances: &mut Vec<(u64, GpuBox)>,
        dt: f32,
        moved: Vec3,
    ) -> (Vec<(GpuBox, u32)>, Vec<(u32, u32, Instance)>) {
        let mut hand_items: Vec<(GpuBox, u32)> = Vec::new();
        let mut own_items: Vec<(u32, u32, Instance)> = Vec::new();
        for Carry { holder, what, lit, biome, age, id } in carried {
            let at = firsts[holder];
            let parts = self
                .models
                .kinds
                .get(posed[holder].kind as usize)
                .map(|m| m.parts.as_slice())
                .unwrap_or_default();
            let own = own_holder == Some(holder);
            let (model, root, off, slot) = match what {
                Carrying::Embedded { stack } => {
                    for (model, first, layer) in self.embedded_models(stack, posed[holder].kind, at, lit) {
                        self.models.queue(model, first, [layer]);
                    }
                    continue;
                }
                Carrying::On { model, bone } => (Some(model), self.models.bone(at + bone), false, model + 2),
                Carrying::Held { stack, model, off } => {
                    let item = stack.item;
                    let side = if off { role::ARM_LEFT } else { role::ARM_RIGHT };
                    let holder_model = self.models.kinds.get(posed[holder].kind as usize);
                    // Its bone for it, as a shoulder that far up the arm from there; else the arm.
                    let on_bone = holder_model.and_then(|m| m.item_bones[off as usize]);
                    let arm = match on_bone {
                        Some(b) => {
                            self.models.bone(at + b) * Mat4::from_translation(Vec3::from_array(parts[b].pivot) + Vec3::Y * PALM_PIXELS)
                        }
                        None => {
                            let Some(i) = parts.iter().position(|p| p.role == side) else {
                                continue;
                            };
                            self.models.bone(at + i) * Mat4::from_translation(Vec3::from_array(parts[i].pivot))
                        }
                    };
                    if model.is_none() {
                        if let Some(ItemModel { boxes: m, solid }) = self.item_model(textures, &stack, biome) {
                            let display = self.models.displays.get(&item).copied();
                            // In its own first-person hand, as much bigger as its kind says.
                            let grow = if own { first_person_held_scale(posed[holder].kind) } else { 1.0 };
                            let inst = item_instance(held_transform(arm, item, solid, off, display, grow), m.0, 0.0, lit);
                            if own {
                                hand_items.push((inst, m.1));
                            } else {
                                instances.push((run_key(m), inst));
                            }
                        }
                        continue;
                    }
                    (model, palm(arm) * Mat4::from_scale(Vec3::splat(1.0 / 16.0)), off, off as u32)
                }
            };
            // A model: posed by the holder's clips of the same names, then whatever moves by itself.
            let Some(kind) = model else { continue };
            let (from, to) = posed[holder].clips;
            let clips = self
                .models
                .clips_along(kind, posed[holder].kind, &self.models.clips[from as usize..to as usize], age, off);
            let bones = self.models.pose_clips(kind, root, clips.all());
            self.models.located_at(id, kind, bones as usize);
            self.models
                .move_live(id, kind, slot, bones as usize, dt, moved);
            if let Some(l) = self.model_layer(kind, lit, (0, 0.0)) {
                if own {
                    own_items.push((kind, bones, l));
                } else {
                    self.models.queue(kind, bones, [l]);
                }
            }
            if let Carrying::Held { stack, .. } = what {
                for (model, first, layer) in self.embedded_models(stack, kind, bones as usize, lit) {
                    if own {
                        own_items.push((model, first, layer));
                    } else {
                        self.models.queue(model, first, [layer]);
                    }
                }
            }
        }
        (hand_items, own_items)
    }

    /// The models posed by this frame's `draw` (and the player's own body in the first person,
    /// `shadow_me`) into the bound sun's shadow map, with `view` the shadow pass's scene.
    pub fn draw_shadow(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, frame: &FrameInfo, view: BufferId) {
        if let Some(pipe) = self.shadow_pipe {
            self.models
                .draw(gpu, list, frame, super::models::Batch::Shadow, pipe, view);
        }
    }

    fn embedded_models(&mut self, stack: ItemStack, kind: u32, bones: usize, lit: u32) -> Vec<(u32, u32, Instance)> {
        let mut out = Vec::new();
        for projectile in crate::stack::embedded(&stack) {
            let Some(model) = content().game.catalog.find(ek::ENTITY, &projectile.kind) else { continue };
            let bone = self.models.kinds[kind as usize]
                .parts
                .iter()
                .position(|p| p.name == projectile.bone)
                .unwrap_or(0);
            let Some(root) = embedded_root(&projectile, self.models.bone(bones + bone)) else { continue };
            let first = self.models.pose_clips(model, root, &[]);
            let (layers, count) = self.layers(model, 0, &[], lit, (0, 0.0));
            if count > 0 {
                out.push((model, first, layers[0]));
            }
        }
        out
    }
}

fn embedded_root(projectile: &weft_abi::EmbeddedProjectile, attached: Mat4) -> Option<Mat4> {
    let direction = Vec3::from_array(projectile.direction).try_normalize()?;
    Some(attached * Mat4::from_rotation_translation(Quat::from_rotation_arc(-Vec3::Z, direction), Vec3::from_array(projectile.position)))
}

/// The selection outline (and the placement preview's) in `root`: thin boxes along the edges of
/// each box, both collected in the same frame so a shared edge produces one box. Each is
/// see-through and about a pixel wide wherever the edge is (vanilla's faint thin lines);
/// `px_at_1` is a pixel's size a block away. Ronald: solid black ones, a fixed 0.012 blocks
/// thick, were 3 px up close and too heavy; he picked "thin and faint like vanilla", about 1 px
/// at about 40 % dark. Drawn with the overlays.
fn outline_boxes<'a>(out: &mut Vec<GpuBox>, shapes: impl IntoIterator<Item = (&'a [[f32; 6]], Vec3)>, root: Mat4, px_at_1: f32) {
    const EPS: f32 = 1e-5;
    let mut edges: Vec<(usize, Vec3, f32)> = Vec::new();
    for (shape, offset) in shapes {
        for sel in shape {
            let min = offset + Vec3::new(sel[0], sel[1], sel[2]);
            let max = offset + Vec3::new(sel[3], sel[4], sel[5]);
            for axis in 0..3 {
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                for corner in 0..4 {
                    let mut start = min;
                    let mut end = max[axis];
                    start[u] = if corner & 1 == 0 { min[u] } else { max[u] };
                    start[v] = if corner & 2 == 0 { min[v] } else { max[v] };
                    edges.retain(|&(other_axis, other_start, other_end)| {
                        if axis != other_axis
                            || (start[u] - other_start[u]).abs() > EPS
                            || (start[v] - other_start[v]).abs() > EPS
                            || start[axis] > other_end + EPS
                            || other_start[axis] > end + EPS
                        {
                            return true;
                        }
                        start[axis] = start[axis].min(other_start[axis]);
                        end = end.max(other_end);
                        false
                    });
                    edges.push((axis, start, end));
                }
            }
        }
    }
    for (axis, start, end) in edges {
        // The edge's middle, and its half width there: OUTLINE_PX pixels across.
        let mut mid = start;
        mid[axis] = (start[axis] + end) * 0.5;
        let t = (root.transform_point3(mid).length() * px_at_1 * OUTLINE_PX * 0.5).max(0.0015);
        let lo = start - Vec3::splat(t);
        let mut size = Vec3::splat(2.0 * t);
        size[axis] += end - start[axis];
        let m = root * Mat4::from_translation(lo) * Mat4::from_scale(size);
        out.push(GpuBox {
            rows: rows(m),
            color: [0.0, 0.0, 0.0, 0.0],
            ids: [0, box_mode::FILLED, OUTLINE_DARK.to_bits(), 0],
            ..Default::default()
        });
    }
}

#[cfg(test)]
#[test]
fn adjacent_block_outlines_share_the_seam() {
    let cube = [[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]];
    let mut boxes = Vec::new();
    outline_boxes(&mut boxes, [(cube.as_slice(), Vec3::ZERO), (cube.as_slice(), Vec3::X)], Mat4::IDENTITY, 0.001);
    // Twelve outer segments, plus the four edges around their shared face.
    assert_eq!(boxes.len(), 16);
}

/// Gizmo lines: thin boxes like the outline, a little thicker so they read from afar.
fn gizmo_line_boxes(out: &mut Vec<GpuBox>, lines: &[super::gizmo::Line], cam: DVec3) {
    for &(a, b, color) in lines {
        let (a, b) = ((a - cam).as_vec3(), (b - cam).as_vec3());
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
        out.push(GpuBox { rows: rows(m), color: [rgb(24), rgb(16), rgb(8), 0.0], ids: [0, box_mode::SOLID, 0, 0], ..Default::default() });
    }
}

/// Filled gizmo boxes: see-through by their own alpha (mode 5), slightly grown so they do not
/// fight with the faces of the blocks they cover.
fn gizmo_fill_boxes(out: &mut Vec<GpuBox>, fills: &[super::gizmo::Fill], cam: DVec3) {
    for &(a, b, color) in fills {
        let (a, b) = ((a - cam).as_vec3() - Vec3::splat(0.002), (b - cam).as_vec3() + Vec3::splat(0.002));
        if a.length_squared().min(b.length_squared()) > 160.0 * 160.0 {
            continue;
        }
        let m = Mat4::from_translation(a) * Mat4::from_scale(b - a);
        let ch = |s: u32| ((color >> s) & 255) as f32 / 255.0;
        out.push(GpuBox {
            rows: rows(m),
            color: [ch(24), ch(16), ch(8), 0.0],
            ids: [0, box_mode::FILLED, ch(0).to_bits(), 0],
            ..Default::default()
        });
    }
}

/// Break progress over the faces of the block's model (a ladder's board, not its whole cube),
/// textured like the block's own faces there.
fn crack_boxes(out: &mut Vec<GpuBox>, breaking: &[Crack], textures: &GameTextures, cam: DVec3, light: &impl Fn(IVec3) -> u32) {
    let c = content();
    for &Crack { pos: p, progress, place, part } in breaking {
        if progress <= 0.0 {
            continue;
        }
        let stage = ((progress * 10.0) as usize).min(9);
        let l = textures.texture(c.crack_textures[stage]);
        let lit = light(place.cell_at(p));
        let root =
            Mat4::from_rotation_translation(place.rotation().as_quat(), (place.to_world(p.as_dvec3() + part.shift()) - cam).as_vec3());
        let shape = c
            .states
            .get(part.state as usize)
            .map_or(&[][..], |s| s.outlined());
        for b in shape {
            let (mn, mx) = (Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5]));
            let (a, b) = (mn - Vec3::splat(0.004), mx + Vec3::splat(0.004));
            let m = root * Mat4::from_translation(a) * Mat4::from_scale(b - a);
            out.push(GpuBox {
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

/// Flames facing the camera up the whole body, like vanilla's burning mobs: a stack of quads,
/// each higher, narrower and a little further back, in front of the body so it doesn't hide them.
fn fire_boxes(out: &mut Vec<GpuBox>, textures: &GameTextures, base: Vec3, width: f32, height: f32, cam_yaw: f32, age: f32) {
    let c = content();
    // Vanilla's units: the body's width times 1.4.
    let unit = width * 1.4;
    let (mut left, mut half) = (height / unit, 0.5f32);
    for i in 0..16 {
        if left <= 0.0 {
            break;
        }
        // Every other one at another point of the animation, as vanilla's two pictures.
        let l = textures.frame_now(c.fire_texture, age * crate::world::TPS as f32 + i as f32 * 11.0);
        let w = 2.0 * half * unit;
        // Toward the camera is +Z here.
        let m = Mat4::from_translation(base)
            * Mat4::from_rotation_y(cam_yaw)
            * Mat4::from_translation(Vec3::new(-w / 2.0, 0.45 * i as f32 * unit, (0.3 - 0.03 * i as f32) * unit))
            * Mat4::from_scale(Vec3::new(w, 1.4 * unit, 0.0));
        out.push(GpuBox {
            rows: rows(m),
            color: [1.0, 1.0, 1.0, 0.0],
            ids: [0, box_mode::FLAMES, l | l << 16, l | l << 16],
            ids2: [l | l << 16, 0, 0, 0],
            ..Default::default()
        });
        (left, half) = (left - 0.45, half * 0.9);
    }
}

/// Which model an item instance draws: its boxes, first and count.
fn run_key(m: (u32, u32)) -> u64 {
    (m.0 as u64) << 32 | m.1 as u64
}

/// Model boxes (a block's, or an item's from its file) in the unit cube `m` maps, each face's
/// layers one over the other as in the world, a kind's in `biome` (an item's own boxes pass air).
fn boxes_shape(
    out: &mut Vec<GpuBox>,
    textures: &GameTextures,
    m: Mat4,
    boxes: &[crate::content::ModelBox],
    s: &crate::content::StateInfo,
    biome: Option<u16>,
    color: Vec4,
) {
    let c = content();
    for b in boxes {
        for d in 0..6 {
            let Some((corners, uv)) = super::mesher::box_face(b, d) else { continue };
            let (n, shade) = super::mesher::face_look(&corners);
            let shade = if b.flat { 1.0 } else { shade };
            let [p0, p1, _, p3] = corners.map(Vec3::from_array);
            for (k, l) in super::mesher::face_layers(c, s, b.textures[d], d)
                .iter()
                .enumerate()
            {
                // Each layer over the one before, just outside it.
                let lift = k as f32 * 0.002;
                let place = Mat4::from_cols((p1 - p0).extend(0.0), (p3 - p0).extend(0.0), Vec4::ZERO, (p0 + n * lift).extend(1.0));
                out.push(GpuBox {
                    rows: rows(m * place),
                    dims: [shade, 0.0, 0.0, 0.0],
                    uvo: uv.map(|v| v / 16.0),
                    color: color.to_array(),
                    ids: [0, box_mode::QUAD, textures.texture(l.tex), 0],
                    ids2: [0, 0, (l.tint != 0) as u32, c.kind_color(l.tint, biome)],
                });
            }
        }
    }
}

/// An item model instance: `m` places the model's unit cube.
fn item_instance(m: Mat4, first: u32, hurt: f32, light: u32) -> GpuBox {
    GpuBox { rows: rows(m), color: [1.0, 1.0, 1.0, hurt], ids: [0, 0, first, 0], ids2: [0, light, 0, 0], ..Default::default() }
}

/// How much bigger than held items are drawn in kind `kind`'s own first-person hand
/// (`comp::FIRST_PERSON_HELD_SCALE`, 1 without it).
fn first_person_held_scale(kind: u32) -> f32 {
    let c = content();
    c.components
        .id(weft_abi::comp::FIRST_PERSON_HELD_SCALE)
        .and_then(|id| c.components.get::<f32>(crate::components::ENTITY, kind, id))
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(1.0)
}

/// Where an item goes in the hand of a model, from its arm at the shoulder (pixels): vanilla's
/// ItemInHandLayer, then the item model's third person display transform. Vanilla's model space is
/// ours turned around z, and it measures from a shoulder one pixel closer to the body. `grow`
/// times as big about where it's held.
fn held_transform(arm: Mat4, item: u32, solid: bool, off: bool, display: Option<Display>, grow: f32) -> Mat4 {
    use std::f32::consts::{FRAC_PI_2, PI};
    let deg = |d: f32| d.to_radians();
    let hand = arm
        * Mat4::from_scale(Vec3::splat(16.0))
        * Mat4::from_rotation_z(PI)
        * Mat4::from_rotation_x(-FRAC_PI_2)
        * Mat4::from_rotation_y(PI)
        * Mat4::from_translation(Vec3::new(0.0, 0.125, -0.625));
    // The model file's, as Blockbench's Display tab has it, else a block straight in the hand,
    // handheld.json or generated.json.
    let plain = |rotation, translation, scale| Display { rotation, translation, scale: [scale; 3] };
    let d = display.unwrap_or_else(|| {
        if solid {
            plain([0.0, 0.0, 0.0], [0.0, 2.5, 0.0], 0.375)
        } else if content().item_has(item, content().ids.handheld) {
            plain([0.0, -90.0, 55.0], [0.0, 4.0, 0.5], 0.85)
        } else {
            plain([0.0, 0.0, 0.0], [0.0, 3.0, 1.0], 0.55)
        }
    });
    // The other hand's display transforms are these turned the other way.
    let side = if off { -1.0 } else { 1.0 };
    let [rx, ry, rz] = d.rotation;
    let [tx, ty, tz] = d.translation;
    hand * Mat4::from_translation(Vec3::new(tx * side, ty, tz) / 16.0)
        * Mat4::from_rotation_x(deg(rx))
        * Mat4::from_rotation_y(deg(ry * side))
        * Mat4::from_rotation_z(deg(rz * side))
        * Mat4::from_scale(Vec3::from_array(d.scale) * grow)
        // Seen from behind, a flat item is its mirror image, without turning its faces inside out.
        * Mat4::from_rotation_y(if off { PI } else { 0.0 })
        * Mat4::from_translation(Vec3::splat(-0.5))
}

/// How an item is held unless it says: a long tool lower down.
fn hold_of(c: &crate::content::Content, m: &super::models::KindModel, item: u32) -> &'static str {
    let tool = c.item_has(item, c.ids.handheld);
    if tool && !m.has_own(item, "hold") { "hold_tool" } else { "hold" }
}

/// From the shoulder to the palm, along the arm.
const PALM_PIXELS: f32 = 10.0;

/// Where a model of bones held in the hand has its origin: on the palm, its front (-Z) along the
/// arm, so an arm held out points it ahead with its top (+Y) up.
fn palm(arm: Mat4) -> Mat4 {
    arm * Mat4::from_scale(Vec3::splat(16.0))
        * Mat4::from_translation(Vec3::new(0.0, -PALM_PIXELS / 16.0, 0.0))
        * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
}

/// How a dropped item turns and bobs.
fn ground_spin(base: Vec3, age: f32) -> Mat4 {
    let bob = (age * 2.5).sin() * 0.06 + 0.15;
    Mat4::from_translation(base + Vec3::Y * bob) * Mat4::from_rotation_y(age * 1.5)
}

/// Dropped items spin and bob: a block a quarter of a block across, a sprite half a block
/// (vanilla's ground transforms).
fn ground_transform(base: Vec3, age: f32, solid: bool) -> Mat4 {
    let spin = ground_spin(base, age);
    if solid {
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

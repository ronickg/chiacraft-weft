pub mod camera;
pub mod cloud_shadow;
pub mod ctm;
pub mod entity_render;
pub mod frame_look;
pub mod froxels;
pub mod gizmo;
pub mod lod;
pub mod looks;
pub mod mesher;
pub mod models;
pub mod modui;
pub mod particles;
mod receive;
pub mod sky;
pub mod soft_clouds;
pub mod stack_look;
pub mod terrain;
pub mod terrain_gpu;
pub mod textures;
pub mod weather;
pub mod wires;
pub mod world;
pub mod world_render;

use std::time::Instant;

use crate::gpu_layout::Scene;
use glam::{DVec3, IVec3, Vec2, Vec3};
use rustc_hash::{FxHashMap, FxHashSet};
use weft_abi::{ItemStack, block_flags as bf};
use weft_render::{Backend, BufferId, CommandList, FrameInfo};

use crate::content::content;
use crate::input::Input;
use crate::net::{ClientConn, ToClient, ToServer};
use crate::physics::{self, Hit};
use crate::ui::text::{Arg, Msg};
use crate::world::{GameMode, Pose};
use entity_render::EntityRenderer;
use textures::GameTextures;
use world::ClientWorld;
use world_render::{Camera, WorldRenderer};

/// The snow line when there is none (`snow_line`): blocks above the sea, over any mountain.
const SNOW_LINE_NONE: f32 = 400.0;

pub struct LocalPlayer {
    pub pos: DVec3,
    pub vel: DVec3,
    /// Position at the previous physics tick.
    pub prev: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    /// Touched the ground at any physics step since the last move sent: a quick re-jump can
    /// leave the ground before the next send, and the server would add up the falls.
    pub landed: bool,
    /// Jumped at any physics step since the last move sent.
    pub jumped: bool,
    pub flying: bool,
    /// Spectator: moves through blocks.
    pub noclip: bool,
    /// Flying speed multiplier, changed with the scroll wheel in spectator mode.
    pub fly_speed: f32,
    pub in_liquid: bool,
    /// On a ladder.
    pub climbing: bool,
    /// Physics steps until the next jump while the key stays held (vanilla's 10 ticks), so a
    /// two-block gap cannot be used to bounce at full speed.
    pub jump_cooldown: u32,
    /// Smoothed eye height (lower while sneaking) and field of view multiplier (wider while
    /// sprinting or flying).
    pub eye: f64,
    pub fov_mul: f32,
    /// Lower box and eyes. Stays on under a low ceiling until there is room for more.
    pub pose: Pose,
    /// A pose a mod holds it in, where it fits, and with a facing it stays put that way (a bed).
    pub forced: Option<(Pose, Option<f32>)>,
    /// Actually sprinting, after the conditions: moving forward, not sneaking, not wading.
    pub sprinting: bool,
    /// Holding the use button on an item that takes time.
    pub using: bool,
    /// By liquid: whether sprinting swims in it (`comp::SWIMS_IN` of its kind and what it wears).
    pub swims_in: Vec<bool>,
    pub can_sprint: bool,
    /// The solid body it rides.
    pub support: Option<u64>,
    /// Its box standing (width, height, eye): the player kind's, or what a mod gave it.
    pub size: weft_abi::EntitySize,
    /// Riding a body: its id and the position relative to it at the previous and the last
    /// step. Drawn on the body as shown, so a rider and its deck never drift apart.
    pub riding: Option<(u64, DVec3, DVec3)>,
    /// Walking speed and jump multipliers from the server (attributes).
    pub speed_mul: f64,
    pub jump_mul: f64,
    /// Blocks a second it climbs walls (`attribute::CLIMB_SPEED`) and its glider's air
    /// (`GLIDE_SPEED` over 0 glides), from the server, per tick.
    pub climb: f64,
    pub glide: physics::Glide,
    /// Held on to a wall at the last step, gliding at the last step: the poses for them.
    pub on_wall: bool,
    pub gliding: bool,
    /// The rope it hangs on (`attribute::ROPE_LENGTH`), how much of it is let out now (pulled in
    /// with jump, let out with sneak), the anchor that length is for, and hanging off the ground on
    /// it at the last step.
    pub rope: Option<physics::Rope>,
    pub rope_len: f64,
    pub rope_for: Option<DVec3>,
    pub on_rope: bool,
}

pub struct ClientEntity {
    pub kind: u32,
    /// What it rides and in which seat: it is drawn there.
    pub vehicle: Option<(u64, u8)>,
    pub pos: DVec3,
    pub target: DVec3,
    pub yaw: f32,
    pub target_yaw: f32,
    pub pitch: f32,
    pub item: ItemStack,
    pub health: f32,
    pub hurt: f32,
    pub worn: Vec<u32>,
    /// What it wears drawn by its own looks (`abi::comp::WORN_LOOK`), in `worn`'s order; empty
    /// for an item's own texture.
    pub worn_looks: Vec<String>,
    /// A player's item in hand, 0 for none, and that stack's own look (`abi::comp::LOOK`).
    pub held: u32,
    pub held_look: Option<String>,
    /// And in the other hand.
    pub off_held: u32,
    /// Into using what it holds, 0..1, and whether with the other hand.
    pub using: Option<(f32, bool)>,
    /// Ticks using it takes.
    pub use_ticks: f32,
    /// How far into sneaking, 0..1: the pose fades in and out.
    pub crouched: f32,
    /// Actions a mod has play as another for now: which, as what.
    pub swaps: Vec<(String, String)>,
    /// Its swing is the other hand's.
    pub swing_off: bool,
    /// Accumulated walk distance and current swing amplitude, for limb animation.
    pub walk: f32,
    /// Blocks actually walked, which limbs at full swing fall behind of.
    pub walked: f32,
    pub swing: f32,
    pub age: f32,
    pub burning: bool,
    /// Seconds since it died: it falls over and turns red, then disappears in a puff.
    pub dying: Option<f32>,
    /// Walk distance at which its next step sounds.
    pub next_step: f32,
    /// Action the server started: name, age it started at, looping.
    pub action: Option<(String, f32, bool)>,
    /// How a player holds itself: its box and how the model stands.
    pub pose: Pose,
    pub sprinting: bool,
    /// Where the body faces: follows the head with a lag, and where it walks.
    pub body_yaw: f32,
    /// Arm swing progress 0..1 while swinging.
    pub arm_swing: Option<f32>,
    /// The shader effect on it (`abi::entity_effect`), its strength and seconds left.
    pub effect: Option<(u32, f32, f32)>,
    /// Solid bodies: where the server had them at its last ticks (tick, position), oldest
    /// first; they are shown `INTERP_TICKS` behind the server (`pose`).
    pub track: std::collections::VecDeque<(f64, DVec3)>,
    /// A picked up item flying to who took it: where it started, the taker, seconds so far.
    pub pulled: Option<(DVec3, u64, f32)>,
    /// Its own box (width, height, eye) and the kind it looks like, when a mod set them.
    pub size: Option<weft_abi::EntitySize>,
    pub look: Option<u32>,
    /// How big it is against its kind (`Host::set_entity_scale`): model and box.
    pub scale: f32,
    /// The solid body it stands on (0: the world), and its position relative to that body:
    /// shown and as the server last had it. Riders are drawn where the body is drawn.
    pub frame: u64,
    pub rel: DVec3,
    pub rel_target: Option<DVec3>,
    /// A moving structure's blocks, in its own coordinates (meshed like the world's).
    pub structure: Option<Box<ClientWorld>>,
    /// The world's light on a moving structure, by its cubes, as the server worked it out.
    pub world_light: rustc_hash::FxHashMap<IVec3, u32>,
    /// How a moving structure leans (chiacraft, `ToClient::BodyLean`): only drawn, never
    /// collided with or picked, while it does.
    pub lean: Option<Lean>,
}

/// A moving structure leaning (chiacraft, a falling tree): turned about `pivot` (its own
/// coordinates) from `from` to `to` over `secs`, `t` of them gone.
#[derive(Clone, Copy, Debug)]
pub struct Lean {
    pub pivot: Vec3,
    pub from: glam::Quat,
    pub to: glam::Quat,
    pub secs: f32,
    pub t: f32,
}

impl Lean {
    /// The rotation shown now.
    pub fn now(&self) -> glam::Quat {
        if self.secs <= 0.0 { self.to } else { self.from.slerp(self.to, (self.t / self.secs).min(1.0)) }
    }
}

/// A picked up item reaches the taker's feet in three ticks (vanilla's pickup animation).
const PULL_SECS: f32 = 0.15;

/// Seconds an entity flashes red after a hit (vanilla's 10 ticks).
const HURT_FLASH: f32 = 0.5;

impl LocalPlayer {
    /// Its box now (crouching lowers it).
    pub fn dims(&self) -> physics::Dims {
        physics::posed(self.size, self.pose)
    }
}

/// One step on a mount: steering it when the seat says so and the kind knows how
/// (`EntityKind::steer`), then sitting where the seat is.
#[allow(clippy::too_many_arguments)]
fn ride_step(
    entities: &FxHashMap<u64, ClientEntity>,
    player: &mut LocalPlayer,
    drive: &mut Drive,
    conn: &mut ClientConn,
    input: &Input,
    block: &impl physics::Cells,
    gravity: f64,
    bodies: &[physics::Body],
    mount: u64,
    seat: u8,
) -> Option<(DVec3, f32)> {
    let c = content();
    let m = entities.get(&mount)?;
    let k = &c.game.entities[m.kind as usize];
    let steers = k.seats.get(seat as usize).is_some_and(|s| s.steers);
    let mut moved = None;
    if let (true, Some((owner, cb))) = (steers, k.steer) {
        let axis = |plus: &str, minus: &str| input.held(plus) as i32 as f32 - input.held(minus) as i32 as f32;
        let jump = input.captured && input.held("core:jump");
        let released = if !jump { drive.jump_held } else { 0 };
        drive.jump_held = if jump { drive.jump_held + 1 } else { 0 };
        let held = |a: &str| input.captured && input.held(a);
        let d = m.dims();
        let mut args = weft_abi::SteerArgs {
            input: weft_abi::MountInput {
                forward: if input.captured { axis("core:forward", "core:back") } else { 0.0 },
                strafe: if input.captured { axis("core:right", "core:left") } else { 0.0 },
                up: jump as u32,
                down: held("core:sneak") as u32,
                jump_held: drive.jump_held,
                jump_released: released,
                sprint: held("core:sprint") as u32,
                yaw: player.yaw,
                pitch: player.pitch,
            },
            mount: weft_abi::EntityState {
                kind: m.kind,
                flags: k.flags,
                pos: m.pos.to_array().into(),
                vel: (drive.vel * crate::world::TPS as f64).to_array().into(),
                yaw: m.yaw,
                pitch: m.pitch,
                on_ground: drive.on_ground as u32,
                size: weft_abi::EntitySize { width: d.half_width as f32 * 2.0, height: d.height as f32, eye: d.eye as f32 },
                ..weft_abi::EntityState::DEFAULT
            },
            out: weft_abi::Steer::DEFAULT,
        };
        crate::host::invoke(owner, cb, &mut args);
        let s = args.out;
        let yaw = if s.yaw.is_finite() { s.yaw } else { m.yaw };
        let tick = crate::world::TICK;
        let target = k.speed as f64 * s.speed.max(0.0) as f64 * tick;
        let wish = crate::server::riding::turned(DVec3::new(s.strafe as f64, 0.0, s.forward as f64), yaw);
        let ctl = physics::Control {
            wish,
            // As a mob walks: this much a tick keeps it at its speed on plain ground.
            ground: target * (1.0 - 0.6 * 0.91),
            air: 0.02,
            jump: if s.jump > 0.0 { s.jump as f64 * tick } else { 0.0 },
            swim: k.is(weft_abi::entity_flags::SWIMS),
            ..Default::default()
        };
        let flies = s.flies != 0;
        if flies {
            drive.vel.y = s.rise as f64 * tick;
        }
        let frame = physics::Frame {
            half_width: d.half_width,
            height: d.height,
            step: physics::STEP_UP,
            gravity: if flies || k.is(weft_abi::entity_flags::NO_GRAVITY) { 0.0 } else { physics::gravity_per_tick(gravity) },
        };
        let mut pos = m.pos;
        let r = physics::walk(c, block, bodies, mount, &mut pos, &mut drive.vel, drive.on_ground, &ctl, &frame);
        drive.on_ground = r.on_ground;
        conn.send(ToServer::MountMove { pos, yaw, vel: drive.vel * crate::world::TPS as f64, on_ground: r.on_ground, flies });
        moved = Some((pos, yaw));
    }
    let (pos, yaw) = moved.unwrap_or((m.pos, m.yaw));
    if let Some(at) = seat_at(c, m.kind, pos, yaw, m.scale, seat) {
        (player.pos, player.vel, player.on_ground) = (at, DVec3::ZERO, false);
    }
    moved
}

/// The mount the local player steers, as its client moves it.
#[derive(Default)]
struct Drive {
    /// Blocks per tick.
    vel: DVec3,
    on_ground: bool,
    jump_held: u32,
}

/// Where a rider's feet go in seat `seat` of a mount of `kind` at `pos` facing `yaw`, `scale`
/// times its kind's size (chiacraft).
fn seat_at(c: &crate::content::Content, kind: u32, pos: DVec3, yaw: f32, scale: f32, seat: u8) -> Option<DVec3> {
    let s = c.game.entities[kind as usize].seats.get(seat as usize)?;
    Some(pos + crate::server::riding::turned(s.pos * scale as f64, yaw))
}

/// `u32` and `f32` values of an engine field component (`registry::HEALTH` ...).
fn words(bytes: &[u8]) -> impl Iterator<Item = [u8; 4]> + '_ {
    bytes.chunks_exact(4).map(|w| [w[0], w[1], w[2], w[3]])
}

fn parse_swaps(bytes: &[u8]) -> Vec<(String, String)> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// A pose comes and goes in about a tenth of a second.
fn fade_pose(now: f32, on: bool, dt: f32) -> f32 {
    (now + if on { dt } else { -dt } / 0.1).clamp(0.0, 1.0)
}

impl ClientEntity {
    pub fn new(kind: u32, pos: DVec3, yaw: f32, pitch: f32, item: ItemStack) -> ClientEntity {
        ClientEntity {
            kind,
            pos,
            target: pos,
            yaw,
            target_yaw: yaw,
            body_yaw: yaw,
            arm_swing: None,
            effect: None,
            pitch,
            item,
            // Arrives as a component right after: no hurt flash for the first.
            health: f32::NAN,
            hurt: 0.0,
            worn: Vec::new(),
            worn_looks: Vec::new(),
            held: 0,
            off_held: 0,
            using: None,
            use_ticks: 0.0,
            crouched: 0.0,
            swaps: Vec::new(),
            swing_off: false,
            walk: 0.0,
            walked: 0.0,
            swing: 0.0,
            age: 0.0,
            burning: false,
            dying: None,
            vehicle: None,
            next_step: 0.0,
            action: None,
            pose: Pose::STAND,
            sprinting: false,
            track: Default::default(),
            pulled: None,
            size: None,
            look: None,
            frame: 0,
            rel: DVec3::ZERO,
            rel_target: None,
            structure: None,
            world_light: Default::default(),
            held_look: None,
            scale: 1.0,
            lean: None,
        }
    }

    /// Something that can be bumped into or hit, like the server's `Entity::is_body`: what isn't
    /// a thing to hit never gets here, and the dead lie there only to be seen.
    pub fn is_body(&self) -> bool {
        self.item.is_empty() && self.dying.is_none()
    }

    /// What its kind's client tick gets (`client` filled in by the caller).
    pub fn args(&self, id: u64, dt: f32) -> weft_abi::ClientEntityArgs {
        weft_abi::ClientEntityArgs {
            client: std::ptr::null_mut(),
            entity: id,
            kind: self.kind,
            pos: self.pos.to_array().into(),
            yaw: self.yaw,
            walked: self.walked,
            dying: self.dying.unwrap_or(-1.0),
            age: self.age,
            dt,
        }
    }

    /// Its box, like the server's `Entity::dims`.
    pub fn dims(&self) -> physics::Dims {
        let s = self
            .size
            .unwrap_or(content().game.entities[self.kind as usize].size());
        let k = self.scale;
        physics::posed(weft_abi::EntitySize { width: s.width * k, height: s.height * k, eye: s.eye * k }, self.pose)
    }

    /// The kind whose model, texture and animations draw it.
    pub fn look(&self) -> u32 {
        self.look.unwrap_or(self.kind)
    }

    /// Where a solid body was at server tick `tick` (fractional): between the two positions
    /// around it; past the newest one it goes on as it last moved for at most a tick (the
    /// server sends bodies every tick, so a stop shows as two equal positions, no overshoot).
    pub fn pose(&self, tick: f64) -> DVec3 {
        let n = self.track.len();
        let Some(&(last_t, last)) = self.track.back() else { return self.target };
        if tick >= last_t {
            return match n {
                1 => last,
                _ => {
                    let (pt, prev) = self.track[n - 2];
                    last + (last - prev) / (last_t - pt).max(1.0) * (tick - last_t).min(1.0)
                }
            };
        }
        let mut before = self.track[0];
        for &(t, p) in &self.track {
            if tick < t {
                return if t > before.0 {
                    before
                        .1
                        .lerp(p, ((tick - before.0) / (t - before.0)).max(0.0))
                } else {
                    p
                };
            }
            before = (t, p);
        }
        last
    }

    /// `pose` for physics, which takes only what the server has said. It runs a step ahead of what
    /// is drawn, past the newest position half the time; going on as last moved from there, it
    /// overshoots every stop, and shoves whoever is next to where the body stops.
    pub fn known(&self, tick: f64) -> DVec3 {
        self.pose(self.track.back().map_or(tick, |t| tick.min(t.0)))
    }
}

/// The camera a frame was drawn with, without the jitter (`Client::frame_camera`): what the
/// backend's upscaler is told (`weft_render::UpscaleParams`).
#[derive(Clone, Copy, Debug)]
pub struct FrameCamera {
    /// Camera-relative world to view (the camera at the origin), and view to clip.
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    /// Takes this frame's camera-relative positions to last frame's clip space (`Scene::taa_prev`).
    pub prev: glam::Mat4,
    /// No last frame to go from.
    pub fresh: bool,
    pub fov_y: f32,
    pub aspect: f32,
}

impl FrameCamera {
    /// `weft_render::UpscaleParams`'s camera part; `Post::apply` fills in the rest.
    pub fn upscale_params(&self) -> weft_render::UpscaleParams {
        let view_proj = self.proj * self.view;
        let clip_to_prev = self.prev * view_proj.inverse();
        let to_world = self.view.inverse();
        let axis = |v: glam::Vec3| {
            to_world
                .transform_vector3(v)
                .normalize_or_zero()
                .extend(0.0)
                .to_array()
        };
        weft_render::UpscaleParams {
            reset: self.fresh as u32,
            fov_y: self.fov_y,
            aspect: self.aspect,
            near: 0.05,
            view_to_clip: self.proj.to_cols_array(),
            clip_to_view: self.proj.inverse().to_cols_array(),
            clip_to_prev_clip: clip_to_prev.to_cols_array(),
            prev_clip_to_clip: clip_to_prev.inverse().to_cols_array(),
            world_to_view: self.view.to_cols_array(),
            view_to_world: to_world.to_cols_array(),
            right: axis(glam::Vec3::X),
            up: axis(glam::Vec3::Y),
            // The camera looks down its -z.
            forward: axis(-glam::Vec3::Z),
            ..Default::default()
        }
    }
}

pub struct ClientMenu {
    pub kind: u32,
    pub window: u32,
    /// The entity it belongs to, 0 for none.
    pub owner: u64,
    /// The block state it was opened on (0: none) and the own name of what it belongs to.
    pub block: u32,
    pub name: String,
    pub slots: Vec<ItemStack>,
    pub cursor: ItemStack,
}

pub enum ClientEvent {
    OpenMenu,
    CloseMenu,
    Died,
}

/// Player physics runs at vanilla's tick rate; the camera is interpolated between ticks.
const STEP: f64 = crate::world::TICK;
/// How far behind the newest server tick bodies are shown (Sable, under Create: Aeronautics,
/// uses 1.5): late moves still find two positions to show between.
const INTERP_TICKS: f64 = 1.5;

/// While loading, a frame takes finished terrain and uploads meshes for what is left of this
/// after the rest of the last frame (half each), at least `MIN_BUDGET` each: loading goes as
/// fast as the frame allows, down to about 60 frames a second.
const FRAME_TARGET: f64 = 0.016;
const MIN_BUDGET: f64 = 0.002;
const MAX_BUDGET: f64 = 0.005;

pub struct Client {
    conn: ClientConn,
    pub world: ClientWorld,
    pub render: WorldRenderer,
    pub entity_render: EntityRenderer,
    pub player: LocalPlayer,
    pub joined: bool,
    /// Every block's textures were loaded once after joining (`preload_block_textures`).
    blocks_preloaded: bool,
    pub player_id: u64,
    /// The local player as an entity: what the server says of it (health, actions, look) and
    /// what its own animation works out, brought up to date each frame (`sync_me`).
    pub me: ClientEntity,
    /// The mount the player steers: its velocity, footing and how long jump has been held.
    drive: Drive,
    accum: f64,
    tick_accum: f64,
    /// Seconds of physics steps taken, the clock solid bodies are shown by.
    body_clock: f64,
    /// Client clock (`body_clock`) less server time (ticks x `STEP`): maps the client's clock
    /// onto the server's timeline bodies are shown on. It follows `offset_target` (arrival
    /// times, averaged) by stretching time at most 5%, so that timeline only ever goes on
    /// evenly: following each arrival's jitter would step bodies back and forth.
    tick_offset: Option<f64>,
    offset_target: f64,
    /// Structures whose cubes are uploaded for drawing.
    drawn_frames: FxHashSet<u64>,
    start: Instant,
    /// How fast the time of day runs (`ToClient::Time`): the sky between the server's seconds.
    day_rate: f32,
    since_time: f64,
    /// Ticks a second the server runs at.
    tick_rate: f64,
    /// Particle kind per block state shown around the player (`ambient`), `u32::MAX` none.
    ambient_kinds: Option<Vec<(Vec<u32>, Option<u32>)>>,
    /// The frame's `Scene`, one buffer per frame in flight.
    scene_bufs: Vec<BufferId>,
    /// Sun shadows (`sun_shadows` packs): the shadow map, the shadow pass's scene per frame, and
    /// this frame's light matrix with whether the map holds it.
    shadow_map: Option<weft_render::TextureId>,
    shadow_bufs: Vec<BufferId>,
    shadow: (glam::Mat4, bool),
    shadow_index: u32,
    /// The far cascade: map, light, scene buffers and bindless index, as the near one's.
    shadow_far_map: Option<weft_render::TextureId>,
    shadow_far: (glam::Mat4, bool),
    shadow_far_bufs: Vec<BufferId>,
    shadow_far_index: u32,
    /// What the far map holds: its light about its anchor and the terrain's `terrain_epoch` when it
    /// was drawn. It is drawn again only when either moved (`render_shadows`).
    shadow_far_drawn: Option<([f32; 16], u64)>,
    /// The far map's light about its anchor (`prepare_shadows`): a place in the world snapped to
    /// `FAR_ANCHOR`, so walking does not move it; `shadow_far` is it seen from the camera.
    shadow_far_anchored: glam::Mat4,
    /// Frames since the far map was last drawn (terrain that changed waits a little).
    shadow_far_age: u32,
    /// The maps' size in texels (`core:shadow_size`), as they were made.
    shadow_size: u32,
    /// The far shadow map's reach to each side (blocks, `core:shadow_far`; 400 as made).
    shadow_far_reach: f32,
    pub target: Option<Hit>,
    pub target_entity: Option<u64>,
    pub selected: usize,
    pub disconnected: Option<String>,
    /// Why the world refused this player or sent them away (`ToClient::Refused`), to show in
    /// their language.
    pub refused: Option<crate::ui::text::Msg>,
    /// Seconds until holding a button repeats its action. A new press never waits.
    break_delay: f32,
    use_delay: f32,
    pub view_radius: i32,
    /// Makes the world in view from the seed (`terrain`); the server's cubes go through it.
    pub terrain: terrain::TerrainHandle,
    pub gpu_terrain: terrain_gpu::GpuTerrain,
    /// Distant terrain beyond the view.
    lod: lod::Lod,
    seed: u64,
    /// Resets sent to the terrain thread: batches of an older one are dropped.
    terrain_resets: u64,
    terrain_view: (IVec3, i32),
    /// The rest of a batch from the terrain thread cut at the frame's budget.
    terrain_left: Option<terrain::Batch>,
    /// Time each of taking terrain and uploading meshes may take this frame (`FRAME_TARGET`),
    /// and how long they took in the last one.
    budget: std::time::Duration,
    spent: f64,
    /// The rest of a frame (seconds), smoothed.
    rest: f64,
    /// Finished meshes not uploaded yet (the frame's budget ran out).
    meshes: std::collections::VecDeque<world::MeshResult>,
    pub fov: f32,
    /// Brightness setting, 0 moody .. 1 bright.
    pub brightness: f32,
    pub entities: FxHashMap<u64, ClientEntity>,
    pub inventory: Vec<ItemStack>,
    pub max_health: f32,
    pub mode: GameMode,
    /// When jump was last pressed, for double-tap flight toggling.
    /// How sneak, sprint and flight are being asked for.
    /// The server ended the sprint; `update` switches the key off.
    sprint_stopped: bool,
    /// 0 first person, 1 third person from behind, 2 from the front.
    pub cameras: camera::Cameras,
    /// F1: no hand and no block outline.
    pub hide_gui: bool,
    /// Show your own body in first person instead of just the hand.
    pub first_person_model: bool,
    /// Arm swing progress 0..1 while swinging.
    hand_swing: Option<f32>,
    /// That swing is the other hand's.
    hand_swing_off: bool,
    using_off: bool,
    /// The stack the hand shows: a new one only comes up after the old one went down.
    hand_stack: ItemStack,
    /// How far the hand is up, 0..1.
    hand_height: f32,
    /// Seconds since the last swing or change of items, and what was in hand then, as the server
    /// judges a hit by.
    swing_since: f32,
    swing_held: ItemStack,
    /// This player's attributes, by id, as the server says.
    pub(crate) attributes: Vec<f32>,
    /// What the server refused to have mined; the button let go or another block lifts it.
    refused_break: Option<(u64, IVec3)>,
    /// Where the hand's view is, easing after the camera: pitch, yaw.
    hand_view: (f32, f32),
    /// Vanilla's bobbing state: distance walked (x0.6) and the smoothed amplitude.
    walk_dist: f32,
    bob: f32,
    last_smooth: DVec3,
    /// What that was relative to: the body it rode, if any.
    last_frame: Option<u64>,
    /// How long the move since `last_smooth` took.
    moved_over: f32,
    /// Held item and when it became held: its name shows above the hotbar for a moment.
    pub held_name: (ItemStack, Instant),
    /// Names of the players online, for completion.
    pub players: Vec<String>,
    /// Lines this player sent, oldest first (chat history).
    pub sent: Vec<String>,
    /// Mouse button and slots of a drag in progress in a menu.
    pub drag: Option<(u8, Vec<u32>)>,
    /// Slots shift-clicked by sweeping over them with shift and the left button held.
    pub sweep: Vec<u32>,
    /// The last slot pressed and when, for double clicks.
    pub last_click: Option<(u32, u8, Instant)>,
    pub particles: particles::Particles,
    pub wires: wires::Wires,
    pub weather_render: weather::Weather,
    pub sky: sky::Sky,
    /// Scratch: taken items and where they fly to this frame.
    pulls: Vec<(u64, DVec3)>,
    /// Scratch: solid bodies and where they are drawn this frame.
    frames: Vec<(u64, DVec3)>,
    /// Commands recorded early but drawn after the opaque ones: translucent blocks and blended
    /// particles, then cracks, gizmo fills and the hand.
    late: CommandList,
    hand_late: CommandList,
    /// The world's copy for what is seen through (`gpu_layout::Scene::behind`), set by the app each
    /// frame; and with it on, what is drawn after the opaque world goes into `late_out` for the app
    /// to put after the copy instead of into the frame's list.
    pub behind: [f32; 4],
    /// Last frame's projection and view without the jitter, where its camera stood and the
    /// shaders' clock then, for temporal anti-aliasing's `Scene::taa_prev` and `taa`.
    taa_last: Option<(glam::Mat4, DVec3, f32)>,
    /// Temporal anti-aliasing's count image this frame, bindless index + 1 (`Scene::taa_frame`),
    /// set by the app before `render` (`post::Post::taa_out`).
    pub taa_out: u32,
    /// The camera the last frame was drawn with, for the backend's upscaler (`upscale_camera`).
    pub frame_camera: Option<FrameCamera>,
    pub late_out: CommandList,
    /// Seconds until the next dust piece from the block being hit, and until its next knock.
    hit_dust: f32,
    hit_sound: f32,
    /// Sounds to play this frame (from the server and from here).
    pub sounds: Vec<crate::audio::Play>,
    /// Own walk distance at which the next step sounds.
    next_step: f32,
    rng: weft_noise::Rng,
    pub dead: bool,
    /// The weather the server says it is (`registry::WeatherDef`).
    pub weather_id: u16,
    /// The fog (0..1) and the wind (m/s, x and z, where it blows to) where the player is, eased
    /// over a few seconds (`ToClient::Air`).
    fog: f32,
    wind: Vec2,
    /// Snow lying where the player is (0..1), and how much of what falls there is snow and how
    /// much dust blows, all eased (`air_flags`).
    snow_cover: f32,
    snowing: f32,
    dust: f32,
    /// How strong an aurora is where the player is (a mod's air, `air_flags::AURORA_SHIFT`).
    aurora: f32,
    /// How closed the canopy is where the player is (a mod's air, `air_flags::CANOPY_SHIFT`).
    canopy: f32,
    /// How wet the ground is where the player is, 0..1 (`weather_effects`, `Scene::canopy.y`).
    wet: f32,
    /// How far puddles have grown where the player is, 0..1 (`weather_effects`, `Scene::canopy.z`):
    /// over minutes of rain, drying slower than the ground after.
    puddles: f32,
    /// Where rain turns to snow up a mountain where the player is, blocks above the sea (a mod's
    /// air, `air_flags::SNOWLINE_SHIFT`; `SNOW_LINE_NONE` for none), eased (`Scene::canopy.w`).
    snow_line: f32,
    /// What happens in the sky (`weft_abi::sky_event`, chiacraft), how strongly, and when it began
    /// (`shaders::time`).
    sky_event: (u32, f32, f32),
    /// What is falling from the sky: where it lands, and when (`shaders::time`).
    sky_landing: Option<(DVec3, f32)>,
    /// The weather's look as it is by now, going over to the weather's own (`weather_effects`).
    look: weft_abi::WeatherLook,
    /// How far the weather's sky has come in where the player is, 0..1 (`weather_effects`): it
    /// gathers over `GLOOM_SECS` before anything falls.
    gloom: f32,
    frame_dt: f32,
    /// Seconds left of a lightning's flash.
    lightning: f32,
    /// Lightning flashes seen so far: mods hear a new one as thunder.
    lightnings: u32,
    /// The last flash for packs (`Scene::lightning`): seconds since it began (real time, so tests'
    /// pinned shader clock still shows it), and a random number of its own (where it strikes).
    lightning_seen: (f32, f32, f32),
    /// Holding the use button on an item that takes time: ticks left, ticks in all.
    using: Option<(f32, f32)>,
    /// Who else is mining what, and how far along.
    others_mining: Vec<crate::net::Crack>,
    /// What the server was last told this player mines.
    told_mining: Option<(u64, IVec3, u32)>,
    /// What the server says this player may do (`abi::permission`).
    pub permission: u32,
    /// Seconds since it died, while it is.
    dead_for: Option<f32>,
    pub components: FxHashMap<u32, Vec<u8>>,
    pub menu: Option<ClientMenu>,
    pub events: Vec<ClientEvent>,
    /// Mod channel messages waiting for their client handler.
    pub packets: Vec<(u32, Vec<u8>)>,
    /// Components of other entities the server shares.
    pub entity_components: FxHashMap<(u64, u32), Vec<u8>>,
    /// Formatted when drawn, in the language shown then.
    pub chat: Vec<(crate::net::Shown, Instant)>,
    /// Text on the screen other than the chat, some of it possibly over by now.
    pub shown: Vec<(crate::net::Shown, Instant)>,
    /// Goes up with every change to `chat` and `shown`, for the HUD to notice.
    pub shown_changes: u32,
    /// Answers to commands run from code, by their id, until read.
    pub command_replies: rustc_hash::FxHashMap<u32, Vec<Msg>>,
    pub next_command: u32,
    pub gizmos: gizmo::Gizmos,
    /// This frame's gizmo shapes, from `collect_gizmos`.
    pub gizmo_lines: Vec<gizmo::Line>,
    pub gizmo_fills: Vec<gizmo::Fill>,
    pub gizmo_labels: Vec<gizmo::Label>,
    /// The block being broken (frame, block as in `Hit`) and how far.
    pub breaking: Option<(u64, IVec3, f32)>,
    /// Which of what is in that cell.
    breaking_part: u32,
    /// Hotbar contents last frame, so the HUD knows when to redraw.
    pub hotbar: Vec<u32>,
}

/// The player's position, sent at most once a frame ahead of an action: the server judges actions
/// by where the player looks, which it otherwise hears only 20 times a second.
struct LookOnce(Option<ToServer>);

impl LookOnce {
    fn send(&mut self, conn: &mut ClientConn) {
        if let Some(m) = self.0.take() {
            conn.send(m);
        }
    }
}

impl Client {
    pub fn toggle_gizmo(&mut self, group: &str) -> bool {
        let on = self.gizmos.toggle(group);
        self.send(ToServer::Gizmos(self.gizmos.enabled.clone()));
        on
    }

    /// `/gizmo [group]`: lists groups or switches one; mods' groups are `modid:name`.
    pub fn gizmo_command(&mut self, group: &str) -> Vec<Msg> {
        let text = |s: &str| Arg::Text(s.to_string());
        if group.is_empty() {
            let mut out = vec![Msg::new("command.gizmo.groups", Vec::new())];
            out.extend(
                gizmo::BUILTIN
                    .iter()
                    .map(|g| Msg::new("command.gizmo.group", vec![text(g), Arg::Key(format!("gizmo.{g}"))])),
            );
            out.push(match self.gizmos.enabled.is_empty() {
                true => Msg::new("command.gizmo.none_on", Vec::new()),
                false => Msg::new("command.gizmo.on_list", vec![text(&self.gizmos.enabled.join(", "))]),
            });
            return out;
        }
        let on = self.toggle_gizmo(group);
        vec![Msg::new(if on { "command.gizmo.on" } else { "command.gizmo.off" }, vec![text(group)])]
    }

    /// Builds the gizmo lines and labels for this frame; labels behind opaque blocks are left out.
    pub fn collect_gizmos(&mut self) {
        let mut g = std::mem::take(&mut self.gizmos);
        let (lines, fills, mut labels) = g.collect(self);
        self.gizmos = g;
        let c = content();
        let eye = self.render_camera().pos;
        let opaque = |p: IVec3| {
            self.world
                .block_or_air(p)
                .map(|s| if c.states[s as usize].flags & bf::OPAQUE != 0 { s } else { crate::content::AIR })
        };
        labels.retain(|(pos, ..)| {
            let d = *pos - eye;
            let len = d.length();
            len < 24.0 && (len < 0.2 || physics::raycast(c, &opaque, eye, d / len, len - 0.1, false).is_none())
        });
        (self.gizmo_lines, self.gizmo_fills, self.gizmo_labels) = (lines, fills, labels);
    }

    /// Shows a line in the chat as if the server had sent it.
    pub fn chat_line(&mut self, line: Msg) {
        self.show(crate::net::Shown::chat(line));
    }

    /// Puts text on the screen. The chat keeps what is over as its history.
    pub fn show(&mut self, s: crate::net::Shown) {
        let now = Instant::now();
        let s = s.settled();
        self.shown_changes += 1;
        let in_chat = s.place == weft_abi::show_place::CHAT && s.at.is_none();
        let list = if in_chat { &mut self.chat } else { &mut self.shown };
        let replaced = (!s.look().stacks || !s.id.is_empty())
            .then(|| {
                list.iter()
                    .position(|(o, _)| o.place == s.place && o.id == s.id)
            })
            .flatten();
        match replaced {
            Some(i) if s.clears() => drop(list.remove(i)),
            Some(i) => list[i] = (s, now),
            None if s.clears() => {}
            None => list.push((s, now)),
        }
        if in_chat {
            let extra = list.len().saturating_sub(100);
            list.drain(..extra);
        } else {
            // Not by alpha: that is 0 too at the start of a fade in.
            list.retain(|(o, at)| now.duration_since(*at).as_secs_f32() < o.times.iter().sum::<f32>());
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gpu: &mut dyn Backend,
        assets: &crate::assets::Assets,
        textures: &GameTextures,
        mut conn: ClientConn,
        name: &str,
        key: &str,
        view_radius: i32,
        skin: Option<Vec<u8>>,
        avatar: Option<crate::net::AvatarData>,
    ) -> Result<Client, String> {
        let c = content();
        let render = WorldRenderer::new(gpu, c, textures, assets)?;
        let entity_render = EntityRenderer::new(gpu, assets)?;
        let particles = particles::Particles::new(gpu, assets);
        let wires = wires::Wires::new(gpu, assets);
        let weather_render = weather::Weather::new(gpu, assets);
        let sky = sky::Sky::new(gpu, assets);
        let account = chiacraft_account::current()
            .and_then(|s| s.vouch_line(key))
            .unwrap_or_default();
        let did = chiacraft_account::current()
            .and_then(|s| s.did_proof())
            .unwrap_or_default();
        log::info!("joining as '{name}'");
        conn.send(ToServer::Join { name: name.to_string(), key: key.to_string(), account, did });
        conn.send(ToServer::View(view_radius as u32));
        if let Some(png) = skin {
            conn.send(ToServer::Skin(png));
        }
        if let Some(a) = avatar {
            conn.send(ToServer::Avatar(a));
        }
        let (gpu_tx, gpu_rx) = crossbeam_channel::unbounded();
        let gpu_on = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let gpu_terrain = terrain_gpu::GpuTerrain::new(gpu, assets, gpu_rx, gpu_on.clone());
        Ok(Client {
            weather_render,
            frame_dt: 0.0,
            conn,
            world: ClientWorld::new(0),
            render,
            entity_render,
            player: LocalPlayer {
                pos: DVec3::new(0.0, 80.0, 0.0),
                vel: DVec3::ZERO,
                prev: DVec3::new(0.0, 80.0, 0.0),
                yaw: 0.0,
                pitch: 0.0,
                on_ground: false,
                landed: false,
                jumped: false,
                jump_cooldown: 0,
                eye: 1.62,
                fov_mul: 1.0,
                pose: Pose::STAND,
                forced: None,
                sprinting: false,
                using: false,
                swims_in: Vec::new(),
                can_sprint: true,
                support: None,
                size: c.game.entities[c.game.player_kind as usize].size(),
                riding: None,
                flying: false,
                noclip: false,
                fly_speed: 1.0,
                in_liquid: false,
                climbing: false,
                speed_mul: 1.0,
                jump_mul: 1.0,
                climb: 0.0,
                glide: physics::Glide::default(),
                on_wall: false,
                gliding: false,
                rope: None,
                rope_len: 0.0,
                rope_for: None,
                on_rope: false,
            },
            joined: false,
            blocks_preloaded: false,
            player_id: 0,
            sprint_stopped: false,
            me: ClientEntity::new(content().game.player_kind, DVec3::ZERO, 0.0, 0.0, ItemStack::EMPTY),
            drive: Drive::default(),
            accum: 0.0,
            tick_accum: 0.0,
            body_clock: 0.0,
            tick_offset: None,
            offset_target: 0.0,
            drawn_frames: FxHashSet::default(),
            start: Instant::now(),
            day_rate: 1.0,
            since_time: 0.0,
            tick_rate: crate::world::TPS as f64,
            ambient_kinds: None,
            scene_bufs: Vec::new(),
            shadow_map: None,
            shadow_bufs: Vec::new(),
            shadow: (glam::Mat4::IDENTITY, false),
            shadow_index: 0,
            shadow_far_map: None,
            shadow_far: (glam::Mat4::IDENTITY, false),
            shadow_far_bufs: Vec::new(),
            shadow_far_index: 0,
            shadow_far_drawn: None,
            shadow_far_anchored: glam::Mat4::IDENTITY,
            shadow_far_age: 0,
            shadow_size: 2048,
            shadow_far_reach: FAR_REACH,
            target: None,
            target_entity: None,
            selected: 0,
            disconnected: None,
            refused: None,
            break_delay: 0.0,
            use_delay: 0.0,
            view_radius,
            terrain: terrain::spawn(gpu_tx, gpu_on.clone()),
            lod: lod::Lod::new(gpu_terrain.lod_tx.clone()),
            gpu_terrain,
            seed: 0,
            terrain_resets: 0,
            terrain_view: (IVec3::MAX, 0),
            meshes: std::collections::VecDeque::new(),
            terrain_left: None,
            budget: std::time::Duration::from_secs_f64(MIN_BUDGET),
            spent: 0.0,
            rest: 0.0,
            fov: 70.0,
            brightness: 0.5,
            entities: FxHashMap::default(),
            inventory: vec![ItemStack::EMPTY; content().game.player_layout.slots as usize],
            max_health: 20.0,
            particles,
            wires,
            sky,
            pulls: Vec::new(),
            frames: Vec::new(),
            late: CommandList::default(),
            hand_late: CommandList::default(),
            behind: [0.0; 4],
            taa_last: None,
            taa_out: 0,
            frame_camera: None,
            late_out: CommandList::default(),
            hit_dust: 0.0,
            hit_sound: 0.0,
            sounds: Vec::new(),
            next_step: 0.0,
            rng: weft_noise::Rng(0x9E37_79B9_7F4A_7C15),
            drag: None,
            sweep: Vec::new(),
            last_click: None,
            players: Vec::new(),
            sent: Vec::new(),
            held_name: (ItemStack::EMPTY, Instant::now()),
            mode: GameMode::SURVIVAL,
            cameras: camera::Cameras::default(),
            hide_gui: false,
            first_person_model: true,
            hand_swing: None,
            hand_swing_off: false,
            using_off: false,
            hand_stack: ItemStack::default(),
            hand_height: 1.0,
            swing_since: 1000.0,
            swing_held: ItemStack::default(),
            attributes: content()
                .game
                .attributes
                .iter()
                .map(|a| a.default as f32)
                .collect(),
            refused_break: None,
            hand_view: (0.0, 0.0),
            walk_dist: 0.0,
            bob: 0.0,
            last_smooth: DVec3::ZERO,
            last_frame: None,
            moved_over: 0.0,
            dead: false,
            permission: 0,
            fog: 0.0,
            wind: Vec2::ZERO,
            snow_cover: 0.0,
            snowing: 0.0,
            dust: 0.0,
            aurora: 0.0,
            canopy: 0.0,
            wet: 0.0,
            puddles: 0.0,
            snow_line: SNOW_LINE_NONE,
            sky_event: (0, 0.0, 0.0),
            sky_landing: None,
            weather_id: 0,
            look: Default::default(),
            gloom: 0.0,
            lightning: 0.0,
            lightnings: 0,
            lightning_seen: (1.0e4, 0.0, 0.0),
            using: None,
            others_mining: Vec::new(),
            told_mining: None,
            dead_for: None,
            components: FxHashMap::default(),
            menu: None,
            events: Vec::new(),
            packets: Vec::new(),
            entity_components: FxHashMap::default(),
            chat: Vec::new(),
            shown: Vec::new(),
            shown_changes: 0,
            command_replies: Default::default(),
            next_command: 0,
            gizmos: Default::default(),
            gizmo_lines: Vec::new(),
            gizmo_fills: Vec::new(),
            gizmo_labels: Vec::new(),
            breaking: None,
            breaking_part: 0,
            hotbar: vec![0; 9],
        })
    }

    pub fn send(&mut self, m: ToServer) {
        self.conn.send(m);
    }

    pub fn leave(&mut self) {
        self.conn.send(ToServer::Leave);
    }

    /// The player's health, full until the server has said.
    pub fn health(&self) -> f32 {
        if self.me.health.is_nan() { self.max_health } else { self.me.health }
    }

    fn attribute(&self, id: u32) -> f32 {
        self.attributes.get(id as usize).copied().unwrap_or(1.0)
    }

    /// How much walking shakes the view now.
    pub fn bob(&self) -> f32 {
        self.bob
    }

    /// How charged the next swing is, 0..1, as the server will judge it.
    pub fn swing_strength(&self) -> f32 {
        crate::content::swing_charge(
            content(),
            &self.swing_held,
            self.swing_since * self.tick_rate as f32,
            self.attribute(crate::server::extend::ATTACK_SPEED),
        )
    }

    pub fn held(&self) -> ItemStack {
        crate::content::in_main_hand(content(), &self.inventory, self.selected as u32)
    }

    /// What the main hand shows: from the eyes the old stack, until the hand has gone down with it.
    pub fn shown_held(&self) -> ItemStack {
        if self.camera_has(weft_abi::camera_flags::HAND) { self.hand_stack } else { self.held() }
    }

    pub fn off_held(&self) -> ItemStack {
        crate::content::in_off_hand(content(), &self.inventory)
    }

    /// Starts the terrain over for dimension `dim` (joined, or moved to another dimension).
    fn reset_terrain(&mut self, dim: u32, fwd: &mut Vec<terrain::In>) {
        self.world.clear();
        self.wires.clear();
        self.meshes.clear();
        self.world.dim = dim;
        self.terrain_resets += 1;
        self.terrain_view = (IVec3::MAX, 0);
        fwd.push(terrain::In::Reset { dim, seed: self.seed });
    }

    /// Distant terrain: tiles shown, made, their quads, asked for and not back.
    pub fn lod_stats(&self) -> (usize, usize, usize, usize) {
        let (made, quads) = self.render.lod_counts();
        (self.render.lod_draw.len(), made, quads, self.lod.pending())
    }

    /// Meshes still to come or to upload.
    pub fn pending_meshes(&self) -> usize {
        self.world.pending_meshes() + self.meshes.len()
    }

    /// The key that swaps what is in the two hands.
    fn swap_hands(&mut self, input: &Input) {
        if input.captured && !self.mode.has(weft_abi::game_mode_flags::NO_INTERACT) && input.pressed("core:swap_hands") {
            self.conn.send(ToServer::SwapHands);
        }
    }

    pub fn close_menu(&mut self) {
        self.drag = None;
        if self.menu.take().is_some() {
            self.conn.send(ToServer::CloseMenu);
        }
    }

    pub fn update(&mut self, gpu: &mut dyn Backend, textures: &GameTextures, input: &mut Input, dt: f64) {
        // The server ended the sprint (no food, say): a toggle or a double tap has to be pressed again.
        if std::mem::take(&mut self.sprint_stopped) {
            input.stop("core:sprint");
        }
        let m = self.camera_mode();
        input.mute_moves = m.has(weft_abi::camera_flags::DETACHED);
        input.mute_hands = m.has(weft_abi::camera_flags::NO_INTERACT);
        let input = &*input;
        crate::diag::end_parts();
        let p = crate::diag::part("messages and terrain");
        self.take_messages(dt);
        drop(p);
        self.read_look(input);
        self.read_controls(input);
        self.fly_speed(input);
        self.select_slot(input);
        self.ease_camera(input, dt);
        let moved = self.animate_player(input, dt);
        self.body_particles(dt as f32);
        self.frame_dt = dt as f32;
        self.weather_effects(dt as f32);
        self.sync_me();
        if input.pressed("core:drop") && !self.mode.has(weft_abi::game_mode_flags::NO_INTERACT) {
            self.conn
                .send(ToServer::Drop { all: input.held("core:sprint") });
        }
        if input.pressed("core:inventory") && input.captured {
            self.conn.send(ToServer::OpenInventory);
        }
        let p = crate::diag::part("physics, camera, act");
        self.step_physics(input, dt);
        self.update_camera(input, dt as f32);
        let look = self.aim(input);
        self.act(input, dt, look);
        self.send_move(dt);
        drop(p);
        let p = crate::diag::part("entities");
        self.animate_entities(dt);
        drop(p);
        self.chip_block(dt);
        let p = crate::diag::part("particles");
        self.particles.update(dt as f32);
        drop(p);
        self.creature_sounds(dt as f32, moved);
        self.me.hurt = (self.me.hurt - dt as f32).max(0.0);
        self.dead_for = self.dead.then(|| self.dead_for.unwrap_or(0.0) + dt as f32);
        self.hotbar = self.inventory[..content().game.player_layout.hotbar as usize]
            .iter()
            .map(|s| if s.is_empty() { 0 } else { s.item | (s.count.min(255) << 24) })
            .collect();

        self.upload_meshes(gpu, textures);
    }

    fn take_messages(&mut self, dt: f64) {
        // Smoothed: from one frame alone, a quick frame gave the next a long budget, which made
        // it slow, and so on.
        self.rest += (dt - self.spent - self.rest) * 0.1;
        self.budget = std::time::Duration::from_secs_f64(((FRAME_TARGET - self.rest) / 2.0).clamp(MIN_BUDGET, MAX_BUDGET));
        let t = Instant::now();
        self.receive();
        self.spent = t.elapsed().as_secs_f64();
        if let Some(o) = &mut self.tick_offset {
            *o += (self.offset_target - *o).clamp(-0.05 * dt, 0.05 * dt);
        }
    }

    fn read_look(&mut self, input: &Input) {
        if input.captured && !input.mute_moves {
            let sens = 0.0022 * input.sensitivity;
            self.player.yaw -= input.mouse_delta.x * sens;
            let look = input.mouse_delta.y * sens * if input.invert_y { -1.0 } else { 1.0 };
            self.player.pitch = (self.player.pitch - look).clamp(-1.55, 1.55);
        }
    }

    /// Flying goes on and off where the game mode lets it; on a mount the dismount key gets off.
    fn read_controls(&mut self, input: &Input) {
        if self.me.vehicle.is_some() && input.captured && input.pressed("core:dismount") {
            self.conn.send(ToServer::Dismount);
        }
        if self.mode.has(weft_abi::game_mode_flags::CAN_FLY) && input.captured && input.pressed("core:fly") {
            self.player.flying = !self.player.flying;
        }
    }

    /// A spectator's scroll wheel sets how fast it flies.
    fn fly_speed(&mut self, input: &Input) {
        if input.scroll != 0.0 && self.mode.has(weft_abi::game_mode_flags::NO_INTERACT) {
            self.player.fly_speed = (self.player.fly_speed * if input.scroll > 0.0 { 1.25 } else { 0.8 }).clamp(0.1, 10.0);
        }
    }

    fn select_slot(&mut self, input: &Input) {
        if self.mode.has(weft_abi::game_mode_flags::NO_INTERACT) {
            return;
        }
        let old = self.selected;
        // In a screen the number keys belong to it (swapping slots with the hotbar), and with F3
        // down to the debug groups.
        if input.captured && !input.held("core:debug") {
            for (i, action) in crate::input::HOTBAR
                .into_iter()
                .enumerate()
                .take(content().game.player_layout.hotbar as usize)
            {
                if input.pressed(action) {
                    self.selected = i;
                }
            }
        }
        if input.scroll != 0.0 {
            self.selected = (self.selected as i32 - input.scroll.signum() as i32)
                .rem_euclid(content().game.player_layout.hotbar.max(1) as i32) as usize;
        }
        if old != self.selected {
            self.conn.send(ToServer::Select(self.selected as u32));
        }
        let held = self
            .inventory
            .get(self.selected)
            .copied()
            .unwrap_or_default();
        if (held.item, held.data) != (self.held_name.0.item, self.held_name.0.data) || old != self.selected {
            self.held_name = (held, Instant::now());
        }
    }

    /// Eye and view widen or drop over a few frames like vanilla, not in one jump.
    fn ease_camera(&mut self, input: &Input, dt: f64) {
        let k = (dt * 12.0).min(1.0);
        let sneaking = input.held("core:sneak") && !self.player.flying;
        let eye_target = self.player.dims().eye;
        self.player.eye += (eye_target - self.player.eye) * k;
        let moving = input.held("core:forward");
        let fov_target =
            if self.player.flying { 1.1 } else { 1.0 } + if input.held("core:sprint") && moving && !sneaking { 0.15 } else { 0.0 };
        self.player.fov_mul += (fov_target - self.player.fov_mul) * k as f32;
        self.camera_keys(input);
    }

    /// The player's body, the camera's bob and the hand in first person. Returns how far it
    /// walked.
    fn animate_player(&mut self, input: &Input, dt: f64) -> f32 {
        let (moved, speed) = self.animate_walk(dt);
        self.animate_bob(moved, speed, dt);
        self.animate_hand(input, dt);
        moved
    }

    /// Walking (limbs, turning, steps) is motion in the frame of what the player stands on.
    /// Returns how far it walked and how fast.
    fn animate_walk(&mut self, dt: f64) -> (f32, f32) {
        let t = (self.accum / STEP).clamp(0.0, 1.0);
        // Walking is how its place changes on what it stands on, not how that goes.
        let (now, frame) = match self.player.riding {
            Some((b, before, rel)) => (before.lerp(rel, t), Some(b)),
            None => (self.smooth_pos(), None),
        };
        // Getting on or off isn't a step.
        let walked_by = if frame == self.last_frame { now - self.last_smooth } else { DVec3::ZERO };
        let moved = walked_by.with_y(0.0).length().min(2.0) as f32;
        (self.last_smooth, self.last_frame) = (now, frame);
        self.me.body_yaw = turn_body(self.me.body_yaw, self.player.yaw, walked_by, dt as f32);
        // That move is the last frame's: physics runs after this. By this frame's time, uneven frames
        // made the speed jump up and down and the bob settle far too low.
        let speed = moved / std::mem::replace(&mut self.moved_over, dt as f32).max(1e-4);
        // Limbs swing with horizontal speed in the air too, like vanilla.
        crate::model::walk_limbs(&mut self.me.walk, &mut self.me.swing, speed, dt as f32);
        self.me.crouched = fade_pose(self.me.crouched, self.player.pose == Pose::CROUCH, dt as f32);
        (moved, speed)
    }

    fn animate_bob(&mut self, moved: f32, speed: f32, dt: f64) {
        // Vanilla's: swimming doesn't bob, even touching the bottom.
        let walking = self.player.on_ground && !self.player.flying && self.player.pose.def().bob && !self.dead;
        // Vanilla: amplitude follows the per-tick speed capped at 0.1, eased by 0.4 per tick.
        self.walk_dist += moved * 0.6;
        let target = if walking { (speed * crate::world::TICK as f32).min(0.1) } else { 0.0 };
        self.bob += (target - self.bob) * (1.0 - 0.6f32.powf(dt as f32 * crate::world::TPS as f32));
        // It eases forever otherwise, and a still picture should be still.
        if target == 0.0 && self.bob < 1e-5 {
            self.bob = 0.0;
        }
    }

    /// Swings, the held item going down and coming up when it changes, and the hand lagging
    /// behind the view.
    fn animate_hand(&mut self, input: &Input, dt: f64) {
        let spectator = self.mode.has(weft_abi::game_mode_flags::NO_INTERACT);
        advance_swing(&mut self.hand_swing, dt as f32);
        self.swap_hands(input);
        // A use swings only when the server says it did something (`ToClient::Swing`).
        if input.captured && !spectator && (input.pressed("core:attack") || (input.held("core:attack") && self.breaking.is_some())) {
            self.swing_arm();
        }
        // Vanilla's 0.4 per tick: the old item goes down, the new one comes up. More of the same
        // item (a count or damage change) is not a change.
        let held = self.held();
        let same = crate::content::same_in_hand(held, self.hand_stack);
        self.swing_since += dt as f32;
        if !crate::content::same_in_hand(held, self.swing_held) {
            self.swing_since = 0.0;
        }
        self.swing_held = held;
        self.player.using = self.using.is_some();
        let step = dt as f32 * 0.4 * crate::world::TPS as f32;
        // The hand dips after a swing and comes back up as it charges (vanilla's strength cubed).
        let up = if same { self.swing_strength().powi(3) } else { 0.0 };
        self.hand_height += (up - self.hand_height).clamp(-step, step);
        if same || self.hand_height < 0.1 {
            self.hand_stack = held;
        }
        let k_lag = 1.0 - 0.5f32.powf(dt as f32 * crate::world::TPS as f32);
        self.hand_view.0 += (self.player.pitch - self.hand_view.0) * k_lag;
        self.hand_view.1 += angle_diff(self.player.yaw, self.hand_view.1) * k_lag;
        // Both ease forever otherwise, and a still picture should be still.
        if (self.player.pitch - self.hand_view.0).abs() < 1e-5 {
            self.hand_view.0 = self.player.pitch;
        }
        if angle_diff(self.player.yaw, self.hand_view.1).abs() < 1e-5 {
            self.hand_view.1 = self.player.yaw;
        }
    }

    fn step_physics(&mut self, input: &Input, dt: f64) {
        let c = content();
        let world = &self.world;
        let block = world::SeenCells { world, air_if_missing: false };
        // Walking waits for the ground under the player to arrive; flying goes on, taking cubes
        // that are not here yet for air (the next ones are usually on their way).
        let flying = self.player.flying || self.player.noclip;
        let ready = self.joined
            && !self.dead
            && (flying
                || world
                    .block_or_air(self.player.pos.floor().as_ivec3())
                    .is_some());
        // The clock bodies are predicted by runs on while the player waits, or they would stop.
        let mut ambient_steps = 0;
        if self.joined {
            let air_if_missing = world::SeenCells { world, air_if_missing: true };
            self.accum += dt.min(0.25);
            while self.accum >= STEP {
                self.accum -= STEP;
                self.since_time = (self.since_time + STEP * self.tick_rate).min(crate::world::TPS as f64);
                ambient_steps += 1;
                // A rider is where it is on its body: if moves arrived late and the body's past
                // changed, the rider moves with it, not over the deck.
                if let Some(e) = self.player.riding.and_then(|(b, _, now)| {
                    self.entities
                        .get(&b)
                        .map(|e| e.known(self.body_tick(self.body_clock)) + now)
                }) {
                    self.player.pos = e;
                }
                self.player.prev = self.player.pos;
                let flying = self.player.flying || self.player.noclip;
                let gravity = c.dims[world.dim as usize].gravity as f64;
                // Solid bodies at this step (predicted like the server moves them) and soft
                // ones nudging the player aside.
                self.body_clock += STEP;
                let at = self.body_tick(self.body_clock);
                // Bodies move by the server's ticks, this step by its own clock.
                let step_ticks = STEP * self.tick_rate;
                let mut bodies = Vec::new();
                let mut push = DVec3::ZERO;
                for (&id, e) in &self.entities {
                    let k = &c.game.entities[e.kind as usize];
                    if !e.is_body() {
                        continue;
                    }
                    // A leaning body (chiacraft) is only drawn.
                    if k.is(weft_abi::entity_flags::SOLID) && e.lean.is_some() {
                        continue;
                    }
                    if k.is(weft_abi::entity_flags::SOLID) {
                        let (now, delta) = (e.known(at), e.known(at) - e.known(at - step_ticks));
                        let physics::Dims { half_width: hw, height: h, .. } = e.dims();
                        let built = e.structure.as_deref().map(|s| s as &dyn physics::Blocks);
                        bodies.extend(physics::Body::of(id, k, now, (hw, h), delta, built));
                    } else if k.is(weft_abi::entity_flags::PUSHABLE) {
                        let d = self.player.pos - e.pos;
                        let (theirs, mine) = (e.dims(), self.player.dims());
                        let reach = theirs.half_width + mine.half_width;
                        if d.x.abs() < reach && d.z.abs() < reach && d.y < theirs.height && -d.y < mine.height {
                            push += physics::nudge(d);
                        }
                    }
                }
                if !ready {
                    continue;
                }
                // On a mount the seat carries it; in one that steers its keys move the mount.
                if let Some((mount, seat)) = self.me.vehicle {
                    let moved = ride_step(
                        &self.entities,
                        &mut self.player,
                        &mut self.drive,
                        &mut self.conn,
                        input,
                        &block,
                        gravity,
                        &bodies,
                        mount,
                        seat,
                    );
                    drop(bodies);
                    if let (Some((pos, yaw)), Some(e)) = (moved, self.entities.get_mut(&mount)) {
                        (e.pos, e.target, e.yaw, e.target_yaw) = (pos, pos, yaw, yaw);
                    }
                    continue;
                }
                if !flying {
                    self.player.vel += push;
                }
                // Out of any block it overlaps, before moving: movement doesn't clip against what
                // the box is already inside (`physics::push_out`). From where it was too, so the
                // eyes don't pass through the block on the way (`smooth_pos`).
                // The same cells the move goes by: flying takes cubes not here yet for air.
                if !self.player.noclip {
                    let d = self.player.dims();
                    let out = if flying {
                        physics::push_out(c, &air_if_missing, self.player.pos, d.half_width, d.height)
                    } else {
                        physics::push_out(c, &block, self.player.pos, d.half_width, d.height)
                    };
                    if let Some(out) = out {
                        (self.player.pos, self.player.prev) = (out, out);
                    }
                }
                if flying {
                    step_player(&mut self.player, input, &air_if_missing, gravity, &bodies);
                } else {
                    step_player(&mut self.player, input, &block, gravity, &bodies);
                }
                let entities = &self.entities;
                let p = &self.player;
                self.player.riding = p.support.and_then(|b| {
                    entities
                        .get(&b)
                        .map(|e| (b, p.prev - e.known(at - step_ticks), p.pos - e.known(at)))
                });
                if crate::log::on(crate::log::Channel::Physics, log::Level::Trace) {
                    let p = &self.player;
                    let feet = p.pos.floor().as_ivec3() - glam::IVec3::Y;
                    log::trace!(
                        target: "physics",
                        "pos {:.4} {:.4} {:.4} vel {:.4} {:.4} {:.4} ground {} liquid {} crouch {} fly {} below {}",
                        p.pos.x, p.pos.y, p.pos.z, p.vel.x, p.vel.y, p.vel.z, p.on_ground, p.in_liquid, p.pose == Pose::CROUCH, p.flying,
                        world.block_or_air(feet).map_or("unloaded".to_string(), |s| c.state_name(s))
                    );
                }
            }
        }
        for _ in 0..ambient_steps {
            self.ambient();
        }
    }

    /// What the crosshair is on. Returns where the eye looks.
    fn aim(&mut self, input: &Input) -> DVec3 {
        let c = content();
        let spectator = self.mode.has(weft_abi::game_mode_flags::NO_INTERACT);
        // From the eye as drawn, the moment structures are drawn at too, so a block on a
        // structure the player rides stays the one aimed at.
        let from = if self.camera_has(weft_abi::camera_flags::AIM) { self.cameras.shown } else { self.camera() };
        let (eye, look) = (from.pos, from.forward().as_dvec3());
        use crate::server::extend::{BLOCK_REACH, ENTITY_REACH};
        let reach = self.attribute(BLOCK_REACH) as f64;
        self.target = if spectator { None } else { self.raycast(eye, look, reach) };
        // Vanilla's: whichever comes first along the look is all that is aimed at.
        let block_dist = self.target.map_or(f64::INFINITY, |h| h.dist);
        self.target_entity =
            if spectator { None } else { self.pick_entity(eye, look, block_dist.min(self.attribute(ENTITY_REACH) as f64)) };
        if self.target_entity.is_some() {
            self.target = None;
        }
        if input.captured && !spectator && input.pressed("core:pick") {
            // In creative a mob gives its egg, there being no other way to get what stands there.
            let egg = || {
                self.target_entity
                    .filter(|_| self.mode.has(weft_abi::game_mode_flags::NO_COSTS))
                    .and_then(|e| c.spawn_egg_of(self.entities.get(&e)?.kind))
            };
            if let Some(item) = self.target.and_then(|h| c.pick_item(h.state)).or_else(egg) {
                let hotbar = &self.inventory[..content().game.player_layout.hotbar as usize];
                // Already in the hotbar: select it. Otherwise creative fills an empty slot first.
                let slot = hotbar
                    .iter()
                    .position(|s| s.item == item && !s.is_empty())
                    .or_else(|| {
                        (self.mode.has(weft_abi::game_mode_flags::NO_COSTS) && !hotbar[self.selected].is_empty())
                            .then(|| hotbar.iter().position(|s| s.is_empty()))
                            .flatten()
                    });
                if let Some(i) = slot.filter(|&i| i != self.selected) {
                    self.selected = i;
                    self.conn.send(ToServer::Select(i as u32));
                }
                if !hotbar.iter().any(|s| s.item == item && !s.is_empty()) {
                    self.conn.send(ToServer::Pick { item });
                }
            }
        }
        look
    }

    fn move_message(&self, landed: bool, jumped: bool) -> ToServer {
        ToServer::Move {
            jumped,
            frame: self.player.riding.map(|(b, _, now)| (b, now)),
            pos: self.player.pos,
            yaw: self.player.yaw,
            pitch: self.player.pitch,
            on_ground: landed || self.player.on_ground || self.player.flying,
            pose: self.player.pose,
            sprint: self.player.sprinting,
            flying: self.player.flying,
        }
    }

    fn act(&mut self, input: &Input, dt: f64, look: DVec3) {
        self.break_delay -= dt as f32;
        self.use_delay -= dt as f32;
        // The server judges actions by where the player looks, which it only hears 20 times a
        // second: a turn made just before a click must arrive first.
        let mut send_look = LookOnce(
            self.joined
                .then(|| self.move_message(self.player.landed, false)),
        );
        if !input.captured || self.dead {
            self.breaking = None;
            return;
        }
        self.attack(input, dt, &mut send_look);
        self.use_held(input, dt, look, &mut send_look);
    }

    fn attack(&mut self, input: &Input, dt: f64, send_look: &mut LookOnce) {
        let c = content();
        let target = self.target;
        self.refused_break = self
            .refused_break
            .filter(|&r| input.held("core:attack") && target.is_some_and(|h| (h.frame, h.block) == r));
        let keeps = (self.mode.has(weft_abi::game_mode_flags::INSTANT_BREAK) && c.no_creative_break(self.held().item))
            || self.mode.has(weft_abi::game_mode_flags::NO_BREAK);
        let refused = keeps || self.refused_break.is_some();
        if input.held("core:attack") {
            if let Some(id) = self.target_entity {
                // A hit is a press, as often as you like; holding hits nothing. How much it
                // does depends on how charged the swing was.
                if input.pressed("core:attack") {
                    send_look.send(&mut self.conn);
                    self.conn.send(ToServer::Attack(id));
                    self.conn.send(ToServer::Swing);
                    self.swing_since = 0.0;
                }
                self.breaking = None;
            } else if refused {
                self.breaking = None;
            } else if let Some(hit) = self.target {
                let rate = crate::server::mining_rate(
                    c,
                    hit.state,
                    &self.held(),
                    self.eye_liquid().is_some(),
                    self.player.on_ground,
                    self.attribute(crate::server::extend::MINING_SPEED),
                    self.mode.has(weft_abi::game_mode_flags::INSTANT_BREAK),
                );
                let done = match self.breaking {
                    Some((f, p, v)) if (f, p) == (hit.frame, hit.block) && self.breaking_part == hit.state => v,
                    _ => 0.0,
                };
                self.breaking_part = hit.state;
                let progress = if rate >= 1.0 { 1.0 } else { done + dt as f32 * self.tick_rate as f32 * rate };
                if progress >= 1.0 && (input.pressed("core:attack") || self.break_delay <= 0.0) {
                    send_look.send(&mut self.conn);
                    self.conn
                        .send(ToServer::Break { pos: hit.block, part: hit.state, face: hit.face, frame: hit.frame });
                    let after = crate::wire::after_break(content(), hit.state, hit.face);
                    match hit.frame {
                        0 => self.world.replace_part(hit.block, hit.state, after),
                        f => {
                            if let Some(s) = self.entities.get_mut(&f).and_then(|e| e.structure.as_mut()) {
                                s.replace_part(hit.block, hit.state, after);
                            }
                        }
                    }
                    self.breaking = None;
                    self.break_delay = 0.25;
                } else if rate < 1.0 {
                    self.breaking = Some((hit.frame, hit.block, progress.min(1.0)));
                } else {
                    // Instant breaks (creative, grass) show no cracks while the next waits.
                    self.breaking = None;
                }
            } else {
                self.breaking = None;
            }
        } else {
            self.breaking = None;
        }
        // Hitting nothing still spends the swing.
        if input.pressed("core:attack") && self.target_entity.is_none() && self.target.is_none() {
            self.conn.send(ToServer::Swing);
            self.swing_since = 0.0;
        }
        // The server times mining too, and shows the cracks to the others.
        let mining = self.breaking.map(|(f, p, _)| (f, p, self.breaking_part));
        if mining != self.told_mining {
            self.told_mining = mining;
            self.conn.send(match mining {
                Some((frame, pos, part)) => ToServer::StartBreak { pos, part, frame },
                None => ToServer::StopBreak,
            });
        }
    }

    fn use_held(&mut self, input: &Input, dt: f64, look: DVec3, send_look: &mut LookOnce) {
        if let Some((left, _)) = &mut self.using {
            *left = (*left - dt as f32 * self.tick_rate as f32).max(0.0);
            if !input.held("core:use") {
                self.using = None;
                self.conn.send(ToServer::StopUsing);
            }
        } else if input.pressed("core:use") || (input.held("core:use") && self.use_delay <= 0.0) {
            send_look.send(&mut self.conn);
            if let Some(id) = self.target_entity {
                self.conn.send(ToServer::Interact(id));
            } else if let Some(hit) = self.target {
                let sneak = input.held("core:sneak");
                let at = self
                    .frame_place(hit.frame)
                    .to_local(self.camera().pos + self.camera().forward().as_dvec3() * hit.dist);
                let at = (at - hit.block.as_dvec3()).as_vec3().to_array();
                self.conn.send(ToServer::Use {
                    pos: hit.block,
                    part: hit.state,
                    face: hit.face,
                    look: look.as_vec3().to_array(),
                    hit: at,
                    sneak,
                    frame: hit.frame,
                });
            } else {
                self.conn.send(ToServer::UseItem { target: None });
            }
            self.use_delay = 0.2;
        }
    }

    fn send_move(&mut self, dt: f64) {
        self.tick_accum += dt;
        if self.tick_accum >= STEP {
            // Keeps the fraction, so moves go out 20 times a second at any frame rate.
            self.tick_accum = (self.tick_accum - STEP).min(STEP);
            if self.joined && !self.dead {
                let landed = std::mem::replace(&mut self.player.landed, false);
                let jumped = std::mem::replace(&mut self.player.jumped, false);
                self.conn.send(self.move_message(landed, jumped));
            }
        }
    }

    fn animate_entities(&mut self, dt: f64) {
        let c = content();
        let k = (dt * 15.0).min(1.0);
        // Solid bodies show where the physics has them, one step behind like the player.
        let shown = self.body_clock - STEP + self.accum;
        let mut pulls = std::mem::take(&mut self.pulls);
        pulls.clear();
        pulls.extend(self.entities.iter().filter_map(|(&id, e)| {
            let (_, by, _) = e.pulled?;
            let to = if by == self.player_id { self.smooth_pos() } else { self.entities.get(&by).map_or(e.pos, |t| t.pos) };
            Some((id, to))
        }));
        for &(id, to) in &pulls {
            let Some(e) = self.entities.get_mut(&id) else { continue };
            let Some((from, _, t)) = &mut e.pulled else { continue };
            *t += dt as f32;
            e.pos = from.lerp(to, (*t / PULL_SECS).min(1.0) as f64);
            e.target = e.pos;
            if *t >= PULL_SECS {
                self.entities.remove(&id);
            }
        }
        self.pulls = pulls;
        // Bodies first: riders are drawn relative to where their body is drawn.
        let mut frames = std::mem::take(&mut self.frames);
        frames.clear();
        let at = self.body_tick(shown);
        frames.extend(
            self.entities
                .iter()
                .filter(|(_, e)| c.game.entities[e.kind as usize].is(weft_abi::entity_flags::SOLID))
                .map(|(&id, e)| (id, e.pose(at))),
        );
        // A crowd's worth on all cores: nothing here depends on another entity.
        let mut moving: Vec<&mut ClientEntity> = self
            .entities
            .values_mut()
            .filter(|e| e.pulled.is_none())
            .collect();
        let frames_ref = &frames;
        let step = |e: &mut &mut ClientEntity| {
            let frames = frames_ref;
            let before = e.pos;
            let body = e
                .rel_target
                .and_then(|t| frames.iter().find(|f| f.0 == e.frame).map(|f| (f.1, t)));
            let mut walked = None;
            e.pos = if c.game.entities[e.kind as usize].is(weft_abi::entity_flags::SOLID) {
                e.pose(at)
            } else if let Some((at, rel_target)) = body {
                let rel_before = e.rel;
                e.rel = e.rel.lerp(rel_target, k);
                walked = Some(e.rel - rel_before);
                at + e.rel
            } else {
                e.pos.lerp(e.target, k)
            };
            if let Some(l) = &mut e.lean {
                l.t += dt as f32;
            }
            let mut dyaw = e.target_yaw - e.yaw;
            if dyaw.abs() > std::f32::consts::PI {
                dyaw = (dyaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            }
            e.yaw += dyaw * k as f32;
            let moved = walked.unwrap_or(e.pos - before);
            e.body_yaw = turn_body(e.body_yaw, e.yaw, moved, dt as f32);
            // A mob standing still brings its body round to where it looks (vanilla's body
            // rotation), so one that turned to face something faces it whole.
            if e.kind != content().game.player_kind && moved.with_y(0.0).length_squared() < 1e-6 {
                e.body_yaw += angle_diff(e.yaw, e.body_yaw) * (1.0 - 0.9f32.powf(dt as f32 * crate::world::TPS as f32));
            }
            advance_swing(&mut e.arm_swing, dt as f32);
            if let Some(fx) = &mut e.effect {
                fx.2 -= dt as f32;
            }
            e.effect = e.effect.filter(|fx| fx.2 > 0.0);
            let speed = (walked.unwrap_or(e.pos - before).with_y(0.0).length() / dt.max(1e-4)) as f32;
            e.walked += speed * dt as f32;
            crate::model::walk_limbs(&mut e.walk, &mut e.swing, speed, dt as f32);
            e.crouched = fade_pose(e.crouched, e.pose == Pose::CROUCH, dt as f32);
            if let Some((p, _)) = &mut e.using {
                *p = (*p + dt as f32 * crate::world::TPS as f32 / e.use_ticks.max(1.0)).min(1.0);
            }
            e.hurt = (e.hurt - dt as f32).max(0.0);
            e.age += dt as f32;
            if let Some(d) = &mut e.dying {
                *d += dt as f32;
                e.hurt = HURT_FLASH;
                e.burning = false;
            }
        };
        if moving.len() >= 512 {
            use rayon::prelude::*;
            moving.par_iter_mut().with_min_len(128).for_each(step);
        } else {
            moving.iter_mut().for_each(step);
        }
        self.frames = frames;
        self.seat_riders();
    }

    /// Riders where their seats are on their mounts as drawn this frame, mounts first.
    fn seat_riders(&mut self) {
        let c = content();
        let riders: Vec<u64> = self
            .entities
            .iter()
            .filter(|e| e.1.vehicle.is_some())
            .map(|e| *e.0)
            .collect();
        for _ in 0..4 {
            for &id in &riders {
                let Some((mount, seat)) = self.entities.get(&id).and_then(|e| e.vehicle) else { continue };
                let Some(at) = self
                    .entities
                    .get(&mount)
                    .and_then(|m| seat_at(c, m.kind, m.pos, m.yaw, m.scale, seat))
                else {
                    continue;
                };
                if let Some(e) = self.entities.get_mut(&id) {
                    (e.pos, e.target) = (at, at);
                }
            }
        }
    }

    /// Hitting a block chips pieces off the face being hit, one per tick like vanilla.
    fn chip_block(&mut self, dt: f64) {
        let c = content();
        self.hit_dust -= dt as f32;
        if let (Some((frame, pos, _)), Some(hit)) = (self.breaking, self.target) {
            if self.hit_dust <= 0.0 && (hit.frame, hit.block) == (frame, pos) {
                self.hit_dust = 0.05;
                if let Some(&kind) = c.game.particle_names.get(weft_abi::particle::BLOCK) {
                    let normal = crate::world::DIRS[hit.face as usize].as_vec3();
                    let at = self
                        .frame_place(frame)
                        .to_world(pos.as_dvec3() + DVec3::splat(0.5) + (normal * 0.55).as_dvec3());
                    let spread = (glam::Vec3::splat(0.45) * (glam::Vec3::ONE - normal.abs())).to_array();
                    let velocity = [0.0, 2.0, 0.0];
                    self.particles.spawn(
                        &weft_abi::ParticleBurst {
                            kind,
                            pos: at.to_array().into(),
                            spread: spread.into(),
                            velocity: velocity.into(),
                            speed: 0.72,
                            count: 1,
                            block: hit.state,
                            ..weft_abi::ParticleBurst::DEFAULT
                        },
                        &self.world,
                    );
                }
            }
        }
    }

    /// Every block state's textures, loaded once while the world comes in after joining: a mesh
    /// loading its own mid-play decoded with the frame waiting (15-35 ms frames in a new biome,
    /// budgeted uploads overrunning to 30 ms; perf review #9). About 0.2 ms a texture, decoded in
    /// parallel. A pack's own looks still load when first drawn.
    fn preload_block_textures(&mut self, gpu: &mut dyn Backend, textures: &GameTextures) {
        if self.blocks_preloaded || !self.joined {
            return;
        }
        self.blocks_preloaded = true;
        let t = Instant::now();
        // All of them, not only the blocks': items, entities, particles, the GUI and connected
        // tiles otherwise loaded inside a mesh upload when first shown (about 80 at a join).
        let n = textures.load_all(gpu);
        log::info!(target: "load", "textures preloaded: {n} in {:.0} ms", t.elapsed().as_secs_f64() * 1000.0);
    }

    fn upload_meshes(&mut self, gpu: &mut dyn Backend, textures: &GameTextures) {
        self.preload_block_textures(gpu, textures);
        let center: IVec3 = (self.player.pos.floor().as_ivec3()) >> 4;
        // Wide enough for every column the server keeps sent (view radius + 2).
        let reach = self.view_radius + 3;
        if self.render.set_tint_tiles(gpu, (reach * 2 + 1) as u32) {
            self.world.retint_all();
        }
        if std::mem::take(&mut self.world.tints_changed) {
            self.render.set_biome_table(gpu, self.world.biome_tints());
        }
        let p = crate::diag::part("tints");
        for (col, tints) in self
            .world
            .take_tints(glam::IVec2::new(center.x, center.z), reach)
        {
            self.render.upload_tints(gpu, col, &tints);
        }
        drop(p);
        if self.joined && self.terrain_view != (center, self.view_radius) {
            self.terrain_view = (center, self.view_radius);
            let _ = self
                .terrain
                .tx
                .send(terrain::In::View { center, radius: self.view_radius });
        }
        // Enough jobs that meshing keeps up at 50 frames per second too (results are taken once
        // a frame), not only at hundreds.
        let p = crate::diag::part("schedule meshing");
        self.world
            .schedule_meshing(center, crate::meshing().current_num_threads() * 512);
        drop(p);
        let p = crate::diag::part("solid cells");
        self.render.update_solid(gpu, &self.world, center);
        drop(p);
        self.world.set_eye(center.y);
        // Uploads too, oldest first; the rest wait for the next frame (dropped if their cube
        // changed or left meanwhile). Edits and what is around the player go right away.
        let p = crate::diag::part("uploads near (no budget)");
        let (now, later) = self.world.take_meshes(center);
        for r in now {
            self.render.upload(gpu, textures, (0, r.pos), &r.mesh);
            self.world.mark_shown(r.pos);
        }
        self.meshes.extend(later);
        drop(p);
        let _p = crate::diag::part("uploads (budget)");
        let start = Instant::now();
        while start.elapsed() < self.budget {
            let Some(r) = self.meshes.pop_front() else { break };
            if self.world.current(&r) {
                self.render.upload(gpu, textures, (0, r.pos), &r.mesh);
                self.world.mark_shown(r.pos);
            }
        }
        self.spent += start.elapsed().as_secs_f64();
        // Structures: meshed in their own coordinates, a changed cube at a time.
        for (&id, e) in self.entities.iter_mut() {
            let Some(s) = e.structure.as_mut() else { continue };
            s.schedule_meshing(IVec3::ZERO, 64);
            let (now, later) = s.take_meshes(IVec3::ZERO);
            for r in now.into_iter().chain(later) {
                self.render.upload(gpu, textures, (id, r.pos), &r.mesh);
            }
            for pos in std::mem::take(&mut s.removed) {
                self.render.remove((id, pos));
            }
            self.drawn_frames.insert(id);
        }
        let entities = &self.entities;
        let gone: Vec<u64> = self
            .drawn_frames
            .iter()
            .copied()
            .filter(|id| !entities.get(id).is_some_and(|e| e.structure.is_some()))
            .collect();
        for id in gone {
            self.render.remove_frame(id);
            self.drawn_frames.remove(&id);
        }
        if self.joined {
            let cam = self.render_camera().pos;
            self.lod
                .update(gpu, &mut self.render, &mut self.world, textures, self.seed, cam, self.view_radius);
        }
        self.render.set_textures(gpu, textures);
    }

    fn chance(&mut self) -> f32 {
        self.rng.float()
    }

    /// What the player's own moving kicks up: a landing's dust, a sprint's, a splash going into
    /// water and bubbles while under it.
    /// Bubbles from the eyes under a liquid that has them; a fall's dust and splash come from
    /// the server, a sprint's from `sprint_dust`, for everyone near.
    fn body_particles(&mut self, dt: f32) {
        let c = content();
        let pos = self.player.pos;
        if self.me.sprinting {
            self.sprint_dust(pos, self.player.vel, self.player.size.width / 2.0, dt);
        }
        let bubble = self.eye_liquid().and_then(|l| {
            let state = c.liquids[l as usize].first_state;
            let id = c.components.id(weft_abi::comp::BUBBLE)?;
            c.game
                .particle_names
                .get(c.components.text(crate::components::BLOCK, state, id)?)
                .copied()
        });
        if let Some(kind) = bubble.filter(|_| self.chance() < dt * 1.5) {
            let at = pos + DVec3::Y * self.player.eye;
            self.particles.spawn(
                &weft_abi::ParticleBurst {
                    kind,
                    pos: at.to_array().into(),
                    count: 2,
                    spread: [0.2, 0.1, 0.2].into(),
                    speed: 0.5,
                    ..weft_abi::ParticleBurst::DEFAULT
                },
                &self.world,
            );
        }
    }

    /// Vanilla's `spawnSprintParticle`: now and then a bit of the ground under a sprinter, thrown
    /// back and up. `vel` in blocks per tick.
    fn sprint_dust(&mut self, pos: DVec3, vel: DVec3, half_width: f32, dt: f32) {
        let c = content();
        let under = self
            .world
            .block_or_air((pos - DVec3::Y * 0.2).floor().as_ivec3())
            .unwrap_or(0);
        let Some(&kind) = c.game.particle_names.get(weft_abi::particle::BLOCK) else { return };
        if under == 0 || c.states[under as usize].liquid.is_some() || self.chance() >= dt * crate::world::TPS as f32 {
            return;
        }
        let mut noise = || self.rng.float() * 0.8 - 0.4;
        let aim = glam::Vec3::new(-4.0 * vel.x as f32 + noise(), 1.5 + noise(), -4.0 * vel.z as f32 + noise());
        let speed = (self.chance() + self.chance() + 1.0) / 3.0 * 3.6;
        let velocity = (aim.normalize_or_zero() * speed + glam::Vec3::Y * 2.0).to_array();
        self.particles.spawn(
            &weft_abi::ParticleBurst {
                kind,
                block: under,
                pos: (pos + DVec3::Y * 0.1).to_array().into(),
                count: 1,
                spread: [half_width, 0.0, half_width].into(),
                velocity: velocity.into(),
                ..weft_abi::ParticleBurst::DEFAULT
            },
            &self.world,
        );
    }

    /// What goes over the screen: the inside of a liquid, of a block, a fire.
    fn overlays(&self, textures: &GameTextures) -> Vec<sky::Overlay> {
        let c = content();
        let mut out = Vec::new();
        if !self.camera_has(weft_abi::camera_flags::EFFECTS) || self.hide_gui {
            return out;
        }
        let light = self.daylight().max(0.25);
        let looks = looks::current();
        if let Some(l) = self
            .eye_liquid()
            .map(|l| &c.liquids[l as usize])
            .filter(|l| l.overlay_opacity > 0.0)
        {
            let st = &c.states[l.first_state as usize];
            let own = looks
                .texture_name(c, looks.drawn(l.first_state, st).textures[0])
                .unwrap_or("");
            let tint = if st.tint != 0 { sky::rgb(crate::content::item_tint(st.tint)) } else { Vec3::ONE };
            out.push(sky::Overlay {
                picture: if l.overlay.is_empty() { own.to_string() } else { l.overlay.clone() },
                color: (tint * light)
                    .powf(2.2)
                    .extend(l.overlay_opacity)
                    .to_array(),
                tiles: 1.0,
                lower: false,
                frame: 0,
            });
        }
        let head = (self.smooth_pos() + DVec3::Y * self.player.eye)
            .floor()
            .as_ivec3();
        if !self.player.noclip {
            if let Some(st) = self
                .world
                .block_or_air(head)
                .filter(|&s| c.states[s as usize].flags & weft_abi::block_flags::SOLID != 0 && c.tight[s as usize].opacity >= 15)
            {
                if let Some(picture) = looks.texture_name(c, looks.drawn(st, &c.states[st as usize]).textures[1]) {
                    out.push(sky::Overlay {
                        picture: picture.to_string(),
                        color: [0.01, 0.01, 0.01, 1.0],
                        tiles: 1.0,
                        lower: false,
                        frame: 0,
                    });
                }
            }
        }
        if self.me.burning {
            if let Some(picture) = c.textures.get(c.fire_texture as usize) {
                let ticks = self.start.elapsed().as_secs_f32() * crate::world::TPS as f32;
                // Loaded by being asked for its picture, for mobs too.
                textures.texture(c.fire_texture);
                let frame = textures.strip_frame(c.fire_texture, ticks);
                out.push(sky::Overlay { picture: picture.clone(), color: [1.0, 1.0, 1.0, 0.9], tiles: 2.0, lower: true, frame });
            }
        }
        out
    }

    /// Like vanilla's block display ticks: random blocks around the player that name a particle
    /// (`abi::comp::AMBIENT_PARTICLE`) show one, each about every three seconds, and liquids
    /// with `abi::comp::DRIP` drip through the ceiling under them. The server has no part in it.
    fn ambient(&mut self) {
        const TRIES: usize = 667;
        let c = content();
        let kinds = self.ambient_kinds.get_or_insert_with(|| {
            let (ambient, drip) = (c.components.id(weft_abi::comp::AMBIENT_PARTICLE), c.components.id(weft_abi::comp::DRIP));
            let named = |s: u32, comp: Option<u32>| {
                comp.and_then(|id| c.components.text(crate::components::BLOCK, s, id))
                    .map_or(Vec::new(), |names| {
                        names
                            .split(',')
                            .filter_map(|n| c.game.particle_names.get(n).copied())
                            .collect()
                    })
            };
            (0..c.states.len() as u32)
                .map(|s| (named(s, ambient), named(s, drip).first().copied()))
                .collect()
        });
        let center = self.player.pos.floor().as_ivec3();
        let mut bursts: Vec<weft_abi::ParticleBurst> = Vec::new();
        for _ in 0..TRIES {
            let r = self.rng.next();
            let p = center + IVec3::new((r % 33) as i32 - 16, ((r >> 8) % 33) as i32 - 16, ((r >> 16) % 33) as i32 - 16);
            let state = self.world.block(p);
            let Some((own, drip)) = kinds.get(state as usize) else { continue };
            for &kind in own {
                // Vanilla's: over a torch, and out from the wall where it leans.
                let wall = c.states[state as usize]
                    .support
                    .map(|(f, _)| f as usize)
                    .filter(|f| f / 2 != 1)
                    .map(|f| crate::world::DIRS[f].as_dvec3());
                let pos = (p.as_dvec3() + wall.map_or(DVec3::new(0.5, 0.7, 0.5), |d| DVec3::new(0.5, 0.92, 0.5) - d * 0.27)).to_array();
                bursts.push(weft_abi::ParticleBurst {
                    kind,
                    pos: pos.into(),
                    spread: [0.1, 0.1, 0.1].into(),
                    speed: 0.1,
                    count: 1,
                    ..weft_abi::ParticleBurst::DEFAULT
                });
            }
            // Through a ceiling: a solid block under the liquid, air under that.
            if let Some(kind) = drip.filter(|_| (r >> 24) % 8 == 0) {
                let (below, air) = (self.world.block(p - IVec3::Y), self.world.block(p - IVec3::Y * 2));
                // A cube not here yet reads `u32::MAX`: no drip (it indexed past the states and
                // crashed by a tarn's water near the top of what had arrived).
                if c.states
                    .get(below as usize)
                    .is_some_and(|s| s.flags & weft_abi::block_flags::SOLID != 0)
                    && air == 0
                {
                    let pos = [p.x as f64 + 0.5, p.y as f64 - 1.05, p.z as f64 + 0.5];
                    bursts.push(weft_abi::ParticleBurst {
                        kind,
                        pos: pos.into(),
                        spread: [0.4, 0.0, 0.4].into(),
                        count: 1,
                        ..weft_abi::ParticleBurst::DEFAULT
                    });
                }
            }
        }
        for b in bursts {
            self.particles.spawn(&b, &self.world);
        }
    }

    /// The engine's cosmetics of one entity, its kind's client tick unless a mod gave it one:
    /// steps by distance walked, idle noises about every twelve seconds (vanilla's ambient
    /// timer), and a puff of smoke when its death animation ends (vanilla's 20 poof particles).
    pub fn entity_effects(&mut self, a: &weft_abi::ClientEntityArgs) {
        use crate::audio::Play;
        use crate::content::block_sound;
        use crate::registry::entity_sound;
        let c = content();
        let k = &c.game.entities[a.kind as usize];
        let pos = crate::world::dvec(a.pos);
        if a.dying >= 1.0 {
            if let Some(&kind) = c.game.particle_names.get(weft_abi::particle::POOF) {
                let crate::physics::Dims { half_width: hw, height: h, .. } = physics::posed(k.size(), Pose::STAND);
                let (w, h) = (hw as f32, h as f32 * 0.5);
                let at = (pos + DVec3::Y * h as f64).to_array();
                self.particles.spawn(
                    &weft_abi::ParticleBurst {
                        kind,
                        pos: at.into(),
                        spread: [w, h, w].into(),
                        velocity: [0.0, 0.4, 0.0].into(),
                        speed: 0.8,
                        count: 20,
                        ..weft_abi::ParticleBurst::DEFAULT
                    },
                    &self.world,
                );
            }
            return;
        }
        if a.dying >= 0.0 {
            return;
        }
        let Some(e) = self.entities.get_mut(&a.entity) else { return };
        if a.walked >= e.next_step {
            e.next_step = a.walked + 1.6;
            let sound = if k.sounds[entity_sound::STEP] != u32::MAX {
                k.sounds[entity_sound::STEP]
            } else {
                self.world
                    .block_or_air((pos - DVec3::Y * 0.2).floor().as_ivec3())
                    .map_or(u32::MAX, |s| c.states[s as usize].sounds[block_sound::STEP])
            };
            if sound != u32::MAX {
                self.sounds
                    .push(Play { sound, pos: Some(pos), volume: 0.2, pitch: 0.0 });
            }
        }
        if e.sprinting {
            // Vanilla's sprint, about 0.28 blocks a tick, the way the body faces.
            let (sin, cos) = (e.body_yaw as f64).sin_cos();
            let (vel, hw) = (DVec3::new(-sin, 0.0, -cos) * 0.28, e.dims().half_width as f32);
            self.sprint_dust(pos, vel, hw, a.dt);
        }
        let Some(e) = self.entities.get(&a.entity) else { return };
        let eye = e.dims().eye;
        if k.sounds[entity_sound::AMBIENT] != u32::MAX && self.chance() < a.dt / 12.0 {
            self.sounds
                .push(Play { sound: k.sounds[entity_sound::AMBIENT], pos: Some(pos + DVec3::Y * eye), volume: 1.0, pitch: 0.0 });
        }
    }

    /// The player's own steps and the knock of the block being mined; decided here, nothing
    /// travels for them.
    fn creature_sounds(&mut self, dt: f32, moved: f32) {
        use crate::audio::Play;
        use crate::content::block_sound;
        let c = content();
        let below = |w: &world::ClientWorld, p: DVec3| {
            w.block_or_air((p - DVec3::Y * 0.2).floor().as_ivec3())
                .map_or(u32::MAX, |s| c.states[s as usize].sounds[block_sound::STEP])
        };
        if self.player.on_ground && !self.player.flying && !self.mode.has(weft_abi::game_mode_flags::HIDDEN) {
            self.next_step -= moved;
            if self.next_step <= 0.0 {
                // Vanilla: a step every 1/0.6 blocks walked.
                self.next_step = 1.6;
                let sound = below(&self.world, self.player.pos);
                self.sounds
                    .push(Play { sound, pos: Some(self.player.pos), volume: 0.25, pitch: 0.0 });
            }
        }
        self.hit_sound -= dt;
        if let Some(hit) = self.target.filter(|h| {
            self.breaking
                .is_some_and(|(f, p, _)| (f, p) == (h.frame, h.block))
        }) {
            if self.hit_sound <= 0.0 {
                // Vanilla knocks every four ticks while mining.
                self.hit_sound = 0.2;
                let sound = c.states[hit.state as usize].sounds[block_sound::HIT];
                self.sounds.push(Play {
                    sound,
                    pos: Some(
                        self.frame_place(hit.frame)
                            .to_world(hit.block.as_dvec3() + DVec3::splat(0.5)),
                    ),
                    volume: 0.4,
                    pitch: 0.0,
                });
            }
        }
        self.sounds.retain(|p| p.sound != u32::MAX);
    }

    /// What the ears need from the camera this frame.
    pub fn listener(&self) -> crate::audio::Listener {
        let c = content();
        let cam = self.render_camera();
        let eye = cam.pos.floor().as_ivec3();
        crate::audio::Listener {
            pos: cam.pos,
            yaw: cam.yaw,
            pitch: cam.pitch,
            underwater: self
                .eye_liquid()
                .is_some_and(|l| !c.liquids[l as usize].burns()),
            enclosed: weft_abi::light::sky(self.world.light(eye)) < 6 && c.dims[self.world.dim as usize].sky_light,
            dim: self.world.dim,
            holds: c
                .game
                .music
                .iter()
                .flat_map(|m| &m.when)
                .copied()
                .filter(|&id| crate::conditions::on_client(Some(self), id))
                .collect(),
        }
    }

    /// No sky light at the camera: a cave, or under a roof.
    pub fn enclosed(&self) -> bool {
        let c = content();
        let eye = self.camera().pos.floor().as_ivec3();
        weft_abi::light::sky(self.world.light(eye)) < 6 && c.dims[self.world.dim as usize].sky_light
    }

    fn pick_entity(&self, eye: DVec3, dir: DVec3, max: f64) -> Option<u64> {
        let mut best: Option<(f64, u64)> = None;
        for (&id, e) in &self.entities {
            // Structures are aimed at by their blocks.
            if !e.is_body() || id == self.player_id || e.structure.is_some() {
                continue;
            }
            let physics::Dims { half_width: hw, height: h, .. } = e.dims();
            let b = physics::Aabb::around(e.pos, hw, h);
            if let Some(t) = physics::ray_aabb(eye, dir, &b, max) {
                if best.is_none_or(|(bt, _)| t < bt) {
                    best = Some((t, id));
                }
            }
        }
        best.map(|b| b.1)
    }

    /// Copies an engine field component into the entity (or the own player) that shows it.
    fn engine_field(&mut self, id: u64, comp: u32, bytes: &[u8]) {
        let c = content();
        let [health, burning, worn, shape, held, off_held, using, swaps, looping, effect, forced, riding, held_look, worn_looks] =
            c.game.field_comps;
        let now = self.world.time;
        let f: Vec<f32> = words(bytes).map(f32::from_le_bytes).collect();
        let shaped = || {
            let size = (f.len() >= 4 && f[0] > 0.0).then(|| weft_abi::EntitySize { width: f[0], height: f[1], eye: f[2] });
            let look = words(bytes)
                .nth(3)
                .map(u32::from_le_bytes)
                .filter(|&l| l != u32::MAX);
            let scale = f.get(4).copied().filter(|&s| s > 0.0).unwrap_or(1.0);
            (size, look, scale)
        };
        let mine = id == self.player_id;
        let Some(e) = (if mine { Some(&mut self.me) } else { self.entities.get_mut(&id) }) else { return };
        if comp == health && f.len() == 2 {
            // Lower health out of a lower maximum, or on joining, isn't a hit.
            if f[0] < e.health && (!mine || (f[1] >= self.max_health && e.health > 0.0)) {
                e.hurt = HURT_FLASH;
            }
            e.health = f[0];
        } else if comp == riding {
            e.vehicle = (bytes.len() >= 12).then(|| {
                let mount = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                (mount, u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as u8)
            });
        } else if comp == burning {
            e.burning = bytes.first().is_some_and(|&b| b != 0);
        } else if comp == worn {
            e.worn = words(bytes).map(u32::from_le_bytes).collect();
        } else if comp == worn_looks {
            e.worn_looks = String::from_utf8_lossy(bytes)
                .split('\n')
                .map(str::to_string)
                .collect();
        } else if comp == held {
            e.held = words(bytes).next().map_or(0, u32::from_le_bytes);
        } else if comp == held_look {
            e.held_look = std::str::from_utf8(bytes)
                .ok()
                .filter(|l| !l.is_empty())
                .map(str::to_string);
        } else if comp == swaps {
            e.swaps = parse_swaps(bytes);
        } else if comp == looping {
            // For who wasn't around when it started; those who were have it from `PlayAction`.
            let name = String::from_utf8_lossy(bytes);
            if !name.is_empty() && e.action.as_ref().is_none_or(|a| a.0 != name) {
                e.action = Some((name.into_owned(), e.age, true));
            }
        } else if comp == off_held {
            e.off_held = words(bytes).next().map_or(0, u32::from_le_bytes);
        } else if comp == using {
            let v = words(bytes).next().map_or(0, u32::from_le_bytes);
            e.use_ticks = (v & !(1 << 31)) as f32;
            e.using = (v != 0).then(|| (e.using.map_or(0.0, |u| u.0), v >> 31 != 0));
        } else if comp == shape {
            (e.size, e.look, e.scale) = shaped();
        } else if comp == effect {
            e.effect = (bytes.len() >= 16).then(|| {
                let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                let strength = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
                let until = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
                (id, strength, until.saturating_sub(now) as f32 / crate::world::TPS as f32)
            });
        }
        if !mine {
            return;
        }
        // What the local player's own movement and screens go by.
        if comp == forced {
            self.player.forced = (bytes.len() >= 8).then(|| {
                let yaw = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
                (Pose::from_id(u32::from_le_bytes(bytes[..4].try_into().unwrap())), (!yaw.is_nan()).then_some(yaw))
            });
        } else if comp == health && f.len() == 2 {
            self.max_health = f[1];
            if f[0] > 0.0 {
                self.dead = false;
            }
        } else if comp == shape {
            self.player.size = shaped()
                .0
                .unwrap_or(c.game.entities[c.game.player_kind as usize].size());
        }
    }

    /// The worn looks of what the local player wears (`abi::comp::WORN_LOOK`), in `worn`'s order.
    pub fn worn_looks(&self) -> Vec<String> {
        let c = content();
        let w = &c.game.player_layout.worn;
        self.inventory
            .get(w.start as usize..w.end as usize)
            .unwrap_or_default()
            .iter()
            .filter(|s| !s.is_empty() && c.worn(s.item).is_some())
            .map(|s| crate::stack::worn_look(s).unwrap_or("").to_string())
            .collect()
    }

    /// Items the local player wears.
    pub fn worn(&self) -> Vec<u32> {
        let c = content();
        let w = &c.game.player_layout.worn;
        let slots = self
            .inventory
            .get(w.start as usize..w.end as usize)
            .unwrap_or_default();
        slots
            .iter()
            .filter(|s| !s.is_empty() && c.worn(s.item).is_some())
            .map(|s| s.item)
            .collect()
    }

    /// Wire of the world and of bodies, before the frame's passes.
    pub fn update_wires(&mut self, gpu: &mut dyn weft_render::Backend, textures: &textures::GameTextures) {
        self.wires.update(gpu, 0, &mut self.world, textures);
        for (&id, e) in &mut self.entities {
            if let Some(s) = e.structure.as_deref_mut() {
                self.wires.update(gpu, id, s, textures);
            }
        }
        let entities = &self.entities;
        self.wires
            .keep_frames(gpu, |id| entities.get(&id).is_some_and(|e| e.structure.is_some()));
    }

    /// Where `frame`'s space is drawn: a structure's pose, the world as it is at 0.
    pub fn frame_place(&self, frame: u64) -> physics::Transform {
        self.entities
            .get(&frame)
            .filter(|_| frame != 0)
            .map_or(physics::Transform::default(), |e| physics::Transform::of(e.pos))
    }

    /// The block looked at: the world's or a structure's, whichever the ray enters first.
    fn raycast(&self, eye: DVec3, look: DVec3, reach: f64) -> Option<Hit> {
        let world = &self.world;
        let now = self.body_tick(self.body_clock - STEP + self.accum);
        let structures: Vec<(u64, physics::Transform, &dyn physics::Blocks)> = self
            .entities
            .iter()
            .filter(|(_, e)| e.lean.is_none())
            .filter_map(|(&id, e)| {
                e.structure
                    .as_deref()
                    .map(|s| (id, physics::Transform::of(e.pose(now)), s as &dyn physics::Blocks))
            })
            .collect();
        physics::pick(content(), &world::SeenCells { world, air_if_missing: false }, &structures, eye, look, reach, false)
    }

    /// Server tick (fractional) bodies are shown at when the client clock reads `clock`.
    fn body_tick(&self, clock: f64) -> f64 {
        (clock - self.tick_offset.unwrap_or(clock)) * self.tick_rate - INTERP_TICKS
    }

    /// Position drawn this frame, between the last two physics ticks; on a body, on the body as
    /// it is shown now.
    pub fn smooth_pos(&self) -> DVec3 {
        let t = (self.accum / STEP).clamp(0.0, 1.0);
        if let Some((b, before, now)) = self.player.riding {
            if let Some(e) = self.entities.get(&b) {
                return e.pose(self.body_tick(self.body_clock - STEP + self.accum)) + before.lerp(now, t);
            }
        }
        self.player.prev.lerp(self.player.pos, t)
    }

    /// From inside a liquid everything looks closer, vanilla's 60 degrees for 70.
    fn inside_fov(&self) -> f32 {
        if self.eye_liquid().is_some() { 6.0 / 7.0 } else { 1.0 }
    }

    /// Where the player's eyes are and look; what the camera mode starts from.
    pub fn camera(&self) -> Camera {
        // From inside a liquid everything looks closer, vanilla's 60 degrees for 70.
        let inside = self.inside_fov();
        // Lying on its back the head is behind the middle of the body (on a bed's pillow).
        let pillow = match self.player.forced {
            Some((pose, Some(yaw))) if pose.def().tilt == weft_abi::pose_tilt::ON_BACK => {
                DVec3::new(yaw.sin() as f64, 0.0, yaw.cos() as f64) * 0.75
            }
            _ => DVec3::ZERO,
        };
        Camera {
            pos: self.smooth_pos() + DVec3::Y * self.player.eye + pillow,
            yaw: self.player.yaw,
            pitch: self.player.pitch,
            fov: self.fov * self.player.fov_mul * self.attribute(crate::server::extend::FOV) * inside,
            bob: (Vec3::ZERO, 0.0, 0.0),
        }
    }

    /// The liquid the eyes are in.
    fn eye_liquid(&self) -> Option<u16> {
        let world = &self.world;
        physics::liquid_at(content(), &world::SeenCells { world, air_if_missing: false }, self.smooth_pos() + DVec3::Y * self.player.eye)
    }

    /// The camera with view bobbing, for drawing (aiming uses the steady one).
    fn bobbed(&self, mut cam: Camera) -> Camera {
        use std::f32::consts::PI;
        let (f, g) = (-self.walk_dist * PI, self.bob);
        // Vanilla's bobHurt: a jolt to the side when hit, and falling over when dead.
        let hit = (self.me.hurt / HURT_FLASH).clamp(0.0, 1.0);
        let jolt = (hit.powi(4) * PI).sin() * 14.0;
        let fallen = self
            .dead_for
            .map_or(0.0, |t| 40.0 - 8000.0 / ((t * crate::world::TPS as f32).min(20.0) + 200.0));
        let (kick, roll, pitch) = self.entity_render.camera_kick;
        cam.bob = (
            Vec3::new(f.sin() * g * 0.5, -(f.cos() * g).abs(), 0.0) + kick,
            (f.sin() * g * 3.0 - jolt + fallen).to_radians() + roll,
            ((f - 0.2).cos() * g).abs().to_radians() * 5.0 + pitch,
        );
        cam
    }

    /// Vanilla's swing: starts over once past halfway, which is the quick arm of holding a block
    /// being mined.
    fn swing_arm(&mut self) {
        restart_swing(&mut self.hand_swing);
        self.hand_swing_off = false;
    }

    /// The camera drawn from, as its mode put it this frame (`camera::Cameras`).
    pub fn render_camera(&self) -> Camera {
        self.cameras.shown
    }

    /// The local player as an entity.
    /// Brings `me` up to what prediction, the inventory and the hand say this frame.
    fn sync_me(&mut self) {
        let w = &content().game.player_layout.worn;
        let worn: Vec<u32> = self
            .inventory
            .get(w.start as usize..w.end as usize)
            .map_or(Vec::new(), |s| s.iter().filter(|s| !s.is_empty()).map(|s| s.item).collect());
        let worn_looks: Vec<String> = self
            .inventory
            .get(w.start as usize..w.end as usize)
            .map_or(Vec::new(), |s| {
                s.iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| crate::stack::worn_look(s).unwrap_or("").to_string())
                    .collect()
            });
        let (pos, held, off_held) = (self.smooth_pos(), self.shown_held().item, self.off_held().item);
        let using = self
            .using
            .map(|(left, all)| (1.0 - left / all.max(1.0), self.using_off));
        let held_look = crate::stack::layers(&self.shown_held()).map(str::to_string);
        let me = &mut self.me;
        me.held_look = held_look;
        (me.pos, me.target, me.yaw, me.target_yaw, me.pitch) = (pos, pos, self.player.yaw, self.player.yaw, self.player.pitch);
        (me.worn, me.held, me.off_held, me.using) = (worn, held, off_held, using);
        me.worn_looks = worn_looks;
        (me.arm_swing, me.swing_off, me.pose, me.walked) = (self.hand_swing, self.hand_swing_off, self.player.pose, me.walk);
        me.sprinting = self.player.sprinting && self.player.on_ground;
        me.age = self.start.elapsed().as_secs_f32();
        self.player.swims_in = swims_in(content(), self.me.kind, &self.worn());
    }

    /// Columns distant terrain takes from saved cubes.
    pub fn lod_columns(&self) -> usize {
        self.lod.columns_len()
    }

    pub fn set_view_distance(&mut self, r: i32) {
        self.view_radius = r;
        self.send(ToServer::View(r as u32));
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some(t) = self.shadow_map.take() {
            gpu.destroy_texture(t);
        }
        if let Some(t) = self.shadow_far_map.take() {
            gpu.destroy_texture(t);
        }
        self.shadow_far_drawn = None;
        for b in self.shadow_far_bufs.drain(..) {
            gpu.destroy_buffer(b);
        }
        for b in self.shadow_bufs.drain(..) {
            gpu.destroy_buffer(b);
        }
        for b in self.scene_bufs.drain(..) {
            gpu.destroy_buffer(b);
        }
        self.render.destroy(gpu);
        self.entity_render.destroy(gpu);
        self.particles.destroy(gpu);
        self.wires.destroy(gpu);
        self.weather_render.destroy(gpu);
        self.sky.destroy(gpu);
        self.gpu_terrain.destroy(gpu);
    }

    /// Fraction of full daylight for the current dimension and time.
    pub fn daylight(&self) -> f32 {
        // Rain takes away a third of the day, a lightning brings it back for a moment.
        let day = content().dims[self.world.dim as usize].daylight(self.world.time) * (1.0 - self.look.darkness);
        if self.lightning > 0.0 { (day + 0.5).min(1.0) } else { day }
    }

    /// The weather's look going over to the current one's, its sound now and then, and a
    /// lightning's flash (`weather` draws the precipitation).
    fn weather_effects(&mut self, dt: f32) {
        let weather = &content().game.weathers[self.weather_id as usize];
        // Where the player stands: the weather only in some biomes, when a mod says where
        // (chiacraft); clear elsewhere. Asked at nine places round the player a stretch apart,
        // so across a border into rain the sky and the rain come in over a few chunks.
        let at = self.player.pos.floor().as_ivec3();
        let mut wet = 0;
        for dx in -1..=1 {
            for dz in -1..=1 {
                if self
                    .world
                    .rains_at(at + IVec3::new(dx * WEATHER_REACH, 0, dz * WEATHER_REACH))
                {
                    wet += 1;
                }
            }
        }
        let share = wet as f32 / 9.0;
        let here = self.world.rains_at(at);
        // The sky gathers first: clouds and gray over GLOOM_SECS (and clears as slowly); what
        // falls comes once it is more than half there, over FALLS_SECS, and stops first. Both
        // went from nothing to all in five seconds, the moment the spell began (Ronald, 11-06:
        // "it like change in 2 seconds from clear to rain").
        self.gloom += (share - self.gloom).clamp(-dt / GLOOM_SECS, dt / GLOOM_SECS);
        let none = weft_abi::WeatherLook::default();
        let to = weather.look;
        let sky = |clear: f32, rain: f32| clear + (rain - clear) * self.gloom;
        let falls_to = if weather.falls() { to.falls * share * smoothstep(0.55, 0.9, self.gloom) } else { 0.0 };
        let step = dt / 5.0;
        let go = |v: &mut f32, t: f32, step: f32| *v += (t - *v).clamp(-step, step);
        let l = &mut self.look;
        go(&mut l.falls, falls_to, dt / FALLS_SECS);
        go(&mut l.sky_gray, sky(none.sky_gray, to.sky_gray), step);
        go(&mut l.sky_cover, sky(none.sky_cover, to.sky_cover), step);
        go(&mut l.fog_gray, sky(none.fog_gray, to.fog_gray), step);
        go(&mut l.cloud_gray, sky(none.cloud_gray, to.cloud_gray), step);
        go(&mut l.cloud_dark, sky(none.cloud_dark, to.cloud_dark), step);
        go(&mut l.darkness, sky(none.darkness, to.darkness), step);
        self.lightning = (self.lightning - dt).max(0.0);
        self.lightning_seen.0 = (self.lightning_seen.0 + dt).min(1.0e4);
        // The air where the player is: fog rolls in over a few seconds (the weather itself changes
        // it slowly), the wind turns in seconds.
        let (fog, speed, cover, flags) = self.world.air_at(self.player.pos.floor().as_ivec3());
        self.fog += (fog - self.fog).clamp(-dt / 8.0, dt / 8.0);
        self.snow_cover += (cover - self.snow_cover).clamp(-dt / 20.0, dt / 20.0);
        let on = |f: u32| if flags & f != 0 { 1.0 } else { 0.0 };
        self.snowing += (on(weft_abi::air_flags::SNOWING) - self.snowing).clamp(-dt / 5.0, dt / 5.0);
        self.dust += (on(weft_abi::air_flags::DUST) - self.dust).clamp(-dt / 5.0, dt / 5.0);
        let aurora = ((flags >> weft_abi::air_flags::AURORA_SHIFT) & 255) as f32 / 255.0;
        self.aurora += (aurora - self.aurora).clamp(-dt / 20.0, dt / 20.0);
        let canopy = ((flags >> weft_abi::air_flags::CANOPY_SHIFT) & 255) as f32 / 255.0;
        self.canopy += (canopy - self.canopy).clamp(-dt / 10.0, dt / 10.0);
        // The ground wets as the rain comes (in about twenty seconds) and dries after it at the
        // pace the air there gives (a mod's `DRYING_SHIFT`; 0, about two minutes): it stayed wet
        // only as long as the rain fell, gone 25 s after (Tester 2's after-rain shots, 11-05).
        let falling = if self.snowing > 0.5 { 0.0 } else { self.look.falls };
        if falling > self.wet {
            self.wet = (self.wet + dt / 20.0).min(falling);
        } else {
            let drying = (flags >> weft_abi::air_flags::DRYING_SHIFT) & 255;
            let secs = if drying == 0 { 120.0 } else { 3600.0 * (30.0f32 / 3600.0).powf((drying - 1) as f32 / 254.0) };
            self.wet = (self.wet - dt / secs).max(falling);
        }
        // Puddles grow over about ten minutes of steady rain (a shower leaves a few) and dry a third
        // as fast as the ground after it, the last of the wet to go (at half its pace they were gone
        // three minutes after a five-minute rain, the ground still dark: PC shpd7).
        if falling > 0.0 {
            self.puddles = (self.puddles + falling * dt / 600.0).min(1.0);
        } else {
            let drying = (flags >> weft_abi::air_flags::DRYING_SHIFT) & 255;
            let secs = if drying == 0 { 120.0 } else { 3600.0 * (30.0f32 / 3600.0).powf((drying - 1) as f32 / 254.0) };
            self.puddles = (self.puddles - dt / (secs * 3.0)).max(0.0);
        }
        let line = (flags >> weft_abi::air_flags::SNOWLINE_SHIFT) & 63;
        let line = if line == 0 { SNOW_LINE_NONE } else { (line - 1) as f32 * 4.0 };
        self.snow_line += (line - self.snow_line).clamp(-dt * 40.0, dt * 40.0);
        let heading = self.world.air.1.to_radians();
        let wind = Vec2::new(heading.sin(), -heading.cos()) * speed;
        self.wind += (wind - self.wind) * (1.0 - (-dt / 4.0).exp());
        if here && self.gloom > 0.9 && weather.lightning() && self.chance() < dt / 12.0 {
            self.lightning = 0.15;
            self.lightnings = self.lightnings.wrapping_add(1);
            // Where the camera faced as it began (radians, clockwise from -z): the pack aims most
            // strikes near it, and the bolt stays put while the player turns.
            let f = self.camera().forward();
            self.lightning_seen = (0.0, self.chance(), f.x.atan2(-f.z));
        }
        if let Some(sound) = weather
            .sound
            .filter(|_| self.look.falls > 0.5 && !self.enclosed())
        {
            if self.chance() < dt {
                let volume = self.look.falls;
                self.sounds
                    .push(crate::audio::Play { sound, pos: None, volume, pitch: 0.0 });
            }
        }
    }

    /// What the wind carries particles by (`gpu_layout::ParticleSimArgs::wind`): the wind where
    /// the player is, how hard it rains, and the shaders' seconds its gusts move by, as the
    /// plants' sway has them.
    pub fn particle_wind(&self) -> [f32; 4] {
        [self.wind.x, self.wind.y, self.rain(), crate::shaders::time()]
    }

    /// The weather where the player is, as drawn (`ClientState::rain` and on, for mods' sound).
    pub fn weather_now(&self) -> (f32, f32, [f32; 2], f32, u32) {
        (self.rain(), self.thunder(), self.wind.to_array(), self.snowing, self.lightnings)
    }

    /// How much falls where the player is, 0..1, as drawn: the weather's look (chiacraft's
    /// `scene.fog_color.w` for packs, mods' rain sound).
    fn rain(&self) -> f32 {
        self.look.falls.clamp(0.0, 1.0)
    }

    /// How far the rain's sky has come where the player is, 0..1, for packs (chiacraft's
    /// `scene.fog_color.w`: its sky's gray, its clouds, the far rain's mist): ahead of what falls,
    /// so the sky darkens before the drops (`weather_effects`), never behind it.
    fn sky_rain(&self) -> f32 {
        let weather = &content().game.weathers[self.weather_id as usize];
        let full = if weather.falls() { weather.look.falls } else { 0.0 };
        (self.gloom * full).max(self.look.falls).clamp(0.0, 1.0)
    }

    /// How stormy it is, 0..1: thunder's clouds go 0.95 dark (`scene.weather.w` for packs).
    fn thunder(&self) -> f32 {
        (self.look.cloud_dark / 0.95).clamp(0.0, 1.0)
    }

    /// Game ticks now, with the fraction of the current one.
    fn smooth_time(&self) -> f64 {
        let fraction = if self.since_time < crate::world::TPS as f64 { self.accum.clamp(0.0, STEP) * self.tick_rate } else { 0.0 };
        self.world.time as f64 + (self.since_time + fraction) * self.day_rate as f64
    }

    /// Fog color (linear): the liquid's from inside one; else vanilla's dimming towards night,
    /// warmed while looking at a sunset.
    fn fog_linear(&self) -> Vec3 {
        if let Some(l) = self.eye_liquid() {
            return self.sky.liquid_color(l);
        }
        let d = &content().dims[self.world.dim as usize];
        let l = self.daylight();
        let c = sky::rained(sky::rgb(d.fog_color) * Vec3::new(l * 0.94 + 0.06, l * 0.94 + 0.06, l * 0.91 + 0.09), 1.0, self.look.fog_gray);
        let forward = self.camera().forward();
        sky::warmed_by_glows(self.world.dim as usize, d, self.smooth_time(), c, forward).powf(2.2)
    }

    /// The frame as every world pass sees it (`gpu_layout::Scene`).
    /// Temporal anti-aliasing's fields (`Scene::taa`, `taa_prev`): this frame's jitter, and last
    /// frame's camera without it, taking this frame's camera-relative positions, with the
    /// shaders' clock then. A camera that moved more than 32 blocks (a teleport, a respawn) or a
    /// clock that went back starts it anew, and so does a frame without it (`post::taa_active`):
    /// `taa.z` 0 then, and the shaders skip their motion.
    fn taa_frame(&mut self, cam: &Camera, frame: &FrameInfo) -> ([f32; 4], [[f32; 4]; 4]) {
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let now = cam.proj_plain(aspect) * cam.view();
        let j = crate::post::jitter();
        let t = crate::shaders::time();
        match self.taa_last.replace((now, cam.pos, t)) {
            Some((last, at, then)) if crate::post::taa_active() && (cam.pos - at).length() < 32.0 && then <= t => {
                let moved = glam::Mat4::from_translation((cam.pos - at).as_vec3());
                ([j.x, j.y, 1.0, then], (last * moved).to_cols_array_2d())
            }
            _ => ([j.x, j.y, 0.0, t], now.to_cols_array_2d()),
        }
    }

    fn scene(&self, cam: &Camera, frame: &FrameInfo) -> Scene {
        // Fog ends where the terrain does: the view, or the distant terrain beyond it.
        let view_distance = (self.view_radius * 16).max(lod::Lod::distance()) as f32;
        // Vanilla's water fog starts behind the eye, lava's right at it.
        let (fog_start, fog_end) = match self
            .eye_liquid()
            .map(|l| content().liquids[l as usize].fog_distance)
        {
            Some(end) => (if end >= 16.0 { -8.0 } else { end * 0.25 }, end),
            None => (view_distance * 0.6, view_distance),
        };
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let view = cam.view();
        let inv = view.inverse();
        // Animated textures follow the game's ticks: between two from the server the client
        // counts on for a second at most, so they stop with the game.
        let ticks = (self.smooth_time() % (1 << 20) as f64) as f32;
        let hand = self.hand_light(cam);
        Scene {
            view_proj: (cam.proj(aspect) * view).to_cols_array_2d(),
            inv_view_proj: (cam.proj(aspect) * view).inverse().to_cols_array_2d(),
            right: inv.transform_vector3(Vec3::X).extend(0.0).to_array(),
            up: inv.transform_vector3(Vec3::Y).extend(0.0).to_array(),
            fog_color: self.fog_color(),
            params: [fog_start, fog_end, self.daylight(), crate::shaders::time()],
            light: [content().dims[self.world.dim as usize].ambient as f32 / 15.0, self.brightness, ticks, 0.0],
            sun: self.sun(),
            weather: [self.wind.x, self.wind.y, self.fog, self.thunder()],
            eye: [cam.pos.x.rem_euclid(8192.0) as f32, cam.pos.y as f32, cam.pos.z.rem_euclid(8192.0) as f32, self.moon_light()],
            ground: [self.snow_cover, self.snowing, self.dust, self.aurora],
            behind: self.behind,
            clouds: self.sky.cloud_shade[0],
            cloud_cells: self.sky.cloud_shade[1],
            moon: self.moon(),
            options: pack_options(),
            hand: hand.0,
            hand_light: hand.1,
            canopy: [self.canopy, self.wet, self.puddles, self.snow_line],
            // Filled in once the sky has readied them (`render`).
            soft_clouds: [0.0; 4],
            // Filled in once the sky has readied it (`render`).
            frame_look: [0.0; 4],
            lightning: [self.lightning_seen.0, self.lightning_seen.1, self.thunder(), self.lightning_seen.2],
            froxels: [0.0; 4],
            upscale: [crate::settings::upscaler() as f32, 0.0, 0.0, 0.0],
            // Filled in by `render` (`taa_frame`).
            taa: [0.0; 4],
            taa_prev: [[0.0; 4]; 4],
            taa_frame: [self.taa_out as f32, crate::post::motion() as u32 as f32, 0.0, 0.0],
            dlss: [crate::post::mip_bias(), 0.0, 0.0, 0.0],
            cloud_shadow: [0.0; 4],
            prev_view_proj: [[0.0; 4]; 4],
            cloud_moved: [0.0; 4],
            // On the real clock (`shaders::clock`): tests pin the shaders' time, and a fireball's
            // countdown and a shower's streaks stood still in their shots.
            sky_event: [self.sky_event.0 as f32, self.sky_event.1, crate::shaders::clock() - self.sky_event.2, 0.0],
            sky_landing: match self.sky_landing {
                Some((at, when)) => (at - cam.pos)
                    .as_vec3()
                    .extend(when - crate::shaders::clock())
                    .to_array(),
                None => [0.0, 0.0, 0.0, -1e6],
            },
            frame: [crate::post::hdr() as u32 as f32, crate::post::bloom() as u32 as f32, self.sky.froxels.cell as f32, {
                // chiacraft: a count of the frames drawn, 0 to 63 and round again, for dithers that
                // change each frame (Far view, 11-06: the cubes' fade into distant terrain).
                // Held under an upscaler (`post::dither_moves`).
                static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                if crate::post::dither_moves() { (FRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 64) as f32 } else { 0.0 }
            }],
            shadow_view_proj: self.shadow.0.to_cols_array_2d(),
            shadow_far_view_proj: self.shadow_far.0.to_cols_array_2d(),
            shadow_far: match self.shadow_far_map {
                Some(_) if self.shadow_far.1 => {
                    [self.shadow_far_index as f32, self.shadow_size as f32, 1.0, 2.0 * self.shadow_far_reach / self.shadow_size as f32]
                }
                _ => [0.0; 4],
            },
            rt: [
                self.render.rt_traced() as u32 as f32,
                (self.render.rt_frame_no() % (1 << 24)) as f32,
                (shadow_setting("core:rt_shadows", 0.0) > 1.5) as u32 as f32,
                self.render.rt_reflects() as u32 as f32,
            ],
            shadow: match self.shadow_map {
                Some(t) if self.shadow.1 => {
                    [self.shadow_index(t) as f32, self.shadow_size as f32, 1.0, 2.0 * SHADOW_REACH / self.shadow_size as f32]
                }
                _ => [0.0; 4],
            },
        }
    }

    /// Toward the sun: it rises in +x and is highest at noon, where the sky draws it: the
    /// dimension's first glowing body that follows the day (its tilt leans the path south, so that
    /// shadows are never straight down and stay long at noon). Without one, a path leaned 19°.
    fn sun(&self) -> [f32; 4] {
        let d = &content().dims[self.world.dim as usize];
        if !d.sky_light {
            return [0.0, 1.0, 0.0, 0.0];
        }
        let time = self.world.time as f64;
        let v = match self.sun_body() {
            Some(s) => sky::frame_of(s, sky::turn_of(s, d, time)).x_axis,
            None => {
                let a = d.day_turn(time) * std::f64::consts::TAU;
                SUN_LEAN * Vec3::new(a.cos() as f32, a.sin() as f32, 0.0)
            }
        };
        [v.x, v.y, v.z, v.y.max(0.0)]
    }

    /// The light the player holds (`Scene::hand`, `Scene::hand_light`): where it is relative to
    /// the camera (by the hand, a little below and in front of the eye, in first person or seen
    /// from behind) and its levels, the most any state of the held item's block gives (a lit
    /// campfire, a torch on any face). Zeros when the item gives no light.
    fn hand_light(&self, cam: &Camera) -> ([f32; 4], [f32; 4]) {
        let none = ([0.0; 4], [0.0; 4]);
        let stack = self.held();
        // Logged when what is held changes, to check the held light from a run's log.
        static LAST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        let changed = LAST.swap(stack.item, std::sync::atomic::Ordering::Relaxed) != stack.item;
        if stack.is_empty() {
            if changed {
                log::info!("held light: nothing held");
            }
            return none;
        }
        let c = content();
        let item = c.items.get(stack.item as usize);
        // The block it places (`ItemDef::block` is that block's first state, not the block), or else
        // the block of the same name (an item that places itself in more than one form may not
        // name one).
        let block = item
            .and_then(|i| i.block)
            .and_then(|st| c.states.get(st as usize))
            .map(|st| st.block)
            .or_else(|| {
                let name = item?.name.as_str();
                // Item and block names may differ in their namespace ("base:torch" and "torch").
                let short = |n: &str| n.rsplit(':').next().unwrap_or(n).to_string();
                let want = short(name);
                c.blocks
                    .iter()
                    .position(|b| b.name == name || short(&b.name) == want)
                    .map(|b| b as u32)
            });
        let Some(def) = block.and_then(|b| c.blocks.get(b as usize)) else {
            if changed {
                log::info!("held light: item {} ({}) places no block", stack.item, item.map_or("?", |i| i.name.as_str()));
            }
            return none;
        };
        let mut light = [0u8; 3];
        for st in def.first_state..def.first_state + def.states {
            if let Some(info) = c.states.get(st as usize) {
                for (l, e) in light.iter_mut().zip(info.light) {
                    *l = (*l).max(e);
                }
            }
        }
        if changed {
            log::info!("held light: item {} block {} levels {:?}", stack.item, def.name, light);
        }
        if light == [0; 3] {
            return none;
        }
        let eye = self.camera();
        let at = (eye.pos - cam.pos).as_vec3() + eye.forward() * 0.35 - Vec3::Y * 0.35;
        (at.extend(0.0).to_array(), [light[0] as f32 / 15.0, light[1] as f32 / 15.0, light[2] as f32 / 15.0, 0.0])
    }

    /// The sky's sun (`sun`): the first glowing body following the day.
    fn sun_body(&self) -> Option<&'static crate::registry::SkyDef> {
        content()
            .game
            .sky
            .get(self.world.dim as usize)?
            .iter()
            .find(|s| s.kind == weft_abi::sky_kind::BODY && s.glow != 0 && s.orbit.is_infinite())
    }

    /// The sky's moon (`moon`): the first other body (the base game's sun, then its moon).
    fn moon_body(&self) -> Option<&'static crate::registry::SkyDef> {
        let sun = self.sun_body();
        content()
            .game
            .sky
            .get(self.world.dim as usize)?
            .iter()
            .find(|s| s.kind == weft_abi::sky_kind::BODY && !sun.is_some_and(|x| std::ptr::eq(x, *s)))
    }

    /// Toward the moon where the sky draws it, and in w how much light it gives: its phase
    /// (`moon_light`, 1 full, 0 new) times how high it stands (none below the horizon). A moon that
    /// lags the day by its phase is up all night when full, half the night at a half moon and only
    /// by day when new (`Scene::moon`). Without a moon body, opposite the sun.
    fn moon(&self) -> [f32; 4] {
        let d = &content().dims[self.world.dim as usize];
        if !d.sky_light {
            return [0.0, -1.0, 0.0, 0.0];
        }
        let v = match self.moon_body() {
            Some(s) => sky::frame_of(s, sky::turn_of(s, d, self.world.time as f64)).x_axis,
            None => {
                let s = self.sun();
                -Vec3::new(s[0], s[1], s[2])
            }
        };
        let up = ((v.y + 0.02) / 0.14).clamp(0.0, 1.0);
        let up = up * up * (3.0 - 2.0 * up);
        [v.x, v.y, v.z, self.moon_light() * up]
    }

    /// The plane the moon turns in (`settled`).
    fn moon_plane(&self) -> glam::Mat3 {
        self.moon_body().map_or(SUN_LEAN, |s| sky::frame_of(s, 0.0))
    }

    /// The plane the sun turns in (its x and y axes; `settled`).
    fn sun_plane(&self) -> glam::Mat3 {
        self.sun_body().map_or(SUN_LEAN, |s| sky::frame_of(s, 0.0))
    }

    /// The moon's light by its phase, 1 full to 0 new: vanilla's eight phases, one a day, the one
    /// the sky's moon picture shows (`sky.rs`).
    fn moon_light(&self) -> f32 {
        let d = &content().dims[self.world.dim as usize];
        let day = if d.day_length != 0 { d.day_length as f64 } else { 24000.0 };
        let phase = (self.smooth_time() / day).floor().rem_euclid(8.0);
        (0.5 + 0.5 * (phase * std::f64::consts::TAU / 8.0).cos()) as f32
    }

    /// The shadow map's bindless index (set when it was made).
    fn shadow_index(&self, _t: weft_render::TextureId) -> u32 {
        self.shadow_index
    }

    /// `core:rt_shadows` turned on or off: the scene made (or let go), and whether the world shader
    /// has to be composed again (the app does it before anything of the frame draws with it).
    pub fn rt_switch(&mut self, gpu: &mut dyn Backend) -> bool {
        let shadows = shadow_setting("core:rt_shadows", 0.0) > 0.5;
        let reflections = shadow_setting("core:rt_reflections", 0.0) > 0.5;
        let traced = shadow_setting("core:path_traced", 0.0) > 0.5;
        self.render.set_rt_modes(shadows, reflections);
        self.render.set_rt(gpu, shadows || reflections || traced);
        self.render.rt_reload
    }

    /// Path traced lighting (`core:path_traced`, `WorldRenderer::path_trace`), after the world's
    /// pass into `target` (the world's image, its size `size`) and before the effects: first the
    /// G-buffer pass (`WorldRenderer::gbuffer_pass`), then the traced light.
    pub fn path_trace(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        slot: usize,
        depth: weft_render::TextureId,
        target: weft_render::Target,
        size: (u32, u32),
    ) {
        if shadow_setting("core:path_traced", 0.0) > 0.5 {
            let debug = shadow_setting("core:pt_debug", 0.0) as u32;
            // The G-buffer (albedo, normal and roughness, spec) the denoisers read: `render.gb_images`.
            self.render.gbuffer_pass(gpu, list, depth, size);
            self.render
                .path_trace(gpu, list, slot, depth, target, size, debug, false);
        }
    }

    /// Sun shadows (`sun_shadows` packs), first their light, before the world is drawn (its scene
    /// samples the map with it): from the sun, or the moon once it has set, an orthographic box
    /// `SHADOW_REACH` blocks to each side of the camera along the light, moved in whole texels so
    /// its edges do not crawl as the camera moves. `render_shadows` then draws the map.
    pub fn prepare_shadows(&mut self, gpu: &mut dyn Backend) {
        self.shadow.1 = false;
        self.shadow_far.1 = false;
        // `--no-shadows`: a benchmark leaving the shadow map out.
        let level = shadow_setting("core:shadows", 2.0) as u32;
        if level == 0
            || !self.render.has_shadows()
            || !content().dims[self.world.dim as usize].sky_light
            || crate::app::api::launch_options().contains_key("no-shadows")
        {
            return;
        }
        // A new size: both maps are made again at it.
        let size = match shadow_setting("core:shadow_size", 2048.0) as u32 {
            0..=1535 => 1024,
            1536..=3071 => 2048,
            _ => 4096,
        };
        if size != self.shadow_size {
            for t in [self.shadow_map.take(), self.shadow_far_map.take()]
                .into_iter()
                .flatten()
            {
                gpu.destroy_texture(t);
            }
            self.shadow_size = size;
        }
        // By day the sun's shadows, by night the moon's where it stands (none when it is down).
        let sun = self.sun();
        let moon = self.moon();
        let dir = if sun[1] > -0.06 {
            settled(Vec3::new(sun[0], sun[1], sun[2]), self.sun_plane())
        } else {
            settled(Vec3::new(moon[0], moon[1], moon[2]), self.moon_plane())
        };
        // So low the shadows would run off the map (the pack fades them out above this).
        if dir.y < 0.02 {
            return;
        }
        let map = |gpu: &mut dyn Backend, label: &str| {
            gpu.create_texture(
                &weft_render::TextureDesc { width: size, height: size, mips: 1, format: weft_render::Format::Depth32, render_target: true },
                label,
            )
        };
        if self.shadow_map.is_none() {
            let t = map(gpu, "shadow map");
            self.shadow_index = gpu.texture_index(t);
            self.shadow_map = Some(t);
        }
        if self.shadow_far_map.is_none() {
            let t = map(gpu, "far shadow map");
            self.shadow_far_index = gpu.texture_index(t);
            self.shadow_far_map = Some(t);
            self.shadow_far_drawn = None;
        }
        // The far map's reach (`core:shadow_far`: auto 2000 with the HDR world, 400 without; to 2400):
        // drawn again when it changes.
        let set = shadow_setting("core:shadow_far", 0.0) as f32;
        let auto = if crate::post::hdr() { 2000.0 } else { FAR_REACH };
        let reach = if set <= 0.0 { auto } else { set.clamp(200.0, 2400.0) };
        if reach != self.shadow_far_reach {
            self.shadow_far_reach = reach;
            self.shadow_far_drawn = None;
        }
        let cam = self.render_camera();
        self.shadow = (light_box(dir, cam.pos, SHADOW_REACH, SHADOW_DEPTH, size), true);
        // The far map about a place in the world, not the camera: built about the camera, its box
        // moved by a fraction of a texel every frame the player moved, and it was drawn again each
        // frame (Research round 16, #8: +4,500 draw records a frame walking, about 0.6 ms on the
        // Mac). Snapped to `FAR_ANCHOR` blocks, it stays put until the player crosses into
        // another cell; seen from the camera it is the same box, moved by camera less anchor.
        let anchor = ((cam.pos / FAR_ANCHOR).floor() + 0.5) * FAR_ANCHOR;
        let anchored = light_box(dir, anchor, reach, reach * FAR_DEPTH / FAR_REACH, size);
        self.shadow_far_anchored = anchored;
        let from_cam = glam::Mat4::from_translation((cam.pos - anchor).as_vec3());
        self.shadow_far = (anchored * from_cam, level >= 2);
    }

    /// The sun's shadow map, after `render` posed this frame's models (the pass is put before the
    /// world's in the frame's list): the terrain and the models, the player's own body too.
    pub fn render_shadows(&mut self, gpu: &mut dyn Backend, list: &mut CommandList, frame: &FrameInfo) {
        let (Some(map), Some(far_map)) = (self.shadow_map, self.shadow_far_map) else { return };
        let cam = self.render_camera();
        while self.shadow_bufs.len() < gpu.frames_in_flight() {
            self.shadow_bufs.push(gpu.create_buffer(
                size_of::<Scene>() as u64,
                weft_render::usage::STORAGE | weft_render::usage::HOST,
                "shadow scene",
            ));
        }
        let mut scene = self.scene(&cam, frame);
        // The map is the pass's target: nothing in it may sample it.
        scene.shadow = [0.0; 4];
        if let (light, true) = self.shadow {
            self.render_near_shadows(gpu, list, frame, &cam, light, map, scene);
        }
        self.render_far_shadows(gpu, list, frame, &cam, far_map, scene);
    }

    /// The near cascade: the terrain and the models, the player's own body too.
    #[allow(clippy::too_many_arguments)]
    fn render_near_shadows(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        light: glam::Mat4,
        map: weft_render::TextureId,
        mut scene: Scene,
    ) {
        scene.view_proj = light.to_cols_array_2d();
        let buf = self.shadow_bufs[frame.slot];
        gpu.mapped(buf)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&scene));
        list.push(weft_render::Cmd::BeginPass { motion: None, color: None, depth: Some((map, Some(0.0))) });
        // Leaning structures (chiacraft, a falling tree) cast theirs, turned.
        let leaning: Vec<(u64, DVec3, (Vec3, glam::Quat))> = self
            .entities
            .iter()
            .filter(|(_, e)| e.structure.is_some())
            .filter_map(|(&id, e)| {
                e.lean
                    .map(|l| (id, physics::Transform::of(e.pos).origin(), (l.pivot, l.now())))
            })
            .collect();
        // With traced shadows (`core:rt_shadows`) the terrain casts by its rays: the near map then
        // holds the models alone (mobs, the player), which the world's rays don't see.
        if !self.render.rt_traced() {
            self.render
                .draw_shadow(gpu, list, frame, cam, light, buf, 0, &leaning);
        }
        self.entity_render.draw_shadow(gpu, list, frame, buf);
        list.push(weft_render::Cmd::EndPass);
    }

    /// The far cascade: the terrain alone, coarser, to where hills stand far off.
    fn render_far_shadows(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        far_map: weft_render::TextureId,
        mut scene: Scene,
    ) {
        let (far, true) = self.shadow_far else { return };
        // Kept from an earlier frame while neither the light about its anchor (in whole texels and
        // `settled` steps) nor the terrain has moved: walking within a cell, the far map costs
        // nothing. Terrain that loads (every mesh bumps `terrain_epoch`) waits FAR_TERRAIN_WAIT
        // frames, so streaming chunks don't redraw it every frame.
        self.shadow_far_age = self.shadow_far_age.saturating_add(1);
        let drawn = (self.shadow_far_anchored.to_cols_array(), self.render.terrain_epoch);
        let light_moved = self.shadow_far_drawn.is_none_or(|d| d.0 != drawn.0);
        let terrain_moved = self.shadow_far_drawn.is_some_and(|d| d.1 != drawn.1);
        if !light_moved && (!terrain_moved || self.shadow_far_age < FAR_TERRAIN_WAIT) {
            return;
        }
        self.shadow_far_drawn = Some(drawn);
        self.shadow_far_age = 0;
        while self.shadow_far_bufs.len() < gpu.frames_in_flight() {
            self.shadow_far_bufs.push(gpu.create_buffer(
                size_of::<Scene>() as u64,
                weft_render::usage::STORAGE | weft_render::usage::HOST,
                "far shadow scene",
            ));
        }
        scene.view_proj = far.to_cols_array_2d();
        scene.shadow_far = [0.0; 4];
        let far_buf = self.shadow_far_bufs[frame.slot];
        gpu.mapped(far_buf)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&scene));
        list.push(weft_render::Cmd::BeginPass { motion: None, color: None, depth: Some((far_map, Some(0.0))) });
        self.render
            .draw_shadow(gpu, list, frame, cam, far, far_buf, 1, &[]);
        list.push(weft_render::Cmd::EndPass);
    }

    /// `list` goes in the world's pass, `early` before it.
    pub fn render(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        early: &mut CommandList,
        frame: &FrameInfo,
        textures: &GameTextures,
    ) {
        self.entity_render.local = self.player_id;
        for (entity, png) in std::mem::take(&mut self.entity_render.skin_pending) {
            self.entity_render.set_skin(gpu, entity, &png);
        }
        let cam = self.render_camera();
        // Ray tracing's scene (`core:rt_shadows`): this frame's builds and its TLAS, before the
        // world's draws that bind it (`app` switched the world shader before this, `rt_switch`).
        self.render.rt_frame(gpu, early, frame.slot, &cam);
        let mut scene = self.scene(&cam, frame);
        (scene.taa, scene.taa_prev) = self.taa_frame(&cam, frame);
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        self.frame_camera = Some(FrameCamera {
            view: cam.view(),
            proj: cam.proj_plain(aspect),
            prev: glam::Mat4::from_cols_array_2d(&scene.taa_prev),
            fresh: scene.taa[2] == 0.0,
            fov_y: cam.fov.to_radians(),
            aspect,
        });
        // The calm world pipelines (`gpu_layout::CALM_OFF`) while nothing they leave out is on, read
        // from the scene the shaders see: no fog in the air, no thunder, no held light, the eye in
        // no liquid (the fog then starts at 0.6 of its end).
        let hand = scene.hand_light;
        let calm = scene.weather[2] <= 0.0
            && scene.lightning[2] < 0.05
            && hand[0].max(hand[1]).max(hand[2]) <= 0.0
            && (scene.params[0] - 0.6 * scene.params[1]).abs() <= 0.5;
        // And dry (`gpu_layout::DRY_OFF`): no rain, the ground dry, no puddles, no snow lying here.
        let dry = scene.fog_color[3] <= 0.0 && scene.canopy[1] <= 0.0 && scene.canopy[2] <= 0.0 && scene.ground[0] <= 0.0;
        self.render.set_calm(if calm { 1 + dry as u8 } else { 0 });
        while self.scene_bufs.len() < gpu.frames_in_flight() {
            self.scene_bufs.push(gpu.create_buffer(
                size_of::<Scene>() as u64,
                weft_render::usage::STORAGE | weft_render::usage::HOST,
                "scene",
            ));
        }
        let scene_buf = self.scene_bufs[frame.slot];
        gpu.mapped(scene_buf)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&scene));
        // Opaque blocks, then entities and cutout particles, then what is seen through:
        // translucent blocks, cracks and gizmo fills, blended particles, and the hand.
        let frames: Vec<(u64, DVec3)> = self
            .entities
            .iter()
            .filter(|(_, e)| e.structure.is_some())
            .map(|(&id, e)| (id, physics::Transform::of(e.pos).origin()))
            .collect();
        // The world's light on each, as the server worked it out when it got there.
        let lights: rustc_hash::FxHashMap<(u64, IVec3), u32> = frames
            .iter()
            .filter_map(|(id, _)| self.entities.get(id).map(|e| (*id, e)))
            .flat_map(|(id, e)| e.world_light.iter().map(move |(cp, l)| ((id, *cp), *l)))
            .collect();
        // How each leans (chiacraft): its pivot and rotation now.
        let leans: rustc_hash::FxHashMap<u64, (Vec3, glam::Quat)> = self
            .entities
            .iter()
            .filter_map(|(&id, e)| e.lean.map(|l| (id, (l.pivot, l.now()))))
            .collect();
        let s = crate::diag::span("render/world");
        self.render
            .draw(gpu, list, early, frame, &cam, scene_buf, &frames, &lights, &leans);
        drop(s);
        let lit: Vec<u32> = frames
            .iter()
            .map(|(id, _)| {
                self.entities
                    .get(id)
                    .map_or(crate::world::light::SKY_FULL as u32, |e| {
                        e.world_light
                            .values()
                            .copied()
                            .max()
                            .unwrap_or(crate::world::light::SKY_FULL as u32)
                    })
            })
            .collect();
        self.wires.draw(list, &cam, scene_buf, &frames, &lit);
        list.push(weft_render::Cmd::Mark);
        let mut late = std::mem::take(&mut self.late);
        late.clear();
        // The sky fills what the opaque world left, and translucent things are seen against it.
        let overlays = self.overlays(textures);
        let s = crate::diag::span("render/sky");
        // Soft clouds trace against the copy of the world (`behind`), made after this.
        self.sky.soft_possible = self.behind[2] > 0.5;
        // The mist's volume, when there is mist and the eye is out of liquids (`fog_through`).
        self.sky.froxels_possible = self.fog >= 0.005 && self.eye_liquid().is_none();
        self.sky.draw(
            gpu,
            &mut late,
            frame,
            &cam,
            scene_buf,
            self.world.dim as usize,
            self.smooth_time(),
            self.camera().pos.y,
            self.eye_liquid().is_some(),
            &self.look,
            self.wind,
            self.frame_dt,
            &overlays,
        );
        drop(s);
        // The sky readied this frame's soft clouds: their fields in the scene.
        (scene.soft_clouds, scene.prev_view_proj, scene.cloud_moved) = self.sky.soft.scene;
        scene.frame_look = self.sky.frame_look.scene;
        scene.froxels = self.sky.froxels.scene;
        scene.cloud_shadow = self.sky.cloud_shadow.scene;
        gpu.mapped(scene_buf)[..size_of::<Scene>()].copy_from_slice(bytemuck::bytes_of(&scene));
        self.render.draw_translucent(&mut late);
        let alive = !self.dead && !self.mode.has(weft_abi::game_mode_flags::NO_INTERACT);
        let hand = (alive && self.camera_has(weft_abi::camera_flags::HAND) && !self.hide_gui).then(|| entity_render::Hand {
            lowered: 1.0 - self.hand_height,
            lag: (self.player.pitch - self.hand_view.0, angle_diff(self.player.yaw, self.hand_view.1)),
            body: self.first_person_model,
        });
        let me = (alive && (self.camera_has(weft_abi::camera_flags::BODY) || hand.is_some())).then_some(&self.me);
        // Where the camera doesn't draw the player's body (first person), it still casts its
        // shadow (chiacraft, sun shadows).
        let shadow_me = (self.shadow.1 && alive && !self.camera_has(weft_abi::camera_flags::BODY)).then_some(&self.me);
        let world = &self.world;
        let light = |p: IVec3| world.light(p);
        let mut hand_late = std::mem::take(&mut self.hand_late);
        hand_late.clear();
        let mut cracks: Vec<entity_render::Crack> = self
            .others_mining
            .iter()
            .map(|&(_, pos, state, progress)| entity_render::Crack { pos, progress, at: DVec3::ZERO, state })
            .collect();
        cracks.extend(
            self.breaking
                .map(|(f, pos, progress)| entity_render::Crack {
                    pos,
                    progress,
                    at: self.frame_place(f).origin(),
                    state: self.breaking_part,
                }),
        );
        let s = crate::diag::span("render/entities");
        self.entity_render.draw(
            gpu,
            list,
            frame,
            &cam,
            &self.entities,
            me,
            shadow_me,
            hand,
            textures,
            &light,
            &scene,
            scene_buf,
            self.target
                .filter(|_| !self.hide_gui)
                .map(|h| (h, self.frame_place(h.frame).origin())),
            &cracks,
            &self.gizmo_lines,
            &self.gizmo_fills,
            &mut hand_late,
        );
        drop(s);
        let s = crate::diag::span("render/particles and weather");
        self.particles.draw(list, &mut late, scene_buf);
        let ticks = self.smooth_time();
        self.weather_render
            .draw(gpu, &mut late, frame, &cam, scene_buf, &self.world, self.look.falls, ticks, self.frame_dt);
        drop(s);
        self.sky.draw_overlay(&mut hand_late, scene_buf);
        if self.behind[2] > 0.5 {
            // After the copy of the world (the app puts it there).
            self.late_out.clear();
            self.late_out.append(&mut late);
            self.late_out.append(&mut hand_late);
        } else {
            list.append(&mut late);
            list.append(&mut hand_late);
        }
        (self.late, self.hand_late) = (late, hand_late);
        // The end of the "translucent" part; with the copy, the app marks it after what it draws
        // after the copy, so the sky, clouds and water are not counted as screen effects (Research
        // round 16, M1).
        if self.behind[2] <= 0.5 {
            list.push(weft_render::Cmd::Mark);
        }
    }

    /// Linear rgb, and in w how hard it rains (0..1) for shader packs.
    /// The buffer this frame's world was drawn with (`gpu_layout::Scene`), once it was.
    pub fn scene_buffer(&self, slot: usize) -> Option<weft_render::BufferId> {
        self.scene_bufs.get(slot).copied()
    }

    pub fn fog_color(&self) -> [f32; 4] {
        self.fog_linear().extend(self.sky_rain()).to_array()
    }
}

/// The sun's shadow map: texels a side, blocks it reaches to each side of the camera, and the
/// depth of its box along the light (half of it on the sun's side of the camera).
/// A shadow setting (`core:shadows`, `core:shadow_size`), or `default` where the engine's configs
/// are not loaded.
fn shadow_setting(name: &str, default: f64) -> f64 {
    content()
        .game
        .config_names
        .get(name)
        .map_or(default, |&id| crate::config::get(id))
}
/// The far cascade's reach and depth (blocks): hills a few hundred blocks off have shadows too.
const FAR_REACH: f32 = 400.0;
/// The far shadow map's anchor snaps to this many blocks (`prepare_shadows`): its 400 blocks to
/// each side cover the camera wherever it stands in the cell.
const FAR_ANCHOR: f64 = 32.0;
/// Seconds the weather's sky takes to gather (or clear) where the player is, and the rain to come
/// in once it has (`Client::weather_effects`); and how far apart the places are that it asks for
/// rain round the player (blocks: three across span about three chunks).
const GLOOM_SECS: f32 = 90.0;
const FALLS_SECS: f32 = 25.0;
const WEATHER_REACH: i32 = 24;

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Frames the far map waits before drawing in terrain that changed (about a sixth of a second).
const FAR_TERRAIN_WAIT: u32 = 10;
const FAR_DEPTH: f32 = 1200.0;

/// The values of the settings the shader pack reads (`Scene::options`).
fn pack_options() -> [[f32; 4]; 8] {
    let mut out = [[0.0; 4]; 8];
    let names = &content().game.config_names;
    for (i, name) in crate::packs::pack_options().iter().enumerate().take(32) {
        if let Some(&id) = names.get(name.as_str()) {
            out[i / 4][i % 4] = crate::config::get(id) as f32;
        }
    }
    out
}

/// The sun's path leaned south when the sky has no sun to follow (`Client::sun`).
const SUN_LEAN: glam::Mat3 = glam::Mat3::from_cols(Vec3::X, Vec3::new(0.0, 0.944, 0.33), Vec3::new(0.0, -0.33, 0.944));

/// The light's direction in steps of 1/2000 of its turn (0.18°, a step every 0.6 s of a 20-minute
/// day) along its path (`plane`, the sun's x and y axes): between steps the shadow maps' light
/// stays put, so a map can be kept (`render_shadows`). A step moves the shadow of a 20-block tree
/// by about 0.06 blocks.
fn settled(dir: Vec3, plane: glam::Mat3) -> Vec3 {
    let step = std::f32::consts::TAU / 2000.0;
    let l = plane.transpose() * dir;
    let a = (l.y.atan2(l.x) / step).round() * step;
    let r = glam::Vec2::new(l.x, l.y).length();
    (plane * Vec3::new(a.cos() * r, a.sin() * r, l.z)).normalize()
}

/// From camera-relative positions to an orthographic box along the light `dir` (toward it), `reach`
/// blocks to each side of the camera and `depth` deep (reverse depth: 1 nearest the light), moved in
/// whole texels of the world so its edges do not crawl as the camera moves.
fn light_box(dir: Vec3, cam: glam::DVec3, reach: f32, depth: f32, size: u32) -> glam::Mat4 {
    let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
    // Looking from the light's side of the camera toward it (right-handed, -z forward).
    let eye = dir * depth * 0.5;
    let f = -dir;
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    let view = glam::Mat4::from_cols(
        glam::Vec4::new(s.x, u.x, -f.x, 0.0),
        glam::Vec4::new(s.y, u.y, -f.y, 0.0),
        glam::Vec4::new(s.z, u.z, -f.z, 0.0),
        glam::Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
    );
    // The camera's place across the light, in whole texels of the world (doubles: far from the
    // world's origin a float would lose them).
    let texel = 2.0 * reach as f64 / size as f64;
    let right = view.row(0).truncate().as_dvec3();
    let upward = view.row(1).truncate().as_dvec3();
    let across = glam::DVec2::new(cam.dot(right), cam.dot(upward));
    let off = (across - (across / texel).floor() * texel).as_vec2();
    let proj = glam::Mat4::from_cols(
        glam::Vec4::new(1.0 / reach, 0.0, 0.0, 0.0),
        glam::Vec4::new(0.0, 1.0 / reach, 0.0, 0.0),
        glam::Vec4::new(0.0, 0.0, 1.0 / depth, 0.0),
        glam::Vec4::new(off.x / reach, off.y / reach, 1.0, 1.0),
    );
    proj * view
}
const SHADOW_REACH: f32 = 80.0;
const SHADOW_DEPTH: f32 = 400.0;

/// Vanilla's tickHeadTurn: the body turns towards where it walks (or stays put), and the head
/// drags it along only once it has turned more than 50 degrees away. Radians, `moved` this frame.
pub(crate) fn turn_body(body: f32, head: f32, moved: DVec3, dt: f32) -> f32 {
    let mut body = body;
    let flat = moved.with_y(0.0);
    if flat.length_squared() > (0.05 * dt as f64 * 20.0).powi(2) * 0.0025 {
        // Backing up, the body faces the way the head does.
        let mut toward = (-flat.x as f32).atan2(-flat.z as f32);
        if angle_diff(toward, head).abs() > 95f32.to_radians() {
            toward += std::f32::consts::PI;
        }
        body += angle_diff(toward, body) * (1.0 - 0.7f32.powf(dt * crate::world::TPS as f32));
    }
    let off = angle_diff(head, body);
    let limit = 50f32.to_radians();
    if off.abs() > limit {
        body += off - limit * off.signum();
    }
    // Keep it near the head, so their difference never needs wrapping where it is used.
    head - angle_diff(head, body)
}

/// Vanilla's swing starts over once past halfway.
fn restart_swing(swing: &mut Option<f32>) {
    if swing.is_none_or(|p| p >= 0.5) {
        *swing = Some(0.0);
    }
}

/// An arm's swing goes on, 0.3 seconds from start to end.
fn advance_swing(swing: &mut Option<f32>, dt: f32) {
    *swing = swing.map(|p| p + dt / 0.3).filter(|&p| p < 1.0);
}

/// `a - b` the short way around, radians.
fn angle_diff(a: f32, b: f32) -> f32 {
    (a - b + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// One 20 Hz tick of player movement with vanilla's numbers (blocks per tick): acceleration
/// scaled by ground slipperiness, 0.91 air drag, 0.98 vertical drag, 0.08 gravity, 0.42 jump,
/// so jumps, sprint-jumps and parkur distances match.
/// By liquid, whether `kind` wearing `worn` swims in it: what its `comp::SWIMS_IN` and theirs name.
fn swims_in(c: &crate::content::Content, kind: u32, worn: &[u32]) -> Vec<bool> {
    use crate::components as comps;
    let Some(id) = c.components.id(weft_abi::comp::SWIMS_IN) else { return vec![false; c.liquids.len()] };
    let words: Vec<&str> = c
        .components
        .text(comps::ENTITY, kind, id)
        .into_iter()
        .chain(
            worn.iter()
                .filter_map(|&i| c.components.text(comps::ITEM, i, id)),
        )
        .flat_map(str::split_whitespace)
        .collect();
    c.liquids
        .iter()
        .map(|l| {
            words.iter().any(|w| match w.strip_prefix('#') {
                Some(tag) => c
                    .components
                    .id(tag)
                    .is_some_and(|t| c.components.has(comps::BLOCK, l.first_state, t)),
                None => *w == l.name,
            })
        })
        .collect()
}

/// Where on a player a rope holds it: its hands over its head, as a share of its height.
const ROPE_HOLD: f64 = 1.05;
/// The air's drag a step on a body swinging on a taut rope (0.91 off it).
const ROPE_DRAG: f64 = 0.995;

fn step_player(p: &mut LocalPlayer, input: &Input, block: &impl physics::Cells, gravity: f64, bodies: &[physics::Body]) {
    let c = content();
    // Held facing one way it stays put: in a bed.
    if let Some((pose, Some(yaw))) = p.forced {
        (p.yaw, p.pitch, p.vel, p.pose) = (yaw, 0.0, DVec3::ZERO, pose);
        p.eye = p.dims().eye;
        return;
    }
    // The body the player rides moves it first, like on the server.
    let hw = p.dims().half_width;
    if !p.noclip {
        let h = p.dims().height;
        let moved_by = physics::carried(c, block, &mut p.pos, hw, h, p.on_ground, &mut p.support, bodies, 0);
        if let Some(d) = moved_by.released {
            p.vel += d;
        }
    } else {
        p.support = None;
    }
    let mut fwd_in = 0.0f64;
    let mut side_in = 0.0f64;
    if input.held("core:forward") {
        fwd_in += 1.0;
    }
    if input.held("core:back") {
        fwd_in -= 1.0;
    }
    if input.held("core:left") {
        side_in -= 1.0;
    }
    if input.held("core:right") {
        side_in += 1.0;
    }
    let sneak = input.held("core:sneak");
    let crawl = input.held("core:crawl");
    let jump = input.held("core:jump");
    // Sprinting needs forward movement and stops when walking backwards, sneaking or crawling.
    let sprint = input.held("core:sprint") && fwd_in > 0.0 && !sneak && !crawl && !p.using && (p.can_sprint || p.flying);
    // A liquid it swims in, at the eyes, or at the feet once it lies down.
    let low = p.pose.def().tilt != weft_abi::pose_tilt::UPRIGHT;
    let swim_at = p.pos + DVec3::Y * if low { 0.1 } else { p.dims().eye };
    let in_swim_liquid = physics::liquid_at(c, block, swim_at).is_some_and(|l| p.swims_in.get(l as usize) == Some(&true));
    let feet_wet = physics::in_liquid(c, block, p.pos, 0.0);
    // `input`: the action held and sprinting count, which a pose fallen back to goes without.
    let holds = |d: &crate::registry::PoseDef, input_too: bool| {
        use weft_abi::pose_when as w;
        let on = |f: u32| d.when & f != 0;
        (!input_too || ((d.key.is_empty() || input.held(&d.key)) && (!on(w::SPRINTING) || sprint)))
            && (!on(w::NOT_FLYING) || !(p.flying || p.noclip))
            && (!on(w::SWIM_LIQUID) || in_swim_liquid)
            && (!on(w::CLIMBING) || p.on_wall)
            && (!on(w::GLIDING) || p.gliding)
            && (!on(w::ROPE) || p.on_rope)
    };
    let want = match p.forced {
        Some((held, _)) => held,
        None => c
            .game
            .pose_order
            .iter()
            .copied()
            .find(|q| holds(q.def(), true))
            .unwrap_or(Pose::STAND),
    };
    let fits =
        |pose: Pose| p.noclip || !physics::collides(c, block, physics::Aabb::around(p.pos, hw - 1e-3, physics::posed(p.size, pose).height));
    p.pose = if fits(want) {
        want
    } else {
        let back = &want.def().fallback;
        back.iter()
            .copied()
            .find(|&q| holds(q.def(), false) && fits(q))
            .or(back.last().copied())
            .unwrap_or(want)
    };
    // Sprinting swims in a pose for it; in a liquid it is no use otherwise.
    let swimming = sprint && p.pose.def().when & weft_abi::pose_when::SWIM_LIQUID != 0;
    let sprint = sprint && (swimming || !feet_wet);
    p.sprinting = sprint;
    if !p.flying {
        let speed = p.pose.def().speed as f64;
        fwd_in *= speed;
        side_in *= speed;
    }
    // Eating and the like: modern's fifth of the speed.
    if p.using {
        fwd_in *= 0.2;
        side_in *= 0.2;
    }
    let (fwd_in, side_in) = (fwd_in * 0.98, side_in * 0.98);
    let (sy, cy) = (p.yaw as f64).sin_cos();
    let forward = DVec3::new(-sy, 0.0, -cy);
    let right = DVec3::new(cy, 0.0, -sy);
    // Vanilla's moveRelative input: normalized only when longer than 1.
    let wish = forward * fwd_in + right * side_in;
    let g = physics::gravity_per_tick(gravity);

    let height = p.dims().height;
    let before = p.pos;

    if p.flying || p.noclip {
        let fly = 0.05 * p.fly_speed as f64 * if sprint { 2.0 } else { 1.0 };
        if jump {
            p.vel.y += fly * 3.0;
        }
        if sneak {
            p.vel.y -= fly * 3.0;
        }
        let len2 = wish.length_squared();
        if len2 > 1e-7 {
            p.vel += wish * (fly / len2.sqrt().max(1.0));
        }
        if p.noclip {
            p.pos += p.vel;
            p.on_ground = false;
        } else {
            let r = physics::move_box_with(c, block, bodies, 0, p.pos, hw, height, p.vel, 0.0);
            p.pos = r.pos;
            for i in 0..3 {
                if r.hit[i] {
                    p.vel[i] = 0.0;
                }
            }
            p.on_ground = r.on_ground;
            p.landed |= p.on_ground;
            // Flying creative players stop flying when they come down onto the ground.
            if r.on_ground && r.hit[1] {
                p.flying = false;
            }
        }
        let here = physics::around(c, block, p.pos, height);
        (p.in_liquid, p.climbing) = (here.in_liquid, here.climbing);
        p.vel.y *= 0.6;
        p.vel.x *= 0.91;
        p.vel.z *= 0.91;
        return;
    }

    // Gliding: the air carries it, it steers by its look; holding back brakes, sneaking dives.
    p.gliding = p.glide.speed > 0.0 && !p.on_ground && !p.in_liquid && !p.on_wall && p.riding.is_none() && p.rope.is_none();
    if p.gliding {
        let brake = if fwd_in < 0.0 { 1.0 } else { 0.0 };
        let dive = if sneak { 1.0 } else { 0.0 };
        physics::glide(&mut p.vel, forward, &p.glide, brake, dive);
        let r = physics::move_box_with(c, block, bodies, 0, p.pos, hw, height, p.vel, 0.0);
        p.pos = r.pos;
        for i in 0..3 {
            if r.hit[i] {
                p.vel[i] = 0.0;
            }
        }
        p.on_ground = r.on_ground;
        p.landed |= p.on_ground;
        let here = physics::around(c, block, p.pos, height);
        (p.in_liquid, p.climbing) = (here.in_liquid, here.climbing);
        return;
    }

    // Walking is the same step mobs take; the player only brings its input.
    p.jump_cooldown = p.jump_cooldown.saturating_sub(1);
    if !jump {
        p.jump_cooldown = 0;
    }
    let ctl = physics::Control {
        wish,
        ground: if sprint { 0.13 } else { 0.1 } * p.speed_mul,
        air: if sprint { 0.026 } else { 0.02 },
        jump: if jump && p.jump_cooldown == 0 { physics::JUMP * p.jump_mul } else { 0.0 },
        jump_boost: if sprint { forward * 0.2 } else { DVec3::ZERO },
        swim: jump,
        careful: sneak,
        sprint: swimming,
        swim_look: swimming.then(|| (p.pitch as f64).sin()),
        climb: p.climb,
    };
    let frame = physics::Frame { half_width: hw, height, step: physics::STEP_UP, gravity: g };
    let w = physics::walk(c, block, bodies, 0, &mut p.pos, &mut p.vel, p.on_ground, &ctl, &frame);
    if w.jumped {
        p.jump_cooldown = 10;
        p.jumped = true;
    }
    p.on_wall = w.on_wall;
    // On a rope: jump pulls it in, sneak lets it out, and the hands stay within it.
    p.on_rope = false;
    match p.rope {
        Some(rope) => {
            if p.rope_for != Some(rope.anchor) {
                (p.rope_for, p.rope_len) = (Some(rope.anchor), rope.length);
            }
            if jump {
                p.rope_len = (p.rope_len - rope.reel).max(physics::ROPE_MIN);
            } else if sneak {
                p.rope_len = (p.rope_len + rope.reel * 0.7).min(rope.length);
            }
            let hold = height * ROPE_HOLD;
            // Hands on a wall: the rope runs slack behind as it climbs, past the anchor too.
            if w.on_wall {
                let reach = (p.pos + DVec3::Y * hold).distance(rope.anchor);
                p.rope_len = p.rope_len.max(reach).min(rope.length);
            }
            let taut =
                !w.on_wall && physics::rope_hold(c, block, bodies, &mut p.pos, &mut p.vel, hw, height, hold, rope.anchor, p.rope_len);
            // Swinging on a taut rope keeps its way: little of the air's drag (0.91 a step
            // across and 0.98 up and down, which stop a pendulum in a swing).
            if taut && !w.on_ground {
                p.vel.x *= ROPE_DRAG / 0.91;
                p.vel.z *= ROPE_DRAG / 0.91;
                p.vel.y *= ROPE_DRAG / 0.98;
            }
            p.on_rope = !w.on_ground && (taut || p.pos.distance(rope.anchor) <= p.rope_len + hold + 0.5);
        }
        None => p.rope_for = None,
    }
    let here = physics::around(c, block, p.pos, height);
    (p.in_liquid, p.climbing) = (here.in_liquid, here.climbing);
    if w.on_ground {
        if let Some(b) = physics::standing_on(c, p.pos, hw, bodies, 0) {
            p.support = Some(b);
        }
    }
    p.on_ground = w.on_ground || (p.pos == before && p.on_ground);
    p.landed |= p.on_ground;
}

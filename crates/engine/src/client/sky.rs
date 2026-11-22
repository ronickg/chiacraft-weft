//! The sky: what is in it is data (`registry::SkyDef`), registered by mods per dimension, and every
//! object moves on its own. All of it is one pipeline and a few draws after the opaque world, so
//! only the pixels no block covers pay for it; stars are made in the vertex shader from their
//! number and need no buffer at all.

use glam::{IVec3, Mat3, Vec3};
use rustc_hash::FxHashMap;
use weft_abi::entry_kind as ek;
use weft_abi::sky_kind;
use weft_render::*;

use super::cloud_shadow::CloudShadow;
use super::frame_look::FrameLook;
use super::froxels::Froxels;
use super::soft_clouds::SoftClouds;
use super::world::ClientWorld;
use super::world_render::Camera;
use crate::assets::{Assets, decode_png};
use crate::content::{DimensionDef, content};
use crate::gpu_layout::{GpuSky, bind, sky as kind};
use crate::registry::SkyDef;
use crate::shaders::{Hook, Slots};

pub const HOOK: Hook = Hook {
    kind: "sky",
    entry: "shade",
    dispatch: "sky_shade",
    params: "p: SkyPixel",
    args: "p",
    ret: "vec4<f32>",
    fallback: "return p.albedo * p.color;",
};

/// A color (not linear) under rain: vanilla's going gray.
pub fn rained(c: Vec3, rain: f32, by: f32) -> Vec3 {
    let gray = (c.x * 0.3 + c.y * 0.59 + c.z * 0.11) * 0.6;
    c.lerp(Vec3::splat(gray), rain * by)
}

/// 0xRRGGBB as 0..1, not yet linear.
pub fn rgb(c: u32) -> Vec3 {
    Vec3::new(((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32) / 255.0
}

/// Where on its orbit it is, in turns.
pub(super) fn turn_of(s: &SkyDef, d: &DimensionDef, time: f64) -> f64 {
    // chiacraft, test only: `--mc-moon` keeps a body on its own orbit (base's moon, a turn every
    // 8/7 days) turning with the day instead, at its phase: the moon opposite the sun all month, as
    // Minecraft's, so like-for-like shots of an evening aren't skewed by our gibbous moon high in
    // the sky (Planner, 11-06).
    static MC_MOON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let mc_moon = *MC_MOON.get_or_init(|| crate::app::api::launch_options().contains_key("mc-moon"));
    let moved = if s.orbit.is_infinite() || (mc_moon && s.orbit > 0.0) {
        d.day_turn(time)
    } else if s.orbit == 0.0 {
        0.0
    } else {
        time / s.orbit as f64
    };
    (moved + s.phase as f64).rem_euclid(1.0)
}

/// Its frame in the world at `turn`: x toward it, y along its way, z out of the orbit's plane.
pub(super) fn frame_of(s: &SkyDef, turn: f64) -> Mat3 {
    Mat3::from_rotation_y(s.heading) * Mat3::from_rotation_x(tilt_of(s)) * Mat3::from_rotation_z((turn * std::f64::consts::TAU) as f32)
}

/// chiacraft: its path's lean, `app::api::sun_tilt` when a run sets one, none under the test flag
/// `--mc-days` (`app::api::mc_days`).
fn tilt_of(s: &SkyDef) -> f32 {
    if let Some(t) = crate::app::api::sun_tilt() {
        return t;
    }
    if crate::app::api::mc_days() { 0.0 } else { s.tilt }
}

/// How far around the sunset of a body is at `height` (the sine of its elevation): how much it
/// glows and how far along it is, 0 at the end of the dusk, 1 at the start. Vanilla's
/// getSunriseColor.
fn sunset(height: f32) -> Option<(f32, f32)> {
    (height.abs() <= 0.4).then(|| {
        let along = height / 0.4 * 0.5 + 0.5;
        let strength = 1.0 - (1.0 - (along * std::f32::consts::PI).sin()) * 0.99;
        (strength * strength, along)
    })
}

/// The glow's color so far along a sunset: `glow` at its start, deep red at the end.
fn glow_color(glow: u32, along: f32) -> Vec3 {
    rgb(glow) * Vec3::new(along * 0.3 + 0.7, (along * along * 0.7 + 0.2) / 0.9, 1.0)
}

/// Where the glowing bodies of a dimension stand now: toward them along the horizon, their
/// color so far (not linear), how strong, and how far along.
fn glows<'a>(
    dim: usize,
    d: &'a DimensionDef,
    time: f64,
    weights: &'a FxHashMap<String, f32>,
) -> impl Iterator<Item = (Vec3, Vec3, f32, f32)> + 'a {
    let list = content().game.sky.get(dim).map_or(&[][..], |l| &l[..]);
    list.iter()
        .filter(|s| s.glow != 0 && s.kind == sky_kind::BODY)
        .filter_map(move |s| {
            let toward = frame_of(s, turn_of(s, d, time)).x_axis;
            let (strength, along) = sunset(toward.y)?;
            let flat = Vec3::new(toward.x, 0.0, toward.z).try_normalize()?;
            Some((flat, glow_color(s.glow, along), strength * weights.get(&s.name).copied().unwrap_or(1.0), along))
        })
}

/// A fog color (not linear) warmed by the sunsets the camera looks at, like vanilla's.
pub fn warmed_by_glows(dim: usize, d: &DimensionDef, time: f64, fog: Vec3, forward: Vec3, weights: &FxHashMap<String, f32>) -> Vec3 {
    glows(dim, d, time, weights).fold(fog, |c, (flat, color, strength, _)| c.lerp(color, flat.dot(forward).max(0.0) * strength))
}

/// Directions through the screen's middle and one screen unit right and up of it. From the eye:
/// bobbing moves the camera by more than the near plane is away from it.
pub(crate) fn screen_rays(cam: &Camera, aspect: f32) -> (Vec3, Vec3, Vec3) {
    let view = cam.view();
    let inverse = (cam.proj(aspect) * view).inverse();
    let eye = view.inverse().w_axis.truncate();
    // Reversed depth: any point on the near plane gives the ray through a screen position.
    let ray = |x: f32, y: f32| inverse.project_point3(Vec3::new(x, y, 1.0)) - eye;
    (ray(0.0, 0.0), ray(1.0, 0.0), ray(0.0, 1.0))
}

struct Picture {
    texture: TextureId,
    index: u32,
    size: [u32; 2],
}

#[derive(Default)]
struct AtmosphereCache {
    fill_clouds: Option<PipelineId>,
    fill_rain: Option<PipelineId>,
    density: Option<(BufferId, u64)>,
    rain: Option<(BufferId, u64)>,
    cover: RainCover,
    cache_key: Option<(usize, u32, u32, [i32; 2])>,
}

pub(super) struct AtmosphereFrame {
    pub density: Option<BufferId>,
    pub rain: Option<BufferId>,
    pub cover: Option<BufferId>,
    pub cache: [u32; 4],
    pub cloud_origin: [i32; 4],
    pub cover_origin: [f32; 4],
}

const CLOUD_CACHE_LIMIT: u32 = if cfg!(target_os = "android") { 256 } else { 512 };
const CLOUD_BRICK: u32 = 8;
const RAIN_STRIDE: u32 = 8;
const RAIN_SLICES: u64 = 16;

#[derive(Default)]
struct RainCover {
    built: Option<(u32, IVec3, std::time::Instant)>,
    roofs: Vec<i32>,
    buffers: Vec<(BufferId, u32)>,
}

impl RainCover {
    fn prepare(&mut self, gpu: &mut dyn Backend, frame: &FrameInfo, cam: &Camera, world: &ClientWorld) -> (BufferId, [f32; 4]) {
        use crate::gpu_layout::rain_cover::{CELL, CUBES};
        const ACROSS: usize = (16 / CELL) as usize;
        const SIDE: usize = CUBES as usize * ACROSS;
        let edge = (cam.pos.floor().as_ivec3() >> 4) - IVec3::splat(CUBES as i32 / 2);
        let now = std::time::Instant::now();
        if self
            .built
            .is_none_or(|(dim, old, since)| dim != world.dim || old != edge || (now - since).as_secs_f32() >= 0.25)
        {
            self.built = Some((world.dim, edge, now));
            self.roofs.resize(SIDE * SIDE, i32::MIN);
            self.roofs.fill(i32::MIN);
            let content = content();
            let blocks_rain =
                |state: u32| content.states[state as usize].flags & (weft_abi::block_flags::SOLID | weft_abi::block_flags::LIQUID) != 0;
            for cz in 0..CUBES as usize {
                for cx in 0..CUBES as usize {
                    let mut found = 0;
                    for y in (edge.y..edge.y + CUBES as i32).rev() {
                        let Some(cube) = world.cube(IVec3::new(edge.x + cx as i32, y, edge.z + cz as i32)) else { continue };
                        let uniform = cube.uniform_state();
                        let light = cube.uniform_light();
                        if uniform.is_some_and(|s| !blocks_rain(s)) && light.is_some_and(|l| weft_abi::light::sky(l) == 15) {
                            continue;
                        }
                        for z in 0..ACROSS {
                            for x in 0..ACROSS {
                                let roof = &mut self.roofs[(cz * ACROSS + z) * SIDE + cx * ACROSS + x];
                                if *roof != i32::MIN {
                                    continue;
                                }
                                let x = x * CELL as usize + CELL as usize / 2;
                                let z = z * CELL as usize + CELL as usize / 2;
                                let floor = (0..16).rev().find(|&y| {
                                    let i = crate::world::cube::index(x, y, z);
                                    blocks_rain(cube.get(i)) || weft_abi::light::sky(cube.light(i)) < 15
                                });
                                if let Some(floor) = floor {
                                    *roof = y * 16 + floor as i32 + 1;
                                    found += 1;
                                }
                            }
                        }
                        if found == ACROSS * ACROSS {
                            break;
                        }
                    }
                }
            }
        }
        let buffer =
            crate::gpu_layout::frame_buffer(gpu, &mut self.buffers, frame.slot, (SIDE * SIDE) as u32, 4, usage::STORAGE, "rain cover");
        let bytes = bytemuck::cast_slice(&self.roofs);
        gpu.mapped(buffer)[..bytes.len()].copy_from_slice(bytes);
        let origin = edge.as_dvec3() * 16.0;
        let relative = cam.pos - origin;
        (buffer, [relative.x as f32, cam.pos.y as f32, relative.z as f32, 1.0])
    }
}

fn atmosphere_buffer(slot: &mut Option<(BufferId, u64)>, gpu: &mut dyn Backend, bytes: u64, label: &str) -> BufferId {
    if let Some((buffer, _)) = slot.filter(|&(_, capacity)| capacity >= bytes) {
        return buffer;
    }
    if let Some((buffer, _)) = slot.take() {
        gpu.destroy_buffer(buffer);
    }
    let buffer = gpu.create_buffer(bytes, usage::STORAGE, label);
    *slot = Some((buffer, bytes));
    buffer
}

impl AtmosphereCache {
    fn destroy(self, gpu: &mut dyn Backend) {
        for pipe in [self.fill_clouds, self.fill_rain].into_iter().flatten() {
            gpu.destroy_pipeline(pipe);
        }
        for (buffer, _) in [self.density, self.rain].into_iter().flatten() {
            gpu.destroy_buffer(buffer);
        }
        for (buffer, _) in self.cover.buffers {
            gpu.destroy_buffer(buffer);
        }
    }

    fn prepare(
        &mut self,
        gpu: &mut dyn Backend,
        commands: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        world: &ClientWorld,
        s: GpuSky,
        objects: BufferId,
        scene: BufferId,
        depth: BufferId,
        depth_pyramid: bool,
        detailed_range: f32,
    ) -> AtmosphereFrame {
        let dim = world.dim as usize;
        let mut out = AtmosphereFrame {
            density: None,
            rain: None,
            cover: None,
            cache: [0, 0, u32::from(depth_pyramid), 0],
            cloud_origin: [0; 4],
            cover_origin: [0.0; 4],
        };

        if let Some(fill) = self.fill_clouds.filter(|_| s.ids[0] == u32::MAX) {
            // Include the furthest detailed cells when possible, plus one tile for alignment.
            let cell_width = s.extra[0] / crate::weather::CLOUD_COLUMNS as f32;
            let radius = (detailed_range / cell_width).ceil() as u32 + CLOUD_BRICK;
            let side = (2 * radius)
                .next_power_of_two()
                .clamp(128, CLOUD_CACHE_LIMIT);
            let brick_count = side / CLOUD_BRICK;
            let size = 5 * (side * side + brick_count * brick_count);
            let density = atmosphere_buffer(&mut self.density, gpu, u64::from(size) * 4, "cloud density");
            let origin = [0, 1].map(|axis| {
                let cells = (s.uv[axis + 2] as i32).wrapping_mul(24) + (s.uv[axis] * 24.0) as i32;
                (cells - side as i32 / 2).div_euclid(CLOUD_BRICK as i32) * CLOUD_BRICK as i32
            });
            let previous = self
                .cache_key
                .filter(|&(dimension, seed, old_side, _)| dimension == dim && seed == s.ids[2] && old_side == side)
                .map_or([i32::MIN + 1; 2], |(_, _, _, last)| last);
            out.cache[0] = side;
            out.cloud_origin = [origin[0], origin[1], previous[0], previous[1]];
            out.density = Some(density);
            if previous != origin {
                commands.pipeline(fill);
                commands.bind(bind::sky::SCENE, scene);
                commands.bind(bind::sky::CLOUD_CELLS_OUT, density);
                commands.push(Cmd::Dispatch { x: brick_count, y: brick_count, z: 5 });
                commands.push(Cmd::Barrier);
            }
            self.cache_key = Some((dim, s.ids[2], side, origin));
        }
        let wet = s.ids[0] == u32::MAX && s.axes[0][3].max(s.axes[1][3]) > 0.0;
        if wet && content().dims[dim].sky_light {
            let (cover, origin) = self.cover.prepare(gpu, frame, cam, world);
            out.cover = Some(cover);
            out.cover_origin = origin;
        }
        if let Some(fill) = self.fill_rain.filter(|_| wet) {
            let rain_grid = [frame.width.div_ceil(RAIN_STRIDE), frame.height.div_ceil(RAIN_STRIDE)];
            let bytes = u64::from(rain_grid[0]) * u64::from(rain_grid[1]) * (RAIN_SLICES + 1) * 8;
            let rain = atmosphere_buffer(&mut self.rain, gpu, bytes, "rain depth samples");
            out.cache[1] = rain_grid[0];
            out.rain = Some(rain);
            commands.pipeline(fill);
            commands.bind(bind::sky::OBJECTS, objects);
            commands.bind(bind::sky::SCENE, scene);
            commands.bind(bind::sky::WORLD_DEPTH, depth);
            commands.bind(bind::sky::RAIN_COVER, out.cover.unwrap_or(scene));
            commands.bind(bind::sky::RAIN_DEPTH_OUT, rain);
            commands.push(Cmd::Dispatch { x: rain_grid[0].div_ceil(8), y: rain_grid[1].div_ceil(8), z: 1 });
            commands.push(Cmd::Barrier);
        }

        out
    }
}

pub struct Sky {
    pipe: Option<PipelineId>,
    clouds_pipe: Option<PipelineId>,
    atmosphere_pipe: Option<PipelineId>,
    bolt_pipe: Option<PipelineId>,
    atmosphere: AtmosphereCache,
    atmosphere_compute: CommandList,
    /// Shader slot of each object, per dimension.
    slots: Vec<Vec<u32>>,
    buffers: Vec<(BufferId, u32)>,
    /// Pictures as they were first drawn, `None` for missing ones.
    pictures: FxHashMap<String, Option<Picture>>,
    assets: Assets,
    list: Vec<GpuSky>,
    /// The fog color of each liquid, linear.
    block_colors: std::cell::RefCell<FxHashMap<u32, Vec3>>,
    /// This frame's buffer and the entries over the screen: where they start, how many.
    overlay: Option<(BufferId, u32, u32)>,
    /// The cloud layer as the last frame drew it, for shader packs' cloud shadows
    /// (`Scene::clouds`, `Scene::cloud_cells`); zeros without one.
    pub cloud_shade: [[f32; 4]; 2],
    /// Soft clouds (chiacraft, always on): their targets, history and passes.
    pub soft: SoftClouds,
    /// The mist's volume (`client::froxels`): wanted (`core:froxels`) and of use this frame (mist
    /// in the air, the eye out of liquids); the client and app set both before `draw`.
    pub froxels: Froxels,
    pub froxels_wanted: bool,
    pub froxels_possible: bool,
    /// The soft clouds' shadow on the land (`client::cloud_shadow`), wanted (the HDR world and
    /// `core:cloud_shadows`); the app sets it before `draw`.
    pub cloud_shadow: CloudShadow,
    pub cloud_shadow_wanted: bool,
    /// What the pack's look works out once a frame (`client::frame_look`).
    pub frame_look: FrameLook,
    /// Soft clouds wanted (always, set by the app) and possible this frame (the world copied, whose depth the
    /// trace stops at); the client sets both before `draw`.
    pub soft_wanted: bool,
    pub soft_possible: bool,
}

/// A picture over the whole screen, drawn last: from inside a liquid, a block or a fire.
#[derive(Clone)]
pub struct Overlay {
    pub picture: String,
    /// Linear rgb and how much shows.
    pub color: [f32; 4],
    pub tiles: f32,
    /// Only the lower part of the screen.
    pub lower: bool,
    /// Which frame of an animated picture.
    pub frame: u32,
}

/// The average color of a liquid's first frame, its tint in.
fn average_color(assets: &Assets, state: u32) -> Vec3 {
    let c = content();
    let st = &c.states[state as usize];
    let img = c
        .textures
        .get(st.textures[0] as usize)
        .and_then(|n| assets.read(&assets.picture(weft_abi::layers(n).next()?.name), "textures", "png"))
        .and_then(|b| decode_png(&b));
    let Some(img) = img else { return Vec3::splat(0.5) };
    let (mut sum, mut weight) = (Vec3::ZERO, 0.0);
    for p in img
        .rgba
        .chunks_exact(4)
        .take((img.width * img.width) as usize)
    {
        let a = p[3] as f32 / 255.0;
        sum += Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) / 255.0 * a;
        weight += a;
    }
    let tint = if st.tint != 0 { rgb(crate::content::item_tint(st.tint)) } else { Vec3::ONE };
    sum / f32::max(weight, 1.0) * tint
}

impl Sky {
    pub fn new(gpu: &mut dyn Backend, assets: &Assets) -> Sky {
        let mut s = Sky {
            pipe: None,
            clouds_pipe: None,
            atmosphere_pipe: None,
            bolt_pipe: None,
            atmosphere: AtmosphereCache::default(),
            atmosphere_compute: CommandList::default(),
            slots: Vec::new(),
            buffers: Vec::new(),
            pictures: FxHashMap::default(),
            assets: assets.clone(),
            list: Vec::new(),
            block_colors: Default::default(),
            overlay: None,
            cloud_shade: [[0.0; 4]; 2],
            soft: SoftClouds::default(),
            froxels: Froxels::default(),
            froxels_wanted: true,
            froxels_possible: false,
            cloud_shadow: CloudShadow::default(),
            cloud_shadow_wanted: false,
            frame_look: FrameLook::default(),
            soft_wanted: false,
            soft_possible: false,
        };
        s.reload(gpu, assets);
        s
    }

    /// Recompiles the shader with the snippets mods and packs have now, and reads pictures anew.
    pub fn reload(&mut self, gpu: &mut dyn Backend, assets: &Assets) {
        self.assets = assets.clone();
        self.drop_pictures(gpu);
        self.block_colors.get_mut().clear();
        let snippets = crate::shaders::snippets(assets, &HOOK);
        let made = crate::packs::compose(assets, "sky.wgsl", &[(&HOOK, &snippets)])
            .map_err(|e| e.0)
            .and_then(|(shader, mut slots)| {
                let pipe = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "sky",
                        shader: &shader,
                        vertex: "vs",
                        fragment: Some("fs"),
                        color: Some(crate::post::world_format()),
                        // Depth 0 is the far plane: it passes only where nothing was drawn.
                        depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                        blend: Blend::Premultiplied,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: crate::post::motion(),
                    })
                    .map_err(|e| e.0)?;
                // The shader supplies depth for testing against solid geometry. Upstream's clouds
                // stop writing it, translucents mixing with the cloud color instead of being
                // rejected by its depth (077bd3318); chiacraft keeps them writing it, as a shader
                // pack's water and glass don't mix the clouds in themselves (notes/ENGINE_CHANGES.md).
                let clouds = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "clouds",
                        shader: &shader,
                        vertex: "vs_clouds",
                        fragment: Some("fs_clouds"),
                        color: Some(crate::post::world_format()),
                        depth: Some(DepthState { write: true, compare: Compare::GreaterEqual }),
                        blend: Blend::Premultiplied,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: crate::post::motion(),
                    })
                    .map_err(|e| e.0)?;
                let atmosphere = gpu
                    .create_pipeline(&PipelineDesc {
                        label: "atmosphere",
                        shader: &shader,
                        vertex: "vs_atmosphere",
                        fragment: Some("fs_atmosphere"),
                        color: Some(crate::post::world_format()),
                        depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                        blend: Blend::Premultiplied,
                        cull_back: false,
                        topology: Topology::Triangles,
                        motion: crate::post::motion(),
                    })
                    .map_err(|e| log::warn!("atmosphere shader: {e}"))
                    .ok();
                let bolt = shader
                    .module
                    .entry_points
                    .iter()
                    .any(|entry| entry.name == "fs_lightning")
                    .then(|| {
                        gpu.create_pipeline(&PipelineDesc {
                            label: "lightning",
                            shader: &shader,
                            vertex: "vs",
                            fragment: Some("fs_lightning"),
                            color: Some(crate::post::world_format()),
                            depth: Some(DepthState { write: false, compare: Compare::GreaterEqual }),
                            blend: Blend::Premultiplied,
                            cull_back: false,
                            topology: Topology::Triangles,
                            motion: crate::post::motion(),
                        })
                        .map_err(|e| log::warn!("lightning shader: {e}"))
                        .ok()
                    })
                    .flatten();
                let mut optional = |entry, label| {
                    shader
                        .module
                        .entry_points
                        .iter()
                        .any(|p| p.name == entry)
                        .then(|| {
                            gpu.create_compute(&ComputeDesc { label, shader: &shader, entry })
                                .map_err(|e| log::warn!("{label}: {e}"))
                                .ok()
                        })
                        .flatten()
                };
                let cache = if atmosphere.is_some() {
                    AtmosphereCache {
                        fill_clouds: optional("cache_cloud_density", "cloud density cache"),
                        fill_rain: optional("cache_rain_depth", "rain depth cache"),
                        ..Default::default()
                    }
                } else {
                    AtmosphereCache::default()
                };
                Ok((pipe, clouds, atmosphere, bolt, cache, slots.pop().unwrap_or_default(), shader))
            });
        match made {
            Ok((pipe, clouds, atmosphere, bolt, cache, slots, shader)) => {
                self.soft.set_shader(gpu, &shader);
                self.froxels.set_shader(gpu, &shader);
                self.cloud_shadow.set_shader(gpu, &shader);
                self.frame_look.set_shader(gpu, &shader);
                if let Some(old) = self.pipe.replace(pipe) {
                    gpu.destroy_pipeline(old);
                }
                if let Some(old) = self.clouds_pipe.replace(clouds) {
                    gpu.destroy_pipeline(old);
                }
                if let Some(old) = std::mem::replace(&mut self.atmosphere_pipe, atmosphere) {
                    gpu.destroy_pipeline(old);
                }
                if let Some(old) = std::mem::replace(&mut self.bolt_pipe, bolt) {
                    gpu.destroy_pipeline(old);
                }
                std::mem::replace(&mut self.atmosphere, cache).destroy(gpu);
                self.set_slots(&slots);
            }
            Err(e) => log::error!("sky shader: {e}"),
        }
    }

    pub(super) fn has_atmosphere(&self) -> bool {
        self.atmosphere_pipe.is_some()
    }

    pub(super) fn sample_atmosphere(&mut self, list: &mut CommandList) {
        list.append(&mut self.atmosphere_compute);
    }

    pub fn block_color(&self, state: u32) -> Vec3 {
        *self
            .block_colors
            .borrow_mut()
            .entry(state)
            .or_insert_with(|| average_color(&self.assets, state).powf(2.2))
    }

    /// What goes over the screen, after the hand.
    pub fn draw_overlay(&self, list: &mut CommandList, scene: BufferId) {
        let (Some(pipe), Some((buf, first, n))) = (self.pipe, self.overlay) else { return };
        list.pipeline(pipe);
        list.bind(bind::sky::OBJECTS, buf);
        list.bind(bind::sky::SCENE, scene);
        for at in first..first + n {
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: at });
        }
    }

    /// The entries of what goes over the screen, after what is in the list so far.
    fn overlay_entries(&mut self, gpu: &mut dyn Backend, overlays: &[Overlay]) -> u32 {
        let mut n = 0;
        for o in overlays {
            let Some((index, [w, h])) = self.picture(gpu, &o.picture) else { continue };
            let frame_h = w as f32 / h as f32;
            self.list.push(GpuSky {
                axes: [[0.0; 4]; 3],
                uv: [0.0, 0.0, 1.0, frame_h],
                color: o.color,
                ids: [index, kind::OVERLAY, 0, 0],
                extra: [o.tiles, o.lower as u32 as f32, (o.frame as f32 * frame_h).fract(), 0.0],
                cloud_sun: [0.0; 4],
            });
            n += 1;
        }
        n
    }

    fn set_slots(&mut self, slots: &Slots) {
        self.slots = content()
            .game
            .sky
            .iter()
            .map(|list| list.iter().map(|o| slots.slot(&o.shader)).collect())
            .collect();
        for o in content()
            .game
            .sky
            .iter()
            .flatten()
            .filter(|o| !o.shader.is_empty() && slots.slot(&o.shader) == 0)
        {
            log::warn!("sky object {}: no shader {}", o.name, o.shader);
        }
    }

    fn drop_pictures(&mut self, gpu: &mut dyn Backend) {
        for p in self.pictures.drain().filter_map(|(_, p)| p) {
            gpu.destroy_texture(p.texture);
        }
    }

    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        self.drop_pictures(gpu);
        self.soft.destroy(gpu);
        self.froxels.destroy(gpu);
        self.cloud_shadow.destroy(gpu);
        self.frame_look.destroy(gpu);
        for p in [self.pipe.take(), self.clouds_pipe.take(), self.atmosphere_pipe.take(), self.bolt_pipe.take()]
            .into_iter()
            .flatten()
        {
            gpu.destroy_pipeline(p);
        }
        std::mem::take(&mut self.atmosphere).destroy(gpu);
        for (b, _) in self.buffers.drain(..) {
            gpu.destroy_buffer(b);
        }
    }

    fn picture(&mut self, gpu: &mut dyn Backend, name: &str) -> Option<(u32, [u32; 2])> {
        if name.is_empty() {
            return None;
        }
        if !self.pictures.contains_key(name) {
            let loaded = self
                .assets
                .read(&self.assets.picture(name), "textures", "png")
                .and_then(|b| decode_png(&b));
            if loaded.is_none() {
                log::error!("sky: no picture {name}");
            }
            let picture = loaded.map(|img| {
                let texture = gpu.create_texture(
                    &TextureDesc { width: img.width, height: img.height, mips: 1, format: Format::Rgba8Srgb, render_target: false },
                    "sky picture",
                );
                gpu.write_texture(texture, 0, &img.rgba);
                Picture { texture, index: gpu.texture_index(texture), size: [img.width, img.height] }
            });
            self.pictures.insert(name.to_string(), picture);
        }
        self.pictures[name].as_ref().map(|p| (p.index, p.size))
    }

    /// `time`: game ticks with the fraction of the current one. `eye_y`: where the horizon's dark
    /// lower half starts to show.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw(
        &mut self,
        gpu: &mut dyn Backend,
        list: &mut CommandList,
        frame: &FrameInfo,
        cam: &Camera,
        scene: BufferId,
        world_depth: Option<BufferId>,
        world: &ClientWorld,
        time: f64,
        weather: crate::weather::WeatherField,
        depth_pyramid: bool,
        detailed_range: f32,
        eye_y: f64,
        environment: &weft_abi::environment::EnvironmentValues,
        lightning: &super::lightning::LightningEffects,
        weights: &FxHashMap<String, f32>,
        overlays: &[Overlay],
    ) -> (GpuSky, Option<AtmosphereFrame>) {
        let dim = world.dim as usize;
        self.overlay = None;
        self.cloud_shade = [[0.0; 4]; 2];
        self.atmosphere_compute.clear();
        let Some(pipe) = self.pipe else { return (GpuSky::default(), None) };
        let c = content();
        let d = &c.dims[dim];
        // No sky to see (from inside a liquid its fog hides it): only the gradient is drawn,
        // marked so (`extra.x` 1), which the engine's sky draws as the fog and a pack as its own
        // water (chiacraft: the world's gaps showed the clear colour, a band of navy streaks where
        // the near sea floor ended, the world playtester's under-water shots).
        if environment.sky_opacity <= 0.0 {
            self.soft
                .prepare(gpu, false, frame.width, frame.height, glam::Mat4::IDENTITY, cam.pos, glam::DVec2::ZERO);
            self.list.clear();
            let aspect = frame.width as f32 / frame.height.max(1) as f32;
            let (o, px, py) = screen_rays(cam, aspect);
            let m = Mat3::from_cols(px - o, py - o, o);
            self.list.push(GpuSky {
                axes: [m.x_axis.extend(0.0).to_array(), m.y_axis.extend(0.0).to_array(), m.z_axis.extend(0.0).to_array()],
                uv: [0.0; 4],
                color: [0.0, 0.0, 0.0, 1.0],
                ids: [u32::MAX, kind::GRADIENT, 0, 0],
                extra: [1.0, 0.0, 0.0, 0.0],
                cloud_sun: [0.0; 4],
            });
            let over = self.overlay_entries(gpu, overlays);
            let n = self.list.len() as u32;
            let stride = size_of::<GpuSky>() as u64;
            let buf = crate::gpu_layout::frame_buffer(gpu, &mut self.buffers, frame.slot, n, stride, usage::STORAGE, "sky");
            gpu.mapped(buf)[..self.list.len() * stride as usize].copy_from_slice(bytemuck::cast_slice(&self.list));
            self.overlay = (over > 0).then_some((buf, 1, over));
            list.pipeline(pipe);
            list.bind(bind::sky::OBJECTS, buf);
            list.bind(bind::sky::SCENE, scene);
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance: 0 });
            return (GpuSky::default(), None);
        }
        let turn = d.day_turn(time);
        let height = (turn * std::f64::consts::TAU).sin() as f32;
        // 0 by day, 1 at night: vanilla's star brightness, before its last factor of 0.5.
        let night = if d.sky_light { (2.0 * (0.75 - height * 2.0)).clamp(0.0, 1.0).powi(2) } else { 1.0 };

        self.list.clear();
        let aspect = frame.width as f32 / frame.height.max(1) as f32;
        let (o, px, py) = screen_rays(cam, aspect);
        let axes = |m: Mat3| [m.x_axis.extend(0.0).to_array(), m.y_axis.extend(0.0).to_array(), m.z_axis.extend(0.0).to_array()];
        let below = d.sea_level().is_some_and(|sea| eye_y < sea as f64);
        self.list.push(GpuSky {
            axes: axes(Mat3::from_cols(px - o, py - o, o)),
            uv: [below as u32 as f32, 0.0, 0.0, 0.0],
            color: (rained(Vec3::from_array(environment.sky_color).powf(1.0 / 2.2), 1.0, environment.sky_gray).powf(2.2))
                .extend(environment.sky_opacity)
                .to_array(),
            ids: [u32::MAX, kind::GRADIENT, 0, 0],
            // `extra.x` 1 marks the sky seen from inside a liquid (above).
            extra: [0.0; 4],
            cloud_sun: [0.0; 4],
        });
        let mut draws: Vec<(u32, u32)> = vec![(3, 0)];

        let objects = c.game.sky.get(dim).map_or(&[][..], |l| &l[..]);
        // Use the visible sun's orbit, including its heading and tilt. A small
        // color curve models the longer path through the air near the horizon.
        let (cloud_sun_dir, cloud_sun) = if d.sky_light && weather.clouds.is_some() && environment.sunlight > 0.0 {
            objects
                .iter()
                .filter(|s| s.glow != 0 && s.kind == sky_kind::BODY)
                .find_map(|s| {
                    let toward = frame_of(s, turn_of(s, d, time)).x_axis;
                    let opacity = (s.opacity[0] + (s.opacity[1] - s.opacity[0]) * night) * weights.get(&s.name).copied().unwrap_or(1.0);
                    let fade = ((toward.y + 0.09) * 9.0).clamp(0.0, 1.0);
                    if opacity <= 0.0 || fade <= 0.0 {
                        return None;
                    }
                    let warmth = (1.0 - toward.y.max(0.0) / 0.46).clamp(0.0, 1.0);
                    let color = Vec3::ONE.lerp(Vec3::new(1.0, 0.36, 0.10), warmth)
                        * rgb(s.color).powf(2.2)
                        * (opacity * fade * environment.sunlight);
                    let path = 0.21 / toward.y.max(0.24);
                    Some((toward.extend(0.0).to_array(), color.extend(path).to_array()))
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        let forced_kind = weather.forced.map_or(0.0, |id| {
            let preset = &c.game.weathers[id as usize];
            if preset.lightning() {
                2.0
            } else if preset.falls() {
                1.0
            } else {
                0.0
            }
        });
        let mut clouds = None;
        for (i, s) in objects.iter().enumerate() {
            // Cloud cover hides celestial objects, not the clouds or sky behind them.
            let opacity = (s.opacity[0] + (s.opacity[1] - s.opacity[0]) * night)
                * if matches!(s.kind, sky_kind::BODY | sky_kind::STARS) { 1.0 - environment.sky_cover } else { 1.0 }
                * environment.sky_opacity
                * weights.get(&s.name).copied().unwrap_or(1.0);
            if opacity <= 0.0 || (s.kind == sky_kind::CLOUDS && (environment.cloud_opacity <= 0.0 || environment.cloud_fog_end <= 0.0)) {
                continue;
            }
            if s.kind == sky_kind::CLOUDS {
                use crate::weather::{CLOUD_CELL, CLOUD_HEIGHT};
                let setting = |name: &str| {
                    c.game
                        .catalog
                        .ids(ek::CONFIG)
                        .get(name)
                        .map_or(0.0, |&id| crate::config::get(id))
                };
                let lifted = s.height as f64 + setting("core:cloud_height");
                let low = (lifted - cam.pos.y) as f32;
                let height = if weather.clouds.is_some() { CLOUD_HEIGHT } else { 4.0 };
                if low >= environment.cloud_fog_end || low + height <= -environment.cloud_fog_end {
                    continue;
                }
                let ambient = environment.ambient.max(0.0);
                let light = ambient + (1.0 - ambient).max(0.0) * environment.sunlight;
                let tint = Vec3::from_array(environment.cloud_color)
                    .max(Vec3::ZERO)
                    .powf(1.0 / 2.2);
                let shade = |gray, dark| {
                    let rgb = rained(rgb(s.color), 1.0, gray) * tint;
                    rgb.lerp(Vec3::splat((rgb.x * 0.3 + rgb.y * 0.59 + rgb.z * 0.11) * 0.2), dark)
                        .powf(2.2)
                        * light
                };
                let automatic = weather.clouds.is_some() && weather.forced.is_none();
                let color = if automatic { shade(0.0, 0.0) } else { shade(environment.cloud_gray, environment.cloud_dark) };
                let (ids, size, pos, colors) = if let Some(field) = weather.clouds {
                    let color_for = |id: u16| {
                        let preset = &c.game.weathers[id as usize];
                        let look = preset.look;
                        shade(look.cloud_gray, look.cloud_dark * 0.74)
                            .extend(if world_depth.is_some() && preset.falls() { look.falls } else { 0.0 })
                            .to_array()
                    };
                    (
                        [u32::MAX, kind::CLOUDS, field.seed, automatic as u32],
                        CLOUD_CELL,
                        field.coords(glam::DVec2::new(cam.pos.x, cam.pos.z)),
                        [
                            color_for(weather.forced.unwrap_or(field.rain)),
                            color_for(weather.forced.unwrap_or(field.thunder)),
                            cloud_sun_dir,
                        ],
                    )
                } else {
                    let Some((index, [w, h])) = self.picture(gpu, &s.texture) else { continue };
                    const TEXTURE_CELL: f64 = 12.0;
                    let pos = glam::DVec2::new(cam.pos.x + time * s.drift as f64, cam.pos.z) / TEXTURE_CELL;
                    ([index, kind::CLOUDS, w, h], TEXTURE_CELL, pos, [[0.0; 4]; 3])
                };
                let origin = if weather.clouds.is_some() { pos.floor() } else { pos.floor().rem_euclid(glam::DVec2::splat(4096.0)) };
                let uv = [pos.x.rem_euclid(1.0) as f32, pos.y.rem_euclid(1.0) as f32, origin.x as f32, origin.y as f32];
                clouds = Some(self.list.len() as u32);
                // chiacraft: a picture's clouds carry what shader packs read of them, the soft
                // clouds' trace and the weather; and their opacity as vanilla's boxes stack.
                let (axes, alpha, extra) = if weather.clouds.is_none() {
                    // For packs' weather: how much falls, and how stormy it is (thunder's clouds
                    // go 0.95 dark).
                    // SYNC: what falls came from the weather's look (`WeatherLook::falls`), which
                    // the environment no longer carries; the forced preset's, else the clouds' gray.
                    let rain = weather
                        .forced
                        .map_or(environment.cloud_gray, |id| c.game.weathers[id as usize].look.falls)
                        .clamp(0.0, 1.0);
                    let thunder = (environment.cloud_dark / 0.95).clamp(0.0, 1.0);
                    // Soft clouds: this frame's targets and the scene's fields for them, where the
                    // picture's cells stand (`offset`: what is added to a place to find it in them).
                    let offset = glam::DVec2::new(time * s.drift as f64, 0.0);
                    let view_proj = cam.proj(aspect) * cam.view();
                    self.soft
                        .prepare(gpu, self.soft_wanted && self.soft_possible, frame.width, frame.height, view_proj, cam.pos, offset);
                    let trace = self
                        .soft
                        .trace_index()
                        .filter(|_| self.soft.scene.0[0] != 0.0)
                        .map_or(0.0, |i| i as f32 + 1.0);
                    // The weather for packs: x how much more of the sky they cover as rain comes, y
                    // how dark their undersides go, z an overcast deck in a storm.
                    let cover = (rain * 0.75 + thunder * 0.25).min(1.0);
                    // The bindless index + 1 (0: none), the cell size, its height over the camera
                    // (its underside) and how much of the gaps the weather fills (`fs_clouds`'
                    // `cloud_cell`).
                    self.cloud_shade = [[ids[0] as f32 + 1.0, size as f32, low, cover * 0.6 + thunder * 0.35], uv];
                    // Past all the terrain there is, distant terrain too, for packs' clouds
                    // (`extra.w`); the cost doesn't depend on it.
                    // chiacraft, test only: `--mc-clouds` ends them as vanilla's do, a little past the
                    // view distance where its fog closes (Bliss's instance at 32 chunks: they stop
                    // about 600 blocks out and leave a clear pale band over the horizon; 11-06).
                    let reach = if crate::app::api::launch_options().contains_key("mc-clouds") {
                        setting("core:view_distance") * 16.0 * 1.2
                    } else {
                        (setting("core:view_distance") * 16.0)
                            .max(super::lod::Lod::distance() as f64)
                            .max(1365.0)
                            * 1.5
                    };
                    (
                        // axes[1].x: the soft clouds' trace target, bindless index + 1 (0: none),
                        // for `fs_cloud_resolve`.
                        [[cover, (rain * 0.6 + thunder * 0.4).min(1.0), thunder, 0.0], [trace, 0.0, 0.0, 0.0], [0.0; 4]],
                        // Vanilla draws every face of a cloud's box at 0.8, so from below its
                        // underside and the top behind it stack to about 0.96; one face at 0.8
                        // here let the blue sky (clouds b-r +10..+23 against vanilla's neutral,
                        // Bliss's world) and the sun show through (Weft bug; Planner's stage 1
                        // review, MC import's s6 sheet, 11-06).
                        0.96 + 0.04 * cover,
                        [size as f32, height, low, reach as f32],
                    )
                } else {
                    (colors, 1.0, [size as f32, height, low, forced_kind])
                };
                self.list.push(GpuSky {
                    axes,
                    uv,
                    color: color
                        .extend(alpha * environment.cloud_opacity * opacity)
                        .to_array(),
                    ids,
                    extra,
                    cloud_sun,
                });
                continue;
            }
            if opacity <= 0.0 {
                continue;
            }
            let t = turn_of(s, d, time);
            let spin =
                if s.spin != 0.0 { Mat3::from_rotation_x((time / s.spin as f64 * std::f64::consts::TAU) as f32) } else { Mat3::IDENTITY };
            let mut m = frame_of(s, t) * spin;
            // A body on a leaned path (`tilt`) keeps its picture upright, its edges level with the
            // horizon (chiacraft): leaned with the path, the moon's square stood as a diamond (the
            // world playtester's "white rotated square"). Its picture's up toward the world's up;
            // right at the zenith the path's own frame.
            if s.kind == sky_kind::BODY && s.tilt != 0.0 && s.spin == 0.0 {
                let toward = m.x_axis;
                let right = toward.cross(Vec3::Y);
                if right.length() > 0.05 {
                    let right = right.normalize();
                    m = Mat3::from_cols(toward, right.cross(toward), right);
                }
            }
            let picture = self.picture(gpu, &s.texture);
            let slot = self
                .slots
                .get(dim)
                .and_then(|l| l.get(i))
                .copied()
                .unwrap_or(0);
            let ticks = if s.frame_ticks != 0 {
                s.frame_ticks as u64
            } else if d.day_length != 0 {
                d.day_length as u64
            } else {
                24000
            };
            let frame_at = (time as u64 / ticks) % (s.frames[0] * s.frames[1]) as u64;
            let (cols, rows) = (s.frames[0] as f32, s.frames[1] as f32);
            let (u, v) = ((frame_at % s.frames[0] as u64) as f32 / cols, (frame_at / s.frames[0] as u64) as f32 / rows);
            let half = (s.size * 0.5).tan();
            let (kind_id, uv, vertices) = match s.kind {
                sky_kind::STARS => (kind::STARS, [half, s.seed as f32, 0.0, 0.0], 6 * s.count),
                sky_kind::DOME => {
                    let net = picture.is_some_and(|(_, [w, h])| w * 2 == h * 3);
                    (kind::DOME, [net as u32 as f32, 0.0, 0.0, 0.0], 3)
                }
                _ => (kind::BODY, [u, v, u + 1.0 / cols, v + 1.0 / rows], 6),
            };
            let at = self.list.len() as u32;
            self.list.push(GpuSky {
                axes: axes(m),
                uv,
                color: rgb(s.color).powf(2.2).extend(opacity).to_array(),
                ids: [picture.map_or(u32::MAX, |p| p.0), kind_id, slot, s.additive as u32],
                extra: [half, 0.0, 0.0, 0.0],
                cloud_sun: [0.0; 4],
            });
            draws.push((vertices, at));
        }
        // Sunsets over the sky and the stars, under the bodies: right after the domes and stars.
        let first_body = draws
            .iter()
            .skip(1)
            .position(|&(_, at)| self.list[at as usize].ids[1] == kind::BODY)
            .map_or(draws.len(), |p| p + 1);
        let glowing: Vec<(Vec3, Vec3, f32, f32)> = glows(dim, d, time, weights).collect();
        for (n, (flat, color, strength, along)) in glowing.into_iter().enumerate() {
            let at = self.list.len() as u32;
            self.list.push(GpuSky {
                axes: axes(Mat3::from_cols(flat, Vec3::Y, flat.cross(Vec3::Y))),
                uv: [along, 0.0, 0.0, 0.0],
                color: color
                    .powf(2.2)
                    .extend(strength * environment.sky_opacity * (1.0 - environment.sky_cover))
                    .to_array(),
                ids: [u32::MAX, kind::GLOW, 0, 0],
                extra: [0.0; 4],
                cloud_sun: [0.0; 4],
            });
            draws.insert(first_body + n, (3, at));
        }

        let bolt_from = self.list.len() as u32;
        lightning.append_bolts(&mut self.list, cam, frame.height, environment.sky_opacity);
        let bolts = self.list.len() as u32 - bolt_from;
        let over_from = self.list.len() as u32;
        let over = self.overlay_entries(gpu, overlays);
        let n = self.list.len() as u32;
        let stride = size_of::<GpuSky>() as u64;
        let buf = crate::gpu_layout::frame_buffer(gpu, &mut self.buffers, frame.slot, n, stride, usage::STORAGE, "sky");
        gpu.mapped(buf)[..self.list.len() * stride as usize].copy_from_slice(bytemuck::cast_slice(&self.list));
        self.overlay = (over > 0).then_some((buf, over_from, over));
        list.pipeline(pipe);
        list.bind(bind::sky::OBJECTS, buf);
        list.bind(bind::sky::SCENE, scene);
        for (vertices, first_instance) in draws {
            list.push(Cmd::Draw { vertices, instances: 1, first_vertex: 0, first_instance });
        }
        // chiacraft: the pack's passes over the sky's objects (the mist's volume, the clouds'
        // shadow, the frame's look, the soft clouds' trace).
        let froxels = self.froxels_wanted && self.froxels_possible;
        self.froxels
            .frame(gpu, froxels, frame.width, frame.height, buf, scene);
        self.cloud_shadow
            .frame(gpu, self.cloud_shadow_wanted, buf, scene, clouds);
        self.frame_look.frame(gpu, buf, scene);
        match clouds.filter(|&i| self.list[i as usize].ids[0] != u32::MAX) {
            Some(first_instance) => self.soft.record(buf, scene, first_instance),
            None => self
                .soft
                .prepare(gpu, false, frame.width, frame.height, glam::Mat4::IDENTITY, cam.pos, glam::DVec2::ZERO),
        }
        let (cloud, shared) = if let (Some(first_instance), Some(cloud_pipe)) = (clouds, self.clouds_pipe) {
            let s = self.list[first_instance as usize];
            let shared = if let (Some(depth), Some(pipe)) = (world_depth, self.atmosphere_pipe) {
                let prepared = self.atmosphere.prepare(
                    gpu,
                    &mut self.atmosphere_compute,
                    frame,
                    cam,
                    world,
                    s,
                    buf,
                    scene,
                    depth,
                    depth_pyramid,
                    detailed_range,
                );
                list.pipeline(pipe);
                list.bind(bind::sky::WORLD_DEPTH, depth);
                Some(prepared)
            } else {
                list.pipeline(cloud_pipe);
                None
            };
            list.bind(bind::sky::OBJECTS, buf);
            list.bind(bind::sky::SCENE, scene);
            list.bind(bind::sky::CLOUD_CELLS, shared.as_ref().and_then(|a| a.density).unwrap_or(scene));
            list.bind(bind::sky::RAIN_DEPTH, shared.as_ref().and_then(|a| a.rain).unwrap_or(scene));
            list.bind(bind::sky::RAIN_COVER, shared.as_ref().and_then(|a| a.cover).unwrap_or(scene));
            list.push(Cmd::Draw { vertices: 3, instances: 1, first_vertex: 0, first_instance });
            (s, shared)
        } else {
            (GpuSky::default(), None)
        };
        if bolts != 0
            && let Some(bolt) = self.bolt_pipe
        {
            list.pipeline(bolt);
            list.bind(bind::sky::OBJECTS, buf);
            list.bind(bind::sky::SCENE, scene);
            list.bind(bind::sky::CLOUD_CELLS, shared.as_ref().and_then(|a| a.density).unwrap_or(scene));
            list.bind(bind::sky::RAIN_DEPTH, shared.as_ref().and_then(|a| a.rain).unwrap_or(scene));
            list.bind(bind::sky::RAIN_COVER, shared.as_ref().and_then(|a| a.cover).unwrap_or(scene));
            list.push(Cmd::Draw { vertices: 6, instances: bolts, first_vertex: 0, first_instance: bolt_from });
        }
        (cloud, shared)
    }
}

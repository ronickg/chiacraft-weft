//! The ground's height about the camera, for shader packs (`Scene::ground_map`, `core:ground_map`):
//! a picture 256 cells each way, 4 blocks a cell (1024 blocks), each cell the top of the ground
//! under the open sky, leaves and other see-through blocks left out, so a pack knows where the
//! land lies under a forest's crowns (chiacraft's mist banks fill valleys and forest floors up to
//! under the crowns, as Bliss's do; held to the sea's height alone, the mist lay over the lakes
//! only, or over the crowns all in the sun, and the canopy's shade never fell on it: the look's
//! live sweeps, 11-24).
//!
//! A cell is the lowest of four columns in it, so a trunk (an opaque log) does not stand up as a
//! spike. Texels: r and g the height's top block + 1, plus 2048, high byte first; b 255 where the
//! top is a liquid; a 255 where it is known (0: its cubes are not here, or nothing solid). It is
//! built on the CPU two rows of cubes a frame and uploaded when a pass ends (the last pass shown
//! meanwhile); a camera 64 blocks from the middle starts a pass about itself.

use glam::{DVec3, IVec3};
use weft_render::{Backend, Format, TextureDesc, TextureId};

use super::world::ClientWorld;
use crate::content::content;

/// Cells each way.
const SIDE: usize = 256;
/// Blocks a cell covers each way.
const CELL: usize = 4;
/// Cubes (16 blocks) the map covers each way.
const CUBES: i32 = (SIDE * CELL / 16) as i32;
/// Rows of cubes built a frame (the whole in 32 frames).
const ROWS_A_FRAME: i32 = 2;
/// How far the camera may move from the map's middle before it is rebuilt about it (cubes).
const RECENTRE: i32 = 4;
/// Added to heights so they stay positive in 16 bits.
const OFFSET: i32 = 2048;

#[derive(Default)]
pub struct GroundMap {
    texture: Option<(TextureId, u32)>,
    /// The pass being built, and its corner (cubes, x and z) and dimension.
    texels: Vec<u8>,
    building: Option<(u32, IVec3)>,
    /// The next row of cubes to build.
    next: i32,
    /// The corner of the map last uploaded (the one the scene points at).
    shown: Option<IVec3>,
    /// What the scene gets (`Scene::ground_map`): x the bindless index + 1 (0: none), y the blocks
    /// a cell covers, z and w the map's corner x and z from the camera (blocks).
    pub scene: [f32; 4],
}

impl GroundMap {
    pub fn destroy(&mut self, gpu: &mut dyn Backend) {
        if let Some((t, _)) = self.texture.take() {
            gpu.destroy_texture(t);
        }
        self.building = None;
        self.shown = None;
        self.scene = [0.0; 4];
    }

    /// Readies this frame's map: `on` from `core:ground_map` (a pack that reads it). The texture
    /// keeps the last whole pass while the next is built, so a camera on the move never waits.
    pub fn frame(&mut self, gpu: &mut dyn Backend, on: bool, cam: DVec3, world: &ClientWorld) {
        if !on
            || !content()
                .dims
                .get(world.dim as usize)
                .is_some_and(|d| d.sky_light)
        {
            self.destroy(gpu);
            return;
        }
        let texture = *self.texture.get_or_insert_with(|| {
            let t = gpu.create_texture(
                &TextureDesc { width: SIDE as u32, height: SIDE as u32, mips: 1, format: Format::Rgba8Unorm, render_target: false },
                "ground map",
            );
            (t, gpu.texture_index(t))
        });
        self.texels.resize(SIDE * SIDE * 4, 0);
        let centre = cam.floor().as_ivec3() >> 4i32;
        let wanted = IVec3::new(centre.x - CUBES / 2, 0, centre.z - CUBES / 2);
        let restart = match self.building {
            Some((dim, e)) => dim != world.dim || (e.x - wanted.x).abs() >= RECENTRE || (e.z - wanted.z).abs() >= RECENTRE,
            None => true,
        };
        if restart {
            if self.building.is_some_and(|(dim, _)| dim != world.dim) {
                self.shown = None;
            }
            self.building = Some((world.dim, wanted));
            self.texels.fill(0);
            self.next = 0;
        }
        let edge = self.building.map_or(wanted, |b| b.1);
        let to = (self.next + ROWS_A_FRAME).min(CUBES);
        self.build_rows(world, edge, self.next, to);
        self.next = to;
        if to >= CUBES {
            gpu.write_texture(texture.0, 0, &self.texels);
            self.shown = Some(edge);
            self.next = 0;
        }
        self.scene = match self.shown {
            Some(shown) => {
                let corner = DVec3::new(shown.x as f64 * 16.0, 0.0, shown.z as f64 * 16.0) - cam;
                [texture.1 as f32 + 1.0, CELL as f32, corner.x as f32, corner.z as f32]
            }
            None => [0.0; 4],
        };
    }

    /// Rebuilds the rows of cubes `from..to` (along z) of the map at `edge`.
    fn build_rows(&mut self, world: &ClientWorld, edge: IVec3, from: i32, to: i32) {
        let content = content();
        let dim = &content.dims[world.dim as usize];
        let (lo, hi) = (dim.min_cube_y, dim.max_cube_y);
        let flags = |state: u32| content.states[state as usize].flags;
        use weft_abi::block_flags::{LIQUID, OPAQUE};
        const ACROSS: usize = 16 / CELL;
        for cz in from..to {
            for cx in 0..CUBES {
                // Each cell: the lowest of its four columns' tops (`None`: not found yet).
                let mut tops = [[None::<(i32, bool)>; 4]; ACROSS * ACROSS];
                let mut left = ACROSS * ACROSS * 4;
                let mut known = false;
                for y in (lo..=hi).rev() {
                    let Some(cube) = world.cube(IVec3::new(edge.x + cx, y, edge.z + cz)) else { continue };
                    known = true;
                    if cube
                        .uniform_state()
                        .is_some_and(|s| flags(s) & (OPAQUE | LIQUID) == 0)
                    {
                        continue;
                    }
                    for (c, cell) in tops.iter_mut().enumerate() {
                        let (gx, gz) = (c % ACROSS, c / ACROSS);
                        for (k, top) in cell.iter_mut().enumerate() {
                            if top.is_some() {
                                continue;
                            }
                            // The four columns: a cell's corners, one block in.
                            let x = gx * CELL + [0, CELL - 1][k % 2];
                            let z = gz * CELL + [0, CELL - 1][k / 2];
                            let hit = (0..16).rev().find_map(|by| {
                                let f = flags(cube.get(crate::world::cube::index(x, by, z)));
                                (f & (OPAQUE | LIQUID) != 0).then_some((by as i32, f & LIQUID != 0))
                            });
                            if let Some((by, liquid)) = hit {
                                *top = Some((y * 16 + by + 1, liquid));
                                left -= 1;
                            }
                        }
                    }
                    if left == 0 {
                        break;
                    }
                }
                for (c, cell) in tops.iter().enumerate() {
                    let (gx, gz) = (c % ACROSS, c / ACROSS);
                    let px = cx as usize * ACROSS + gx;
                    let pz = cz as usize * ACROSS + gz;
                    let at = (pz * SIDE + px) * 4;
                    let low = cell.iter().flatten().min_by_key(|t| t.0);
                    self.texels[at..at + 4].copy_from_slice(&match low {
                        Some(&(h, liquid)) if known => {
                            let v = (h + OFFSET).clamp(0, 65535) as u16;
                            [(v >> 8) as u8, v as u8, if liquid { 255 } else { 0 }, 255]
                        }
                        _ => [0; 4],
                    });
                }
            }
        }
    }
}

//! Blocks and what belongs to them. The world's are a `Space`, a moving structure's are one, and
//! everything that works on blocks works on whichever `Dim::space` is: see `Dim::in_frame`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::{Arc, OnceLock};

use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::content::{AIR, Content};
use crate::physics::Aabb;
use crate::world::{Cube, cube_of, local_index};

/// A timer on a block (`Host::timer`): when, where, the event and its value.
pub type BlockTimer = (u64, [i32; 3], u32, Box<[u8]>);

#[derive(Default)]
pub struct Space {
    pub cubes: FxHashMap<IVec3, Arc<Cube>>,
    pub(super) scheduled: BinaryHeap<Reverse<BlockTimer>>,
    /// Next growth step of every growing block (the source of truth) and the queue ordering
    /// them (entries that no longer match the map are stale).
    pub(super) growth: FxHashMap<IVec3, u64>,
    pub(super) growth_queue: BinaryHeap<Reverse<(u64, i32, i32, i32)>>,
    /// Time of the growth step being run, so blocks it places time from it rather than now.
    pub(super) growing_at: Option<u64>,
    /// Blocks along the edge of area edits waiting for their neighbor update, a share per tick
    /// (a crater's rim is millions).
    pub(super) edge_updates: VecDeque<IVec3>,
    pub(super) neighbor_queue: VecDeque<(IVec3, IVec3)>,
    /// Block changes this tick, sent as deltas.
    pub changes: Vec<(IVec3, u32)>,
    /// The part of a cell whose callback is running, what the mod's `get` and `set` mean there.
    pub(super) acting: Option<(IVec3, u32)>,
    /// Cells whose other blocks changed this tick, with what they have now.
    pub extra_changes: Vec<(IVec3, Vec<u32>)>,
    /// Cubes whose light changed this tick, sent whole.
    pub resend: Vec<(IVec3, Arc<Cube>)>,
    pub(super) light_dirty: FxHashSet<IVec3>,
    /// Cubes changed since the last save. Untouched cubes are never written (regenerating is cheaper).
    pub modified: FxHashSet<IVec3>,
    /// Cubes with block changes since players' sight was last brought up to date (lines of
    /// sight may have opened or closed).
    pub edited: FxHashSet<IVec3>,
    /// Face connectivity of loaded cubes (see `Cube::connectivity`); missing = recompute.
    pub conn: FxHashMap<IVec3, u16>,
    pub grid: super::energy::Grid,
    pub signals: super::signal::Signals,
    /// Event values waiting for delivery, by event id (ordered, so delivery is deterministic).
    pub event_queue: std::collections::BTreeMap<u32, Vec<u8>>,
    /// Bodies that landed here and clients haven't seen arrive, and then those that wait to go,
    /// with the ticks they have left.
    pub(super) landings: Vec<super::structure::Landing>,
    pub(super) lingering: Vec<(u64, u8)>,
    /// Cells of those landings, what clients are left seeing there until then, and whose they are.
    pub(super) not_shown: FxHashMap<IVec3, (u32, u64)>,
    /// Light from bodies going through here: the cells they light from, each body's and in all.
    pub(super) body_lights: FxHashMap<u64, Vec<(IVec3, [u8; 3])>>,
    pub(super) body_light: FxHashMap<IVec3, [u8; 3]>,
    /// Cells of that which changed, for clients.
    pub(super) body_light_changes: Vec<(IVec3, [u8; 3])>,
    /// The world's light where a body's blocks are, by its cubes, and whether that is news.
    pub world_light: FxHashMap<IVec3, u32>,
    pub(super) world_light_changed: bool,
    /// A block giving light changed on this body.
    pub(super) emitters_changed: bool,
    /// A moving structure's place among the world's cells: what counts as next to what.
    pub cell: Option<IVec3>,
    pub(super) projected_pose: Option<crate::physics::Transform>,
    /// Keeps the occupied cells of a mount in order for its per-tick sweep.
    mounted_blocks: Option<Vec<(IVec3, u32)>>,
    colliders: OnceLock<BodyColliders>,
    pub(super) geometry_version: u64,
    pub(super) dry_boxes: Vec<(IVec3, IVec3)>,
    pub(super) dry_changed: bool,
    /// Its circuits and the world's have heard of each other.
    pub in_touch: bool,
    /// Going to a whole block, to land there.
    pub(super) stopping: bool,
    /// Floaters set it off: without any left it stops, as without power.
    pub(super) floated: bool,
    /// Where it was and which way it was going when it last looked at what is ahead.
    pub(super) looked_from: Option<(IVec3, IVec3)>,
    /// A moving structure's block bounds (inclusive) and cubes to light again.
    pub bounds: Option<(IVec3, IVec3)>,
    pub relight: FxHashSet<IVec3>,
}

pub(super) struct BodyColliders {
    pub bounds: Option<Aabb>,
    pub groups: Vec<ColliderGroup>,
}

pub(super) struct ColliderGroup {
    pub area: Aabb,
    pub boxes: Vec<Aabb>,
}

pub(super) fn block_collider(p: IVec3, bounds: [f32; 6]) -> Aabb {
    let [x, y, z, x2, y2, z2] = bounds.map(f64::from);
    let p = p.as_dvec3();
    Aabb { min: p + glam::DVec3::new(x, y, z), max: p + glam::DVec3::new(x2, y2, z2) }
}

impl crate::physics::Blocks for Space {
    fn block(&self, p: IVec3) -> u32 {
        self.get(p).unwrap_or(AIR)
    }
    fn bounds(&self) -> Option<(IVec3, IVec3)> {
        self.bounds
    }
    fn extras(&self, p: IVec3) -> &[u32] {
        self.extras(p)
    }
}

/// The cubes of a world as physics reads them, on any thread while they stay put.
#[derive(Clone, Copy)]
pub struct CubeCells<'a>(pub &'a FxHashMap<IVec3, Arc<Cube>>);

impl CubeCells<'_> {
    pub(super) fn may_collide(&self, c: &Content, area: crate::physics::Aabb) -> bool {
        let lo = cube_of(area.min.floor().as_ivec3() - IVec3::Y);
        let hi = cube_of(area.max.floor().as_ivec3());
        let solid = |s: u32| !c.states[s as usize].collision.is_empty() || c.covered_step(s).is_some();
        for z in lo.z..=hi.z {
            for y in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    let Some(cube) = self.0.get(&IVec3::new(x, y, z)) else { return true };
                    if cube.uniform_state().map_or_else(
                        || {
                            cube.palette().iter().copied().any(solid)
                                || cube
                                    .all_extras()
                                    .flat_map(|(_, s)| s.iter().copied())
                                    .any(solid)
                        },
                        solid,
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl crate::physics::Cells for CubeCells<'_> {
    fn main(&self, p: IVec3) -> Option<u32> {
        self.0.get(&cube_of(p)).map(|cube| cube.get(local_index(p)))
    }
    fn extras(&self, p: IVec3) -> &[u32] {
        self.0
            .get(&cube_of(p))
            .map_or(&[], |cube| cube.extras(local_index(p)))
    }
    #[inline]
    fn cell(&self, p: IVec3) -> Option<(u32, &[u32])> {
        let (cube, i) = (self.0.get(&cube_of(p))?, local_index(p));
        Some((cube.get(i), cube.extras(i)))
    }
}

/// A body's blocks to walk on: everything outside them is air, nothing unloaded.
pub struct DeckCells<'a>(pub &'a Space);

impl crate::physics::Cells for DeckCells<'_> {
    fn main(&self, p: IVec3) -> Option<u32> {
        Some(self.0.get(p).unwrap_or(AIR))
    }
    fn extras(&self, p: IVec3) -> &[u32] {
        Space::extras(self.0, p)
    }
}

impl crate::physics::Cells for Space {
    fn main(&self, p: IVec3) -> Option<u32> {
        self.get(p)
    }
    fn extras(&self, p: IVec3) -> &[u32] {
        Space::extras(self, p)
    }
}

impl Space {
    /// The blocks of a moving structure.
    pub fn with_blocks(c: &Content, blocks: &[(IVec3, u32)]) -> Space {
        let mut s = Space::default();
        s.build(c, blocks);
        s
    }

    pub fn from_saved(c: &Content, blocks: Vec<(IVec3, u32)>) -> Space {
        let mut body = Self::default();
        let mut main = Vec::new();
        let mut extras: FxHashMap<IVec3, Vec<u32>> = FxHashMap::default();
        for (at, state) in blocks {
            if main.last().is_some_and(|&(p, _)| p == at) {
                extras.entry(at).or_default().push(state);
            } else {
                main.push((at, state));
            }
        }
        body.build(c, &main);
        for (at, states) in extras {
            body.put_extras(at, states);
        }
        body
    }

    /// Its blocks to begin with, with what comes with them there already.
    pub fn build(&mut self, c: &Content, blocks: &[(IVec3, u32)]) {
        for &(p, state) in blocks {
            self.put(c, p, state);
        }
        self.relight();
        self.edited.clear();
        let cubes: Vec<(IVec3, Arc<Cube>)> = self
            .cubes
            .iter()
            .map(|(p, cube)| (*p, cube.clone()))
            .collect();
        for (p, cube) in cubes {
            self.signals.cube_loaded(c, p, &cube);
        }
    }

    /// All its blocks but air.
    pub fn all_blocks(&self) -> Vec<(IVec3, u32)> {
        self.block_snapshot().into_owned()
    }

    pub(super) fn track_mounted_blocks(&mut self) {
        self.mounted_blocks = Some(self.all_blocks());
    }

    pub(super) fn colliders(&self, c: &Content) -> &BodyColliders {
        self.colliders.get_or_init(|| {
            let mut groups = std::collections::BTreeMap::new();
            for &(p, state) in self.block_snapshot().iter() {
                for part in std::iter::once(state).chain(self.extras(p).iter().copied()) {
                    let info = &c.states[part as usize];
                    if info.liquid.is_some() || c.covered_step(part).is_some() {
                        continue;
                    }
                    for &bounds in &info.collision {
                        let shape = block_collider(p, bounds);
                        let group = groups
                            .entry((p >> 2_i32).to_array())
                            .or_insert_with(|| ColliderGroup { area: shape, boxes: Vec::new() });
                        group.area.min = group.area.min.min(shape.min);
                        group.area.max = group.area.max.max(shape.max);
                        group.boxes.push(shape);
                    }
                }
            }
            let groups: Vec<_> = groups.into_values().collect();
            let bounds = groups
                .iter()
                .map(|g| g.area)
                .reduce(|a, b| Aabb { min: a.min.min(b.min), max: a.max.max(b.max) });
            BodyColliders { bounds, groups }
        })
    }

    pub(super) fn block_snapshot(&self) -> std::borrow::Cow<'_, [(IVec3, u32)]> {
        if let Some(cells) = &self.mounted_blocks {
            return std::borrow::Cow::Borrowed(cells);
        }
        let mut out = Vec::new();
        for (&cp, cube) in &self.cubes {
            if cube.uniform_state() == Some(AIR) {
                continue;
            }
            for i in 0..crate::world::cube::VOLUME {
                let s = cube.get(i);
                if s != AIR {
                    out.push((cp * 16 + crate::world::cube::offset(i), s));
                }
            }
        }
        out.sort_unstable_by_key(|b| b.0.to_array());
        std::borrow::Cow::Owned(out)
    }

    pub fn get(&self, p: IVec3) -> Option<u32> {
        self.cubes
            .get(&cube_of(p))
            .map(|cube| cube.get(local_index(p)))
    }

    /// What else is in the cell at `p`.
    pub fn extras(&self, p: IVec3) -> &[u32] {
        self.cubes
            .get(&cube_of(p))
            .map_or(&[], |cube| cube.extras(local_index(p)))
    }

    /// Puts what else is in a cell of a structure without block updates (building it).
    pub(super) fn put_extras(&mut self, p: IVec3, extras: Vec<u32>) {
        if let Some(cube) = self.cubes.get_mut(&cube_of(p)) {
            Arc::make_mut(cube).set_extras(local_index(p), extras);
            self.geometry_changed();
        }
    }

    /// Writes a block of a structure without block updates (building it, loading it).
    fn put(&mut self, c: &Content, p: IVec3, state: u32) {
        let cp = cube_of(p);
        self.structure_cube(c, cp);
        if let Some(cube) = self.cubes.get_mut(&cp) {
            Arc::make_mut(cube).set(local_index(p), state);
        }
        self.grow_bounds(p, state);
        self.relight.insert(cp);
    }

    /// Makes cube `cp` of a structure exist (air) so blocks can go there.
    pub fn structure_cube(&mut self, c: &Content, cp: IVec3) {
        if !self.cubes.contains_key(&cp) {
            let cube = Cube::uniform(AIR, crate::world::light::SKY_FULL as u32);
            self.grid.cube_loaded(c, cp, &cube);
            self.signals.cube_loaded(c, cp, &cube);
            self.cubes.insert(cp, Arc::new(cube));
        }
    }

    pub(super) fn geometry_changed(&mut self) {
        self.geometry_version = self.geometry_version.wrapping_add(1);
        self.colliders.take();
    }

    pub fn grow_bounds(&mut self, p: IVec3, state: u32) {
        self.geometry_changed();
        if let Some(cells) = &mut self.mounted_blocks {
            match cells.binary_search_by_key(&p.to_array(), |(at, _)| at.to_array()) {
                Ok(i) if state == AIR => {
                    cells.remove(i);
                }
                Ok(i) => cells[i].1 = state,
                Err(i) if state != AIR => cells.insert(i, (p, state)),
                Err(_) => {}
            }
        }
        if state == AIR {
            // Shrinking needs a look over everything (only when the edge itself went).
            if self
                .bounds
                .is_some_and(|(lo, hi)| p.cmpeq(lo).any() || p.cmpeq(hi).any())
            {
                self.bounds = self.scan_bounds();
            }
        } else {
            self.bounds = Some(
                self.bounds
                    .map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p))),
            );
        }
    }

    fn scan_bounds(&self) -> Option<(IVec3, IVec3)> {
        crate::world::block_bounds(self.cubes.iter().map(|(p, c)| (p, &**c)))
    }

    /// A structure lights each of its cubes as open to the sky above (it moves under the sky of
    /// whatever world it is in).
    pub fn relight(&mut self) {
        let c = crate::content::content();
        for cp in std::mem::take(&mut self.relight) {
            if let Some(cube) = self.cubes.get_mut(&cp) {
                let cube = Arc::make_mut(cube);
                cube.fill_light(0);
                crate::world::light::prelight(c, cube, true);
                self.resend.push((cp, Arc::new(cube.clone())));
            }
        }
    }

    /// The blocks of a structure for saving: (position, state name), air left out.
    pub fn structure_blocks(&self, c: &Content) -> Vec<(IVec3, String)> {
        self.all_blocks()
            .into_iter()
            .flat_map(|(p, s)| {
                std::iter::once(s)
                    .chain(self.extras(p).iter().copied())
                    .map(move |s| (p, c.state_name(s)))
            })
            .collect()
    }
}

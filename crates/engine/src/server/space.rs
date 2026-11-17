//! Blocks and their local simulation, for terrain regions and moving bodies alike.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::{Arc, OnceLock};

use glam::IVec3;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::content::{AIR, Content};
use crate::physics::{Aabb, Solid, block_solids};
use crate::world::light::{self, Lighter, Sky};
use crate::world::{Cube, cube_of, local_index};

use super::dim::Cubes;

/// A timer on a block (`Host::timer`): when, where, the event and its value.
pub type BlockTimer = (u64, [i32; 3], u32, Box<[u8]>);

#[derive(Default)]
pub struct Space {
    pub(super) finite: bool,
    pub(super) mass: std::sync::OnceLock<f64>,
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
    /// Bodies transferring into this space, through display and late-input retirement.
    pub(super) landings: Vec<super::structure::Landing>,
    /// Where the cells that transferred out of this display-only body were committed.
    pub(super) transfer: Option<Box<super::structure::Transfer>>,
    /// Cells of those landings, what clients are left seeing there until then, and whose they are.
    pub(super) not_shown: FxHashMap<IVec3, (u32, Vec<u32>, u64)>,
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
    pub(super) junction_pose: Option<(crate::physics::Transform, bool)>,
    pub(super) live_helm: Option<IVec3>,
    /// Keeps the occupied cells of a mount in order for its per-tick sweep.
    mounted_blocks: Option<Vec<(IVec3, u32)>>,
    colliders: OnceLock<BodyColliders>,
    pub(super) geometry_version: u64,
    pub(super) dry_boxes: Vec<(IVec3, IVec3)>,
    pub(super) dry_changed: bool,
    /// Its circuits and the world's have heard of each other.
    pub in_touch: bool,
    pub(super) floater_path: Option<FloaterPath>,
    /// A finite space's block bounds (inclusive).
    pub bounds: Option<(IVec3, IVec3)>,
}

#[derive(Clone, Copy)]
pub(super) struct FloaterPath {
    pub cell: IVec3,
    pub direction: IVec3,
    pub geometry: u64,
    pub breaks: bool,
    pub blocked: Option<IVec3>,
}

pub(super) struct BodyColliders {
    pub bounds: Option<Aabb>,
    pub groups: Vec<ColliderGroup>,
    pub grip_radius: f64,
}

pub(super) struct ColliderGroup {
    pub area: Aabb,
    pub shapes: Vec<Solid>,
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

impl crate::physics::Cells for Space {
    fn main(&self, p: IVec3) -> Option<u32> {
        self.get(p)
    }
    fn extras(&self, p: IVec3) -> &[u32] {
        Space::extras(self, p)
    }
    fn cell(&self, p: IVec3) -> Option<(u32, &[u32])> {
        let Some(cube) = self.cubes.get(&cube_of(p)) else { return self.finite.then_some((AIR, &[])) };
        let index = local_index(p);
        Some((cube.get(index), cube.extras(index)))
    }
}

impl Space {
    pub fn finite() -> Self {
        Self { finite: true, ..Self::default() }
    }

    pub fn restore_timers(&mut self, timers: Vec<BlockTimer>) {
        self.scheduled.extend(timers.into_iter().map(Reverse));
    }

    /// The blocks of a moving structure.
    pub fn with_blocks(c: &Content, blocks: &[(IVec3, u32)]) -> Space {
        let mut s = Self::finite();
        s.build(c, blocks, []);
        s
    }

    pub fn from_saved(c: &Content, blocks: Vec<(IVec3, u32)>) -> Space {
        let mut body = Self::finite();
        let mut main = Vec::new();
        let mut extras: FxHashMap<IVec3, Vec<u32>> = FxHashMap::default();
        for (at, state) in blocks {
            if main.last().is_some_and(|&(p, _)| p == at) {
                extras.entry(at).or_default().push(state);
            } else {
                main.push((at, state));
            }
        }
        body.build(c, &main, extras);
        body
    }

    /// Its blocks to begin with, with what comes with them there already.
    pub fn build(&mut self, c: &Content, blocks: &[(IVec3, u32)], extras: impl IntoIterator<Item = (IVec3, Vec<u32>)>) {
        for &(pos, state) in blocks {
            let cube = self
                .cubes
                .entry(cube_of(pos))
                .or_insert_with(|| Arc::new(Cube::uniform(AIR, 0)));
            Arc::make_mut(cube).set(local_index(pos), state);
            self.grow_bounds(pos, state);
        }
        for (pos, parts) in extras {
            if let Some(cube) = self.cubes.get_mut(&cube_of(pos)) {
                Arc::make_mut(cube).set_extras(local_index(pos), parts);
            }
        }
        self.light_structure(c);
        self.light_dirty.clear();
        for (&pos, cube) in &self.cubes {
            self.grid.cube_loaded(c, pos, cube);
            self.signals.cube_loaded(c, pos, cube);
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
            let mut grip_radius = 0.0_f64;
            for &(p, state) in self.block_snapshot().iter() {
                for part in std::iter::once(state).chain(self.extras(p).iter().copied()) {
                    let info = &c.states[part as usize];
                    if info.liquid.is_some() || c.covered_step(part).is_some() {
                        continue;
                    }
                    for shape in block_solids(p, info) {
                        if let Solid::Cylinder(wheel) = shape
                            && wheel.grip > 0.0
                        {
                            grip_radius = grip_radius.max(wheel.radius);
                        }
                        let bounds = shape.bounds();
                        let group = groups
                            .entry((p >> 2_i32).to_array())
                            .or_insert_with(|| ColliderGroup { area: bounds, shapes: Vec::new() });
                        group.area.min = group.area.min.min(bounds.min);
                        group.area.max = group.area.max.max(bounds.max);
                        group.shapes.push(shape);
                    }
                }
            }
            let groups: Vec<_> = groups.into_values().collect();
            let bounds = groups
                .iter()
                .map(|g| g.area)
                .reduce(|a, b| Aabb { min: a.min.min(b.min), max: a.max.max(b.max) });
            BodyColliders { bounds, groups, grip_radius }
        })
    }

    pub(super) fn coaxial_tire(&self, c: &Content, axis: glam::DVec3) -> bool {
        matches!(self.colliders(c).groups.as_slice(), [g] if matches!(g.shapes.as_slice(), [Solid::Cylinder(tire)]
            if tire.axis.dot(axis).abs() > 1.0 - 1e-8 && (tire.center - glam::DVec3::splat(0.5)).cross(axis).length_squared() < 1e-12))
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
            .or(self.finite.then_some(AIR))
    }

    /// What else is in the cell at `p`.
    pub fn extras(&self, p: IVec3) -> &[u32] {
        self.cubes
            .get(&cube_of(p))
            .map_or(&[], |cube| cube.extras(local_index(p)))
    }

    /// Allocates the air cube addressed by a write in a finite space.
    pub fn structure_cube(&mut self, c: &Content, cp: IVec3) {
        if !self.cubes.contains_key(&cp) {
            self.cubes.insert(cp, Arc::new(Cube::uniform(AIR, 0)));
            if self.cubes.contains_key(&(cp + IVec3::Y)) && self.cubes.contains_key(&(cp - IVec3::Y)) {
                // Filling a gap closes a previous sky boundary.
                self.light_structure(c);
            } else {
                let mut cubes = Cubes(&mut self.cubes);
                let mut light = Lighter::new(&mut cubes, c, true);
                light.light_new(cp, true);
                self.light_dirty.extend(light.changed);
            }
        }
    }

    pub(super) fn geometry_changed(&mut self) {
        self.geometry_version = self.geometry_version.wrapping_add(1);
        self.colliders.take();
        self.mass.take();
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

    fn light_structure(&mut self, c: &Content) {
        let mut columns: Vec<_> = self.cubes.keys().copied().collect();
        columns.sort_unstable_by_key(|pos| std::cmp::Reverse(pos.y));
        for cube in self.cubes.values_mut() {
            Arc::make_mut(cube).fill_light(0);
        }
        let mut cubes = Cubes(&mut self.cubes);
        let mut light = Lighter::new(&mut cubes, c, true);
        for pos in columns {
            light.light_new(pos, true);
        }
        self.light_dirty.extend(light.changed);
    }

    pub(super) fn relight_at(&mut self, c: &Content, sky_light: bool, pos: IVec3, old: Option<u32>) {
        let sky = Sky::local(self.finite, sky_light);
        self.light_dirty
            .extend(light::relight(&mut Cubes(&mut self.cubes), c, sky, &self.body_light, pos, old));
    }

    pub(super) fn queue_cell(&mut self, pos: IVec3) {
        self.neighbor_queue.push_back((pos, pos));
        self.neighbor_queue
            .extend(crate::world::DIRS.map(|offset| (pos + offset, pos)));
    }

    pub(super) fn publish(&mut self) {
        self.changes
            .retain(|(pos, _)| !self.light_dirty.contains(&cube_of(*pos)));
        self.extra_changes
            .retain(|(pos, _)| !self.not_shown.contains_key(pos) && !self.light_dirty.contains(&cube_of(*pos)));
        for cp in self.light_dirty.drain() {
            if let Some(cube) = self.cubes.get(&cp) {
                self.resend
                    .push((cp, Self::mask_cube(&self.not_shown, cp, cube.clone())));
            }
        }
    }

    pub(super) fn visible_cube(&self, pos: IVec3, cube: Arc<Cube>) -> Arc<Cube> {
        Self::mask_cube(&self.not_shown, pos, cube)
    }

    fn mask_cube(hidden: &FxHashMap<IVec3, (u32, Vec<u32>, u64)>, cp: IVec3, mut cube: Arc<Cube>) -> Arc<Cube> {
        for (&pos, (state, parts, _)) in hidden.iter().filter(|(pos, _)| cube_of(**pos) == cp) {
            let cube = Arc::make_mut(&mut cube);
            cube.set(local_index(pos), *state);
            cube.set_extras(local_index(pos), parts.clone());
        }
        cube
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

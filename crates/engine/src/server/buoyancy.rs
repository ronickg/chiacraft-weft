use glam::{DQuat, DVec3, IVec2, IVec3};
use rustc_hash::{FxHashMap, FxHashSet};
use weft_abi::{self as abi, entry_kind as ek};

use super::dim::Dim;
use super::entity::Motion;
use super::space::{BodyColliders, ColliderGroup, CubeCells, DeckCells, Space, block_collider};
use crate::content::Content;
use crate::physics::{Aabb, BoxContact, Cells, Transform};
use crate::world::{DIRS, TICK};

pub(super) struct Ship {
    pub rotation: DQuat,
    pub spin: DVec3,
    pub front: u8,
    pub(super) controls: Option<abi::ShipControls>,
    geometry: Option<Hull>,
}

impl Ship {
    pub fn new(front: u8, rotation: DQuat, spin: DVec3) -> Self {
        Self { rotation, spin, front, controls: None, geometry: None }
    }

    pub(super) fn flooded_cells(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.geometry.iter().flat_map(|hull| {
            hull.air
                .iter()
                .zip(&hull.flooded)
                .filter_map(|(cell, &wet)| wet.then_some(cell.pos))
        })
    }
}

struct Hull {
    version: u64,
    air: Vec<AirCell>,
    flooded: Vec<bool>,
    mass: f64,
    center: DVec3,
    inertia: DVec3,
    radius: f64,
}

struct AirCell {
    pos: IVec3,
    neighbors: [Option<usize>; 6],
    openings: u8,
}

fn cube(p: IVec3) -> Aabb {
    let min = p.as_dvec3();
    Aabb { min, max: min + DVec3::ONE }
}

/// Candidate air has walls in both horizontal directions and a floor below it.
fn hull_air(seals: &FxHashSet<IVec3>) -> Vec<AirCell> {
    let line = |p: IVec3, axis: usize| IVec2::new(p[(axis + 1) % 3], p[(axis + 2) % 3]);
    let mut ends: [FxHashMap<IVec2, (i32, i32)>; 3] = std::array::from_fn(|_| FxHashMap::default());
    for &p in seals {
        for (axis, lines) in ends.iter_mut().enumerate() {
            let v = p[axis];
            let (lo, hi) = lines.entry(line(p, axis)).or_insert((v, v));
            *lo = (*lo).min(v);
            *hi = (*hi).max(v);
        }
    }
    let axis = [0, 2]
        .into_iter()
        .min_by_key(|&a| {
            ends[a]
                .values()
                .map(|&(lo, hi)| i64::from(hi) - i64::from(lo))
                .sum::<i64>()
        })
        .unwrap();
    let mut candidates = Vec::new();
    for (&other, &(lo, hi)) in &ends[axis] {
        let mut p = IVec3::ZERO;
        p[(axis + 1) % 3] = other.x;
        p[(axis + 2) % 3] = other.y;
        for v in lo + 1..hi {
            p[axis] = v;
            if !seals.contains(&p)
                && ends[2 - axis]
                    .get(&line(p, 2 - axis))
                    .is_some_and(|&(lo, hi)| p[2 - axis] > lo && p[2 - axis] < hi)
                && ends[1]
                    .get(&line(p, 1))
                    .is_some_and(|&(floor, _)| p.y > floor)
            {
                candidates.push(p);
            }
        }
    }
    candidates.sort_unstable_by_key(|p| p.to_array());
    let index: FxHashMap<_, _> = candidates
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();
    candidates
        .into_iter()
        .map(|pos| {
            let mut openings = 0;
            let neighbors = std::array::from_fn(|face| {
                let adjacent = pos + DIRS[face];
                let next = index.get(&adjacent).copied();
                if next.is_none() && !seals.contains(&adjacent) {
                    openings |= 1 << face;
                }
                next
            });
            AirCell { pos, neighbors, openings }
        })
        .collect()
}

fn dry_boxes(air: &[AirCell], flooded: &[bool]) -> Vec<(IVec3, IVec3)> {
    let dry: Vec<_> = air
        .iter()
        .zip(flooded)
        .filter_map(|(a, &wet)| (!wet).then_some(a.pos))
        .collect();
    let mut left: FxHashSet<_> = dry.iter().copied().collect();
    let mut boxes = Vec::new();
    for lo in dry {
        if !left.contains(&lo) {
            continue;
        }
        let mut hi = lo + IVec3::ONE;
        while left.contains(&IVec3::new(lo.x, lo.y, hi.z)) {
            hi.z += 1;
        }
        while (lo.z..hi.z).all(|z| left.contains(&IVec3::new(lo.x, hi.y, z))) {
            hi.y += 1;
        }
        while (lo.y..hi.y).all(|y| (lo.z..hi.z).all(|z| left.contains(&IVec3::new(hi.x, y, z)))) {
            hi.x += 1;
        }
        for x in lo.x..hi.x {
            for y in lo.y..hi.y {
                for z in lo.z..hi.z {
                    left.remove(&IVec3::new(x, y, z));
                }
            }
        }
        boxes.push((lo, hi));
    }
    boxes
}

impl Hull {
    fn new(c: &Content, space: &Space) -> Self {
        let density = c.components.id(abi::comp::block::DENSITY);
        let mut masses = Vec::new();
        let mut seals = FxHashSet::default();
        for &(p, state) in space.block_snapshot().iter() {
            for st in std::iter::once(state).chain(space.extras(p).iter().copied()) {
                let info = &c.states[st as usize];
                if info.liquid.is_some() || c.covered_step(st).is_some() {
                    continue;
                }
                let d = c
                    .block_value::<f32>(st, density)
                    .filter(|n| n.is_finite() && *n > 0.0)
                    .map_or(1.45, f64::from);
                let min = p.as_dvec3();
                if info.collision.is_empty() {
                    masses.push((min + DVec3::splat(0.5), d * 0.04));
                }
                for &bounds in &info.collision {
                    let shape = block_collider(p, bounds);
                    let size = shape.max - shape.min;
                    let volume = size.x * size.y * size.z;
                    let (lo, hi) = ((shape.min - 1e-8).ceil().as_ivec3(), (shape.max + 1e-8).floor().as_ivec3());
                    for x in lo.x..hi.x {
                        for y in lo.y..hi.y {
                            for z in lo.z..hi.z {
                                seals.insert(IVec3::new(x, y, z));
                            }
                        }
                    }
                    masses.push(((shape.min + shape.max) * 0.5, d * volume));
                }
            }
        }
        let mass = masses.iter().map(|(_, m)| m).sum::<f64>().max(0.01);
        let center = masses.iter().map(|(p, m)| *p * *m).sum::<DVec3>() / mass;
        let inertia = masses
            .iter()
            .map(|(p, m)| {
                let r = *p - center;
                (DVec3::splat(r.length_squared() + 1.0 / 6.0) - r * r) * *m
            })
            .sum::<DVec3>()
            .max(DVec3::splat(0.01));
        let radius = space
            .colliders(c)
            .groups
            .iter()
            .flat_map(|g| &g.boxes)
            .map(|b| (b.min - center).abs().max((b.max - center).abs()).length())
            .fold(1.0, f64::max);
        let air = hull_air(&seals);
        let flooded = vec![false; air.len()];
        Self { version: space.geometry_version, air, flooded, mass, center, inertia, radius }
    }

    fn flood(&mut self, c: &Content, world: &impl Cells, place: Transform) -> bool {
        let mut flooded = vec![false; self.air.len()];
        let mut open = Vec::new();
        let downhill = -place.vector_to_local(DVec3::Y).signum() * 0.48;
        let touches_water = |p: IVec3, face: IVec3| {
            let f = face.as_dvec3();
            let center = p.as_dvec3() + DVec3::splat(0.5) + f * 0.501;
            let tangent = DVec3::ONE - f.abs();
            let low = downhill * tangent;
            let wet = |p| crate::physics::liquid_at(c, world, place.to_world(p)).is_some_and(|id| !c.liquids[id as usize].rises);
            wet(center + low) || wet(center)
        };
        for (i, cell) in self.air.iter().enumerate().filter(|(_, a)| a.openings != 0) {
            if DIRS
                .iter()
                .enumerate()
                .any(|(face, &d)| cell.openings & (1 << face) != 0 && touches_water(cell.pos, d))
            {
                flooded[i] = true;
                open.push(i);
            }
        }
        while let Some(i) = open.pop() {
            let cell = &self.air[i];
            for (face, next) in cell.neighbors.iter().enumerate() {
                if let Some(j) = *next
                    && !flooded[j]
                    && touches_water(cell.pos, DIRS[face])
                {
                    flooded[j] = true;
                    open.push(j);
                }
            }
        }
        let changed = self.flooded != flooded;
        self.flooded = flooded;
        changed
    }

    fn submerged(&self, c: &Content, cells: &impl Cells, solid: &[ColliderGroup], place: Transform, center: DVec3) -> (f64, DVec3) {
        let mut total = 0.0;
        let mut moment = DVec3::ZERO;
        let dry = self
            .air
            .iter()
            .zip(&self.flooded)
            .filter_map(|(a, &wet)| (!wet).then_some(cube(a.pos)));
        for b in solid
            .iter()
            .flat_map(|g| g.boxes.iter().copied())
            .chain(dry)
        {
            let size = b.max - b.min;
            let volume = size.x * size.y * size.z;
            let span = place.box_to_world(b);
            let height = (span.max.y - span.min.y).max(1e-6);
            let mid = (b.min + b.max) * 0.5;
            for (x, z) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
                let p = place.to_world(mid + DVec3::new(x * size.x * 0.25, 0.0, z * size.z * 0.25));
                let low = p.y - height * 0.5;
                let high = p.y + height * 0.5;
                let at = p.floor().as_ivec3();
                for y in low.floor() as i32..=high.floor() as i32 {
                    let Some((liquid, surface)) = crate::physics::liquid_surface(c, cells, at.with_y(y)) else { continue };
                    if c.liquids[liquid as usize].rises {
                        continue;
                    }
                    let bottom = low.max(y as f64);
                    let top = high.min(surface);
                    let submerged = (top - bottom).max(0.0) * volume / (height * 4.0);
                    total += submerged;
                    if submerged > 0.0 {
                        moment += (p.with_y((top + bottom) * 0.5) - center).cross(DVec3::Y * submerged);
                    }
                }
            }
        }
        (total, moment)
    }
}

impl Dim {
    pub(crate) fn set_ship_controls(&mut self, id: u64, input: Option<abi::ShipControls>) -> bool {
        let Some(ship) = self
            .entities
            .get_mut(id)
            .filter(|e| !e.removed && e.motion != Motion::Parked)
            .and_then(|e| e.ship.as_mut())
        else {
            return false;
        };
        if input.is_some_and(|i| !i.throttle.is_finite() || !i.steering.is_finite()) {
            return false;
        }
        ship.controls = input.map(|i| abi::ShipControls { throttle: i.throttle.clamp(-1.0, 1.0), steering: i.steering.clamp(-1.0, 1.0) });
        true
    }

    pub(crate) fn ship_flooded_cells(&self, id: u64) -> Option<impl Iterator<Item = IVec3> + '_> {
        let e = self
            .entities
            .get(id)
            .filter(|e| !e.removed && e.motion != Motion::Parked)?;
        Some(e.ship.as_ref()?.flooded_cells())
    }

    pub(super) fn float_ship(&mut self, id: u64, front: u8) {
        if let Some(e) = self.entities.get_mut(id) {
            e.ship = Some(Box::new(Ship::new(front, DQuat::IDENTITY, DVec3::ZERO)));
        }
        if let Some(space) = self.bodies.get_mut(&id) {
            space.track_mounted_blocks();
        }
    }

    pub(super) fn advance_ships(&mut self, c: &'static Content, before: &mut FxHashMap<u64, Transform>) {
        let mut ships: Vec<_> = self
            .bodies
            .keys()
            .copied()
            .filter(|id| {
                self.entities
                    .get(*id)
                    .is_some_and(|e| e.ship.is_some() && e.motion != Motion::Parked)
            })
            .collect();
        ships.sort_unstable();
        let watch_hits = c
            .game
            .event_names
            .get(abi::event::SHIP_COLLISION)
            .is_some_and(|&event| !c.game.events[event as usize].handlers.is_empty());
        for id in ships {
            if self.bodies.get(&id).is_some_and(|b| b.stopping) && self.try_dock(c, id) {
                continue;
            }
            let Some(e) = self.entities.get_mut(id) else { continue };
            let old = e.body_place();
            let (pos, vel) = (e.pos, e.vel);
            let mut ship = e.ship.take().unwrap();
            let Some(space) = self.bodies.get(&id) else { continue };
            let edited = ship
                .geometry
                .as_ref()
                .is_none_or(|h| h.version != space.geometry_version);
            if edited {
                ship.geometry = Some(Hull::new(c, space));
            }
            let hull = ship.geometry.as_mut().unwrap();
            let dry = (hull.flood(c, &CubeCells(&self.space.cubes), old) || edited).then(|| dry_boxes(&hull.air, &hull.flooded));
            let (drive, steer) = ship
                .controls
                .map_or_else(|| self.ship_input(c, id), |input| (f64::from(input.throttle), -f64::from(input.steering)));
            let mut collisions = Vec::new();
            let (pos, vel, spin, rotation) = self.step_ship(
                c,
                id,
                hull,
                space.colliders(c),
                ShipStep { old, pos, vel, spin: ship.spin },
                ship.front,
                drive,
                steer,
                watch_hits.then_some(&mut collisions),
            );
            if let Some(space) = self.bodies.get_mut(&id) {
                space.dry_changed = dry.as_ref().is_some_and(|boxes| *boxes != space.dry_boxes);
                if let Some(boxes) = dry {
                    space.dry_boxes = boxes;
                }
            }
            if let Some(e) = self.entities.get_mut(id) {
                e.pos = pos;
                e.vel = vel;
                ship.rotation = rotation;
                ship.spin = spin;
                e.ship = Some(ship);
                if old != e.body_place() {
                    before.insert(id, old);
                }
            }
            for collision in collisions {
                self.emit_engine(c, abi::event::SHIP_COLLISION, &collision);
            }
        }
    }

    fn try_dock(&mut self, c: &'static Content, id: u64) -> bool {
        self.in_frame(id, |d| d.finish_moving_here(c));
        let Some(e) = self.entities.get(id) else { return false };
        let Some(space) = self.bodies.get(&id) else { return false };
        let ahead = e.body_place().vector_to_world(DVec3::Z);
        let quarter = std::f64::consts::FRAC_PI_2;
        let yaw = (ahead.x.atan2(ahead.z) / quarter).round() * quarter;
        let rotation = DQuat::from_rotation_y(yaw);
        let pos = Transform::of(e.pos).cell(None).as_dvec3() + Transform::FEET;
        let place = Transform::rotated(pos, rotation);
        if !self.mount_fits(c, place, &space.block_snapshot()) {
            return false;
        }
        if let Some((lo, hi)) = space.bounds {
            let area = place.box_to_world(Aabb { min: lo.as_dvec3(), max: (hi + IVec3::ONE).as_dvec3() });
            let other_here = self.bodies.iter().any(|(&other, b)| {
                if other == id {
                    return false;
                }
                let Some(e) = self
                    .entities
                    .get(other)
                    .filter(|e| e.motion != Motion::Parked)
                else {
                    return false;
                };
                b.bounds.is_some_and(|(lo, hi)| {
                    area.intersects(
                        &e.body_place()
                            .box_to_world(Aabb { min: lo.as_dvec3(), max: (hi + IVec3::ONE).as_dvec3() }),
                    )
                })
            });
            if other_here {
                return false;
            }
        }
        if let Some(rider) = self.rider_in(id, 0) {
            let seat = c.game.entities[e.kind as usize].seats.first();
            let at = seat.map(|s| place.to_world(Transform::FEET + s.pos));
            if !self.dismount_at(c, rider, at) {
                return false;
            }
        }
        let Some(e) = self.entities.get_mut(id) else { return false };
        e.pos = pos;
        if let Some(ship) = &mut e.ship {
            ship.rotation = rotation;
            ship.spin = DVec3::ZERO;
        }
        self.put_down_whole(c, id);
        if let Some(space) = self.bodies.get_mut(&id) {
            space.dry_changed = !space.dry_boxes.is_empty();
            space.dry_boxes.clear();
        }
        true
    }

    fn ship_input(&self, c: &Content, ship: u64) -> (f64, f64) {
        let Some(e) = self.rider_in(ship, 0).and_then(|id| self.entities.get(id)) else { return (0.0, 0.0) };
        let held = |name| {
            c.game
                .catalog
                .find(ek::KEY, name)
                .is_some_and(|id| e.keys.iter().any(|&(k, _)| k == id)) as u8 as f64
        };
        (held("core:forward") - held("core:back"), held("core:left") - held("core:right"))
    }

    fn step_ship(
        &self,
        c: &Content,
        id: u64,
        hull: &Hull,
        colliders: &BodyColliders,
        step: ShipStep,
        front: u8,
        drive: f64,
        steer: f64,
        mut collisions: Option<&mut Vec<abi::ShipCollision>>,
    ) -> (DVec3, DVec3, DVec3, DQuat) {
        let ShipStep { old, pos, vel, spin } = step;
        let Some(bounds) = colliders.bounds else {
            return (pos, DVec3::ZERO, DVec3::ZERO, old.rotation());
        };
        let solid = &colliders.groups;
        let world = CubeCells(&self.space.cubes);
        let center = old.to_world(hull.center);
        let gravity = c.dims[self.id as usize].gravity as f64;
        let trace = crate::log::on(crate::log::Channel::Physics, log::Level::Debug);
        let water_started = trace.then(std::time::Instant::now);
        let (wet, moment) = hull.submerged(c, &world, solid, old, center);
        let water_time = water_started.map_or(0.0, |start| start.elapsed().as_secs_f64() * 1000.0);
        let contact = (wet / hull.mass).clamp(0.0, 1.0);
        let forward = old
            .vector_to_world(DIRS[front as usize].as_dvec3())
            .with_y(0.0)
            .normalize_or_zero();
        let thrust = crate::config::get(c.game.ship_thrust);
        let water_drag = crate::config::get(c.game.ship_water_drag);
        let max_speed = crate::config::get(c.game.ship_max_speed);
        let acceleration = DVec3::Y * (gravity * (wet / hull.mass - 1.0)) + forward * drive * contact * thrust;
        let vel = ((vel + acceleration * TICK) * (-TICK * (0.05 + contact * water_drag)).exp()).clamp_length_max(max_speed);
        let torque = old.vector_to_world(old.vector_to_local(moment * gravity) / hull.inertia) + DVec3::Y * steer * contact * 2.7;
        let turn_travel = 0.22;
        let spin = ((spin + torque * TICK) * (-TICK * (0.1 + contact * 2.8)).exp())
            .clamp_length_max((turn_travel / (hull.radius * TICK)).min(2.5));
        let at = |center, rotation| center - rotation * (hull.center - DVec3::splat(0.5)) - DVec3::Y * 0.5;
        let started = trace.then(std::time::Instant::now);
        let probes = std::cell::Cell::new(0);
        let envelope = old.box_to_world(bounds);
        let margin = DVec3::splat((vel.length() * TICK + turn_travel).max(0.5));
        let swept = Aabb { min: envelope.min - margin, max: envelope.max + margin };
        let mut others: Vec<_> = self
            .bodies
            .iter()
            .filter_map(|(&other, space)| {
                let e = self.entities.get(other)?;
                if id == other || e.frame == Some(id) || e.motion == Motion::Parked {
                    return None;
                }
                let place = e.body_place();
                let bounds = place.box_to_world(space.colliders(c).bounds?);
                swept
                    .intersects(&bounds)
                    .then_some((other, place, bounds, &**space))
            })
            .collect();
        others.sort_unstable_by_key(|entry| entry.0);
        let contacts = |center, rotation, delta: DVec3, angular: DVec3| {
            let place = Transform::rotated(at(center, rotation), rotation);
            let mut impact = Impact::default();
            let mut report = |other, hit: BoxContact| {
                let arm = hit.point - (center + delta * hit.fraction);
                if delta == DVec3::ZERO || hit.depth > 0.0 || (delta + angular.cross(arm)).dot(hit.normal) < -1e-9 {
                    impact.add(other, hit);
                }
            };
            ship_collision_shapes(c, &world, solid, &others, place, delta, |b, place, delta, peer| {
                if trace {
                    probes.set(probes.get() + 1);
                }
                match peer {
                    None => crate::physics::rotated_contacts(c, &world, b, place, delta, |hit| report(0, hit)),
                    Some((id, other, body)) => {
                        crate::physics::rotated_contacts(c, &DeckCells(body), b, place, delta, |mut hit| {
                            hit.normal = other.vector_to_world(hit.normal);
                            hit.point = other.to_world(hit.point);
                            report(id, hit);
                        });
                    }
                }
                false
            });
            impact.contact()
        };
        let overlaps = |center, rotation| {
            let place = Transform::rotated(at(center, rotation), rotation);
            ship_collision_shapes(c, &world, solid, &others, place, DVec3::ZERO, |b, place, _, peer| {
                if trace {
                    probes.set(probes.get() + 1);
                }
                match peer {
                    None => crate::physics::rotated_collides(c, &world, b, place),
                    Some((_, _, body)) => crate::physics::rotated_collides(c, &DeckCells(body), b, place),
                }
            })
        };
        let mut moved = center;
        let mut vel = vel;
        let mut spin = spin;
        let mut rotation = old.rotation();
        let mut time = TICK;
        let mut impacts = 0;
        for _ in 0..5 {
            let delta = vel * time;
            let angle = spin * time;
            let pose = |f| (DQuat::from_scaled_axis(angle * f) * rotation).normalize();
            let mut hit = if delta.length_squared() > 1e-16 { contacts(moved, rotation, delta, angle) } else { None };
            let mut fraction = hit.map_or(1.0, |(_, h)| h.fraction);
            if angle.length_squared() > 1e-16 && fraction > 0.0 {
                let mut free = 0.0;
                for t in [fraction * 0.5, fraction] {
                    if overlaps(moved + delta * t, pose(t)) {
                        let mut blocked = t;
                        for _ in 0..6 {
                            let mid = (free + blocked) * 0.5;
                            if overlaps(moved + delta * mid, pose(mid)) {
                                blocked = mid;
                            } else {
                                free = mid;
                            }
                        }
                        fraction = free;
                        hit = contacts(moved + delta * blocked, pose(blocked), DVec3::ZERO, DVec3::ZERO);
                        break;
                    }
                    free = t;
                }
            }
            moved += delta * fraction;
            rotation = pose(fraction);
            let Some((other, hit)) = hit else { break };
            impacts += 1;
            let arm = hit.point - moved;
            let axis = arm.cross(hit.normal);
            let turn = rotation * ((rotation.conjugate() * axis) / hull.inertia);
            let normal_speed = (vel + spin.cross(arm)).dot(hit.normal);
            let impulse = (-normal_speed).max(0.0) / (hull.mass.recip() + axis.dot(turn));
            if impulse > 0.0
                && let Some(collisions) = collisions.as_mut()
            {
                let event = abi::ShipCollision {
                    ship: id,
                    other,
                    point: hit.point.to_array().into(),
                    normal: hit.normal.to_array().into(),
                    speed: -normal_speed,
                    impulse,
                };
                if let Some(last) = collisions.iter_mut().find(|e| e.other == other) {
                    if impulse > last.impulse {
                        *last = event;
                    }
                } else {
                    collisions.push(event);
                }
            }
            vel += hit.normal * (impulse / hull.mass);
            spin += turn * impulse;
            moved += hit.normal * (hit.depth.clamp(0.0, 0.025) + 1e-5);
            time *= 1.0 - fraction;
            if time < 1e-6 {
                break;
            }
        }
        if let Some(started) = started {
            log::debug!(target: "physics", "ship {id}: collision {:.3} ms, buoyancy {:.3} ms, {} box queries, {} nearby bodies; pos={:?}, mass={:.2}, displaced={:.2}, air={}, up={:?}, front={:?}, impacts={impacts}, vel={vel:?}, spin={spin:?}",
                started.elapsed().as_secs_f64() * 1000.0, water_time, probes.get(), others.len(), at(moved, rotation), hull.mass, wet, hull.flooded.iter().filter(|&&f| !f).count(),
                rotation * DVec3::Y, rotation * DIRS[front as usize].as_dvec3());
        }
        (at(moved, rotation), vel, spin, rotation)
    }
}

fn ship_collision_shapes(
    c: &Content,
    world: &CubeCells<'_>,
    solid: &[ColliderGroup],
    others: &[(u64, Transform, Aabb, &Space)],
    place: Transform,
    delta: DVec3,
    mut visit: impl FnMut(Aabb, Transform, DVec3, Option<(u64, Transform, &Space)>) -> bool,
) -> bool {
    for group in solid {
        let area = place.box_to_world(group.area).swept(delta);
        if world.may_collide(c, area)
            && crate::physics::collides(c, world, area)
            && group.boxes.iter().any(|&b| visit(b, place, delta, None))
        {
            return true;
        }
        for &(id, other, bounds, body) in others
            .iter()
            .filter(|(_, _, bounds, _)| bounds.intersects(&area))
        {
            let relative = place.relative_to(other);
            let moved = other.vector_to_local(delta);
            if group
                .boxes
                .iter()
                .any(|&b| bounds.intersects(&place.box_to_world(b).swept(delta)) && visit(b, relative, moved, Some((id, other, body))))
            {
                return true;
            }
        }
    }
    false
}

#[derive(Default)]
struct Impact {
    first: Option<BoxContact>,
    other: u64,
    point_sum: DVec3,
    count: u32,
}

impl Impact {
    fn add(&mut self, other: u64, hit: BoxContact) {
        let better = self.first.is_none_or(|old| {
            hit.fraction < old.fraction - 1e-5 || (hit.fraction - old.fraction).abs() <= 1e-5 && hit.depth > old.depth + 1e-4
        });
        if better {
            *self = Self { first: Some(hit), other, point_sum: hit.point, count: 1 };
        } else if self.other == other
            && self.first.is_some_and(|old| {
                (hit.fraction - old.fraction).abs() <= 1e-5 && (hit.depth - old.depth).abs() <= 1e-4 && hit.normal.dot(old.normal) > 0.999
            })
        {
            self.point_sum += hit.point;
            self.count += 1;
        }
    }

    fn contact(self) -> Option<(u64, BoxContact)> {
        self.first
            .map(|hit| (self.other, BoxContact { point: self.point_sum / self.count as f64, ..hit }))
    }
}

struct ShipStep {
    old: Transform,
    pos: DVec3,
    vel: DVec3,
    spin: DVec3,
}

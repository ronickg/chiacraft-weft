//! What each player hears every tick: cubes, entities, their own state, an open menu.

use super::*;

/// Entities' moves go to a player every tick within a third of `core:entity_send_distance`,
/// every 2nd tick within two thirds, every `FAR_EVERY`th beyond. (At its largest, 1024, all
/// within the simulation distance go every tick, as before it was there.)
const FAR_EVERY: u64 = 4;

/// An entity that moved in the last `FAR_EVERY` ticks, where it is now.
pub(super) struct Moved {
    mv: crate::net::Move,
    /// The tick it last moved.
    at: u64,
    /// Sent every tick it moves, however far.
    exact: bool,
}

/// How often moves go to a player this far away (squared), out of `send2`, the send
/// distance's square.
fn move_every(dist2: f64, send2: f64) -> u64 {
    if dist2 <= send2 / 9.0 {
        1
    } else if dist2 <= send2 * 4.0 / 9.0 {
        2
    } else {
        FAR_EVERY
    }
}

/// Whether a move goes now to a player hearing it every `every` ticks, on its own `phase`
/// (`tick` shifted by something of the player's): on its ticks, if it moved since the last one.
fn move_due(phase: u64, every: u64, moved_at: u64, tick: u64) -> bool {
    phase % every == 0 && moved_at + every > tick
}

impl Server {
    pub(super) fn sync_players(&mut self, c: &'static Content) {
        let mining_now = self.mining_now();
        let settled: Vec<Vec<IVec3>> = self.dims.iter_mut().map(|d| d.take_settled()).collect();
        let moved = self.collect_moves(c);

        let mut unsent = 0u64;
        for pi in 0..self.players.len() {
            if !self.players[pi].streams_world() {
                continue;
            }
            let view = self.view(&self.players[pi]);
            let full = self.full_view(&self.players[pi]);
            let Server { players, dims, tick, storage, tick_rate, .. } = self;
            let p = &mut players[pi];
            let dim = &mut dims[p.dim as usize];
            let d = p.dim as usize;
            send_far(c, p, dim, storage.as_ref(), full, *tick);
            send_block_changes(p, dim);
            send_clock(p, dim, *tick, *tick_rate);
            if !p.joined {
                p.first_sync = false;
                unsent += send_cubes(p, dim, &c.dims[d], view, full, &settled[d]);
                continue;
            }
            send_mining(p, &mining_now[d], *tick);
            let viewer = dim
                .entity(p.entity)
                .map_or((DVec3::ZERO, GameMode::SURVIVAL), |e| (e.pos, e.mode));
            let send = crate::config::get(c.game.entity_send_distance);
            send_entities(c, p, dim, &moved[d], *tick, (send, view.radius as f64 * 16.0), viewer);
            send_own_state(c, p, dim);
            let shapes = gizmos_for(p, dim, *tick, viewer.0);
            if !shapes.is_empty() {
                p.conn.send(ToClient::Gizmos(shapes));
            }
            unsent += send_cubes(p, dim, &c.dims[d], view, full, &settled[d]);
        }
        self.shared.unsent.store(unsent, Ordering::Relaxed);
        let pending = self.backlog.len() + self.dims.iter().map(|d| d.generating.len()).sum::<usize>();
        self.shared.pending.store(pending as u64, Ordering::Relaxed);
        self.clear_tick_changes();
        self.unload_unused(c);
    }

    /// Who is breaking what in each dimension, in tenths as crack pictures go, so most ticks
    /// nothing is sent.
    fn mining_now(&self) -> Vec<Vec<crate::net::Crack>> {
        self.dims
            .iter()
            .map(|d| {
                d.regions
                    .iter()
                    .flat_map(|r| &r.entities.list)
                    .filter(|e| !e.removed)
                    .filter_map(|e| e.mining.filter(|m| m.frame == 0).map(|m| (e.id, m)))
                    .map(|(id, m)| (id, m.pos, m.part, (m.progress.min(0.99) * 10.0).floor() / 10.0))
                    .collect()
            })
            .collect()
    }

    /// Entities that moved in the last `FAR_EVERY` ticks, per dimension (all regions), where
    /// they are now.
    fn collect_moves(&mut self, c: &'static Content) -> Vec<Vec<Moved>> {
        let tick = self.tick;
        let mut moved: Vec<Vec<Moved>> = Vec::with_capacity(self.dims.len());
        for d in &mut self.dims {
            let mut m = Vec::new();
            for r in &mut d.regions {
                r.publish_spaces();
                r.mirror_fields(c);
                r.entities.find_changes(&c.game.synced_comps);
                for e in r
                    .entities
                    .list
                    .iter_mut()
                    .filter(|e| !e.removed && e.anchor.is_none())
                {
                    let now = crate::net::Move {
                        id: e.id,
                        pos: e.pos,
                        yaw: e.yaw,
                        pitch: e.pitch,
                        body_yaw: e.body_yaw,
                        strode: e.strode,
                        stride: e.stride,
                        pose: e.pose,
                        sprinting: e.sprinting,
                        frame: e.frame.unwrap_or(0),
                        rotation: e.wire_rotation(),
                    };
                    // Solid bodies every tick: clients show them between server positions, and a
                    // body that stops must say so (two equal positions).
                    let solid = c.game.entities[e.kind as usize].is(crate::tag_bits::entity::SOLID);
                    if solid
                        || e.sent.is_none_or(|s| {
                            (s.pos - now.pos).length_squared() > 1e-6
                                || (s.yaw - now.yaw).abs() > 0.01
                                || (s.pitch - now.pitch).abs() > 0.01
                                || s.body_yaw != now.body_yaw
                                || s.strode != now.strode
                                || s.stride != now.stride
                                || s.pose != now.pose
                                || s.sprinting != now.sprinting
                                || s.frame != now.frame
                        })
                    {
                        if e.sent.is_some() {
                            e.moved_at = tick;
                        }
                        e.sent = Some(now);
                    }
                    if e.moved_at + FAR_EVERY > tick
                        && let Some(mv) = e.sent
                    {
                        // Bodies and what rides them go every tick: clients time bodies by the
                        // tick and place riders on them.
                        let exact = solid || e.frame.is_some();
                        m.push(Moved { mv, at: e.moved_at, exact });
                    }
                }
            }
            moved.push(m);
        }
        moved
    }

    /// This tick's lists were heard by everyone.
    fn clear_tick_changes(&mut self) {
        for d in &mut self.dims {
            d.day_moved = false;
            d.weather_moved = false;
            d.air_moved = false;
            d.sky_event_moved = false;
            d.landings.clear();
            d.tints_moved = false;
            for r in &mut d.regions {
                for space in std::iter::once(&mut r.space).chain(r.bodies.values_mut()) {
                    space.resend.clear();
                    space.changes.clear();
                    space.extra_changes.clear();
                    space.signals.level_changes.clear();
                    space.body_light_changes.clear();
                    space.world_light_changed = false;
                    space.dry_changed = false;
                    space.lean_moved = None;
                }
                r.bodies.values_mut().for_each(|space| space.edited.clear());
                r.entities.killed.clear();
                r.entities.taken.clear();
                r.entities.told.clear();
                r.entities.dirty_comp.clear();
                r.gizmos.clear();
                // Block entity inventory changes must reach the save.
                let anchored: Vec<IVec3> = r
                    .entities
                    .list
                    .iter()
                    .filter(|e| e.anchor.is_some() && r.entities.dirty_inv.contains(&e.id))
                    .map(|e| cube_of(e.pos.floor().as_ivec3()))
                    .collect();
                r.space.modified.extend(anchored);
                r.entities.dirty_inv.clear();
            }
        }
    }
}

fn send_block_changes(p: &mut Player, dim: &DimSet) {
    for reg in &dim.regions {
        // For what the client works out itself; all of it when it gets here, then what changes.
        let glow: Vec<(IVec3, [u8; 3])> =
            if !p.glow_sent { reg.space.body_light.iter().map(|(c, v)| (*c, *v)).collect() } else { reg.space.body_light_changes.clone() };
        if !glow.is_empty() {
            p.conn.send(ToClient::BodyGlow { dim: p.dim, cells: glow });
        }
        send_space_changes(&mut p.conn, p.dim, 0, &reg.space, |cube| p.sent.contains(&cube));
    }
    p.glow_sent = true;
}

fn send_space_changes(conn: &mut crate::net::ServerConn, dim: u32, frame: u64, space: &super::space::Space, sent: impl Fn(IVec3) -> bool) {
    for (pos, cube) in &space.resend {
        if sent(*pos) {
            conn.send(ToClient::Cube { dim, frame, pos: *pos, cube: cube.clone(), edit: true });
        }
    }
    let changes: Vec<_> = space
        .changes
        .iter()
        .filter(|(pos, _)| sent(cube_of(*pos)))
        .copied()
        .collect();
    let extras: Vec<_> = space
        .extra_changes
        .iter()
        .filter(|(pos, _)| sent(cube_of(*pos)))
        .cloned()
        .collect();
    if !changes.is_empty() || !extras.is_empty() {
        if log::log_enabled!(target: "churn", log::Level::Debug) {
            let c = crate::content::content();
            for (pos, state) in changes.iter().take(8) {
                log::debug!(target: "churn", "block {} {} {} now {}", pos.x, pos.y, pos.z, c.state_name(*state));
            }
            for (pos, _) in space.resend.iter().filter(|(pos, _)| sent(*pos)).take(4) {
                log::debug!(target: "churn", "cube resent {} {} {}", pos.x, pos.y, pos.z);
            }
        }
        conn.send(ToClient::Blocks { dim, frame, changes, extras });
    }
    let changes: Vec<_> = space
        .signals
        .level_changes
        .iter()
        .filter(|(pos, ..)| sent(cube_of(pos.cell)))
        .copied()
        .collect();
    if !changes.is_empty() {
        conn.send(ToClient::Signals { dim, frame, changes });
    }
}

/// Everyone else's mining in this dimension: cracks for those near, and their arms.
fn send_mining(p: &mut Player, cracks: &[crate::net::Crack], tick: u64) {
    let mine: Vec<crate::net::Crack> = cracks
        .iter()
        .copied()
        .filter(|m| m.0 != p.entity && p.known.contains(&m.0))
        .collect();
    if mine != p.sent_cracks {
        p.conn.send(ToClient::Cracks(mine.clone()));
        p.sent_cracks = mine;
    }
    if tick % 5 == 0 {
        for m in cracks
            .iter()
            .filter(|m| m.0 != p.entity && p.known.contains(&m.0))
        {
            p.conn.send(ToClient::EntitySwing { id: m.0, off: false });
        }
    }
}

fn send_clock(p: &mut Player, dim: &DimSet, tick: u64, rate: u32) {
    if p.first_sync || p.sent_rate != rate || dim.weather_moved || dim.day_moved || tick % TPS as u64 == 0 {
        p.sent_rate = rate;
        p.conn.send(ToClient::Clock {
            dim: p.dim,
            time: dim.day_time(),
            ticks: dim.time,
            weather: dim.weather_wire(),
            rate,
            day_rate: dim.day_rate as f32,
        });
    }
    // chiacraft: a mod's weather by place, its air, its sky, its tints.
    if dim.weather_moved || p.first_sync {
        p.conn
            .send(ToClient::Weather { id: dim.weather_wire(), only: dim.rain_here().map(|o| o.to_vec()) });
    }
    if dim.air_moved || p.first_sync {
        p.conn
            .send(ToClient::Air { air: dim.air.0.clone(), heading: dim.air.1 });
    }
    if dim.sky_event_moved || p.first_sync {
        p.conn
            .send(ToClient::SkyEvent { kind: dim.sky_event.0, strength: dim.sky_event.1 });
    }
    for &(pos, seconds) in &dim.landings {
        p.conn.send(ToClient::SkyLanding { pos, seconds });
    }
    if dim.tints_moved || (p.first_sync && !dim.tints.is_empty()) {
        p.conn.send(ToClient::Tints { tints: dim.tints.to_vec() });
    }
}

/// Entities within the send distance (and the simulated blocks): spawn new ones, despawn gone
/// ones, stream movement.
fn send_entities(
    c: &'static Content,
    p: &mut Player,
    dim: &DimSet,
    moved: &[Moved],
    tick: u64,
    (send, simulated): (f64, f64),
    (me, me_mode): (DVec3, GameMode),
) {
    let range2 = send.min(simulated).powi(2);
    let mount = dim
        .entity(p.entity)
        .and_then(|e| e.vehicle)
        .map(|(id, _)| id);
    let mut spawn = Vec::new();
    let mut visible = FxHashSet::default();
    for reg in &dim.regions {
        // Spectators are seen only by other spectators.
        // Hidden ones are seen by the hidden alone.
        let hidden = |m: GameMode| m.has(crate::tag_bits::game_mode::HIDDEN);
        let shown = |e: &&Entity| !hidden(e.mode) || hidden(me_mode);
        // A dead player is gone for the others (its death animation plays) until it respawns.
        for e in reg
            .entities
            .list
            .iter()
            .filter(|e| e.seen(c) && e.id != p.entity && (!e.player || e.alive()))
            .filter(shown)
        {
            if !p.known.contains(&e.id)
                && reg
                    .bodies
                    .get(&e.id)
                    .and_then(|space| space.transfer.as_ref())
                    .is_some_and(|transfer| transfer.shown_at.is_some())
            {
                continue;
            }
            if Some(e.id) != mount && (e.pos - me).length_squared() > range2 {
                let near_body = reg
                    .bodies
                    .get(&e.id)
                    .and_then(|b| b.bounds)
                    .is_some_and(|(lo, hi)| {
                        let bounds = e
                            .body_place()
                            .box_to_world(crate::physics::Aabb { min: lo.as_dvec3(), max: (hi + IVec3::ONE).as_dvec3() });
                        (me - me.clamp(bounds.min, bounds.max)).length_squared() <= range2
                    });
                if !near_body {
                    continue;
                }
            }
            visible.insert(e.id);
            if !p.known.contains(&e.id) {
                spawn.push(EntityView {
                    id: e.id,
                    kind: e.kind,
                    has_space: reg.bodies.contains_key(&e.id),
                    pos: e.pos,
                    yaw: e.yaw,
                    pitch: e.pitch,
                    born: tick.saturating_sub(e.age),
                    body_yaw: e.body_yaw,
                    strode: e.strode,
                    stride: e.stride,
                    rotation: e.wire_rotation(),
                });
            }
        }
    }
    let taken: Vec<(u64, u64)> = dim
        .regions
        .iter()
        .flat_map(|r| r.entities.taken.iter())
        .filter(|t| p.known.contains(&t.0))
        .copied()
        .collect();
    if !taken.is_empty() {
        p.conn.send(ToClient::Taken(taken));
    }
    // A mob killed this tick is gone already: the flag starts its death animation.
    let killed = |id: &u64| dim.regions.iter().any(|r| r.entities.killed.contains(id));
    let gone: Vec<(u64, bool)> = p
        .known
        .iter()
        .copied()
        .filter(|id| !visible.contains(id))
        .map(|id| (id, killed(&id)))
        .collect();
    if !gone.is_empty() {
        if log::log_enabled!(target: "churn", log::Level::Debug) {
            for (id, killed) in gone.iter().take(8) {
                log::debug!(target: "churn", "despawn {id} killed {killed}");
            }
        }
        p.conn.send(ToClient::Despawn(gone));
    }
    // Components every viewer gets: all of newly seen entities, changes of known ones.
    let mut shared: Vec<(u64, u32, Vec<u8>)> = Vec::new();
    for reg in &dim.regions {
        for &(id, comp) in &reg.entities.dirty_comp {
            if id != p.entity
                && p.known.contains(&id)
                && visible.contains(&id)
                && c.game.components[comp as usize].flags & cf::SYNC_WATCHERS != 0
            {
                shared.extend(
                    reg.entities
                        .comps
                        .get(&(id, comp))
                        .map(|b| (id, comp, b.to_vec())),
                );
            }
        }
        for v in &spawn {
            for &comp in &c.game.watched_comps {
                shared.extend(
                    reg.entities
                        .comps
                        .get(&(v.id, comp))
                        .map(|b| (v.id, comp, b.to_vec())),
                );
            }
        }
    }
    if !spawn.is_empty() {
        // chiacraft: what keeps coming into view (`--log churn=debug`), for the bench that never
        // goes idle after a far teleport (weft sync 11-22).
        if log::log_enabled!(target: "churn", log::Level::Debug) {
            for v in spawn.iter().take(8) {
                let name = c
                    .game
                    .entities
                    .get(v.kind as usize)
                    .map_or("?", |k| k.name.as_str());
                log::debug!(target: "churn", "spawn {} {} at {:.1} {:.1} {:.1}, age {}", v.id, name, v.pos.x, v.pos.y, v.pos.z, tick.saturating_sub(v.born));
            }
        }
        p.conn.send(ToClient::Spawn(spawn));
    }
    for reg in &dim.regions {
        for (&frame, space) in &reg.bodies {
            if !visible.contains(&frame) {
                continue;
            }
            let new = !p.known.contains(&frame);
            if if new { !space.dry_boxes.is_empty() } else { space.dry_changed } {
                p.conn
                    .send(ToClient::BodyAir { frame, boxes: space.dry_boxes.clone() });
            }
            if new || space.world_light_changed {
                p.conn.send(ToClient::BodyLight {
                    frame,
                    lights: space
                        .world_light
                        .iter()
                        .map(|(&pos, &light)| (pos, light))
                        .collect(),
                });
            }
            // How it leans (chiacraft, felling): all of it when first seen, then when told.
            if let Some(ticks) = if new { space.lean.map(|_| 0) } else { space.lean_moved } {
                let (pivot, rot) = space.lean.unwrap_or(([0.0; 3], [0.0, 0.0, 0.0, 1.0]));
                p.conn.send(ToClient::BodyLean { frame, pivot, rot, ticks });
            }
            if new {
                for (&pos, cube) in &space.cubes {
                    p.send_space_cube(c, frame, pos, cube.clone(), Some(space));
                }
            } else {
                send_space_changes(&mut p.conn, p.dim, frame, space, |_| true);
            }
        }
    }
    for reg in &dim.regions {
        for (&frame, space) in &reg.bodies {
            let Some(transfer) = space
                .transfer
                .as_ref()
                .filter(|transfer| transfer.shown_at == Some(reg.time) && visible.contains(&frame))
            else {
                continue;
            };
            let destination = transfer.destination;
            let mut cubes: Vec<_> = transfer
                .parts
                .values()
                .flat_map(|&(pos, part)| {
                    let footprint = part.footprint(c);
                    std::iter::once(cube_of(pos))
                        .chain(footprint.iter().map(|&d| cube_of(pos + d)))
                        .collect::<Vec<_>>()
                })
                .filter(|pos| if destination == 0 { p.sent.contains(pos) } else { visible.contains(&destination) })
                .collect();
            cubes.sort_unstable_by_key(|pos| pos.to_array());
            cubes.dedup();
            p.conn
                .send(ToClient::SpaceLanded { dim: p.dim, frame, destination, cubes });
        }
    }
    if !shared.is_empty() {
        p.conn.send(ToClient::EntityComponents(shared));
    }
    // Far ones every few ticks, where they are now if they moved since the last time (each
    // player on its own ticks, so not all at once).
    let due = |m: &Moved| {
        let every = if m.exact { 1 } else { move_every((m.mv.pos - me).length_squared(), send * send) };
        move_due(tick + p.entity, every, m.at, tick)
    };
    let moves: Vec<crate::net::Move> = moved
        .iter()
        .filter(|m| p.known.contains(&m.mv.id) && visible.contains(&m.mv.id) && due(m))
        .map(|m| m.mv)
        .collect();
    // Every tick, moving or not: clients show what they see by its timeline, clips included.
    p.conn.send(ToClient::Moves { tick, moves });
    p.known = visible;
}

/// What only the player itself hears: pushes, movement numbers, inventory, components, its menu.
fn send_own_state(c: &'static Content, p: &mut Player, dim: &mut DimSet) {
    let Some(reg) = dim.entity_region(p.entity) else { return };
    for &(_, told) in reg.entities.told.iter().filter(|h| h.0 == p.entity) {
        p.conn.send(match told {
            entity::Told::Knock(push) => ToClient::Knock { push },
            entity::Told::Add(add) => ToClient::Velocity { add },
            entity::Told::Velocity(v) => ToClient::SetVelocity(v),
            entity::Told::Look(look) => ToClient::Look(look),
            entity::Told::Flying(on) => ToClient::Flying(on),
            entity::Told::Selected(slot) => ToClient::Selected(slot),
        });
    }
    // Own inventory and synced components.
    if p.first_sync || reg.entities.dirty_inv.contains(&p.entity) {
        if let Some(inv) = reg.entities.inventories.get(&p.entity) {
            p.conn.send(ToClient::Inventory(
                inv.iter()
                    .enumerate()
                    .map(|(i, s)| (i as u32, crate::stack::for_client(*s)))
                    .collect(),
            ));
        }
    }
    let comps: Vec<(u32, Vec<u8>)> = reg
        .entities
        .comps
        .iter()
        .filter(|((id, comp), _)| {
            *id == p.entity
                && c.game.components[*comp as usize].flags & cf::SYNC_OWNER != 0
                && (p.first_sync || reg.entities.dirty_comp.contains(&(*id, *comp)))
        })
        .map(|((_, comp), b)| (*comp, b.to_vec()))
        .collect();
    if !comps.is_empty() {
        p.conn.send(ToClient::Components(comps));
    }
    p.first_sync = false;
    if let Some(m) = &mut p.menu {
        // The machine's menu-synced components, all at first, then what changed.
        let owner = m.owner();
        let first = !std::mem::replace(&mut m.comps_sent, true);
        let comps: Vec<(u64, u32, Vec<u8>)> = c
            .game
            .menu_comps
            .iter()
            .filter(|&&comp| owner != 0 && (first || reg.entities.dirty_comp.contains(&(owner, comp))))
            .filter_map(|&comp| {
                reg.entities
                    .comps
                    .get(&(owner, comp))
                    .map(|b| (owner, comp, b.to_vec()))
            })
            .collect();
        if !comps.is_empty() {
            p.conn.send(ToClient::EntityComponents(comps));
        }
        let (slots, cursor) = menu::changes(m, reg, c, p.entity);
        if !slots.is_empty() {
            p.conn.send(ToClient::MenuSlots(slots));
        }
        if let Some(message) = cursor {
            p.conn.send(message);
        }
    }
}

/// Cubes that came into view, nearest first, and unloading those far behind. Returns how many are
/// still waiting for their turn.
fn send_cubes(p: &mut Player, dim: &DimSet, def: &crate::content::DimensionDef, view: View, full: View, settled: &[IVec3]) -> u64 {
    let (center, r, h, down) = (view.center, view.radius, view.up, view.down);
    let in_range = |pos: IVec3| view.contains_in(def, pos);
    let moved = p.sync_range != Some(view);
    if moved {
        // Settled cubes that came into range; the ones that left are unloaded below.
        let was = p.sync_range;
        p.unsent.retain(|&q| in_range(q));
        for dy in -down..=h {
            for dz in -r..=r {
                for dx in -r..=r {
                    let pos = center + IVec3::new(dx, dy, dz);
                    if in_range(pos) && !was.is_some_and(|w| w.contains(pos)) && !p.sent.contains(&pos) && dim.settled(pos) {
                        p.unsent.insert(pos);
                    }
                }
            }
        }
        p.sync_range = Some(view);
    }
    p.unsent.extend(
        settled
            .iter()
            .copied()
            .filter(|&q| in_range(q) && !p.sent.contains(&q)),
    );
    // Send a cube once its neighbors exist and its sky light is final.
    let mut to_send: Vec<(i32, IVec3)> = p
        .unsent
        .iter()
        .copied()
        .filter(|&pos| {
            crate::world::DIRS.iter().all(|d| {
                let n = pos + *d;
                dim.has(n) || !in_range(n) || !p.vis.as_ref().is_some_and(|v| v.seen.contains(&n))
            })
        })
        .map(|pos| {
            let d = pos - center;
            (d.x * d.x + d.z * d.z + d.y * d.y, pos)
        })
        .collect();
    to_send.sort_by_key(|t| t.0);
    let unsent = to_send.len().saturating_sub(SEND_PER_TICK) as u64;
    for &(_, pos) in to_send.iter().take(SEND_PER_TICK) {
        if let Some(cube) = dim.snapshot(pos) {
            p.send_terrain_cube(dim, pos, cube);
            p.sent.insert(pos);
            p.unsent.remove(&pos);
        }
    }
    if !moved {
        return unsent;
    }
    // A cube more around it, against flicker at the edge.
    let keep = full.grown(2);
    let (_, hi) = full.loaded_y(def);
    let gone = |pos: IVec3| !(keep.column(pos) && pos.y >= keep.center.y - keep.down && pos.y <= hi + 2);
    p.announced.retain(|&q| !gone(q));
    let far: Vec<IVec3> = p.sent.iter().copied().filter(|&q| gone(q)).collect();
    for pos in far {
        p.sent.remove(&pos);
        p.conn.send(ToClient::Unload { dim: p.dim, pos });
    }
    unsent
}

/// Gizmo shapes for one player: what mods drew in the groups it shows, and every second the
/// engine's server-side groups.
pub(super) fn gizmos_for(p: &Player, dim: &DimSet, tick: u64, me: DVec3) -> Vec<crate::net::GizmoShape> {
    use crate::net::GizmoShape;
    let mut out: Vec<GizmoShape> = Vec::new();
    for r in &dim.regions {
        out.extend(
            r.gizmos
                .iter()
                .filter(|(who, g)| {
                    (g.group.is_empty() || g.group == GizmoShape::ASSEMBLY || p.gizmos.contains(&g.group))
                        && super::reaches(*who, p.entity, p.dim, || Some(me), dim.id, |_| true)
                })
                .map(|(_, g)| g.clone()),
        );
    }
    if tick % TPS as u64 != 0 {
        return out;
    }
    let on = |g: &str| p.gizmos.iter().any(|x| x == g);
    let shape = |group: &str, kind: u8, a: DVec3, b: DVec3, color: u32| GizmoShape::shape(group, kind, a, b, color, TPS + 2);
    if on("regions") {
        const COLORS: [u32; 6] = [0xFF4040FF, 0x40FF40FF, 0x4080FFFF, 0xFFFF40FF, 0xFF40FFFF, 0x40FFFFFF];
        for (i, r) in dim.regions.iter().enumerate() {
            let (Some(lo), Some(hi)) = (r.space.cubes.keys().copied().reduce(IVec3::min), r.space.cubes.keys().copied().reduce(IVec3::max))
            else {
                continue;
            };
            let (a, b) = ((lo * 16).as_dvec3() + 0.05, ((hi + 1) * 16).as_dvec3() - 0.05);
            out.push(shape("regions", 1, a, b, COLORS[i % COLORS.len()]));
        }
    }
    if let Some(v) = p.vis.as_ref().filter(|_| on("visibility")) {
        let here: IVec3 = me.floor().as_ivec3() >> 4;
        for &c in &v.seen {
            let d = c - here;
            if d.x.abs() > 8 || d.z.abs() > 8 || d.y.abs() > 4 {
                continue;
            }
            let color = if dim.has(c) { 0x40FF60FF } else { 0xFFA030FF };
            let m = (c * 16).as_dvec3() + 8.0;
            for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
                out.push(shape("visibility", 0, m - axis, m + axis, color));
            }
        }
    }
    out
}

impl Player {
    /// What has power in a cube this player just got.
    pub(super) fn send_terrain_cube(&mut self, dim: &DimSet, pos: IVec3, cube: Arc<Cube>) {
        let space = dim
            .region_index(pos * 16)
            .map(|index| &*dim.regions[index].space);
        self.send_space_cube(crate::content::content(), 0, pos, cube, space);
    }

    fn send_space_cube(&mut self, c: &Content, frame: u64, pos: IVec3, cube: Arc<Cube>, space: Option<&super::space::Space>) {
        let (cube, changes) = if let Some(space) = space {
            let changes = space
                .cubes
                .get(&pos)
                .map(|actual| space.signals.levels_in(c, pos, actual))
                .unwrap_or_default();
            (space.visible_cube(pos, cube), changes)
        } else {
            (cube, Vec::new())
        };
        self.conn
            .send(ToClient::Cube { dim: self.dim, frame, pos, cube, edit: false });
        if !changes.is_empty() {
            self.conn
                .send(ToClient::Signals { dim: self.dim, frame, changes });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_go_less_often_farther_away() {
        let send2 = 96.0f64 * 96.0;
        let every = |d: f64| move_every(d * d, send2);
        assert_eq!((every(0.0), every(32.0), every(33.0), every(64.0), every(65.0), every(500.0)), (1, 1, 2, 2, FAR_EVERY, FAR_EVERY));
        // At the largest send distance everything simulated goes every tick, as before.
        assert_eq!(move_every(128.0 * 128.0, 1024.0 * 1024.0), 1);
    }

    /// An entity moving on some ticks, its player now near, now far: every move reaches it
    /// within the tier's interval, and the last one always does.
    #[test]
    fn a_skipped_tick_loses_no_move() {
        let moves_at = [3u64, 4, 9, 10, 11, 17, 30, 31, 32, 33, 34, 50];
        for phase in 0..FAR_EVERY {
            let mut moved_at = 0;
            let mut heard_at = 0;
            for tick in 1..80u64 {
                if moves_at.contains(&tick) {
                    moved_at = tick;
                }
                let every = [1, 2, FAR_EVERY][(tick / 7 % 3) as usize];
                // `collect_moves` lists it while it moved within the longest interval.
                if moved_at + FAR_EVERY > tick && move_due(tick + phase, every, moved_at, tick) {
                    heard_at = tick;
                }
                // Heard since it last moved, or its tick to hear isn't here yet.
                assert!(
                    heard_at >= moved_at || tick < moved_at + FAR_EVERY,
                    "phase {phase}: moved at {moved_at}, heard at {heard_at}, tick {tick}"
                );
            }
            assert!(heard_at >= 50, "phase {phase}: the last move is heard");
        }
    }
}

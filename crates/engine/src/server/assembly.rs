use glam::IVec3;
use rustc_hash::FxHashSet;
use weft_abi::{self as abi, assembly_result as result};

use super::dim::Dim;
use super::signal::BODY_LIMIT;
use super::structure::{Lifted, Liveness};
use crate::content::{AIR, Content};
use crate::net::codec::Wire;
use crate::physics::Transform;
use crate::world::DIRS;

pub const BOUNDS: &str = "core:assembly_bounds";

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    lo: IVec3,
    hi: IVec3,
}

impl Bounds {
    fn new(a: IVec3, b: IVec3) -> Self {
        Self { lo: a.min(b), hi: a.max(b) }
    }

    fn contains(self, p: IVec3) -> bool {
        p.cmpge(self.lo).all() && p.cmple(self.hi).all()
    }
}

struct Preview {
    controller: IVec3,
    cells: Vec<Lifted>,
    expires: u64,
}

#[derive(Default)]
pub(super) struct Edit {
    dimension: u32,
    corner: Option<IVec3>,
    bounds: Option<Bounds>,
    preview: Option<Preview>,
}

impl Dim {
    pub(super) fn assembly_bounds(&self, c: &Content, p: IVec3) -> Option<Bounds> {
        let id = self.anchored(p)?;
        let data = self.entities.comps.get(&(id, c.component(BOUNDS)?))?;
        let (lo, hi) = <(IVec3, IVec3)>::from_bytes(data)?;
        Some(Bounds::new(lo + p, hi + p))
    }

    fn save_assembly_bounds(&mut self, c: &Content, p: IVec3, bounds: Bounds) -> bool {
        let (Some(id), Some(comp)) = (self.anchored(p), c.component(BOUNDS)) else { return false };
        self.entities
            .comps
            .insert((id, comp), (bounds.lo - p, bounds.hi - p).to_bytes().into_boxed_slice());
        true
    }

    pub(super) fn turn_assembly_bounds(&mut self, c: &Content, p: IVec3, place: Transform) {
        let Some(bounds) = self.assembly_bounds(c, p) else { return };
        let root = place.cell_at(IVec3::ZERO);
        self.save_assembly_bounds(c, p, Bounds::new(place.cell_at(bounds.lo - root), place.cell_at(bounds.hi - root)));
    }

    pub(super) fn connected_assembly(&self, c: &Content, p: IVec3, bounds: Option<Bounds>) -> Result<Vec<IVec3>, u32> {
        let state = self.get(p).ok_or(result::UNAVAILABLE)?;
        let signal = c.states[state as usize].signal;
        let helm = signal.flags as u32 & abi::signal_flags::HELM != 0;
        let grip = if helm { p } else { p + DIRS[signal.front as usize] };
        let allowed = |at| (helm || at != p) && bounds.is_none_or(|b| b.contains(at));
        let mut seen = FxHashSet::default();
        let mut pending = vec![grip];
        let mut cells = Vec::new();
        while let Some(at) = pending.pop() {
            if !allowed(at) || !seen.insert(at) {
                continue;
            }
            let state = self.get(at).ok_or(result::UNAVAILABLE)?;
            let info = &c.states[state as usize];
            if state == AIR || info.liquid.is_some() {
                continue;
            }
            if self.space.signals.reserved(at) {
                return Err(result::UNAVAILABLE);
            }
            if let Some(step) = c.covered_step(state) {
                let root = at + step;
                if !allowed(root) {
                    return Err(result::UNAVAILABLE);
                }
                pending.push(root);
                continue;
            }
            if (at != grip && !info.solid()) || (!self.movable(c, state) && !(helm && at == p)) {
                continue;
            }
            cells.push(at);
            for &(offset, _) in &info.cells {
                let extra = at + offset;
                if !allowed(extra) || self.space.signals.reserved(extra) || self.get(extra).is_none() {
                    return Err(result::UNAVAILABLE);
                }
                cells.push(extra);
                seen.insert(extra);
                pending.extend(DIRS.map(|d| extra + d));
            }
            if cells.len() > BODY_LIMIT {
                return Err(result::TOO_LARGE);
            }
            pending.extend(DIRS.map(|d| at + d));
        }
        if cells.is_empty() {
            return Err(result::UNAVAILABLE);
        }
        cells.sort_unstable_by_key(|p| p.to_array());
        cells.dedup();
        Ok(cells)
    }

    pub(super) fn connected_lifted(&self, c: &Content, at: IVec3, center: IVec3, cells: Vec<IVec3>, bounds: Option<Bounds>) -> Vec<Lifted> {
        let helm = at == center;
        let (riders, _) = self.who_rides(c, &cells, (!helm).then_some(at), IVec3::ZERO);
        let mut out: Vec<_> = cells
            .into_iter()
            .map(|p| (p, None))
            .chain(
                riders
                    .into_iter()
                    .filter(|(p, _)| bounds.is_none_or(|b| b.contains(*p)))
                    .map(|(p, state)| (p, Some(state))),
            )
            .map(|(p, part)| Lifted { from: p, local: p - center, to: p, part })
            .collect();
        out.sort_unstable_by_key(|l| (l.from.to_array(), l.part));
        out
    }

    pub(crate) fn assembly_select(&mut self, c: &'static Content, player: u64, at: Option<IVec3>, tool: bool) -> u32 {
        let Some(e) = self.entities.get_mut(player).filter(|e| e.player) else { return result::UNHANDLED };
        if at.is_none() {
            e.assembly_edit = None;
            self.draw_assembly(player, None, &[]);
            return result::CLEARED;
        }
        let at = at.unwrap();
        let info = self.get(at).map(|s| c.states[s as usize].signal);
        let controller = info.filter(|s| s.kind == abi::signal_kind::BODY_MOUNT);
        let helm = controller.is_some_and(|s| s.flags as u32 & abi::signal_flags::HELM != 0);
        if self.frame != 0 {
            if !helm
                || at != IVec3::ZERO
                || !self
                    .entities
                    .get(self.frame)
                    .is_some_and(|e| e.ship.is_some())
            {
                return result::UNHANDLED;
            }
            if tool {
                self.space.stopping = true;
                return result::DOCKING;
            }
            return if self.mount(c, player, self.frame, Some(0)) { result::SEATED } else { result::UNAVAILABLE };
        }
        if !helm && !tool {
            return result::UNHANDLED;
        }
        let mut edit = self
            .entities
            .get_mut(player)
            .and_then(|e| e.assembly_edit.take())
            .filter(|d| d.dimension == self.id)
            .unwrap_or_else(|| Box::new(Edit { dimension: self.id, ..Edit::default() }));
        let answer = if let Some(info) = controller {
            let bounds = edit.bounds.or_else(|| self.assembly_bounds(c, at));
            let center = if helm { at } else { at + DIRS[info.front as usize] };
            if !helm && self.has_mount(at) {
                result::UNAVAILABLE
            } else {
                match self.connected_assembly(c, at, bounds) {
                    Err(code) => code,
                    Ok(cells) => {
                        let lifted = self.connected_lifted(c, at, center, cells, bounds);
                        if lifted.len() > BODY_LIMIT {
                            result::TOO_LARGE
                        } else if edit
                            .preview
                            .as_ref()
                            .is_some_and(|p| p.controller == at && p.expires > self.time && p.cells == lifted)
                        {
                            edit.preview = None;
                            self.draw_assembly(player, None, &[]);
                            if helm {
                                self.launch_ship(c, at, info.front, lifted);
                                result::CONFIRMED
                            } else {
                                let fit = lifted
                                    .iter()
                                    .fold(Bounds::new(center, center), |bounds, l| {
                                        Bounds::new(bounds.lo.min(l.from), bounds.hi.max(l.from))
                                    });
                                if self.save_assembly_bounds(c, at, bounds.unwrap_or(fit)) {
                                    self.space.signals.recheck_device(at);
                                    result::CONFIRMED
                                } else {
                                    result::UNAVAILABLE
                                }
                            }
                        } else {
                            self.draw_assembly(player, bounds, &lifted);
                            edit.preview = Some(Preview { controller: at, cells: lifted, expires: self.time + 160 });
                            result::PREVIEW
                        }
                    }
                }
            }
        } else {
            edit.preview = None;
            if let Some(first) = edit.corner.take() {
                let bounds = Bounds::new(first, at);
                edit.bounds = Some(bounds);
                self.draw_assembly(player, Some(bounds), &[]);
                result::BOUNDS
            } else {
                edit.bounds = None;
                edit.corner = Some(at);
                self.draw_assembly(player, Some(Bounds::new(at, at)), &[]);
                result::FIRST_CORNER
            }
        };
        if let Some(e) = self.entities.get_mut(player) {
            e.assembly_edit = Some(edit);
        }
        answer
    }

    fn draw_assembly(&mut self, player: u64, bounds: Option<Bounds>, cells: &[Lifted]) {
        let mut draw = |lo: IVec3, hi: IVec3, color| {
            self.gizmos.push((
                abi::Who::Player(player),
                crate::net::GizmoShape::shape(
                    crate::net::GizmoShape::ASSEMBLY,
                    abi::gizmo_kind::BOX as u8,
                    lo.as_dvec3(),
                    (hi + IVec3::ONE).as_dvec3(),
                    color,
                    if color == 0 { 0 } else { 160 },
                ),
            ))
        };
        if let Some(b) = bounds {
            draw(b.lo, b.hi, 0xFFCE5CFF);
        }
        let occupied: FxHashSet<_> = cells.iter().map(|l| l.from).collect();
        for &at in &occupied {
            if DIRS.iter().any(|d| !occupied.contains(&(at + *d))) {
                draw(at, at, 0x4ADFBECC);
            }
        }
        if bounds.is_none() && occupied.is_empty() {
            draw(IVec3::ZERO, IVec3::ZERO, 0);
        }
    }

    fn launch_ship(&mut self, c: &'static Content, center: IVec3, face: u8, cells: Vec<Lifted>) {
        let (id, _) = self.lift(c, c.game.ship_kind, center.as_dvec3() + Transform::FEET, cells, &[], Liveness::Assembly);
        self.float_ship(id, face);
    }
}

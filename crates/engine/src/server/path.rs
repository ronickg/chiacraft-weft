//! Paths for walking mobs: A* over block cells with vanilla-like rules (room for the whole body,
//! as wide and as tall as it is, a step up of one block by jumping, drops of up to three, no hot
//! liquids), and the navigation that walks an entity along one.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use glam::{DVec3, IVec3};
use rustc_hash::FxHashMap;

use crate::content::Content;

/// Where an entity is walking to.
pub struct Nav {
    pub target: DVec3,
    pub speed: f32,
    pub path: Vec<IVec3>,
    pub next: usize,
    /// Earliest age the path may be looked for again.
    pub repath: u64,
    /// The target the path was found for; `None` looks again at `repath`.
    pub found_for: Option<DVec3>,
    /// Ticks without getting anywhere.
    pub stuck: u32,
    /// The body it walks on, whose coordinates the target and the path are in.
    pub frame: Option<u64>,
}

impl Nav {
    /// Worth a new search: never searched, the target moved off, or the walk got stuck. A path
    /// to a target that stays put is kept (searching every mob's path every second costs most of
    /// a tick with thousands of mobs).
    pub fn stale(&self, age: u64) -> bool {
        age >= self.repath
            && self
                .found_for
                .is_none_or(|f| (f - self.target).length_squared() > 4.0 || self.stuck > 10)
    }
}

const MAX_NODES: usize = 600;

type Scratch = (FxHashMap<IVec3, (IVec3, f32)>, BinaryHeap<Reverse<(u32, i32, i32, i32)>>);

thread_local! {
    /// The search's map and queue, kept between searches (thousands of mobs walk about).
    static SCRATCH: std::cell::RefCell<Scratch> = std::cell::RefCell::new((FxHashMap::default(), BinaryHeap::new()));
}
const MAX_DROP: i32 = 3;

/// The cells round a feet cell that a body `width` blocks wide, standing on that cell's middle,
/// overlaps: one for any body up to a block wide.
pub fn footprint(width: f32) -> std::ops::RangeInclusive<i32> {
    let half = (width.max(0.0) / 2.0 - 0.01).max(0.0);
    (0.5 - half).floor() as i32..=((0.5 + half).ceil() as i32 - 1)
}

/// Cells a body `height` blocks tall and `width` wide can stand in, from the feet cell `from`
/// towards `to`: the whole way, or as far as it gets (the reachable cell closest to the target)
/// when `to` is out of reach or more than `MAX_NODES` cells away. A wide body needs room across
/// its whole footprint, so it isn't led between tree trunks it can't pass (the animals mod's big
/// Cretaceous animals, 2 to 4.5 blocks wide, jammed in its forests on one-cell paths).
pub fn find(c: &Content, block: &impl crate::physics::Cells, from: IVec3, to: IVec3, height: i32, width: f32) -> Vec<IVec3> {
    // Judged by the collision boxes walking bumps into, like the step itself.
    let top = |s: u32| crate::physics::collision_top(c, s);
    let open = |p: IVec3| {
        let free = |s: u32| {
            let st = &c.states[s as usize];
            top(s).is_none() && st.liquid.is_none_or(|l| !c.liquids[l as usize].burns())
        };
        block.main(p).is_some_and(free) && block.extras(p).iter().all(|&s| free(s))
    };
    let cells = footprint(width);
    let wide = cells.start() != cells.end();
    let column = |p: IVec3| (0..height).all(|k| open(p + IVec3::Y * k));
    // A wide body's room is asked of each cell several times: worked out once per search.
    let known: std::cell::RefCell<FxHashMap<IVec3, bool>> = Default::default();
    let room = |p: IVec3| {
        if !wide {
            return column(p);
        }
        if let Some(&r) = known.borrow().get(&p) {
            return r;
        }
        let r = cells
            .clone()
            .all(|dx| cells.clone().all(|dz| column(p + IVec3::new(dx, 0, dz))));
        known.borrow_mut().insert(p, r);
        r
    };
    let burns = |s: u32| {
        c.states[s as usize]
            .liquid
            .is_some_and(|l| c.liquids[l as usize].burns())
    };
    let ground = |p: IVec3| {
        let below = p - IVec3::Y;
        let holds = |s: u32| c.states[s as usize].liquid.is_some() || top(s).is_some_and(|t| t >= 0.5);
        block.main(below).is_some_and(|s| {
            let extras = block.extras(below);
            !burns(s) && !extras.iter().any(|&e| burns(e)) && (holds(s) || extras.iter().any(|&e| holds(e)))
        })
    };
    let standable = |p: IVec3| room(p) && ground(p);
    let wet = |p: IVec3| block.liquid(c, p).is_some();
    let h = |p: IVec3| (p - to).as_vec3().length();

    let (mut came, mut queue) = SCRATCH.with_borrow_mut(std::mem::take);
    came.clear();
    queue.clear();
    came.insert(from, (from, 0.0));
    queue.push(Reverse(((h(from) * 1000.0) as u32, from.x, from.y, from.z)));
    let mut best = (h(from), from);
    let mut visited = 0;
    while let Some(Reverse((_, x, y, z))) = queue.pop() {
        let p = IVec3::new(x, y, z);
        let g = came[&p].1;
        let d = h(p);
        if d < best.0 {
            best = (d, p);
        }
        if p == to || (d < 1.5 && (p.y - to.y).abs() <= 1) {
            best = (0.0, p);
            break;
        }
        visited += 1;
        if visited > MAX_NODES {
            break;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let side = IVec3::new(dx, 0, dz);
            // Diagonals only where both straight neighbors leave room (no corner cutting).
            if dx != 0 && dz != 0 && !(room(p + IVec3::new(dx, 0, 0)) && room(p + IVec3::new(0, 0, dz))) {
                continue;
            }
            let base = if dx != 0 && dz != 0 { 1.414 } else { 1.0 };
            let q = p + side;
            let mut next = None;
            if standable(q) {
                next = Some((q, base));
            } else if !room(q) {
                // A step up: jump, with room above the head first.
                let up = q + IVec3::Y;
                if standable(up) && open(p + IVec3::Y * height) {
                    next = Some((up, base + 1.0));
                }
            } else {
                // Open but nothing to stand on: drop down to the first ground.
                for k in 1..=MAX_DROP {
                    let down = q - IVec3::Y * k;
                    if !open(down) {
                        break;
                    }
                    if ground(down) {
                        next = Some((down, base + 0.5 * k as f32));
                        break;
                    }
                }
            }
            if let Some((n, cost)) = next {
                let g2 = g + cost + if wet(n) { 2.0 } else { 0.0 };
                if came.get(&n).is_none_or(|&(_, old)| g2 < old) {
                    came.insert(n, (p, g2));
                    queue.push(Reverse((((g2 + h(n)) * 1000.0) as u32, n.x, n.y, n.z)));
                }
            }
        }
    }
    let mut path = vec![best.1];
    while let Some(&(prev, _)) = came.get(path.last().unwrap()) {
        if prev == *path.last().unwrap() {
            break;
        }
        path.push(prev);
    }
    path.reverse();
    SCRATCH.with_borrow_mut(|s| *s = (came, queue));
    path
}

/// Steering for this tick: walk direction, jump, and whether the target is reached.
pub fn steer(nav: &mut Nav, pos: DVec3, blocked: bool) -> Option<([f32; 2], bool)> {
    let feet = pos.floor().as_ivec3();
    while let Some(&w) = nav.path.get(nav.next) {
        let center = w.as_dvec3() + DVec3::new(0.5, 0.0, 0.5);
        let flat = (center - pos).with_y(0.0);
        if flat.length() < 0.35 && (w.y - feet.y).abs() <= 1 {
            nav.next += 1;
            continue;
        }
        let dir = flat.normalize_or_zero();
        return Some(([dir.x as f32, dir.z as f32], w.y > feet.y || blocked));
    }
    None
}

#[cfg(test)]
mod tests {
    use weft_abi::{self as abi, BlockModel, ModId, RenderLayer, block_flags as bf};

    use super::*;
    use crate::content::{AIR, ModMeta, RawBlock, RawRegistrar, RawSignal, RawState};

    /// Stone, in force on this thread (each test has its own).
    fn stone() -> (&'static Content, u32) {
        let mods =
            vec![ModMeta { id: "t".into(), version: "1".into(), deps: Vec::new(), order: 0, assets: "/nonexistent".into(), package: None }];
        let mut r = RawRegistrar::new(ModId(0));
        let state = RawState {
            flags: bf::SOLID | bf::OPAQUE,
            filter: [0; 3],
            opacity: 15,
            layer: RenderLayer::Opaque,
            model: BlockModel::Cube,
            textures: std::array::from_fn(|_| "t:block/x".to_string()),
            turns: [0; 6],
            boxes: Vec::new(),
            hitbox: Vec::new(),
            outline: Vec::new(),
            liquid_walls: 0,
            signal: RawSignal {
                kind: abi::SignalDesc::DEFAULT.kind,
                attach: 6,
                front: 6,
                delay: 0,
                flags: 0,
                inputs: 0,
                head: String::new(),
            },
            components: Vec::new(),
        };
        r.blocks.push(RawBlock {
            name: "t:stone".into(),
            cell: 0,
            states: vec![state],
            props: Vec::new(),
            components: Vec::new(),
            cb: Default::default(),
            replace: false,
            own_states: true,
            retexture: Vec::new(),
            opens: String::new(),
            extra_cells: Vec::new(),
            derive: None,
            shares_cell: Vec::new(),
            model_file: String::new(),
            model_textures: Vec::new(),
        });
        let c = crate::content::freeze(mods, vec![r]).unwrap_or_else(|e| panic!("{e:?}"));
        let c: &'static Content = Box::leak(Box::new(c));
        crate::content::for_thread(c);
        (c, c.parse_state("t:stone").unwrap())
    }

    #[test]
    fn a_footprint_covers_the_cells_a_body_overlaps() {
        assert_eq!(footprint(0.6), 0..=0);
        assert_eq!(footprint(1.0), 0..=0);
        assert_eq!(footprint(1.2), -1..=1);
        assert_eq!(footprint(2.0), -1..=1);
        assert_eq!(footprint(3.8), -2..=2);
    }

    /// A wall with a one-block gap: a body a block wide walks through it, one three blocks wide
    /// goes round one of the wall's ends and never into the gap.
    #[test]
    fn a_wide_body_goes_round_a_gap_it_cannot_pass() {
        let (c, stone) = stone();
        // Floor below y 0; a wall along z = 0 from x -6 to 6, three high, open at x 0.
        let block = |p: IVec3| -> Option<u32> {
            Some(if p.y < 0 || (p.z == 0 && p.y < 3 && (-6..=6).contains(&p.x) && p.x != 0) { stone } else { AIR })
        };
        let (from, to) = (IVec3::new(0, 0, -4), IVec3::new(0, 0, 4));
        let thin = find(c, &block, from, to, 2, 0.6);
        assert!(thin.iter().any(|p| p.x == 0 && p.z == 0), "a thin body goes through the gap: {thin:?}");
        assert!(thin.last().is_some_and(|p| p.z > 0), "a thin body gets past the wall: {thin:?}");
        let wide = find(c, &block, from, to, 2, 3.0);
        assert!(!wide.iter().any(|p| p.z == 0 && p.x.abs() <= 7), "a wide body was led into the wall: {wide:?}");
        assert!(wide.last().is_some_and(|p| p.z > 0), "a wide body goes round the wall: {wide:?}");
    }
}

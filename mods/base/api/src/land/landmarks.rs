//! Landmarks built from blocks (Ronald, 11-04: "all four"; the third proposal sheet): natural
//! arches and sea-cliff waterfalls. A decoration builds them, each cube its own part, from sites
//! worked out once per cell and kept, so overhangs need no engine hook; distant terrain doesn't
//! show them.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, Fields, div_floor, q12};

use super::{
    ARCH_CELL, ARCH_CHANCE, ARCH_DEPTH, ARCH_INNER, ARCH_THICK, FALL_CELL, FALL_CHANCE, FALL_CLIFF, RockSet, height, rock_set, salt,
};
use crate::SEA;

/// A natural arch: a half ring standing on two legs. Its middle (blocks), whether it spans
/// along x (else along z), its inner and outer radius and depth (blocks), the height its ring
/// starts from (the legs go down to the ground under them), and its rock.
#[derive(Clone, Copy, Debug)]
pub struct Arch {
    pub x: i32,
    pub z: i32,
    pub along_x: bool,
    pub inner: i32,
    pub outer: i32,
    pub depth: i32,
    pub base: i32,
    pub rocks: RockSet,
}

impl Arch {
    /// Whether the arch holds block (`x`, `y`, `z`), the ground under it being `ground` high.
    pub fn holds(&self, x: i32, y: i32, z: i32, ground: i32) -> bool {
        let (s, t) = if self.along_x { (x - self.x, z - self.z) } else { (z - self.z, x - self.x) };
        if t.abs() > self.depth / 2 || s.abs() > self.outer || y <= ground {
            return false;
        }
        let v = y - self.base;
        if v < 0 {
            // A leg, down to the ground.
            return s.abs() >= self.inner;
        }
        let d = s * s + v * v;
        d >= self.inner * self.inner && d <= self.outer * self.outer
    }

    /// The box it fills (blocks): x, z from and to, and its top.
    pub fn reach(&self) -> (i32, i32, i32, i32, i32) {
        let (a, b) = if self.along_x { (self.outer, self.depth / 2) } else { (self.depth / 2, self.outer) };
        (self.x - a, self.x + a, self.z - b, self.z + b, self.base + self.outer)
    }
}

/// A waterfall: a spring in a sea cliff's face at `spring` (blocks), spilling into the block in
/// front of it (`front`, x and z, the same height), which falls to the sea.
#[derive(Clone, Copy, Debug)]
pub struct Waterfall {
    pub spring: [i32; 3],
    pub front: [i32; 2],
}

/// Sites by (seed, cell), kept.
type Sites<T> = RefCell<HashMap<(u64, i32, i32), Option<T>>>;

thread_local! {
    static ARCHES: Sites<Arch> = RefCell::new(HashMap::new());
    static FALLS: Sites<Waterfall> = RefCell::new(HashMap::new());
}

fn keep<T: Copy>(map: &'static std::thread::LocalKey<Sites<T>>, key: (u64, i32, i32), make: impl FnOnce() -> Option<T>) -> Option<T> {
    if let Some(hit) = map.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let v = make();
    map.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4096 {
            c.clear();
        }
        c.insert(key, v);
    });
    v
}

/// Cell (`cx`, `cz`)'s arch, if it has one: in the strata land on level ground, or standing in
/// the shallow sea off a cliff coast.
fn arch(f: &Fields, cx: i32, cz: i32) -> Option<Arch> {
    keep(&ARCHES, (f.seed, cx, cz), || {
        let hash = fixed::hash(fixed::seed32(f.seed, salt::ARCHES), cx, 0, cz);
        if (fixed::fmix(hash) & 4095) >= q12(ARCH_CHANCE) as u32 {
            return None;
        }
        let h2 = fixed::fmix(hash ^ 0xA2C4);
        let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
        let (inner, thick, depth) = (pick(h2, ARCH_INNER), pick(h2 >> 6, ARCH_THICK), pick(h2 >> 12, ARCH_DEPTH));
        let outer = inner + thick;
        let margin = outer + 2;
        let span = (ARCH_CELL - 2 * margin) as u32;
        let x = cx * ARCH_CELL + margin + (hash % span) as i32;
        let z = cz * ARCH_CELL + margin + ((hash >> 12) % span) as i32;
        let along_x = h2 >> 20 & 1 == 0;
        let ground = |x: i32, z: i32| height(f, x * 16, z * 16) >> 4;
        let leg = (inner + outer) / 2;
        let (l1, l2) = if along_x { (ground(x - leg, z), ground(x + leg, z)) } else { (ground(x, z - leg), ground(x, z + leg)) };
        let here = ground(x, z);
        if (SEA - 5..SEA).contains(&here) && l1 < SEA && l2 < SEA {
            // A sea arch: in shallow water with cliffs (high land) near.
            let cliff = (0..16).find_map(|k| {
                let a = k as f64 / 16.0 * std::f64::consts::TAU;
                (4..=16).step_by(2).find_map(|d| {
                    let (px, pz) = (x + (a.cos() * d as f64) as i32, z + (a.sin() * d as f64) as i32);
                    (ground(px, pz) >= SEA + 6).then_some((px, pz))
                })
            })?;
            let rocks = rock_set(f, cliff.0 * 16, cliff.1 * 16, ground(cliff.0, cliff.1), false);
            return Some(Arch { x, z, along_x, inner, outer, depth, base: SEA - 1, rocks });
        }
        let rocks = rock_set(f, x * 16, z * 16, here, false);
        (rocks == RockSet::Strata && here > SEA + 2 && (l1 - l2).abs() <= 4 && (here - l1.max(l2)).abs() <= 6).then_some(Arch {
            x,
            z,
            along_x,
            inner,
            outer,
            depth,
            base: l1.max(l2) - 1,
            rocks,
        })
    })
}

/// Cell (`cx`, `cz`)'s waterfall, if it has one: the first sea cliff `FALL_CLIFF` or more over
/// the sea found in the cell (every 8 blocks, then the block itself), where a spring two under
/// the top opens only to its front, over water.
fn waterfall(f: &Fields, cx: i32, cz: i32) -> Option<Waterfall> {
    keep(&FALLS, (f.seed, cx, cz), || {
        let hash = fixed::hash(fixed::seed32(f.seed, salt::FALLS), cx, 0, cz);
        if (fixed::fmix(hash) & 4095) >= q12(FALL_CHANCE) as u32 {
            return None;
        }
        let ground = |x: i32, z: i32| height(f, x * 16, z * 16) >> 4;
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (x0, z0) = (cx * FALL_CELL, cz * FALL_CELL);
        // Start the search somewhere in the cell, so the falls don't line up.
        let off = (hash % (FALL_CELL / 8) as u32) as i32;
        for j in 0..FALL_CELL / 8 {
            for i in 0..FALL_CELL / 8 {
                let (sx, sz) = (x0 + ((i + off) % (FALL_CELL / 8)) * 8 + 4, z0 + j * 8 + 4);
                if ground(sx, sz) < SEA + FALL_CLIFF
                    || !dirs
                        .iter()
                        .any(|(dx, dz)| ground(sx + dx * 4, sz + dz * 4) < SEA)
                {
                    continue;
                }
                for dz in -4..=4 {
                    for dx in -4..=4 {
                        let (x, z) = (sx + dx, sz + dz);
                        let h = ground(x, z);
                        if h < SEA + FALL_CLIFF {
                            continue;
                        }
                        let y = h - 2;
                        for (ex, ez) in dirs {
                            let (fx, fz) = (x + ex, z + ez);
                            // Over water, the sides of the spring solid at its height.
                            if ground(fx, fz) < SEA && ground(x + ez, z + ex) >= y && ground(x - ez, z - ex) >= y {
                                return Some(Waterfall { spring: [x, y, z], front: [fx, fz] });
                            }
                        }
                    }
                }
            }
        }
        None
    })
}

/// The arches whose box comes within `r` blocks of (`x`, `z`) (blocks).
pub fn arches_near(f: &Fields, x: i32, z: i32, r: i32) -> Vec<Arch> {
    let mut out = Vec::new();
    for cz in div_floor(z - r, ARCH_CELL).0..=div_floor(z + r, ARCH_CELL).0 {
        for cx in div_floor(x - r, ARCH_CELL).0..=div_floor(x + r, ARCH_CELL).0 {
            if let Some(a) = arch(f, cx, cz) {
                let (x0, x1, z0, z1, _) = a.reach();
                if x1 >= x - r && x0 <= x + r && z1 >= z - r && z0 <= z + r {
                    out.push(a);
                }
            }
        }
    }
    out
}

/// The waterfalls within `r` blocks of (`x`, `z`) (blocks).
pub fn waterfalls_near(f: &Fields, x: i32, z: i32, r: i32) -> Vec<Waterfall> {
    let mut out = Vec::new();
    for cz in div_floor(z - r, FALL_CELL).0..=div_floor(z + r, FALL_CELL).0 {
        for cx in div_floor(x - r, FALL_CELL).0..=div_floor(x + r, FALL_CELL).0 {
            if let Some(w) = waterfall(f, cx, cz)
                && (w.spring[0] - x).abs() <= r + 1
                && (w.spring[2] - z).abs() <= r + 1
            {
                out.push(w);
            }
        }
    }
    out
}

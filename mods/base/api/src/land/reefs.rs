//! Warm shallow seas (Ronald, 11-04: "all three", Terrain's fourth sheet): off warm coasts the
//! sea floor rises into a lagoon along the shore and a reef farther out, a few blocks under the
//! surface. How far a column is from the shore is read from a grid of the land's base ground every
//! `REEF_STEP` blocks, worked out once per `REEF_REGION` square and kept.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use weft_sdk::noise::fixed::{div_floor, q4, q12, ramp};

use super::{
    Here, REEF_COAST, REEF_LAGOON, REEF_OUT, REEF_PATCH_SIZE, REEF_PATCHES, REEF_REGION, REEF_RUBBLE, REEF_SHORE, REEF_SLOPE, REEF_STEP,
    REEF_WARM, REEF_WARM_RULE, base_ground, salt,
};
use crate::SEA;

/// The shore distance grid of a region: nodes `REEF_STEP` apart over the region and `REEF_SLOPE`
/// round it, each its distance to the nearest land node (in nodes, capped).
struct Shore {
    nodes: i32,
    dist: Vec<u8>,
}

/// The regions' grids by (seed, region), kept.
type Shores = RefCell<HashMap<(u64, i32, i32), Option<Arc<Shore>>>>;

thread_local! {
    static SHORES: Shores = RefCell::new(HashMap::new());
}

/// Whether the sea at (`x`, `z`) (Q4) is warm enough for a reef: 0 to `ONE`.
pub(super) fn warm(h: &Here) -> i32 {
    ramp(h.temperature(), q12(REEF_WARM.0), q12(REEF_WARM.1))
}

fn shore(h: &Here, rx: i32, rz: i32) -> Option<Arc<Shore>> {
    let key = (h.f.seed, rx, rz);
    if let Some(hit) = SHORES.with(|c| c.borrow().get(&key).cloned()) {
        return hit;
    }
    let margin = REEF_SLOPE / REEF_STEP + 1;
    let nodes = REEF_REGION / REEF_STEP + 2 * margin;
    let (x0, z0) = (rx * REEF_REGION - margin * REEF_STEP, rz * REEF_REGION - margin * REEF_STEP);
    // Only regions with warm sea at their middle work the grid out.
    let mid = Here::new(h.f, (rx * REEF_REGION + REEF_REGION / 2) * 16, (rz * REEF_REGION + REEF_REGION / 2) * 16);
    let site = (warm(&mid) > 0).then(|| {
        let cap = (REEF_SLOPE / REEF_STEP + 2) as u8;
        let mut dist = vec![cap; (nodes * nodes) as usize];
        let mut queue = std::collections::VecDeque::new();
        for j in 0..nodes {
            for i in 0..nodes {
                let here = Here::new(h.f, (x0 + i * REEF_STEP) * 16, (z0 + j * REEF_STEP) * 16);
                if base_ground(&here) >= q4(SEA as f64) {
                    dist[(j * nodes + i) as usize] = 0;
                    queue.push_back((i, j));
                }
            }
        }
        while let Some((i, j)) = queue.pop_front() {
            let d = dist[(j * nodes + i) as usize];
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (a, b) = (i + di, j + dj);
                if (0..nodes).contains(&a) && (0..nodes).contains(&b) && dist[(b * nodes + a) as usize] > d + 1 {
                    dist[(b * nodes + a) as usize] = d + 1;
                    queue.push_back((a, b));
                }
            }
        }
        Arc::new(Shore { nodes, dist })
    });
    SHORES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 1024 {
            c.clear();
        }
        c.insert(key, site.clone());
    });
    site
}

/// The distance from column (`bx`, `bz`) (blocks) to the shore (blocks), if its sea is warm
/// and the shore within `REEF_SLOPE`.
pub(super) fn shore_distance(h: &Here) -> Option<i32> {
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let (rx, rz) = (div_floor(bx, REEF_REGION).0, div_floor(bz, REEF_REGION).0);
    let s = shore(h, rx, rz)?;
    let margin = REEF_SLOPE / REEF_STEP + 1;
    let i = div_floor(bx - rx * REEF_REGION, REEF_STEP).0 + margin;
    let j = div_floor(bz - rz * REEF_REGION, REEF_STEP).0 + margin;
    let d = s.dist[(j * s.nodes + i) as usize] as i32 * REEF_STEP;
    (d <= REEF_SLOPE).then_some(d)
}

/// What the warm seas add to sea floor `base` (Q4) at a column: the lagoon's floor
/// (`REEF_LAGOON`, under and over), the reef's top `REEF_OUT` blocks out (2 under the surface, in
/// patches over `REEF_PATCHES`), and a slope down to the deep beyond. Nothing on land, in cold
/// seas or near no shore.
pub(super) fn add(h: &Here, base: i32) -> i32 {
    let sea = q4(SEA as f64);
    if base >= sea || warm(h) < q12(0.5) {
        return 0;
    }
    let Some(d) = shore_distance(h) else { return 0 };
    let ground = base >> 4;
    let target = if d < REEF_SHORE {
        return 0;
    } else if d < REEF_OUT.0 {
        ground.clamp(SEA - REEF_LAGOON.1, SEA - REEF_LAGOON.0)
    } else if d <= REEF_OUT.1 {
        let patch = h.f.fbm(salt::REEFS, h.x, h.z, q4(REEF_PATCH_SIZE), 3);
        if patch > q12(REEF_PATCHES) { SEA - 2 } else { ground.max(SEA - REEF_LAGOON.1 - 1) }
    } else {
        ground.max(SEA - REEF_LAGOON.1 - 4)
    };
    q4(target as f64) - base
}

/// Which part of a warm sea column (`x`, `z`) (Q4) is, its ground being `height` (blocks): the
/// reef's top, the lagoon's floor, or neither.
pub(super) fn part(h: &Here, height: i32) -> Option<bool> {
    // The engine's rules see height, climate and continent only: warm water at those depths near
    // the coasts (the continent under `REEF_COAST`), not the inland lakes.
    if h.temperature() < q12(REEF_WARM_RULE) || h.continent() >= q12(REEF_COAST) {
        return None;
    }
    match height {
        y if y == SEA - 2 => Some(true),
        y if (SEA - REEF_LAGOON.1..=SEA - REEF_LAGOON.0).contains(&y) => Some(false),
        _ => None,
    }
}

/// Whether the reef itself stands at a column (its top made by `add`, not natural shallows at
/// the same depth): warm sea, `REEF_OUT` blocks from the shore, in a patch.
pub(super) fn top(h: &Here) -> bool {
    if warm(h) < q12(0.5) || base_ground(h) >= q4(SEA as f64) {
        return false;
    }
    shore_distance(h).is_some_and(|d| (REEF_OUT.0..=REEF_OUT.1).contains(&d))
        && h.f.fbm(salt::REEFS, h.x, h.z, q4(REEF_PATCH_SIZE), 3) > q12(REEF_PATCHES)
}

/// Whether a column is the reef's rubble (broken coral on its sand): in the reef's gaps, and in a
/// band `REEF_RUBBLE` blocks wide on its landward side.
pub(super) fn rubble(h: &Here) -> bool {
    if warm(h) < q12(0.5) || base_ground(h) >= q4(SEA as f64) {
        return false;
    }
    shore_distance(h).is_some_and(|d| {
        (REEF_OUT.0 - REEF_RUBBLE..REEF_OUT.0).contains(&d)
            || ((REEF_OUT.0..=REEF_OUT.1).contains(&d) && h.f.fbm(salt::REEFS, h.x, h.z, q4(REEF_PATCH_SIZE), 3) <= q12(REEF_PATCHES))
    })
}

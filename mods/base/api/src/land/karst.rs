//! Karst towers (`reports/terrain-terralith-tectonic.md`, step 5; Ronald, 11-04): steep limestone
//! pillars in the warm, wet land and its shallow water. One possible tower per `KARST_CELL`
//! square, kept by the karst area's strength at its middle; the sites are worked out once per
//! cell and kept, since every column near one asks for its three by three cells.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, ONE, div_floor, q4, q12, ramp};

use super::{
    Here, KARST_AREA, KARST_CELL, KARST_CHANCE, KARST_CLUSTER, KARST_CLUSTER_SIZE, KARST_HEIGHT, KARST_LONE, KARST_RADIUS,
    KARST_WIDE_HEIGHT, KARST_WIDE_ONE_IN, KARST_WIDE_RADIUS, salt,
};
use crate::SEA;

/// A tower: its middle (blocks), radius and height (blocks).
#[derive(Clone, Copy)]
struct Tower {
    x: i32,
    z: i32,
    radius: i32,
    height: i32,
}

/// Towers stand only in water this shallow at most (blocks): not out of the deep sea.
const KARST_DEEPEST: i32 = 8;

thread_local! {
    static SITES: RefCell<HashMap<(u64, i32, i32), Option<Tower>>> = RefCell::new(HashMap::new());
}

/// Cell (`cx`, `cz`)'s tower, if it has one.
fn tower(h: &Here, cx: i32, cz: i32) -> Option<Tower> {
    let key = (h.f.seed, cx, cz);
    if let Some(hit) = SITES.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let hash = fixed::hash(fixed::seed32(h.f.seed, salt::KARST), cx, 0, cz);
    // Anywhere in the cell: a tower reaches at most `reach` (under a cell) past its middle, and
    // the columns look in their three by three cells.
    let span = KARST_CELL as u32;
    let x = cx * KARST_CELL + (hash % span) as i32;
    let z = cz * KARST_CELL + ((hash >> 12) % span) as i32;
    let (qx, qz) = (x * 16, z * 16);
    let cluster = ramp(h.f.fbm(salt::KARST_CLUSTER, qx, qz, q4(KARST_CLUSTER_SIZE), 2), q12(KARST_CLUSTER.0), q12(KARST_CLUSTER.1));
    let keep = q12(KARST_LONE) + (((q12(KARST_CHANCE) - q12(KARST_LONE)) * cluster) >> 12);
    let chance = (keep * KARST_AREA.value(h.f, qx, qz)) >> 12;
    let site = ((fixed::fmix(hash) & 4095) < chance as u32).then(|| {
        let h2 = fixed::fmix(hash ^ 0x9E37);
        let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
        let wide = (h2 >> 20).is_multiple_of(KARST_WIDE_ONE_IN);
        let (r, ht) = if wide { (KARST_WIDE_RADIUS, KARST_WIDE_HEIGHT) } else { (KARST_RADIUS, KARST_HEIGHT) };
        Tower { x, z, radius: pick(h2, r), height: pick(h2 >> 8, ht) }
    });
    SITES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 16384 {
            c.clear();
        }
        c.insert(key, site);
    });
    site
}

/// What the karst towers add to ground `base` (Q4) at a column inside the karst area: steep
/// sides over the outer quarter, a gently rounded top `height` over the water (or the ground, if
/// higher), the outline wobbled by a noise.
pub(super) fn add(h: &Here, base: i32) -> i32 {
    let sea = q4(SEA as f64);
    if base < sea - q4(KARST_DEEPEST as f64) || KARST_AREA.at(h) == 0 {
        return 0;
    }
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let (cx, cz) = (div_floor(bx, KARST_CELL).0, div_floor(bz, KARST_CELL).0);
    let mut add = 0;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let Some(t) = tower(h, cx + dx, cz + dz) else { continue };
            let reach = t.radius + t.radius / 2 + 1;
            if (bx - t.x).abs() > reach || (bz - t.z).abs() > reach {
                continue;
            }
            let (ex, ez) = ((bx - t.x) as i64, (bz - t.z) as i64);
            let d = ((ex * ex + ez * ez) as u64 * 256).isqrt() as i32; // Q4
            let wobble = h.f.fbm(salt::KARST_WOBBLE, h.x, h.z, q4(9.0), 2);
            let r = q4(t.radius as f64);
            let rr = r + ((r * wobble) >> 12) / 2;
            if d >= rr {
                continue;
            }
            let u = d * ONE / rr.max(1);
            let side = ONE - ramp(u, q12(0.75), ONE);
            let dome = ONE - ((((u * u) >> 12) * q12(0.12)) >> 12);
            let top = base.max(sea) + ((q4(t.height as f64) * ((side * dome) >> 12)) >> 12);
            add = add.max(top - base);
        }
    }
    add
}

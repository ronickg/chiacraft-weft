//! Karst towers (`reports/terrain-terralith-tectonic.md`, step 5; Ronald, 11-04): steep limestone
//! pillars in the warm, wet land and its shallow water. One possible tower per `KARST_CELL`
//! square, kept by the karst area's strength at its middle; the sites are worked out once per
//! cell and kept, since every column near one asks for its three by three cells.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, ONE, div_floor, q4, q12, ramp};

use super::{
    Here, KARST_AREA, KARST_CELL, KARST_CHANCE, KARST_CLUSTER, KARST_CLUSTER_SIZE, KARST_HEIGHT, KARST_LONE, KARST_RADIUS,
    KARST_WIDE_HEIGHT, KARST_WIDE_ONE_IN, KARST_WIDE_RADIUS, SINK_CELL, SINK_CHANCE, SINK_CLEAR, SINK_FLOOR, SINK_LAND, SINK_RADIUS, salt,
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

/// A sinkhole (cenote): its middle (blocks) and radius.
#[derive(Clone, Copy)]
struct Sink {
    x: i32,
    z: i32,
    radius: i32,
}

thread_local! {
    static SINKS: RefCell<HashMap<(u64, i32, i32), Option<Sink>>> = RefCell::new(HashMap::new());
}

/// Cell (`cx`, `cz`)'s sinkhole, if it has one: kept with `SINK_CHANCE` times the karst area's
/// strength, on land `SINK_LAND` over the sea, clear of the towers' feet by `SINK_CLEAR` blocks.
fn sink_site(h: &Here, cx: i32, cz: i32) -> Option<Sink> {
    let key = (h.f.seed, cx, cz);
    if let Some(hit) = SINKS.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let hash = fixed::hash(fixed::seed32(h.f.seed, salt::SINKS), cx, 0, cz);
    let margin = SINK_RADIUS.1 + 2;
    let span = (SINK_CELL - 2 * margin) as u32;
    let x = cx * SINK_CELL + margin + (hash % span) as i32;
    let z = cz * SINK_CELL + margin + ((hash >> 12) % span) as i32;
    let (qx, qz) = (x * 16, z * 16);
    let chance = (q12(SINK_CHANCE) * KARST_AREA.value(h.f, qx, qz)) >> 12;
    let site = ((fixed::fmix(hash) & 4095) < chance as u32)
        .then(|| {
            let radius = SINK_RADIUS.0 + (fixed::fmix(hash ^ 0x5174) % (SINK_RADIUS.1 - SINK_RADIUS.0 + 1) as u32) as i32;
            let here = Here::new(h.f, qx, qz);
            let ground = super::base_ground(&here);
            let land = (q4((SEA + SINK_LAND.0) as f64)..=q4((SEA + SINK_LAND.1) as f64)).contains(&ground);
            let (tx, tz) = (div_floor(x, KARST_CELL).0, div_floor(z, KARST_CELL).0);
            let clear = (-2..=2).all(|dz| {
                (-2..=2).all(|dx| {
                    tower(h, tx + dx, tz + dz).is_none_or(|t| {
                        let foot = t.radius + t.radius / 2 + 1 + radius + SINK_CLEAR;
                        (t.x - x).pow(2) + (t.z - z).pow(2) > foot * foot
                    })
                })
            });
            (land && clear).then_some(Sink { x, z, radius })
        })
        .flatten();
    SINKS.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 16384 {
            c.clear();
        }
        c.insert(key, site);
    });
    site
}

/// What a sinkhole takes off the ground (`base`, Q4, asked only inside the hole) at a column: inside its radius
/// down to `SINK_FLOOR` under the sea (the sea's level of water stands in it), a lip a block or
/// two lower round it.
pub(super) fn sink(h: &Here, base: impl FnOnce() -> i32) -> i32 {
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let Some(s) = sink_site(h, div_floor(bx, SINK_CELL).0, div_floor(bz, SINK_CELL).0) else { return 0 };
    let (ex, ez) = ((h.x - s.x * 16) as i64, (h.z - s.z * 16) as i64);
    let d = ((ex * ex + ez * ez) as u64).isqrt() as i32; // Q4
    let r = q4(s.radius as f64);
    if d < r {
        (q4((SEA - SINK_FLOOR) as f64) - base()).min(0)
    } else if d < r + q4(2.0) {
        -((q4(2.0) - (d - r)) / 2).max(0)
    } else {
        0
    }
}

/// The sinkholes (middle, radius) within `r` blocks of (`x`, `z`) (blocks), for the tools.
pub(super) fn sinks_near(f: &weft_sdk::noise::fixed::Fields, x: i32, z: i32, r: i32) -> Vec<(i32, i32, i32)> {
    let h = Here::new(f, x * 16, z * 16);
    let mut out = Vec::new();
    for cz in div_floor(z - r, SINK_CELL).0..=div_floor(z + r, SINK_CELL).0 {
        for cx in div_floor(x - r, SINK_CELL).0..=div_floor(x + r, SINK_CELL).0 {
            if let Some(s) = sink_site(&h, cx, cz) {
                out.push((s.x, s.z, s.radius));
            }
        }
    }
    out
}

//! Meteor craters (Ronald, 11-04: "all four"; the third proposal sheet): round bowls with a raised
//! rim in the open, dry land, rare. At most one per `CRATER_CELL` square, kept with
//! `CRATER_CHANCE` where the land at its middle fits; worked out once per cell and kept. The site
//! stays far enough inside its cell that a column only asks its own cell.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, ONE, div_floor, q4, q12};

use super::{
    CRATER_CELL, CRATER_CHANCE, CRATER_DEPTH, CRATER_RADIUS, CRATER_RIM, CRATER_RIM_OUT, FOREST_HUMIDITY, Here, KARST_AREA, MOUNTAINS_AREA,
    TAIGA_BELOW, base_ground, salt,
};
use crate::SEA;

/// A crater: its middle (blocks), radius to the rim's crest, depth and rim height (blocks).
#[derive(Clone, Copy)]
struct Crater {
    x: i32,
    z: i32,
    radius: i32,
    depth: i32,
    rim: i32,
}

/// The crater's ground stands on land this high over the sea at least (blocks).
const CRATER_LAND: i32 = 4;

thread_local! {
    static SITES: RefCell<HashMap<(u64, i32, i32), Option<Crater>>> = RefCell::new(HashMap::new());
}

/// How far a crater reaches past its middle (blocks): its rim's outer foot.
fn reach() -> i32 {
    ((CRATER_RADIUS.1 as f64) * CRATER_RIM_OUT) as i32 + 2
}

/// Cell (`cx`, `cz`)'s crater, if it has one.
fn crater(h: &Here, cx: i32, cz: i32) -> Option<Crater> {
    let key = (h.f.seed, cx, cz);
    if let Some(hit) = SITES.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let hash = fixed::hash(fixed::seed32(h.f.seed, salt::CRATERS), cx, 0, cz);
    let site = ((fixed::fmix(hash) & 4095) < q12(CRATER_CHANCE) as u32)
        .then(|| {
            let span = (CRATER_CELL - 2 * reach()) as u32;
            let x = cx * CRATER_CELL + reach() + (hash % span) as i32;
            let z = cz * CRATER_CELL + reach() + ((hash >> 11) % span) as i32;
            let here = Here::new(h.f, x * 16, z * 16);
            // Open, dry land: not the cold, the woods' wet, the mountains or the karst.
            let fits = here.temperature() >= q12(TAIGA_BELOW)
                && here.humidity() < q12(FOREST_HUMIDITY)
                && MOUNTAINS_AREA.at(&here) == 0
                && KARST_AREA.at(&here) == 0
                && base_ground(&here) >= q4((SEA + CRATER_LAND) as f64);
            fits.then(|| {
                let h2 = fixed::fmix(hash ^ 0xC4A7);
                let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
                let radius = pick(h2, CRATER_RADIUS);
                Crater { x, z, radius, depth: (radius as f64 * CRATER_DEPTH) as i32, rim: pick(h2 >> 8, CRATER_RIM) }
            })
        })
        .flatten();
    SITES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4096 {
            c.clear();
        }
        c.insert(key, site);
    });
    site
}

/// What a crater adds to the ground (Q4) at a column: inside the rim's crest a bowl `depth` deep
/// at the middle rising to the rim, outside it the rim falling away to nothing at
/// `CRATER_RIM_OUT` radii.
pub(super) fn add(h: &Here) -> i32 {
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let (cx, cz) = (div_floor(bx, CRATER_CELL).0, div_floor(bz, CRATER_CELL).0);
    let Some(c) = crater(h, cx, cz) else { return 0 };
    let reach = reach();
    if (bx - c.x).abs() > reach || (bz - c.z).abs() > reach {
        return 0;
    }
    let (ex, ez) = ((h.x - c.x * 16) as i64, (h.z - c.z * 16) as i64);
    let d = ((ex * ex + ez * ez) as u64).isqrt() as i32; // Q4
    let u = d * ONE / q4(c.radius as f64); // Q12, 1 at the crest
    let out = q12(CRATER_RIM_OUT);
    if u >= out {
        return 0;
    }
    let rim = q4(c.rim as f64);
    if u < ONE {
        let u2 = (u * u) >> 12;
        let u6 = (((u2 * u2) >> 12) * u2) >> 12;
        -((q4(c.depth as f64) * (ONE - u2)) >> 12) + ((rim * u6) >> 12)
    } else {
        let v = (out - u) * ONE / (out - ONE);
        (rim * ((v * v) >> 12)) >> 12
    }
}

/// The crater sites (middle, radius) within `r` blocks of (`x`, `z`) (blocks), for the tools.
pub(super) fn near(f: &weft_sdk::noise::fixed::Fields, x: i32, z: i32, r: i32) -> Vec<(i32, i32, i32)> {
    let h = Here::new(f, x * 16, z * 16);
    let (c0, c1) = (div_floor(x - r, CRATER_CELL).0, div_floor(x + r, CRATER_CELL).0);
    let (d0, d1) = (div_floor(z - r, CRATER_CELL).0, div_floor(z + r, CRATER_CELL).0);
    let mut out = Vec::new();
    for cz in d0..=d1 {
        for cx in c0..=c1 {
            if let Some(c) = crater(&h, cx, cz) {
                out.push((c.x, c.z, c.radius));
            }
        }
    }
    out
}

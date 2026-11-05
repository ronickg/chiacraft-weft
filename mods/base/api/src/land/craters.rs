//! Meteor craters (Ronald, 11-04: "all four"; the third proposal sheet): round bowls with a raised
//! rim in the open, dry land, rare. At most one per `CRATER_CELL` square, kept with
//! `CRATER_CHANCE` where the land at its middle fits; worked out once per cell and kept. The site
//! stays far enough inside its cell that a column only asks its own cell.

use std::cell::RefCell;
use std::collections::HashMap;

use weft_sdk::noise::fixed::{self, ONE, div_floor, q4, q12};

use super::{
    CRATER_CELL, CRATER_CHANCE, CRATER_DEPTH, CRATER_METEORITE_CHANCE, CRATER_METEORITE_DEPTH, CRATER_METEORITE_SPREAD, CRATER_METEORITES, CRATER_RADIUS, CRATER_RIM, CRATER_RIM_OUT, FOREST_HUMIDITY, Here, KARST_AREA, MOUNTAINS_AREA,
    TAIGA_BELOW, base_ground, lakes_on, rivers, salt,
};
use crate::SEA;

/// A crater: its middle (blocks), radius to the rim's crest, depth and rim height (blocks), and
/// the ground at its middle before it (Q4), which its floor and rim are measured from.
#[derive(Clone, Copy)]
struct Crater {
    x: i32,
    z: i32,
    radius: i32,
    depth: i32,
    rim: i32,
    ground: i32,
}

/// The ground round a crater's site may rise and fall this much (blocks) at most: on a slope a
/// bowl reads as nothing (the first look, seed 42 x -230 z -57, between mesas and a lake).
const CRATER_LEVEL: i32 = 10;

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
            let ground = base_ground(&here);
            // Open, dry land: not the cold, the woods' wet, the mountains or the karst; level.
            let open = here.temperature() >= q12(TAIGA_BELOW)
                && here.humidity() < q12(FOREST_HUMIDITY)
                && MOUNTAINS_AREA.at(&here) == 0
                && KARST_AREA.at(&here) == 0
                && ground >= q4((SEA + CRATER_LAND) as f64);
            let level = || {
                let r = CRATER_RADIUS.1 as f64 * 1.3;
                let round: Vec<i32> = (0..8)
                    .map(|k| {
                        let a = k as f64 * std::f64::consts::FRAC_PI_4;
                        let (px, pz) = (x + (a.cos() * r) as i32, z + (a.sin() * r) as i32);
                        base_ground(&Here::new(h.f, px * 16, pz * 16))
                    })
                    .collect();
                round.iter().max().unwrap() - round.iter().min().unwrap() <= q4(CRATER_LEVEL as f64)
            };
            // Dry: no river or lake in the bowl (the rivers run before the craters; one passing outside
            // meets the rim, which the crater's own heights win).
            let dry = || {
                let out = CRATER_RADIUS.1 as f64;
                (0..24).all(|k| {
                    let a = k as f64 / 24.0 * std::f64::consts::TAU;
                    [0.0, 0.35, 0.7, 1.0].iter().all(|m| {
                        let (px, pz) = ((x as f64 + a.cos() * out * m) as i32 * 16, (z as f64 + a.sin() * out * m) as i32 * 16);
                        rivers(h.f, px, pz) == 0 && lakes_on(&Here::new(h.f, px, pz), i32::MIN) == 0
                    })
                })
            };
            (open && level() && dry()).then(|| {
                let h2 = fixed::fmix(hash ^ 0xC4A7);
                let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
                let radius = pick(h2, CRATER_RADIUS);
                Crater { x, z, radius, depth: (radius as f64 * CRATER_DEPTH) as i32, rim: pick(h2 >> 8, CRATER_RIM), ground }
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

/// What a crater adds to the ground (`base`, Q4: the land with the rivers' cut; asked only near
/// one) at a column. Measured
/// from the ground at its middle, so it stands level on gentle slopes: inside the rim's crest a
/// bowl `depth` deep at the middle rising to the rim; outside it the rim, blending back into the
/// land by `CRATER_RIM_OUT` radii.
pub(super) fn add(h: &Here, base: impl FnOnce() -> i32) -> i32 {
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
    let base = base();
    if u < ONE {
        let u2 = (u * u) >> 12;
        let u6 = (((u2 * u2) >> 12) * u2) >> 12;
        c.ground - ((q4(c.depth as f64) * (ONE - u2)) >> 12) + ((rim * u6) >> 12) - base
    } else {
        // From the crest's height down the rim's outer side, blending into the land.
        let v = (out - u) * ONE / (out - ONE);
        let w = (v * v) >> 12;
        let crest = c.ground + ((rim * w) >> 12);
        ((crest - base) * w) >> 12
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

/// The meteorites under the floors of the craters whose middles are within `r` blocks of (`x`,
/// `z`) (blocks): 1 crater in 3 (`CRATER_METEORITE_CHANCE`) holds `CRATER_METEORITES`, each
/// `CRATER_METEORITE_DEPTH` under the floor over it, near the middle.
pub(super) fn meteorites(f: &weft_sdk::noise::fixed::Fields, x: i32, z: i32, r: i32) -> Vec<[i32; 3]> {
    let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
    let mut out = Vec::new();
    for (cx, cz, radius) in near(f, x, z, r) {
        let seed = fixed::seed32(f.seed, salt::CRATER_METEORITES);
        let hash = fixed::hash(seed, cx, 0, cz);
        if (fixed::fmix(hash) & 4095) >= q12(CRATER_METEORITE_CHANCE) as u32 {
            continue;
        }
        let spread = ((radius as f64 * CRATER_METEORITE_SPREAD) as i32).max(1);
        for i in 0..pick(hash >> 12, CRATER_METEORITES) {
            let k = fixed::fmix(fixed::hash(seed, cx, i + 1, cz));
            let (mx, mz) = (cx + pick(k, (-spread, spread)), cz + pick(k >> 10, (-spread, spread)));
            let floor = super::height(f, mx * 16, mz * 16) >> 4;
            out.push([mx, floor - pick(k >> 20, CRATER_METEORITE_DEPTH), mz]);
        }
    }
    out
}

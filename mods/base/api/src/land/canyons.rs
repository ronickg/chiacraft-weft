//! Badland canyons (`reports/terrain-terralith-tectonic.md`, step 5; Ronald, 11-04): winding,
//! stepped cuts in the hot, dry land along a warped line noise's zero lines. The area only says
//! where they may run; inside it they cut their full depth (from `CANYON_FULL` of its strength),
//! so they read as canyons, not scratches.

use weft_sdk::noise::fixed::{ONE, div_floor, q4, q12, ramp};

use super::{CANYON_AREA, CANYON_DEPTH, CANYON_DRY, CANYON_FULL, CANYON_SCALE, CANYON_TIER, CANYON_WARP, CANYON_WIDTH, Here, salt};
use crate::SEA;

/// The canyon noise's distance from its zero line (Q12) at the wandered point.
fn line(h: &Here) -> i32 {
    let (f, x, z) = (h.f, h.x, h.z);
    let (warp, ws) = (q4(CANYON_WARP.0), q4(CANYON_WARP.1));
    let wx = x + ((f.fbm(salt::CANYON_WARP, x, z, ws, 2) * warp) >> 12);
    let wz = z + ((f.fbm(salt::CANYON_WARP + 1, x, z, ws, 2) * warp) >> 12);
    f.fbm(salt::CANYONS, wx, wz, q4(CANYON_SCALE), 3).abs()
}

/// Whether a canyon could cut here: inside its area (the climate ramps first) and its line.
pub(super) fn near(h: &Here) -> bool {
    CANYON_AREA.at(h) > 0 && line(h) < q12(CANYON_WIDTH)
}

/// What the canyons take off ground `base` (Q4, zero or less): tiers of a steep wall and a ledge
/// from the rim down to the floor, the floor kept dry.
pub(super) fn cut(h: &Here, base: i32) -> i32 {
    let w = CANYON_AREA.at(h);
    if w == 0 {
        return 0;
    }
    let half = q12(CANYON_WIDTH);
    let d = line(h);
    if d >= half {
        return 0;
    }
    // 0 at the rim, ONE in the middle; the depth there, full from `CANYON_FULL` of the area.
    let u = ONE - d * ONE / half;
    let depth = (q4(CANYON_DEPTH as f64) * ramp(w, 0, q12(CANYON_FULL))) >> 12;
    let tier = q4(CANYON_TIER as f64);
    let tiers = (depth / tier).max(1);
    // Each tier: the first fifth of its width is the wall, the rest its ledge; the middle third
    // of the canyon is its floor.
    let u = (u * 3 / 2).min(ONE);
    let span = (ONE / tiers).max(1);
    let (k, into) = div_floor(u, span);
    let wall = ramp(into, 0, span / 5);
    let down = ((k * ONE / tiers) + ((wall * (ONE / tiers)) >> 12)).min(ONE);
    let cut = -((depth * down) >> 12);
    // Dry: not under the sea's level and a little.
    let floor = q4((SEA + CANYON_DRY) as f64);
    cut.max((floor - base).min(0))
}

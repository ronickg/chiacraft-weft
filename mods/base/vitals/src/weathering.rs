//! How building materials age in the world (the RTX showcase blocks, `base::showcase`).

use crate::DAY_SECS;

/// A copper block with air above it turns to verdigris after about this long on average (a few
/// in-game days, so a new roof or trim shines for a while first).
pub const VERDIGRIS_SECS: f32 = 3.0 * DAY_SECS;

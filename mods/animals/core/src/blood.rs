//! Wounds and blood trails (`notes/crew/plans/carcass-and-skinning.md`, "hunters following blood"): a heavy blow opens a
//! wound that bleeds a while, taking health a second, and an animal bleeding as it goes leaves spots of blood behind
//! it that dry and fade. A hungry hunter with no prey in sight follows the freshest spots it finds: to the bleeding
//! animal, or to the carcass where it bled out. The game's side is `mods/animals/src/blood.rs`.
//!
//! The numbers follow the players' wound (`mods/base/src/survival/body.rs`: a blow of a fifth of max health opens one)
//! and what hunters know: a deer hit badly runs on and lies down a few hundred metres off within the hour; one hit
//! lightly runs off and lives (the blood thins out over the trail and stops).

/// A blow that takes this share of max health or more opens a wound (as a player's, `survival::body::WOUND`).
pub const WOUND: f32 = 0.2;
/// It bleeds this long, seconds, for a blow of `WOUND`, and as much longer as the blow was heavier, up to
/// `BLEED_MAX_SECS`; a second wound bleeds as long as the longer of the two.
pub const BLEED_SECS: f32 = 30.0;
pub const BLEED_MAX_SECS: f32 = 120.0;
/// Health a second while it bleeds, as a share of max health: a full `BLEED_MAX_SECS` takes 48 %, so an animal left
/// under half by a heavy blow bleeds out, and one left well over half lives.
pub const BLEED_RATE: f32 = 0.004;

/// A spot falls at most this often, seconds, while it bleeds, and only when it has moved this many blocks since the
/// last one (one standing still or lying bleeds into one spot).
pub const SPOT_EVERY_SECS: f32 = 2.0;
pub const SPOT_STEP: f64 = 1.0;
/// Spots stay this long, seconds, shrinking as they dry to `SPOT_DRY` of their size, then go.
pub const SPOT_LIFE_SECS: f32 = 300.0;
pub const SPOT_DRY: f32 = 0.4;

/// A hungry hunter smells or sees blood this many blocks off, and follows a trail while its bleeder is within
/// `TRAIL_REACH` (further, it has run too far; the spots tell of a carcass only where it lies).
pub const TRAIL_SMELL: f64 = 24.0;
pub const TRAIL_REACH: f64 = 96.0;

/// How long a blow of `share` of max health bleeds, seconds (0: not a wound).
pub fn bleed_secs(share: f32) -> f32 {
    if share < WOUND { 0.0 } else { (BLEED_SECS * share / WOUND).min(BLEED_MAX_SECS) }
}

/// Health lost bleeding for `secs` seconds at `max_health`.
pub fn bleed_loss(max_health: f32, secs: f32) -> f32 {
    max_health * BLEED_RATE * secs
}

/// A spot's drawn size, 1 fresh to `SPOT_DRY` dried, at `age` seconds (`None`: gone).
pub fn spot_size(age: f32) -> Option<f32> {
    (age < SPOT_LIFE_SECS).then(|| 1.0 - (1.0 - SPOT_DRY) * (age / SPOT_LIFE_SECS).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wounds_bleed_by_the_blow() {
        assert_eq!(bleed_secs(0.1), 0.0);
        assert_eq!(bleed_secs(WOUND), BLEED_SECS);
        assert!(bleed_secs(0.4) > bleed_secs(0.3));
        assert_eq!(bleed_secs(5.0), BLEED_MAX_SECS);
        // The longest bleed takes under half: a light wound lives, a heavy one left low dies.
        let most = bleed_loss(100.0, BLEED_MAX_SECS);
        assert!(most > 40.0 && most < 50.0, "{most}");
        assert_eq!(spot_size(0.0), Some(1.0));
        assert!(spot_size(SPOT_LIFE_SECS * 0.5).is_some_and(|s| s < 1.0 && s > SPOT_DRY));
        assert_eq!(spot_size(SPOT_LIFE_SECS), None);
        // A whole bleed leaves at most 60 spots.
        assert!(BLEED_MAX_SECS / SPOT_EVERY_SECS <= 60.0);
    }
}

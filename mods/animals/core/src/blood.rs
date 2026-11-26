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

/// A spot falls every this many blocks it goes while it bleeds, laid along its way since the last (one standing still or
/// lying bleeds into one spot), at most `SPOT_PER_SEC` a second: r4's deer, a spot every 2 s, left the first one 12
/// blocks from the next as it ran.
pub const SPOT_GAP: f64 = 3.0;
pub const SPOT_PER_SEC: u32 = 2;
/// Spots stay this long, seconds, shrinking as they dry to `SPOT_DRY` of their size, then go.
pub const SPOT_LIFE_SECS: f32 = 300.0;
pub const SPOT_DRY: f32 = 0.4;
/// A fresh spot is drawn this many times its model's size (`blood_spot`, 0.4 blocks): cov_blood r1's read as specks a
/// pixel or two across from 36 blocks up and barely showed from the ground.
pub const SPOT_SCALE: f32 = 1.8;

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

/// A spot's drawn size, `SPOT_SCALE` fresh to `SPOT_DRY` of that dried, at `age` seconds (`None`: gone).
pub fn spot_size(age: f32) -> Option<f32> {
    (age < SPOT_LIFE_SECS).then(|| SPOT_SCALE * (1.0 - (1.0 - SPOT_DRY) * (age / SPOT_LIFE_SECS).clamp(0.0, 1.0)))
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
        assert_eq!(spot_size(0.0), Some(SPOT_SCALE));
        assert!(spot_size(SPOT_LIFE_SECS * 0.5).is_some_and(|s| s < SPOT_SCALE && s > SPOT_DRY * SPOT_SCALE));
        assert_eq!(spot_size(SPOT_LIFE_SECS), None);
        // A whole bleed leaves at most 240 spots.
        assert!(BLEED_MAX_SECS * SPOT_PER_SEC as f32 <= 240.0);
    }
}

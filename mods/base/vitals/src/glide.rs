//! The glider (`mods/base/src/glider.rs`; the engine glides by `core:glide_speed` and the rest):
//! opened with jump in the air, it flies where one looks at its airspeed and sinks through the air
//! at its rate. The air moves it: the day's wind carries it (further downwind, less far into it),
//! sun-warmed ground and wind up a slope lift it. It wears by the second aloft.

use crate::curve::{LEVELS_PER_AGE, g};

/// A glider: airspeed and sink in still air (blocks a second, a block a metre), and its item
/// level (durability from the curve).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wing {
    pub name: &'static str,
    pub speed: f32,
    pub sink: f32,
    pub level: f32,
}

impl Wing {
    /// Blocks forward for each block down in still air.
    pub fn ratio(&self) -> f32 {
        self.speed / self.sink
    }

    /// Seconds aloft it lasts.
    pub fn durability(&self) -> u32 {
        (DURABILITY_SECS * g(self.level)).round() as u32
    }

    /// Blocks it covers from `height` blocks up with the wind `wind` blocks a second along its
    /// way (negative into it), no rising air.
    pub fn range(&self, height: f32, wind: f32) -> f32 {
        height / self.sink * (self.speed + wind).max(0.0)
    }
}

/// The hide glider: rawhide stretched on a frame of poles, lashed with sinew. Stone Age by its
/// materials (leather's level, `gear::MATERIALS`: tier 0, depth 5). Its 4.4 to 1 is a primitive
/// wing's: a paraglider makes 8-10, a hang glider 15 (proposed).
pub const HIDE: Wing = Wing { name: "base:glider", speed: 8.0, sink: 1.8, level: 0.0 * LEVELS_PER_AGE + 5.0 };

pub const WINGS: [Wing; 1] = [HIDE];

pub fn wing(item: &str) -> Option<&'static Wing> {
    WINGS.iter().find(|w| w.name == item)
}

/// Seconds aloft at level 0 (×1.26 at the hide glider's 5: ten minutes).
pub const DURABILITY_SECS: f32 = 480.0;

/// The share of the wind (`air::wind`, m/s at the surface) that carries a glider: it flies low,
/// in the air the ground slows.
pub const WIND_SHARE: f32 = 0.75;

/// Rising air over sun-warmed ground: at most this many blocks a second at the day's warmest, in
/// one of `THERMAL_SHARE` cells of `THERMAL_CELL` blocks, up to `THERMAL_REACH` over the ground.
/// None at night or in rain, little over grass, none over water, snow or leaves.
pub const THERMAL: f32 = 1.6;
pub const THERMAL_CELL: i32 = 40;
pub const THERMAL_SHARE: f32 = 0.3;
pub const THERMAL_REACH: f32 = 80.0;

/// The sun's warmth through the day for thermals, 0 to 1: from 09:00, strongest at 14:00, gone
/// by 18:00.
pub fn sun(hour: f32) -> f32 {
    if !(9.0..18.0).contains(&hour) {
        return 0.0;
    }
    let up = ((hour - 9.0) / 5.0).min(1.0);
    let down = ((18.0 - hour) / 4.0).min(1.0);
    (up * down).clamp(0.0, 1.0)
}

/// How much a ground warms the air over it, by its kind.
pub fn warms(ground: Ground) -> f32 {
    match ground {
        Ground::Bare => 1.0,
        Ground::Grass => 0.5,
        Ground::Cool => 0.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// Sand, rock, dry dirt.
    Bare,
    /// Grass and low plants.
    Grass,
    /// Water, snow, ice, leaves.
    Cool,
}

/// Wind up a slope lifts: the wind (blocks a second) times the rise over `RIDGE_LOOK` blocks
/// downwind (capped at 1), times `RIDGE_SHARE`, up to `RIDGE_REACH` over the top.
pub const RIDGE_LOOK: f32 = 8.0;
pub const RIDGE_SHARE: f32 = 0.8;
pub const RIDGE_REACH: f32 = 24.0;

pub fn ridge(wind: f32, rise: f32) -> f32 {
    wind.max(0.0) * (rise / RIDGE_LOOK).clamp(0.0, 1.0) * RIDGE_SHARE
}

/// Rising air never lifts more than this, blocks a second.
pub const LIFT_MAX: f32 = 3.0;

/// Seconds between looks at the air round a glider (wind, thermals, ridges).
pub const AIR_EVERY: u64 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wind_sets_the_range() {
        let still = HIDE.range(40.0, 0.0);
        let down = HIDE.range(40.0, 4.0);
        let into = HIDE.range(40.0, -4.0);
        assert!((still / 40.0 - HIDE.ratio()).abs() < 1e-3);
        assert!(down > still * 1.4 && into < still * 0.6, "{down} {still} {into}");
    }

    #[test]
    fn it_lasts_about_ten_minutes() {
        let d = HIDE.durability();
        assert!((500..=700).contains(&d), "{d}");
    }

    #[test]
    fn thermals_follow_the_sun() {
        assert_eq!(sun(3.0), 0.0);
        assert_eq!(sun(20.0), 0.0);
        assert!(sun(14.0) > 0.99);
        assert!(sun(10.0) < sun(13.0));
    }
}

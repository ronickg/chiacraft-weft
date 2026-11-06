//! Wind life: small things the wind moves (Ronald, 11-05: "he even has like leaves flying around";
//! the Recrafted study's items 1 and 3, `reports/Recrafted study.md`). Leaves come off the trees one
//! at a time in calm air and in bursts as a gust passes, tumble down, lie as litter for minutes,
//! blow on along the ground in gusts and up again in a gale, and float on water; petals blow off
//! flowers in bloom and puff up where someone walks through them; snow blows off snowfields in a
//! strong wind (spindrift) and sand off the dunes; pollen drifts over meadows on spring and summer
//! days; leaves drip after rain. The client puts them about (`base/src/wind_life.rs`), the GPU
//! carries them on the wind (`particle_sim.wgsl`). These are the numbers.
//!
//! Rates are per block a second: a leaf block whose underside is open drops one about every four
//! minutes in calm air (Recrafted's 1.25% a display tick, about 0.005 a second), so a forest round
//! the player sheds a few a second.

use crate::{calendar, foliage};

/// The setting `base:fx_wind_life`: off, few (the default, cheap on the Mac and Android), many,
/// ultra (a beefy GPU). How many of everything, as a factor of the rates below.
pub const DENSITY: [f32; 4] = [0.0, 1.0, 2.5, 6.0];

/// How far round the player things are put (blocks), across and up; and down.
pub const REACH: i32 = 20;
pub const REACH_DOWN: i32 = 6;
/// Blocks looked at a second, at random in that box (vanilla's display ticks look at some 13,000);
/// a block's chance of being picked is this over the box's volume.
pub const LOOKS_PER_SEC: f32 = 6000.0;

// ---- leaves ----

/// A leaf block with open air under it: leaves a second it drops in calm air, by how its leaves
/// turn (`wild::Turn`): broadleaves, and evergreens' needles and scales far fewer.
pub const LEAF_RATE: f32 = 0.005;
pub const NEEDLE_RATE: f32 = 0.0012;
/// In the windiest part of the day more come off: the rate times `1 + LEAF_WIND * wind / 10 m/s`.
pub const LEAF_WIND: f32 = 0.8;
/// A gust passing (`gust` over `GUST_FROM`) blows up to this many times the calm rate off the
/// downwind side of the crowns (Recrafted: 7).
pub const GUST_BURST: f32 = 7.0;
pub const GUST_FROM: (f32, f32) = (1.15, 1.5);
/// Rain beats more down, by how hard it falls.
pub const RAIN_MORE: f32 = 1.0;
/// Seconds a leaf lives, shortest and longest: its fall and then lying as litter (Recrafted's
/// complete preset: 4 minutes).
pub const LEAF_LIFE: (f32, f32) = (90.0, 200.0);
/// A leaf's size (blocks): 5 to 7 of the world's 32 texels.
pub const LEAF_SIZE: (f32, f32) = (0.16, 0.22);
/// How fast a leaf takes the air's speed (per second), and how fast it falls through still air
/// (blocks a second; a real broadleaf about 1-1.5 m/s).
pub const LEAF_AIR: f32 = 3.0;
pub const LEAF_SINK: f32 = 1.1;
/// The wind (m/s, gusts included) that blows a lying leaf on along the ground; in a gale it is
/// lifted again.
pub const LEAF_LIFT: f32 = 7.0;

/// How many more leaves fall in autumn, at its peak, where the year's seasons are strongest
/// (`foliage::strength`); in spring's fresh green fewer.
pub const AUTUMN_FALL: f32 = 4.0;
pub const SPRING_FALL: f32 = 0.6;

/// The share of the calm rate deciduous trees shed on `day` of the year (0 at spring's start), where
/// the year has `strength` (`foliage::strength`, 0 in the tropics): the autumn peak follows the
/// leaves' turning (`foliage::tint`), late in autumn most.
pub fn fall(day: f32, strength: f32) -> f32 {
    let d = day.rem_euclid(calendar::YEAR_DAYS as f32);
    let q = calendar::YEAR_DAYS as f32 / 4.0;
    let autumn = smooth(2.0 * q, 2.7 * q, d) * (1.0 - smooth(3.1 * q, 3.5 * q, d));
    let spring = smooth(0.0, 0.1 * q, d) * (1.0 - smooth(0.6 * q, 1.0 * q, d));
    1.0 + (AUTUMN_FALL - 1.0) * autumn * strength - (1.0 - SPRING_FALL) * spring * strength
}

/// The share for a place whose year swings `seasons` °C (`climate::Climate::seasons`).
pub fn fall_at(day: f32, seasons: f32) -> f32 {
    fall(day, foliage::strength(seasons))
}

// ---- petals and pollen ----

/// A flower in bloom: petals a second in calm air, and the most a gust blows off (Recrafted's
/// flowers: 0.8 of the leaves' rate, gusts twice).
pub const PETAL_RATE: f32 = 0.004;
pub const PETAL_GUST: f32 = 2.0;
/// A tree in blossom (cherry, dogwood): its blooming leaves shed petals instead of leaves, more.
pub const BLOSSOM_RATE: f32 = 0.008;
pub const PETAL_LIFE: (f32, f32) = (40.0, 90.0);
pub const PETAL_SIZE: (f32, f32) = (0.07, 0.1);
pub const PETAL_AIR: f32 = 4.0;
pub const PETAL_SINK: f32 = 0.55;
pub const PETAL_LIFT: f32 = 5.0;
/// Walking through flowers in bloom: the chance a second, at walking speed (4.3 m/s), of a puff,
/// and its petals.
pub const STIR_CHANCE: f32 = 2.5;
pub const STIR_PETALS: (u32, u32) = (2, 4);

/// Pollen: motes a second off a flower in bloom by day (06:00 to 19:00), when it is dry.
pub const POLLEN_RATE: f32 = 0.006;
pub const POLLEN_LIFE: (f32, f32) = (5.0, 9.0);
pub const POLLEN_SIZE: (f32, f32) = (0.03, 0.05);
pub const POLLEN_AIR: f32 = 8.0;
pub const POLLEN_SINK: f32 = 0.05;
pub const POLLEN_HOURS: (f32, f32) = (6.0, 19.0);

// ---- snow and sand ----

/// Wind (m/s) from which snow lying in the open blows (spindrift), and at which it blows most;
/// grains a second off a snow surface at the most.
pub const SPINDRIFT: (f32, f32) = (6.0, 14.0);
pub const SPINDRIFT_RATE: f32 = 0.6;
pub const SPINDRIFT_LIFE: (f32, f32) = (1.5, 3.5);
pub const SPINDRIFT_SIZE: (f32, f32) = (0.04, 0.08);
pub const SPINDRIFT_AIR: f32 = 6.0;
pub const SPINDRIFT_SINK: f32 = 0.35;

/// Wind (m/s) from which sand blows off dunes and beaches (air::DUST's range: grains move in a
/// fresh breeze), and grains a second off a sand surface at the most; a dust storm
/// (`air::dust`) fills the air with them wherever it blows.
pub const SAND: (f32, f32) = (7.0, 13.0);
pub const SAND_RATE: f32 = 0.5;
pub const SAND_LIFE: (f32, f32) = (1.0, 2.5);
pub const SAND_SIZE: (f32, f32) = (0.03, 0.06);
pub const SAND_AIR: f32 = 7.0;
pub const SAND_SINK: f32 = 0.6;
/// In a dust storm: grains a second put in the air round the player at its thickest.
pub const DUST_STORM: f32 = 120.0;

// ---- drips ----

/// A leaf block over open air while it rains, and for `DRIP_AFTER` seconds after: drops a second.
pub const DRIP_RATE: f32 = 0.01;
pub const DRIP_AFTER: f32 = 90.0;

/// Seconds between the server's looks at each player's season and dust (`fall`, `air::dust`).
pub const REFRESH_SECS: f32 = 5.0;

/// How much a gust passing raises the rate: 1 under `GUST_FROM.0`, `GUST_BURST` from `GUST_FROM.1`.
pub fn burst(gust: f32) -> f32 {
    1.0 + (GUST_BURST - 1.0) * smooth(GUST_FROM.0, GUST_FROM.1, gust)
}

/// The calm rate's factor for the wind `wind` (m/s) and the rain `rain` (0..1).
pub fn weather(wind: f32, rain: f32) -> f32 {
    (1.0 + LEAF_WIND * wind / 10.0) * (1.0 + RAIN_MORE * rain.clamp(0.0, 1.0))
}

/// The share (0..1) of the most that blows at `wind` m/s over the range `range` (`SPINDRIFT`, `SAND`).
pub fn blows(wind: f32, range: (f32, f32)) -> f32 {
    smooth(range.0, range.1, wind)
}

/// How many to put now for an expected `mean` (a Poisson-ish draw with `u` uniform in 0..1): the
/// whole part, and one more by the fraction.
pub fn draw(mean: f32, u: f32) -> u32 {
    let whole = mean.max(0.0).floor();
    whole as u32 + (u < mean - whole) as u32
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leaves fall most in autumn, fewest in spring, and the same all year where the seasons don't
    /// swing (the tropics).
    #[test]
    fn the_year() {
        let q = calendar::YEAR_DAYS as f32 / 4.0;
        let at = |s: f32| fall(s * q, 1.0);
        assert!(at(2.85) > 3.5, "late autumn {}", at(2.85));
        assert!(at(0.3) < 0.7, "spring {}", at(0.3));
        assert!((at(1.5) - 1.0).abs() < 1e-3, "summer {}", at(1.5));
        for d in 0..calendar::YEAR_DAYS * 4 {
            assert!((fall(d as f32 / 4.0, 0.0) - 1.0).abs() < 1e-6);
        }
    }

    /// In a forest of some 1,000 open leaf undersides in reach a few leaves a second come off in
    /// calm air on the default setting; a gust's burst many more.
    #[test]
    fn the_rates() {
        let calm = 1000.0 * LEAF_RATE * weather(2.0, 0.0) * DENSITY[1];
        assert!((2.0..10.0).contains(&calm), "calm {calm}");
        assert!(burst(1.0) == 1.0 && (burst(2.0) - GUST_BURST).abs() < 1e-6);
    }

    #[test]
    fn draws() {
        assert_eq!(draw(0.0, 0.5), 0);
        assert_eq!(draw(0.3, 0.2), 1);
        assert_eq!(draw(0.3, 0.4), 0);
        assert_eq!(draw(2.5, 0.9), 2);
        assert_eq!(draw(2.5, 0.1), 3);
    }
}

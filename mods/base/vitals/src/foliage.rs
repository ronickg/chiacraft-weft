//! Grass and leaves through the year: the colors a biome's grass and foliage tints take by the
//! day of the year, fresh in spring, the biome's own in summer, gold and orange then straw in
//! autumn, dull in winter, as strongly as the place's year swings (`mods/base/src/seasons.rs`
//! tells the engine, `World::set_biome_tints`). Leaves turn as their tree does (`wild::Turn`):
//! evergreens keep their color, birches go gold, oaks russet.

use crate::calendar::YEAR_DAYS;

/// What a tint colors: grass, or leaves that turn to `autumn` (0xRRGGBB, `wild::Turn::autumn`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Grass,
    Leaves { autumn: u32 },
}

/// How strongly a place's year shows in its plants, 0 to 1: from its seasons' swing (°C a side
/// of the mean, `Climate::seasons`). The tropics (a degree or two) stay green; Kassel (8.7) and
/// colder places turn fully.
pub fn strength(seasons: f32) -> f32 {
    smoothstep(2.0, 8.0, seasons)
}

/// The look each season pulls toward, grass then foliage (0xRRGGBB), and how far each goes at
/// its fullest.
const FRESH: (u32, u32) = (0x98d860, 0xa8d85a);
const FRESH_AMOUNT: (f32, f32) = (0.35, 0.35);
const AUTUMN_GRASS: u32 = 0xb0a058;
const AUTUMN_AMOUNT: (f32, f32) = (0.45, 0.75);
const WINTER: (u32, u32) = (0x908860, 0x6e6648);
const WINTER_AMOUNT: (f32, f32) = (0.5, 0.6);

/// The color of `base` (the biome's own, 0xRRGGBB) on `day` of the year (0 at spring's start, a
/// fraction for the hour; `YEAR_DAYS` long) where the year has `strength` (`strength`).
pub fn tint(base: u32, part: Part, day: f32, strength: f32) -> u32 {
    let d = day.rem_euclid(YEAR_DAYS as f32);
    let q = YEAR_DAYS as f32 / 4.0;
    // Fresh green peaks halfway through spring; the leaves turn from late summer, are at their
    // most colored late in autumn and fall into winter's dull look, which lasts to spring.
    let fresh = bump(d, 0.1 * q, 0.5 * q, 1.3 * q);
    let autumn = smoothstep(1.85 * q, 2.6 * q, d) * (1.0 - smoothstep(2.9 * q, 3.25 * q, d));
    let winter = smoothstep(2.9 * q, 3.3 * q, d).max(1.0 - smoothstep(0.0, 0.25 * q, d));
    let pick = |look: (u32, u32)| if part == Part::Grass { look.0 } else { look.1 };
    let amount = |a: (f32, f32)| if part == Part::Grass { a.0 } else { a.1 };
    let autumn_look = match part {
        Part::Grass => AUTUMN_GRASS,
        Part::Leaves { autumn } => autumn,
    };
    let mut c = rgb(base);
    c = mix(c, rgb(pick(FRESH)), fresh * amount(FRESH_AMOUNT) * strength);
    c = mix(c, rgb(autumn_look), autumn * amount(AUTUMN_AMOUNT) * strength);
    c = mix(c, rgb(pick(WINTER)), winter * amount(WINTER_AMOUNT) * strength);
    pack(c)
}

fn bump(x: f32, from: f32, peak: f32, to: f32) -> f32 {
    smoothstep(from, peak, x) * (1.0 - smoothstep(peak, to, x))
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn rgb(c: u32) -> [f32; 3] {
    [(c >> 16 & 255) as f32, (c >> 8 & 255) as f32, (c & 255) as f32]
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t.clamp(0.0, 1.0))
}

fn pack(c: [f32; 3]) -> u32 {
    let ch = |v: f32| v.round().clamp(0.0, 255.0) as u32;
    ch(c[0]) << 16 | ch(c[1]) << 8 | ch(c[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    const OAK: u32 = 0x59ae30;
    const OAK_LEAVES: Part = Part::Leaves { autumn: 0xb8622a };

    #[test]
    fn summer_keeps_the_biomes_own_color() {
        let mid_summer = YEAR_DAYS as f32 * 3.0 / 8.0;
        assert_eq!(tint(OAK, OAK_LEAVES, mid_summer, 1.0), OAK);
    }

    #[test]
    fn the_tropics_stay_green_all_year() {
        for day in 0..YEAR_DAYS {
            assert_eq!(tint(OAK, OAK_LEAVES, day as f32, strength(1.0)), OAK);
        }
    }

    #[test]
    fn late_autumn_leaves_are_warm() {
        let late_autumn = YEAR_DAYS as f32 * 0.7;
        let c = rgb(tint(OAK, OAK_LEAVES, late_autumn, 1.0));
        assert!(c[0] > c[1] * 0.9, "{c:?}");
    }

    #[test]
    fn the_year_wraps_without_a_jump() {
        let end = rgb(tint(OAK, Part::Grass, YEAR_DAYS as f32 - 0.01, 1.0));
        let start = rgb(tint(OAK, Part::Grass, 0.0, 1.0));
        assert!((0..3).all(|i| (end[i] - start[i]).abs() < 2.0), "{end:?} {start:?}");
    }
}

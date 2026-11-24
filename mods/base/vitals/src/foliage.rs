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

/// How much of a grass or leaf tint's colour stays, 0 to 1: every biome's tints, and the looks the
/// seasons and the rains pull them toward, go this far from their own grey (`muted`), their
/// brightness and hue kept. Ronald, 11-04: "Tints -15% + small step" (under the new look the plains'
/// grass read 0.67 saturation against Bliss's land at about 0.3, and Art measured sunlit leaves at
/// 0.71 against 0.45-0.65).
pub const TINT_SATURATION: f32 = 0.85;

/// The engine's own grass and foliage tints (`core:grass`, `core:foliage`), and base's leaf kinds'
/// (`base_api::trees`), for the biomes that named none: a mod can't change the engine's.
pub const ENGINE_GRASS: u32 = 0x8db360;
pub const ENGINE_FOLIAGE: u32 = 0x77ab2f;
pub const LEAVES: u32 = 0x48b518;

/// `c` (0xRRGGBB) with `TINT_SATURATION` of its colour: each channel taken toward the colour's
/// luminance (Rec. 709), which stays.
pub const fn muted(c: u32) -> u32 {
    let (r, g, b) = ((c >> 16 & 255) as f32, (c >> 8 & 255) as f32, (c & 255) as f32);
    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    channel(l, r) << 16 | channel(l, g) << 8 | channel(l, b)
}

/// One channel `v` of a colour of luminance `l`, muted (`muted`).
const fn channel(l: f32, v: f32) -> u32 {
    let v = l + (v - l) * TINT_SATURATION + 0.5;
    if v < 0.0 {
        0
    } else if v > 255.0 {
        255
    } else {
        v as u32
    }
}

/// The look each season pulls toward, grass then foliage (0xRRGGBB), and how far each goes at
/// its fullest.
const FRESH: (u32, u32) = (0xa0e060, 0xb0e060);
const FRESH_AMOUNT: (f32, f32) = (0.45, 0.45);
const AUTUMN_GRASS: u32 = 0xb0a058;
const AUTUMN_AMOUNT: (f32, f32) = (0.45, 0.75);
// Winter: grass gone the color of hay, the leaves that turned a dead grey-brown, darker and less
// green than autumn's russet so the year runs on from it (until trees can stand bare; at 0.75 of
// an older, lighter brown they came out olive and read as early autumn: the world playtest).
const WINTER: (u32, u32) = (0x9a8f62, 0x66574a);
const WINTER_AMOUNT: (f32, f32) = (0.6, 0.9);
/// Where winter's look starts greening, in seasons from spring's start: winter's last quarter (3.5
/// days), gone by spring's first day.
const WINTER_ENDS: f32 = 3.75;

/// The color of `base` (the biome's own, 0xRRGGBB, already `muted`) on `day` of the year (0 at spring's start, a
/// fraction for the hour; `YEAR_DAYS` long) where the year has `strength` (`strength`).
pub fn tint(base: u32, part: Part, day: f32, strength: f32) -> u32 {
    let d = day.rem_euclid(YEAR_DAYS as f32);
    let q = YEAR_DAYS as f32 / 4.0;
    // Fresh green from spring's first day to its peak halfway through; the leaves turn from late
    // summer, are at their most colored late in autumn and fall into winter's dull look, which
    // greens again over winter's last quarter, so spring's first day is green: every world starts
    // on it (the world playtest, 11-04: forests and lakesides khaki and grey-brown at "Spring 1",
    // when winter's look still faded over spring's first 3.5 days).
    let fresh = bump(d, 0.0, 0.5 * q, 1.3 * q);
    let autumn = smoothstep(1.85 * q, 2.6 * q, d) * (1.0 - smoothstep(2.9 * q, 3.25 * q, d));
    let winter = smoothstep(2.9 * q, 3.3 * q, d) * (1.0 - smoothstep(WINTER_ENDS * q, 4.0 * q, d));
    let pick = |look: (u32, u32)| if part == Part::Grass { look.0 } else { look.1 };
    let amount = |a: (f32, f32)| if part == Part::Grass { a.0 } else { a.1 };
    let autumn_look = match part {
        Part::Grass => AUTUMN_GRASS,
        Part::Leaves { autumn } => autumn,
    };
    let mut c = rgb(base);
    c = mix(c, rgb(muted(pick(FRESH))), fresh * amount(FRESH_AMOUNT) * strength);
    c = mix(c, rgb(muted(autumn_look)), autumn * amount(AUTUMN_AMOUNT) * strength);
    c = mix(c, rgb(muted(pick(WINTER))), winter * amount(WINTER_AMOUNT) * strength);
    pack(c)
}

/// How strongly a place's grass follows its rain, 0 to 1: where the year hardly swings in warmth
/// (the tropics) but its seasons' rain does (the llanos: 0.11 to 0.7), the grass greens in the
/// rains and goes to straw in the dry season. Places whose warmth turns them (`strength`) keep that.
pub fn rain_strength(seasons: f32, rain: [f32; 4]) -> f32 {
    let (lo, hi) = rain
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), &r| (l.min(r), h.max(r)));
    if hi <= 0.0 {
        return 0.0;
    }
    smoothstep(0.3, 0.8, (hi - lo) / hi) * (1.0 - strength(seasons))
}

/// The look grass pulls toward in the rains, and in the dry season (a pale straw, not the yellow
/// of the dry biomes' base: the playtest read that as neon).
const RAINY_GRASS: u32 = 0x86ad4e;
const DRY_GRASS: u32 = 0xc4b680;
const RAIN_AMOUNT: (f32, f32) = (0.6, 0.65);

/// The grass color of `base` on `day` of the year where the rain (`rain`, a season's share, spring
/// first) drives the year with `strength` (`rain_strength`): wetness eased between the seasons'
/// middles, green the wetter, straw the drier.
pub fn rain_tint(base: u32, day: f32, rain: [f32; 4], strength: f32) -> u32 {
    let q = YEAR_DAYS as f32 / 4.0;
    let d = day.rem_euclid(YEAR_DAYS as f32);
    // Each season's rain stands at its middle; in between it eases from one to the next.
    let pos = (d / q - 0.5).rem_euclid(4.0);
    let (i, t) = (pos.floor() as usize % 4, pos.fract());
    let t = t * t * (3.0 - 2.0 * t);
    let now = rain[i] + (rain[(i + 1) % 4] - rain[i]) * t;
    let (lo, hi) = rain
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), &r| (l.min(r), h.max(r)));
    let wet = if hi > lo { (now - lo) / (hi - lo) } else { 0.5 };
    let mut c = rgb(base);
    c = mix(c, rgb(muted(RAINY_GRASS)), wet * RAIN_AMOUNT.0 * strength);
    c = mix(c, rgb(muted(DRY_GRASS)), (1.0 - wet) * RAIN_AMOUNT.1 * strength);
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
    fn winter_leaves_are_duller_than_autumns_not_greener() {
        let late_autumn = rgb(tint(OAK, OAK_LEAVES, YEAR_DAYS as f32 * 0.7, 1.0));
        // Winter's middle (its full look runs 0.825 to 0.94 of the year; it greens after, e142b1478).
        let deep_winter = rgb(tint(OAK, OAK_LEAVES, YEAR_DAYS as f32 * 0.875, 1.0));
        // Green no stronger than its red, and darker than the autumn color.
        assert!(deep_winter[1] <= deep_winter[0] * 1.05, "{deep_winter:?}");
        assert!(deep_winter.iter().sum::<f32>() < late_autumn.iter().sum::<f32>(), "{deep_winter:?} {late_autumn:?}");
    }

    #[test]
    fn the_llanos_green_in_the_rains_and_go_straw_when_dry() {
        let llanos = [0.4, 0.7, 0.45, 0.11];
        let s = rain_strength(1.6, llanos);
        assert!(s > 0.8, "{s}");
        let mid_summer = rgb(rain_tint(0xb0aa60, YEAR_DAYS as f32 * 3.0 / 8.0, llanos, s));
        let mid_winter = rgb(rain_tint(0xb0aa60, YEAR_DAYS as f32 * 7.0 / 8.0, llanos, s));
        assert!(mid_summer[1] > mid_summer[0], "the rains are green: {mid_summer:?}");
        assert!(mid_winter[0] >= mid_winter[1] * 0.95, "the dry season is straw: {mid_winter:?}");
        // Kassel's warmth turns it; its even rain doesn't.
        assert!(rain_strength(8.7, [0.34, 0.34, 0.37, 0.41]) < 0.05);
    }

    fn luma(c: [f32; 3]) -> f32 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    fn saturation(c: [f32; 3]) -> f32 {
        let (hi, lo) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
        if hi <= 0.0 { 0.0 } else { (hi - lo) / hi }
    }

    fn hue(c: [f32; 3]) -> f32 {
        let (hi, lo) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
        let d = hi - lo;
        let h = if hi == c[0] {
            (c[1] - c[2]) / d
        } else if hi == c[1] {
            (c[2] - c[0]) / d + 2.0
        } else {
            (c[0] - c[1]) / d + 4.0
        };
        (h * 60.0).rem_euclid(360.0)
    }

    /// "Tints -15%": every tint loses about 15% of its saturation, its brightness and hue kept.
    #[test]
    fn muted_tints_keep_brightness_and_hue() {
        for c in [ENGINE_GRASS, ENGINE_FOLIAGE, LEAVES, OAK, 0xbfb755, 0x6a7039, 0x4d8a3c, 0xa0e060] {
            let (a, b) = (rgb(c), rgb(muted(c)));
            assert!((luma(a) - luma(b)).abs() <= 1.0, "{c:06x}: luma {} -> {}", luma(a), luma(b));
            assert!((hue(a) - hue(b)).abs() <= 3.0, "{c:06x}: hue {} -> {}", hue(a), hue(b));
            let drop = 1.0 - saturation(b) / saturation(a);
            assert!((0.08..=0.25).contains(&drop), "{c:06x}: saturation {} -> {} ({drop})", saturation(a), saturation(b));
        }
    }

    /// The seasons' looks are muted as the biomes' are: a muted biome stays muted all year.
    #[test]
    fn the_seasons_dont_bring_the_colour_back() {
        let base = muted(OAK);
        for day in 0..YEAR_DAYS {
            for part in [Part::Grass, OAK_LEAVES] {
                let c = rgb(tint(base, part, day as f32, 1.0));
                let raw = rgb(tint(OAK, part, day as f32, 1.0));
                // At least 0.01 below (0.0126 at the least, late in the year, when both lean on
                // the same muted season look).
                assert!(saturation(c) < saturation(raw) - 0.01, "day {day}: {c:?} vs {raw:?}");
            }
        }
    }

    /// Every grass and leaf tint a biome registers (base's and the animals' worlds) is `muted`,
    /// and every biome names all four, so none falls back to the engine's full colour.
    #[test]
    fn every_biome_tint_is_muted() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for file in ["../src/worldgen/mod.rs", "../../animals/src/cretaceous.rs"] {
            let src = std::fs::read_to_string(dir.join(file)).unwrap();
            // Comments dropped: a `;` in one would end a biome early.
            let src: String = src
                .lines()
                .map(|l| l.split("//").next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            for biome in src.split("r.biome(&").skip(1) {
                let body = &biome[..biome.find(';').unwrap()];
                let name = &body[..body.find(')').unwrap()];
                for kind in ["grass", "foliage", "&base_api::trees::EVERGREEN", "&base_api::trees::YELLOW_LEAVES"] {
                    let at = body
                        .find(&format!(".tint({kind}, "))
                        .unwrap_or_else(|| panic!("{file}: {name} has no {kind} tint"));
                    let line = body[at..].lines().next().unwrap();
                    assert!(line.contains("muted("), "{file}: {name}'s {kind} tint isn't muted: {line}");
                }
            }
        }
    }

    /// Every world starts on spring's first day: its leaves are green then, not winter's.
    #[test]
    fn spring_starts_green() {
        for day in [0.0, 0.25, 1.0] {
            let c = rgb(tint(OAK, OAK_LEAVES, day, 1.0));
            assert!(c[1] > c[0] * 1.3, "day {day}: {c:?}");
        }
    }

    #[test]
    fn the_year_wraps_without_a_jump() {
        let end = rgb(tint(OAK, Part::Grass, YEAR_DAYS as f32 - 0.01, 1.0));
        let start = rgb(tint(OAK, Part::Grass, 0.0, 1.0));
        assert!((0..3).all(|i| (end[i] - start[i]).abs() < 2.0), "{end:?} {start:?}");
    }
}

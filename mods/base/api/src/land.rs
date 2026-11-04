//! The overworld's land as numbers and integer math (`reports/world-looks.md`, change 5): what
//! base's terrain is made of, so the registration (`mods/base/src/land.rs`), the map preview
//! (`cargo run -p base_api --example land_map`) and any other mod read one source.
//!
//! The engine sums the features; two of ours are mods' code (`Feature::new`) because they need to
//! know the land around them:
//! - **Rivers** cut valleys down to the sea along the zero lines of a wandering noise, wider and
//!   deeper toward the coast, fading out far inland and in the highlands (where they rise), and
//!   rare where it's dry. A river's water is the sea's: its bed is under sea level all the way to
//!   the coast, so it fills where the sea does. To know how deep to cut it rebuilds the height of
//!   every other feature here (`land_height`), the same integers the engine adds.
//! - **Mesas** in the hot, dry land: flat tops on two or three tiers, each a steep cliff and a
//!   ledge, the cliffs bare (`ROCK_FALL`).
//!
//! Snow lies above a snow line that falls with the climate's temperature (`SNOW_LINES`), lower in
//! drifts (`SNOW_DRIFT`); steep faces are bare rock in patches of stone and cobbles.
//!
//! Positions are in 1/16 block (Q4) and noise values in 1/4096 (`ONE`), as in
//! `weft_noise::fixed`; nothing here uses floats at run time.

use std::cell::Cell;

use weft_sdk::noise::fixed::{self, Fields, ONE, div_floor, q4, q12, ramp, shape};
use weft_sdk::terrain::{Feature, Mask, preset};

use crate::SEA;

/// Salts of our own noises (the engine's are in `weft_noise::fixed::salt`).
pub mod salt {
    pub const RIVERS: u64 = 0x21FE5;
    pub const RIVER_WARP_X: u64 = 0x21FE6;
    pub const RIVER_WARP_Z: u64 = 0x21FE7;
    pub const MESAS: u64 = 0x3E5A;
    pub const MESA_AREA: u64 = 0x3E5B;
    pub const HIGHLANDS: u64 = 0x416A;
    pub const SNOW_DRIFT: u64 = 0x5A0D;
    pub const MOUNTAIN_AREA: u64 = 0x44;
}

/// One factor of where a feature applies: 0 at `from`, 1 at `to` (Q12 after `q12`).
#[derive(Clone, Copy, Debug)]
pub enum Ramp {
    /// A noise of its own, patches about `scale` blocks across (3 octaves, as `Mask::noise`).
    Noise {
        salt: u64,
        scale: f64,
        from: f64,
        to: f64,
    },
    Continent(f64, f64),
    Temperature(f64, f64),
    Humidity(f64, f64),
}

/// Where a feature applies: the product of its ramps, turned around with `not`. The engine's
/// `Mask` and `value` give the same integers.
#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub all: &'static [Ramp],
    pub not: bool,
}

/// A column (`x`, `z` in Q4) and its climate fields, each worked out once when first asked: the
/// features ask for them again and again.
pub struct Here<'a> {
    pub f: &'a Fields,
    pub x: i32,
    pub z: i32,
    fields: [Cell<Option<i32>>; 3],
}

impl<'a> Here<'a> {
    pub fn new(f: &'a Fields, x: i32, z: i32) -> Here<'a> {
        Here { f, x, z, fields: Default::default() }
    }

    fn field(&self, i: usize, get: impl FnOnce() -> i32) -> i32 {
        self.fields[i].get().unwrap_or_else(|| {
            let v = get();
            self.fields[i].set(Some(v));
            v
        })
    }

    pub fn continent(&self) -> i32 {
        self.field(0, || self.f.continent(self.x, self.z))
    }

    pub fn temperature(&self) -> i32 {
        self.field(1, || self.f.temperature(self.x, self.z))
    }

    pub fn humidity(&self) -> i32 {
        self.field(2, || self.f.humidity(self.x, self.z))
    }
}

impl Ramp {
    fn value(&self, h: &Here) -> i32 {
        match *self {
            Ramp::Noise { salt, scale, from, to } => ramp(h.f.fbm(salt, h.x, h.z, q4(scale), 3), q12(from), q12(to)),
            Ramp::Continent(a, b) => ramp(h.continent(), q12(a), q12(b)),
            Ramp::Temperature(a, b) => ramp(h.temperature(), q12(a), q12(b)),
            Ramp::Humidity(a, b) => ramp(h.humidity(), q12(a), q12(b)),
        }
    }

    fn mask(&self) -> Mask {
        match *self {
            Ramp::Noise { salt, scale, from, to } => Mask::noise(scale, from, to).salt(salt),
            Ramp::Continent(a, b) => Mask::Continent(a, b),
            Ramp::Temperature(a, b) => Mask::Temperature(a, b),
            Ramp::Humidity(a, b) => Mask::Humidity(a, b),
        }
    }
}

impl Area {
    /// Its strength at (`x`, `z`) (Q4), 0 to `ONE`, as the engine computes the mask.
    pub fn value(&self, f: &Fields, x: i32, z: i32) -> i32 {
        self.at(&Here::new(f, x, z))
    }

    pub fn at(&self, h: &Here) -> i32 {
        // The engine multiplies every factor; once one is 0 the rest can't change that.
        let v = match self.all {
            [one] => one.value(h),
            all => all
                .iter()
                .fold(ONE, |acc, r| if acc == 0 { 0 } else { (acc * r.value(h)) >> 12 }),
        };
        if self.not { ONE - v } else { v }
    }

    /// The engine's mask for it.
    pub fn mask(&self) -> Mask {
        let mut it = self.all.iter();
        let first = it.next().expect("an area has a ramp").mask();
        let m = match self.all.len() {
            1 => first,
            _ => it.fold(first, |m, r| m.and(r.mask())),
        };
        if self.not { m.not() } else { m }
    }
}

/// Where the biomes stand in the climate fields. The temperature and humidity noises spread
/// about ±0.28 over nine tenths of the world (their deviation is 0.17, far inside the ±1.4 they
/// could reach), so every threshold sits inside that and each biome turns up within a few
/// kilometres (`biome_shares` is tested). The biomes take them in `mods/base/src/worldgen.rs`,
/// tried in the order of `overworld_biome`; the features' climate ramps below match them.
/// Snowy plains colder than this, taiga colder than `TAIGA_BELOW`.
pub const SNOWY_BELOW: f64 = -0.22;
pub const TAIGA_BELOW: f64 = -0.12;
/// Desert from this temperature and under this humidity; savanna likewise, after it.
pub const DESERT: (f64, f64) = (0.14, 0.0);
pub const SAVANNA: (f64, f64) = (0.1, 0.1);
/// Swamp from this temperature and this humidity, on low land.
pub const SWAMP: (f64, f64) = (0.0, 0.04);
/// Forest from this humidity; birch forest under this temperature and from this humidity.
pub const FOREST_HUMIDITY: f64 = 0.07;
pub const BIRCH: (f64, f64) = (0.05, 0.0);

/// Oceans and land masses, this many blocks above or below the sea.
pub const CONTINENTS: f64 = 18.0;

/// Rolling hills (height, size), flattened in the cold into tundra.
pub const HILLS: (f64, f64) = (16.0, 110.0);
pub const HILLS_AREA: Area = Area { all: &[Ramp::Temperature(-0.15, -0.26)], not: true };

/// Highlands: the land swells up to this many blocks in broad patches inland, and the hills roll
/// on top, so the temperate lands stand some 30 blocks over the sea in places; not in the cold
/// (tundra) nor the hot and dry (mesas there).
pub const HIGHLANDS: f64 = 24.0;
pub const HIGHLANDS_AREA: Area = Area {
    all: &[
        Ramp::Noise { salt: salt::HIGHLANDS, scale: 900.0, from: -0.05, to: 0.3 },
        Ramp::Continent(0.08, 0.3),
        Ramp::Temperature(-0.26, -0.15),
        Ramp::Humidity(-0.12, 0.0),
    ],
    not: false,
};

/// Sand dunes (height, wavelength) in the hot, dry land.
pub const DUNES: (f64, f64) = (4.0, 26.0);
pub const DUNES_AREA: Area = Area { all: &[Ramp::Temperature(0.12, 0.22), Ramp::Humidity(0.0, -0.08)], not: false };

/// Mountain ranges (height, width between ridge lines), in patches inland; the mountains biome
/// stands where this area is 0.45 or more (`MOUNTAIN_BIOME`).
pub const MOUNTAINS: (f64, f64) = (150.0, 260.0);
pub const MOUNTAINS_AREA: Area =
    Area { all: &[Ramp::Noise { salt: salt::MOUNTAIN_AREA, scale: 700.0, from: -0.1, to: 0.3 }, Ramp::Continent(0.0, 0.2)], not: false };
pub const MOUNTAIN_BIOME: f64 = 0.45;

/// Mesas: flat-topped hills `MESA_HEIGHT` blocks high in `MESA_TIERS` tiers, patches about
/// `MESA_SIZE` blocks across where the mesa noise runs from `MESA_FROM` to `MESA_TO`; in each
/// tier the first `MESA_CLIFF` of the climb is the cliff, the rest the ledge.
pub const MESA_HEIGHT: f64 = 34.0;
pub const MESA_TIERS: i32 = 3;
pub const MESA_SIZE: f64 = 170.0;
pub const MESA_FROM: f64 = 0.04;
pub const MESA_TO: f64 = 0.24;
pub const MESA_CLIFF: f64 = 0.12;
/// Where mesas rise: hot and dry, in patches (the desert and the dry savanna).
pub const MESA_AREA: Area = Area {
    all: &[
        Ramp::Temperature(0.08, 0.18),
        Ramp::Humidity(0.06, -0.04),
        Ramp::Noise { salt: salt::MESA_AREA, scale: 520.0, from: -0.15, to: 0.1 },
    ],
    not: false,
};

/// Rivers: the line noise's lattice and its wander, in blocks.
pub const RIVER_SCALE: f64 = 520.0;
pub const RIVER_WARP: f64 = 56.0;
pub const RIVER_WARP_SCALE: f64 = 170.0;
/// The channel's half width in line-noise units (about 0.001 a block), far inland and at the
/// coast, and its depth in blocks under the sea at the middle (inland, at the coast).
pub const RIVER_HALF_WIDTH: (f64, f64) = (0.0045, 0.014);
pub const RIVER_DEPTH: (i32, i32) = (3, 5);
/// The coast for rivers: continentalness falling from inland (`.0`) to the coast (`.1`).
pub const RIVER_COAST: (f64, f64) = (0.35, 0.05);
/// Rivers fade out far inland, where the continent noise rises from `.0` to `.1`.
pub const RIVER_INLAND: (f64, f64) = (0.45, 0.62);
/// No rivers where it's this dry (humidity), all of them from `.1` on.
pub const RIVER_DRY: (f64, f64) = (-0.25, -0.08);
/// Rivers rise in the mountains: full where the mountains' area (`MOUNTAINS_AREA`) is up to `.0`,
/// gone at `.1`; through lower hills and mesas they cut their valleys.
pub const RIVER_MOUNTAINS: (f64, f64) = (0.1, 0.45);
/// Where rivers fade (far inland, the dry, the mountains) they narrow to a creek, still at the
/// sea's level; only under this share of their full width does the creek rise and end.
pub const RIVER_CREEK: f64 = 0.6;
/// The valley beside the channel: a floodplain this wide (line-noise units), then banks rising to
/// the land, wider the higher it stands: `RIVER_VALLEY_PER_BLOCK` more (in 1/4096 of the noise)
/// for each block of the land over the sea, counted up to `RIVER_VALLEY_RISE`.
pub const RIVER_FLOODPLAIN: f64 = 0.006;
pub const RIVER_VALLEY_PER_BLOCK: i32 = 6;
pub const RIVER_VALLEY_RISE: i32 = 80;

/// Water deeper than a block is the ocean's where continentalness is under this, out to the
/// coast; inland, a river's or a lake's water belongs to the land around it (its climate).
pub const OCEAN_CONTINENT: f64 = 0.1;
/// The land's biome rules in the order they're tried, after the coasts and the mountains, with
/// the bed under their water: gravel in the north and the birch woods, sand in the warm lands,
/// mud (dirt) in the swamp.
pub const RIVER_BEDS: [(&str, &str); 8] = [
    ("snowy_plains", "gravel"),
    ("taiga", "gravel"),
    ("desert", "sand"),
    ("savanna", "sand"),
    ("swamp", "dirt"),
    ("forest", "sand"),
    ("birch_forest", "gravel"),
    ("plains", "sand"),
];

/// Snow lies above these heights, by the climate's temperature: (below this temperature, the
/// line), coldest first; the last band holds for everything warmer.
pub const SNOW_LINES: [(f64, i32); 3] = [(-0.15, 146), (0.12, 162), (f64::INFINITY, 180)];
/// In drifts the snow comes this many blocks lower: patches about `SNOW_DRIFT_SIZE` blocks
/// across, where the drift noise is over `SNOW_DRIFT_FROM`.
pub const SNOW_DRIFT: i32 = 9;
pub const SNOW_DRIFT_SIZE: f64 = 40.0;
pub const SNOW_DRIFT_FROM: f64 = 0.5;
/// The snow lies this deep over the rock.
pub const SNOW_DEPTH: i32 = 2;

/// A column whose ground falls this many blocks to a neighbour is a rock face: its soil is bare
/// rock as far down as the face shows (its fall), at least `ROCK_DEPTH` and at most
/// `ROCK_MAX_DEPTH` blocks; on sand the whole face is sandstone (the mesas). Snow slides off at
/// `SNOW_FALL`.
pub const ROCK_FALL: i32 = 4;
pub const SNOW_FALL: i32 = 3;
pub const ROCK_DEPTH: i32 = 5;
pub const ROCK_MAX_DEPTH: i32 = 16;
/// What rock a face shows, by its column's climate and height (`rock_set`): the hot, dry land's
/// strata (`STRATA`), slate in the cold (under `TAIGA_BELOW`), granite on ground this high over
/// the sea or in the mountains, limestone on the lower cliffs and river banks.
pub const GRANITE_ABOVE: i32 = 40;
/// In patches about `ROCK_PATCH` blocks across: where the patch noise is over `ROCK_PATCHES` the
/// face shows its second rock (stone in slate and limestone, cobbles in granite), under
/// `-ROCK_MOSS` mossy cobbles in the woods and the wet (limestone) or stone (granite).
pub const ROCK_PATCH: f64 = 9.0;
pub const ROCK_PATCHES: f64 = 0.22;
pub const ROCK_MOSS: f64 = 0.25;
/// The mesas' strata from the bottom up, repeating: each layer's rock, 1 to 3 blocks thick by the
/// world's seed; the layers tilt gently, up to `STRATA_TILT` blocks over some 300.
pub const STRATA: [&str; 6] = ["sandstone", "ochre_sandstone", "sandstone", "red_sandstone", "ochre_sandstone", "red_sandstone"];
pub const STRATA_TILT: i32 = 3;

/// The engine's features of the land, in the order they're added; `mesas` and `rivers` go after
/// them (`land_height`, `rivers`).
pub fn features() -> Vec<Feature> {
    vec![
        preset::continents(CONTINENTS),
        preset::hills(HILLS.0, HILLS.1).mask(HILLS_AREA.mask()),
        Feature::new(|_, _, _| q4(HIGHLANDS))
            .mask(HIGHLANDS_AREA.mask())
            .named("highlands"),
        preset::dunes(DUNES.0, DUNES.1).mask(DUNES_AREA.mask()),
        preset::mountains(MOUNTAINS.0, MOUNTAINS.1)
            .mask(MOUNTAINS_AREA.mask())
            .named("mountains"),
        Feature::new(mesa).named("mesas"),
        Feature::new(rivers).named("rivers"),
        // No height: its mask is where snow drifts (the snow rules ask for it).
        preset::hills(0.0, 64.0)
            .mask(Mask::noise(SNOW_DRIFT_SIZE, -0.2, 0.2).salt(salt::SNOW_DRIFT))
            .named("snow_drift"),
    ]
}

/// A preset's height times its area's strength, as the engine adds it.
fn masked(h: &Here, area: &Area, height: impl FnOnce() -> i32) -> i32 {
    let w = area.at(h);
    if w > 0 { (w * height()) >> 12 } else { 0 }
}

/// The ground's height (Q4) at (`x`, `z`) (Q4) before the rivers: the sea level and every feature
/// in `features` before `rivers`, the same integers the engine sums.
pub fn land_height(f: &Fields, x: i32, z: i32) -> i32 {
    land_height_at(&Here::new(f, x, z))
}

fn land_height_at(h: &Here) -> i32 {
    let (f, x, z) = (h.f, h.x, h.z);
    q4(SEA as f64)
        + shape::continents(f, x, z, q4(CONTINENTS))
        + masked(h, &HILLS_AREA, || shape::hills(f, x, z, q4(HILLS.0), q4(HILLS.1)))
        + masked(h, &HIGHLANDS_AREA, || q4(HIGHLANDS))
        + masked(h, &DUNES_AREA, || shape::dunes(f, x, z, q4(DUNES.0), q4(DUNES.1)))
        + masked(h, &MOUNTAINS_AREA, || shape::mountains(f, x, z, q4(MOUNTAINS.0), q4(MOUNTAINS.1)))
        + mesa_at(h)
}

/// The ground's height (Q4) with the rivers: what the engine makes of `features`.
pub fn height(f: &Fields, x: i32, z: i32) -> i32 {
    land_height(f, x, z) + rivers(f, x, z)
}

/// The mesas' height (Q4) at (`x`, `z`).
pub fn mesa(f: &Fields, x: i32, z: i32) -> i32 {
    mesa_at(&Here::new(f, x, z))
}

fn mesa_at(h: &Here) -> i32 {
    let (f, x, z) = (h.f, h.x, h.z);
    let w = MESA_AREA.at(h);
    if w == 0 {
        return 0;
    }
    let n = ramp(f.fbm(salt::MESAS, x, z, q4(MESA_SIZE), 4), q12(MESA_FROM), q12(MESA_TO));
    if n == 0 {
        return 0;
    }
    let v = (((n * w) >> 12) * q4(MESA_HEIGHT)) >> 12;
    let step = (q4(MESA_HEIGHT) / MESA_TIERS).max(1);
    let (tier, climb) = div_floor(v, step);
    tier * step + (climb * ONE / q12(MESA_CLIFF).max(1)).min(step)
}

/// Where a river runs: the line noise's distance from its zero line (Q12), at the wandered point.
pub fn river_line(f: &Fields, x: i32, z: i32) -> i32 {
    let (warp, ws) = (q4(RIVER_WARP), q4(RIVER_WARP_SCALE));
    let wx = x + ((f.fbm(salt::RIVER_WARP_X, x, z, ws, 2) * warp) >> 12);
    let wz = z + ((f.fbm(salt::RIVER_WARP_Z, x, z, ws, 2) * warp) >> 12);
    f.fbm(salt::RIVERS, wx, wz, q4(RIVER_SCALE), 3).abs()
}

/// What the rivers take off the ground (Q4, zero or less) at (`x`, `z`).
pub fn rivers(f: &Fields, x: i32, z: i32) -> i32 {
    let d = river_line(f, x, z);
    let widest = q12(RIVER_HALF_WIDTH.1 + RIVER_FLOODPLAIN) + RIVER_VALLEY_PER_BLOCK * RIVER_VALLEY_RISE;
    if d >= widest {
        return 0;
    }
    let here = Here::new(f, x, z);
    let cont = here.continent();
    let inland = ONE - ramp(cont, q12(RIVER_INLAND.0), q12(RIVER_INLAND.1));
    let wet = ramp(here.humidity(), q12(RIVER_DRY.0), q12(RIVER_DRY.1));
    let s = (inland * wet) >> 12;
    if s == 0 {
        return 0;
    }
    let s = (s * (ONE - ramp(MOUNTAINS_AREA.at(&here), q12(RIVER_MOUNTAINS.0), q12(RIVER_MOUNTAINS.1)))) >> 12;
    if s == 0 {
        return 0;
    }
    let h0 = land_height_at(&here);
    let sea = q4(SEA as f64);
    let rise = (h0 - sea) >> 4;
    if rise < 0 {
        return 0;
    }
    let coast = ramp(cont, q12(RIVER_COAST.0), q12(RIVER_COAST.1));
    let lerp = |a: i32, b: i32, t: i32| a + (((b - a) * t) >> 12);
    // A fading river narrows first (`s`), then its creek rises out of the sea's level (`deep`).
    let deep = ramp(s, 0, q12(RIVER_CREEK));
    let half = ((lerp(q12(RIVER_HALF_WIDTH.0), q12(RIVER_HALF_WIDTH.1), coast) * s) >> 12).max(1);
    let valley = half + (((q12(RIVER_FLOODPLAIN) + RIVER_VALLEY_PER_BLOCK * rise.min(RIVER_VALLEY_RISE)) * deep) >> 12);
    if d >= valley {
        return 0;
    }
    let target = if d < half {
        // The channel: a block under the sea at its edge, `depth` in the middle.
        let depth = lerp(q4(RIVER_DEPTH.0 as f64), q4(RIVER_DEPTH.1 as f64), coast);
        let t = d * ONE / half;
        let bowl = ONE - ((t * t) >> 12);
        sea - 16 - (((depth - 16) * bowl) >> 12)
    } else {
        // The banks: level with the water by the channel, then rising smoothly to the land.
        let t = (d - half) * ONE / (valley - half).max(1);
        let smooth = (((t * t) >> 12) * (3 * ONE - 2 * t)) >> 12;
        sea + (((h0 - sea) * smooth) >> 12)
    };
    (((target - h0).min(0)) * deep) >> 12
}

/// The snow line (blocks) at temperature `t` (Q12): the band's height from `SNOW_LINES`.
pub fn snow_line(t: i32) -> i32 {
    SNOW_LINES
        .iter()
        .find(|(below, _)| t < q12(*below))
        .map_or(SNOW_LINES[SNOW_LINES.len() - 1].1, |b| b.1)
}

/// Whether snow lies on ground at `height` (blocks) at (`x`, `z`) (Q4), as the snow rules decide
/// (`mods/base/src/land.rs`): above the snow line, or `SNOW_DRIFT` lower in a drift.
pub fn snowy(f: &Fields, x: i32, z: i32, height: i32) -> bool {
    let line = snow_line(f.temperature(x, z));
    let drift = ramp(f.fbm(salt::SNOW_DRIFT, x, z, q4(SNOW_DRIFT_SIZE), 3), q12(-0.2), q12(0.2));
    height >= line || (height >= line - SNOW_DRIFT && drift > q12(SNOW_DRIFT_FROM))
}

/// The rocks a column's face shows (`rock_set`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RockSet {
    Strata,
    Slate,
    Granite,
    Limestone,
}

/// The rocks of the face of column (`x`, `z`) (Q4), its ground `height` (blocks) high; `sandy`
/// faces (sand on top) are strata wherever they are.
pub fn rock_set(f: &Fields, x: i32, z: i32, height: i32, sandy: bool) -> RockSet {
    let here = Here::new(f, x, z);
    let (t, hu) = (here.temperature(), here.humidity());
    if sandy || t >= q12(SAVANNA.0) && hu < q12(SAVANNA.1) {
        RockSet::Strata
    } else if t < q12(TAIGA_BELOW) {
        RockSet::Slate
    } else if height >= SEA + GRANITE_ABOVE || MOUNTAINS_AREA.at(&here) > q12(MOUNTAIN_BIOME) {
        RockSet::Granite
    } else {
        RockSet::Limestone
    }
}

impl RockSet {
    /// The block (its name in base) at `pos` (world blocks) of a face of these rocks; mossy cobbles
    /// only where `mossy`.
    pub fn block(self, seed: u64, [x, y, z]: [i32; 3], mossy: bool) -> &'static str {
        if self == RockSet::Strata {
            return strata(seed, [x, y, z]);
        }
        let s = q4(ROCK_PATCH);
        let v = fixed::perlin3(fixed::seed32(seed, 0x50C4), x * 16, y * 16, z * 16, s, s / 2, s);
        let (patch, low) = (v > q12(ROCK_PATCHES), v < -q12(ROCK_MOSS));
        match self {
            RockSet::Slate if patch => "stone",
            RockSet::Slate => "slate",
            RockSet::Granite if patch => "cobblestone",
            RockSet::Granite if low => "stone",
            RockSet::Granite => "granite",
            RockSet::Limestone if patch => "stone",
            RockSet::Limestone if low && mossy => "mossy_cobblestone",
            _ => "limestone",
        }
    }
}

/// The mesas' layer at `pos` (world blocks): `STRATA` repeating up the height, each layer 1 to 3
/// blocks thick by the seed, the whole gently tilted.
pub fn strata(seed: u64, [x, y, z]: [i32; 3]) -> &'static str {
    let s = fixed::seed32(seed, 0x57A7);
    let thick = |i: usize| 1 + (fixed::hash(s, i as i32, 0, 0) % 3) as i32;
    let period: i32 = (0..STRATA.len()).map(thick).sum();
    let tilt = (fixed::perlin2(s ^ 1, x * 16, z * 16, 300 * 16) * STRATA_TILT) >> 12;
    let mut at = (y + tilt).rem_euclid(period);
    for (i, rock) in STRATA.iter().enumerate() {
        at -= thick(i);
        if at < 0 {
            return rock;
        }
    }
    STRATA[0]
}

/// The overworld biome the rules in `mods/base/src/land.rs` pick at (`x`, `z`) (Q4), by name, in
/// their order: for the map preview and the checks here (the engine's own pick is the truth).
pub fn overworld_biome(f: &Fields, x: i32, z: i32) -> &'static str {
    let here = Here::new(f, x, z);
    let h = height(f, x, z) >> 4;
    let (t, hu, c) = (here.temperature(), here.humidity(), here.continent());
    let at = |v: f64| q12(v);
    if h <= SEA - 2 && c < at(OCEAN_CONTINENT) {
        "ocean"
    } else if h <= SEA + 1 && c < at(0.1) {
        if t < at(SNOWY_BELOW) { "snowy_plains" } else { "beach" }
    } else if MOUNTAINS_AREA.at(&here) > at(MOUNTAIN_BIOME) || snowy(f, x, z, h) {
        "mountains"
    } else if t < at(SNOWY_BELOW) {
        "snowy_plains"
    } else if t < at(TAIGA_BELOW) {
        "taiga"
    } else if t >= at(DESERT.0) && hu < at(DESERT.1) {
        "desert"
    } else if t >= at(SAVANNA.0) && hu < at(SAVANNA.1) {
        "savanna"
    } else if t >= at(SWAMP.0) && hu >= at(SWAMP.1) && h <= SEA + 12 && c >= at(0.1) {
        "swamp"
    } else if hu >= at(FOREST_HUMIDITY) {
        "forest"
    } else if t < at(BIRCH.0) && hu >= at(BIRCH.1) {
        "birch_forest"
    } else {
        "plains"
    }
}

/// Each overworld biome's share of the land (not the sea), sampled every `step` blocks out to
/// `radius` from the middle, and how near the middle it first turns up (blocks, the larger of
/// x and z).
pub fn biome_shares(seed: u64, radius: i32, step: usize) -> Vec<(&'static str, f64, i32)> {
    let f = Fields { seed };
    let mut n: Vec<(&'static str, usize, i32)> = Vec::new();
    let mut land = 0;
    for z in (-radius..=radius).step_by(step) {
        for x in (-radius..=radius).step_by(step) {
            let b = overworld_biome(&f, x * 16, z * 16);
            if b == "ocean" {
                continue;
            }
            land += 1;
            let d = x.abs().max(z.abs());
            match n.iter_mut().find(|e| e.0 == b) {
                Some(e) => (e.1, e.2) = (e.1 + 1, e.2.min(d)),
                None => n.push((b, 1, d)),
            }
        }
    }
    n.sort();
    n.into_iter()
        .map(|(b, c, d)| (b, c as f64 / land.max(1) as f64, d))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rivers_only_cut_and_reach_under_the_sea() {
        let f = Fields { seed: 7 };
        let mut wet = 0;
        for i in 0..20_000 {
            let (x, z) = ((i % 200) * 16 * 16, (i / 200) * 16 * 16);
            let r = rivers(&f, x, z);
            assert!(r <= 0);
            if r < 0 && height(&f, x, z) >> 4 < SEA {
                wet += 1;
            }
        }
        assert!(wet > 0, "no river reached under the sea in 20 000 columns");
    }

    #[test]
    fn mesas_are_tiers_no_higher_than_their_height() {
        let f = Fields { seed: 3 };
        for i in 0..20_000 {
            let (x, z) = ((i % 200) * 8 * 16, (i / 200) * 8 * 16);
            let m = mesa(&f, x, z);
            assert!((0..=q4(MESA_HEIGHT)).contains(&m));
        }
    }

    #[test]
    fn strata_are_layers_of_their_rocks() {
        let rocks: Vec<&str> = (0..40).map(|y| strata(5, [0, y, 0])).collect();
        assert!(rocks.iter().all(|r| STRATA.contains(r)));
        assert!(STRATA.iter().all(|r| rocks.contains(r)), "{rocks:?}");
    }

    #[test]
    fn snow_lines_fall_with_the_cold() {
        assert!(
            SNOW_LINES
                .windows(2)
                .all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1)
        );
        assert_eq!(snow_line(q12(-0.5)), SNOW_LINES[0].1);
        assert_eq!(snow_line(q12(0.9)), SNOW_LINES[2].1);
    }

    #[test]
    fn every_biome_turns_up_near_the_middle() {
        for seed in [1, 2, 3] {
            let shares = biome_shares(seed, 6000, 48);
            for biome in ["plains", "forest", "birch_forest", "taiga", "snowy_plains", "desert", "savanna", "swamp", "mountains", "beach"] {
                let (share, near) = shares
                    .iter()
                    .find(|e| e.0 == biome)
                    .map_or((0.0, i32::MAX), |e| (e.1, e.2));
                assert!(share >= 0.01, "seed {seed}: {biome} is {:.1}% of the land: {shares:?}", share * 100.0);
                assert!(near <= 4000, "seed {seed}: the nearest {biome} is {near} blocks out");
            }
        }
    }
}

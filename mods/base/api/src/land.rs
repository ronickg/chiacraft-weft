//! The overworld's land as numbers and integer math (`reports/world-looks.md`, change 5): what
//! base's terrain is made of, so the registration (`mods/base/src/land.rs`), the map preview
//! (`cargo run -p base_api --example land_map`) and any other mod read one source.
//!
//! The engine sums the features; two of ours are mods' code (`Feature::new`) because they need to
//! know the land around them:
//! - **Rivers** run where the rain gathers: a network of nodes, each draining downhill to a
//!   neighbour (`net`), so streams meet into rivers that grow toward the sea or a lake, and end
//!   in a lake of their own where they find no way out. Where the flow is small the river is a
//!   dry gully; with more it reaches the water and widens, with a floodplain and valley sides that
//!   climb to the land. A river's water is the sea's: its bed is under sea level, so it fills
//!   where the sea does. To know how deep to cut it rebuilds the height of every other feature
//!   here (`land_height`), the same integers the engine adds.
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

mod canyons;
mod craters;
mod karst;
pub mod landmarks;
mod net;

/// Salts of our own noises (the engine's are in `weft_noise::fixed::salt`).
pub mod salt {
    pub const RIVERS: u64 = 0x21FE5;
    pub const LAKES: u64 = 0x1A4E;
    pub const LAKE_SHORE: u64 = 0x1A4F;
    pub const LAKE_ISLANDS: u64 = 0x1A50;
    pub const RIVER_WARP_X: u64 = 0x21FE6;
    pub const RIVER_WARP_Z: u64 = 0x21FE7;
    pub const RIVER_NODES: u64 = 0x21FE8;
    pub const MESAS: u64 = 0x3E5A;
    pub const MESA_AREA: u64 = 0x3E5B;
    pub const HIGHLANDS: u64 = 0x416A;
    pub const SNOW_DRIFT: u64 = 0x5A0D;
    pub const MOUNTAIN_AREA: u64 = 0x44;
    pub const SEQUOIA: u64 = 0x5E0A;
    pub const COAST_CLIFFS: u64 = 0xC1F;
    pub const ISLANDS: u64 = 0x151A;
    pub const ARCHIPELAGOS: u64 = 0x151B;
    pub const KARST: u64 = 0x4A5;
    pub const KARST_AREA: u64 = 0x4A6;
    pub const KARST_WOBBLE: u64 = 0x4A7;
    pub const KARST_CLUSTER: u64 = 0x4A8;
    pub const ARCHES: u64 = 0xA2C;
    pub const FALLS: u64 = 0xFA11;
    pub const SINKS: u64 = 0x4A9;
    pub const CRATERS: u64 = 0xC4A7;
    pub const FACE_COAL: u64 = 0x0C0A;
    pub const FACE_IRONSTONE: u64 = 0x0F3A;
    pub const FACE_MAGNETITE: u64 = 0x0F3B;
    pub const FACE_GOLD: u64 = 0x0A0B;
    pub const FRAY: u64 = 0xF4A;
    pub const CANYONS: u64 = 0xCA1;
    pub const CANYON_AREA: u64 = 0xCA2;
    pub const CANYON_WARP: u64 = 0xCA3;
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

/// Giant sequoia groves (Ronald, 11-04: "the massive Giant sequoia"): the Sierra Nevada's west
/// slope, groves of 1-1,000 ha among the mixed conifers at 1,400-2,100 m, where the winters are
/// cool and snowy and the summers dry, in moist draws with deep soil (Giant Forest's climate,
/// `vitals::climate`). Here: temperate (`SEQUOIA_TEMPERATURE`), not dry (`SEQUOIA_HUMIDITY` and
/// up), on the mountains' front slopes `SEQUOIA_RISE` blocks over the sea, in grove patches:
/// where `SEQUOIA_AREA` is over `SEQUOIA_GROVE`. Its ramps are the grove noise's patches (some
/// 260 blocks across), the mountains' own area noise just short of the mountains biome (their
/// foothills), and inland.
pub const SEQUOIA_TEMPERATURE: (f64, f64) = (-0.12, 0.1);
pub const SEQUOIA_HUMIDITY: f64 = 0.0;
pub const SEQUOIA_RISE: (i32, i32) = (28, 85);
pub const SEQUOIA_GROVE: f64 = 0.5;
pub const SEQUOIA_AREA: Area = Area {
    all: &[
        Ramp::Noise { salt: salt::SEQUOIA, scale: 260.0, from: 0.02, to: 0.15 },
        Ramp::Noise { salt: salt::MOUNTAIN_AREA, scale: 700.0, from: -0.08, to: -0.03 },
        Ramp::Noise { salt: salt::MOUNTAIN_AREA, scale: 700.0, from: 0.08, to: 0.05 },
        Ramp::Continent(0.15, 0.25),
    ],
    not: false,
};

/// Oceans and land masses, this many blocks above or below the sea.
pub const CONTINENTS: f64 = 18.0;
/// How many blocks across the overworld's land masses and seas are: the continent noise's scale
/// (the engine's default is 900). At 1800 (Ronald, 11-04) the seas join into real seas between
/// bigger land and rivers run longer.
pub const CONTINENT_SCALE: f64 = 1800.0;

/// The overworld's noise fields for world `seed` (its continent scale): what the engine hands
/// base's features, for everything here that asks the climate outside of them.
pub const fn fields(seed: u64) -> Fields {
    Fields::with_continent_scale(seed, (CONTINENT_SCALE * 16.0) as i32)
}

/// Rolling hills (height, size), flattened in the cold into tundra and calmed by the coast:
/// `COAST_CALM` of them everywhere (`HILLS_AREA`), the rest only inland from the coast
/// (`HILLS_INLAND`), so the land comes down to the sea in a gentle plain with wide beaches
/// instead of ragged hills.
pub const HILLS: (f64, f64) = (16.0, 110.0);
pub const HILLS_AREA: Area = Area { all: &[Ramp::Temperature(-0.15, -0.26)], not: true };
pub const COAST_CALM: f64 = 0.45;
pub const HILLS_INLAND: Area = Area { all: &[Ramp::Temperature(-0.26, -0.15), Ramp::Continent(0.0, 0.12)], not: false };

/// Sea cliffs: in patches along the coast (`COAST_CLIFF_AREA`: the cliff noise's patches, some
/// 650 blocks across, about 11% of the shore on seeds 1-3; Ronald asked for 10-15%; its noise
/// ramp sets the share) the land stands `COAST_CLIFF` blocks higher from the shoreline inland, so
/// it meets the sea in a rock face (rock faces show their stone, limestone low down) and rivers
/// cut gorges through it.
pub const COAST_CLIFF: f64 = 14.0;
pub const COAST_CLIFF_AREA: Area = Area {
    all: &[Ramp::Noise { salt: salt::COAST_CLIFFS, scale: 650.0, from: -0.15, to: -0.05 }, Ramp::Continent(0.0, 0.0025)],
    not: false,
};
/// The deep sea: past the shallows the floor drops `OCEAN_DEEP` blocks more (continentalness
/// `OCEAN_DEEP_AREA`), so the sea shows shallow water by the shore and dark water out deep.
pub const OCEAN_DEEP: f64 = 22.0;
pub const OCEAN_DEEP_AREA: Area = Area { all: &[Ramp::Continent(-0.1, -0.25)], not: false };
/// Islands in groups (Lithosphere's archipelagos): in archipelago patches (`ARCHIPELAGO`, some
/// 900 blocks across) out in the sea past the shallows, the island noise's blobs (some
/// `ISLAND_SIZE` blocks across) lift the sea floor by up to `ISLAND_HEIGHT` blocks.
pub const ISLAND_HEIGHT: f64 = 30.0;
pub const ISLAND_SIZE: f64 = 120.0;
pub const ISLANDS_AREA: Area = Area {
    all: &[
        Ramp::Noise { salt: salt::ISLANDS, scale: ISLAND_SIZE, from: 0.12, to: 0.32 },
        Ramp::Noise { salt: salt::ARCHIPELAGOS, scale: 900.0, from: -0.05, to: 0.1 },
        Ramp::Continent(-0.02, -0.07),
    ],
    not: false,
};

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
/// Not by the sea: they fade out toward the coast (combs of dunes ran into the water).
pub const DUNES_AREA: Area =
    Area { all: &[Ramp::Temperature(0.12, 0.22), Ramp::Humidity(0.0, -0.08), Ramp::Continent(0.01, 0.07)], not: false };

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

/// Badland canyons (Tectonic, Lithosphere; Ronald, 11-04: "All 3, in order"): in the hot, dry
/// land (`CANYON_AREA`; full depth from `CANYON_FULL` of its strength), winding cuts along a
/// warped line noise's zero lines (`CANYON_SCALE` blocks between, wandering by `CANYON_WARP`),
/// `CANYON_WIDTH` of the noise wide, cut `CANYON_DEPTH` blocks down in tiers `CANYON_TIER` high
/// (a steep wall, then a ledge), the floor kept `CANYON_DRY` over the sea. Their walls show the
/// strata (rock faces).
pub const CANYON_AREA: Area = Area {
    all: &[
        Ramp::Temperature(0.1, 0.16),
        Ramp::Humidity(0.02, -0.04),
        Ramp::Noise { salt: salt::CANYON_AREA, scale: 900.0, from: -0.05, to: 0.05 },
        Ramp::Continent(0.06, 0.14),
    ],
    not: false,
};
pub const CANYON_FULL: f64 = 0.3;
pub const CANYON_SCALE: f64 = 700.0;
pub const CANYON_WARP: (f64, f64) = (90.0, 160.0);
pub const CANYON_WIDTH: f64 = 0.05;
pub const CANYON_DEPTH: i32 = 40;
pub const CANYON_TIER: i32 = 8;
pub const CANYON_DRY: i32 = 3;

/// Karst towers (Lithosphere; Ha Long Bay, Guilin): in the warm, wet land and its shallow seas
/// (`KARST_AREA`), steep limestone pillars, at most one per `KARST_CELL` blocks square, in
/// clusters, `KARST_RADIUS` (or `KARST_WIDE_RADIUS`) blocks from their middle to their foot and
/// `KARST_HEIGHT` over the water or the ground they stand on, their outline wobbled (Ronald,
/// 11-04: "All 3, in order").
pub const KARST_AREA: Area = Area {
    all: &[
        Ramp::Temperature(0.02, 0.08),
        Ramp::Humidity(0.06, 0.12),
        Ramp::Noise { salt: salt::KARST_AREA, scale: 700.0, from: 0.0, to: 0.12 },
        Ramp::Continent(-0.12, -0.04),
    ],
    not: false,
};
pub const KARST_CELL: i32 = 32;
/// Towers stand in clusters (Planner, 11-04: "clusters of 3-6 towers with a few wider ones"):
/// where the cluster noise (patches about `KARST_CLUSTER_SIZE` blocks across) runs from
/// `KARST_CLUSTER.0` to `.1` a cell keeps its tower with `KARST_CHANCE`, elsewhere with
/// `KARST_LONE`; both times the karst area's strength.
pub const KARST_CHANCE: f64 = 0.85;
pub const KARST_LONE: f64 = 0.06;
pub const KARST_CLUSTER_SIZE: f64 = 140.0;
pub const KARST_CLUSTER: (f64, f64) = (0.04, 0.14);
/// Radius and height (blocks); one tower in `KARST_WIDE_ONE_IN` is a wide one.
pub const KARST_RADIUS: (i32, i32) = (7, 12);
pub const KARST_HEIGHT: (i32, i32) = (16, 36);
pub const KARST_WIDE_ONE_IN: u32 = 4;
pub const KARST_WIDE_RADIUS: (i32, i32) = (13, 17);
pub const KARST_WIDE_HEIGHT: (i32, i32) = (22, 38);

/// Karst sinkholes (cenotes; Ronald, 11-04: "all four"): in the karst area, round shafts
/// `SINK_RADIUS` blocks from their middle to their wall, at most one per `SINK_CELL` square, kept
/// with `SINK_CHANCE` times the area's strength, down to `SINK_FLOOR` blocks under the sea, so
/// water stands in them at the sea's level; only on land `SINK_LAND` over the sea and
/// `SINK_CLEAR` blocks clear of the towers' feet.
pub const SINK_CELL: i32 = 128;
pub const SINK_CHANCE: f64 = 0.35;
pub const SINK_RADIUS: (i32, i32) = (6, 11);
pub const SINK_FLOOR: i32 = 6;
pub const SINK_LAND: (i32, i32) = (4, 30);
pub const SINK_CLEAR: i32 = 4;

/// Meteor craters (Ronald, 11-04: "all four"): round bowls, at most one per `CRATER_CELL` square,
/// kept with `CRATER_CHANCE` where the land at the middle is open and dry (not the cold, the
/// woods' wet, the mountains or the karst) and over the sea: `CRATER_RADIUS` blocks from the
/// middle to the rim's crest, `CRATER_DEPTH` radii deep, the rim `CRATER_RIM` blocks high and
/// blending back into the land by `CRATER_RIM_OUT` radii; all measured from the ground at the
/// middle, and only where the land round it is level and no river or lake reaches; the last of
/// the features, after the rivers. Where the floor dips under the sea's level
/// water stands in it.
pub const CRATER_CELL: i32 = 640;
pub const CRATER_CHANCE: f64 = 1.0;
pub const CRATER_RADIUS: (i32, i32) = (20, 30);
pub const CRATER_DEPTH: f64 = 0.65;
pub const CRATER_RIM: (i32, i32) = (5, 8);
pub const CRATER_RIM_OUT: f64 = 1.8;

/// Natural arches (Ronald, 11-04: "all four"; `land::landmarks`): at most one per `ARCH_CELL`
/// square, kept with `ARCH_CHANCE` where it fits: in the strata land on level ground, or in the
/// shallow sea off a cliff coast (a sea arch, of the cliffs' rock). A half ring `ARCH_INNER` blocks
/// inside, `ARCH_THICK` thick and `ARCH_DEPTH` deep, its legs down to the ground.
pub const ARCH_CELL: i32 = 512;
pub const ARCH_CHANCE: f64 = 0.6;
pub const ARCH_INNER: (i32, i32) = (5, 9);
pub const ARCH_THICK: (i32, i32) = (3, 4);
pub const ARCH_DEPTH: (i32, i32) = (3, 5);
/// Sea-cliff waterfalls (`land::landmarks`): in one `FALL_CELL` square in `1 / FALL_CHANCE`, the
/// first sea cliff `FALL_CLIFF` or more over the sea in it gets a spring two under its top,
/// falling into the sea.
pub const FALL_CELL: i32 = 256;
pub const FALL_CHANCE: f64 = 0.3;
pub const FALL_CLIFF: i32 = 8;

/// Rivers run where rain gathers (`net`): over nodes `RIVER_NODE` blocks apart, each draining
/// to its steepest neighbour, worked out `RIVER_REGION` nodes square at a time. A node's flow is
/// the rain of the nodes upstream within `RIVER_REACH` steps (about 2 km of river).
pub const RIVER_NODE: i32 = 48;
pub const RIVER_REGION: i32 = 32;
pub const RIVER_REACH: i32 = 40;
/// The height rain runs down is the land's big shape (`net::route_height`): the mountains count
/// as a dome this high, the hills at this share.
pub const RIVER_ROUTE_MOUNTAINS: f64 = 120.0;
pub const RIVER_ROUTE_HILLS: f64 = 0.0;
/// Where a river ends inland with at least this flow, it fills a lake, its radius from `.0` to
/// `.1` blocks as its flow grows to `RIVER_FLOW.2`.
pub const RIVER_SINK_FLOW: f64 = 6.0;
pub const RIVER_SINK_RADIUS: (i32, i32) = (24, 80);
/// A node's rain, by humidity: all of it from `.1` on, falling to the driest's share `.2` at `.0`.
pub const RIVER_RAIN: (f64, f64, f64) = (-0.25, 0.05, 0.15);
/// Flow, in wet nodes' rain: a dry gully starts at `.0`, the river reaches the water at `.1`
/// (between, the gully deepens), and it is full size at `.2`.
pub const RIVER_FLOW: (f64, f64, f64) = (4.0, 9.0, 110.0);
/// The channel's half width and its depth under the sea in the middle, and the floodplain beside
/// it, flush with the water (blocks; the smallest river's, the full one's).
pub const RIVER_HALF_WIDTH: (i32, i32) = (3, 12);
pub const RIVER_DEPTH: (i32, i32) = (3, 6);
pub const RIVER_FLOODPLAIN: (i32, i32) = (2, 12);
/// Then the valley's sides rise to the land over `RIVER_BANKS` blocks, plus
/// `RIVER_VALLEY_PER_BLOCK` (sixteenths) for each block the land stands over the sea, counted up
/// to `RIVER_VALLEY_RISE`. 36, as the lakes' shores (24 left a sixth of the bank in steps of two
/// blocks or more, 36 a ninth; 48 costs a third more time for little more).
pub const RIVER_BANKS: i32 = 8;
pub const RIVER_VALLEY_PER_BLOCK: i32 = 36;
pub const RIVER_VALLEY_RISE: i32 = 60;
/// River terraces (Tectonic's tiers by a river's distance): the valley sides of the bigger
/// rivers (flow from `RIVER_TERRACE_FLOW.0`, full at `.1`) climb in benches `RIVER_TERRACE`
/// blocks high, each flat for all but `RIVER_TERRACE_RISER` of its width: meadows by the water
/// (Ronald, 11-04: "All 3, in order"). Low steps with wide risers so a player can walk up them:
/// at 5 blocks with 0.3 risers the walk test stuck at the first one (a 5-block rise in 3).
pub const RIVER_TERRACE_FLOW: (f64, f64) = (10.0, 40.0);
pub const RIVER_TERRACE: i32 = 3;
pub const RIVER_TERRACE_RISER: f64 = 0.5;
/// Rivers don't carve in the mountains: full where the mountains' area (`MOUNTAINS_AREA`) is up
/// to `.0`, gone at `.1`. Their rain still runs down, so rivers come out at the mountains' feet.
pub const RIVER_MOUNTAINS: (f64, f64) = (0.1, 0.45);
/// The meander: the river's line is pushed by up to about half `RIVER_WARP` blocks by a noise
/// `RIVER_WARP_SCALE` blocks across.
pub const RIVER_WARP: f64 = 110.0;
pub const RIVER_WARP_SCALE: f64 = 75.0;

/// Lakes in basins (Lithosphere's "big lakes anywhere, with islands"): a site per `LAKE_CELL`
/// blocks square, kept with `LAKE_CHANCE` times its climate's share (`LAKE_WET`: the dry land a
/// quarter of it, the wet all; `LAKE_COLD` more in the cold, as glacial country is full of lakes).
/// Only on low land, its middle at most `LAKE_MAX_RISE` blocks over the sea, inland, and out of
/// the mountains. Their bed is under the sea's level, so the sea's water fills them.
pub const LAKE_CELL: i32 = 420;
pub const LAKE_CHANCE: f64 = 0.8;
pub const LAKE_WET: (f64, f64) = (-0.2, 0.1);
pub const LAKE_COLD: (f64, f64) = (-0.05, -0.2);
pub const LAKE_COLD_MORE: f64 = 0.35;
pub const LAKE_MAX_RISE: i32 = 40;
/// A lake's radius in blocks (the smallest, the largest), its shore wobbled by up to
/// `LAKE_WOBBLE` of it (a noise `LAKE_WOBBLE_SIZE` times the radius across), and its depth under the sea's
/// level in the middle (blocks, the shallowest, the deepest).
pub const LAKE_RADIUS: (i32, i32) = (36, 130);
pub const LAKE_WOBBLE: f64 = 1.0;
pub const LAKE_WOBBLE_SIZE: f64 = 0.7;
/// Lakes run long: across their own direction (one of 8) they are this many sixteenths as wide
/// as along it at most (16: round).
pub const LAKE_STRETCH: (i32, i32) = (16, 34);
/// The 8 directions a lake can lie in, as (x, z) in 1/64.
const LAKE_AXES: [(i32, i32); 8] = [(64, 0), (59, 24), (45, 45), (24, 59), (0, 64), (-24, 59), (-45, 45), (-59, 24)];
pub const LAKE_DEPTH: (i32, i32) = (3, 9);
/// A shallow shelf round the shore: over the outer `LAKE_SHELF` of the lake the water is
/// `LAKE_SHELF_DEPTH` blocks deep (shallowest at the shore), then the bed drops to the bowl, so a
/// lake reads as a wide turquoise rim round a darker middle (Lighting, 11-04: the bed shows
/// through only to some 3-4 blocks).
pub const LAKE_SHELF: f64 = 0.33;
pub const LAKE_SHELF_DEPTH: (i32, i32) = (1, 2);
/// The shore: first `LAKE_BEACH` blocks flush with the water (room for reeds and a beach, so
/// trees don't stand in it), then rising to the land over `LAKE_SHORE` blocks, plus
/// `LAKE_SHORE_PER_BLOCK` (in sixteenths) for each block the land stands over the sea, counted up
/// to `LAKE_SHORE_RISE`. At 36 the steepest of the rise is about two blocks in three (Tester 2,
/// 11-04: at 24 the banks rose in blocky steps where trees came down to the water).
pub const LAKE_BEACH: i32 = 3;
pub const LAKE_SHORE: i32 = 10;
pub const LAKE_SHORE_PER_BLOCK: i32 = 36;
pub const LAKE_SHORE_RISE: i32 = 40;
/// Islands: where the island noise (patches about `LAKE_ISLAND_SIZE` blocks across) is over
/// `LAKE_ISLANDS`, the bed rises back over the water, up to `LAKE_ISLAND_HEIGHT` blocks; not in
/// the outer `LAKE_ISLAND_SHORE` of the lake, so islands stand clear of the shore.
pub const LAKE_ISLANDS: f64 = 0.16;
pub const LAKE_ISLAND_SIZE: f64 = 44.0;
pub const LAKE_ISLAND_HEIGHT: i32 = 5;
pub const LAKE_ISLAND_SHORE: f64 = 0.25;

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
/// In the hot, dry land's strata (mesas, canyons) a face goes bare from this fall: steps of three
/// would otherwise take the planting's gravel for steep ground, which read as dark bands in the
/// canyon walls (canblack, 11-04).
pub const ROCK_FALL_STRATA: i32 = 3;

impl RockSet {
    /// The fall from which a face of these rocks goes bare.
    pub fn bare_from(self) -> i32 {
        if self == RockSet::Strata { ROCK_FALL_STRATA } else { ROCK_FALL }
    }
}
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
pub const STRATA: [&str; 8] = [
    "sandstone",
    "ochre_sandstone",
    "cream_sandstone",
    "red_sandstone",
    "banded_clay",
    "ochre_sandstone",
    "cream_sandstone",
    "red_sandstone",
];
pub const STRATA_TILT: i32 = 3;

/// The engine's features of the land, in the order they're added; `mesas` and `rivers` go after
/// them (`land_height`, `rivers`).
pub fn features() -> Vec<Feature> {
    vec![
        preset::continents(CONTINENTS),
        preset::hills(HILLS.0 * COAST_CALM, HILLS.1).mask(HILLS_AREA.mask()),
        preset::hills(HILLS.0 * (1.0 - COAST_CALM), HILLS.1).mask(HILLS_INLAND.mask()),
        Feature::new(|_, _, _| q4(COAST_CLIFF))
            .mask(COAST_CLIFF_AREA.mask())
            .named("coast_cliffs"),
        Feature::new(|_, _, _| -q4(OCEAN_DEEP)).mask(OCEAN_DEEP_AREA.mask()),
        Feature::new(|_, _, _| q4(ISLAND_HEIGHT))
            .mask(ISLANDS_AREA.mask())
            .named("islands"),
        Feature::new(|_, _, _| q4(HIGHLANDS))
            .mask(HIGHLANDS_AREA.mask())
            .named("highlands"),
        preset::dunes(DUNES.0, DUNES.1).mask(DUNES_AREA.mask()),
        preset::mountains(MOUNTAINS.0, MOUNTAINS.1)
            .mask(MOUNTAINS_AREA.mask())
            .named("mountains"),
        Feature::new(mesa).named("mesas"),
        Feature::new(canyons).named("canyons"),
        Feature::new(karst).named("karst"),
        Feature::new(lakes).named("lakes"),
        Feature::new(rivers).named("rivers"),
        // Last: a crater is measured from its ground, the rivers' cut included (none reach one).
        Feature::new(craters).named("craters"),
        // No height: their masks say where the snow drifts and where the sequoias stand (rules
        // ask for them).
        preset::hills(0.0, 64.0)
            .mask(SEQUOIA_AREA.mask())
            .named("sequoia_groves"),
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
/// in `features` before `rivers` (the lakes included), the same integers the engine sums.
pub fn land_height(f: &Fields, x: i32, z: i32) -> i32 {
    land_height_at(&Here::new(f, x, z))
}

fn land_height_at(h: &Here) -> i32 {
    let ground = ground_at(h);
    ground + lakes_on(h, ground)
}

/// The ground before the lakes and the rivers.
fn ground_at(h: &Here) -> i32 {
    let base = base_ground(h);
    base + canyons::cut(h, base) + karst::add(h, base) + karst::sink(h, || base)
}

/// The ground before the karst towers (which stand on it, or rise out of the water over it).
fn base_ground(h: &Here) -> i32 {
    let (f, x, z) = (h.f, h.x, h.z);
    q4(SEA as f64)
        + shape::continents(f, x, z, q4(CONTINENTS))
        + masked(h, &HILLS_AREA, || shape::hills(f, x, z, q4(HILLS.0 * COAST_CALM), q4(HILLS.1)))
        + masked(h, &HILLS_INLAND, || shape::hills(f, x, z, q4(HILLS.0 * (1.0 - COAST_CALM)), q4(HILLS.1)))
        + masked(h, &COAST_CLIFF_AREA, || q4(COAST_CLIFF))
        + masked(h, &OCEAN_DEEP_AREA, || -q4(OCEAN_DEEP))
        + masked(h, &ISLANDS_AREA, || q4(ISLAND_HEIGHT))
        + masked(h, &HIGHLANDS_AREA, || q4(HIGHLANDS))
        + masked(h, &DUNES_AREA, || shape::dunes(f, x, z, q4(DUNES.0), q4(DUNES.1)))
        + masked(h, &MOUNTAINS_AREA, || shape::mountains(f, x, z, q4(MOUNTAINS.0), q4(MOUNTAINS.1)))
        + mesa_at(h)
}

/// The ground's height (Q4) with the rivers: what the engine makes of `features`.
pub fn height(f: &Fields, x: i32, z: i32) -> i32 {
    let ground = land_height(f, x, z) + rivers(f, x, z);
    ground + craters::add(&Here::new(f, x, z), || ground)
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

/// A lake's site: its middle (blocks), radius and depth (blocks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lake {
    pub x: i32,
    pub z: i32,
    pub radius: i32,
    pub depth: i32,
    /// Its direction (`LAKE_AXES`) and how much narrower it is across (`LAKE_STRETCH`).
    pub axis: u8,
    pub stretch: i32,
}

thread_local! {
    /// Lake sites already worked out, by seed and cell: the climate and the ground at a site's
    /// middle cost as much as a column, and every column near it asks.
    static LAKE_SITES: std::cell::RefCell<std::collections::HashMap<(u64, i32, i32), Option<Lake>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Cell (`cx`, `cz`)'s two hashes and the middle (blocks) its lake would have: cheap, so a column
/// far from every middle skips the lakes without asking for their climate.
fn lake_middle(seed: u64, cx: i32, cz: i32) -> (u32, u32, [i32; 2]) {
    let h = fixed::hash(fixed::seed32(seed, salt::LAKES), cx, 0, cz);
    let h2 = fixed::fmix(h);
    let span = LAKE_CELL - 2 * LAKE_RADIUS.1 / 3;
    let at = |c: i32, v: u32| c * LAKE_CELL + LAKE_CELL / 2 - span / 2 + (v % span as u32) as i32;
    (h, h2, [at(cx, h2), at(cz, h2 >> 16)])
}

/// The lake of cell (`cx`, `cz`), if it has one (`LAKE_CELL`).
pub fn lake_site(f: &Fields, cx: i32, cz: i32) -> Option<Lake> {
    if let Some(hit) = LAKE_SITES.with(|c| c.borrow().get(&(f.seed, cx, cz)).copied()) {
        return hit;
    }
    let site = (|| {
        let (h, h2, [x, z]) = lake_middle(f.seed, cx, cz);
        let here = Here::new(f, x * 16, z * 16);
        let wet = q12(0.25) + ((q12(0.75) * ramp(here.humidity(), q12(LAKE_WET.0), q12(LAKE_WET.1))) >> 12);
        let cold = ONE + ((q12(LAKE_COLD_MORE) * ramp(here.temperature(), q12(LAKE_COLD.0), q12(LAKE_COLD.1))) >> 12);
        let chance = (((q12(LAKE_CHANCE) * wet) >> 12) * cold) >> 12;
        if (h & 4095) as i32 >= chance {
            return None;
        }
        if here.continent() < q12(OCEAN_CONTINENT + 0.08) || MOUNTAINS_AREA.at(&here) > q12(0.3) {
            return None;
        }
        let rise = (ground_at(&here) >> 4) - SEA;
        if !(1..=LAKE_MAX_RISE).contains(&rise) {
            return None;
        }
        let pick = |v: u32, (lo, hi): (i32, i32)| lo + (v % (hi - lo + 1) as u32) as i32;
        let h3 = fixed::fmix(h2);
        Some(Lake {
            x,
            z,
            radius: pick(h2 >> 4, LAKE_RADIUS),
            depth: pick(h >> 12, LAKE_DEPTH),
            axis: (h3 % 8) as u8,
            stretch: pick(h3 >> 3, LAKE_STRETCH),
        })
    })();
    LAKE_SITES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4096 {
            c.clear();
        }
        c.insert((f.seed, cx, cz), site);
    });
    site
}

/// What the lakes take off the ground (Q4, zero or less) at (`x`, `z`) (Q4).
pub fn lakes(f: &Fields, x: i32, z: i32) -> i32 {
    let here = Here::new(f, x, z);
    lakes_on(&here, i32::MIN)
}

/// The lakes' cut at a column whose ground before them is `ground` (Q4; `i32::MIN`: not yet
/// known, worked out only if a lake is near).
fn lakes_on(h: &Here, ground: i32) -> i32 {
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let (cx, cz) = (div_floor(bx, LAKE_CELL).0, div_floor(bz, LAKE_CELL).0);
    let reach = LAKE_RADIUS.1 + ((LAKE_RADIUS.1 * q12(LAKE_WOBBLE)) >> 12) + lake_shore_most();
    let mut cut = 0;
    let mut ground = ground;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (_, _, [mx, mz]) = lake_middle(h.f.seed, cx + dx, cz + dz);
            if (bx - mx).abs() > reach || (bz - mz).abs() > reach {
                continue;
            }
            let Some(lake) = lake_site(h.f, cx + dx, cz + dz) else { continue };
            cut = cut.min(lake_cut(h, lake, &mut ground));
        }
    }
    for lake in net::region_at(h.f, bx, bz).lakes_near(bx, bz) {
        cut = cut.min(lake_cut(h, lake, &mut ground));
    }
    cut
}

/// The widest a lake's shore can be (blocks): the beach and the rise at its widest.
const fn lake_shore_most() -> i32 {
    LAKE_BEACH + LAKE_SHORE + ((LAKE_SHORE_PER_BLOCK * LAKE_SHORE_RISE) >> 4)
}

/// How far a lake of this radius (blocks) can reach with its shore at its widest (blocks).
fn lake_reach_most(radius: i32) -> i32 {
    radius + ((radius * q12(LAKE_WOBBLE)) >> 12) + lake_shore_most()
}

/// A column's distance from a lake's middle and the lake's radius there (both Q4), if it's within
/// `extra` (Q4) of the shore. Lakes run long along their own direction (`LAKE_STRETCH`), so the
/// distance is measured as if the lake were round; the shore wobbles by a noise as coarse as the
/// lake is big.
fn lake_reach(h: &Here, lake: Lake, extra: i32) -> Option<(i32, i32)> {
    let (bx, bz) = (h.x >> 4, h.z >> 4);
    let (ux, uz) = LAKE_AXES[lake.axis as usize];
    let (dx, dz) = ((bx - lake.x) as i64, (bz - lake.z) as i64);
    let along = (dx * ux as i64 + dz * uz as i64) / 64;
    let across = (dz * ux as i64 - dx * uz as i64) / 64;
    let across = across * lake.stretch as i64 / 16;
    let d = ((along * along + across * across) as u64).isqrt() as i32 * 16;
    let widest = q4(lake.radius as f64) + ((q4(lake.radius as f64) * q12(LAKE_WOBBLE)) >> 12);
    if d >= widest + extra {
        return None;
    }
    let size = q4(lake.radius as f64 * LAKE_WOBBLE_SIZE);
    let wobble = h.f.fbm(salt::LAKE_SHORE, h.x, h.z, size, 4);
    let r = q4(lake.radius as f64) + ((((q4(lake.radius as f64) * wobble) >> 12) * q12(LAKE_WOBBLE)) >> 12);
    let r = r.max(16 * 8);
    (d < r + extra).then_some((d, r))
}

/// One lake's cut at a column near it; `ground` (Q4) is worked out only where the lake reaches
/// (`i32::MIN` until then).
fn lake_cut(h: &Here, lake: Lake, ground: &mut i32) -> i32 {
    let sea = q4(SEA as f64);
    let shore_most = q4((lake_shore_most()) as f64);
    let Some((d, r)) = lake_reach(h, lake, shore_most) else { return 0 };
    if *ground == i32::MIN {
        *ground = ground_at(h);
    }
    let ground = *ground;
    let target = if d < r {
        // The shelf, 1-2 blocks deep, then the bowl down to `depth` in the middle.
        let t = d * ONE / r;
        let edge = q12(1.0 - LAKE_SHELF);
        let (shallow, shelf) = (q4(LAKE_SHELF_DEPTH.0 as f64), q4(LAKE_SHELF_DEPTH.1 as f64));
        let bed = if t >= edge {
            sea - shallow - (((shelf - shallow) * ramp(t, ONE, edge)) >> 12)
        } else {
            let u = t * ONE / edge.max(1);
            let bowl = ONE - ((u * u) >> 12);
            let deepest = q4(lake.depth as f64).max(shelf);
            sea - shelf - (((deepest - shelf) * bowl) >> 12)
        };
        // Islands, clear of the shore.
        let inner = ramp(t, ONE, q12(1.0 - LAKE_ISLAND_SHORE));
        let n =
            h.f.fbm(salt::LAKE_ISLANDS, h.x, h.z, q4(LAKE_ISLAND_SIZE), 3);
        let lift = (ramp(n, q12(LAKE_ISLANDS), q12(LAKE_ISLANDS + 0.2)) * inner) >> 12;
        let island = sea - 16 + ((q4((LAKE_ISLAND_HEIGHT + 1) as f64) * lift) >> 12);
        if lift > 0 { bed.max(island) } else { bed }
    } else {
        // The shore: level with the water for the beach, then rising smoothly to the land.
        let rise = ((ground - sea) >> 4).clamp(0, LAKE_SHORE_RISE);
        let beach = q4(LAKE_BEACH as f64);
        let width = q4((LAKE_SHORE + ((LAKE_SHORE_PER_BLOCK * rise) >> 4)) as f64);
        if d >= r + beach + width {
            return 0;
        }
        let t = (d - r - beach).max(0) * ONE / width;
        let smooth = (((t * t) >> 12) * (3 * ONE - 2 * t)) >> 12;
        sea + (((ground - sea) * smooth) >> 12)
    };
    (target - ground).min(0)
}

/// What the rivers take off the ground (Q4, zero or less) at (`x`, `z`) (Q4): the deepest cut
/// of the river stretches near (`net`), so tributaries join their river.
pub fn rivers(f: &Fields, x: i32, z: i32) -> i32 {
    let (bx, bz) = (x >> 4, z >> 4);
    let region = net::region_at(f, bx, bz);
    let mut segs = region.near(bx, bz).peekable();
    if segs.peek().is_none() {
        return 0;
    }
    let here = Here::new(f, x, z);
    let p = net::meander(f, x, z);
    let mountains = net::mountains(&here);
    if mountains == 0 {
        return 0;
    }
    let sea = q4(SEA as f64);
    let mut h0 = i32::MIN;
    let mut cut = 0;
    for s in segs {
        let (d, flow) = s.place(p);
        let sh = net::shape(flow, mountains);
        if sh.strength == 0 || d >= sh.half + sh.floodplain + q4(RIVER_BANKS as f64) + RIVER_VALLEY_PER_BLOCK * s.rise {
            continue;
        }
        if h0 == i32::MIN {
            h0 = land_height_at(&here);
        }
        let rise = (h0 - sea) >> 4;
        if rise < 0 {
            return 0;
        }
        let deep = sh.strength;
        let flat = sh.half + ((sh.floodplain * deep) >> 12);
        // The sides widen with the land's height, up to the stretch's bound (`net::Seg::rise`).
        let banks = ((q4(RIVER_BANKS as f64) + RIVER_VALLEY_PER_BLOCK * rise.min(s.rise)) * deep) >> 12;
        if d >= flat + banks {
            continue;
        }
        let target = if d < sh.half {
            // The channel: a block under the sea at its edge, `depth` in the middle.
            let t = d * ONE / sh.half;
            let bowl = ONE - ((t * t) >> 12);
            sea - 16 - (((sh.depth - 16) * bowl) >> 12)
        } else if d < flat {
            sea
        } else {
            // The valley's sides: rising smoothly from the floodplain to the land, in benches
            // by the bigger rivers.
            let t = (d - flat) * ONE / banks.max(1);
            let smooth = (((t * t) >> 12) * (3 * ONE - 2 * t)) >> 12;
            let raise = ((h0 - sea) * smooth) >> 12;
            let benches = ramp(flow, q12(RIVER_TERRACE_FLOW.0), q12(RIVER_TERRACE_FLOW.1));
            sea + raise + (((net::terraced(raise) - raise) * benches) >> 12)
        };
        cut = cut.min((((target - h0).min(0)) * deep) >> 12);
    }
    cut
}

/// The river network's region holding block (`bx`, `bz`): its nodes, the pits that spilled over
/// a neighbour, and its river stretches (for the map tools).
pub fn river_region_stats(f: &Fields, bx: i32, bz: i32) -> [usize; 3] {
    net::region_at(f, bx, bz).stats
}

/// The river network's stretches in the region holding block (`bx`, `bz`) that reach its cells:
/// their ends (blocks) and flow at each (wet nodes' rain, Q12), for the map tools.
pub fn river_region_stretches(f: &Fields, bx: i32, bz: i32) -> Vec<([i32; 2], [i32; 2], [i32; 2])> {
    net::region_at(f, bx, bz)
        .segs
        .iter()
        .map(|s| ([s.a[0] >> 4, s.a[1] >> 4], [s.b[0] >> 4, s.b[1] >> 4], s.flow))
        .collect()
}

/// What the canyons take off the ground at (`x`, `z`) (Q4), as the engine adds it.
pub fn canyons(f: &Fields, x: i32, z: i32) -> i32 {
    let h = Here::new(f, x, z);
    if !canyons::near(&h) { 0 } else { canyons::cut(&h, base_ground(&h)) }
}

/// What the karst towers and sinkholes add at (`x`, `z`) (Q4), as the engine adds it.
pub fn karst(f: &Fields, x: i32, z: i32) -> i32 {
    let h = Here::new(f, x, z);
    let towers = if KARST_AREA.at(&h) == 0 { 0 } else { karst::add(&h, base_ground(&h)) };
    towers + karst::sink(&h, || base_ground(&h))
}

/// What the meteor craters add at (`x`, `z`) (Q4), as the engine adds it.
pub fn craters(f: &Fields, x: i32, z: i32) -> i32 {
    craters::add(&Here::new(f, x, z), || land_height(f, x, z) + rivers(f, x, z))
}

/// The meteor craters (middle x, z and radius, blocks) within `r` blocks of (`x`, `z`) (blocks).
pub fn craters_near(f: &Fields, x: i32, z: i32, r: i32) -> Vec<(i32, i32, i32)> {
    craters::near(f, x, z, r)
}

/// The karst sinkholes (middle x, z and radius, blocks) within `r` blocks of (`x`, `z`) (blocks).
pub fn sinks_near(f: &Fields, x: i32, z: i32, r: i32) -> Vec<(i32, i32, i32)> {
    karst::sinks_near(f, x, z, r)
}

/// A fair first place (Ronald, 11-04: "All 3: 1, 2, 3"): the nearest spot to the world's middle,
/// searched in rings `SPAWN_STEP` blocks apart out to `SPAWN_REACH`, that is plains (few trees to
/// spawn in), temperate (`SPAWN_TEMPERATURE`: away from the taiga's cold and the desert's heat),
/// `SPAWN_RISE` over the sea, level (no neighbour two blocks off more than a block higher or
/// lower), out of the mountains, with water (a lake or a river, not the sea) within
/// `SPAWN_WATER` blocks and woods within `SPAWN_WOODS`. The engine's own search took the first
/// plains column near the middle: on seeds 1, 7, 42 and 1234 that stood 34-54 blocks up a hill,
/// two of them without water within 96 blocks.
pub const SPAWN_STEP: i32 = 16;
pub const SPAWN_REACH: i32 = 1600;
pub const SPAWN_TEMPERATURE: (f64, f64) = (-0.06, 0.12);
pub const SPAWN_RISE: (i32, i32) = (2, 30);
pub const SPAWN_WATER: i32 = 48;
pub const SPAWN_WOODS: i32 = 64;

/// The first block's column (x, height, z in blocks) of the fair first place for world `seed`,
/// or, failing that within `SPAWN_REACH`, the first level plains, then the first land above the
/// sea, on the same rings.
pub fn spawn(seed: u64) -> [i32; 3] {
    spawn_where(seed, &|_, _| true)
}

/// `spawn`, only where `open` (blocks) also holds: what the terrain here can't see, as the
/// engine's caves (seed 42's first fair place stood over a cave's mouth, PC look fv_new_42: the
/// player fell 15 blocks).
pub fn spawn_where(seed: u64, open: &dyn Fn(i32, i32) -> bool) -> [i32; 3] {
    let f = fields(seed);
    let h = |x: i32, z: i32| height(&f, x * 16, z * 16) >> 4;
    let biome = |x: i32, z: i32| overworld_biome(&f, x * 16, z * 16);
    let level = |x: i32, z: i32, hh: i32| {
        [(2, 0), (-2, 0), (0, 2), (0, -2)]
            .iter()
            .all(|(dx, dz)| (h(x + dx, z + dz) - hh).abs() <= 1)
    };
    let near = |x: i32, z: i32, reach: i32, step: i32, hit: &dyn Fn(i32, i32) -> bool| {
        (0..=reach).step_by(step as usize).any(|d| {
            (0..16).any(|k| {
                let a = k as f64 / 16.0 * std::f64::consts::TAU;
                let (px, pz) = (x + (d as f64 * a.cos()).round() as i32, z + (d as f64 * a.sin()).round() as i32);
                hit(px, pz)
            })
        })
    };
    let fair = |x: i32, z: i32| -> bool {
        let hh = h(x, z);
        let t = f.temperature(x * 16, z * 16);
        biome(x, z) == "plains"
            && (q12(SPAWN_TEMPERATURE.0)..q12(SPAWN_TEMPERATURE.1)).contains(&t)
            && (SEA + SPAWN_RISE.0..=SEA + SPAWN_RISE.1).contains(&hh)
            && level(x, z, hh)
            && MOUNTAINS_AREA.value(&f, x * 16, z * 16) < q12(0.2)
            && near(x, z, SPAWN_WATER, 4, &|px, pz| h(px, pz) < SEA && biome(px, pz) != "ocean")
            && near(x, z, SPAWN_WOODS, 8, &|px, pz| matches!(biome(px, pz), "forest" | "birch_forest" | "taiga" | "sequoia_grove"))
    };
    let plains = |x: i32, z: i32| biome(x, z) == "plains" && h(x, z) > SEA + 1 && level(x, z, h(x, z));
    let land = |x: i32, z: i32| h(x, z) > SEA + 1;
    for test in [&fair as &dyn Fn(i32, i32) -> bool, &plains, &land] {
        for r in 0..=SPAWN_REACH / SPAWN_STEP {
            let n = (r * 8).max(1);
            for k in 0..n {
                let a = k as f64 / n as f64 * std::f64::consts::TAU;
                let (x, z) = ((r * SPAWN_STEP) as f64 * a.cos(), (r * SPAWN_STEP) as f64 * a.sin());
                let (x, z) = (x.round() as i32, z.round() as i32);
                if test(x, z) && open(x, z) {
                    return [x, h(x, z), z];
                }
            }
        }
    }
    [0, h(0, 0).max(SEA), 0]
}

/// Which water a bank belongs to (`bank`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Water {
    River,
    Lake,
}

/// How a column stands to the nearest river or lake: for plants that grow by water (reeds,
/// rushes, lily pads, dense bank growth). `in_water`: the column is the river's channel or the
/// lake itself; `wet`: 1 (`ONE`) in the water and at its edge, falling to 0 at the outer edge of
/// the bank (`RIVER_BANK`, `LAKE_BANK`), the river's by how strong it runs there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bank {
    pub water: Water,
    pub in_water: bool,
    pub wet: i32,
}

/// The bank band beside a river's channel and beside a lake's shore, in blocks. Both start at the water's level: rivers' and lakes' banks are flush
/// with the water for their first few blocks, room for reeds.
pub const RIVER_BANK: i32 = 12;
pub const LAKE_BANK: i32 = 14;

/// The river or lake bank at (`x`, `z`) (Q4), if any; the wetter of the two where both are near.
/// Cheap enough for a decoration's every column (it never rebuilds the land's height): the river
/// stretches listed near it (none in most places), and the lake sites near.
pub fn bank(f: &Fields, x: i32, z: i32) -> Option<Bank> {
    let here = Here::new(f, x, z);
    let mut best: Option<Bank> = None;
    let mut take = |b: Bank| {
        if best.is_none_or(|o| b.wet > o.wet) {
            best = Some(b);
        }
    };
    // Rivers: the channel's half width as `rivers` has it, without the land.
    let (bx, bz) = (x >> 4, z >> 4);
    let region = net::region_at(f, bx, bz);
    let mut segs = region.near(bx, bz).peekable();
    if segs.peek().is_some() {
        let mountains = net::mountains(&here);
        let p = net::meander(f, x, z);
        for s in segs {
            let (d, flow) = s.place(p);
            let sh = net::shape(flow, mountains);
            if sh.strength == 0 {
                continue;
            }
            if d < sh.half {
                take(Bank { water: Water::River, in_water: true, wet: sh.strength });
            } else if d < sh.half + q4(RIVER_BANK as f64) {
                let w = ONE - (d - sh.half) * ONE / q4(RIVER_BANK as f64);
                take(Bank { water: Water::River, in_water: false, wet: (w * sh.strength) >> 12 });
            }
        }
    }
    // Lakes.
    let (cx, cz) = (div_floor(bx, LAKE_CELL).0, div_floor(bz, LAKE_CELL).0);
    let reach = LAKE_RADIUS.1 + ((LAKE_RADIUS.1 * q12(LAKE_WOBBLE)) >> 12) + LAKE_BANK;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (_, _, [mx, mz]) = lake_middle(f.seed, cx + dx, cz + dz);
            if (bx - mx).abs() > reach || (bz - mz).abs() > reach {
                continue;
            }
            let Some(lake) = lake_site(f, cx + dx, cz + dz) else { continue };
            lake_bank(&here, lake, &mut take);
        }
    }
    for lake in region.lakes_near(bx, bz) {
        lake_bank(&here, lake, &mut take);
    }
    best
}

/// A lake's bank at a column, if it reaches (`bank`).
fn lake_bank(here: &Here, lake: Lake, take: &mut impl FnMut(Bank)) {
    let Some((d, r)) = lake_reach(here, lake, q4(LAKE_BANK as f64)) else { return };
    if d < r {
        take(Bank { water: Water::Lake, in_water: true, wet: ONE });
    } else {
        take(Bank { water: Water::Lake, in_water: false, wet: ONE - (d - r) * ONE / q4(LAKE_BANK as f64) });
    }
}

/// A column near a biome border (`biome_edge`): the biome over it, and how near (`ONE` on the
/// border, 0 at `EDGE_BAND` of the climate away).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub to: &'static str,
    pub near: i32,
}

/// How far in the climate fields a biome's edge reaches into its neighbour, for transition
/// planting (forest thinning into plains, savanna into desert): about 60-100 blocks of ground.
pub const EDGE_BAND: f64 = 0.03;

/// The biome the climate rules pick for these fields (`overworld_biome` without the coasts).
fn climate_biome(t: i32, hu: i32, c: i32, h: i32, mountain: bool, grove: bool) -> &'static str {
    let at = q12;
    if mountain || h >= snow_line(t) {
        "mountains"
    } else if grove && sequoia_fits(t, hu, h) {
        "sequoia_grove"
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

/// The nearest other biome by climate at (`x`, `z`) (Q4), ground `height` (blocks) high, if a
/// border is within `EDGE_BAND`: which it is and how near. Climate borders only (not the coasts'
/// height rules); the mountains' edge counts where their area crosses `MOUNTAIN_BIOME`.
pub fn biome_edge(f: &Fields, x: i32, z: i32, height: i32) -> Option<Edge> {
    let here = Here::new(f, x, z);
    let (t, hu, c) = (here.temperature(), here.humidity(), here.continent());
    let m = MOUNTAINS_AREA.at(&here);
    let line = q12(MOUNTAIN_BIOME);
    let grove = SEQUOIA_AREA.at(&here) > q12(SEQUOIA_GROVE);
    let own = climate_biome(t, hu, c, height, m > line, grove);
    let band = q12(EDGE_BAND);
    let mut best: Option<Edge> = None;
    for k in 1..=3 {
        let step = band * k / 3;
        for (dt, dh, dm) in [(step, 0, 0), (-step, 0, 0), (0, step, 0), (0, -step, 0), (0, 0, step), (0, 0, -step)] {
            let to = climate_biome(t + dt, hu + dh, c, height, m + dm > line, grove);
            if to != own {
                let near = ONE - (k - 1) * ONE / 3;
                if best.is_none_or(|b| near > b.near) {
                    best = Some(Edge { to, near });
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    None
}

/// Biome borders that fray (Ronald, 11-04: "All 3: 1, 2, 3"): where two biomes with different
/// ground meet (the desert's sand against grass, the taiga's podzol, the snowy plains' snow-grass),
/// the neighbour's ground comes in patches (`FRAY_SIZE` blocks across) over the border band
/// (`biome_edge`): on the border `FRAY_SHARE` of the ground is the other's, less further in.
pub const FRAY_SHARE: f64 = 0.55;
pub const FRAY_SIZE: f64 = 7.0;

/// A biome's ground: its top block and what lies under it (base's biome rules' `surface`).
pub fn surface_of(biome: &str) -> (&'static str, &'static str) {
    match biome {
        "desert" | "beach" => ("sand", "sand"),
        "taiga" => ("podzol", "dirt"),
        "snowy_plains" => ("snowy_grass", "dirt"),
        _ => ("grass", "dirt"),
    }
}

/// The ground column (`x`, `z`) (Q4), `height` blocks high, takes from the biome over the border,
/// if it frays here: its top and under block. Cheap away from the borders that change the ground
/// (a test of the climate against their thresholds first).
pub fn fray(f: &Fields, x: i32, z: i32, height: i32) -> Option<(&'static str, &'static str)> {
    let here = Here::new(f, x, z);
    let (t, hu) = (here.temperature(), here.humidity());
    let band = q12(EDGE_BAND);
    let close = |v: i32, at: f64| (v - q12(at)).abs() < band;
    let ground_border = close(t, SNOWY_BELOW)
        || close(t, TAIGA_BELOW)
        || (close(t, DESERT.0) && hu < q12(DESERT.1) + band)
        || (close(hu, DESERT.1) && t >= q12(DESERT.0) - band);
    if !ground_border {
        return None;
    }
    let e = biome_edge(f, x, z, height)?;
    let own = biome_at(f, x, z, height);
    // The coasts and the mountains keep their own ground (theirs are height rules, not climate).
    if matches!(own, "beach" | "ocean" | "mountains") {
        return None;
    }
    let theirs = surface_of(e.to);
    if surface_of(own) == theirs {
        return None;
    }
    let n = ONE / 2 + f.fbm(salt::FRAY, x, z, q4(FRAY_SIZE), 2) * 2;
    (n < (q12(FRAY_SHARE) * e.near) >> 12).then_some(theirs)
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
    /// The karst towers: weathered limestone, rain-streaked.
    Karst,
}

/// The rocks of the face of column (`x`, `z`) (Q4), its ground `height` (blocks) high; `sandy`
/// faces (sand on top) are strata wherever they are.
pub fn rock_set(f: &Fields, x: i32, z: i32, height: i32, sandy: bool) -> RockSet {
    let here = Here::new(f, x, z);
    let (t, hu) = (here.temperature(), here.humidity());
    if sandy {
        RockSet::Strata
    } else if KARST_AREA.at(&here) > 0 {
        // Karst towers are weathered limestone however high they stand, and where the savanna's
        // strata would be (kclusters: a striped tower at the warm edge).
        RockSet::Karst
    } else if t >= q12(SAVANNA.0) && hu < q12(SAVANNA.1) {
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
    /// only where `mossy`; the rock's ore where it has one (`face_ore`).
    pub fn block(self, seed: u64, pos: [i32; 3], mossy: bool) -> &'static str {
        let rock = self.rock(seed, pos, mossy);
        face_ore(seed, pos, rock).unwrap_or(rock)
    }

    fn rock(self, seed: u64, [x, y, z]: [i32; 3], mossy: bool) -> &'static str {
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
            RockSet::Karst if patch => "limestone",
            RockSet::Karst if low && mossy => "mossy_cobblestone",
            RockSet::Karst => "weathered_limestone",
            _ => "limestone",
        }
    }
}

/// Ores in the rock faces (Ronald, 11-04: "All 3: 1, 2, 3"; Art's textures, "Yes, all four"):
/// what a face shows where its rock carries ore, so a player finds early iron and coal on the
/// walls of the land, not only underground. Shares of the rock's face blocks (checked by
/// `face_ores_come_in_their_shares`); crafting's early needs are some 15 to 20 iron (a stone age
/// kit's metal heads, a bucket, a cooking pot) and coal for the first smelting, and a canyon or
/// granite face shows hundreds of blocks.
///
/// - Coal seams in the banded clay: level runs some `FACE_COAL.1` blocks long, `FACE_COAL.0` of it.
/// - Ironstone in the red beds: knots of a few blocks, `FACE_IRONSTONE.0` of red sandstone.
/// - Magnetite specks in granite, `FACE_MAGNETITE.0`, and thin gold-quartz veins, `FACE_GOLD.0`.
pub const FACE_COAL: (f64, f64) = (0.45, 10.0);
pub const FACE_IRONSTONE: (f64, f64) = (0.05, 4.0);
pub const FACE_MAGNETITE: (f64, f64) = (0.022, 1.5);
pub const FACE_GOLD: (f64, f64) = (0.003, 14.0);

/// The ore block (its name in base) a face of `rock` shows at `pos` (world blocks), if any.
pub fn face_ore(seed: u64, [x, y, z]: [i32; 3], rock: &str) -> Option<&'static str> {
    let (bx, by, bz) = (x * 16, y * 16, z * 16);
    // A noise of `size` blocks (stretched `flat` times along the level) over its threshold for
    // `share` of the rock: the noise's values spread about evenly over -0.6..0.6 (Q12).
    let over = |salt: u64, (share, size): (f64, f64), flat: i32| {
        let s = q4(size);
        let v = fixed::perlin3(fixed::seed32(seed, salt), bx, by, bz, s * flat, s, s * flat);
        v > q12(ore_line(share))
    };
    match rock {
        "banded_clay" if over(salt::FACE_COAL, (FACE_COAL.0, 1.0), FACE_COAL.1 as i32) => Some("banded_clay_coal_ore"),
        "red_sandstone" if over(salt::FACE_IRONSTONE, FACE_IRONSTONE, 1) => Some("red_sandstone_iron_ore"),
        "granite" => {
            // Gold in thin veins: near the zero sheet of a slow noise (some 6% of the rock), where
            // a second noise allows (a fifteenth of the rest).
            let s = q4(FACE_GOLD.1);
            let sheet = fixed::perlin3(fixed::seed32(seed, salt::FACE_GOLD), bx, by, bz, s, s, s).abs();
            if sheet < q12(0.02) && over(salt::FACE_GOLD + 1, (FACE_GOLD.0 * 15.0, 6.0), 1) {
                Some("granite_gold_ore")
            } else if over(salt::FACE_MAGNETITE, FACE_MAGNETITE, 1) {
                Some("granite_iron_ore")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The threshold that leaves `share` of a 3D noise over it: measured on `perlin3` (close to a
/// normal spread a quarter wide), between these points straight.
fn ore_line(share: f64) -> f64 {
    const LINES: [(f64, f64); 9] =
        [(0.003, 0.666), (0.01, 0.582), (0.022, 0.510), (0.05, 0.428), (0.1, 0.374), (0.2, 0.249), (0.3, 0.140), (0.45, 0.017), (0.5, 0.0)];
    let i = LINES
        .iter()
        .position(|&(s, _)| s >= share)
        .unwrap_or(LINES.len() - 1)
        .max(1);
    let ((s0, l0), (s1, l1)) = (LINES[i - 1], LINES[i]);
    l0 + (l1 - l0) * ((share - s0) / (s1 - s0)).clamp(0.0, 1.0)
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
    biome_at(f, x, z, height(f, x, z) >> 4)
}

/// `overworld_biome` for ground `h` blocks high, already known.
fn biome_at(f: &Fields, x: i32, z: i32, h: i32) -> &'static str {
    let here = Here::new(f, x, z);
    let (t, hu, c) = (here.temperature(), here.humidity(), here.continent());
    let at = |v: f64| q12(v);
    if h <= SEA - 2 && c < at(OCEAN_CONTINENT) {
        "ocean"
    } else if h <= SEA + 1 && c < at(0.1) {
        if t < at(SNOWY_BELOW) { "snowy_plains" } else { "beach" }
    } else if MOUNTAINS_AREA.at(&here) > at(MOUNTAIN_BIOME) || snowy(f, x, z, h) {
        "mountains"
    } else if sequoia_fits(t, hu, h) && SEQUOIA_AREA.at(&here) > at(SEQUOIA_GROVE) {
        "sequoia_grove"
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

/// Whether the climate and the height (blocks) fit a sequoia grove (its rule but the grove patches).
fn sequoia_fits(t: i32, hu: i32, h: i32) -> bool {
    t >= q12(SEQUOIA_TEMPERATURE.0)
        && t < q12(SEQUOIA_TEMPERATURE.1)
        && hu >= q12(SEQUOIA_HUMIDITY)
        && (SEA + SEQUOIA_RISE.0..=SEA + SEQUOIA_RISE.1).contains(&h)
}

/// Each overworld biome's share of the land (not the sea), sampled every `step` blocks out to
/// `radius` from the middle, and how near the middle it first turns up (blocks, the larger of
/// x and z).
pub fn biome_shares(seed: u64, radius: i32, step: usize) -> Vec<(&'static str, f64, i32)> {
    let f = fields(seed);
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
        let f = fields(7);
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
    fn river_regions_agree_and_rivers_meet() {
        let f = fields(2);
        let region = RIVER_REGION * RIVER_NODE;
        let mut seen = std::collections::HashMap::new();
        let mut shared = 0;
        for rz in -1..=0 {
            for rx in -1..=0 {
                for (a, b, flow) in river_region_stretches(&f, rx * region, rz * region) {
                    if let Some(other) = seen.insert((a, b), flow) {
                        assert_eq!(other, flow, "two regions disagree on the stretch {a:?} -> {b:?}");
                        shared += 1;
                    }
                }
            }
        }
        assert!(shared > 0, "no stretch near a region's border");
        let mut into = std::collections::HashMap::new();
        for (_, b) in seen.keys() {
            *into.entry(*b).or_insert(0) += 1;
        }
        assert!(into.values().any(|&n| n >= 2), "no tributary joins a river");
    }

    #[test]
    fn coasts_have_cliffs_and_islands() {
        let f = fields(1);
        let (mut cliffs, mut islands) = (0, 0);
        for i in 0..40_000 {
            let (bx, bz) = ((i % 200) * 16 - 1600, (i / 200) * 16 - 1600);
            let h = height(&f, bx * 16, bz * 16) >> 4;
            if h < SEA {
                continue;
            }
            if ISLANDS_AREA.value(&f, bx * 16, bz * 16) > q12(0.3) {
                islands += 1;
            }
            let sea_near = [(16, 0), (-16, 0), (0, 16), (0, -16)]
                .iter()
                .any(|(dx, dz)| height(&f, (bx + dx) * 16, (bz + dz) * 16) >> 4 < SEA);
            if sea_near && h >= SEA + 8 && COAST_CLIFF_AREA.value(&f, bx * 16, bz * 16) > q12(0.8) {
                cliffs += 1;
            }
        }
        assert!(cliffs > 0 && islands > 0, "{cliffs} cliff columns, {islands} island columns in 3.2 km");
    }

    #[test]
    fn terraces_are_benches_that_never_fill() {
        let mut last = 0;
        let mut flat = 0;
        for raise in 0..q4(60.0) {
            let t = net::terraced(raise);
            assert!(t >= last, "benches only climb");
            assert!(t <= raise + q4(1.0), "a bench stands at most a block over the smooth side");
            if t == last {
                flat += 1;
            }
            last = t;
        }
        assert!(flat > q4(60.0) * 2 / 5, "much of the side is bench: {flat}");
    }

    #[test]
    fn karst_towers_stand_in_their_land_only() {
        let f = fields(1);
        let (mut towers, mut outside) = (0, 0);
        for i in 0..40_000 {
            let (x, z) = ((i % 200) * 12 * 16 - 1200 * 16, (i / 200) * 12 * 16 - 1200 * 16);
            let add = karst(&f, x, z);
            assert!((0..=q4(KARST_WIDE_HEIGHT.1 as f64 + 9.0)).contains(&add), "{add}");
            if add > q4(8.0) {
                towers += 1;
                if KARST_AREA.value(&f, x, z) == 0 {
                    outside += 1;
                }
            }
        }
        assert!(towers > 0 && outside == 0, "{towers} tower columns, {outside} outside the karst land");
    }

    #[test]
    fn canyons_cut_dry_in_their_land_only() {
        let f = fields(1);
        let mut deep = 0;
        for i in 0..40_000 {
            let (x, z) = ((i % 200) * 16 * 16 - 1600 * 16, (i / 200) * 16 * 16 - 1600 * 16);
            let cut = canyons(&f, x, z);
            assert!((-q4(CANYON_DEPTH as f64)..=0).contains(&cut), "{cut}");
            if cut < 0 {
                assert!(CANYON_AREA.value(&f, x, z) > 0);
                assert!(land_height(&f, x, z) >> 4 >= SEA + CANYON_DRY || lakes(&f, x, z) < 0, "a canyon floor under the sea at {x} {z}");
            }
            if cut < -q4(20.0) {
                deep += 1;
            }
        }
        assert!(deep > 0, "no canyon 20 deep in 3.2 km");
    }

    #[test]
    fn face_ores_come_in_their_shares() {
        let n = 200_000;
        let share = |rock: &str, ore: &str| {
            let hits = (0..n)
                .filter(|i| {
                    let pos = [(i % 211) - 100 + i / 50_000 * 997, 60 + (i / 211) % 80, (i * 31) % 4001 - 2000];
                    face_ore(1, pos, rock) == Some(ore)
                })
                .count();
            hits as f64 / n as f64
        };
        for (rock, ore, want) in [
            ("banded_clay", "banded_clay_coal_ore", FACE_COAL.0),
            ("red_sandstone", "red_sandstone_iron_ore", FACE_IRONSTONE.0),
            ("granite", "granite_iron_ore", FACE_MAGNETITE.0),
            ("granite", "granite_gold_ore", FACE_GOLD.0),
        ] {
            let got = share(rock, ore);
            assert!((got - want).abs() < want * 0.3, "{ore}: {got:.4} of {rock}, want {want}");
        }
        assert_eq!(face_ore(1, [0, 70, 0], "limestone"), None);
    }

    #[test]
    fn spawns_are_fair() {
        for seed in [1, 2, 3, 7, 42, 1234] {
            let [x, y, z] = spawn(seed);
            let f = fields(seed);
            assert_eq!(overworld_biome(&f, x * 16, z * 16), "plains", "seed {seed}: spawn at {x} {z}");
            assert!((SEA + SPAWN_RISE.0..=SEA + SPAWN_RISE.1).contains(&y), "seed {seed}: spawn {y} high");
            let t = f.temperature(x * 16, z * 16);
            assert!((q12(SPAWN_TEMPERATURE.0)..q12(SPAWN_TEMPERATURE.1)).contains(&t), "seed {seed}: too cold or hot");
            assert!(x.abs().max(z.abs()) <= SPAWN_REACH, "seed {seed}: spawn {x} {z} far out");
        }
    }

    #[test]
    fn borders_fray_only_where_the_ground_changes() {
        let f = fields(1);
        let mut frayed = 0;
        for i in 0..40_000 {
            let (x, z) = ((i % 200) * 8 * 16 - 800 * 16, (i / 200) * 8 * 16 - 800 * 16);
            let h = height(&f, x, z) >> 4;
            if let Some((top, _)) = fray(&f, x, z, h) {
                assert_ne!(top, surface_of(overworld_biome(&f, x, z)).0, "a fray to the same ground at {x} {z}");
                frayed += 1;
            }
        }
        assert!(frayed > 0 && frayed < 4_000, "{frayed} frayed columns of 40 000");
    }

    #[test]
    fn craters_and_sinkholes_keep_their_shapes() {
        let f = fields(42);
        let sites = craters_near(&f, 0, 0, 8000);
        assert!(sites.len() >= 4, "{} craters in 16 km", sites.len());
        for &(x, z, r) in &sites {
            // A bowl at the middle under its rim's crest, nothing past the rim's foot.
            let deep = height(&f, x * 16, z * 16);
            for (dx, dz) in [(r, 0), (-r, 0), (0, r), (0, -r)] {
                let crest = height(&f, (x + dx) * 16, (z + dz) * 16);
                assert!(crest - deep >= q4(r as f64 * CRATER_DEPTH * 0.8), "crater at {x} {z}: shallow");
            }
            assert_eq!(craters(&f, (x + r * 2) * 16, z * 16), 0, "crater at {x} {z}: reaches too far");
        }
        let f = fields(1);
        let sinks = sinks_near(&f, 0, 0, 3000);
        assert!(!sinks.is_empty(), "no sinkhole in 6 km");
        for &(x, z, _) in &sinks {
            assert!(land_height(&f, x * 16, z * 16) <= q4((SEA - SINK_FLOOR) as f64), "sinkhole at {x} {z} holds no water");
        }
    }

    #[test]
    fn arches_stand_and_waterfalls_fall_into_the_sea() {
        let f = fields(7);
        let arches = landmarks::arches_near(&f, 0, 0, 3000);
        assert!(!arches.is_empty(), "no arch within 3 km");
        for a in &arches {
            // The ring's top and both legs' feet.
            let leg = (a.inner + a.outer) / 2;
            let (lx, lz) = if a.along_x { (leg, 0) } else { (0, leg) };
            assert!(a.holds(a.x, a.base + (a.inner + a.outer) / 2, a.z, i32::MIN), "arch at {} {}: no top", a.x, a.z);
            assert!(
                a.holds(a.x + lx, a.base - 1, a.z + lz, a.base - 2) && a.holds(a.x - lx, a.base - 1, a.z - lz, a.base - 2),
                "arch at {} {}: no legs",
                a.x,
                a.z
            );
            assert!(!a.holds(a.x, a.base, a.z, i32::MIN), "arch at {} {}: filled under", a.x, a.z);
        }
        let falls = landmarks::waterfalls_near(&f, 0, 0, 3000);
        assert!(!falls.is_empty(), "no waterfall within 3 km");
        for w in &falls {
            let [x, y, z] = w.spring;
            assert!(height(&f, x * 16, z * 16) >> 4 > y, "a spring at {x} {z} over its ground");
            assert!(height(&f, w.front[0] * 16, w.front[1] * 16) >> 4 < SEA, "a waterfall at {x} {z} not over the sea");
        }
    }

    #[test]
    fn lakes_only_cut_and_hold_water() {
        let f = fields(1);
        let (mut wet, mut lakes_seen) = (0, 0);
        for i in 0..40_000 {
            let (x, z) = ((i % 200) * 12 * 16, (i / 200) * 12 * 16);
            let cut = lakes(&f, x, z);
            assert!(cut <= 0);
            if cut < 0 {
                lakes_seen += 1;
                if land_height(&f, x, z) >> 4 < SEA {
                    wet += 1;
                }
            }
        }
        assert!(lakes_seen > 0 && wet > 0, "no lake in 2.4 km by 2.4 km ({lakes_seen} cut, {wet} wet)");
    }

    #[test]
    fn sequoia_groves_are_rare_and_found() {
        for seed in [1, 2, 3, 7, 42, 1234] {
            let shares = biome_shares(seed, 8000, 32);
            let (share, near) = shares
                .iter()
                .find(|e| e.0 == "sequoia_grove")
                .map_or((0.0, i32::MAX), |e| (e.1, e.2));
            assert!(share > 0.0005 && share < 0.03, "seed {seed}: sequoia groves are {:.2}% of the land", share * 100.0);
            assert!(near <= 6000, "seed {seed}: the nearest grove is {near} blocks out");
        }
    }

    #[test]
    fn banks_lie_by_their_water() {
        let f = fields(1);
        let (mut river, mut lake) = (0, 0);
        for i in 0..40_000 {
            let (x, z) = ((i % 200) * 12 * 16, (i / 200) * 12 * 16);
            if let Some(b) = bank(&f, x, z) {
                assert!((0..=ONE).contains(&b.wet));
                match b.water {
                    Water::River => river += 1,
                    Water::Lake => lake += 1,
                }
                // In a lake is under the sea's level.
                if b.water == Water::Lake && b.in_water && lakes(&f, x, z) < 0 {
                    assert!(height(&f, x, z) >> 4 < SEA + LAKE_ISLAND_HEIGHT + 1);
                }
            }
        }
        assert!(river > 0 && lake > 0, "{river} river banks, {lake} lake banks");
    }

    #[test]
    fn biome_edges_name_another_biome() {
        let f = fields(2);
        let mut seen = 0;
        for i in 0..5_000 {
            let (x, z) = ((i % 100) * 40 * 16, (i / 100) * 40 * 16);
            let h = height(&f, x, z) >> 4;
            if let Some(e) = biome_edge(&f, x, z, h) {
                assert!((0..=ONE).contains(&e.near));
                seen += 1;
            }
        }
        assert!(seen > 0);
    }

    #[test]
    fn mesas_are_tiers_no_higher_than_their_height() {
        let f = fields(3);
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

    /// Tester 2 found no desert in 114 spots out to 18,000 blocks on seed 1 while the desert's
    /// threshold sat at 0.35 (past the temperature noise's 95th percentile); the grid over the
    /// same reach must find a share near the real world's.
    #[test]
    fn deserts_across_18_km_of_seed_1() {
        let shares = biome_shares(1, 18_000, 96);
        let share = |b: &str| shares.iter().find(|e| e.0 == b).map_or(0.0, |e| e.1);
        assert!(share("desert") > 0.04 && share("desert") < 0.16, "desert is {:.1}% of the land: {shares:?}", share("desert") * 100.0);
        for (b, s, _) in &shares {
            assert!(*s < 0.3, "{b} is {:.1}% of the land", s * 100.0);
        }
    }

    #[test]
    fn every_biome_turns_up_near_the_middle() {
        // The playtesters' seeds too (7, 42, 1234), at the continent scale of 1800.
        for seed in [1, 2, 3, 7, 42, 1234] {
            let shares = biome_shares(seed, 6000, 48);
            for (b, s, _) in &shares {
                assert!(*s < 0.25, "seed {seed}: {b} is {:.1}% of the land", s * 100.0);
            }
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

//! Terrain from parts: a base height plus landforms, biome rules that pick each column's
//! biome and surface, water and caves. The mod that owns a dimension describes its terrain and
//! any mod can edit it afterwards (add a biome before another, a landform, a cave biome), so the
//! engine evaluates the merged result. Heights, masks and fields are expressions (`Expr`) the
//! engine runs on the CPU and the GPU alike, so a mod's own shapes are as fast as the presets.
//!
//! ```ignore
//! r.terrain(&MARS)
//!     .stone("basalt")
//!     .base_height(60.0)
//!     .field("mars:peaks", Expr::noise(500.0).ramp(0.2, 0.5))
//!     .landform(preset::hills(8.0, 220.0))
//!     .landform(preset::mountains(60.0, 160.0).mask(Expr::field("mars:peaks")))
//!     .rule(BiomeRule::new("peaks").field("mars:peaks", 0.5..))
//!     .rule(BiomeRule::new("dust"));
//! // Another mod, whose savanna biome declares its climate and surface:
//! r.edit(OVERWORLD.terrain()).rule_before("base:desert", BiomeRule::new("savanna"));
//! ```

use std::ops::{Add, Bound, Mul, Neg, RangeBounds, Sub};

use crate::abi::{self, ExprStep as Step, expr as ek, rule_field as rf};
use crate::comps::terrain as t;
use crate::noise::fixed::{Fields, ONE, q4, q12, salt};
use crate::{Component, Dimension, Gen, Registry, WithComponents, callback, qualify};

/// chiacraft: our terrain components beside upstream's (`abi::comp::terrain`): the continent
/// scale, steep faces and far crowns' leaves for distant terrain, and CPU landforms' heights.
mod ours {
    use crate::Component;
    use crate::abi::{self, comp::terrain as t};
    use crate::kind::Terrain;
    pub static CONTINENT_SCALE: Component<f64, Terrain> = Component::named(t::CONTINENT_SCALE);
    pub static FACES: Component<abi::Callback, Terrain> = Component::named(t::FACES);
    pub static FACE_FALL: Component<i32, Terrain> = Component::named(t::FACE_FALL);
    pub static FAR_LEAVES: Component<abi::Callback, Terrain> = Component::named(t::FAR_LEAVES);
}

/// Surface `height` and `biome` id of a column.
pub type Column = abi::TerrainColumn;

fn step(kind: u32, a: i32) -> Step {
    Step { kind, a, ..Default::default() }
}

/// A number for every column of a terrain, worked out from where the column is: the height a
/// landform adds, the strength of a mask, a field biome rules ask for. Built from numbers, the
/// position and noise with `+`, `-`, `*` and the methods here; the engine runs it on the CPU
/// and the GPU with the same integer math, so every machine gets the same world.
///
/// Lengths (heights, positions, sizes) are in blocks, `Expr::blocks(12.0)`; everything else
/// (noise, strengths, factors) is a plain number, `Expr::value(0.5)`. Multiplying a length by a
/// number gives a length. Results run about -2048..2048 blocks or -64..64 as numbers; a factor
/// of a product is held to about +-11 (+-2896 blocks).
#[derive(Clone)]
pub struct Expr(Vec<Step>);

impl Expr {
    /// chiacraft: its steps as sent to the engine (`FeatureBuilder::thin`).
    pub(crate) fn into_steps(self) -> Vec<Step> {
        self.0
    }

    fn leaf(step: Step) -> Expr {
        Expr(vec![step])
    }

    fn then(mut self, step: Step) -> Expr {
        self.0.push(step);
        self
    }

    fn with(mut self, other: Expr, kind: u32) -> Expr {
        self.0.extend(other.0);
        self.then(step(kind, 0))
    }

    fn noise_raw(salt: u64, scale: i32, octaves: u32) -> Expr {
        Expr(vec![step(ek::X, 0), step(ek::Z, 0), Step { salt, b: octaves as i32, ..step(ek::NOISE, scale) }])
    }

    fn ramp_raw(self, from: i32, to: i32) -> Expr {
        self.then(Step { b: to, ..step(ek::RAMP, from) })
    }

    /// A plain number.
    pub fn value(v: f64) -> Expr {
        Expr::leaf(step(ek::CONST, q12(v)))
    }

    /// A length in blocks.
    pub fn blocks(b: f64) -> Expr {
        Expr::leaf(step(ek::CONST, q4(b)))
    }

    /// An integer as it is, for `idiv` and `modulo` by a count.
    pub fn int(n: i32) -> Expr {
        Expr::leaf(step(ek::CONST, n))
    }

    /// The column's position, a length.
    pub fn x() -> Expr {
        Expr::leaf(step(ek::X, 0))
    }

    pub fn z() -> Expr {
        Expr::leaf(step(ek::Z, 0))
    }

    /// A field of the terrain: `temperature`, `humidity`, `continent`, or one declared with
    /// `TerrainBuilder::field` before what uses it. Worked out once per column however many
    /// times it is used, so share costly parts through a field.
    pub fn field(name: &str) -> Expr {
        Expr::leaf(Step { name: name.to_string(), ..step(ek::FIELD, 0) })
    }

    /// About -1.4..1.4, cold to hot.
    pub fn temperature() -> Expr {
        Expr::field("temperature")
    }

    /// About -1.4..1.4, dry to wet.
    pub fn humidity() -> Expr {
        Expr::field("humidity")
    }

    /// Continentalness: below 0 is sea, coasts are about 0..0.1.
    pub fn continent() -> Expr {
        Expr::field("continent")
    }

    /// Noise about -1..1 in patches about `scale` blocks across, `octaves` layers of it each
    /// twice as fine and half as strong. Noises with the same `salt` and size line up.
    pub fn fbm(salt: u64, scale: f64, octaves: u32) -> Expr {
        Expr::noise_raw(salt, q4(scale), octaves)
    }

    /// `fbm` of 3 octaves with a salt of its own size.
    pub fn noise(scale: f64) -> Expr {
        Expr::fbm(scale.to_bits(), scale, 3)
    }

    /// 1 along lines where the noise crosses 0, falling to 0 a quarter of the way to its peaks:
    /// winding ridges or cuts `scale` blocks apart.
    pub fn ridge(salt: u64, scale: f64) -> Expr {
        Expr::value(1.0)
            - (Expr::fbm(salt, scale, 4) * Expr::value(2.0))
                .abs()
                .min(Expr::value(1.0))
    }

    /// Points scattered one per `spacing` x `spacing` square at most, each there with `chance`
    /// and `radius` blocks big times 0.5 to 1.5: the square of the distance to the nearest, in
    /// units of its radius (below 1 inside it), 2 or more where none is near.
    pub fn sites(salt: u64, spacing: f64, chance: f64, radius: f64) -> Expr {
        Expr(vec![step(ek::X, 0), step(ek::Z, 0)]).then(Step { salt, b: q12(chance), c: q4(radius), ..step(ek::SITE, q4(spacing)) })
    }

    /// This expression at another position: `noise.at(Expr::x() + warp, Expr::z())` bends it.
    /// Fields stay what they are at the column.
    pub fn at(self, x: Expr, z: Expr) -> Expr {
        let mut out = Vec::with_capacity(self.0.len());
        for s in self.0 {
            match s.kind {
                ek::X => out.extend(x.0.iter().cloned()),
                ek::Z => out.extend(z.0.iter().cloned()),
                _ => out.push(s),
            }
        }
        Expr(out)
    }

    pub fn abs(self) -> Expr {
        self.then(step(ek::ABS, 0))
    }

    pub fn min(self, other: Expr) -> Expr {
        self.with(other, ek::MIN)
    }

    pub fn max(self, other: Expr) -> Expr {
        self.with(other, ek::MAX)
    }

    pub fn clamp(self, lo: Expr, hi: Expr) -> Expr {
        self.max(lo).min(hi)
    }

    /// Integer division, toward zero: a length by `Expr::int(2)` is half as long.
    pub fn idiv(self, other: Expr) -> Expr {
        self.with(other, ek::DIV)
    }

    /// What is left over below `other` (a length or a count), never negative: repeats along
    /// the world.
    pub fn modulo(self, other: Expr) -> Expr {
        self.with(other, ek::REM)
    }

    /// 0 at `from`, 1 at `to` and straight in between, a strength for a mask; `from > to`
    /// turns it around.
    pub fn ramp(self, from: f64, to: f64) -> Expr {
        self.ramp_raw(q12(from), q12(to))
    }

    /// `ramp` of a length, `from` and `to` in blocks.
    pub fn ramp_blocks(self, from: f64, to: f64) -> Expr {
        self.ramp_raw(q4(from), q4(to))
    }

    /// 1 minus it, for masks.
    pub fn not(self) -> Expr {
        Expr::value(1.0) - self
    }

    /// Both masks: their product.
    pub fn and(self, other: Expr) -> Expr {
        self * other
    }
}

impl Add for Expr {
    type Output = Expr;
    fn add(self, other: Expr) -> Expr {
        self.with(other, ek::ADD)
    }
}

impl Sub for Expr {
    type Output = Expr;
    fn sub(self, other: Expr) -> Expr {
        self.with(other, ek::SUB)
    }
}

impl Mul for Expr {
    type Output = Expr;
    fn mul(self, other: Expr) -> Expr {
        self.with(other, ek::MUL)
    }
}

impl Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        Expr::int(0) - self
    }
}

/// Adds to the height of every column, scaled by its mask.
pub struct Landform {
    name: String,
    height: Expr,
    mask: Option<Expr>,
    /// chiacraft: `FeatureArgs`, a CPU height instead of `height`.
    custom: Option<abi::Callback>,
}

impl Landform {
    /// A landform adding `height` (a length) at every column.
    pub fn new(height: Expr) -> Landform {
        Landform { name: String::new(), height, mask: None, custom: None }
    }

    /// chiacraft: a landform of your own computed on the CPU: the height it adds at (x, z).
    /// Positions and the height are in 1/16 block and noise values in 1/4096 (`ONE`): terrain
    /// is integers only, so every machine agrees. It runs for every column, on many threads at
    /// once, and like a generator it must be a pure function of its input. Distant terrain asks
    /// the CPU for its heights; where an `Expr` can say it, `new` is cheaper.
    pub fn custom(height: impl Fn(&Fields, i32, i32) -> i32 + Send + Sync + 'static) -> Landform {
        let custom =
            callback(move |a: &mut abi::FeatureArgs| a.height = height(&Fields::with_continent_scale(a.seed, a.continent_scale), a.x, a.z));
        Landform { name: String::new(), height: Expr::int(0), mask: None, custom: Some(custom) }
    }

    /// Other mods find the landform by this name.
    pub fn named(mut self, name: &str) -> Landform {
        self.name = name.to_string();
        self
    }

    /// Where it applies, 0..1: nothing where it is 0 or less (and its height isn't worked out
    /// there), all of it from 1.
    pub fn mask(mut self, mask: Expr) -> Landform {
        self.mask = Some(mask);
        self
    }

    fn raw(self, ns: &str) -> abi::Landform {
        abi::Landform { name: qualify(ns, &self.name), height: self.height.0, mask: self.mask.map_or(Vec::new(), |m| m.0) }
    }
}

impl From<Expr> for Landform {
    fn from(height: Expr) -> Landform {
        Landform::new(height)
    }
}

/// Ready-made landforms. Each is `Landform::new` of its `shape`, which can be changed or
/// combined into a new kind.
pub mod preset {
    use super::*;

    /// Oceans and land masses: `depth` blocks above or below the base height.
    pub fn continents(depth: f64) -> Landform {
        Landform::new(shape::continents(depth))
    }

    /// Rolling hills.
    pub fn hills(height: f64, size: f64) -> Landform {
        Landform::new(shape::hills(height, size))
    }

    /// Mountain ranges along ridge lines `width` blocks apart, with rough detail on top.
    pub fn mountains(height: f64, width: f64) -> Landform {
        Landform::new(shape::mountains(height, width))
    }

    /// Flat-topped mesas `height` blocks high in terraces of `step`.
    pub fn plateaus(height: f64, size: f64, step: f64) -> Landform {
        Landform::new(shape::plateaus(height, size, step))
    }

    /// Winding cuts `depth` deep and about `width` wide; below sea level they fill with water.
    pub fn canyons(depth: f64, width: f64) -> Landform {
        Landform::new(shape::canyons(depth, width))
    }

    /// Parallel sand waves `wavelength` blocks apart.
    pub fn dunes(height: f64, wavelength: f64) -> Landform {
        Landform::new(shape::dunes(height, wavelength))
    }

    /// Bowls with a raised rim, one per `spacing`-block cell at most.
    pub fn craters(radius: f64, depth: f64, spacing: f64) -> Landform {
        Landform::new(shape::craters(radius, depth, spacing))
    }
}

/// The heights of the presets, as expressions.
pub mod shape {
    use super::*;

    pub fn continents(depth: f64) -> Expr {
        let c = (Expr::continent() * Expr::value(5.0)).idiv(Expr::int(2));
        c.clamp(Expr::value(-1.0), Expr::value(1.0)) * Expr::blocks(depth)
    }

    pub fn hills(height: f64, size: f64) -> Expr {
        Expr::fbm(salt::HILLS, size, 4) * Expr::blocks(height)
    }

    pub fn mountains(height: f64, width: f64) -> Expr {
        let r = || Expr::ridge(salt::MOUNTAINS, width);
        r() * r() * Expr::blocks(height) + (Expr::fbm(salt::HILLS, 60.0, 4) * Expr::blocks(height)).idiv(Expr::int(7))
    }

    pub fn plateaus(height: f64, size: f64, step: f64) -> Expr {
        let v = || Expr::fbm(salt::PLATEAUS, size, 3).ramp(0.0, 0.25) * Expr::blocks(height);
        v() - v().modulo(Expr::blocks(step))
    }

    pub fn canyons(depth: f64, width: f64) -> Expr {
        let scale = q4(width).min(1 << 20) * 40;
        let r = Expr::value(1.0)
            - (Expr::noise_raw(salt::CANYONS, scale, 4) * Expr::value(2.0))
                .abs()
                .min(Expr::value(1.0));
        -(r.ramp_raw(3994, ONE) * Expr::blocks(depth))
    }

    pub fn dunes(height: f64, wavelength: f64) -> Expr {
        let wl = q4(wavelength).clamp(1, (1 << 18) - 1);
        let t = || {
            let warp = Expr::noise_raw(salt::DUNES, wl * 4, 2) * Expr::int(wl);
            (Expr::x() + Expr::z().idiv(Expr::int(2)) + warp)
                .modulo(Expr::int(wl))
                .ramp_raw(0, wl)
        };
        // Gentle windward slope, steep lee side.
        t().ramp_raw(0, 3277).min(t().ramp_raw(ONE, 3277)) * Expr::blocks(height)
    }

    pub fn craters(radius: f64, depth: f64, spacing: f64) -> Expr {
        let d = || Expr::sites(salt::CRATERS, spacing, 0.6, radius);
        let bowl = d().ramp(1.0, 0.0) * Expr::blocks(depth);
        let rim = (d() - Expr::int(5325)).abs().ramp_raw(1229, 0) * Expr::blocks(depth);
        -bowl + rim.idiv(Expr::int(4))
    }
}

/// Where a biome stands in a terrain. Its climate and surface come from the biome
/// (`r.biome(&X).temperature(..).surface(..)`); a rule adds where else it must hold (height,
/// continent, any field) and sets only what differs here. Rules are tried in order; the first
/// match wins and the last one is used where none does.
#[derive(Clone)]
pub struct BiomeRule(abi::BiomeRule);

impl BiomeRule {
    /// A rule for `biome`, named like it.
    pub fn new(biome: &str) -> BiomeRule {
        BiomeRule(abi::BiomeRule { fields: rf::BIOME, biome: biome.to_string(), ..BiomeRule::blank().0 })
    }

    fn blank() -> BiomeRule {
        BiomeRule(abi::BiomeRule {
            height: [i32::MIN, i32::MAX],
            depth: 3,
            deep_below: i32::MIN,
            bare_above: i32::MAX,
            ..Default::default()
        })
    }

    /// A name of its own, for a second rule of the same biome (other mods find rules by name).
    pub fn named(mut self, name: &str) -> Self {
        self.0.name = name.to_string();
        self
    }
    /// In an edit: the rule picks this biome instead.
    pub fn biome(mut self, biome: &str) -> Self {
        (self.0.biome, self.0.fields) = (biome.to_string(), self.0.fields | rf::BIOME);
        self
    }
    /// Where the field (`temperature`, `humidity`, `continent` or one the terrain declares) is in
    /// `range`, like `0.35..` or `..-0.45` (from inclusive, to exclusive). Given again for the
    /// same field, the new range replaces the old.
    pub fn field(mut self, name: &str, range: impl RangeBounds<f64>) -> Self {
        let (min, max) = range_f64(range);
        match self.0.ranges.iter_mut().find(|r| r.name == name) {
            Some(r) => (r.min, r.max) = (min, max),
            None => self
                .0
                .ranges
                .push(abi::RuleRange { name: name.to_string(), min, max }),
        }
        self.0.fields |= rf::RANGES;
        self
    }
    /// Climate, about -1.4..1.4. Usually the biome's.
    pub fn temperature(self, range: impl RangeBounds<f64>) -> Self {
        self.field("temperature", range)
    }
    pub fn humidity(self, range: impl RangeBounds<f64>) -> Self {
        self.field("humidity", range)
    }
    /// Continentalness: below 0 is sea, coasts are about 0..0.1.
    pub fn continent(self, range: impl RangeBounds<f64>) -> Self {
        self.field("continent", range)
    }
    /// Surface heights, like `..=SEA + 1`.
    pub fn height(mut self, range: impl RangeBounds<i32>) -> Self {
        let (min, max) = range_i32(range);
        (self.0.height, self.0.fields) = ([min, max], self.0.fields | rf::HEIGHT);
        self
    }
    /// Top block, then `depth` blocks of filler, then the terrain's stone.
    pub fn surface(mut self, top: &str, filler: &str, depth: i32) -> Self {
        (self.0.top, self.0.filler, self.0.depth) = (top.to_string(), filler.to_string(), depth);
        self.0.fields |= rf::SURFACE;
        self
    }
    /// The lowest `layers` of the filler become `block` (sandstone under sand).
    pub fn under(mut self, block: &str, layers: i32) -> Self {
        (self.0.under, self.0.under_layers) = (block.to_string(), layers);
        self.0.fields |= rf::UNDER;
        self
    }
    /// Top block for columns lower than `below` (gravel on deep sea floors).
    pub fn deep_top(mut self, block: &str, below: i32) -> Self {
        (self.0.deep_top, self.0.deep_below) = (block.to_string(), below);
        self.0.fields |= rf::DEEP_TOP;
        self
    }
    /// Plain stone above this height (bare peaks).
    pub fn bare_above(mut self, y: i32) -> Self {
        (self.0.bare_above, self.0.fields) = (y, self.0.fields | rf::BARE_ABOVE);
        self
    }
    /// The water surface freezes into `block`.
    pub fn frozen(mut self, block: &str) -> Self {
        (self.0.ice, self.0.fields) = (block.to_string(), self.0.fields | rf::FROZEN);
        self
    }

    /// As sent: names qualified with `ns`, the rule named like its biome unless it has a name.
    pub(crate) fn qualified(&self, ns: &str) -> abi::BiomeRule {
        let r = &self.0;
        let q = |s: &str| qualify(ns, s);
        abi::BiomeRule {
            name: q(if r.name.is_empty() { &r.biome } else { &r.name }),
            biome: q(&r.biome),
            top: q(&r.top),
            filler: q(&r.filler),
            under: q(&r.under),
            deep_top: q(&r.deep_top),
            ice: q(&r.ice),
            ..r.clone()
        }
    }
}

pub(crate) fn range_f64(r: impl RangeBounds<f64>) -> (f64, f64) {
    let at = |b: Bound<&f64>, open: f64| match b {
        Bound::Included(v) | Bound::Excluded(v) => *v,
        Bound::Unbounded => open,
    };
    (at(r.start_bound(), f64::NEG_INFINITY), at(r.end_bound(), f64::INFINITY))
}

/// Inclusive.
fn range_i32(r: impl RangeBounds<i32>) -> (i32, i32) {
    let min = match r.start_bound() {
        Bound::Included(v) => *v,
        Bound::Excluded(v) => v + 1,
        Bound::Unbounded => i32::MIN,
    };
    let max = match r.end_bound() {
        Bound::Included(v) => *v,
        Bound::Excluded(v) => v - 1,
        Bound::Unbounded => i32::MAX,
    };
    (min, max)
}

/// Two-noise tunnels plus large caverns. Between `lava_from` and `lava_to` some layers of height
/// hold flat lava lakes.
pub struct Caves {
    lava: String,
    lava_from: i32,
    lava_to: i32,
}

impl Caves {
    pub fn new(lava: &str, lava_from: i32, lava_to: i32) -> Caves {
        Caves { lava: lava.to_string(), lava_from, lava_to }
    }
}

impl Registry {
    /// The terrain of your dimension; another mod's is `r.edit(OVERWORLD.terrain())`, applied
    /// after the owner's description in load order.
    pub fn terrain(&mut self, dimension: &Dimension) -> TerrainBuilder<'_> {
        self.terrain_entry(dimension.terrain())
    }

    fn terrain_entry(&mut self, terrain: &'static crate::Terrain) -> TerrainBuilder<'_> {
        TerrainBuilder { reg: self, e: crate::entry::Entry::of(terrain) }
    }
}

pub struct TerrainBuilder<'a> {
    reg: &'a mut Registry,
    e: crate::entry::Entry,
}

crate::entry::entry_builder!(
    Terrain => TerrainBuilder, terrain_entry;
);

impl TerrainBuilder<'_> {
    fn q(&self, name: &str) -> String {
        qualify(self.reg.ns, name)
    }

    fn rule_as_sent(&self, rule: BiomeRule) -> abi::BiomeRule {
        rule.qualified(self.reg.ns)
    }

    /// The block the ground is made of.
    pub fn stone(self, block: &str) -> Self {
        let block = self.q(block);
        self.set_text(&t::STONE, &block)
    }
    /// The ground height where no landform adds anything (64 unless said).
    pub fn base_height(self, y: f64) -> Self {
        self.set(&t::BASE_HEIGHT, y)
    }
    /// Everything up to `level` that isn't ground is `water`.
    pub fn sea(self, level: i32, water: &str) -> Self {
        let block = self.q(water);
        self.set(&t::SEA, abi::TerrainSea { level, block })
    }
    pub fn caves(self, caves: Caves) -> Self {
        let lava = self.q(&caves.lava);
        self.set(&t::CAVES, abi::TerrainCaves { lava, from: caves.lava_from, to: caves.lava_to })
    }
    /// A named value over the terrain's columns, for biome rules (`BiomeRule::field`) and
    /// other expressions (`Expr::field`), worked out once per column. The name is used as written:
    /// give yours your mod's prefix (`mymod:aridity`). Declaring `temperature`, `humidity`,
    /// `continent` or another mod's field again replaces it for everything that uses it.
    pub fn field(self, name: &str, value: Expr) -> Self {
        self.push(&t::FIELDS, abi::TerrainField { name: name.to_string(), value: value.0 })
    }
    pub fn landform(self, f: impl Into<Landform>) -> Self {
        let mut f = f.into();
        // chiacraft: a CPU landform's height goes as a callback under its name
        // (`comp::terrain::CPU_HEIGHT`), so it needs one: an unnamed one gets its own.
        let custom = f.custom.take();
        if custom.is_some() && f.name.is_empty() {
            static UNNAMED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            f.name = format!("cpu_landform_{}", UNNAMED.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        }
        let f = f.raw(self.reg.ns);
        let name = f.name.clone();
        let s = self.push(&t::LANDFORMS, f);
        match custom {
            Some(cb) => {
                // Registered once per landform at load: the component's name lives on.
                let key: &'static str = Box::leak(format!("{}{name}", abi::comp::terrain::CPU_HEIGHT).into_boxed_str());
                let comp: &'static Component<abi::Callback, crate::kind::Terrain> = Box::leak(Box::new(Component::named(key)));
                s.set(comp, cb)
            }
            None => s,
        }
    }
    pub fn landform_remove(self, name: &str) -> Self {
        let name = self.q(name);
        self.remove_item(&t::LANDFORMS, &name)
    }
    /// Limits the named landform further: its mask is multiplied by `mask`.
    pub fn landform_mask(self, name: &str, mask: Expr) -> Self {
        let name = self.q(name);
        self.push(&t::MASKS, abi::LandformMask { name, mask: mask.0 })
    }
    /// A rule after all others.
    pub fn rule(self, rule: BiomeRule) -> Self {
        let rule = self.rule_as_sent(rule);
        self.push(&t::RULES, rule)
    }
    /// A rule tried right before the rule named `rule` (a biome's rule is named like the biome).
    pub fn rule_before(self, rule: &str, new: BiomeRule) -> Self {
        let (anchor, new) = (self.q(rule), self.rule_as_sent(new));
        self.insert_before(&t::RULES, &anchor, new)
    }
    pub fn rule_after(self, rule: &str, new: BiomeRule) -> Self {
        let (anchor, new) = (self.q(rule), self.rule_as_sent(new));
        self.insert_after(&t::RULES, &anchor, new)
    }
    pub fn rule_replace(self, rule: &str, new: BiomeRule) -> Self {
        let (anchor, new) = (self.q(rule), self.rule_as_sent(new));
        self.replace_item(&t::RULES, &anchor, new)
    }
    pub fn rule_remove(self, rule: &str) -> Self {
        let anchor = self.q(rule);
        self.remove_item(&t::RULES, &anchor)
    }
    /// Changes what `f` sets of the rule named `rule`, the rest stays:
    /// `.rule_edit("base:desert", |r| r.height(..=90))`.
    pub fn rule_edit(self, rule: &str, f: impl FnOnce(BiomeRule) -> BiomeRule) -> Self {
        let mut edit = self.rule_as_sent(f(BiomeRule::blank()));
        edit.name = self.q(rule);
        self.push(&t::RULE_EDITS, edit)
    }
    /// A biome in 3D: whole cubes at least `depth` blocks under the surface where a 3D noise of
    /// patches about `size` blocks across is above `threshold` (-1..1). A threshold of -1
    /// matches any cube deep enough, as a fallback after patch rules. Biomes are kept per
    /// column of each cube, so underground ones change every 16 blocks of height at most.
    pub fn cave_biome(self, biome: &str, depth: i32, size: f64, threshold: f64) -> Self {
        let name = self.q(biome);
        self.push(&t::CAVE_BIOMES, abi::CaveBiome { name, depth, size, threshold })
    }
    pub fn cave_biome_remove(self, biome: &str) -> Self {
        let name = self.q(biome);
        self.remove_item(&t::CAVE_BIOMES, &name)
    }

    /// chiacraft: how many blocks across land masses and seas are (the continent noise's scale;
    /// 900 when not given). Masks and rules on continentalness follow it.
    pub fn continent_scale(self, blocks: f64) -> Self {
        self.set(&ours::CONTINENT_SCALE, blocks)
    }

    /// chiacraft: what steep faces show, for distant terrain, which runs no decorations: where
    /// the ground falls `fall` blocks or more to its lowest neighbour, `f(face)` turns the block
    /// the surface rules put there into the one the generator's own passes would (bare rock). No
    /// scattered things stand on such faces far away. Pure, called from many threads.
    pub fn faces(self, fall: i32, f: impl Fn(&mut Face) + Send + Sync + 'static) -> Self {
        let face = callback(move |a: &mut abi::FaceArgs| f(Face::wrap(a)));
        self.set(&ours::FACE_FALL, fall).set(&ours::FACES, face)
    }

    /// chiacraft: the state far trees' crowns are drawn in at a time of year: `f(time, state)`
    /// gives it for leaves of `state` at the world's day time `time`. Distant terrain makes its
    /// crowns from sample trees once, so leaves that change with the seasons (a bloom) answer here,
    /// and the far tiles are made again when an answer changes. Pure, called from many threads.
    pub fn far_leaves(self, f: impl Fn(u64, u32) -> u32 + Send + Sync + 'static) -> Self {
        let call = callback(move |a: &mut abi::FarLeavesArgs| a.state = f(a.time, a.state));
        self.set(&ours::FAR_LEAVES, call)
    }
}

/// chiacraft: a steep face's block as distant terrain asks for it (`TerrainBuilder::faces`).
#[repr(transparent)]
pub struct Face {
    args: abi::FaceArgs,
}

impl Face {
    fn wrap(a: &mut abi::FaceArgs) -> &mut Face {
        unsafe { &mut *(a as *mut abi::FaceArgs as *mut Face) }
    }
    pub fn seed(&self) -> u64 {
        self.args.seed
    }
    pub fn dimension(&self) -> u32 {
        self.args.dim
    }
    /// The block on the face: its column's top, or one lower down it.
    pub fn pos(&self) -> [i32; 3] {
        self.args.pos
    }
    pub fn biome(&self) -> u32 {
        self.args.biome
    }
    /// The column's ground height (its top block's y).
    pub fn height(&self) -> i32 {
        self.args.height
    }
    /// The block the terrain leaves at the top of the column.
    pub fn surface_block(&self) -> u32 {
        let [x, _, z] = self.args.pos;
        crate::Generation::new(abi::GenSource {
            dim: self.args.dim,
            seed: self.args.seed,
            before: abi::stage::UNDERGROUND,
            ctx: std::ptr::null(),
        })
        .surface_block(x, z)
    }
    /// How far the ground falls to its lowest neighbour.
    pub fn fall(&self) -> i32 {
        self.args.fall
    }
    /// The block the surface rules put there.
    pub fn block(&self) -> u32 {
        self.args.block
    }
    /// What the face shows instead.
    pub fn set(&mut self, block: u32) {
        self.args.block = block;
    }
}

impl Dimension {
    /// Inspect the deterministic result before `stage` (use `abi::stage::POST_PROCESS`
    /// for terrain, structures and vegetation), without loading the live world.
    pub fn generation(&self, seed: u64, stage: u32) -> crate::Generation<'_> {
        crate::Generation::new(abi::GenSource { dim: self.id(), seed, before: stage, ctx: std::ptr::null() })
    }
    /// Height and biome of column (`x`, `z`) for `seed`, generated or not: from the terrain or
    /// the dimension's `column`. A dimension with neither gives height `i32::MIN`.
    pub fn column(&self, seed: u64, x: i32, z: i32) -> Column {
        self.generation(seed, abi::stage::UNDERGROUND).column(x, z)
    }

    /// Whether caves took the surface block of column (`x`, `z`), so nothing should stand there.
    pub fn surface_carved(&self, seed: u64, x: i32, z: i32) -> bool {
        self.generation(seed, abi::stage::UNDERGROUND)
            .surface_carved(x, z)
    }
}

impl Gen {
    /// The dimension being generated.
    pub fn dimension(&self) -> u32 {
        self.args.dim
    }

    /// The surface of the cube's 256 columns, index `z * 16 + x`: from the terrain or the
    /// dimension's `column`. None for a dimension with neither.
    pub fn columns(&self) -> Option<&[Column; 256]> {
        unsafe { self.args.columns.cast::<[Column; 256]>().as_ref() }
    }

    /// Any column of this dimension, in world coordinates.
    pub fn column(&self, x: i32, z: i32) -> Column {
        self.view().column(x, z)
    }

    /// Whether caves took the surface block of column (`x`, `z`), in world coordinates.
    pub fn surface_carved(&self, x: i32, z: i32) -> bool {
        self.view().surface_carved(x, z)
    }

    /// The block at the terrain's reference surface, in this callback's generation view.
    pub fn surface_block(&self, x: i32, z: i32) -> u32 {
        self.view().surface_block(x, z)
    }
}

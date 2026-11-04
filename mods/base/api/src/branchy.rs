//! Trees grown from branches (the world-looks round's first change, after Vintage Story's tree
//! generator): a trunk grows up in half-block steps, losing width as it goes, drifting a little
//! and drooping under its own weight; branches leave it at set heights, spiralling round it (the
//! golden angle) or in whorls, each a trunk of its own that thins to nothing over its length;
//! leaves gather where the wood is thin. A species is a `Recipe` (`tree_recipes.rs`): an oak's few
//! big limbs and lumpy crown, a ginkgo's narrow open one, a spruce's drooping cone, an acacia's
//! or a mesquite's low fork under a flat parasol, a palm's curved trunk and fronds, an araucaria's
//! tall bare trunk under flat whorls.
//!
//! - Positions are relative to the base: the trunk's foot is the block just above the ground at
//!   (0, 0, 0) (a 2 x 2 trunk covers x and z 0 and 1). The caller puts the dirt under it.
//! - Every block lies within `REACH` sideways of the base (the scatter's reach: more is left out)
//!   and below `MAX_HEIGHT`, or the recipe's own `reach` and `max_height` (a giant sequoia's).
//! - A trunk 1.5 wide or more is 2 x 2; from `ROUND` wide it is a round disc each step, flaring
//!   at the foot (`flare`) into buttresses (`lobes`).
//! - The same recipe and seed give the same blocks on every machine: only +, -, *, / and sqrt
//!   (exactly rounded everywhere), the angles from a polynomial, the randomness from splitmix64.
//! - Wood is joined face to face (a diagonal step is bridged), carries the axis it runs along, and
//!   wins over leaves where both fall in a block.

/// What a block of a grown tree is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    /// Wood, lying along `axis` (0 x, 1 y, 2 z).
    Log {
        axis: u8,
    },
    Leaves,
}

/// How far a tree reaches sideways from its base (the worldgen scatter's reach).
pub const REACH: i32 = 7;
/// How high a tree may grow above its base.
pub const MAX_HEIGHT: i32 = 24;
/// From this width a trunk is a round disc each step, not a 2 x 2.
pub const ROUND: f32 = 2.5;

/// How a species grows. Lengths in blocks, angles in degrees from straight up.
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    /// The log it grows from (`base:oak_log`): `tree_recipes::of` finds a recipe by it.
    pub log: &'static str,
    /// The trunk's height before it gives out, and how much a tree varies in size (a fraction:
    /// its height, its width and its branches together).
    pub height: f32,
    pub size_var: f32,
    /// Width at the base (1: a block; 1.5 and over: 2 x 2) and how much the trunk loses a block
    /// of growth; leaves gather where a branch is under `leaf_width`.
    pub width: f32,
    pub width_loss: f32,
    pub leaf_width: f32,
    /// Random turn a step (degrees), and a steady lean of the trunk (degrees a block) toward a
    /// direction random per tree; `rise` turns the trunk back upright (1 straight, less curves).
    pub drift: f32,
    pub lean: f32,
    pub rise: f32,
    /// How hard thin branches bend down (negative: their tips turn up).
    pub droop: f32,
    /// Branches: where they start up the trunk (a share of its height), every how many blocks,
    /// how many at a time (1: a spiral by the golden angle; more: a whorl), their angle from
    /// upright, their width against the trunk's there, and the deepest order (1: off the trunk
    /// only; 0 none).
    pub branch_start: f32,
    pub branch_every: f32,
    pub branch_count: u32,
    pub branch_angle: f32,
    pub branch_width: f32,
    pub orders: u32,
    /// A branch's length against the trunk's height, and how much shorter nearer the top (1: a
    /// cone; 0: all alike).
    pub branch_len: f32,
    pub taper: f32,
    /// Whether the trunk ends at its first branching, in a fork of leaders (acacia, mesquite).
    pub fork: bool,
    /// Leaf clusters: radius across and up (blocks), how ragged their edge (0..1), and whether
    /// leaves line the whole branch and the upper trunk (a conifer's needles) or only its thin end.
    pub leaf_r: f32,
    pub leaf_ry: f32,
    pub ragged: f32,
    pub leaf_along: bool,
    /// Fronds at the top (palms, tree ferns): how many, how long, and how hard they droop.
    pub fronds: u32,
    pub frond_len: f32,
    pub frond_droop: f32,
    /// Its own reach sideways and height (0: `REACH`, `MAX_HEIGHT`), for the giants; its
    /// scatter must reach as far.
    pub reach: i32,
    pub max_height: i32,
    /// A round trunk's foot: wider by `flare` (a share) at the ground, the flare gone `flare_up`
    /// blocks up, and ridged there into four buttresses `lobes` deep (a share of the radius).
    pub flare: f32,
    pub flare_up: f32,
    pub lobes: f32,
}

impl Recipe {
    /// What every recipe starts from (`..Recipe::BASE`).
    pub const BASE: Recipe = Recipe {
        log: "",
        height: 6.0,
        size_var: 0.2,
        width: 1.0,
        width_loss: 0.08,
        leaf_width: 0.55,
        drift: 6.0,
        lean: 0.0,
        rise: 1.0,
        droop: 0.0,
        branch_start: 0.5,
        branch_every: 1.2,
        branch_count: 1,
        branch_angle: 50.0,
        branch_width: 0.7,
        orders: 2,
        branch_len: 0.55,
        taper: 0.0,
        fork: false,
        leaf_r: 2.0,
        leaf_ry: 1.6,
        ragged: 0.35,
        leaf_along: false,
        fronds: 0,
        frond_len: 0.0,
        frond_droop: 0.42,
        reach: 0,
        max_height: 0,
        flare: 0.0,
        flare_up: 1.0,
        lobes: 0.0,
    };

    /// How far it reaches sideways and up (`reach`, `max_height`, or the defaults).
    pub fn limits(&self) -> (i32, i32) {
        (if self.reach > 0 { self.reach } else { REACH }, if self.max_height > 0 { self.max_height } else { MAX_HEIGHT })
    }
}

// ------------------------------------------------------------------ deterministic numbers

/// splitmix64: a small random stream, the same on every machine.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// 0..1.
    fn f(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    /// -1..1.
    fn s(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
}

/// sin of `deg` degrees from basic arithmetic only (range-reduced, then a 9th-order odd
/// polynomial on -90..90: under 1e-5 off), so every machine agrees bit for bit.
fn sin_deg(deg: f32) -> f32 {
    let mut a = deg % 360.0;
    if a < 0.0 {
        a += 360.0;
    }
    // to -180..180, then fold into -90..90
    if a > 180.0 {
        a -= 360.0;
    }
    if a > 90.0 {
        a = 180.0 - a;
    } else if a < -90.0 {
        a = -180.0 - a;
    }
    let x = a * (std::f32::consts::PI / 180.0);
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}

fn cos_deg(deg: f32) -> f32 {
    sin_deg(deg + 90.0)
}

type V = [f32; 3];

fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mul(a: V, k: f32) -> V {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn norm(a: V) -> V {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-6);
    mul(a, 1.0 / l)
}
/// A direction `angle` degrees from straight up, turned `yaw` degrees round it.
fn dir(angle: f32, yaw: f32) -> V {
    let s = sin_deg(angle);
    [s * cos_deg(yaw), cos_deg(angle), s * sin_deg(yaw)]
}
/// The axis a direction runs most along.
fn axis_of(d: V) -> u8 {
    let (x, y, z) = (d[0].abs(), d[1].abs(), d[2].abs());
    if y >= x && y >= z {
        1
    } else if x >= z {
        0
    } else {
        2
    }
}
fn block(p: V) -> (i32, i32, i32) {
    (p[0].floor() as i32, p[1].floor() as i32, p[2].floor() as i32)
}

// ------------------------------------------------------------------ growing

/// The blocks so far: each block's part, wood over leaves; and the tree's reach and height.
struct Out(std::collections::HashMap<(i32, i32, i32), Part>, i32, i32);

impl Out {
    fn put(&mut self, k: (i32, i32, i32), part: Part) {
        if k.0.abs() > self.1 + 1 || k.2.abs() > self.1 + 1 || k.1 < 0 || k.1 >= self.2 {
            return; // trimmed to the reach exactly in `blocks`, once the 2 x 2's offset is known
        }
        match (self.0.get(&k), part) {
            (Some(Part::Log { .. }), Part::Leaves) => {}
            _ => {
                self.0.insert(k, part);
            }
        }
    }
    /// Wood at `p` (`thick`: 2 x 2 round it), joined face to face to `last`. Returns `p`'s block.
    fn wood(&mut self, p: V, d: V, thick: bool, last: Option<(i32, i32, i32)>) -> (i32, i32, i32) {
        self.wood_round(p, d, if thick { 1.5 } else { 1.0 }, 0.0, 0.0, last)
    }

    /// Wood at `p` `width` wide: a block, a 2 x 2 from 1.5, a disc that wide from `ROUND`,
    /// ridged into four buttresses `lobes` deep turned `turn` degrees. Returns `p`'s block.
    fn wood_round(&mut self, p: V, d: V, width: f32, lobes: f32, turn: f32, last: Option<(i32, i32, i32)>) -> (i32, i32, i32) {
        if width >= ROUND {
            let k = block(p);
            let axis = axis_of(d);
            let r = width * 0.5;
            let reach = (r * (1.0 + lobes)).ceil() as i32 + 1;
            let (c, s) = (cos_deg(turn), sin_deg(turn));
            for dz in -reach..=reach {
                for dx in -reach..=reach {
                    let (x, z) = ((k.0 + dx) as f32 + 0.5 - p[0], (k.2 + dz) as f32 + 0.5 - p[2]);
                    let d2 = x * x + z * z;
                    // cos 4θ from the direction alone (θ turned), so the ridges need no angles.
                    let (u, v) = (x * c + z * s, z * c - x * s);
                    let cos4 = if d2 > 1e-6 { (u * u * u * u - 6.0 * u * u * v * v + v * v * v * v) / (d2 * d2) } else { 0.0 };
                    let edge = r * (1.0 + lobes * cos4.max(0.0));
                    if d2 <= edge * edge {
                        self.put((k.0 + dx, k.1, k.2 + dz), Part::Log { axis });
                    }
                }
            }
            return k;
        }
        let thick = width >= 1.5;
        let k = block(p);
        let axis = axis_of(d);
        if let Some(mut c) = last {
            while c != k {
                // one axis at a time: up the trunk first, then x, then z
                let step = if c.1 != k.1 {
                    c.1 += (k.1 - c.1).signum();
                    1
                } else if c.0 != k.0 {
                    c.0 += (k.0 - c.0).signum();
                    0
                } else {
                    c.2 += (k.2 - c.2).signum();
                    2
                };
                self.put(c, Part::Log { axis: if c == k { axis } else { step } });
            }
        }
        if thick {
            for (dx, dz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                self.put(block(add(p, [dx, 0.0, dz])), Part::Log { axis });
            }
        } else {
            self.put(k, Part::Log { axis });
        }
        k
    }
    /// A cluster of leaves round `c`: an ellipsoid `r` across and `ry` up, its outer shell ragged
    /// by a hash of each block (the same tree grows the same holes).
    fn cluster(&mut self, c: V, r: f32, ry: f32, ragged: f32, seed: u64) {
        let (ri, ryi) = (r.ceil() as i32, ry.ceil() as i32);
        let b = block(c);
        for dy in -ryi..=ryi {
            for dz in -ri..=ri {
                for dx in -ri..=ri {
                    let k = (b.0 + dx, b.1 + dy, b.2 + dz);
                    let (px, py, pz) = (k.0 as f32 + 0.5 - c[0], k.1 as f32 + 0.5 - c[1], k.2 as f32 + 0.5 - c[2]);
                    let d = (px * px + pz * pz) / (r * r) + py * py / (ry * ry);
                    if d > 1.0 {
                        continue;
                    }
                    let h = Rng(seed
                        ^ (k.0 as i64 as u64).wrapping_mul(73_856_093)
                        ^ (k.1 as i64 as u64).wrapping_mul(19_349_663)
                        ^ (k.2 as i64 as u64).wrapping_mul(83_492_791))
                    .next();
                    // Holes: most in the outer shell, some deeper in (so a crown isn't a wall).
                    let hole = ((h >> 20) % 1000) as f32 / 1000.0;
                    if d > 0.55 && hole < ragged || d > 0.25 && hole < ragged * 0.35 {
                        continue; // a hole in the outer shell
                    }
                    self.put(k, Part::Leaves);
                }
            }
        }
    }
}

const STEP: f32 = 0.5;
/// Wood thinner than this is twig, left to the leaves.
const MIN_WOOD: f32 = 0.08;

struct Grow<'a> {
    r: &'a Recipe,
    rng: Rng,
    out: Out,
    seed: u64,
    size: f32,
    /// How the buttresses are turned (degrees).
    turn: f32,
}

impl Grow<'_> {
    /// One limb from `p` going `d`, `width` wide, of `order` (0 the trunk), `len` at most.
    fn limb(&mut self, mut p: V, mut d: V, mut width: f32, order: u32, len: f32, lean: V) {
        let r = *self.r;
        let mut grown = 0.0;
        let mut next_branch = if order == 0 { r.branch_start * len } else { f32::MAX };
        let mut spiral = self.rng.f() * 360.0;
        let mut last = None;
        let mut needled = None; // the block the last needle cluster sat on: once a block is enough
        // A branch thins to nothing over its own length; the trunk by the recipe's loss.
        let loss = if order == 0 { r.width_loss / self.size } else { width / len.max(STEP) };
        // Under needles (a conifer) the twigs and the trunk's top are all needles, no bare wood.
        let min_wood = if r.leaf_along { 0.25 } else { MIN_WOOD };
        while grown < len && width > 0.0 {
            let needle_top = r.leaf_along && order == 0 && grown > len * 0.85;
            if (width >= min_wood || order == 0) && !needle_top {
                // A round trunk flares at its foot into buttresses (`flare`, `lobes`).
                let foot = if order == 0 { (1.0 - grown / r.flare_up.max(STEP)).max(0.0) } else { 0.0 };
                let w = width * (1.0 + r.flare * foot * foot);
                last = Some(if w >= ROUND {
                    self.out
                        .wood_round(p, d, w, r.lobes * foot, self.turn, last)
                } else {
                    self.out.wood(p, d, width >= 1.5, last)
                });
            }
            if r.leaf_along && (order > 0 || grown > len * 0.3) && needled != Some(block(p)) {
                needled = Some(block(p));
                let (lr, lry) = if order == 0 { (1.4, 1.0) } else { (r.leaf_r, r.leaf_ry) };
                self.out.cluster(p, lr, lry, r.ragged, self.seed); // needles on the twigs and the upper trunk
            }
            if order < r.orders && grown >= next_branch {
                let n = r.branch_count.max(1);
                let up = grown / len;
                for i in 0..n {
                    let yaw = if n == 1 { spiral } else { spiral + 360.0 * i as f32 / n as f32 };
                    let angle = r.branch_angle + self.rng.s() * 10.0;
                    let mut bd = dir(angle, yaw);
                    if order > 0 {
                        bd = norm(add(bd, mul(d, 0.6))); // a twig follows its branch's way
                    }
                    let blen = (len * r.branch_len).max(1.5) * (1.0 - r.taper * up).max(0.15) * (0.8 + 0.4 * self.rng.f());
                    let blen = if order == 0 { blen } else { blen.min((len - grown) * 0.8 + 1.0) };
                    self.limb(p, bd, width.min(1.4) * r.branch_width, order + 1, blen, [0.0; 3]);
                }
                spiral += 137.5;
                next_branch = grown + r.branch_every * (0.8 + 0.4 * self.rng.f());
                if r.fork && order == 0 {
                    break; // the trunk ends in its leaders
                }
            }
            // A step on: drift, droop where thin, the trunk turning back upright.
            let thin = (1.0 - width / r.width).clamp(0.0, 1.0);
            let turn = r.drift * (std::f32::consts::PI / 180.0) * STEP;
            d = add(d, [self.rng.s() * turn, self.rng.s() * turn * 0.3, self.rng.s() * turn]);
            d = add(d, lean);
            if order == 0 {
                d = add(d, [0.0, 0.25 * r.rise * STEP, 0.0]);
            } else {
                d = add(d, [0.0, -r.droop * thin * STEP, 0.0]);
            }
            d = norm(d);
            p = add(p, mul(d, STEP));
            grown += STEP;
            width -= loss * STEP;
        }
        if !(r.leaf_along && order > 0) && (order > 0 || r.orders == 0 && r.fronds == 0 || r.fork) {
            self.out
                .cluster(p, r.leaf_r, r.leaf_ry, r.ragged, self.seed); // leaves where the wood gave out
        }
        if order == 0 && r.fronds > 0 {
            self.fronds(p);
        }
        if order == 0 && !r.fork && r.orders > 0 && !r.leaf_along {
            self.out
                .cluster(p, r.leaf_r * 0.9, r.leaf_ry, r.ragged, self.seed); // a crown on the leader
        }
        if order == 0 && r.leaf_along {
            self.out.cluster(p, 0.9, 1.4, 0.0, self.seed); // a conifer's tip
        }
    }

    /// A palm's crown: fronds out from the top, rising then drooping to their tips, and a tuft.
    fn fronds(&mut self, top: V) {
        let r = *self.r;
        self.out.cluster(top, 1.0, 0.9, 0.0, self.seed);
        for i in 0..r.fronds {
            let yaw = 360.0 * i as f32 / r.fronds as f32 + self.rng.s() * 12.0;
            let mut d = dir(62.0 + self.rng.s() * 8.0, yaw);
            let mut p = top;
            let mut l = 0.0;
            while l < r.frond_len * self.size {
                self.out.put(block(p), Part::Leaves);
                d = norm(add(d, [0.0, -r.frond_droop * STEP, 0.0]));
                p = add(p, mul(d, STEP));
                l += STEP;
            }
        }
    }
}

/// The blocks of one tree of `recipe` from `seed`, relative to its base (see the module's notes),
/// in a fixed order.
pub fn blocks(recipe: &Recipe, seed: u64) -> Vec<(i32, i32, i32, Part)> {
    let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ 0x5DEE_CE66);
    let size = 1.0 + recipe.size_var * rng.s();
    let height = recipe.height * size;
    let lean_yaw = rng.f() * 360.0;
    let lean = mul(dir(90.0, lean_yaw), sin_deg(recipe.lean) * STEP);
    let start = dir(recipe.lean * 0.5, lean_yaw);
    let width = recipe.width * size;
    let (reach, max_height) = recipe.limits();
    // A 2 x 2 covers 0 and 1; a block and a round trunk stand on the base block's middle.
    let two = (1.5..ROUND).contains(&width);
    let foot = if two { [1.0, 0.0, 1.0] } else { [0.5, 0.0, 0.5] };
    let turn = if recipe.lobes > 0.0 { rng.f() * 90.0 } else { 0.0 };
    let mut g = Grow { r: recipe, rng, out: Out(Default::default(), reach, max_height), seed, size, turn };
    g.limb(foot, start, width, 0, height.min(max_height as f32 - 2.0), lean);
    // The reach is measured from the trunk's middle: a 2 x 2's is at 1, so 1 more on the plus side.
    let lo = -reach;
    let hi = reach + if two { 1 } else { 0 };
    let mut out: Vec<_> = g
        .out
        .0
        .into_iter()
        .filter(|&((x, _, z), _)| x >= lo && x <= hi && z >= lo && z <= hi)
        .map(|((x, y, z), p)| (x, y, z, p))
        .collect();
    out.sort_unstable_by_key(|b| (b.0, b.1, b.2)); // a fixed order, whatever the map's
    out
}

/// Grows a tree of `recipe` at (`x`, `y`, `z`) (its base) from `seed`: `put(x, y, z, part)` for
/// each block, wood and leaves both (the caller decides what it may replace). Returns how many.
pub fn grow(recipe: &Recipe, x: i32, y: i32, z: i32, seed: u64, mut put: impl FnMut(i32, i32, i32, Part)) -> usize {
    let b = blocks(recipe, seed);
    for &(dx, dy, dz, p) in &b {
        put(x + dx, y + dy, z + dz, p);
    }
    b.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_recipes::RECIPES;

    #[test]
    fn every_species_grows_a_tree_within_its_reach() {
        for r in RECIPES {
            let (reach, max_height) = r.limits();
            for seed in 0..30 {
                let t = blocks(r, seed);
                let name = r.log;
                assert!(
                    t.iter()
                        .any(|&(x, y, z, p)| (x, y, z) == (0, 0, 0) && matches!(p, Part::Log { .. })),
                    "{name} {seed}: no trunk at the base"
                );
                assert!(t.iter().filter(|b| b.3 == Part::Leaves).count() > 8, "{name} {seed}: too few leaves");
                assert!(t.iter().all(|b| b.1 >= 0 && b.1 < max_height), "{name} {seed}: out of its height");
                assert!(
                    t.iter()
                        .all(|b| b.0 >= -reach && b.0 <= reach + 1 && b.2 >= -reach && b.2 <= reach + 1),
                    "{name} {seed}: out of reach"
                );
                // A scatter's shape keeps 4,096 blocks; past that the engine drops the rest
                // (`dim.rs`), and a giant would lose its crown (the sequoia did).
                assert!(t.len() <= 4096, "{name} {seed}: {} blocks, over a shape's 4,096", t.len());
            }
        }
    }

    #[test]
    fn wood_is_joined_face_to_face() {
        for r in RECIPES {
            let (reach, _) = r.limits();
            for seed in 0..10 {
                let t = blocks(r, seed);
                let wood: std::collections::BTreeSet<_> = t
                    .iter()
                    .filter(|b| matches!(b.3, Part::Log { .. }))
                    .map(|b| (b.0, b.1, b.2))
                    .collect();
                // every wood block is reachable from the base through wood sharing a face
                let mut seen = std::collections::BTreeSet::new();
                let mut todo = vec![(0, 0, 0)];
                while let Some(c) = todo.pop() {
                    if !wood.contains(&c) || !seen.insert(c) {
                        continue;
                    }
                    for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                        todo.push((c.0 + dx, c.1 + dy, c.2 + dz));
                    }
                }
                // Wood cut off by the reach may be left alone; nothing else.
                let loose: Vec<_> = wood
                    .iter()
                    .filter(|c| !seen.contains(c) && c.0.abs() < reach && c.2.abs() < reach)
                    .collect();
                assert!(loose.is_empty(), "{} {seed}: loose wood {loose:?}", r.log);
            }
        }
    }

    #[test]
    fn the_same_seed_grows_the_same_tree() {
        let oak = crate::tree_recipes::of("base:oak_log").unwrap();
        assert_eq!(blocks(oak, 7), blocks(oak, 7));
        assert_ne!(blocks(oak, 7), blocks(oak, 8));
    }

    #[test]
    fn angles_come_from_arithmetic_alone() {
        for deg in [0.0f32, 30.0, 45.0, 90.0, 137.5, 180.0, 270.0, -45.0, 400.0] {
            assert!((sin_deg(deg) - deg.to_radians().sin()).abs() < 1e-4, "{deg}");
        }
    }
}

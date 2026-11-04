//! Trees by space colonization (Runions, Lane and Prusinkiewicz 2007, "Modeling Trees with a Space
//! Colonization Algorithm"): a cloud of points fills the species' crown shape (an oak's broad dome,
//! a spruce's cone, a birch's narrow oval); branches grow a step at a time toward the points near
//! them, a point is used up when a branch reaches it, and the wood is thickened by what it carries
//! (the pipe model: a limb's area is its twigs' areas added). Leaves sit at the twigs. A study next
//! to `branchy` (Ronald, 11-05: "pls google a better one"), on the not-to-merge branch.
//!
//! - Relative to the base like `branchy::blocks`: the trunk's foot is the block at (0, 0, 0).
//! - The same recipe and seed give the same blocks on every machine: +, -, *, / and sqrt only,
//!   splitmix64 for the randomness, every list in a fixed order.

use crate::branchy::{MAX_HEIGHT, Part, REACH};
use std::collections::HashMap;

/// The crown's shape: its radius at each height through it (0 at the crown's foot, 1 at its top),
/// as a share of `crown_r`.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// An oak's: widest a little under half way, flattened on top.
    Dome,
    /// A spruce's: widest at the foot, to a point.
    Cone,
    /// A birch's: an upright ellipse.
    Oval,
    /// An umbrella thorn's: a wide flat disc on top, nothing under it (the stems fan out to it).
    Umbrella,
}

impl Shape {
    fn radius(self, v: f32) -> f32 {
        if !(0.0..=1.0).contains(&v) {
            return 0.0;
        }
        match self {
            Shape::Dome => {
                let t = (v - 0.4) / if v < 0.4 { 0.45 } else { 0.62 };
                (1.0 - t * t).max(0.0).sqrt()
            }
            Shape::Cone => 1.0 - 0.92 * v,
            Shape::Umbrella => {
                if v < 0.55 {
                    0.0
                } else {
                    let t = (v - 0.8) / 0.25;
                    (1.0 - t * t).max(0.0).sqrt()
                }
            }
            Shape::Oval => {
                let t = 2.0 * v - 1.0;
                (1.0 - t * t).max(0.0).sqrt()
            }
        }
    }
}

/// How a species grows by colonization. Lengths in blocks.
#[derive(Clone, Copy, Debug)]
pub struct Colonize {
    pub log: &'static str,
    /// The bare trunk under the crown, the crown's height and radius, and how much a tree varies.
    pub trunk: f32,
    pub crown_h: f32,
    pub crown_r: f32,
    pub size_var: f32,
    pub shape: Shape,
    /// Points in the crown; a branch sees points within `influence`, a point is used up within
    /// `kill` of a branch; a branch grows `step` a time.
    pub points: u32,
    pub influence: f32,
    pub kill: f32,
    pub step: f32,
    /// A pull on every step (y up: positive grows up, negative droops).
    pub tropism: f32,
    /// A twig's radius (the pipe model adds areas from it down), and the foot's least width.
    pub twig_r: f32,
    pub min_trunk: f32,
    /// Leaf clusters at each twig: radius across and up, how ragged.
    pub leaf_r: f32,
    pub leaf_ry: f32,
    pub ragged: f32,
    /// Needles along every thin limb (a spruce's), not only at its tip.
    pub leaf_along: bool,
    /// Wood thinner than this is twig: leaves, no block (Vintage Story's logs stop at 0.3 wide,
    /// its branchy leaves carry the twigs).
    pub twig: f32,
    /// The trunk goes on to the crown's top as a leader the branches grow from (a birch's), not
    /// only to the crown's foot (an oak's, which divides into its limbs).
    pub leader: bool,
    /// Stems the trunk divides into at its top before the crown grows (an acacia's 3 or 4 fanning
    /// out like a vase); 0 none.
    pub fork: u32,
    /// Its own reach sideways and height (0: `branchy::REACH`, `MAX_HEIGHT`), for the canopy trees
    /// and the giants; their scatter must reach as far.
    pub reach: i32,
    pub max_height: i32,
    /// A trunk grown by this `branchy` recipe (its foot, flare and buttresses), the crown grown by
    /// colonization from its top (the Brazil nut's: Plants keeps its trunk); its height is the
    /// trunk recipe's own, cut where the crown starts (`trunk` as a share of it).
    pub trunk_from: Option<&'static crate::branchy::Recipe>,
}

impl Colonize {
    /// How far it reaches sideways and up.
    pub fn limits(&self) -> (i32, i32) {
        (if self.reach > 0 { self.reach } else { REACH }, if self.max_height > 0 { self.max_height } else { MAX_HEIGHT })
    }
}

pub const OAK: Colonize = Colonize {
    log: "base:oak_log",
    trunk: 3.0,
    crown_h: 8.0,
    crown_r: 6.3,
    size_var: 0.18,
    shape: Shape::Dome,
    points: 190,
    influence: 4.5,
    kill: 2.2,
    step: 1.0,
    tropism: 0.08,
    twig_r: 0.15,
    min_trunk: 1.35,
    leaf_r: 1.9,
    leaf_ry: 1.5,
    ragged: 0.4,
    leaf_along: false,
    twig: 0.32,
    leader: false,
    fork: 0,
    reach: 0,
    max_height: 0,
    trunk_from: None,
};

pub const SPRUCE: Colonize = Colonize {
    log: "base:spruce_log",
    trunk: 1.0,
    crown_h: 13.0,
    crown_r: 3.6,
    size_var: 0.22,
    shape: Shape::Cone,
    points: 260,
    influence: 3.2,
    kill: 1.6,
    step: 1.0,
    tropism: -0.12,
    twig_r: 0.08,
    min_trunk: 1.0,
    leaf_r: 1.2,
    leaf_ry: 0.9,
    ragged: 0.3,
    leaf_along: true,
    twig: 0.3,
    leader: false,
    fork: 0,
    reach: 0,
    max_height: 0,
    trunk_from: None,
};

pub const BIRCH: Colonize = Colonize {
    log: "base:birch_log",
    trunk: 4.0,
    crown_h: 8.0,
    crown_r: 3.3,
    size_var: 0.2,
    shape: Shape::Oval,
    points: 180,
    influence: 3.0,
    kill: 1.8,
    step: 1.0,
    tropism: -0.05,
    twig_r: 0.08,
    min_trunk: 0.9,
    leaf_r: 1.3,
    leaf_ry: 1.3,
    ragged: 0.5,
    leaf_along: true,
    twig: 0.16,
    leader: true,
    fork: 0,
    reach: 0,
    max_height: 0,
    trunk_from: None,
};

/// An umbrella thorn (Vachellia tortilis): a short trunk dividing low into stems that fan out to a
/// wide flat crown, far wider than it is tall (`reports/Tree looks.md`, section 12).
pub const ACACIA: Colonize = Colonize {
    log: "base:acacia_log",
    trunk: 2.2,
    crown_h: 7.5,
    crown_r: 6.4,
    size_var: 0.15,
    shape: Shape::Umbrella,
    points: 220,
    influence: 4.0,
    kill: 2.0,
    step: 1.0,
    tropism: 0.05,
    twig_r: 0.13,
    min_trunk: 1.0,
    leaf_r: 2.0,
    leaf_ry: 0.8,
    ragged: 0.3,
    leaf_along: false,
    twig: 0.32,
    leader: false,
    fork: 3,
    reach: 0,
    max_height: 0,
    trunk_from: None,
};

/// A Pacific dogwood (Cornus nuttallii): a small understory tree, a short trunk under a rounded,
/// open crown of level sprays. Tried and not used: colonization gives a ball, `branchy`'s
/// opposite level sprays read better (its `RECIPES` row).
pub const DOGWOOD: Colonize = Colonize {
    log: "base:dogwood_log",
    trunk: 1.5,
    crown_h: 5.0,
    crown_r: 3.8,
    size_var: 0.2,
    shape: Shape::Dome,
    points: 110,
    influence: 3.5,
    kill: 1.8,
    step: 1.0,
    tropism: -0.05,
    twig_r: 0.1,
    min_trunk: 0.9,
    leaf_r: 1.7,
    leaf_ry: 0.9,
    ragged: 0.4,
    leaf_along: false,
    twig: 0.3,
    leader: false,
    fork: 0,
    reach: 0,
    max_height: 0,
    trunk_from: None,
};

/// The golden ipê (Plants' recipe, `tree_recipes::IPE`, its numbers kept): a straight trunk to
/// about 21, a high, open, spreading crown topping out at 28-32 (it may poke a little above the
/// canopy); its clumps within the bloom's reach of each other (`BLOOM_REACH`, 4).
pub const IPE: Colonize = Colonize {
    log: "base:ipe_log",
    trunk: 22.0,
    crown_h: 7.5,
    crown_r: 6.6,
    size_var: 0.12,
    shape: Shape::Dome,
    points: 170,
    influence: 5.0,
    kill: 2.2,
    step: 1.0,
    tropism: 0.05,
    twig_r: 0.14,
    min_trunk: 1.55,
    leaf_r: 2.3,
    leaf_ry: 1.4,
    ragged: 0.4,
    leaf_along: false,
    twig: 0.32,
    leader: false,
    fork: 0,
    reach: 8,
    max_height: 42,
    trunk_from: None,
};

/// Andiroba (Carapa guianensis): a straight 2 x 2 trunk under a dense, round crown level with
/// the canopy (28-32).
pub const ANDIROBA: Colonize = Colonize {
    log: "base:andiroba_log",
    trunk: 20.5,
    crown_h: 8.0,
    crown_r: 6.6,
    size_var: 0.12,
    shape: Shape::Dome,
    points: 240,
    influence: 5.0,
    kill: 2.0,
    step: 1.0,
    tropism: 0.05,
    twig_r: 0.14,
    min_trunk: 1.6,
    leaf_r: 2.4,
    leaf_ry: 1.8,
    ragged: 0.3,
    leaf_along: false,
    twig: 0.32,
    leader: false,
    fork: 0,
    reach: 8,
    max_height: 40,
    trunk_from: None,
};

/// Matamatá (Eschweilera): Manaus's commonest canopy tree, a slimmer trunk under a narrower,
/// rounder crown level with the canopy.
pub const MATAMATA: Colonize = Colonize {
    log: "base:matamata_log",
    trunk: 19.5,
    crown_h: 10.0,
    crown_r: 5.0,
    size_var: 0.12,
    shape: Shape::Oval,
    points: 180,
    influence: 5.0,
    kill: 1.9,
    step: 1.0,
    tropism: 0.05,
    twig_r: 0.14,
    min_trunk: 1.45,
    leaf_r: 2.0,
    leaf_ry: 1.8,
    ragged: 0.35,
    leaf_along: false,
    twig: 0.32,
    leader: false,
    fork: 0,
    reach: 8,
    max_height: 38,
    trunk_from: None,
};

/// The Brazil nut, the emergent: its own trunk (`tree_recipes::BRAZIL_NUT`: round, flared,
/// buttressed, as Plants made it, 2 x 2 at least at its top) to about two thirds of its height,
/// then its limbs fanning up to a flat, wide umbrella of a crown 15-20 over the canopy (about
/// 45-55; a dome read as a lollipop from the side, Plants' look).
pub const BRAZIL_NUT: Colonize = Colonize {
    log: "base:brazil_nut_log",
    trunk: 0.68,
    crown_h: 12.0,
    crown_r: 10.5,
    size_var: 0.1,
    shape: Shape::Umbrella,
    points: 320,
    influence: 6.0,
    kill: 2.6,
    step: 1.0,
    tropism: 0.05,
    twig_r: 0.16,
    min_trunk: 2.0,
    leaf_r: 2.6,
    leaf_ry: 1.2,
    ragged: 0.4,
    leaf_along: false,
    twig: 0.34,
    leader: false,
    fork: 0,
    reach: 12,
    max_height: 80,
    trunk_from: Some(&crate::tree_recipes::BRAZIL_NUT),
};

pub const ALL: &[Colonize] = &[OAK, SPRUCE, BIRCH, ACACIA, DOGWOOD, IPE, ANDIROBA, MATAMATA, BRAZIL_NUT];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn f(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn s(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
}

type V = [f32; 3];
fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mul(a: V, k: f32) -> V {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn len2(a: V) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}
fn norm(a: V) -> V {
    mul(a, 1.0 / len2(a).sqrt().max(1e-6))
}
fn block(p: V) -> (i32, i32, i32) {
    (p[0].floor() as i32, p[1].floor() as i32, p[2].floor() as i32)
}

struct Node {
    p: V,
    parent: Option<usize>,
}

/// The nodes of the grown skeleton and each one's radius.
fn skeleton(c: &Colonize, seed: u64, foot: V, trunk_len: Option<f32>) -> (Vec<Node>, Vec<f32>) {
    let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ 0xC0105);
    let size = 1.0 + c.size_var * rng.s();
    let (trunk, ch, cr) = (trunk_len.unwrap_or(c.trunk * size), c.crown_h * size, c.crown_r * size);
    // The crown leans a little off the foot, as a tree's does toward its light.
    let lean = [rng.s() * cr * 0.08, 0.0, rng.s() * cr * 0.08];
    let base = add(foot, add(lean, [0.0, trunk, 0.0]));
    // The points, uniform in the crown (rejection from its box).
    let mut pts: Vec<V> = Vec::with_capacity(c.points as usize);
    let mut tries = 0;
    while pts.len() < c.points as usize && tries < c.points * 40 {
        tries += 1;
        let v = rng.f();
        let (x, z) = (rng.s(), rng.s());
        let r = c.shape.radius(v);
        if x * x + z * z <= r * r {
            pts.push(add(base, [x * cr, v * ch, z * cr]));
        }
    }
    let mut nodes = vec![Node { p: foot, parent: None }];
    if c.fork > 0 {
        // The trunk straight up, then its stems out at about 40 degrees, evenly round with a
        // random turn (directions by rotating one unit vector: arithmetic only).
        while nodes[nodes.len() - 1].p[1] < trunk {
            let last = nodes.len() - 1;
            nodes.push(Node { p: add(nodes[last].p, [0.0, c.step, 0.0]), parent: Some(last) });
        }
        let top = nodes.len() - 1;
        let n = c.fork.clamp(2, 4) as usize;
        let (cs, sn) = [(-1.0f32, 0.0f32), (-0.5, 0.866_025_4), (0.0, 1.0)][n - 2];
        let mut h = norm([rng.s(), 0.0, rng.s() + 0.001]);
        for _ in 0..n {
            let mut q = top;
            let d = norm(add(h, [0.0, 1.2, 0.0]));
            for _ in 0..2 {
                let p = add(nodes[q].p, mul(d, c.step));
                nodes.push(Node { p, parent: Some(q) });
                q = nodes.len() - 1;
            }
            h = [h[0] * cs - h[2] * sn, 0.0, h[0] * sn + h[2] * cs];
        }
    }
    if c.leader {
        // The leader straight up through the crown to near its top, swaying a little.
        let top = base[1] + ch * 0.92;
        while nodes[nodes.len() - 1].p[1] < top {
            let last = nodes.len() - 1;
            let q = nodes[last].p;
            let toward = sub([base[0], q[1], base[2]], q);
            let d = norm(add([rng.s() * 0.12, 1.0, rng.s() * 0.12], mul(toward, 0.3)));
            nodes.push(Node { p: add(q, mul(d, c.step)), parent: Some(last) });
        }
    }
    let mut alive = vec![true; pts.len()];
    let (inf2, kill2) = (c.influence * c.influence, c.kill * c.kill);
    for _ in 0..400 {
        // Each live point pulls the nearest node within its influence.
        let mut pull: Vec<V> = vec![[0.0; 3]; nodes.len()];
        let mut pulled = vec![false; nodes.len()];
        let mut any = false;
        for (i, &a) in pts.iter().enumerate() {
            if !alive[i] {
                continue;
            }
            let mut best = (f32::MAX, usize::MAX);
            for (k, n) in nodes.iter().enumerate() {
                let d = len2(sub(a, n.p));
                if d < best.0 {
                    best = (d, k);
                }
            }
            if best.0 < inf2 {
                pull[best.1] = add(pull[best.1], norm(sub(a, nodes[best.1].p)));
                pulled[best.1] = true;
                any = true;
            }
        }
        let before = nodes.len();
        if !any {
            // Nothing in reach yet: the leader grows on up toward the crown.
            let top = nodes.len() - 1;
            if nodes[top].p[1] > base[1] + ch {
                break;
            }
            let d = norm(add(sub(base, nodes[top].p), [0.0, 4.0, 0.0]));
            let p = add(nodes[top].p, mul(d, c.step));
            nodes.push(Node { p, parent: Some(top) });
            continue;
        }
        for k in 0..before {
            if !pulled[k] {
                continue;
            }
            let d = norm(add(norm(pull[k]), [0.0, c.tropism, 0.0]));
            let p = add(nodes[k].p, mul(d, c.step));
            // A node pulled straight back and forth makes no new branch.
            if nodes[before..].iter().any(|n| len2(sub(n.p, p)) < 0.04) {
                continue;
            }
            nodes.push(Node { p, parent: Some(k) });
        }
        if nodes.len() == before {
            break;
        }
        for (i, &a) in pts.iter().enumerate() {
            if alive[i] && nodes[before..].iter().any(|n| len2(sub(a, n.p)) < kill2) {
                alive[i] = false;
            }
        }
    }
    // The pipe model: a node's area is its children's added (twigs `twig_r`), children always
    // after their parents in the list.
    let mut area = vec![0.0f32; nodes.len()];
    for k in (0..nodes.len()).rev() {
        if area[k] == 0.0 {
            area[k] = c.twig_r * c.twig_r;
        }
        if let Some(p) = nodes[k].parent {
            area[p] += area[k];
        }
    }
    let mut r: Vec<f32> = area.iter().map(|a| a.sqrt()).collect();
    // The bare trunk (the first nodes, each on the last, up to the crown) at least `min_trunk`;
    // a leader on through the crown thinning to a branch's width at its top.
    let crown_top = base[1] + ch;
    for i in 0..nodes.len() {
        if i > 0 && nodes[i].parent != Some(i - 1) {
            break;
        }
        let y = nodes[i].p[1];
        if y <= base[1] {
            r[i] = r[i].max(c.min_trunk * 0.5 * size);
        } else if c.leader {
            let t = ((y - base[1]) / (crown_top - base[1])).clamp(0.0, 1.0);
            r[i] = r[i].max(c.min_trunk * 0.5 * size * (1.0 - t) + 0.25 * t);
        } else {
            break;
        }
    }
    (nodes, r)
}

/// The blocks of one tree of `c` from `seed` with each wood block's width (blocks), in a fixed
/// order: wood joined face to face, 2 x 2 from 1.5 wide; leaves at the twigs.
pub fn blocks(c: &Colonize, seed: u64) -> Vec<(i32, i32, i32, Part, f32)> {
    let mut wood: HashMap<(i32, i32, i32), (u8, f32)> = HashMap::new();
    // A trunk from its `branchy` recipe (the Brazil nut's), cut where the crown starts; the crown
    // grows from the middle of its top.
    let (foot, trunk_len) = match c.trunk_from {
        Some(t) => {
            let trunk_only = crate::branchy::Recipe { orders: 0, fronds: 0, leaf_r: 0.0, leaf_ry: 0.0, ..*t };
            let stem = crate::branchy::wood_widths(&trunk_only, seed);
            let top = (stem.iter().map(|b| b.1).max().unwrap_or(0) as f32 * c.trunk) as i32;
            let mut sum = ([0.0f32; 3], 0.0f32);
            let mut per_layer: HashMap<i32, u32> = HashMap::new();
            for &(_, y, _, _) in &stem {
                *per_layer.entry(y).or_insert(0) += 1;
            }
            for &(x, y, z, w) in &stem {
                if y > top {
                    continue;
                }
                wood.insert((x, y, z), (1, w));
                // At least 2 x 2 up its top quarter, so it reaches the crown at least 2 wide (a
                // thinner one's crown had leaves with no wood near enough, Plants' leaf reach).
                if per_layer[&y] < 4 && y * 4 > top * 3 {
                    for k in [(x + 1, y, z), (x, y, z + 1), (x + 1, y, z + 1)] {
                        wood.entry(k).or_insert((1, 1.5));
                    }
                }
                if y == top {
                    sum.0 = add(sum.0, [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5]);
                    sum.1 += 1.0;
                }
            }
            (mul(sum.0, 1.0 / sum.1.max(1.0)), Some(0.5))
        }
        None => ([0.5, 0.0, 0.5], None),
    };
    let (nodes, r) = skeleton(c, seed, foot, trunk_len);
    let mut leaves: HashMap<(i32, i32, i32), ()> = HashMap::new();
    let mut has_child = vec![false; nodes.len()];
    for n in &nodes {
        if let Some(p) = n.parent {
            has_child[p] = true;
        }
    }
    let put_wood = |wood: &mut HashMap<(i32, i32, i32), (u8, f32)>, k: (i32, i32, i32), axis: u8, w: f32| {
        let e = wood.entry(k).or_insert((axis, 0.0));
        if w > e.1 {
            *e = (axis, w);
        }
    };
    for (i, n) in nodes.iter().enumerate() {
        let Some(pi) = n.parent else { continue };
        let a = nodes[pi].p;
        let b = n.p;
        let d = sub(b, a);
        let axis = {
            let (x, y, z) = (d[0].abs(), d[1].abs(), d[2].abs());
            if y >= x && y >= z {
                1
            } else if x >= z {
                0
            } else {
                2
            }
        };
        let w = 2.0 * r[i];
        if w < c.twig {
            // A twig: a spray of leaves on it, no wood.
            cluster(&mut leaves, b, c.leaf_r * 0.75, c.leaf_ry * 0.8, c.ragged, seed);
            continue;
        }
        // Along the segment in half steps, a step on one axis at a time between blocks.
        let mut last = block(a);
        let steps = (len2(d).sqrt() / 0.5).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let p = add(a, mul(d, s as f32 / steps as f32));
            let k = block(p);
            let mut c0 = last;
            while c0 != k {
                if c0.1 != k.1 {
                    c0.1 += (k.1 - c0.1).signum();
                } else if c0.0 != k.0 {
                    c0.0 += (k.0 - c0.0).signum();
                } else {
                    c0.2 += (k.2 - c0.2).signum();
                }
                put_wood(&mut wood, c0, axis, w);
            }
            put_wood(&mut wood, k, axis, w);
            if w >= 1.5 {
                for (dx, dz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                    put_wood(&mut wood, block(add(p, [dx, 0.0, dz])), axis, w);
                }
            }
            last = k;
        }
        // Leaves at the twigs, and along thin limbs under needles.
        if !has_child[i] || (c.leaf_along && w < 0.5 && n.p[1] > c.trunk) {
            let (lr, lry) = if has_child[i] { (c.leaf_r * 0.7, c.leaf_ry * 0.8) } else { (c.leaf_r, c.leaf_ry) };
            cluster(&mut leaves, b, lr, lry, c.ragged, seed);
        }
    }
    // Within the scatter's reach and height (`branchy::REACH`, `MAX_HEIGHT`), as every tree.
    let (reach, max_height) = c.limits();
    let inside = |&(x, y, z): &(i32, i32, i32)| x.abs() <= reach && z.abs() <= reach && (0..max_height).contains(&y);
    let mut out: Vec<_> = wood
        .iter()
        .filter(|(k, _)| inside(k))
        .map(|(&(x, y, z), &(axis, w))| (x, y, z, Part::Log { axis }, w))
        .collect();
    // No leaf past `LEAF_REACH` of the wood (leaf decay would drop it): a long chain of twigs,
    // leaves without wood, can carry one that far (one Brazil nut in thirty did). A summed
    // volume table of the wood over the tree's box answers "any wood within d" per leaf at once.
    let d = crate::branchy::LEAF_REACH;
    let (nx, ny) = ((2 * reach + 1) as usize, max_height as usize);
    let idx = |x: i32, y: i32, z: i32| -> usize { ((y as usize) * nx + (z + reach) as usize) * nx + (x + reach) as usize };
    // sums[(y+1, z+1, x+1)] = wood blocks in [0..=y] x [-reach..=z] x [-reach..=x]
    let (sx, sz) = (nx + 1, nx + 1);
    let mut sums = vec![0u32; (ny + 1) * sz * sx];
    let at = |y: usize, z: usize, x: usize| (y * sz + z) * sx + x;
    let mut grid = vec![false; ny * nx * nx];
    for b in &out {
        grid[idx(b.0, b.1, b.2)] = true;
    }
    for y in 0..ny {
        for z in 0..nx {
            for x in 0..nx {
                let v = grid[(y * nx + z) * nx + x] as u32;
                // The added terms first, so the sum never dips under 0 on the way.
                sums[at(y + 1, z + 1, x + 1)] =
                    v + sums[at(y, z + 1, x + 1)] + sums[at(y + 1, z, x + 1)] + sums[at(y + 1, z + 1, x)] + sums[at(y, z, x)]
                        - sums[at(y, z, x + 1)]
                        - sums[at(y, z + 1, x)]
                        - sums[at(y + 1, z, x)];
            }
        }
    }
    let near = |&(x, y, z): &(i32, i32, i32)| -> bool {
        let c = |v: i32, hi: usize| v.clamp(0, hi as i32) as usize;
        let (x0, x1) = (c(x + reach - d, nx), c(x + reach + d + 1, nx));
        let (z0, z1) = (c(z + reach - d, nx), c(z + reach + d + 1, nx));
        let (y0, y1) = (c(y - d, ny), c(y + d + 1, ny));
        sums[at(y1, z1, x1)] + sums[at(y0, z0, x1)] + sums[at(y0, z1, x0)] + sums[at(y1, z0, x0)]
            >= sums[at(y0, z1, x1)] + sums[at(y1, z0, x1)] + sums[at(y1, z1, x0)] + sums[at(y0, z0, x0)] + 1
    };
    out.extend(
        leaves
            .keys()
            .filter(|k| !wood.contains_key(k) && inside(k) && near(k))
            .map(|&(x, y, z)| (x, y, z, Part::Leaves, 0.0)),
    );
    out.sort_unstable_by_key(|b| (b.0, b.1, b.2));
    out
}

fn cluster(out: &mut HashMap<(i32, i32, i32), ()>, c: V, r: f32, ry: f32, ragged: f32, seed: u64) {
    let (ri, ryi) = (r.ceil() as i32, ry.ceil() as i32);
    let b = block(c);
    for dy in -ryi..=ryi {
        for dz in -ri..=ri {
            for dx in -ri..=ri {
                let k = (b.0 + dx, b.1 + dy, b.2 + dz);
                let (px, py, pz) = (k.0 as f32 + 0.5 - c[0], k.1 as f32 + 0.5 - c[1], k.2 as f32 + 0.5 - c[2]);
                let d = (px * px + pz * pz) / (r * r) + py * py / (ry * ry);
                if d > 1.0 || k.1 < 0 {
                    continue;
                }
                let h = Rng(seed
                    ^ (k.0 as i64 as u64).wrapping_mul(73_856_093)
                    ^ (k.1 as i64 as u64).wrapping_mul(19_349_663)
                    ^ (k.2 as i64 as u64).wrapping_mul(83_492_791))
                .next();
                let hole = ((h >> 20) % 1000) as f32 / 1000.0;
                if d > 0.55 && hole < ragged || d > 0.25 && hole < ragged * 0.35 {
                    continue;
                }
                out.insert(k, ());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branchy::LEAF_REACH;

    /// The rules every grown tree keeps (`branchy`'s tests): a log at the base, every leaf within
    /// `LEAF_REACH` of a log (leaf decay), wood joined face to face (felling), within the scatter's
    /// reach and height, under a shape's 4,096 blocks; the same tree from the same seed.
    #[test]
    fn colonized_trees_keep_the_rules() {
        for c in ALL {
            for seed in 0..20 {
                let t = blocks(c, seed);
                let name = c.log;
                let logs: std::collections::HashSet<_> = t
                    .iter()
                    .filter(|b| matches!(b.3, Part::Log { .. }))
                    .map(|b| (b.0, b.1, b.2))
                    .collect();
                assert!(logs.contains(&(0, 0, 0)), "{name} {seed}: no trunk at the base");
                assert!(t.len() <= 4096, "{name} {seed}: {} blocks", t.len());
                assert!(
                    t.iter()
                        .all(|b| b.0.abs() <= c.limits().0 && b.2.abs() <= c.limits().0 && (0..c.limits().1).contains(&b.1)),
                    "{name} {seed}: out of reach"
                );
                let d = LEAF_REACH;
                for &(x, y, z, _, _) in t.iter().filter(|b| b.3 == Part::Leaves) {
                    let near = (-d..=d).any(|dy| (-d..=d).any(|dz| (-d..=d).any(|dx| logs.contains(&(x + dx, y + dy, z + dz)))));
                    assert!(near, "{name} {seed}: a leaf at {x} {y} {z} past LEAF_REACH");
                }
                let mut seen = std::collections::HashSet::new();
                let mut todo = vec![(0, 0, 0)];
                while let Some(k) = todo.pop() {
                    if !logs.contains(&k) || !seen.insert(k) {
                        continue;
                    }
                    for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                        todo.push((k.0 + dx, k.1 + dy, k.2 + dz));
                    }
                }
                let loose: Vec<_> = logs
                    .iter()
                    .filter(|k| !seen.contains(k) && k.0.abs() < c.limits().0 && k.2.abs() < c.limits().0)
                    .collect();
                assert!(loose.is_empty(), "{name} {seed}: loose wood {loose:?}");
            }
            assert_eq!(blocks(c, 7), blocks(c, 7));
        }
    }
}

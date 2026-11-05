//! Ley lines (`notes/PROGRESSION.md`: "The lore", "Ley lines", "Round 6"): the network the
//! star-born ancients laid across the world, which still carries magic. Pure and from the seed, so
//! the server, the client, worldgen, the map and the checks all see the same lines.
//!
//! The ancients laid it on a grid: one node per `CELL` x `CELL` blocks, somewhere inside its cell
//! (`INSET` from the edges), and straight lines between neighbouring nodes. Each pair of
//! neighbours along x or z is joined with `KEEP_AXIS` percent, and each square of four nodes has
//! one of its two diagonals, laid with `KEEP_DIAGONAL` percent. So the lines run as a loose net,
//! mostly along the two axes, with every node in every biome (`mods/base/api/tests/ley.rs`
//! checks that, and the spacing). A node is as strong as the lines that meet there
//! (`NODE_POWER`), so crossings are the strong ones, and the strong ones (`WAYSTONE_LINES`
//! lines or more) carry a sleeping waystone (base's `ley.rs` builds it, suited to the biome).
//!
//! Nothing here needs a float to place a block: nodes and links are integer hashes. The
//! readings (`strength`, `read`) use floats, which only feed feel (the rod, the drawn lines,
//! later mana), never the world's blocks.
//!
//! What reads it: base's dowsing rod (Stone Age, the magic path's first step) and its drawn lines;
//! `ley sight` (a perk, `base_api::LEY_SIGHT`) draws them too; mana from nodes and ley taps later
//! read `strength`.

/// Blocks a side of a node's cell.
pub const CELL: i32 = 512;
/// A node lies at least this far inside its cell, so two nodes are always `2 * INSET` apart.
pub const INSET: i32 = 80;
/// Percent of neighbour pairs along x or z the ancients joined.
pub const KEEP_AXIS: u64 = 70;
/// Percent of squares that have their diagonal.
pub const KEEP_DIAGONAL: u64 = 35;

/// What a node gives off by the number of lines meeting there (index), the scale `strength`
/// reads in (1 is standing on a full line). A node no line reaches isn't one.
pub const NODE_POWER: [f32; 9] = [0.0, 0.3, 0.45, 0.7, 1.0, 1.2, 1.4, 1.55, 1.7];
/// A line's flow, from the seed: how strongly it carries, at its middle (`strength` on it).
pub const FLOW: (f32, f32) = (0.55, 1.0);
/// How far from a line its pull is felt (blocks): the rod's reach. Felt as (1 - d / reach)².
pub const LINE_REACH: f64 = 160.0;
/// How far from a node its power is felt (blocks), the same way.
pub const NODE_REACH: f64 = 96.0;
/// Lines meeting at a node for it to carry a sleeping waystone: the great crossings (at 4, two
/// nodes in three had one on the first map; at 5 about one in five, some 600 blocks apart).
pub const WAYSTONE_LINES: u8 = 5;

/// How far round the player the rod shows the lines (blocks), and ley sight.
pub const SHOWN_ROD: f64 = 40.0;
pub const SHOWN_SIGHT: f64 = 72.0;
/// How fast the motes drawn on a line flow along it (blocks a second).
pub const FLOW_SPEED: f32 = 1.6;
/// Motes drawn a second on the lines in view, at most (the client's budget), and a mote's life
/// (seconds). Every line shown is drawn alike: faint, flowing. `MOTES_PER_BLOCK` a second for
/// each block of line in reach (the first look, 0.9, showed a few specks, not a line).
pub const MOTES_A_SECOND: f32 = 260.0;
pub const MOTE_LIFE: (f32, f32) = (2.0, 3.2);
pub const MOTES_PER_BLOCK: f32 = 3.0;
/// How high over the ground the motes ride (blocks over its top block): over tall grass.
pub const MOTE_RISE: (f64, f64) = (0.8, 1.8);

/// The rod's reading as words: under `STILL` it hangs still, then faint, a pull, strong; within
/// `ON_LINE` blocks of a line's middle it's on it, within `AT_NODE` of a node at the node.
pub const STILL: f32 = 0.04;
pub const FAINT: f32 = 0.25;
pub const STRONG: f32 = 0.6;
pub const ON_LINE: f64 = 3.0;
pub const AT_NODE: f64 = 6.0;

/// A node: its cell, where it is (blocks), how many lines meet there, and its power.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub cell: (i32, i32),
    pub x: i32,
    pub z: i32,
    pub lines: u8,
    pub power: f32,
}

impl Node {
    /// Whether it carries a sleeping waystone.
    pub fn waystone(&self) -> bool {
        self.lines >= WAYSTONE_LINES
    }
}

/// A line between two nodes (blocks), flowing from `from` to `to`, `flow` strong.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub from: [i32; 2],
    pub to: [i32; 2],
    pub flow: f32,
}

impl Line {
    /// The nearest point on it to (`x`, `z`), and how far that is.
    pub fn nearest(&self, x: f64, z: f64) -> ([f64; 2], f64) {
        let (ax, az) = (self.from[0] as f64, self.from[1] as f64);
        let (dx, dz) = (self.to[0] as f64 - ax, self.to[1] as f64 - az);
        let len2 = dx * dx + dz * dz;
        let t = if len2 > 0.0 { (((x - ax) * dx + (z - az) * dz) / len2).clamp(0.0, 1.0) } else { 0.0 };
        let p = [ax + dx * t, az + dz * t];
        (p, (p[0] - x).hypot(p[1] - z))
    }

    /// The way it flows, a unit vector (x, z).
    pub fn dir(&self) -> [f64; 2] {
        let (dx, dz) = ((self.to[0] - self.from[0]) as f64, (self.to[1] - self.from[1]) as f64);
        let l = dx.hypot(dz).max(1e-9);
        [dx / l, dz / l]
    }

    pub fn length(&self) -> f64 {
        ((self.to[0] - self.from[0]) as f64).hypot((self.to[1] - self.from[1]) as f64)
    }
}

/// What the rod feels at a place: the strength (`strength`), the nearest line within
/// `LINE_REACH` (its nearest point, distance and the way it flows) and the nearest node within
/// `NODE_REACH`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub strength: f32,
    pub line: Option<NearLine>,
    pub node: Option<NearNode>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearLine {
    pub at: [f64; 2],
    pub dist: f64,
    pub dir: [f64; 2],
    pub flow: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearNode {
    pub node: Node,
    pub dist: f64,
}

// ------------------------------------------------------------------ the network, from the seed

const SALT_NODE: u64 = 0x1E7_0001;
const SALT_EAST: u64 = 0x1E7_0002;
const SALT_SOUTH: u64 = 0x1E7_0003;
const SALT_DIAGONAL: u64 = 0x1E7_0004;
const SALT_WHICH: u64 = 0x1E7_0005;
const SALT_FLOW: u64 = 0x1E7_0006;

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The seed's bits for cell (`i`, `j`) and what is asked (`salt`).
fn hash(seed: u64, i: i32, j: i32, salt: u64) -> u64 {
    mix(seed ^ mix((i as u32 as u64) | ((j as u32 as u64) << 32)) ^ mix(salt))
}

/// Where cell (`i`, `j`)'s node stands (blocks), lines or not.
fn spot(seed: u64, i: i32, j: i32) -> [i32; 2] {
    let h = hash(seed, i, j, SALT_NODE);
    let span = (CELL - 2 * INSET) as u64;
    [i * CELL + INSET + (h % span) as i32, j * CELL + INSET + ((h >> 32) % span) as i32]
}

/// The diagonal square (`i`, `j`) (nodes (i, j) to (i + 1, j + 1)) has: `Some(false)` from
/// (i, j) to (i + 1, j + 1), `Some(true)` from (i + 1, j) to (i, j + 1), None without one.
fn diagonal(seed: u64, i: i32, j: i32) -> Option<bool> {
    (hash(seed, i, j, SALT_DIAGONAL) % 100 < KEEP_DIAGONAL).then(|| hash(seed, i, j, SALT_WHICH) & 1 == 1)
}

/// Whether nodes (`i`, `j`) and the one `d` cells over (each of x and z -1 to 1) are joined.
fn joined(seed: u64, i: i32, j: i32, (di, dj): (i32, i32)) -> bool {
    match (di, dj) {
        (1, 0) => hash(seed, i, j, SALT_EAST) % 100 < KEEP_AXIS,
        (-1, 0) => joined(seed, i - 1, j, (1, 0)),
        (0, 1) => hash(seed, i, j, SALT_SOUTH) % 100 < KEEP_AXIS,
        (0, -1) => joined(seed, i, j - 1, (0, 1)),
        (1, 1) => diagonal(seed, i, j) == Some(false),
        (-1, -1) => diagonal(seed, i - 1, j - 1) == Some(false),
        (-1, 1) => diagonal(seed, i - 1, j) == Some(true),
        (1, -1) => diagonal(seed, i, j - 1) == Some(true),
        _ => false,
    }
}

const AROUND: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (-1, 1), (1, -1)];

/// Cell (`i`, `j`)'s node, None where no line reaches it.
pub fn node(seed: u64, i: i32, j: i32) -> Option<Node> {
    let lines = AROUND.iter().filter(|&&d| joined(seed, i, j, d)).count() as u8;
    (lines > 0).then(|| {
        let [x, z] = spot(seed, i, j);
        Node { cell: (i, j), x, z, lines, power: NODE_POWER[lines as usize] }
    })
}

/// The line from cell `a`'s node to cell `b`'s (neighbours), flowing the way the seed says.
fn line(seed: u64, a: (i32, i32), b: (i32, i32)) -> Line {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let way = ((hi.0 - lo.0 + 1) * 3 + (hi.1 - lo.1 + 1)) as u64;
    let h = hash(seed, lo.0, lo.1, SALT_FLOW + (way << 20));
    let flow = FLOW.0 + (FLOW.1 - FLOW.0) * ((h >> 8) % 1000) as f32 / 999.0;
    let (from, to) = if h & 1 == 0 { (lo, hi) } else { (hi, lo) };
    Line { from: spot(seed, from.0, from.1), to: spot(seed, to.0, to.1), flow }
}

fn cell_of(v: i32) -> i32 {
    v.div_euclid(CELL)
}

/// Every node within `r` blocks (on each axis) of (`x`, `z`).
pub fn nodes_near(seed: u64, x: i32, z: i32, r: i32) -> Vec<Node> {
    let mut out = Vec::new();
    for j in cell_of(z - r)..=cell_of(z + r) {
        for i in cell_of(x - r)..=cell_of(x + r) {
            if let Some(n) = node(seed, i, j)
                && (n.x - x).abs() <= r
                && (n.z - z).abs() <= r
            {
                out.push(n);
            }
        }
    }
    out
}

/// The waystones' nodes within `r` blocks (on each axis) of (`x`, `z`): the crossings.
pub fn waystones_near(seed: u64, x: i32, z: i32, r: i32) -> Vec<Node> {
    let mut v = nodes_near(seed, x, z, r);
    v.retain(Node::waystone);
    v
}

/// Every line that could come within `r` blocks of (`x`, `z`), each once.
pub fn lines_near(seed: u64, x: i32, z: i32, r: i32) -> Vec<Line> {
    let mut out = Vec::new();
    // A line runs between neighbouring cells, so one reaching the area has its first cell at
    // most one cell before it.
    for j in cell_of(z - r) - 1..=cell_of(z + r) {
        for i in cell_of(x - r) - 1..=cell_of(x + r) {
            if joined(seed, i, j, (1, 0)) {
                out.push(line(seed, (i, j), (i + 1, j)));
            }
            if joined(seed, i, j, (0, 1)) {
                out.push(line(seed, (i, j), (i, j + 1)));
            }
            match diagonal(seed, i, j) {
                Some(false) => out.push(line(seed, (i, j), (i + 1, j + 1))),
                Some(true) => out.push(line(seed, (i + 1, j), (i, j + 1))),
                None => {}
            }
        }
    }
    out
}

/// A line's pull at `d` blocks from it.
pub fn line_pull(flow: f32, d: f64) -> f32 {
    if d >= LINE_REACH { 0.0 } else { flow * ((1.0 - d / LINE_REACH) as f32).powi(2) }
}

/// A node's power felt `d` blocks from it.
pub fn node_pull(power: f32, d: f64) -> f32 {
    if d >= NODE_REACH { 0.0 } else { power * ((1.0 - d / NODE_REACH) as f32).powi(2) }
}

/// What is felt at (`x`, `z`) (blocks): the strongest line's pull plus the strongest node's.
/// About 0 away from the lines, the line's flow (0.55 to 1) on one, up to some 2.7 on a great
/// crossing. Cheap: some fifty lines and a few nodes looked at.
pub fn read(seed: u64, x: f64, z: f64) -> Reading {
    let (bx, bz) = (x.floor() as i32, z.floor() as i32);
    let reach = LINE_REACH.ceil() as i32;
    let mut line_best: Option<(f32, NearLine)> = None;
    let mut nearest: Option<NearLine> = None;
    for l in lines_near(seed, bx, bz, reach) {
        let (at, dist) = l.nearest(x, z);
        if dist >= LINE_REACH {
            continue;
        }
        let near = NearLine { at, dist, dir: l.dir(), flow: l.flow };
        let pull = line_pull(l.flow, dist);
        if line_best.is_none_or(|(p, _)| pull > p) {
            line_best = Some((pull, near));
        }
        if nearest.is_none_or(|n| dist < n.dist) {
            nearest = Some(near);
        }
    }
    let mut node_best: Option<(f32, NearNode)> = None;
    for n in nodes_near(seed, bx, bz, NODE_REACH.ceil() as i32) {
        let dist = (n.x as f64 - x).hypot(n.z as f64 - z);
        let pull = node_pull(n.power, dist);
        if pull > 0.0 && node_best.is_none_or(|(p, _)| pull > p) {
            node_best = Some((pull, NearNode { node: n, dist }));
        }
    }
    Reading { strength: line_best.map_or(0.0, |b| b.0) + node_best.map_or(0.0, |b| b.0), line: nearest, node: node_best.map(|b| b.1) }
}

/// `read`'s strength alone: what magic, mana and machines draw on at (`x`, `z`).
pub fn strength(seed: u64, x: f64, z: f64) -> f32 {
    read(seed, x, z).strength
}

/// What the rod says about a reading, in order of strength.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Feel {
    Still,
    Faint,
    Pull,
    Strong,
    OnLine,
    AtNode,
}

impl Feel {
    pub fn of(r: &Reading) -> Feel {
        if r.node.is_some_and(|n| n.dist <= AT_NODE) {
            Feel::AtNode
        } else if r.line.is_some_and(|l| l.dist <= ON_LINE) {
            Feel::OnLine
        } else if r.strength >= STRONG {
            Feel::Strong
        } else if r.strength >= FAINT {
            Feel::Pull
        } else if r.strength >= STILL {
            Feel::Faint
        } else {
            Feel::Still
        }
    }

    /// Its word in the language files (`ley.base.feel.<name>`).
    pub const fn name(self) -> &'static str {
        match self {
            Feel::Still => "still",
            Feel::Faint => "faint",
            Feel::Pull => "pull",
            Feel::Strong => "strong",
            Feel::OnLine => "on_line",
            Feel::AtNode => "at_node",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_agree_from_both_ends() {
        for seed in [1u64, 7, 42] {
            for j in -6..6 {
                for i in -6..6 {
                    for &(di, dj) in &AROUND {
                        assert_eq!(joined(seed, i, j, (di, dj)), joined(seed, i + di, j + dj, (-di, -dj)), "{seed} ({i},{j}) {di},{dj}");
                    }
                }
            }
        }
    }

    #[test]
    fn nodes_stay_inside_their_cells_and_apart() {
        for j in -20..20 {
            for i in -20..20 {
                let [x, z] = spot(3, i, j);
                assert!((i * CELL + INSET..(i + 1) * CELL - INSET).contains(&x));
                assert!((j * CELL + INSET..(j + 1) * CELL - INSET).contains(&z));
            }
        }
    }

    #[test]
    fn lines_near_finds_each_line_once_and_every_node_line() {
        let seed = 11;
        let lines = lines_near(seed, 0, 0, 2000);
        for (k, a) in lines.iter().enumerate() {
            assert!(
                !lines[k + 1..]
                    .iter()
                    .any(|b| (a.from == b.from && a.to == b.to) || (a.from == b.to && a.to == b.from))
            );
        }
        for n in nodes_near(seed, 0, 0, 1000) {
            let ends = lines
                .iter()
                .filter(|l| l.from == [n.x, n.z] || l.to == [n.x, n.z])
                .count();
            assert_eq!(ends, n.lines as usize, "node {n:?}");
        }
    }

    #[test]
    fn readings_are_the_same_every_time_and_peak_on_the_lines() {
        let seed = 5;
        let lines = lines_near(seed, 0, 0, 1500);
        let l = lines.iter().find(|l| l.length() > 300.0).expect("a line");
        let mid = [(l.from[0] + l.to[0]) as f64 / 2.0, (l.from[1] + l.to[1]) as f64 / 2.0];
        let on = read(seed, mid[0], mid[1]);
        assert_eq!(on, read(seed, mid[0], mid[1]));
        assert!(on.strength >= l.flow - 1e-4, "{on:?} on {l:?}");
        // Off to the side, weaker.
        let side = [-l.dir()[1], l.dir()[0]];
        let off = read(seed, mid[0] + side[0] * 60.0, mid[1] + side[1] * 60.0);
        assert!(off.strength < on.strength, "{off:?} < {on:?}");
        assert!(Feel::of(&on) >= Feel::OnLine);
    }

    #[test]
    fn crossings_are_stronger() {
        let seed = 9;
        let nodes = nodes_near(seed, 0, 0, 4000);
        let weak = nodes
            .iter()
            .find(|n| n.lines == 2)
            .expect("a node of two lines");
        let cross = nodes
            .iter()
            .find(|n| n.lines >= WAYSTONE_LINES)
            .expect("a great crossing");
        assert!(strength(seed, cross.x as f64, cross.z as f64) > strength(seed, weak.x as f64, weak.z as f64));
        assert!(cross.waystone() && !weak.waystone());
    }
}

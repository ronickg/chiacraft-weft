//! A copy of the engine's caves where they meet the surface (`crates/engine/src/server/terrain.rs`:
//! `cave_node`, `interpolate`, `carves`, `cavern_roof`, `surface_carved`), for the cave mouths
//! (Ronald, 11-04: "all four"): asking the engine column by column costs some 2 us each, this
//! works out each 4-block node once for a decoration's region. The same integers as the engine;
//! `mouth` must say what `Dimension::surface_carved` says (checked in the game).

use std::collections::HashMap;

use weft_sdk::noise::fixed::{ONE, perlin2, perlin3, ramp, salt, seed32};

/// The engine's tunnels and caverns at the 4-block nodes, worked out as asked and kept: in a
/// dense box over the region a decoration asks about, a map outside it.
pub struct Caves {
    seed: u64,
    /// The box's lowest node (blocks) and its size in nodes.
    lo: (i32, i32, i32),
    size: (i32, i32, i32),
    dense: Vec<Option<(i32, i32)>>,
    nodes: HashMap<(i32, i32, i32), (i32, i32)>,
}

/// From the engine: where caverns stay under a roof (`cavern_roof`) and its pits' strength.
const PIT: i32 = 492;

impl Caves {
    pub fn new(seed: u64) -> Caves {
        Caves { seed, lo: (0, 0, 0), size: (0, 0, 0), dense: Vec::new(), nodes: HashMap::new() }
    }

    /// For the columns `x0..x1`, `z0..z1` (blocks) with ground `y0..=y1` high.
    pub fn over(seed: u64, (x0, x1): (i32, i32), (z0, z1): (i32, i32), (y0, y1): (i32, i32)) -> Caves {
        let lo = (x0.div_euclid(4) * 4, y0.div_euclid(4) * 4, z0.div_euclid(4) * 4);
        let n = |a: i32, b: i32| (b.div_euclid(4) * 4 - a) / 4 + 2;
        let size = (n(lo.0, x1), n(lo.1, y1), n(lo.2, z1));
        Caves { seed, lo, size, dense: vec![None; (size.0 * size.1 * size.2) as usize], nodes: HashMap::new() }
    }

    /// Tunnels, caverns (Q12) at a node (blocks, multiples of 4).
    fn node(&mut self, x: i32, y: i32, z: i32) -> (i32, i32) {
        let seed = self.seed;
        let make = || {
            let (x, y, z) = (x * 16, y * 16, z * 16);
            let a = perlin3(seed32(seed, salt::CAVE_A), x, y, z, 48 * 16, 32 * 16, 48 * 16);
            let b = perlin3(seed32(seed, salt::CAVE_B), x, y, z, 48 * 16, 32 * 16, 48 * 16);
            let spaghetti = 328 - (a.abs() + b.abs());
            let cheese = perlin3(seed32(seed, salt::CAVERN), x, y, z, 90 * 16, 50 * 16, 90 * 16) - 1720;
            (spaghetti, cheese)
        };
        let (i, j, k) = ((x - self.lo.0) / 4, (y - self.lo.1) / 4, (z - self.lo.2) / 4);
        if x >= self.lo.0 && y >= self.lo.1 && z >= self.lo.2 && i < self.size.0 && j < self.size.1 && k < self.size.2 {
            let at = ((j * self.size.2 + k) * self.size.0 + i) as usize;
            *self.dense[at].get_or_insert_with(make)
        } else {
            *self.nodes.entry((x, y, z)).or_insert_with(make)
        }
    }

    /// The fields at a block, blended from its cell's eight nodes as the engine blends them.
    fn fields(&mut self, x: i32, y: i32, z: i32) -> (i32, i32) {
        let (gx, gy, gz) = (x.div_euclid(4) * 4, y.div_euclid(4) * 4, z.div_euclid(4) * 4);
        let (fx, fy, fz) = ((x - gx) * 1024, (y - gy) * 1024, (z - gz) * 1024);
        let l = |a: i32, b: i32, t: i32| a + (((b - a) * t) >> 12);
        let mut c = [[[(0, 0); 2]; 2]; 2];
        for (dy, row) in c.iter_mut().enumerate() {
            for (dz, line) in row.iter_mut().enumerate() {
                for (dx, v) in line.iter_mut().enumerate() {
                    *v = self.node(gx + dx as i32 * 4, gy + dy as i32 * 4, gz + dz as i32 * 4);
                }
            }
        }
        let f = |k: usize| {
            let v = |dy: usize, dz: usize, dx: usize| if k == 0 { c[dy][dz][dx].0 } else { c[dy][dz][dx].1 };
            l(
                l(l(v(0, 0, 0), v(0, 0, 1), fx), l(v(0, 1, 0), v(0, 1, 1), fx), fz),
                l(l(v(1, 0, 0), v(1, 0, 1), fx), l(v(1, 1, 0), v(1, 1, 1), fx), fz),
                fy,
            )
        };
        (f(0), f(1))
    }

    /// Whether the caves took the surface block of column (`x`, `z`), `h` high, the heights of
    /// its 4-block cell's corners being `corners` ((x0, z0), (x1, z0), (x0, z1), (x1, z1)):
    /// the engine's `surface_carved` over the sea.
    pub fn mouth(&mut self, x: i32, z: i32, h: i32, corners: [i32; 4]) -> bool {
        let (gx, gz) = (x.div_euclid(4) * 4, z.div_euclid(4) * 4);
        let (dx, dz) = (x - gx, z - gz);
        let smooth = (corners[0] * (4 - dx) + corners[1] * dx) * (4 - dz) + (corners[2] * (4 - dx) + corners[3] * dx) * dz;
        let (t, c) = self.fields(x, h, z);
        let closed = ramp(perlin2(seed32(self.seed, salt::CAVERN_ROOF), x * 16, z * 16, 120 * 16), 1638, 1024);
        carves(t > 0, c, closed, smooth - h * 16)
    }
}

/// The engine's: whether a cave takes a block `depth` (Q4) under the smoothed surface.
fn carves(tunnel: bool, cavern: i32, closed: i32, depth: i32) -> bool {
    let d = depth.clamp(-(1 << 16), 1 << 16);
    let near = (ONE - d * ONE / (12 * 16)).max(0);
    let pit = (ONE - d * ONE / (24 * 16)).max(0);
    let need = ((((closed * 1024) >> 12) * near) >> 12) - (((((ONE - closed) * PIT) >> 12) * pit) >> 12);
    tunnel || cavern > need
}

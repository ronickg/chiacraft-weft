//! A forest's canopy off the game, for the tree-shape study (`vitals::canopy`): the old-growth
//! stand's sites (a 5 x 5 scatter at `TREES`, a 3 x 3 one at `STAND_TREES` in its stands, as
//! `worldgen.rs` lays them), each grown with `trees::simple_standing` in its tier, on flat ground
//! repeating every `N` blocks; then a picture raymarched over it from the tree diagnosis' three
//! cameras (about 30, 150 and 600 blocks out), sunlit with shadows, as PPM files, for
//! `tester-tools/tree-shapes/rough.py`'s roughness against Bliss's.
//!
//!     cargo run --release -p base_api --example forest_canopy -- <out_dir> [new|old] [trees] [stand]
//!
//!     cargo run -p base_api --example forest_canopy -- ground <seed> <x> <z>   (the ground's height)
//!
//! `branched` / `branched_old` grow the branched style (`base:tree_style` 1) with and without the
//! stand's layers. `old` grows the crowns as they were before the stand tiers (b34890b3: every crown a canopy
//! tree's, the round ones seven across), for the before half of the sheet.

use base_api::branchy::Part;
use base_api::tree_recipes::{Variant, colonized, of, tiered, tiered_colonized, varied, varied_colonized};
use base_api::trees::{Crown, simple_at, simple_standing};
use vitals::canopy::{self, Tier};
use weft_sdk::noise::hash;

const N: i32 = 256;
const H: i32 = 40;
/// Leaves 1..=3 (oak, spruce, birch), logs 4..=6, ground 7.
type Grid = Vec<u8>;

fn at(x: i32, y: i32, z: i32) -> Option<usize> {
    (0..H)
        .contains(&y)
        .then(|| ((y * N + z.rem_euclid(N)) * N + x.rem_euclid(N)) as usize)
}

fn unit(h: u64) -> f64 {
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// A smooth value noise about `size` across, 0..1, repeating every `N`.
fn smooth(seed: u64, x: f64, z: f64, size: f64) -> f64 {
    let cells = (N as f64 / size).round().max(1.0) as i64;
    let (fx, fz) = (x / N as f64 * cells as f64, z / N as f64 * cells as f64);
    let (ix, iz) = (fx.floor() as i64, fz.floor() as i64);
    let (tx, tz) = (fx - ix as f64, fz - iz as f64);
    let v = |a: i64, b: i64| unit(hash(seed, a.rem_euclid(cells), 0, b.rem_euclid(cells)));
    let s = |t: f64| t * t * (3.0 - 2.0 * t);
    let (sx, sz) = (s(tx), s(tz));
    let a = v(ix, iz) * (1.0 - sx) + v(ix + 1, iz) * sx;
    let b = v(ix, iz + 1) * (1.0 - sx) + v(ix + 1, iz + 1) * sx;
    a * (1.0 - sz) + b * sz
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    // `ground <seed> <x> <z>`: the ground's height there and round it (for a walk shot's /tp).
    if a.first().is_some_and(|m| m == "ground") {
        let n: Vec<i32> = a[1..].iter().map(|s| s.parse().unwrap()).collect();
        let f = base_api::land::fields(n[0] as u64);
        for (dx, dz) in [(0, 0), (8, 0), (-8, 0), (0, 8), (0, -8)] {
            let (x, z) = (n[1] + dx, n[2] + dz);
            println!("{x} {z} {}", base_api::land::height(&f, x * 16, z * 16) >> 4);
        }
        return;
    }
    let out = a.first().cloned().unwrap_or_else(|| ".".into());
    let mode = a.get(1).map_or("new", |m| m.as_str());
    let old = mode.ends_with("old");
    let branched = mode.starts_with("branched");
    let trees: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.8);
    let stand: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.15);
    let mut g: Grid = vec![0; (N * N * H) as usize];
    for z in 0..N {
        for x in 0..N {
            g[at(x, 0, z).unwrap()] = 7;
        }
    }
    let mut sites = Vec::new();
    // The tree scatter (5 x 5) everywhere but the clearings' few, the stand scatter (3 x 3) in the
    // stands (a third of the ground, as `stand_over(STAND)`).
    for (salt, cell, chance, stands_only) in [(0x7EE5u64, 5, trees, false), (0x57A2, 3, stand, true)] {
        for cz in 0..N / cell {
            for cx in 0..N / cell {
                let h = hash(salt, cx as i64, 0, cz as i64);
                let (x, z) = (cx * cell + (h % cell as u64) as i32, cz * cell + ((h >> 8) % cell as u64) as i32);
                if unit(h.rotate_left(17)) >= chance {
                    continue;
                }
                let s = smooth(0x57A4D, x as f64, z as f64, 40.0);
                if stands_only && s < 0.62 || !stands_only && s < 0.18 && h >> 40 & 3 != 0 {
                    continue;
                }
                sites.push((x, z, hash(h, 1, 2, 3)));
            }
        }
    }
    let mut count = [0u32; 3];
    for &(x, z, h) in &sites {
        // Oak 6 : spruce 2 : birch 1 (`TREE_KINDS`).
        let k = (h >> 40) % 9;
        let (crown, sp) = match k {
            0..6 => (Crown::Round, 0u8),
            6..8 => (Crown::Cone, 1),
            _ => (Crown::Slim, 2),
        };
        let roll = (hash(h, 0x71E2, 0, 0) >> 40) as f32 / (1u64 << 24) as f32;
        let tier = canopy::tier("base:old_growth_forest", roll);
        let t = simple_at(crown, 4 + sp as u32, 1 + sp as u32, x, 1, z, h >> 32, true);
        if !t.has_crown() {
            continue;
        }
        let t = if old || branched { t } else { t.with_tier(tier) };
        count[tier as usize] += 1;
        let mut put = |x: i32, y: i32, z: i32, s: u32, force: bool| {
            if let Some(i) = at(x, y, z) {
                if force || g[i] == 0 {
                    g[i] = s as u8;
                }
            }
        };
        if branched {
            // The branched style (`base:tree_style` 1): colonized or grown from its recipe, tiered
            // (`branched` new) or not (`branched_old`).
            let log = ["base:oak_log", "base:spruce_log", "base:birch_log"][sp as usize];
            let tier = if old { Tier::Canopy } else { tier };
            let v = if (h >> 40) % 100 < 14 { Variant::Big } else { Variant::Standing };
            let b: Vec<(i32, i32, i32, Part)> = match colonized(log) {
                Some(c) => base_api::tree_recipes::capped(&base_api::colonize::blocks(&tiered_colonized(&varied_colonized(c, v), tier), h >> 32))
                    .into_iter()
                    .map(|(x, y, z, p, _)| (x, y, z, p))
                    .collect(),
                None => base_api::branchy::blocks(&tiered(&varied(of(log).unwrap(), v), tier), h >> 32),
            };
            for (dx, dy, dz, part) in b {
                match part {
                    Part::Leaves => put(x + dx, 1 + dy, z + dz, 1 + sp as u32, false),
                    Part::Log { .. } => put(x + dx, 1 + dy, z + dz, 4 + sp as u32, true),
                }
            }
        } else if old {
            legacy(crown, &t, h >> 32, &mut put);
        } else {
            simple_standing(&t, &mut put);
        }
    }
    eprintln!("sites {} (under/canopy/emergent {count:?})", sites.len());
    // Canopy cover: columns with leaves over 4 blocks up.
    let mut cover = 0;
    for z in 0..N {
        for x in 0..N {
            if (4..H).any(|y| (1..=3).contains(&g[at(x, y, z).unwrap()])) {
                cover += 1;
            }
        }
    }
    eprintln!("cover {:.2}", cover as f64 / (N * N) as f64);
    for (name, height, pitch, dist) in [("near", 30.0, -30.0, 0.0), ("mid", 70.0, -15.0, 0.0), ("far", 105.0, -8.0, 0.0)] {
        let img = render(&g, height, pitch, dist);
        let path = format!("{out}/{mode}_{name}.ppm");
        std::fs::write(&path, img).unwrap();
        eprintln!("{path}");
    }
}

/// The crowns as they were before the stand tiers (b34890b3), for the before pictures: round
/// seven across in four flat layers, cones of stepping rings, birches five across.
fn legacy(crown: Crown, t: &base_api::trees::TreeAt, h: u64, put: &mut impl FnMut(i32, i32, i32, u32, bool)) {
    let big = ((h >> 8) % 100 >= 8 && (h >> 8) % 100 < 22) as i32;
    let height = match crown {
        Crown::Cone => 6 + (h % 4) as i32 + 3 * big,
        Crown::Slim => 6 + (h % 3) as i32 + 2 * big,
        _ => 5 + (h % 3) as i32 + 2 * big,
    };
    let (x0, y0, z0) = (t_x(t), 1, t_z(t));
    let leaves = match crown {
        Crown::Round => 1,
        Crown::Cone => 2,
        _ => 3,
    };
    let top = y0 + height;
    match crown {
        Crown::Cone => {
            for dy in 2..=height + 1 {
                let y = y0 + dy;
                let r = if dy == height + 1 { 0 } else { 1 + ((height + 1 - dy) % 3).min(2) };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz <= r * r + 1 {
                            put(x0 + dx, y, z0 + dz, leaves, false);
                        }
                    }
                }
            }
        }
        Crown::Slim => {
            for y in top - 4..=top {
                let r: i32 = if y == top || y == top - 4 { 1 } else { 2 };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() == r && dz.abs() == r {
                            continue;
                        }
                        put(x0 + dx, y, z0 + dz, leaves, false);
                    }
                }
            }
        }
        _ => {
            let wide = 3 + big;
            let depth = 3 + big;
            for y in top - depth..=top {
                let r: i32 = if y >= top - 1 { wide - 1 } else { wide };
                for dz in -r..=r {
                    for dx in -r..=r {
                        let corner = dx.abs() == r && dz.abs() == r;
                        if corner && (y == top || hash(0, (x0 + dx) as i64, y as i64, (z0 + dz) as i64) & 1 == 0) {
                            continue;
                        }
                        put(x0 + dx, y, z0 + dz, leaves, false);
                    }
                }
            }
        }
    }
    for y in y0..top {
        put(x0, y, z0, leaves as u32 + 3, true);
    }
}

fn t_x(t: &base_api::trees::TreeAt) -> i32 {
    t.pos()[0]
}
fn t_z(t: &base_api::trees::TreeAt) -> i32 {
    t.pos()[2]
}

/// A 640 x 360 picture from `height` over the ground, pitched `pitch` degrees, facing +x with
/// the sun from the side and behind (as the diagnosis' noon shots), as binary PPM.
fn render(g: &Grid, height: f64, pitch: f64, _dist: f64) -> Vec<u8> {
    let (w, hgt) = (640usize, 360usize);
    let mut img = format!("P6\n{w} {hgt}\n255\n").into_bytes();
    let p = pitch.to_radians();
    let yaw: f64 = 0.35;
    let fwd = [p.cos() * yaw.cos(), p.sin(), p.cos() * yaw.sin()];
    let right = [-yaw.sin(), 0.0, yaw.cos()];
    let up = cross(right, fwd);
    let sun = norm([-0.35, 0.8, 0.45]);
    let tanf = (35f64).to_radians().tan();
    let eye = [7.5, height, 3.5];
    for j in 0..hgt {
        for i in 0..w {
            let u = (2.0 * (i as f64 + 0.5) / w as f64 - 1.0) * tanf * w as f64 / hgt as f64;
            let v = (1.0 - 2.0 * (j as f64 + 0.5) / hgt as f64) * tanf;
            let d = norm([fwd[0] + u * right[0] + v * up[0], fwd[1] + u * right[1] + v * up[1], fwd[2] + u * right[2] + v * up[2]]);
            let sky = [0.62, 0.74, 0.92];
            let col = match march(g, eye, d, 1400.0) {
                None => sky,
                Some((b, pos, n, t)) => {
                    let base = match b {
                        1 => [0.20, 0.40, 0.11],
                        2 => [0.12, 0.27, 0.12],
                        3 => [0.32, 0.50, 0.17],
                        4..=6 => [0.30, 0.22, 0.14],
                        _ => [0.26, 0.40, 0.15],
                    };
                    let cell = [
                        (pos[0] - n[0] * 0.01).floor() as i64,
                        (pos[1] - n[1] * 0.01).floor() as i64,
                        (pos[2] - n[2] * 0.01).floor() as i64,
                    ];
                    let tex = 0.88 + 0.24 * unit(hash(9, cell[0], cell[1], cell[2]));
                    let lit = dot(n, sun).max(0.0);
                    let o = [pos[0] + n[0] * 0.02, pos[1] + n[1] * 0.02, pos[2] + n[2] * 0.02];
                    let shadow = if lit > 0.0 && march(g, o, sun, 80.0).is_some() { 0.0 } else { 1.0 };
                    let sky_l = 0.2 + 0.1 * n[1];
                    let l = sky_l + 0.9 * lit * shadow;
                    let haze = 1.0 - (-t / 2500.0).exp();
                    [0, 1, 2].map(|k| (base[k] * tex * l) * (1.0 - haze) + sky[k] * haze)
                }
            };
            for c in col {
                img.push((c.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0) as u8);
            }
        }
    }
    img
}

/// The first block along the ray: its kind, the hit point, the face's normal and the distance.
fn march(g: &Grid, o: [f64; 3], d: [f64; 3], max: f64) -> Option<(u8, [f64; 3], [f64; 3], f64)> {
    let mut cell = o.map(|v| v.floor() as i32);
    let step = d.map(|v| if v > 0.0 { 1 } else { -1 });
    let delta = d.map(|v| if v == 0.0 { f64::INFINITY } else { 1.0 / v.abs() });
    let mut next = [0, 1, 2].map(|k| {
        let edge = if d[k] > 0.0 { cell[k] as f64 + 1.0 - o[k] } else { o[k] - cell[k] as f64 };
        edge * delta[k]
    });
    let mut t = 0.0;
    let mut n = [0.0; 3];
    loop {
        if cell[1] < 0 || t > max || cell[1] >= H && d[1] >= 0.0 {
            return None;
        }
        if let Some(i) = at(cell[0], cell[1], cell[2]) {
            let b = g[i];
            if b != 0 {
                let p = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
                return Some((b, p, n, t));
            }
        }
        let k = if next[0] < next[1] && next[0] < next[2] {
            0
        } else if next[1] < next[2] {
            1
        } else {
            2
        };
        t = next[k];
        next[k] += delta[k];
        cell[k] += step[k];
        n = [0.0; 3];
        n[k] = -(step[k] as f64);
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = dot(a, a).sqrt();
    a.map(|v| v / l)
}

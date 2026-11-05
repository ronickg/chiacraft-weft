//! A top-down map of the ley network (`vitals::ley`) over the overworld's biomes: the lines in
//! light blue (brighter the stronger they flow), the rod's reach round them as a faint halo, each
//! node a dot as big as the lines meeting there, and the waystones' crossings ringed in white.
//! It prints the counts: nodes and waystones per biome, how far apart they are, and how far the
//! farthest ground is from a line. No game, no GPU.
//!
//!     cargo run -p base_api --example ley_map -- <seed> <x> <z> <blocks across> <blocks a pixel> <out.png>

use base_api::SEA;
use base_api::land as l;
use vitals::ley;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(1, 1) as u64;
    let (cx, cz) = (arg(2, 0) as i32, arg(3, 0) as i32);
    let across = arg(4, 4096) as i32;
    let step = arg(5, 4).max(1) as i32;
    let out = a.get(6).cloned().unwrap_or_else(|| "ley_map.png".into());
    let f = l::fields(seed);
    let n = (across / step) as usize;
    let (x0, z0) = (cx - across / 2, cz - across / 2);
    let lines = ley::lines_near(seed, cx, cz, across / 2 + ley::CELL);
    let nodes = ley::nodes_near(seed, cx, cz, across / 2);
    let mut img = vec![0u8; n * n * 3];
    let mut far = 0f64;
    let mut far_at = (0, 0);
    for j in 0..n {
        for i in 0..n {
            let (x, z) = (x0 + i as i32 * step, z0 + j as i32 * step);
            let h = l::height(&f, x * 16, z * 16) >> 4;
            let biome = if h < SEA { "ocean" } else { l::overworld_biome(&f, x * 16, z * 16) };
            let c = colour(biome);
            // Muted, so the lines stand out.
            let mut px = c.map(|v| v as f32 * 0.55 + 30.0);
            let (fx, fz) = (x as f64 + 0.5, z as f64 + 0.5);
            let (mut d, mut flow) = (f64::MAX, 0f32);
            for li in &lines {
                let (_, dd) = li.nearest(fx, fz);
                if dd < d {
                    (d, flow) = (dd, li.flow);
                }
            }
            if d > far {
                (far, far_at) = (d, (x, z));
            }
            let blend = |px: &mut [f32; 3], to: [f32; 3], k: f32| {
                for c in 0..3 {
                    px[c] += (to[c] - px[c]) * k;
                }
            };
            // The rod's reach: a faint halo.
            let pull = ley::line_pull(flow, d);
            blend(&mut px, [120.0, 200.0, 255.0], pull * 0.35);
            // The line itself, two pixels wide whatever the scale.
            if d <= step as f64 * 1.0 {
                blend(&mut px, [150.0, 225.0, 255.0], 0.55 + 0.45 * flow);
            }
            let o = (j * n + i) * 3;
            for c in 0..3 {
                img[o + c] = px[c].clamp(0.0, 255.0) as u8;
            }
        }
    }
    // Nodes: a dot of radius by the lines meeting there; waystones ringed.
    let mut put = |x: i32, z: i32, rgb: [u8; 3]| {
        let (i, j) = ((x - x0) / step, (z - z0) / step);
        if (0..n as i32).contains(&i) && (0..n as i32).contains(&j) {
            let o = (j as usize * n + i as usize) * 3;
            img[o..o + 3].copy_from_slice(&rgb);
        }
    };
    for nd in &nodes {
        let r = (2 + nd.lines as i32) * step.max(2) / 2;
        for dz in -r - 3 * step..=r + 3 * step {
            for dx in -r - 3 * step..=r + 3 * step {
                let d2 = dx * dx + dz * dz;
                if d2 <= r * r {
                    put(nd.x + dx, nd.z + dz, if nd.waystone() { [255, 245, 170] } else { [90, 200, 255] });
                } else if nd.waystone() && d2 <= (r + 3 * step) * (r + 3 * step) && d2 > (r + 2 * step) * (r + 2 * step) {
                    put(nd.x + dx, nd.z + dz, [255, 255, 255]);
                }
            }
        }
    }
    std::fs::write(&out, png(n, n, &img)).expect("write the map");

    // The counts, by the biome the rules pick (`mods/base/api/tests/ley.rs` counts the same way).
    let biome_at = |x: i32, z: i32| l::overworld_biome(&f, x * 16, z * 16);
    let mut by: Vec<(&str, usize, usize)> = Vec::new();
    for nd in &nodes {
        let b = biome_at(nd.x, nd.z);
        match by.iter_mut().find(|e| e.0 == b) {
            Some(e) => (e.1, e.2) = (e.1 + 1, e.2 + nd.waystone() as usize),
            None => by.push((b, 1, nd.waystone() as usize)),
        }
    }
    by.sort();
    let gaps = |set: &[&ley::Node]| -> (f64, f64, f64) {
        let mut d: Vec<f64> = set
            .iter()
            .map(|a| {
                set.iter()
                    .filter(|b| *b != a)
                    .map(|b| ((a.x - b.x) as f64).hypot((a.z - b.z) as f64))
                    .fold(f64::MAX, f64::min)
            })
            .collect();
        d.sort_by(f64::total_cmp);
        (d[0], d[d.len() / 2], d[d.len() - 1])
    };
    let all: Vec<&ley::Node> = nodes.iter().collect();
    let stones: Vec<&ley::Node> = nodes.iter().filter(|n| n.waystone()).collect();
    println!("seed {seed}, {across} blocks across at ({cx}, {cz}), {step} a pixel -> {out}");
    println!("  nodes {} ({} waystones), lines {}", nodes.len(), stones.len(), lines.len());
    let mut by_lines = [0usize; 9];
    for nd in &nodes {
        by_lines[nd.lines as usize] += 1;
    }
    println!("  nodes by lines meeting: {:?}", &by_lines[1..]);
    let (lo, mid, hi) = gaps(&all);
    println!("  nearest node to a node: {lo:.0} min, {mid:.0} median, {hi:.0} max (blocks)");
    if stones.len() > 1 {
        let (lo, mid, hi) = gaps(&stones);
        println!("  nearest waystone to a waystone: {lo:.0} min, {mid:.0} median, {hi:.0} max");
    }
    println!("  farthest ground from a line: {far:.0} blocks, at ({}, {}); the rod feels {:.0}", far_at.0, far_at.1, ley::LINE_REACH);
    // The waystones nearest the middle, to aim a look: where, the lines meeting, biome and ground.
    let mut near: Vec<&ley::Node> = stones.clone();
    near.sort_by_key(|n| (n.x - cx).pow(2) + (n.z - cz).pow(2));
    for n in near.iter().take(12) {
        let h = l::height(&f, n.x * 16, n.z * 16) >> 4;
        println!("  waystone at {} {} ({} lines): {}, ground {h}", n.x, n.z, n.lines, biome_at(n.x, n.z));
    }
    // The lines out of the nearest one: where to stand on a line, and off it.
    if let Some(n) = near.first() {
        for li in lines
            .iter()
            .filter(|li| li.from == [n.x, n.z] || li.to == [n.x, n.z])
        {
            let mid = [(li.from[0] + li.to[0]) / 2, (li.from[1] + li.to[1]) / 2];
            println!("  its line {:?} -> {:?}, flow {:.2}, middle {} {}", li.from, li.to, li.flow, mid[0], mid[1]);
        }
    }
    for (b, k, w) in by {
        println!("  {b:16} nodes {k:4}  waystones {w:4}");
    }
}

/// A biome's colour, as `land_map` has them.
fn colour(biome: &str) -> [u8; 3] {
    match biome {
        "ocean" => [30, 60, 160],
        "coral_reef" | "lagoon" => [40, 150, 170],
        "beach" => [220, 205, 150],
        "mountains" => [120, 130, 115],
        "snowy_plains" => [215, 225, 225],
        "taiga" => [60, 100, 70],
        "sequoia_grove" => [175, 70, 45],
        "desert" => [220, 200, 130],
        "savanna" => [170, 165, 80],
        "swamp" => [90, 105, 60],
        "forest" => [55, 125, 45],
        "tropical_forest" => [25, 105, 55],
        "birch_forest" => [110, 160, 90],
        _ => [125, 175, 85],
    }
}

/// An RGB image as a PNG, uncompressed (stored deflate blocks), as `land_map` writes it.
fn png(w: usize, h: usize, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for row in rgb.chunks(w * 3) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let chunks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, c) in chunks.iter().enumerate() {
        z.push((i + 1 == chunks.len()) as u8);
        z.extend_from_slice(&(c.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        z.extend_from_slice(c);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &v in &raw {
        a = (a + v as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10];
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

fn crc(data: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

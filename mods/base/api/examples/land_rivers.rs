//! The river network (`base_api::land::rivers`) drawn over the land: every stretch from a node to
//! the one it drains to, thicker and bluer the more it carries, on the land's height shaded grey.
//! It prints each region's nodes, spilled pits and stretches, and the flows' spread.
//!
//!     cargo run -p base_api --example land_rivers -- <seed> <x> <z> <blocks across> <blocks a pixel> <out.png>

use base_api::SEA;
use base_api::land as l;
use weft_sdk::noise::fixed::Fields;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(1, 1) as u64;
    let (cx, cz) = (arg(2, 0) as i32, arg(3, 0) as i32);
    let across = arg(4, 4096) as i32;
    let step = arg(5, 4).max(1) as i32;
    let out = a
        .get(6)
        .cloned()
        .unwrap_or_else(|| "land_rivers.png".into());
    let f = Fields { seed };
    let n = (across / step) as usize;
    let (x0, z0) = (cx - across / 2, cz - across / 2);
    let mut img = vec![0u8; n * n * 3];
    for j in 0..n {
        for i in 0..n {
            let (bx, bz) = (x0 + i as i32 * step, z0 + j as i32 * step);
            let h = l::land_height(&f, bx * 16, bz * 16) >> 4;
            let c = if h < SEA {
                [40, 60, 110]
            } else {
                let v = (90 + (h - SEA) * 2).min(240) as u8;
                [v, v, v]
            };
            img[(j * n + i) * 3..(j * n + i) * 3 + 3].copy_from_slice(&c);
        }
    }
    let region = base_api::land::RIVER_REGION * base_api::land::RIVER_NODE;
    let mut flows = Vec::new();
    let mut seen = std::collections::HashMap::new();
    let (mut shared, mut differ) = (0, 0);
    let (mut nodes, mut spilled, mut stretches) = (0, 0, 0);
    let mut rz = (z0).div_euclid(region) * region;
    while rz < z0 + across {
        let mut rx = (x0).div_euclid(region) * region;
        while rx < x0 + across {
            let s = l::river_region_stats(&f, rx, rz);
            nodes += s[0];
            spilled += s[1];
            stretches += s[2];
            for (a, b, fl) in l::river_region_stretches(&f, rx, rz) {
                if let Some(&other) = seen.get(&(a, b)) {
                    shared += 1;
                    if other != fl {
                        differ += 1;
                    }
                    continue;
                }
                seen.insert((a, b), fl);
                flows.push(fl[0] >> 12);
                let w = ((fl[0] >> 12) as f64).log2().max(0.0) as i32 / 2;
                let blue = (120 + (fl[0] >> 12).min(400) * 135 / 400) as u8;
                let len = ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / step).max(1);
                for t in 0..=len {
                    let px = (a[0] + (b[0] - a[0]) * t / len - x0) / step;
                    let pz = (a[1] + (b[1] - a[1]) * t / len - z0) / step;
                    for dz in -w / 2..=w / 2 {
                        for dx in -w / 2..=w / 2 {
                            let (qx, qz) = (px + dx, pz + dz);
                            if qx >= 0 && qz >= 0 && (qx as usize) < n && (qz as usize) < n {
                                img[(qz as usize * n + qx as usize) * 3..][..3].copy_from_slice(&[20, 60, blue]);
                            }
                        }
                    }
                }
            }
            rx += region;
        }
        rz += region;
    }
    flows.sort();
    let q = |p: usize| flows.get(flows.len() * p / 100).copied().unwrap_or(0);
    println!("seed {seed}: {nodes} nodes in the regions, {spilled} pits spilled, {stretches} stretches ({} drawn)", flows.len());
    println!("flows (wet nodes): 10% {}  50% {}  90% {}  99% {}  max {}", q(10), q(50), q(90), q(99), flows.last().copied().unwrap_or(0));
    std::fs::write(&out, png(n, n, &img)).expect("write the map");
    println!("stretches two regions both list: {shared}, of them with a different flow: {differ}");
    println!("-> {out}");
}

fn png(w: usize, h: usize, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for row in rgb.chunks(w * 3) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let chunk = |out: &mut Vec<u8>, kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = kind.to_vec();
        c.extend_from_slice(data);
        out.extend_from_slice(&c);
        out.extend_from_slice(&crc(&c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    // Stored (uncompressed) deflate blocks.
    let mut z = vec![0x78, 0x01];
    for (i, block) in raw.chunks(65535).enumerate() {
        let last = (i + 1) * 65535 >= raw.len();
        z.push(last as u8);
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut s1, mut s2) = (1u32, 0u32);
    for &b in &raw {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    z.extend_from_slice(&((s2 << 16) | s1).to_be_bytes());
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

fn crc(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

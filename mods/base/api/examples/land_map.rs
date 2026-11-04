//! A map of the overworld's land from `base_api::land`, before (the land as it was: continents,
//! hills, dunes, mountains) and after (highlands, mesas, rivers, snow, rock faces), side by side,
//! with each biome's colour as the rules in `mods/base/src/land.rs` would pick it. No game, no
//! GPU: the same integer math the engine adds. It also lists spots for before/after shots.
//!
//!     cargo run -p base_api --example land_map -- <seed> <x> <z> <blocks across> <blocks a pixel> <out.png>

use base_api::SEA;
use base_api::land::{self as l};
use weft_sdk::noise::fixed::{Fields, ONE, q4, q12, shape};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: i64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seed = arg(1, 1) as u64;
    let (cx, cz) = (arg(2, 0) as i32, arg(3, 0) as i32);
    let across = arg(4, 4096) as i32;
    let step = arg(5, 4).max(1) as i32;
    let out = a.get(6).cloned().unwrap_or_else(|| "land_map.png".into());
    let f = Fields { seed };
    let n = (across / step) as usize;
    let (x0, z0) = (cx - across / 2, cz - across / 2);
    // Heights before and after, a pixel's column each (with one more row and column for shading).
    let (mut before, mut after) = (vec![0i32; (n + 1) * (n + 1)], vec![0i32; (n + 1) * (n + 1)]);
    for j in 0..=n {
        for i in 0..=n {
            let (x, z) = ((x0 + i as i32 * step) * 16, (z0 + j as i32 * step) * 16);
            before[j * (n + 1) + i] = old_height(&f, x, z) >> 4;
            after[j * (n + 1) + i] = l::height(&f, x, z) >> 4;
        }
    }
    let w = 2 * n + 8;
    let mut img = vec![0u8; w * n * 3];
    let mut stats = [0usize; 6];
    let mut falls = std::collections::BTreeMap::new();
    let mut spots: Vec<(&str, i32, i32, i32, i32)> = Vec::new();
    for (side, hs) in [&before, &after].into_iter().enumerate() {
        for j in 0..n {
            for i in 0..n {
                let (bx, bz) = (x0 + i as i32 * step, z0 + j as i32 * step);
                let (x, z) = (bx * 16, bz * 16);
                let h = hs[j * (n + 1) + i];
                let (east, south) = (hs[j * (n + 1) + i + 1], hs[(j + 1) * (n + 1) + i]);
                // Steepness in blocks per block, from the pixel's neighbours.
                let fall = ((h - east).abs().max((h - south).abs()) + step - 1) / step;
                let after = side == 1;
                let (c, kind) = colour(&f, x, z, h, after && fall >= l::ROCK_FALL, after);
                // Lit from the north-west, and lighter the higher the land stands.
                let lift = if h > SEA { ((h - SEA) / 3).min(40) } else { 0 };
                let shade = (((h - east) + (h - south)) * 10 / step).clamp(-60, 60) + lift - 10;
                let px = (j * w + i + side * (n + 8)) * 3;
                for k in 0..3 {
                    img[px + k] = (c[k] as i32 + shade).clamp(0, 255) as u8;
                }
                if after {
                    stats[kind] += 1;
                    if l::mesa(&f, x, z) > 0 && step == 1 {
                        *falls.entry(fall.min(8)).or_insert(0usize) += 1;
                    }
                    let old = before[j * (n + 1) + i];
                    let near = |s: &[(&str, i32, i32, i32, i32)], name: &str| {
                        s.iter()
                            .any(|p| p.0 == name && (p.1 - bx).abs() + (p.2 - bz).abs() < 600)
                    };
                    let name = match kind {
                        1 if h < SEA && old >= SEA + 4 => Some("river"),
                        2 if h > old + 20 => Some("mesa"),
                        3 => Some("snow"),
                        4 if fall >= 6 => Some("rock face"),
                        _ => None,
                    };
                    if let Some(name) = name
                        && spots.iter().filter(|p| p.0 == name).count() < 3
                        && !near(&spots, name)
                    {
                        spots.push((name, bx, bz, old, h));
                    }
                }
            }
        }
    }
    std::fs::write(&out, png(w, n, &img)).expect("write the map");
    // Heights of the mountains biome's ground, by tenths.
    let mut hist = std::collections::BTreeMap::new();
    for j in 0..n {
        for i in 0..n {
            let (x, z) = ((x0 + i as i32 * step) * 16, (z0 + j as i32 * step) * 16);
            if l::MOUNTAINS_AREA.value(&f, x, z) > q12(l::MOUNTAIN_BIOME) {
                *hist
                    .entry(after[j * (n + 1) + i] / 10 * 10)
                    .or_insert(0usize) += 1;
            }
        }
    }
    println!("mountains biome ground by height: {hist:?}");
    println!("mesas' ground by its fall to a neighbour: {falls:?}");
    let all = (n * n) as f64;
    println!("seed {seed}, {across} blocks across at ({cx}, {cz}), {step} a pixel -> {out} (left before, right after)");
    for (k, name) in ["land", "river water", "mesa", "snow", "rock face", "sea"]
        .iter()
        .enumerate()
    {
        println!("  {name}: {:.1}%", stats[k] as f64 * 100.0 / all);
    }
    for (name, x, z, old, new) in spots {
        println!("  spot {name}: x {x} z {z}, ground {old} before, {new} after");
    }
}

/// The land as it was before `base_api::land` (dev at 0d0cd58a): the same presets and masks,
/// without the highlands, the mesas and the rivers.
fn old_height(f: &Fields, x: i32, z: i32) -> i32 {
    let m = |a: &l::Area, h: i32| {
        let w = a.value(f, x, z);
        if w > 0 { (w * h) >> 12 } else { 0 }
    };
    q4(SEA as f64)
        + shape::continents(f, x, z, q4(l::CONTINENTS))
        + m(&l::HILLS_AREA, shape::hills(f, x, z, q4(l::HILLS.0), q4(l::HILLS.1)))
        + m(&l::DUNES_AREA, shape::dunes(f, x, z, q4(l::DUNES.0), q4(l::DUNES.1)))
        + m(&l::MOUNTAINS_AREA, shape::mountains(f, x, z, q4(l::MOUNTAINS.0), q4(l::MOUNTAINS.1)))
}

/// A column's colour as its biome rule (`mods/base/src/land.rs`) would pick it, and what it counts
/// as: 0 land, 1 river water, 2 mesa, 3 snow, 4 rock face, 5 sea.
fn colour(f: &Fields, x: i32, z: i32, h: i32, rock: bool, after: bool) -> ([u8; 3], usize) {
    let (t, cont) = (f.temperature(x, z), f.continent(x, z));
    if h < SEA {
        let deep = ((SEA - h) * 6).min(120) as u8;
        let ocean = h <= SEA - 2 && (!after || cont < q12(l::OCEAN_CONTINENT));
        return if ocean || h <= SEA - 2 && !after { ([30, 70u8.saturating_sub(deep / 3), 200 - deep], 5) } else { ([40, 120, 230], 1) };
    }
    if h <= SEA + 1 && cont < q12(0.1) {
        return (if t < q12(l::SNOWY_BELOW) { [235, 240, 245] } else { [220, 205, 150] }, 0);
    }
    if after && l::snowy(f, x, z, h) {
        return if rock { ([125, 125, 130], 4) } else { ([250, 252, 255], 3) };
    }
    let mesa = after && l::mesa(f, x, z) > q4(4.0);
    let base: [u8; 3] = match l::overworld_biome(f, x, z) {
        "mountains" if h > 130 => [130, 130, 130],
        "mountains" => [105, 140, 100],
        "snowy_plains" => [215, 225, 225],
        "taiga" => [60, 100, 70],
        // Giant sequoia groves: rust-red, to find them.
        "sequoia_grove" => [175, 70, 45],
        "desert" => [220, 200, 130],
        "savanna" => [170, 165, 80],
        "swamp" => [90, 105, 60],
        "forest" => [55, 125, 45],
        "birch_forest" => [110, 160, 90],
        _ => [125, 175, 85],
    };
    if rock {
        let sand = base == [220, 200, 130];
        return (if sand { [200, 160, 110] } else { [120, 118, 115] }, 4);
    }
    if mesa {
        let up = (l::mesa(f, x, z) * 60 / q4(l::MESA_HEIGHT)) as u8;
        return ([150 + up, 95 + up / 2, 60], 2);
    }
    (base, 0)
}

/// An RGB image as a PNG, uncompressed (stored deflate blocks).
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
    let _ = ONE;
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

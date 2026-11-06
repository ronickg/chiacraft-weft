//! Distant Horizons' far data (format 2 of its `FullData` table), as `tools/mcimport/dh_export.py`
//! writes it out of DH's sqlite: `CCDH0001`, then per detail-level-0 section (64x64 columns of
//! one block) its x, z, min y (i32 LE) and two zstd blobs (u32 LE length each): the data and the
//! mapping.
//!
//! The mapping is a big-endian i32 count of Java UTF strings `biome_DH-BSW_state`, a state being
//! `AIR` or `minecraft:water_STATE_{level:0}`. The data holds the inner 62x62 columns (x outer,
//! z inner; the border columns live in the neighbours' edge blobs, not read here) as varints:
//! every column's run count, then every run's `id << 2 | light << 1 | jump`, then every run's
//! height, then for the runs with `jump` set a zigzag correction of its bottom from the one
//! expected (the previous run's bottom minus this one's height; runs go from the top down), then
//! light bytes. Bottoms count from the dimension's lowest block.

use std::io::Read;
use std::path::Path;

/// One vertical run of a column: mapping entry, lowest block (world y) and height in blocks.
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub id: u32,
    pub bottom: i32,
    pub height: i32,
}

impl Run {
    pub fn top(&self) -> i32 {
        self.bottom + self.height - 1
    }
}

/// A 64x64-block section of DH's level-0 data.
pub struct Section {
    /// Block corner (x, z).
    pub x: i32,
    pub z: i32,
    /// Mapping entries: Minecraft biome and block state (`minecraft:water[level=0]`, `AIR`).
    pub entries: Vec<(String, String)>,
    /// Runs per inner column, `(x - 1) * 62 + (z - 1)` for local x, z in 1..63.
    pub columns: Vec<Vec<Run>>,
}

impl Section {
    /// The runs of local column (x, z); the border (0 or 63) takes the nearest inner one.
    pub fn column(&self, x: i32, z: i32) -> &[Run] {
        let (x, z) = (x.clamp(1, 62), z.clamp(1, 62));
        &self.columns[((x - 1) * 62 + (z - 1)) as usize]
    }
}

fn zstd(b: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut src = b;
    ruzstd::decoding::StreamingDecoder::new(&mut src)
        .ok()?
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

fn varint(b: &[u8], i: &mut usize) -> Option<u64> {
    let mut v = 0u64;
    for s in (0..64).step_by(7) {
        let c = *b.get(*i)?;
        *i += 1;
        v |= ((c & 0x7F) as u64) << s;
        if c < 0x80 {
            return Some(v);
        }
    }
    None
}

/// DH's `minecraft:water_STATE_{level:0}` as ours `minecraft:water[level=0]` (properties sorted).
fn state(dh: &str) -> String {
    let Some((name, props)) = dh.split_once("_STATE_") else { return dh.to_string() };
    let inner = props.trim_start_matches('{').trim_end_matches('}');
    if inner.is_empty() {
        return name.to_string();
    }
    let mut kv: Vec<String> = inner.split(',').map(|p| p.replacen(':', "=", 1)).collect();
    kv.sort();
    format!("{name}[{}]", kv.join(","))
}

fn mapping(b: &[u8]) -> Option<Vec<(String, String)>> {
    let n = i32::from_be_bytes(b.get(..4)?.try_into().ok()?) as usize;
    let mut i = 4;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let l = u16::from_be_bytes(b.get(i..i + 2)?.try_into().ok()?) as usize;
        let s = String::from_utf8_lossy(b.get(i + 2..i + 2 + l)?);
        i += 2 + l;
        let (biome, st) = s.split_once("_DH-BSW_").unwrap_or(("minecraft:plains", &s));
        out.push((biome.to_string(), if st == "AIR" { "minecraft:air".into() } else { state(st) }));
    }
    Some(out)
}

fn columns(b: &[u8], min_y: i32) -> Option<Vec<Vec<Run>>> {
    let mut i = 0;
    let n = 62 * 62;
    let mut counts = Vec::with_capacity(n);
    for _ in 0..n {
        counts.push(varint(b, &mut i)? as usize);
    }
    let total: usize = counts.iter().sum();
    let mut ids = Vec::with_capacity(total);
    for _ in 0..total {
        ids.push(varint(b, &mut i)?);
    }
    let mut heights = Vec::with_capacity(total);
    for _ in 0..total {
        heights.push(varint(b, &mut i)? as i32);
    }
    let mut prev = 0i64;
    let mut k = 0;
    let mut out = Vec::with_capacity(n);
    for c in counts {
        let mut col = Vec::with_capacity(c);
        for _ in 0..c {
            let h = heights[k] as i64;
            let mut bottom = prev - h;
            if ids[k] & 1 != 0 {
                let z = varint(b, &mut i)?;
                bottom += ((z >> 1) as i64) ^ -((z & 1) as i64);
            }
            prev = bottom;
            col.push(Run { id: (ids[k] >> 2) as u32, bottom: bottom as i32 + min_y, height: h as i32 });
            k += 1;
        }
        out.push(col);
    }
    Some(out)
}

/// Every section of an exported file, decoded on all cores and handed to `each` (in no order).
/// `min_y`: the dimension's lowest block (-64 in the overworld). Broken sections are logged.
pub fn for_each_section(path: &Path, min_y: i32, each: impl Fn(Section) + Sync) -> std::io::Result<usize> {
    use rayon::prelude::*;
    let f = std::fs::read(path)?;
    if f.get(..8) != Some(b"CCDH0001") {
        return Err(std::io::Error::other("not an exported DH file (CCDH0001)"));
    }
    // Index the records first, then decode them in parallel.
    let mut recs: Vec<(i32, i32, &[u8], &[u8])> = Vec::new();
    let mut i = 8;
    let rd = |i: &mut usize| -> Option<u32> {
        let v = u32::from_le_bytes(f.get(*i..*i + 4)?.try_into().ok()?);
        *i += 4;
        Some(v)
    };
    while i < f.len() {
        let (Some(x), Some(z), Some(_my), Some(dl)) = (rd(&mut i), rd(&mut i), rd(&mut i), rd(&mut i)) else { break };
        let data = &f[i..(i + dl as usize).min(f.len())];
        i += dl as usize;
        let Some(ml) = rd(&mut i) else { break };
        let map = &f[i..(i + ml as usize).min(f.len())];
        i += ml as usize;
        recs.push((x as i32, z as i32, data, map));
    }
    let n = recs.len();
    recs.par_iter().for_each(|&(x, z, data, map)| {
        let decoded = (|| Some((mapping(&zstd(map)?)?, columns(&zstd(data)?, min_y)?)))();
        match decoded {
            Some((entries, columns)) => each(Section { x: x * 64, z: z * 64, entries, columns }),
            None => log::warn!("mc import: DH section {x} {z} not read"),
        }
    });
    Ok(n)
}

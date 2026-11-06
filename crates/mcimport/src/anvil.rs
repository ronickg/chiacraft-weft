//! Anvil region files: `r.X.Z.mca`, 32x32 chunks each, a 4 KiB table of (sector offset,
//! sectors) then each chunk as length, compression byte and data (zlib by default; gzip, none
//! and LZ4 too; bit 7 set: the data is in `c.X.Z.mcc` beside it). A chunk is NBT with 16-high
//! sections, each a palette of block states and 4096 indices packed into longs (none span two
//! longs), and a palette of biomes over its 4x4x4 cells.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::nbt::{self, Tag};

/// A 16x16x16 section of a chunk.
pub struct Section {
    /// Section height (block y / 16).
    pub y: i32,
    /// Block states as `minecraft:oak_log[axis=y]`, properties sorted by name.
    pub palette: Vec<String>,
    /// Palette index per block, `(y * 16 + z) * 16 + x`; empty when the palette has one state.
    pub blocks: Vec<u16>,
    pub biome_palette: Vec<String>,
    /// Biome palette index per 4x4x4 cell, `(y * 4 + z) * 4 + x`.
    pub biomes: [u8; 64],
}

impl Section {
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        if self.blocks.is_empty() { 0 } else { self.blocks[(y * 16 + z) * 16 + x] as usize }
    }
}

/// A fully generated chunk.
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub sections: Vec<Section>,
}

/// The region files of a dimension's `region` folder, as (region x, region z, path).
pub fn regions(dir: &Path) -> Vec<(i32, i32, PathBuf)> {
    let mut out: Vec<(i32, i32, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let mut p = name.strip_prefix("r.")?.strip_suffix(".mca")?.split('.');
            let x = p.next()?.parse().ok()?;
            let z = p.next()?.parse().ok()?;
            Some((x, z, e.path()))
        })
        .collect();
    out.sort_by_key(|r| (r.0, r.1));
    out
}

/// Java's `LZ4BlockOutputStream`, which Minecraft writes for compression type 4: blocks of
/// "LZ4Block", a token, compressed and raw lengths and a checksum.
fn lz4_blocks(mut b: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    while b.len() >= 21 && &b[..8] == b"LZ4Block" {
        let token = b[8];
        let clen = u32::from_le_bytes(b[9..13].try_into().ok()?) as usize;
        let rlen = u32::from_le_bytes(b[13..17].try_into().ok()?) as usize;
        let data = b.get(21..21 + clen)?;
        if rlen == 0 {
            break;
        }
        match token & 0xF0 {
            0x10 => out.extend_from_slice(data),
            _ => out.extend_from_slice(&lz4_flex::block::decompress(data, rlen).ok()?),
        }
        b = &b[21 + clen..];
    }
    Some(out)
}

fn inflate(kind: u8, data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    match kind {
        1 => flate2::read::GzDecoder::new(data)
            .read_to_end(&mut out)
            .ok()?,
        2 => flate2::read::ZlibDecoder::new(data)
            .read_to_end(&mut out)
            .ok()?,
        3 => return Some(data.to_vec()),
        4 => return lz4_blocks(data),
        _ => return None,
    };
    Some(out)
}

/// A palette entry as one state string: `name` or `name[k=v,...]`. 26.3 writes plain strings
/// for states without properties, compounds `{id, properties}` (older: `{Name, Properties}`)
/// for the rest, and wraps strings as `{"": name}` in a list that mixes both.
fn state_name(t: &Tag) -> Option<String> {
    let t = t.unwrap_list_item();
    if let Some(s) = t.str() {
        return Some(s.to_string());
    }
    let name = t.get("id").or_else(|| t.get("Name"))?.str()?;
    let mut props: Vec<(String, String)> = t
        .get("properties")
        .or_else(|| t.get("Properties"))
        .map_or(&[][..], |p| p.fields())
        .iter()
        .filter_map(|(k, v)| {
            let v = v.unwrap_list_item();
            let s = v
                .str()
                .map(str::to_string)
                .or_else(|| v.int().map(|i| i.to_string()))?;
            Some((k.clone(), s))
        })
        .collect();
    if props.is_empty() {
        return Some(name.to_string());
    }
    props.sort();
    let inner: Vec<String> = props.iter().map(|(k, v)| format!("{k}={v}")).collect();
    Some(format!("{name}[{}]", inner.join(",")))
}

/// `count` indices of `bits` bits packed into longs, none spanning two.
fn unpack(data: &[i64], bits: usize, count: usize) -> Vec<u16> {
    let per = 64 / bits.max(1);
    let mask = (1u64 << bits) - 1;
    (0..count)
        .map(|i| {
            data.get(i / per)
                .map_or(0, |&w| ((w as u64) >> ((i % per) * bits)) & mask) as u16
        })
        .collect()
}

fn section(t: &Tag) -> Option<Section> {
    let y = t.get("Y")?.int()? as i32;
    let states = t.get("block_states")?;
    let palette: Vec<String> = states
        .get("palette")?
        .list()
        .iter()
        .filter_map(state_name)
        .collect();
    if palette.is_empty() {
        return None;
    }
    let blocks = if palette.len() > 1 {
        let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(4) as usize;
        let mut b = unpack(states.get("data").map_or(&[][..], |d| d.longs()), bits, 4096);
        b.iter_mut()
            .for_each(|i| *i = (*i).min(palette.len() as u16 - 1));
        b
    } else {
        Vec::new()
    };
    let (mut biome_palette, mut biomes) = (Vec::new(), [0u8; 64]);
    if let Some(b) = t.get("biomes") {
        biome_palette = b
            .get("palette")
            .map_or(&[][..], |p| p.list())
            .iter()
            .filter_map(|e| e.unwrap_list_item().str().map(str::to_string))
            .collect();
        if biome_palette.len() > 1 {
            let bits = (usize::BITS - (biome_palette.len() - 1).leading_zeros()) as usize;
            for (i, v) in unpack(b.get("data").map_or(&[][..], |d| d.longs()), bits, 64)
                .into_iter()
                .enumerate()
            {
                biomes[i] = (v as usize).min(biome_palette.len() - 1) as u8;
            }
        }
    }
    if biome_palette.is_empty() {
        biome_palette.push("minecraft:plains".into());
    }
    Some(Section { y, palette, blocks, biome_palette, biomes })
}

/// The chunk's NBT as a chunk, if it is fully generated (`Status` `minecraft:full`).
pub fn chunk(root: &Tag) -> Option<Chunk> {
    let status = root.get("Status").and_then(Tag::str).unwrap_or("");
    if status != "minecraft:full" && status != "full" {
        return None;
    }
    let x = root.get("xPos")?.int()? as i32;
    let z = root.get("zPos")?.int()? as i32;
    let mut sections: Vec<Section> = root
        .get("sections")?
        .list()
        .iter()
        .filter_map(section)
        .collect();
    sections.sort_by_key(|s| s.y);
    Some(Chunk { x, z, sections })
}

/// Every fully generated chunk of a region file. Broken chunks are logged and left out.
pub fn read_region(path: &Path) -> Vec<Chunk> {
    let Ok(f) = std::fs::read(path) else {
        log::warn!("mc import: cannot read {}", path.display());
        return Vec::new();
    };
    if f.len() < 8192 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for k in 0..1024 {
        let e = &f[k * 4..k * 4 + 4];
        let off = u32::from_be_bytes([0, e[0], e[1], e[2]]) as usize * 4096;
        if off == 0 {
            continue;
        }
        let Some(head) = f.get(off..off + 5) else { continue };
        let len = u32::from_be_bytes(head[..4].try_into().unwrap()) as usize;
        let kind = head[4];
        let raw = if kind & 0x80 != 0 {
            // Too big for the region: in its own file beside it.
            let (rx, rz) = (k as i32 % 32, k as i32 / 32);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let mut p = name.split('.').skip(1);
            let (Some(Ok(x)), Some(Ok(z))) = (p.next().map(str::parse::<i32>), p.next().map(str::parse::<i32>)) else { continue };
            match std::fs::read(path.with_file_name(format!("c.{}.{}.mcc", x * 32 + rx, z * 32 + rz))) {
                Ok(b) => b,
                Err(_) => continue,
            }
        } else {
            match f.get(off + 5..off + 4 + len) {
                Some(b) => b.to_vec(),
                None => continue,
            }
        };
        let Some(data) = inflate(kind & 0x7F, &raw) else {
            log::warn!("mc import: {} chunk {k}: compression {kind} not read", path.display());
            continue;
        };
        match nbt::parse(&data) {
            Some(root) => out.extend(chunk(&root)),
            None => log::warn!("mc import: {} chunk {k}: broken NBT", path.display()),
        }
    }
    out
}

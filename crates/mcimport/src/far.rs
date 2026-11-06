//! The far-terrain grid: what distant tiles show of an imported world beyond the real cubes,
//! instead of the terrain formula. Distant tiles sample their cells' middles: (4i + 2) for
//! 4-block cells, multiples of 4 for 8-block cells and wider. So two grids every 4 blocks over the
//! same square: `mid` at (4i + 2, 4j + 2) and `corner` at (4i, 4j); a column on neither takes the
//! nearest corner. A cell is the column's top block that shows from afar (solid, liquid or leaves:
//! Distant Horizons draws leaves too, and our tiles have no crowns for it), the block under it
//! and the biome.
//!
//! Saved as `CCFAR003` and an LZ4 body (lengths and numbers little-endian): x0, z0, cells across
//! and deep, the state and biome names (u16 count, u16-length strings), then the mid cells and
//! the corner cells (rows of z), each y (i16, `NONE` for nothing), top and under (u16 indices
//! into the names), biome (u8 index) and, for water, its depth down to the bed (u8, blocks).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub const NONE: i16 = i16::MIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub y: i16,
    pub top: u16,
    pub under: u16,
    pub biome: u8,
    /// Water: blocks from its surface down to the bed (seagrass and kelp count as water); 0
    /// otherwise.
    pub depth: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { y: NONE, top: 0, under: 0, biome: 0, depth: 0 }
    }
}

pub struct FarGrid {
    /// Block corner of the square.
    pub x0: i32,
    pub z0: i32,
    /// Cells across and deep (each grid).
    pub w4: u32,
    pub h4: u32,
    /// Our state names (`base:grass`) and biome names the cells index.
    pub states: Vec<String>,
    pub biomes: Vec<String>,
    pub mid: Vec<Cell>,
    pub corner: Vec<Cell>,
}

impl FarGrid {
    /// The cell for column (x, z): exact on either grid, else the nearest corner; none outside
    /// the square or where nothing shows.
    pub fn sample(&self, x: i32, z: i32) -> Option<Cell> {
        let (dx, dz) = (x - self.x0, z - self.z0);
        let (grid, i, j) = if dx.rem_euclid(4) == 2 && dz.rem_euclid(4) == 2 {
            (&self.mid, dx.div_euclid(4), dz.div_euclid(4))
        } else {
            (&self.corner, (dx + 2).div_euclid(4), (dz + 2).div_euclid(4))
        };
        if i < 0 || j < 0 || i >= self.w4 as i32 || j >= self.h4 as i32 {
            return None;
        }
        let c = grid[(j as u32 * self.w4 + i as u32) as usize];
        (c.y != NONE).then_some(c)
    }

    pub fn write(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(16 + (self.mid.len() + self.corner.len()) * 8);
        for v in [self.x0, self.z0, self.w4 as i32, self.h4 as i32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for list in [&self.states, &self.biomes] {
            b.extend_from_slice(&(list.len() as u16).to_le_bytes());
            for s in list.iter() {
                b.extend_from_slice(&(s.len() as u16).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            }
        }
        for c in self.mid.iter().chain(&self.corner) {
            b.extend_from_slice(&c.y.to_le_bytes());
            b.extend_from_slice(&c.top.to_le_bytes());
            b.extend_from_slice(&c.under.to_le_bytes());
            b.push(c.biome);
            b.push(c.depth);
        }
        let mut out = b"CCFAR003".to_vec();
        out.extend_from_slice(&lz4_flex::compress_prepend_size(&b));
        out
    }

    pub fn read(data: &[u8]) -> Option<FarGrid> {
        if data.get(..8) != Some(b"CCFAR003") {
            return None;
        }
        let b = lz4_flex::decompress_size_prepended(&data[8..]).ok()?;
        let mut i = 0;
        let mut take = |n: usize| -> Option<&[u8]> {
            let s = b.get(i..i + n)?;
            i += n;
            Some(s)
        };
        let mut head = [0i32; 4];
        for h in &mut head {
            *h = i32::from_le_bytes(take(4)?.try_into().ok()?);
        }
        let mut lists: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        for list in &mut lists {
            let n = u16::from_le_bytes(take(2)?.try_into().ok()?) as usize;
            for _ in 0..n {
                let l = u16::from_le_bytes(take(2)?.try_into().ok()?) as usize;
                list.push(String::from_utf8_lossy(take(l)?).into_owned());
            }
        }
        let (w4, h4) = (head[2] as u32, head[3] as u32);
        let mut cells = |n: usize| -> Option<Vec<Cell>> {
            let raw = take(n.checked_mul(8)?)?;
            Some(
                raw.as_chunks::<8>()
                    .0
                    .iter()
                    .map(|c| Cell {
                        y: i16::from_le_bytes([c[0], c[1]]),
                        top: u16::from_le_bytes([c[2], c[3]]),
                        under: u16::from_le_bytes([c[4], c[5]]),
                        biome: c[6],
                        depth: c[7],
                    })
                    .collect(),
            )
        };
        let mid = cells((w4 * h4) as usize)?;
        let corner = cells((w4 * h4) as usize)?;
        let [states, biomes] = lists;
        Some(FarGrid { x0: head[0], z0: head[1], w4, h4, states, biomes, mid, corner })
    }
}

/// What a block is to the far view.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shows {
    /// It is the top when nothing that shows is above it.
    Top,
    /// Seen through or too small (air, plants, snow layers): the top is under it.
    Through,
}

/// Names interned to u16 indices.
#[derive(Default)]
struct Names {
    list: Vec<String>,
    index: HashMap<String, u16>,
}

impl Names {
    fn get(&mut self, s: &str) -> u16 {
        if let Some(&i) = self.index.get(s) {
            return i;
        }
        let i = self.list.len().min(u16::MAX as usize) as u16;
        self.list.push(s.to_string());
        self.index.insert(s.to_string(), i);
        i
    }
}

/// Builds the grid from an exported DH file (`dh::for_each_section`). `block`: a Minecraft state
/// to our state name and whether it shows; `biome`: a Minecraft biome to ours.
pub fn from_dh(
    path: &Path,
    min_y: i32,
    block: impl Fn(&str) -> (String, Shows) + Sync,
    biome: impl Fn(&str) -> String + Sync,
) -> std::io::Result<FarGrid> {
    let names = Mutex::new((Names::default(), Names::default()));
    // Per section: its corner, mid and corner cells.
    type Part = (i32, i32, Vec<Cell>, Vec<Cell>);
    let parts: Mutex<Vec<Part>> = Mutex::new(Vec::new());
    crate::dh::for_each_section(path, min_y, |s| {
        // Per mapping entry: our state, whether it shows, the biome, and whether it is water.
        let wet: Vec<bool> = s
            .entries
            .iter()
            .map(|(_, st)| {
                ["minecraft:water", "minecraft:bubble_column", "minecraft:seagrass", "minecraft:tall_seagrass", "minecraft:kelp"]
                    .iter()
                    .any(|w| st.starts_with(w))
            })
            .collect();
        let entries: Vec<(u16, Shows, u8)> = {
            let resolved: Vec<(String, Shows, String)> = s
                .entries
                .iter()
                .map(|(b, st)| {
                    let (name, shows) = block(st);
                    (name, shows, biome(b))
                })
                .collect();
            let mut n = names.lock().unwrap();
            resolved
                .iter()
                .map(|(name, shows, b)| (n.0.get(name), *shows, n.1.get(b).min(u8::MAX as u16) as u8))
                .collect()
        };
        let cell = |x: i32, z: i32| -> Cell {
            let runs = s.column(x, z);
            for (k, r) in runs.iter().enumerate() {
                let Some(&(top, shows, biome)) = entries.get(r.id as usize) else { continue };
                if shows != Shows::Top {
                    continue;
                }
                let under = runs
                    .get(k + 1)
                    .and_then(|u| entries.get(u.id as usize))
                    .filter(|u| u.1 == Shows::Top)
                    .map_or(top, |u| u.0);
                // Water's depth: down past every water run to the first block that isn't.
                let depth = if wet.get(r.id as usize).copied().unwrap_or(false) {
                    let bed = runs[k..]
                        .iter()
                        .find(|b| !wet.get(b.id as usize).copied().unwrap_or(false));
                    bed.map_or(0, |b| (r.top() - b.top()).clamp(0, u8::MAX as i32)) as u8
                } else {
                    0
                };
                return Cell { y: r.top().clamp(i16::MIN as i32 + 1, i16::MAX as i32) as i16, top, under, biome, depth };
            }
            Cell::default()
        };
        let mut mid = Vec::with_capacity(256);
        let mut corner = Vec::with_capacity(256);
        for j in 0..16 {
            for i in 0..16 {
                mid.push(cell(4 * i + 2, 4 * j + 2));
                corner.push(cell(4 * i, 4 * j));
            }
        }
        parts.lock().unwrap().push((s.x, s.z, mid, corner));
    })?;
    let parts = parts.into_inner().unwrap();
    let (names_s, names_b) = names.into_inner().unwrap();
    if parts.is_empty() {
        return Err(std::io::Error::other("no DH sections"));
    }
    let x0 = parts.iter().map(|p| p.0).min().unwrap();
    let z0 = parts.iter().map(|p| p.1).min().unwrap();
    let w4 = ((parts.iter().map(|p| p.0).max().unwrap() - x0) / 64 + 1) as u32 * 16;
    let h4 = ((parts.iter().map(|p| p.1).max().unwrap() - z0) / 64 + 1) as u32 * 16;
    let mut grid = FarGrid {
        x0,
        z0,
        w4,
        h4,
        states: names_s.list,
        biomes: names_b.list,
        mid: vec![Cell::default(); (w4 * h4) as usize],
        corner: vec![Cell::default(); (w4 * h4) as usize],
    };
    for (x, z, mid, corner) in parts {
        let (si, sj) = (((x - x0) / 64) as u32, ((z - z0) / 64) as u32);
        for j in 0..16u32 {
            for i in 0..16u32 {
                let at = ((sj * 16 + j) * w4 + si * 16 + i) as usize;
                grid.mid[at] = mid[(j * 16 + i) as usize];
                grid.corner[at] = corner[(j * 16 + i) as usize];
            }
        }
    }
    Ok(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_round_trips_and_samples() {
        let mut g = FarGrid {
            x0: -64,
            z0: 0,
            w4: 16,
            h4: 16,
            states: vec!["base:grass".into(), "base:dirt".into()],
            biomes: vec!["base:plains".into()],
            mid: vec![Cell::default(); 256],
            corner: vec![Cell::default(); 256],
        };
        g.mid[16 + 1] = Cell { y: 70, top: 0, under: 1, biome: 0, depth: 3 };
        g.corner[2 * 16 + 2] = Cell { y: 71, top: 0, under: 1, biome: 0, depth: 0 };
        let back = FarGrid::read(&g.write()).unwrap();
        assert_eq!(back.sample(-64 + 6, 6).map(|c| (c.y, c.depth)), Some((70, 3)));
        assert_eq!(back.sample(-64 + 8, 8).map(|c| c.y), Some(71));
        assert_eq!(back.sample(-64 + 9, 7).map(|c| c.y), Some(71));
        assert_eq!(back.sample(-67, 0), None);
        assert_eq!(back.states, g.states);
    }
}

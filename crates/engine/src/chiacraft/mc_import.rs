//! Test only (Ronald, 11-06: "let's load that same Minecraft world ... it makes testing even
//! easier"): loads a Minecraft world into a new world block for block, so our look can be shot
//! from the camera of the Bliss + Distant Horizons comparison in the identical scene.
//!
//! `weft --world <name> --import-mc <folder>` (folder: the Minecraft world's `region` files, or a
//! save with `dimensions/minecraft/overworld/region`, and optionally `dh0.bin`, Distant Horizons'
//! far data exported by `tools/mcimport/dh_export.py`). Before the world opens the first time,
//! every fully generated chunk is written as saved cubes of `--import-mc-dim` (the overworld by
//! default), so they load instead of being generated, and the far data becomes the dimension's
//! `far.bin`, which distant tiles show instead of the terrain formula (`client::lod`). Done once
//! per world (`mc_import.txt` in its folder says what was made and every block that stood in for
//! one we lack). Ronald's world is local only: never commit it or build it in.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use chiacraft_mcimport::{anvil, far, map};
use glam::IVec3;

use crate::content::{AIR, Content};
use crate::saves::{Storage, WorldMeta};
use crate::world::{Cube, cube};

/// Minecraft names to ours, each worked out once.
struct Resolver<'c> {
    c: &'c Content,
    states: Mutex<HashMap<String, u32>>,
    biomes: Mutex<HashMap<String, (u16, String)>>,
    /// Minecraft states that took a stand-in, with what they became and how often.
    stand_ins: Mutex<HashMap<String, (String, u64)>>,
}

impl<'c> Resolver<'c> {
    fn new(c: &'c Content) -> Self {
        Resolver { c, states: Mutex::default(), biomes: Mutex::default(), stand_ins: Mutex::default() }
    }

    fn state(&self, mc: &str) -> u32 {
        if let Some(&s) = self.states.lock().unwrap().get(mc) {
            return s;
        }
        let (candidates, exact) = map::block(mc);
        let mut chosen = None;
        for (i, name) in candidates.iter().enumerate() {
            if let Some(s) = self.c.state_by_name(name) {
                chosen = Some((s, i == 0 && exact));
                break;
            }
        }
        let (s, exact) = chosen.unwrap_or((AIR, false));
        if !exact {
            let name = self.c.state_name(s);
            log::debug!("mc import: {mc} stands in as {name} (no exact match)");
            self.stand_ins
                .lock()
                .unwrap()
                .insert(mc.to_string(), (name, 0));
        }
        self.states.lock().unwrap().insert(mc.to_string(), s);
        s
    }

    fn count_stand_in(&self, mc: &str, n: u64) {
        if let Some(e) = self.stand_ins.lock().unwrap().get_mut(mc) {
            e.1 += n;
        }
    }

    fn biome(&self, mc: &str) -> (u16, String) {
        if let Some(b) = self.biomes.lock().unwrap().get(mc) {
            return b.clone();
        }
        let candidates = map::biome(mc);
        let found = candidates
            .iter()
            .find_map(|n| {
                self.c
                    .biome_names
                    .get(*n)
                    .map(|&b| (b as u16, n.to_string()))
            })
            .unwrap_or((
                0,
                candidates
                    .last()
                    .copied()
                    .unwrap_or("base:plains")
                    .to_string(),
            ));
        if candidates.first() != Some(&found.1.as_str()) {
            log::info!("mc import: biome {mc} stands in as {}", found.1);
        }
        self.biomes
            .lock()
            .unwrap()
            .insert(mc.to_string(), found.clone());
        found
    }
}

/// Whether a state is a column's top from afar: solid and opaque, a liquid, or leaves.
fn shows(c: &Content, s: u32) -> far::Shows {
    use weft_abi::block_flags as bf;
    let st = &c.states[s as usize];
    let solid = st.flags & bf::SOLID != 0 && st.flags & bf::OPAQUE != 0;
    if s != AIR && (solid || st.liquid.is_some() || c.state_name(s).contains("leaves")) { far::Shows::Top } else { far::Shows::Through }
}

/// The chunk's cubes from `lo` to `hi` (cube heights; missing sections are air, so generation
/// never fills them), lit as generation lights them.
fn chunk_cubes(c: &Content, r: &Resolver, dim: usize, chunk: &anvil::Chunk, lo: i32, hi: i32) -> Vec<(IVec3, Cube)> {
    let sky = c.dims[dim].sky_light;
    let mut out = Vec::with_capacity((hi - lo + 1).max(0) as usize);
    for y in lo..=hi {
        let pos = IVec3::new(chunk.x, y, chunk.z);
        let Some(sec) = chunk.sections.iter().find(|s| s.y == y) else {
            out.push((pos, Cube::uniform(AIR, 0)));
            continue;
        };
        let ids: Vec<u32> = sec.palette.iter().map(|p| r.state(p)).collect();
        let mut counts = vec![0u64; ids.len()];
        let mut states = Box::new([AIR; cube::VOLUME]);
        for ly in 0..16 {
            for lz in 0..16 {
                for lx in 0..16 {
                    let k = sec.index(lx, ly, lz);
                    counts[k] += 1;
                    states[cube::index(lx, ly, lz)] = ids[k];
                }
            }
        }
        for (p, n) in sec.palette.iter().zip(counts) {
            if n > 0 {
                r.count_stand_in(p, n);
            }
        }
        let mut cube = Cube::from_states(&states);
        // A cube's columns take the biome of their 4x4 cell halfway up it.
        let mut biomes = Box::new([0u16; 256]);
        let ids: Vec<u16> = sec.biome_palette.iter().map(|b| r.biome(b).0).collect();
        for z in 0..16 {
            for x in 0..16 {
                biomes[z * 16 + x] = ids
                    .get(sec.biomes[(2 * 4 + z / 4) * 4 + x / 4] as usize)
                    .copied()
                    .unwrap_or(0);
            }
        }
        cube.biomes = Some(biomes);
        crate::world::light::prelight(c, &mut cube, sky);
        out.push((pos, cube));
    }
    out
}

fn first_dir(list: &[PathBuf]) -> Option<PathBuf> {
    list.iter().find(|p| p.is_dir()).cloned()
}

/// Imports the Minecraft world `--import-mc` names into `meta`'s world, if asked and not done yet.
pub fn maybe_import(meta: &mut WorldMeta, c: &Content) {
    let opts = crate::app::api::launch_options();
    let Some(src) = opts.get("import-mc") else { return };
    let marker = meta.dir.join("mc_import.txt");
    if marker.exists() {
        log::info!("mc import: '{}' already has the Minecraft world ({})", meta.name, marker.display());
        return;
    }
    let dim_name = opts
        .get("import-mc-dim")
        .map_or("base:overworld", String::as_str);
    let Some(dim) = c.dims.iter().position(|d| d.name == dim_name) else {
        log::error!("mc import: no dimension {dim_name}");
        return;
    };
    let src = PathBuf::from(src);
    let Some(region_dir) = first_dir(&[src.join("region"), src.join("dimensions/minecraft/overworld/region"), src.clone()]) else {
        log::error!("mc import: no region folder in {}", src.display());
        return;
    };
    // A test scene: nothing spawns in it from the first tick (scripts set the rule only after
    // they load, and players spawned into the imported land were killed before that).
    meta.configs.retain(|(name, _)| name != "core:mob_spawning");
    meta.configs.push(("core:mob_spawning".into(), 0.0));
    meta.save();
    let start = Instant::now();
    let def = &c.dims[dim];
    // Minecraft's overworld is y -64 to 319: cubes -4 to 19.
    let (lo, hi) = (def.min_cube_y.max(-4), def.max_cube_y.min(19));
    let r = Resolver::new(c);
    let storage = Mutex::new(Storage::open(meta.clone(), c));
    let regions = anvil::regions(&region_dir);
    log::info!("mc import: {} region files from {} into {} cubes {lo} to {hi}", regions.len(), region_dir.display(), dim_name);
    let chunks = std::sync::atomic::AtomicU64::new(0);
    {
        use rayon::prelude::*;
        regions.par_iter().for_each(|(_, _, path)| {
            for chunk in anvil::read_region(path) {
                let cubes = chunk_cubes(c, &r, dim, &chunk, lo, hi);
                let mut st = storage.lock().unwrap();
                for (pos, cube) in &cubes {
                    st.save_imported(c, dim as u32, *pos, cube);
                }
                chunks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }
    let chunks = chunks.into_inner();
    let near_secs = start.elapsed().as_secs_f64();
    let mut storage = storage.into_inner().unwrap();
    let mut report = format!("minecraft world: {}\nchunks: {chunks} ({near_secs:.0} s)\n", src.display());
    // The far view: Distant Horizons' columns.
    let dh = [src.join("dh0.bin"), region_dir.join("../dh0.bin")]
        .into_iter()
        .find(|p| p.is_file());
    match dh {
        Some(path) => {
            let t = Instant::now();
            let grid = far::from_dh(
                &path,
                -64,
                |mc| {
                    let s = r.state(mc);
                    (c.state_name(s), shows(c, s))
                },
                |b| r.biome(b).1,
            );
            match grid {
                Ok(g) => {
                    let bytes = g.write();
                    let out = storage.dim_dir(dim as u32).join("far.bin");
                    let _ = std::fs::create_dir_all(storage.dim_dir(dim as u32));
                    match std::fs::write(&out, &bytes) {
                        Ok(()) => {
                            report += &format!(
                                "far grid: {} x {} blocks from x {} z {}, {} KB ({:.0} s)\n",
                                g.w4 * 4,
                                g.h4 * 4,
                                g.x0,
                                g.z0,
                                bytes.len() / 1024,
                                t.elapsed().as_secs_f64()
                            );
                            storage.far[dim] = Some(std::sync::Arc::new(bytes));
                        }
                        Err(e) => log::error!("mc import: writing {}: {e}", out.display()),
                    }
                }
                Err(e) => log::error!("mc import: far data {}: {e}", path.display()),
            }
        }
        None => report += "far grid: none (no dh0.bin)\n",
    }
    storage.flush();
    let mut stand_ins: Vec<(String, (String, u64))> = r.stand_ins.into_inner().unwrap().into_iter().collect();
    stand_ins.sort_by(|a, b| b.1.1.cmp(&a.1.1));
    report += "stand-ins (minecraft state -> ours, blocks):\n";
    for (mc, (ours, n)) in &stand_ins {
        report += &format!("  {mc} -> {ours} {n}\n");
    }
    let _ = std::fs::write(&marker, &report);
    log::info!("mc import: done in {:.0} s: {}", start.elapsed().as_secs_f64(), report.lines().take(3).collect::<Vec<_>>().join("; "));
}

/// An imported world's far terrain on the client: the grid with its names as runtime ids.
pub struct FarTerrain {
    grid: far::FarGrid,
    states: Vec<u32>,
    biomes: Vec<u32>,
}

impl FarTerrain {
    pub fn read(c: &Content, bytes: &[u8]) -> Option<FarTerrain> {
        if bytes.is_empty() {
            return None;
        }
        let grid = far::FarGrid::read(bytes)?;
        let states = grid
            .states
            .iter()
            .map(|n| c.state_by_name(n).unwrap_or(AIR))
            .collect();
        let biomes = grid
            .biomes
            .iter()
            .map(|n| c.biome_names.get(n).copied().unwrap_or(0))
            .collect();
        log::info!("far terrain of an imported world: {} x {} blocks", grid.w4 * 4, grid.h4 * 4);
        Some(FarTerrain { grid, states, biomes })
    }

    /// Column (x, z)'s top height, top block, block under it and biome.
    pub fn cell(&self, x: i32, z: i32) -> Option<(i32, u32, u32, u32)> {
        let cell = self.grid.sample(x, z)?;
        let st = |i: u16| self.states.get(i as usize).copied().unwrap_or(AIR);
        Some((cell.y as i32, st(cell.top), st(cell.under), self.biomes.get(cell.biome as usize).copied().unwrap_or(0)))
    }
}

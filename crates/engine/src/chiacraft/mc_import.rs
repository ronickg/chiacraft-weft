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
        let (mut candidates, mut exact) = map::block(mc);
        // A data package with the id `mc` may define Minecraft's own blocks by their names (a test
        // environment: the RTX showcase world with its own textures and material maps,
        // `notes/RTX_SHOWCASE.md`); such a block comes before our nearest one. Without that
        // package nothing changes.
        let own = format!("mc:{}", map::State::parse(mc).name);
        if self.c.state_by_name(&own).is_some() {
            candidates.insert(0, own);
            exact = true;
        }
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

/// Each column's biome at its surface: the 4x4x4 cell of its highest block that isn't air (the
/// top section's for an empty column).
fn surface_biomes(r: &Resolver, chunk: &anvil::Chunk) -> [u16; 256] {
    let air = |name: &str| matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air");
    let mut out = [u16::MAX; 256];
    for sec in chunk.sections.iter().rev() {
        let ids: Vec<u16> = sec.biome_palette.iter().map(|b| r.biome(b).0).collect();
        let cell = |x: usize, y: usize, z: usize| {
            ids.get(sec.biomes[(y / 4 * 4 + z / 4) * 4 + x / 4] as usize)
                .copied()
                .unwrap_or(0)
        };
        for z in 0..16 {
            for x in 0..16 {
                let o = &mut out[z * 16 + x];
                if *o != u16::MAX {
                    continue;
                }
                if let Some(y) = (0..16)
                    .rev()
                    .find(|&y| !air(&sec.palette[sec.index(x, y, z)]))
                {
                    *o = cell(x, y, z);
                }
            }
        }
    }
    let top = chunk.sections.last();
    for (i, o) in out.iter_mut().enumerate().filter(|(_, o)| **o == u16::MAX) {
        *o = top.map_or(0, |sec| {
            let id = sec.biomes[(3 * 4 + i / 16 / 4) * 4 + i % 16 / 4] as usize;
            sec.biome_palette.get(id).map_or(0, |b| r.biome(b).0)
        });
    }
    out
}

/// The chunk's cubes from `lo` to `hi` (cube heights; missing sections are air, so generation
/// never fills them), lit as generation lights them.
fn chunk_cubes(c: &Content, r: &Resolver, dim: usize, chunk: &anvil::Chunk, lo: i32, hi: i32) -> Vec<(IVec3, Cube)> {
    let sky = c.dims[dim].sky_light;
    let surface = surface_biomes(r, chunk);
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
        // Our biomes are per column (the client tints a column by the first of its cubes to
        // arrive): every cube takes the biome at its column's surface, not Minecraft's 3D one,
        // whose caves turned an island's grass and leaves teal.
        let biomes = Box::new(surface);
        cube.biomes = Some(biomes);
        crate::world::light::prelight(c, &mut cube, sky);
        out.push((pos, cube));
    }
    out
}

/// The column new players start on: the vista of the Bliss + DH shots (x, z).
const SPAWN: [i32; 2] = [165, 168];
/// The file in the world's folder that says where they start: `<dimension> x y z`.
const SPAWN_FILE: &str = "mc_spawn.txt";

/// The highest block of column (x, z) that stands solid, among a chunk's cubes.
fn top_solid(c: &Content, cubes: &[(IVec3, Cube)], x: i32, z: i32) -> Option<i32> {
    use weft_abi::block_flags as bf;
    cubes.iter().rev().find_map(|(pos, cube)| {
        (0..16usize).rev().find_map(|y| {
            let s = cube.get(cube::index(x as usize, y, z as usize));
            (s != AIR && c.states[s as usize].flags & bf::SOLID != 0).then_some(pos.y * 16 + y as i32)
        })
    })
}

/// Where new players start in an imported world (`mc_spawn.txt`), if it is one: the dimension and
/// the place.
pub fn spawn(c: &Content, world_dir: &std::path::Path) -> Option<(u32, glam::DVec3)> {
    let text = std::fs::read_to_string(world_dir.join(SPAWN_FILE)).ok()?;
    let mut w = text.split_whitespace();
    let name = w.next()?;
    let dim = c.dims.iter().position(|d| d.name == name)? as u32;
    let mut n = || w.next()?.parse::<f64>().ok();
    Some((dim, glam::DVec3::new(n()?, n()?, n()?)))
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
    // Where new players start: on the ground under the vista (our own spawn search would put
    // them anywhere in the imported land, the sea included).
    let ground = Mutex::new(None);
    {
        use rayon::prelude::*;
        regions.par_iter().for_each(|(_, _, path)| {
            for chunk in anvil::read_region(path) {
                let cubes = chunk_cubes(c, &r, dim, &chunk, lo, hi);
                if (chunk.x, chunk.z) == (SPAWN[0] >> 4, SPAWN[1] >> 4) {
                    *ground.lock().unwrap() = top_solid(c, &cubes, SPAWN[0] & 15, SPAWN[1] & 15);
                }
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
    match ground.into_inner().unwrap() {
        Some(y) => {
            let _ = std::fs::write(meta.dir.join(SPAWN_FILE), format!("{dim_name} {}.5 {} {}.5\n", SPAWN[0], y + 1, SPAWN[1]));
            report += &format!("spawn: {} {} {}\n", SPAWN[0], y + 1, SPAWN[1]);
        }
        None => report += "spawn: none (no ground under the vista)\n",
    }
    let mut stand_ins: Vec<(String, (String, u64))> = r.stand_ins.into_inner().unwrap().into_iter().collect();
    stand_ins.sort_by_key(|e| std::cmp::Reverse(e.1.1));
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

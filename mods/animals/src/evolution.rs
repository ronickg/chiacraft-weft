//! Evolution (`notes/ANIMALS.md`, "Evolution"): how common each gene variant is changes where the
//! animals live, as it does in real populations. Every species has a gene pool per place (a cell
//! of 256 x 256 blocks and its habitat): wild animals spawning there are drawn from it
//! (`coats::upkeep`), and it starts as the species' gene file has it. Each in-game day is a
//! generation: a population of the pool lives, the ones whose coat stands out from the habitat
//! (sandy raptors on grass, dark ones on snow) leave fewer young (natural selection, by the
//! species' `camouflage`), chance decides much in a small population (drift), alleles now and
//! then mutate, and a little of each neighboring place's pool mixes in (migration). Mammals also
//! grow bigger where it is cold and smaller where it is hot (Bergmann's rule). So raptors in
//! deserts turn sandy over a few weeks, ones in forests dark, and neighbors stay alike.
//!
//! Players change them too: a wild animal that dies (hunted or otherwise) or is tamed takes its
//! genes out of its place's pool, and a tame its owner releases puts its genes in (`count`).
//!
//! Pools evolve when they are next needed (up to `CATCH_UP` generations at once), as numbers, not
//! animals, so a pool far from everyone costs nothing. They are saved in the world
//! (`mods/animals/pools.json`, through the engine's bridge).

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use weft_sdk::*;

pub use animals_core::evolution::*;

use crate::coats::{self, Pool};
use crate::species::SPECIES;

static COLD: Tag = Tag::named("cold");
static DRY: Tag = Tag::named("dry");
static FOREST: Tag = Tag::named("forest");
static MOUNTAIN: Tag = Tag::named("mountain");

/// The habitat of the biome at `pos`, by the biome's tags.
pub fn habitat_at(w: &World, pos: [f64; 3]) -> Habitat {
    let b = w.biome(pos.map(|v| v.floor() as i32));
    match (COLD.on_biome(b), FOREST.on_biome(b)) {
        (true, true) => Habitat::Taiga,
        (true, false) => Habitat::Snow,
        _ if DRY.on_biome(b) => Habitat::Desert,
        (false, true) => Habitat::Forest,
        _ if MOUNTAIN.on_biome(b) => Habitat::Mountains,
        _ => Habitat::Grassland,
    }
}

// ------------------------------------------------------------------ pools, kept and saved

/// A species' pool in one place.
#[derive(Serialize, Deserialize, Clone)]
struct Local {
    /// The generation (day) it has evolved to.
    day: u64,
    pool: Pool,
}

/// The pools of the world open now, by `key`, and the folder they are saved in.
#[derive(Default)]
struct Pools {
    dir: String,
    all: HashMap<String, Local>,
    dirty: bool,
}

static POOLS: Mutex<Option<Pools>> = Mutex::new(None);

/// A pool's key: species, place and habitat (`raptor 3 -2 desert`).
fn key(index: usize, cell: (i64, i64), h: Habitat) -> String {
    format!("{} {} {} {}", SPECIES[index].name, cell.0, cell.1, h.name())
}

fn cell_of(pos: [f64; 3]) -> (i64, i64) {
    ((pos[0] / CELL).floor() as i64, (pos[2] / CELL).floor() as i64)
}

/// Runs `f` on the open world's pools, read from its save the first time (and again when
/// another world is opened).
fn with_pools<R>(f: impl FnOnce(&mut Pools) -> R) -> R {
    let mut guard = POOLS.lock().unwrap_or_else(|e| e.into_inner());
    let dir = crate::data_dir();
    if guard.as_ref().is_none_or(|p| p.dir != dir) {
        let all = std::fs::read_to_string(std::path::Path::new(&dir).join("pools.json"))
            .ok()
            .and_then(|t| match serde_json::from_str(&t) {
                Ok(a) => Some(a),
                Err(e) => {
                    weft_sdk::error!("animals pools.json: {e}");
                    None
                }
            })
            .unwrap_or_default();
        *guard = Some(Pools { dir, all, dirty: false });
    }
    f(guard.as_mut().unwrap())
}

/// Writes the pools if they changed (`init`: every thirty seconds).
fn save() {
    let (dir, text) = {
        let mut guard = POOLS.lock().unwrap_or_else(|e| e.into_inner());
        let Some(p) = guard.as_mut().filter(|p| p.dirty && !p.dir.is_empty()) else { return };
        p.dirty = false;
        (p.dir.clone(), serde_json::to_string(&p.all).unwrap_or_default())
    };
    let path = std::path::Path::new(&dir).join("pools.json");
    let tmp = path.with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, text).and_then(|()| std::fs::rename(&tmp, &path)) {
        weft_sdk::warn!("saving animals pools.json: {e}");
    }
}

static TICKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn init(r: &mut Registry) {
    // A wild animal that dies (hunted, killed by another, starved...) is gone from its pool.
    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            if !crate::animal::tamed(&w, d.entity) {
                count_at(&w, d.entity, d.pos, -1.0);
            }
        }
    });
    r.world_tick(|_| {
        if TICKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 600 == 599 {
            save();
        }
    });
}

/// Species `index`'s pool in a place, evolved to `day` (made from the gene file the first time).
fn evolved(p: &mut Pools, index: usize, cell: (i64, i64), h: Habitat, day: u64, r: &mut impl coats::Rng) -> Option<Pool> {
    let k = key(index, cell, h);
    let new = !p.all.contains_key(&k);
    if new {
        // As if it had evolved since the world began (up to `HISTORY` generations).
        let start = day.saturating_sub(HISTORY);
        p.all
            .insert(k.clone(), Local { day: start, pool: coats::wild_pool(index, h.name())? });
        p.dirty = true;
    }
    let from = p.all[&k].day;
    for _ in 0..day
        .saturating_sub(from)
        .min(if new { HISTORY } else { CATCH_UP })
    {
        step(p, &k, index, cell, h, r);
    }
    let local = p.all.get_mut(&k)?;
    if local.day < day {
        local.day = day;
        p.dirty = true;
    }
    Some(local.pool.clone())
}

/// One generation of the pool at `k`, with the neighbors' pools (the places around, any habitat)
/// mixing in.
fn step(p: &mut Pools, k: &str, index: usize, cell: (i64, i64), h: Habitat, r: &mut impl coats::Rng) {
    let Some(mut next) = p.all.get(k).and_then(|l| generation(index, &l.pool, h, r)) else { return };
    let neighbors: Vec<Pool> = (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dz| (cell.0 + dx, cell.1 + dz)))
        .flat_map(|c| HABITATS.map(|h2| key(index, c, h2)))
        .filter(|n| n != k)
        .filter_map(|n| p.all.get(&n).map(|l| l.pool.clone()))
        .collect();
    if !neighbors.is_empty() {
        let share = MIGRATION / neighbors.len() as f32;
        for n in &neighbors {
            mix_in(&mut next, n, share);
        }
    }
    if let Some(l) = p.all.get_mut(k) {
        l.pool = next;
        l.day += 1;
        p.dirty = true;
    }
}

/// Animal `id` leaves the wild where it is (`sign` -1: it died, or was tamed) or joins it (+1:
/// released): its genes out of or into its place's pool, as one animal of the population. So
/// hunting the white ones makes white rarer there, and released tames spread their coats.
pub fn count(w: &World, id: u64, sign: f32) {
    if let Some(st) = w.entity(id) {
        count_at(w, id, st.pos, sign);
    }
}

/// `count` for animal `id` at `pos` (one that just died is nowhere any more).
fn count_at(w: &World, id: u64, pos: [f64; 3], sign: f32) {
    let Some(g) = w.component(id, &coats::GENOME).copied() else { return };
    let Some(index) = (g.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
    else {
        return;
    };
    let (cell, h, day) = (cell_of(pos), habitat_at(w, pos), w.time() / GENERATION);
    let mut r = coats::world_rng(w);
    with_pools(|p| {
        evolved(p, index, cell, h, day, &mut r)?;
        let l = p.all.get_mut(&key(index, cell, h))?;
        coats::count_in(index, &mut l.pool, &g, sign, POPULATION as f32);
        p.dirty = true;
        weft_sdk::info!("animal {id} {} the wild pool {}", if sign < 0.0 { "left" } else { "joined" }, key(index, cell, h));
        Some(())
    });
}

/// The pool a wild animal of species `index` spawning at `pos` is drawn from: its kind's there,
/// as it has evolved by today.
pub fn pool_at(w: &World, index: usize, pos: [f64; 3]) -> Option<Pool> {
    let (cell, h, day) = (cell_of(pos), habitat_at(w, pos), w.time() / GENERATION);
    with_pools(|p| evolved(p, index, cell, h, day, &mut coats::world_rng(w)))
}

/// `/animal evolve [count] [habitat]` (for testing): `count` more generations of every species'
/// pool where `player` stands (in `habitat` instead of the one there, if given); a line per
/// species, saying how common each variant is now.
pub fn evolve_here(w: &World, player: u64, count: u32, habitat: Option<Habitat>) -> Vec<String> {
    let Some(p) = w.entity(player) else { return Vec::new() };
    let (cell, h, day) = (cell_of(p.pos), habitat.unwrap_or_else(|| habitat_at(w, p.pos)), w.time() / GENERATION);
    let mut r = coats::world_rng(w);
    with_pools(|pools| {
        (0..SPECIES.len())
            .filter_map(|i| {
                evolved(pools, i, cell, h, day, &mut r)?;
                let k = key(i, cell, h);
                for _ in 0..count {
                    step(pools, &k, i, cell, h, &mut r);
                }
                if let Some(l) = pools.all.get_mut(&k) {
                    l.day = day;
                }
                let pool = &pools.all.get(&k)?.pool;
                Some(format!("{} ({}): {}", SPECIES[i].kind_name(), h.name(), coats::describe_pool(i, pool)))
            })
            .collect()
    })
}

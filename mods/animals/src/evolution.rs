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
//! animals, so a pool far from everyone costs nothing. They are saved in the world's animals
//! folder (through the engine's bridge) in shards of 8 x 8 places (`pools/<x>_<z>.json`), read
//! when first needed (64 at most open), written when changed on a thread of their own, and a
//! file that fails to read is kept aside, never written over (`store.rs`). A pool nobody needed
//! for 400 days is dropped, and starts afresh with its history when next needed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, mpsc};

use serde::{Deserialize, Serialize};
use weft_sdk::*;

pub use animals_core::evolution::*;

use crate::coats::{self, Pool};
use crate::species::SPECIES;
use crate::store;

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

/// The pools of `SHARD` x `SHARD` cells, a file of their own (`pools/<x>_<z>.json` in the world's
/// animals folder, about 280 KB once explored), by `key`.
type ShardPools = HashMap<String, Local>;

/// Cells a shard is wide, shards open at once (the least used is put away past this), and the
/// generations an untouched pool is kept (a place nobody went to in 400 days starts afresh from
/// its gene file, with its history).
const SHARD: i64 = 8;
const OPEN: usize = 64;
const PRUNE: u64 = 400;

struct Shard {
    all: ShardPools,
    dirty: bool,
    /// False when its file was there but couldn't be read nor moved aside: it is never written
    /// over (`store::read_json`).
    writable: bool,
    /// When it was last used, for putting the least used away.
    used: u64,
}

/// The open world's pools, by shard, and the folder they are saved in.
#[derive(Default)]
struct Pools {
    dir: String,
    shards: HashMap<(i64, i64), Shard>,
    uses: u64,
    /// The latest generation asked for (pools older by `PRUNE` are dropped as shards load).
    today: u64,
}

static POOLS: Mutex<Option<Pools>> = Mutex::new(None);

/// A pool's key: species, place and habitat (`raptor 3 -2 desert`).
fn key(index: usize, cell: (i64, i64), h: Habitat) -> String {
    format!("{} {} {} {}", SPECIES[index].name, cell.0, cell.1, h.name())
}

fn cell_of(pos: [f64; 3]) -> (i64, i64) {
    ((pos[0] / CELL).floor() as i64, (pos[2] / CELL).floor() as i64)
}

fn shard_of(cell: (i64, i64)) -> (i64, i64) {
    (cell.0.div_euclid(SHARD), cell.1.div_euclid(SHARD))
}

fn shard_path(dir: &str, s: (i64, i64)) -> PathBuf {
    Path::new(dir)
        .join("pools")
        .join(format!("{}_{}.json", s.0, s.1))
}

impl Pools {
    /// Shard `s`, read from its file the first time (or from a write still on its way there).
    fn shard(&mut self, s: (i64, i64)) -> &mut Shard {
        self.uses += 1;
        let used = self.uses;
        if !self.shards.contains_key(&s) {
            if self.shards.len() >= OPEN {
                self.put_away();
            }
            let path = shard_path(&self.dir, s);
            let (mut all, writable) = match pending(&path).map(|a| (*a).clone()) {
                Some(all) => (all, true),
                None => match store::read_json::<ShardPools>(&path) {
                    store::Read::Fine(all) => (all, true),
                    store::Read::Missing => (ShardPools::new(), true),
                    store::Read::Bad { moved } => (ShardPools::new(), moved),
                },
            };
            let before = all.len();
            let today = self.today;
            all.retain(|_, l| l.day + PRUNE >= today);
            let dirty = all.len() != before;
            self.shards.insert(s, Shard { all, dirty, writable, used });
        }
        let shard = self.shards.get_mut(&s).unwrap();
        shard.used = used;
        shard
    }

    /// Puts the least used shard away, writing it first if it changed.
    fn put_away(&mut self) {
        let Some(&s) = self
            .shards
            .iter()
            .min_by_key(|(_, sh)| sh.used)
            .map(|(s, _)| s)
        else {
            return;
        };
        if let Some(sh) = self.shards.remove(&s)
            && sh.dirty
            && sh.writable
        {
            write_later(shard_path(&self.dir, s), sh.all);
        }
    }

    fn get(&mut self, k: &str, cell: (i64, i64)) -> Option<&Local> {
        self.shard(shard_of(cell)).all.get(k)
    }

    /// The pool at `k` to change (its shard is saved after).
    fn get_mut(&mut self, k: &str, cell: (i64, i64)) -> Option<&mut Local> {
        let sh = self.shard(shard_of(cell));
        let l = sh.all.get_mut(k)?;
        sh.dirty = true;
        Some(l)
    }

    fn insert(&mut self, k: String, cell: (i64, i64), l: Local) {
        let sh = self.shard(shard_of(cell));
        sh.all.insert(k, l);
        sh.dirty = true;
    }

    /// Hands every changed shard to the writer.
    fn flush(&mut self) {
        let dir = self.dir.clone();
        for (&s, sh) in self.shards.iter_mut().filter(|(_, sh)| sh.dirty) {
            sh.dirty = false;
            if sh.writable && !dir.is_empty() {
                write_later(shard_path(&dir, s), sh.all.clone());
            }
        }
    }

    /// A world saved before the shards has its pools in one `pools.json`: spread into the shards
    /// and written, then the old file renamed (`pools.json.moved-<secs>`). One that fails to read
    /// is kept aside (`store::read_json`).
    fn move_old_file(&mut self) {
        let old = Path::new(&self.dir).join("pools.json");
        let store::Read::Fine(all) = store::read_json::<ShardPools>(&old) else { return };
        let n = all.len();
        for (k, l) in all {
            let mut words = k.split_whitespace().skip(1);
            let cell = (words.next().and_then(|v| v.parse().ok()), words.next().and_then(|v| v.parse().ok()));
            let (Some(x), Some(z)) = cell else { continue };
            self.insert(k, (x, z), l);
        }
        self.flush();
        wait_for_writes();
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let moved = old.with_file_name(format!("pools.json.moved-{secs}"));
        match std::fs::rename(&old, &moved) {
            Ok(()) => {
                weft_sdk::info!("animals: {n} gene pools moved from pools.json into pools/ (the old file kept as {})", moved.display())
            }
            Err(e) => weft_sdk::warn!("animals: {n} gene pools moved into pools/, but pools.json stays ({e})"),
        }
    }
}

/// Runs `f` on the open world's pools (opened afresh when another world is: the last one's
/// changes are written first).
fn with_pools<R>(f: impl FnOnce(&mut Pools) -> R) -> R {
    let mut guard = POOLS.lock().unwrap_or_else(|e| e.into_inner());
    let dir = crate::data_dir();
    if guard.as_ref().is_none_or(|p| p.dir != dir) {
        if let Some(old) = guard.as_mut() {
            old.flush();
        }
        let mut p = Pools { dir, ..Pools::default() };
        if !p.dir.is_empty() {
            p.move_old_file();
        }
        *guard = Some(p);
    }
    f(guard.as_mut().unwrap())
}

// ------------------------------------------------------------------ writing, off the tick

/// Shards on their way to their files, the latest of each; the writer thread takes them from
/// here, and a shard opened meanwhile is read from here, not from its older file.
static PENDING: Mutex<Option<HashMap<PathBuf, Arc<ShardPools>>>> = Mutex::new(None);
static WRITER: Mutex<Option<mpsc::Sender<PathBuf>>> = Mutex::new(None);
/// Writes handed over and not yet done.
static WRITING: (Mutex<u32>, Condvar) = (Mutex::new(0), Condvar::new());

/// The shard on its way to `path`, if one is.
fn pending(path: &Path) -> Option<Arc<ShardPools>> {
    PENDING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()?
        .get(path)
        .cloned()
}

/// Writes `all` into `path` on the writer thread: serialised there, atomically
/// (`store::write_atomic`), so the world's tick never waits on it.
fn write_later(path: PathBuf, all: ShardPools) {
    PENDING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashMap::new)
        .insert(path.clone(), Arc::new(all));
    *WRITING.0.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    let mut writer = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let tx = writer.get_or_insert_with(|| {
        let (tx, rx) = mpsc::channel::<PathBuf>();
        std::thread::Builder::new()
            .name("animals pools".into())
            .spawn(move || {
                for path in rx {
                    write_now(&path);
                }
            })
            .expect("the animals' pool writer starts");
        tx
    });
    if tx.send(path.clone()).is_err() {
        write_now(&path);
    }
}

fn write_now(path: &Path) {
    if let Some(all) = pending(path) {
        match serde_json::to_string(&*all) {
            Ok(text) => {
                if let Err(e) = store::write_atomic(path, text.as_bytes()) {
                    weft_sdk::warn!("saving animals pools {}: {e}", path.display());
                }
            }
            Err(e) => weft_sdk::warn!("saving animals pools {}: {e}", path.display()),
        }
        // Done with it, unless a newer one came meanwhile (its own write follows).
        let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = pending.as_mut()
            && m.get(path).is_some_and(|a| Arc::ptr_eq(a, &all))
        {
            m.remove(path);
        }
    }
    let (n, done) = &WRITING;
    let mut n = n.lock().unwrap_or_else(|e| e.into_inner());
    *n = n.saturating_sub(1);
    done.notify_all();
}

/// Blocks until every write handed over is done.
fn wait_for_writes() {
    let (n, done) = &WRITING;
    let mut n = n.lock().unwrap_or_else(|e| e.into_inner());
    while *n > 0 {
        n = done.wait(n).unwrap_or_else(|e| e.into_inner());
    }
}

/// Hands the changed shards to the writer (`init`: every thirty seconds).
fn save() {
    let mut guard = POOLS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(p) = guard.as_mut() {
        p.flush();
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
    p.today = p.today.max(day);
    let new = p.get(&k, cell).is_none();
    if new {
        // As if it had evolved since the world began (up to `HISTORY` generations): the species'
        // history in this habitat (`history`, once for every place found that day), then its
        // last `OWN` generations here, its neighbors mixing in.
        let gens = day.min(HISTORY);
        let own = gens.min(OWN);
        let pool = history(index, h, gens - own, r)?;
        p.insert(k.clone(), cell, Local { day: day - own, pool });
    }
    let from = p.get(&k, cell)?.day;
    for _ in 0..day.saturating_sub(from).min(CATCH_UP) {
        step(p, &k, index, cell, h, r);
    }
    let local = p.get_mut(&k, cell)?;
    local.day = local.day.max(day);
    Some(local.pool.clone())
}

/// Generations a new place evolves itself (with its neighbors) after its habitat's history.
const OWN: u64 = 10;

/// Species `index`'s pool in habitat `h` after `gens` generations from its gene file, alone: the
/// start of every new place of that habitat. Kept, so new places share it and the next day's
/// takes it up a generation on: 300 generations of 200 animals each was a hitch on the game thread for every
/// species in every new place (P4).
fn history(index: usize, h: Habitat, gens: u64, r: &mut impl coats::Rng) -> Option<Pool> {
    static KEPT: Mutex<Option<HashMap<(usize, &'static str), (u64, Pool)>>> = Mutex::new(None);
    let key = (index, h.name());
    // Taken up where it was kept (a young world's history grows a generation a day).
    let kept = KEPT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|m| m.get(&key))
        .filter(|(g, _)| *g <= gens)
        .cloned();
    let (from, mut pool) = match kept {
        Some((g, pool)) if g == gens => return Some(pool),
        Some(k) => k,
        None => (0, coats::wild_pool(index, h.name())?),
    };
    for _ in from..gens {
        pool = generation(index, &pool, h, r)?;
    }
    KEPT.lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashMap::new)
        .insert(key, (gens, pool.clone()));
    Some(pool)
}

/// One generation of the pool at `k`, with the neighbors' pools (the places around, any habitat)
/// mixing in.
fn step(p: &mut Pools, k: &str, index: usize, cell: (i64, i64), h: Habitat, r: &mut impl coats::Rng) {
    let Some(mut next) = p
        .get(k, cell)
        .and_then(|l| generation(index, &l.pool, h, r))
    else {
        return;
    };
    let neighbors: Vec<Pool> = (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dz| (cell.0 + dx, cell.1 + dz)))
        .flat_map(|c| HABITATS.map(|h2| (key(index, c, h2), c)))
        .filter(|(n, _)| n != k)
        .filter_map(|(n, c)| p.get(&n, c).map(|l| l.pool.clone()))
        .collect();
    if !neighbors.is_empty() {
        let share = MIGRATION / neighbors.len() as f32;
        for n in &neighbors {
            mix_in(&mut next, n, share);
        }
    }
    if let Some(l) = p.get_mut(k, cell) {
        l.pool = next;
        l.day += 1;
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
        let l = p.get_mut(&key(index, cell, h), cell)?;
        coats::count_in(index, &mut l.pool, &g, sign, POPULATION as f32);
        weft_sdk::debug!("animal {id} {} the wild pool {}", if sign < 0.0 { "left" } else { "joined" }, key(index, cell, h));
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
                if let Some(l) = pools.get_mut(&k, cell) {
                    l.day = day;
                }
                let pool = &pools.get(&k, cell)?.pool;
                Some(format!("{} ({}): {}", SPECIES[i].kind_name(), h.name(), coats::describe_pool(i, pool)))
            })
            .collect()
    })
}

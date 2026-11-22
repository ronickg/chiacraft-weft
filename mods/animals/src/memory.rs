//! What animals remember (`reports/Creature AI.md`, part 3, step 5; the numbers are
//! `animals_core::memory`):
//!
//! - **Each animal** remembers the last few players who hurt or fed it (`MEMORIES`, saved with it):
//!   its senses notice one who hurt it sooner and one who fed it later (`senses::players`).
//! - **Each place** has its danger: deaths and hurts leave fear on their square (`fear_at`), fading
//!   over days. Grazers keep off a feared square for food and water unless need drives them
//!   (`forage::find_food`, `drink::find_water`); a hungry hunter that sees no prey goes where kills
//!   were (`haunt`, from `hunt::hunt`).
//! - **Each species in each area** has a reputation with each player (`reputation`): a kill or a
//!   hurt lowers it, feeding raises it; its animals' wariness of that player follows it.
//!
//! Places and reputations are kept for the world in `memory.json` in the mod's folder (written now
//! and then, `SAVE_EVERY`), and fade as they are read.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use weft_sdk::*;

use animals_core::memory as m;

use crate::species::SPECIES;

value! {
    #[derive(Default)]
    /// The players an animal remembers: who (`player_id`, 0: none), how (-1 hurt it, +1 fed it)
    /// and when (game time). Saved with it.
    pub struct Memories {
        pub who: [u64; m::MEMORIES],
        pub feel: [i8; m::MEMORIES],
        pub at: [u64; m::MEMORIES],
    }
}

pub static MEMORIES: Component<Memories> = Component::named("animal_memories");

/// A place's fear, or a reputation, and when it was last changed (game time).
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
struct Kept {
    v: f32,
    at: u64,
}

/// The world's places and reputations (`memory.json`).
#[derive(Serialize, Deserialize, Default)]
struct Saved {
    /// (dimension, square x, square z) and its fear.
    danger: Vec<((u32, i32, i32), Kept)>,
    /// (dimension, species, area x, area z, player) and its reputation.
    reputation: Vec<((u32, u8, i32, i32, u64), Kept)>,
}

#[derive(Default)]
struct World_ {
    dir: String,
    /// The game time the folder was last asked for (once a tick is enough).
    checked: u64,
    danger: HashMap<(u32, i32, i32), Kept>,
    reputation: HashMap<(u32, u8, i32, i32, u64), Kept>,
    dirty: bool,
}

static WORLD: Mutex<Option<World_>> = Mutex::new(None);
/// Ticks between saves, and what is dropped as faded (a square's fear, a reputation's size).
const SAVE_EVERY: u64 = 1200;
const GONE: f32 = 0.02;

fn path(dir: &str) -> std::path::PathBuf {
    Path::new(dir).join("memory.json")
}

/// Runs `f` on the open world's places (read from its file the first time; another world's are
/// saved first).
fn with<R>(w: &World, f: impl FnOnce(&mut World_) -> R) -> R {
    let mut guard = WORLD.lock().unwrap_or_else(|e| e.into_inner());
    let now = w.game_time();
    if let Some(open) = guard
        .as_mut()
        .filter(|o| o.checked == now && !o.dir.is_empty())
    {
        return f(open);
    }
    let dir = crate::data_dir();
    if guard.as_ref().is_none_or(|w| w.dir != dir) {
        if let Some(old) = guard.as_mut() {
            save(old);
        }
        let mut open = World_ { dir: dir.clone(), ..Default::default() };
        if !dir.is_empty()
            && let crate::store::Read::Fine(s) = crate::store::read_json::<Saved>(&path(&dir))
        {
            open.danger = s.danger.into_iter().collect();
            open.reputation = s.reputation.into_iter().collect();
        }
        *guard = Some(open);
    }
    let open = guard.as_mut().unwrap();
    open.checked = now;
    f(open)
}

fn save(w: &mut World_) {
    if !w.dirty || w.dir.is_empty() {
        return;
    }
    w.dirty = false;
    let s = Saved {
        danger: w.danger.iter().map(|(k, v)| (*k, *v)).collect(),
        reputation: w.reputation.iter().map(|(k, v)| (*k, *v)).collect(),
    };
    match serde_json::to_string(&s) {
        Ok(text) => {
            if let Err(e) = crate::store::write_atomic(&path(&w.dir), text.as_bytes()) {
                weft_sdk::warn!("saving animals' memory: {e}");
            }
        }
        Err(e) => weft_sdk::warn!("saving animals' memory: {e}"),
    }
}

fn days(w: &World, since: u64) -> f32 {
    w.game_time().saturating_sub(since) as f32 / animals_core::evolution::GENERATION as f32
}

fn square(pos: [f64; 3]) -> (i32, i32) {
    let c = m::DANGER_CELL as f64;
    ((pos[0] / c).floor() as i32, (pos[2] / c).floor() as i32)
}

fn area(pos: [f64; 3]) -> (i32, i32) {
    let c = m::REPUTATION_AREA as f64;
    ((pos[0] / c).floor() as i32, (pos[2] / c).floor() as i32)
}

/// Adds `v` to what is kept at `k`, faded first.
fn add<K: std::hash::Hash + Eq>(map: &mut HashMap<K, Kept>, k: K, v: f32, now: u64, over: f32) {
    let e = map.entry(k).or_default();
    let d = now.saturating_sub(e.at) as f32 / animals_core::evolution::GENERATION as f32;
    *e = Kept { v: m::faded(e.v, d, over) + v, at: now };
}

/// How feared the square at `pos` is now.
pub fn fear_at(w: &World, pos: [f64; 3]) -> f32 {
    let (x, z) = square(pos);
    let k = (w.dimension(), x, z);
    with(w, |s| s.danger.get(&k).copied()).map_or(0.0, |e| m::faded(e.v, days(w, e.at), m::DANGER_DAYS))
}

/// Fear on the square at `pos`.
pub fn frighten(w: &World, pos: [f64; 3], fear: f32) {
    let (x, z) = square(pos);
    let (k, now) = ((w.dimension(), x, z), w.game_time());
    with(w, |s| {
        add(&mut s.danger, k, fear, now, m::DANGER_DAYS);
        s.dirty = true;
    });
}

/// The most feared square within `HAUNT_LOOK` blocks of `pos` with at least `HAUNT` fear (a kill
/// site): its middle.
pub fn haunt(w: &World, pos: [f64; 3]) -> Option<[f64; 3]> {
    let (x, z) = square(pos);
    let r = m::HAUNT_LOOK / m::DANGER_CELL;
    let dim = w.dimension();
    let now = w.game_time();
    let best = with(w, |s| {
        let mut best: Option<((i32, i32), f32)> = None;
        for dx in -r..=r {
            for dz in -r..=r {
                let Some(e) = s.danger.get(&(dim, x + dx, z + dz)) else { continue };
                let v = m::faded(e.v, now.saturating_sub(e.at) as f32 / animals_core::evolution::GENERATION as f32, m::DANGER_DAYS);
                if v >= m::HAUNT && best.is_none_or(|b| v > b.1) {
                    best = Some(((x + dx, z + dz), v));
                }
            }
        }
        best
    })?;
    let c = m::DANGER_CELL as f64;
    Some([(best.0.0 as f64 + 0.5) * c, pos[1], (best.0.1 as f64 + 0.5) * c])
}

/// A player's lasting id: the hash of their key (the same after a restart; their entity id isn't).
pub fn player_id(player: u64) -> u64 {
    static IDS: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);
    let mut ids = IDS.lock().unwrap_or_else(|e| e.into_inner());
    let ids = ids.get_or_insert_with(HashMap::new);
    if let Some(&h) = ids.get(&player) {
        return h;
    }
    let key = crate::player_key(player);
    if key.is_empty() {
        return player;
    }
    // FNV-1a: stable across runs (the std hasher is seeded).
    let h = key
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
        .max(1);
    ids.insert(player, h);
    h
}

/// Species `index`'s reputation in the area of `pos` with `player` (negative: feared).
pub fn reputation(w: &World, index: usize, pos: [f64; 3], player: u64) -> f32 {
    let (x, z) = area(pos);
    let k = (w.dimension(), index as u8, x, z, player_id(player));
    with(w, |s| s.reputation.get(&k).copied()).map_or(0.0, |e| m::faded(e.v, days(w, e.at), m::REPUTATION_DAYS))
}

fn change_reputation(w: &World, index: usize, pos: [f64; 3], player: u64, v: f32) {
    let (x, z) = area(pos);
    let (k, now) = ((w.dimension(), index as u8, x, z, player_id(player)), w.game_time());
    with(w, |s| {
        add(&mut s.reputation, k, v, now, m::REPUTATION_DAYS);
        s.dirty = true;
    });
}

/// Animal `id` remembers `player` (`feel`: -1 hurt it, +1 fed it), over its oldest memory.
pub fn remember(w: &World, id: u64, player: u64, feel: i8) {
    let who = player_id(player);
    let now = w.game_time();
    let Some(mem) = w.component_mut(id, &MEMORIES) else { return };
    let i = mem
        .who
        .iter()
        .position(|&p| p == who)
        .unwrap_or_else(|| (0..m::MEMORIES).min_by_key(|&i| mem.at[i]).unwrap_or(0));
    // A hurt outweighs a feeding.
    let hurt_before = mem.who[i] == who && mem.feel[i] < 0;
    mem.who[i] = who;
    mem.feel[i] = if hurt_before && feel > 0 { -1 } else { feel };
    mem.at[i] = now;
}

/// How animal `id`'s memories and its kind's reputation change how fast it notices `player`
/// (times), and its shy distance of him (times).
pub fn wary_of(w: &World, id: u64, index: usize, pos: [f64; 3], player: u64) -> (f32, f32) {
    let who = player_id(player);
    let rep = m::wariness(reputation(w, index, pos, player));
    let mine = w.component(id, &MEMORIES).and_then(|mem| {
        (0..m::MEMORIES)
            .find(|&i| mem.who[i] == who && days(w, mem.at[i]) < m::REMEMBERS_DAYS)
            .map(|i| if mem.feel[i] < 0 { m::HURT_ME } else { m::FED_ME })
    });
    (rep * mine.unwrap_or(1.0), rep)
}

/// A player fed animal `id` (`taming::feed`): it remembers, and its kind there thinks better of him.
pub fn fed(w: &World, id: u64, player: u64) {
    remember(w, id, player, 1);
    if let (Some(st), Some(index)) = (w.entity(id), species_of(w, id)) {
        change_reputation(w, index, st.pos.to_array(), player, m::FED);
    }
}

fn species_of(w: &World, id: u64) -> Option<usize> {
    w.component(id, &crate::animal::ANIMAL)
        .and_then(|a| (a.species as usize).checked_sub(1))
        .filter(|&i| i < SPECIES.len())
}

pub fn init(r: &mut Registry) {
    r.entity_component(&MEMORIES, Memories::default(), component_flags::SAVE);
    for s in &SPECIES {
        r.attach(s.kind, &MEMORIES);
    }
    // A hurt: fear where it happened; a player who hurt an animal is remembered, and its kind
    // there thinks worse of him.
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            let Some(e) = w.entity(h.entity) else { continue };
            let Some(index) = species_of(&w, h.entity) else { continue };
            frighten(&w, e.pos.to_array(), m::HURT_FEAR);
            if h.source != 0 && w.players_iter().any(|p| p == h.source) {
                remember(&w, h.entity, h.source, -1);
                change_reputation(&w, index, e.pos.to_array(), h.source, m::HURT);
            }
        }
    });
    // A death: fear where it fell; a player's kill, its kind there fears him.
    r.on(&actions::DIE, |b| {
        let w = b.world();
        for d in b.iter() {
            let Some(index) = SPECIES.iter().position(|s| s.kind.id() == d.kind) else { continue };
            let pos = d.pos.to_array();
            frighten(&w, pos, m::DEATH_FEAR);
            if d.source != 0 && w.players_iter().any(|p| p == d.source) {
                change_reputation(&w, index, pos, d.source, m::KILLED);
                weft_sdk::info!("{} {} killed by player {}: its kind here fears him more", SPECIES[index].name, d.entity, d.source);
            }
        }
    });
    r.world_tick(|w| {
        if w.game_time().is_multiple_of(SAVE_EVERY) {
            let now = w.game_time();
            with(w, |s| {
                // Faded ones go.
                let over = |e: &Kept, days: f32| {
                    m::faded(e.v, now.saturating_sub(e.at) as f32 / animals_core::evolution::GENERATION as f32, days).abs() > GONE
                };
                let before = s.danger.len() + s.reputation.len();
                s.danger.retain(|_, e| over(e, m::DANGER_DAYS));
                s.reputation.retain(|_, e| over(e, m::REPUTATION_DAYS));
                s.dirty |= before != s.danger.len() + s.reputation.len();
                save(s);
            });
        }
    });
}

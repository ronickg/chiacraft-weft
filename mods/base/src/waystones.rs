//! Waystones and the map, the server's side (`vitals::waystones`; `notes/PROGRESSION.md`: "a map
//! that fills where you walk, with waystones (ancient ley nodes you wake)").
//!
//! - **Each player's atlas** (`Atlas`): the map tiles they walked or saw (a bit per 16 x 16 block
//!   tile, in regions of 32 x 32 tiles) and the waystones they woke, kept in the world's base folder
//!   under `maps/<key>.bin`, `key` a number saved on the player (`MAP_KEY`). Its own file per player,
//!   so a tribe can later share by merging files. Sent to the player's client when it asks
//!   (`ASK`) and after a waking.
//! - **Waking:** the dowsing rod held on use for `WAKE_TICKS` near a sleeping waystone, with the
//!   offering in the inventory: a ley flare and a chime, then `ley::heart_opened` (the perk web's
//!   stone may drop).
//! - **Travel:** using a waystone one woke opens the map to pick another (`OPEN`); the pick
//!   (`TRAVEL`) starts a channel of `CHANNEL_SECS` that moving, breaking a block or being hurt
//!   cancels, then the player is there and the stomach pays `travel_secs`. One player at a time:
//!   no passengers, no tames.
//!
//! `/waystone` lists the player's woken waystones and map size; `/waystone wake` (game masters)
//! wakes the nearest without the rod or offering, and `/waystone goto` (game masters) stands the
//! player 4 blocks south of the nearest waystone they haven't woken, for runs on any seed.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use vitals::ley::{self, Node};
use vitals::waystones as n;
use weft_sdk::*;

use crate::ley::{DOWSING_ROD, WAYSTONE};

value! {
    /// A waystone by its ley cell.
    pub struct Cell { pub i: i32, pub j: i32 }
}

/// The client asks for its atlas.
pub static ASK: Channel<()> = Channel::named("base:map_ask");
/// The atlas, encoded (`Atlas::encode`).
pub static ATLAS: Channel<Vec<u8>> = Channel::named("base:map_atlas");
/// Opens the map to travel from this waystone.
pub static OPEN: Channel<Cell> = Channel::named("base:map_travel_open");
/// The player picked a waystone to travel to.
pub static TRAVEL: Channel<Cell> = Channel::named("base:map_travel");

/// The player's atlas file's name, a random number kept on the player.
static MAP_KEY: Component<[u32; 2]> = Component::named("map_key");
static WAKE: Sound = Sound::named("ley.wake");
pub(crate) static FLARE: Particle = Particle::named("ley_flare");

// ------------------------------------------------------------------ the atlas

/// What a player knows of the world: the tiles seen, by region (a bit a tile, row by row), and
/// the waystones woken.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Atlas {
    pub regions: BTreeMap<(i32, i32), [u64; 16]>,
    pub woken: Vec<(i32, i32)>,
}

const MAGIC: &[u8; 4] = b"WMP1";

impl Atlas {
    /// Region and bit of tile (`tx`, `tz`).
    fn place(tx: i32, tz: i32) -> ((i32, i32), usize) {
        let (rx, rz) = (tx.div_euclid(n::REGION), tz.div_euclid(n::REGION));
        let (lx, lz) = (tx.rem_euclid(n::REGION), tz.rem_euclid(n::REGION));
        ((rx, rz), (lz * n::REGION + lx) as usize)
    }

    pub fn seen(&self, tx: i32, tz: i32) -> bool {
        let (r, b) = Atlas::place(tx, tz);
        self.regions
            .get(&r)
            .is_some_and(|bits| bits[b / 64] >> (b % 64) & 1 != 0)
    }

    /// Marks a tile seen; true if it is new.
    pub fn see(&mut self, tx: i32, tz: i32) -> bool {
        let (r, b) = Atlas::place(tx, tz);
        let bits = self.regions.entry(r).or_insert([0; 16]);
        let had = bits[b / 64] >> (b % 64) & 1 != 0;
        bits[b / 64] |= 1 << (b % 64);
        !had
    }

    pub fn tiles(&self) -> usize {
        self.regions
            .values()
            .map(|b| b.iter().map(|w| w.count_ones() as usize).sum::<usize>())
            .sum()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + self.regions.len() * 136 + self.woken.len() * 8);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.regions.len() as u32).to_le_bytes());
        for (&(rx, rz), bits) in &self.regions {
            out.extend_from_slice(&rx.to_le_bytes());
            out.extend_from_slice(&rz.to_le_bytes());
            for w in bits {
                out.extend_from_slice(&w.to_le_bytes());
            }
        }
        out.extend_from_slice(&(self.woken.len() as u32).to_le_bytes());
        for &(i, j) in &self.woken {
            out.extend_from_slice(&i.to_le_bytes());
            out.extend_from_slice(&j.to_le_bytes());
        }
        out
    }

    pub fn decode(b: &[u8]) -> Option<Atlas> {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            let s = b.get(at..at + n)?;
            at += n;
            Some(s)
        };
        if take(4)? != MAGIC {
            return None;
        }
        let u32_of = |s: &[u8]| u32::from_le_bytes(s.try_into().unwrap());
        let i32_of = |s: &[u8]| i32::from_le_bytes(s.try_into().unwrap());
        let mut a = Atlas::default();
        let regions = u32_of(take(4)?);
        for _ in 0..regions {
            let (rx, rz) = (i32_of(take(4)?), i32_of(take(4)?));
            let mut bits = [0u64; 16];
            for w in &mut bits {
                *w = u64::from_le_bytes(take(8)?.try_into().unwrap());
            }
            a.regions.insert((rx, rz), bits);
        }
        let woken = u32_of(take(4)?);
        for _ in 0..woken {
            a.woken.push((i32_of(take(4)?), i32_of(take(4)?)));
        }
        Some(a)
    }
}

/// A player's atlas as the server holds it.
struct Held {
    key: [u32; 2],
    atlas: Atlas,
    dirty: bool,
}

/// The open world's atlases by player (entity), and its folder.
struct Atlases {
    dir: String,
    by_player: HashMap<u64, Held>,
}

static ATLASES: Mutex<Option<Atlases>> = Mutex::new(None);

fn file(dir: &str, key: [u32; 2]) -> PathBuf {
    PathBuf::from(dir)
        .join("maps")
        .join(format!("{:08x}{:08x}.bin", key[0], key[1]))
}

fn save(dir: &str, h: &mut Held) {
    if !h.dirty || dir.is_empty() {
        return;
    }
    h.dirty = false;
    let path = file(dir, h.key);
    let bytes = h.atlas.encode();
    let tmp = path.with_extension("tmp");
    let done = std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| std::fs::write(&tmp, &bytes))
        .and_then(|_| std::fs::rename(&tmp, &path));
    if let Err(e) = done {
        weft_sdk::warn!("map: couldn't save {}: {e}", path.display());
    }
}

/// Runs `f` on `player`'s atlas (read from its file the first time; another world's are saved
/// and dropped first).
fn with_atlas<R>(w: &World, player: u64, f: impl FnOnce(&mut Held) -> R) -> R {
    let mut guard = ATLASES.lock().unwrap_or_else(|e| e.into_inner());
    let dir = crate::bridge("data_dir", "base");
    if guard.as_ref().is_none_or(|a| a.dir != dir) {
        if let Some(old) = guard.as_mut() {
            for h in old.by_player.values_mut() {
                save(&old.dir, h);
            }
        }
        *guard = Some(Atlases { dir: dir.clone(), by_player: HashMap::new() });
    }
    let all = guard.as_mut().unwrap();
    let key = key_of(w, player);
    let held = all.by_player.entry(player).or_insert_with(|| {
        let atlas = if dir.is_empty() {
            Atlas::default()
        } else {
            std::fs::read(file(&dir, key))
                .ok()
                .and_then(|b| Atlas::decode(&b))
                .unwrap_or_default()
        };
        Held { key, atlas, dirty: false }
    });
    f(held)
}

/// The player's atlas key, made the first time.
fn key_of(w: &World, player: u64) -> [u32; 2] {
    let Some(k) = w.component_mut(player, &MAP_KEY) else { return [0, 0] };
    if *k == [0, 0] {
        let r = w.random();
        *k = [r as u32 | 1, (r >> 32) as u32];
    }
    *k
}

fn send_atlas(w: &World, player: u64) {
    let bytes = with_atlas(w, player, |h| h.atlas.encode());
    w.send(player, &ATLAS, &bytes);
}

// ------------------------------------------------------------------ registering

pub fn register(r: &mut Registry) {
    r.entity_component(&MAP_KEY, [0, 0], component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.attach(&PLAYER, &MAP_KEY);
    r.sound(&WAKE)
        .files(&["random/ley_wake"])
        .category(&sound_categories::BLOCKS)
        .range(48.0);
    r.particle(&FLARE)
        .textures(&["core:glint_0", "core:glint_1", "core:glint_2", "core:glint_3"])
        .life(1.2, 2.4)
        .size(0.2, 0.4)
        .gravity(-0.02)
        .drag(0.94)
        .color(0xE8FDFF, 0x7C8CFF)
        .fullbright()
        .translucent()
        .shrinking();
    r.packet(&ASK, |p, ()| send_atlas(&p.world(), p.player()));
    r.packet(&TRAVEL, |p, to| start_travel(&p.world(), p.player(), (to.i, to.j)));
    r.world_tick(tick);
    r.on(&actions::HURT, |b| {
        for e in b.iter() {
            cancel(&b.world(), e.entity, "hint.base.waystone.cancel_hurt");
        }
    });
    r.on(&actions::BREAK_BLOCK, |b| {
        for e in b.iter() {
            cancel(&b.world(), e.player, "hint.base.waystone.cancel_moved");
        }
    });
    r.command("waystone", "[what:word]", |cmd| {
        let w = cmd.world();
        let p = cmd.player();
        if cmd.text("what") == Some("goto") {
            if w.permission(p) < permission::GAME_MASTER {
                return;
            }
            let Some(st) = w.entity(p) else { return };
            let woken = with_atlas(&w, p, |h| h.atlas.woken.clone());
            let (x, z) = (st.pos.x, st.pos.z);
            let Some(nd) = ley::waystones_near(w.seed(), x as i32, z as i32, 6 * ley::CELL)
                .into_iter()
                .filter(|nd| !woken.contains(&nd.cell))
                .min_by_key(|nd| ((nd.x as f64 - x).powi(2) + (nd.z as f64 - z).powi(2)) as i64)
            else {
                cmd.reply("command.base.waystone.none_near", &[]);
                return;
            };
            let f = base_api::land::fields(w.seed());
            let (gx, gz) = (nd.x, nd.z + n::ARRIVE_SOUTH);
            let y = (base_api::land::height(&f, gx * 16, gz * 16) >> 4).max(base_api::SEA) + 2;
            w.entity_edit(p)
                .pos([gx as f64 + 0.5, y as f64, gz as f64 + 0.5])
                .velocity([0.0, 0.0, 0.0])
                .fall_distance(0.0);
            cmd.reply("command.base.waystone.goto", &[Arg::Int(nd.x as i64), Arg::Int(nd.z as i64), Arg::Int(nd.lines as i64)]);
            return;
        }
        if cmd.text("what") == Some("wake") {
            if w.permission(p) < permission::GAME_MASTER {
                return;
            }
            let Some((node, _)) = near_waystone(&w, p) else {
                cmd.reply("command.base.waystone.none_near", &[]);
                return;
            };
            wake(&w, p, &node);
            return;
        }
        let (tiles, woken, bytes) = with_atlas(&w, p, |h| (h.atlas.tiles(), h.atlas.woken.clone(), h.atlas.encode().len()));
        let list: Vec<String> = woken
            .iter()
            .filter_map(|&(i, j)| ley::node(w.seed(), i, j))
            .map(|n| format!("{} {}", n.x, n.z))
            .collect();
        cmd.reply(
            "command.base.waystone.list",
            &[Arg::Int(woken.len() as i64), Arg::Text(&list.join(", ")), Arg::Int(tiles as i64), Arg::Int(bytes as i64)],
        );
    });
}

/// The waystone `player` stands at (its node within `NEAR` across), with the distance.
fn near_waystone(w: &World, player: u64) -> Option<(Node, f64)> {
    if w.dimension() != crate::OVERWORLD.id() {
        return None;
    }
    let st = w.entity(player)?;
    let (x, z) = (st.pos.x, st.pos.z);
    ley::waystones_near(w.seed(), x as i32, z as i32, n::NEAR.ceil() as i32 + 1)
        .into_iter()
        .map(|nd| {
            let d = (nd.x as f64 + 0.5 - x).hypot(nd.z as f64 + 0.5 - z);
            (nd, d)
        })
        .filter(|&(_, d)| d <= n::NEAR)
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// The stone's top block at a node, near height `y` (the waystone block itself), if loaded.
fn stone_top(w: &World, node: &Node, y: i32) -> Option<i32> {
    (-24..=24)
        .map(|dy| y + dy)
        .find(|&yy| w.block_loaded([node.x, yy, node.z]) == Some(WAYSTONE.id()))
}

// ------------------------------------------------------------------ waking

/// The rod held on use for `WAKE_TICKS` (its `on_use`): wakes the waystone the player stands at,
/// taking the offering. True when it did something (woke, or said why not).
pub fn rod_used(u: &mut ItemUse) -> bool {
    let w = u.world();
    let p = u.player();
    let Some((node, _)) = near_waystone(&w, p) else { return false };
    if with_atlas(&w, p, |h| h.atlas.woken.contains(&node.cell)) {
        w.hint(p, "hint.base.waystone.already", &[]);
        return true;
    }
    if !take_offering(&w, p) {
        w.hint(p, "hint.base.waystone.offering", &[Arg::Int(n::OFFERING.1 as i64), Arg::Name(&format!("item:{}", n::OFFERING.0))]);
        return true;
    }
    wake(&w, p, &node);
    true
}

/// Takes the offering from anywhere in the inventory; false (and nothing taken) without enough.
fn take_offering(w: &World, player: u64) -> bool {
    let Some(item) = Item::lookup(n::OFFERING.0) else { return false };
    let inv = w.inventory(player);
    let slots: Vec<u32> = (0..inv.len())
        .filter(|&i| inv.get(i).item == item)
        .collect();
    let have: u32 = slots.iter().map(|&i| inv.get(i).count).sum();
    if have < n::OFFERING.1 {
        return false;
    }
    let mut left = n::OFFERING.1;
    for i in slots {
        let s = inv.get(i);
        let t = s.count.min(left);
        inv.set(i, s.shrink(t));
        left -= t;
        if left == 0 {
            break;
        }
    }
    true
}

/// Wakes `node`'s waystone for `player`: kept in the atlas, a flare and a chime at the stone, the
/// heart opened (the perk web's stone), the client told.
fn wake(w: &World, player: u64, node: &Node) {
    with_atlas(w, player, |h| {
        if !h.atlas.woken.contains(&node.cell) {
            h.atlas.woken.push(node.cell);
            h.dirty = true;
        }
    });
    let y = w.entity(player).map_or(80, |s| s.pos.y.floor() as i32);
    let top = stone_top(w, node, y).unwrap_or(y + 2);
    let at = [node.x as f64 + 0.5, top as f64 + 0.6, node.z as f64 + 0.5];
    drop(
        w.spawn_particles(&FLARE, at)
            .count(140)
            .speed(3.5)
            .outward(),
    );
    drop(
        w.spawn_particles(&FLARE, [at[0], at[1] - 2.0, at[2]])
            .count(60)
            .spread_box([0.4, 2.0, 0.4])
            .velocity([0.0, 1.5, 0.0]),
    );
    drop(w.play_sound(&WAKE, at, 1.0));
    w.hint(player, "hint.base.waystone.woke", &[]);
    if crate::ley::heart_opened(w, node, 0, top) {
        w.hint(player, "hint.base.waystone.stone", &[]);
    }
    weft_sdk::info!("waystone: player {player} woke {} {} ({} lines)", node.x, node.z, node.lines);
    send_atlas(w, player);
}

// ------------------------------------------------------------------ travel

/// Using a waystone or the ley stone of its structure (not with the rod: that wakes it), standing
/// at its node: a woken one opens the map to travel. Ley stone placed elsewhere does nothing.
pub fn stone_used(u: &mut Use) -> bool {
    let w = u.world();
    let p = u.player();
    let held = w.inventory(p).group("hotbar").get(w.selected_slot(p)).item;
    if DOWSING_ROD.id() != u32::MAX && held == DOWSING_ROD.id() {
        return false;
    }
    let Some((node, _)) = near_waystone(&w, p) else { return false };
    if !with_atlas(&w, p, |h| h.atlas.woken.contains(&node.cell)) {
        w.hint(p, "hint.base.waystone.sleeps", &[]);
        return true;
    }
    send_atlas(&w, p);
    w.send(p, &OPEN, &Cell { i: node.cell.0, j: node.cell.1 });
    true
}

/// A trip under way: where the player stood, where to, when it lands (game time), how far.
struct Trip {
    from: [f64; 3],
    to: Node,
    lands: u64,
    dist: f64,
}

static TRIPS: Mutex<Vec<(u64, Trip)>> = Mutex::new(Vec::new());

fn start_travel(w: &World, player: u64, to: (i32, i32)) {
    let Some((from, _)) = near_waystone(w, player) else {
        w.hint(player, "hint.base.waystone.not_at_one", &[]);
        return;
    };
    let woken = with_atlas(w, player, |h| h.atlas.woken.clone());
    if !woken.contains(&from.cell) {
        w.hint(player, "hint.base.waystone.sleeps", &[]);
        return;
    }
    let Some(dest) = ley::node(w.seed(), to.0, to.1).filter(|d| d.waystone() && woken.contains(&d.cell) && d.cell != from.cell) else {
        return;
    };
    let Some(st) = w.entity(player) else { return };
    let dist = ((dest.x - from.x) as f64).hypot((dest.z - from.z) as f64);
    let lands = w.game_time() + (n::CHANNEL_SECS * 20.0) as u64;
    let mut trips = TRIPS.lock().unwrap();
    trips.retain(|(p, _)| *p != player);
    trips.push((player, Trip { from: [st.pos.x, st.pos.y, st.pos.z], to: dest, lands, dist }));
    w.hint(player, "hint.base.waystone.channel", &[Arg::Int(n::CHANNEL_SECS as i64), Arg::Int(dist.round() as i64)]);
}

fn cancel(w: &World, player: u64, why: &str) {
    let mut trips = TRIPS.lock().unwrap();
    let before = trips.len();
    trips.retain(|(p, _)| *p != player);
    if trips.len() != before {
        w.hint(player, why, &[]);
    }
}

fn arrive(w: &World, player: u64, t: &Trip) {
    let f = base_api::land::fields(w.seed());
    let (x, z) = (t.to.x, t.to.z + n::ARRIVE_SOUTH);
    let ground = base_api::land::height(&f, x * 16, z * 16) >> 4;
    let y = ground.max(base_api::SEA) as f64 + 2.0;
    let (from, to) = (t.from, [x as f64 + 0.5, y, z as f64 + 0.5]);
    drop(
        w.spawn_particles(&FLARE, from)
            .count(80)
            .speed(2.5)
            .outward(),
    );
    drop(w.play_sound(&WAKE, from, 0.6).pitch(1.3));
    w.entity_edit(player)
        .pos(to)
        .velocity([0.0, 0.0, 0.0])
        .fall_distance(0.0);
    drop(w.spawn_particles(&FLARE, to).count(80).speed(2.5).outward());
    drop(w.play_sound(&WAKE, to, 0.6).pitch(1.3));
    // The food: what walking a share of the way would have digested.
    if let Some(v) = w.component_mut(player, &base_api::VITALS) {
        let mut s = v.stomach();
        s.digest(n::travel_secs(t.dist), 1.0);
        v.set_stomach(&s);
    }
    w.hint(player, "hint.base.waystone.arrived", &[Arg::Int(t.dist.round() as i64)]);
    weft_sdk::info!("waystone: player {player} travelled {:.0} blocks to {} {}", t.dist, t.to.x, t.to.z);
}

// ------------------------------------------------------------------ the tick

fn tick(w: &World) {
    let now = w.game_time();
    // Trips: drifting cancels, a landed one arrives; a shimmer round the traveller meanwhile.
    let due: Vec<(u64, Trip)> = {
        let mut trips = TRIPS.lock().unwrap();
        let mut keep = Vec::new();
        let mut due = Vec::new();
        for (p, t) in trips.drain(..) {
            let Some(st) = w.entity(p) else { continue };
            let drift = (st.pos.x - t.from[0]).hypot(st.pos.z - t.from[2]) + (st.pos.y - t.from[1]).abs();
            if drift > n::CHANNEL_DRIFT {
                w.hint(p, "hint.base.waystone.cancel_moved", &[]);
                continue;
            }
            if now >= t.lands {
                due.push((p, t));
            } else {
                if now.is_multiple_of(5) {
                    // A ring round the feet, rising, clear of the eyes (sparks at the chest filled
                    // the camera: PC look wm_look3).
                    let a = (now / 5) as f64 * 2.4;
                    for k in 0..3 {
                        let b = a + k as f64 * std::f64::consts::TAU / 3.0;
                        drop(
                            w.spawn_particles(&FLARE, [st.pos.x + b.cos() * 1.4, st.pos.y + 0.2, st.pos.z + b.sin() * 1.4])
                                .count(2)
                                .spread(0.15)
                                .velocity([0.0, 0.9, 0.0]),
                        );
                    }
                }
                keep.push((p, t));
            }
        }
        *trips = keep;
        due
    };
    for (p, t) in &due {
        arrive(w, *p, t);
    }
    // What each player in the overworld sees: the tiles round them.
    if now.is_multiple_of(n::SEEN_EVERY) && w.dimension() == crate::OVERWORLD.id() {
        for p in w.players() {
            let Some(st) = w.entity(p) else { continue };
            let (tx, tz) = ((st.pos.x.floor() as i32).div_euclid(n::TILE), (st.pos.z.floor() as i32).div_euclid(n::TILE));
            with_atlas(w, p, |h| {
                let r = n::SEEN_TILES;
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz <= r * r + r && h.atlas.see(tx + dx, tz + dz) {
                            h.dirty = true;
                        }
                    }
                }
            });
        }
    }
    if now.is_multiple_of(n::SAVE_EVERY) {
        let mut guard = ATLASES.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(all) = guard.as_mut() {
            let dir = all.dir.clone();
            for h in all.by_player.values_mut() {
                save(&dir, h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_round_trips_and_counts() {
        let mut a = Atlas::default();
        assert!(a.see(0, 0) && !a.see(0, 0));
        assert!(a.see(-1, -33) && a.see(31, 31) && a.see(32, 31));
        a.woken.push((3, -4));
        assert_eq!(a.tiles(), 4);
        assert!(a.seen(-1, -33) && !a.seen(-2, -33));
        let b = Atlas::decode(&a.encode()).unwrap();
        assert_eq!(a, b);
        assert!(Atlas::decode(b"nope").is_none());
    }
}

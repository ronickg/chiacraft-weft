//! The ancients' ruins and their blueprints: the knowledge gate (`vitals::ruins`; `notes/
//! PROGRESSION.md`, "How an age is gated").
//!
//! - **Ruins** stand on the ley lines, from the seed alone (`vitals::ruins::ruins_near`), built by a
//!   decoration like the waystones: a Stone Age ring open to the sky, a Bronze Age vault half
//!   buried under a roof of sealed stone (a copper pickaxe breaks it), an Iron Age vault deep under
//!   a shrine and a spiral stair, its shell of warded stone (a bronze pickaxe). One Stone ruin always
//!   stands within `vitals::ruins::SPAWN_REACH` blocks of the world's spawn.
//! - **The pedestal** in each gives every player the ruin's blueprint once (`TAKEN`, by the
//!   ruin's key), so a server never runs out.
//! - **A blueprint** read (used) teaches its recipes (`KNOWS`) and is kept. Until then the grid
//!   and the book won't craft what it teaches (`craft_locked`) and a furnace gives back the raw
//!   ore it teaches to smelt (`furnace_changed`). Creative knows them all.
//!
//! `/ruin` names the nearest ruin of each kind; `/ruin goto <ring|vault|deep>` (game masters)
//! stands the player beside the nearest; `/ruin learn|forget <id|all>` (game masters) for runs.

use vitals::ruins::{self as n, Blueprint, Ruin};
use weft_sdk::*;

/// The blueprints a player knows, one bit each (`Blueprint::bit`).
pub static KNOWS: Component<u32> = Component::named("blueprints_known");

value! {
    /// The ruins whose pedestal has given this player its blueprint, by `Ruin::key`.
    pub struct Taken { pub keys: [u32; 32] }
}
static TAKEN: Component<Taken> = Component::named("ruins_taken");

pub static BLUEPRINT_CRUCIBLE: Item = Item::named("blueprint_crucible");
pub static BLUEPRINT_IRONWORK: Item = Item::named("blueprint_ironwork");
pub static BLUEPRINT_FORGE: Item = Item::named("blueprint_forge");
/// The Bronze vaults' shell: a copper pickaxe breaks it (`vitals::ruins::WARD_BRONZE`).
pub static SEALED_STONE: Block = Block::named("sealed_stone");
/// The Iron vaults' shell: a bronze pickaxe breaks it (`vitals::ruins::WARD_IRON`).
pub static WARDED_STONE: Block = Block::named("warded_stone");
/// The ancients' lectern in every ruin: gives each player the ruin's blueprint once.
pub static PEDESTAL: Block = Block::named("ruin_pedestal");
static MOSSY_COBBLESTONE: Block = Block::named("mossy_cobblestone");
static PICKAXES: Tag = Tag::named("tools/pickaxe");

fn item_of(b: &Blueprint) -> &'static Item {
    match b.id {
        "crucible" => &BLUEPRINT_CRUCIBLE,
        "ironwork" => &BLUEPRINT_IRONWORK,
        _ => &BLUEPRINT_FORGE,
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(&KNOWS, 0, component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&KNOWS);
    r.entity_component(&TAKEN, Taken { keys: [0; 32] }, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&TAKEN);
    for b in n::BLUEPRINTS {
        r.item(item_of(b))
            .max_stack(1)
            .on_use(move |u| read(u, b))
            .tag("blueprints")
            .tab(&crate::tabs::INGREDIENTS);
    }
    r.block(&SEALED_STONE)
        .sound_group("stone")
        .hardness(6.0)
        .mined_by("pickaxe", n::WARD_BRONZE)
        .texture("sealed_stone")
        .tag("ley_stone")
        .tab(&crate::tabs::BUILDING);
    r.block(&WARDED_STONE)
        .sound_group("stone")
        .hardness(8.0)
        .mined_by("pickaxe", n::WARD_IRON)
        .texture("warded_stone")
        .tag("ley_stone")
        .tab(&crate::tabs::BUILDING);
    // Not carried off: what it gives is what it is for.
    r.block(&PEDESTAL)
        .sound_group("stone")
        .unbreakable()
        .light(6)
        .on_use(pedestal_used)
        .texture("ruin_pedestal_side")
        .top("ruin_pedestal_top")
        .bottom("ley_stone_top")
        .tab(&crate::tabs::BUILDING);
    // A ward refuses a pickaxe below its harvest: mining doesn't start (a block too hard for a
    // tool otherwise still breaks, dropping nothing, which would open the vault).
    r.before_event(&actions::BREAK_BLOCK, |q| {
        let state = q.state();
        let need = if SEALED_STONE.is(state) {
            n::WARD_BRONZE
        } else if WARDED_STONE.is(state) {
            n::WARD_IRON
        } else {
            return Verdict::Abstain;
        };
        let w = q.world();
        let p = q.entity();
        if p != 0 && w.creative(p) {
            return Verdict::Abstain;
        }
        let tool = if p == 0 { ItemStack::EMPTY } else { w.inventory(p).group("hotbar").get(w.selected_slot(p)) };
        let level = if PICKAXES.on_item(tool.item) { TOOL_LEVEL.on_stack(&tool).unwrap_or(0) } else { 0 };
        if level >= need {
            return Verdict::Abstain;
        }
        if p != 0 {
            w.hint("hint.base.ward.refuses", &[Arg::Name(&format!("block:{}", block_name(state)))]).to(p);
        }
        Verdict::Deny
    });
    r.decoration(ruins).dimension(&crate::OVERWORLD);
    r.command("ruin", "[what:word] [which:word] [how:word]", command);
}

// ------------------------------------------------------------------ knowing

/// What `player` knows: every blueprint in creative.
pub fn known(w: &World, player: u64) -> u32 {
    if w.creative(player) {
        return n::all_bits();
    }
    w.component(player, &KNOWS).unwrap_or(0)
}

/// The blueprint `player` lacks to craft `item`, if one.
pub fn craft_locked(w: &World, player: u64, item: u32) -> Option<&'static Blueprint> {
    n::craft_lock(item_name(item)).filter(|b| !n::knows(known(w, player), b))
}

/// Whether `player` lacks the blueprint for recipe `r`'s output.
pub fn recipe_locked(w: &World, player: u64, r: &RecipeView) -> bool {
    r.outputs()
        .first()
        .is_some_and(|(o, _)| craft_locked(w, player, o.item).is_some())
}

/// The same on the client, for the book: what the local player lacks to craft `item`.
pub fn craft_locked_here(view: &ClientView, item: u32) -> Option<&'static Blueprint> {
    if view.creative() {
        return None;
    }
    let known = view.component(&KNOWS).unwrap_or(0);
    n::craft_lock(item_name(item)).filter(|b| !n::knows(known, b))
}

/// The blueprint's item name, for messages.
pub fn name_of(b: &Blueprint) -> String {
    format!("item:{}", b.item())
}

fn learn(w: &World, player: u64, bits: u32) -> bool {
    let Some(k) = w.component_mut(player, &KNOWS) else { return false };
    let was = *k;
    *k |= bits;
    *k != was
}

/// A blueprint used: its recipes known, the page kept.
fn read(u: &mut ItemUse, b: &'static Blueprint) -> bool {
    let w = u.world();
    let p = u.entity();
    if learn(&w, p, b.bit()) {
        w.hint("hint.base.blueprint.learned", &[Arg::Name(&name_of(b))])
            .to(p);
        crate::advancements::crafted(&w, p, item_of(b).id());
    } else {
        w.hint("hint.base.blueprint.known", &[Arg::Name(&name_of(b))])
            .to(p);
    }
    true
}

/// A furnace's input changed: raw ore its player hasn't the blueprint to smelt goes back to them.
pub fn furnace_changed(m: &MenuCtx) {
    let Some(input) = m.find_group("input") else { return };
    let s = input.get(0);
    if s.is_empty() {
        return;
    }
    let w = m.world();
    let p = m.player();
    let Some(b) = n::smelt_lock(item_name(s.item)) else { return };
    if n::knows(known(&w, p), b) {
        return;
    }
    input.set(0, ItemStack::EMPTY);
    let left = w.inventory(p).insert(s);
    if !left.is_empty()
        && let Some(e) = w.entity(p)
    {
        w.spawn_item(e.pos, left);
    }
    w.hint("hint.base.blueprint.smelt_locked", &[Arg::Name(&format!("item:{}", item_name(s.item))), Arg::Name(&name_of(b))])
        .to(p);
}

// ------------------------------------------------------------------ the pedestal

/// The first ruin near the world's spawn (`vitals::ruins::spawn_ruin`), on ground above the sea;
/// worked out once per seed.
fn first(seed: u64) -> Option<Ruin> {
    static KNOWN: std::sync::Mutex<Vec<(u64, Ruin)>> = std::sync::Mutex::new(Vec::new());
    if let Some(&(_, r)) = KNOWN.lock().unwrap().iter().find(|(s, _)| *s == seed) {
        return Some(r);
    }
    let s = crate::fair_spawn(seed);
    let f = base_api::land::fields(seed);
    let dry = |x: i32, z: i32| base_api::land::height(&f, x * 16, z * 16) >> 4 >= base_api::SEA;
    let r = n::spawn_ruin(seed, [s[0], s[2]], &dry);
    KNOWN.lock().unwrap().push((seed, r));
    Some(r)
}

/// The ruin whose pedestal stands at (`x`, `z`).
fn ruin_at(seed: u64, x: i32, z: i32) -> Option<Ruin> {
    n::ruins_near(seed, first(seed), x, z, 2)
        .into_iter()
        .find(|r| r.x == x && r.z == z)
}

fn pedestal_used(u: &mut Use) -> bool {
    let w = u.world();
    let p = u.player();
    let pos = u.pos();
    let Some(ruin) = ruin_at(w.seed(), pos.x, pos.z) else {
        w.hint("hint.base.pedestal.empty", &[]).to(p);
        return true;
    };
    let Some(b) = n::BLUEPRINTS.iter().find(|b| b.age == ruin.kind().age) else { return true };
    let key = ruin.key();
    let Some(t) = w.component_mut(p, &TAKEN) else { return true };
    if t.keys.contains(&key) {
        w.hint("hint.base.pedestal.taken", &[Arg::Name(&name_of(b))])
            .to(p);
        return true;
    }
    // The oldest giving makes room: a player who has emptied 32 pedestals is past needing one.
    t.keys.rotate_right(1);
    t.keys[0] = key;
    let page = ItemStack::new(item_of(b).id(), 1);
    let left = w.inventory(p).insert(page);
    if !left.is_empty() {
        w.spawn_item([pos.x as f64 + 0.5, pos.y as f64 + 1.2, pos.z as f64 + 0.5], left);
    }
    w.hint("hint.base.pedestal.gives", &[Arg::Name(&name_of(b))])
        .to(p);
    true
}

// ------------------------------------------------------------------ building them

/// The deep vault's spiral stair: the ring cells round the shaft's pillar, a step one lower each.
const STAIR: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

/// A ruin's own noise at (`dx`, `dz`) from its centre.
fn noise(ruin: &Ruin, dx: i32, dz: i32) -> u32 {
    let mut v = ((ruin.x + dx) as u32 as u64) ^ (((ruin.z + dz) as u32 as u64) << 32) ^ ruin.key() as u64;
    v = v.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (v >> 40) as u32
}

/// A ring's eight pillars on its floor's edge: (dx, dz from the centre, how high: 2, broken, to 4).
fn ring_pillars(ruin: &Ruin) -> Vec<(i32, i32, i32)> {
    (0..8)
        .map(|k| {
            let a = k as f64 / 8.0 * std::f64::consts::TAU;
            let (dx, dz) = ((4.0 * a.cos()).round() as i32, (4.0 * a.sin()).round() as i32);
            (dx, dz, 2 + (noise(ruin, dx, dz) % 3) as i32)
        })
        .collect()
}

/// The blocks of `ruin`: (position, block, whether it replaces only what isn't solid). `ground(x,
/// z)` is a column's top block.
fn ruin_blocks(ruin: &Ruin, ground: &dyn Fn(i32, i32) -> i32) -> Vec<([i32; 3], u32, bool)> {
    let (x, z) = (ruin.x, ruin.z);
    let gy = ground(x, z);
    let stone = crate::ley::LEY_STONE.id();
    let moss = MOSSY_COBBLESTONE.id();
    let mut out: Vec<([i32; 3], u32, bool)> = Vec::new();
    let h = |dx: i32, dz: i32| noise(ruin, dx, dz);
    // A box from `a` to `b` (inclusive): its shell in `wall`, its inside air.
    let room = |out: &mut Vec<([i32; 3], u32, bool)>, a: [i32; 3], b: [i32; 3], wall: u32| {
        for y in a[1]..=b[1] {
            for zz in a[2]..=b[2] {
                for xx in a[0]..=b[0] {
                    let edge = y == a[1] || y == b[1] || zz == a[2] || zz == b[2] || xx == a[0] || xx == b[0];
                    out.push(([xx, y, zz], if edge { wall } else { 0 }, false));
                }
            }
        }
    };
    match ruin.kind().id {
        "ring" => {
            // A level floor of worn ley stone and moss (radius 4), its own ground cut away above and
            // built up below on a slope, and a broken ring of pillars standing on its edge: on the
            // floor, so they are seen from the pedestal however the land falls (11-24,
            // pl1123_ruin1 and 3: pillars on their own columns hid below the floor's edge downhill
            // and above the cut uphill).
            for dz in -4..=4i32 {
                for dx in -4..=4i32 {
                    if dx * dx + dz * dz > 20 {
                        continue;
                    }
                    let b = if h(dx, dz) % 3 == 0 { moss } else { stone };
                    out.push(([x + dx, gy, z + dz], b, false));
                    // Built up to the floor from the ground below it (at most 8 deep).
                    let under = ground(x + dx, z + dz).max(gy - 8);
                    for y in under.min(gy - 1)..gy {
                        out.push(([x + dx, y, z + dz], stone, false));
                    }
                    for y in gy + 1..=gy + 5 {
                        out.push(([x + dx, y, z + dz], 0, false));
                    }
                }
            }
            for (dx, dz, tall) in ring_pillars(ruin) {
                for y in 1..=tall {
                    out.push(([x + dx, gy + y, z + dz], if y == tall && tall < 4 { moss } else { stone }, false));
                }
            }
            out.push(([x, gy + 1, z], PEDESTAL.id(), false));
        }
        "vault" => {
            // Roof flush with the ground, of sealed stone; inside 5 x 5 x 3, the pedestal in the
            // middle; four broken pillars of ley stone at its corners above ground.
            let ward = SEALED_STONE.id();
            let floor = gy - ruin.kind().depth;
            room(&mut out, [x - 3, floor, z - 3], [x + 3, gy, z + 3], ward);
            out.push(([x, floor + 1, z], PEDESTAL.id(), false));
            for (dx, dz) in [(-3, -3), (3, -3), (-3, 3), (3, 3)] {
                let tall = 1 + h(dx, dz) % 3;
                for y in 1..=tall as i32 {
                    out.push(([x + dx, gy + y, z + dz], stone, false));
                }
            }
            for dz in -3..=3 {
                for dx in -3..=3 {
                    for y in gy + 1..=gy + 3 {
                        out.push(([x + dx, y, z + dz], 0, true));
                    }
                }
            }
        }
        _ => {
            // A shrine of ley stone round a shaft, a spiral stair down to a vault of warded stone:
            // the stair ends on its roof.
            let ward = WARDED_STONE.id();
            let floor = gy - ruin.kind().depth;
            let roof = floor + 4;
            room(&mut out, [x - 3, floor, z - 3], [x + 3, roof, z + 3], ward);
            out.push(([x, floor + 1, z], PEDESTAL.id(), false));
            // The shaft: walls 5 x 5 from the roof to a little above the ground, inside 3 x 3 with
            // a pillar in the middle and a step in each ring cell, one lower each.
            for y in roof + 1..=gy + 2 {
                for dz in -2..=2i32 {
                    for dx in -2..=2i32 {
                        let wall = dx.abs() == 2 || dz.abs() == 2;
                        let b = if wall {
                            // The shrine's doorway on the ruin's turn side, at the ground.
                            let door = (y == gy + 1 || y == gy + 2)
                                && match ruin.turn() {
                                    0 => (dx, dz) == (0, -2),
                                    1 => (dx, dz) == (2, 0),
                                    2 => (dx, dz) == (0, 2),
                                    _ => (dx, dz) == (-2, 0),
                                };
                            if door || (y == gy + 2 && h(dx, dz) % 3 == 0) { 0 } else { stone }
                        } else if (dx, dz) == (0, 0) {
                            stone
                        } else {
                            0
                        };
                        out.push(([x + dx, y, z + dz], b, false));
                    }
                }
            }
            let mut k = 0;
            let mut y = gy;
            while y > roof + 1 {
                let (dx, dz) = STAIR[k % 8];
                out.push(([x + dx, y, z + dz], stone, false));
                k += 1;
                y -= 1;
            }
            // The shrine's floor round it, and its doorway cleared out to the open.
            for dz in -3..=3 {
                for dx in -3..=3 {
                    for y in gy + 1..=gy + 3 {
                        out.push(([x + dx, y, z + dz], 0, true));
                    }
                }
            }
        }
    }
    out
}

/// The ruins (`vitals::ruins`), each cube its part.
fn ruins(g: &mut Gen) {
    if PEDESTAL.id() == u32::MAX {
        return;
    }
    let [ox, oy, oz] = g.origin().to_array();
    let seed = g.seed();
    let list = n::ruins_near(seed, first(seed), ox + 8, oz + 8, 8 + n::REACH);
    if list.is_empty() {
        return;
    }
    let inside = |p: [i32; 3]| (ox..ox + 16).contains(&p[0]) && (oy..oy + 16).contains(&p[1]) && (oz..oz + 16).contains(&p[2]);
    for ruin in list {
        let heights: std::collections::HashMap<(i32, i32), i32> = (-n::REACH..=n::REACH)
            .flat_map(|dz| (-n::REACH..=n::REACH).map(move |dx| (ruin.x + dx, ruin.z + dz)))
            .map(|(x, z)| ((x, z), g.column(x, z).height))
            .collect();
        let ground = |x: i32, z: i32| heights.get(&(x, z)).copied().unwrap_or(i32::MIN / 2);
        // Every cube the ruin reaches, by its footprint's lowest and highest ground: a ring's
        // pillars stand on their own columns, which on a slope lie well above or below the
        // centre's (11-24, pl1123_ruin1: a ring on a hillside showed no pillars).
        let gy = ground(ruin.x, ruin.z);
        let low = heights
            .values()
            .copied()
            .min()
            .unwrap_or(gy)
            .min(gy - ruin.kind().depth);
        let high = heights.values().copied().max().unwrap_or(gy).max(gy);
        if low - 2 > oy + 15 || high + 6 < oy {
            continue;
        }
        for (p, b, soft) in ruin_blocks(&ruin, &ground) {
            if !inside(p) || b == u32::MAX {
                continue;
            }
            let (x, y, z) = ((p[0] - ox) as usize, (p[1] - oy) as usize, (p[2] - oz) as usize);
            if soft {
                let now = g.get(x, y, z);
                if now == 0 || liquid_of(now).is_some() || state_flags(now) & abi::block_flags::SOLID != 0 {
                    continue;
                }
            }
            g.set(x, y, z, b);
        }
    }
}

// ------------------------------------------------------------------ /ruin

fn command(cmd: &mut ServerCommand) {
    let w = cmd.world();
    let p = cmd.player();
    let seed = w.seed();
    let spawn = first(seed);
    let Some(st) = w.entity(p) else { return };
    let (x, z) = (st.pos.x.floor() as i32, st.pos.z.floor() as i32);
    let master = w.permission(p) >= permission::GAME_MASTER;
    let which = cmd.text("which").map(str::to_string);
    match cmd.text("what") {
        Some("goto") if master => {
            let k = which
                .as_deref()
                .and_then(|id| n::KINDS.iter().position(|k| k.id == id))
                .unwrap_or(0);
            let Some(ruin) = n::nearest(seed, spawn, k, x, z, 6) else {
                cmd.reply("command.base.ruin.none", &[Arg::Key(&format!("ruin.base.{}", n::KINDS[k].id))]);
                return;
            };
            let f = base_api::land::fields(seed);
            // A ring: on its floor two blocks south of the pedestal (yaw 0 faces it). A vault:
            // beside it, outside its walls (`ruin.turn()`'s side, the shrine's door).
            let ring = ruin.kind().depth == 0;
            let (dx, dz) = match (ring, ruin.turn()) {
                (true, _) => (0, 2),
                (_, 0) => (0, -6),
                (_, 1) => (6, 0),
                (_, 2) => (0, 6),
                _ => (-6, 0),
            };
            // `above`: 12 over its floor, 6 south (look 0 -60 to see it whole); `side`: 15 south at
            // eye height over the floor or the ground there, whichever is higher (look 0 -5).
            // For look shots in spectator (11-24, Planner).
            let how = cmd.text("how").map(str::to_string);
            let centre = base_api::land::height(&f, ruin.x * 16, ruin.z * 16) >> 4;
            let floor = centre - ruin.kind().depth;
            let (dx, dz, up) = match how.as_deref() {
                Some("above") => (0, 6, Some(centre + 12)),
                // Inside a vault, two south of its pedestal (yaw 0 faces it).
                Some("inside") if !ring => (0, 2, Some(floor + 1)),
                // On a vault's roof with roof one north of it (`look 0 -60` aims at that one, so
                // breaking it doesn't drop the player in): one south of the sealed vault's middle;
                // the deep vault's in its shaft, a side cell pair clear of the stair's last two
                // steps (head room; pl1124_vaults1: digging straight down fell through the floor).
                Some("roof") if !ring => {
                    if ruin.kind().id == "deep" {
                        let last = (centre - (floor + 4) - 2).max(0) as usize;
                        let steps = [STAIR[last % 8], STAIR[(last + 7) % 8]];
                        let (cx, cz) = [(-1, 1), (-1, 0), (1, 1), (1, 0)]
                            .into_iter()
                            .find(|&(cx, cz)| !steps.contains(&(cx, cz)) && !steps.contains(&(cx, cz - 1)))
                            .unwrap_or((-1, 1));
                        (cx, cz, Some(floor + 5))
                    } else {
                        (0, 1, Some(centre + 1))
                    }
                }
                // Over the deep vault's shaft, to look down its stair.
                Some("shaft") if !ring => (1, 1, Some(centre + 3)),
                Some("side") => {
                    let there = base_api::land::height(&f, ruin.x * 16, (ruin.z + 15) * 16) >> 4;
                    (0, 15, Some(there.max(centre) + 1))
                }
                _ => (dx, dz, None),
            };
            let (gx, gz) = (ruin.x + dx, ruin.z + dz);
            let (hx, hz) = if ring { (ruin.x, ruin.z) } else { (gx, gz) };
            let y = (base_api::land::height(&f, hx * 16, hz * 16) >> 4) + if ring { 1 } else { 2 };
            let y = if ring { y } else { y.max(base_api::SEA + 2) };
            let y = up.unwrap_or(y);
            w.entity_modify(p)
                .pos([gx as f64 + 0.5, y as f64, gz as f64 + 0.5])
                .velocity([0.0, 0.0, 0.0])
                .fall_distance(0.0);
            cmd.reply(
                "command.base.ruin.goto",
                &[Arg::Key(&format!("ruin.base.{}", ruin.kind().id)), Arg::Int(ruin.x as i64), Arg::Int(ruin.z as i64)],
            );
        }
        Some("check") if master => {
            // The nearest ruin of a kind as it stands, against what its decoration builds: each
            // block it puts (not the soft clearing), by block, how many are there (a run's proof).
            let k = which.as_deref().and_then(|id| n::KINDS.iter().position(|k| k.id == id)).unwrap_or(0);
            let Some(ruin) = n::nearest(seed, spawn, k, x, z, 2) else {
                cmd.reply("command.base.ruin.none", &[Arg::Key(&format!("ruin.base.{}", n::KINDS[k].id))]);
                return;
            };
            let f = base_api::land::fields(seed);
            let ground = |gx: i32, gz: i32| base_api::land::height(&f, gx * 16, gz * 16) >> 4;
            let mut planned: std::collections::BTreeMap<[i32; 3], u32> = std::collections::BTreeMap::new();
            for (p3, b, soft) in ruin_blocks(&ruin, &ground) {
                if !soft {
                    planned.insert(p3, b);
                }
            }
            let mut by: std::collections::BTreeMap<String, (i64, i64)> = std::collections::BTreeMap::new();
            for (p3, b) in &planned {
                let here = w.block(*p3);
                let name = if *b == 0 { "air".to_string() } else { block_name(*b).to_string() };
                let ok = here == *b || block_of(here) == block_of(*b);
                let e = by.entry(name).or_insert((0, 0));
                e.0 += ok as i64;
                e.1 += 1;
            }
            let all_ok = by.values().all(|(a, b)| a == b);
            for (name, (a, b)) in &by {
                cmd.reply("command.base.ruin.check_line", &[Arg::Text(name), Arg::Int(*a), Arg::Int(*b)]);
            }
            cmd.reply(
                "command.base.ruin.check",
                &[
                    Arg::Key(&format!("ruin.base.{}", ruin.kind().id)),
                    Arg::Int(ruin.x as i64),
                    Arg::Int(ruin.z as i64),
                    Arg::Text(if all_ok { "whole" } else { "broken" }),
                ],
            );
        }
        Some("pillars") if master => {
            // The nearest ring's pillars as they stand in the world: each column over the floor,
            // counted up while it is ley stone or moss (a run's proof beside its shots).
            let Some(ruin) = n::nearest(seed, spawn, 0, x, z, 2) else {
                cmd.reply("command.base.ruin.none", &[Arg::Key("ruin.base.ring")]);
                return;
            };
            let f = base_api::land::fields(seed);
            let gy = base_api::land::height(&f, ruin.x * 16, ruin.z * 16) >> 4;
            let mut standing = 0;
            let pillars = ring_pillars(&ruin);
            for &(dx, dz, tall) in &pillars {
                let (px, pz) = (ruin.x + dx, ruin.z + dz);
                let high = (1..=6)
                    .take_while(|&y| {
                        let b = w.block([px, gy + y, pz]);
                        crate::ley::LEY_STONE.is(b) || MOSSY_COBBLESTONE.is(b)
                    })
                    .count() as i64;
                if high == tall as i64 {
                    standing += 1;
                }
                cmd.reply(
                    "command.base.ruin.pillar",
                    &[Arg::Int(px as i64), Arg::Int((gy + 1) as i64), Arg::Int(pz as i64), Arg::Int(high), Arg::Int(tall as i64)],
                );
            }
            let floor = w.block([ruin.x, gy, ruin.z]);
            cmd.reply(
                "command.base.ruin.pillars",
                &[Arg::Int(standing), Arg::Int(pillars.len() as i64), Arg::Int(gy as i64), Arg::Text(block_name(floor))],
            );
        }
        Some(what @ ("learn" | "forget")) if master => {
            let bits = match which.as_deref() {
                Some("all") | None => n::all_bits(),
                Some(id) => match n::blueprint(id) {
                    Some(b) => b.bit(),
                    None => {
                        cmd.reply("command.base.ruin.no_blueprint", &[Arg::Text(id)]);
                        return;
                    }
                },
            };
            if let Some(k) = w.component_mut(p, &KNOWS) {
                if what == "learn" { *k |= bits } else { *k &= !bits }
            }
            if what == "forget"
                && let Some(t) = w.component_mut(p, &TAKEN)
            {
                t.keys = [0; 32];
            }
            let now = w.component(p, &KNOWS).unwrap_or(0);
            cmd.reply("command.base.ruin.knows", &[Arg::Text(&known_list(now))]);
        }
        _ => {
            for (k, kind) in n::KINDS.iter().enumerate() {
                match n::nearest(seed, spawn, k, x, z, 4) {
                    Some(r) => {
                        let d = (((r.x - x) as f64).powi(2) + ((r.z - z) as f64).powi(2))
                            .sqrt()
                            .round() as i64;
                        cmd.reply(
                            "command.base.ruin.nearest",
                            &[Arg::Key(&format!("ruin.base.{}", kind.id)), Arg::Int(r.x as i64), Arg::Int(r.z as i64), Arg::Int(d)],
                        );
                    }
                    None => cmd.reply("command.base.ruin.none", &[Arg::Key(&format!("ruin.base.{}", kind.id))]),
                }
            }
            let now = known(&w, p);
            cmd.reply("command.base.ruin.knows", &[Arg::Text(&known_list(now))]);
        }
    }
}

/// The ids of the blueprints in `bits`, "none" for none.
fn known_list(bits: u32) -> String {
    let v: Vec<&str> = n::BLUEPRINTS
        .iter()
        .filter(|b| n::knows(bits, b))
        .map(|b| b.id)
        .collect();
    if v.is_empty() { "-".into() } else { v.join(" ") }
}

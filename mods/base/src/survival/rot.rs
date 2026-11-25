//! Rot by place (`vitals::rot`): food keeps as tuned in a cool larder, slower in the cold and faster
//! in the heat. A food stack's making stamp (`MADE`, in ticks) is moved on by where it sits: forward
//! where it keeps better than a larder, back where it keeps worse, so everything that reads its age
//! (eating, the tooltip, cooking, crafting, the animals' carrion) sees the place without knowing it.
//!
//! - In a store (a chest), once a game hour, by the store's air (`store_celsius`), catching up the
//!   time it was unloaded or the clock jumped by the same rule.
//! - Carried, every second, by the air the player feels (`Vitals::outside`).
//!
//! `/rot <pos>` says each food in a store, its freshness, the air and the rate (Tester's runs).

use std::collections::HashMap;
use std::sync::Mutex;

use base_api::{FRESHNESS, MADE};
use vitals::rot;
use weft_sdk::*;

value! {
    /// A store's rot clock: the day time its food was last aged at (0: not yet), and the part of
    /// a tick left over (`rot::shift`).
    pub struct Larder { pub last: u64, pub carry: f32 }
}
pub static LARDER: Component<Larder> = Component::named("base:larder");

/// Blocks around a store whose fires warm it (a chest by a furnace).
const STORE_REACH: i32 = 2;

/// Per player: the day time their carried food was last aged at, and its carry.
static CARRIED: Mutex<Option<HashMap<u64, (u64, f32)>>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.entity_component(&LARDER, Larder { last: 0, carry: 0.0 }, component_flags::SAVE);
    r.system(&LARDER, tick_stores);
    r.command("rot", "<pos:pos>", |cmd| {
        let w = cmd.world();
        let Some(p) = cmd.pos("pos") else { return };
        let at = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        let air = store_celsius(&w, at);
        let rate = rot::rate(air);
        let inv = w.inventory(BlockPos::new(at[0], at[1], at[2]));
        let now = w.day_time();
        let mut foods = Vec::new();
        for i in 0..inv.len() {
            let s = inv.get(i);
            let Some(used) = super::food::life_used(&s, now) else { continue };
            let state = FRESHNESS.on_stack_own(&s).unwrap_or(0);
            foods.push(format!("{}x{} {} {:.0}%", item_name(s.item), s.count, freshness_name(state), used * 100.0));
        }
        let list = foods.join(", ");
        weft_sdk::info!(
            "rot: {} {} {} at {:.1} °C rate x{:.2} day {:.2}: {}",
            at[0],
            at[1],
            at[2],
            air,
            rate,
            now as f64 / vitals::calendar::DAY_TICKS as f64,
            list
        );
        cmd.reply("command.base.rot", &[Arg::Float(air as f64, 1), Arg::Float(rate as f64, 2), Arg::Text(&list)]);
    });
}

fn freshness_name(state: u8) -> &'static str {
    match state {
        base_api::freshness::FRESH => "fresh",
        base_api::freshness::STALE => "stale",
        base_api::freshness::SPOILED => "spoiled",
        _ => "hot",
    }
}

/// Snow or ice: a store next to it is an ice house (`rot::ICE_C`).
fn icy(state: u32) -> bool {
    let n = block_name(state).split('[').next().unwrap_or("");
    let n = n.split_once(':').map_or(n, |(_, n)| n);
    n == "ice" || n.ends_with("_ice") || n.starts_with("snow") || n == "powder_snow"
}

/// The air food in a store at `at` keeps in: the air there out of the wind (the sun on it, caves'
/// year-round mean, a fire close by), a room's evened-out air inside walls as a player feels it
/// (`shelter`), held at `rot::ICE_C` by snow or ice next to it.
pub fn store_celsius(w: &World, at: [i32; 3]) -> f32 {
    let pos = [at[0] as f64 + 0.5, at[1] as f64, at[2] as f64 + 0.5];
    let here = base_api::air_at(w, pos, 1.0, false, STORE_REACH, |_| {});
    let room = if here.underground { None } else { super::shelter::room(w, at) };
    let air = room.map_or(here.still, |r| r.air(here.mean, here.still - here.fire, here.fire));
    const SIDES: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    let ice = SIDES
        .iter()
        .any(|d| icy(w.block([at[0] + d[0], at[1] + d[1], at[2] + d[2]])));
    rot::store_air(air, ice)
}

/// Moves every food stamp in an inventory on by `ticks` at `rate`, then its freshness mark.
fn rot_inventory(w: &World, id: impl Into<Holder>, ticks: u64, rate: f32, carry: f32, merge: bool) -> f32 {
    let (shift, carry) = rot::shift(ticks, rate, carry);
    let inv = w.inventory(id);
    let now = w.day_time();
    for i in 0..inv.len() {
        let s = inv.get(i);
        if s.is_empty() || FRESHNESS.on_item(s.item).is_none() {
            continue;
        }
        let made = MADE.on_stack_own(&s).unwrap_or(0);
        if made == 0 || shift == 0 {
            continue;
        }
        // Never younger than now (the stamp is now + 1), never unstamped (0).
        let moved = (made as i64 + shift).clamp(1, now as i64 + 1) as u32;
        if moved != made {
            inv.set(i, s.with(&MADE, moved));
        }
    }
    super::food::age_inventory_of(&inv, now, merge);
    carry
}

/// Every chest's food, once a game hour by day time (so it catches up unloaded time and clock
/// jumps the same way).
fn tick_stores(b: &mut SystemBatch<Larder>) {
    let w = b.world();
    let now = w.day_time();
    for (id, l, _) in b.iter_mut() {
        if l.last == 0 || now < l.last {
            l.last = now;
            continue;
        }
        let ticks = now - l.last;
        if ticks < rot::STORE_EVERY as u64 {
            continue;
        }
        let Some(st) = w.entity(id) else { continue };
        let rate = rot::rate(store_celsius(&w, st.block_pos().to_array()));
        l.carry = rot_inventory(&w, id, ticks, rate, l.carry, false);
        l.last = now;
    }
}

/// A player's carried food, by the air they feel (`celsius`), then stamped and marked as it ages.
pub fn carried(w: &World, player: u64, celsius: f32) {
    let now = w.day_time();
    let mut map = CARRIED.lock().unwrap();
    let map = map.get_or_insert_with(HashMap::new);
    let (last, carry) = map.get(&player).copied().unwrap_or((now, 0.0));
    let ticks = now.saturating_sub(last);
    let carry = if ticks > 0 {
        rot_inventory(w, player, ticks, rot::rate(celsius), carry, true)
    } else {
        super::food::age(w, player);
        carry
    };
    map.insert(player, (now, carry));
}

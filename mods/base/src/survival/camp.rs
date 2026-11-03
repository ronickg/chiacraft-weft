//! Camp (Valheim's): near a fire, under a roof, and rested. Resting under a roof by a fire for a
//! while, or asleep in a bed, makes one rested for eight minutes and one more for each kind of
//! comfort around (a fire, a bed, a table, storage, something soft, flowers, a lamp), kept full
//! while one rests on.

use base_api::{Air, Vitals};
use vitals::effects::{self, Effect};
use weft_sdk::*;

use super::set_group;

/// Seconds of resting (under a roof by a fire, or asleep) that make one rested.
pub const REST_SECS: f32 = 20.0;
/// Degrees from a fire that count as near one.
const NEAR_FIRE: f32 = 3.0;

/// The kind of comfort a block gives, a bit each (each kind counts once).
pub fn comfort_of(state: u32) -> u32 {
    match block_name(state).split('[').next().unwrap_or("") {
        "base:lava" | "base:torch" | "base:wall_torch" => 1,
        "base:furnace" if state_prop(state, "lit") == Some("true") => 1,
        "base:bed" => 2,
        "base:crafting_table" => 4,
        "base:chest" => 8,
        "base:wool" => 16,
        "base:dandelion" | "base:rose" => 32,
        "base:glowstone" | "base:redstone_lamp" | "base:copper_lamp" => 64,
        _ => 0,
    }
}

/// This second's camp: near a fire, sheltered, resting and rested.
pub fn rest(v: &mut Vitals, st: &EntityState, here: &Air, comfort: u32) {
    let near_fire = here.fire >= NEAR_FIRE;
    let sheltered = here.sky < 13 && !here.underground;
    v.comfort = 1 + comfort.count_ones() as u8;
    if (sheltered && near_fire) || st.pose == abi::pose::SLEEP {
        v.resting = (v.resting + 1.0).min(REST_SECS);
    } else {
        v.resting = 0.0;
    }
    if v.resting >= REST_SECS {
        v.rested = effects::rested_secs(v.comfort as u32);
    } else {
        v.rested = (v.rested - 1.0).max(0.0);
    }
    let on: Vec<Effect> = [(near_fire, Effect::NearFire), (sheltered, Effect::Sheltered), (v.rested > 0.0, Effect::Rested)]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, e)| e)
        .collect();
    set_group(v, &[Effect::NearFire, Effect::Sheltered, Effect::Rested], &on);
}

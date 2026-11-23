//! Waterholes that shrink in the dry season (`notes/ECOLOGY.md`, "Later"; `vitals::climate`,
//! `waterholes_shrink`): a pond's open still water, a block deep, dries from its rim into a crust
//! of cracked mud (`CRUST`) in its biome's dry season, an edge block in about
//! `climate::SHRINK_DAYS`, down to a last pool of a block or two (`climate::edge_dries`), and fills
//! back once the season is over (`climate::REFILL_SECS`). The herds crowd round what is left: a
//! drinker's remembered shore that dried is no shore, so it looks for the nearest water again, and
//! with none near treks to what its herd remembers (the animals mod's `drink.rs`).
//!
//! Cheaply, as lake ice forms (`snow.rs`): drying is looked at on the snow's visits to columns
//! near players; refilling is the crust's own growth step, also away from players.

use std::sync::Mutex;

use vitals::{air, climate};
use weft_sdk::*;

use crate::blocks::WATER;
use base_api::CLIMATE;

/// Dried pond bed: a thin crust of cracked mud where the water stood, walked over. It remembers
/// the water: it fills back when its season is no longer dry.
pub static CRUST: Block = Block::named("dried_mud");

/// Testing (`/waterholes`): `Some(true)` every place's still water shrinks as in a dry season,
/// `Some(false)` none does (the crust fills back), and how many times faster; None: the climate.
static FORCE: Mutex<(Option<bool>, f32)> = Mutex::new((None, 1.0));

pub fn register(r: &mut Registry) {
    r.block(&CRUST)
        .states(|_, s| s.model_box([0.0, 0.0, 0.0], [16.0, 2.0, 16.0]).not_solid())
        .texture("dried_mud")
        .sound_group("gravel")
        .hardness(0.2)
        .mined_by("shovel", weft_sdk::abi::tool_tier::HAND)
        .drops_nothing()
        .no_item()
        .grows(climate::REFILL_SECS, 0)
        .on_grow(|e| {
            let w = e.world();
            let pos = e.pos().to_array();
            if !shrinks(&w, pos) {
                w.set_block(pos, WATER.id());
            }
        });
    r.command("waterholes", "<what:word(dry,wet,real)> [speed:float]", |cmd| {
        let force = match cmd.text("what").unwrap_or("real") {
            "dry" => Some(true),
            "wet" => Some(false),
            _ => None,
        };
        let speed = cmd.float("speed").unwrap_or(1.0).max(1.0) as f32;
        *FORCE.lock().unwrap() = (force, speed);
        let n = Arg::Int(speed.round() as i64);
        match force {
            Some(true) => cmd.reply("command.base.waterholes.dry", &[n]),
            Some(false) => cmd.reply("command.base.waterholes.wet", &[n]),
            None => cmd.reply("command.base.waterholes.real", &[]),
        }
    });
}

/// Whether still water at `pos` shrinks now: its biome's dry season (not the sea's).
fn shrinks(w: &World, pos: [i32; 3]) -> bool {
    if let (Some(f), _) = *FORCE.lock().unwrap() {
        return f;
    }
    let biome = w.biome(pos);
    let Some(c) = CLIMATE.on_biome(biome) else { return false };
    let (_, _, season) = crate::weather::now(w);
    climate::waterholes_shrink(name_of(abi::entry_kind::BIOME, biome), c.rain, season)
}

/// A snow visit (`snow::column`) to open still water at `pos`: an edge block of a pond a block
/// deep dries in its dry season.
pub fn visit_water(w: &World, pos: [i32; 3]) {
    let speed = FORCE.lock().unwrap().1;
    let chance = (climate::shrink_chance(air::snow_visit_secs()) * speed).min(1.0);
    if (w.random_below(1 << 16) as f32) >= chance * (1 << 16) as f32 {
        return;
    }
    let [x, y, z] = pos;
    // A block deep: the floor under it is ground.
    if w.block_loaded([x, y - 1, z])
        .is_none_or(|b| b == 0 || b == WATER.id())
    {
        return;
    }
    let (mut water, mut land) = (0u8, 0u8);
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        match w.block_loaded([x + dx, y, z + dz]) {
            None => return,
            Some(b) if b == WATER.id() => water += 1,
            // Flowing water, ice, plants in it: the pool goes on there.
            Some(b) if state_flags(b) & abi::block_flags::LIQUID != 0 => water += 1,
            Some(b) if b == 0 || CRUST.is(b) || state_flags(b) & abi::block_flags::REPLACEABLE != 0 => land += 1,
            Some(_) => land += 1,
        }
    }
    if climate::edge_dries(water, land) && shrinks(w, pos) {
        w.set_block(pos, CRUST.id());
    }
}

/// A snow visit to a crust at `pos`: it fills back sooner near players when the season has
/// turned (`speed` times its growth step's rate, for testing).
pub fn visit_crust(w: &World, pos: [i32; 3]) {
    let speed = FORCE.lock().unwrap().1;
    let chance = (air::snow_visit_secs() / climate::REFILL_SECS * speed).min(1.0);
    if (w.random_below(1 << 16) as f32) < chance * (1 << 16) as f32 && !shrinks(w, pos) {
        w.set_block(pos, WATER.id());
    }
}

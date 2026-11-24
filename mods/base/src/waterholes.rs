//! Waterholes that shrink in the dry season (`notes/ECOLOGY.md`, "Later"; `vitals::climate`,
//! `waterholes_shrink`): a pond's open still water, a block deep, dries from its rim into a crust
//! of cracked mud (`CRUST`) in its biome's dry season, a corner block in about
//! `climate::SHRINK_DAYS`, receding from its corners down to one last block (`climate::edge_dries`), and fills
//! back once the season is over (`climate::REFILL_SECS`). The herds crowd round what is left: a
//! drinker's remembered shore that dried is no shore, so it looks for the nearest water again, and
//! with none near treks to what its herd remembers (the animals mod's `drink.rs`).
//!
//! Cheaply, as lake ice forms (`snow.rs`): drying is looked at on the snow's visits to columns
//! near players; refilling is the crust's own growth step, also away from players.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use vitals::{air, climate};
use weft_sdk::*;

use crate::blocks::WATER;
use base_api::CLIMATE;

/// Dried pond bed: a thin crust of cracked mud where the water stood, stepped onto (solid, so the
/// water beside it can't flow back in). It remembers
/// the water: it fills back when its season is no longer dry.
pub static CRUST: Block = Block::named("dried_mud");

/// Testing (`/waterholes`): `Some(true)` every place's still water shrinks as in a dry season,
/// `Some(false)` none does (the crust fills back), and how many times faster; None: the climate.
static FORCE: Mutex<(Option<bool>, f32)> = Mutex::new((None, 1.0));

/// Testing: what the visits found while `/waterholes` is forced, logged every `REPORT` ticks:
/// water visited, dried; not dried for the roll, a floor of water or air, not an edge, the season;
/// crusts visited, filled.
static SEEN: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
const REPORT: u64 = 400;

fn seen(i: usize) {
    SEEN[i].fetch_add(1, Relaxed);
}

/// Testing: the visits' counts since the last, logged while `/waterholes` is forced.
fn report(w: &World) {
    if FORCE.lock().unwrap().0.is_none() || !w.game_time().is_multiple_of(REPORT) {
        return;
    }
    let n: Vec<u32> = SEEN.iter().map(|c| c.swap(0, Relaxed)).collect();
    weft_sdk::info!(
        "waterholes: {} water visits, {} dried (not: {} roll, {} floor, {} edge, {} season); {} crust visits, {} filled",
        n[0],
        n[1],
        n[2],
        n[3],
        n[4],
        n[5],
        n[6],
        n[7]
    );
}

pub fn register(r: &mut Registry) {
    r.world_tick(report);
    r.block(&CRUST)
        // Solid, though thin: water renews itself into any room with two sources beside it (the
        // engine's `renewable`, as vanilla's infinite water), which filled every dried block back
        // at once while it was not solid (ai1123_waterhole5: 30-40 dried a window, none stayed).
        .states(|_, s| {
            s.model_box([0.0, 0.0, 0.0], [16.0, 2.0, 16.0])
                .hitbox([0.0, 0.0, 0.0], [16.0, 2.0, 16.0])
        })
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

/// A snow visit (`snow::column`) to open still water at `pos`: a corner block of a pond a block
/// deep dries in its dry season.
pub fn visit_water(w: &World, pos: [i32; 3]) {
    seen(0);
    let speed = FORCE.lock().unwrap().1;
    let chance = (climate::shrink_chance(air::snow_visit_secs()) * speed).min(1.0);
    if (w.random_below(1 << 16) as f32) >= chance * (1 << 16) as f32 {
        seen(2);
        return;
    }
    let [x, y, z] = pos;
    // A block deep: the floor under it is ground.
    if w.block_loaded([x, y - 1, z])
        .is_none_or(|b| b == 0 || WATER.is(b))
    {
        seen(3);
        return;
    }
    // Water, flowing or still, and other liquids: the pool goes on there.
    let is_water = |p: [i32; 3]| {
        w.block_loaded(p)
            .map(|b| WATER.is(b) || state_flags(b) & abi::block_flags::LIQUID != 0)
    };
    let mut sides = [false; 4];
    for (i, (dx, dz)) in [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().enumerate() {
        let Some(b) = is_water([x + dx, y, z + dz]) else { return };
        sides[i] = b;
    }
    if !climate::edge_dries(sides, |dx, dz| is_water([x + dx, y, z + dz]).unwrap_or(false)) {
        seen(4);
    } else if !shrinks(w, pos) {
        seen(5);
    } else {
        seen(1);
        w.set_block(pos, CRUST.id());
    }
}

/// A snow visit to a crust at `pos`: it fills back sooner near players when the season has
/// turned (`speed` times its growth step's rate, for testing).
pub fn visit_crust(w: &World, pos: [i32; 3]) {
    seen(6);
    let speed = FORCE.lock().unwrap().1;
    let chance = (air::snow_visit_secs() / climate::REFILL_SECS * speed).min(1.0);
    if (w.random_below(1 << 16) as f32) < chance * (1 << 16) as f32 && !shrinks(w, pos) {
        seen(7);
        w.set_block(pos, WATER.id());
    }
}

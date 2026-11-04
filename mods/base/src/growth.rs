//! Plants grow by the place (`notes/ECOLOGY.md`, part 3): every growth step asks the plant's rate
//! (`vitals::plants`) from the ground where it stands (`base_api::ground_at`: the day's mean air
//! and the soil's water from the last days' rain), and goes ahead with that chance. So wheat
//! rests through winter and a drought, berries ripen only in their season, and a field by water
//! keeps growing when the rain stops. Unloaded plants catch up at today's season. `/growth` says
//! how a crop would grow where you stand. Frost hurts the tender ones (`vitals::plants`, `frost`):
//! a growth step in air colder than the plant takes (a night's frost where it stands; a fire
//! nearby keeps it off) kills spring wheat, takes a berry bush's fruit, and kills a cane's or a
//! cactus's stalk down to its base, which grows again in spring.

use vitals::plants::{self, Plant};
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    // The ground where the player stands, and how fast wheat would grow there now.
    r.command("growth", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let pos = st.pos.map(|v| v.floor() as i32);
        let g = base_api::ground_at(&w, pos);
        // Standing on farmland (a little lower than a block: the feet are in its block), its
        // fertility too (`farming.rs`).
        let below = [pos[0], pos[1] - 1, pos[2]];
        let fertility = crate::farming::fertility(&w, pos).or_else(|| crate::farming::fertility(&w, below));
        let wheat = plants::rate(plant("base:wheat"), g.mean_air, g.soil) * fertility.map_or(1.0, vitals::soil::growth);
        let (air, soil, rate) = (g.mean_air as f64, (g.soil * 100.0).round() as i64, (wheat * 100.0).round() as i64);
        match fertility {
            Some(f) => cmd.reply(
                "command.base.growth_farmland",
                &[Arg::Float(air, 1), Arg::Int(soil), Arg::Int(rate), Arg::Int(f as i64), Arg::Int(vitals::soil::MAX as i64)],
            ),
            None => cmd.reply("command.base.growth", &[Arg::Float(air, 1), Arg::Int(soil), Arg::Int(rate)]),
        }
    });
}

/// Its plant's facts, by block name (`base_api::grow::plant`).
pub fn plant(block: &str) -> &'static Plant {
    base_api::grow::plant(block)
}

/// The engine's mean seconds between growth steps (`base_api::grow::step_secs`).
pub fn step_secs(block: &str, steps: u32, game: f32) -> f32 {
    base_api::grow::step_secs(block, steps, game)
}

/// A growth step of the plant at `e` (`base_api::grow::step`): `step` if it grows, the frost's
/// harm if a frost came; `last` says whether this step would be its last stage (the ripe fruit),
/// which waits for its season.
pub fn grow(e: &mut BlockEvent, block: &str, last: bool, step: impl FnOnce(&mut BlockEvent)) {
    match base_api::grow::step(e, block, last) {
        base_api::grow::Step::Grow => step(e),
        base_api::grow::Step::Frost => frost(e, block),
        base_api::grow::Step::Rest => {}
    }
}

/// What a frost does to the plant at `e` (`grow`).
fn frost(e: &mut BlockEvent, block: &str) {
    let w = e.world();
    let pos = e.pos();
    let below = [pos[0], pos[1] - 1, pos[2]];
    let above = [pos[0], pos[1] + 1, pos[2]];
    match block {
        // The crop dies; the farmland stays.
        "base:wheat" => {
            w.set(pos, 0);
        }
        // The stalk dies from the top, down to the base, which sprouts again in spring.
        "base:sugar_cane" | "base:cactus" if w.get(below) == e.state() && w.get(above) != e.state() => {
            w.set(pos, 0);
        }
        _ => crate::wild::frost(e, block),
    }
}

/// The block's `age` one on, up to `max`.
pub fn next_age(e: &mut BlockEvent, max: u32) {
    let age = state_prop(e.state(), "age")
        .and_then(|a| a.parse::<u32>().ok())
        .unwrap_or(max);
    if age < max {
        e.world().set_prop(e.pos(), "age", &(age + 1).to_string());
    }
}

/// The block's `age` (its stage).
pub fn age(e: &BlockEvent) -> u32 {
    state_prop(e.state(), "age")
        .and_then(|a| a.parse().ok())
        .unwrap_or(0)
}

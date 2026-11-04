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

/// Its plant's facts, by block name; panics on a block with no row (add it to `vitals::plants`).
pub fn plant(block: &str) -> &'static Plant {
    plants::of(block).unwrap_or_else(|| panic!("no plant for {block} in vitals::plants::PLANTS"))
}

/// The engine's mean seconds between growth steps for a plant of `steps` stages: from its real
/// days by the growing pace, or `game` for the plants that keep the game's own time.
pub fn step_secs(block: &str, steps: u32, game: f32) -> f32 {
    plants::step_secs(plant(block), steps).unwrap_or(game)
}

/// A growth step of the plant at `e`: `step` with the chance of its rate there; `last` says
/// whether this step would be its last stage (the ripe fruit), which waits for its season.
pub fn grow(e: &mut BlockEvent, block: &str, last: bool, step: impl FnOnce(&mut BlockEvent)) {
    // Grown (at its last age): nothing to do, so nothing to read. A ripe crop or bush keeps its
    // fruit until it's picked or eaten, which sets it back and lets it grow again.
    if let Some(age) = state_prop(e.state(), "age").and_then(|a| a.parse::<u32>().ok())
        && state_with(e.state(), "age", &(age + 1).to_string()).is_none()
    {
        return;
    }
    let p = plant(block);
    let w = e.world();
    if p.frost > plants::HARDY {
        let at = e.pos().map(|v| v as f64 + 0.5);
        // The still air: wind chill is what a body loses, a plant doesn't fall below the air
        // (Research round 3, B5).
        let air = base_api::air_at(&w, [at[0], at[1] - 0.5, at[2]], 0.5, false, 3, |_| {}).still;
        if plants::frosted(p, air) {
            frost(e, block);
            return;
        }
    }
    let ground = base_api::ground_at(&w, e.pos());
    if last && !plants::ripens_in(p, ground.season) {
        return;
    }
    let rate = plants::rate(p, ground.mean_air, if p.water == plants::Water::Wet { 1.0 } else { ground.soil });
    // On farmland, as fast as its fertility lets (`vitals::soil`).
    let pos = e.pos();
    let rate = rate * crate::farming::fertility(&w, [pos[0], pos[1] - 1, pos[2]]).map_or(1.0, vitals::soil::growth);
    if (w.random() >> 11) as f32 / (1u64 << 53) as f32 >= rate {
        return;
    }
    step(e);
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
        // The flowers and fruit die; the bush lives.
        "base:berry_bush" if age(e) > 1 => {
            w.set_prop(pos, "age", "1");
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

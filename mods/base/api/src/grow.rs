//! Plants grow by the place (`notes/ECOLOGY.md`, part 3), for any mod: every growth step asks the
//! plant's rate (`vitals::plants`) from the ground where it stands (`ground_at`: the day's mean
//! air and the soil's water from the last days' rain) and goes ahead with that chance; frost hurts
//! the tender ones. The mod whose block it is decides what a step and a frost do
//! (`mods/base/src/growth.rs` for base's).

use vitals::plants::{self, Plant};
use weft_sdk::*;

static FARMLAND: Block = Block::named("base:farmland");

/// What a growth step comes to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    /// Nothing this time: grown, out of season, or the chance didn't come.
    Rest,
    /// A frost where it stands, colder than it takes (`Plant::frost`).
    Frost,
    /// It grows a stage.
    Grow,
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

/// Farmland's fertility at `pos` (`None`: not farmland): `vitals::soil::MAX` less its `worn`.
pub fn fertility(w: &World, pos: [i32; 3]) -> Option<u8> {
    let state = w.get(pos);
    if !FARMLAND.is(state) {
        return None;
    }
    let worn = state_prop(state, "worn")
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);
    Some(vitals::soil::MAX.saturating_sub(worn))
}

/// A growth step of the plant `block` at `e`: whether it grows, with the chance of its rate there;
/// `last` says whether this step would be its last stage (the ripe fruit), which waits for its
/// season. A plant at its last age rests: nothing to do, so nothing to read (a ripe crop or bush
/// keeps its fruit until it's picked or eaten, which sets it back and lets it grow again).
pub fn step(e: &BlockEvent, block: &str, last: bool) -> Step {
    if let Some(age) = state_prop(e.state(), "age").and_then(|a| a.parse::<u32>().ok())
        && state_with(e.state(), "age", &(age + 1).to_string()).is_none()
    {
        return Step::Rest;
    }
    chance(e, block, last)
}

/// A growth step of a ripe plant that bears beside itself (a pumpkin or melon stem setting its
/// fruit): as `step` for its last stage, in its ripening season, though it has no stage left.
pub fn bear(e: &BlockEvent, block: &str) -> Step {
    chance(e, block, true)
}

/// `step` past the last-age rest: frost, the ripening season, then its rate there.
fn chance(e: &BlockEvent, block: &str, last: bool) -> Step {
    let p = plant(block);
    let w = e.world();
    if p.frost > plants::HARDY {
        let at = e.pos().map(|v| v as f64 + 0.5);
        // The still air: wind chill is what a body loses, a plant does not fall below the air (Research round 3, B5).
        let air = crate::air_at(&w, [at[0], at[1] - 0.5, at[2]], 0.5, false, 3, |_| {}).still;
        if plants::frosted(p, air) {
            return Step::Frost;
        }
    }
    let ground = crate::ground_at(&w, e.pos());
    if last && !plants::ripens_in(p, ground.season) {
        return Step::Rest;
    }
    let rate = plants::rate(p, ground.mean_air, if p.water == plants::Water::Wet { 1.0 } else { ground.soil });
    // On farmland, as fast as its fertility lets (`vitals::soil`).
    let pos = e.pos();
    let rate = rate * fertility(&w, [pos[0], pos[1] - 1, pos[2]]).map_or(1.0, vitals::soil::growth);
    if (w.random() >> 11) as f32 / (1u64 << 53) as f32 >= rate { Step::Rest } else { Step::Grow }
}

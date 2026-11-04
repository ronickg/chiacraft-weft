//! Snow lying as blocks (`notes/SYSTEMS.md`, "Snow lying"): the weather's snow cover per biome
//! (`vitals::air::Place::snow_lying`, kept by `weather::snow_in`) lies as `base:snow_layer`, an
//! eighth of a block a layer, on open ground near players, and melts back. Cheaply: no column
//! is scanned every tick. Each tick a few columns at random near each player move a layer toward
//! the cover (`air::SNOW_SAMPLES`, `air::SNOW_REACH`), and each layer block steps down on its own
//! while there is less snow than it (`air::SNOW_MELT_SECS`, a growth step, so land away from
//! players melts too and catches up when it loads). Far land shows the cover in the shader
//! pack's dusting (`snow_on`).

use vitals::air;
use weft_sdk::*;

use crate::weather;

pub static SNOW_LAYER: Block = Block::named("snow_layer");
static LEAVES: Tag = Tag::named("leaves");

pub fn register(r: &mut Registry) {
    r.block(&SNOW_LAYER)
        .prop("layers", 1..=8)
        .states(|p, s| {
            let n: f32 = p.get("layers").parse().unwrap_or(1.0);
            let s = s.model_box([0.0, 0.0, 0.0], [16.0, 2.0 * n, 16.0]);
            // As in vanilla: one layer is walked through and built over, more hold a foot an
            // eighth lower than they look.
            if n <= 1.0 { s.not_solid().replaceable() } else { s.collision([0.0, 0.0, 0.0], [16.0, 2.0 * (n - 1.0), 16.0]) }
        })
        .texture("snow")
        .sound_group("snow")
        .hardness(0.1)
        .mineable("shovel")
        .drops_nothing()
        .no_item()
        .growth(air::SNOW_MELT_SECS, 0)
        .on_grow(|e| {
            let w = e.world();
            let pos = e.pos();
            let Some(snow) = weather::snow_in(w.biome(pos)) else { return };
            let have = layers(e.state());
            let want = air::snow_layers(snow);
            if want < have {
                w.set(pos, with_layers(have - 1));
            }
        });
    r.world_tick(tick);
}

/// How many layers `state` (a snow layer's) has.
fn layers(state: u32) -> u8 {
    state_prop(state, "layers")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

/// A snow layer block of `n` layers, air at none.
fn with_layers(n: u8) -> u32 {
    if n == 0 { 0 } else { SNOW_LAYER.with(&[("layers", &n.min(8).to_string())]) }
}

fn tick(w: &World) {
    if w.dimension() != crate::OVERWORLD.id() {
        return;
    }
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let [px, py, pz] = st.pos.map(|v| v.floor() as i32);
        let side = 2 * air::SNOW_REACH as u32 + 1;
        for _ in 0..air::SNOW_SAMPLES {
            let x = px - air::SNOW_REACH + w.random_below(side) as i32;
            let z = pz - air::SNOW_REACH + w.random_below(side) as i32;
            column(w, x, py, z);
        }
    }
}

/// Moves the snow in column `x, z` (looked down from above `y`) a layer toward the cover there.
fn column(w: &World, x: i32, y: i32, z: i32) {
    // The column's top: the first block down from above the player (unloaded land: none).
    let mut top = None;
    for dy in (-air::SNOW_SCAN..=air::SNOW_SCAN / 2).rev() {
        let pos = [x, y + dy, z];
        match w.get_loaded(pos) {
            None => return,
            Some(0) => continue,
            Some(state) => {
                top = Some((pos, state));
                break;
            }
        }
    }
    let Some((pos, state)) = top else { return };
    let above = [x, pos[1] + 1, z];
    let Some(snow) = weather::snow_in(w.biome(above)) else { return };
    let want = air::snow_layers(snow);
    if SNOW_LAYER.is(state) {
        let have = layers(state);
        if want != have {
            w.set(pos, with_layers(if want > have { have + 1 } else { have - 1 }));
        }
        return;
    }
    if want == 0 {
        return;
    }
    let flags = state_flags(state);
    let ground = |state: u32| state_flags(state) & abi::block_flags::OPAQUE != 0 || LEAVES.on_block(state);
    if ground(state) {
        // Open sky over it: snow falls there.
        if w.sky_light(above) >= 15 {
            w.set(above, with_layers(1));
        }
    } else if want >= air::SNOW_BURIES
        && flags & abi::block_flags::REPLACEABLE != 0
        && flags & abi::block_flags::LIQUID == 0
        && w.sky_light(pos) >= 15
        && w.get_loaded([x, pos[1] - 1, z]).is_some_and(ground)
    {
        // Grass and small plants under a deep enough cover are buried.
        w.set(pos, with_layers(1));
    }
}

//! Snow lying as blocks (`notes/SYSTEMS.md`, "Snow lying"): the weather's snow cover per biome
//! (`vitals::air::Place::snow_lying`, kept by `weather::snow_in`) lies as `base:snow_layer`, an
//! eighth of a block a layer, on open ground near players, and melts back. Cheaply: no column
//! is scanned every tick. Each tick a few columns at random near each player move a layer toward
//! the cover (`air::SNOW_SAMPLES`, `air::SNOW_REACH`), and each layer block steps down on its own
//! while there is less snow than it and the air is above freezing (`air::SNOW_MELT_SECS`, `air::melt_chance`: a growth step, so land away from
//! players melts too and catches up when it loads). Far land shows the cover in the shader
//! pack's dusting (`snow_on`). Still water freezes the same way (`air::freeze_chance`): its open top
//! turns to ice in the cold, and ice on water thaws in the warmth. The cover is blended over the biomes round a column (`cover_at`).

use vitals::air;
use weft_sdk::*;

use crate::blocks::{ICE, LAKE_ICE, WATER};
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
            let Some(snow) = cover_at(&w, pos) else { return };
            let have = layers(e.state());
            let want = air::snow_layers(snow);
            if want < have && melts(&w, pos) {
                w.set(pos, with_layers(have - 1));
            }
        });
    r.world_tick(tick);
}

/// Whether a visit freezes the open water at `pos`: by the cold of the air there (`air::freeze_chance`).
fn freezes(w: &World, pos: [i32; 3]) -> bool {
    let here = base_api::air_at(w, [pos[0] as f64 + 0.5, pos[1] as f64, pos[2] as f64 + 0.5], 0.2, false, 0, |_| {});
    (w.random_below(1 << 16) as f32) < air::freeze_chance(here.celsius) * (1 << 16) as f32
}

/// A growth step of water's skin of ice (`LAKE_ICE`, `air::ICE_THAW_SECS`): it thaws in the
/// warmth, also away from players. Over water or over the ground of a shallow edge alike (the first
/// look's last skins, over a river's one-deep edges, never thawed while it asked for water under).
pub fn ice_step(e: &mut BlockEvent) {
    let w = e.world();
    let pos = e.pos();
    if melts(&w, pos) {
        w.set(pos, WATER.id());
    }
}

/// Whether a melt step takes a layer of the snow at `pos`: by the warmth of the air there
/// (`base_api::air_at`: the hour, sun or shade, the season, the height), `air::melt_chance`.
fn melts(w: &World, pos: [i32; 3]) -> bool {
    let here = base_api::air_at(w, [pos[0] as f64 + 0.5, pos[1] as f64, pos[2] as f64 + 0.5], 0.2, false, 0, |_| {});
    (w.random_below(1 << 16) as f32) < air::melt_chance(here.celsius) * (1 << 16) as f32
}

/// The snow cover at `pos`, blended over the biomes round it (`vitals::air::AIR_BLEND`), so the
/// layers thin out across a border instead of stopping at it; `None` before the weather's first
/// look.
fn cover_at(w: &World, pos: [i32; 3]) -> Option<f32> {
    let mid = weather::snow_in(w.biome(pos))?;
    let ring = air::ring(pos[0], pos[2], air::AIR_BLEND / 2);
    Some(air::blend(ring.map(|(x, z)| weather::snow_in(w.biome([x, pos[1], z])).unwrap_or(mid))))
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
    // Open still water freezes in the cold; a skin of ice on water thaws in the warmth.
    if state == WATER.id() {
        if w.sky_light(above) >= 15 && freezes(w, pos) {
            w.set(pos, LAKE_ICE.id());
        }
        return;
    }
    if LAKE_ICE.is(state) && melts(w, pos) {
        w.set(pos, WATER.id());
        return;
    }
    let Some(snow) = cover_at(w, above) else { return };
    let want = air::snow_layers(snow);
    if SNOW_LAYER.is(state) {
        let have = layers(state);
        if want > have {
            w.set(pos, with_layers(have + 1));
        } else if want < have && melts(w, pos) {
            w.set(pos, with_layers(have - 1));
        }
        return;
    }
    if want == 0 {
        return;
    }
    let flags = state_flags(state);
    // Snow lies on the ground, on leaves and on ice.
    let ground =
        |state: u32| state_flags(state) & abi::block_flags::OPAQUE != 0 || LEAVES.on_block(state) || ICE.is(state) || LAKE_ICE.is(state);
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

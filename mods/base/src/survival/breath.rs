//! Breath under water (the oxygen stat's, later): it runs out in fifteen seconds, then drowning
//! hurts; out of the water it comes back. Only water takes it, whatever else burns or drags.

use base_api::Vitals;
use weft_sdk::*;

pub const MAX_AIR: f32 = 300.0;
static WATER_LIKE: Tag = Tag::named("water");

/// Whether the eyes are in water.
pub fn underwater(w: &World, st: &EntityState) -> bool {
    let eye = st.eye();
    // In a block standing in water (kelp, a waterlogged slab) too: any part of the cell.
    WATER_LIKE.at(w, block_pos_of(eye)) && w.in_liquid(eye)
}

/// A tick's breath: used under water (`drowning`), else what was used comes back.
pub fn breathe(w: &World, id: u64, v: &mut Vitals, drowning: bool) {
    if drowning {
        v.air -= 1.0;
        if v.air <= -20.0 {
            v.air = 0.0;
            w.damage(id, Damage::new(2.0, damage_cause::DROWN));
        }
    } else if v.air < MAX_AIR {
        v.air = (v.air + 4.0).min(MAX_AIR);
    }
}

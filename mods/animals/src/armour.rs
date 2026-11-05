//! Animals' armour (`animals_core::danger`; Ronald, 11-07: health, damage and armour "from size
//! class, role and level"): a step of the damage chain, `armour / (armour + 5 × hit)` at most 80 %
//! (`vitals::combat`), so a light blow bounces off a gastonia's plates and a heavy one goes
//! through. What bypasses armour (fire, falls, starving) bypasses it here too.

use weft_sdk::*;

use crate::animal::{ANIMAL, armour};

pub fn init(r: &mut Registry) {
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        if DamageCause::flags_of(q.cause) & damage_flags::BYPASSES_ARMOR != 0 {
            return;
        }
        let Some(a) = m.world().component(q.entity, &ANIMAL) else { return };
        let cut = vitals::combat::armour_cut(armour(&a), m.value as f32);
        m.value *= 1.0 - cut as f64;
    });
}

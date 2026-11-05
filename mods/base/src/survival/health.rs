//! Health: its max follows the stomach (protein reaches the health stat, a `base:fed` modifier on
//! the engine's 20 points, a point 5 of the vitals'), it comes back a second at a time from
//! protein and greens, and freezing, heatstroke, poison and bleeding take a share of it a second
//! (a gentle world stops them at a tenth; a death is put down to the worst); heatstroke takes
//! stamina too.

use base_api::{HEALTH_SCALE, Vitals};
use vitals::difficulty::Difficulty;
use vitals::effects::Modifiers;
use vitals::food::NUTRIENTS;
use vitals::stamina;
use weft_sdk::*;

/// The engine's max health a player has before food (the base game's 20).
const ENGINE_HEALTH: f32 = 20.0;

/// A tick's health: the max, healing and losses (`pulse`: once a second).
#[allow(clippy::too_many_arguments)]
pub fn tick(
    w: &World,
    id: u64,
    st: &mut EntityState,
    v: &mut Vitals,
    mods: &Modifiers,
    full: &[f32; NUTRIENTS],
    (max_health, max_stamina): (f32, f32),
    d: &Difficulty,
    pulse: bool,
) {
    let engine_max = max_health / HEALTH_SCALE;
    if (v.max_health - max_health).abs() > 0.01 {
        v.max_health = max_health;
        w.attribute_add_modifier(id, &attribute::MAX_HEALTH, "base:fed", attr_op::ADD, (engine_max - ENGINE_HEALTH) as f64, 0);
        if st.health > engine_max {
            st.health = engine_max;
            w.entity_edit(id).health(st.health);
        }
    }
    if !pulse {
        return;
    }
    if st.health > 0.0 && st.health < engine_max {
        let back = stamina::health_regen(max_health, full, mods) / HEALTH_SCALE;
        st.health = (st.health + back).min(engine_max);
        w.entity_edit(id).health(st.health);
    }
    if mods.health_loss > 0.0 && st.health > 0.0 {
        let floor = if d.deadly { 0.0 } else { engine_max * 0.1 };
        let loss = (mods.health_loss * max_health * d.harm / HEALTH_SCALE).min((st.health - floor).max(0.0));
        if loss > 0.0 {
            let worst = v.effects().max_by(|a, b| {
                a.modifiers()
                    .health_loss
                    .total_cmp(&b.modifiers().health_loss)
            });
            w.damage(id, loss, 0, worst.map_or(damage_causes::GENERIC.id(), super::body::cause_of));
        }
    }
    if mods.stamina_loss > 0.0 {
        v.stamina = (v.stamina - mods.stamina_loss * max_stamina * d.harm).max(0.0);
    }
}

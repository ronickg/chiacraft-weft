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

/// The `base:fed` modifier base last set on a player's max health, engine points (not saved: 0 till
/// it is set this session; till then nothing counts over base's own max).
static FED_SET: Component<f32> = Component::named("base:fed_set");

pub fn register(r: &mut Registry) {
    r.entity_component(&FED_SET, 0.0f32, 0);
    r.edit(&PLAYER).attach(&FED_SET);
}

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
    // What other modifiers add to the engine's max over base's own (the health boost effect's +4, a
    // test's): the engine's max less what base set last. Clamped to base's alone, a boosted player's
    // health was cut back at every change of the fed max, and `/tk heal` filled it past the HUD's max
    // ("1585 / 90", cov_bosses' Drenkaun shot).
    let fed_set = w.component(id, &FED_SET).unwrap_or(0.0);
    let others = if fed_set != 0.0 { (st.max_health - ENGINE_HEALTH - fed_set).max(0.0) } else { 0.0 };
    let engine_max = max_health / HEALTH_SCALE + others;
    if (v.max_health - max_health).abs() > 0.01 || fed_set == 0.0 {
        v.max_health = max_health;
        let fed = max_health / HEALTH_SCALE - ENGINE_HEALTH;
        w.attribute_add_modifier(id, "base:fed", &Modifier::add(&attribute::MAX_HEALTH, fed as f64), 0);
        // Never 0, so that it reads as set (a fed max of exactly the engine's 20).
        if let Some(f) = w.component_mut(id, &FED_SET) {
            *f = if fed == 0.0 { f32::MIN_POSITIVE } else { fed };
        }
        if st.health > engine_max {
            st.health = engine_max;
            w.entity_modify(id).health(st.health);
        }
    }
    if !pulse {
        return;
    }
    if st.health > 0.0 && st.health < engine_max {
        let back = stamina::health_regen(max_health, full, mods) / HEALTH_SCALE;
        st.health = (st.health + back).min(engine_max);
        w.entity_modify(id).health(st.health);
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
            w.damage(id, Damage::new(loss, worst.map_or(damage_causes::GENERIC.id(), super::body::cause_of)));
        }
    }
    if mods.stamina_loss > 0.0 {
        v.stamina = (v.stamina - mods.stamina_loss * max_stamina * d.harm).max(0.0);
    }
}

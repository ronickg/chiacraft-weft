//! What gets into the body (`base_api::{poison, bleed, sedate}`, for any mod): poison and
//! bleeding take health while they last (`health`), a heavy blow opens a wound that bleeds, a
//! bandage stops it; torpor from darts wears off, and past the torpor stat one is out cold until
//! it falls under half.

use base_api::{HEALTH_SCALE, VITALS, Vitals};
use vitals::effects::Effect;
use vitals::stats::{STATS, Stat};
use weft_sdk::*;

use super::{DT, set};

/// Torpor that wears off a second, as a share of the torpor stat (out at the stat, awake again
/// under half: about half a minute; proposed).
const TORPOR_DECAY: f32 = 0.03;
/// A blow that takes this share of max health or more opens a wound that bleeds this long.
const WOUND: f32 = 0.2;
const BLEED_SECS: f32 = 20.0;
/// Health (the vitals' 100 a person) a bandage gives back as it stops the bleeding.
const BANDAGE_HEALS: f32 = 10.0;
/// What a bandage heals, for its tooltip.
static HEALS: Component<u8> = Component::named("base:heals");

pub fn register(r: &mut Registry) {
    for (cause, key) in [(&FREEZE, "death.freeze"), (&HEAT, "death.heat"), (&POISON, "death.poison"), (&BLEED, "death.bleed")] {
        r.damage_cause(cause).bypasses_armor().death_message(key);
    }
    // A heavy blow on a player opens a wound that bleeds.
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| e.cause == damage_cause::ATTACK) {
            // A blow that killed leaves nothing to bleed: it would carry over the respawn (the
            // death handler clears bleeding, and this event can come after it).
            if w.entity(e.entity).is_none_or(|st| st.health <= 0.0) {
                continue;
            }
            let heavy = w
                .component(e.entity, &VITALS)
                .is_some_and(|v| e.amount * HEALTH_SCALE >= v.max_health * WOUND);
            if heavy {
                base_api::bleed(&w, e.entity, BLEED_SECS);
            }
        }
    });
    // A bandage (from wool): stops the bleeding and gives a little health back.
    r.item(Item::generated("bandage".into()))
        .set(&HEALS, BANDAGE_HEALS as u8)
        .use_ticks(40)
        .on_use(bandage)
        .tab(&crate::tabs::COMBAT);
    r.shapeless(&base_api::CRAFTING, "bandage", 3, &["wool"], 0);
}

/// A tick of poison, bleeding and torpor.
pub fn tick(v: &mut Vitals, stats: &[f32; STATS]) {
    v.poison = (v.poison - DT).max(0.0);
    v.bleed = (v.bleed - DT).max(0.0);
    set(v, Effect::Poisoned, v.poison > 0.0);
    set(v, Effect::Bleeding, v.bleed > 0.0);
    let torpor = stats[Stat::Torpor.index()];
    v.torpor = (v.torpor - torpor * TORPOR_DECAY * DT).max(0.0);
    if v.out == 0 && v.torpor >= torpor {
        v.out = 1;
    } else if v.out != 0 && v.torpor < torpor * 0.5 {
        v.out = 0;
    }
    set(v, Effect::Sedated, v.out != 0 || v.torpor > torpor * 0.25);
}

/// Binds a wound: only when bleeding or hurt.
fn bandage(u: &mut ItemUse) -> bool {
    let w = u.world();
    let player = u.player();
    let Some(mut st) = w.entity(player) else { return false };
    let Some(v) = w.component_mut(player, &VITALS) else { return false };
    let max = v.max_health / HEALTH_SCALE;
    if v.bleed <= 0.0 && st.health >= max {
        return false;
    }
    v.bleed = 0.0;
    st.health = (st.health + BANDAGE_HEALS / HEALTH_SCALE).min(max);
    w.entity_edit(player).health(st.health);
    if !w.creative(player) {
        u.set_stack(u.stack().shrink(1));
    }
    true
}

// The vitals' own causes of death: none of them armor softens.
pub static FREEZE: DamageCause = DamageCause::named("freeze");
pub static HEAT: DamageCause = DamageCause::named("heat");
pub static POISON: DamageCause = DamageCause::named("poison");
pub static BLEED: DamageCause = DamageCause::named("bleed");

/// What a death from an effect's loss of health is put down to.
pub fn cause_of(e: Effect) -> u32 {
    match e {
        Effect::Heatstroke => HEAT.id(),
        Effect::Poisoned => POISON.id(),
        Effect::Bleeding => BLEED.id(),
        _ => FREEZE.id(),
    }
}

//! What a hit does to an animal (`animals_core::danger`, `vitals::combat`; Ronald: health, damage
//! and armour "from size class, role and level", and the fight bands and floors "as proposed"),
//! one step of the damage chain, in order:
//!
//! 1. **The gear cap.** A player's weapon counts at most its size's cap in item levels over the
//!    animal (`vitals::combat::cap_share`; `base_api::ITEM_LEVEL` on the held stack): an iron sword
//!    hits a Stone Age utahraptor as a sword twenty levels over it, not forty.
//! 2. **Armour**: `armour / (armour + 5 × hit)` at most 80 %, so a light blow bounces off a
//!    gastonia's plates and a heavy one goes through. What bypasses armour (fire, falls, starving)
//!    bypasses it here too.
//! 3. **The rate floor.** A large, huge or titan animal loses at most its health over its floor
//!    a second (10, 45 and 120 s), from everything together (`vitals::combat::floor_take`). What
//!    spills over is kept as its stagger, for knockdowns and part breaks to come.

use weft_sdk::*;

use animals_core::danger;

use crate::animal::{ANIMAL, armour};

/// The rate floor's bucket, its last game tick, and the stagger spilled over it (not saved: a
/// loaded animal starts with a full bucket).
pub static FLOOR: Component<[f64; 3]> = Component::named("animal_floor");

pub fn init(r: &mut Registry) {
    r.entity_component(&FLOOR, [f64::NAN, 0.0, 0.0], 0);
    for s in crate::species::SPECIES.iter() {
        r.attach(s.kind, &FLOOR);
    }
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        let w = m.world();
        let Some(a) = w.component(q.entity, &ANIMAL) else { return };
        let Some(s) = a.species() else { return };
        let size = danger::size(s);
        let level = danger::danger(s.name).map_or(0.0, |d| d.area_level(a.level().min(danger::LEVELS.1)));
        let raw = m.value as f32;
        let by_player = q.cause == damage_cause::ATTACK && w.players_iter().any(|p| p == q.other);
        // 1. A player's weapon, capped by the animal's size.
        let mut hit = raw * player_cap(&w, q.other, q.cause, size, level);
        let capped = hit;
        // 2. Armour.
        if DamageCause::flags_of(q.cause) & damage_flags::BYPASSES_ARMOR == 0 {
            hit *= 1.0 - vitals::combat::armour_cut(armour(&a), hit);
        }
        // 3. The floor (not for what ends a life outright: the void, `/kill`).
        if q.cause != damage_cause::VOID
            && let Some(st) = w.entity(q.entity)
        {
            let (took, _, stagger) = floor(&w, q.entity, size, st.max_health, hit);
            // One line per player hit, for the fight checks (`scripts/floors.txt`): what each step
            // left of it, and the health it leaves.
            if by_player {
                weft_sdk::info!(
                    "floor: {} {} ({size:?}, level {level:.0}) hit {raw:.2}, capped {capped:.2}, armour {hit:.2}, took {took:.2}, stagger {stagger:.1}, health {:.1}/{:.1}",
                    s.name,
                    q.entity,
                    (st.health - took).max(0.0),
                    st.max_health,
                );
            }
            hit = took;
        }
        m.value = hit as f64;
    });
}

/// The share of a hit by `attacker` (with `cause`) a creature of `size` at area level `level`
/// lets count: a player's held weapon at most its size's cap in item levels over it
/// (`vitals::combat::cap_share`), everything else whole.
pub fn player_cap(w: &World, attacker: u64, cause: u32, size: vitals::combat::Size, level: f32) -> f32 {
    if cause != damage_cause::ATTACK || !w.players_iter().any(|p| p == attacker) {
        return 1.0;
    }
    let held = w
        .inventory(attacker)
        .group("hotbar")
        .get(w.selected_slot(attacker));
    base_api::ITEM_LEVEL
        .on_stack(&held)
        .map_or(1.0, |lv| vitals::combat::cap_share(size, lv, level))
}

/// The rate floor on `entity` (it carries `FLOOR`): of `hit`, what it takes now, what spilled
/// over, and all its stagger so far (`vitals::combat::floor_take`, refilled since its last hit).
pub fn floor(w: &World, entity: u64, size: vitals::combat::Size, max_health: f32, hit: f32) -> (f32, f32, f64) {
    let now = w.game_time();
    let [budget, last, stagger] = w
        .component(entity, &FLOOR)
        .unwrap_or([f64::NAN, 0.0, 0.0]);
    let budget = if budget.is_nan() { max_health / size.band().floor.unwrap_or(1.0) } else { budget as f32 };
    let secs = now.saturating_sub(last as u64) as f32 / vitals::TICKS as f32;
    let (took, left, over) = vitals::combat::floor_take(size, max_health, budget, secs, hit);
    let stagger = stagger + over as f64;
    if size.band().floor.is_some() {
        w.set_component(entity, &FLOOR, [left as f64, now as f64, stagger]);
    }
    (took, over, stagger)
}

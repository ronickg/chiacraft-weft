//! What a hit does to an animal (`animals_core::danger`, `vitals::combat`; Ronald: health, damage
//! and armour "from size class, role and level", and the fight bands and floors "as proposed"),
//! one step of the damage chain, in order:
//!
//! 1. **The gear cap.** A player's weapon counts at most its size's cap in item levels over the
//!    animal (`vitals::combat::cap_share`; `base_api::ITEM_LEVEL` on the held stack): an iron sword
//!    hits a Stone Age utahraptor as a sword twenty levels over it, not forty. A tame's bite counts
//!    at most the cap over it too, by its wild level, and a player's over-tier armour against it
//!    (`base_api::chains::GEAR_COUNTS`, answered here and read by base's armour step).
//! 2. **Armour**: `armour / (armour + 5 × hit)` at most 80 %, so a light blow bounces off a
//!    gastonia's plates and a heavy one goes through. What bypasses armour (fire, falls, starving)
//!    bypasses it here too.
//! 3. **The rate floor.** A large, huge or titan animal loses at most its health over its floor
//!    a second (10, 45 and 120 s), from everything together (`vitals::combat::floor_take`). What
//!    spills over, and half of what it took, is its stagger (`vitals::combat::stagger_gain`).
//! 4. **Stagger** (Ronald 11-22, option B, as the bosses have it): at 20 % and 40 % of its health
//!    a break drops one more of its own loot; every 25 % it is knocked down for 3 s (lying as it
//!    rests, its goals held, hits on it weak-point hits), and stands at least 12 s before the next.

use weft_sdk::*;

use animals_core::danger;

use crate::animal::{ANIMAL, armour};

/// The rate floor's bucket, its last game tick, and its stagger (not saved: a loaded animal
/// starts with a full bucket).
pub static FLOOR: Component<[f64; 3]> = Component::named("animal_floor");
/// Its stagger's effects: stagger toward the next knockdown, the tick it stands up (0: up), the
/// tick it can be knocked down again, and the breaks it has had (not saved).
pub static STAGGER: Component<[f64; 4]> = Component::named("animal_stagger");

pub fn init(r: &mut Registry) {
    r.entity_component(&FLOOR, [f64::NAN, 0.0, 0.0], 0);
    r.entity_component(&STAGGER, [0.0; 4], 0);
    for s in crate::species::SPECIES.iter() {
        r.edit(s.kind).attach(&FLOOR);
        r.edit(s.kind).attach(&STAGGER);
    }
    // Over-tier armour against an animal counts at most its size's cap, as a weapon does
    // (`base_api::chains::GEAR_COUNTS`, read by base's armour step).
    r.modify(&base_api::chains::GEAR_COUNTS, &phase::MULTIPLY, |m| {
        let q = *m.query();
        if let Some((size, level)) = foe(&m.world(), q.other) {
            m.value *= vitals::combat::cap_share(size, q.amount as f32, level) as f64;
        }
    });
    r.modify(&chain::DAMAGE, &phase::MULTIPLY, |m| {
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
        // Knocked down, it's hit where it's weak.
        if down(&w, q.entity) {
            hit *= vitals::combat::WEAK_POINT;
        }
        // 2. Armour.
        if !comps::damage_cause::BYPASSES_ARMOR.on_id(q.cause) {
            hit *= 1.0 - vitals::combat::armour_cut(armour(&a), hit);
        }
        // 3. The floor (not for what ends a life outright: the void, `/kill`).
        if q.cause != damage_cause::VOID
            && let Some(st) = w.entity(q.entity)
        {
            let (took, over, stagger) = floor(&w, q.entity, size, st.max_health, hit);
            // 4. What the stagger does.
            if vitals::combat::staggers(size) {
                staggered(&w, q.entity, s, &st, vitals::combat::stagger_gain(took, over), stagger as f32);
            }
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
/// lets count (`vitals::combat::cap_share`): a player's held weapon at most its size's cap in item
/// levels over it, a tame's bite at most the cap in area levels over it (its wild level's: tamed
/// points, imprint and bond are its grade, under their own ceiling), everything else whole.
pub fn player_cap(w: &World, attacker: u64, cause: u32, size: vitals::combat::Size, level: f32) -> f32 {
    if cause != damage_cause::ATTACK {
        return 1.0;
    }
    if !w.players_iter().any(|p| p == attacker) {
        return tame_level(w, attacker).map_or(1.0, |at| vitals::combat::cap_share(size, at, level));
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
    let [budget, last, stagger] = w.component(entity, &FLOOR).unwrap_or([f64::NAN, 0.0, 0.0]);
    // A fresh one holds a second's worth.
    let budget = if budget.is_nan() {
        vitals::combat::floor_rate(size, max_health).unwrap_or(max_health) * vitals::combat::FLOOR_HELD
    } else {
        budget as f32
    };
    let secs = now.saturating_sub(last as u64) as f32 / vitals::TICKS as f32;
    let (took, left, over) = vitals::combat::floor_take(size, max_health, budget, secs, hit);
    let stagger = stagger + vitals::combat::stagger_gain(took, over) as f64;
    if size.band().floor.is_some() {
        w.set_component(entity, &FLOOR, [left as f64, now as f64, stagger]);
    }
    (took, over, stagger)
}

/// The area level a tame fights at: its wild level's (`danger::Danger::area_level`), its tamed
/// points left out (they're its grade). None for a wild one or anything not an animal.
pub fn tame_level(w: &World, id: u64) -> Option<f32> {
    if !crate::animal::tamed(w, id) {
        return None;
    }
    let a = w.component(id, &ANIMAL)?;
    let wild: u32 = a.wild.iter().map(|&p| p as u32).sum();
    Some(danger::danger(a.species()?.name)?.area_level(wild.min(danger::LEVELS.1)))
}

/// An animal's size class and the area level it fights at (a tame at its wild level's).
pub fn foe(w: &World, id: u64) -> Option<(vitals::combat::Size, f32)> {
    let a = w.component(id, &ANIMAL)?;
    let s = a.species()?;
    let level = danger::danger(s.name)?.area_level(a.level().min(danger::LEVELS.1));
    Some((danger::size(s), tame_level(w, id).unwrap_or(level)))
}

/// Whether `id` lies knocked down by its stagger.
pub fn down(w: &World, id: u64) -> bool {
    w.component(id, &STAGGER)
        .is_some_and(|s| s[1] > w.game_time() as f64)
}

/// `gain` more stagger on `id` (of species `s`), `total` in all: its breaks (each one more of its
/// own loot, rolled from its table) and knockdowns.
fn staggered(w: &World, id: u64, s: &animals_core::species::Species, st: &EntityState, gain: f32, total: f32) {
    let now = w.game_time() as f64;
    let Some(mut c) = w.component(id, &STAGGER) else { return };
    let max = st.max_health;
    let at = [st.pos.x, st.pos.y + st.size.height as f64 * 0.6, st.pos.z];
    // Breaks: one more of what it drops, each.
    let reached = vitals::combat::breaks_at(total, max) as u32;
    while (c[3] as u32) < reached {
        c[3] += 1.0;
        let ctx = LootCtx { by_player: true, ..LootCtx::default() };
        let one = w
            .roll_loot(&EntityKind::named(s.name), &ctx)
            .into_iter()
            .find(|x| !x.is_empty());
        let dropped = one
            .as_ref()
            .map_or_else(|| "nothing".to_string(), |x| weft_sdk::item_name(x.item).to_string());
        if let Some(one) = one {
            w.spawn_item(at, ItemStack::new(one.item, 1));
        }
        for p in w.entities_near(st.pos, 24.0, Some(&PLAYER)) {
            w.subtitle("subtitle.animals.stagger_break", &[])
                .to(p)
                .times(0.2, 1.5, 0.5);
        }
        weft_sdk::info!("stagger: {} {id} break {} at {total:.1}/{max:.1}, dropped {dropped}", s.name, c[3]);
    }
    // Knockdowns: every `KNOCK_AT` of its health, not while down or resting from the last.
    c[0] += gain as f64;
    let knock = (vitals::combat::KNOCK_AT * max) as f64;
    if c[0] >= knock && now >= c[2] && c[1] <= now {
        c[0] = 0.0;
        let ticks = (vitals::combat::KNOCK_SECS * vitals::TICKS as f32) as f64;
        c[1] = now + ticks;
        c[2] = c[1] + (vitals::combat::KNOCK_REST * vitals::TICKS as f32) as f64;
        w.navigate(id, st.pos.to_array(), 0.0);
        for p in w.entities_near(st.pos, 24.0, Some(&PLAYER)) {
            w.subtitle("subtitle.animals.stagger_down", &[])
                .to(p)
                .times(0.2, 1.5, 0.5);
        }
        weft_sdk::info!("stagger: {} {id} knocked down at {total:.1}/{max:.1}", s.name);
    }
    w.set_component(id, &STAGGER, c);
}

/// Its goal while knocked down (first, in `animal::upkeep`): it lies as it rests and does nothing
/// else; standing up, it wakes. True while it is down.
pub fn lie_down(w: &World, id: u64, index: usize, st: &EntityState) -> bool {
    let Some(c) = w.component(id, &STAGGER) else { return false };
    if c[1] == 0.0 {
        return false;
    }
    if c[1] > w.game_time() as f64 {
        crate::sleep(w, id, index);
        if w.navigating(id) {
            w.navigate(id, st.pos.to_array(), 0.0);
        }
        return true;
    }
    if let Some(c) = w.component_mut(id, &STAGGER) {
        c[1] = 0.0;
    }
    crate::wake(w, id, index);
    false
}

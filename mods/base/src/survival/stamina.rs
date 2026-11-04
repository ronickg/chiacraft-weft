//! Stamina (`vitals::stamina`): sprinting, jumping and swimming cost it as one moves, blows that
//! land cost it by what they're struck with, and every block broken by how hard it is; it comes
//! back a moment after spending, from energy and water in the stomach. Run out, no sprinting until
//! some has come back.

use base_api::{VITALS, Vitals};
use vitals::difficulty::Difficulty;
use vitals::effects::{self, Modifiers};
use vitals::food::NUTRIENTS;
use vitals::stamina;
use weft_sdk::*;

use super::DT;

/// Stamina it takes to start sprinting again once run out.
const SPRINT_AGAIN: f32 = 10.0;
static HANDHELD: Tag = Tag::named("handheld");

pub fn register(r: &mut Registry) {
    // Mining and chopping: a swing's worth for every block, by how hard it is.
    r.on(&actions::BREAK_BLOCK, |b| {
        let w = b.world();
        for e in b.iter() {
            let hardness = HARDNESS.on_block(e.state).unwrap_or(1.0).max(0.0);
            spend(&w, e.player, stamina::work(hardness));
        }
    });
    // A blow that lands: by what it was struck with (a weapon or tool, else the fist).
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| w.players().contains(&e.source)) {
            let held = w
                .inventory(e.source)
                .group("hotbar")
                .get(w.selected_slot(e.source));
            spend(&w, e.source, stamina::swing(if HANDHELD.on_item(held.item) { 1.5 } else { 0.5 }));
        }
    });
}

/// Spends stamina on an action (nothing in creative).
fn spend(w: &World, player: u64, amount: f32) {
    if w.creative(player) {
        return;
    }
    let mods = w
        .component(player, &VITALS)
        .map_or(effects::NONE, |v| effects::combine(v.effects()));
    if let Some(v) = w.component_mut(player, &VITALS) {
        v.stamina = (v.stamina - amount * base_api::difficulty().stamina_cost * mods.stamina_cost).max(0.0);
        v.rest_wait = (stamina::REGEN_DELAY * 20.0) as u32;
    }
}

/// A tick's stamina: moving costs, coming back, and no sprinting when run out (or out cold).
#[allow(clippy::too_many_arguments)]
pub fn tick(
    w: &World,
    id: u64,
    st: &EntityState,
    v: &mut Vitals,
    mods: &Modifiers,
    full: &[f32; NUTRIENTS],
    max_stamina: f32,
    d: &Difficulty,
) {
    let mut cost = 0.0;
    if st.sprinting != 0 && st.moved > 0.0 {
        cost += stamina::SPRINT * DT;
    }
    if st.jumped != 0 {
        cost += stamina::JUMP;
    }
    if st.in_liquid != 0 && st.moved > 0.0 {
        cost += stamina::SWIM * DT;
    }
    if cost > 0.0 {
        v.stamina -= cost * d.stamina_cost * mods.stamina_cost;
        v.rest_wait = (stamina::REGEN_DELAY * 20.0) as u32;
    }
    if v.rest_wait > 0 {
        v.rest_wait -= 1;
    } else {
        v.stamina += stamina::stamina_regen(max_stamina, full, mods) * DT;
    }
    v.stamina = v.stamina.clamp(0.0, max_stamina);
    v.max_stamina = max_stamina;
    let tired = v.stamina < 1.0 || (v.stamina < SPRINT_AGAIN && st.sprinting == 0);
    if tired || !mods.can_sprint || v.out != 0 {
        w.attribute_add_modifier(id, &attribute::CAN_SPRINT, "base:tired", attr_op::ADD, -1.0, 2);
    }
}

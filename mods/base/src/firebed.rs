//! The first fire (`notes/FIRST_HOURS.md`; Ronald, 11-05: "Fire bed + bow drill"; the numbers are
//! `vitals::first`'s). A fire bed (three sticks and a fibre for tinder) is put on the ground and lit
//! by holding a drill on it: a bow drill (two sticks and a string) takes 6 s and gives an ember most
//! times, a hand drill (two sticks) takes 10 s and fails more often than not. In the rain with
//! the sky open it gives none. Lit, the bed is a campfire (`cooking.rs`), and the drill wears.
//! Today's campfire (two logs, two sticks) is laid unlit the same way (`campfire_laid`) and lit by
//! the drill too: one rule for fire (Planner, 11-05). Flint and steel and torches come later.
//!
//! The engine finishes a held use with no block aimed at, so where the drill was aimed when the
//! hold began is kept here (`question::USE_ITEM`).

use std::sync::Mutex;

use vitals::first::{self as f, Drill};
use weft_sdk::*;

pub static FIRE_BED: Block = Block::named("fire_bed");
/// Today's campfire (two logs, two sticks) laid and not lit yet.
pub static CAMPFIRE_LAID: Block = Block::named("campfire_laid");

/// What a drill lights: a fire bed or a laid campfire.
fn lightable(state: u32) -> bool {
    FIRE_BED.is(state) || CAMPFIRE_LAID.is(state)
}
static TRY: Sound = Sound::named("block.wood.hit");

/// Where each player's drill is: (player, the fire bed's place).
static DRILLING: Mutex<Vec<(u64, [i32; 3])>> = Mutex::new(Vec::new());

/// How far a player may be from the fire bed when the drill finishes, blocks.
const REACH: f64 = 6.0;

pub fn register(r: &mut Registry) {
    r.block(&FIRE_BED)
        .sound_group("wood")
        .texture("fire_bed")
        .not_solid()
        .hardness(0.3)
        .needs(2, "")
        .model_box([2.0, 0.0, 7.0], [14.0, 1.5, 9.0])
        .model_box([7.0, 1.5, 2.0], [9.0, 3.0, 14.0])
        .model_box([3.0, 3.0, 3.0], [13.0, 4.5, 5.0])
        .tag("fire_beds")
        .tab(&crate::tabs::FUNCTIONAL);
    r.block(&CAMPFIRE_LAID)
        .sound_group("wood")
        .textures("campfire_top", "campfire_side", "campfire_side")
        .not_solid()
        .hardness(1.0)
        .mineable("axe")
        .needs(2, "")
        .model_box([1.0, 0.0, 1.0], [15.0, 5.0, 15.0])
        .tag("fire_beds")
        .tab(&crate::tabs::FUNCTIONAL);
    let mut bed = vec!["plant_fibre"];
    bed.extend(std::iter::repeat_n("#stick", f::FIRE_BED_STICKS as usize));
    r.shapeless(&crate::recipes::CRAFTING, "fire_bed", 1, &bed, 0);
    r.shapeless(&crate::recipes::CRAFTING, f::BOW_DRILL.item, 1, &["#stick", "#stick", "#string"], 0);
    r.shapeless(&crate::recipes::CRAFTING, f::HAND_DRILL.item, 1, &["#stick", "#stick"], 0);
    for d in f::DRILLS {
        r.item(Item::generated(d.item.into()))
            .handheld()
            .max_stack(1)
            .durability(d.uses)
            .use_ticks((d.secs * vitals::TICKS as f32).round() as u32)
            .on_use(move |u| drill(u, d))
            .tab(&crate::tabs::TOOLS);
    }
    // Where a drill is aimed when its hold begins: a fire bed, or the hold doesn't start.
    r.intercept(&question::USE_ITEM, |q| {
        let player = q.entity();
        if player == 0
            || !f::DRILLS
                .iter()
                .any(|d| item_name(q.item()) == format!("base:{}", d.item))
        {
            return Verdict::Abstain;
        }
        let mut drilling = DRILLING.lock().unwrap();
        // The bed itself, or the ground it stands on (small things are easy to aim past).
        let w = q.world();
        let above = [q.pos()[0], q.pos()[1] + 1, q.pos()[2]];
        let bed = if lightable(q.state()) {
            Some(q.pos())
        } else if q.state() != 0 && lightable(w.get(above)) {
            Some(above)
        } else {
            None
        };
        if let Some(at) = bed {
            drilling.retain(|(p, _)| *p != player);
            drilling.push((player, at));
            return Verdict::Abstain;
        }
        // The hold finishing comes back with nothing aimed at: let it through if a bed was.
        if q.state() == 0 && drilling.iter().any(|(p, _)| *p == player) {
            return Verdict::Abstain;
        }
        drilling.retain(|(p, _)| *p != player);
        Verdict::Deny
    });
}

/// A drill's hold done: an ember on the fire bed it was aimed at, or not.
fn drill(u: &mut ItemUse, d: Drill) -> bool {
    let w = u.world();
    let player = u.player();
    let pos = {
        let mut drilling = DRILLING.lock().unwrap();
        let Some(i) = drilling.iter().position(|(p, _)| *p == player) else { return false };
        drilling.remove(i).1
    };
    if !lightable(w.get(pos)) {
        return false;
    }
    let at = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.3, pos[2] as f64 + 0.5];
    let near = w.entity(player).is_some_and(|e| {
        let dd = [e.pos[0] - at[0], e.pos[1] - at[1], e.pos[2] - at[2]];
        dd.iter().map(|x| x * x).sum::<f64>() <= REACH * REACH
    });
    if !near {
        return false;
    }
    // Rain on an open bed: no ember. Otherwise the drill's chance.
    let open = (1..=f::ROOF_REACH).all(|k| w.get([pos[0], pos[1] + k, pos[2]]) == 0);
    let wet = open && w.raining_at(pos);
    let u01 = (w.random() >> 40) as f32 / (1u64 << 24) as f32;
    weft_sdk::info!("firebed: {} at {pos:?}: open {open}, raining {}, roll {u01:.2} of {}", d.item, w.raining_at(pos), d.chance);
    if !wet && u01 < d.chance {
        w.set(pos, crate::cooking::CAMPFIRE.id());
        crate::hints::fire_lit(&w, player);
        crate::advancements::placed(&w, player, crate::cooking::CAMPFIRE.id());
    } else {
        w.play_sound(&TRY, at, 0.6);
    }
    if !w.creative(player) {
        let s = u.stack();
        let max = DURABILITY.on_stack(&s).unwrap_or(d.uses);
        u.set_stack(crate::felling::worn(s, 1, max));
    }
    true
}

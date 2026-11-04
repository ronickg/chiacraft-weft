//! Food (`vitals::food`, `vitals::stomach`): eating puts a portion in the stomach if there's
//! room (`base_api::eat`), it digests, and what it holds sets how much of the stats is reached.
//! Food stacks carry when they were made and age: fresh, stale, spoiled, or hot off the fire.
//! Well fed from three foods; food poisoning from raw, stale or spoiled food.

use base_api::{FOOD, FRESHNESS, HOT, MADE, MEAL, VITALS, Vitals, freshness};
use vitals::effects::Effect;
use vitals::{calendar, food, meals, stomach};
use weft_sdk::*;

use super::{DT, set};

pub fn register(r: &mut Registry) {
    // Food's stamps, kept and saved with each stack (`age`).
    r.stack_component(&MADE, 0);
    r.stack_component(&HOT, 0);
    r.stack_component(&FRESHNESS, 0);
    // No room in the stomach: it doesn't start eating (the other way round it would only find out
    // at the end).
    r.intercept(&question::USE_ITEM, |q| {
        let w = q.world();
        if w.creative(q.entity()) {
            return Verdict::Abstain;
        }
        let full = if FOOD.on_item(q.item()).is_some() {
            !base_api::has_room(&w, q.entity(), q.item())
        } else if q.item() == crate::cooking::STEW.id() {
            // A stew all at once: what it holds is on the stack in hand.
            let held = w.inventory(q.entity()).group("hotbar").get(w.selected_slot(q.entity()));
            held.item == q.item() && !base_api::has_room_for_meal(&w, q.entity(), &held)
        } else {
            false
        };
        if full { Verdict::Deny } else { Verdict::Abstain }
    });
    // Opening a chest (or any block's inventory) brings its food's freshness up to date: it has
    // aged all along from its stamp, the stamp just says so now.
    r.intercept(&question::USE_BLOCK, |q| {
        let w = q.world();
        let be = w.block_entity(q.pos());
        if be != 0 {
            age_inventory(&w, be);
        }
        Verdict::Abstain
    });
    // Fills the stomach with a balanced meal (an apple, a loaf, a steak, over and over) up to that
    // many units; 0 empties it. For testing.
    r.command("feed", "[units:int]", |cmd| {
        let w = cmd.world();
        let Some(v) = w.component_mut(cmd.player(), &VITALS) else { return };
        let units = cmd.int("units").map_or(v.capacity(), |u| u.max(0) as f32);
        let mut s = stomach::Stomach::default();
        let meal = ["base:apple", "base:bread", "base:cooked_beef"].map(|n| food::find(n).unwrap_or(0));
        for &f in meal.iter().cycle().take(stomach::PORTIONS) {
            if s.used() + food::FOODS[f].units() > units || s.eat(f, 1.0, food::HOT_FOR, v.capacity()).is_err() {
                break;
            }
        }
        v.set_stomach(&s);
        cmd.reply("command.base.feed", &[Arg::Int(s.used().round() as i64)]);
    })
    .permission(permission::EVERYONE);
}

/// A tick's digesting at the world's pace; what the stomach holds now.
pub fn digest(v: &mut Vitals, pace: f32) -> stomach::Stomach {
    let mut s = v.stomach();
    s.digest(DT, pace);
    v.set_stomach(&s);
    s
}

/// Well fed, and food poisoning while it lasts.
pub fn effects(v: &mut Vitals, s: &stomach::Stomach, capacity: f32) {
    set(v, Effect::WellFed, s.well_fed(capacity));
    v.sick = (v.sick - DT).max(0.0);
    set(v, Effect::FoodPoisoning, v.sick > 0.0);
}

/// Stamps the food one carries with the day it was made (cooked food fresh off the fire as hot,
/// with the second) and moves its freshness on as it ages: stale, then spoiled (`food::freshness`;
/// a stew by its shortest-keeping ingredient, `meals::keeps`); hot food cools after a while and
/// then stacks with the day's.
pub fn age(w: &World, player: u64) {
    age_inventory(w, player);
}

/// `age` for every food stack in an inventory (a player's, a chest's).
pub fn age_inventory(w: &World, id: u64) {
    let inv = w.inventory(id);
    let now = w.day_time();
    for slot in 0..inv.size() {
        let stack = inv.get(slot);
        if stack.is_empty() || FRESHNESS.on_item(stack.item).is_none() {
            continue;
        }
        let new = aged(stack, now);
        if new != stack {
            inv.set(slot, new);
        }
    }
}

/// A food stack stamped and its freshness moved on, at day time `now`.
fn aged(stack: ItemStack, now: u64) -> ItemStack {
    let today = (now / calendar::DAY_TICKS) as u32 + 1;
    let (keeps, cooked) = if let Some(packed) = MEAL.stack_only(&stack) {
        (meals::keeps(&meals::ingredients(packed)), true)
    } else if let Some(f) = food::find(item_name(stack.item)).map(|f| &food::FOODS[f]) {
        (f.keeps, f.meal == food::Meal::Cooked)
    } else {
        return stack;
    };
    let mut new = stack;
    if MADE.stack_only(&stack).unwrap_or(0) == 0 {
        new = new.with(&MADE, today);
        if cooked {
            new = new.with(&HOT, (now / 20) as u32);
        }
    }
    let (age, hot) = base_api::food_age(&new, now);
    if HOT.stack_only(&new).is_some() && !hot {
        new = new.without(&HOT);
    }
    let state = if hot {
        freshness::HOT
    } else {
        match food::freshness_of(keeps, age, base_api::difficulty().keep) {
            food::Freshness::Fresh => freshness::FRESH,
            food::Freshness::Stale => freshness::STALE,
            food::Freshness::Spoiled => freshness::SPOILED,
        }
    };
    if FRESHNESS.stack_only(&new) != Some(state) {
        new = new.with(&FRESHNESS, state);
    }
    new
}

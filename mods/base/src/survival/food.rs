//! Food (`vitals::food`, `vitals::stomach`): eating puts a portion in the stomach if there's
//! room (`base_api::eat`), it digests, and what it holds sets how much of the stats is reached.
//! Food stacks carry when they were made and age: fresh, stale, spoiled, or hot off the fire.
//! Well fed from three foods; food poisoning from raw, stale or spoiled food.

use base_api::{FOOD, FRESHNESS, HOT, MADE, MEAL, VITALS, Vitals, freshness};
use vitals::effects::Effect;
use vitals::{calendar, food, meals, stomach};
use weft_sdk::*;

use super::{DT, set};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// Server to client: why eating was refused, by its index in `REFUSED` (Tester, 11-04: a full
/// stomach let nothing be eaten on a 4 s hold, and nothing said why).
static REFUSED: Channel<u32> = Channel::named("food_refused");
/// The lines, `food.base.refused.<name>`: no room for it, or every portion's place taken.
const REFUSALS: [&str; 2] = ["full", "too_many"];
const FULL: u32 = 0;
const TOO_MANY: u32 = 1;
/// A hold asks every tick: the line goes out once in this many ticks a player, and shows this long.
const REFUSED_EVERY: u64 = 30;
const REFUSED_SECS: f32 = 2.5;
const REFUSED_FADE: f32 = 0.5;
/// Where it shows, a share of the screen's height down: the bed's messages' line.
const REFUSED_Y: f32 = 0.72;
/// The tick each player was last told (server), and the line showing (client).
static TOLD: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);
static SHOWING: Mutex<Option<(u32, Instant)>> = Mutex::new(None);

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
        let player = q.entity();
        // How many portions it takes, and whether there's room for it.
        let (portions, room) = if FOOD.on_item(q.item()).is_some() {
            (1, base_api::has_room(&w, player, q.item()))
        } else if q.item() == crate::cooking::STEW.id() {
            // A stew all at once: what it holds is on the stack in hand.
            let held = w.inventory(player).group("hotbar").get(w.selected_slot(player));
            if held.item != q.item() {
                return Verdict::Abstain;
            }
            (meals::ingredients(MEAL.stack_only(&held).unwrap_or(0)).len(), base_api::has_room_for_meal(&w, player, &held))
        } else {
            return Verdict::Abstain;
        };
        let free = w
            .component(player, &VITALS)
            .map_or(stomach::PORTIONS, |v| v.stomach().portions.iter().filter(|p| p.food == 0).count());
        let refused = if !room {
            FULL
        } else if free < portions {
            TOO_MANY
        } else {
            return Verdict::Abstain;
        };
        let now = w.time();
        let mut told = TOLD.lock().unwrap();
        let last = told.get_or_insert_with(HashMap::new).entry(player).or_insert(0);
        if *last == 0 || now >= *last + REFUSED_EVERY || now < *last {
            *last = now.max(1);
            w.send(player, &REFUSED, &refused);
        }
        Verdict::Deny
    });
    r.client_packet(&REFUSED, |_, why| *SHOWING.lock().unwrap() = Some((why, Instant::now())));
    r.client_frame(|c| {
        let mut showing = SHOWING.lock().unwrap();
        let Some((why, at)) = *showing else { return };
        let t = at.elapsed().as_secs_f32();
        if t > REFUSED_SECS {
            *showing = None;
            return;
        }
        let alpha = ((REFUSED_SECS - t) / REFUSED_FADE).clamp(0.0, 1.0);
        let key = format!("food.base.refused.{}", REFUSALS[why as usize % REFUSALS.len()]);
        c.screen_text(0.5, REFUSED_Y, &key, &[], 0xffffff00 | (alpha * 255.0) as u32, 1.0);
    });
    // Opening a chest (or any block's inventory) brings its food's freshness up to date: it has
    // aged all along from its stamp, the stamp just says so now.
    r.intercept(&question::USE_BLOCK, |q| {
        let w = q.world();
        let be = w.block_entity(q.pos());
        if be != 0 {
            // Stacks merge only in a chest: a pot takes one of each slot for a stew and a rack
            // dries each slot apart (integration review: merging broke both).
            age_inventory(&w, be, crate::blocks::CHEST.is(q.state()));
        }
        Verdict::Abstain
    });
    // Fills the stomach with a balanced meal (an apple, a loaf, a steak, over and over) up to that
    // many units; 0 empties it. For testing: game masters only, as `/give` (Research round 5: open to
    // everyone it filled stomachs in multiplayer).
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
    });
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

/// Stamps the food one carries with the day it was made (what was cooked came hot off the fire,
/// stamped there) and moves its freshness on as it ages: stale, then spoiled (`food::freshness`;
/// a stew by its shortest-keeping ingredient, `meals::keeps`); hot food cools after a while and
/// then stacks with the day's.
pub fn age(w: &World, player: u64) {
    age_inventory(w, player, true);
}

/// `age` for every food stack in an inventory (a player's, a chest's, a machine's), merging stacks
/// alike (`merge_food`) when `merge`.
pub fn age_inventory(w: &World, id: u64, merge: bool) {
    let inv = w.inventory(id);
    let now = w.day_time();
    let size = inv.size();
    let before: Vec<ItemStack> = (0..size).map(|i| inv.get(i)).collect();
    let mut after = before.clone();
    for stack in after.iter_mut() {
        if !stack.is_empty() && FRESHNESS.on_item(stack.item).is_some() {
            *stack = aged(*stack, now);
        }
    }
    if merge {
        merge_food(&mut after);
    }
    for (i, (b, a)) in before.iter().zip(&after).enumerate() {
        if a != b {
            inv.set(i as u32, *a);
        }
    }
}

/// Food stacks that differ only in their day stamp and are at the same freshness stage stack
/// together into the earlier slot, keeping the older day (TerraFirmaCraft's way: a stack never gets
/// fresher by merging). Picked or crafted food comes in unstamped and was stamped a moment later in
/// a slot of its own, so three picks sat in three slots (Tester, 11-04). Hot food stays apart until
/// it cools.
pub fn merge_food(stacks: &mut [ItemStack]) {
    let alike = |s: &ItemStack| ItemStack { count: 1, ..s.without(&MADE) };
    let mergeable =
        |s: &ItemStack| !s.is_empty() && FRESHNESS.on_item(s.item).is_some() && HOT.stack_only(s).is_none() && MEAL.stack_only(s).is_none();
    for i in 0..stacks.len() {
        if !mergeable(&stacks[i]) {
            continue;
        }
        for j in 0..i {
            let (t, s) = (stacks[j], stacks[i]);
            if !mergeable(&t) || alike(&t) != alike(&s) {
                continue;
            }
            let room = max_stack(t.item).saturating_sub(t.count);
            if room == 0 {
                continue;
            }
            let n = room.min(s.count);
            let made = match (MADE.stack_only(&t), MADE.stack_only(&s)) {
                (Some(a), Some(b)) if a > 0 && b > 0 => a.min(b),
                (Some(a), _) if a > 0 => a,
                (_, Some(b)) => b,
                _ => 0,
            };
            let merged = t.grow(n);
            stacks[j] = if made > 0 { merged.with(&MADE, made) } else { merged };
            stacks[i] = s.shrink(n);
            if stacks[i].is_empty() {
                break;
            }
        }
    }
}

/// A food stack stamped and its freshness moved on, at day time `now`.
fn aged(stack: ItemStack, now: u64) -> ItemStack {
    let today = (now / calendar::DAY_TICKS) as u32 + 1;
    let keeps = if let Some(packed) = MEAL.stack_only(&stack) {
        meals::keeps(&meals::ingredients(packed))
    } else if let Some(f) = food::find(item_name(stack.item)).map(|f| &food::FOODS[f]) {
        f.keeps
    } else {
        return stack;
    };
    let mut new = stack;
    // Stamped with today when first seen; hot only where it was cooked (the furnace, the campfire,
    // the pot stamp it), not here: crafted bread came out hot (integration review).
    if MADE.stack_only(&stack).unwrap_or(0) == 0 {
        new = new.with(&MADE, today);
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

/// Game days a food stack keeps fresh, then stale (a stew by its ingredients), if it is a food.
fn keeps_of(stack: &ItemStack) -> Option<(f32, f32)> {
    if let Some(packed) = MEAL.stack_only(stack) {
        return Some(meals::keeps(&meals::ingredients(packed)));
    }
    food::find(item_name(stack.item)).map(|f| food::FOODS[f].keeps)
}

/// The share of its life (fresh and stale days) a food stack has used by `now`: 0 new (or not
/// stamped yet), past 1 spoiled; `None` if it isn't a food. Hot food is new.
pub fn life_used(stack: &ItemStack, now: u64) -> Option<f32> {
    let keeps = keeps_of(stack)?;
    let (age, hot) = base_api::food_age(stack, now);
    if hot {
        return Some(0.0);
    }
    let days = age / vitals::DAY_SECS / base_api::difficulty().keep.max(0.01);
    Some(days / (keeps.0 + keeps.1).max(0.01))
}

/// Whether a food stack has spoiled by `now`.
pub fn spoiled(stack: &ItemStack, now: u64) -> bool {
    life_used(stack, now).is_some_and(|u| u >= 1.0)
}

/// The day stamp (`MADE`) for a new food keeping `keeps` that starts with `used` of its life gone
/// (Research round 3, D1: what is made of stale food is as far along as its stalest part, never
/// fresh again; spoiled food isn't taken at all).
pub fn made_for(used: f32, keeps: (f32, f32), now: u64) -> u32 {
    let today = (now / calendar::DAY_TICKS) as u32 + 1;
    let days = (used.clamp(0.0, 1.0) * (keeps.0 + keeps.1) * base_api::difficulty().keep).floor() as u32;
    today.saturating_sub(days).max(1)
}

/// A crafted food (salted meat, a pie) from these cells: refused if any food in them has spoiled,
/// else stamped as far along as its stalest ingredient.
pub fn crafted(cells: &[ItemStack], result: ItemStack, now: u64) -> ItemStack {
    let Some(keeps) = keeps_of(&result) else { return result };
    let mut used = 0.0f32;
    for c in cells.iter().filter(|c| !c.is_empty()) {
        if let Some(u) = life_used(c, now) {
            if u >= 1.0 {
                return ItemStack::EMPTY;
            }
            used = used.max(u);
        }
    }
    if used > 0.0 { result.with(&MADE, made_for(used, keeps, now)) } else { result }
}

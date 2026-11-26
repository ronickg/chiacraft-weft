//! Furnace and chest: blocks with their own inventories and state.

use weft_sdk::*;

use crate::blocks::{CHEST, FURNACE};
use crate::items::FUEL;

pub use base_api::{
    chains::SMELT_TIME,
    components::{SMELTING_STATE as SMELTING, Smelting},
    events::{SMELTED, Smelted},
    menus::{CHEST as CHEST_MENU, FURNACE as FURNACE_MENU},
};

pub fn register(r: &mut Registry) {
    r.instance_component(
        &SMELTING,
        Smelting { burn: 0, burn_max: 0, cook: 0, cook_max: 0, cooking_item: 0 },
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&SMELTING, tick_furnaces);
    // The furnace's state syncs to whoever has it open: the screen binds to it directly.
    r.block(&FURNACE)
        // Front towards the player; unlit and lit.
        .sound_group("stone")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("lit", ["false", "true"])
        .states(|p, s| {
            let lit = p.is("lit", "true");
            let s = s
                .texture("furnace_side")
                .ends("furnace_top")
                .front(if lit { "furnace_front_lit" } else { "furnace_front" });
            if lit { s.light_rgb(13, 11, 8) } else { s }
        })
        .hardness(3.5)
        .mined_by("pickaxe", 0)
        .tab(&crate::tabs::FUNCTIONAL)
        .attach(&SMELTING)
        .inventory(3)
        .opens(&FURNACE_MENU);
    r.menu(&FURNACE_MENU)
        .group("input", Slots::single().holder())
        .group(
            "fuel",
            Slots::single()
                .accepts("#fuel")
                .priority(1, "")
                .background("slot_fuel")
                .holder(),
        )
        .group("output", Slots::single().output().holder())
        .recipes(&crate::recipes::SMELTING)
        // Raw ore a blueprint not yet read teaches to smelt goes back (`ruins.rs`).
        .on_change(crate::ruins::furnace_changed)
        .player_inventory()
        .screen(|ui| {
            crate::window::window(
                ui,
                S::column()
                    .pad(crate::window::WINDOW_PAD)
                    .gap(4.0)
                    .stretch(),
                &[],
                |ui| {
                    ui.row(S::row().gap(8.0), |ui| {
                        ui.spacer();
                        ui.column(S::column().gap(2.0), |ui| {
                            ui.group("input");
                            ui.progress_bind_dir("flame", S::default().size(14.0, 14.0), abi::ui_dir::UP, |c| {
                                c.menu_component(&SMELTING)
                                    .map_or(0.0, |s| ratio(s.burn, s.burn_max))
                            });
                            ui.group("fuel");
                        });
                        ui.recipe_arrow_bind(|c| {
                            c.menu_component(&SMELTING)
                                .map_or(0.0, |s| ratio(s.cook, s.cook_max))
                        });
                        ui.group("output");
                        ui.spacer();
                    });
                    ui.player_inventory(true);
                },
            );
        });

    r.block(&CHEST)
        .sound_group("wood")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .states(|_, s| {
            s.texture("chest_side")
                .ends("chest_top")
                .front("chest_front")
                .model_box([1.0, 0.0, 1.0], [15.0, 14.0, 15.0])
        })
        .hardness(2.5)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::tabs::FUNCTIONAL)
        // Its food rots by its place (`survival/rot.rs`).
        .attach(&crate::survival::LARDER)
        .inventory(27)
        .opens(&CHEST_MENU);
    r.menu(&CHEST_MENU)
        .group("chest", Slots::chest(3).holder())
        // Its screen (ours, not the default one) draws the player's slots: the menu must have them.
        .player_inventory()
        .screen(|ui| {
            crate::window::window(ui, S::column().pad(crate::window::WINDOW_PAD).gap(4.0), &[], |ui| {
                ui.group("chest");
                ui.player_inventory(true);
            });
        });
}

/// A furnace's result, stamped as food fresh off the fire: made today and hot (`survival/food.rs`
/// ages it from there); anything else as it is.
fn cooked(w: &World, out: ItemStack) -> ItemStack {
    let Some(f) = vitals::food::find(item_name(out.item)).map(|f| &vitals::food::FOODS[f]) else { return out };
    let now = w.day_time();
    let made = out.with(&base_api::MADE, crate::survival::stamp(now));
    if f.meal == vitals::food::Meal::Cooked {
        made.with(&base_api::HOT, (now / 20) as u32)
            .with(&base_api::FRESHNESS, base_api::freshness::HOT)
    } else {
        made
    }
}

/// Items the output slot still takes of `out`, as whole results (by item: what's there keeps its
/// stamps). The SDK's `ItemStack::room_for` goes by `stacks_with`, which a cooked stamp (made,
/// hot) would stop.
fn room_for(output: ItemStack, out: ItemStack) -> u32 {
    let room = if output.is_empty() {
        max_stack(out.item)
    } else if output.item == out.item {
        max_stack(out.item).saturating_sub(output.count)
    } else {
        0
    };
    room / out.count.max(1)
}

/// Raw ore of a metal that melts only in a hot fire (`vitals::smelt::HOT_METALS`: copper, so
/// raw copper, malachite and native copper; `metal_tags_are_the_hot_metals`).
static HOT_ORES: [Tag; 1] = [Tag::named("raw_ores/copper")];

fn needs_heat(item: u32) -> bool {
    item != 0 && HOT_ORES.iter().any(|t| t.on_item(item))
}

/// `ticks` of one furnace at once (its area was unloaded, the time jumped): it smelts while
/// input, room and fuel last, lighting fuel as it goes, then the fire burns down and the
/// progress cools, as tick by tick.
fn catch_up(w: &World, owner: Holder, pos: BlockPos, s: &mut Smelting, ticks: u32) {
    let inv = w.inventory(owner);
    let (input_slot, fuel_slot, output_slot) = (inv.group("input"), inv.group("fuel"), inv.group("output"));
    let (input, fuel, output) = (input_slot.get(0), fuel_slot.get(0), output_slot.get(0));
    let mut idle = ticks as u64;
    // Copper away from a hot fire waits, as tick by tick (its fuel hot and bellows beside).
    let hot_enough = || {
        let fuel_ticks = FUEL.on_stack_item(&fuel).unwrap_or(0).max(s.burn_max);
        !needs_heat(input.item) || vitals::smelt::hot(fuel_ticks, crate::casting::bellows_beside(w, pos))
    };
    if let Some((out, time)) = (!input.is_empty() && hot_enough())
        .then(|| recipe_process(&crate::recipes::SMELTING, input.item))
        .flatten()
    {
        let q = abi::Query { pos, item: input.item, ..query() };
        s.cook_max = w
            .chain_compute(&SMELT_TIME, time as f64, &q)
            .round()
            .max(1.0) as u32;
        let items = (input.count).min(room_for(output, out)) as u64;
        let per_fuel = FUEL.on_stack_item(&fuel).unwrap_or(0) as u64;
        let fuel_items = if per_fuel > 0 { fuel.count as u64 } else { 0 };
        let work = (items * s.cook_max as u64).saturating_sub(s.cook as u64);
        let worked = idle.min(work).min(s.burn as u64 + fuel_items * per_fuel);
        let lit = if worked > s.burn as u64 { (worked - s.burn as u64).div_ceil(per_fuel.max(1)) } else { 0 };
        if lit > 0 {
            let mut left = fuel;
            for _ in 0..lit {
                left = consume_one(left);
            }
            fuel_slot.set(0, left);
            s.burn_max = per_fuel as u32;
        }
        s.burn = (s.burn as u64 + lit * per_fuel - worked) as u32;
        let total = s.cook as u64 + worked;
        let done = (total / s.cook_max as u64) as u32;
        s.cook = (total % s.cook_max as u64) as u32;
        if done > 0 {
            input_slot.set(0, input.shrink(done));
            output_slot.set(0, if output.is_empty() { cooked(w, output.merged(out, done)) } else { output.merged(out, done) });
            for _ in 0..done {
                w.emit(&SMELTED, Smelted { furnace: owner.entity, input: input.item, output: out });
            }
        }
        idle -= worked;
    }
    s.burn = (s.burn as u64).saturating_sub(idle) as u32;
    s.cook = (s.cook as u64).saturating_sub(idle * 2) as u32;
}

/// Every furnace of a region at once, through its state (a system: no host call to reach it).
fn tick_furnaces(b: &mut SystemBatch<Smelting>) {
    let w = b.world();
    for (owner, s, ticks) in b.iter_mut() {
        let Some(pos) = w.block_location(owner.entity) else { continue };
        if ticks > 1 {
            let was_lit = s.burn > 0;
            catch_up(&w, owner, pos, s, ticks);
            if (s.burn > 0) != was_lit {
                set_lit(&w, owner, s.burn > 0);
            }
            continue;
        }
        let inv = w.inventory(owner);
        let (input_slot, fuel_slot, output_slot) = (inv.group("input"), inv.group("fuel"), inv.group("output"));
        let (input, fuel, output) = (input_slot.get(0), fuel_slot.get(0), output_slot.get(0));
        let was_lit = s.burn > 0;
        if s.burn > 0 {
            s.burn -= 1;
        }
        // Another item in the input starts over, before this tick counts for it.
        if input.item != s.cooking_item {
            s.cooking_item = input.item;
            s.cook = 0;
        }
        let recipe = (!input.is_empty())
            .then(|| recipe_process(&crate::recipes::SMELTING, input.item))
            .flatten();
        let fits = recipe.is_some_and(|(out, _)| room_for(output, out) > 0);
        // Copper melts only in a hot fire (`vitals::smelt`): it lights only hot fuel with bellows
        // beside, and cooks only while that burns.
        let needs = fits && needs_heat(input.item);
        let bellows = needs && crate::casting::bellows_beside(&w, pos);
        if s.burn == 0
            && fits
            && let Some(ticks) = FUEL
                .on_stack_item(&fuel)
                .filter(|&t| !needs || vitals::smelt::hot(t, bellows))
        {
            s.burn = ticks;
            s.burn_max = ticks;
            fuel_slot.set(0, consume_one(fuel));
        }
        let warm = !needs || vitals::smelt::hot(s.burn_max, bellows);
        if needs && !(s.burn > 0 && warm) && w.day_time() % 20 == 0 {
            crate::hints::copper_too_cold(&w, pos);
        }
        if s.burn > 0 && fits && warm {
            let (out, time) = recipe.unwrap();
            let q = abi::Query { pos, item: input.item, ..query() };
            s.cook_max = w
                .chain_compute(&SMELT_TIME, time as f64, &q)
                .round()
                .max(1.0) as u32;
            s.cook += 1;
            if s.cook >= s.cook_max {
                s.cook = 0;
                // The last one leaves its remainder: a water bucket boiled to salt, the bucket.
                input_slot.set(0, consume_one(input));
                // Into a slot already holding some, it takes the stamps there (`cooked`).
                let merged = if output.is_empty() { cooked(&w, output.merged(out, 1)) } else { output.merged(out, 1) };
                output_slot.set(0, merged);
                w.emit(&SMELTED, Smelted { furnace: owner.entity, input: input.item, output: out });
            }
        } else if s.burn > 0 {
            // Modern's: burning with nothing it can do starts over; out of fuel it cools down.
            s.cook = 0;
        } else {
            s.cook = s.cook.saturating_sub(2);
        }
        let lit = s.burn > 0;
        if lit != was_lit {
            set_lit(&w, owner, lit);
        }
    }
}

fn set_lit(w: &World, owner: Holder, lit: bool) {
    if let Some(state) = w.block_instance_state(owner.entity)
        && let Some(next) = state_with(state, "lit", if lit { "true" } else { "false" })
    {
        w.set_block_instance_state(owner.entity, next);
    }
}

#[cfg(test)]
mod heat_tests {
    #[test]
    fn metal_tags_are_the_hot_metals() {
        assert_eq!(super::HOT_ORES.len(), vitals::smelt::HOT_METALS.len());
        for (t, m) in super::HOT_ORES.iter().zip(vitals::smelt::HOT_METALS) {
            assert!(t.name().ends_with(&format!("raw_ores/{m}")));
        }
    }
}

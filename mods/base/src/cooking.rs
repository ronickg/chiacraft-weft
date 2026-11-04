//! Cooking and keeping food (`vitals::meals`, `vitals::preserve`):
//!
//! - **The cooking pot**: two to four foods and a bowl over its own fire make a stew, which is what
//!   went into it (a stack component), hot off the fire. Eating it puts each ingredient into the
//!   stomach, stronger together (`base_api::eat_meal`), and leaves the bowl.
//! - **The drying rack**: dries meat, apples and berries by the air where it stands (`air_at`: warm,
//!   dry, breezy, not in the rain), or smokes the meat over a fire below it, in any weather.
//! - **Salt**, boiled out of a water bucket in a furnace, salts raw meat at the crafting table.
//!
//! What the pot and the rack make is stamped with its day as it's made, so it ages wherever it is
//! kept (`survival/food.rs` for the rest of food).

use base_api::{BOWL, FRESHNESS, FUEL, HOT, MADE, MEAL, MEAL_NUTRITION, Nutrition, freshness};
use vitals::food;
use vitals::meals;
use vitals::preserve::{self, Method};
use weft_sdk::*;

pub use base_api::{CRAFTING, SMELTING};

pub static COOKING_POT: Block = Block::named("cooking_pot");
pub static DRYING_RACK: Block = Block::named("drying_rack");
/// Logs and sticks burning, no stone needed: a fire to warm by (`base_api::heat_of`, +20 °C
/// beside it), to rest by (`survival::camp`) and to smoke over (Research round 3, B1).
pub static CAMPFIRE: Block = Block::named("campfire");
pub static POT_MENU: Menu = Menu::named("cooking_pot");
pub static RACK_MENU: Menu = Menu::named("drying_rack");
pub static CAMPFIRE_MENU: Menu = Menu::named("campfire");
pub static STEW: Item = Item::named("stew");
pub static SALT: Item = Item::named("salt");

/// What preserving makes, by name (`preserve::product`).
static PRODUCTS: [Item; 5] = [
    Item::named("dried_meat"),
    Item::named("smoked_meat"),
    Item::named("salted_meat"),
    Item::named("dried_apple"),
    Item::named("dried_berries"),
];

value! {
    /// A cooking pot's fire and how far the stew is along (`cooking`: the ingredients it is for).
    pub struct Cooking { pub burn: u32, pub burn_max: u32, pub cook: u32, pub cooking: u32 }
}
value! {
    /// A drying rack: how far each slot's food is dried (0 to 1000), the item it is for, the ticks
    /// since it last looked at the air, and whether a fire below smokes it.
    pub struct Drying { pub done: [u32; 4], pub items: [u32; 4], pub wait: u32, pub smoking: u32 }
}
value! {
    /// A campfire's cooking: how far along, of how long, and the item it's for.
    pub struct Roasting { pub cook: u32, pub cook_max: u32, pub item: u32 }
}
static ROASTING: Component<Roasting> = Component::named("campfire");
static COOKING: Component<Cooking> = Component::named("cooking_pot");
static DRYING: Component<Drying> = Component::named("drying_rack");

/// The most ticks a pot or rack catches up at once when its area loads again.
const CATCH_UP: u32 = 24000;

pub fn register(r: &mut Registry) {
    r.stack_component(&MEAL, 0);
    r.stack_component(&MEAL_NUTRITION, 0);
    r.item(&BOWL)
        .tag("bowl")
        .max_stack(16)
        .tab(&crate::tabs::FOOD);
    r.item(&STEW)
        .tag("stew")
        .max_stack(1)
        .set_text(&REMAINDER, "base:bowl")
        .set(&FRESHNESS, freshness::FRESH)
        .use_ticks(32)
        .on_use(base_api::eat_meal)
        .tab(&crate::tabs::FOOD);
    r.item(&SALT).tag("salt").tab(&crate::tabs::INGREDIENTS);
    r.shaped(&CRAFTING, "bowl", 4, &["p p", " p "], &[('p', "#planks")]);
    // Sea water boiled dry; the bucket comes back (`use_one` in the furnace).
    r.shapeless(&SMELTING, "salt", 1, &["water_bucket"], preserve::SALT_TICKS);
    r.shapeless(&CRAFTING, "salted_meat", 1, &["#raw_meat", "#salt"], 0);
    r.shaped(&CRAFTING, "cooking_pot", 1, &["i i", "i i", "iii"], &[('i', "#ingots/iron")]);
    r.shaped(&CRAFTING, "drying_rack", 1, &["sss", "s s"], &[('s', "#stick")]);
    // In the inventory's own grid: a fire comes before a crafting table (Research round 4).
    r.shaped(&CRAFTING, "campfire", 1, &["ss", "ll"], &[('s', "#stick"), ('l', "#logs")]);
    r.entity_component(&ROASTING, Roasting { cook: 0, cook_max: 0, item: 0 }, component_flags::SAVE | component_flags::SYNC_MENU);
    r.system(&ROASTING, tick_campfires);
    // It cooks what a furnace cooks of food, a little slower, with no fuel.
    r.machine(&CAMPFIRE, &CAMPFIRE_MENU)
        .block(|b| {
            b.sound_group("wood")
                .textures("campfire_top", "campfire_side", "campfire_side")
                .model_box([0.0, 0.0, 0.0], [16.0, 7.0, 16.0])
                .light_rgb(15, 11, 6)
                .hardness(2.0)
                .mineable("axe")
                .item_texture("campfire")
                .tag("campfires")
                .tab(&crate::tabs::FUNCTIONAL)
        })
        .group("input", Slots::single().accepts("food"))
        .group("output", Slots::single().output())
        .attach(&ROASTING)
        .screen(|ui| {
            crate::window::window(ui, S::col().pad(7.0).gap(4.0).stretch(), &[], |ui| {
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.group("input");
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&ROASTING)
                            .map_or(0.0, |s| ratio(s.cook, s.cook_max))
                    });
                    ui.group("output");
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });

    r.entity_component(&COOKING, Cooking { burn: 0, burn_max: 0, cook: 0, cooking: 0 }, component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(
        &DRYING,
        Drying { done: [0; 4], items: [0; 4], wait: 0, smoking: 0 },
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&COOKING, tick_pots);
    r.system(&DRYING, tick_racks);

    r.machine(&COOKING_POT, &POT_MENU)
        .block(|b| {
            b.sound_group("stone")
                .faces([
                    "cooking_pot_side",
                    "cooking_pot_side",
                    "cooking_pot_top",
                    "cooking_pot_side",
                    "cooking_pot_side",
                    "cooking_pot_side",
                ])
                .model_box([2.0, 0.0, 2.0], [14.0, 10.0, 14.0])
                .hardness(3.0)
                .mineable("pickaxe")
                .item_texture("cooking_pot")
                .tab(&crate::tabs::FUNCTIONAL)
        })
        .group("ingredients", Slots::grid(2, 2).accepts("food"))
        .group("bowl", Slots::single().accepts("bowl"))
        .group("fuel", Slots::single().accepts("fuel"))
        .group("output", Slots::single().output())
        .attach(&COOKING)
        .screen(|ui| {
            crate::window::window(ui, S::col().pad(7.0).gap(4.0).stretch(), &[], |ui| {
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.group("ingredients");
                    ui.column(S::col().gap(2.0), |ui| {
                        ui.group("bowl");
                        ui.progress_bind_dir("flame", S::default().size(14.0, 14.0), abi::ui_dir::UP, |c| {
                            c.menu_component(&COOKING)
                                .map_or(0.0, |s| ratio(s.burn, s.burn_max))
                        });
                        ui.group("fuel");
                    });
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&COOKING)
                            .map_or(0.0, |s| ratio(s.cook, meals::COOK_TICKS))
                    });
                    ui.group("output");
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });

    r.machine(&DRYING_RACK, &RACK_MENU)
        .block(|b| {
            b.sound_group("wood")
                .texture("drying_rack")
                // A frame of sticks: the texture's gaps are see-through.
                .cutout()
                .model_box([0.0, 0.0, 0.0], [16.0, 14.0, 16.0])
                .hardness(1.5)
                .mineable("axe")
                .item_texture("drying_rack")
                .set(&base_api::FLAMMABLE, 20)
                .tab(&crate::tabs::FUNCTIONAL)
        })
        .group("rack", Slots::row(4).accepts("food"))
        .attach(&DRYING)
        .screen(|ui| {
            crate::window::window(ui, S::col().pad(7.0).gap(4.0).stretch(), &[], |ui| {
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.column(S::col().gap(2.0), |ui| {
                        ui.group("rack");
                        ui.progress_bind("attack_bar", S::default().size(72.0, 4.0), |c| {
                            c.menu_component(&DRYING)
                                .map_or(0.0, |s| *s.done.iter().max().unwrap_or(&0) as f32 / 1000.0)
                        });
                    });
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });
}

fn ratio(a: u32, b: u32) -> f32 {
    if b == 0 { 0.0 } else { (a as f32 / b as f32).min(1.0) }
}

fn food_of(item: u32) -> Option<usize> {
    if item == 0 { None } else { food::find(item_name(item)) }
}

/// A stew of these foods (`FOODS` indexes, as they went in), hot off the fire, `used` of its life
/// gone (its stalest ingredient's share, `survival::made_for`).
pub fn stew(w: &World, foods: &[usize], used: f32) -> ItemStack {
    let packed = meals::pack(foods);
    let cooked = meals::ingredients(packed);
    let n = meals::nutrients(&cooked).map(|g| g.round() as u16);
    let nutrition = Nutrition {
        units: meals::units(&cooked).round() as u16,
        protein: n[food::PROTEIN],
        energy: n[food::ENERGY],
        fat: n[food::FAT],
        greens: n[food::GREENS],
        water: n[food::WATER],
    };
    ItemStack::new(STEW.id(), 1)
        .with(&MEAL, packed)
        .with(&MEAL_NUTRITION, nutrition)
        .with(&MADE, crate::survival::made_for(used, meals::keeps(&cooked), w.day_time()))
        .with(&HOT, (w.day_time() / 20) as u32)
        .with(&FRESHNESS, freshness::HOT)
}

/// Every campfire of a region: cooking the food in it.
fn tick_campfires(b: &mut SystemBatch<Roasting>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        for _ in 0..ticks.clamp(1, CATCH_UP) {
            if !roast_step(&w, id, s) {
                break;
            }
        }
    }
}

/// A tick of a campfire; false when it has nothing to cook.
fn roast_step(w: &World, id: u64, s: &mut Roasting) -> bool {
    let inv = w.inventory(id);
    let (input, output) = (inv.group("input"), inv.group("output"));
    let (st, out_now) = (input.get(0), output.get(0));
    if st.item != s.item {
        (s.item, s.cook) = (st.item, 0);
    }
    let now = w.day_time();
    let Some((out, time)) = (!st.is_empty() && !crate::survival::spoiled(&st, now))
        .then(|| recipe_process(&SMELTING, st.item))
        .flatten()
        .filter(|(out, _)| food_of(out.item).is_some())
    else {
        s.cook = 0;
        return false;
    };
    let keeps = food_of(out.item).map_or((1.0, 1.0), |f| food::FOODS[f].keeps);
    let used = crate::survival::life_used(&st, now).unwrap_or(0.0);
    let made = out
        .with(&MADE, crate::survival::made_for(used, keeps, now))
        .with(&HOT, (now / 20) as u32)
        .with(&FRESHNESS, freshness::HOT);
    let fits = out_now.is_empty() || (out_now.stacks_with(&made) && out_now.count + made.count <= max_stack(made.item));
    if !fits {
        return false;
    }
    s.cook_max = (time as f32 * meals::CAMPFIRE_PACE).round().max(1.0) as u32;
    s.cook += 1;
    if s.cook >= s.cook_max {
        s.cook = 0;
        input.set(0, st.shrink(1));
        output.set(0, if out_now.is_empty() { made } else { out_now.grow(made.count) });
    }
    true
}

/// Every pot of a region: burning its fuel while it has foods and a bowl to cook, then a stew.
fn tick_pots(b: &mut SystemBatch<Cooking>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        for _ in 0..ticks.clamp(1, CATCH_UP) {
            if !pot_step(&w, id, s) {
                break;
            }
        }
    }
}

/// A tick of a pot; false when there's nothing more it can do (so catching up stops).
fn pot_step(w: &World, id: u64, s: &mut Cooking) -> bool {
    let inv = w.inventory(id);
    let (ings, bowl, fuel, output) = (inv.group("ingredients"), inv.group("bowl"), inv.group("fuel"), inv.group("output"));
    // Spoiled food doesn't go in (Research round 3, D1).
    let now = w.day_time();
    let good = |st: &ItemStack| food_of(st.item).is_some() && !crate::survival::spoiled(st, now);
    let foods: Vec<usize> = (0..ings.len())
        .map(|i| ings.get(i))
        .filter(&good)
        .filter_map(|st| food_of(st.item))
        .collect();
    let can = foods.len() >= meals::MIN && !bowl.get(0).is_empty() && output.get(0).is_empty();
    let key = meals::pack(&foods) as u32 ^ (meals::pack(&foods) >> 32) as u32;
    if key != s.cooking {
        (s.cooking, s.cook) = (key, 0);
    }
    if s.burn > 0 {
        s.burn -= 1;
    }
    if s.burn == 0 && can {
        let f = fuel.get(0);
        if let Some(t) = FUEL.on_item(f.item).filter(|_| !f.is_empty()) {
            (s.burn, s.burn_max) = (t, t);
            fuel.set(0, use_one(f));
        }
    }
    if s.burn > 0 && can {
        s.cook += 1;
        if s.cook >= meals::COOK_TICKS {
            s.cook = 0;
            let mut used = 0.0f32;
            for i in 0..ings.len() {
                let st = ings.get(i);
                if good(&st) {
                    used = used.max(crate::survival::life_used(&st, now).unwrap_or(0.0));
                    ings.set(i, use_one(st));
                }
            }
            bowl.set(0, use_one(bowl.get(0)));
            output.set(0, stew(w, &foods, used));
        }
        true
    } else {
        s.cook = s.cook.saturating_sub(2);
        s.burn > 0
    }
}

/// Every rack of a region, looking at the air once a second.
fn tick_racks(b: &mut SystemBatch<Drying>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        s.wait += ticks.min(CATCH_UP);
        if s.wait < 20 {
            continue;
        }
        let secs = s.wait as f32 / 20.0;
        s.wait = 0;
        let Some(st) = w.entity(id) else { continue };
        let pos = st.block_pos();
        let smoking = (1..=preserve::SMOKE_REACH).any(|dy| base_api::heat_of(w.get([pos[0], pos[1] - dy, pos[2]])) >= preserve::SMOKE_HEAT);
        s.smoking = smoking as u32;
        let rate = if smoking {
            preserve::smoke_rate()
        } else {
            let air = base_api::air_at(&w, [pos[0] as f64 + 0.5, pos[1] as f64, pos[2] as f64 + 0.5], 1.0, false, 0, |_| {});
            // The still air: wind chill is a body's, food on a rack doesn't fall below the air (the
            // wind's drying is `dry_rate`'s breeze).
            preserve::dry_rate(air.still, air.humidity, air.wind, air.raining && air.open_sky)
        };
        let inv = w.inventory(id);
        let rack = inv.group("rack");
        for i in 0..rack.len().min(4) as usize {
            let stack = rack.get(i as u32);
            if stack.item != s.items[i] {
                (s.items[i], s.done[i]) = (stack.item, 0);
            }
            let Some(f) = food_of(stack.item) else { continue };
            let now = w.day_time();
            if crate::survival::spoiled(&stack, now) {
                continue;
            }
            // Over a fire meat smokes and fruit dries the faster; in the air both dry.
            let method = if smoking && preserve::product(Method::Smoked, f).is_some() { Method::Smoked } else { Method::Dried };
            let Some(product) = preserve::product(method, f) else { continue };
            s.done[i] = (s.done[i] as f32 + rate * secs * 1000.0).round() as u32;
            if s.done[i] >= 1000 {
                let Some(item) = PRODUCTS.iter().find(|p| item_name(p.id()) == product) else { continue };
                let used = crate::survival::life_used(&stack, now).unwrap_or(0.0);
                let keeps = food::find(product).map_or((1.0, 1.0), |p| food::FOODS[p].keeps);
                let made = ItemStack::new(item.id(), stack.count).with(&MADE, crate::survival::made_for(used, keeps, now));
                rack.set(i as u32, made);
                (s.items[i], s.done[i]) = (made.item, 0);
            }
        }
    }
}

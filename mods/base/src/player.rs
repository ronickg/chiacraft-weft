//! The player's armor, its HUD points and the inventory screen with its 2x2 crafting grid; eating,
//! stamina, health and breath are the vitals' (`survival/`).

use weft_sdk::*;

use crate::items::ARMOR;

pub use base_api::{FEET, HEAD, LEGS};

pub static INVENTORY: Menu = Menu::named("inventory");
pub static CRAFTING: Menu = Menu::named("crafting_table");
static HUD_ARMOR: HudElement = HudElement::named("armor");
static HUD_AIR: HudElement = HudElement::named("air");
static HUD_ATTACK: HudElement = HudElement::named("attack_indicator");

/// Head to feet.
pub const EQUIPMENT: [&EquipmentSlot; 4] = [&HEAD, &base_api::CHEST, &LEGS, &FEET];

/// Armor points and toughness of what `entity` wears.
fn armor_of(w: &World, entity: u64) -> (u32, f32) {
    let Some(worn) = w.inventory(entity).find_group(EQUIPMENT_GROUP) else { return (0, 0.0) };
    (0..worn.len())
        .filter_map(|i| ARMOR.on_item(worn.get(i).item))
        .fold((0, 0.0), |t, a| (t.0 + a.points, t.1 + a.toughness))
}

pub fn register(r: &mut Registry) {
    r.player_inventory()
        .group("hotbar", Slots::row(9).hotbar())
        .group("main", Slots::grid(9, 3));
    for (slot, parts, piece) in [
        (&HEAD, worn_parts::HEAD, "helmet"),
        (&base_api::CHEST, worn_parts::BODY | worn_parts::ARMS, "chestplate"),
        (&LEGS, worn_parts::BODY | worn_parts::LEGS, "leggings"),
        (&FEET, worn_parts::LEGS, "boots"),
    ] {
        r.equipment_slot(slot, parts, &format!("slot_{piece}"));
    }
    // Clothes under the armor (`vitals::clothing`): warmth, no protection.
    for (slot, parts, piece) in [
        (&base_api::HAT, worn_parts::HEAD, "hat"),
        (&base_api::COAT, worn_parts::BODY | worn_parts::ARMS, "coat"),
        (&base_api::TROUSERS, worn_parts::BODY | worn_parts::LEGS, "trousers"),
        (&base_api::SHOES, worn_parts::LEGS, "shoes"),
        // And a cloak over everything (CONTENT decision 3: nine slots).
        (&base_api::CLOAK, worn_parts::BODY | worn_parts::ARMS, "cloak"),
    ] {
        r.equipment_slot(slot, parts, &format!("slot_{piece}"));
    }
    // Armor is a step of the damage chain (modern's CombatRules: big hits go through more of it,
    // unless it is tough), and wears when the hit actually lands, a quarter of the damage per
    // piece (at least 1) like vanilla.
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        if !armor_applies(q.cause) {
            return;
        }
        let (points, toughness) = armor_of(&m.world(), q.entity);
        let points = points as f64;
        let through = m.value / (2.0 + toughness as f64 / 4.0);
        m.value *= 1.0 - (points - through).clamp(points * 0.2, 20.0) / 25.0;
    });
    r.on(&ENTITY_HURT, |b| {
        let w = b.world();
        for h in b.iter().filter(|h| armor_applies(h.cause)) {
            let Some(worn) = w.inventory(h.entity).find_group(EQUIPMENT_GROUP) else { continue };
            let wear = ((h.amount / 4.0) as u32).max(1);
            for slot in 0..worn.len() {
                let s = worn.get(slot);
                if ARMOR.on_item(s.item).is_some() {
                    let durability = DURABILITY.on_item(s.item).unwrap_or(0);
                    let damage = s.damage() + wear;
                    worn.set(slot, if durability > 0 && damage >= durability { ItemStack::EMPTY } else { s.with_damage(damage) });
                }
            }
        }
    });

    // Names the held stack, like an anvil: a chest placed from it is called so.
    r.command("rename", "[name:text]", |cmd| {
        let w = cmd.world();
        let hotbar = w.inventory(cmd.player()).group("hotbar");
        let slot = w.selected_slot(cmd.player());
        let s = hotbar.get(slot);
        if !s.is_empty() {
            hotbar.set(slot, with_name(s, cmd.text("name")));
        }
    })
    .permission(permission::EVERYONE);
    // Armor and breath in the top left, under the vitals' bars (`hud.rs`); a negative value hides
    // them (creative).
    r.hud_element(&HUD_ARMOR, hud_anchor::TOP_LEFT, 6, |ui| {
        ui.icons("armor", 10, |c| {
            // From what the client knows of its own inventory, like the server's damage chain.
            let points: u32 = c
                .inventory_group(EQUIPMENT_GROUP)
                .iter()
                .filter_map(|s| ARMOR.on_item(s.item))
                .map(|a| a.points)
                .sum();
            if points > 0 && !c.creative() { points as f32 } else { -1.0 }
        })
    });
    r.hud_element(&HUD_AIR, hud_anchor::TOP_LEFT, 7, |ui| {
        ui.icons("air", 10, |c| {
            if c.creative() {
                -1.0
            } else {
                c.component(&base_api::VITALS)
                    .map_or(-1.0, |v| air_bubbles(v.air))
            }
        })
    });

    // While a swing charges.
    r.hud_element(&HUD_ATTACK, hud_anchor::UNDER_CROSSHAIR, 0, |ui| {
        ui.progress_bind("attack_bar", S::default().size(16.0, 4.0), |c| {
            let charge = c.swing_strength();
            if charge < 1.0 { charge } else { -1.0 }
        })
    });

    // The equipment slots take what is worn there and show its silhouette while empty.
    r.menu(&INVENTORY)
        .group("grid", Slots::grid(2, 2))
        .group("result", Slots::single().output().large())
        .equipment()
        .player_inventory()
        .recipes(&crate::recipes::CRAFTING)
        .on_change(craft_result)
        .on_take(consume_grid)
        .screen(|ui| {
            let tabs: [crate::window::Tab; 2] = [
                ("window.base.tab.stats", |c| {
                    c.client().open_screen(&crate::levels::SCREEN);
                }),
                ("window.base.tab.advancements", |c| {
                    c.client().open_screen(&crate::advancements::SCREEN);
                }),
            ];
            // The recipe book beside it, hand recipes only (`book.rs`).
            ui.row(S::row().gap(4.0), |ui| {
                crate::window::window(ui, S::col().pad(7.0).gap(5.0), &tabs, |ui| {
                    ui.row(S::row().gap(1.0), |ui| {
                        // Armor, and the clothes worn under it beside, the cloak under them.
                        ui.column(S::col(), |ui| {
                            for slot in EQUIPMENT.iter() {
                                ui.group(slot.name());
                            }
                        });
                        ui.column(S::col(), |ui| {
                            for slot in base_api::CLOTHES.iter() {
                                ui.group(slot.name());
                            }
                            ui.group(base_api::CLOAK.name());
                        });
                        ui.player_model(51.0, 72.0);
                        ui.spacer();
                        ui.column(S::col().gap(2.0), |ui| {
                            ui.text_styled(S::default().no_wrap(), "window.base.crafting", &[]);
                            ui.row(S::row().gap(4.0), |ui| {
                                ui.group("grid");
                                ui.recipe_arrow();
                                ui.group("result");
                            });
                        });
                    });
                    ui.player_inventory(false);
                });
                crate::book::book(ui, crate::book::Station::Hands);
            });
        });

    r.menu(&CRAFTING)
        .group("grid", Slots::grid(3, 3))
        .group("result", Slots::single().output().large())
        // Items built from parts (`parts.rs`).
        .group("assembly", Slots::row(4))
        .group("assembled", Slots::single().output())
        .group("apart", Slots::single())
        .player_inventory()
        .recipes(&crate::recipes::CRAFTING)
        .on_change(|m| {
            craft_result(m);
            crate::parts::on_change(m);
        })
        .on_take(|m| {
            if !crate::parts::on_take(m) {
                consume_grid(m);
            }
        })
        .screen(|ui| {
            // The recipe book beside it: every recipe, from the bag and the chests near the table.
            ui.row(S::row().gap(4.0), |ui| {
                crate::window::window(ui, S::col().pad(7.0).gap(4.0).stretch(), &[], |ui| {
                    ui.row(S::row().gap(8.0), |ui| {
                        ui.spacer();
                        ui.group("grid");
                        ui.recipe_arrow();
                        ui.group("result");
                        ui.spacer();
                    });
                    crate::parts::ui(ui);
                    ui.player_inventory(true);
                });
                crate::book::book(ui, crate::book::Station::Table);
            });
        });
    r.player_menu(&INVENTORY);
}

fn armor_applies(cause: u32) -> bool {
    cause & damage::BYPASSES_ARMOR == 0
}

/// The `result` slot shows what the `grid` makes.
fn craft_result(m: &MenuCtx) {
    let grid = m.group("grid");
    let cells: Vec<ItemStack> = (0..grid.len()).map(|i| grid.get(i)).collect();
    let made = recipe_match(&crate::recipes::CRAFTING, &cells, grid.cols(), grid.rows());
    // Food made of food is as far along as its stalest part, and nothing comes of spoiled food.
    let made = crate::survival::crafted(&cells, made, m.world().day_time());
    m.group("result").set(0, made);
}

/// Takes one of each ingredient; the last of a stack leaves its `remainder` (a bucket). Crafting
/// gives XP by how many things went into it.
fn consume_grid(m: &MenuCtx) {
    // Only for the grid's own result.
    if m.group("result").index_of(m.slot()).is_none() {
        return;
    }
    let grid = m.group("grid");
    let cells: Vec<ItemStack> = (0..grid.len()).map(|i| grid.get(i)).collect();
    let made = recipe_match(&crate::recipes::CRAFTING, &cells, grid.cols(), grid.rows());
    if !made.is_empty() {
        crate::advancements::crafted(&m.world(), m.player(), made.item);
    }
    let mut used = 0;
    for i in 0..grid.len() {
        let s = grid.get(i);
        if !s.is_empty() {
            grid.set(i, use_one(s));
            used += 1;
        }
    }
    base_api::give_xp(&m.world(), m.player(), vitals::levels::xp::craft(used));
}

/// What the air bar shows: whole bubbles like vanilla's, hidden with a full breath.
fn air_bubbles(air: f32) -> f32 {
    use crate::survival::MAX_AIR;
    if air < MAX_AIR { (air.max(0.0) / MAX_AIR * 10.0).ceil() * 2.0 } else { -1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_shows_whole_bubbles() {
        assert_eq!(air_bubbles(crate::survival::MAX_AIR), -1.0, "hidden with a full breath");
        assert_eq!(air_bubbles(299.0), 20.0);
        assert_eq!(air_bubbles(270.0), 18.0);
        assert_eq!(air_bubbles(1.0), 2.0);
        assert_eq!(air_bubbles(-5.0), 0.0);
    }
}

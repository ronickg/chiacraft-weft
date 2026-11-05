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

/// Armor and the health it adds (engine points) of what `entity` wears, from the curve by each
/// piece's shell and slot (`vitals::gear::armour`): a piece built from parts carries its own
/// (`parts::ARMOUR_OF`); an old whole one, until it's converted, goes by its name
/// (`<material>_<piece>`). A worn-out piece stops nothing.
pub fn armor_of(w: &World, entity: u64) -> (f32, f32) {
    let Some(worn) = w.inventory(entity).find_group(EQUIPMENT_GROUP) else { return (0.0, 0.0) };
    (0..worn.len())
        .filter_map(|i| {
            let s = worn.get(i);
            ARMOR.on_item(s.item)?;
            if crate::parts::broken(&s) {
                return None;
            }
            match crate::parts::ARMOUR_OF.on_stack_own(&s) {
                Some(a) => Some((a.armour, a.health)),
                None => piece_armour(item_name(s.item)),
            }
        })
        .fold((0.0, 0.0), |t, a| (t.0 + a.0, t.1 + a.1))
}

/// An armour piece's armour and health, by its name.
fn piece_armour(name: &str) -> Option<(f32, f32)> {
    let name = name.rsplit(':').next().unwrap_or(name);
    let (material, piece) = name.rsplit_once('_')?;
    let slot = ["helmet", "chestplate", "leggings", "boots"]
        .iter()
        .position(|p| *p == piece)?;
    vitals::gear::armour_of(slot, material)
}

/// How much bigger held items are in the player's own first-person hand than Weft draws them
/// (`comp::FIRST_PERSON_HELD_SCALE`).
const FIRST_PERSON_ITEMS: f32 = 1.8;

pub fn register(r: &mut Registry) {
    // Third person sways and jolts as first person does (chiacraft's feel; Weft's own behind and
    // front cameras are still).
    drop(
        r.camera(&cameras::BEHIND)
            .with(weft_sdk::abi::camera_flags::EFFECTS),
    );
    drop(
        r.camera(&cameras::FRONT)
            .with(weft_sdk::abi::camera_flags::EFFECTS),
    );
    // Held items about as big in the view as they were before the Weft sync (Ronald: "our items
    // a bit bigger"): Weft holds them in the posed hand, smaller than our old corner.
    r.set_on(Target::Entity("core:player"), &FIRST_PERSON_HELD_SCALE, FIRST_PERSON_ITEMS);
    r.player_inventory()
        .group("hotbar", Slots::row(9).hotbar())
        .group("main", Slots::grid(9, 3));
    for (slot, covers, piece) in [
        (&HEAD, worn::HEAD, "helmet"),
        (&base_api::CHEST, worn::CHEST, "chestplate"),
        (&LEGS, worn::LEGS, "leggings"),
        (&FEET, worn::FEET, "boots"),
    ] {
        r.equipment_slot(slot, covers, &format!("slot_{piece}"));
    }
    // Clothes under the armor (`vitals::clothing`): warmth, no protection.
    for (slot, parts, piece) in [
        (&base_api::HAT, worn::HEAD, "hat"),
        (&base_api::COAT, worn::CHEST, "coat"),
        (&base_api::TROUSERS, worn::LEGS, "trousers"),
        (&base_api::SHOES, worn::FEET, "shoes"),
        // And a cloak over everything (CONTENT decision 3: nine slots).
        (&base_api::CLOAK, worn::CHEST, "cloak"),
    ] {
        r.equipment_slot(slot, parts, &format!("slot_{piece}"));
    }
    // Upstream's off-hand: a second item to hold (a shield, a torch).
    r.equipment_slot(&OFF_HAND, &[], "slot_offhand");
    // Armor is a step of the damage chain (`vitals::combat`: armour / (armour + 5 × hit), at most
    // 80 %: big hits go through more of it), and wears when the hit actually lands, a quarter of
    // the damage per piece (at least 1) like vanilla.
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        if !armor_applies(q.cause) {
            return;
        }
        let (armour, _) = armor_of(&m.world(), q.entity);
        m.value *= 1.0 - vitals::combat::armour_cut(armour, m.value as f32) as f64;
    });
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter().filter(|h| armor_applies(h.cause)) {
            let Some(worn) = w.inventory(h.entity).find_group(EQUIPMENT_GROUP) else { continue };
            let wear = ((h.amount / 4.0) as u32).max(1);
            for slot in 0..worn.len() {
                let s = worn.get(slot);
                if ARMOR.on_item(s.item).is_some() {
                    worn.set(slot, s.add_wear(wear));
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
            hotbar.set(slot, stack_with_name(s, cmd.text("name")));
        }
    })
    .permission(permission::EVERYONE);
    // Armor and breath in the top left, under the vitals' bars (`hud.rs`); a negative value hides
    // them (creative).
    r.hud_element(&HUD_ARMOR, hud_anchor::TOP_LEFT, 6, |ui| {
        ui.icons("armor", 10, |c| {
            // From what the client knows of its own inventory, like the server's damage chain: a
            // built piece's own points (`parts::ARMOUR_PER_POINT`), an old whole one's item's.
            let points: u32 = c
                .inventory_group(EQUIPMENT_GROUP)
                .iter()
                .filter_map(|s| ARMOR.on_stack(s))
                .map(|a| a.points)
                .sum();
            if points > 0 && !c.outside_survival() { points as f32 } else { -1.0 }
        });
    });
    r.hud_element(&HUD_AIR, hud_anchor::TOP_LEFT, 7, |ui| {
        ui.icons("air", 10, |c| {
            if c.outside_survival() {
                -1.0
            } else {
                c.component(&base_api::VITALS)
                    .map_or(-1.0, |v| air_bubbles(v.air))
            }
        });
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
                    c.client().screen_open(&crate::levels::SCREEN);
                }),
                ("window.base.tab.advancements", |c| {
                    c.client().screen_open(&crate::advancements::SCREEN);
                }),
            ];
            // The recipe book beside it, hand recipes only (`book.rs`).
            ui.row(S::row().gap(4.0), |ui| {
                crate::window::family_window(ui, S::column().pad(7.0).gap(5.0), None, &tabs, |ui| {
                    ui.row(S::row().gap(1.0), |ui| {
                        // Armor, and the clothes worn under it beside, the cloak under them.
                        ui.column(S::column(), |ui| {
                            for slot in EQUIPMENT.iter() {
                                ui.group(slot.name());
                            }
                        });
                        ui.column(S::column(), |ui| {
                            for slot in base_api::CLOTHES.iter() {
                                ui.group(slot.name());
                            }
                            ui.group(base_api::CLOAK.name());
                        });
                        ui.player_model(51.0, 72.0);
                        ui.spacer();
                        ui.column(S::column().gap(2.0).stretch(), |ui| {
                            ui.text_styled(S::default().no_wrap(), "window.base.crafting", &[]);
                            ui.row(S::row().gap(4.0), |ui| {
                                ui.group("grid");
                                ui.recipe_arrow();
                                ui.group("result");
                            });
                            // Upstream's off-hand slot, under the grid.
                            ui.spacer();
                            ui.group(OFF_HAND.name());
                        });
                    });
                    ui.player_inventory(false);
                });
                // Not in creative: its item picker takes the left side and has every item, and the
                // picker, the inventory and the book side by side were wider than a 1280 x 720
                // screen at GUI scale 2 (both ends cut off, the label check, 11-04).
                if !ui.view().creative() {
                    crate::book::book(ui, crate::book::Station::Hands);
                }
            });
        });

    r.menu(&CRAFTING)
        .group("grid", Slots::grid(3, 3))
        .group("result", Slots::single().output().large())
        // Items built from parts (`parts.rs`): four slots in a row, each showing while empty what
        // usually goes there (a head, a haft, a fitting; they take parts in any order), and the
        // slot that takes a tool apart.
        .group(
            crate::parts::ASSEMBLY[0],
            Slots::single()
                .background("slot_part_head")
                .tooltip("crafting.parts.slot.head"),
        )
        .group(
            crate::parts::ASSEMBLY[1],
            Slots::single()
                .background("slot_part_haft")
                .tooltip("crafting.parts.slot.haft"),
        )
        .group(
            crate::parts::ASSEMBLY[2],
            Slots::single()
                .background("slot_part_fitting")
                .tooltip("crafting.parts.slot.fitting"),
        )
        .group(crate::parts::ASSEMBLY[3], Slots::single().tooltip("crafting.parts.slot.more"))
        .group("assembled", Slots::single().output())
        .group(
            "apart",
            Slots::single()
                .background("slot_part_tool")
                .tooltip("crafting.parts.slot.apart"),
        )
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
                crate::window::window(ui, S::column().pad(7.0).gap(4.0).stretch(), &[], |ui| {
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

pub fn armor_applies(cause: u32) -> bool {
    DamageCause::flags_of(cause) & damage_flags::BYPASSES_ARMOR == 0
}

/// The `result` slot shows what the `grid` makes.
fn craft_result(m: &MenuCtx) {
    let grid = m.group("grid");
    let cells: Vec<ItemStack> = (0..grid.len()).map(|i| grid.get(i)).collect();
    let made = recipe_grid_output(&crate::recipes::CRAFTING, &cells, grid.cols(), grid.rows());
    // Food made of food is as far along as its stalest part, and nothing comes of spoiled food.
    let made = crate::survival::crafted(&cells, made, m.world().day_time());
    // An axe lashed by hand takes its parts from the grid (`first.rs`).
    let made = crate::first::finish(&m.world(), m.player(), &cells, made, None);
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
    let made = recipe_grid_output(&crate::recipes::CRAFTING, &cells, grid.cols(), grid.rows());
    if !made.is_empty() {
        crate::advancements::crafted(&m.world(), m.player(), made.item);
    }
    let mut used = 0;
    for i in 0..grid.len() {
        let s = grid.get(i);
        if !s.is_empty() {
            grid.set(i, consume_one(s));
            used += 1;
        }
    }
    base_api::give_xp(&m.world(), m.player(), vitals::levels::xp::craft(used));
    crate::skills::made(&m.world(), m.player(), &made, used);
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

//! Copper's smelt and casting (`notes/COPPER.md`, slice 2; numbers in `vitals::smelt`):
//! - **Bellows**: a block that, beside a furnace burning charcoal, makes it hot enough for copper
//!   (`block_entities.rs` holds raw copper until it is).
//! - **Moulds**: fired-clay blocks, one for each head (`vitals::smelt::MOULDS`). Clay in the
//!   head's shape makes an unfired mould; fired in the furnace it is the mould block. Beside a hot
//!   furnace, a mould casts the copper ingots put in it into its head (`CAST_INGOTS` a head, in
//!   `cast_secs`), graded as a mould's work (`knapping::cast_head`). It is kept, as moulds were.

use vitals::smelt as sm;
use weft_sdk::*;

pub use base_api::recipes::{CRAFTING, SMELTING};

pub static BELLOWS: Block = Block::named("bellows");
/// The moulds, in `vitals::smelt::MOULDS`' order (`moulds_match_the_shapes`).
pub static MOULD_BLOCKS: [Block; 3] =
    [Block::named("mould_felling_axe_head"), Block::named("mould_pick_head"), Block::named("mould_shovel_blade")];
static UNFIRED: [&str; 3] = ["unfired_mould_felling_axe_head", "unfired_mould_pick_head", "unfired_mould_shovel_blade"];
/// Clay in each head's shape (clay blocks: ours drop as blocks, not balls). None is another clay
/// recipe's (the cooking pot's `c c` cup, the slime block's ring).
const PATTERNS: [&[&str]; 3] = [&["cc", "c "], &["ccc", " c "], &["cc", "cc"]];
static MOULD_MENU: Menu = Menu::named("mould");
static COPPER_INGOTS: Tag = Tag::named("ingots/copper");

value! {
    /// A mould's cast: how far along (0 to 1).
    pub struct Cast { pub done: f32 }
}
static CAST: Component<Cast> = Component::named("mould_cast");

/// The six blocks around `pos`.
fn around(pos: BlockPos) -> [BlockPos; 6] {
    let BlockPos { x, y, z } = pos;
    [
        BlockPos::new(x + 1, y, z),
        BlockPos::new(x - 1, y, z),
        BlockPos::new(x, y + 1, z),
        BlockPos::new(x, y - 1, z),
        BlockPos::new(x, y, z + 1),
        BlockPos::new(x, y, z - 1),
    ]
}

/// Whether bellows stand beside `pos`.
pub fn bellows_beside(w: &World, pos: BlockPos) -> bool {
    around(pos).iter().any(|&p| BELLOWS.is(w.block(p)))
}

/// Whether the furnace at `pos` burns hot: lit with hot fuel (`vitals::smelt::HOT_FUEL_TICKS`)
/// and bellows beside it.
pub fn burning_hot(w: &World, pos: BlockPos) -> bool {
    if !crate::blocks::FURNACE.is(w.block(pos)) {
        return false;
    }
    let id = w.block_instance(pos);
    id != 0
        && w.component(Holder::block_id(id), &base_api::components::SMELTING_STATE)
            .is_some_and(|s| s.burn > 0 && sm::hot(s.burn_max, bellows_beside(w, pos)))
}

/// The mould at `state`, by its head's shape.
fn shape_of(state: u32) -> Option<&'static str> {
    MOULD_BLOCKS
        .iter()
        .position(|b| b.is(state))
        .map(|i| sm::MOULDS[i])
}

pub fn register(r: &mut Registry) {
    r.block(&BELLOWS)
        .sound_group("wood")
        .texture("bellows_side")
        .top("bellows_top")
        .bottom("bellows_top")
        .model_box([1.0, 0.0, 2.0], [15.0, 10.0, 14.0])
        .hardness(1.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::tabs::FUNCTIONAL);
    r.shaped(&CRAFTING, "bellows", 1, &["ppp", "lll", "psp"], &[('p', "#planks"), ('l', "#leather"), ('s', "#stick")]);

    r.instance_component(&CAST, Cast { done: 0.0 }, component_flags::SAVE | component_flags::SYNC_MENU);
    r.system(&CAST, tick);
    for (i, block) in MOULD_BLOCKS.iter().enumerate() {
        let unfired = UNFIRED[i];
        r.item(Item::generated(unfired.into()))
            .tag("unfired_moulds")
            .tab(&crate::tabs::INGREDIENTS);
        r.shaped(&CRAFTING, unfired, 1, PATTERNS[i], &[('c', "#clay")]);
        let shape = sm::MOULDS[i];
        r.shapeless(&SMELTING, &format!("mould_{shape}"), 1, &[unfired], 200);
        r.block(block)
            .sound_group("stone")
            .texture("mould_side")
            .top(&format!("mould_{shape}_top"))
            .bottom("mould_side")
            .model_box([1.0, 0.0, 1.0], [15.0, 6.0, 15.0])
            .hardness(1.0)
            .mined_by("pickaxe", tool_tier::HAND)
            .tag("moulds")
            .tab(&crate::tabs::FUNCTIONAL)
            .attach_with(&CAST, Cast { done: 0.0 })
            .inventory(2)
            .opens(&MOULD_MENU);
    }
    r.menu(&MOULD_MENU)
        .group(
            "input",
            Slots::single()
                .accepts("#ingots/copper")
                .limit(sm::CAST_INGOTS)
                .holder(),
        )
        .group("output", Slots::single().output().holder())
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
                        ui.group("input");
                        ui.recipe_arrow_bind(|c| c.menu_component(&CAST).map_or(0.0, |m| m.done));
                        ui.group("output");
                        ui.spacer();
                    });
                    ui.player_inventory(true);
                },
            );
        });
}

/// Every mould of a region: with its ingots in, its head's place empty and a hot furnace beside,
/// the cast goes on; anything short of that starts it over.
fn tick(b: &mut SystemBatch<Cast>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        let Some(pos) = w.block_location(id.entity) else { continue };
        let Some(shape) = shape_of(w.block(pos)) else { continue };
        crate::crafts::push_out(&w, id, "input", |i| COPPER_INGOTS.on_item(i));
        let inv = w.inventory(id);
        let (input, output) = (inv.group("input"), inv.group("output"));
        let st = input.get(0);
        let ready = COPPER_INGOTS.on_item(st.item)
            && st.count >= sm::CAST_INGOTS
            && output.get(0).is_empty()
            && around(pos).iter().any(|&p| burning_hot(&w, p));
        if !ready {
            s.done = 0.0;
            continue;
        }
        s.done = (s.done + ticks as f32 / (sm::cast_secs() * vitals::TICKS as f32)).min(1.0);
        if s.done >= 1.0 {
            output.set(0, crate::knapping::cast_head(&w, shape, "copper"));
            input.set(0, st.shrink(sm::CAST_INGOTS));
            s.done = 0.0;
            weft_sdk::info!("casting: a copper {shape} cast at {pos:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moulds_match_the_shapes() {
        for (i, shape) in sm::MOULDS.iter().enumerate() {
            assert!(MOULD_BLOCKS[i].name().ends_with(&format!("mould_{shape}")));
            assert_eq!(UNFIRED[i], format!("unfired_mould_{shape}"));
        }
        // Each shape its own: no two patterns the same.
        assert!(PATTERNS[0] != PATTERNS[1] && PATTERNS[1] != PATTERNS[2] && PATTERNS[0] != PATTERNS[2]);
    }
}

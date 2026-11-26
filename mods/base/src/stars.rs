//! The star materials (reports/Research round 17.md, Addendum 2 §2; Ronald: "Yes, as designed"):
//! what only a big impact brings. Three real, one fantasy:
//! - **star steel**: the impact's nickel-iron core, its etched plates the Widmanstätten pattern; a
//!   tier between steel and diamond (crafting's `star_steel`), smelted from the core's lumps;
//! - **peridot**: olivine gems in nickel-iron, from a pallasite (1 impact in 4); the pallasite
//!   block is decor, or crafts into peridot;
//! - **tektite**: the ground melted and thrown out, moldavite's green glass; a razor edge for a
//!   newcomer (crafting's `tektite`);
//! - **the Starheart**: a fragment of the core that never cools (1 impact in 3): a star lantern's
//!   light, cold and without warmth, so it never stands in for a fire.
//!
//! Terrain's live carve (`impact.rs`, its branch) places the core; `core_block` decides each core
//! block as it goes: the Starheart at the centre when the impact has one, pallasite when it is a
//! pallasite, else star steel ore.

use std::sync::Mutex;

use vitals::events as ev;
use weft_sdk::*;

pub static STAR_STEEL_ORE: Block = Block::named("star_steel_ore");
pub static PALLASITE: Block = Block::named("pallasite");
pub static STAR_LANTERN: Block = Block::named("star_lantern");
/// Tektite lying in a crater's strewn field (Terrain's carve places it): a few green shards.
pub static LOOSE_TEKTITE: Block = Block::named("loose_tektite");
pub static STAR_STEEL_LUMP: Item = Item::named("star_steel_lump");
pub static STAR_STEEL_INGOT: Item = Item::named("star_steel_ingot");
pub static PERIDOT: Item = Item::named("peridot");
pub static TEKTITE: Item = Item::named("tektite");
pub static STARHEART: Item = Item::named("starheart");

/// The core being placed: (dimension, a Starheart still to place, pallasite blocks still to place).
static CORE: Mutex<Option<(u32, bool, u32)>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    // The core: an iron pick to mine it (the iron age), lumps for the furnace. Its `heart`
    // state holds the Starheart, faintly lit.
    r.block(&STAR_STEEL_ORE)
        .prop("heart", 0..=1u32)
        .sound_group("stone")
        .texture("star_steel_ore")
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::IRON)
        .states(|p, s| if p.is("heart", "1") { s.light_rgb(5, 6, 8) } else { s })
        .tag("star_ores")
        .tab(&crate::tabs::NATURAL);
    r.loot(&STAR_STEEL_ORE)
        .pool(
            Pool::new()
                .when(Cond::harvested())
                .items("star_steel_lump", ev::STAR_STEEL_PER_BLOCK.0, ev::STAR_STEEL_PER_BLOCK.1),
        )
        .pool(
            Pool::new()
                .when(Cond::harvested())
                .when(Cond::state("heart=1"))
                .item("starheart"),
        );
    crate::first::loose(
        r,
        &LOOSE_TEKTITE,
        &[],
        "glass",
        "loose_tektite",
        &[([3.0, 0.0, 4.0], [7.0, 2.0, 8.0]), ([9.0, 0.0, 9.0], [12.0, 1.5, 12.0]), ([8.0, 0.0, 3.0], [10.0, 1.0, 5.0])],
    );
    r.loot(&LOOSE_TEKTITE)
        .pool(Pool::new().items("tektite", ev::TEKTITE_PER_SHARDS.0, ev::TEKTITE_PER_SHARDS.1));
    r.block(&PALLASITE)
        .sound_group("stone")
        .texture("pallasite")
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::STONE)
        .tag("decor")
        .tab(&crate::tabs::BUILDING);
    r.shapeless(&crate::recipes::CRAFTING, "peridot", ev::PERIDOT_PER_PALLASITE, &["pallasite"], 0);
    r.block(&STAR_LANTERN)
        .sound_group("metal")
        .texture("star_lantern")
        // A lantern a hand tall in a cage (Art's `models/block/star_lantern.json`, picked up by
        // its name): not a solid cube, see-through, so its neighbours keep their faces.
        .not_solid()
        .cutout()
        .hardness(1.0)
        .mined_by("pickaxe", weft_sdk::abi::tool_tier::HAND)
        .light_rgb(10, 12, 15)
        .tag("lights")
        .tab(&crate::tabs::FUNCTIONAL);
    r.shaped(&crate::recipes::CRAFTING, "star_lantern", 1, &["i", "s", "i"], &[('i', "#ingots/iron"), ('s', "starheart")]);
    r.item(&STAR_STEEL_LUMP)
        .tag("star_steel_lump")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&STAR_STEEL_INGOT)
        .tag("ingots/star_steel")
        .tab(&crate::tabs::INGREDIENTS);
    r.shapeless(&crate::recipes::SMELTING, "star_steel_ingot", 1, &["#star_steel_lump"], ev::STAR_STEEL_SMELT_TICKS);
    r.item(&PERIDOT)
        .tag("gems/peridot")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&TEKTITE)
        .tag("tektite")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&STARHEART)
        .tag("starheart")
        .max_stack(1)
        .tab(&crate::tabs::INGREDIENTS);
}

/// A big impact's core is about to be placed in this dimension: whether it holds a Starheart
/// and how many pallasite blocks (`vitals::events`, by the impact's seed).
pub fn core_begins(w: &World, seed: u64) {
    let (heart, pallasite) = ev::core_kind(seed);
    weft_sdk::info!("stars: the core holds {} and {pallasite} pallasite", if heart { "a Starheart" } else { "no Starheart" });
    *CORE.lock().unwrap() = Some((w.dimension(), heart, pallasite));
}

/// Terrain's carve calls this on each core block it placed (centre first): the Starheart, then
/// pallasite, then star steel ore as placed.
#[allow(dead_code)] // Called by Terrain's carve (`impact::Loot::hot`) once its branch is in.
pub fn core_block(w: &World, pos: [i32; 3]) {
    let mut core = CORE.lock().unwrap();
    let dim = w.dimension();
    let Some((_, heart, pallasite)) = core.as_mut().filter(|c| c.0 == dim) else { return };
    if *heart {
        *heart = false;
        if let Some(s) = state_with(STAR_STEEL_ORE.id(), "heart", "1") {
            w.set_block(pos, s);
        }
    } else if *pallasite > 0 {
        *pallasite -= 1;
        w.set_block(pos, PALLASITE.id());
    }
}

//! The base game. It is an ordinary mod: everything here goes through `weft_sdk`, nothing in the
//! engine knows about stone, water or zombies.

mod advancements;
mod beds;
mod blocks;
mod book;
mod clothing;
mod cooking;
mod discovery;
mod farming;
mod growth;
mod hints;
mod hud;
mod items;
mod levels;
mod machines;
mod mobs;
mod parts;
mod player;
mod recipes;
mod redstone;
mod shaderpack;
mod sounds;
mod survival;
mod weather;
mod wild;
mod worldgen;

use weft_sdk::*;

pub use base_api::{CLOUDS, FLATLANDS, MOON, OVERWORLD, STARS, SUN, tabs};

declare_mod!(deps: [], init: init);

fn init(r: &mut Registry) {
    // Vanilla's creative tabs; blocks and items join them with `.tab(..)`.
    for (tab, icon) in [
        (&tabs::BUILDING, &Item::named("bricks")),
        (&tabs::COLORED, &Item::named("red_stained_glass")),
        (&tabs::NATURAL, &Item::named("grass")),
        (&tabs::FUNCTIONAL, &Item::named("crafting_table")),
        (&tabs::TOOLS, &Item::named("iron_pickaxe")),
        (&tabs::COMBAT, &Item::named("iron_sword")),
        (&tabs::FOOD, &Item::named("apple")),
        (&tabs::INGREDIENTS, &items::IRON_INGOT),
        (&tabs::REDSTONE, &Item::named("redstone")),
    ] {
        r.creative_tab(tab, icon);
    }
    blocks::register(r);
    wild::register(r);
    items::register(r);
    player::register(r);
    survival::register(r);
    levels::register(r);
    machines::register(r);
    clothing::register(r);
    cooking::register(r);
    recipes::register(r);
    book::register(r);
    discovery::register(r);
    parts::register(r);
    sounds::register(r);
    mobs::register(r);
    worldgen::register(r);
    weather::register(r);
    growth::register(r);
    farming::register(r);
    advancements::register(r);
    hints::register(r);
    beds::register(r);
    hud::register(r);
    redstone::register(r);
    shaderpack::register(r);
    shaderpack::register_for_scripts(r);
    shaderpack::register_shafts(r);
    // Its terrain is described in `worldgen`.
    // A year of four 14-day seasons (notes/SURVIVAL.md): sixteen hours of daylight at midsummer,
    // eight at midwinter.
    r.dimension(&OVERWORLD)
        .player_spawn()
        .sky_above(16)
        .day_length(24000)
        .seasons(vitals::calendar::YEAR_DAYS, base_api::calendars::FOUR_SWING)
        .set(&base_api::SEASONS, base_api::calendars::FOUR)
        .weather();
    // Second dimension on purpose: nothing in the engine may assume there is only one world.
    r.dimension(&FLATLANDS)
        .generator(worldgen::flat)
        .bare()
        .spawn(|_| [0, 3, 0])
        .height(-2, 8)
        .sky_above(1)
        .fixed_time(6000)
        .gravity(20.0)
        .sky(0xb0c8ff, 0xe0e8ff);
    for dim in [&OVERWORLD, &FLATLANDS] {
        r.sky_stars(dim, &STARS).count(1500);
        r.sky_clouds(dim, &CLOUDS).texture("clouds");
        r.sky_body(dim, &SUN)
            .texture("sun")
            .additive()
            .size(33.0)
            .glow(0xFFE633);
        r.sky_body(dim, &MOON)
            .texture("moon_phases")
            .additive()
            .frames(4, 2)
            .size(23.0)
            .phase(0.5);
    }
}

//! The base game. It is an ordinary mod: everything here goes through `weft_sdk`, nothing in the
//! engine knows about stone, water or zombies.

mod advancements;
mod beds;
mod blocks;
mod clothing;
mod cooking;
mod crafts;
mod death;
mod farming;
mod growth;
mod hints;
mod hud;
mod items;
mod land;
mod levels;
mod machines;
mod mobs;
mod player;
mod recipes;
mod redstone;
mod seasons;
mod look;
mod shaderpack;
mod sounds;
mod survival;
mod video;
mod weather;
mod weather_sound;
mod wild;
mod window;
mod worldgen;

use weft_sdk::*;

pub use base_api::{CLOUDS, FLATLANDS, MOON, OVERWORLD, STARS, SUN, tabs};

declare_mod!(deps: [], init: init);

// ------------------------------------------------------------------ the engine's bridge

type Query = unsafe extern "C" fn(query: weft_sdk::abi::Str, arg: weft_sdk::abi::Str) -> weft_sdk::abi::Str;
static BRIDGE: std::sync::OnceLock<Query> = std::sync::OnceLock::new();

/// The engine hands the mod its bridge when the mod loads (chiacraft): what Weft's interface
/// doesn't carry, such as asking the world to sign an achievement.
///
/// # Safety
/// `q` must be the engine's `bridge::query`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chiacraft_bridge(q: Query) {
    let _ = BRIDGE.set(q);
}

/// Asks the engine's bridge `q` with `arg` (empty without a bridge: plain Weft).
pub(crate) fn bridge(q: &str, arg: &str) -> String {
    use weft_sdk::abi::Str;
    BRIDGE
        .get()
        .map(|f| unsafe { f(Str::new(q), Str::new(arg)).as_str().to_string() })
        .unwrap_or_default()
}

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
    crafts::register(r);
    recipes::register(r);
    sounds::register(r);
    mobs::register(r);
    worldgen::register(r);
    weather::register(r);
    weather_sound::register(r);
    seasons::register(r);
    growth::register(r);
    farming::register(r);
    advancements::register(r);
    hints::register(r);
    death::register(r);
    beds::register(r);
    hud::register(r);
    redstone::register(r);
    look::register(r);
    shaderpack::register(r);
    shaderpack::register_for_scripts(r);
    shaderpack::register_shafts(r);
    video::register(r);
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
            .glow(0xFFE633)
            // Its path leaned 35° to the south (Bliss's and Complementary's sun paths lean about
            // 40°): noon stands 55° up, so shadows stay long all day and slopes and walls keep a
            // lit and a shaded side (Ronald, 11-04: "it all seems so flat"). The engine's sun
            // light and shadows follow this body.
            .tilt(35.0);
        // The moon on the same leaned path, lagging the day as the real one does: a turn every 8/7
        // of a day, so it rises later each night and goes round the sky once against the sun in
        // the eight days of its phases (one picture a day). Full (day 0) it is opposite the sun and
        // up all night; at a half moon half the night; new (day 4) it is up with the sun and the
        // night has none. The night's light and shadows come from it (Ronald, 11-04: a full moon
        // lights the night, no moon is dark).
        r.sky_body(dim, &MOON)
            .texture("moon_phases")
            .additive()
            .frames(4, 2)
            .size(23.0)
            .orbit(vitals::calendar::DAY_TICKS as f32 * 8.0 / 7.0)
            .phase(0.5)
            .tilt(35.0);
    }
}

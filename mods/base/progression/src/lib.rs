//! Progression and balance, checked (`notes/PROGRESSION.md`, "How it stays balanced when things
//! are added"; `reports/Balance model.md`, Part 3 and 5): every item, block, recipe, tool part and
//! species tagged with its age and path (`content`, `animals_core::danger`), the progression graph
//! (`graph`), the combat matrix (`matrix`), tweak bounds, and a first making budget per age. Each
//! check is a function returning its problems; the tests fail on any, and the sheet
//! (`PROGRESSION_SHEET.md`) lists them with the numbers: `cargo run -p progression --bin sheet >
//! mods/base/progression/PROGRESSION_SHEET.md`.
//!
//! The numbers themselves live in `vitals::curve`, `vitals::combat`, `vitals::web` and
//! `animals_core::danger`; this crate only tags and checks.

pub mod content;
pub mod graph;
pub mod loot;
pub mod matrix;
pub mod sheet;

/// Every check, by name, with what it found wrong (empty: it passes).
pub fn checks() -> Vec<(&'static str, Vec<String>)> {
    let g = graph::solve();
    vec![
        ("every item and block tagged, every made one with a recipe", graph::tagging(&g)),
        ("every recipe in the mods' sources declared", graph::declared(&g)),
        ("everything reachable from the start", graph::reach(&g)),
        ("nothing at tier N needs anything above N", graph::tiers(&g)),
        ("each age's key by each declared route", graph::keys(&g)),
        ("path parity: each age's key by each route within ×1.15 of its cheapest, in EM", graph::path_parity(&g)),
        ("every tool-part material tagged, made from its tier or below", graph::materials(&g)),
        ("each age's making inside its budget", graph::budget(&g)),
        ("every size and role inside its bands, floors hold", matrix::sizes()),
        ("every species inside its bands at every level, floors hold", matrix::species()),
        ("tames capped by their saddle and ceiling", matrix::tames()),
        ("tweaks inside ×0.75-×1.25 with a reason", matrix::tweaks()),
        ("grade and gems at most +60% over a plain item", matrix::gem_ceiling()),
        ("stagger: same-age fights break every part, knockdowns at most a third of a fight", matrix::stagger()),
        ("bows on the curve: full draw a shot a second, a reference bow's arrows the reference's damage", matrix::ranged()),
        ("saddles lift tames by their fitting, as the tame model does, tooltip in step", matrix::saddles()),
        (
            "loot per hour: kill drops priced from their cheapest kill, no kill paying over its time (+30 %, +50 % more for what fights back)",
            loot::check(&g),
        ),
    ]
}

//! The map paints what tops the ground (`land::top`, `vitals::waystones::ground_colour`): every top
//! the land makes has a colour, and seed 1's grassy mountains round 0,0 read green, not grey stone
//! (Ronald 11-22: "what is that gray area? Isn't it supposed to be green?").

use base_api::land as l;
use vitals::waystones as n;

#[test]
fn every_top_has_a_colour() {
    let f = l::fields(1);
    let mut seen = std::collections::BTreeSet::new();
    for z in (-2048..2048).step_by(37) {
        for x in (-2048..2048).step_by(41) {
            let (_, _, top) = l::top(&f, x, z);
            seen.insert(top.name());
        }
    }
    for name in &seen {
        assert!(*name == "water" || n::GROUND_COLOURS.iter().any(|e| e.0 == *name), "{name} has no map colour");
    }
    assert!(seen.contains("grass") && seen.contains("water"), "{seen:?}");
}

#[test]
fn grassy_mountains_are_green_on_the_map() {
    // Round 0,0 on seed 1: mountains, about a third alpine grass, the rest snow and a few granite
    // faces (the shot's green was the test platform's own grass, built over them).
    let f = l::fields(1);
    let green = |c: [u8; 3]| c[1] > c[0] && c[1] > c[2];
    let (mut grass, mut snow) = (0, 0);
    for z in -40..40 {
        for x in -40..40 {
            let (_, biome, top) = l::top(&f, x, z);
            let c = n::ground_colour(top.name(), biome);
            match top {
                l::Top::Ground("grass") => {
                    assert!(green(c), "{x} {z} {biome}: {c:?}");
                    grass += 1;
                }
                l::Top::Snow => {
                    assert!(c.iter().all(|&v| v > 0xd0), "{x} {z}: {c:?}");
                    snow += 1;
                }
                _ => {}
            }
        }
    }
    assert!(grass > 200 && snow > 200, "{grass} grass, {snow} snow round 0,0");
}

//! Each creature's box (its hitbox: what a sword, an arrow or a click hits, what a rider sits on,
//! and what bumps into the world), by its model (`assets/models/<model>.geo.json`). The box and
//! the model are drawn and scaled together (`World::set_scale`: genes and growth), so a box that
//! fits its model at its own size fits it at every size.
//!
//! A box is square and stands on the animal's feet, round its middle, so for a long animal it
//! can't cover nose to tail. As Minecraft's animals do, its width is about the geometric mean of
//! the model's width and length and its height the model's top (`tests/boxes.rs` checks every one
//! against its model: height 0.75 to 1.35 times the top, width 0.75 to 1.5 times the mean, the
//! model centred within 0.3 of the width; the tiniest get a tenth of a block more to click).
//! Ronald's hitbox check (2026-11-05) found the big Cretaceous animals' boxes half their models'
//! size (an iguanodont 1.8 wide round a 9-block body, a sauropod 4.2 high under a 9-block neck),
//! and the dogs, wolves and jaguars' a little small.

/// A creature's box by its model: width and height in blocks, at the model's own size.
pub const BOXES: &[(&str, (f32, f32))] = &[
    ("chihuahua", (0.35, 0.42)),
    ("chihuahua_pup", (0.25, 0.26)),
    ("cat", (0.35, 0.44)),
    ("capybara", (0.7, 0.6)),
    ("raptor", (0.7, 0.9)),
    ("utahraptor", (2.0, 2.3)),
    ("wyvern", (5.0, 5.5)),
    ("wolf", (0.7, 1.0)),
    ("jaguar", (0.9, 1.0)),
    ("hare", (0.4, 0.45)),
    ("mouse", (0.2, 0.15)),
    ("vole", (0.2, 0.15)),
    ("kryptobaatar", (0.2, 0.15)),
    ("lizard", (0.2, 0.1)),
    ("deer", (0.9, 1.3)),
    ("fox", (0.5, 0.6)),
    ("protoceratops", (0.8, 0.75)),
    ("reindeer", (1.0, 1.6)),
    ("coyote", (0.55, 0.8)),
    ("caiman", (1.2, 0.26)),
    ("psittacosaurus", (0.7, 0.8)),
    ("ankylosaur", (3.0, 1.3)),
    ("iguanodont", (3.8, 2.7)),
    ("citipati", (0.9, 1.7)),
    ("sauropod", (4.5, 5.5)),
    ("raven", (0.35, 0.35)),
    ("small_bird", (0.2, 0.15)),
    ("owl", (0.4, 0.5)),
    ("eagle", (0.55, 0.55)),
    ("vulture", (0.45, 0.45)),
    ("gull", (0.4, 0.4)),
    ("early_bird", (0.35, 0.35)),
    ("fish", (0.3, 0.2)),
    ("egg", (0.1, 0.14)),
    ("nest", (0.9, 0.15)),
];

/// Boxes the height rule doesn't hold to, and why.
pub const LOW: &[(&str, &str)] = &[(
    "sauropod",
    "its raised neck reaches 8.6 blocks, above anything a player on the ground can reach (about \
     5.5); a box that tall found no room under the fern forest's crowns and stood still there \
     (PC run aw_walk_path), so its box stops at its back and the base of its neck",
)];

/// Boxes the width rule doesn't hold to, and why: what the model's bounds count isn't body.
pub const LOOSE: &[(&str, &str)] = &[(
    "wyvern",
    "its wings (folded, 8 blocks across; 24 spread) and tail are most of its bounds: the box is \
     its body and the wings' roots, so it can land and walk where a dragon fits",
)];

/// The box of the creature drawn with `model`.
pub fn of(model: &str) -> (f32, f32) {
    BOXES
        .iter()
        .find(|(m, _)| *m == model)
        .map(|&(_, b)| b)
        .unwrap_or_else(|| panic!("no box for the model {model} in animals_core::bodies::BOXES"))
}

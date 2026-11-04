//! What a wild animal needs, as the numbers every part reads (`notes/ECOLOGY.md`, parts 4-7):
//! how much food fills it, by mass, the same for a kill, a mouthful off the ground and a carcass;
//! how fed one arrives; how long it lasts with nothing.
//!
//! The mod's goals (`mods/animals/src/hunt.rs`, `forage.rs`) and the species sheet read them here.

use crate::species::{SPECIES, Species};

/// A meal of this share of its own mass fills it (its whole food stat): a fox's 550 g, so a hare
/// more than fills it and a mouse is a twentieth. Real hunters eat about this at a sitting (a red
/// fox takes about a tenth of its mass a day; a wolf up to a fifth at a kill), and a grazer's tenth
/// of fresh grass is its 2-3% of dry matter.
pub const MEAL: f32 = 0.1;

/// The share of its food stat that `kg` of food fills in an eater of `own` kg (at most all of it).
pub fn fills(own: f32, kg: f32) -> f32 {
    if own <= 0.0 {
        return 0.0;
    }
    (kg / (own * MEAL)).min(1.0)
}

/// Share of its food a wild one has when it arrives where nothing it eats grows this season (a
/// plant eater), or where its prey are thinnest (a hunter); the rest comes with the place's food.
pub const ARRIVES_FED: f32 = 0.3;

/// Seconds a wild one lives on an empty stomach at full health before it starves (a game day).
pub const STARVES_SECS: f32 = 1200.0;

/// The prey of `hunter` that live in `biome`: species of other families it takes alone, by their
/// grown masses (farm animals aren't here: they don't come and go with the seasons).
pub fn prey_in<'a>(hunter: &'a Species, biome: &'a str) -> impl Iterator<Item = &'static Species> + 'a {
    SPECIES.iter().filter(move |o| {
        o.family.name != hunter.family.name
            && hunter.takes(hunter.bio.mass, o.bio.mass)
            && o.habitats.iter().any(|&(b, _)| b == biome)
    })
}

/// How many of its biggest prey at home fill `hunter` (`fills`): one hunt, or several small
/// ones. `None`: no prey of its sizes lives where it does (it waits for them to come).
pub fn kills_to_fill(hunter: &Species) -> Option<f32> {
    let biggest = hunter
        .habitats
        .iter()
        .flat_map(|&(biome, _)| prey_in(hunter, biome).map(|o| o.bio.mass))
        .chain(
            crate::species::OTHER_PREY
                .iter()
                .filter(|_| hunter.habitats.iter().any(|&(b, _)| b.starts_with("base:")))
                .filter(|&&(_, kg)| hunter.takes(hunter.bio.mass, kg))
                .map(|&(_, kg)| kg),
        )
        .reduce(f32::max)?;
    Some(1.0 / fills(hunter.bio.mass, biggest))
}

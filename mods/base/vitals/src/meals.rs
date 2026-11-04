//! Meals from several ingredients: a stew cooked in a pot (`base:cooking_pot`). It is what went
//! into it: each ingredient goes into the stomach as its own portion (raw meat as its cooked row),
//! stronger together (`BONUS`), so a stew of three foods counts as three foods toward being well
//! fed. It is hot off the fire like any cooked food, and keeps as long as its shortest-keeping
//! ingredient from when it was cooked.

use crate::food::{self, FOODS, NUTRIENTS};

/// How much more a meal gives than its ingredients eaten apart (`notes/SURVIVAL.md` proposed
/// x1.5 for cooking; Valheim's meals about x1.3).
pub const BONUS: f32 = 1.25;
/// Ingredients a stew takes.
pub const MIN: usize = 2;
pub const MAX: usize = 4;
/// Ticks a pot takes to cook a stew (a furnace's smelt).
pub const COOK_TICKS: u32 = 200;
/// How much longer food takes over a campfire than in a furnace (an open fire, no fuel needed).
pub const CAMPFIRE_PACE: f32 = 1.5;

/// The food an ingredient is in a stew: raw meat cooks (`base:beef` is `base:cooked_beef`), the
/// rest stays as it is (a spider's eye stays poisonous).
pub fn cooked(food: usize) -> usize {
    let f = &FOODS[food];
    if !f.raw {
        return food;
    }
    let (ns, n) = f.name.split_once(':').unwrap_or(("base", f.name));
    food::find(&format!("{ns}:cooked_{n}")).unwrap_or(food)
}

/// The ingredients of a stew (`FOODS` indexes), packed in a stack's component: 16 bits each,
/// index + 1, 0 for none.
pub fn pack(foods: &[usize]) -> u64 {
    foods
        .iter()
        .take(MAX)
        .enumerate()
        .fold(0, |p, (i, &f)| p | ((f as u64 + 1) << (16 * i)))
}

pub fn unpack(packed: u64) -> Vec<usize> {
    (0..MAX)
        .map(|i| ((packed >> (16 * i)) & 0xffff) as usize)
        .filter(|&f| f > 0 && f <= FOODS.len())
        .map(|f| f - 1)
        .collect()
}

/// A stew's ingredients as they are in it (cooked).
pub fn ingredients(packed: u64) -> Vec<usize> {
    unpack(packed).into_iter().map(cooked).collect()
}

/// Its size in the stomach, units.
pub fn units(foods: &[usize]) -> f32 {
    foods.iter().map(|&f| FOODS[f].units()).sum()
}

/// Grams it weighs.
pub fn grams(foods: &[usize]) -> f32 {
    foods.iter().map(|&f| FOODS[f].grams).sum()
}

/// Grams of each nutrient it gives fresh, the bonus included.
pub fn nutrients(foods: &[usize]) -> [f32; NUTRIENTS] {
    let mut n = [0.0; NUTRIENTS];
    for &f in foods {
        for (i, g) in FOODS[f].portion().iter().enumerate() {
            n[i] += g * BONUS;
        }
    }
    n
}

/// Game days it keeps fresh, then stale: its shortest-keeping ingredient's.
pub fn keeps(foods: &[usize]) -> (f32, f32) {
    foods
        .iter()
        .map(|&f| FOODS[f].keeps)
        .fold((f32::MAX, f32::MAX), |a, k| (a.0.min(k.0), a.1.min(k.1)))
}

/// The chance its worst ingredient gives food poisoning at that freshness.
pub fn poisoning_chance(foods: &[usize], fresh: food::Freshness) -> f32 {
    foods
        .iter()
        .map(|&f| food::poisoning_chance(&FOODS[f], fresh))
        .fold(0.0, f32::max)
}

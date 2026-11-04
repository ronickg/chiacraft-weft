//! What a wild animal's body gives to eat (`reports/Research round 3.md`, B2): deer and reindeer
//! venison, hares their meat, trout and pacu their fish, each a raw item (`animals:<meat>`) that
//! cooks to `animals:cooked_<meat>` (their nutrition is the vitals' food table's,
//! `vitals::food::FOODS`, from USDA rows). The animals mod drops them when one dies (`lib.rs`), as
//! it does hides (`hides`).
//!
//! How much: a carcass is about 45% meat by its live mass (dressed and boned: a 70 kg deer gives
//! about 30 kg). An item is a meal, not a weight, so the count is game-sized as hides' are: the
//! cube root of the meat's kilograms, a hare's or a fish's one or two, a deer's three or four, a
//! red deer's four or five.

use crate::species::SPECIES;

/// Share of live mass that is meat.
pub const YIELD: f32 = 0.45;

/// Species (`species::SPECIES` name) and the meat it gives (the item's name without its mod).
pub const MEAT: &[(&str, &str)] = &[
    ("white_tailed_deer", "venison"),
    ("red_deer", "venison"),
    ("reindeer", "venison"),
    ("hare", "hare"),
    ("mountain_hare", "hare"),
    ("brown_trout", "trout"),
    ("pacu", "pacu"),
];

/// The meats, each once: the raw item, and cooked as `cooked_<meat>`.
pub const MEATS: &[&str] = &["venison", "hare", "trout", "pacu"];

/// Portions a grown one of `mass` kg gives, fewest to most.
pub fn portions(mass: f32) -> (u32, u32) {
    let c = (mass * YIELD).cbrt();
    (c.floor().max(1.0) as u32, c.ceil().max(1.0) as u32)
}

/// Each species that gives meat, its meat, and how many portions.
pub fn drops() -> impl Iterator<Item = (&'static str, &'static str, u32, u32)> {
    MEAT.iter().filter_map(|&(name, meat)| {
        let s = SPECIES.iter().find(|s| s.name == name)?;
        let (lo, hi) = portions(s.bio.mass);
        Some((name, meat, lo, hi))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meat_is_a_species_and_a_food() {
        assert_eq!(drops().count(), MEAT.len(), "every MEAT species is in species::SPECIES");
        for &(_, meat) in MEAT {
            assert!(MEATS.contains(&meat), "{meat} isn't in MEATS");
        }
        for meat in MEATS {
            for item in [format!("animals:{meat}"), format!("animals:cooked_{meat}")] {
                assert!(vitals::food::FOODS.iter().any(|f| f.name == item), "{item} has no row in vitals::food::FOODS");
            }
        }
    }

    #[test]
    fn a_deer_gives_a_few_meals_and_a_hare_one_or_two() {
        assert_eq!(portions(70.0), (3, 4));
        assert_eq!(portions(3.8), (1, 2));
        assert_eq!(portions(0.5), (1, 1));
    }
}

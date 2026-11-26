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

/// What a dead animal leaves when it gives no meat of its own (`MEAT`): carcass pieces
/// (`animals:carcass`, half a kilogram of carrion each, a scavenger's), as many as its meat would
/// be portions; one under a kilogram leaves none (the beetles have it).
pub const CARCASS: &str = "animals:carcass";
pub const CARCASS_FROM_KG: f32 = 1.0;

/// Carcass pieces a grown one of species `name` leaves, fewest to most (`None`: none).
pub fn carcass(name: &str) -> Option<(u32, u32)> {
    let s = SPECIES.iter().find(|s| s.name == name)?;
    (!MEAT.iter().any(|(n, _)| *n == name) && s.bio.mass >= CARCASS_FROM_KG).then(|| portions(s.bio.mass))
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
    fn carcasses_feed_the_scavengers() {
        // One that gives meat leaves its meat; a big one without, carcass pieces; a mouse, none.
        assert_eq!(carcass("red_deer"), None);
        assert!(carcass("grey_wolf").is_some_and(|(lo, hi)| lo >= 1 && hi >= lo));
        assert_eq!(carcass("wood_mouse"), None);
        // The vulture lives on them, and only carrion eaters are scavengers.
        let scavengers: Vec<&str> = SPECIES
            .iter()
            .filter(|s| s.scavenges())
            .map(|s| s.name)
            .collect();
        assert_eq!(scavengers, ["turkey_vulture"]);
        // The pieces are carrion on the ground (`vitals::forage::DROPPED`).
        assert_eq!(vitals::forage::dropped(CARCASS, false).map(|d| d.0), Some(vitals::food::Category::Carrion));
    }

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

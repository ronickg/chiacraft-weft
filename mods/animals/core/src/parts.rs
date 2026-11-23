//! What a large animal's stagger breaks drop (`armour.rs`, `vitals::combat::breaks_at`): its own
//! parts broken off in the fight (a feather, a claw, a scute), one row a break in turn, instead of
//! a roll of its death loot, which for one with no hide or meat rows is only carcass pieces
//! (Planner 11-22: a Utahraptor's breaks dropped carcass). Every species big enough to stagger
//! (`Size::Large` and up) has rows here or a hide row (`hides::HIDES`), checked below.
//!
//! Which part is the animal's: dromaeosaurs were feathered and carried the big sickle claw on the
//! second toe (Utahraptor, Kirkland et al. 1993); ankylosaurs (Gastonia, Pinacosaurus) and
//! crocodyliforms wore bony scutes (osteoderms); the iguanodonts' thumb spike counts as a claw;
//! the sauropod's thick hide is hide; the wyvern, a fantasy animal, is scaled as a crocodile.

/// Claws and scutes, the items this adds.
pub const CLAW: &str = "animals:claw";
pub const SCUTE: &str = "animals:scute";

/// Species, item, fewest and most a break drops. A species' rows are dropped in turn, break by
/// break.
pub const BREAKS: &[(&str, &str, u32, u32)] = &[
    ("utahraptor", CLAW, 1, 1),
    ("utahraptor", "base:feather", 2, 4),
    ("wyvern", SCUTE, 1, 2),
    ("wyvern", CLAW, 1, 1),
    ("gastonia", SCUTE, 1, 2),
    ("pinacosaurus", SCUTE, 1, 2),
    ("crocodyliform", SCUTE, 1, 1),
    ("iguanodont", CLAW, 1, 1),
    ("cedarosaurus", "base:hide", 2, 3),
];

/// What `species`'s break number `n` (1, 2, ...) drops: its rows in turn; None: it has no rows
/// (a roll of its loot, as before).
pub fn break_drop(species: &str, n: u32) -> Option<(&'static str, u32, u32)> {
    let rows: Vec<_> = BREAKS.iter().filter(|r| r.0 == species).collect();
    (!rows.is_empty()).then(|| {
        let r = rows[(n.max(1) as usize - 1) % rows.len()];
        (r.1, r.2, r.3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_staggering_species_breaks_something_of_its_own() {
        for s in crate::species::SPECIES.iter() {
            let staggers = vitals::combat::staggers(vitals::combat::Size::of_mass(s.bio.mass));
            let hide = crate::hides::HIDES.iter().any(|h| h.0 == s.name);
            if staggers {
                assert!(hide || break_drop(s.name, 1).is_some(), "{}: breaks drop only carcass", s.name);
            }
        }
    }

    #[test]
    fn rows_take_turns() {
        assert_eq!(break_drop("utahraptor", 1).map(|r| r.0), Some(CLAW));
        assert_eq!(break_drop("utahraptor", 2).map(|r| r.0), Some("base:feather"));
        assert_eq!(break_drop("utahraptor", 3).map(|r| r.0), Some(CLAW));
        assert_eq!(break_drop("red_deer", 1), None);
        for r in BREAKS {
            assert!(crate::species::SPECIES.iter().any(|s| s.name == r.0), "{}", r.0);
            assert!(r.2 >= 1 && r.3 >= r.2);
        }
    }
}

//! What a wild animal's body gives for clothes (`vitals::clothing`): furred ones a pelt or a few
//! (`base:fur`), deer and capybaras their hide as leather (`base:leather`), by their size. Real
//! counts: a winter caribou hide is about one parka's back, a hare's pelt a mitten's worth, so a
//! fur coat takes a reindeer or several foxes. The animals mod drops them when one dies (`lib.rs`).

/// Species (`species::SPECIES` name), the item, and how many, fewest to most.
pub const HIDES: &[(&str, &str, u32, u32)] = &[
    ("hare", "base:fur", 1, 1),
    ("mountain_hare", "base:fur", 1, 1),
    ("red_fox", "base:fur", 1, 1),
    ("arctic_fox", "base:fur", 1, 2),
    ("coyote", "base:fur", 1, 2),
    ("grey_wolf", "base:fur", 2, 2),
    ("jaguar", "base:fur", 2, 3),
    ("reindeer", "base:fur", 2, 3),
    ("white_tailed_deer", "base:leather", 1, 2),
    ("red_deer", "base:leather", 1, 3),
    ("capybara", "base:leather", 1, 1),
];

#[cfg(test)]
mod tests {
    use super::HIDES;
    use crate::species::SPECIES;

    #[test]
    fn every_hide_is_a_species() {
        for (name, _, lo, hi) in HIDES {
            assert!(SPECIES.iter().any(|s| s.name == *name), "{name} gives a hide but isn't a species");
            assert!(lo <= hi && *hi > 0, "{name}: {lo} to {hi}");
        }
    }
}

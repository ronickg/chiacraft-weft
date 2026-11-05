//! Where each bird nests (`notes/ANIMALS.md`, "Nests for the birds"). A bird lays its clutch in a
//! nest it builds at its kind of site (`Site`): on a treetop's leaves, on a high bare ledge, or on
//! the ground; it falls back to the ground where its site isn't near. Its eggs are brooded: a
//! parent sits on them (off screen), so in a bird's nest they are held at `life::BROODING`
//! whatever the air, as a real clutch is. Its young hatch there and leave as fledglings.

/// A kind of nest site.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Site {
    /// On top of a tree's leaves.
    Tree,
    /// On the highest bare ledge near (a cliff, a crag), else a tree.
    Cliff,
    /// On the ground where she is (a scrape, a mound, a cave floor, a crevice).
    Ground,
}

/// Each bird species (`species::SPECIES` name) and where it nests.
pub const SITES: &[(&str, Site)] = &[
    // Stick nests high in trees, or on cliffs.
    ("raven", Site::Tree),
    // In cavities, eaves and dense shrubs; a tree's crown stands in.
    ("house_sparrow", Site::Tree),
    // In rock crevices and among boulders on the ground.
    ("snow_bunting", Site::Ground),
    // In old hawk and crow nests in trees; it builds none of its own.
    ("great_horned_owl", Site::Tree),
    // A scrape on a raised mound of the open tundra.
    ("snowy_owl", Site::Ground),
    // Mostly on cliff ledges, sometimes in a big tree.
    ("golden_eagle", Site::Cliff),
    // Stick nests in palms and lone savanna trees.
    ("savanna_hawk", Site::Tree),
    // No nest: the eggs on the bare floor of a cave, a hollow log or a thicket.
    ("turkey_vulture", Site::Ground),
    // A grass scrape on the ground, in colonies by the sea.
    ("herring_gull", Site::Ground),
    // Est.: in trees, as its feet and the Jehol forests suggest.
    ("confuciusornis", Site::Tree),
    // Its eggs were found half buried in the Gobi's sand, upright (Elzanowski 1981).
    ("gobipteryx", Site::Ground),
];

/// Where species `name` nests, if it is a bird that builds one.
pub fn site(name: &str) -> Option<Site> {
    SITES.iter().find(|(n, _)| *n == name).map(|&(_, s)| s)
}

/// Blocks round its mother a nest site is looked for: across, and up and down.
pub const LOOK: i32 = 12;
pub const LOOK_UP: i32 = 20;
pub const LOOK_DOWN: i32 = 8;
/// A ledge counts as a cliff from this many blocks above the ground under the mother.
pub const CLIFF_ABOVE: i32 = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SPECIES;

    #[test]
    fn every_bird_nests_and_lays() {
        for (name, _) in SITES {
            let s = SPECIES
                .iter()
                .find(|s| s.name == *name)
                .unwrap_or_else(|| panic!("{name}: no such species"));
            assert_eq!(s.gestation(), 0, "{name} nests, so it lays eggs: no gestation");
            assert!(s.incubation() > 0, "{name}: its eggs need an incubation");
        }
        // Every flying bird of the overworld and the Cretaceous has a site (the wyvern and the
        // raptors nest on the ground by `eggs::nest`).
        for s in SPECIES.iter() {
            let bird = s
                .family
                .above
                .iter()
                .any(|a| *a == "Aves" || *a == "Avialae");
            assert!(!bird || site(s.name).is_some(), "{}: a bird with no nest site", s.name);
        }
    }
}

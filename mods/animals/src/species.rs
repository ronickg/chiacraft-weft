//! Every species as data (`notes/ANIMALS.md`, "Species are data"): adding an animal is mostly a
//! row here. Each row is placed in the tree of life as biology places it: its family (with the
//! groups above it) and subfamily, its scientific name (genus and species). Animals breed only
//! within a species (`same_species`): a house cat and a lion share a family, not a species. A
//! domestic species with breeds (dogs) has a row per breed: the breed is its own entity kind
//! with its own model and numbers, the species (genes, behavior, what it is) shared. A wild
//! species is never a breed: Velociraptor's two species are two rows, two species.

//!
//! The table itself is in `animals_core::species` (shared with the website's simulator, compiled
//! to WASM); here each row gets its entity kind.

use std::ops::Deref;

use animals_core::species as data;
pub use animals_core::species::*;
use weft_sdk::Entity;

/// A species' row (`animals_core::species::Species`, through `Deref`) with its entity kind.
pub struct Species {
    pub kind: &'static Entity,
    data: &'static data::Species,
}

impl Deref for Species {
    type Target = data::Species;
    fn deref(&self) -> &data::Species {
        self.data
    }
}

pub static SPECIES: [Species; 7] = [
    Species { kind: &crate::dog::CHIHUAHUA, data: &data::SPECIES[0] },
    Species { kind: &crate::cat::CAT, data: &data::SPECIES[1] },
    Species { kind: &crate::capybara::CAPYBARA, data: &data::SPECIES[2] },
    Species { kind: &crate::raptor::RAPTOR, data: &data::SPECIES[3] },
    Species { kind: &crate::utahraptor::UTAHRAPTOR, data: &data::SPECIES[4] },
    Species { kind: &crate::raptor::OSMOLSKAE, data: &data::SPECIES[5] },
    Species { kind: &crate::wyvern::WYVERN, data: &data::SPECIES[6] },
];

impl Species {
    /// The name of its kind, without the namespace (`entity.animals.<it>`, its saddle).
    pub fn kind_name(&self) -> &'static str {
        self.kind.path()
    }

    /// Its food `item` (an item id), if it eats it.
    pub fn food(&self, item: u32) -> Option<(usize, &'static Food)> {
        let name = weft_sdk::item_name(item);
        self.foods.iter().enumerate().find(|(_, f)| f.item == name)
    }
}

/// Whether some species eats `item`: offered to one that doesn't, it is disliked.
pub fn anyone_eats(item: u32) -> bool {
    SPECIES.iter().any(|s| s.food(item).is_some())
}

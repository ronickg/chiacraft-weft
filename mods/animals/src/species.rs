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
use weft_sdk::EntityKind;

/// A species' row (`animals_core::species::Species`, through `Deref`) with its entity kind.
pub struct Species {
    pub kind: &'static EntityKind,
    data: &'static data::Species,
}

impl Deref for Species {
    type Target = data::Species;
    fn deref(&self) -> &data::Species {
        self.data
    }
}

pub static SPECIES: [Species; 46] = [
    Species { kind: &crate::dog::CHIHUAHUA, data: &data::SPECIES[0] },
    Species { kind: &crate::cat::CAT, data: &data::SPECIES[1] },
    Species { kind: &crate::capybara::CAPYBARA, data: &data::SPECIES[2] },
    Species { kind: &crate::raptor::RAPTOR, data: &data::SPECIES[3] },
    Species { kind: &crate::utahraptor::UTAHRAPTOR, data: &data::SPECIES[4] },
    Species { kind: &crate::raptor::OSMOLSKAE, data: &data::SPECIES[5] },
    Species { kind: &crate::wyvern::WYVERN, data: &data::SPECIES[6] },
    Species { kind: &crate::hare::HARE, data: &data::SPECIES[7] },
    Species { kind: &crate::hare::MOUNTAIN_HARE, data: &data::SPECIES[8] },
    Species { kind: &crate::mouse::WOOD_MOUSE, data: &data::SPECIES[9] },
    Species { kind: &crate::vole::FIELD_VOLE, data: &data::SPECIES[10] },
    Species { kind: &crate::deer::WHITE_TAILED_DEER, data: &data::SPECIES[11] },
    Species { kind: &crate::deer::RED_DEER, data: &data::SPECIES[12] },
    Species { kind: &crate::fish::BROWN_TROUT, data: &data::SPECIES[13] },
    Species { kind: &crate::fish::PACU, data: &data::SPECIES[14] },
    Species { kind: &crate::fox::RED_FOX, data: &data::SPECIES[15] },
    Species { kind: &crate::fox::ARCTIC_FOX, data: &data::SPECIES[16] },
    Species { kind: &crate::raven::RAVEN, data: &data::SPECIES[17] },
    Species { kind: &crate::protoceratops::PROTOCERATOPS, data: &data::SPECIES[18] },
    Species { kind: &crate::kryptobaatar::KRYPTOBAATAR, data: &data::SPECIES[19] },
    Species { kind: &crate::lizard::PRISCAGAMA, data: &data::SPECIES[20] },
    Species { kind: &crate::reindeer::REINDEER, data: &data::SPECIES[21] },
    Species { kind: &crate::coyote::COYOTE, data: &data::SPECIES[22] },
    Species { kind: &crate::small_bird::HOUSE_SPARROW, data: &data::SPECIES[23] },
    Species { kind: &crate::small_bird::SNOW_BUNTING, data: &data::SPECIES[24] },
    Species { kind: &crate::owl::GREAT_HORNED_OWL, data: &data::SPECIES[25] },
    Species { kind: &crate::owl::SNOWY_OWL, data: &data::SPECIES[26] },
    Species { kind: &crate::eagle::GOLDEN_EAGLE, data: &data::SPECIES[27] },
    Species { kind: &crate::eagle::SAVANNA_HAWK, data: &data::SPECIES[28] },
    Species { kind: &crate::vulture::TURKEY_VULTURE, data: &data::SPECIES[29] },
    Species { kind: &crate::gull::HERRING_GULL, data: &data::SPECIES[30] },
    Species { kind: &crate::caiman::YACARE_CAIMAN, data: &data::SPECIES[31] },
    Species { kind: &crate::caiman::SPECTACLED_CAIMAN, data: &data::SPECIES[32] },
    Species { kind: &crate::wolf::WOLF, data: &data::SPECIES[33] },
    Species { kind: &crate::jaguar::JAGUAR, data: &data::SPECIES[34] },
    Species { kind: &crate::fish::LUNGFISH, data: &data::SPECIES[35] },
    Species { kind: &crate::fish::LYCOPTERA, data: &data::SPECIES[36] },
    Species { kind: &crate::psittacosaurus::PSITTACOSAURUS, data: &data::SPECIES[37] },
    Species { kind: &crate::ankylosaur::GASTONIA, data: &data::SPECIES[38] },
    Species { kind: &crate::ankylosaur::PINACOSAURUS, data: &data::SPECIES[39] },
    Species { kind: &crate::iguanodont::IGUANODONT, data: &data::SPECIES[40] },
    Species { kind: &crate::caiman::CROCODYLIFORM, data: &data::SPECIES[41] },
    Species { kind: &crate::citipati::CITIPATI, data: &data::SPECIES[42] },
    Species { kind: &crate::early_bird::CONFUCIUSORNIS, data: &data::SPECIES[43] },
    Species { kind: &crate::early_bird::GOBIPTERYX, data: &data::SPECIES[44] },
    Species { kind: &crate::sauropod::CEDAROSAURUS, data: &data::SPECIES[45] },
];

impl Species {
    /// Its row in `SPECIES`.
    pub fn index(&'static self) -> usize {
        (self as *const Species as usize - SPECIES.as_ptr() as usize) / size_of::<Species>()
    }

    /// `data::Species::hungry_in_the_wild`, worked out once a species: it looks over every
    /// species' habitats, and foraging and drinking ask it for every animal every tick.
    pub fn hungry_in_the_wild(&'static self) -> bool {
        static HUNGRY: std::sync::OnceLock<Vec<bool>> = std::sync::OnceLock::new();
        HUNGRY.get_or_init(|| {
            SPECIES
                .iter()
                .map(|s| s.data.hungry_in_the_wild())
                .collect()
        })[self.index()]
    }

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

/// The row in `SPECIES` of the species whose entity kind is `kind` (each creature's code finds its
/// own by its kind, never by a number kept by hand: Research round 5, item 7).
pub fn of(kind: &EntityKind) -> usize {
    SPECIES
        .iter()
        .position(|s| std::ptr::eq(s.kind, kind))
        .unwrap_or_else(|| panic!("{} is no species' kind (species.rs SPECIES)", kind.path()))
}

/// Rows whose entity kind isn't their species' (its name, or a breed's own kind), as when the list
/// above and `animals_core::species::SPECIES` fall out of order.
pub fn problems() -> Vec<String> {
    SPECIES
        .iter()
        .enumerate()
        .filter(|(_, s)| s.breed.is_none() && !s.kind.path().ends_with(s.name))
        .map(|(i, s)| format!("animals species.rs row {i}: kind {} with the row of {}", s.kind.path(), s.name))
        .collect()
}

/// Whether some species eats `item`: offered to one that doesn't, it is disliked.
pub fn anyone_eats(item: u32) -> bool {
    SPECIES.iter().any(|s| s.food(item).is_some())
}

/// Species `s`'s crowd where `pos` is: as heavily as its biome is stocked
/// (`animals_core::species::Species::crowd_in`).
pub fn crowd_at(w: &weft_sdk::World, s: &Species, pos: impl Into<weft_sdk::BlockPos>) -> u32 {
    s.crowd_in(weft_sdk::name_of(weft_sdk::abi::entry_kind::BIOME, w.biome(pos)))
}

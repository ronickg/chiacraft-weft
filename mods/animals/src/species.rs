//! Every species as data (`notes/ANIMALS.md`, "Species are data"): adding an animal is mostly a
//! row here. Each row is placed in the tree of life as biology places it: its family (with the
//! groups above it) and subfamily, its scientific name (genus and species). Animals breed only
//! within a species (`same_species`): a house cat and a lion share a family, not a species. A
//! domestic species with breeds (dogs) has a row per breed: the breed is its own entity kind
//! with its own model and numbers, the species (genes, behavior, what it is) shared. A wild
//! species is never a breed: Velociraptor's two species are two rows, two species.

use weft_sdk::Entity;

/// The nine stats, in the order of every `[_; STATS]` table (`STAT_NAMES`); those the core uses
/// by name.
pub const STATS: usize = 9;
pub const HEALTH: usize = 0;
pub const FOOD: usize = 2;
pub const DAMAGE: usize = 3;
pub const SPEED: usize = 4;
pub const TORPOR: usize = 7;
/// Their names in translations (`stat.animals.<name>`).
pub const STAT_NAMES: [&str; STATS] =
    ["health", "stamina", "food", "damage", "speed", "weight", "oxygen", "torpor", "fortitude"];

#[derive(Clone, Copy, PartialEq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Diet {
    Carnivore,
    Herbivore,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Taming {
    Passive,
    Knockout,
}

/// How much an animal likes a food: taming progress per feeding, as a multiple of plain food.
#[derive(Clone, Copy, PartialEq)]
pub enum Liking {
    Eats,
    Favorite,
    Kibble,
}

impl Liking {
    pub fn taming(self) -> f32 {
        match self {
            Liking::Eats => 1.0,
            Liking::Favorite => 1.5,
            Liking::Kibble => 2.0,
        }
    }
}

/// A family, and the groups above it, broadest first. Ranks are named where biology still uses
/// them (a class, an order); dinosaurs are placed by clades instead.
pub struct Family {
    /// Its scientific name (`family.animals.<it, lowercase>` is what people call it).
    pub name: &'static str,
    pub above: &'static [&'static str],
}

pub static CANIDAE: Family = Family { name: "Canidae", above: &["Mammalia", "Carnivora"] };
pub static FELIDAE: Family = Family { name: "Felidae", above: &["Mammalia", "Carnivora"] };
pub static CAVIIDAE: Family = Family { name: "Caviidae", above: &["Mammalia", "Rodentia"] };
pub static DROMAEOSAURIDAE: Family = Family { name: "Dromaeosauridae", above: &["Dinosauria", "Theropoda"] };

pub struct Food {
    /// The item's full name.
    pub item: &'static str,
    /// Points of food it gives.
    pub food: u16,
    pub liking: Liking,
}

const fn eats(item: &'static str, food: u16) -> Food {
    Food { item, food, liking: Liking::Eats }
}

const fn favorite(item: &'static str, food: u16) -> Food {
    Food { item, food, liking: Liking::Favorite }
}

/// Every species takes kibble, best of all.
const KIBBLE: Food = Food { item: "animals:kibble", food: 30, liking: Liking::Kibble };

pub struct Species {
    pub kind: &'static Entity,
    /// Its model: the coat layers its coat is painted on (`genetics/layers/<it>`).
    pub model: &'static str,
    /// The species (`species.animals.<name>`, its gene file `genetics/<name>.json`).
    pub name: &'static str,
    /// Its scientific name: genus and species ("Canis familiaris").
    pub binomial: &'static str,
    pub family: &'static Family,
    pub subfamily: &'static str,
    /// The breed, for domestic species that have them (the gene file's breed).
    pub breed: Option<&'static str>,
    pub size: Size,
    pub diet: Diet,
    pub taming: Taming,
    /// Each stat's value at level 1 (damage and speed as the kind's own: 100%).
    pub base: [f32; STATS],
    /// What a point adds, as a share of the base: a wild point, a tamed point.
    pub wild_step: [f32; STATS],
    pub tamed_step: [f32; STATS],
    /// Seconds a tame takes to use one point of food.
    pub hunger: f32,
    /// What it eats; anything else in another species' diet it dislikes.
    pub foods: &'static [Food],
    /// Feedings of plain food that tame one at level 1 (a level-150 one takes 2.5 times as many).
    pub feedings: f32,
    /// Blocks within which a player walking upright (not sneaking) scares it; twice as far
    /// running. 0: nothing scares it.
    pub shy: f32,
    /// Whether it fights (a tame's stance does nothing for one that doesn't).
    pub fights: bool,
    /// What it does best (`signatures.rs`; `screen.animals.signature.<it>`).
    pub signature: &'static str,
    /// Where a rider sits, in blocks above its feet; 0: it can't be ridden (`riding.rs`).
    pub seat: f32,
    /// How much a coat that stands out from its surroundings costs it in the wild (0..1, natural
    /// selection in `evolution.rs`): most for prey and ambush hunters, little for dogs, which
    /// live off people.
    pub camouflage: f32,
    /// How much bigger males are than females, as heights (capybara females are the bigger;
    /// unknown for dinosaurs, so 1); individuals' sizes in `coats::size`.
    pub dimorphism: f32,
    /// How far an individual's size goes either way from its sex's, at the size gene's ends.
    pub size_spread: f32,
    /// A grown average one's height in meters (its model's), and where it is measured
    /// (`screen.animals.size.<at>`): the shoulder, or the hip of a two-legged one.
    pub height: (f32, &'static str),
    /// Young born or hatched at once (`breeding.rs`), fewest and most.
    pub litter: (u32, u32),
    /// Ticks a mother carries her young (an in-game day is 24 000); 0: it lays eggs.
    pub gestation: u32,
    /// Ticks its eggs take to hatch, kept warm.
    pub incubation: u32,
    /// Ticks a mother rests after a birth before she can breed again.
    pub recovery: u32,
    /// Ticks its young take to grow up.
    pub growth: u32,
    /// A newborn's size against a grown one's (1: it has its own young model, the chihuahua's
    /// puppy).
    pub birth_size: f32,
}

/// ARK's shape: health grows fastest, speed slowest, torpor only from wild points.
const WILD_STEP: [f32; STATS] = [0.2, 0.1, 0.1, 0.05, 0.01, 0.02, 0.1, 0.06, 0.05];
const TAMED_STEP: [f32; STATS] = [0.1, 0.1, 0.1, 0.04, 0.015, 0.04, 0.1, 0.0, 0.05];

pub static SPECIES: [Species; 6] = [
    Species {
        kind: &crate::dog::CHIHUAHUA,
        model: "chihuahua",
        name: "dog",
        binomial: "Canis familiaris",
        family: &CANIDAE,
        subfamily: "Caninae",
        breed: Some("chihuahua"),
        size: Size::Small,
        diet: Diet::Carnivore,
        taming: Taming::Passive,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [8.0, 100.0, 80.0, 2.0, 1.0, 40.0, 100.0, 60.0, 10.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
        foods: &[
            eats("base:beef", 10),
            eats("base:porkchop", 10),
            eats("base:chicken", 8),
            eats("base:mutton", 8),
            favorite("base:cooked_beef", 20),
            favorite("base:cooked_porkchop", 20),
            favorite("base:cooked_chicken", 16),
            favorite("base:cooked_mutton", 16),
            favorite("base:bone", 5),
            KIBBLE,
        ],
        feedings: 4.0,
        shy: 3.0,
        fights: true,
        signature: "alarm",
        seat: 0.0,
        camouflage: 0.3,
        dimorphism: 1.04,
        size_spread: 0.2,
        height: (0.20, "shoulder"),
        litter: (1, 4),
        gestation: 24_000,
        incubation: 0,
        recovery: 48_000,
        growth: 72_000,
        birth_size: 1.0,
    },
    Species {
        kind: &crate::cat::CAT,
        model: "cat",
        name: "cat",
        binomial: "Felis catus",
        family: &FELIDAE,
        subfamily: "Felinae",
        breed: None,
        size: Size::Small,
        diet: Diet::Carnivore,
        taming: Taming::Passive,
        base: [6.0, 80.0, 60.0, 1.0, 1.0, 30.0, 80.0, 40.0, 10.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 40.0,
        // No fish in the game yet: birds and eggs are what it likes best.
        foods: &[
            eats("base:beef", 8),
            eats("base:mutton", 8),
            eats("base:egg", 6),
            eats("base:cooked_beef", 14),
            eats("base:cooked_mutton", 14),
            favorite("base:chicken", 10),
            favorite("base:cooked_chicken", 16),
            KIBBLE,
        ],
        feedings: 5.0,
        shy: 6.0,
        fights: true,
        signature: "gifts",
        seat: 0.0,
        camouflage: 0.5,
        dimorphism: 1.08,
        size_spread: 0.08,
        height: (0.24, "shoulder"),
        litter: (2, 5),
        gestation: 24_000,
        incubation: 0,
        recovery: 48_000,
        growth: 72_000,
        birth_size: 0.35,
    },
    Species {
        kind: &crate::capybara::CAPYBARA,
        model: "capybara",
        name: "capybara",
        binomial: "Hydrochoerus hydrochaeris",
        family: &CAVIIDAE,
        subfamily: "Hydrochoerinae",
        breed: None,
        size: Size::Medium,
        diet: Diet::Herbivore,
        taming: Taming::Passive,
        // It holds its breath for five minutes.
        base: [12.0, 150.0, 200.0, 1.0, 1.0, 150.0, 400.0, 150.0, 20.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 25.0,
        foods: &[
            eats("base:wheat", 10),
            eats("base:wheat_seeds", 3),
            eats("base:bread", 15),
            favorite("base:apple", 15),
            favorite("base:sugar_cane", 12),
            KIBBLE,
        ],
        feedings: 3.0,
        shy: 0.0,
        fights: false,
        signature: "calm",
        seat: 0.0,
        camouflage: 0.8,
        dimorphism: 0.97,
        size_spread: 0.08,
        height: (0.56, "shoulder"),
        litter: (2, 6),
        gestation: 48_000,
        incubation: 0,
        recovery: 72_000,
        growth: 96_000,
        birth_size: 0.3,
    },
    Species {
        kind: &crate::raptor::RAPTOR,
        model: "raptor",
        name: "raptor",
        binomial: "Velociraptor mongoliensis",
        family: &DROMAEOSAURIDAE,
        subfamily: "Velociraptorinae",
        breed: None,
        size: Size::Medium,
        diet: Diet::Carnivore,
        taming: Taming::Knockout,
        base: [14.0, 150.0, 150.0, 4.0, 1.0, 140.0, 150.0, 200.0, 15.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 20.0,
        foods: &[
            eats("base:chicken", 10),
            eats("base:mutton", 10),
            eats("base:porkchop", 12),
            favorite("base:beef", 15),
            KIBBLE,
        ],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        signature: "pack",
        seat: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (0.50, "hip"),
        litter: (2, 4),
        gestation: 0,
        incubation: 24_000,
        recovery: 96_000,
        growth: 96_000,
        birth_size: 0.3,
    },
    Species {
        kind: &crate::utahraptor::UTAHRAPTOR,
        model: "utahraptor",
        name: "utahraptor",
        binomial: "Utahraptor ostrommaysi",
        family: &DROMAEOSAURIDAE,
        subfamily: "Dromaeosaurinae",
        breed: None,
        size: Size::Large,
        diet: Diet::Carnivore,
        taming: Taming::Knockout,
        base: [60.0, 300.0, 400.0, 10.0, 1.0, 400.0, 200.0, 700.0, 30.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 12.0,
        foods: &[
            eats("base:chicken", 10),
            eats("base:porkchop", 15),
            favorite("base:beef", 20),
            favorite("base:mutton", 15),
            KIBBLE,
        ],
        feedings: 18.0,
        shy: 10.0,
        fights: true,
        signature: "mount",
        // On its back, above the hips (1.69 blocks up, the rider's hips a little above).
        seat: 1.55,
        camouflage: 0.8,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (1.50, "hip"),
        litter: (2, 4),
        gestation: 0,
        incubation: 48_000,
        recovery: 96_000,
        growth: 288_000,
        birth_size: 0.15,
    },
    // Velociraptor's second species (named 2008, from Bayan Mandahu, Inner Mongolia; known from
    // skull bones only, so the same body and numbers, its own coat).
    Species {
        kind: &crate::raptor::OSMOLSKAE,
        model: "raptor",
        name: "osmolskae",
        binomial: "Velociraptor osmolskae",
        family: &DROMAEOSAURIDAE,
        subfamily: "Velociraptorinae",
        breed: None,
        size: Size::Medium,
        diet: Diet::Carnivore,
        taming: Taming::Knockout,
        base: [14.0, 150.0, 150.0, 4.0, 1.0, 140.0, 150.0, 200.0, 15.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 20.0,
        foods: &[
            eats("base:chicken", 10),
            eats("base:mutton", 10),
            eats("base:porkchop", 12),
            favorite("base:beef", 15),
            KIBBLE,
        ],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        signature: "pack",
        seat: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (0.50, "hip"),
        litter: (2, 4),
        gestation: 0,
        incubation: 24_000,
        recovery: 96_000,
        growth: 96_000,
        birth_size: 0.3,
    },
];

impl Species {
    /// The name of its kind, without the namespace (`entity.animals.<it>`, its saddle).
    pub fn kind_name(&self) -> &'static str {
        self.kind.path()
    }

    /// Its food `item` (an item id), if it eats it.
    pub fn food(&self, item: u32) -> Option<(usize, &'static Food)> {
        let name = weft_sdk::item_name(item);
        self.foods
            .iter()
            .enumerate()
            .find(|(_, f)| f.item == name)
    }
}

/// Whether two animals' species are one (and so can breed together): breeds of one species are,
/// two species of one genus are not.
pub fn same_species(a: &Species, b: &Species) -> bool {
    a.binomial == b.binomial
}

/// Whether some species eats `item`: offered to one that doesn't, it is disliked.
pub fn anyone_eats(item: u32) -> bool {
    SPECIES.iter().any(|s| s.food(item).is_some())
}

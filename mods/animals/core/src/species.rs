//! Every species as data (`notes/ANIMALS.md`, "Species are data"): adding an animal is mostly a
//! row here. Each row is placed in the tree of life as biology places it: its family (with the
//! groups above it) and subfamily, its scientific name (genus and species). Animals breed only
//! within a species (`same_species`): a house cat and a lion share a family, not a species. A
//! domestic species with breeds (dogs) has a row per breed: the breed is its own entity kind
//! with its own model and numbers, the species (genes, behavior, what it is) shared. A wild
//! species is never a breed: Velociraptor's two species are two rows, two species.
//!
//! Every species' settings are on one page, `mods/animals/SPECIES.md` (`sheet.rs`): rewrite it
//! after a change, `mods/animals/web/build.sh && node mods/animals/web/sheet.mjs`.

pub use crate::biology::{Biology, Rarity, Social, Tune};

/// Life stages, from a newborn to grown (`animal::Animal::stage`).
pub const BABY: u8 = 0;
pub const JUVENILE: u8 = 1;
pub const ADOLESCENT: u8 = 2;
pub const ADULT: u8 = 3;
/// Old: from `life::ELDERLY` of its lifespan on (greying, past breeding).
pub const ELDER: u8 = 4;

/// The nine stats, in the order of every `[_; STATS]` table (`STAT_NAMES`); those the core uses
/// by name.
pub const STATS: usize = 9;
pub const HEALTH: usize = 0;
pub const STAMINA: usize = 1;
pub const FOOD: usize = 2;
pub const DAMAGE: usize = 3;
pub const SPEED: usize = 4;
pub const WEIGHT: usize = 5;
pub const TORPOR: usize = 7;
/// Their names in translations (`stat.animals.<name>`).
pub const STAT_NAMES: [&str; STATS] = ["health", "stamina", "food", "damage", "speed", "weight", "oxygen", "torpor", "fortitude"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Diet {
    Carnivore,
    Herbivore,
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
/// Not a real family: wyverns are fantasy animals, placed under "Fantasy" so they never pass
/// for real ones. The names follow how real ones are made (a family in -idae, a subfamily in
/// -inae).
pub static WYVERNIDAE: Family = Family { name: "Wyvernidae", above: &["Fantasy", "Draconia"] };

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
    /// Its attacks: animations in its model's file, one played at random on each hit
    /// (`commands::fight`).
    pub attacks: &'static [&'static str],
    /// How far past its box it strikes, in blocks (a long neck reaches further).
    pub reach: f32,
    /// What it does best (`signatures.rs`; `screen.animals.signature.<it>`).
    pub signature: &'static str,
    /// Where a rider sits, in blocks above its feet; 0: it can't be ridden (`riding.rs`).
    pub seat: f32,
    /// How far ahead of its middle the seat is, in blocks (a dragon's rider sits at the base of
    /// the neck).
    pub seat_forward: f32,
    /// How fast it flies ridden, against its walking speed (0: it doesn't fly).
    pub flight: f32,
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
    /// Where it lives: biomes by full name, each with how common it is there against where it is
    /// commonest (1). It spawns only there (`mods/animals/src/habitats.rs`), how often by its
    /// rarity, in groups by its social life (`Rarity::spawn_chance`, `Social::group`).
    pub habitats: &'static [(&'static str, f32)],
    /// It as life has it (mass, carrying, growing up, a life, litters; `biology`): its
    /// breeding numbers follow from these by one pacing for every species (`gestation()`..).
    pub bio: Biology,
    /// Numbers it sets itself instead (`Tune::NONE`: all derived).
    pub tune: Tune,
    /// A newborn's size against a grown one's (1: it has its own young model, the chihuahua's
    /// puppy).
    pub birth_size: f32,
}

impl Species {
    /// Ticks a mother carries her young (an in-game day is 24 000); 0: it lays eggs.
    pub fn gestation(&self) -> u32 {
        self.tune.gestation.unwrap_or_else(|| self.bio.gestation())
    }
    /// Ticks its eggs take to hatch, kept warm.
    pub fn incubation(&self) -> u32 {
        self.tune
            .incubation
            .unwrap_or_else(|| self.bio.incubation())
    }
    /// Ticks a mother rests after a birth before she can breed again.
    pub fn recovery(&self) -> u32 {
        self.tune.recovery.unwrap_or_else(|| self.bio.recovery())
    }
    /// Ticks its young take to grow up.
    pub fn growth(&self) -> u32 {
        self.tune.growth.unwrap_or_else(|| self.bio.growth())
    }
    /// Ticks an average one lives (each its own, give or take a fifth: `life::lifespan`).
    pub fn lifespan(&self) -> u32 {
        self.tune.lifespan.unwrap_or_else(|| self.bio.lifespan())
    }
    /// How many of its kind may live within `life::CROWD_RADIUS` blocks before the wild ones
    /// breed less (a pack, a herd, a pair of dragons).
    pub fn crowd(&self) -> u32 {
        self.tune.crowd.unwrap_or_else(|| self.bio.crowd())
    }
    /// Young born or hatched at once (`breeding.rs`), fewest and most.
    pub fn litter(&self) -> (u32, u32) {
        self.tune.litter.unwrap_or_else(|| self.bio.litter())
    }
}

/// ARK's shape: health grows fastest, speed slowest, torpor only from wild points.
pub const WILD_STEP: [f32; STATS] = [0.2, 0.1, 0.1, 0.05, 0.01, 0.02, 0.1, 0.06, 0.05];
pub const TAMED_STEP: [f32; STATS] = [0.1, 0.1, 0.1, 0.04, 0.015, 0.04, 0.1, 0.0, 0.05];

pub static SPECIES: [Species; 7] = [
    Species {
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
        attacks: &["bite"],
        reach: 0.8,
        signature: "alarm",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 0.3,
        dimorphism: 1.04,
        size_spread: 0.2,
        height: (0.20, "shoulder"),
        // Chihuahua, Mexico: dry country, and wherever people are; never the cold.
        habitats: &[("base:plains", 1.0), ("base:savanna", 0.6), ("base:desert", 0.4), ("base:forest", 0.3), ("base:birch_forest", 0.3)],
        bio: Biology {
            mass: 2.5,
            gestation: 63.0,
            incubation: 0.0,
            maturity: 300.0,
            lifespan: 4750.0,
            interval: 180.0,
            litter: 3.0,
            social: Social::Pack,
            rarity: Rarity::Common,
        },
        tune: Tune::NONE,
        birth_size: 1.0,
    },
    Species {
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
        attacks: &["swipe", "bite"],
        reach: 0.8,
        signature: "gifts",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 0.5,
        dimorphism: 1.08,
        size_spread: 0.08,
        height: (0.24, "shoulder"),
        // From the African wildcat of desert and savanna edges; feral almost everywhere but the
        // deep cold.
        habitats: &[
            ("base:plains", 1.0),
            ("base:forest", 0.6),
            ("base:birch_forest", 0.6),
            ("base:savanna", 0.5),
            ("base:desert", 0.4),
            ("base:taiga", 0.2),
        ],
        bio: Biology {
            mass: 4.5,
            gestation: 65.0,
            incubation: 0.0,
            maturity: 240.0,
            lifespan: 4750.0,
            interval: 200.0,
            litter: 3.0,
            social: Social::Solitary,
            rarity: Rarity::Common,
        },
        tune: Tune::NONE,
        birth_size: 0.35,
    },
    Species {
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
        attacks: &[],
        reach: 0.8,
        signature: "calm",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 0.8,
        dimorphism: 0.97,
        size_spread: 0.08,
        height: (0.56, "shoulder"),
        // Savannas, flooded grasslands and swamps of South America, always near water.
        habitats: &[("base:swamp", 1.0), ("base:savanna", 0.8), ("base:forest", 0.2)],
        bio: Biology {
            mass: 50.0,
            gestation: 140.0,
            incubation: 0.0,
            maturity: 550.0,
            lifespan: 2920.0,
            interval: 365.0,
            litter: 4.0,
            social: Social::Herd,
            rarity: Rarity::Common,
        },
        tune: Tune::NONE,
        birth_size: 0.3,
    },
    Species {
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
        foods: &[eats("base:chicken", 10), eats("base:mutton", 10), eats("base:porkchop", 12), favorite("base:beef", 15), KIBBLE],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        attacks: &["bite", "kick"],
        reach: 0.8,
        signature: "pack",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (0.50, "hip"),
        // The Djadokhta Formation, Mongolia: wind-blown dunes, warm and semi-arid.
        habitats: &[("animals:red_dunes", 1.0), ("animals:scrubland", 0.4)],
        bio: Biology {
            mass: 15.0,
            gestation: 0.0,
            incubation: 74.0,
            maturity: 900.0,
            lifespan: 7300.0,
            interval: 365.0,
            litter: 3.0,
            social: Social::Pack,
            rarity: Rarity::Uncommon,
        },
        tune: Tune::NONE,
        birth_size: 0.3,
    },
    Species {
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
        foods: &[eats("base:chicken", 10), eats("base:porkchop", 15), favorite("base:beef", 20), favorite("base:mutton", 15), KIBBLE],
        feedings: 18.0,
        shy: 10.0,
        fights: true,
        attacks: &["bite", "kick"],
        reach: 1.2,
        signature: "mount",
        // On its back, above the hips (1.69 blocks up, the rider's hips a little above).
        seat: 1.55,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 0.8,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (1.50, "hip"),
        // The Cedar Mountain Formation, Utah: a floodplain whose soils swelled and cracked with
        // wet and dry seasons, conifers and ferns.
        habitats: &[("animals:floodplain", 1.0), ("animals:scrubland", 0.5), ("animals:fern_forest", 0.3)],
        bio: Biology {
            mass: 300.0,
            gestation: 0.0,
            incubation: 140.0,
            maturity: 1800.0,
            lifespan: 14600.0,
            interval: 548.0,
            litter: 2.0,
            social: Social::Pack,
            rarity: Rarity::Rare,
        },
        tune: Tune::NONE,
        birth_size: 0.15,
    },
    // Velociraptor's second species (named 2008, from Bayan Mandahu, Inner Mongolia; known from
    // skull bones only, so the same body and numbers, its own coat).
    Species {
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
        foods: &[eats("base:chicken", 10), eats("base:mutton", 10), eats("base:porkchop", 12), favorite("base:beef", 15), KIBBLE],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        attacks: &["bite", "kick"],
        reach: 0.8,
        signature: "pack",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.08,
        height: (0.50, "hip"),
        // The Bayan Mandahu Formation, Inner Mongolia: dunes, drier than its cousin's.
        habitats: &[("animals:red_dunes", 1.0)],
        bio: Biology {
            mass: 15.0,
            gestation: 0.0,
            incubation: 74.0,
            maturity: 900.0,
            lifespan: 7300.0,
            interval: 365.0,
            litter: 3.0,
            social: Social::Pack,
            rarity: Rarity::Uncommon,
        },
        tune: Tune::NONE,
        birth_size: 0.3,
    },
    // The wyvern, a fantasy animal (`wyvern.rs`): a dragon with two legs and bat wings, about
    // 3.9 m at the shoulder and 16 m long, rare in the mountains.
    Species {
        model: "wyvern",
        name: "wyvern",
        binomial: "Wyverna montana",
        family: &WYVERNIDAE,
        subfamily: "Wyverninae",
        breed: None,
        size: Size::Large,
        diet: Diet::Carnivore,
        taming: Taming::Knockout,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [400.0, 800.0, 2000.0, 30.0, 1.0, 1200.0, 400.0, 3000.0, 80.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 5.0,
        foods: &[
            eats("base:chicken", 10),
            eats("base:porkchop", 15),
            eats("base:beef", 20),
            favorite("base:mutton", 20),
            favorite("base:cooked_mutton", 30),
            KIBBLE,
        ],
        feedings: 45.0,
        shy: 0.0,
        fights: true,
        attacks: &["bite", "claw", "tail"],
        // Its head is 7 blocks ahead of its middle: a bite reaches well past its box.
        reach: 5.0,
        signature: "mount",
        // At the base of the neck between the shoulder spikes, as the show's riders sit: its back
        // there 3.8 blocks up and 1.6 ahead of its middle, the rider's hips a little above.
        seat: 3.65,
        seat_forward: 1.55,
        // Flying about 18 blocks a second, 28 sprinting (the engine's movement speed included).
        flight: 8.0,
        camouflage: 0.2,
        dimorphism: 1.08,
        size_spread: 0.12,
        height: (3.9, "shoulder"),
        // A fantasy animal of high crags, each country's kind (`genetics/wyvern.json`).
        habitats: &[("animals:crags", 1.0), ("animals:fern_forest", 0.33), ("animals:red_dunes", 0.33), ("animals:scrubland", 0.33)],
        bio: Biology {
            mass: 5000.0,
            gestation: 0.0,
            incubation: 450.0,
            maturity: 7300.0,
            lifespan: 40150.0,
            interval: 1095.0,
            litter: 1.0,
            social: Social::Solitary,
            rarity: Rarity::Legendary,
        },
        tune: Tune::NONE,
        birth_size: 0.08,
    },
];

/// Whether two animals' species are one (and so can breed together): breeds of one species are,
/// two species of one genus are not.
pub fn same_species(a: &Species, b: &Species) -> bool {
    a.binomial == b.binomial
}

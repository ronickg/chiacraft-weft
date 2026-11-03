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

pub use crate::biology::{Biology, Rarity, Social, Tune, seasons};
pub use vitals::food::Category;
use vitals::food::Category::*;

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
pub const FORTITUDE: usize = 8;
/// Their names in translations (`stat.animals.<name>`).
pub const STAT_NAMES: [&str; STATS] = ["health", "stamina", "food", "damage", "speed", "weight", "oxygen", "torpor", "fortitude"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Size {
    Small,
    Medium,
    Large,
}

/// What kind of eater it is, from its diet's plant share (`Species::diet`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Diet {
    Carnivore,
    Omnivore,
    Herbivore,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Taming {
    Passive,
    Knockout,
    /// Never tamed: a wild creature of the food web (a hare, a fox) that lives, flees, breeds and
    /// is eaten (`notes/ECOLOGY.md`, the species to add).
    Wild,
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
pub static LEPORIDAE: Family = Family { name: "Leporidae", above: &["Mammalia", "Lagomorpha"] };
pub static MURIDAE: Family = Family { name: "Muridae", above: &["Mammalia", "Rodentia"] };
pub static CRICETIDAE: Family = Family { name: "Cricetidae", above: &["Mammalia", "Rodentia"] };
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

/// The other animals hunters take, by entity and mass in kg (the overworld's farm animals as
/// their wild kinds: junglefowl and grouse, boar, wild sheep, aurochs; `reports/ecology/Overworld
/// food webs.md`). They live all over the overworld.
pub const OTHER_PREY: [(&str, f32); 4] = [("base:chicken", 1.0), ("base:pig", 80.0), ("base:sheep", 70.0), ("base:cow", 600.0)];

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
    /// What it really eats, as shares of its food by kind (`vitals::food::Category`), from its
    /// research (`reports/ecology/*food webs.md`): the forage goal looks for these in the world.
    pub eats: &'static [(Category, f32)],
    /// The prey it takes alone, as shares of its own mass, smallest and biggest (`(0, 0)`: it
    /// hunts nothing), from its research: small hunters mostly under half their mass, a lone
    /// Velociraptor up to six times its own (the Fighting Dinosaurs).
    pub prey: (f32, f32),
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
    /// The share of its food that is plants.
    pub fn plant_share(&self) -> f32 {
        self.eats
            .iter()
            .filter(|(c, _)| c.plant())
            .map(|(_, s)| s)
            .sum()
    }
    /// A plant eater from three quarters plants, a meat eater up to a quarter.
    pub fn diet(&self) -> Diet {
        match self.plant_share() {
            p if p >= 0.75 => Diet::Herbivore,
            p if p <= 0.25 => Diet::Carnivore,
            _ => Diet::Omnivore,
        }
    }
    /// Whether it eats this kind of food at all.
    pub fn eats(&self, c: Category) -> bool {
        self.eats.iter().any(|&(k, _)| k == c)
    }
    /// Whether a wild one gets hungry and forages (`notes/ECOLOGY.md`, part 5): mostly a plant
    /// eater, whose food the world grows.
    pub fn forages_wild(&self) -> bool {
        self.plant_share() >= 0.5
    }
    /// Whether it hunts at all.
    pub fn hunts(&self) -> bool {
        self.prey.1 > 0.0
    }
    /// Whether a hunter of `own` kg takes prey of `kg` alone.
    pub fn takes(&self, own: f32, kg: f32) -> bool {
        self.hunts() && kg >= own * self.prey.0 && kg <= own * self.prey.1
    }
    /// Its prey where it lives (part 6): the species of other families sharing one of its
    /// habitats within its prey's sizes, and the farm animals' wild kinds in the overworld.
    pub fn prey_at_home(&self) -> impl Iterator<Item = &'static str> + '_ {
        let mine = |biome: &str| self.habitats.iter().any(|&(b, _)| b == biome);
        let species = SPECIES
            .iter()
            .filter(move |o| o.family.name != self.family.name && self.takes(self.bio.mass, o.bio.mass))
            .filter(move |o| o.habitats.iter().any(|&(b, _)| mine(b)))
            .map(|o| o.name);
        let overworld = self.habitats.iter().any(|&(b, _)| b.starts_with("base:"));
        let mobs = OTHER_PREY
            .iter()
            .filter(move |&&(_, kg)| overworld && self.takes(self.bio.mass, kg))
            .map(|&(name, _)| name);
        species.chain(mobs)
    }
    /// Whether a wild one gets hungry: a plant eater, or a hunter with prey where it lives (a
    /// hunter with none waits for its prey to come to the world).
    pub fn hungry_in_the_wild(&self) -> bool {
        self.forages_wild() || self.prey_at_home().next().is_some()
    }
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

pub static SPECIES: [Species; 11] = [
    Species {
        model: "chihuahua",
        name: "dog",
        binomial: "Canis familiaris",
        family: &CANIDAE,
        subfamily: "Caninae",
        breed: Some("chihuahua"),
        size: Size::Small,
        eats: &[(Meat, 0.4), (Carrion, 0.2), (Insects, 0.15), (Fruit, 0.15), (Seeds, 0.1)],
        prey: (0.008, 0.8),
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
            // Dogs, unlike wolves, come into heat at any time of year, about twice a year.
            breeds: seasons::ALL_YEAR,
            peak: 0,
            // Small short-coated dogs feel the cold below about 7 °C (vets' 45 °F) and risk heatstroke
            // from about 29 °C.
            comfort: (7.0, 29.0),
            cold_blooded: false,
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
        eats: &[(Meat, 0.8), (Insects, 0.1), (Carrion, 0.05), (Fish, 0.05)],
        prey: (0.005, 0.4),
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
            // A long-day breeder: cycling from spring into autumn, not in winter's short days.
            breeds: seasons::SPRING | seasons::SUMMER | seasons::AUTUMN,
            peak: seasons::SPRING,
            // Cats keep warmer than dogs in their coats: shelter wanted below about 5 °C, heat from 32.
            comfort: (5.0, 32.0),
            cold_blooded: false,
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
        eats: &[(Grass, 0.75), (WaterPlants, 0.2), (Fruit, 0.05)],
        prey: (0.0, 0.0),
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
            // A bale is nine wheat: a tame's food for a long while in one go.
            eats("base:hay_bale", 90),
            favorite("base:apple", 15),
            favorite("base:sugar_cane", 12),
            favorite("base:berries", 8),
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
            // Mates all year; most young come as the rains begin (April and May in the llanos).
            breeds: seasons::ALL_YEAR,
            peak: seasons::SPRING,
            // Tropical: zoos keep them over 12 °C; in the midday heat over about 32 they lie in the water.
            comfort: (12.0, 32.0),
            cold_blooded: false,
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
        eats: &[(Meat, 0.7), (Carrion, 0.25), (Insects, 0.05)],
        prey: (0.002, 6.0),
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
            social: Social::Solitary, // no Velociraptor was ever found with another (Ronald, 2026-11-03)
            rarity: Rarity::Uncommon,
            // No record; nesting colonies came back year after year as birds' do, and their kin
            // nest so their young hatch into the growing season: the wet one.
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // No record: feathered and warm-blooded like birds of its size, at home in the Gobi's dunes
            // (hot days, cold nights).
            comfort: (12.0, 40.0),
            cold_blooded: false,
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
        eats: &[(Meat, 0.8), (Carrion, 0.15), (Fish, 0.05)],
        prey: (0.01, 3.0),
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
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // No record: a 300 kg feathered body keeps its heat (Bergmann), the Cedar Mountain floodplain
            // was warm and seasonal.
            comfort: (6.0, 36.0),
            cold_blooded: false,
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
        eats: &[(Meat, 0.7), (Carrion, 0.25), (Insects, 0.05)],
        prey: (0.002, 6.0),
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
            social: Social::Solitary, // no Velociraptor was ever found with another (Ronald, 2026-11-03)
            rarity: Rarity::Uncommon,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // As its cousin's.
            comfort: (12.0, 40.0),
            cold_blooded: false,
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
        eats: &[(Meat, 0.8), (Fish, 0.1), (Carrion, 0.1)],
        prey: (0.001, 0.6),
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
            // As the great birds of prey: courting in late winter, laying in spring.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // A fantasy animal: a huge body in the cold crags, too big to lose heat fast, overheats
            // easily.
            comfort: (-10.0, 30.0),
            cold_blooded: false,
        },
        tune: Tune::NONE,
        birth_size: 0.08,
    },
    // The hares (`reports/ecology/Overworld food webs.md`: the mid-sized prey everywhere, the fox's,
    // coyote's, eagle's and lynx's main food): wild, never tamed (`Taming::Wild`). Two species on
    // one model (`source/hare`), as the two Velociraptors: the brown hare of open country and the
    // mountain hare of the north, white in the snow (their gene files).
    Species {
        model: "hare",
        name: "hare",
        binomial: "Lepus europaeus",
        family: &LEPORIDAE,
        subfamily: "Leporinae",
        breed: None,
        size: Size::Small,
        // Grasses most of the year, herbs and crops, twigs and bark in winter.
        eats: &[(Grass, 0.6), (Leaves, 0.3), (Roots, 0.1)],
        prey: (0.0, 0.0),
        taming: Taming::Wild,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [5.0, 120.0, 40.0, 0.5, 1.0, 15.0, 60.0, 20.0, 5.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
        foods: &[eats("base:wheat", 8), eats("base:wheat_seeds", 2), favorite("base:apple", 10), favorite("base:berries", 6)],
        feedings: 0.0,
        // It sits tight in its form, then bolts when a walker comes within about 12 m.
        shy: 12.0,
        fights: false,
        attacks: &[],
        reach: 0.8,
        signature: "bolt",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        // Females a little bigger (by mass, about 5%).
        dimorphism: 0.98,
        size_spread: 0.1,
        height: (0.30, "shoulder"),
        // Open country: farmland and grassland first, woods' edges; the jackrabbit's desert, the
        // cottontail's llanos and the rabbit's dunes by the sea in the research share its row.
        habitats: &[("base:plains", 1.0), ("base:desert", 1.0), ("base:forest", 0.6), ("base:savanna", 0.4), ("base:beach", 0.4)],
        bio: Biology {
            mass: 3.8,
            gestation: 42.0,
            incubation: 0.0,
            // First breeding at 7-8 months.
            maturity: 220.0,
            // Up to 12 years; few live past 5 in the wild.
            lifespan: 1825.0,
            // Three or four litters from January to August, the next conceived before the last is born.
            interval: 60.0,
            litter: 3.0,
            social: Social::Solitary,
            rarity: Rarity::Common,
            breeds: seasons::SPRING | seasons::SUMMER,
            peak: seasons::SPRING,
            // Est.: a dense winter coat keeps it in the open through European frosts; in the heat it lies
            // up in its form by day.
            comfort: (-5.0, 30.0),
            cold_blooded: false,
        },
        tune: Tune::NONE,
        birth_size: 0.3,
    },
    Species {
        model: "hare",
        name: "mountain_hare",
        binomial: "Lepus timidus",
        family: &LEPORIDAE,
        subfamily: "Leporinae",
        breed: None,
        size: Size::Small,
        // Grasses and herbs in summer; in winter birch, willow and heather twigs and bark.
        eats: &[(Grass, 0.4), (Leaves, 0.55), (Lichen, 0.05)],
        prey: (0.0, 0.0),
        taming: Taming::Wild,
        base: [5.0, 120.0, 40.0, 0.5, 1.0, 13.0, 60.0, 20.0, 5.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
        foods: &[eats("base:wheat", 8), eats("base:wheat_seeds", 2), favorite("base:apple", 10), favorite("base:berries", 6)],
        feedings: 0.0,
        shy: 12.0,
        fights: false,
        attacks: &[],
        reach: 0.8,
        signature: "bolt",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        // Females slightly heavier.
        dimorphism: 0.98,
        size_spread: 0.1,
        height: (0.30, "shoulder"),
        // The boreal forest, birch woods, the tundra and the Alps (700-3,800 m).
        habitats: &[("base:taiga", 1.0), ("base:birch_forest", 1.0), ("base:snowy_plains", 0.75), ("base:mountains", 0.75)],
        bio: Biology {
            mass: 3.0,
            gestation: 50.0,
            incubation: 0.0,
            // First breeding the summer after its birth.
            maturity: 300.0,
            lifespan: 1825.0,
            // One to three litters from February to August.
            interval: 70.0,
            litter: 2.5,
            social: Social::Solitary,
            rarity: Rarity::Common,
            breeds: seasons::SPRING | seasons::SUMMER,
            peak: seasons::SPRING,
            // Est.: snowshoe and mountain hares keep warm in their winter coats to about -10 °C before
            // working harder; in the Alps they shelter from summer heat.
            comfort: (-15.0, 24.0),
            cold_blooded: false,
        },
        tune: Tune::NONE,
        birth_size: 0.3,
    },
    // The wood mouse (`reports/ecology/Overworld food webs.md`: the rodent, the small hunters' main
    // food), a simple wild creature on its own model (`source/mouse`). It stands in for the deer
    // mouse of the prairie, the kangaroo rat of the desert, the cane mice of the llanos and the
    // dune mice; the field vole takes the north.
    Species {
        model: "mouse",
        name: "wood_mouse",
        binomial: "Apodemus sylvaticus",
        family: &MURIDAE,
        subfamily: "Murinae",
        breed: None,
        size: Size::Small,
        // Seeds and mast most of the year, insects and snails in late spring, berries, fungi and
        // roots later.
        eats: &[(Seeds, 0.45), (Insects, 0.2), (Fruit, 0.15), (Grass, 0.1), (Fungi, 0.05), (Roots, 0.05)],
        prey: (0.0, 0.0),
        taming: Taming::Wild,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 2.0, 30.0, 5.0, 2.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
        foods: &[eats("base:wheat_seeds", 4), eats("base:wheat", 6), favorite("base:berries", 4)],
        feedings: 0.0,
        shy: 6.0,
        fights: false,
        attacks: &[],
        reach: 0.5,
        signature: "dart",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.1,
        // About 1.3 times life (its model's), as a real one is six texels long.
        height: (0.05, "shoulder"),
        // Hedgerows, woods, fields and grassland; the desert's, the llanos' and the dunes' mice share
        // its row.
        habitats: &[("base:plains", 1.0), ("base:forest", 1.0), ("base:desert", 0.6), ("base:savanna", 0.6), ("base:beach", 0.5)],
        bio: Biology {
            mass: 0.023,
            gestation: 25.0,
            incubation: 0.0,
            // Grown at about two months.
            maturity: 60.0,
            // Most live less than a year; up to two.
            lifespan: 548.0,
            // A litter every month or so, February to October.
            interval: 30.0,
            litter: 4.5,
            social: Social::Solitary,
            rarity: Rarity::Common,
            breeds: seasons::SPRING | seasons::SUMMER | seasons::AUTUMN,
            peak: seasons::SUMMER,
            // Est.: it shelters in a nest underground, out in frost and in summer warmth alike.
            comfort: (-5.0, 32.0),
            cold_blooded: false,
        },
        // A mouse's real pace (grown in two months, gone within the year) would make a line four
        // generations deep in five game days and crowd 36 near: its growing up, life and room are
        // set to the common animals' pace instead (`balance::checks`).
        tune: Tune { growth: Some(72_000), lifespan: Some(384_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.35,
    },
    // The field vole, the north's rodent (the lemming's, the snow vole's and the marsh rats' stand-in),
    // a simple wild creature on its own model (`source/vole`): what the owls and foxes of the cold
    // biomes live on.
    Species {
        model: "vole",
        name: "field_vole",
        binomial: "Microtus agrestis",
        family: &CRICETIDAE,
        subfamily: "Arvicolinae",
        breed: None,
        size: Size::Small,
        // Grasses and herbs, root tubers and moss; bark in winter; a few insect larvae.
        eats: &[(Grass, 0.6), (Leaves, 0.15), (Roots, 0.15), (Seeds, 0.05), (Insects, 0.05)],
        prey: (0.0, 0.0),
        taming: Taming::Wild,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 3.0, 30.0, 5.0, 2.0],
        wild_step: WILD_STEP,
        tamed_step: TAMED_STEP,
        hunger: 30.0,
        foods: &[eats("base:wheat", 6), eats("base:wheat_seeds", 3), favorite("base:apple", 6)],
        feedings: 0.0,
        shy: 5.0,
        fights: false,
        attacks: &[],
        reach: 0.5,
        signature: "dart",
        seat: 0.0,
        seat_forward: 0.0,
        flight: 0.0,
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.1,
        // About 1.3 times life, as the wood mouse (its model's).
        height: (0.05, "shoulder"),
        // Moist grassland, woodland verges, marshes and moors to 1,700 m; the tundra's lemmings
        // and the swamp's marsh rats share its row.
        habitats: &[
            ("base:taiga", 1.0),
            ("base:birch_forest", 1.0),
            ("base:snowy_plains", 0.8),
            ("base:mountains", 0.8),
            ("base:swamp", 0.6),
        ],
        bio: Biology {
            mass: 0.035,
            gestation: 21.0,
            incubation: 0.0,
            // Out of the nest at three weeks, mature soon after.
            maturity: 35.0,
            // About two years at most; spring-born ones die sooner.
            lifespan: 730.0,
            // Up to seven litters a year.
            interval: 25.0,
            litter: 4.5,
            social: Social::Solitary,
            rarity: Rarity::Common,
            // All year, even under the snow, most in spring and summer.
            breeds: seasons::ALL_YEAR,
            peak: seasons::SPRING | seasons::SUMMER,
            // Est.: snug in runs under the snow through the winter; it shelters from summer heat.
            comfort: (-15.0, 28.0),
            cold_blooded: false,
        },
        // A vole's real pace, as the mouse's, set to the common animals' (`balance::checks`);
        // its life shorter still, as it breeds faster.
        tune: Tune { growth: Some(72_000), lifespan: Some(336_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.35,
    },
];

/// Whether two animals' species are one (and so can breed together): breeds of one species are,
/// two species of one genus are not.
pub fn same_species(a: &Species, b: &Species) -> bool {
    a.binomial == b.binomial
}

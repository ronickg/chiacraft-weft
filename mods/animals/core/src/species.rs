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
pub static CERVIDAE: Family = Family { name: "Cervidae", above: &["Mammalia", "Artiodactyla"] };
pub static SALMONIDAE: Family = Family { name: "Salmonidae", above: &["Actinopterygii", "Salmoniformes"] };
pub static SERRASALMIDAE: Family = Family { name: "Serrasalmidae", above: &["Actinopterygii", "Characiformes"] };
pub static CORVIDAE: Family = Family { name: "Corvidae", above: &["Aves", "Passeriformes"] };
pub static DJADOCHTATHERIIDAE: Family = Family { name: "Djadochtatheriidae", above: &["Mammalia", "Multituberculata"] };
pub static PRISCAGAMIDAE: Family = Family { name: "Priscagamidae", above: &["Reptilia", "Squamata"] };
pub static PROTOCERATOPSIDAE: Family = Family { name: "Protoceratopsidae", above: &["Dinosauria", "Ceratopsia"] };
pub static PASSERIDAE: Family = Family { name: "Passeridae", above: &["Aves", "Passeriformes"] };
pub static CALCARIIDAE: Family = Family { name: "Calcariidae", above: &["Aves", "Passeriformes"] };
pub static STRIGIDAE: Family = Family { name: "Strigidae", above: &["Aves", "Strigiformes"] };
pub static ACCIPITRIDAE: Family = Family { name: "Accipitridae", above: &["Aves", "Accipitriformes"] };
pub static CATHARTIDAE: Family = Family { name: "Cathartidae", above: &["Aves", "Accipitriformes"] };
pub static LARIDAE: Family = Family { name: "Laridae", above: &["Aves", "Charadriiformes"] };
pub static ALLIGATORIDAE: Family = Family { name: "Alligatoridae", above: &["Reptilia", "Crocodylia"] };
pub static CERATODONTIDAE: Family = Family { name: "Ceratodontidae", above: &["Sarcopterygii", "Dipnoi"] };
pub static LYCOPTERIDAE: Family = Family { name: "Lycopteridae", above: &["Actinopterygii", "Osteoglossomorpha"] };
pub static PSITTACOSAURIDAE: Family = Family { name: "Psittacosauridae", above: &["Dinosauria", "Ceratopsia"] };
pub static POLACANTHIDAE: Family = Family { name: "Polacanthidae", above: &["Dinosauria", "Ankylosauria"] };
pub static ANKYLOSAURIDAE: Family = Family { name: "Ankylosauridae", above: &["Dinosauria", "Ankylosauria"] };
pub static STYRACOSTERNA: Family = Family { name: "Styracosterna", above: &["Dinosauria", "Ornithopoda"] };
pub static NEOSUCHIA: Family = Family { name: "Neosuchia", above: &["Crocodylomorpha", "Crocodyliformes"] };
pub static OVIRAPTORIDAE: Family = Family { name: "Oviraptoridae", above: &["Dinosauria", "Theropoda"] };
pub static CONFUCIUSORNITHIDAE: Family = Family { name: "Confuciusornithidae", above: &["Dinosauria", "Avialae"] };
pub static ENANTIORNITHIDAE: Family = Family { name: "Enantiornithes", above: &["Dinosauria", "Avialae"] };
pub static BRACHIOSAURIDAE: Family = Family { name: "Brachiosauridae", above: &["Dinosauria", "Sauropoda"] };
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

/// Taming foods outside the eater's diet (`reports/Research round 3.md`, C5): a hand-fed food whose
/// kind (`vitals::forage::dropped`: the food table's, or a dropped seed's) isn't in its `eats`.
/// Kibble is every species' (`taming.rs`); an item neither table knows is left out (ferns and
/// seeds of the Cretaceous's plants, which the tables don't list as foods).
pub fn foods_off_diet() -> Vec<String> {
    SPECIES
        .iter()
        .flat_map(|s| s.foods.iter().map(move |f| (s, f)))
        .filter(|(_, f)| f.item != "animals:kibble")
        .filter_map(|(s, f)| {
            let (c, _) = vitals::forage::dropped(f.item, false)?;
            (!s.eats(c))
                .then(|| format!("{}: is hand-fed {} ({c:?}), which isn't in its diet (species.rs foods against eats)", s.name, f.item))
        })
        .collect()
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

/// The hunters that take bigger prey together than alone, and how big, as shares of one's mass:
/// grey wolves kill moose, elk and bison, ten times a wolf (`reports/ecology/Overworld food webs.md`:
/// "packs take moose and elk"; the hunting is `hunt.rs`).
pub const PACK_PREY: &[(&str, f32)] = &[
    ("grey_wolf", 10.0),
    // Utahraptors in a group went after an iguanodont many times their mass (the Stikes Quarry
    // block: an adult, juveniles and hatchlings caught with one).
    ("utahraptor", 15.0),
];
/// How many of its kind hunt together as a pack (itself among them).
pub const PACK: u32 = 3;

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
    /// Whether it lives in the water (the ray-finned fishes and the lungfishes): it never spawns on
    /// the ground, its mod puts it in the water (`mods/animals/src/fish.rs`).
    pub fn in_water(&self) -> bool {
        matches!(self.family.above.first(), Some(&"Actinopterygii") | Some(&"Sarcopterygii"))
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
    /// Whether it lives in the Cretaceous rather than the overworld: its habitats are that
    /// dimension's biomes (`animals:`; the overworld's are `base:`). Each species lives in one, and
    /// spawns only there (`habitats.rs`), even where the two share a biome (the Cretaceous's
    /// beaches and seas are `base:beach` and `base:ocean`: hares, mice, foxes, sparrows, gulls and
    /// ravens came to its shores).
    pub fn cretaceous(&self) -> bool {
        self.habitats.iter().any(|(b, _)| b.starts_with("animals:"))
    }
    /// The chance a natural spawn that picks it in `biome` brings a group (0 where it doesn't
    /// live): its rarity's, by how common it is there (`Rarity::spawn_chance`, its habitat's
    /// share). In the Cretaceous, each biome's rules are raised together until they average
    /// `CRETACEOUS_FLOW`, each keeping its share by rarity: the engine picks a rule at random and
    /// then rolls its chance, and the overworld's biomes also hold base's farm animals at 5%,
    /// so the Cretaceous, with only its own and rarer ones, brought a half to a fifth as many
    /// (the creatures playtest: no utahraptor, wyvern, crocodyliform or fish in 7 minutes).
    /// Fish count among fish (their own spawner, `fish.rs`), the rest among the rest.
    pub fn spawn_chance(&self, biome: &str) -> f32 {
        let at = |t: &Species| {
            t.habitats
                .iter()
                .find(|(b, _)| *b == biome)
                .map(|&(_, share)| t.bio.rarity.spawn_chance() * share)
        };
        let Some(mine) = at(self) else { return 0.0 };
        if !self.cretaceous() {
            return mine;
        }
        let peers: Vec<f32> = SPECIES
            .iter()
            .filter(|t| t.cretaceous() && t.in_water() == self.in_water())
            .filter_map(at)
            .collect();
        let mean = peers.iter().sum::<f32>() / peers.len().max(1) as f32;
        (mine * (CRETACEOUS_FLOW / mean).max(1.0)).min(1.0)
    }
    /// Whether it hunts at all.
    pub fn hunts(&self) -> bool {
        self.prey.1 > 0.0
    }
    /// Whether a hunter of `own` kg takes prey of `kg` alone.
    pub fn takes(&self, own: f32, kg: f32) -> bool {
        self.hunts() && kg >= own * self.prey.0 && kg <= own * self.prey.1
    }
    /// The biggest prey a pack of its kind takes together, as a share of one's mass (`PACK_PREY`;
    /// `None`: it hunts alone, or its pack takes nothing bigger).
    pub fn pack_prey(&self) -> Option<f32> {
        PACK_PREY
            .iter()
            .find(|&&(n, _)| n == self.name)
            .map(|&(_, m)| m)
    }
    /// Whether a hunter of `own` kg takes prey of `kg` with `pack` of its kind hunting with it (itself
    /// among them): a pack of `PACK` or more takes up to `pack_prey`.
    pub fn takes_with(&self, own: f32, kg: f32, pack: u32) -> bool {
        match self.pack_prey() {
            Some(most) if pack >= PACK => self.hunts() && kg >= own * self.prey.0 && kg <= own * most,
            _ => self.takes(own, kg),
        }
    }
    /// Whether it takes prey of `kg` at all, alone or with its pack (the food web, prey's watch).
    pub fn takes_at_most(&self, own: f32, kg: f32) -> bool {
        self.takes_with(own, kg, PACK)
    }
    /// Its prey where it lives (part 6): the species of other families sharing one of its
    /// habitats within its prey's sizes, and the farm animals' wild kinds in the overworld.
    pub fn prey_at_home(&self) -> impl Iterator<Item = &'static str> + '_ {
        let mine = |biome: &str| self.habitats.iter().any(|&(b, _)| b == biome);
        let species = SPECIES
            .iter()
            .filter(move |o| o.family.name != self.family.name && self.takes_at_most(self.bio.mass, o.bio.mass))
            .filter(move |o| o.habitats.iter().any(|&(b, _)| mine(b)))
            .map(|o| o.name);
        let overworld = self.habitats.iter().any(|&(b, _)| b.starts_with("base:"));
        let mobs = OTHER_PREY
            .iter()
            .filter(move |&&(_, kg)| overworld && self.takes_at_most(self.bio.mass, kg))
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
    /// Its crowd in `biome`: a plant eater's (`forages_wild`) as heavily as the biome is stocked
    /// (`STOCKING`), others' as they are; never under two, a pair. What natural spawning, wild
    /// breeding, dying of want and the food webs' grazing all weigh (`life::room_for`,
    /// `love_chance`, `want_chance`, `web::grazing`), so a stocked place holds its grazers and
    /// doesn't starve them back down.
    pub fn crowd_in(&self, biome: &str) -> u32 {
        if !self.forages_wild() {
            return self.crowd();
        }
        ((self.crowd() as f32 * stocking(biome)).round() as u32).max(2)
    }
    /// Young born or hatched at once (`breeding.rs`), fewest and most.
    pub fn litter(&self) -> (u32, u32) {
        self.tune.litter.unwrap_or_else(|| self.bio.litter())
    }
}

/// What a Cretaceous biome's spawn rules average (`Species::spawn_chance`): about the
/// overworld's plains, whose own animals and base's farm animals (5%) come to this.
pub const CRETACEOUS_FLOW: f32 = 0.025;

/// How heavily each biome is stocked with plant eaters, against their crowd (`Species::crowd_in`;
/// 1 where unlisted, the Cretaceous's too): open grassland heavily, deep forest and the giant
/// groves lightly, as real grazer densities go. With grass feeding as real grass does (Plants'
/// tufts, `vitals::forage`), every grazed biome's herds eat about a third of what its grass
/// grows back, as real herds take 25-50% (`notes/ECOLOGY.md`; Plants' and Animals' grazing plan,
/// 2026-11-05). The savanna is held at 2.5 (3.5 would land it with the rest) until it has a heavy
/// grazer, which eats as much with fewer animals.
pub const STOCKING: &[(&str, f32)] = &[
    ("base:savanna", 2.5),
    ("base:plains", 1.4),
    ("base:forest", 1.0),
    ("base:swamp", 0.9),
    ("base:birch_forest", 0.55),
    ("base:taiga", 0.35),
    ("base:mountains", 0.26),
    ("base:sequoia_grove", 0.09),
];

/// How heavily `biome` is stocked with plant eaters (`STOCKING`, 1 unlisted).
pub fn stocking(biome: &str) -> f32 {
    STOCKING
        .iter()
        .find(|(b, _)| *b == biome)
        .map_or(1.0, |&(_, s)| s)
}

/// ARK's shape: health grows fastest, speed slowest, torpor only from wild points.
pub const WILD_STEP: [f32; STATS] = [0.2, 0.1, 0.1, 0.05, 0.01, 0.02, 0.1, 0.06, 0.05];
pub const TAMED_STEP: [f32; STATS] = [0.1, 0.1, 0.1, 0.04, 0.015, 0.04, 0.1, 0.0, 0.05];

/// What most rows share, written once (`..BASE` ends every row; Research round 5, item 4): a wild
/// species of no breed that hunts nothing, never fights, carries no rider and doesn't fly, on the
/// usual level steps and its research's pace (`Tune::NONE`). The fields every row sets for itself
/// (its name, family, diet, numbers, habitats and biology) are placeholders here.
const BASE: Species = Species {
    model: "",
    name: "",
    binomial: "",
    family: &NO_FAMILY,
    subfamily: "",
    breed: None,
    size: Size::Small,
    eats: &[],
    prey: (0.0, 0.0),
    taming: Taming::Wild,
    base: [0.0; STATS],
    wild_step: WILD_STEP,
    tamed_step: TAMED_STEP,
    hunger: 0.0,
    foods: &[],
    feedings: 0.0,
    shy: 0.0,
    fights: false,
    attacks: &[],
    reach: 0.0,
    signature: "",
    seat: 0.0,
    seat_forward: 0.0,
    flight: 0.0,
    camouflage: 0.0,
    dimorphism: 1.0,
    size_spread: 0.0,
    height: (0.0, ""),
    habitats: &[],
    bio: Biology {
        mass: 0.0,
        gestation: 0.0,
        incubation: 0.0,
        maturity: 0.0,
        lifespan: 0.0,
        interval: 0.0,
        litter: 0.0,
        social: Social::Solitary,
        rarity: Rarity::Common,
        breeds: 0,
        peak: 0,
        comfort: (0.0, 0.0),
        cold_blooded: false,
    },
    tune: Tune::NONE,
    birth_size: 0.0,
};
/// `BASE`'s family, a placeholder every row replaces.
static NO_FAMILY: Family = Family { name: "", above: &[] };

pub static SPECIES: [Species; 46] = [
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
        birth_size: 1.0,
        ..BASE
    },
    Species {
        model: "cat",
        name: "cat",
        binomial: "Felis catus",
        family: &FELIDAE,
        subfamily: "Felinae",
        size: Size::Small,
        eats: &[(Meat, 0.8), (Insects, 0.1), (Carrion, 0.05), (Fish, 0.05)],
        prey: (0.005, 0.4),
        taming: Taming::Passive,
        base: [6.0, 80.0, 60.0, 1.0, 1.0, 30.0, 80.0, 40.0, 10.0],
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(20), ..Tune::NONE },
        birth_size: 0.35,
        ..BASE
    },
    Species {
        model: "capybara",
        name: "capybara",
        binomial: "Hydrochoerus hydrochaeris",
        family: &CAVIIDAE,
        subfamily: "Hydrochoerinae",
        size: Size::Medium,
        eats: &[(Grass, 0.7), (WaterPlants, 0.2), (Seeds, 0.05), (Fruit, 0.05)],
        taming: Taming::Passive,
        // It holds its breath for five minutes.
        base: [12.0, 150.0, 200.0, 1.0, 1.0, 150.0, 400.0, 150.0, 20.0],
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
        reach: 0.8,
        signature: "calm",
        camouflage: 0.8,
        dimorphism: 0.97,
        size_spread: 0.08,
        height: (0.56, "shoulder"),
        // Savannas, flooded grasslands and swamps of South America, always near water (not the
        // forest, whose climate is a German beech wood's: B10).
        habitats: &[("base:swamp", 1.0), ("base:savanna", 0.8)],
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(24), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "raptor",
        name: "raptor",
        binomial: "Velociraptor mongoliensis",
        family: &DROMAEOSAURIDAE,
        subfamily: "Velociraptorinae",
        size: Size::Medium,
        eats: &[(Meat, 0.7), (Carrion, 0.25), (Insects, 0.05)],
        prey: (0.002, 6.0),
        taming: Taming::Knockout,
        base: [14.0, 150.0, 150.0, 4.0, 1.0, 140.0, 150.0, 200.0, 15.0],
        hunger: 20.0,
        foods: &[eats("base:chicken", 10), eats("base:mutton", 10), eats("base:porkchop", 12), favorite("base:beef", 15), KIBBLE],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        attacks: &["bite", "kick"],
        reach: 0.8,
        signature: "pack",
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
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "utahraptor",
        name: "utahraptor",
        binomial: "Utahraptor ostrommaysi",
        family: &DROMAEOSAURIDAE,
        subfamily: "Dromaeosaurinae",
        size: Size::Large,
        eats: &[(Meat, 0.8), (Carrion, 0.15), (Fish, 0.05)],
        prey: (0.01, 3.0),
        taming: Taming::Knockout,
        base: [60.0, 300.0, 400.0, 10.0, 1.0, 400.0, 200.0, 700.0, 30.0],
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
        birth_size: 0.15,
        ..BASE
    },
    // Velociraptor's second species (named 2008, from Bayan Mandahu, Inner Mongolia; known from
    // skull bones only, so the same body and numbers, its own coat).
    Species {
        model: "raptor",
        name: "osmolskae",
        binomial: "Velociraptor osmolskae",
        family: &DROMAEOSAURIDAE,
        subfamily: "Velociraptorinae",
        size: Size::Medium,
        eats: &[(Meat, 0.7), (Carrion, 0.25), (Insects, 0.05)],
        prey: (0.002, 6.0),
        taming: Taming::Knockout,
        base: [14.0, 150.0, 150.0, 4.0, 1.0, 140.0, 150.0, 200.0, 15.0],
        hunger: 20.0,
        foods: &[eats("base:chicken", 10), eats("base:mutton", 10), eats("base:porkchop", 12), favorite("base:beef", 15), KIBBLE],
        feedings: 10.0,
        shy: 8.0,
        fights: true,
        attacks: &["bite", "kick"],
        reach: 0.8,
        signature: "pack",
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
        birth_size: 0.3,
        ..BASE
    },
    // The wyvern, a fantasy animal (`wyvern.rs`): a dragon with two legs and bat wings, about
    // 3.9 m at the shoulder and 16 m long, rare in the mountains.
    Species {
        model: "wyvern",
        name: "wyvern",
        binomial: "Wyverna montana",
        family: &WYVERNIDAE,
        subfamily: "Wyverninae",
        size: Size::Large,
        eats: &[(Meat, 0.8), (Fish, 0.1), (Carrion, 0.1)],
        prey: (0.001, 0.6),
        taming: Taming::Knockout,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [400.0, 800.0, 2000.0, 30.0, 1.0, 1200.0, 400.0, 3000.0, 80.0],
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(8), ..Tune::NONE },
        birth_size: 0.08,
        ..BASE
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
        size: Size::Small,
        // Grasses most of the year, herbs and crops, twigs and bark in winter.
        eats: &[(Grass, 0.55), (Leaves, 0.25), (Roots, 0.1), (Seeds, 0.05), (Fruit, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [5.0, 120.0, 40.0, 0.5, 1.0, 15.0, 60.0, 20.0, 5.0],
        hunger: 30.0,
        foods: &[eats("base:wheat", 8), eats("base:wheat_seeds", 2), favorite("base:apple", 10), favorite("base:berries", 6)],
        // It sits tight in its form, then bolts when a walker comes within about 12 m.
        shy: 12.0,
        reach: 0.8,
        signature: "bolt",
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(26), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "hare",
        name: "mountain_hare",
        binomial: "Lepus timidus",
        family: &LEPORIDAE,
        subfamily: "Leporinae",
        size: Size::Small,
        // Grasses and herbs in summer; in winter birch, willow and heather twigs and bark.
        eats: &[(Grass, 0.4), (Leaves, 0.5), (Fruit, 0.05), (Lichen, 0.05)],
        base: [5.0, 120.0, 40.0, 0.5, 1.0, 13.0, 60.0, 20.0, 5.0],
        hunger: 30.0,
        foods: &[eats("base:crowberries", 4), favorite("base:lingonberries", 6)],
        shy: 12.0,
        reach: 0.8,
        signature: "bolt",
        camouflage: 1.0,
        // Females slightly heavier.
        dimorphism: 0.98,
        size_spread: 0.1,
        height: (0.30, "shoulder"),
        // The boreal forest, birch woods, the tundra and the Alps (700-3,800 m).
        habitats: &[
            ("base:taiga", 1.0),
            ("base:birch_forest", 1.0),
            ("base:snowy_plains", 0.75),
            ("base:mountains", 0.75),
            ("base:sequoia_grove", 0.3),
        ],
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(26), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
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
        size: Size::Small,
        // Seeds and mast most of the year, insects and snails in late spring, berries, fungi and
        // roots later.
        eats: &[(Seeds, 0.45), (Insects, 0.2), (Fruit, 0.15), (Grass, 0.1), (Fungi, 0.05), (Roots, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 2.0, 30.0, 5.0, 2.0],
        hunger: 30.0,
        foods: &[eats("base:wheat_seeds", 4), eats("base:wheat", 6), favorite("base:berries", 4)],
        shy: 6.0,
        reach: 0.5,
        signature: "dart",
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
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { growth: Some(72_000), lifespan: Some(384_000), crowd: Some(30), ..Tune::NONE },
        birth_size: 0.35,
        ..BASE
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
        size: Size::Small,
        // Grasses and herbs, root tubers and moss; bark in winter; a few insect larvae.
        eats: &[(Grass, 0.6), (Leaves, 0.15), (Roots, 0.15), (Seeds, 0.05), (Insects, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 3.0, 30.0, 5.0, 2.0],
        hunger: 30.0,
        foods: &[eats("base:wheat", 6), eats("base:wheat_seeds", 3), favorite("base:sunflower_seeds", 6)],
        shy: 5.0,
        reach: 0.5,
        signature: "dart",
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
            ("base:sequoia_grove", 1.0),
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
        // A life paced as the wood mouse's, so it is grown most of it, and litters of 2-5 (4-6 in a good
        // year, fewer in most), so it doesn't raise more than 40 young in it (`balance::checks`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { growth: Some(72_000), lifespan: Some(384_000), litter: Some((2, 5)), crowd: Some(30), ..Tune::NONE },
        birth_size: 0.35,
        ..BASE
    },
    // The deer (`reports/ecology/Overworld food webs.md`: the big browser and grazer of seven biomes,
    // the wolf's and jaguar's prey): two simple wild creatures on one model (`source/deer`), as the
    // hares. A buck's (a stag's) antlers are a build gene its sex sets (`genetics/*deer.json`).
    Species {
        model: "deer",
        name: "white_tailed_deer",
        binomial: "Odocoileus virginianus",
        family: &CERVIDAE,
        subfamily: "Capreolinae",
        size: Size::Large,
        // A browser: leaves, twigs and buds first, grass and herbs, acorns and other mast, fruit, fungi.
        eats: &[(Leaves, 0.5), (Grass, 0.2), (Fruit, 0.15), (Seeds, 0.1), (Fungi, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [30.0, 200.0, 120.0, 2.0, 1.0, 70.0, 80.0, 60.0, 15.0],
        hunger: 15.0,
        foods: &[eats("base:wheat", 8), favorite("base:apple", 12), favorite("base:berries", 6)],
        shy: 16.0,
        reach: 1.0,
        signature: "bolt",
        camouflage: 0.9,
        dimorphism: 1.08,
        size_spread: 0.1,
        height: (0.95, "shoulder"),
        // The Americas' deer: prairie and woods' edges in Kansas, the llanos' savanna, the desert edge
        // (Coues deer), the Pantanal's marshes (the marsh deer's place).
        habitats: &[("base:plains", 1.0), ("base:savanna", 0.8), ("base:desert", 0.5), ("base:forest", 0.5), ("base:swamp", 0.4)],
        bio: Biology {
            mass: 70.0,
            gestation: 200.0,
            incubation: 0.0,
            // Does breed at about 18 months.
            maturity: 540.0,
            // Few live past 10 in the wild.
            lifespan: 3650.0,
            interval: 365.0,
            // Twins are usual for a grown doe, one or three less often.
            litter: 1.8,
            social: Social::Herd,
            rarity: Rarity::Common,
            // The rut in autumn (late October, November), the fawns born in May and June.
            breeds: seasons::AUTUMN,
            peak: seasons::AUTUMN,
            // Est.: a winter coat for the prairie's frost, shade-seeking in the llanos' midday heat.
            comfort: (-15.0, 30.0),
            cold_blooded: false,
        },
        birth_size: 0.4,
        ..BASE
    },
    Species {
        model: "deer",
        name: "red_deer",
        binomial: "Cervus elaphus",
        family: &CERVIDAE,
        subfamily: "Cervinae",
        size: Size::Large,
        // A mixed feeder: grasses and sedges, browse of trees and shrubs, heather, mast and fruit.
        eats: &[(Grass, 0.5), (Leaves, 0.4), (Fruit, 0.05), (Seeds, 0.05)],
        base: [45.0, 250.0, 180.0, 3.0, 1.0, 120.0, 80.0, 90.0, 20.0],
        hunger: 15.0,
        foods: &[eats("base:wheat", 10), favorite("base:apple", 12)],
        shy: 20.0,
        reach: 1.0,
        signature: "bolt",
        camouflage: 0.9,
        dimorphism: 1.1,
        size_spread: 0.3,
        // The model's (the white-tailed deer's); its size genes draw it about 1.25 times as big: 1.2 m.
        height: (0.95, "shoulder"),
        // Europe's deer of the forests (Białowieża), birch woods and the Alps, wintering lower in the
        // woods, summering high.
        habitats: &[("base:forest", 1.0), ("base:birch_forest", 0.8), ("base:mountains", 0.8), ("base:sequoia_grove", 0.8)],
        bio: Biology {
            mass: 160.0,
            gestation: 250.0,
            incubation: 0.0,
            // Hinds breed in their second or third autumn.
            maturity: 730.0,
            // 10-13 years in the wild, about 15 where predators are few.
            lifespan: 5475.0,
            interval: 365.0,
            litter: 1.0,
            social: Social::Herd,
            rarity: Rarity::Uncommon,
            // The rut from late August into early winter; calves born in late spring.
            breeds: seasons::AUTUMN,
            peak: seasons::AUTUMN,
            // Est.: a cool-country deer, at home in snow, seeking shade and wallows in summer heat.
            comfort: (-20.0, 25.0),
            cold_blooded: false,
        },
        tune: Tune {
            // A hind calves every year and raises few: her rest after a calf set shorter so the wild
            // can replace its dead (`balance::checks`: six young in a life or more).
            recovery: Some(72_000),
            ..Tune::NONE
        },
        birth_size: 0.4,
        ..BASE
    },
    // The fish (`reports/ecology/Overworld food webs.md`: the middle of the water webs): two wild
    // species on one model (`source/fish`), spawned into water by the mod (`fish.rs`). They don't
    // breed in the game yet: no breeding season (their eggs would need water, not a nest), the
    // spawner keeps them. Insects and small fish are their food; hunting them waits for those to be
    // food in the world, so they hunt nothing yet.
    Species {
        model: "fish",
        name: "brown_trout",
        binomial: "Salmo trutta",
        family: &SALMONIDAE,
        subfamily: "Salmoninae",
        size: Size::Small,
        // Insects (larvae and those fallen on the water), crustaceans and snails, small fish.
        eats: &[(Insects, 0.65), (Fish, 0.3), (Carrion, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [3.0, 60.0, 20.0, 0.3, 1.0, 5.0, 400.0, 10.0, 3.0],
        hunger: 30.0,
        foods: &[],
        shy: 6.0,
        reach: 0.5,
        signature: "dart",
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.2,
        height: (0.37, "fork"),
        // Cool, clear rivers, streams and lakes of the temperate and northern country.
        habitats: &[("base:forest", 1.0), ("base:birch_forest", 1.0), ("base:mountains", 1.0), ("base:taiga", 0.8), ("base:plains", 0.6)],
        bio: Biology {
            mass: 0.5,
            gestation: 0.0,
            // The eggs lie in the gravel over winter.
            incubation: 45.0,
            maturity: 1095.0,
            // A well-fed one lives 10-20 years.
            lifespan: 4380.0,
            interval: 365.0,
            // Young that live, of the hundreds of eggs.
            litter: 8.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            breeds: 0,
            peak: 0,
            // The water it does best in: 4-19 degrees; it suffers over 20.
            comfort: (4.0, 20.0),
            cold_blooded: true,
        },
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        // Of a redd's hundreds of eggs, the fry that grow up: 3-8 here, or a group overshoots its room
        // threefold (`web/test.mjs`).
        tune: Tune { litter: Some((3, 8)), crowd: Some(26), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    Species {
        model: "fish",
        name: "pacu",
        binomial: "Piaractus mesopotamicus",
        family: &SERRASALMIDAE,
        subfamily: "Colossomatinae",
        size: Size::Small,
        // Fruit and seeds fallen into the floods (it spreads the forest's seeds), insects, snails
        // and small fish the rest of the year.
        // (Under half plants: it doesn't forage on land, `Species::forages_wild`.)
        eats: &[(Fruit, 0.3), (Seeds, 0.15), (Insects, 0.35), (Fish, 0.2)],
        base: [6.0, 80.0, 40.0, 0.5, 1.0, 20.0, 400.0, 15.0, 4.0],
        hunger: 25.0,
        foods: &[],
        shy: 5.0,
        reach: 0.5,
        signature: "dart",
        camouflage: 0.8,
        dimorphism: 1.0,
        size_spread: 0.4,
        // The model's (the trout's); its size genes draw it about 1.35 times as long: half a metre.
        height: (0.37, "fork"),
        // The Pantanal's rivers and flooded fields, and the llanos'.
        habitats: &[("base:swamp", 1.0), ("base:savanna", 0.8)],
        bio: Biology {
            mass: 4.0,
            gestation: 0.0,
            // Spawned in the rivers at the start of the floods; the eggs hatch within a day.
            incubation: 1.0,
            maturity: 1095.0,
            lifespan: 5475.0,
            interval: 365.0,
            litter: 6.0,
            social: Social::Herd,
            rarity: Rarity::Uncommon,
            breeds: 0,
            peak: 0,
            // Warm water: 20-32 degrees.
            comfort: (20.0, 32.0),
            cold_blooded: true,
        },
        birth_size: 0.05,
        ..BASE
    },
    // The foxes (`reports/ecology/Overworld food webs.md`: the small hunter of ten biomes, the
    // tundra's main one): two wild hunters on one model (`source/fox`), the dog's family (so they
    // leave chihuahuas be), taking voles, mice, hares and birds by size (`hunt.rs`).
    Species {
        model: "fox",
        name: "red_fox",
        binomial: "Vulpes vulpes",
        family: &CANIDAE,
        subfamily: "Caninae",
        size: Size::Small,
        // Small rodents first, rabbits and hares, birds and eggs; insects, berries and other fruit,
        // carrion.
        eats: &[(Meat, 0.5), (Fruit, 0.2), (Insects, 0.15), (Carrion, 0.15)],
        // Voles and mice from about 20 g to hares near its own size (most under a fifth of it).
        prey: (0.004, 0.8),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [10.0, 120.0, 60.0, 2.0, 1.0, 30.0, 80.0, 40.0, 10.0],
        hunger: 30.0,
        foods: &[eats("base:chicken", 10), eats("base:beef", 10), favorite("base:berries", 6)],
        shy: 10.0,
        fights: true,
        attacks: &["pounce", "bite"],
        reach: 0.6,
        signature: "pounce",
        camouflage: 0.6,
        dimorphism: 1.05,
        size_spread: 0.12,
        // The model's (the red fox's); a fox's size genes draw it bigger or smaller.
        height: (0.38, "shoulder"),
        // Woods' edges, farmland and moors across the north; the desert's kit fox and the beach's
        // dune foxes share its row.
        habitats: &[
            ("base:forest", 1.0),
            ("base:birch_forest", 1.0),
            ("base:taiga", 0.8),
            ("base:mountains", 0.8),
            ("base:plains", 0.6),
            ("base:desert", 0.4),
            ("base:beach", 0.4),
            ("base:sequoia_grove", 0.6),
        ],
        bio: Biology {
            mass: 5.5,
            gestation: 52.0,
            incubation: 0.0,
            // Breeding in its first winter, at about ten months.
            maturity: 300.0,
            // Few live past three or four in the wild; up to 10 or more.
            lifespan: 1825.0,
            interval: 365.0,
            litter: 5.0,
            social: Social::Pack,
            rarity: Rarity::Common,
            // Mating in midwinter (December to February), the cubs born in spring.
            breeds: seasons::WINTER,
            peak: seasons::WINTER,
            // Est.: a thick winter coat for the northern cold; it lies up in shade in the heat.
            comfort: (-20.0, 30.0),
            cold_blooded: false,
        },
        // A vixen's rest after her cubs set shorter: with its short wild life she raised two litters,
        // too few for a group to last (`web/test.mjs`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { recovery: Some(60_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.25,
        ..BASE
    },
    Species {
        model: "fox",
        name: "arctic_fox",
        binomial: "Vulpes lagopus",
        family: &CANIDAE,
        subfamily: "Caninae",
        size: Size::Small,
        // Lemmings and voles first (a family eats dozens a day), birds and eggs, fish, carrion
        // (following wolves), berries.
        eats: &[(Meat, 0.55), (Carrion, 0.25), (Fruit, 0.1), (Insects, 0.05), (Fish, 0.05)],
        // Lemmings to an Arctic hare about its own size.
        prey: (0.004, 1.0),
        base: [8.0, 120.0, 60.0, 1.6, 1.0, 22.0, 80.0, 35.0, 12.0],
        hunger: 30.0,
        foods: &[eats("base:chicken", 10), eats("base:beef", 10), favorite("base:berries", 6)],
        shy: 10.0,
        fights: true,
        attacks: &["pounce", "bite"],
        reach: 0.6,
        signature: "pounce",
        camouflage: 0.6,
        dimorphism: 1.05,
        size_spread: 0.35,
        // The model's (the red fox's); a fox's size genes draw it bigger or smaller.
        height: (0.38, "shoulder"),
        // The tundra, into the taiga's northern edge.
        habitats: &[("base:snowy_plains", 1.0), ("base:taiga", 0.3)],
        bio: Biology {
            mass: 3.5,
            gestation: 52.0,
            incubation: 0.0,
            maturity: 300.0,
            // 3-4 years on average after its first, up to 11.
            lifespan: 1825.0,
            interval: 365.0,
            // 6-18 in a lemming year, few or none in a poor one.
            litter: 7.0,
            social: Social::Pack,
            rarity: Rarity::Common,
            // April and May.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // Its winter coat keeps it at ease to -7 degrees before it works harder; it overheats
            // easily.
            comfort: (-35.0, 18.0),
            cold_blooded: false,
        },
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(24), ..Tune::NONE },
        birth_size: 0.25,
        ..BASE
    },
    // The raven (`reports/ecology/Overworld food webs.md`: for the dead in the cold biomes, where
    // there are no vultures): wild, on the ground and in the air (`source/raven`, `raven.rs`).
    Species {
        model: "raven",
        name: "raven",
        binomial: "Corvus corax",
        family: &CORVIDAE,
        subfamily: "Corvinae",
        size: Size::Small,
        // Carrion first (it follows wolves), eggs and small animals, insects, grain, berries and
        // scraps.
        eats: &[(Carrion, 0.4), (Meat, 0.2), (Insects, 0.15), (Seeds, 0.15), (Fruit, 0.1)],
        // Mice and voles to a nestling or a young hare, a tenth of its own mass.
        prey: (0.01, 0.1),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [6.0, 120.0, 40.0, 1.0, 1.0, 10.0, 60.0, 20.0, 5.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 10), eats("base:wheat_seeds", 2), favorite("base:rotten_flesh", 8)],
        shy: 14.0,
        fights: true,
        reach: 0.4,
        signature: "soar",
        camouflage: 0.2,
        dimorphism: 1.08,
        size_spread: 0.1,
        // Standing, at the crown.
        height: (0.34, "crown"),
        // Woods, mountains, the tundra, the desert and the coast: about anywhere in the north, and
        // the dry places to the south.
        habitats: &[
            ("base:taiga", 1.0),
            ("base:mountains", 0.8),
            ("base:forest", 0.6),
            ("base:birch_forest", 0.6),
            ("base:snowy_plains", 0.6),
            ("base:desert", 0.4),
            ("base:plains", 0.3),
            ("base:beach", 0.3),
            ("base:sequoia_grove", 0.8),
        ],
        bio: Biology {
            mass: 1.2,
            gestation: 0.0,
            // 18-21 days, the female sitting; the young fly at five or six weeks.
            incubation: 20.0,
            // Grown at two; pairing and breeding from three or four.
            maturity: 730.0,
            // 10-15 years in the wild, up to 23.
            lifespan: 4380.0,
            interval: 365.0,
            // 3-7 eggs.
            litter: 4.5,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            // Laying from late winter to April.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // From the Arctic's winters to the desert's heat (it pants and keeps to shade).
            comfort: (-30.0, 32.0),
            cold_blooded: false,
        },
        // No nests yet (Ronald, 2026-11-03): eggs laid on the ground would never hatch in the
        // north's cold (`eggs.rs`), so its young come out as fledglings after the eggs' time.
        tune: Tune { gestation: Some(12_700), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // Protoceratops (`reports/ecology/Cretaceous food webs.md`: the Djadokhta's "extremely common"
    // plant eater and the Velociraptors' prey, the Fighting Dinosaurs): wild, in herds of mixed
    // ages (`source/protoceratops`).
    Species {
        model: "protoceratops",
        name: "protoceratops",
        binomial: "Protoceratops andrewsi",
        family: &PROTOCERATOPSIDAE,
        subfamily: "Protoceratopsinae",
        size: Size::Medium,
        // Low browse, chopped and sheared with a strong bite (Button & Zanno 2019; Tanoue et al.
        // 2009): ferns, cycads and shrubs, their roots and seeds.
        eats: &[(Leaves, 0.75), (Roots, 0.15), (Seeds, 0.1)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [35.0, 200.0, 120.0, 2.0, 1.0, 90.0, 80.0, 70.0, 18.0],
        hunger: 15.0,
        foods: &[eats("animals:fern", 8), eats("base:wheat", 8), favorite("animals:cycad_seed", 10)],
        shy: 12.0,
        reach: 1.0,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.05,
        size_spread: 0.1,
        height: (0.6, "hip"),
        // The Djadokhta's dunes first; the scrubland's flats less often.
        habitats: &[("animals:red_dunes", 1.0), ("animals:scrubland", 0.6)],
        bio: Biology {
            mass: 83.0,
            gestation: 0.0,
            // About 83 days, from the growth lines in embryos' teeth (Erickson et al. 2017).
            incubation: 83.0,
            // Breeding at about three, before it is fully grown, as dinosaurs did (Erickson et al.
            // 2007).
            maturity: 1095.0,
            // Est.: growth lines put the oldest at about 13.
            lifespan: 5110.0,
            interval: 365.0,
            // A clutch of 12 or more soft-shelled eggs, buried (Norell et al. 2020).
            litter: 12.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // No record; its nests are buried in the dunes' sand, and its young hatch into the
            // growing season, the wet one, as Velociraptor's.
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // No record: a warm-blooded dinosaur at home in the Gobi's dunes (hot days, cold nights).
            comfort: (8.0, 40.0),
            cold_blooded: false,
        },
        // The most common animal of its dunes and a raptor's food: it grows up at a common
        // species' pace (six days), and its clutches are the eggs that hatch (most of a wild one is
        // lost to storms and egg eaters), so the wild keeps a line's young to a mother's life.
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { growth: Some(144_000), litter: Some((3, 8)), crowd: Some(20), ..Tune::NONE },
        birth_size: 0.15,
        ..BASE
    },
    // Kryptobaatar (`reports/ecology/Cretaceous food webs.md`: the commonest mammal of the
    // Djadokhta, Kielan-Jaworowska et al. 2003): a multituberculate, wild, on the wood mouse's build
    // (`source/kryptobaatar`); what the raptors take when there is no Protoceratops.
    Species {
        model: "kryptobaatar",
        name: "kryptobaatar",
        binomial: "Kryptobaatar dashzevegi",
        family: &DJADOCHTATHERIIDAE,
        subfamily: "Djadochtatheriinae",
        size: Size::Small,
        // An omnivore leaning to animal food (its teeth): insects and their grubs first, seeds,
        // fruit and roots.
        eats: &[(Insects, 0.6), (Seeds, 0.25), (Fruit, 0.1), (Roots, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 2.0, 30.0, 5.0, 2.0],
        hunger: 30.0,
        foods: &[eats("base:wheat_seeds", 4), eats("base:wheat", 6), favorite("base:berries", 4)],
        shy: 6.0,
        reach: 0.5,
        signature: "dart",
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.1,
        // About 1.3 times life (its model's, the wood mouse's build).
        height: (0.05, "shoulder"),
        // The Djadokhta's dunes, and Bayan Mandahu's flats (its sister species, K. mandahuensis).
        habitats: &[("animals:red_dunes", 1.0), ("animals:scrubland", 1.0)],
        bio: Biology {
            mass: 0.04,
            // No record (est.: a small mammal's of its size).
            gestation: 20.0,
            incubation: 0.0,
            maturity: 60.0,
            lifespan: 730.0,
            interval: 30.0,
            // No record (est.: fewer than a mouse's, as for most small mammals of its build).
            litter: 3.5,
            social: Social::Solitary,
            rarity: Rarity::Common,
            // No record: young born into the wet season's plenty.
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // Est.: it burrowed (Kielan-Jaworowska & Gambaryan 1994), out of the dunes' midday heat
            // and the night's cold.
            comfort: (5.0, 40.0),
            cold_blooded: false,
        },
        // As the wood mouse's: a small mammal's real pace would make a line four generations deep
        // in a few game days; its growing up, life and room are set to the common animals' pace.
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { growth: Some(72_000), lifespan: Some(384_000), crowd: Some(30), ..Tune::NONE },
        birth_size: 0.35,
        ..BASE
    },
    // Priscagama (`reports/ecology/Cretaceous food webs.md`: the Djadokhta's lizards, "rather
    // abundant and diverse"): an agama's kin, wild, on the dunes and the scrubland
    // (`source/lizard`); the raptors' and Kryptobaatar's neighbors in the food web, an insect eater.
    Species {
        model: "lizard",
        name: "priscagama",
        binomial: "Priscagama gobiensis",
        family: &PRISCAGAMIDAE,
        subfamily: "Priscagaminae",
        size: Size::Small,
        // Insects first (as agamas), a few leaves and flowers.
        eats: &[(Insects, 0.9), (Leaves, 0.1)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 12.0, 0.2, 1.0, 2.0, 40.0, 5.0, 2.0],
        hunger: 40.0,
        foods: &[favorite("base:spider_eye", 4)],
        shy: 5.0,
        reach: 0.4,
        signature: "bask",
        camouflage: 1.0,
        dimorphism: 1.05,
        size_spread: 0.1,
        // About 1.3 times life (its model's), as the mouse.
        height: (0.04, "shoulder"),
        habitats: &[("animals:red_dunes", 1.0), ("animals:scrubland", 1.0)],
        bio: Biology {
            mass: 0.03,
            gestation: 0.0,
            // No record (est.: an agama's eggs, buried, hatch in two months).
            incubation: 60.0,
            maturity: 365.0,
            lifespan: 2190.0,
            interval: 365.0,
            // No record (est.: an agama's clutch, 4-8 small eggs).
            litter: 4.5,
            social: Social::Solitary,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // Cold-blooded: at ease basking at 25-40 degrees, sluggish below.
            comfort: (12.0, 42.0),
            cold_blooded: true,
        },
        // A lizard's room by its tiny mass (35) would crowd its dunes: as the mouse's.
        tune: Tune { crowd: Some(24), ..Tune::NONE },
        birth_size: 0.2,
        ..BASE
    },
    // The reindeer (`reports/ecology/Overworld food webs.md`: the only big grazer of the tundra, the
    // wolf's prey there): a simple wild creature on its own model, built on the deer's
    // (`source/reindeer`). The only deer whose females carry antlers too: a build gene its sex sets
    // smaller in cows (`genetics/reindeer.json`).
    Species {
        model: "reindeer",
        name: "reindeer",
        binomial: "Rangifer tarandus",
        family: &CERVIDAE,
        subfamily: "Capreolinae",
        size: Size::Large,
        // Lichen through the winter (dug from under the snow), sedges, grasses and willow and birch
        // leaves in summer, mushrooms in autumn.
        eats: &[(Lichen, 0.4), (Grass, 0.35), (Leaves, 0.2), (Fungi, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [40.0, 260.0, 160.0, 3.0, 1.0, 110.0, 80.0, 80.0, 25.0],
        hunger: 15.0,
        foods: &[eats("base:mushroom_brown", 8)],
        shy: 18.0,
        reach: 1.0,
        signature: "crater",
        camouflage: 0.7,
        // Bulls usually 159-182 kg, cows 80-120: about 1.2 times as tall.
        dimorphism: 1.12,
        size_spread: 0.15,
        height: (1.05, "shoulder"),
        // The tundra's herds, wintering in the taiga's edge; the mountain reindeer of Norway's
        // fells.
        habitats: &[("base:snowy_plains", 1.0), ("base:taiga", 0.6), ("base:mountains", 0.3)],
        bio: Biology {
            mass: 120.0,
            // 228-234 days.
            gestation: 230.0,
            incubation: 0.0,
            // Cows breed from one to three years old, by their summer's food.
            maturity: 600.0,
            // Cows live to 17 at most, bulls about four years less; 12 or so is old in the wild.
            lifespan: 4380.0,
            interval: 365.0,
            litter: 1.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // The rut from late September to early November, the calves born in May and June.
            breeds: seasons::AUTUMN,
            peak: seasons::AUTUMN,
            // Hollow hair and countercurrent legs: at ease far below -30; it pants and seeks snow
            // patches and wind in the summer's warmth (est.).
            comfort: (-45.0, 18.0),
            cold_blooded: false,
        },
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune {
            // A calf runs with the herd within a day and doubles its weight in two weeks, and a cow
            // calves nearly every year: growing up and her rest set shorter, so a herd keeps up
            // with its wolves (`balance::checks`: a fourth generation in 25 days, six young in a
            // life).
            growth: Some(110_000),
            recovery: Some(40_000),
            crowd: Some(20),
            ..Tune::NONE
        },
        birth_size: 0.35,
        ..BASE
    },
    // The coyote (`reports/ecology/Overworld food webs.md`: the middle hunter of the prairie and the
    // desert, the hare's, the fawn's and the small dog's hunter): a simple wild hunter on its own
    // model, built on the fox's (`source/coyote`), the dog's family (so it leaves foxes and
    // chihuahuas be for now).
    Species {
        model: "coyote",
        name: "coyote",
        binomial: "Canis latrans",
        family: &CANIDAE,
        subfamily: "Caninae",
        size: Size::Medium,
        // Rabbits, hares and rodents first, deer fawns and birds; carrion, insects, fruit.
        eats: &[(Meat, 0.6), (Fruit, 0.15), (Carrion, 0.15), (Insects, 0.1)],
        // Mice of a few dozen grams to a hare or a fawn, under half its own mass alone (a family takes
        // deer).
        prey: (0.001, 0.4),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [18.0, 150.0, 80.0, 3.5, 1.0, 45.0, 80.0, 60.0, 12.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 12), favorite("base:chicken", 10), eats("base:berries", 4)],
        shy: 14.0,
        fights: true,
        attacks: &["pounce", "bite"],
        reach: 0.7,
        signature: "howl",
        camouflage: 0.6,
        // Males 8-20 kg, females 7-18.
        dimorphism: 1.05,
        size_spread: 0.2,
        height: (0.61, "shoulder"),
        // The prairie and the desert first (its old range), into the woods and the mountains.
        habitats: &[("base:plains", 1.0), ("base:desert", 1.0), ("base:forest", 0.4), ("base:mountains", 0.4), ("base:sequoia_grove", 0.6)],
        bio: Biology {
            // Southern ones about 11.5 kg, northern ones 18.
            mass: 13.0,
            gestation: 63.0,
            incubation: 0.0,
            // Many breed in their first winter, at about ten months.
            maturity: 300.0,
            // Few live past six in the wild; up to 14.
            lifespan: 2190.0,
            interval: 365.0,
            // Six on average, by its food and how many coyotes are about.
            litter: 6.0,
            social: Social::Pack,
            rarity: Rarity::Common,
            // Mating in midwinter (January to March), the pups born in spring.
            breeds: seasons::WINTER,
            peak: seasons::WINTER,
            // Est.: from the northern prairie's winters to the Sonoran summer (it lies up in the shade).
            comfort: (-30.0, 36.0),
            cold_blooded: false,
        },
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(22), ..Tune::NONE },
        birth_size: 0.25,
        ..BASE
    },
    // The small birds (`reports/ecology/Overworld food webs.md`: the songbirds of every land biome, the
    // cat's, the owl's and the hawk's prey): two wild species on one model (`source/small_bird`), on
    // the ground and in the air (`small_bird.rs`). No nests yet (Ronald, 2026-11-03): as the raven's,
    // their young come out as fledglings after the eggs' time.
    Species {
        model: "small_bird",
        name: "house_sparrow",
        binomial: "Passer domesticus",
        family: &PASSERIDAE,
        subfamily: "Passerinae",
        size: Size::Small,
        // Seeds and grain first, insects (most of what the nestlings get), buds and berries.
        eats: &[(Seeds, 0.6), (Insects, 0.3), (Fruit, 0.1)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 1.0, 40.0, 5.0, 2.0],
        hunger: 20.0,
        foods: &[eats("base:wheat_seeds", 2), favorite("base:berries", 3)],
        shy: 8.0,
        reach: 0.2,
        signature: "flock",
        camouflage: 0.6,
        dimorphism: 1.02,
        size_spread: 0.1,
        // Standing, at the crown.
        height: (0.1, "crown"),
        // Wherever people farm and build, from the prairie to the desert's towns and the coast; few in
        // the deep woods. Not in the birch woods', the Alps' or the llanos' dry-season winters, where
        // it lives only on what people leave (`web`: nothing of its diet there then).
        habitats: &[("base:plains", 1.0), ("base:desert", 0.5), ("base:beach", 0.5), ("base:forest", 0.4), ("base:swamp", 0.3)],
        bio: Biology {
            mass: 0.03,
            gestation: 0.0,
            // 11-14 days, both parents sitting; the young fly at about two weeks.
            incubation: 12.0,
            // Breeding in its first spring.
            maturity: 300.0,
            // Most live about three years; up to 13.
            lifespan: 1095.0,
            // Two to four broods a summer.
            interval: 100.0,
            // 4-5 eggs.
            litter: 4.5,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // From the prairie's winters (it roosts in holes and huddles) to the Sonoran heat.
            comfort: (-25.0, 38.0),
            cold_blooded: false,
        },
        // No nests yet: the young come out as fledglings after the eggs' time. Its life paced as the
        // mouse's (a small animal's derived span is too short for a line to be bred), growing up a
        // fifth of it; many live close together, at feeders and in hedges (`balance::checks`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { gestation: Some(8_665), growth: Some(66_000), lifespan: Some(336_000), crowd: Some(30), ..Tune::NONE },
        birth_size: 0.6,
        ..BASE
    },
    Species {
        model: "small_bird",
        name: "snow_bunting",
        binomial: "Plectrophenax nivalis",
        family: &CALCARIIDAE,
        subfamily: "",
        size: Size::Small,
        // Seeds of grasses and sedges, their shoots and willow buds, insects in the Arctic summer
        // (what the young are raised on).
        eats: &[(Seeds, 0.4), (Insects, 0.3), (Grass, 0.15), (Leaves, 0.1), (Fruit, 0.05)],
        base: [2.0, 60.0, 15.0, 0.2, 1.0, 1.2, 40.0, 5.0, 3.0],
        hunger: 20.0,
        foods: &[eats("base:wheat_seeds", 2), favorite("base:berries", 3)],
        shy: 8.0,
        reach: 0.2,
        signature: "flock",
        camouflage: 0.6,
        dimorphism: 1.03,
        size_spread: 0.1,
        // The model's (the house sparrow's): the bunting is as big.
        height: (0.1, "crown"),
        // The high Arctic's tundra and the fells, wintering south into the taiga's edge.
        habitats: &[("base:snowy_plains", 1.0), ("base:mountains", 0.4), ("base:taiga", 0.3)],
        bio: Biology {
            mass: 0.035,
            gestation: 0.0,
            // 12-13 days; the young leave the nest at 12-14.
            incubation: 12.5,
            maturity: 300.0,
            // Two or three years on average; up to 8.
            lifespan: 1095.0,
            interval: 365.0,
            // 4-6 eggs.
            litter: 5.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // Laying in June, in the Arctic's short summer.
            breeds: seasons::SUMMER,
            peak: seasons::SUMMER,
            // A songbird of the Arctic winter, panting above about 10 degrees in the sun.
            comfort: (-40.0, 18.0),
            cold_blooded: false,
        },
        // No nests yet: the young come out as fledglings after the eggs' time; paced as the sparrow,
        // a second brood in a long summer, flocks close together (`balance::checks`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune {
            gestation: Some(8_935),
            growth: Some(66_000),
            lifespan: Some(336_000),
            recovery: Some(60_000),
            crowd: Some(30),
            ..Tune::NONE
        },
        birth_size: 0.6,
        ..BASE
    },
    // The owls (`reports/ecology/Overworld food webs.md`: the night's hunter of every land biome, the
    // rodent's and the hare's): two wild species on one model (`source/owl`), on the ground and in
    // the air (`owl.rs`). No nests yet (Ronald, 2026-11-03): as the raven's, their young come out as
    // fledglings after the eggs' time.
    Species {
        model: "owl",
        name: "great_horned_owl",
        binomial: "Bubo virginianus",
        family: &STRIGIDAE,
        subfamily: "Striginae",
        size: Size::Small,
        // Rodents and rabbits first, birds up to geese, skunks, frogs, fish, big insects.
        eats: &[(Meat, 0.9), (Insects, 0.05), (Fish, 0.05)],
        // A deer mouse of 20 g to a hare or a skunk near twice its own mass.
        prey: (0.005, 1.8),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [8.0, 100.0, 40.0, 2.5, 1.0, 14.0, 60.0, 20.0, 6.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 10), favorite("base:chicken", 10)],
        shy: 14.0,
        fights: true,
        reach: 0.5,
        signature: "silent",
        camouflage: 0.7,
        // Females are the bigger.
        dimorphism: 0.95,
        size_spread: 0.15,
        // Perched upright, at the crown (the tufts above it).
        height: (0.45, "crown"),
        // The Americas' owl of every land but the high Arctic: woods' edges first, the prairie, the
        // desert's canyons, the swamps.
        habitats: &[
            ("base:forest", 1.0),
            ("base:birch_forest", 0.8),
            ("base:taiga", 0.8),
            ("base:plains", 0.8),
            ("base:mountains", 0.6),
            ("base:desert", 0.6),
            ("base:swamp", 0.6),
            ("base:savanna", 0.4),
            ("base:sequoia_grove", 0.8),
        ],
        bio: Biology {
            mass: 1.4,
            gestation: 0.0,
            // 30-37 days, the female sitting; the young climb out at six weeks, fly at nine or ten.
            incubation: 33.0,
            // Breeding at two.
            maturity: 730.0,
            // About 13 years in the wild; up to 28.
            lifespan: 4745.0,
            interval: 365.0,
            // 1-4 eggs, usually two.
            litter: 2.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            // The earliest nester: laying in January and February, hooting through the winter.
            breeds: seasons::WINTER,
            peak: seasons::WINTER,
            // From the taiga's winter to the Sonoran heat.
            comfort: (-35.0, 38.0),
            cold_blooded: false,
        },
        tune: Tune { gestation: Some(18_505), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "owl",
        name: "snowy_owl",
        binomial: "Bubo scandiacus",
        family: &STRIGIDAE,
        subfamily: "Striginae",
        size: Size::Small,
        // Lemmings and voles above all (3-5 a day), ptarmigan, Arctic hares, seabirds.
        eats: &[(Meat, 1.0)],
        // A lemming of 50 g to a hare about its own mass.
        prey: (0.005, 1.2),
        base: [10.0, 110.0, 45.0, 3.0, 1.0, 20.0, 60.0, 24.0, 8.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 10), favorite("base:chicken", 10)],
        shy: 16.0,
        fights: true,
        reach: 0.5,
        signature: "silent",
        camouflage: 0.7,
        dimorphism: 0.95,
        size_spread: 0.12,
        // The model's (the great horned owl's); a snowy owl's size genes draw it bigger.
        height: (0.45, "crown"),
        // The open tundra, hunting by day in the Arctic summer; south into the taiga and the fells in
        // a hard winter.
        habitats: &[("base:snowy_plains", 1.0), ("base:taiga", 0.3), ("base:mountains", 0.3)],
        bio: Biology {
            mass: 2.0,
            gestation: 0.0,
            // 31-33 days.
            incubation: 32.0,
            maturity: 730.0,
            // About ten years in the wild.
            lifespan: 3650.0,
            interval: 365.0,
            // 3-11 eggs by the lemmings, none in a poor year.
            litter: 6.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            // Laying in May on the tundra.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // Feathered to the toes: at ease far below -40, panting in the Arctic summer's sun.
            comfort: (-50.0, 16.0),
            cold_blooded: false,
        },
        tune: Tune { gestation: Some(18_083), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // The eagles (`reports/ecology/Overworld food webs.md`: the day's big hunter of every land biome,
    // the hare's, the marmot's, the fawn's and the small dog's): the golden eagle of the north's and
    // the dry country's open land, and the savanna hawk of the llanos and the Pantanal, where there
    // are no golden eagles: two wild species on one model (`source/eagle`), the hawk drawn smaller
    // by its size genes; on the ground and in the air (`eagle.rs`). No nests yet (Ronald,
    // 2026-11-03): their young come out as fledglings after the eggs' time.
    Species {
        model: "eagle",
        name: "golden_eagle",
        binomial: "Aquila chrysaetos",
        family: &ACCIPITRIDAE,
        subfamily: "Aquilinae",
        size: Size::Medium,
        // Hares, rabbits, marmots and ground squirrels first, grouse, foxes and young deer; carrion
        // in winter.
        eats: &[(Meat, 0.85), (Carrion, 0.15)],
        // A vole to a fawn or a fox, up to about one and a half times its own mass.
        prey: (0.01, 1.5),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [16.0, 150.0, 60.0, 4.0, 1.0, 30.0, 60.0, 40.0, 10.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 12), favorite("base:chicken", 10), eats("base:rotten_flesh", 4)],
        shy: 24.0,
        fights: true,
        reach: 0.6,
        signature: "stoop",
        camouflage: 0.4,
        // Females are the bigger (a third heavier).
        dimorphism: 0.92,
        size_spread: 0.1,
        // Perched, at the crown.
        height: (0.6, "crown"),
        // Mountains first, the prairie and the desert's canyons, the taiga and the tundra's edge.
        habitats: &[
            ("base:mountains", 1.0),
            ("base:plains", 0.6),
            ("base:desert", 0.6),
            ("base:taiga", 0.5),
            ("base:snowy_plains", 0.3),
            ("base:forest", 0.3),
            ("base:birch_forest", 0.3),
            ("base:sequoia_grove", 0.3),
        ],
        bio: Biology {
            mass: 4.5,
            gestation: 0.0,
            // 41-45 days, the female sitting; the young fly at ten or eleven weeks.
            incubation: 43.0,
            // In adult plumage and breeding at four or five.
            maturity: 1500.0,
            // About 20 years in the wild; up to 32.
            lifespan: 7300.0,
            interval: 365.0,
            // 1-4 eggs, usually two (often one fledges).
            litter: 2.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            // Laying in March and April.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // From the Arctic's edge to the Sonoran canyons.
            comfort: (-40.0, 38.0),
            cold_blooded: false,
        },
        // No nests yet: the young come out as fledglings after the eggs' time. A young eagle flies in
        // its first summer and looks grown in a year; it waits years to breed, which no line of
        // breeding in the game could wait for (`balance::checks`: a fourth generation in 40 days).
        tune: Tune { gestation: Some(22_568), growth: Some(200_000), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "eagle",
        name: "savanna_hawk",
        binomial: "Buteogallus meridionalis",
        family: &ACCIPITRIDAE,
        subfamily: "Buteoninae",
        size: Size::Small,
        // Rodents, lizards, snakes, frogs and crabs, big insects; following fires and the plough.
        eats: &[(Meat, 0.7), (Insects, 0.2), (Fish, 0.1)],
        prey: (0.005, 0.6),
        base: [8.0, 110.0, 40.0, 2.0, 1.0, 10.0, 60.0, 20.0, 6.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 8), favorite("base:chicken", 8)],
        shy: 16.0,
        fights: true,
        reach: 0.5,
        signature: "stoop",
        camouflage: 0.4,
        dimorphism: 0.95,
        // Its size genes draw the golden eagle's model about two thirds as big (55 cm).
        size_spread: 0.4,
        height: (0.6, "crown"),
        // The llanos' and the Pantanal's open grassland and marsh edges.
        habitats: &[("base:savanna", 1.0), ("base:swamp", 0.8)],
        bio: Biology {
            mass: 0.85,
            gestation: 0.0,
            // About 39 days, one egg.
            incubation: 39.0,
            maturity: 730.0,
            lifespan: 4380.0,
            interval: 365.0,
            litter: 1.0,
            social: Social::Solitary,
            rarity: Rarity::Common,
            // Nesting in the dry season (December to March in the llanos).
            breeds: seasons::WINTER,
            peak: seasons::WINTER,
            // A tropical hawk: at ease in the heat, uneasy below about 5 degrees (est.).
            comfort: (5.0, 42.0),
            cold_blooded: false,
        },
        // No nests yet: fledglings after the eggs' time. One young a year: her rest set shorter so
        // the wild replaces its dead (`balance::checks`: six young in a life).
        tune: Tune { gestation: Some(20_975), recovery: Some(60_000), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // The turkey vulture (`reports/ecology/Overworld food webs.md`: something for the dead in the warm
    // biomes, the raven's counterpart): a wild bird on its own model (`source/vulture`), on the
    // ground and in the air (`vulture.rs`), finding the dead by smell from the air. It hunts nothing;
    // carcasses as food in the world come with the web's third tier. No nests yet (Ronald,
    // 2026-11-03): its young come out as fledglings after the eggs' time.
    Species {
        model: "vulture",
        name: "turkey_vulture",
        binomial: "Cathartes aura",
        family: &CATHARTIDAE,
        subfamily: "",
        size: Size::Small,
        // Carrion almost only, found by smell; rotting fruit now and then (the rare newborn it kills is
        // left out: it hunts nothing here).
        eats: &[(Carrion, 0.95), (Fruit, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [10.0, 140.0, 50.0, 1.0, 1.0, 14.0, 60.0, 24.0, 10.0],
        hunger: 40.0,
        foods: &[favorite("base:rotten_flesh", 10)],
        shy: 16.0,
        reach: 0.4,
        signature: "scent",
        camouflage: 0.2,
        dimorphism: 1.0,
        size_spread: 0.1,
        // Standing, at the crown.
        height: (0.5, "crown"),
        // The llanos and the Pantanal above all, the desert and the prairie, a few over the woods and
        // the mountains.
        habitats: &[
            ("base:savanna", 1.0),
            ("base:swamp", 1.0),
            ("base:desert", 0.8),
            ("base:plains", 0.6),
            ("base:forest", 0.3),
            ("base:mountains", 0.3),
            ("base:sequoia_grove", 0.4),
        ],
        bio: Biology {
            mass: 1.4,
            gestation: 0.0,
            // 38-41 days, both parents sitting; the young fly at 60-80 days.
            incubation: 40.0,
            // Breeding at about three.
            maturity: 1095.0,
            // About 16 years in the wild.
            lifespan: 5840.0,
            interval: 365.0,
            // 1-3 eggs, usually two.
            litter: 2.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // It leaves the north's winters; at ease in the tropics' heat (cooling by urohidrosis).
            comfort: (-12.0, 42.0),
            cold_blooded: false,
        },
        // No nests yet: fledglings after the eggs' time. A young vulture is grown in its first year
        // and breeds at three, which no line in the game could wait for (`balance::checks`).
        // Roosts of dozens, but spread over a big range by day: room near as a herd's at most.
        // Her rest after a brood set shorter, so a roost of a few lasts on its own (`web/test.mjs`).
        tune: Tune { gestation: Some(21_377), growth: Some(110_000), recovery: Some(80_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // The herring gull (`reports/ecology/Overworld food webs.md`: the coast's hunter and its eater of
    // the dead): a wild bird on its own model (`source/gull`), on the ground and in the air
    // (`gull.rs`). No nests yet (Ronald, 2026-11-03): its young come out as fledglings after the
    // eggs' time.
    Species {
        model: "gull",
        name: "herring_gull",
        binomial: "Larus argentatus",
        family: &LARIDAE,
        subfamily: "Larinae",
        size: Size::Small,
        // Fish first, crabs and mussels, the strandline's dead, eggs and young birds, insects, scraps.
        eats: &[(Fish, 0.45), (Carrion, 0.25), (Meat, 0.1), (Insects, 0.1), (Seeds, 0.05), (Fruit, 0.05)],
        // Small fish and mice to a young rabbit or a duckling, under a third of its mass.
        prey: (0.005, 0.3),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [8.0, 130.0, 40.0, 1.5, 1.0, 10.0, 70.0, 20.0, 8.0],
        hunger: 30.0,
        foods: &[eats("base:beef", 8), favorite("base:bread", 6), eats("base:rotten_flesh", 6)],
        shy: 10.0,
        fights: true,
        reach: 0.5,
        signature: "colony",
        camouflage: 0.2,
        // Males a little bigger.
        dimorphism: 1.05,
        size_spread: 0.1,
        // Standing, at the crown.
        height: (0.45, "crown"),
        // The beaches and the sea's shore; inland along rivers to fields and dumps.
        habitats: &[("base:beach", 1.0), ("base:ocean", 0.6), ("base:plains", 0.1), ("base:swamp", 0.2)],
        bio: Biology {
            mass: 1.0,
            gestation: 0.0,
            // 28-30 days, both parents sitting; the young fly at 35-40 days.
            incubation: 29.0,
            // In adult plumage and breeding at four.
            maturity: 1460.0,
            // About 12 years on average; over 30 known.
            lifespan: 4380.0,
            interval: 365.0,
            // Three eggs, usually.
            litter: 3.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // Laying in April and May, in colonies.
            breeds: seasons::SPRING,
            peak: seasons::SPRING,
            // From the North Sea's winters to the Mediterranean's summer.
            comfort: (-15.0, 34.0),
            cold_blooded: false,
        },
        // No nests yet: fledglings after the eggs' time. A young gull flies in five or six weeks and
        // is grown in a year; it waits four years to breed, which no line in the game could wait for
        // (`balance::checks`).
        tune: Tune { gestation: Some(16_796), growth: Some(110_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // The caimans (`reports/ecology/Overworld food webs.md`: the water's hunter of the swamp and the
    // savanna, the jaguar's main prey in the Pantanal and the capybara's danger): the yacare caiman of
    // the Pantanal and the spectacled caiman of the llanos, two wild species on one model
    // (`source/caiman`), the spectacled drawn smaller by its size genes. Cold-blooded; eggs in nest
    // mounds in the rains.
    Species {
        model: "caiman",
        name: "yacare_caiman",
        binomial: "Caiman yacare",
        family: &ALLIGATORIDAE,
        subfamily: "Caimaninae",
        size: Size::Large,
        // Fish first, snails and crabs, water birds, rodents and the young of capybaras; it eats
        // little, being cold-blooded.
        eats: &[(Fish, 0.6), (Meat, 0.3), (Insects, 0.1)],
        // A snail or a minnow to a capybara near its own mass.
        prey: (0.005, 1.0),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [30.0, 160.0, 120.0, 5.0, 1.0, 80.0, 300.0, 60.0, 30.0],
        hunger: 60.0,
        foods: &[eats("base:beef", 14), favorite("base:chicken", 12), eats("base:porkchop", 12)],
        shy: 8.0,
        fights: true,
        attacks: &["bite"],
        reach: 1.0,
        signature: "ambush",
        camouflage: 0.8,
        // Males the longer by about a fifth.
        dimorphism: 1.2,
        size_spread: 0.2,
        // Its length, from the snout to the tail's tip.
        height: (2.2, "length"),
        // The Pantanal's lagoons and flooded grassland, by the thousand at the dry season's pools.
        habitats: &[("base:swamp", 1.0), ("base:savanna", 0.3)],
        bio: Biology {
            mass: 35.0,
            gestation: 0.0,
            // 70-75 days in a mound of rotting plants the female guards.
            incubation: 73.0,
            // Females lay at about 1.2 m long, at six to eight years.
            maturity: 2555.0,
            // 30 to 40 years.
            lifespan: 12775.0,
            interval: 365.0,
            litter: 30.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // Nesting in the rains, the young hatching as the water rises.
            breeds: seasons::WET,
            peak: seasons::SUMMER,
            // Cold-blooded: at ease at 25-35 degrees, basking or cooling in the water, sluggish below.
            comfort: (20.0, 36.0),
            cold_blooded: true,
        },
        // A clutch of 20-35 eggs, most of which jaguars, foxes, coatis and birds take: a dozen at
        // most, so a pen doesn't flood. Growing up and a life paced so a line can be bred, and a
        // female's rest after a clutch long, as she guards the mound and the young for months
        // (`balance::checks`: a fourth generation in 25 days, at most 40 young in a life).
        // Clutches of 4-8 here: a dozen made a group overshoot its room threefold (`web/test.mjs`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune {
            litter: Some((4, 8)),
            growth: Some(150_000),
            lifespan: Some(1_800_000),
            recovery: Some(400_000),
            crowd: Some(26),
            ..Tune::NONE
        },
        birth_size: 0.15,
        ..BASE
    },
    Species {
        model: "caiman",
        name: "spectacled_caiman",
        binomial: "Caiman crocodilus",
        family: &ALLIGATORIDAE,
        subfamily: "Caimaninae",
        size: Size::Large,
        // Fish first, snails and crabs, water birds, rodents and the young of capybaras; it eats
        // little, being cold-blooded.
        eats: &[(Fish, 0.6), (Meat, 0.3), (Insects, 0.1)],
        // A snail or a minnow to a capybara near its own mass.
        prey: (0.005, 1.0),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [26.0, 150.0, 100.0, 4.5, 1.0, 60.0, 300.0, 50.0, 26.0],
        hunger: 60.0,
        foods: &[eats("base:beef", 14), favorite("base:chicken", 12), eats("base:porkchop", 12)],
        shy: 8.0,
        fights: true,
        attacks: &["bite"],
        reach: 1.0,
        signature: "ambush",
        camouflage: 0.8,
        // Males the longer by about a fifth.
        dimorphism: 1.2,
        size_spread: 0.2,
        // Its length, from the snout to the tail's tip.
        height: (2.2, "length"),
        // The llanos' rivers, lagoons and flooded savanna.
        habitats: &[("base:savanna", 1.0), ("base:swamp", 0.5)],
        bio: Biology {
            mass: 25.0,
            gestation: 0.0,
            // 70-75 days in a mound of rotting plants the female guards.
            incubation: 73.0,
            // Females lay at about 1.2 m long, at six to eight years.
            maturity: 2555.0,
            // 30 to 40 years.
            lifespan: 12775.0,
            interval: 365.0,
            litter: 25.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // Nesting in the rains, the young hatching as the water rises.
            breeds: seasons::WET,
            peak: seasons::SUMMER,
            // Cold-blooded: at ease at 25-35 degrees, basking or cooling in the water, sluggish below.
            comfort: (20.0, 36.0),
            cold_blooded: true,
        },
        // A clutch of 20-35 eggs, most of which jaguars, foxes, coatis and birds take: a dozen at
        // most, so a pen doesn't flood. Growing up and a life paced so a line can be bred, and a
        // female's rest after a clutch long, as she guards the mound and the young for months
        // (`balance::checks`: a fourth generation in 25 days, at most 40 young in a life).
        // Clutches of 4-8 here: a dozen made a group overshoot its room threefold (`web/test.mjs`).
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune {
            litter: Some((4, 8)),
            growth: Some(150_000),
            lifespan: Some(1_800_000),
            recovery: Some(400_000),
            crowd: Some(26),
            ..Tune::NONE
        },
        birth_size: 0.15,
        ..BASE
    },
    // The grey wolf (`reports/ecology/Overworld food webs.md`: the top hunter of the forests, the
    // taiga, the tundra and the mountains, the deer's, the reindeer's and the boar's): a full creature
    // on its own model built on the coyote's (`source/wolf`), and the dog's ancestor, so a natural
    // tame: fed from the hand it comes round, as the cat does (`wolf.rs`).
    Species {
        model: "wolf",
        name: "grey_wolf",
        binomial: "Canis lupus",
        family: &CANIDAE,
        subfamily: "Caninae",
        size: Size::Medium,
        // Hoofed animals above all (red and roe deer, moose and reindeer, boar, bison calves), hares
        // and beavers; carrion, its own kills' and others'.
        eats: &[(Meat, 0.85), (Carrion, 0.15)],
        // A hare to a deer or a caribou near three times its mass alone (lone wolves kill caribou; a
        // pack takes moose, elk and bison).
        prey: (0.02, 3.0),
        taming: Taming::Passive,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [30.0, 220.0, 120.0, 6.0, 1.0, 60.0, 80.0, 90.0, 20.0],
        hunger: 40.0,
        foods: &[
            eats("base:beef", 10),
            eats("base:porkchop", 10),
            eats("base:chicken", 6),
            eats("base:cooked_beef", 14),
            eats("base:cooked_mutton", 14),
            favorite("base:mutton", 12),
            KIBBLE,
        ],
        // Shy of people and slow to trust: twice the cat's feedings.
        feedings: 10.0,
        shy: 20.0,
        fights: true,
        attacks: &["pounce", "bite"],
        reach: 0.9,
        signature: "pack",
        camouflage: 0.5,
        // Males a little bigger (about a fifth heavier).
        dimorphism: 1.07,
        size_spread: 0.2,
        height: (0.8, "shoulder"),
        // The taiga first, the forests and birch woods, the mountains, the tundra; rare on the prairie
        // now.
        habitats: &[
            ("base:taiga", 1.0),
            ("base:forest", 0.6),
            ("base:birch_forest", 0.6),
            ("base:mountains", 0.5),
            ("base:snowy_plains", 0.4),
            ("base:plains", 0.1),
            ("base:sequoia_grove", 0.3),
        ],
        bio: Biology {
            mass: 42.0,
            gestation: 63.0,
            incubation: 0.0,
            // Breeding at about 22 months, often later in a pack (only the leading pair breeds).
            maturity: 660.0,
            // Five to seven years in the wild, 12-16 kept: a tame's (the game's wolves are the dog's
            // ancestor, raised and kept), so it is grown most of its life (`balance::checks`).
            lifespan: 4380.0,
            interval: 365.0,
            // 4-6 pups, up to 11.
            litter: 5.5,
            social: Social::Pack,
            rarity: Rarity::Uncommon,
            // Mating from January to March, the pups born in spring.
            breeds: seasons::WINTER,
            peak: seasons::WINTER,
            // At ease far below -40 in its winter coat; it pants and lies in the shade in summer.
            comfort: (-40.0, 28.0),
            cold_blooded: false,
        },
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { crowd: Some(16), ..Tune::NONE },
        birth_size: 0.2,
        ..BASE
    },
    // The jaguar (`reports/ecology/Overworld food webs.md`: the top hunter of the swamp and the savanna,
    // which in the Pantanal eats mostly caiman, capybara and fish): a full creature on its own model
    // built on the cat's (`source/jaguar`). Too dangerous to feed from the hand: tamed as the raptors
    // are, knocked out and fed while it sleeps (`knockout.rs`).
    Species {
        model: "jaguar",
        name: "jaguar",
        binomial: "Panthera onca",
        family: &FELIDAE,
        subfamily: "Pantherinae",
        size: Size::Large,
        // Capybaras, caimans and peccaries first, deer, giant anteaters, fish, turtles; cattle calves.
        eats: &[(Meat, 0.85), (Fish, 0.15)],
        // A fish of a few hundred grams to a capybara or a caiman near its own mass (prey of 1-130 kg,
        // 45-85 kg liked best).
        prey: (0.01, 1.5),
        taming: Taming::Knockout,
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [45.0, 260.0, 200.0, 9.0, 1.0, 120.0, 160.0, 300.0, 25.0],
        hunger: 20.0,
        foods: &[eats("base:chicken", 8), eats("base:porkchop", 14), favorite("base:beef", 18), eats("base:mutton", 14), KIBBLE],
        feedings: 14.0,
        shy: 12.0,
        fights: true,
        attacks: &["swipe", "bite"],
        reach: 1.0,
        signature: "skull_bite",
        camouflage: 0.8,
        // Females 10-20% smaller.
        dimorphism: 1.1,
        size_spread: 0.15,
        height: (0.7, "shoulder"),
        // The Pantanal's marshes and gallery forest first, the llanos, a few in the Sonoran desert's
        // canyons (the northernmost jaguars).
        habitats: &[("base:swamp", 1.0), ("base:savanna", 0.6), ("base:desert", 0.05)],
        bio: Biology {
            // Pantanal males 90-100 kg, females 70.
            mass: 85.0,
            // 91-111 days.
            gestation: 100.0,
            incubation: 0.0,
            // Females at about 2.5 years, males at 3-4.
            maturity: 900.0,
            // Rarely past 11 in the wild; up to 22 kept.
            lifespan: 6200.0,
            // The cubs stay with her nearly two years.
            interval: 700.0,
            // One or two in the Pantanal.
            litter: 1.6,
            social: Social::Solitary,
            // The Pantanal has the most jaguars anywhere (about 12 in 100 km2): uncommon in its
            // biomes rather than rare.
            rarity: Rarity::Uncommon,
            // Mating all year, more in the rains.
            breeds: seasons::ALL_YEAR,
            peak: seasons::WET,
            // A tropical cat at ease in the heat (it cools off swimming), uneasy in frost.
            comfort: (8.0, 38.0),
            cold_blooded: false,
        },
        // A mother's rest after a litter set shorter than her two years with the cubs, so the wild
        // replaces its dead (`balance::checks`: six young in a life); shorter again (100 000
        // ticks) when a solitary cat's small group, two pairs, died out in its first months.
        // Room near raised so a small wild group of its kind lasts on its own (`web/test.mjs`: two pairs
        // left alone at their room neither die out nor flood); the food web's grazing still holds.
        tune: Tune { recovery: Some(100_000), crowd: Some(12), ..Tune::NONE },
        birth_size: 0.12,
        ..BASE
    },
    // The Cretaceous's fish (`reports/ecology/Cretaceous food webs.md`, tier 3: fish as a catch in its
    // waters), two wild species on the fish model, spawned into water as the overworld's are
    // (`fish.rs`): the Cedar Mountain floodplain's lungfish and the Yixian lakes' Lycoptera, what the
    // Utahraptor, the crocodyliform and the early birds take from the water.
    Species {
        model: "fish",
        name: "lungfish",
        binomial: "Ceratodus kempae",
        family: &CERATODONTIDAE,
        subfamily: "",
        size: Size::Small,
        // Snails, insect larvae and crayfish, water plants, small fish.
        eats: &[(Insects, 0.5), (WaterPlants, 0.2), (Fish, 0.3)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [8.0, 90.0, 40.0, 0.6, 1.0, 12.0, 600.0, 20.0, 6.0],
        hunger: 30.0,
        foods: &[],
        shy: 6.0,
        reach: 0.5,
        signature: "dart",
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.6,
        // The model's (the trout's); its size genes draw it about 1.55 times as long: 57 cm (Ceratodus
        // reached about a metre).
        height: (0.37, "fork"),
        // The Cedar Mountain Formation's rivers and oxbow ponds (Ceratodus teeth are common there).
        habitats: &[("animals:floodplain", 1.0)],
        bio: Biology {
            mass: 3.0,
            gestation: 0.0,
            incubation: 30.0,
            maturity: 1825.0,
            lifespan: 7300.0,
            interval: 365.0,
            litter: 6.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            breeds: 0,
            peak: 0,
            // A warm floodplain's rivers and ponds; it breathes air through the dry season.
            comfort: (14.0, 32.0),
            cold_blooded: true,
        },
        // Slow and long-lived, so fewer young a spawning season than the trout and growing a
        // little sooner, or a cell fills with lungfish (the balance checks' generation and young).
        tune: Tune { growth: Some(240_000), litter: Some((1, 4)), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    Species {
        model: "fish",
        name: "lycoptera",
        binomial: "Lycoptera davidi",
        family: &LYCOPTERIDAE,
        subfamily: "",
        size: Size::Small,
        // Insect larvae and plankton.
        eats: &[(Insects, 1.0)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [1.0, 40.0, 8.0, 0.1, 1.0, 1.0, 300.0, 4.0, 1.0],
        hunger: 30.0,
        foods: &[],
        shy: 6.0,
        reach: 0.5,
        signature: "dart",
        camouflage: 1.0,
        dimorphism: 1.0,
        size_spread: 0.5,
        // The model's (the trout's); its size genes draw it about 0.55 times as long: 20 cm, about twice
        // life (a 6-12 cm fish would be a speck).
        height: (0.37, "fork"),
        // The Yixian Formation's lakes (whole schools in its shales).
        habitats: &[("animals:fern_forest", 1.0)],
        bio: Biology {
            mass: 0.02,
            gestation: 0.0,
            incubation: 5.0,
            maturity: 365.0,
            lifespan: 1460.0,
            interval: 365.0,
            litter: 8.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: 0,
            peak: 0,
            // The Yixian's cool lakes.
            comfort: (4.0, 24.0),
            cold_blooded: true,
        },
        // A small fish of a short life: grown in its first season, and schools that keep to a
        // room a pond holds (the balance checks' growth and room).
        tune: Tune { growth: Some(70_000), crowd: Some(30), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    // Psittacosaurus (`reports/ecology/Cretaceous food webs.md`: the Jehol's "dominant" plant eater,
    // the commonest dinosaur of the Yixian, and the Utahraptor's prey where it is rare): wild, the
    // young in groups of one age (Zhao et al. 2014), on its hind legs (`source/psittacosaurus`).
    Species {
        model: "psittacosaurus",
        name: "psittacosaurus",
        binomial: "Psittacosaurus lujiatunensis",
        family: &PSITTACOSAURIDAE,
        subfamily: "",
        size: Size::Medium,
        // Tough plants ground with stomach stones, even as hatchlings (Wang et al. 2025): leaves and
        // fronds first, seeds and nuts, some fruit.
        eats: &[(Leaves, 0.6), (Seeds, 0.3), (Fruit, 0.1)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [22.0, 160.0, 80.0, 1.5, 1.1, 45.0, 70.0, 45.0, 14.0],
        hunger: 15.0,
        foods: &[eats("animals:fern", 8), eats("animals:ginkgo_seed", 8), favorite("base:berries", 10)],
        shy: 12.0,
        reach: 1.0,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.05,
        size_spread: 0.1,
        height: (0.5, "hip"),
        // The Yixian's forests and lake shores first; the floodplain and the scrub less often.
        habitats: &[("animals:fern_forest", 1.0), ("animals:floodplain", 0.3), ("animals:scrubland", 0.3)],
        bio: Biology {
            mass: 25.0,
            gestation: 0.0,
            // No record; Protoceratops's 83 days (Erickson et al. 2017), for a smaller egg.
            incubation: 75.0,
            // Growth lines: grown at about 9 to 11 (Erickson et al. 2009); breeding before that, as
            // dinosaurs did, at about four (est.).
            maturity: 1460.0,
            lifespan: 4015.0,
            interval: 365.0,
            // Clutches are big: a nest of 24 young with an adult (Meng et al. 2004).
            litter: 12.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            // The Jehol's seasons: its young hatch into the wet summer (est.).
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The Jehol's cool, seasonal forests (a mean year near 10 °C: Amiot et al. 2011).
            comfort: (0.0, 32.0),
            cold_blooded: false,
        },
        // The commonest animal of its forest and the hunters' food: a common species' pace, a wild
        // clutch's young kept to what a mother raises, and a room that lets a small group last
        // (`web/test.mjs`).
        tune: Tune { growth: Some(144_000), litter: Some((3, 8)), crowd: Some(20), ..Tune::NONE },
        birth_size: 0.15,
        ..BASE
    },
    // Gastonia (`reports/ecology/Cretaceous food webs.md`: "among the most common dinosaur fossils
    // in the Cedar Mountain Formation", found with the Utahraptor): a polacanthid ankylosaur,
    // armoured, so the Utahraptor takes its young (`source/ankylosaur`, its own colors).
    Species {
        model: "ankylosaur",
        name: "gastonia",
        binomial: "Gastonia burgei",
        family: &POLACANTHIDAE,
        subfamily: "",
        size: Size::Large,
        // Low plants: ferns, horsetails, cycads' fronds, a little fruit and seed.
        eats: &[(Leaves, 0.9), (Fruit, 0.05), (Seeds, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [120.0, 300.0, 400.0, 6.0, 0.8, 600.0, 200.0, 400.0, 60.0],
        hunger: 12.0,
        foods: &[eats("animals:fern", 10), favorite("animals:cycad_seed", 12)],
        shy: 8.0,
        reach: 1.5,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.03,
        size_spread: 0.1,
        height: (1.25, "hip"),
        // The Yellow Cat floodplain first; its bonebeds suggest groups (one formed in a drought or
        // a flood).
        habitats: &[("animals:floodplain", 1.0), ("animals:scrubland", 0.3), ("animals:fern_forest", 0.3)],
        bio: Biology {
            mass: 1900.0,
            gestation: 0.0,
            // No record; a big ornithischian's egg, longer than Protoceratops's 83 days (est.).
            incubation: 100.0,
            // No record of its growth lines; breeding at about six, before it is fully grown, as
            // dinosaurs did (est.), and living about 25 years.
            maturity: 2190.0,
            lifespan: 9125.0,
            interval: 365.0,
            // No record of a clutch (est., as Protoceratops's).
            litter: 10.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The floodplain's monsoon heat and its dry spells.
            comfort: (8.0, 38.0),
            cold_blooded: false,
        },
        // A big plant eater at a common species' pace, its young what the hunters take, a wild
        // clutch kept to what a mother raises, and a room that lets a small group last
        // (`web/test.mjs`).
        tune: Tune { growth: Some(144_000), litter: Some((1, 4)), crowd: Some(12), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    // Pinacosaurus (`reports/ecology/Cretaceous food webs.md`: the scrubland's big plant eater,
    // "dominant" at Bayan Mandahu, mostly juveniles in groups killed together by sandstorms:
    // Currie et al. 2011): an ankylosaurid with a tail club, on the same model in its own colors.
    Species {
        model: "ankylosaur",
        name: "pinacosaurus",
        binomial: "Pinacosaurus grangeri",
        family: &ANKYLOSAURIDAE,
        subfamily: "",
        size: Size::Large,
        // Low plants, perhaps pulpy fruit, roots and a few insects (speculative).
        eats: &[(Leaves, 0.8), (Fruit, 0.1), (Roots, 0.05), (Insects, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [120.0, 300.0, 400.0, 6.0, 0.8, 600.0, 200.0, 400.0, 60.0],
        hunger: 12.0,
        foods: &[eats("animals:fern", 10), favorite("base:berries", 12)],
        shy: 8.0,
        reach: 1.5,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.03,
        size_spread: 0.1,
        height: (1.25, "hip"),
        // The Gobi's scrub first, the dunes less often, and now and then the rocky crags' foot
        // (the only plant eater there big enough for the wyvern).
        habitats: &[("animals:scrubland", 1.0), ("animals:red_dunes", 0.5), ("animals:crags", 0.2)],
        bio: Biology {
            mass: 1900.0,
            gestation: 0.0,
            // No record; a big ornithischian's egg, longer than Protoceratops's 83 days (est.).
            incubation: 100.0,
            // No record of its growth lines; breeding at about six, before it is fully grown, as
            // dinosaurs did (est.), and living about 25 years.
            maturity: 2190.0,
            lifespan: 9125.0,
            interval: 365.0,
            // No record of a clutch (est., as Protoceratops's).
            litter: 10.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The Djadokhta's semi-arid seasons: hot days, cold nights.
            comfort: (0.0, 40.0),
            cold_blooded: false,
        },
        // A big plant eater at a common species' pace, its young what the hunters take, a wild
        // clutch kept to what a mother raises, and a room that lets a small group last
        // (`web/test.mjs`).
        tune: Tune { growth: Some(144_000), litter: Some((1, 4)), crowd: Some(12), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    // The iguanodont (`reports/ecology/Cretaceous food webs.md`: the floodplain's and the forest's big
    // plant eater, what the Utahraptor was caught hunting in the Stikes Quarry block): Iguanacolossus
    // fortis standing for the Yellow Cat's iguanodonts and the Yixian's Jinzhousaurus, wild for now
    // (a tame beast of burden later: tier 1), in herds (`source/iguanodont`).
    Species {
        model: "iguanodont",
        name: "iguanodont",
        binomial: "Iguanacolossus fortis",
        family: &STYRACOSTERNA,
        subfamily: "",
        size: Size::Large,
        // Browse up to about 4 m: ferns, horsetails, conifers and cycads; a few seeds and fruit.
        eats: &[(Leaves, 0.85), (Seeds, 0.1), (Fruit, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [150.0, 400.0, 600.0, 8.0, 1.0, 800.0, 250.0, 500.0, 50.0],
        hunger: 12.0,
        foods: &[eats("animals:fern", 10), favorite("animals:cycad_seed", 14)],
        shy: 10.0,
        reach: 2.0,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.03,
        size_spread: 0.1,
        height: (2.2, "hip"),
        // The Yellow Cat floodplain first, the fern forest (as Jinzhousaurus) less often.
        habitats: &[("animals:floodplain", 1.0), ("animals:fern_forest", 0.6)],
        bio: Biology {
            mass: 4000.0,
            gestation: 0.0,
            // No record; a big ornithopod's egg, longer than Protoceratops's 83 days (est.).
            incubation: 110.0,
            // Growth lines of its kin (Iguanodon, Tenontosaurus): grown at about ten, breeding
            // before that, at about seven (est.); living about 30 years.
            maturity: 2555.0,
            lifespan: 10950.0,
            interval: 365.0,
            // No record of a clutch (est.).
            litter: 12.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The floodplain's monsoon heat and dry spells, the forest's cooler seasons.
            comfort: (4.0, 38.0),
            cold_blooded: false,
        },
        // A big plant eater at a common species' pace, its young what the hunters take, a wild
        // clutch kept to what a mother raises, and a room that lets a small group last
        // (`web/test.mjs`).
        tune: Tune { growth: Some(144_000), litter: Some((1, 3)), crowd: Some(12), ..Tune::NONE },
        birth_size: 0.1,
        ..BASE
    },
    // The crocodyliform (`reports/ecology/Cretaceous food webs.md`: the water's hunter, a predator
    // for young Utahraptors as caimans are for capybaras): a neosuchian of the Yellow Cat's rivers
    // (its remains are fragments, no named species), on the caiman's model drawn about 1.6 times
    // as long by its genes (`source/caiman`, `caiman.rs`).
    Species {
        model: "caiman",
        name: "crocodyliform",
        binomial: "Neosuchia indet.",
        family: &NEOSUCHIA,
        subfamily: "",
        size: Size::Large,
        // Fish first, then what comes to the water to drink, and carrion.
        eats: &[(Fish, 0.6), (Meat, 0.3), (Carrion, 0.1)],
        // A fish or a bird at the water (as crocodiles take them) to a young dinosaur near its own mass.
        prey: (0.003, 1.0),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [45.0, 180.0, 160.0, 7.0, 1.0, 120.0, 400.0, 90.0, 40.0],
        hunger: 60.0,
        foods: &[eats("animals:trout", 10), favorite("animals:venison", 14)],
        shy: 6.0,
        fights: true,
        attacks: &["bite"],
        reach: 1.2,
        signature: "ambush",
        camouflage: 0.8,
        dimorphism: 1.2,
        // Its size genes draw it 1.6 times the caiman's model: about 3.5 m (`genetics`, Sz).
        size_spread: 0.67,
        // The model's length (the yacare's); its size genes draw it longer.
        height: (2.2, "length"),
        // The floodplain's rivers and oxbow ponds, the fern forest's lakes less often.
        habitats: &[("animals:floodplain", 1.0), ("animals:fern_forest", 0.4)],
        bio: Biology {
            // About 150 kg (est., the report's).
            mass: 150.0,
            gestation: 0.0,
            // As a living crocodile's, about 80 days in a mound (est.).
            incubation: 80.0,
            maturity: 3650.0,
            lifespan: 18250.0,
            interval: 365.0,
            litter: 25.0,
            social: Social::Solitary,
            rarity: Rarity::Uncommon,
            breeds: seasons::WET,
            peak: seasons::SUMMER,
            // Cold-blooded: at ease in the floodplain's heat, sluggish below 20 degrees.
            comfort: (20.0, 36.0),
            cold_blooded: true,
        },
        // As the caimans': clutches mostly lost to egg eaters, a long life, a long rest after a
        // clutch, and a room that lets a small group last (`web/test.mjs`).
        tune: Tune {
            litter: Some((4, 8)),
            growth: Some(150_000),
            lifespan: Some(1_800_000),
            recovery: Some(400_000),
            crowd: Some(20),
            ..Tune::NONE
        },
        birth_size: 0.15,
        ..BASE
    },
    // Citipati (`reports/ecology/Cretaceous food webs.md`: the red dunes' omnivore, "common at Ukhaa
    // Tolgod", and the oviraptorid found brooding its nests like a bird, Norell et al. 1995, 2018):
    // wild, on two legs, its arms feathered (`source/citipati`). It takes lizards and Kryptobaatar
    // as well as plants, and lies on its eggs with its wings spread.
    Species {
        model: "citipati",
        name: "citipati",
        binomial: "Citipati osmolskae",
        family: &OVIRAPTORIDAE,
        subfamily: "Citipatiinae",
        size: Size::Medium,
        // An omnivore or a plant eater (its beak; inference): leaves and seeds, fruit, small animals
        // and insects.
        eats: &[(Leaves, 0.4), (Seeds, 0.25), (Fruit, 0.1), (Meat, 0.15), (Insects, 0.1)],
        // A lizard (50 g) to a Kryptobaatar or a hatchling.
        prey: (0.0004, 0.05),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [40.0, 220.0, 120.0, 3.0, 1.2, 80.0, 90.0, 80.0, 20.0],
        hunger: 15.0,
        foods: &[eats("animals:cycad_seed", 8), favorite("base:berries", 10), eats("base:wheat_seeds", 3)],
        shy: 12.0,
        signature: "herd",
        reach: 1.0,
        camouflage: 0.9,
        dimorphism: 1.05,
        size_spread: 0.1,
        height: (0.9, "hip"),
        // The Djadokhta's dunes first, the scrub's flats often.
        habitats: &[("animals:red_dunes", 1.0), ("animals:scrubland", 0.7)],
        bio: Biology {
            mass: 90.0,
            gestation: 0.0,
            // No record; Protoceratops's 83 days (Erickson et al. 2017), for a bird-like egg it sat on.
            incubation: 70.0,
            // Breeding at about three, as Protoceratops (est.); living about 15 years.
            maturity: 1095.0,
            lifespan: 5475.0,
            interval: 365.0,
            // Nests of up to 22 eggs in a ring, brooded (Norell et al. 1995, 2018).
            litter: 18.0,
            social: Social::Solitary,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The Djadokhta's semi-arid seasons: hot days, cold nights; feathered.
            comfort: (2.0, 40.0),
            cold_blooded: false,
        },
        // A common species' pace, a clutch most of which the dunes' egg eaters take, and a room
        // that lets a small group last (`web/test.mjs`).
        tune: Tune { growth: Some(144_000), litter: Some((2, 6)), crowd: Some(18), ..Tune::NONE },
        birth_size: 0.12,
        ..BASE
    },
    // The early birds (`reports/ecology/Cretaceous food webs.md`: "flocks in the air; the crags' small
    // life"): Confuciusornis, the first bird with a toothless beak and "abundant" in the Jehol's
    // forests and lakes, many killed together in flocks by eruptions; and Gobipteryx of the Gobi,
    // an enantiornithine, on the same model (`source/early_bird`) in its own colors.
    Species {
        model: "early_bird",
        name: "confuciusornis",
        binomial: "Confuciusornis sanctus",
        family: &CONFUCIUSORNITHIDAE,
        subfamily: "",
        size: Size::Small,
        // Fish remains in one specimen; seeds or plants proposed; insects (inference).
        eats: &[(Fish, 0.4), (Seeds, 0.4), (Insects, 0.2)],
        // It caught its fish from the water (fish remains in one specimen); on the ground here it
        // takes none (its fish are what lies on the shore).
        prey: (0.0, 0.0),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [6.0, 120.0, 30.0, 1.0, 1.0, 6.0, 60.0, 15.0, 6.0],
        hunger: 30.0,
        foods: &[eats("animals:cycad_seed", 4), favorite("base:wheat_seeds", 3)],
        shy: 10.0,
        fights: true,
        reach: 0.5,
        signature: "flock",
        camouflage: 0.6,
        dimorphism: 1.05,
        size_spread: 0.15,
        // Standing, at the crown.
        height: (0.3, "crown"),
        // The Jehol's forests and lakes first, the floodplain less often.
        habitats: &[("animals:fern_forest", 1.0), ("animals:floodplain", 0.6)],
        bio: Biology {
            mass: 0.5,
            gestation: 0.0,
            // No record; a bird's of its size, about two weeks (est.).
            incubation: 14.0,
            // Early birds grew slowly, over a year or more (their bones' growth lines): breeding at
            // about one (est.); living about ten years.
            maturity: 365.0,
            lifespan: 3650.0,
            interval: 365.0,
            litter: 3.0,
            social: Social::Herd,
            rarity: Rarity::Common,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The Jehol's cool, seasonal forests.
            comfort: (0.0, 34.0),
            cold_blooded: false,
        },
        // No nests yet: fledglings after the eggs' time, as the other birds (`birds.rs`), grown in
        // a few weeks of the game's, and a room that lets a flock last (`web/test.mjs`).
        tune: Tune { gestation: Some(16_796), growth: Some(110_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    Species {
        model: "early_bird",
        name: "gobipteryx",
        binomial: "Gobipteryx minuta",
        family: &ENANTIORNITHIDAE,
        subfamily: "",
        size: Size::Small,
        // Insects and seeds (inference), and in the dry winter the green buds and shoots of the
        // conifers that stay green (est.).
        eats: &[(Insects, 0.5), (Seeds, 0.4), (Leaves, 0.1)],
        prey: (0.0, 0.0),
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [6.0, 120.0, 30.0, 1.0, 1.0, 6.0, 60.0, 15.0, 6.0],
        hunger: 30.0,
        foods: &[eats("animals:cycad_seed", 4), favorite("base:wheat_seeds", 3)],
        shy: 10.0,
        fights: false,
        reach: 0.5,
        signature: "flock",
        camouflage: 0.6,
        dimorphism: 1.05,
        size_spread: 0.15,
        // Standing, at the crown.
        height: (0.3, "crown"),
        // The Gobi's crags and scrub, the dunes less often (its eggs, with embryos, are known).
        habitats: &[("animals:crags", 1.0), ("animals:scrubland", 1.0), ("animals:red_dunes", 0.5)],
        bio: Biology {
            mass: 0.3,
            gestation: 0.0,
            // No record; a bird's of its size, about two weeks (est.).
            incubation: 14.0,
            // Early birds grew slowly, over a year or more (their bones' growth lines): breeding at
            // about one (est.); living about ten years.
            maturity: 365.0,
            lifespan: 3650.0,
            interval: 365.0,
            litter: 3.0,
            social: Social::Herd,
            rarity: Rarity::Uncommon,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The Djadokhta's semi-arid seasons: hot days, cold nights.
            comfort: (0.0, 40.0),
            cold_blooded: false,
        },
        // No nests yet: fledglings after the eggs' time, as the other birds (`birds.rs`), grown in
        // a few weeks of the game's, and a room that lets a flock last (`web/test.mjs`).
        tune: Tune { gestation: Some(16_796), growth: Some(150_000), crowd: Some(24), ..Tune::NONE },
        birth_size: 0.3,
        ..BASE
    },
    // The sauropod (`reports/ecology/Cretaceous food webs.md`: the floodplain's high browser, "a giant
    // carcass for the dead"): Cedarosaurus weiskopfae, a brachiosaurid of the Yellow Cat, standing for
    // the Yixian's Dongbeititan in the fern forest too; wild, in herds (`source/sauropod`).
    Species {
        model: "sauropod",
        name: "cedarosaurus",
        binomial: "Cedarosaurus weiskopfae",
        family: &BRACHIOSAURIDAE,
        subfamily: "",
        size: Size::Large,
        // High browse: conifers (Araucaria), horsetails, ferns; a few seeds and cones.
        eats: &[(Leaves, 0.95), (Seeds, 0.05)],
        //     health stamina food damage speed weight oxygen torpor fortitude
        base: [300.0, 600.0, 1200.0, 10.0, 0.8, 2000.0, 400.0, 1200.0, 80.0],
        hunger: 10.0,
        foods: &[eats("animals:fern", 10), favorite("animals:araucaria_cone", 16)],
        shy: 6.0,
        reach: 3.0,
        signature: "herd",
        camouflage: 0.9,
        dimorphism: 1.03,
        size_spread: 0.1,
        // At the shoulder (the back slopes down to the hips).
        height: (4.1, "shoulder"),
        // The floodplain first; the fern forest (as Dongbeititan) less often.
        habitats: &[("animals:floodplain", 1.0), ("animals:fern_forest", 0.5)],
        bio: Biology {
            // About 10 t (est.).
            mass: 10000.0,
            gestation: 0.0,
            // No record; sauropod eggs took perhaps four to six months (est., from embryos' teeth
            // in other dinosaurs).
            incubation: 120.0,
            // Sauropods grew fast: breeding at about ten (est.), living about 40 years.
            maturity: 3650.0,
            lifespan: 14600.0,
            interval: 365.0,
            // Clutches of 20-40 small eggs left in the ground, the young on their own (est.).
            litter: 25.0,
            social: Social::Herd,
            rarity: Rarity::Uncommon,
            breeds: seasons::WET,
            peak: seasons::SPRING,
            // The floodplain's monsoon heat and its dry spells.
            comfort: (6.0, 40.0),
            cold_blooded: false,
        },
        // A giant at a common species' pace, a clutch most of which the hunters and the floods take,
        // and a room that lets a small herd last (`web/test.mjs`).
        tune: Tune { growth: Some(160_000), litter: Some((1, 2)), crowd: Some(10), ..Tune::NONE },
        birth_size: 0.05,
        ..BASE
    },
];

/// Whether two animals' species are one (and so can breed together): breeds of one species are,
/// two species of one genus are not.
pub fn same_species(a: &Species, b: &Species) -> bool {
    a.binomial == b.binomial
}

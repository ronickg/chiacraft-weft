//! Food in the world for animals (`notes/ECOLOGY.md`, parts 3 and 5): the blocks a forager eats
//! from, what kind of food each is (`food::Category`), how much it feeds, and what eating it
//! leaves. Grazed grass grows back by the plant rule (`plants::PLANTS`, `base:grazed_grass`), a
//! berry bush starts its fruit again, leaves and cactus pads stay (a browser takes a little):
//! browse is there where its trees grow (`found`), the deciduous only from spring to autumn,
//! poorer food than fresh grass (`BROWSE`).

use crate::food::Category;
use crate::wild;

/// What eating it leaves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Eaten {
    /// Another block in its place.
    Becomes(&'static str),
    /// The same block at this `age`.
    Age(u8),
    /// Nothing changes.
    Stays,
}

#[derive(Clone, Copy, Debug)]
pub struct Source {
    /// The block's full name.
    pub block: &'static str,
    /// Only at this `age` (ripe); `None`: any.
    pub age: Option<u8>,
    pub category: Category,
    /// Points of food a mouthful gives (the animals' food stat: a capybara holds 200).
    pub food: u16,
    pub eaten: Eaten,
    /// The plant whose growth brings it back (`plants::PLANTS`); `None`: it's always there
    /// (browse) in `found` biomes, in `seasons`.
    pub renews: Option<&'static str>,
    /// Where browse grows (the biomes whose trees or plants bear it, `worldgen.rs`), or where
    /// the world puts a plant that grows back (a wild plant's, `wild::WILD`; empty: anywhere).
    pub found: &'static [&'static str],
    /// The seasons browse is there, as bits spring first (deciduous leaves: spring to autumn).
    pub seasons: u8,
}

const fn s(block: &'static str, age: Option<u8>, category: Category, food: u16, eaten: Eaten, renews: Option<&'static str>) -> Source {
    Source { block, age, category, food, eaten, renews, found: &[], seasons: 0 }
}

/// A wild plant (`wild::WILD`): eaten at `age`, growing back by its own plant where it's found.
const fn wild(block: &'static str, age: u8, category: Category, food: u16, eaten: Eaten, found: &'static [&'static str]) -> Source {
    Source { block, age: Some(age), category, food, eaten, renews: Some(block), found, seasons: 0 }
}

/// Browse: always there in `found`, in `seasons`.
const fn browse(block: &'static str, category: Category, food: u16, found: &'static [&'static str], seasons: u8) -> Source {
    Source { block, age: None, category, food, eaten: Eaten::Stays, renews: None, found, seasons }
}

/// How much of a diet's food browse is worth where it's found, against fresh growth at its best.
pub const BROWSE: f32 = 0.4;
const GREEN: u8 = 1 | 2 | 4;
const SUMMER: u8 = 2;
const SUMMER_AUTUMN: u8 = 2 | 4;
const SPRING_SUMMER: u8 = 1 | 2;
const AUTUMN: u8 = 4;
const AUTUMN_WINTER: u8 = 4 | 8;
const WINTER: u8 = 8;
const ALL_YEAR: u8 = 15;

/// Where the ground is turf (`worldgen.rs` surfaces: "grass"), in both worlds.
const TURF: &[&str] = &[
    "base:plains",
    "base:forest",
    "base:birch_forest",
    "base:savanna",
    "base:swamp",
    "base:mountains",
    "base:sequoia_grove",
    "animals:scrubland",
    "animals:floodplain",
    "animals:fern_forest",
    "animals:crags",
];
/// Where oaks grow (the plains' and swamp's few trees are oaks, the forest's mostly).
const OAKS: &[&str] = &["base:forest", "base:plains", "base:swamp"];
const MESQUITE: &[&str] = &["base:desert", "base:savanna"];
const ARAUCARIAS: &[&str] = &["animals:scrubland", "animals:floodplain", "animals:fern_forest", "animals:crags"];
const SCALE_CONIFERS: &[&str] = &["animals:red_dunes", "animals:scrubland", "animals:crags"];
const BIRCHES: &[&str] = &["base:forest", "base:birch_forest"];

pub const SOURCES: [Source; 76] = [
    s("base:tall_grass", None, Category::Grass, 10, Eaten::Becomes("base:grazed_grass"), Some("base:grazed_grass")),
    wild("base:berry_bush", 3, Category::Fruit, 8, Eaten::Age(1), wild::FORESTS),
    browse("base:oak_leaves", Category::Leaves, 4, OAKS, GREEN),
    browse("base:birch_leaves", Category::Leaves, 4, &["base:forest", "base:birch_forest"], GREEN),
    // Evergreen: a winter's browse (and the tundra's few trees).
    browse(
        "base:spruce_leaves",
        Category::Leaves,
        4,
        &["base:taiga", "base:snowy_plains", "base:mountains", "base:sequoia_grove"],
        ALL_YEAR,
    ),
    browse("base:acacia_leaves", Category::Leaves, 4, &["base:savanna"], ALL_YEAR),
    // Cactus pads, the desert's browse (jackrabbits live on them in the dry months).
    browse("base:cactus", Category::Leaves, 3, &["base:desert"], ALL_YEAR),
    // Seeds: tall grass seeding in summer and autumn (the plains' mice and birds live on them),
    // and the desert's seed bank under its shrubs all year (the kangaroo rat's).
    browse(
        "base:tall_grass",
        Category::Seeds,
        3,
        &[
            "base:plains",
            "base:forest",
            "base:birch_forest",
            "base:taiga",
            "base:savanna",
            "base:swamp",
            "base:mountains",
            "base:sequoia_grove",
            "base:beach",
        ],
        SUMMER_AUTUMN,
    ),
    browse("base:dead_bush", Category::Seeds, 2, &["base:desert"], ALL_YEAR),
    // Prickly pear fruit, July to September.
    browse("base:cactus", Category::Fruit, 3, &["base:desert"], SUMMER_AUTUMN),
    // The Cretaceous's ferns (the animals mod's), browse all year where they grow: the low
    // browse Protoceratops cropped.
    browse(
        "animals:fern",
        Category::Leaves,
        4,
        &["animals:red_dunes", "animals:scrubland", "animals:floodplain", "animals:fern_forest", "animals:crags"],
        ALL_YEAR,
    ),
    // Winter (`animals_core::web` found the temperate biomes bare then): the standing grass,
    // cured into hay (tall grass doesn't grow back in winter, but it stands, poor food: bison and
    // hares live on it), the deciduous trees' twigs and buds (deer and hares browse them), and the
    // acorns and beechnuts fallen under them through autumn and winter (mice and deer).
    browse("base:tall_grass", Category::Grass, 3, TURF, WINTER),
    browse("base:oak_leaves", Category::Leaves, 2, OAKS, WINTER),
    browse("base:birch_leaves", Category::Leaves, 2, BIRCHES, WINTER),
    browse("base:oak_leaves", Category::Seeds, 3, OAKS, AUTUMN_WINTER),
    // Lichen: the tundra's and the taiga's ground cover, and the mountains' rocks, all year (a
    // reindeer's winter food, dug from under the snow).
    browse("base:snowy_grass", Category::Lichen, 3, &["base:snowy_plains"], ALL_YEAR),
    browse("base:podzol", Category::Lichen, 3, &["base:taiga"], ALL_YEAR),
    browse("base:stone", Category::Lichen, 2, &["base:mountains"], ALL_YEAR),
    // Fungi: mushrooms on the taiga's floor and brackets on the trees, summer and autumn.
    browse("base:podzol", Category::Fungi, 2, &["base:taiga"], SUMMER_AUTUMN),
    browse("base:oak_log", Category::Fungi, 2, OAKS, SUMMER_AUTUMN),
    browse("base:birch_log", Category::Fungi, 2, BIRCHES, SUMMER_AUTUMN),
    // Roots, bulbs and tubers dug from the ground, all year (voles live on them under the snow).
    browse("base:grass", Category::Roots, 2, TURF, ALL_YEAR),
    browse("base:sand", Category::Roots, 1, &["base:desert", "base:beach"], ALL_YEAR),
    browse("animals:red_sand", Category::Roots, 1, &["animals:red_dunes"], ALL_YEAR),
    // Insects in the grass and leaf litter from spring to autumn, in the warm sand all year,
    // in the north's short summer.
    browse("base:grass", Category::Insects, 2, TURF, GREEN),
    browse("base:sand", Category::Insects, 2, &["base:desert", "base:beach"], ALL_YEAR),
    browse("animals:red_sand", Category::Insects, 2, &["animals:red_dunes"], ALL_YEAR),
    browse("base:podzol", Category::Insects, 2, &["base:taiga"], SUMMER),
    browse("base:snowy_grass", Category::Insects, 2, &["base:snowy_plains"], SUMMER),
    // Water plants: reeds at the water's edge (the sugar cane stands for them) and the
    // Cretaceous's horsetails on wet ground, all year.
    browse(
        "base:sugar_cane",
        Category::WaterPlants,
        4,
        &["base:plains", "base:forest", "base:birch_forest", "base:savanna", "base:swamp", "base:beach"],
        ALL_YEAR,
    ),
    browse(
        "animals:horsetail",
        Category::WaterPlants,
        4,
        &["animals:red_dunes", "animals:scrubland", "animals:floodplain", "animals:fern_forest", "animals:crags"],
        ALL_YEAR,
    ),
    // The wild plants (`wild::WILD`): the tundra's sedges and cottongrass are its grass, grazed
    // down and growing back; reindeer lichen the same, but over years; clover in the meadows.
    wild("base:sedge", 1, Category::Grass, 8, Eaten::Age(0), wild::SEDGE_LANDS),
    wild("base:cottongrass", 1, Category::Grass, 6, Eaten::Age(0), wild::TUNDRA),
    wild("base:reindeer_lichen", 1, Category::Lichen, 6, Eaten::Age(0), wild::NORTH),
    wild("base:clover", 1, Category::Leaves, 8, Eaten::Age(0), wild::MEADOWS),
    // Ripe bushes, as the berry bush: the fruit eaten, they flower again.
    wild("base:lingonberry_bush", 3, Category::Fruit, 6, Eaten::Age(1), wild::BOREAL),
    wild("base:cloudberry_bush", 3, Category::Fruit, 6, Eaten::Age(1), wild::TUNDRA),
    wild("base:crowberry_bush", 3, Category::Fruit, 6, Eaten::Age(1), wild::NORTH),
    wild("base:hazel_bush", 3, Category::Seeds, 8, Eaten::Age(1), wild::WOODS),
    wild("base:bramble_bush", 3, Category::Fruit, 8, Eaten::Age(1), wild::BRAMBLES),
    // Ripe fruit and seed heads: prickly pears, wild sunflowers, wild rice.
    wild("base:prickly_pear", 1, Category::Fruit, 6, Eaten::Age(0), wild::DESERT),
    wild("base:wild_sunflower", 1, Category::Seeds, 6, Eaten::Age(0), wild::PRAIRIE),
    wild("base:wild_rice", 1, Category::Seeds, 6, Eaten::Age(0), wild::WETLANDS),
    // Prickly pear pads all year (javelinas, tortoises and packrats live on them), the leaves
    // and stems of wild rice, crowberry and lingonberry, evergreen, through winter (geese and
    // ptarmigan take last year's under the snow), mushrooms in their season, and the water
    // plants: hyacinth and cattails.
    browse("base:prickly_pear", Category::Leaves, 4, wild::DESERT, ALL_YEAR),
    browse("base:wild_rice", Category::Grass, 4, wild::WETLANDS, GREEN),
    browse("base:crowberry_bush", Category::Fruit, 2, wild::NORTH, WINTER),
    browse("base:lingonberry_bush", Category::Fruit, 2, wild::BOREAL, WINTER),
    browse("base:mushroom_brown", Category::Fungi, 5, wild::FOREST_FLOOR, SUMMER_AUTUMN),
    browse("base:water_hyacinth", Category::WaterPlants, 6, wild::WETLANDS, ALL_YEAR),
    browse("base:cattail", Category::WaterPlants, 5, wild::MARSHES, GREEN),
    // The Cretaceous's (`cretaceous.rs`): cycads' tough fronds and their seeds after the rains,
    // the early flowering shrubs' leaves, and the ponds' water lilies.
    browse("animals:cycad", Category::Leaves, 3, wild::CYCAD_LANDS, ALL_YEAR),
    browse("animals:cycad", Category::Seeds, 3, wild::CYCAD_LANDS, SUMMER_AUTUMN),
    browse("animals:flowering_shrub", Category::Leaves, 4, wild::FLOWERING, GREEN),
    browse("animals:water_lily", Category::WaterPlants, 5, wild::PONDS, ALL_YEAR),
    // Kelp washed up on the beach, all year.
    browse("base:kelp_wrack", Category::WaterPlants, 4, &["base:beach"], ALL_YEAR),
    // Turtle grass on the lagoons' floor, all year (green turtles and manatees graze it).
    browse("base:turtle_grass", Category::WaterPlants, 5, wild::LAGOONS, ALL_YEAR),
    // Grubs and beetles in the rotten logs, all year.
    browse("animals:rotten_log", Category::Insects, 3, wild::DEADWOOD, ALL_YEAR),
    // The next batch (`wild::WILD`): beachgrass grazed and growing back, beach plums and
    // huckleberries ripe on their bushes, chia's seed heads in spring; dwarf birch's leaves in
    // summer and its buds and catkins in winter (the ptarmigan's winter food).
    wild("base:dune_grass", 1, Category::Grass, 4, Eaten::Age(0), &["base:beach"]),
    wild("base:beach_plum_bush", 3, Category::Fruit, 6, Eaten::Age(1), &["base:beach"]),
    wild("base:huckleberry_bush", 3, Category::Fruit, 8, Eaten::Age(1), &["base:mountains"]),
    wild("base:chia", 1, Category::Seeds, 6, Eaten::Age(0), wild::DESERT),
    browse("base:dwarf_birch", Category::Leaves, 4, &["base:snowy_plains"], GREEN),
    browse("base:dwarf_birch", Category::Leaves, 2, &["base:snowy_plains"], WINTER),
    // Termites in their mounds on the dry lands, all year (the lizards', the small mammals', the
    // birds'), the crown as the mound.
    browse("animals:termite_mound", Category::Insects, 3, wild::TERMITE_LANDS, ALL_YEAR),
    browse("animals:termite_mound_upper", Category::Insects, 3, wild::TERMITE_LANDS, ALL_YEAR),
    // Our trees (`worldgen.rs`, `cretaceous.rs`), as browse where they grow: mesquite's leaves
    // and, June to October, its pods (the desert's biggest single food); the acuri palm's nuts
    // in the Pantanal's dry season and the moriche's fruit from the llanos' late dry season
    // into the rains.
    browse("base:mesquite_leaves", Category::Leaves, 4, MESQUITE, GREEN),
    browse("base:mesquite_leaves", Category::Seeds, 4, MESQUITE, SUMMER_AUTUMN),
    browse("base:acuri_palm_leaves", Category::Seeds, 5, &["base:swamp"], AUTUMN_WINTER),
    browse("base:moriche_palm_leaves", Category::Fruit, 5, &["base:savanna", "base:swamp"], SPRING_SUMMER),
    // The Cretaceous's: araucaria, rich foliage and big cones late in the wet season; ginkgo
    // leaves and their fleshy seeds in autumn; swamp cypress, deciduous; the scale-leaved
    // conifers and Tempskya's fronds, poor all year.
    browse("animals:araucaria_leaves", Category::Leaves, 5, ARAUCARIAS, ALL_YEAR),
    browse("animals:araucaria_leaves", Category::Seeds, 4, ARAUCARIAS, AUTUMN),
    browse("animals:ginkgo_leaves", Category::Leaves, 4, &["animals:fern_forest"], GREEN),
    browse("animals:ginkgo_leaves", Category::Fruit, 4, &["animals:fern_forest"], AUTUMN),
    browse("animals:swamp_cypress_leaves", Category::Leaves, 3, &["animals:floodplain"], GREEN),
    browse("animals:scale_conifer_leaves", Category::Leaves, 2, SCALE_CONIFERS, ALL_YEAR),
    browse("animals:tempskya_leaves", Category::Leaves, 2, wild::DEADWOOD, ALL_YEAR),
];

/// Food on the ground (`notes/ECOLOGY.md`, parts 5 and 8): items animals eat that aren't a
/// player's food (`food::FOODS` has those, each with its kind and weight): grain and seeds, as
/// they fall from a field or a hand. By name, kind and grams.
pub const DROPPED: [(&str, Category, f32); 14] = [
    // A handful of seed, and an ear's grain.
    ("base:wheat_seeds", Category::Seeds, 10.0),
    ("base:wheat", Category::Seeds, 50.0),
    // A few acorns (bitter raw: tannins), a handful of wild rice's grain, a cycad's seed
    // (poisonous to people: cycasin).
    ("base:acorn", Category::Seeds, 20.0),
    ("base:rice", Category::Seeds, 30.0),
    ("animals:cycad_seed", Category::Seeds, 20.0),
    // Our trees' fruit as it falls: a handful of mesquite pods, an acuri palm's nut, a moriche
    // palm's fruit; the Cretaceous's araucaria cones and ginkgo seeds.
    ("base:mesquite_pod", Category::Seeds, 30.0),
    ("base:acuri_nut", Category::Seeds, 30.0),
    ("base:moriche_fruit", Category::Fruit, 50.0),
    // The tropical forest's: a handful of Brazil nuts (agoutis open the pods and bury the nuts),
    // a cacao pod's sweet pulp (monkeys and birds).
    ("base:brazil_nut", Category::Seeds, 30.0),
    ("base:cacao_pod", Category::Fruit, 40.0),
    // Andiroba's oily seeds (pacas and agoutis), matamatá's seeds from their woody capsules
    // (monkeys and parrots).
    ("base:andiroba_seed", Category::Seeds, 20.0),
    ("base:matamata_fruit", Category::Seeds, 20.0),
    ("animals:araucaria_cone", Category::Seeds, 50.0),
    ("animals:ginkgo_seed", Category::Fruit, 10.0),
];

/// What a dropped item `name` is to animals: its kind of food and grams (a food's portion,
/// `food::FOODS`, or `DROPPED`). Spoiled food is carrion (`spoiled`): only carrion eaters take it.
/// `None`: not food.
pub fn dropped(name: &str, spoiled: bool) -> Option<(Category, f32)> {
    let (c, grams) = crate::food::FOODS
        .iter()
        .find(|f| f.name == name)
        .map(|f| (f.category, f.grams))
        .or_else(|| DROPPED.iter().find(|d| d.0 == name).map(|d| (d.1, d.2)))?;
    Some((if spoiled { Category::Carrion } else { c }, grams))
}

/// Whether the world has food of this kind on the ground for animals now and then: a food item or
/// a dropped seed of the kind, or carrion (anything spoiled).
pub fn dropped_kind(c: Category) -> bool {
    c == Category::Carrion || crate::food::FOODS.iter().any(|f| f.category == c) || DROPPED.iter().any(|d| d.1 == c)
}

/// What a forager of that diet can eat of the block `block` at `age` (`None`: it has no age) in
/// `season` (0 spring; `None`: no seasons there). One block can feed several diets: tall grass
/// is grass to a grazer, and seeds to a mouse in summer and autumn. The first source that fits
/// is the one eaten: tall grass eaten in winter is the growing grass's row (it grows back in
/// spring); the winter hay's row only counts it in what a place feeds (`season_lushness`).
pub fn of(block: &str, age: Option<u8>, eats: impl Fn(Category) -> bool, season: Option<usize>) -> Option<&'static Source> {
    SOURCES
        .iter()
        .find(|s| s.block == block && (s.age.is_none() || s.age == age) && eats(s.category) && s.in_season(season))
}

/// Share of the ground tall grass covers where the world makes it (`worldgen.rs`,
/// `cretaceous.rs` read it): the grass that grazers eat and that grows back. A biome not listed
/// gets the world's default (`GRASS_COVER_ELSE`) wherever its ground takes plants; frozen ground
/// and sand take none (`grass_grows`).
pub const GRASS_COVER: [(&str, f32); 13] = [
    ("base:plains", 0.25),
    ("base:forest", 0.12),
    ("base:birch_forest", 0.12),
    ("base:taiga", 0.08),
    ("base:savanna", 0.35),
    ("base:swamp", 0.2),
    ("base:mountains", 0.05),
    // The groves' floor is duff and ferns; the grass is in the clearings and the meadows.
    ("base:sequoia_grove", 0.06),
    // The rainforest's canopy shades out grass: a little in its gaps.
    ("base:tropical_forest", 0.03),
    ("animals:red_dunes", 0.0),
    ("animals:scrubland", 0.1),
    ("animals:floodplain", 0.05),
    ("animals:fern_forest", 0.03),
];
pub const GRASS_COVER_ELSE: f32 = 0.05;
/// Where no tall grass grows at all: frozen ground, and sand (no plant soil).
const NO_GRASS: [&str; 4] = ["base:snowy_plains", "base:desert", "base:beach", "base:ocean"];

/// A biome's tall grass cover as the world sets it (`None`: the world's default).
pub fn grass_cover(biome: &str) -> Option<f32> {
    GRASS_COVER
        .iter()
        .find(|&&(b, _)| b == biome)
        .map(|&(_, c)| c)
}

/// The share of a biome's ground in tall grass, as the world makes it.
pub fn grass_grows(biome: &str) -> f32 {
    if NO_GRASS.contains(&biome) { 0.0 } else { grass_cover(biome).unwrap_or(GRASS_COVER_ELSE) }
}

/// The ground itself (turf, sand, the forest floor, the tundra, bare rock): roots, insects and
/// lichen are in it everywhere, so a forager eats from it only when no richer food is in sight
/// (`Source::fallback`), as real ones graze and browse first.
pub const GROUND: [&str; 6] = ["base:grass", "base:sand", "animals:red_sand", "base:podzol", "base:snowy_grass", "base:stone"];

impl Source {
    /// Whether a forager takes it only when nothing else of its diet is in sight (the ground's
    /// roots, insects and lichen).
    pub fn fallback(&self) -> bool {
        GROUND.contains(&self.block)
    }
    /// Whether it's there in `season` (food that grows back always is; browse in its seasons).
    pub fn in_season(&self, season: Option<usize>) -> bool {
        self.seasons == 0 || season.is_none_or(|s| self.seasons >> s & 1 != 0)
    }
}

/// Whether the world has food of this kind for foragers.
pub fn grows(c: Category) -> bool {
    SOURCES.iter().any(|s| s.category == c)
}

/// How much of a diet's food a place grows now, 0 to 1 (`notes/ECOLOGY.md`, part 7): the best
/// rate (`plants::rate`) among the plants that bring back the foods it eats (those the world puts
/// in `biome`), in that day's mean air and soil water. `None`: it eats nothing that grows (a hunter, until it hunts).
pub fn lushness(eats: impl Fn(Category) -> bool, biome: &str, air: f32, soil: f32) -> Option<f32> {
    SOURCES
        .iter()
        .filter(|s| eats(s.category) && (s.found.is_empty() || s.found.contains(&biome)))
        .filter_map(|s| s.renews.and_then(crate::plants::of))
        .map(|p| crate::plants::rate(p, air, soil))
        .reduce(f32::max)
}

/// How much of a diet's food biome `biome` grows through a season, 0 to 1: `lushness` on average
/// over the season (`plants::season_rate_in`) in a biome of that mean, seasons and rain, or the
/// browse found there then (`BROWSE`), whichever is more; fruit and seed only in the seasons they
/// ripen (`plants::Plant::ripens`); without seasons (`None`), the year's.
/// What a place feeds, for how many come and how fed (part 7). `None`: it eats nothing that
/// grows.
pub fn season_lushness(
    eats: impl Fn(Category) -> bool,
    biome: &str,
    celsius: f32,
    seasons: f32,
    rain: [f32; 4],
    season: Option<usize>,
) -> Option<f32> {
    let mine: Vec<&Source> = SOURCES.iter().filter(|s| eats(s.category)).collect();
    let at = |season: usize| {
        mine.iter()
            .map(|s| match s.renews.and_then(crate::plants::of) {
                // A fruit or seed feeds only in the seasons it ripens.
                Some(p) if (s.found.is_empty() || s.found.contains(&biome)) && p.ripens >> season & 1 != 0 => {
                    crate::plants::season_rate_in(p, celsius, seasons, rain, season)
                }
                Some(_) => 0.0,
                None if s.found.contains(&biome) && s.seasons >> season & 1 != 0 => BROWSE,
                None => 0.0,
            })
            .fold(0.0f32, f32::max)
    };
    (!mine.is_empty()).then(|| season.map_or_else(|| (0..4).map(at).sum::<f32>() / 4.0, at))
}

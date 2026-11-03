//! Food in the world for animals (`notes/ECOLOGY.md`, parts 3 and 5): the blocks a forager eats
//! from, what kind of food each is (`food::Category`), how much it feeds, and what eating it
//! leaves. Grazed grass grows back by the plant rule (`plants::PLANTS`, `base:grazed_grass`), a
//! berry bush starts its fruit again, leaves and cactus pads stay (a browser takes a little):
//! browse is there where its trees grow (`found`), the deciduous only from spring to autumn,
//! poorer food than fresh grass (`BROWSE`).

use crate::food::Category;

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
    /// Where browse grows (the biomes whose trees or plants bear it, `worldgen.rs`).
    pub found: &'static [&'static str],
    /// The seasons browse is there, as bits spring first (deciduous leaves: spring to autumn).
    pub seasons: u8,
}

const fn s(block: &'static str, age: Option<u8>, category: Category, food: u16, eaten: Eaten, renews: Option<&'static str>) -> Source {
    Source { block, age, category, food, eaten, renews, found: &[], seasons: 0 }
}

/// Browse: always there in `found`, in `seasons`.
const fn browse(block: &'static str, category: Category, food: u16, found: &'static [&'static str], seasons: u8) -> Source {
    Source { block, age: None, category, food, eaten: Eaten::Stays, renews: None, found, seasons }
}

/// How much of a diet's food browse is worth where it's found, against fresh growth at its best.
pub const BROWSE: f32 = 0.4;
const GREEN: u8 = 1 | 2 | 4;
const ALL_YEAR: u8 = 15;

pub const SOURCES: [Source; 7] = [
    s("base:tall_grass", None, Category::Grass, 10, Eaten::Becomes("base:grazed_grass"), Some("base:grazed_grass")),
    s("base:berry_bush", Some(3), Category::Fruit, 8, Eaten::Age(1), Some("base:berry_bush")),
    browse("base:oak_leaves", Category::Leaves, 4, &["base:forest"], GREEN),
    browse("base:birch_leaves", Category::Leaves, 4, &["base:forest", "base:birch_forest"], GREEN),
    // Evergreen: a winter's browse (and the tundra's few trees).
    browse("base:spruce_leaves", Category::Leaves, 4, &["base:taiga", "base:snowy_plains", "base:mountains"], ALL_YEAR),
    browse("base:acacia_leaves", Category::Leaves, 4, &["base:savanna"], ALL_YEAR),
    // Cactus pads, the desert's browse (jackrabbits live on them in the dry months).
    browse("base:cactus", Category::Leaves, 3, &["base:desert"], ALL_YEAR),
];

/// What a forager can eat of the block `block` at `age` (`None`: it has no age).
pub fn of(block: &str, age: Option<u8>) -> Option<&'static Source> {
    SOURCES
        .iter()
        .find(|s| s.block == block && (s.age.is_none() || s.age == age))
}

/// Whether the world has food of this kind for foragers.
pub fn grows(c: Category) -> bool {
    SOURCES.iter().any(|s| s.category == c)
}

/// How much of a diet's food a place grows now, 0 to 1 (`notes/ECOLOGY.md`, part 7): the best
/// rate (`plants::rate`) among the plants that bring back the foods it eats, in that day's mean
/// air and soil water. `None`: it eats nothing that grows (a hunter, until it hunts).
pub fn lushness(eats: impl Fn(Category) -> bool, air: f32, soil: f32) -> Option<f32> {
    SOURCES
        .iter()
        .filter(|s| eats(s.category))
        .filter_map(|s| s.renews.and_then(crate::plants::of))
        .map(|p| crate::plants::rate(p, air, soil))
        .reduce(f32::max)
}

/// How much of a diet's food biome `biome` grows through a season, 0 to 1: `lushness` on average
/// over the season (`plants::season_rate_in`) in a biome of that mean, seasons and rain, or the
/// browse found there then (`BROWSE`), whichever is more; without seasons (`None`), the year's.
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
                Some(p) => crate::plants::season_rate_in(p, celsius, seasons, rain, season),
                None if s.found.contains(&biome) && s.seasons >> season & 1 != 0 => BROWSE,
                None => 0.0,
            })
            .fold(0.0f32, f32::max)
    };
    (!mine.is_empty()).then(|| season.map_or_else(|| (0..4).map(at).sum::<f32>() / 4.0, at))
}

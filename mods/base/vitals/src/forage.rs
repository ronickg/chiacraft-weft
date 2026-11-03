//! Food in the world for animals (`notes/ECOLOGY.md`, parts 3 and 5): the blocks a forager eats
//! from, what kind of food each is (`food::Category`), how much it feeds, and what eating it
//! leaves. Grazed grass grows back by the plant rule (`plants::PLANTS`, `base:grazed_grass`), a
//! berry bush starts its fruit again, leaves stay (a browser takes a little from a tree).

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
    /// (leaves).
    pub renews: Option<&'static str>,
}

const fn s(block: &'static str, age: Option<u8>, category: Category, food: u16, eaten: Eaten, renews: Option<&'static str>) -> Source {
    Source { block, age, category, food, eaten, renews }
}

pub const SOURCES: [Source; 6] = [
    s("base:tall_grass", None, Category::Grass, 10, Eaten::Becomes("base:grazed_grass"), Some("base:grazed_grass")),
    s("base:berry_bush", Some(3), Category::Fruit, 8, Eaten::Age(1), Some("base:berry_bush")),
    s("base:oak_leaves", None, Category::Leaves, 4, Eaten::Stays, None),
    s("base:birch_leaves", None, Category::Leaves, 4, Eaten::Stays, None),
    s("base:spruce_leaves", None, Category::Leaves, 4, Eaten::Stays, None),
    s("base:acacia_leaves", None, Category::Leaves, 4, Eaten::Stays, None),
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

/// How much of a diet's food a biome grows through a season, 0 to 1: `lushness` on average over
/// the season (`plants::season_rate_in`), in a biome of that mean, seasons and rain; without
/// seasons (`None`), the year's. What a place feeds, for how many come and how fed (part 7).
pub fn season_lushness(eats: impl Fn(Category) -> bool, celsius: f32, seasons: f32, rain: [f32; 4], season: Option<usize>) -> Option<f32> {
    let plants: Vec<&crate::plants::Plant> = SOURCES
        .iter()
        .filter(|s| eats(s.category))
        .filter_map(|s| s.renews.and_then(crate::plants::of))
        .collect();
    let at = |season: usize| {
        plants
            .iter()
            .map(|p| crate::plants::season_rate_in(p, celsius, seasons, rain, season))
            .fold(0.0f32, f32::max)
    };
    (!plants.is_empty()).then(|| season.map_or_else(|| (0..4).map(at).sum::<f32>() / 4.0, at))
}

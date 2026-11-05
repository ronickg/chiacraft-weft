//! The crops (`notes/ECOLOGY.md`, part 3; the agronomy in `reports/ecology/Plants and crops.md`
//! and `plants.json`): what each is sown from and gives ripe, how it leaves the soil, and where
//! its wild seed turns up. How it grows (its warmth, water, days and frost) is its row in
//! `plants::PLANTS`, the same rule as every plant's: so barley and rye grow in the taiga's short
//! summer, maize, millet and sorghum in the heat, rice only by water, and winter rye waits through
//! the winter to set its grain in summer.
//!
//! A crop picked before it is ripe gives back what it was sown from and nothing more; ripe, it
//! gives its harvest (`count`) and, when it has a seed of its own, up to `EXTRA_SEEDS` more seed.

/// What a ripe harvest does to the farmland under it (`soil`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Soil {
    /// It takes a level: the grains and roots take 40-200 kg of nitrogen a hectare.
    Tires,
    /// It gives a level back: beans fix 20-60 kg of nitrogen a hectare, a little.
    Feeds,
}

/// A crop, by its block.
#[derive(Clone, Copy, Debug)]
pub struct Crop {
    /// The block's full name; its `plants::PLANTS` row has the same.
    pub block: &'static str,
    /// The item that sows it.
    pub seed: &'static str,
    /// What it gives ripe, and how many (more, when it's the seed too).
    pub gives: &'static str,
    pub count: (u32, u32),
    /// Its last age: ripe.
    pub ripe: u32,
    pub soil: Soil,
    /// The biomes whose tall grass gives its seed now and then (`WILD_SEED_CHANCE`): where the
    /// research grows it without watering (so not maize in the desert, nor cassava on the beach).
    /// Wheat's seed comes from tall grass everywhere instead.
    pub wild: &'static [&'static str],
    /// A stem's fruit (pumpkin, melon): ripe, it sets this block on soil beside it in its season,
    /// again and again, instead of being harvested itself.
    pub fruit: Option<&'static str>,
    /// The first age it stands two blocks tall (maize, sorghum, millet: Ronald, 11-04, "Yes, two
    /// blocks" for their last two stages): from then on its upper half is a block of its own,
    /// `<block>_top`, that breaks with it.
    pub tall: Option<u32>,
}

/// The chance broken tall grass gives a seed of one of its biome's crops. Rare, so a seed is a
/// find and cutting grass for fibre doesn't fill the hotbar with seed kinds (the first-hours
/// playtest after the sync, round 13: 14 cuts gave six kinds of seed).
pub const WILD_SEED_CHANCE: f32 = 1.0 / 32.0;
/// The chance broken tall grass gives a wheat seed, anywhere (it was an eighth, vanilla's).
pub const WHEAT_SEED_CHANCE: f32 = 1.0 / 32.0;

/// Seed a ripe harvest gives beyond the one back, at most, for a crop with a seed of its own.
pub const EXTRA_SEEDS: u32 = 3;

const COOL: &[&str] = &["base:plains", "base:forest", "base:birch_forest", "base:taiga", "base:mountains"];

/// The crops. The grains are sown from their seed and give their grain; carrots, potatoes,
/// beetroot and cassava from the root itself (cassava really from stem cuttings, beetroot from
/// seed: a seed of its own waits for an icon); paddy rice's grain is its seed.
pub const CROPS: [Crop; 15] = [
    c("base:wheat", "base:wheat_seeds", "base:wheat", (1, 1), 7, Soil::Tires, &[]),
    c(
        "base:barley",
        "base:barley_seeds",
        "base:barley",
        (1, 1),
        3,
        Soil::Tires,
        &["base:taiga", "base:mountains", "base:plains", "base:birch_forest"],
    ),
    c(
        "base:rye",
        "base:rye_seeds",
        "base:rye",
        (1, 1),
        3,
        Soil::Tires,
        &["base:taiga", "base:birch_forest", "base:plains", "base:mountains"],
    ),
    c("base:carrots", "base:carrot", "base:carrot", (1, 3), 3, Soil::Tires, COOL),
    c("base:potatoes", "base:potato", "base:potato", (1, 3), 3, Soil::Tires, COOL),
    c("base:beetroots", "base:beetroot", "base:beetroot", (1, 2), 3, Soil::Tires, COOL),
    c("base:maize", "base:maize_seeds", "base:corn", (1, 2), 3, Soil::Tires, &["base:plains", "base:forest", "base:savanna"]).tall(2),
    c("base:paddy_rice", "base:rice_seeds", "base:rice_seeds", (1, 3), 3, Soil::Tires, &["base:swamp", "base:savanna"]),
    c(
        "base:beans",
        "base:bean_seeds",
        "base:beans",
        (1, 2),
        3,
        Soil::Feeds,
        &["base:plains", "base:forest", "base:savanna", "base:mountains"],
    ),
    // Tepary beans: the Sonoran desert's own bean, sown dry into the July monsoon and picked in
    // September and October; a dry bean sows itself.
    c("base:tepary_beans", "base:tepary_beans", "base:tepary_beans", (1, 3), 3, Soil::Feeds, &["base:desert"]),
    c("base:sorghum", "base:sorghum_seeds", "base:sorghum", (1, 1), 3, Soil::Tires, &["base:savanna", "base:plains", "base:desert"])
        .tall(2),
    c("base:millet", "base:millet_seeds", "base:millet", (1, 1), 3, Soil::Tires, &["base:savanna", "base:desert"]).tall(2),
    c("base:cassava", "base:cassava", "base:cassava", (1, 3), 3, Soil::Tires, &["base:savanna", "base:swamp"]),
    // Stems that set their fruit beside them (ECOLOGY.md, the research's pumpkin and melon):
    // pumpkins in the prairie's, the forest's and the llanos' fields, melons where it's hot.
    stem("base:pumpkin_stem", "base:pumpkin_seeds", "base:pumpkin", &["base:plains", "base:forest", "base:savanna"]),
    stem("base:melon_stem", "base:melon_seeds", "base:melon", &["base:savanna", "base:plains"]),
];

const fn c(
    block: &'static str,
    seed: &'static str,
    gives: &'static str,
    count: (u32, u32),
    ripe: u32,
    soil: Soil,
    wild: &'static [&'static str],
) -> Crop {
    Crop { block, seed, gives, count, ripe, soil, wild, fruit: None, tall: None }
}

/// A stem: sown from its seed, picked it gives the seed back, ripe it sets `fruit` beside it.
const fn stem(block: &'static str, seed: &'static str, fruit: &'static str, wild: &'static [&'static str]) -> Crop {
    Crop { block, seed, gives: seed, count: (1, 1), ripe: 3, soil: Soil::Tires, wild, fruit: Some(fruit), tall: None }
}

impl Crop {
    /// Two blocks tall from age `from` on (`Crop::tall`).
    const fn tall(self, from: u32) -> Crop {
        Crop { tall: Some(from), ..self }
    }

    /// Two blocks tall at `age`.
    pub fn tall_at(&self, age: u32) -> bool {
        self.tall.is_some_and(|t| age >= t)
    }
}

/// The crop that grows as this block.
pub fn of(block: &str) -> Option<&'static Crop> {
    CROPS.iter().find(|c| c.block == block)
}

/// The crop this item sows.
pub fn sown_by(item: &str) -> Option<&'static Crop> {
    CROPS.iter().find(|c| c.seed == item)
}

/// The crops whose seed turns up in this biome's tall grass.
pub fn wild_in(biome: &str) -> impl Iterator<Item = &'static Crop> + '_ {
    CROPS.iter().filter(move |c| c.wild.contains(&biome))
}

/// The grains bread is baked from: wheat, and as rye, barley, sorghum and millet breads are.
pub const BREAD_GRAINS: [&str; 5] = ["base:wheat", "base:rye", "base:barley", "base:sorghum", "base:millet"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{climate, plants};

    #[test]
    fn every_crop_grows_and_is_found() {
        for c in &CROPS {
            let p = plants::of(c.block).unwrap_or_else(|| panic!("{} has no plants row", c.block));
            assert!(p.days > 0.0, "{} grows on the calendar", c.block);
            for b in c.wild {
                let place = climate::of(b).unwrap_or_else(|| panic!("{} finds its seed in {b}, no such biome", c.block));
                assert!((0..4).any(|s| plants::season_rate(p, place, s) >= 0.2), "{} is found in {b}, where it never grows", c.block);
            }
        }
    }
}

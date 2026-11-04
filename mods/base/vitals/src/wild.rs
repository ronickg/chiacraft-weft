//! The wild plants (`notes/ECOLOGY.md`, part 3, "Gaps that matter for food chains"; the research
//! in `reports/ecology/Plants and crops.md` and `plants.json`): the food the webs need that
//! nothing in the world gave, each where its real place grows it. The tundra's sedges,
//! cottongrass, lichen and low berries; the desert's prickly pear; the swamp's and the llanos'
//! water hyacinth, wild rice and cattails; the forest's mushrooms, hazel and brambles; clover and
//! wild sunflowers in the grass. Their growth is in `plants::PLANTS`, what animals eat of them in
//! `forage::SOURCES`, their fruit and nuts in `food::FOODS`. The base mod and the animals mod
//! register and place the rows of their own names (`mods/base/src/wild.rs`, `cretaceous.rs`).

/// Where it stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ground {
    /// Plant soil (turf, dirt, podzol, the tundra's snowy turf, red sand).
    Soil,
    /// Sand.
    Sand,
    /// Soil at the water's edge, at the sea's level beside water.
    Shore,
    /// Floating on still, shallow water.
    Water,
}

/// How it grows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// A tuft or a mat, grazed down (`age` 0) and growing back (`age` 1).
    Tuft,
    /// A bush whose fruit ripens through four stages (`age` 0 to 3), as the berry bush.
    Bush,
    /// A plant whose fruit or seeds ripen in their season (`age` 1); picked or eaten, they start
    /// again (`age` 0).
    Ripens,
    /// New ones come up near the old ones in their season, up to `CROWD` within `REACH`.
    Spreads,
    /// It stands as it is.
    Still,
    /// A fallen log lying on the ground, a full block, rotting (grubs live in it).
    Log,
    /// A flat mat lying on the ground (kelp washed up at the tide line).
    Mat,
}

#[derive(Clone, Copy, Debug)]
pub struct Wild {
    /// The block's full name.
    pub block: &'static str,
    pub form: Form,
    pub ground: Ground,
    /// What picking it gives (its ripe fruit, its seeds, itself), which plants it again.
    pub gives: Option<&'static str>,
    /// The biomes it grows in.
    pub found: &'static [&'static str],
    /// The share of the ground of its kind it covers there.
    pub cover: f32,
}

/// Bushes whose fruit holds on the plant over winter into spring (lingonberries under the snow,
/// crowberries for geese and ptarmigan in spring): the world makes them ripe.
pub const HOLDS_FRUIT: [&str; 2] = ["base:lingonberry_bush", "base:crowberry_bush"];

/// Where still ponds lie, and in what share of the ground's 8 x 8 cells: the Pantanal's and the
/// Cedar Mountain floodplain's standing water, a block deep, for their floating plants and the
/// wild rice and cattails on their shores (the world's other water is the sea's, with beaches).
pub const POND_LANDS: [(&str, f32); 2] = [("base:swamp", 0.35), ("animals:floodplain", 0.2)];

/// The most of one ground all its wild plants cover in a biome (the world rolls for them first).
pub const MOST: f32 = 0.3;

/// Blocks within which a spreading plant puts a new one, and how many it allows there.
pub const REACH: i32 = 4;
pub const CROWD: usize = 5;

pub const TUNDRA: &[&str] = &["base:snowy_plains", "base:taiga"];
pub const NORTH: &[&str] = &["base:snowy_plains", "base:taiga", "base:mountains"];
pub const BOREAL: &[&str] = &["base:taiga", "base:birch_forest", "base:mountains"];
pub const SEDGE_LANDS: &[&str] = &["base:snowy_plains", "base:taiga", "base:swamp"];
pub const MEADOWS: &[&str] = &["base:plains", "base:forest", "base:birch_forest", "base:mountains"];
pub const WOODS: &[&str] = &["base:forest", "base:birch_forest", "base:plains"];
pub const BRAMBLES: &[&str] = &["base:forest", "base:birch_forest", "base:plains", "base:taiga", "base:mountains"];
pub const FOREST_FLOOR: &[&str] = &["base:forest", "base:birch_forest", "base:taiga", "base:mountains"];
pub const AGARICS: &[&str] = &["base:forest", "base:birch_forest", "base:taiga"];
/// Where the berry bush (blueberries) grows: the forests' floor.
pub const FORESTS: &[&str] = &["base:forest", "base:birch_forest", "base:taiga"];
pub const WETLANDS: &[&str] = &["base:swamp", "base:savanna"];
pub const MARSHES: &[&str] = &["base:swamp", "base:plains", "base:forest", "base:taiga", "base:savanna"];
pub const DESERT: &[&str] = &["base:desert"];
pub const PRAIRIE: &[&str] = &["base:plains"];
/// The Cretaceous (the animals mod's): its cycads and bennettitaleans all over, its early
/// flowering shrubs on the floodplain and in the scrub, water lilies' kin in its ponds.
pub const CYCAD_LANDS: &[&str] = &["animals:red_dunes", "animals:scrubland", "animals:floodplain", "animals:fern_forest", "animals:crags"];
pub const FLOWERING: &[&str] = &["animals:floodplain", "animals:scrubland"];
pub const PONDS: &[&str] = &["animals:floodplain", "animals:red_dunes"];
/// Where the Cretaceous's trees fall and rot: its forests and the wooded floodplain.
pub const DEADWOOD: &[&str] = &["animals:floodplain", "animals:fern_forest"];

const fn w(
    block: &'static str,
    form: Form,
    ground: Ground,
    gives: Option<&'static str>,
    found: &'static [&'static str],
    cover: f32,
) -> Wild {
    Wild { block, form, ground, gives, found, cover }
}

/// Covers are shares of the cells of that ground (tall grass covers 5-35% of the turf, berry bushes
/// 0.4% of the forest's; `forage::GRASS_COVER`): the tundra's sedges and cottongrass are its grass,
/// the bushes and mushrooms are found, not walked through.
pub const WILD: [Wild; 22] = [
    w("base:sedge", Form::Tuft, Ground::Soil, None, SEDGE_LANDS, 0.12),
    w("base:cottongrass", Form::Tuft, Ground::Soil, None, TUNDRA, 0.08),
    w("base:reindeer_lichen", Form::Tuft, Ground::Soil, None, NORTH, 0.05),
    w("base:clover", Form::Tuft, Ground::Soil, None, MEADOWS, 0.03),
    w("base:berry_bush", Form::Bush, Ground::Soil, Some("base:berries"), FORESTS, 0.004),
    w("base:lingonberry_bush", Form::Bush, Ground::Soil, Some("base:lingonberries"), BOREAL, 0.01),
    w("base:cloudberry_bush", Form::Bush, Ground::Soil, Some("base:cloudberries"), TUNDRA, 0.006),
    w("base:crowberry_bush", Form::Bush, Ground::Soil, Some("base:crowberries"), NORTH, 0.012),
    w("base:hazel_bush", Form::Bush, Ground::Soil, Some("base:hazelnut"), WOODS, 0.006),
    w("base:bramble_bush", Form::Bush, Ground::Soil, Some("base:raspberries"), BRAMBLES, 0.006),
    w("base:prickly_pear", Form::Ripens, Ground::Sand, Some("base:prickly_pear_fruit"), DESERT, 0.01),
    w("base:wild_sunflower", Form::Ripens, Ground::Soil, Some("base:sunflower_seeds"), PRAIRIE, 0.01),
    w("base:wild_rice", Form::Ripens, Ground::Shore, Some("base:rice"), WETLANDS, 0.1),
    w("base:mushroom_brown", Form::Spreads, Ground::Soil, Some("base:mushroom_brown"), FOREST_FLOOR, 0.006),
    w("base:mushroom_red", Form::Spreads, Ground::Soil, Some("base:mushroom_red"), AGARICS, 0.003),
    w("base:water_hyacinth", Form::Spreads, Ground::Water, None, WETLANDS, 0.08),
    w("base:cattail", Form::Still, Ground::Shore, None, MARSHES, 0.1),
    w("animals:cycad", Form::Still, Ground::Soil, Some("animals:cycad_seed"), CYCAD_LANDS, 0.01),
    w("animals:flowering_shrub", Form::Still, Ground::Soil, None, FLOWERING, 0.02),
    w("animals:water_lily", Form::Still, Ground::Water, None, PONDS, 0.06),
    w("animals:rotten_log", Form::Log, Ground::Soil, None, DEADWOOD, 0.004),
    // Kelp washed up on the beach (wrack, the beach's food: sand fleas and flies on it feed the
    // shorebirds). Kelp growing under the sea waits for a block that stands in water.
    w("base:kelp", Form::Mat, Ground::Sand, Some("base:kelp"), &["base:beach"], 0.03),
];

/// A wild plant by its block's full name.
pub fn of(block: &str) -> Option<&'static Wild> {
    WILD.iter().find(|p| p.block == block)
}

/// Which wild plant (its index in `WILD`) grows on a cell of `ground` in `biome`, for a roll in
/// 0..1: the plants of that ground found there, each taking its cover of the rolls in turn.
pub fn pick(biome: &str, ground: Ground, roll: f32) -> Option<usize> {
    let mut at = 0.0;
    for (i, p) in WILD.iter().enumerate() {
        if p.ground != ground || !p.found.contains(&biome) {
            continue;
        }
        at += p.cover;
        if roll < at {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_wild_plant_has_its_numbers() {
        for p in &WILD {
            if !matches!(p.form, Form::Still | Form::Log | Form::Mat) {
                assert!(crate::plants::of(p.block).is_some(), "{}: no row in plants::PLANTS", p.block);
            }
            if let Some(item) = p.gives.filter(|g| *g != p.block) {
                let food = crate::food::find(item).is_some() || crate::forage::DROPPED.iter().any(|d| d.0 == item);
                assert!(food, "{}: gives {item}, which is no food (food::FOODS, forage::DROPPED)", p.block);
            }
        }
        // Each grows where it's found: a third of its best or more there in some season.
        for p in &WILD {
            let Some(plant) = crate::plants::of(p.block) else { continue };
            for b in p.found {
                let Some(c) = crate::climate::of(b) else { continue };
                let best = (0..4)
                    .map(|s| crate::plants::season_rate(plant, c, s))
                    .fold(0.0f32, f32::max);
                assert!(best >= 0.33, "{}: grows at most {:.0}% of its best in {b}", p.block, best * 100.0);
            }
        }
        // The covers of one ground in one biome leave room for the rest.
        for p in &WILD {
            for b in p.found {
                let all: f32 = WILD
                    .iter()
                    .filter(|q| q.ground == p.ground && q.found.contains(b))
                    .map(|q| q.cover)
                    .sum();
                assert!(all < MOST, "{b}: wild plants cover {:.0}% of its {:?} ground", all * 100.0, p.ground);
            }
        }
    }
}

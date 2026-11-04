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
    /// Rooted on the bottom under still water, standing in it (`depths` says how deep).
    Bed,
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
    /// A stalk standing in water, growing up a block at a time until one below the surface
    /// (kelp); the world makes it a random height of that.
    Column,
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

/// A wild flower (Research round 7, `reports/world-looks.md`, "Flowers per biome"): each biome's
/// flowers from the real place its climate comes from, in patches of one kind (`worldgen.rs`).
#[derive(Clone, Copy, Debug)]
pub struct Flower {
    /// The block's full name.
    pub block: &'static str,
    pub found: &'static [&'static str],
    pub ground: Ground,
    /// The seasons it blooms in, as bits spring first (for a bloom look to come).
    pub blooms: u8,
}

const SP: u8 = 1;
const SU: u8 = 2;
const AU: u8 = 4;
const WI: u8 = 8;
const fn f(block: &'static str, found: &'static [&'static str], ground: Ground, blooms: u8) -> Flower {
    Flower { block, found, ground, blooms }
}
const PLAINS: &[&str] = &["base:plains"];
const FOREST: &[&str] = &["base:forest"];
const BIRCH: &[&str] = &["base:birch_forest"];
const TAIGA: &[&str] = &["base:taiga"];
const SNOWY: &[&str] = &["base:snowy_plains"];
const SAVANNA: &[&str] = &["base:savanna"];
const SWAMP: &[&str] = &["base:swamp"];
const MOUNTAINS: &[&str] = &["base:mountains"];
const BEACH: &[&str] = &["base:beach"];

pub const FLOWERS: [Flower; 45] = [
    // The Konza Prairie's tallgrass.
    f("base:purple_coneflower", PLAINS, Ground::Soil, SU),
    f("base:black_eyed_susan", PLAINS, Ground::Soil, SU),
    f("base:butterfly_milkweed", PLAINS, Ground::Soil, SU),
    f("base:leadplant", PLAINS, Ground::Soil, SU),
    f("base:blazing_star", PLAINS, Ground::Soil, SU | AU),
    // Hessian beech-oak woods: spring's carpets before the leaves, foxglove in the clearings.
    f("base:wood_anemone", FOREST, Ground::Soil, SP),
    f("base:lesser_celandine", FOREST, Ground::Soil, SP),
    f("base:lungwort", FOREST, Ground::Soil, SP),
    f("base:sweet_woodruff", FOREST, Ground::Soil, SP),
    f("base:foxglove", FOREST, Ground::Soil, SU),
    // Moscow's mixed birch forest.
    f("base:lily_of_the_valley", BIRCH, Ground::Soil, SP),
    f("base:wood_cranesbill", BIRCH, Ground::Soil, SU),
    f("base:may_lily", BIRCH, Ground::Soil, SP),
    f("base:bellflower", BIRCH, Ground::Soil, SU),
    f("base:fireweed", &["base:birch_forest", "base:taiga"], Ground::Soil, SU),
    // Interior Alaska's taiga.
    f("base:twinflower", TAIGA, Ground::Soil, SU),
    f("base:bunchberry", TAIGA, Ground::Soil, SU),
    f("base:prickly_rose", TAIGA, Ground::Soil, SU),
    f("base:labrador_tea", TAIGA, Ground::Soil, SU),
    // Utqiaġvik's coastal tundra: coltsfoot first as the snow goes.
    f("base:arctic_poppy", SNOWY, Ground::Soil, SU),
    f("base:lousewort", SNOWY, Ground::Soil, SU),
    f("base:saxifrage", SNOWY, Ground::Soil, SU),
    f("base:arctic_coltsfoot", SNOWY, Ground::Soil, SP | SU),
    f("base:pygmy_buttercup", SNOWY, Ground::Soil, SU),
    // The Sonoran desert, on its sand, after the winter rains.
    f("base:brittlebush", DESERT, Ground::Sand, WI | SP),
    f("base:globemallow", DESERT, Ground::Sand, SP),
    f("base:gold_poppy", DESERT, Ground::Sand, WI | SP),
    f("base:desert_marigold", DESERT, Ground::Sand, SP | SU | AU),
    f("base:ocotillo", DESERT, Ground::Sand, SP),
    // The llanos in the wet season.
    f("base:sensitive_plant", SAVANNA, Ground::Soil, SU),
    f("base:senna", SAVANNA, Ground::Soil, SU),
    f("base:pencil_flower", SAVANNA, Ground::Soil, SU),
    f("base:morning_glory", &["base:savanna", "base:swamp"], Ground::Soil, SP | SU | AU),
    // The Pantanal (and its water hyacinth, a wild plant).
    f("base:water_primrose", SWAMP, Ground::Soil, SU),
    f("base:alligator_flag", SWAMP, Ground::Soil, SU),
    // Davos's subalpine meadows.
    f("base:alpenrose", MOUNTAINS, Ground::Soil, SU),
    f("base:gentian", MOUNTAINS, Ground::Soil, SP | SU),
    f("base:edelweiss", MOUNTAINS, Ground::Soil, SU),
    f("base:globeflower", MOUNTAINS, Ground::Soil, SU),
    f("base:alpine_aster", MOUNTAINS, Ground::Soil, SU),
    // The Ria Formosa's dunes at Faro.
    f("base:sea_daffodil", BEACH, Ground::Sand, SU),
    f("base:sea_holly", BEACH, Ground::Sand, SU),
    f("base:horned_poppy", BEACH, Ground::Sand, SU),
    f("base:sea_rocket", BEACH, Ground::Sand, SP | SU),
    f("base:hottentot_fig", BEACH, Ground::Sand, SP),
];

/// The flowers of a biome on that ground.
pub fn flowers(biome: &str, ground: Ground) -> impl Iterator<Item = &'static Flower> + '_ {
    FLOWERS
        .iter()
        .filter(move |f| f.ground == ground && f.found.contains(&biome))
}

/// How a tree's leaves turn in autumn (world-looks #7, with the lighting session's seasonal tints):
/// evergreens never, birch-like trees yellow, the rest orange to brown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Turn {
    Evergreen,
    Yellow,
    Orange,
}

impl Turn {
    /// The leaves' tint kind (`base_api::trees`): `core:foliage` for the orange ones, as before.
    pub fn tint(self) -> &'static str {
        match self {
            Turn::Evergreen => "base:evergreen",
            Turn::Yellow => "base:yellow_leaves",
            Turn::Orange => "core:foliage",
        }
    }
    /// The colour at autumn's height, for the leaves' tint (`None`: it doesn't turn): a birch's or
    /// a ginkgo's gold, an oak's russet.
    pub fn autumn(self) -> Option<u32> {
        match self {
            Turn::Evergreen => None,
            Turn::Yellow => Some(0xd9b23a),
            Turn::Orange => Some(0xb8622a),
        }
    }
}

/// Every tree's leaves and how they turn: the conifers, palms and the tree fern stay green;
/// birches and ginkgos go gold; oaks, acacias, mesquite and the swamp cypress (a deciduous conifer)
/// go russet.
pub const LEAVES: [(&str, Turn); 12] = [
    ("base:oak_leaves", Turn::Orange),
    ("base:birch_leaves", Turn::Yellow),
    ("base:spruce_leaves", Turn::Evergreen),
    ("base:acacia_leaves", Turn::Orange),
    ("base:mesquite_leaves", Turn::Orange),
    ("base:acuri_palm_leaves", Turn::Evergreen),
    ("base:moriche_palm_leaves", Turn::Evergreen),
    ("animals:araucaria_leaves", Turn::Evergreen),
    ("animals:ginkgo_leaves", Turn::Yellow),
    ("animals:swamp_cypress_leaves", Turn::Orange),
    ("animals:scale_conifer_leaves", Turn::Evergreen),
    ("animals:tempskya_leaves", Turn::Evergreen),
];

/// How `leaves` turn (`Turn::Orange` for leaves not listed).
pub fn turn(leaves: &str) -> Turn {
    LEAVES
        .iter()
        .find(|(l, _)| *l == leaves)
        .map_or(Turn::Orange, |&(_, t)| t)
}

/// Where still ponds lie, and in what share of the ground's 8 x 8 cells: the Pantanal's and the
/// Cedar Mountain floodplain's standing water, a block deep, for their floating plants and the
/// wild rice and cattails on their shores (the world's other water is the sea's, with beaches).
pub const POND_LANDS: [(&str, f32); 2] = [("base:swamp", 0.35), ("animals:floodplain", 0.2)];

/// Cordage plants: the chance cutting one gives plant fibre (`clothing`; tall grass gives it a time
/// in eight). Sedges, cottongrass and cattail leaves were twisted into cord wherever they grow, so
/// the tundra and the marshes have fibre too (Research round 3, B3).
pub const FIBRE: [(&str, f32); 3] = [("base:sedge", 0.5), ("base:cottongrass", 0.5), ("base:cattail", 1.0)];

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
/// Still fresh water in the temperate and boreal lands: white water lilies' (Nymphaea alba, and
/// N. tetragona into the taiga).
/// Water lilies on still water; the llanos' and the Pantanal's lagoons have theirs too.
pub const LAKES: &[&str] = &["base:plains", "base:forest", "base:birch_forest", "base:swamp", "base:taiga", "base:savanna"];
/// Common reed (Phragmites australis) rings the temperate and boreal lakes; the swamp's and the
/// llanos' shores are mostly the cattails' and the wild rice's, with a few reeds (`COVER_IN`).
pub const REEDBEDS: &[&str] = &["base:plains", "base:forest", "base:birch_forest", "base:taiga", "base:swamp"];
/// Sheltered sea floor: eelgrass meadows (Zostera) off the beaches.
pub const SEA_MEADOWS: &[&str] = &["base:ocean", "base:beach"];
/// Kelp forests: the open sea's rocky and sandy floor.
pub const KELP_FORESTS: &[&str] = &["base:ocean"];
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
pub const WILD: [Wild; 26] = [
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
    w("base:cattail", Form::Still, Ground::Shore, Some("base:cattail_root"), MARSHES, 0.15),
    w("base:reed", Form::Still, Ground::Shore, None, REEDBEDS, 0.12),
    w("base:lily_pad", Form::Still, Ground::Water, None, LAKES, 0.12),
    w("base:seagrass", Form::Still, Ground::Bed, None, SEA_MEADOWS, 0.2),
    // Kelp standing in the sea; picked, a handful of kelp (`food::FOODS`).
    w("base:kelp", Form::Column, Ground::Bed, Some("base:kelp"), KELP_FORESTS, 0.08),
    w("animals:cycad", Form::Still, Ground::Soil, Some("animals:cycad_seed"), CYCAD_LANDS, 0.01),
    w("animals:flowering_shrub", Form::Still, Ground::Soil, None, FLOWERING, 0.02),
    w("animals:water_lily", Form::Still, Ground::Water, None, PONDS, 0.06),
    w("animals:rotten_log", Form::Log, Ground::Soil, None, DEADWOOD, 0.004),
    // Kelp washed up on the beach (wrack, the beach's food: sand fleas and flies on it feed the
    // shorebirds); the kelp standing in the sea is `base:kelp` above.
    w("base:kelp_wrack", Form::Mat, Ground::Sand, Some("base:kelp"), &["base:beach"], 0.03),
];

/// How deep the water over a plant rooted on the bed is, from its floor to the surface, in
/// blocks: eelgrass in the shallows, kelp further out.
pub fn depths(block: &str) -> std::ops::RangeInclusive<i32> {
    match block {
        "base:kelp" => 6..=20,
        _ => 1..=6,
    }
}

/// A wild plant by its block's full name.
pub fn of(block: &str) -> Option<&'static Wild> {
    WILD.iter().find(|p| p.block == block)
}

/// A plant's cover in one biome where it isn't its usual (`Wild::cover`): reeds in the swamp at
/// 4%, leaving its shore to the wild rice (spring's grain and the capybaras' food) and the cattails.
pub const COVER_IN: [(&str, &str, f32); 1] = [("base:reed", "base:swamp", 0.04)];

/// `p`'s cover in `biome`: its own, unless `COVER_IN` gives that biome another.
pub fn cover_in(p: &Wild, biome: &str) -> f32 {
    COVER_IN
        .iter()
        .find(|&&(b, at, _)| b == p.block && at == biome)
        .map_or(p.cover, |&(_, _, c)| c)
}

/// Which wild plant (its index in `WILD`) grows on a cell of `ground` in `biome`, for a roll in
/// 0..1: the plants of that ground found there, each taking its cover of the rolls in turn.
pub fn pick(biome: &str, ground: Ground, roll: f32) -> Option<usize> {
    let mut at = 0.0;
    for (i, p) in WILD.iter().enumerate() {
        if p.ground != ground || !p.found.contains(&biome) {
            continue;
        }
        at += cover_in(p, biome);
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
                    .map(|q| cover_in(q, b))
                    .sum();
                assert!(all < MOST, "{b}: wild plants cover {:.0}% of its {:?} ground", all * 100.0, p.ground);
            }
        }
    }
}

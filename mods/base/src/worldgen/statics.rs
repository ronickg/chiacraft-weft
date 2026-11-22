//! What a biome grows, as data any mod can set on any biome, the tags worldgen reads, base's
//! biomes and its features.

use super::*;

pub(super) static PLANT_SOIL: Tag = Tag::named("plant_soil");
pub(super) static CACTUS_SOIL: Tag = Tag::named("cactus_soil");
pub(super) static SUGAR_CANE_SOIL: Tag = Tag::named("sugar_cane_soil");
pub(super) static SAND_TAG: Tag = Tag::named("sand");
/// What a biome grows, as data any mod can set on any biome: trees per 5x5 cell (0..1), share of
/// grass on plantable soil, which trees and flowers (block lists the engine checks), and tags `frozen` (nothing grows, set by
/// `.frozen(..)`), `dry` (cacti).
pub static TREES: Component<f32> = Component::named("trees");
/// Boulders: the chance of a heap of stone per 12 x 12 cell (`boulder`).
pub static BOULDERS: Component<f32> = Component::named("boulders");
/// Where a forest stands thick: the chance of a tree per 3 x 3 cell in its stands, on top of
/// `TREES`, with clearings left open between them. Mature stands hold 170-244 trees a hectare
/// (Research round 7): a 5 x 5 site at 0.4 is 160, a 3 x 3 one at 0.12 adds 130 in the stands (the
/// trees grown from branches are wide: denser, the forest and taiga read as walls of leaves).
pub static STAND_TREES: Component<f32> = Component::named("stand_trees");
pub static GRASS_COVER: Component<f32> = Component::named("grass");
/// The logs of the trees a biome grows, with weights: `"base:birch_log:1 base:oak_log:4"`
/// (oaks where unset).
pub static TREE_KINDS: Component<BlockList> = Component::named("tree_kinds");
/// Flowers a biome grows, picked evenly (`"base:dandelion base:rose"` where unset).
pub static FLOWERS: Component<BlockList> = Component::named("flowers");
/// Giant trees (a sequoia grove's): the chance of one per 24 x 24 cell, and which (logs with a
/// recipe, `tree_recipes`), in a scatter of their own so distant terrain draws them at their own
/// height, not averaged with the small trees.
pub static GIANT_TREES: Component<f32> = Component::named("giant_trees");
pub static GIANT_KINDS: Component<BlockList> = Component::named("giant_kinds");
/// The layers under the trees (`vitals::layers`, `notes/VEGETATION.md`), as data on the biome:
/// floor blocks swapped in for the top block in patches, by weight, and the share they cover;
/// ferns on the bare soil; shrub clumps per 4 x 4 cell in thickets, made of these leaves.
pub static FLOOR: Component<BlockList> = Component::named("floor");
pub static FLOOR_COVER: Component<f32> = Component::named("floor_cover");
pub static FERNS: Component<f32> = Component::named("ferns");
pub static SHRUBS: Component<f32> = Component::named("shrubs");
pub static SHRUB_KINDS: Component<BlockList> = Component::named("shrub_kinds");
/// Wild cherries in their groves (`vitals::wild::CHERRY_GROVES`): the chance per 4 x 4 cell in a
/// grove patch.
pub static CHERRY_TREES: Component<f32> = Component::named("cherry_trees");
/// Young trees of the biome's `YOUNG_KINDS` (else its `TREE_KINDS`), the chance per 5 x 5 cell,
/// out of the thick stands.
pub static YOUNG_TREES: Component<f32> = Component::named("young_trees");
pub static YOUNG_KINDS: Component<BlockList> = Component::named("young_kinds");
/// The biome's fern (`FERN`, bracken, where unset).
pub static FERN_KINDS: Component<BlockList> = Component::named("fern_kinds");
/// Mushrooms at the foot of trunks: which, and their share of the bare soil beside a trunk.
pub static MUSHROOMS: Component<BlockList> = Component::named("mushrooms");
pub static MUSHROOM_COVER: Component<f32> = Component::named("mushroom_cover");
/// The share of a biome's trees with lianas hanging from their crowns (`vitals::layers`).
pub static LIANAS: Component<f32> = Component::named("lianas");
/// Treefall gaps (`vitals::layers`): the chance a 48-block cell holds one.
pub static TREEFALL: Component<f32> = Component::named("treefall");
/// The share of a biome's tall grass that grows two blocks tall (`TALL_GRASS_DOUBLE`).
pub static TALL_GRASS_SHARE: Component<f32> = Component::named("tall_grass_share");
/// Big-leaved plants two blocks tall (`vitals::layers`): the share of the bare soil, and which.
pub static BIG_LEAVES: Component<f32> = Component::named("big_leaves");
pub static BIG_LEAF_KINDS: Component<BlockList> = Component::named("big_leaf_kinds");
pub(super) static FROZEN: Tag = Tag::named("frozen");
pub(super) static DRY: Tag = Tag::named("dry");
pub static PLAINS: Biome = Biome::named("plains");
pub static FOREST: Biome = Biome::named("forest");
pub static BIRCH_FOREST: Biome = Biome::named("birch_forest");
pub static TAIGA: Biome = Biome::named("taiga");
pub static SNOWY: Biome = Biome::named("snowy_plains");
pub static DESERT: Biome = Biome::named("desert");
pub static SAVANNA: Biome = Biome::named("savanna");
pub static SWAMP: Biome = Biome::named("swamp");
/// The hot, wet corner (`base_api::land::TROPICAL`): rainforest.
pub static TROPICAL_FOREST: Biome = Biome::named("tropical_forest");
pub static MOUNTAINS: Biome = Biome::named("mountains");
pub static SEQUOIA_GROVE: Biome = Biome::named("sequoia_grove");
/// Old-growth forest (`base_api::land::OLD_GROWTH_AREA`): the closed canopy of old oaks in
/// Bliss's comparison shots, rivers and lakes through it.
pub static OLD_GROWTH_FOREST: Biome = Biome::named("old_growth_forest");
pub static OCEAN: Biome = Biome::named("ocean");
/// Warm shallow seas (`base_api::land::REEF_*`): the lagoon along the shore, the reef farther out.
pub static LAGOON: Biome = Biome::named("lagoon");
pub static CORAL_REEF: Biome = Biome::named("coral_reef");
pub static BEACH: Biome = Biome::named("beach");
pub static DEEP_CAVES: Biome = Biome::named("deep_caves");
pub(super) static TREE_SITES: Feature = Feature::named("trees");
pub(super) static CHERRY_SITES: Feature = Feature::named("cherry_trees");
pub(super) static STAND_SITES: Feature = Feature::named("stand_trees");
pub(super) static GIANT_SITES: Feature = Feature::named("giant_trees");
pub(super) static BOULDER_SITES: Feature = Feature::named("boulders");
pub(super) static SHRUB_SITES: Feature = Feature::named("shrubs");
pub(super) static YOUNG_SITES: Feature = Feature::named("young_trees");
pub(super) static LOGS: Tag = Tag::named("logs");
pub(super) static LEAVES: Tag = Tag::named("leaves");
pub(super) static WET: Tag = Tag::named("wet");
pub(super) static WOODS: Tag = Tag::named("forest");

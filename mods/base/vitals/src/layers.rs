//! The layers of a biome's vegetation under and between its trees (`notes/VEGETATION.md`): the
//! floor the ground shows, ferns, shrubs, and young trees, each biome's from the real place its
//! climate comes from (`notes/ECOLOGY.md`, "Each biome's real place"). Base sets them on its
//! biomes as components (`mods/base/src/worldgen.rs`), so another mod sets its own the same way.
//! The canopy is `TREES` and `STAND_TREES`; the grass is `forage::GRASS_COVER`; the wild plants
//! and flowers are `wild`.

#[derive(Clone, Copy, Debug)]
pub struct Layers {
    pub biome: &'static str,
    /// Floor blocks swapped in for the top block in patches, by weight (a forest's leaf litter,
    /// its bare dirt): `"base:podzol:3 base:dirt:1"`. Empty for none.
    pub floor: &'static str,
    /// The share of the plant soil the floor patches cover; under the crowns they're twice as
    /// thick, in the open half.
    pub floor_cover: f32,
    /// Ferns, the share of the plant soil left bare by grass and flowers; round trunks three times
    /// as thick.
    pub ferns: f32,
    /// Which fern (`base:fern`, bracken, where empty; the grove's sword fern).
    pub fern: &'static str,
    /// Shrub clumps, the chance of one per 4 x 4 cell (in thickets: patches some 24 blocks across
    /// hold them, the rest of the ground none).
    pub shrubs: f32,
    /// The leaves the shrubs are made of, by weight.
    pub shrub_kinds: &'static str,
    /// Young trees of the biome's kinds, the chance of one per 5 x 5 cell, in clearings and at
    /// the stands' edges where light comes in (none deep in a stand).
    pub young: f32,
    /// The young trees' kinds, by weight (the biome's own `TREE_KINDS` where empty).
    pub young_kinds: &'static str,
    /// The share of the tall grass that grows two blocks tall (the prairie's big bluestem, the
    /// llanos' grasses).
    pub tall_grass: f32,
    /// Mushrooms at the foot of trunks, by weight, and the share of the bare soil beside a trunk
    /// they take (before the ferns).
    pub mushrooms: &'static str,
    pub mushroom_cover: f32,
    /// The share of the biome's trees with lianas hanging from their crowns (the tropical
    /// forest's climbers: a quarter to a half of its canopy trees carry them).
    pub lianas: f32,
}

const fn l(
    biome: &'static str,
    floor: &'static str,
    floor_cover: f32,
    ferns: f32,
    shrubs: f32,
    shrub_kinds: &'static str,
    young: f32,
) -> Layers {
    Layers {
        biome,
        floor,
        floor_cover,
        ferns,
        fern: "",
        shrubs,
        shrub_kinds,
        young,
        young_kinds: "",
        tall_grass: 0.0,
        mushrooms: "",
        mushroom_cover: 0.0,
        lianas: 0.0,
    }
}

/// Each biome's layers. Floors: Białowieża's leaf litter and moss, Fairbanks' moss carpet,
/// Konza's, the llanos' and Sonora's bare stony ground. Shrubs: Białowieża's hazel and hornbeam
/// under the oaks; Moscow's rowan and raspberry under birches (birch leaves stand in); Fairbanks'
/// willow at the edges and juniper; Konza's dogwood and sumac thickets (oak leaves stand in); the
/// llanos' and the Pantanal's scattered shrubs; Davos' juniper mats; the tundra's dwarf willow;
/// Sonora's creosote bush.
pub const LAYERS: [Layers; 11] = [
    Layers { tall_grass: 0.4, ..l("base:plains", "base:coarse_dirt", 0.03, 0.0, 0.06, "base:oak_leaves", 0.0) },
    // Mushrooms round trunks: Białowieża's honey fungus on the oaks' roots and ceps; Moscow's
    // chanterelles under birches; the taiga's chanterelles and ceps under spruce.
    Layers {
        mushrooms: "base:honey_fungus:3 base:porcini:2 base:mushroom_brown:1",
        mushroom_cover: 0.12,
        ..l("base:forest", "base:leaf_litter:3 base:moss_block:1", 0.25, 0.05, 0.2, "base:oak_leaves:3 base:birch_leaves:1", 0.15)
    },
    Layers {
        mushrooms: "base:chanterelle:3 base:honey_fungus:1 base:mushroom_red:1",
        mushroom_cover: 0.12,
        ..l("base:birch_forest", "base:leaf_litter", 0.15, 0.06, 0.18, "base:birch_leaves:2 base:oak_leaves:1", 0.25)
    },
    Layers {
        mushrooms: "base:chanterelle:2 base:porcini:2 base:mushroom_red:1",
        mushroom_cover: 0.1,
        ..l("base:taiga", "base:moss_block:3 base:podzol:1", 0.2, 0.03, 0.16, "base:willow_leaves:2 base:juniper_leaves:1", 0.25)
    },
    l("base:snowy_plains", "", 0.0, 0.0, 0.1, "base:willow_leaves", 0.0),
    Layers { tall_grass: 0.5, ..l("base:savanna", "base:coarse_dirt", 0.04, 0.0, 0.05, "base:acacia_leaves", 0.0) },
    l("base:swamp", "base:moss_block", 0.08, 0.02, 0.12, "base:oak_leaves", 0.0),
    l("base:mountains", "base:moss_block:1 base:gravel:1", 0.06, 0.02, 0.14, "base:juniper_leaves", 0.1),
    l("base:desert", "base:gravel:1 base:coarse_dirt:1", 0.03, 0.0, 0.08, "base:creosote_leaves", 0.0),
    // Giant Forest: needle and bark litter under the giants, sword fern on the moist floor, the
    // Pacific dogwood the commonest small tree under them with young firs and sequoias in the
    // gaps, few shrubs (`blocks.rs`, `grove`).
    Layers {
        fern: "base:sword_fern",
        mushrooms: "base:honey_fungus",
        mushroom_cover: 0.06,
        young_kinds: "base:dogwood_log:3 base:spruce_log:1 base:sequoia_log:1",
        ..l("base:sequoia_grove", "base:podzol:2 base:leaf_litter:1", 0.2, 0.1, 0.06, "base:dogwood_leaves", 0.2)
    },
    // The central Amazon's terra firme (Manaus): a deep litter of leaves on a dark floor, ferns in
    // the shade, cacao and young trees in the understorey (the canopy shades out grass: little of
    // it), fungi on the litter, and lianas on some four trees in ten.
    Layers {
        mushrooms: "base:mushroom_brown",
        mushroom_cover: 0.06,
        young_kinds: "base:cacao_log:3 base:brazil_nut_log:1 base:acuri_palm_log:1",
        lianas: 0.4,
        ..l("base:tropical_forest", "base:leaf_litter:3 base:podzol:1", 0.4, 0.1, 0.25, "base:cacao_leaves", 0.3)
    },
];

/// Biomes with layers here that the world doesn't register yet: the tropical forest is Terrain's
/// (its own branch). Their rows wait; take a biome out once it's registered.
pub const PENDING: &[&str] = &["base:tropical_forest"];

/// A biome's layers, if it has any.
pub fn of(biome: &str) -> Option<&'static Layers> {
    LAYERS.iter().find(|l| l.biome == biome)
}

/// How tall a shrub clump grows in `biome`: the tundra's and the mountains' hug the ground (one
/// block), the rest reach two.
pub fn shrub_height(biome: &str) -> i32 {
    match biome {
        "base:snowy_plains" | "base:mountains" | "base:desert" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_leave_room() {
        for l in &LAYERS {
            // Floor patches up to twice their cover under crowns; under half the soil.
            assert!(l.floor_cover * 2.0 < 0.5, "{}: floor patches too thick", l.biome);
            // Ferns round trunks are three times as thick; still under half the bare soil.
            assert!(l.ferns * 3.0 < 0.5, "{}: ferns too thick", l.biome);
            assert!((0.0..=0.5).contains(&l.shrubs), "{}: shrubs", l.biome);
            assert!(l.shrubs == 0.0 || !l.shrub_kinds.is_empty(), "{}: shrubs with no kinds", l.biome);
            assert!(l.floor_cover == 0.0 || !l.floor.is_empty(), "{}: a floor cover with no blocks", l.biome);
            assert!((0.0..=0.5).contains(&l.young), "{}: young trees", l.biome);
            assert!((0.0..=1.0).contains(&l.tall_grass), "{}: tall grass share", l.biome);
            // Mushrooms and ferns round a trunk share its bare soil.
            assert!(l.mushroom_cover + l.ferns * 3.0 < 0.6, "{}: mushrooms and ferns crowd the trunks", l.biome);
            assert!(l.mushroom_cover == 0.0 || !l.mushrooms.is_empty(), "{}: mushrooms with no kinds", l.biome);
            assert!(crate::climate::of(l.biome).is_some(), "{}: no climate", l.biome);
            assert!((0.0..=1.0).contains(&l.lianas), "{}: lianas {}", l.biome, l.lianas);
        }
        // A biome waiting to be registered has its row here, or it isn't waiting for anything.
        for b in PENDING {
            assert!(of(b).is_some(), "{b}: pending with no layers");
        }
    }
}

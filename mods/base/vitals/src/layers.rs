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
    /// Shrub clumps, the chance of one per 4 x 4 cell (in thickets: patches some 24 blocks across
    /// hold them, the rest of the ground none).
    pub shrubs: f32,
    /// The leaves the shrubs are made of, by weight.
    pub shrub_kinds: &'static str,
}

const fn l(biome: &'static str, floor: &'static str, floor_cover: f32, ferns: f32, shrubs: f32, shrub_kinds: &'static str) -> Layers {
    Layers { biome, floor, floor_cover, ferns, shrubs, shrub_kinds }
}

/// Each biome's layers. Shrubs: Białowieża's hazel and hornbeam under the oaks; Moscow's rowan
/// and raspberry under birches (birch leaves stand in); Fairbanks' willow and alder at the edges
/// (birch leaves stand in) and juniper (spruce); Konza's dogwood and sumac thickets (oak); the
/// llanos' and the Pantanal's scattered shrubs; Davos' dwarf mountain pine and juniper mats
/// (spruce); the tundra's dwarf willow and birch (birch). The look of their own leaves is Chiacraft's
/// to draw; until then they are the trees' leaves.
pub const LAYERS: [Layers; 9] = [
    l("base:plains", "base:dirt", 0.03, 0.0, 0.06, "base:oak_leaves"),
    l("base:forest", "base:podzol:3 base:dirt:1", 0.25, 0.05, 0.2, "base:oak_leaves:3 base:birch_leaves:1"),
    l("base:birch_forest", "base:podzol", 0.15, 0.06, 0.18, "base:birch_leaves:2 base:oak_leaves:1"),
    l("base:taiga", "", 0.0, 0.03, 0.16, "base:birch_leaves:2 base:spruce_leaves:1"),
    l("base:snowy_plains", "", 0.0, 0.0, 0.1, "base:birch_leaves"),
    l("base:savanna", "base:dirt", 0.04, 0.0, 0.05, "base:acacia_leaves"),
    l("base:swamp", "", 0.0, 0.02, 0.12, "base:oak_leaves"),
    l("base:mountains", "base:gravel", 0.04, 0.02, 0.14, "base:spruce_leaves"),
    l("base:desert", "base:gravel", 0.03, 0.0, 0.08, "base:mesquite_leaves"),
];

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
            assert!(crate::climate::of(l.biome).is_some(), "{}: no climate", l.biome);
        }
    }
}

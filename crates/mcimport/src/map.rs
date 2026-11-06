//! Minecraft block states and biomes to ours. Each gives candidates in order: the engine takes
//! the first that exists in the loaded content, so a block we have under the Minecraft name
//! (`base:cherry_leaves`) wins and the rest are the nearest matches we surely have. The texture
//! test mode (`references/mc-pack`) paints base's terrain blocks with Minecraft's own textures,
//! so terrain goes to those blocks: grass, dirt, stone, sand, gravel, clay, the logs and leaves.

/// A state split into its name (without `minecraft:`) and properties.
pub struct State<'a> {
    pub name: &'a str,
    pub props: Vec<(&'a str, &'a str)>,
}

impl<'a> State<'a> {
    pub fn parse(s: &'a str) -> State<'a> {
        let s = s.strip_prefix("minecraft:").unwrap_or(s);
        let (name, props) = match s.split_once('[') {
            Some((n, p)) => (n, p.trim_end_matches(']')),
            None => (s, ""),
        };
        let props = props
            .split(',')
            .filter_map(|kv| kv.split_once('='))
            .collect();
        State { name, props }
    }

    pub fn prop(&self, k: &str) -> Option<&'a str> {
        self.props.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
    }
}

const AIR: &str = "core:air";

/// Wood kinds: Minecraft's, then ours in order of likeness.
fn wood(kind: &str) -> &'static [&'static str] {
    match kind {
        "oak" => &["oak"],
        "spruce" => &["spruce"],
        "birch" => &["birch"],
        "acacia" => &["acacia"],
        "cherry" => &["cherry", "dogwood", "oak"],
        "dark_oak" => &["dark_oak", "oak"],
        "pale_oak" => &["pale_oak", "birch"],
        "jungle" => &["jungle", "cacao", "oak"],
        "mangrove" => &["mangrove", "matamata", "oak"],
        // 26.3's poplars (the dappled forest): pale trunks like birch's, autumn-coloured leaves.
        "poplar" | "orange_poplar" | "red_poplar" | "yellow_poplar" => &["poplar", "birch"],
        "azalea" | "flowering_azalea" => &["oak"],
        _ => &["oak"],
    }
}

fn axis(s: &State) -> String {
    format!("[axis={}]", s.prop("axis").unwrap_or("y"))
}

fn names(list: &[String]) -> Vec<String> {
    list.to_vec()
}

/// Candidates for a Minecraft block state (`minecraft:oak_log[axis=y]`), best first. The last is
/// always one we have (stone for solid unknowns, air for small ones), and `exact` says whether
/// the first is meant as the same block rather than a stand-in.
pub fn block(mc: &str) -> (Vec<String>, bool) {
    let s = State::parse(mc);
    let n = s.name;
    let one = |b: &str| (vec![b.to_string()], true);
    let near = |v: &[&str]| (v.iter().map(|b| b.to_string()).collect::<Vec<_>>(), true);
    // Logs, wood, leaves, planks of every kind.
    for (suffix, of) in [("_log", "log"), ("_wood", "log"), ("_leaves", "leaves"), ("_planks", "planks")] {
        if let Some(kind) = n.strip_suffix(suffix) {
            let (stripped, kind) = match kind.strip_prefix("stripped_") {
                Some(k) => ("stripped_", k),
                None => ("", kind),
            };
            if kind.ends_with("crimson") || kind.ends_with("warped") {
                break;
            }
            let props = if of == "log" { axis(&s) } else { String::new() };
            let mut v: Vec<String> = Vec::new();
            // Our block of the very same name first (`base:yellow_poplar_leaves`), if we have it.
            // Dark oak's bark and planks are nearest spruce's (its leaves stay oak's, tinted).
            let near: &[&str] = if kind == "dark_oak" && of != "leaves" { &["dark_oak", "spruce", "oak"] } else { wood(kind) };
            let kinds = std::iter::once(kind).chain(near.iter().copied().filter(|k| *k != kind));
            for k in kinds {
                // Leaves as placed: ours decay farther than 4 or so from a log, Minecraft's only
                // past 6, so a poplar's crown fell off and left a bare trunk.
                if of == "leaves" {
                    // Minecraft's cherry is pink all year; ours only in its spring bloom.
                    if k == "cherry" {
                        v.push("base:cherry_leaves[bloom=1,placed=true]".into());
                        v.push("base:cherry_leaves[bloom=1]".into());
                    }
                    v.push(format!("base:{k}_leaves[placed=true]"));
                }
                v.push(format!("base:{stripped}{k}_{of}{props}"));
                v.push(format!("base:{stripped}{k}_{of}"));
            }
            if !stripped.is_empty() {
                v.push(format!("base:oak_log{props}"));
            }
            v.push(format!("base:oak_{of}"));
            return (names(&v), true);
        }
    }
    match n {
        "air" | "cave_air" | "void_air" | "structure_void" | "light" | "barrier" => one(AIR),
        "water" | "bubble_column" => {
            let l = s.prop("level").unwrap_or("0");
            (vec![format!("base:water[level={l}]"), "base:water".into()], true)
        }
        "lava" => {
            let l = s.prop("level").unwrap_or("0");
            (vec![format!("base:lava[level={l}]"), "base:lava".into()], true)
        }
        "grass_block" => {
            if s.prop("snowy") == Some("true") {
                near(&["base:snowy_grass", "base:grass"])
            } else {
                one("base:grass")
            }
        }
        "dirt" | "rooted_dirt" | "mud" | "packed_mud" => one("base:dirt"),
        "coarse_dirt" => near(&["base:coarse_dirt", "base:dirt"]),
        "podzol" | "mycelium" => near(&["base:podzol", "base:dirt"]),
        "dirt_path" => near(&["base:dirt_path", "base:coarse_dirt", "base:dirt"]),
        "farmland" => near(&["base:farmland", "base:dirt"]),
        "moss_block" => near(&["base:moss_block", "base:grass"]),
        "stone" | "deepslate" | "tuff" | "andesite" | "diorite" | "calcite" | "smooth_basalt" | "infested_stone" | "infested_deepslate"
        | "dripstone_block" | "smooth_stone" | "polished_andesite" | "polished_diorite" => one("base:stone"),
        "bedrock" => one("base:stone"),
        "granite" | "polished_granite" => near(&["base:granite", "base:stone"]),
        "cobblestone" | "cobbled_deepslate" => near(&["base:cobblestone", "base:stone"]),
        "mossy_cobblestone" => near(&["base:mossy_cobblestone", "base:cobblestone"]),
        "stone_bricks" | "mossy_stone_bricks" | "cracked_stone_bricks" | "deepslate_bricks" => near(&["base:stone_bricks", "base:stone"]),
        "bricks" => near(&["base:bricks", "base:stone_bricks"]),
        "sand" | "suspicious_sand" => one("base:sand"),
        "red_sand" => near(&["animals:red_sand", "base:sand"]),
        "sandstone" | "smooth_sandstone" | "cut_sandstone" | "chiseled_sandstone" => near(&["base:sandstone", "base:sand"]),
        "red_sandstone" | "smooth_red_sandstone" | "cut_red_sandstone" => {
            near(&["base:red_sandstone", "animals:red_sandstone", "base:sandstone"])
        }
        "gravel" | "suspicious_gravel" => one("base:gravel"),
        "clay" => one("base:clay"),
        "terracotta"
        | "white_terracotta"
        | "orange_terracotta"
        | "yellow_terracotta"
        | "red_terracotta"
        | "brown_terracotta"
        | "light_gray_terracotta" => near(&["base:banded_clay", "base:clay"]),
        "snow_block" | "powder_snow" => one("base:snow"),
        "snow" => {
            let l = s.prop("layers").unwrap_or("1");
            (vec![format!("base:snow_layer[layers={l}]"), "base:snow_layer".into(), AIR.into()], true)
        }
        "ice" | "frosted_ice" => one("base:ice"),
        "packed_ice" | "blue_ice" => near(&["base:glacier_ice", "base:ice"]),
        "obsidian" | "crying_obsidian" => one("base:obsidian"),
        "magma_block" => near(&["base:stone"]),
        "coal_ore" | "deepslate_coal_ore" => one("base:coal_ore"),
        "iron_ore" | "deepslate_iron_ore" => one("base:iron_ore"),
        "copper_ore" | "deepslate_copper_ore" => near(&["base:copper_ore", "base:iron_ore"]),
        "gold_ore" | "deepslate_gold_ore" => one("base:gold_ore"),
        "diamond_ore" | "deepslate_diamond_ore" => one("base:diamond_ore"),
        "redstone_ore" | "deepslate_redstone_ore" => near(&["base:redstone_ore", "base:stone"]),
        "lapis_ore" | "deepslate_lapis_ore" | "emerald_ore" | "deepslate_emerald_ore" => (vec!["base:stone".into()], false),
        "raw_iron_block" | "iron_block" => near(&["base:iron_block", "base:stone"]),
        "glass" => one("base:glass"),
        "hay_block" => near(&["base:hay_bale", "base:dirt"]),
        "pumpkin" | "carved_pumpkin" => one("base:pumpkin"),
        "melon" => one("base:melon"),
        "cactus" => one("base:cactus"),
        "sugar_cane" | "bamboo" => one("base:sugar_cane"),
        "brain_coral_block" | "bubble_coral_block" => near(&["base:brain_coral_block", "base:sand"]),
        "tube_coral_block" | "horn_coral_block" => near(&["base:staghorn_coral_block", "base:sand"]),
        "fire_coral_block" => near(&["base:porites_coral_block", "base:sand"]),
        n if n.starts_with("dead_") && n.ends_with("_coral_block") => near(&["base:coral_rubble", "base:sand"]),
        // Plants.
        "short_grass" | "grass" => one("base:tall_grass"),
        "tall_grass" | "large_fern" => {
            let half = s.prop("half").unwrap_or("lower");
            (
                vec![format!("base:tall_grass_double[half={half}]"), if half == "lower" { "base:tall_grass".into() } else { AIR.into() }],
                true,
            )
        }
        "fern" => near(&["base:fern", "base:sword_fern", "base:tall_grass"]),
        "dead_bush" | "short_dry_grass" | "tall_dry_grass" => near(&["base:dead_bush", AIR]),
        "dandelion" => one("base:dandelion"),
        "poppy" | "red_tulip" | "rose_bush" => near(&["base:rose", "base:dandelion"]),
        "cornflower" | "blue_orchid" => near(&["base:lupine", "base:rose"]),
        "lily_of_the_valley" | "oxeye_daisy" | "azure_bluet" | "white_tulip" => {
            near(&["base:lily_of_the_valley", "base:wood_anemone", "base:dandelion"])
        }
        "allium" | "pink_tulip" | "orange_tulip" | "lilac" | "peony" => near(&["base:foxglove", "base:rose"]),
        "sunflower" => {
            let half = s.prop("half").unwrap_or("lower");
            (vec![format!("base:wild_sunflower[half={half}]"), if half == "lower" { "base:dandelion".into() } else { AIR.into() }], false)
        }
        // 26.3's wildflowers are mostly yellow patches on the forest floor.
        "wildflowers" => near(&["base:dandelion", "base:clover", AIR]),
        "pink_petals" => near(&["base:clover", AIR]),
        // A thin layer on the floor in Minecraft; ours is a whole block.
        "leaf_litter" => (vec![AIR.into()], false),
        "bush" | "firefly_bush" | "azalea" | "flowering_azalea" | "sweet_berry_bush" => near(&["base:hazel_bush", "base:tall_grass"]),
        "lily_pad" => one("base:lily_pad"),
        "seagrass" | "tall_seagrass" => near(&["base:seagrass", "base:water"]),
        "kelp" | "kelp_plant" => near(&["base:kelp", "base:water"]),
        "brown_mushroom" => near(&["base:mushroom_brown", AIR]),
        "red_mushroom" => near(&["base:mushroom_red", AIR]),
        "torch" | "wall_torch" => near(&["base:torch", AIR]),
        _ => {
            // Waterlogged small things stand in water; other small things are left out; any other
            // unknown full block is the stone it most likely sits in.
            let small = [
                "slab",
                "stairs",
                "wall",
                "fence",
                "door",
                "button",
                "pressure_plate",
                "carpet",
                "sign",
                "banner",
                "torch",
                "rail",
                "pane",
                "bars",
                "sapling",
                "vine",
                "coral",
                "fan",
                "candle",
                "lantern",
                "chain",
                "head",
                "skull",
                "pot",
                "bed",
                "lichen",
                "roots",
                "dripleaf",
                "spore",
                "propagule",
                "pickle",
                "tulip",
                "orchid",
                "flower",
                "petals",
                "berries",
                "plant",
                "grass",
                "bush",
                "mushroom",
                "dripstone",
                "amethyst_bud",
                "cluster",
                "web",
                "ladder",
                "lever",
                "hopper",
                "chest",
                "eyeblossom",
                "moss_carpet",
                "hanging",
                "egg",
                "frogspawn",
                "lily",
                "scaffolding",
                "rod",
                "string",
                "cake",
                "shrub",
                "spike",
                "cocoa",
                "wheat",
                "potatoes",
                "carrots",
                "bell",
                "cauldron",
                "stand",
                "nest",
            ];
            if small.iter().any(|k| n.contains(k)) {
                if s.prop("waterlogged") == Some("true") {
                    return (vec!["base:water[level=0]".into(), "base:water".into()], false);
                }
                return (vec![AIR.into()], false);
            }
            (vec!["base:stone".into()], false)
        }
    }
}

/// Candidates for a Minecraft biome (`minecraft:cherry_grove`), best first; the last is one we
/// have. Leaf and grass tints follow the biome (`base:mc_leaf_colours` gives each of ours its
/// Minecraft counterpart's foliage colour).
pub fn biome(mc: &str) -> Vec<&'static str> {
    let n = mc.strip_prefix("minecraft:").unwrap_or(mc);
    match n {
        "plains" | "sunflower_plains" | "meadow" => vec!["base:plains"],
        "forest" | "flower_forest" => vec!["base:forest"],
        "dappled_forest" => vec!["base:dappled_forest", "base:birch_forest", "base:forest"],
        "cherry_grove" => vec!["base:cherry_grove", "base:forest"],
        "birch_forest" | "old_growth_birch_forest" => vec!["base:birch_forest", "base:forest"],
        "dark_forest" | "pale_garden" => vec!["base:old_growth_forest", "base:forest"],
        "taiga" | "old_growth_pine_taiga" | "old_growth_spruce_taiga" => vec!["base:taiga"],
        "snowy_taiga" | "grove" => vec!["base:taiga", "base:snowy_plains"],
        "snowy_plains" | "ice_spikes" | "snowy_slopes" | "frozen_peaks" | "jagged_peaks" | "snowy_beach" => {
            vec!["base:snowy_plains"]
        }
        "stony_peaks" | "windswept_hills" | "windswept_gravelly_hills" | "windswept_forest" | "stony_shore" => {
            vec!["base:mountains"]
        }
        "windswept_savanna" | "savanna" | "savanna_plateau" => vec!["base:savanna"],
        "desert" => vec!["base:desert"],
        "badlands" | "wooded_badlands" | "eroded_badlands" => vec!["animals:red_dunes", "base:desert"],
        "jungle" | "sparse_jungle" | "bamboo_jungle" => vec!["base:tropical_forest", "base:forest"],
        "swamp" | "mangrove_swamp" => vec!["base:swamp"],
        "beach" => vec!["base:beach"],
        "river" | "frozen_river" => vec!["base:river", "base:ocean"],
        "warm_ocean" | "lukewarm_ocean" | "deep_lukewarm_ocean" => vec!["base:lagoon", "base:ocean"],
        n if n.contains("ocean") => vec!["base:ocean"],
        "dripstone_caves" | "lush_caves" | "deep_dark" | "sulfur_caves" => vec!["base:deep_caves", "base:plains"],
        _ => vec!["base:plains"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_keep_their_axis_and_kind() {
        let (v, exact) = block("minecraft:spruce_log[axis=x]");
        assert!(exact);
        assert_eq!(v[0], "base:spruce_log[axis=x]");
        let (v, _) = block("minecraft:cherry_leaves[distance=1,persistent=false,waterlogged=false]");
        assert_eq!(v[0], "base:cherry_leaves[bloom=1,placed=true]");
        assert_eq!(block("minecraft:oak_leaves[distance=6]").0[0], "base:oak_leaves[placed=true]");
        assert!(v.contains(&"base:oak_leaves".to_string()));
        let (v, _) = block("minecraft:stripped_birch_log[axis=y]");
        assert_eq!(v[0], "base:stripped_birch_log[axis=y]");
    }

    #[test]
    fn unknowns_fall_to_stone_or_air() {
        assert_eq!(block("minecraft:amethyst_block").0, vec!["base:stone"]);
        assert_eq!(block("minecraft:oak_fence[east=true]").0, vec![AIR]);
        assert_eq!(block("minecraft:grass_block[snowy=false]").0, vec!["base:grass"]);
        assert_eq!(biome("minecraft:deep_cold_ocean"), vec!["base:ocean"]);
    }
}

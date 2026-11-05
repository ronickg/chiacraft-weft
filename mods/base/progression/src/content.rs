//! Every thing in the game, tagged (`notes/PROGRESSION.md`, rule 1: "every thing has a tier and a
//! path"): each named item and block of the mods (the universe is their `en_us.lang` names) matched
//! by one rule that gives its age, path, kind and how it's had; each crafting material of the tool
//! parts with its age. A new item with no rule fails the checks, so nothing joins the game
//! untagged.
//!
//! These tag what exists (11-07) as it is: an iron-made thing is the Iron Age's even where the
//! plan would have it earlier (the cooking pot's stew), so the sheet shows where today's recipes
//! and the ages disagree.

use vitals::curve::Path;

/// What kind of thing it is, for the sheet's counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Ground,
    Wood,
    Plant,
    Crop,
    Food,
    Material,
    Tool,
    Armour,
    Clothing,
    Station,
    Building,
    Light,
    Machine,
    Part,
    Beast,
    Sky,
    Technical,
}

impl Kind {
    pub const fn name(self) -> &'static str {
        match self {
            Kind::Ground => "ground",
            Kind::Wood => "wood",
            Kind::Plant => "plant",
            Kind::Crop => "crop",
            Kind::Food => "food",
            Kind::Material => "material",
            Kind::Tool => "tool",
            Kind::Armour => "armour",
            Kind::Clothing => "clothing",
            Kind::Station => "station",
            Kind::Building => "building",
            Kind::Light => "light",
            Kind::Machine => "machine",
            Kind::Part => "part",
            Kind::Beast => "beast",
            Kind::Sky => "sky",
            Kind::Technical => "technical",
        }
    }
}

/// What breaking or gathering needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Hand,
    /// A tool of a kind (`TOOLS`) that harvests at least this level (`abi::tool_tier`: wood 0,
    /// stone 100, iron 200, diamond 300).
    Tool(&'static str, u32),
}

/// How a thing is had.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Get {
    /// Gathered in the world with `Needs`, at this many effort minutes a piece (finding it
    /// included).
    World(Needs, f32),
    /// Dropped by creatures, effort minutes a piece (the hunt included).
    Drop(f32),
    /// Made: a recipe in `graph::RECIPES` must make it.
    Made,
    /// Not had as an item: a block that breaks into something else, a liquid, a growing stage, a
    /// world's own (a portal), or one with no way in yet.
    Not,
}

/// A tag rule: the ids it matches (`*` matches any run of letters), and what they are.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub pats: &'static [&'static str],
    pub tier: u8,
    pub path: Path,
    pub kind: Kind,
    pub get: Get,
    /// Why, where it isn't plain.
    pub note: &'static str,
}

use Get::{Drop, Made, Not, World};
use Kind::*;
use Needs::Hand;
use Path::{Beasts, Magic, Shared, Tech};

pub const AXE: Needs = Needs::Tool("axe", 0);
pub const PICK: Needs = Needs::Tool("pickaxe", 0);
pub const PICK_STONE: Needs = Needs::Tool("pickaxe", 100);
pub const PICK_IRON: Needs = Needs::Tool("pickaxe", 200);
pub const PICK_STEEL: Needs = Needs::Tool("pickaxe", 300);
pub const PICK_DIAMOND: Needs = Needs::Tool("pickaxe", 300);
pub const CUTTER: Needs = Needs::Tool("cutter", 0);

const fn r(pats: &'static [&'static str], tier: u8, path: Path, kind: Kind, get: Get) -> Rule {
    Rule { pats, tier, path, kind, get, note: "" }
}

const fn n(pats: &'static [&'static str], tier: u8, path: Path, kind: Kind, get: Get, note: &'static str) -> Rule {
    Rule { pats, tier, path, kind, get, note }
}

/// The rules, first match wins.
pub const RULES: &[Rule] = &[
    // ---- not items: what breaks into something else, liquids, stages, the world's own
    n(&["base:water", "base:lava"], 0, Shared, Technical, Not, "liquids, carried in buckets"),
    n(
        &["base:farmland", "base:grazed_grass", "base:*_top", "base:*_stem", "base:paddy_rice", "base:tombstone"],
        0,
        Shared,
        Technical,
        Not,
        "growing stages, tilled or grazed ground, the grave a death leaves",
    ),
    n(&["base:campfire"], 0, Shared, Technical, Not, "a laid campfire, lit"),
    n(
        &["base:loose_rock", "base:loose_flint", "base:loose_tektite", "base:fallen_branch"],
        0,
        Shared,
        Ground,
        Not,
        "picked up as a rock, flint, tektite or sticks",
    ),
    n(&["base:coal_ore", "base:banded_clay_coal_ore"], 0, Shared, Ground, Not, "breaks into coal"),
    n(
        &[
            "base:iron_ore",
            "base:granite_iron_ore",
            "base:red_sandstone_iron_ore",
            "base:meteorite",
            "base:gold_ore",
            "base:granite_gold_ore",
        ],
        1,
        Tech,
        Ground,
        Not,
        "breaks into raw iron, meteoric iron or raw gold (a stone pickaxe)",
    ),
    n(&["base:redstone_ore", "base:star_steel_ore"], 2, Tech, Ground, Not, "breaks into its ore (an iron pickaxe)"),
    n(&["base:diamond_ore"], 3, Tech, Ground, Not, "breaks into a diamond (a steel pickaxe: harvest 300)"),
    n(&["animals:amber_ore"], 0, Beasts, Ground, Not, "breaks into amber"),
    n(
        &["animals:gateway", "animals:termite_mound_upper"],
        0,
        Beasts,
        Technical,
        Not,
        "the Cretaceous gateway is built of amber blocks; a mound's top",
    ),
    n(&["portals:portal", "packages:unknown"], 0, Shared, Technical, Not, "a linked world's door; a missing package's stand-in"),
    n(&["base:copper_lamp"], 3, Tech, Machine, Not, "no recipe yet (no copper in the game)"),
    // ---- the ground
    r(
        &[
            "base:dirt",
            "base:grass",
            "base:snowy_grass",
            "base:sand",
            "base:gravel",
            "base:clay",
            "base:podzol",
            "base:coarse_dirt",
            "base:moss_block",
            "base:snow",
            "base:snow_layer",
            "base:leaf_litter",
            "base:deep_leaf_litter",
            "animals:red_sand",
            "base:impact_glass",
        ],
        0,
        Shared,
        Ground,
        World(Hand, 0.1),
    ),
    r(
        &[
            "base:cobblestone",
            "base:mossy_cobblestone",
            "base:granite",
            "base:slate",
            "base:limestone",
            "base:weathered_limestone",
            "base:sandstone",
            "base:red_sandstone",
            "base:ochre_sandstone",
            "base:cream_sandstone",
            "base:banded_clay",
            "base:suevite",
            "animals:red_sandstone",
            "base:ice",
            "base:lake_ice",
            "base:glacier_ice",
            "base:*coral_block",
            "base:coral_rubble",
        ],
        0,
        Shared,
        Ground,
        World(PICK, 0.15),
    ),
    n(&["base:obsidian"], 3, Shared, Ground, World(PICK_DIAMOND, 1.0), "a steel pickaxe or better (harvest 300)"),
    n(&["base:pallasite"], 1, Magic, Sky, World(PICK_STONE, 20.0), "a stony-iron meteorite's olivine, found where one fell"),
    n(&["base:eclipse_ash"], 0, Magic, Sky, World(Hand, 40.0), "falls in an eclipse"),
    // ---- wood
    r(&["base:*_log", "animals:*_log", "base:brazil_nut_buttress"], 0, Shared, Wood, World(AXE, 0.3)),
    r(&["base:*_branch"], 0, Shared, Wood, World(AXE, 0.2)),
    r(&["base:*_planks"], 0, Shared, Wood, Made),
    r(&["base:*_leaves", "animals:*_leaves", "base:*_sapling", "animals:*_sapling", "base:sapling"], 0, Shared, Wood, World(Hand, 0.1)),
    r(&["base:oak_bark"], 0, Shared, Material, Made),
    // ---- the first tools and fire (vitals::first)
    r(&["base:rock", "base:flint", "base:tektite", "base:stick", "base:plant_fibre", "base:sharp_flake"], 0, Shared, Material, Made),
    r(&["base:bow_drill", "base:hand_drill", "base:fire_bed", "base:campfire_laid"], 0, Shared, Station, Made),
    // the ley lines (ley.rs, `vitals::ley`): the magic path's first step, and the ancients' stone
    n(&["base:dowsing_rod"], 0, Magic, Tool, Made, "senses the ley lines: the Stone Age's magic route"),
    n(&["base:ley_stone"], 0, Magic, Building, World(PICK, 3.0), "the ancients' carved stone round the waystones, some 600 blocks apart"),
    n(&["base:waystone"], 0, Magic, Technical, Not, "a sleeping waystone on a great ley crossing: not carried off, woken later"),
    // glider.rs: leather on a frame of sticks (`vitals::glide::HIDE`).
    r(&["base:glider"], 0, Tech, Tool, Made),
    // ---- tools, weapons and armour: all built from parts (Ronald, 11-05)
    n(
        &["base:axe", "base:pickaxe", "base:shovel", "base:hoe", "base:knife", "base:spear", "base:sword", "base:bow"],
        0,
        Tech,
        Tool,
        Made,
        "built from parts: its parts' materials set its real age (`MATERIALS`)",
    ),
    n(
        &["base:helmet", "base:chestplate", "base:leggings", "base:boots"],
        0,
        Tech,
        Armour,
        Made,
        "built from a shell, fastening and lining: its shell sets its real age; a leather-look shell (leather, rawhide, bone)",
    ),
    // grapple.rs: wrought-iron flukes on a string cord (`vitals::grapple::IRON`).
    r(&["base:grappling_hook"], 2, Tech, Tool, Made),
    n(
        &["base:helmet_mail", "base:chestplate_mail", "base:leggings_mail", "base:boots_mail"],
        2,
        Tech,
        Armour,
        Made,
        "the same armour with an iron or steel shell, an item of its own only for its worn look (`parts::ARMOUR_LOOKS`)",
    ),
    n(
        &["base:helmet_scale", "base:chestplate_scale", "base:leggings_scale", "base:boots_scale"],
        1,
        Tech,
        Armour,
        Not,
        "a bronze or copper shell's worn look: no copper or bronze in the game yet (only `/gear`)",
    ),
    n(
        &["base:helmet_gem", "base:chestplate_gem", "base:leggings_gem", "base:boots_gem"],
        3,
        Tech,
        Armour,
        Made,
        "a diamond-scale shell's worn look",
    ),
    n(&["base:part_*"], 0, Tech, Part, Made, "a part's material sets its real age (`MATERIALS`)"),
    // Gems for gear sockets (`crafting::gems`), cut from their found stones, of their stone's age.
    n(&["base:gem_tektite", "base:gem_amber"], 0, Tech, Material, Made, "cut gems: a boost in a tool's, weapon's or armour's socket"),
    r(&["base:gem_peridot"], 2, Tech, Material, Made),
    r(&["base:gem_diamond"], 3, Tech, Material, Made),
    n(&["base:arrow"], 0, Tech, Tool, Drop(2.0), "dropped by skeletons; no recipe yet"),
    // ---- metals and gems
    r(&["base:coal", "base:coal_block"], 0, Shared, Material, Made),
    n(&["base:meteoric_iron"], 1, Tech, Material, Made, "hammered cold, iron-grade before a player can smelt (Research round 17)"),
    r(
        &["base:raw_iron", "base:iron_ingot", "base:iron_block", "base:bucket", "base:water_bucket", "base:lava_bucket"],
        2,
        Tech,
        Material,
        Made,
    ),
    n(
        &["base:raw_gold", "base:gold_ingot", "base:gold_block"],
        1,
        Tech,
        Material,
        Made,
        "cast and soft: the Bronze Age's (Ronald, 11-07: \"move them to fit\")",
    ),
    r(&["base:star_steel_lump", "base:star_steel_ingot"], 2, Tech, Material, Made),
    r(&["base:starheart", "base:peridot"], 2, Magic, Sky, Made),
    n(&["base:star_lantern"], 2, Magic, Light, Made, "a starheart in iron"),
    n(&["base:unlearning_stone"], 2, Magic, Material, Made, "a peridot ground with tektites: gives back perk web nodes (vitals::perks)"),
    n(
        &["base:maunduk", "base:aruduk", "base:remaru", "base:emakaun"],
        2,
        Magic,
        Material,
        Drop(60.0),
        "stones for the web's sockets (vitals::sockets), found, never made: in pallasite today; bosses, ley nodes, ruins and rare animals add theirs (an earlier tier then)",
    ),
    r(&["base:diamond", "base:diamond_block"], 3, Tech, Material, Made),
    // ---- smelting's things (the furnace: the Stone Age's last build, the Bronze Age's first station)
    n(&["base:furnace"], 0, Tech, Station, Made, "the Stone Age key's infrastructure: what it makes is the Bronze Age's"),
    r(&["base:glass", "base:*_stained_glass", "base:bricks", "base:stone", "base:stone_bricks"], 1, Tech, Building, Made),
    r(&["base:glowstone", "base:red_lamp", "base:green_lamp", "base:blue_lamp"], 1, Tech, Light, Made),
    // ---- machines (redstone: the Age of Steam's first machines)
    n(
        &[
            "base:redstone",
            "base:redstone_block",
            "base:redstone_torch",
            "base:repeater",
            "base:comparator",
            "base:lever",
            "base:stone_button",
            "base:redstone_lamp",
            "base:piston",
            "base:sticky_piston",
            "base:slime_block",
            "base:observer",
            "base:dispenser",
            "base:floater",
            "base:anchor",
            "base:golden_lamp",
        ],
        3,
        Tech,
        Machine,
        Made,
        "redstone ore needs an iron pickaxe; machines are the Age of Steam's",
    ),
    // ---- building and stations of the Stone Age
    r(
        &["base:crafting_table", "base:chest", "base:ladder", "base:bed", "base:bookshelf", "base:hay_bale", "base:wool"],
        0,
        Shared,
        Building,
        Made,
    ),
    r(&["base:torch"], 0, Shared, Light, Made),
    r(&["base:tanning_frame", "base:tanning_pit", "base:charcoal_clamp", "base:drying_rack", "base:compost_bin"], 0, Shared, Station, Made),
    n(&["base:cooking_pot", "base:stew"], 0, Shared, Station, Made, "a clay pot (Ronald, 11-07: \"move them to fit\")"),
    n(&["base:salt", "base:salted_meat"], 2, Shared, Food, Made, "salt is boiled from a water bucket (iron)"),
    // ---- materials of hunting, farming and sewing
    r(
        &[
            "base:hide",
            "base:fur",
            "base:bone",
            "base:feather",
            "base:egg",
            "base:gunpowder",
            "base:spider_eye",
            "base:rotten_flesh",
            "base:dung",
        ],
        0,
        Shared,
        Material,
        Drop(2.0),
    ),
    r(
        &[
            "base:string",
            "base:cloth",
            "base:leather",
            "base:sole_leather",
            "base:paper",
            "base:book",
            "base:sugar",
            "base:bowl",
            "base:compost",
            "base:bandage",
        ],
        0,
        Shared,
        Material,
        Made,
    ),
    r(&["base:fur_*", "base:hide_*", "base:wool_*", "base:linen_*", "base:rain_cape"], 0, Shared, Clothing, Made),
    // ---- food
    r(
        &["base:beef", "base:porkchop", "base:chicken", "base:mutton", "animals:venison", "animals:hare", "animals:trout", "animals:pacu"],
        0,
        Shared,
        Food,
        Drop(3.0),
    ),
    r(
        &[
            "base:cooked_*",
            "animals:cooked_*",
            "base:baked_potato",
            "base:roasted_corn",
            "base:roasted_cacao",
            "base:bread",
            "base:apple_pie",
            "base:dried_*",
            "base:smoked_meat",
            "base:cacao_beans",
            "base:pumpkin_seeds",
            "base:melon_seeds",
            "base:pumpkin_slice",
            "base:melon_slice",
        ],
        0,
        Shared,
        Food,
        Made,
    ),
    // ---- crops (sown, grown, picked; seeds from grass)
    r(
        &[
            "base:wheat",
            "base:wheat_seeds",
            "base:barley",
            "base:barley_seeds",
            "base:rye",
            "base:rye_seeds",
            "base:maize",
            "base:maize_seeds",
            "base:corn",
            "base:rice",
            "base:rice_seeds",
            "base:beans",
            "base:bean_seeds",
            "base:sorghum",
            "base:sorghum_seeds",
            "base:millet",
            "base:millet_seeds",
            "base:carrot",
            "base:carrots",
            "base:potato",
            "base:potatoes",
            "base:beetroot",
            "base:beetroots",
            "base:cassava",
            "base:pumpkin",
            "base:melon",
            "base:sugar_cane",
            "base:cactus",
            "base:tepary_beans",
            "base:chia",
            "base:chia_seeds",
            "base:wild_rice",
        ],
        0,
        Shared,
        Crop,
        World(Hand, 0.2),
    ),
    // ---- wild fruit, nuts and fungi
    r(
        &[
            "base:apple",
            "base:berries",
            "base:lingonberries",
            "base:cloudberries",
            "base:crowberries",
            "base:raspberries",
            "base:huckleberries",
            "base:beach_plums",
            "base:prickly_pear_fruit",
            "base:rose_hip",
            "base:hazelnut",
            "base:pine_nut",
            "base:sunflower_seeds",
            "base:acorn",
            "base:sequoia_cone",
            "base:mesquite_pod",
            "base:brazil_nut",
            "base:cacao_pod",
            "base:bananas",
            "base:andiroba_seed",
            "base:matamata_fruit",
            "base:ipe_seed",
            "base:acuri_nut",
            "base:moriche_fruit",
            "base:cattail_root",
            "base:kelp",
            "base:mushroom_brown",
            "base:mushroom_red",
            "animals:araucaria_cone",
            "animals:cycad_seed",
            "animals:ginkgo_seed",
        ],
        0,
        Shared,
        Food,
        World(Hand, 0.2),
    ),
    // ---- the animals mod: taming, riding, eggs, the gateway
    r(&["animals:*_egg", "animals:carcass"], 0, Beasts, Beast, Drop(4.0)),
    r(
        &[
            "animals:club",
            "animals:sedative",
            "animals:tranquilizer_dart",
            "animals:nest",
            "animals:utahraptor_saddle",
            "animals:amber",
            "animals:amber_block",
        ],
        0,
        Beasts,
        Beast,
        Made,
    ),
    // ---- the biome bosses (animals_core::bosses, drenkaun.rs): the altar and its offering made,
    // the boss's parts from the fight (a fight about 5 minutes, the offering's gathering and the
    // altar's, over the kills its bad-luck help needs)
    n(&["animals:drenkaun_altar"], 0, Shared, Station, Made, "a biome boss's altar: the player picks when to fight (round 6)"),
    n(&["animals:forest_offering"], 0, Shared, Material, Made, "the old forest's offering: calls Drenkaun, tames its calf"),
    n(
        &["animals:warden_heartseed"],
        0,
        Shared,
        Part,
        Drop(45.0),
        "Drenkaun's loot, certain by the third kill: the Stone Age key's boss part",
    ),
    n(&["animals:warden_antler", "animals:warden_bark"], 0, Shared, Material, Drop(15.0), "Drenkaun's part breaks and loot"),
    n(&["animals:kibble"], 0, Beasts, Beast, Made, "an egg, food, grain and fibre (no iron bucket; Ronald, 11-07: \"move them to fit\")"),
    n(&["animals:wyvern_saddle"], 2, Beasts, Beast, Made, "a dragon-sized saddle has an iron frame"),
    r(&["animals:spyglass"], 2, Shared, Tool, Made),
    r(&["animals:rotten_log", "animals:termite_mound"], 0, Beasts, Ground, World(AXE, 0.3)),
    // ---- wild plants: picked by hand, grass cut for fibre with a flake
    r(
        &[
            "base:dead_bush",
            "base:fern",
            "base:tall_grass",
            "base:tall_grass_double",
            "base:dune_grass",
            "base:sedge",
            "base:cottongrass",
            "base:reindeer_lichen",
            "base:clover",
            "base:honey_fungus",
            "base:chanterelle",
            "base:porcini",
            "base:bracket_fungus",
            "base:foxfire",
            "base:glow_mycena",
            "base:goblins_gold",
            "base:*_bush",
            "base:dwarf_birch",
            "base:prickly_pear",
            "base:wild_sunflower",
            "base:water_hyacinth",
            "base:cattail",
            "base:reed",
            "base:lily_pad",
            "base:seagrass",
            "base:kelp",
            "base:staghorn_coral",
            "base:elkhorn_coral",
            "base:sea_fan",
            "base:finger_coral",
            "base:sea_rod",
            "base:turtle_grass",
            "base:tepary_beans",
            "base:bromeliad",
            "base:banana",
            "base:philodendron",
            "base:heliconia",
            "base:liana",
            "base:sword_fern",
            "base:lupine",
            "base:dandelion",
            "base:rose",
            "base:purple_coneflower",
            "base:black_eyed_susan",
            "base:butterfly_milkweed",
            "base:leadplant",
            "base:blazing_star",
            "base:wood_anemone",
            "base:lesser_celandine",
            "base:lungwort",
            "base:sweet_woodruff",
            "base:foxglove",
            "base:lily_of_the_valley",
            "base:wood_cranesbill",
            "base:may_lily",
            "base:bellflower",
            "base:fireweed",
            "base:twinflower",
            "base:bunchberry",
            "base:prickly_rose",
            "base:labrador_tea",
            "base:arctic_poppy",
            "base:lousewort",
            "base:saxifrage",
            "base:arctic_coltsfoot",
            "base:pygmy_buttercup",
            "base:brittlebush",
            "base:globemallow",
            "base:gold_poppy",
            "base:desert_marigold",
            "base:ocotillo",
            "base:sensitive_plant",
            "base:senna",
            "base:pencil_flower",
            "base:morning_glory",
            "base:water_primrose",
            "base:alligator_flag",
            "base:alpenrose",
            "base:gentian",
            "base:edelweiss",
            "base:globeflower",
            "base:alpine_aster",
            "base:sea_daffodil",
            "base:sea_holly",
            "base:horned_poppy",
            "base:sea_rocket",
            "base:hottentot_fig",
            "base:dung_pat",
            "animals:cycad",
            "animals:fern",
            "animals:flowering_shrub",
            "animals:horsetail",
            "animals:water_lily",
        ],
        0,
        Shared,
        Plant,
        World(Hand, 0.1),
    ),
];

/// Whether `id` matches `pat`, where `*` matches any run (also an empty one).
pub fn matches(pat: &str, id: &str) -> bool {
    let parts: Vec<&str> = pat.split('*').collect();
    if parts.len() == 1 {
        return pat == id;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !id.starts_with(first) || !id[first.len()..].ends_with(last) || id.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &id[first.len()..id.len() - last.len()];
    for mid in &parts[1..parts.len() - 1] {
        match rest.find(mid) {
            Some(i) => rest = &rest[i + mid.len()..],
            None => return false,
        }
    }
    true
}

/// The rule that tags `id`.
pub fn rule(id: &str) -> Option<&'static Rule> {
    RULES.iter().find(|r| r.pats.iter().any(|p| matches(p, id)))
}

// ------------------------------------------------------------------ the universe

/// The mods' language files: every item and block with a name is in one.
const LANG: [(&str, &str); 4] = [
    ("base", include_str!("../../assets/lang/en_us.lang")),
    ("animals", include_str!("../../../animals/assets/lang/en_us.lang")),
    ("portals", include_str!("../../../portals/assets/lang/en_us.lang")),
    ("packages", include_str!("../../../packages/assets/lang/en_us.lang")),
];

/// Every named item and block, `mod:name`, sorted.
pub fn universe() -> Vec<String> {
    let mut ids = Vec::new();
    for (ns, text) in LANG {
        for line in text.lines() {
            let Some((key, _)) = line.split_once('=') else { continue };
            let key = key.trim();
            for kind in ["item.", "block."] {
                if let Some(rest) = key
                    .strip_prefix(kind)
                    .and_then(|k| k.strip_prefix(ns))
                    .and_then(|k| k.strip_prefix('.'))
                    && !rest.is_empty()
                    && rest
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                {
                    ids.push(format!("{ns}:{rest}"));
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// An id's tier and path, if tagged.
pub fn tier(id: &str) -> Option<u8> {
    rule(id).map(|r| r.tier)
}

// ------------------------------------------------------------------ tool parts' materials

/// A crafting material's path and the item a part of it is made from (`parts.rs`'s `SOURCES`;
/// `""` where there is none in the game yet). Its age is `vitals::gear::MATERIALS`'.
#[derive(Clone, Copy, Debug)]
pub struct MaterialTag {
    pub id: &'static str,
    pub path: Path,
    pub from: &'static str,
}

impl MaterialTag {
    /// Its tier, from `vitals::gear` (none: no age there).
    pub fn tier(&self) -> Option<u8> {
        vitals::gear::material(self.id).map(|m| m.tier)
    }
}

const fn mt(id: &'static str, from: &'static str) -> MaterialTag {
    MaterialTag { id, path: Tech, from }
}

/// Every row of `crafting::MATERIALS` (a check fails on one without a tag).
pub const MATERIALS: &[MaterialTag] = &[
    mt("flint", "base:flint"),
    mt("chert", ""),
    mt("tektite", "base:tektite"),
    mt("obsidian", ""),
    mt("bone", "base:bone"),
    mt("antler", ""),
    mt("stone", "base:cobblestone"),
    mt("diamond", "base:diamond"),
    mt("copper", ""),
    mt("bronze", ""),
    mt("wrought_iron", "base:iron_ingot"),
    mt("meteoric_iron", "base:meteoric_iron"),
    mt("steel", ""),
    mt("star_steel", "base:star_steel_ingot"),
    mt("gold", "base:gold_ingot"),
    mt("oak", "base:oak_planks"),
    mt("mesquite", "base:mesquite_planks"),
    mt("birch", "base:birch_planks"),
    mt("yew", ""),
    mt("elm", ""),
    mt("osage_orange", ""),
    mt("spruce", "base:spruce_planks"),
    mt("branch", "base:stick"),
    mt("plant_cordage", "base:string"),
    mt("sinew", ""),
    mt("rawhide", ""),
    mt("birch_tar", ""),
    mt("leather", "base:sole_leather"),
    mt("linen", "base:cloth"),
    mt("wool", "base:wool"),
    mt("fur", "base:fur"),
];

pub fn material(id: &str) -> Option<&'static MaterialTag> {
    MATERIALS.iter().find(|m| m.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns() {
        assert!(matches("base:*_log", "base:oak_log"));
        assert!(!matches("base:*_log", "base:oak_logs"));
        assert!(matches("base:*coral_block", "base:brain_coral_block"));
        assert!(matches("base:part_*", "base:part_haft"));
        assert!(matches("a*b*c", "aXbYc"));
        assert!(!matches("a*b*c", "ac"));
        assert!(universe().iter().any(|i| i == "base:stick"));
    }
}

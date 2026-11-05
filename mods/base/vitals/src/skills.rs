//! Skills that rise by doing (`notes/PROGRESSION.md`, "Skills and perks (like Skyrim)"): mining
//! raises Mining, hitting raises Melee, taming raises Taming. A skill runs 0-100. Its experience is
//! priced in effort minutes (`curve::Em`), the balance model's one currency: an action gives the
//! minutes an average player spends on it, times how its tier stands against the skill
//! (`against`), never a flat amount a piece. Doing work below your skill teaches less and less, so
//! a skill climbs with the ages (Skyrim's iron daggers fixed, `reports/Progression systems.md`).
//!
//! Each skill-up feeds the player's level (`up_xp`, Skyrim's: the new skill level in XP, times
//! `UP_XP`), and each level gives a perk point for the web (`web::POINTS_PER_LEVEL`). Skills are
//! always the player's own, even in a tribe.

use crate::curve::{AGES, Age, Em, LEVELS_PER_AGE, Path};

/// The groups of the web's regions (`notes/PROGRESSION.md`, the regions table).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Fighting,
    Magic,
    Crafting,
    Beasts,
    /// Planner's proposed trees (mining, engineering, exploring, trading, survival).
    Life,
}

impl Group {
    pub const fn name(self) -> &'static str {
        match self {
            Group::Fighting => "Fighting",
            Group::Magic => "Magic",
            Group::Crafting => "Crafting",
            Group::Beasts => "Beasts",
            Group::Life => "Life",
        }
    }
    pub const fn id(self) -> &'static str {
        match self {
            Group::Fighting => "fighting",
            Group::Magic => "magic",
            Group::Crafting => "crafting",
            Group::Beasts => "beasts",
            Group::Life => "life",
        }
    }
}

/// A skill. The order is the web's, clockwise from the top: neighbours share a bridge, so related
/// skills sit side by side (Defence beside Smithing for armour, Conjuring beside Taming for rune
/// collars). Its index is saved (`Skills::em`): new skills go at the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Skill {
    Melee,
    Archery,
    Defence,
    Smithing,
    Engineering,
    Mining,
    Building,
    Alchemy,
    Runecraft,
    Destruction,
    Restoration,
    Conjuring,
    Taming,
    Survival,
    Exploring,
    Trading,
}

pub const COUNT: usize = 16;

pub const SKILLS: [Skill; COUNT] = [
    Skill::Melee,
    Skill::Archery,
    Skill::Defence,
    Skill::Smithing,
    Skill::Engineering,
    Skill::Mining,
    Skill::Building,
    Skill::Alchemy,
    Skill::Runecraft,
    Skill::Destruction,
    Skill::Restoration,
    Skill::Conjuring,
    Skill::Taming,
    Skill::Survival,
    Skill::Exploring,
    Skill::Trading,
];

impl Skill {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn id(self) -> &'static str {
        match self {
            Skill::Melee => "melee",
            Skill::Archery => "archery",
            Skill::Defence => "defence",
            Skill::Smithing => "smithing",
            Skill::Engineering => "engineering",
            Skill::Mining => "mining",
            Skill::Building => "building",
            Skill::Alchemy => "alchemy",
            Skill::Runecraft => "runecraft",
            Skill::Destruction => "destruction",
            Skill::Restoration => "restoration",
            Skill::Conjuring => "conjuring",
            Skill::Taming => "taming",
            Skill::Survival => "survival",
            Skill::Exploring => "exploring",
            Skill::Trading => "trading",
        }
    }

    /// Its name for the sheets; the game's is the translation key `skill.base.<id>`.
    pub const fn name(self) -> &'static str {
        match self {
            Skill::Melee => "Melee",
            Skill::Archery => "Archery and throwing",
            Skill::Defence => "Defence",
            Skill::Smithing => "Smithing and tinkering",
            Skill::Engineering => "Engineering",
            Skill::Mining => "Mining and prospecting",
            Skill::Building => "Building and farming",
            Skill::Alchemy => "Alchemy and cooking",
            Skill::Runecraft => "Runecraft and enchanting",
            Skill::Destruction => "Destruction",
            Skill::Restoration => "Restoration",
            Skill::Conjuring => "Conjuring and wards",
            Skill::Taming => "Taming and riding",
            Skill::Survival => "Survival",
            Skill::Exploring => "Exploring and cartography",
            Skill::Trading => "Trading",
        }
    }

    pub const fn group(self) -> Group {
        match self {
            Skill::Melee | Skill::Archery | Skill::Defence => Group::Fighting,
            Skill::Destruction | Skill::Restoration | Skill::Conjuring => Group::Magic,
            Skill::Smithing | Skill::Alchemy | Skill::Runecraft | Skill::Building => Group::Crafting,
            Skill::Taming => Group::Beasts,
            Skill::Mining | Skill::Engineering | Skill::Exploring | Skill::Trading | Skill::Survival => Group::Life,
        }
    }

    /// The braid's path it leans on (`curve::Path`).
    pub const fn path(self) -> Path {
        match self {
            Skill::Destruction | Skill::Restoration | Skill::Conjuring | Skill::Runecraft => Path::Magic,
            Skill::Taming => Path::Beasts,
            Skill::Smithing | Skill::Engineering | Skill::Mining => Path::Tech,
            _ => Path::Shared,
        }
    }

    /// What raises it, for the sheet and the tooltip; empty while nothing in the game does yet
    /// (the magic schools, Engineering and Trading wait for spells, machines and traders).
    pub const fn by(self) -> &'static str {
        match self {
            Skill::Melee => "hitting creatures in melee",
            Skill::Archery => "hitting with arrows and thrown things",
            Skill::Defence => "taking hits",
            Skill::Smithing => "making tools, weapons, armour and their parts; smelting",
            Skill::Mining => "breaking stone and ore",
            Skill::Building => "placing blocks; sowing and harvesting crops",
            Skill::Alchemy => "cooking",
            Skill::Taming => "taming and riding animals",
            Skill::Survival => "chopping trees; making the other things one needs",
            Skill::Exploring => "finding new biomes and dimensions",
            Skill::Engineering | Skill::Runecraft | Skill::Destruction | Skill::Restoration | Skill::Conjuring | Skill::Trading => "",
        }
    }

    pub fn of_id(id: &str) -> Option<Skill> {
        SKILLS.into_iter().find(|s| s.id() == id)
    }
}

// ------------------------------------------------------------------ levels

/// The top skill level (Skyrim's 100).
pub const MAX: u32 = 100;
/// Skill levels an age spans: 0-14 is the Stone Age's, 86-100 the Age of Stars'.
pub const PER_AGE: f32 = MAX as f32 / AGES.len() as f32;
/// The share of an age's hours one skill gets when a player leans on it: a main skill climbs an
/// age's worth of levels in a quarter of that age's hours (90 effort minutes in the Stone Age,
/// 26½ hours from 0 to 100 in all).
pub const SHARE: f32 = 0.25;

/// The skill level that matches content of item or area level `level` (`curve::item_level`):
/// ten item levels an age, `PER_AGE` skill levels an age.
pub fn for_level(level: f32) -> f32 {
    level.max(0.0) / LEVELS_PER_AGE * PER_AGE
}

/// The skill level of an age's middle (its reference content).
pub fn for_age(age: Age) -> f32 {
    for_level(age.reference())
}

/// Effort minutes from 0 to skill level `s` (fractional levels interpolate): `SHARE` of the
/// ages' minutes up to there, so each age's levels cost what that age's hours allow.
pub fn em_to(s: f32) -> Em {
    let s = s.clamp(0.0, MAX as f32);
    let mut em = 0.0;
    for (i, age) in AGES.iter().enumerate() {
        let from = i as f32 * PER_AGE;
        if s <= from {
            break;
        }
        let part = ((s - from) / PER_AGE).min(1.0);
        em += age.em() * SHARE * part;
    }
    em
}

/// The skill level `em` effort minutes reach, whole levels, and the share of the way into the
/// next (0..1).
pub fn level_of(em: Em) -> (u32, f32) {
    let mut level = 0;
    while level < MAX && em_to(level as f32 + 1.0) <= em + 1e-4 {
        level += 1;
    }
    if level >= MAX {
        return (MAX, 0.0);
    }
    let (a, b) = (em_to(level as f32), em_to(level as f32 + 1.0));
    (level, ((em - a) / (b - a)).clamp(0.0, 1.0))
}

/// How much doing work of skill level `work` teaches at skill `skill`: all of it at or above the
/// skill, half every half an age below. One age below, a quarter; two ages below, a sixteenth. So
/// a Stone Age smith stops learning from flint near the age's end and needs bronze.
pub fn against(work: f32, skill: f32) -> f32 {
    if work >= skill { 1.0 } else { 0.5f32.powf((skill - work) / HALVING) }
}
/// Skill levels below one's skill that halve what work teaches.
pub const HALVING: f32 = PER_AGE / 2.0;

/// Player XP for a skill-up to `level` (Skyrim's: the new level, here times `UP_XP`).
pub fn up_xp(level: u32) -> u64 {
    level as u64 * UP_XP
}
/// The XP a skill-up gives per level of the skill reached: with four main skills near 70 and the
/// rest near 30 by the Age of Stars, skill-ups give about half of `levels::total_to(100)`; the
/// other half stays with kills, discovery and advancements (`levels::xp`).
pub const UP_XP: u64 = 8;

// ------------------------------------------------------------------ what doing pays

/// What each kind of doing is worth in effort minutes (`curve::Em`), before `against`. Each is the
/// time an average player spends on one, travel and search included, so a skill's levels cost the
/// hours the age gives it.
pub mod gain {
    use crate::combat::{Player, REF_RATE};
    use crate::curve::{Age, Em};

    /// Mining or chopping a block: a second of breaking per `HARDNESS_SECS` of hardness, times
    /// the walking and sorting around it (`MINE_OVERHEAD`). Stone (1.5): one second.
    pub fn mine(hardness: f32) -> Em {
        hardness.max(0.0) * HARDNESS_SECS * MINE_OVERHEAD / 60.0
    }
    pub const HARDNESS_SECS: f32 = 1.0 / 3.0;
    pub const MINE_OVERHEAD: f32 = 2.0;
    /// An ore is worth finding: seconds of prospecting each, on top of breaking it.
    pub const ORE_FIND_SECS: f32 = 6.0;
    pub fn ore(hardness: f32) -> Em {
        mine(hardness) + ORE_FIND_SECS / 60.0
    }

    /// A hit for `damage` on a creature, by an `age` fighter: the seconds the age's reference
    /// player takes to deal it (`combat::Player::hit` at `REF_RATE`), times the stalking and
    /// dodging around it (`FIGHT_OVERHEAD`). A reference hit: three seconds.
    pub fn hit(damage: f32, age: Age) -> Em {
        let per_sec = Player::of(age).hit() * REF_RATE;
        damage.max(0.0) / per_sec * FIGHT_OVERHEAD / 60.0
    }
    pub const FIGHT_OVERHEAD: f32 = 3.0;

    /// A hit taken for `damage` (Defence, as Skyrim's armour skills): what blocking and bracing
    /// for it costs, a reference hit two seconds.
    pub fn taken(damage: f32, age: Age) -> Em {
        hit(damage, age) * TAKEN_SHARE
    }
    pub const TAKEN_SHARE: f32 = 2.0 / 3.0;

    /// Making something with `inputs` things in it: the seconds of gathering and crafting each
    /// input stands for (`CRAFT_SECS_PER_INPUT`), so a sword teaches more than a stick.
    pub fn craft(inputs: u32) -> Em {
        inputs.max(1) as f32 * CRAFT_SECS_PER_INPUT / 60.0
    }
    pub const CRAFT_SECS_PER_INPUT: f32 = 12.0;

    /// A smelted piece: the furnace's work is the furnace's, the loading and fuel the smith's.
    pub const SMELT: Em = 4.0 / 60.0;

    /// A cooked food (`inputs` things in it; a stew several).
    pub fn cook(inputs: u32) -> Em {
        inputs.max(1) as f32 * COOK_SECS_PER_INPUT / 60.0
    }
    pub const COOK_SECS_PER_INPUT: f32 = 20.0;

    /// A block placed: picking the spot and the block.
    pub const PLACE: Em = 1.5 / 60.0;
    /// A seed sown and a ripe crop harvested (its tending is the growing's).
    pub const SOW: Em = 3.0 / 60.0;
    pub const HARVEST: Em = 6.0 / 60.0;

    /// Taming an animal of a size class's tame time: its whole time, taming done.
    pub fn tame(minutes: f32) -> Em {
        minutes.max(0.0)
    }
    /// A tame's taming time by its size class (`combat::Size`), when the taming code gives none:
    /// about 3-6 % of an age's effort minutes for a large one at the Stone Age (`balance.rs`).
    pub const TAME_MINUTES: [f32; 5] = [2.0, 5.0, 12.0, 20.0, 30.0];
    /// Riding: a second in the saddle, moving.
    pub const RIDE_SEC: Em = 1.0 / 60.0;

    /// A biome found: the walk to it.
    pub const DISCOVER: Em = 4.0;
    /// A dimension found.
    pub const DISCOVER_DIMENSION: Em = 20.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_the_ages() {
        assert_eq!(em_to(0.0), 0.0);
        // The Stone Age's levels cost a quarter of its 360 minutes.
        assert!((em_to(PER_AGE) - 90.0).abs() < 1e-3);
        // 0-100: a quarter of all 106 hours.
        assert!((em_to(100.0) - 0.25 * 106.0 * 60.0).abs() < 1e-2);
        assert_eq!(level_of(0.0), (0, 0.0));
        assert_eq!(level_of(em_to(37.0)).0, 37);
        assert_eq!(level_of(1e9).0, MAX);
        for s in 0..MAX {
            assert!(em_to(s as f32 + 1.0) > em_to(s as f32));
        }
    }

    #[test]
    fn low_work_teaches_less() {
        assert_eq!(against(30.0, 20.0), 1.0);
        assert!((against(20.0 - PER_AGE, 20.0) - 0.25).abs() < 1e-5);
        assert!(against(0.0, 90.0) < 1e-3);
    }

    #[test]
    fn every_skill_has_a_place() {
        for (i, s) in SKILLS.iter().enumerate() {
            assert_eq!(s.index(), i);
            assert_eq!(Skill::of_id(s.id()), Some(*s));
        }
    }
}

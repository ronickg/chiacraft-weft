//! Stones for the web's sockets (Ronald, 11-05: "insert items which generate boosts and you find
//! those items in the world, relics or whatever. It's a bit like how Path of Exile 2 has that,
//! where you have certain slots where you can insert them"). The ancients' work, so their names
//! are Old Eskenai (`mods/base/lore/lexicon.toml`):
//! - **Maunduk** ("mind-stone"): 1-3 boosts to families, rolled inside its item level's band;
//! - **Aruduk** ("flow-stone"): works on the small nodes taken within `RADIUS` of its socket,
//!   amplifying them or turning them to another family (Path of Exile's radius jewels);
//! - **unique relics**, each with an effect of its own (`Unique`).
//!
//! A stone is found, never made: ruins, boss loot, ley nodes, rare animals, meteorites
//! (`SOURCES`, the odds of each kind there). Its item level is the place's (`curve::item_level`:
//! tier × 10 + depth), and it's priced like the web's nodes, in units (`web::UNIT`): a mind-stone
//! is worth a notable at the Stone Age (3 units) and grows with the ages to `STONE_MAX` (5), so a
//! deeper find is always a little better and never past a family's ceiling. Stones come out and go
//! in freely: the web's checks (`perks::problems`, `dominance`) assume the best stone for the job in
//! every socket taken.

use crate::curve::MAX_LEVEL;
use crate::web::{FAMILIES, Family};

/// Units a mind-stone holds at item level 0 and at the top (`MAX_LEVEL`).
pub const STONE_MIN: f32 = 3.0;
pub const STONE_MAX: f32 = 5.0;
/// The least share of its band a roll takes (a poor roll is 60 % of a perfect one).
pub const ROLL_FLOOR: f32 = 0.6;
/// How far a flow-stone or a relic reaches from its socket, in GUI pixels of the web (1.6 rings).
pub const RADIUS: f32 = 48.0;
/// A flow-stone's amplifying: the share it adds to each small node in reach, at level 0 and the
/// top (+25 % to +50 % of their units).
pub const AMPLIFY_MIN: f32 = 0.25;
pub const AMPLIFY_MAX: f32 = 0.5;
/// Emakaun's health per node taken in reach, units.
pub const LIFEKEEP_PER_NODE: f32 = 0.5;
/// Remaru's mana-cost units beside its ley sight.
pub const REMARU_UNITS: f32 = 2.0;

/// A stone's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Mind,
    Flow,
    Unique(Unique),
}

/// The unique relics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unique {
    /// "Flow-watcher": shows ley lines, and eases mana.
    Remaru,
    /// "Life-keeper": health for every node taken near its socket.
    Emakaun,
}

pub const UNIQUES: [Unique; 2] = [Unique::Remaru, Unique::Emakaun];

impl Unique {
    pub const fn id(self) -> &'static str {
        match self {
            Unique::Remaru => "remaru",
            Unique::Emakaun => "emakaun",
        }
    }
}

impl Kind {
    /// Its item's id (`base:<id>`).
    pub const fn id(self) -> &'static str {
        match self {
            Kind::Mind => "maunduk",
            Kind::Flow => "aruduk",
            Kind::Unique(u) => u.id(),
        }
    }
    /// The saved code: 1 mind, 2 flow, 10 + a unique's index (0: no stone).
    pub const fn code(self) -> u8 {
        match self {
            Kind::Mind => 1,
            Kind::Flow => 2,
            Kind::Unique(Unique::Remaru) => 10,
            Kind::Unique(Unique::Emakaun) => 11,
        }
    }
    pub fn of_code(c: u8) -> Option<Kind> {
        match c {
            1 => Some(Kind::Mind),
            2 => Some(Kind::Flow),
            10 => Some(Kind::Unique(Unique::Remaru)),
            11 => Some(Kind::Unique(Unique::Emakaun)),
            _ => None,
        }
    }
}

pub const KINDS: [Kind; 4] = [Kind::Mind, Kind::Flow, Kind::Unique(Unique::Remaru), Kind::Unique(Unique::Emakaun)];

/// What a flow-stone does to the small nodes taken within its reach.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flow {
    /// Each gives this share more of its own units.
    Amplify(f32),
    /// Each gives this family instead of its own.
    Turn(Family),
}

/// A rolled stone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stone {
    pub kind: Kind,
    /// Its item level (0 to `curve::MAX_LEVEL`).
    pub level: u8,
    /// Up to three boosts: a family and its units (a mind-stone's).
    pub rolls: [Option<(Family, f32)>; 3],
    /// A flow-stone's work.
    pub flow: Option<Flow>,
}

/// The units a mind-stone holds at `level`: a notable's at the Stone Age, growing to `STONE_MAX`.
pub fn band(level: f32) -> f32 {
    STONE_MIN + (STONE_MAX - STONE_MIN) * (level / MAX_LEVEL).clamp(0.0, 1.0)
}

/// A flow-stone's amplifying share at `level`.
pub fn amplify(level: f32) -> f32 {
    AMPLIFY_MIN + (AMPLIFY_MAX - AMPLIFY_MIN) * (level / MAX_LEVEL).clamp(0.0, 1.0)
}

/// Rolls a stone of `kind` at item `level` from `rand`, a source of numbers in 0..1. A mind-stone
/// takes 1-3 boosts (`roll_among` the web's families); a flow-stone amplifies (two in three) or
/// turns small nodes to one family.
pub fn roll(kind: Kind, level: f32, mut rand: impl FnMut() -> f32) -> Stone {
    let level = level.clamp(0.0, MAX_LEVEL);
    let mut stone = Stone { kind, level: level.round() as u8, rolls: [None; 3], flow: None };
    let pick = |u: f32| FAMILIES[((u * FAMILIES.len() as f32) as usize).min(FAMILIES.len() - 1)];
    match kind {
        Kind::Mind => stone.rolls = roll_among(&FAMILIES, level, &mut rand),
        Kind::Flow => {
            stone.flow = Some(if rand() < 2.0 / 3.0 { Flow::Amplify(amplify(level)) } else { Flow::Turn(pick(rand())) });
        }
        Kind::Unique(_) => {}
    }
    stone
}

/// What gems for gear can boost (`crafting`'s gem sockets, Ronald 11-05: "pieces and items should
/// also get like gem slots"): only what a tool, weapon or piece of armour carries itself.
pub const GEAR_FAMILIES: [Family; 6] =
    [Family::MeleeDamage, Family::RangedDamage, Family::Defence, Family::Health, Family::MineSpeed, Family::GatherSpeed];

/// A mind-stone's boosts, picked from `families` (a gem's from `GEAR_FAMILIES`): 1-3 of them (one
/// most often), the band at `level` split among them, each rolled between `ROLL_FLOOR` and all of
/// its share, in half units; no family twice.
pub fn roll_among(families: &[Family], level: f32, mut rand: impl FnMut() -> f32) -> [Option<(Family, f32)>; 3] {
    let mut rolls: [Option<(Family, f32)>; 3] = [None; 3];
    if families.is_empty() {
        return rolls;
    }
    let level = level.clamp(0.0, MAX_LEVEL);
    let pick = |u: f32| families[((u * families.len() as f32) as usize).min(families.len() - 1)];
    let n = match rand() {
        u if u < 0.6 => 1,
        u if u < 0.9 => 2,
        _ => 3,
    }
    .min(families.len());
    let share = band(level) / n as f32;
    for i in 0..n {
        let mut f = pick(rand());
        // No family twice: the next one round.
        while rolls.iter().flatten().any(|r| r.0 == f) {
            let at = families.iter().position(|x| *x == f).unwrap_or(0);
            f = families[(at + 1) % families.len()];
        }
        let units = (share * (ROLL_FLOOR + (1.0 - ROLL_FLOOR) * rand()) * 2.0).round() / 2.0;
        rolls[i] = Some((f, units.max(0.5)));
    }
    rolls
}

/// The best stone a socket can hold for a family: a mind-stone at the top level, one perfect roll.
pub fn best_units() -> f32 {
    band(MAX_LEVEL)
}

// ------------------------------------------------------------------ where they're found

/// Where stones are found, and the odds of each kind there (mind, flow, unique; summing to 1),
/// with the item level's depth inside the place's tier (`curve::item_level`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// Ruins of the ancients: their vaults (the Exploring work).
    Ruin,
    /// A biome boss's loot (Drenkaun's table and the others').
    Boss,
    /// A ley node's heart (the Ley lines work).
    LeyNode,
    /// A rare animal (a high-level or rare-coloured one).
    RareAnimal,
    /// A fallen star: pallasite.
    Meteorite,
}

pub struct Found {
    pub source: Source,
    pub odds: [f32; 3],
    /// The depth of its item level inside the tier (`curve::item_level`).
    pub depth: u8,
    /// The chance one drops each time (a boss's kill, a vault opened, a node's heart, a rare
    /// animal's death, a pallasite broken).
    pub chance: f32,
}

pub const SOURCES: [Found; 5] = [
    Found { source: Source::Ruin, odds: [0.6, 0.3, 0.1], depth: 6, chance: 0.5 },
    Found { source: Source::Boss, odds: [0.5, 0.35, 0.15], depth: 7, chance: 1.0 },
    Found { source: Source::LeyNode, odds: [0.4, 0.5, 0.1], depth: 5, chance: 0.25 },
    Found { source: Source::RareAnimal, odds: [0.85, 0.15, 0.0], depth: 4, chance: 0.2 },
    Found { source: Source::Meteorite, odds: [0.7, 0.25, 0.05], depth: 8, chance: 0.125 },
];

impl Source {
    pub const fn id(self) -> &'static str {
        match self {
            Source::Ruin => "ruin",
            Source::Boss => "boss",
            Source::LeyNode => "ley_node",
            Source::RareAnimal => "rare_animal",
            Source::Meteorite => "meteorite",
        }
    }
    pub fn found(self) -> &'static Found {
        SOURCES.iter().find(|f| f.source == self).unwrap()
    }
}

/// The kind a find at `source` is, from `u` in 0..1 (a unique: each as likely).
pub fn kind_at(source: Source, u: f32, v: f32) -> Kind {
    let o = source.found().odds;
    if u < o[0] {
        Kind::Mind
    } else if u < o[0] + o[1] {
        Kind::Flow
    } else {
        Kind::Unique(UNIQUES[((v * UNIQUES.len() as f32) as usize).min(UNIQUES.len() - 1)])
    }
}

/// A find at `source` in tier `tier` (0 Stone Age .. 6), or none by its chance: the stone, rolled.
pub fn find(source: Source, tier: u8, mut rand: impl FnMut() -> f32) -> Option<Stone> {
    let f = source.found();
    if rand() >= f.chance {
        return None;
    }
    let (u, v) = (rand(), rand());
    let kind = kind_at(source, u, v);
    Some(roll(kind, crate::curve::item_level(tier, f.depth), rand))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_stay_in_their_band() {
        let mut seed = 1u64;
        let mut rand = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 40) as f32 / (1u64 << 24) as f32
        };
        for level in [0.0, 25.0, MAX_LEVEL] {
            for _ in 0..200 {
                let s = roll(Kind::Mind, level, &mut rand);
                let total: f32 = s.rolls.iter().flatten().map(|r| r.1).sum();
                assert!(total <= band(level) + 0.5 && total >= 0.5, "{total} at {level}");
            }
        }
        for s in SOURCES {
            assert!((s.odds.iter().sum::<f32>() - 1.0).abs() < 1e-5, "{:?}", s.source);
        }
        for k in KINDS {
            assert_eq!(Kind::of_code(k.code()), Some(k));
        }
    }

    #[test]
    fn gems_roll_gear_families() {
        let mut seed = 7u64;
        let mut rand = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 40) as f32 / (1u64 << 24) as f32
        };
        for _ in 0..300 {
            let r = roll_among(&GEAR_FAMILIES, 30.0, &mut rand);
            assert!(r.iter().flatten().all(|(f, _)| GEAR_FAMILIES.contains(f)));
            let total: f32 = r.iter().flatten().map(|x| x.1).sum();
            assert!(total <= band(30.0) + 0.5 && total >= 0.5);
        }
    }
}

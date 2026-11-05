//! The first hours (`notes/FIRST_HOURS.md`; Ronald, 11-05: "you can't just punch down a tree. You
//! first got to get a rock"): what lies loose on the ground, striking flakes and knapping an axe
//! head from flint, cutting grass for fibre, and lashing the first stone axe by hand. Base's
//! `first.rs` wires them; worldgen places the ground's things by `ground_chance`.
//!
//! Real basis (`reports/The first hours.md`): river cobbles and scree are every knapper's
//! hammerstones; flint weathers out of chalk and gravel; a flake is struck in seconds, a
//! hand-knapped axe head takes a skilled knapper tens of minutes (here a few strikes); deadfall,
//! not cut wood, is what was burned and hafted first; grass and bast twist into cord.

/// Things that lie loose on the ground, picked up by hand at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// A cobble of the local rock: a hammerstone.
    Rock,
    /// A flint nodule: the core flakes and heads are knapped from.
    Flint,
    /// A fallen branch: sticks.
    Branch,
}

impl Ground {
    /// Its block in base (`base:loose_rock` ...).
    pub fn block(self) -> &'static str {
        match self {
            Ground::Rock => "loose_rock",
            Ground::Flint => "loose_flint",
            Ground::Branch => "fallen_branch",
        }
    }
}

/// The share of open ground columns (soil, sand, gravel) with a loose rock, a flint nodule, or a
/// fallen branch (under a crown; `BRANCH_OPEN` in the open). Tuned so a new player finds a rock
/// in a minute's walk and flint in two (`notes/FIRST_HOURS.md`, the timing).
pub const ROCK: f32 = 0.0025;
pub const FLINT: f32 = 0.0008;
pub const BRANCH: f32 = 0.02;
pub const BRANCH_OPEN: f32 = 0.001;
/// Steep ground and shores (scree, river banks, beaches) have this many times the rocks and flint.
pub const STONY: f32 = 3.0;
/// Within this many blocks of the world's spawn the ground has `NEAR_SPAWN_TIMES` as much of
/// everything, so the first minutes always have it at hand (`spawn_has_enough`).
pub const NEAR_SPAWN: i32 = 32;
pub const NEAR_SPAWN_TIMES: f32 = 4.0;

/// What the spawn has within `SPAWN_LOOK` blocks at the least, on average (the check): rocks,
/// flint nodules, fallen branches.
pub const SPAWN_LOOK: i32 = 24;
pub const SPAWN_LEAST: [(Ground, f32); 3] = [(Ground::Rock, 4.0), (Ground::Flint, 2.0), (Ground::Branch, 3.0)];
/// The share of the columns round a plains spawn that are open ground (no water, trunk or plant
/// in the way), for the check: a conservative guess from the first-day playtest's hillside.
pub const OPEN_SHARE: f32 = 0.5;

/// The chance a column of open ground has `g`: `stony` on steep ground or a shore, `crown` under
/// a tree's crown, `from_spawn` its distance from the world's spawn in blocks.
pub fn ground_chance(g: Ground, stony: bool, crown: bool, from_spawn: f32) -> f32 {
    let base = match g {
        Ground::Rock => ROCK * if stony { STONY } else { 1.0 },
        Ground::Flint => FLINT * if stony { STONY } else { 1.0 },
        Ground::Branch => {
            if crown {
                BRANCH
            } else {
                BRANCH_OPEN
            }
        }
    };
    base * if from_spawn <= NEAR_SPAWN as f32 { NEAR_SPAWN_TIMES } else { 1.0 }
}

/// The most the three chances add up to anywhere (stony, under a crown, by the spawn): worldgen
/// looks no further when its roll is over it.
pub const MOST: f32 = 0.125;

/// Which of the ground's things a column gets for a roll `u` (0..1), if any: rock first, then
/// flint, then a branch, each its own share of the roll.
pub fn ground_at(u: f32, stony: bool, crown: bool, from_spawn: f32) -> Option<Ground> {
    let mut at = 0.0;
    for g in [Ground::Rock, Ground::Flint, Ground::Branch] {
        at += ground_chance(g, stony, crown, from_spawn);
        if u < at {
            return Some(g);
        }
    }
    None
}

/// Sticks a fallen branch gives: one, and a second this often.
pub const BRANCH_SECOND_STICK: f32 = 0.5;

/// A leaf block broken by a bare hand gives a stick this often (Ronald: "if you break leaves,
/// maybe you get some sticks"); the leaves' own loot gives 5% to anything, decay included.
pub const LEAF_STICK_BY_HAND: f32 = 0.20;
pub const LEAF_STICK_LOOT: f32 = 0.05;

/// The extra chance on top of the loot's own, so a leaf broken by hand gives a stick
/// `LEAF_STICK_BY_HAND` of the time.
pub fn leaf_stick_extra() -> f32 {
    top_up(LEAF_STICK_LOOT, LEAF_STICK_BY_HAND)
}

/// A fibre plant cut with a sharp edge (a flake, a knife) gives fibre this often; pulled by hand,
/// its own loot's chance (`clothing::FIBRE_CHANCE` for grass, `wild::FIBRE` for the rest).
pub const FIBRE_CUT: f32 = 0.5;

/// The plants that give fibre, and their own chance by hand, besides `wild::FIBRE`'s.
pub const FIBRE_PLANTS: [(&str, f32); 7] = [
    ("base:tall_grass", crate::clothing::FIBRE_CHANCE),
    ("base:tall_grass_double", crate::clothing::FIBRE_CHANCE),
    ("base:fern", crate::clothing::FIBRE_CHANCE),
    ("base:sword_fern", crate::clothing::FIBRE_CHANCE),
    ("base:banana", crate::clothing::FIBRE_CHANCE),
    ("base:philodendron", crate::clothing::FIBRE_CHANCE),
    ("base:liana", 0.5),
];

/// A block's own fibre chance by hand, if it is a fibre plant (by name, `base:sedge`).
pub fn fibre_by_hand(block: &str) -> Option<f32> {
    FIBRE_PLANTS
        .iter()
        .chain(crate::wild::FIBRE.iter())
        .find(|(b, _)| *b == block)
        .map(|p| p.1)
}

/// The extra chance of fibre when a plant with `by_hand` is cut with an edge, so at least one
/// comes `FIBRE_CUT` of the time (never less than by hand).
pub fn fibre_cut_extra(by_hand: f32) -> f32 {
    top_up(by_hand, FIBRE_CUT)
}

/// The chance `x` of a second, independent roll so that one or both come `want` of the time when
/// the first comes `have` of it: 1 − (1 − want) / (1 − have).
fn top_up(have: f32, want: f32) -> f32 {
    if have >= want || have >= 1.0 { 0.0 } else { 1.0 - (1.0 - want) / (1.0 - have) }
}

/// Striking: a rock used on a nodule or a rock lying on the ground. One strike at most every
/// `STRIKE_SECS` (a quicker click only knocks).
pub const STRIKE_SECS: f32 = 1.5;
/// The chance a strike takes a flake off: good flint, a plain rock (granite, basalt) rarely and
/// poorly.
pub const FLAKE_FROM_FLINT: f32 = 0.8;
pub const FLAKE_FROM_ROCK: f32 = 0.25;
/// Flakes a nodule or rock gives before it is used up.
pub const FLAKES_PER_STONE: u32 = 3;
/// Knapping a head (sneaking while striking a flint nodule opens the knapping screen): a 5 × 5
/// nodule, each square outside the head's outline struck away with a click; a click inside the
/// outline breaks the nodule into a flake (TerraFirmaCraft's rock knapping, about 8 clicks;
/// Ronald, 11-05: "Knapping screen, then book"). After a head of a shape has been held, the
/// book makes it from a nodule (`book_heads`).
pub const KNAP_SIZE: usize = 5;

/// A head knapped by hand: its part shape, the tool kind it hafts into, and its outline, top row
/// first (`X` kept, `.` struck away).
#[derive(Clone, Copy, Debug)]
pub struct Knapped {
    pub shape: &'static str,
    pub kind: &'static str,
    pub outline: [&'static str; KNAP_SIZE],
    /// Flint nodules the book takes for it (the grid's own pattern: one, two side by side, two
    /// stacked), so the 2 × 2 tells them apart.
    pub book: &'static [&'static str],
}

pub const KNAPPED: [Knapped; 3] = [
    Knapped { shape: "felling_axe_head", kind: "axe", outline: [".XX..", "XXXX.", "XXXXX", "XXXX.", ".XX.."], book: &["F"] },
    Knapped { shape: "pick_head", kind: "pickaxe", outline: [".....", "XXXXX", "XXXXX", "X...X", "....."], book: &["FF"] },
    Knapped { shape: "shovel_blade", kind: "shovel", outline: [".XXX.", ".XXX.", "XXXXX", "XXXXX", "..X.."], book: &["F", "F"] },
];

impl Knapped {
    /// Whether square `i` (row by row) is inside the outline.
    pub fn keeps(&self, i: usize) -> bool {
        self.outline
            .get(i / KNAP_SIZE)
            .and_then(|row| row.as_bytes().get(i % KNAP_SIZE))
            == Some(&b'X')
    }

    /// The squares to strike away, as a bit each.
    pub fn away(&self) -> u32 {
        (0..KNAP_SIZE * KNAP_SIZE)
            .filter(|&i| !self.keeps(i))
            .fold(0, |m, i| m | 1 << i)
    }
}

/// The knapped head of part shape `shape`.
pub fn knapped(shape: &str) -> Option<&'static Knapped> {
    KNAPPED.iter().find(|k| k.shape == shape)
}

/// What hafting a head of a kind by hand takes: the holding part's shape and the cordage of its
/// lashing. A knife is a flake on a short grip.
pub const HAFTED: [(&str, &str, u32); 4] = [("axe", "haft", 2), ("pickaxe", "haft", 2), ("shovel", "haft", 2), ("knife", "grip", 1)];

/// Hands-on work scores this much over the same work from memory (`crafting::grades`), up to the
/// most work gives.
pub const BY_HAND_BONUS: u32 = 10;

/// A sharp flake's uses before it's blunt: grass cut, sticks shaved.
pub const FLAKE_USES: u32 = 40;

/// Cordage a stone tool's lashing takes (string: two fibres twisted, `clothing::FIBRE_PER_STRING`).
pub const LASHING_CORDAGE: u32 = 2;
/// The grade of the hand-made haft (a stick, `branch` wood) and of the lashing: Plain (3).
pub const HAND_PART_GRADE: u32 = 3;

/// Grade points for a hand-knapped head: flint's material share, the hands-on work (rising with the
/// knapping skill, as from the book), no station.
pub const HEAD_MATERIAL_POINTS: u32 = 24;

/// The first fire (Ronald, 11-05: "Fire bed + bow drill"): a fire bed of sticks and tinder (fibre)
/// put on the ground, lit by holding a drill on it. Friction fire, for real: a bow drill gives an
/// ember in under a minute with good wood, a hand drill took 3-4 minutes over 370 wood pairs and
/// often failed (`reports/The first hours.md`).
pub const FIRE_BED_STICKS: u32 = 3;
/// A drill: (seconds held, the chance it gives an ember, uses before it's worn out).
#[derive(Clone, Copy, Debug)]
pub struct Drill {
    pub item: &'static str,
    pub secs: f32,
    pub chance: f32,
    pub uses: u32,
}
pub const BOW_DRILL: Drill = Drill { item: "bow_drill", secs: 6.0, chance: 0.7, uses: 12 };
pub const HAND_DRILL: Drill = Drill { item: "hand_drill", secs: 10.0, chance: 0.35, uses: 6 };
pub const DRILLS: [Drill; 2] = [BOW_DRILL, HAND_DRILL];
/// Blocks over a fire bed that keep the rain off it.
pub const ROOF_REACH: i32 = 24;

/// The first axe's hint names the nearest tree foot (a log on ground) within this many blocks
/// across (playtest, 11-05: on seeds 1 and 42 no trunk was in reach of spawn in a minute, and
/// nothing said where one was) ...
pub const TREE_LOOK: i32 = 48;
/// ... and this many up or down: a foot on a ledge far above isn't one to walk to.
pub const TREE_UP: i32 = 8;

/// Seconds to an ember, on average, with a drill: its holds until one takes.
pub fn ember_secs(d: Drill) -> f32 {
    d.secs / d.chance
}

/// What the first stone axe takes, start to finish, for a player who knows the way, in seconds:
/// the sum of the steps' own times (walking left out). The timing table in `notes/FIRST_HOURS.md`.
pub fn axe_hands_secs() -> f32 {
    // A flake: strikes until one comes.
    let flake = STRIKE_SECS / FLAKE_FROM_FLINT;
    // A head: a second a click on the knapping screen.
    let head = (KNAPPED[0].away().count_ones()) as f32;
    // Fibre: 2 cordage of 2 fibres, cut at 0.4 s a plant.
    let fibre = LASHING_CORDAGE as f32 * crate::clothing::FIBRE_PER_STRING as f32 / FIBRE_CUT * 0.4;
    flake + head + fibre
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spawn_has_enough_at_hand() {
        let area = std::f32::consts::PI * (SPAWN_LOOK * SPAWN_LOOK) as f32 * OPEN_SHARE;
        for (g, least) in SPAWN_LEAST {
            // A plains spawn: open ground, few crowns (branches by the open chance).
            let expect = area * ground_chance(g, false, false, 0.0);
            assert!(expect >= least, "{g:?}: {expect:.1} expected within {SPAWN_LOOK}, want {least}");
        }
        // Flint stays rarer than rock everywhere, and far from the spawn the ground is thinner.
        assert!(ground_chance(Ground::Flint, true, false, 500.0) < ground_chance(Ground::Rock, true, false, 500.0));
        assert!(ground_chance(Ground::Rock, false, false, 500.0) < ground_chance(Ground::Rock, false, false, 0.0));
    }

    #[test]
    fn most_covers_every_case() {
        for stony in [false, true] {
            for crown in [false, true] {
                let sum: f32 = [Ground::Rock, Ground::Flint, Ground::Branch]
                    .iter()
                    .map(|&g| ground_chance(g, stony, crown, 0.0))
                    .sum();
                assert!(sum <= MOST, "{sum}");
            }
        }
    }

    #[test]
    fn rolls_split_in_order() {
        let r = ground_chance(Ground::Rock, false, true, 900.0);
        assert_eq!(ground_at(0.0, false, true, 900.0), Some(Ground::Rock));
        assert_eq!(ground_at(r + 1e-6, false, true, 900.0), Some(Ground::Flint));
        assert_eq!(ground_at(0.99, false, true, 900.0), None);
    }

    #[test]
    fn cutting_beats_pulling_and_hands_find_sticks() {
        let grass = fibre_by_hand("base:tall_grass").unwrap();
        let x = fibre_cut_extra(grass);
        assert!((1.0 - (1.0 - grass) * (1.0 - x) - FIBRE_CUT).abs() < 1e-5);
        assert_eq!(fibre_cut_extra(fibre_by_hand("base:cattail").unwrap()), 0.0);
        assert!(fibre_by_hand("base:sedge").is_some() && fibre_by_hand("base:stone").is_none());
        let y = leaf_stick_extra();
        assert!((1.0 - (1.0 - LEAF_STICK_LOOT) * (1.0 - y) - LEAF_STICK_BY_HAND).abs() < 1e-5);
    }

    #[test]
    fn the_axe_is_earned_not_a_chore() {
        // More than a minute of the hands' own work for one who knows the way, less than three.
        let s = axe_hands_secs();
        assert!((15.0..180.0).contains(&s), "{s} s");
    }

    #[test]
    fn the_bow_drill_beats_the_hand_drill() {
        assert!(ember_secs(BOW_DRILL) < 15.0 && ember_secs(HAND_DRILL) > 2.0 * ember_secs(BOW_DRILL));
        const { assert!(BOW_DRILL.uses > HAND_DRILL.uses) };
    }

    #[test]
    fn heads_take_a_few_clicks_and_tell_apart() {
        for k in KNAPPED {
            let clicks = k.away().count_ones();
            assert!((6..=13).contains(&clicks), "{}: {clicks} clicks", k.shape);
            assert!(k.outline.iter().all(|r| r.len() == KNAP_SIZE));
            assert!(HAFTED.iter().any(|h| h.0 == k.kind), "{} hafts", k.kind);
        }
        for (i, a) in KNAPPED.iter().enumerate() {
            assert!(
                KNAPPED[i + 1..]
                    .iter()
                    .all(|b| b.book != a.book && b.outline != a.outline)
            );
        }
    }
}

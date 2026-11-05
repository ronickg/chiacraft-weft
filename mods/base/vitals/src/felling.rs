//! Felling's numbers (`mods/base/src/felling.rs`; Ronald, 11-05: "chopping should be pretty hard.
//! So it takes longer as you always chop down the bottom part"). A tree comes down when its foot
//! log is chopped with an axe, and that one chop takes longer the more logs stand on it:
//! `logs^GROWTH` times as long as one log, with the axe's own speed on top (the engine times a
//! log by its hardness and the axe's mining speed). An axe has a tier (its harvest level), and
//! each tier takes trees up to a size (`takes`). A tree past it can still be felled (Ronald chose
//! "Slow and fruitless"): `PAST_TIER` times slower still, and it gives no wood.

/// How chop time grows with the logs on the foot: a 6-log birch is 3.8 logs' time, a 30-log
/// oak 12.8, a 1350-log sequoia 222. Under 1, so the big trees aren't hopeless with the right
/// axe; over 0.5, so they're still a job.
pub const GROWTH: f32 = 0.75;
/// A laid tree's log whose cell is taken (a crown's branches turn onto the same cells) lies on the
/// first free one up to this many over it; past that it drops as its item.
pub const PILE_UP: i32 = 3;

/// A tree past its axe's tier takes this many times longer (and gives nothing).
pub const PAST_TIER: f32 = 8.0;

/// A felled trunk lies in pieces this many logs long (Valheim's large log in two halves; a
/// lying trunk is cut a piece at a time, `piece`).
pub const PIECE: i32 = 4;

/// How far a slice of a felled trunk may drop to the ground (down a slope) before it rests where
/// it is.
pub const LAND_DROP: i32 = 8;

/// The fall (`mods/base/src/fall.rs`; Panda's Falling Trees' curve, Dynamic Trees' slower tall
/// trees): a creak leaning `CREAK_DEGREES` over `CREAK_TICKS`, then an ease-in fall of
/// `fall_ticks(height)`, then a bounce `BOUNCE_DEGREES` up and back over `BOUNCE_TICKS`.
pub const CREAK_TICKS: u32 = 20;
pub const CREAK_DEGREES: f32 = 3.0;
pub const BOUNCE_TICKS: u32 = 12;
pub const BOUNCE_DEGREES: f32 = 8.0;
/// Seconds a tree 10 blocks high takes to fall; others by the square root of their height,
/// from `FALL_SECS.1` to `FALL_SECS.2`.
pub const FALL_SECS: (f32, f32, f32) = (2.5, 2.0, 5.0);
/// The pose goes to clients every this many ticks, eased between.
pub const POSE_EVERY: u32 = 2;
/// Leaves start breaking off past this angle, and by the ground this share of them has.
pub const SHED_FROM: f32 = 20.0;
pub const SHED_SHARE: f32 = 0.7;
/// Pieces a breaking leaf throws; one in this many drops its loot.
pub const SHED_BITS: u32 = 4;
pub const SHED_LOOT_ONE_IN: u32 = 3;
/// What is left of the crown at the ground: one in this many bursts (the rest just goes).
pub const SHED_BURST_ONE_IN: u32 = 2;

/// Ticks a tree `height` blocks high takes to fall (after its creak).
pub fn fall_ticks(height: f32) -> u32 {
    let (ten, least, most) = FALL_SECS;
    ((ten * (height.max(1.0) / 10.0).sqrt()).clamp(least, most) * crate::TICKS as f32).round() as u32
}

/// A falling tree strikes what is in its way from this angle on (near the stump it is its own
/// stump's neighbours), testing at most this many of its logs a tick; another tree's trunk it
/// strikes falls too if it has this many times that tree's logs.
pub const KNOCK_FROM: f32 = 10.0;
pub const KNOCK_SAMPLES: usize = 96;
pub const KNOCK_FELLS: usize = 2;

/// A tree caught on a trunk it can't knock down leans back this much onto it and settles there
/// as blocks after this many ticks.
pub const HANG_BACK: f32 = 4.0;
pub const HANG_SETTLE_TICKS: u32 = 16;

/// What a falling tree does to a creature in its way (Planner, 11-05): 2 and a tenth of its logs,
/// at most 20, once; it hits what is within this of a log's middle each way (half a block and a
/// little more, for the ticks between).
pub const HURT_BASE: f32 = 2.0;
pub const HURT_PER_LOGS: f32 = 10.0;
pub const HURT_MOST: f32 = 20.0;
pub const HURT_REACH: f64 = 0.8;

pub fn hurt(logs: u32) -> f32 {
    (HURT_BASE + logs as f32 / HURT_PER_LOGS).min(HURT_MOST)
}

/// The thud sounds this many ticks before the tree reaches the ground (Panda's 0.25 s: the sound
/// lands with the sight of it).
pub const THUD_EARLY_TICKS: u32 = 5;

/// How loud a falling tree `height` blocks high sounds: a sapling's half, a 20-high tree's full.
pub fn sound_volume(height: f32) -> f32 {
    (0.5 + height / 40.0).clamp(0.5, 1.0)
}

/// The longest a felling may take, in seconds, past tier or not: past an hour nobody is
/// chopping, and the engine's mining speed has a floor.
pub const LONGEST_SECS: f32 = 3600.0;

/// Axe tiers by harvest level (`abi::tool_tier`, the crafting materials' `harvest`): the most
/// logs a tree may have for an axe of that level to fell it for its wood. The highest row at or
/// under the axe's level counts. Sized on the species (`cargo run -p base_api --example
/// tree_logs`, 11-05, after the trees by space colonization): the ordinary trees grow 4 to 87
/// logs (a stone axe takes them all); a wooden one the first day's common trees, the medians of
/// the oak (43), birch (9) and spruce (21) and most oaks (Planner, 11-05: raised from 40 when the
/// colonized oak's median passed it); bone a little more (most araucarias); a young sequoia 76 to
/// 231 and the tropical canopy trees up to 181 (copper), the Brazil nut up to 432 (bronze), a
/// grown sequoia 444 to 1421 (steel).
pub const TIERS: &[Tier] = &[
    Tier { harvest: 0, name: "wood, gold", logs: 56, speed: 2.0 },
    Tier { harvest: 50, name: "bone, antler", logs: 72, speed: 2.5 },
    Tier { harvest: 100, name: "flint, chert, obsidian, stone", logs: 96, speed: 2.91 },
    Tier { harvest: 150, name: "copper", logs: 192, speed: 4.85 },
    Tier { harvest: 200, name: "bronze, iron, wrought iron", logs: 512, speed: 6.79 },
    Tier { harvest: 300, name: "steel, diamond (old)", logs: 1536, speed: 7.76 },
    Tier { harvest: 400, name: "diamond", logs: 4096, speed: 8.0 },
];

#[derive(Clone, Copy, Debug)]
pub struct Tier {
    pub harvest: u32,
    pub name: &'static str,
    /// The most logs a tree may have for this tier to fell it for its wood.
    pub logs: u32,
    /// A typical axe's mining speed at this tier, for the sheet only (the crafting sheet's
    /// felling heads on an oak haft; the wooden and bone ones the old items'): each axe has its own.
    pub speed: f32,
}

/// Trees the sheet times (logs, from `tree_logs`): a median oak, birch and araucaria, a young
/// sequoia and a grown one.
pub const SHEET_TREES: &[(&str, u32)] = &[("oak", 43), ("birch", 9), ("araucaria", 64), ("young sequoia", 165), ("sequoia", 1015)];

/// The tier of an axe of `harvest` level.
pub fn tier(harvest: u32) -> &'static Tier {
    TIERS
        .iter()
        .rev()
        .find(|t| t.harvest <= harvest)
        .unwrap_or(&TIERS[0])
}

/// What felling a tree of `logs` with an axe of `harvest` level is like.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Felling {
    /// How many times one log's chop the foot takes (the engine's mining time of the log with
    /// this axe, times this).
    pub chops: f32,
    /// The tree is within the axe's tier: it gives its wood.
    pub fruitful: bool,
}

pub fn felling(logs: u32, harvest: u32) -> Felling {
    let fruitful = logs <= tier(harvest).logs;
    let chops = (logs.max(1) as f32).powf(GROWTH) * if fruitful { 1.0 } else { PAST_TIER };
    Felling { chops, fruitful }
}

/// Chopping a tree's foot with bare hands or any tool but an axe: as slow as an axe past its tier
/// and as fruitless (Planner, 11-05), with one log's time the hand's. The first axe is made off the
/// ground by hand (`first`: rocks, flint, a lashed stone axe; Ronald, 11-05: "you can't just punch
/// down a tree").
pub fn by_hand(logs: u32) -> Felling {
    fruitless(logs)
}

/// A tree that falls for nothing: cut above its foot, or its foot gone to no player (an
/// explosion, a falling tree). Slow if chopped, and it gives no wood.
pub fn fruitless(logs: u32) -> Felling {
    Felling { chops: (logs.max(1) as f32).powf(GROWTH) * PAST_TIER, fruitful: false }
}

/// Cutting a piece of `logs` of a felled trunk: as long as felling a tree of them, whatever the
/// axe (the tree was already its tier's).
pub fn piece(logs: u32) -> Felling {
    Felling { chops: (logs.max(1) as f32).powf(GROWTH), fruitful: true }
}

/// What cutting a laid tree's piece wears the axe by (Planner, 11-04, option A): the felling is the
/// hard work and wears the axe by its chops (`wear`); cutting the 15 pieces of a 60-log oak at
/// `wear` each cost another 45, so a hand-lashed axe (cordage, 30 uses) broke on its first tree
/// (PC run fh_natural2). At 1 a piece, one oak costs about 36: the first axe fells it and cuts it up.
pub const PIECE_WEAR: u32 = 1;

/// What a felling wears the axe by: its chops at the tier's pace (a tree past the tier takes
/// longer, not more of the axe).
pub fn wear(logs: u32) -> u32 {
    (logs.max(1) as f32).powf(GROWTH).round() as u32
}

/// Seconds to fell it, given one log's mining time with the axe (`log_secs`), capped at
/// `LONGEST_SECS`.
pub fn seconds(f: Felling, log_secs: f32) -> f32 {
    (f.chops * log_secs).min(LONGEST_SECS)
}

/// The mining speed multiplier on the player while chopping the foot, so the engine's own timer
/// (client and server alike) takes `seconds`: one log's time over the whole felling's.
pub fn speed(f: Felling, log_secs: f32) -> f32 {
    if log_secs <= 0.0 {
        return 1.0;
    }
    log_secs / seconds(f, log_secs)
}

/// One log's mining time in seconds with an axe of `speed` (the engine's: hardness × 1.5 over
/// the tool's speed, for a block the tool harvests).
pub fn log_secs(hardness: f32, speed: f32) -> f32 {
    hardness * 1.5 / speed.max(0.01)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bigger_trees_take_longer_and_better_axes_take_bigger_trees() {
        // More logs, more chops; a tree past the tier is slower still and gives nothing.
        assert!(felling(30, 100).chops > felling(6, 100).chops);
        assert!(felling(30, 100).fruitful);
        // The first day's trees (`tree_logs`, 11-05): a wooden axe fells the median oak (43 logs)
        // for its wood, the first stone axe (flint, harvest 100) the biggest (87).
        assert!(felling(43, 0).fruitful && felling(87, 100).fruitful);
        let giant = felling(1350, 100);
        assert!(!giant.fruitful && giant.chops > felling(1350, 300).chops);
        assert!(felling(1350, 300).fruitful, "steel fells a sequoia for its wood");
        // Tiers climb and are found by the highest row at or under the level.
        assert!(
            TIERS
                .windows(2)
                .all(|w| w[0].harvest < w[1].harvest && w[0].logs < w[1].logs)
        );
        assert_eq!(tier(220).harvest, 200);
        assert_eq!(tier(0).logs, TIERS[0].logs);
        // The speed multiplier makes the engine's timer take the felling's seconds.
        let (f, one) = (felling(30, 100), log_secs(2.0, 2.91));
        assert!((one / speed(f, one) - seconds(f, one)).abs() < 1e-3);
        assert!(seconds(felling(1350, 0), log_secs(2.0, 1.0)) <= LONGEST_SECS);
        // Bare hands are as slow and fruitless as an axe past its tier.
        assert_eq!(by_hand(60), felling(60, 0));
        assert!(!fruitless(5).fruitful && fruitless(5).chops > felling(5, 0).chops);
        assert!(!by_hand(1).fruitful);
        // A piece of four logs is 2.8 logs' time; the axe wears by chops, not by logs.
        assert!((piece(4).chops - 2.83).abs() < 0.01);
        assert_eq!(wear(20), 9);
        assert!(wear(1015) < 1015 / 4);
        // A falling tree hurts by its size, within bounds.
        assert_eq!(hurt(20), 4.0);
        assert_eq!(hurt(1015), HURT_MOST);
        // A tall tree falls slower, within its bounds.
        assert_eq!(fall_ticks(10.0), 50);
        assert!(fall_ticks(40.0) > fall_ticks(10.0) && fall_ticks(400.0) == 100 && fall_ticks(1.0) == 40);
    }
}

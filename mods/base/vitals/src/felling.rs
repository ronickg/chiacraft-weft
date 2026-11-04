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

/// A tree past its axe's tier takes this many times longer (and gives nothing).
pub const PAST_TIER: f32 = 8.0;

/// A felled trunk lies in pieces this many logs long (Valheim's large log in two halves; a
/// lying trunk is cut a piece at a time, `piece`).
pub const PIECE: i32 = 4;

/// How far a slice of a felled trunk may drop to the ground (down a slope) before it rests where
/// it is.
pub const LAND_DROP: i32 = 8;

/// The longest a felling may take, in seconds, past tier or not: past an hour nobody is
/// chopping, and the engine's mining speed has a floor.
pub const LONGEST_SECS: f32 = 3600.0;

/// Axe tiers by harvest level (`abi::tool_tier`, the crafting materials' `harvest`): the most
/// logs a tree may have for an axe of that level to fell it for its wood. The highest row at or
/// under the axe's level counts. Sized on the species (`cargo run -p base_api --example
/// tree_logs`, 11-05): the ordinary trees grow 5 to 72 logs (a stone axe takes them all, a
/// wooden one the median oak), a young sequoia 76 to 231 (copper), a grown one 444 to 1421
/// (steel).
pub const TIERS: &[Tier] = &[
    Tier { harvest: 0, name: "wood, gold", logs: 24, speed: 2.0 },
    Tier { harvest: 50, name: "bone, antler", logs: 40, speed: 2.5 },
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
pub const SHEET_TREES: &[(&str, u32)] = &[("oak", 20), ("birch", 34), ("araucaria", 72), ("young sequoia", 165), ("sequoia", 1015)];

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

/// Cutting a piece of `logs` of a felled trunk: as long as felling a tree of them, whatever the
/// axe (the tree was already its tier's).
pub fn piece(logs: u32) -> Felling {
    Felling { chops: (logs.max(1) as f32).powf(GROWTH), fruitful: true }
}

/// What a felling or a piece wears the axe by: its chops at the tier's pace (a tree past the tier
/// takes longer, not more of the axe).
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
        // A piece of four logs is 2.8 logs' time; the axe wears by chops, not by logs.
        assert!((piece(4).chops - 2.83).abs() < 0.01);
        assert_eq!(wear(20), 9);
        assert!(wear(1015) < 1015 / 4);
    }
}

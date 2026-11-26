//! Levels and XP: a point to place a level, up to `MAX_LEVEL`; each level asks more XP than the
//! last (ARK's shape).

pub const MAX_LEVEL: u32 = 100;

/// XP from `level` to the next (proposed `12 + 8 * level^1.45`: 20 for the first, about 240 at
/// level 10, 2 300 at 50, 6 300 from 99 to 100; about 260 000 from 1 to 100).
pub fn to_next(level: u32) -> u64 {
    (12.0 + 8.0 * (level.max(1) as f64).powf(1.45)).round() as u64
}

/// XP from level 1 to `level`.
pub fn total_to(level: u32) -> u64 {
    (1..level.min(MAX_LEVEL)).map(to_next).sum()
}

/// The level that much XP earned since level 1 reaches, and the XP into it.
pub fn level_of(xp: u64) -> (u32, u64) {
    let mut level = 1;
    let mut left = xp;
    while level < MAX_LEVEL && left >= to_next(level) {
        left -= to_next(level);
        level += 1;
    }
    (level, left)
}

/// XP left after dying: `loss` of the XP into the current level goes (never a level).
pub fn after_death(xp: u64, loss: f32) -> u64 {
    let (level, into) = level_of(xp);
    total_to(level) + (into as f64 * (1.0 - loss.clamp(0.0, 1.0) as f64)) as u64
}

/// Where XP comes from (proposed amounts; `notes/SURVIVAL.md`, "XP").
pub mod xp {
    /// Killing a creature: by its level and how dangerous its kind is (1 a harmless one).
    pub fn kill(level: u32, danger: f32) -> u64 {
        (4.0 * danger * (1.0 + level as f32 * 0.1)).round() as u64
    }
    /// Mining or gathering a block: by how long it takes by hand (its hardness).
    pub fn gather(hardness: f32) -> u64 {
        (hardness * 0.7).ceil().max(1.0) as u64
    }
    /// Crafting or cooking: by how many things went into it.
    pub fn craft(ingredients: u32) -> u64 {
        ingredients.max(1) as u64
    }
    /// Placing a block (most building is cheap; a little for every one).
    pub const BUILD: u64 = 1;
    /// Taming an animal of that level.
    pub fn tame(level: u32) -> u64 {
        50 + level as u64 * 4
    }
    /// A young one born or hatched in your care.
    pub const RAISE: u64 = 40;
    /// The first visit to a biome; a dimension gives ten times that.
    pub const DISCOVER: u64 = 60;
    /// An advancement made.
    pub const ADVANCEMENT: u64 = 100;
}

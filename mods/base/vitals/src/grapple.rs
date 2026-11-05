//! The grappling hook (`mods/base/src/grapple.rs`; the engine holds the rope: `core:rope_length`
//! and the rest, the `ROPE` pose): thrown with use, it catches on a solid block that gives a hold
//! (its grip, `climb::GRIP_*`, over 0) within reach, or falls back. On the rope jump pulls one up
//! hand over hand and sneak lets one down, gravity swings one; use again lets go. Hanging and
//! pulling cost climbing's stamina (`climb::cost`, the same burdens); run out and the hands open.

use crate::curve::{LEVELS_PER_AGE, g};

/// A hook: how far it is thrown (blocks), how fast its rope is pulled in (blocks a second), and its
/// item level (throws from the curve).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hook {
    pub name: &'static str,
    pub reach: f32,
    pub reel: f32,
    pub level: f32,
}

impl Hook {
    /// Throws it lasts.
    pub fn durability(&self) -> u32 {
        (DURABILITY_THROWS * g(self.level)).round() as u32
    }
}

/// The iron grappling hook: three wrought-iron flukes on a shank and a long cord of string. Iron
/// Age by its iron (`gear::MATERIALS`: wrought iron, tier 2, depth 3). A grapnel thrown by hand
/// reaches some 15-25 m; the rope is climbed at a brisk climber's pace (proposed).
pub const IRON: Hook = Hook { name: "base:grappling_hook", reach: 22.0, reel: 3.0, level: 2.0 * LEVELS_PER_AGE + 3.0 };

pub const HOOKS: [Hook; 1] = [IRON];

pub fn hook(item: &str) -> Option<&'static Hook> {
    HOOKS.iter().find(|h| h.name == item)
}

/// Throws at level 0 (×2.95 at the iron hook's 23: about 240).
pub const DURABILITY_THROWS: f32 = 80.0;

/// Letting out is this share of pulling in (the engine's, `client::step_player`).
pub const LET_OUT_SHARE: f32 = 0.7;

/// Stamina it takes to take hold of a rope again after running out on one.
pub const AGAIN: f32 = crate::climb::AGAIN;

/// Seconds a missed throw's rope shows going out and coming back.
pub const MISS_SECS: f32 = 0.6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_lasts_a_few_hundred_throws() {
        let d = IRON.durability();
        assert!((150..=400).contains(&d), "{d}");
    }

    #[test]
    fn a_fresh_player_pulls_up_a_cliff() {
        // 100 stamina at climbing's cost: 20 blocks of rope and more.
        let up = 100.0 / crate::climb::cost(true, crate::climb::FREE) * IRON.reel;
        assert!(up > 20.0 && up < 80.0, "{up}");
    }
}

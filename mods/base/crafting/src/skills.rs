//! Crafting skills (`notes/CRAFTING.md`, "The ten grades"): one per way of making a part, rising
//! as parts are made. A skill's level is the work share of a part made from the book
//! (`grades::work_from_memory`), which stops at `grades::MEMORY_MAX`: skill makes book work good,
//! only hands-on work makes it the best.

use crate::materials::Making;

/// The crafts, in the order a player's experience is kept.
pub const SKILLS: [(Making, &str); 6] = [
    (Making::Knap, "skill.base.knapping"),
    (Making::Carve, "skill.base.carving"),
    (Making::Cast, "skill.base.casting"),
    (Making::Forge, "skill.base.smithing"),
    (Making::Hands, "skill.base.handwork"),
    (Making::Sew, "skill.base.sewing"),
];

/// The highest level.
pub const MAX_LEVEL: u32 = 10;

/// Where a craft's experience is kept.
pub fn index(m: Making) -> usize {
    SKILLS.iter().position(|(x, _)| *x == m).unwrap_or(0)
}

/// Experience a level needs, in parts made: 5, 15, 30 ... (5 x the triangle number), so level 10
/// is about 275 parts of that craft.
pub fn needs(level: u32) -> u32 {
    5 * level * (level + 1) / 2
}

/// The level `xp` reaches.
pub fn level(xp: u32) -> u32 {
    (0..=MAX_LEVEL).rev().find(|&l| xp >= needs(l)).unwrap_or(0)
}

/// Experience one part gives: one, and one more for a part above Sound (a good one teaches more).
pub fn xp_for(grade: u8) -> u32 {
    1 + (grade > 4) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grades::{MEMORY_MAX, work_from_memory};

    #[test]
    fn levels_rise_and_stop() {
        assert_eq!(level(0), 0);
        assert_eq!(level(5), 1);
        assert_eq!(level(14), 1);
        assert_eq!(level(needs(MAX_LEVEL)), MAX_LEVEL);
        assert_eq!(level(u32::MAX), MAX_LEVEL);
        // The top level's work from memory is the cap, never past it.
        assert_eq!(work_from_memory(level(u32::MAX)), MEMORY_MAX);
    }

    #[test]
    fn every_making_has_a_skill() {
        for m in [Making::Knap, Making::Carve, Making::Cast, Making::Forge, Making::Hands, Making::Sew] {
            assert_eq!(SKILLS[index(m)].0, m);
        }
    }
}

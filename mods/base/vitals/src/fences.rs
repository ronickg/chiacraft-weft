//! Fences, gates and the scarecrow (`mods/base/src/fences.rs`): what keeps animals off a field.
//! Animals' crop raids (`animals_core::needs::RAID_LIGHT`, on the animals-plants branch) leave
//! alone a plant in block light of 8 or more and one they can't path to; a fence keeps them out,
//! a lit scarecrow lights the field.

/// A fence's and a shut gate's height, blocks: over the block an animal steps up
/// (`STEP_UP`) and short of two, so the pathfinder finds no room over it and nothing jumps it
/// (vanilla's fences are as tall, for the same reason).
pub const FENCE_HEIGHT: f32 = 1.5;
/// The most an animal goes up in one step, blocks (the pathfinder's step: one cell up).
pub const STEP_UP: f32 = 1.0;
/// Fences a recipe makes (two planks and a stick each, vanilla's 6 planks and 2 sticks for 3).
pub const FENCES_MADE: u32 = 3;

/// A lit scarecrow's light, as a torch's (`blocks.rs`, 15): a lantern hung on it.
pub const SCARECROW_LIGHT: u8 = 15;
/// The raid rule's block light (`animals_core::needs::RAID_LIGHT`): at or over it, a ripe plant
/// isn't raided. Kept here until that branch lands; a check then ties the two.
pub const RAID_LIGHT: u8 = 8;
/// Blocks a lit scarecrow keeps at the raid rule's light, along the ground (light falls one a
/// block, counted along the axes): a field 15 across round one scarecrow.
pub const fn scarecrow_reach() -> u8 {
    SCARECROW_LIGHT - RAID_LIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing steps over a fence, and a body two blocks tall can't stand on one under a block.
    #[test]
    fn fences_keep_animals_out() {
        const { assert!(FENCE_HEIGHT > STEP_UP && FENCE_HEIGHT < 2.0) };
        const { assert!(FENCES_MADE >= 1) };
    }

    /// A lit scarecrow lights a small field to the raid rule: 7 blocks each way.
    #[test]
    fn a_scarecrow_lights_a_field() {
        assert_eq!(scarecrow_reach(), 7);
        const { assert!(SCARECROW_LIGHT <= 15 && RAID_LIGHT > 0) };
    }
}

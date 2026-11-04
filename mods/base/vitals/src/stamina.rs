//! What stamina pays for, and what health and stamina come back by.

use crate::effects::Modifiers;
use crate::food::{ENERGY, GREENS, NUTRIENTS, PROTEIN, WATER};

/// A second of sprinting (proposed; a level-1 player well fed sprints about fourteen seconds).
pub const SPRINT: f32 = 7.0;
pub const JUMP: f32 = 8.0;
/// A second swimming, climbing.
pub const SWIM: f32 = 5.0;
pub const CLIMB: f32 = 6.0;
/// Blocking a hit, a dodge.
pub const BLOCK: f32 = 10.0;
pub const DODGE: f32 = 15.0;

/// A swing with a weapon or tool of that weight in kilograms (a sword 1.5 kg: 12).
pub fn swing(kg: f32) -> f32 {
    6.0 + 4.0 * kg
}

/// A swing at a block of that hardness (mining, chopping, digging; stone 1.5: 3.5).
pub fn work(hardness: f32) -> f32 {
    2.0 + hardness
}

/// Seconds after spending stamina before it comes back.
pub const REGEN_DELAY: f32 = 1.0;

/// Stamina back a second: 6% of max at full energy and water, half that on an empty stomach.
pub fn stamina_regen(max: f32, full: &[f32; NUTRIENTS], m: &Modifiers) -> f32 {
    max * 0.06 * (0.5 + 0.3 * full[ENERGY] + 0.2 * full[WATER]) * m.stamina_regen
}

/// Health back a second: 0.4% of max well fed on protein and greens (four minutes from empty),
/// a third of that on an empty stomach.
pub fn health_regen(max: f32, full: &[f32; NUTRIENTS], m: &Modifiers) -> f32 {
    max * 0.004 * (0.3 + 0.4 * full[PROTEIN] + 0.3 * full[GREENS]) * m.health_regen
}

//! Day routines (`animals_core::routines`): how about animal `id` is now, by its species' hours
//! (`Species::active`).

use weft_sdk::*;

use animals_core::routines;

/// The hour now (0..24, 6 dawn).
pub fn hour(w: &World) -> f32 {
    routines::hour(w.day_time())
}

/// How about species `index` is now, 0..1.
pub fn activity(w: &World, index: usize) -> f32 {
    crate::species::SPECIES
        .get(index)
        .map_or(1.0, |s| routines::activity(s.active, hour(w)))
}

/// How about animal `id` is now (its species'), 0..1; 1 for one that isn't an animal.
pub fn activity_of(w: &World, id: u64) -> f32 {
    w.component(id, &crate::animal::ANIMAL)
        .and_then(|a| (a.species as usize).checked_sub(1))
        .map_or(1.0, |i| activity(w, i))
}

/// The chance divisor of lying down (`1 in base`) by how sleepy it is: more often out of its
/// hours (`REST_SLEEPY` times), less in its busiest (`REST_BUSY`).
pub fn rest_chance(base: u32, activity: f32) -> u32 {
    let k = routines::REST_BUSY + (routines::REST_SLEEPY - routines::REST_BUSY) * (1.0 - activity).powi(2);
    ((base as f32 / k).round() as u32).max(1)
}

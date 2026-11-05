//! Climbing any wall on stamina (`mods/base/src/climbing.rs`; the engine climbs by
//! `core:climb_speed` times the wall's `grip`): pushing into a wall in the air holds on to it and
//! climbs, sneaking hangs still, sneaking into it climbs down, a wall that ends under the knees is
//! climbed over. Stamina pays by the second (`stamina::CLIMB`); run out and the hands let go.
//! What one carries, being wet and being cold make it slower and dearer.

use crate::effects::Effect;

/// Blocks a second up a wall of grip 1, unburdened: a 20-block cliff in about 13 seconds, a
/// little slower than a ladder's 2.4 (proposed).
pub const SPEED: f32 = 1.5;

/// How well hands and feet hold on, by the wall (the block's `grip`; the engine's default is 1:
/// cut stone, dirt, wood that's been worked). Rough rock and bark have holds everywhere; smooth
/// faces few; loose ground crumbles; ice and glass none.
pub const GRIP_ROUGH: f32 = 1.25;
pub const GRIP_SMOOTH: f32 = 0.6;
pub const GRIP_LOOSE: f32 = 0.75;
pub const GRIP_NONE: f32 = 0.0;

/// A second hanging still costs this share of a second climbing (`stamina::CLIMB`).
pub const HANG_SHARE: f32 = 0.35;

/// Stamina it takes to take hold again after running out on a wall.
pub const AGAIN: f32 = 15.0;

/// What the body's state does to climbing: `pace` times the speed, `cost` times the stamina.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burden {
    pub pace: f32,
    pub cost: f32,
}

pub const FREE: Burden = Burden { pace: 1.0, cost: 1.0 };

/// Each effect's share: the weight one carries (encumbered, overloaded), wet hands and clothes,
/// cold fingers. The rest change nothing here (they change stamina's own costs and return).
pub const BURDENS: [(Effect, Burden); 5] = [
    (Effect::Encumbered, Burden { pace: 0.8, cost: 1.25 }),
    (Effect::Overloaded, Burden { pace: 0.55, cost: 1.6 }),
    (Effect::Wet, Burden { pace: 0.85, cost: 1.15 }),
    (Effect::Cold, Burden { pace: 0.9, cost: 1.15 }),
    (Effect::Freezing, Burden { pace: 0.75, cost: 1.35 }),
];

/// The burden of all the effects on, together.
pub fn burden(on: &[Effect]) -> Burden {
    BURDENS
        .iter()
        .filter(|(e, _)| on.contains(e))
        .fold(FREE, |b, (_, x)| Burden { pace: b.pace * x.pace, cost: b.cost * x.cost })
}

/// Stamina a second on a wall: climbing (or climbing down), or hanging still.
pub fn cost(moving: bool, b: Burden) -> f32 {
    crate::stamina::CLIMB * if moving { 1.0 } else { HANG_SHARE } * b.cost
}

/// Blocks a second up a wall of grip 1.
pub fn speed(b: Burden) -> f32 {
    SPEED * b.pace
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burdens_slow_and_cost() {
        assert_eq!(burden(&[]), FREE);
        let heavy = burden(&[Effect::Overloaded, Effect::Wet, Effect::Freezing]);
        assert!(heavy.pace < 0.5 && heavy.pace > 0.25, "{heavy:?}");
        assert!(heavy.cost > 2.0, "{heavy:?}");
        assert!(cost(false, FREE) < cost(true, FREE));
    }

    #[test]
    fn a_fresh_player_climbs_a_cliff() {
        // A level-1 player well fed has about 100 stamina: 20 blocks of rough rock and more.
        let secs = 100.0 / cost(true, FREE);
        let blocks = secs * speed(FREE) * GRIP_ROUGH;
        assert!(blocks > 20.0 && blocks < 40.0, "{blocks}");
    }
}

//! Herds and packs that keep together (`reports/Creature AI.md`, part 3, step 9: shared group
//! plans; the crew's step 10). A wild herd's or pack's leader is the one of its kind near with the
//! lowest id, itself included (every member works out the same one from its own look round, so no
//! record has to be kept or handed over when one dies: the next lowest leads). The leader wanders
//! as one alone does; the others, wandering, keep to their own place round it (`slot`) instead of
//! walking off each its own way, so the herd drifts together and only the leader picks where to.
//!
//! The places are a loose ring round the leader, each member's own by its id, wider for a bigger
//! group: deer graze a few metres apart (Clutton-Brock, Guinness and Albon 1982, *Red Deer*), a
//! wolf pack on the move strings out within a few lengths of its lead.

/// Blocks from the leader to the nearest place, and how much wider the ring gets for each member
/// (by the square root of the group: the area grows with the group).
pub const SLOT_NEAR: f32 = 2.0;
pub const SLOT_PER: f32 = 1.2;
/// The ring's widest (half the distance kin count within, `senses::KIN` 16).
pub const SLOT_FAR: f32 = 8.0;
/// A member walks back to its place once it is this many blocks off it, at its wander's speed,
/// this much faster once twice that off (it keeps up with a leader on the move).
pub const FOLLOW_FROM: f32 = 3.0;
pub const CATCH_UP: f32 = 1.4;
/// Ticks between a member's checks of its place (a path every check at most).
pub const FOLLOW_EVERY: u64 = 20;
// The ring and a member's way back to it stay inside the distance its kin count within
// (`senses::KIN`, 16), so a member at its place still counts its leader.
const _: () = assert!(SLOT_FAR + FOLLOW_FROM * 2.0 <= 16.0);

/// Fewest of its kind near (itself among them) for a group to keep together.
pub const GROUP: u16 = 2;

/// Doing as its leader does (slice 2): a member whose leader is drinking drinks too from this much
/// of its water left, one whose leader grazes eats from this much of its food (instead of
/// `needs::THIRSTY` and `HUNGRY`, or the dusk drink's `routines::DRINKS_AT`). Herd animals
/// graze, drink and rest in step (Rook and Penning 1991 on grazing synchrony; red deer in
/// Clutton-Brock, Guinness and Albon 1982): the group moves on together instead of one at a time.
pub const JOIN_DRINK: f32 = 0.9;
pub const JOIN_EAT: f32 = 0.85;
// Joining starts before it would on its own (and before the dusk drink), and stops where it would.
const _: () = assert!(
    JOIN_DRINK > crate::routines::DRINKS_AT
        && JOIN_DRINK > crate::needs::THIRSTY
        && JOIN_DRINK < crate::needs::SLAKED
        && JOIN_EAT > crate::needs::HUNGRY
        && JOIN_EAT < crate::needs::FULL
);

/// The ring's radius for a group of `kin`.
pub fn ring(kin: u16) -> f32 {
    (SLOT_NEAR + SLOT_PER * (kin.max(1) as f32).sqrt()).min(SLOT_FAR)
}

/// `id`'s place round its leader in a group of `kin`: an offset (x, z) in blocks. Its angle by its
/// id (spread by the golden angle so neighbours' ids don't bunch), its distance between half the
/// ring and the ring.
pub fn slot(id: u64, kin: u16) -> [f32; 2] {
    let h = id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let a = (h >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
    let r = ring(kin) * (0.5 + 0.5 * ((h >> 16) & 0xFFFF) as f32 / 65535.0);
    [a.cos() * r, a.sin() * r]
}

/// The leader of a group from the ids of its kind near (itself among them): the lowest.
pub fn leader(ids: impl IntoIterator<Item = u64>) -> u64 {
    ids.into_iter().min().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_lie_round_the_leader_inside_the_ring() {
        for kin in [2u16, 5, 12, 40] {
            for id in 1..500u64 {
                let [x, z] = slot(id, kin);
                let d = (x * x + z * z).sqrt();
                assert!(d >= ring(kin) * 0.5 - 1e-3 && d <= ring(kin) + 1e-3, "{id} in {kin}: {d}");
            }
        }
        assert!(ring(40) <= SLOT_FAR && ring(2) > SLOT_NEAR);
    }

    #[test]
    fn places_spread_round() {
        // A dozen ids' places fall in at least five of eight directions.
        let mut seen = [false; 8];
        for id in 100..112u64 {
            let [x, z] = slot(id, 12);
            let a = z.atan2(x).rem_euclid(std::f32::consts::TAU);
            seen[(a / std::f32::consts::TAU * 8.0) as usize % 8] = true;
        }
        assert!(seen.iter().filter(|&&s| s).count() >= 5);
    }

    #[test]
    fn members_join_before_they_would_alone() {
        use crate::needs::{THIRSTY, hunger_score_from, thirst_score_from};
        // At 85% water a member drinks with its leader, and not on its own; full, it doesn't.
        assert!(thirst_score_from(0.85, JOIN_DRINK, false) > 0.0);
        assert_eq!(thirst_score_from(0.85, THIRSTY, false), 0.0);
        assert_eq!(thirst_score_from(0.95, JOIN_DRINK, false), 0.0);
        assert!(hunger_score_from(0.8, JOIN_EAT, false) > 0.0);
    }

    #[test]
    fn the_lowest_leads() {
        assert_eq!(leader([9, 4, 7]), 4);
        assert_eq!(leader([]), 0);
    }
}

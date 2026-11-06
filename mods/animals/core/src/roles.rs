//! Herds and packs with roles (`reports/Creature AI.md`, part 3, step 5 of the report's plan, the
//! crew's step 6): the numbers the mod reads (`mods/animals/src/senses.rs`, `hunt.rs`).
//!
//! - **Alarm.** The first of a herd or pack to notice a hunter and run calls: its kin within
//!   `ALARM` blocks know at once and run with it (alarm calls carry a warning faster than sight).
//! - **The selfish herd.** A herd animal runs away from the hunter and toward the herd's middle
//!   (`SELFISH` of the way), putting the others between (Hamilton 1971).
//! - **Sentinels.** In a herd of `SENTINEL_HERD` or more, one adult at a time is on watch (the
//!   watch moves on every `SENTINEL_TURN` ticks): it stands with its head up and notices
//!   `SENTINEL` times as fast and far (the many-eyes effect).
//! - **Standing ground.** A big plant eater (`STANDS_KG` or more) faces a lone hunter instead of
//!   running, and a mother of `GUARDS_KG` or more stays between a hunter and her young; either
//!   charges one that comes within `CHARGE` blocks.
//! - **Packs.** A pack's hunter picks the weak: the young and the hurt count as nearer
//!   (`prey_score`). Members other than the driver run wide at the prey's flanks
//!   (`FLANK` blocks to a side, ahead of where it is going), all aiming where it will be
//!   (`LEAD_SECS` ahead at most).
//! - **Ambush.** Cats and crocodilians stalk slowly to within `BURST` blocks, quiet as a sneak, then
//!   charge, giving up a charge after `BURST_TICKS`.

/// Blocks an alarm call carries to kin.
pub const ALARM: f64 = 24.0;
/// Ticks an animal that heard an alarm doesn't call one itself (no echo through the herd).
pub const ALARM_QUIET: u64 = 60;

/// The share of a herd animal's flight toward its herd's middle.
pub const SELFISH: f32 = 0.5;

/// Kin near (itself among them) from which a herd keeps a sentinel, ticks its watch lasts, and how
/// much faster and farther a sentinel notices.
pub const SENTINEL_HERD: u16 = 3;
pub const SENTINEL_TURN: u64 = 600;
pub const SENTINEL: f32 = 1.4;
/// What standing watch scores in the needs band (`needs`): over wandering, under a rest.
pub const WATCH: f32 = 0.42;

/// Mass, kg, from which a plant eater faces a lone hunter, and a mother guards her young; blocks
/// her young may be from her, a hunter's packmates may be from it for it to count as lone, and
/// within which she charges.
pub const STANDS_KG: f32 = 150.0;
pub const GUARDS_KG: f32 = 30.0;
pub const YOUNG_NEAR: f64 = 10.0;
pub const LONE: f64 = 10.0;
pub const CHARGE: f64 = 3.5;
/// A charge's hit, as a share of the hunter's max health, and the speed it charges at (times its
/// kind's).
pub const CHARGE_HIT: f32 = 0.15;
pub const CHARGE_SPEED: f32 = 1.5;

/// Social moments in an animal's idle life (`anim`, every 15-40 s while it stands idle), by its
/// kind's `biology::Social`: a young one's moment is play, in percent, more often in a pack or
/// herd (pups, foals and kids play with each other; a lone kind's young less); a mother's is
/// nuzzling one of her young when one is within `NUZZLE_NEAR` blocks; otherwise it looks round,
/// sniffs, scratches or grooms, and a pack's or herd's grooming comes up twice as often.
pub const PLAY_GROUP: u32 = 60;
pub const PLAY_ALONE: u32 = 35;
pub const NUZZLE: u32 = 50;
pub const NUZZLE_NEAR: f64 = 4.0;

/// How far a young or hurt prey counts as from a pack's hunter, times its distance: the young at
/// `YOUNG`, the hurt by how hurt (half at no health left).
pub const YOUNG: f32 = 0.5;
pub fn prey_score(d2: f64, young: bool, health_share: f32) -> f64 {
    d2 * if young { YOUNG as f64 } else { 1.0 } * (0.5 + 0.5 * health_share.clamp(0.0, 1.0)) as f64
}
/// Blocks to a side a flanker runs from the prey's path, from how far it closes straight in, and
/// the most seconds ahead a chaser aims.
pub const FLANK: f64 = 5.0;
pub const FLANK_FROM: f64 = 7.0;
pub const LEAD_SECS: f64 = 1.0;

/// The families that ambush, blocks from which they charge, the speed they stalk at (times their
/// kind's), and ticks a charge lasts before they give up.
pub const AMBUSHERS: [&str; 3] = ["Felidae", "Alligatoridae", "Neosuchia"];
pub const BURST: f64 = 9.0;
pub const STALK: f32 = 0.45;
pub const BURST_TICKS: u64 = 200;

/// Personality (`coats::personality`: temperament, curiosity, fierceness, each 0..1, 0.5 middling).
/// A fierce hunter starts hunting up to `FIERCE_HUNGER` sooner (of its food) and keeps at a chase
/// up to `FIERCE_CHASE` times as long (a timid one that much less); a fierce plant eater stands its
/// ground from `FIERCE_STAND` of `STANDS_KG`; a skittish one notices hunters up to `SKITTISH`
/// times as fast (a calm one that much slower).
pub const FIERCE_HUNGER: f32 = 0.15;
pub const FIERCE_CHASE: f32 = 0.5;
pub const FIERCE_STAND: f32 = 0.6;
pub const SKITTISH: f32 = 0.4;
/// A curious one (curiosity past `CURIOUS`) comes to look at a player standing within `LOOKS_FROM`
/// times its shy distance, to `COMES_TO` of it, while it is calm; what that scores in the needs
/// band, at full curiosity.
pub const CURIOUS: f32 = 0.6;
pub const LOOKS_FROM: f32 = 1.5;
pub const COMES_TO: f32 = 0.5;
pub const INVESTIGATE: f32 = 0.4;

/// `v` (0..1, 0.5 middling) as -1..1.
pub fn lean(v: f32) -> f32 {
    (v - 0.5) * 2.0
}

pub fn ambushes(family: &str) -> bool {
    AMBUSHERS.contains(&family)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weak_first() {
        // A young deer 10 blocks off before a grown one 8 off; a badly hurt one before a whole one.
        assert!(prey_score(100.0, true, 1.0) < prey_score(64.0, false, 1.0));
        assert!(prey_score(100.0, false, 0.2) < prey_score(80.0, false, 1.0));
        assert!(WATCH < crate::needs::REST && WATCH > crate::needs::WANDER);
    }
}

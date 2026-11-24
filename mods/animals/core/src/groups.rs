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
/// Resting in step (slice 3): a member whose leader lies down lies down too, on average this many
/// ticks after (one chance in this many a tick: about 5 s), and one lying while its leader is up
/// and about gets up, about 3 s after. Red deer in a group lie down and rise nearly together
/// (Clutton-Brock, Guinness and Albon 1982; synchrony in grazers, Rook and Penning 1991), so a
/// herd rests as one and moves on as one.
pub const JOIN_REST_IN: u32 = 100;
pub const JOIN_UP_IN: u32 = 60;
// Within a few seconds at 20 ticks a second, and up quicker than down.
const _: () = assert!(JOIN_REST_IN <= 200 && JOIN_UP_IN <= JOIN_REST_IN);

/// How pressing joining is, at least, once it is under those (the band's score for drinking or
/// eating): above a midday rest (`needs::REST` plus `routines::SLEEPY` for a twilight animal at its
/// lowest, held at `KEEP_AT`), so a resting member gets up to go with its herd, under a night's
/// sleep. ai1122_drink1: at the drink's own score (about 0.35) the herd lay on while its leader
/// drank.
pub const JOIN_PULL: f32 = 0.85;
const MIDDAY_REST: f32 = (crate::needs::REST + crate::routines::SLEEPY * (1.0 - 0.35)) * crate::needs::KEEP_AT;
const NIGHT_SLEEP: f32 = (crate::needs::REST + crate::routines::SLEEPY) * crate::needs::KEEP_AT;
const _: () = assert!(JOIN_PULL > MIDDAY_REST && JOIN_PULL < NIGHT_SLEEP);

// Joining starts before it would on its own (and before the dusk drink), and stops where it would.
const _: () = assert!(
    JOIN_DRINK > crate::routines::DRINKS_AT
        && JOIN_DRINK > crate::needs::THIRSTY
        && JOIN_DRINK < crate::needs::SLAKED
        && JOIN_EAT > crate::needs::HUNGRY
        && JOIN_EAT < crate::needs::FULL
);

/// Migration's first step: one that finds no water within its look round (`drink.rs`, 16 blocks)
/// goes to the nearest water it or a wild one of its kind near remembers drinking at, up to this
/// many blocks off; its herd goes with it (a member keeps to its place round its leader, and
/// drinks with it). Herds of the dry seasons walk tens of kilometres to the water they know
/// (wildebeest and zebra in the Serengeti, red deer to their summer and winter ranges): here a few
/// chunks, the reach of what one herd shares.
pub const WATER_TREK: f32 = 96.0;
// Further than it looks or goes back on its own (`drink.rs` LOOK 16, RECALL 32).
const _: () = assert!(WATER_TREK > 32.0);

/// A wild drinker keeps within this many blocks of the nearest water it knows (`drink::home`):
/// wandering further, it turns back toward it; and a herd's leader leads it only to ground this
/// near water its herd knows (`migrate.rs`). Grazers that drink stay within reach of water in
/// the dry season (zebra and wildebeest within a day's walk of it, Redfern et al. 2003, *Ecology*
/// 84; savanna elephants and buffalo the same) and the dry-season migrations follow the rivers;
/// half the trek here, so a thirsty one is always within its trek of what it knows.
/// Without it a winter savanna's deer grazed and wandered 130-145 blocks from the only pool and
/// died parched (ai1124_graze2).
pub const WATER_RANGE: f32 = 48.0;
const _: () = assert!(2.0 * WATER_RANGE <= WATER_TREK && WATER_RANGE >= MIGRATE_RINGS[0]);

/// The dry season's migration (slice 5; `notes/ECOLOGY.md`, "Later": herds move towards wetter
/// ground in the dry season). A grazing herd's leader in its place's dry season
/// (`vitals::climate::dry_season`) looks this often (ticks: a game minute) at the rain of the
/// places round it, `MIGRATE_DIRS` ways at each of `MIGRATE_RINGS` blocks, and leads its herd
/// to the wettest that rains on `MIGRATE_GAIN` more of its days than here, a leg of `MIGRATE_LEG`
/// blocks at a time (waiting for its herd, `MIGRATE_WAIT`), until it is within `MIGRATE_ARRIVED` of it, the season here turns, or
/// `MIGRATE_GIVE_UP` ticks have gone. Wildebeest and zebra follow the rains round the Serengeti
/// (Boone, Thirgood and Hopcraft 2006, *Ecology* 87), the llanos' deer and capybara crowd to the
/// rivers and the last pools in the dry months: here over a few chunks.
pub const MIGRATE_EVERY: u64 = 1200;
pub const MIGRATE_RINGS: [f32; 2] = [48.0, 96.0];
pub const MIGRATE_DIRS: u32 = 8;
pub const MIGRATE_GAIN: f32 = 0.15;
pub const MIGRATE_LEG: f32 = 12.0;
pub const MIGRATE_ARRIVED: f32 = 12.0;
pub const MIGRATE_GIVE_UP: u64 = 6000;
/// The leader waits before its next leg while its herd's middle (`senses::Senses::kin_mid`,
/// itself among them) is more than this many blocks behind it; and the herd goes on as pressingly
/// as it joins its leader to drink (`JOIN_PULL`: above a daytime rest, under a night's sleep).
/// ai1123_migrate1: going on only from the wander, the herd lay down at mid morning, and the
/// leader walked off alone (24-block legs, no waiting) out of its members' 16 blocks.
pub const MIGRATE_WAIT: f32 = 4.0;
pub const MIGRATE_PULL: f32 = JOIN_PULL;
/// A thirsty one leaves the march to drink (`migrate_pull`): a migration pulls this share of
/// its thirst's score, under it even while marching (`needs::KEEP_AT`), so it goes to water near
/// or treks back to water its herd remembers (`drink::trek`), and the leader waits for it. Pulling
/// above thirst, a herd led out of the llanos' winter followed its leader past its last pool and
/// died parched within two days (ai1124_savanna3: four of five deer).
pub const MIGRATE_THIRST: f32 = 0.75;
const _: () = assert!(MIGRATE_THIRST * crate::needs::KEEP_AT < 1.0);

/// How hard a migration pulls one whose thirst scores `thirst` (`needs::thirst_score`; 0 not
/// thirsty): `MIGRATE_PULL`, or under its thirst when it is thirsty.
pub fn migrate_pull(thirst: f32) -> f32 {
    if thirst > 0.0 { MIGRATE_PULL.min(thirst * MIGRATE_THIRST) } else { MIGRATE_PULL }
}
// A leg ends inside the distance its members count it within (`senses::KIN` 16, less the ring),
// and it gets there before giving up (6000 ticks: five minutes; 96 blocks at a deer's walk with
// its waits is two or three).
const _: () = assert!(MIGRATE_LEG + SLOT_FAR / 2.0 <= 16.0 && MIGRATE_ARRIVED < MIGRATE_RINGS[0] && MIGRATE_GIVE_UP >= 20 * 120);

/// The places a leader at `(x, z)` looks at, nearest ring first.
pub fn migrate_looks(x: f32, z: f32) -> impl Iterator<Item = [f32; 2]> {
    MIGRATE_RINGS.into_iter().flat_map(move |r| {
        (0..MIGRATE_DIRS).map(move |i| {
            let a = i as f32 / MIGRATE_DIRS as f32 * std::f32::consts::TAU;
            [x + a.cos() * r, z + a.sin() * r]
        })
    })
}

/// Where a herd goes from a place where it rains on `here` of the season's days: the wettest of
/// `places` (where, and its share) that is `MIGRATE_GAIN` wetter, the nearer first among equals;
/// None: nowhere is worth the walk.
pub fn wetter(here: f32, places: impl IntoIterator<Item = ([f32; 2], f32)>) -> Option<[f32; 2]> {
    let mut best: Option<([f32; 2], f32)> = None;
    for (p, rain) in places {
        if rain >= here + MIGRATE_GAIN && best.is_none_or(|(_, b)| rain > b) {
            best = Some((p, rain));
        }
    }
    best.map(|(p, _)| p)
}

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
    fn herds_go_only_where_it_is_wetter_enough() {
        let looks: Vec<_> = migrate_looks(0.0, 0.0).collect();
        assert_eq!(looks.len(), (MIGRATE_DIRS as usize) * MIGRATE_RINGS.len());
        // The llanos' winter (0.11) against the forest's (0.41): it goes; against the plains' (0.14): not.
        assert_eq!(wetter(0.11, [([48.0, 0.0], 0.14)]), None);
        assert_eq!(wetter(0.11, [([48.0, 0.0], 0.14), ([0.0, 96.0], 0.41)]), Some([0.0, 96.0]));
        // Equally wet: the nearer (looked at first).
        assert_eq!(wetter(0.1, [([48.0, 0.0], 0.4), ([96.0, 0.0], 0.4)]), Some([48.0, 0.0]));
    }

    #[test]
    fn the_lowest_leads() {
        assert_eq!(leader([9, 4, 7]), 4);
        assert_eq!(leader([]), 0);
    }

    /// A thirsty one leaves the march for water even while marching; one not thirsty marches.
    #[test]
    fn thirst_goes_before_the_march() {
        use crate::needs::{KEEP_AT, THIRSTY, thirst_score};
        assert_eq!(migrate_pull(thirst_score(0.9, false)), MIGRATE_PULL);
        for water in [THIRSTY - 0.01, 0.3, 0.1, 0.0] {
            let thirst = thirst_score(water, false);
            assert!(migrate_pull(thirst) * KEEP_AT < thirst, "at {water} water the march pulls over thirst");
        }
    }
}

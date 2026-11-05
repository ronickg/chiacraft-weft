//! How animals notice what is near them (`reports/Creature AI.md`, part 3, step 4): the numbers
//! the mod's senses read (`mods/animals/src/senses.rs`). An animal no longer knows of a hunter or
//! a walker the moment it is in range: it builds up awareness of it, from what it sees (in its
//! field of view, with nothing solid between, less in the dark and of one sneaking), hears (a
//! sprint, a walk, a sneak much less; blocks breaking and fights near) and smells (downwind of it,
//! by the wind's way, `vitals::air::heading`), and acts when that reaches `NOTICED`. Unnoticed it
//! fades again. Half way it stops and stares (`STARE`): the player sees it has noticed before it
//! bolts. Splinter Cell: Blacklist's awareness (Walsh, GDC 2014), Tu and Terzopoulos's fish
//! (a field of view, occlusion), RDR2's scent on the wind.

/// The field of view, degrees, of prey (eyes on the sides of the head: only a blind spot behind)
/// and of hunters (eyes in front).
pub const PREY_FIELD: f32 = 300.0;
pub const HUNTER_FIELD: f32 = 200.0;

/// Awareness at which it acts (runs), and at which it stops to stare.
pub const NOTICED: f32 = 1.0;
pub const STARE: f32 = 0.4;
/// Its awareness once it has run: alert, staring if it still senses the danger, bolting again
/// only if it is noticed again (it ran from deer 30 blocks off while its awareness faded).
pub const ALERT: f32 = 0.6;
/// Awareness a second at the edge of its senses and close by (it rises faster the nearer: as
/// `1 - d / range`, plus this floor so a walker at the edge still gets noticed in time).
pub const CLOSE_PER_SEC: f32 = 2.5;
pub const EDGE_PER_SEC: f32 = 0.35;
/// How fast it fades with nothing to notice, a second.
pub const FADES_PER_SEC: f32 = 0.2;

/// How far it sees a walker or a hunter in the open by day, as a share of its watch (the mod's
/// `hunt::WATCH`, or a species' shy distance): crouching shrinks what it sees of one, and so does
/// the dark (a light of 0 sees this share at most).
pub const SNEAK_SEEN: f32 = 0.3;
pub const DARK_SEEN: f32 = 0.45;

/// Blocks a sound carries: a sprint, a walk, a sneak (quiet, not silent), a block broken, a
/// fight (a hurt, a kill). Through solid ground between (the ray blocked) it carries half as far.
pub const SPRINT_HEARD: f32 = 18.0;
pub const WALK_HEARD: f32 = 9.0;
pub const SNEAK_HEARD: f32 = 2.5;
pub const BREAK_HEARD: f32 = 16.0;
pub const FIGHT_HEARD: f32 = 20.0;
pub const MUFFLED: f32 = 0.5;
/// Ticks a block broken or a fight is still heard.
pub const NOISE_TICKS: u64 = 20;

/// Blocks it smells something downwind of it in a wind (straight downwind; across the wind a
/// quarter of it, upwind nothing), and all round in still air.
pub const SCENT: f32 = 28.0;
pub const SCENT_STILL: f32 = 6.0;
/// The wind, m/s, from which scent goes with it (under it, still air).
pub const WIND_CARRIES: f32 = 1.0;

/// Whether `to` (a direction, any length, x and z) is within a field of view of `field` degrees
/// round `facing` (the way it faces, x and z).
pub fn in_view(facing: [f32; 2], to: [f32; 2], field: f32) -> bool {
    let (lf, lt) = (facing[0].hypot(facing[1]), to[0].hypot(to[1]));
    if lf < 1e-6 || lt < 1e-6 {
        return true;
    }
    let cos = (facing[0] * to[0] + facing[1] * to[1]) / (lf * lt);
    cos >= (field.to_radians() / 2.0).cos()
}

/// How far it smells a scent from `from` (x and z, the source's offset from it: where the smell
/// comes from), with the wind blowing toward `heading` (degrees clockwise from north, -z) at
/// `wind` m/s.
pub fn scent_reach(from: [f32; 2], heading: f32, wind: f32) -> f32 {
    if wind < WIND_CARRIES {
        return SCENT_STILL;
    }
    let h = heading.to_radians();
    // The wind's way, and the way from the source to the animal (the smell's way).
    let (wx, wz) = (h.sin(), -h.cos());
    let l = from[0].hypot(from[1]).max(1e-6);
    let (sx, sz) = (-from[0] / l, -from[1] / l);
    let down = wx * sx + wz * sz;
    if down <= 0.0 {
        return SCENT_STILL * 0.5;
    }
    SCENT_STILL.max(SCENT * (0.25 + 0.75 * down) * down.sqrt())
}

/// How much awareness a second something gives at `d` blocks of a sense that reaches `range`.
pub fn gain(d: f32, range: f32) -> f32 {
    if range <= 0.0 || d >= range {
        return 0.0;
    }
    EDGE_PER_SEC + (CLOSE_PER_SEC - EDGE_PER_SEC) * (1.0 - d / range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downwind_smells_far_upwind_not() {
        // The wind blows north (toward -z). A source south of it (+z) is upwind: its smell comes.
        let from_south = scent_reach([0.0, 10.0], 0.0, 5.0);
        let from_north = scent_reach([0.0, -10.0], 0.0, 5.0);
        assert!(from_south >= SCENT * 0.9, "{from_south}");
        assert!(from_north < SCENT_STILL, "{from_north}");
        assert_eq!(scent_reach([0.0, 10.0], 0.0, 0.2), SCENT_STILL);
    }

    #[test]
    fn a_blind_spot_behind() {
        assert!(in_view([0.0, -1.0], [0.3, -1.0], PREY_FIELD));
        assert!(in_view([0.0, -1.0], [1.0, 0.2], PREY_FIELD));
        assert!(!in_view([0.0, -1.0], [0.0, 1.0], PREY_FIELD));
        assert!(!in_view([0.0, -1.0], [1.0, 0.2], 120.0));
    }

    #[test]
    fn nearer_is_noticed_faster() {
        assert!(gain(1.0, 10.0) > gain(9.0, 10.0));
        assert_eq!(gain(11.0, 10.0), 0.0);
        // A sprinter at the edge of hearing is noticed within about three seconds.
        assert!(gain(SPRINT_HEARD - 0.5, SPRINT_HEARD) * 3.0 >= NOTICED);
    }
}

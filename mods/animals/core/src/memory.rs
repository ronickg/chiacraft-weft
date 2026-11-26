//! What animals remember (`reports/Creature AI.md`, part 3, step 5): the numbers the mod's memory
//! reads (`mods/animals/src/memory.rs`).
//!
//! - **Each animal** keeps a few memories of players: who hurt it (it notices them sooner and
//!   runs sooner) and who fed it (it lets them closer). Saved with it.
//! - **Each place** keeps its danger: a death there (a kill, a hunt's end) or a hurt leaves fear
//!   in its `DANGER_CELL`-block square that fades over game days. Grazers keep off ground that
//!   frightened them unless hunger or thirst drives them; a hungry hunter that sees no prey goes
//!   where kills were (Shadow of Mordor's and Rain World's memory, Mark's influence maps).
//! - **Each species in each area** keeps a reputation with each player (Rain World): hunting a
//!   valley's deer makes its deer warier of that hunter (their shy distance and how fast they
//!   notice him, `wariness`), feeding them calms them. It fades over weeks.

/// Memories of players each animal keeps (the oldest goes when a new one comes).
pub const MEMORIES: usize = 4;
/// How much sooner it notices a player who hurt it, and how much later one who fed it (its
/// awareness a second, times).
pub const HURT_ME: f32 = 1.6;
pub const FED_ME: f32 = 0.5;
/// Game days a memory of a player lasts.
pub const REMEMBERS_DAYS: f32 = 10.0;

/// Blocks a square of danger is wide.
pub const DANGER_CELL: i32 = 8;
/// Fear a death leaves on its square, and a hurt.
pub const DEATH_FEAR: f32 = 1.0;
pub const HURT_FEAR: f32 = 0.25;
/// Game days over which fear fades to a third (e-folding).
pub const DANGER_DAYS: f32 = 3.0;
/// Fear on a square from which a grazer keeps off it, and the hunger or thirst score
/// (`needs::hunger_score`) past which it goes anyway.
pub const AVOID: f32 = 0.6;
pub const DRIVEN: f32 = 0.6;
/// Blocks a hungry hunter that sees no prey looks round for a square where kills were, and the
/// fear one needs to draw it.
pub const HAUNT_LOOK: i32 = 32;
pub const HAUNT: f32 = 0.8;

/// Blocks an area of reputation is wide.
pub const REPUTATION_AREA: i32 = 128;
/// What a kill, a hurt and a feeding do to a species' reputation of a player in an area (negative:
/// feared).
pub const KILLED: f32 = -1.0;
pub const HURT: f32 = -0.3;
pub const FED: f32 = 0.25;
/// Game days over which a reputation fades to a third.
pub const REPUTATION_DAYS: f32 = 14.0;

/// What a fear of `fear` left `days` ago is now.
pub fn faded(fear: f32, days: f32, over: f32) -> f32 {
    fear * (-days.max(0.0) / over).exp()
}

/// How a reputation `rep` changes an animal's wariness of that player: its shy distance and how
/// fast it notices him, times this (up to 2 when hated, down to 0.6 when liked).
pub fn wariness(rep: f32) -> f32 {
    if rep < 0.0 { 1.0 + (-rep).min(1.0) } else { 1.0 - 0.4 * rep.min(1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fear_fades_and_reputation_bites() {
        assert!(faded(DEATH_FEAR, 0.0, DANGER_DAYS) > AVOID);
        assert!(faded(DEATH_FEAR, DANGER_DAYS * 2.0, DANGER_DAYS) < AVOID * 0.25);
        assert_eq!(wariness(0.0), 1.0);
        assert_eq!(wariness(KILLED * 3.0), 2.0);
        assert!(wariness(FED * 4.0) < 1.0);
    }
}

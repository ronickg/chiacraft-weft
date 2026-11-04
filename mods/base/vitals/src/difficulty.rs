//! How hard survival is, set per world (Valheim's world modifiers): gentle, normal, harsh.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Difficulty {
    pub name: &'static str,
    /// How fast food digests (1 normal).
    pub digest: f32,
    /// How long food keeps (1 normal; more keeps longer).
    pub keep: f32,
    /// How strongly cold, heat, poison and bleeding hurt.
    pub harm: f32,
    /// Whether cold, heat and poison can kill; if not, they stop at a tenth of max health.
    pub deadly: bool,
    /// The share of the XP into the current level lost on dying.
    pub death_xp: f32,
    /// What actions cost in stamina.
    pub stamina_cost: f32,
    /// A new world's first nights with no monsters appearing by themselves (gentle's grace: the
    /// first-hour report, so a beginner learns fire and clothes before the first zombie).
    pub grace_nights: u32,
}

pub const GENTLE: Difficulty =
    Difficulty { name: "gentle", digest: 0.75, keep: 1.5, harm: 0.5, deadly: false, death_xp: 0.1, stamina_cost: 0.8, grace_nights: 3 };
pub const NORMAL: Difficulty =
    Difficulty { name: "normal", digest: 1.0, keep: 1.0, harm: 1.0, deadly: true, death_xp: 0.25, stamina_cost: 1.0, grace_nights: 0 };
pub const HARSH: Difficulty =
    Difficulty { name: "harsh", digest: 1.25, keep: 0.6, harm: 1.5, deadly: true, death_xp: 0.5, stamina_cost: 1.15, grace_nights: 0 };

pub const ALL: [Difficulty; 3] = [GENTLE, NORMAL, HARSH];

pub fn named(name: &str) -> Difficulty {
    ALL.into_iter().find(|d| d.name == name).unwrap_or(NORMAL)
}

/// Seconds after respawning that monsters leave a player be, or until they reach their tombstone
/// (Valheim's corpse run; Research round 4 and the first-hour report).
pub const GRACE_SECS: f32 = 180.0;
/// Blocks from the tombstone that count as reaching it.
pub const GRAVE_REACHED: f64 = 3.0;

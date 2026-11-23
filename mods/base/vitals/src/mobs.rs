//! Base's hostile mobs on the one curve (`combat`), as the animals are (`animals_core::danger`):
//! a size class from mass, a role and a home age give health, armour and a hit; a tweak (x0.75 to
//! x1.25, with its reason) shapes one stat. They spawn anywhere at night, so they fight at their
//! age's start: a fresh Stone Age player meets them in their size's band.

use crate::combat::{self, Role, Size, Stats};
use crate::curve::{Age, Tweak};

/// Whom a mob's hit is priced against: its size's hits to down (`combat::Band::hits`) for
/// that player. Night one's mobs meet a fresh player, not the reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meets {
    /// A fresh player: base health (`combat::REF_HEALTH`), no armour. Planner, 11-22 (option A
    /// after the first night run: a zombie's 6 killed a fresh player in 3 hits).
    Fresh,
    /// The age's reference player, as the animals are priced (option B: armour is needed
    /// from night one).
    Reference,
}

/// What every mob's hit is priced against (switch to `Meets::Reference` for option B).
pub const MEETS: Meets = Meets::Fresh;

/// The reference player of its age in that age's armour takes at least this many of a mob's hits
/// to go down, while they meet a fresh player (the check: armour pays off against them).
pub const ARMOURED_HITS: f32 = 12.0;

/// Which of a mob's numbers a tweak shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Health,
    Hit,
    Armour,
}

/// One mob's row.
#[derive(Clone, Copy, Debug)]
pub struct Mob {
    /// Its entity kind's name (`base:<name>`).
    pub name: &'static str,
    /// Kilograms, a real body of its build: its size class.
    pub mass: f32,
    pub role: Role,
    /// The age it fights at the start of.
    pub home: Age,
    pub tweaks: &'static [(Stat, Tweak)],
}

const fn m(name: &'static str, mass: f32, role: Role) -> Mob {
    Mob { name, mass, role, home: Age::Stone, tweaks: &[] }
}

pub const MOBS: &[Mob] = &[
    // A walking corpse of a grown man.
    m("zombie", 70.0, Role::Hunter),
    Mob {
        tweaks: &[(Stat::Health, Tweak::new(0.8, "light and fast: a leg-span of 1.3 m on a small body"))],
        ..m("spider", 40.0, Role::Hunter)
    },
    Mob {
        // Bones only: it keeps its distance, so its arrow is its hit, and it breaks easily.
        tweaks: &[
            (Stat::Health, Tweak::new(0.8, "bones only, it breaks easily")),
            (Stat::Hit, Tweak::new(0.8, "it shoots from range, never in reach")),
        ],
        ..m("skeleton", 30.0, Role::Hunter)
    },
    // Its blast is its own (`events`); this is what it takes to bring down.
    m("creeper", 50.0, Role::Prey),
];

/// What each drops when it dies (vanilla's amounts): mob, item (`base:<item>`), fewest, most, the
/// chance of the roll (a spider's eye only when a player killed it). `mods/base/src/mobs.rs`
/// builds their loot tables from these; the loot check prices them.
pub const LOOT: &[(&str, &str, u32, u32, f32)] = &[
    ("zombie", "rotten_flesh", 0, 2, 1.0),
    ("spider", "string", 0, 2, 1.0),
    ("spider", "spider_eye", 1, 1, 0.33),
    ("creeper", "gunpowder", 0, 2, 1.0),
    ("skeleton", "arrow", 0, 2, 1.0),
    ("skeleton", "bone", 0, 2, 1.0),
];

/// A mob's row.
pub fn mob(name: &str) -> Option<&'static Mob> {
    MOBS.iter().find(|m| m.name == name)
}

impl Mob {
    pub fn size(&self) -> Size {
        Size::of_mass(self.mass)
    }

    /// Its tweak on `stat` (1 if none).
    pub fn tweak(&self, stat: Stat) -> f32 {
        self.tweaks
            .iter()
            .filter(|(s, _)| *s == stat)
            .map(|(_, t)| t.x)
            .product()
    }

    /// The area level it fights at.
    pub fn level(&self) -> f32 {
        self.home.start()
    }

    /// Its numbers (engine points): derived, then tweaked.
    pub fn stats(&self) -> Stats {
        let mut st = combat::derive(self.size(), self.role, self.level());
        if MEETS == Meets::Fresh {
            // The same hits to down, against base health and no armour, times its role's hit.
            st.hit = combat::hit_to_down(self.size().band().hits, combat::REF_HEALTH, 0.0) * self.role.times().1;
        }
        st.health *= self.tweak(Stat::Health);
        st.hit *= self.tweak(Stat::Hit);
        st.armour *= self.tweak(Stat::Armour);
        st
    }
}

/// Hits of `m`'s that down the reference player of its home age, in that age's armour.
pub fn armoured_hits(m: &Mob) -> f32 {
    let p = combat::Player::of(m.home);
    let hit = m.stats().hit;
    p.health() / (hit * (1.0 - combat::armour_cut(p.armour(), hit))).max(1e-6)
}

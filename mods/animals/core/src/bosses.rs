//! Biome bosses (`notes/PROGRESSION.md`, "Biome bosses" and round 6): a giant native creature of
//! each biome, summoned at an altar with the biome's offerings so the player picks when to fight,
//! a harder form for each later age, random loot with bad-luck help, and a young one to tame after
//! a win. Each declares a few facts; its numbers come from the curve like every creature's
//! (`vitals::combat`, `danger`): a boss is a titan, so a same-age player fights it 2-5 minutes,
//! over-tier gear counts at most 5 levels over it, and it loses at most its health over 120 s a
//! second whatever hits it (the floor). What the floor holds back becomes its stagger: knockdowns
//! and part breaks.
//!
//! The first four (Ronald, 11-07): Borkon, Drenkaun, Askaun and Nokaun. Drenkaun is built first
//! (Planner, 11-05: old forests are near most spawns, and it is a Stone Age boss).

use vitals::combat::{self, Role, Size, Stats};
use vitals::curve::{Age, Tweak};

use crate::danger::Stat;

/// One age's form: the offering that calls it (`None`: not built yet, data for later) and the
/// item-level depth its drops carry (`vitals::curve::item_level`).
#[derive(Clone, Copy, Debug)]
pub struct Form {
    pub age: Age,
    pub offering: Option<&'static str>,
    pub depth: u8,
}

/// One of its attacks: its hit as a share of the form's (`Stats::hit`), how far it reaches past
/// its body, seconds of warning before it lands (the tell a player reads) and before it comes
/// again, and the knockback it gives, in blocks a second.
#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub name: &'static str,
    pub hit: f32,
    pub reach: f32,
    pub windup: f32,
    pub cooldown: f32,
    pub knockback: f32,
}

/// A part that breaks: when its stagger (a share of its health, everything over the floor and a
/// share of every hit) passes `at`, it drops `drop` and the attack it `weakens` hits a quarter
/// softer.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub name: &'static str,
    pub at: f32,
    pub drop: &'static str,
    pub count: u32,
    pub weakens: Option<&'static str>,
}

/// A line of its loot: rolled for each player who fought it, from the form `from` on. Bad-luck
/// help (round 1: "certain by a set kill count"): every kill without it raises the chance, so by
/// the `certain_by`-th kill it is certain (0: no help, its chance each time).
#[derive(Clone, Copy, Debug)]
pub struct Loot {
    pub item: &'static str,
    pub count: (u32, u32),
    pub chance: f32,
    pub certain_by: u32,
    pub from: Age,
}

/// A biome boss.
#[derive(Clone, Copy, Debug)]
pub struct Boss {
    /// Its name in `mods/base/lore/lexicon.toml` (`[name.<it>]`) and its entity's.
    pub name: &'static str,
    pub role: Role,
    /// The biomes it guards, as tags of base's biomes; its altar wants their trees round it.
    pub home: &'static str,
    pub tweaks: &'static [(Stat, Tweak)],
    pub forms: &'static [Form],
    pub attacks: &'static [Attack],
    pub parts: &'static [Part],
    pub loot: &'static [Loot],
    /// Its young after a win (`Loot::item` an entity kind), tamed with its offering.
    pub young: Loot,
    /// Stagger (a share of its health) that knocks it down, and for how many seconds.
    pub knockdown: f32,
    pub down_secs: f32,
    /// At what share of its health left it turns fiercer: its cooldowns shorter by `FIERCE`.
    pub fierce_at: f32,
}

/// Every hit's share that goes into stagger, on top of all that spills over the floor: a same-age
/// fight from full to dead gathers this much of its health, enough for its part breaks and three or
/// four knockdowns; over-tier gear, whose hits mostly spill over, gathers it faster.
pub const STAGGER_SHARE: f32 = 0.35;
/// Its cooldowns in its fierce phase.
pub const FIERCE: f32 = 0.7;
/// How much softer a broken part's attack hits.
pub const BROKEN: f32 = 0.75;

const fn form(age: Age, offering: Option<&'static str>, depth: u8) -> Form {
    Form { age, offering, depth }
}

const fn loot(item: &'static str, count: (u32, u32), chance: f32, certain_by: u32) -> Loot {
    Loot { item, count, chance, certain_by, from: Age::Stone }
}

pub static BOSSES: [Boss; 1] = [Boss {
    name: "drenkaun",
    role: Role::Apex,
    home: "forests",
    tweaks: &[(Stat::Health, Tweak::new(0.8, "a boss is fought in windows: its tells, knockdowns and the misses add the time back"))],
    // Only the Stone form is playable; the later ones wait for their ages' offerings.
    forms: &[
        form(Age::Stone, Some("animals:forest_offering"), 5),
        form(Age::Bronze, None, 5),
        form(Age::Iron, None, 5),
        form(Age::Steam, None, 5),
    ],
    attacks: &[
        // Its form's hit downs a same-age player in about one (a titan's band), so each attack
        // takes a share: two charges or three sweeps down them (round 1: "can still down you in
        // two hits").
        // Head down, antlers forward, a straight run that tramples what is in its path; it is
        // stunned if it runs into a trunk or a wall (the window to punish it).
        Attack { name: "charge", hit: 0.75, reach: 1.5, windup: 1.2, cooldown: 9.0, knockback: 14.0 },
        // A sideways sweep of the antlers in front of it.
        Attack { name: "sweep", hit: 0.5, reach: 4.5, windup: 0.8, cooldown: 3.5, knockback: 9.0 },
        // It rears and stamps: a ring round it, for whoever stands under or behind it.
        Attack { name: "stomp", hit: 0.4, reach: 3.5, windup: 0.9, cooldown: 7.0, knockback: 7.0 },
        // Roots burst up under a player it can see far off and hold them (slowed) a while.
        Attack { name: "roots", hit: 0.25, reach: 24.0, windup: 1.4, cooldown: 12.0, knockback: 2.0 },
    ],
    parts: &[
        Part { name: "left_antler", at: 0.10, drop: "animals:warden_antler", count: 1, weakens: Some("sweep") },
        Part { name: "right_antler", at: 0.20, drop: "animals:warden_antler", count: 1, weakens: Some("charge") },
        Part { name: "crown", at: 0.30, drop: "animals:warden_bark", count: 2, weakens: Some("roots") },
    ],
    loot: &[
        // The Stone Age key's boss part.
        loot("animals:warden_heartseed", (1, 1), 0.35, 3),
        loot("animals:warden_antler", (1, 2), 0.6, 2),
        loot("animals:warden_bark", (2, 4), 1.0, 0),
        loot("base:leather", (6, 10), 1.0, 0),
        loot("base:bone", (4, 8), 1.0, 0),
    ],
    young: loot("animals:drenkaun_calf", (1, 1), 0.3, 4),
    knockdown: 0.08,
    down_secs: 4.0,
    fierce_at: 0.5,
}];

/// A boss by name.
pub fn boss(name: &str) -> Option<&'static Boss> {
    BOSSES.iter().find(|b| b.name == name)
}

/// Every boss is a titan (`combat::Size::Titan`: "over 20 t, or any boss").
pub const SIZE: Size = Size::Titan;

impl Boss {
    /// Its form for `age` (the last there is past its last).
    pub fn form(&self, age: Age) -> &Form {
        self.forms
            .iter()
            .rev()
            .find(|f| f.age <= age)
            .unwrap_or(&self.forms[0])
    }

    /// The form `offering` calls, if any.
    pub fn form_for(&self, offering: &str) -> Option<&Form> {
        self.forms.iter().find(|f| f.offering == Some(offering))
    }

    /// The area level a form fights at: its age's middle, where a same-age reference player is.
    pub fn level(&self, f: &Form) -> f32 {
        f.age.reference()
    }

    /// A form's numbers: a titan of its role at its level, times its tweaks.
    pub fn stats(&self, f: &Form) -> Stats {
        let mut st = combat::derive(SIZE, self.role, self.level(f));
        st.health *= self.tweak(Stat::Health);
        st.armour *= self.tweak(Stat::Armour);
        st.hit *= self.tweak(Stat::Hit);
        st
    }

    pub fn tweak(&self, stat: Stat) -> f32 {
        self.tweaks
            .iter()
            .filter(|(s, _)| *s == stat)
            .map(|(_, t)| t.x)
            .product()
    }

    pub fn attack(&self, name: &str) -> Option<&Attack> {
        self.attacks.iter().find(|a| a.name == name)
    }

    /// The same-age reference fight against a form (`combat::fight`, the floor and cap held).
    pub fn reference_fight(&self, f: &Form) -> combat::Fight {
        combat::fight(combat::Player::of(f.age), SIZE, self.level(f), self.stats(f))
    }

    /// A fight against a form by a player of `age` (over-tier gear counts at most the cap).
    pub fn fight_with(&self, f: &Form, age: Age) -> combat::Fight {
        combat::fight(combat::Player::of(age), SIZE, self.level(f), self.stats(f))
    }

    /// Its loot for a form.
    pub fn loot_for(&self, f: &Form) -> impl Iterator<Item = &Loot> {
        self.loot.iter().filter(move |l| l.from <= f.age)
    }
}

impl Loot {
    /// Its chance after `misses` kills without it: rising straight from `chance` to certain at
    /// the `certain_by`-th kill.
    pub fn chance_after(&self, misses: u32) -> f32 {
        if self.certain_by <= 1 {
            return if self.certain_by == 1 { 1.0 } else { self.chance };
        }
        let steps = (self.certain_by - 1) as f32;
        (self.chance + (1.0 - self.chance) * misses as f32 / steps).min(1.0)
    }

    /// A count for a roll `r` in 0..1.
    pub fn count_at(&self, r: f32) -> u32 {
        let (lo, hi) = self.count;
        lo + ((hi - lo + 1) as f32 * r.clamp(0.0, 0.999)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stone_form_is_a_stone_age_boss_fight() {
        let b = boss("drenkaun").unwrap();
        let f = b.form(Age::Stone);
        let fight = b.reference_fight(f);
        // A titan's band: 2-5 minutes for a same-age player, two hits or so to down them.
        assert!((120.0..=300.0).contains(&fight.ttk), "{fight:?}");
        assert!(fight.hits >= 1.0 && fight.hits <= 2.5, "{fight:?}");
        // Over-tier gear can't hurry it past the floor.
        for age in [Age::Iron, Age::Steam, Age::Stars] {
            assert!(b.fight_with(f, age).ttk >= 120.0, "{age:?}");
        }
        for (_, t) in b.tweaks {
            assert!(t.ok(), "{t:?}");
        }
    }

    #[test]
    fn bad_luck_help_is_certain_by_its_count() {
        let b = boss("drenkaun").unwrap();
        for l in b.loot.iter().chain([&b.young]) {
            if l.certain_by > 0 {
                assert_eq!(l.chance_after(l.certain_by - 1), 1.0, "{}", l.item);
                assert!(l.chance_after(0) < 1.0 || l.chance >= 1.0);
            }
            assert_eq!(l.count_at(0.0), l.count.0);
            assert_eq!(l.count_at(1.0), l.count.1);
        }
    }

    #[test]
    fn parts_break_inside_one_fight() {
        let b = boss("drenkaun").unwrap();
        // A same-age fight gathers `STAGGER_SHARE` of its health in stagger: every part breaks.
        assert!(b.parts.iter().all(|p| p.at < STAGGER_SHARE));
        assert!(b.parts.windows(2).all(|w| w[0].at < w[1].at));
        for p in b.parts {
            assert!(p.weakens.is_none_or(|a| b.attack(a).is_some()), "{}", p.name);
        }
        // Only the Stone form has an offering yet; every later age has its form as data.
        assert!(b.forms[0].offering.is_some() && b.forms[0].age == Age::Stone);
        assert!(b.forms.windows(2).all(|w| w[0].age < w[1].age));
        assert!(b.stats(b.form(Age::Iron)).health > b.stats(b.form(Age::Stone)).health * 2.0);
    }
}

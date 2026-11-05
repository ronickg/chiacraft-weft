//! Combat's numbers, from the one curve (`curve::g`; `reports/Balance model.md`, Part 4;
//! Ronald's answers in `notes/PROGRESSION.md`, "The balance model"): the damage formula, damage
//! types, the reference player, size classes and roles, the time-to-kill (TTK) and hits-to-down
//! bands they set a creature's health, armour and hit from, the cap on how much over-tier gear
//! counts against big creatures, the damage-rate floors, and a fight simulator the checks run on
//! every gear age against every creature.
//!
//! Nothing in the game reads this yet (11-07): `mods/base/src/player.rs` keeps its armour maths
//! and the animals their stats until Ronald decides the gaps `PROGRESSION_SHEET.md` shows.
//!
//! Units: health, hits and armour in engine points (the vitals show health ×5, `HEALTH_SCALE`).

use crate::curve::{Age, g, levels_for};

// ------------------------------------------------------------------ the damage formula

/// Damage types: the physical ones go through armour, the rest through resistances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Cut,
    Pierce,
    Blunt,
    Fire,
    Cold,
    Shock,
    Poison,
    Ley,
}

pub const TYPES: [Type; 8] = [Type::Cut, Type::Pierce, Type::Blunt, Type::Fire, Type::Cold, Type::Shock, Type::Poison, Type::Ley];

impl Type {
    pub const fn physical(self) -> bool {
        matches!(self, Type::Cut | Type::Pierce | Type::Blunt)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Type::Cut => "cut",
            Type::Pierce => "pierce",
            Type::Blunt => "blunt",
            Type::Fire => "fire",
            Type::Cold => "cold",
            Type::Shock => "shock",
            Type::Poison => "poison",
            Type::Ley => "ley",
        }
    }
}

/// Armour's cut is `A / (A + K × hit)` (Path of Exile 1's form): it stops small hits well and big
/// ones badly, so a fox's bite bounces off armour a titan's stomp goes through, and it's
/// scale-free (armour and hits both growing ×1.6 an age give the same cut in every age).
pub const ARMOUR_K: f32 = 5.0;
/// The most armour cuts.
pub const ARMOUR_CAP: f32 = 0.8;
/// Resistances, from a weakness of -100 % (double) to +75 %.
pub const RESIST_RANGE: (f32, f32) = (-1.0, 0.75);
/// A weak point's share of a hit, and an armoured plate's (Monster Hunter's hitzones).
pub const WEAK_POINT: f32 = 1.5;
pub const PLATE: f32 = 0.5;
/// Penetration a pierce weapon has by its type (a share of the target's armour ignored).
pub const PIERCE_PEN: f32 = 0.3;

/// The share of a hit of `hit` that armour `armour` stops.
pub fn armour_cut(armour: f32, hit: f32) -> f32 {
    if armour <= 0.0 || hit <= 0.0 {
        return 0.0;
    }
    (armour / (armour + ARMOUR_K * hit)).min(ARMOUR_CAP)
}

/// The armour that cuts a hit of `hit` by `cut` (the cut's inverse).
pub fn armour_for(cut: f32, hit: f32) -> f32 {
    let cut = cut.clamp(0.0, ARMOUR_CAP);
    ARMOUR_K * hit * cut / (1.0 - cut)
}

/// One hit, before the target's defences.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub raw: f32,
    pub kind: Type,
    /// The part it lands on: 1 the default, `WEAK_POINT`, `PLATE`, or a part's own for the type.
    pub part: f32,
    /// A share of armour ignored, then a flat amount (picks and war hammers).
    pub pen: f32,
    pub pen_flat: f32,
}

impl Hit {
    pub const fn plain(raw: f32, kind: Type) -> Hit {
        Hit { raw, kind, part: 1.0, pen: 0.0, pen_flat: 0.0 }
    }
}

/// What a hit does to a target of `armour` and `resist` (its resistance to the hit's type, used
/// for the elemental types): the formula's steps 2-5 (`reports/Balance model.md`, 4.6). The
/// world's `difficulty.harm` is applied by the caller to damage taken by players.
pub fn damage(hit: Hit, armour: f32, resist: f32) -> f32 {
    let r = hit.raw * hit.part;
    if hit.kind.physical() {
        let a = (armour * (1.0 - hit.pen.clamp(0.0, 1.0)) - hit.pen_flat).max(0.0);
        r * (1.0 - armour_cut(a, r))
    } else {
        r * (1.0 - resist.clamp(RESIST_RANGE.0, RESIST_RANGE.1))
    }
}

// ------------------------------------------------------------------ the reference player

/// The reference player of an age: same-age gear at grade 4 (Sound), no marks, no perks, no food
/// buffs, measured at the age's middle (`Age::reference`). Calibrated to today: 20 engine points
/// of health (100 shown) and a stone sword's hit about a second at the Stone Age's start.
pub const REF_HIT: f32 = 6.0;
pub const REF_HEALTH: f32 = 20.0;
pub const REF_ARMOUR: f32 = 25.0;
/// Hits a second the reference lands.
pub const REF_RATE: f32 = 1.0;

/// A player (or what a creature meets) at a level, with a multiplier on all of it (grade and marks
/// at most `curve::GRADE_CEILING`).
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub level: f32,
    pub times: f32,
}

impl Player {
    pub const fn of(age: Age) -> Player {
        Player { level: age.reference(), times: 1.0 }
    }

    pub fn hit(&self) -> f32 {
        REF_HIT * g(self.level) * self.times
    }

    /// Health above the base 20 comes from the age's armour and the Defence and Survival web,
    /// never from a character level.
    pub fn health(&self) -> f32 {
        REF_HEALTH * g(self.level) * self.times
    }

    pub fn armour(&self) -> f32 {
        REF_ARMOUR * g(self.level) * self.times
    }
}

// ------------------------------------------------------------------ sizes and roles

/// Size class, from mass, so it's a fact and not a choice. Size changes how a fight goes, not only
/// how long: knockback, the over-tier cap, the rate floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size {
    Small,
    Medium,
    Large,
    Huge,
    /// Over 20 t, or any boss.
    Titan,
}

pub const SIZES: [Size; 5] = [Size::Small, Size::Medium, Size::Large, Size::Huge, Size::Titan];

/// What a same-age reference player meets in a size class (Ronald, 11-06: "as proposed").
#[derive(Clone, Copy, Debug)]
pub struct Band {
    /// Seconds of fighting: the band and the middle the stats are derived at.
    pub ttk: (f32, f32),
    pub mid: f32,
    /// The creature's hits that down a same-age player.
    pub hits: f32,
    /// Seconds between its hits.
    pub interval: f32,
    /// How much its armour cuts the reference's hit.
    pub cut: f32,
    /// How many levels above it a player's gear counts at most (`None`: no cap).
    pub cap: Option<f32>,
    /// The least seconds it takes to kill, whatever the gear: at most 1/floor of its health a
    /// second from all sources together (players, tames, fire, traps). Damage over the floor fills
    /// its stagger (knockdowns and part breaks), not its death.
    pub floor: Option<f32>,
    /// Knockback resistance (1: none).
    pub knockback: f32,
}

impl Size {
    /// The class of an animal of `kg`.
    pub fn of_mass(kg: f32) -> Size {
        match kg {
            m if m < 10.0 => Size::Small,
            m if m < 150.0 => Size::Medium,
            m if m < 2000.0 => Size::Large,
            m if m < 20000.0 => Size::Huge,
            _ => Size::Titan,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Medium => "medium",
            Size::Large => "large",
            Size::Huge => "huge",
            Size::Titan => "titan",
        }
    }

    /// Its band: a large dinosaur 20-40 s for a same-age player, a huge one 60-120 s, a titan or
    /// boss 2-5 minutes, small ones a hit or two. Over-tier: a large one at least 10 s, a huge one
    /// about 50 s (the cap; 45 the floor), a titan 2½ minutes (the cap; 2 the floor).
    pub const fn band(self) -> Band {
        match self {
            Size::Small => Band { ttk: (0.0, 2.0), mid: 1.0, hits: 25.0, interval: 1.0, cut: 0.0, cap: None, floor: None, knockback: 0.0 },
            Size::Medium => {
                Band { ttk: (3.0, 8.0), mid: 5.0, hits: 8.0, interval: 1.2, cut: 0.10, cap: Some(30.0), floor: None, knockback: 0.3 }
            }
            Size::Large => Band {
                ttk: (20.0, 40.0),
                mid: 30.0,
                hits: 4.0,
                interval: 2.0,
                cut: 0.25,
                cap: Some(20.0),
                floor: Some(10.0),
                knockback: 0.7,
            },
            Size::Huge => Band {
                ttk: (60.0, 120.0),
                mid: 90.0,
                hits: 2.5,
                interval: 3.0,
                cut: 0.35,
                cap: Some(10.0),
                floor: Some(45.0),
                knockback: 1.0,
            },
            Size::Titan => Band {
                ttk: (120.0, 300.0),
                mid: 210.0,
                hits: 1.5,
                interval: 4.0,
                cut: 0.45,
                cap: Some(5.0),
                floor: Some(120.0),
                knockback: 1.0,
            },
        }
    }
}

/// Role shapes a creature's budget without adding to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// Flees.
    Prey,
    /// Defends its herd.
    Grazer,
    Hunter,
    /// Priced as about half a size class up.
    Apex,
    /// Armoured, like the ankylosaurs (gastonia, pinacosaurus).
    Tank,
}

pub const ROLES: [Role; 5] = [Role::Prey, Role::Grazer, Role::Hunter, Role::Apex, Role::Tank];

impl Role {
    pub const fn name(self) -> &'static str {
        match self {
            Role::Prey => "prey",
            Role::Grazer => "grazer",
            Role::Hunter => "hunter",
            Role::Apex => "apex",
            Role::Tank => "tank",
        }
    }

    /// Health, hit and armour, as multiples of the size's.
    pub const fn times(self) -> (f32, f32, f32) {
        match self {
            Role::Prey => (0.7, 0.5, 1.0),
            Role::Grazer => (1.0, 0.8, 1.0),
            Role::Hunter => (0.9, 1.2, 1.0),
            Role::Apex => (1.3, 1.3, 1.0),
            Role::Tank => (1.0, 0.7, 1.6),
        }
    }
}

// ------------------------------------------------------------------ derived stats

/// A fighter's numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub health: f32,
    pub armour: f32,
    /// One hit, and seconds between hits.
    pub hit: f32,
    pub interval: f32,
}

/// The hit that downs a player of `health` and `armour` in `hits` hits through armour's cut.
pub fn hit_to_down(hits: f32, health: f32, armour: f32) -> f32 {
    // h × (1 - A/(A + K h)) = q  →  K h² - K q h - q A = 0.
    let q = health / hits.max(0.01);
    let k = ARMOUR_K;
    let h = (k * q + (k * k * q * q + 4.0 * k * q * armour).sqrt()) / (2.0 * k);
    if armour_cut(armour, h) >= ARMOUR_CAP { q / (1.0 - ARMOUR_CAP) } else { h }
}

/// A creature of `size` and `role` at area level `level`: health so a same-age reference kills it
/// in its band's middle, armour so it cuts the reference's hit by the band's share, a hit so it
/// downs the reference in the band's hits. Every number grows as `g(level)`.
pub fn derive(size: Size, role: Role, level: f32) -> Stats {
    let base = derive_base(size, level);
    let (th, tx, ta) = role.times();
    Stats { health: base.health * th, armour: base.armour * ta, hit: base.hit * tx, interval: base.interval }
}

/// A size class's own numbers at `level`, before its role.
pub fn derive_base(size: Size, level: f32) -> Stats {
    let b = size.band();
    let p = Player { level, times: 1.0 };
    let r = p.hit();
    Stats {
        health: b.mid * r * REF_RATE * (1.0 - b.cut),
        armour: armour_for(b.cut, r),
        hit: hit_to_down(b.hits, p.health(), p.armour()),
        interval: b.interval,
    }
}

// ------------------------------------------------------------------ the fight simulator

/// How a fight between a player and a creature comes out, both at the reference's rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fight {
    /// Seconds to kill, with the rate floor.
    pub ttk: f32,
    /// Seconds to kill from damage alone (under the floor, the rest went to stagger).
    pub raw: f32,
    /// The creature's hits that down the player.
    pub hits: f32,
    /// The level the player's gear counted as.
    pub counted: f32,
}

/// A player of `player` against a creature of `size` with `them` at area level `level`: the
/// player's gear counts at most `size`'s cap above the creature (Destiny's power cap, only for
/// big creatures), the creature loses at most its floor's share a second.
pub fn fight(player: Player, size: Size, level: f32, them: Stats) -> Fight {
    let b = size.band();
    let counted = match b.cap {
        Some(cap) => player.level.min(level + cap),
        None => player.level,
    };
    // Grade and marks still count over the cap, but they don't make it one: the cap is on levels.
    let p = Player { level: counted, times: player.times };
    let r = p.hit();
    let per_hit = r * (1.0 - armour_cut(them.armour, r));
    let raw = them.health / (per_hit * REF_RATE).max(1e-6);
    let ttk = b.floor.map_or(raw, |f| raw.max(f));
    let taken = them.hit * (1.0 - armour_cut(p.armour(), them.hit));
    Fight { ttk, raw, hits: p.health() / taken.max(1e-6), counted }
}

/// The share of a hit an over-tier weapon keeps against a creature of `size` at `level`: its item
/// level counts at most the size's cap above the creature (grade and marks still multiply what's
/// counted). 1 for small creatures and for gear within the cap.
pub fn cap_share(size: Size, item_level: f32, level: f32) -> f32 {
    match size.band().cap {
        Some(cap) if item_level > level + cap => g(level + cap) / g(item_level),
        _ => 1.0,
    }
}

/// The rate floor as a bucket: a big creature may lose at most `max_health / floor` a second, from
/// everything together, with one second's worth held in hand (`FLOOR_HELD`). What it can take now
/// out of `budget` (refilled over `secs` since the last hit), the budget left, and what spilled
/// over (for stagger and part breaks).
pub const FLOOR_HELD: f32 = 1.0;

pub fn floor_take(size: Size, max_health: f32, budget: f32, secs: f32, hit: f32) -> (f32, f32, f32) {
    let Some(floor) = size.band().floor else { return (hit, budget, 0.0) };
    let rate = max_health / floor;
    let have = (budget + rate * secs.max(0.0)).min(rate * FLOOR_HELD);
    let took = hit.min(have);
    (took, have - took, hit - took)
}

/// A same-age reference player of `age` against a creature of `size` and `role` at `level`.
pub fn reference_fight(age: Age, size: Size, role: Role, level: f32) -> Fight {
    fight(Player::of(age), size, level, derive(size, role, level))
}

/// The level shift that makes a stat `x` times stronger: a level-200 animal is 5× a level-0 one,
/// so the levels between are `levels_for(5)`, about 34 area levels, three and a half ages.
pub fn shift_for(x: f32) -> f32 {
    levels_for(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The report's computed table (4.4), at L 5, in engine points.
    #[test]
    fn derived_stats_match_the_report() {
        let close = |a: f32, b: f32, tol: f32| (a - b).abs() <= tol;
        let large = derive(Size::Large, Role::Grazer, 5.0);
        assert!(close(large.health, 171.0, 1.0), "{large:?}");
        assert!(close(large.armour, 12.6, 0.2), "{large:?}");
        assert!(close(large.hit / 0.8, 10.2, 0.3), "{large:?}");
        let titan = derive(Size::Titan, Role::Grazer, 5.0);
        assert!(close(titan.health, 877.0, 2.0), "{titan:?}");
        let small = derive(Size::Small, Role::Grazer, 5.0);
        assert!(close(small.health, 7.6, 0.1), "{small:?}");
    }

    #[test]
    fn same_age_fights_land_in_their_bands() {
        for size in SIZES {
            let b = size.band();
            let f = reference_fight(Age::Iron, size, Role::Grazer, Age::Iron.reference());
            // A grazer has the size's health and four fifths of its hit.
            assert!((f.ttk - b.mid).abs() < 0.01, "{size:?}: {f:?}");
            assert!(f.hits > b.hits && f.hits < b.hits / 0.8 * 1.3, "{size:?}: {f:?}");
        }
    }

    #[test]
    fn over_tier_meets_the_floor() {
        let star = Player::of(Age::Stars);
        for size in [Size::Large, Size::Huge, Size::Titan] {
            let lv = Age::Stone.reference();
            let f = fight(star, size, lv, derive(size, Role::Grazer, lv));
            assert!(f.ttk >= size.band().floor.unwrap(), "{size:?}: {f:?}");
        }
    }

    #[test]
    fn the_floor_holds_whatever_the_hits() {
        // A large creature of 100 health: at most 10 a second, a second's worth at once.
        let (mut budget, mut dealt) = (10.0, 0.0);
        for _ in 0..100 {
            let (took, left, over) = floor_take(Size::Large, 100.0, budget, 0.1, 50.0);
            assert!((took + over - 50.0).abs() < 1e-4);
            budget = left;
            dealt += took;
        }
        assert!(dealt <= 10.0 * 10.0 + 10.0 + 1e-3, "{dealt}");
        assert_eq!(floor_take(Size::Small, 5.0, 0.0, 0.0, 9.0), (9.0, 0.0, 0.0));
        assert!((cap_share(Size::Large, 45.0, 5.0) - g(25.0) / g(45.0)).abs() < 1e-6);
        assert_eq!(cap_share(Size::Small, 60.0, 0.0), 1.0);
    }

    #[test]
    fn armour_formula() {
        assert_eq!(armour_cut(0.0, 10.0), 0.0);
        assert!((armour_cut(armour_for(0.25, 7.0), 7.0) - 0.25).abs() < 1e-5);
        assert!(armour_cut(1e9, 1.0) <= ARMOUR_CAP);
        let pierce = Hit { pen: PIERCE_PEN, ..Hit::plain(10.0, Type::Pierce) };
        assert!(damage(pierce, 20.0, 0.0) > damage(Hit::plain(10.0, Type::Cut), 20.0, 0.0));
        assert!((damage(Hit::plain(10.0, Type::Fire), 99.0, 0.5) - 5.0).abs() < 1e-5);
    }
}

//! The one growth curve every system reads (`notes/PROGRESSION.md`, "The balance model";
//! `reports/Balance model.md`, Part 3 and 5): the seven ages as tiers 0-6 with their hour
//! budgets, the currency they're priced in (effort minutes), power as `g(L) = 1.6^(L/10)` over an
//! area or item level `L` (ten levels an age), the paths a thing belongs to, and bounded hand
//! tweaks.
//!
//! The rule that ties it together: **declare, then derive.** A creature, item or perk states a
//! few facts (its age, size, role, kind); its numbers come from this curve, and a hand tweak only
//! multiplies them by ×0.75-×1.25 with a reason. The checks (`mods/base/progression`) still run on
//! the result, so a tweak can't push anything out of its band.

/// The ages, in order: each is a tier number (Ronald, 11-06 and 11-07: plain English names).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Age {
    Stone,
    Bronze,
    Iron,
    Steam,
    Lightning,
    Future,
    Stars,
}

pub const AGES: [Age; 7] = [Age::Stone, Age::Bronze, Age::Iron, Age::Steam, Age::Lightning, Age::Future, Age::Stars];

impl Age {
    /// Its tier, 0-6.
    pub const fn tier(self) -> u8 {
        self as u8
    }

    /// The age of tier `t` (above the last, the last: the ladder stays open at the top).
    pub const fn of_tier(t: u8) -> Age {
        match t {
            0 => Age::Stone,
            1 => Age::Bronze,
            2 => Age::Iron,
            3 => Age::Steam,
            4 => Age::Lightning,
            5 => Age::Future,
            _ => Age::Stars,
        }
    }

    /// Its name (round 6, 11-07: "Stone Age ... Age of Stars"), for the sheets; the game's are
    /// translation keys, `age.base.<id>`.
    pub const fn name(self) -> &'static str {
        match self {
            Age::Stone => "Stone Age",
            Age::Bronze => "Bronze Age",
            Age::Iron => "Iron Age",
            Age::Steam => "Age of Steam",
            Age::Lightning => "Age of Lightning",
            Age::Future => "Age of the Future",
            Age::Stars => "Age of Stars",
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Age::Stone => "stone",
            Age::Bronze => "bronze",
            Age::Iron => "iron",
            Age::Steam => "steam",
            Age::Lightning => "lightning",
            Age::Future => "future",
            Age::Stars => "stars",
        }
    }

    /// Hours an average player spends in it (`notes/PROGRESSION.md`, "The ages": about 106 in
    /// all, Ronald's "100+ hours").
    pub const fn hours(self) -> f32 {
        match self {
            Age::Stone => 6.0,
            Age::Bronze => 12.0,
            Age::Iron => 16.0,
            Age::Steam => 18.0,
            Age::Lightning => 20.0,
            Age::Future => 18.0,
            Age::Stars => 16.0,
        }
    }

    /// Its whole budget in effort minutes.
    pub fn em(self) -> f32 {
        self.hours() * 60.0
    }

    /// The level its first place, item or creature starts at (10 a tier).
    pub const fn start(self) -> f32 {
        self.tier() as f32 * LEVELS_PER_AGE
    }

    /// The level of a same-age reference: the middle of the age (depth 5), what the bands are
    /// measured at (`combat::reference`).
    pub const fn reference(self) -> f32 {
        self.start() + REFERENCE_DEPTH
    }
}

/// The age a level falls in.
pub fn age_of(level: f32) -> Age {
    Age::of_tier((level.max(0.0) / LEVELS_PER_AGE).floor().min(6.0) as u8)
}

/// Hours from the first flint to the stars.
pub fn total_hours() -> f32 {
    AGES.iter().map(|a| a.hours()).sum()
}

/// The currency: **effort minutes** (EM), what an average player spends to get something
/// (gathering, travel, crafting, station time and risk). An age's whole budget is its hours in
/// them (`Age::em`).
pub type Em = f32;

/// The share of an age's hours making things may take, the first time each (gathering for it and
/// crafting): the rest is exploring, fighting, taming, building and travel. A first budget; the
/// crafting simulation of `reports/Balance model.md` (5.5) refines it.
pub const MAKING_SHARE: f32 = 0.4;

/// Power grows this much an age: a plain item of the next age is about a perfect one of this
/// (grade and marks add at most `GRADE_CEILING`).
pub const GROWTH: f32 = 1.6;
/// Levels in an age (item level and area level: tier × 10 + a depth of 0-9).
pub const LEVELS_PER_AGE: f32 = 10.0;
/// The depth of a same-age reference inside its age.
pub const REFERENCE_DEPTH: f32 = 5.0;
/// The top level of places and items (the Age of Stars' deepest).
pub const MAX_LEVEL: f32 = 69.0;

/// Power at level `L`: 1 at the start of the Stone Age, 2.56 at the Iron Age's, 16.8 at the Age of
/// Stars', about +4.8 % a level.
pub fn g(level: f32) -> f32 {
    GROWTH.powf(level / LEVELS_PER_AGE)
}

/// The level that gives power `x` against level 0 (`g`'s inverse): `g(levels_for(x)) == x`.
pub fn levels_for(x: f32) -> f32 {
    LEVELS_PER_AGE * x.ln() / GROWTH.ln()
}

/// An item's level: its tier × 10 plus a depth of 0-9 from where its material or drop came from
/// (a ley vein, a boss form, a dimension's depth). It caps which marks can roll.
pub fn item_level(tier: u8, depth: u8) -> f32 {
    tier as f32 * LEVELS_PER_AGE + depth.min(9) as f32
}

/// The most grade and marks together add to an item over a plain one of its tier (+60 %): a
/// perfect tier-N item is about a plain tier-N+1.
pub const GRADE_CEILING: f32 = 1.6;
/// The most everything together (gear's grade and marks, the web, a tame's bond, food) multiplies
/// the reference by: two ages.
pub const TOTAL_CEILING: f32 = 2.5;

/// Which branch of the braid a thing belongs to (Ronald, 11-06: tech, magic and taming side by
/// side, and every age needs a bit of all three); `Shared` is what every path uses (the ground,
/// food, building).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Path {
    Tech,
    Magic,
    Beasts,
    Shared,
}

pub const PATHS: [Path; 4] = [Path::Tech, Path::Magic, Path::Beasts, Path::Shared];
/// The three an age's key needs a route by.
/// Path parity (`reports/Balance model.md`, 5.3): an age's key by one route costs at most this much
/// more effort than by its cheapest.
pub const PARITY: f32 = 0.15;

pub const ROUTES: [Path; 3] = [Path::Tech, Path::Magic, Path::Beasts];

impl Path {
    pub const fn name(self) -> &'static str {
        match self {
            Path::Tech => "tech",
            Path::Magic => "magic",
            Path::Beasts => "beasts",
            Path::Shared => "shared",
        }
    }
}

/// A hand tweak on a derived number: a multiplier, with why. Outside `TWEAK_RANGE`, or without a
/// reason, it fails the checks: then the role or the curve is wrong, and the fix goes there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tweak {
    pub x: f32,
    pub why: &'static str,
}

/// How far a tweak may move a derived number (`reports/Balance model.md`, 5.4).
pub const TWEAK_RANGE: (f32, f32) = (0.75, 1.25);

impl Tweak {
    pub const NONE: Tweak = Tweak { x: 1.0, why: "" };

    pub const fn new(x: f32, why: &'static str) -> Tweak {
        Tweak { x, why }
    }

    /// Whether it's inside the range and says why (no tweak needs no reason).
    pub fn ok(&self) -> bool {
        (TWEAK_RANGE.0..=TWEAK_RANGE.1).contains(&self.x) && (self.x == 1.0 || !self.why.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_is_the_reports() {
        assert!((g(0.0) - 1.0).abs() < 1e-6);
        assert!((g(20.0) - 2.56).abs() < 1e-3);
        assert!((g(60.0) - 16.777).abs() < 1e-2);
        assert!((g(levels_for(5.0)) - 5.0).abs() < 1e-4);
        assert!((total_hours() - 106.0).abs() < 1e-3);
        assert_eq!(age_of(25.0), Age::Iron);
        assert_eq!(age_of(99.0), Age::Stars);
    }

    #[test]
    fn tweaks_are_bounded() {
        assert!(Tweak::NONE.ok());
        assert!(Tweak::new(0.85, "herds fight together").ok());
        assert!(!Tweak::new(0.85, "").ok());
        assert!(!Tweak::new(1.5, "too much").ok());
    }
}

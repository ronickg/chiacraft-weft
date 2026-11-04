//! The stomach: what was eaten, digesting. Its size is the food stat (units of 10 g); each
//! portion gives its nutrients while it is there and shrinks as it digests, so a meal's strength
//! fades as Valheim's food does. Fullness of a nutrient is what the stomach holds against what a
//! balanced full stomach of its size would (`food::FULL`).
//!
//! **A stat sets the ceiling, food sets how much of it is reached**: max health from protein, max
//! stamina from energy, between `FLOOR` (an empty stomach) and the stat's value.

use crate::food::{self, FOODS, FULL, NUTRIENTS};

/// Portions held at once (a slot each; plenty, since room runs out first).
pub const PORTIONS: usize = 12;
/// Max health and stamina on an empty stomach, as a share of the stat (proposed).
pub const FLOOR: f32 = 0.4;
/// A nutrient over this fullness counts toward being well fed; four of the five make it, from at
/// least three different foods (a varied diet: a loaf alone is nearly balanced, but bread every
/// meal isn't).
pub const WELL_FED_AT: f32 = 0.5;
pub const WELL_FED_NUTRIENTS: usize = 4;
pub const WELL_FED_FOODS: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Portion {
    /// Index in `FOODS` + 1 (0: an empty slot).
    pub food: u16,
    /// Units left to digest.
    pub units: f32,
    /// The share of its nutrients it gives (its freshness when eaten).
    pub quality: f32,
    /// Degrees it warms (cools, negative) its eater, and the seconds left of that.
    pub warmth: f32,
    pub warm_secs: f32,
}

impl Portion {
    pub fn food(&self) -> Option<&'static food::Food> {
        (self.food > 0).then(|| &FOODS[self.food as usize - 1])
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stomach {
    pub portions: [Portion; PORTIONS],
}

/// Why a food can't be eaten.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Refusal {
    /// Not enough room for the whole portion.
    Full,
    /// Every slot taken.
    TooMany,
}

impl Stomach {
    /// Units taken.
    pub fn used(&self) -> f32 {
        self.portions.iter().map(|p| p.units).sum()
    }

    pub fn room(&self, capacity: f32) -> f32 {
        (capacity - self.used()).max(0.0)
    }

    /// Eats a portion of `FOODS[food]` of that freshness (`Freshness::quality`), `age_secs` since
    /// it was made (hot or not).
    pub fn eat(&mut self, food: usize, quality: f32, age_secs: f32, capacity: f32) -> Result<(), Refusal> {
        let f = &FOODS[food];
        if f.units() > self.room(capacity) + 1e-4 {
            return Err(Refusal::Full);
        }
        let slot = self
            .portions
            .iter_mut()
            .find(|p| p.food == 0)
            .ok_or(Refusal::TooMany)?;
        let (warmth, warm_secs) = food::meal_warmth(f, age_secs).unwrap_or((0.0, 0.0));
        *slot = Portion { food: food as u16 + 1, units: f.units(), quality, warmth, warm_secs };
        Ok(())
    }

    /// Eats a meal of several foods (`meals`: a stew's ingredients, cooked), each its own portion
    /// stronger for the cooking (`meals::BONUS`), the whole meal warming as one hot dish.
    pub fn eat_meal(&mut self, foods: &[usize], quality: f32, age_secs: f32, capacity: f32) -> Result<(), Refusal> {
        if crate::meals::units(foods) > self.room(capacity) + 1e-4 {
            return Err(Refusal::Full);
        }
        if self.portions.iter().filter(|p| p.food == 0).count() < foods.len() {
            return Err(Refusal::TooMany);
        }
        let hot = age_secs < food::HOT_FOR;
        for (i, &f) in foods.iter().enumerate() {
            let slot = self
                .portions
                .iter_mut()
                .find(|p| p.food == 0)
                .ok_or(Refusal::TooMany)?;
            let (warmth, warm_secs) = if i == 0 && hot { food::HOT_MEAL } else { (0.0, 0.0) };
            *slot = Portion { food: f as u16 + 1, units: FOODS[f].units(), quality: quality * crate::meals::BONUS, warmth, warm_secs };
        }
        Ok(())
    }

    /// `secs` of digestion at `pace` (1 normal; the world's difficulty, being asleep).
    pub fn digest(&mut self, secs: f32, pace: f32) {
        for p in &mut self.portions {
            let Some(f) = p.food() else { continue };
            p.units -= f.digest_rate() * pace * secs / 60.0;
            p.warm_secs = (p.warm_secs - secs).max(0.0);
            if p.units <= 1e-3 {
                *p = Portion::default();
            }
        }
    }

    /// Empties it (dying).
    pub fn clear(&mut self) {
        *self = Stomach::default();
    }

    /// Grams of each nutrient held.
    pub fn nutrients(&self) -> [f32; NUTRIENTS] {
        let mut n = [0.0; NUTRIENTS];
        for p in &self.portions {
            let Some(f) = p.food() else { continue };
            for (i, g) in f.per_unit().iter().enumerate() {
                n[i] += g * p.units * p.quality;
            }
        }
        n
    }

    /// Each nutrient's fullness, 0 to 1, for a stomach of `capacity` units.
    pub fn fullness(&self, capacity: f32) -> [f32; NUTRIENTS] {
        let n = self.nutrients();
        std::array::from_fn(|i| (n[i] / (FULL[i] * capacity.max(1.0) / 100.0)).min(1.0))
    }

    /// Four of the five nutrients over half full, from three different foods: a varied diet.
    pub fn well_fed(&self, capacity: f32) -> bool {
        let mut foods: Vec<u16> = self
            .portions
            .iter()
            .filter(|p| p.food > 0)
            .map(|p| p.food)
            .collect();
        foods.sort_unstable();
        foods.dedup();
        foods.len() >= WELL_FED_FOODS
            && self
                .fullness(capacity)
                .iter()
                .filter(|&&f| f >= WELL_FED_AT)
                .count()
                >= WELL_FED_NUTRIENTS
    }

    /// Degrees warmer its warmest hot meal makes its eater feel now: the strongest dish, not the
    /// sum (Research round 3, B6: four hot steaks aren't +16 °C).
    pub fn meal_warming(&self) -> f32 {
        self.portions
            .iter()
            .filter(|p| p.warm_secs > 0.0)
            .map(|p| p.warmth)
            .fold(0.0, f32::max)
    }

    /// Degrees cooler (0 or less) its coolest cooling food makes it feel now.
    pub fn meal_cooling(&self) -> f32 {
        self.portions
            .iter()
            .filter(|p| p.warm_secs > 0.0)
            .map(|p| p.warmth)
            .fold(0.0, f32::min)
    }
}

/// What a new player arrives with in them, and respawns with (SURVIVAL.md steps 11 and 13): a
/// loaf, an apple and a cooked chicken, 65 units, max health 90 of 100 and stamina 83 (Ronald,
/// 11-04, "Bigger arrival meal": with the loaf and apple alone, 54, a 9-block fall took 61% of a
/// fresh player's health in the taiga). Three foods make one well fed until it digests below half;
/// it wears off like any meal, so hunger still matters from the first hour.
pub const ARRIVAL: [&str; 3] = ["base:bread", "base:apple", "base:cooked_chicken"];

/// A new player's stomach: `ARRIVAL`, fresh, neither hot nor cooling.
pub fn arrival() -> Stomach {
    let mut s = Stomach::default();
    for (slot, f) in ARRIVAL.iter().filter_map(|n| food::find(n)).enumerate() {
        s.portions[slot] = Portion { food: f as u16 + 1, units: FOODS[f].units(), quality: 1.0, warmth: 0.0, warm_secs: 0.0 };
    }
    s
}

/// A stat's ceiling reached at that fullness of its nutrient.
pub fn fed(stat: f32, fullness: f32) -> f32 {
    stat * (FLOOR + (1.0 - FLOOR) * fullness.clamp(0.0, 1.0))
}

/// Max health and max stamina from the stats and the stomach (well fed adds 10% stamina).
pub fn max_health_stamina(health: f32, stamina: f32, stomach: &Stomach, capacity: f32) -> (f32, f32) {
    let full = stomach.fullness(capacity);
    let bonus = if stomach.well_fed(capacity) { 1.1 } else { 1.0 };
    (fed(health, full[food::PROTEIN]), fed(stamina, full[food::ENERGY]) * bonus)
}

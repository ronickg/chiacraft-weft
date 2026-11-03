//! Every food's facts, from real nutrition per 100 g (USDA FoodData Central, rounded), and what
//! they come to in the game's five nutrients; how long a food keeps; hot and cold meals.
//!
//! A unit is 10 g: a food's size in the stomach is its portion's weight in units, and its
//! nutrients are per unit.

use crate::DAY_SECS;

pub const NUTRIENTS: usize = 5;
pub const PROTEIN: usize = 0;
pub const ENERGY: usize = 1;
pub const FAT: usize = 2;
pub const GREENS: usize = 3;
pub const WATER: usize = 4;
pub const NUTRIENT_NAMES: [&str; NUTRIENTS] = ["protein", "energy", "fat", "greens", "water"];

/// Grams of each nutrient a full stomach of 100 units holds when it's balanced (proposed): what
/// counts as 100% of a nutrient, scaled to the stomach's size.
pub const FULL: [f32; NUTRIENTS] = [100.0, 250.0, 50.0, 25.0, 500.0];

/// Nutrition per 100 g: water, protein, carbohydrate, fat, fiber in grams, vitamin C in mg.
#[derive(Clone, Copy, Debug)]
pub struct Per100 {
    pub water: f32,
    pub protein: f32,
    pub carbs: f32,
    pub fat: f32,
    pub fiber: f32,
    pub vitamin_c: f32,
}

/// Warmth a meal brings: hot food warms for a while after cooking, fruit and drinks cool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Meal {
    Plain,
    /// Hot for a while after it is made (`HOT_FOR`).
    Cooked,
    /// Always cooling.
    Cooling,
}

#[derive(Clone, Copy, Debug)]
pub struct Food {
    /// The item it is (`base:apple`).
    pub name: &'static str,
    /// A portion's weight in grams: the item eaten once.
    pub grams: f32,
    pub per100: Per100,
    pub meal: Meal,
    /// Raw meat: a chance of food poisoning even fresh.
    pub raw: bool,
    /// Game days fresh, then stale (half the nutrients), then spoiled.
    pub keeps: (f32, f32),
    /// The chance of food poisoning at any freshness (a poisonous food, already rotten meat).
    pub poison: f32,
}

const fn per(water: f32, protein: f32, carbs: f32, fat: f32, fiber: f32, vitamin_c: f32) -> Per100 {
    Per100 { water, protein, carbs, fat, fiber, vitamin_c }
}

/// The game's foods. Portions: an apple 180 g, a loaf 250 g, a pork chop 150 g raw and 110 g
/// cooked (meat loses about a quarter of its weight cooking, its water), a steak 200 g and 150 g,
/// a whole small chicken's meat 300 g and 220 g, a lamb chop 150 g and 110 g.
pub const FOODS: &[Food] = &[
    Food {
        name: "base:apple",
        grams: 180.0,
        per100: per(85.6, 0.3, 13.8, 0.2, 2.4, 4.6),
        meal: Meal::Cooling,
        raw: false,
        keeps: (20.0, 20.0),
        poison: 0.0,
    },
    Food {
        name: "base:bread",
        grams: 250.0,
        per100: per(36.0, 9.0, 49.0, 3.3, 2.7, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (5.0, 5.0),
        poison: 0.0,
    },
    Food {
        name: "base:porkchop",
        grams: 150.0,
        per100: per(72.2, 21.0, 0.0, 5.7, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
    },
    Food {
        name: "base:cooked_porkchop",
        grams: 110.0,
        per100: per(61.0, 27.0, 0.0, 8.9, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
    },
    Food {
        name: "base:beef",
        grams: 200.0,
        per100: per(70.0, 20.9, 0.0, 8.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
    },
    Food {
        name: "base:cooked_beef",
        grams: 150.0,
        per100: per(59.0, 28.0, 0.0, 11.0, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
    },
    Food {
        name: "base:chicken",
        grams: 300.0,
        per100: per(68.6, 18.6, 0.0, 15.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
    },
    Food {
        name: "base:cooked_chicken",
        grams: 220.0,
        per100: per(59.5, 27.3, 0.0, 13.6, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
    },
    Food {
        name: "base:mutton",
        grams: 150.0,
        per100: per(64.0, 17.0, 0.0, 16.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
    },
    Food {
        name: "base:cooked_mutton",
        grams: 110.0,
        per100: per(57.0, 25.0, 0.0, 16.5, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
    },
    Food {
        name: "base:rotten_flesh",
        grams: 120.0,
        per100: per(65.0, 15.0, 0.0, 10.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (0.0, 0.0),
        poison: 0.8,
    },
    Food {
        name: "base:spider_eye",
        grams: 30.0,
        per100: per(75.0, 14.0, 2.0, 4.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (3.0, 3.0),
        poison: 1.0,
    },
    // Two eggs fried (100 g): protein and fat without meat.
    Food {
        name: "base:cooked_egg",
        grams: 100.0,
        per100: per(69.5, 13.6, 0.8, 14.8, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (2.0, 2.0),
        poison: 0.0,
    },
    // A quarter of an apple pie (200 g): sweet and filling, a little fruit in it.
    Food {
        name: "base:apple_pie",
        grams: 200.0,
        per100: per(52.0, 2.4, 37.1, 12.5, 1.6, 1.7),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
    },
    // Dry pet food (the animals mod's), as dry dog food.
    Food {
        name: "animals:kibble",
        grams: 100.0,
        per100: per(10.0, 25.0, 45.0, 14.0, 4.0, 0.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (90.0, 90.0),
        poison: 0.0,
    },
];

/// A food by its item's name (`base:apple`), or the name without its mod (`apple`).
pub fn find(name: &str) -> Option<usize> {
    FOODS
        .iter()
        .position(|f| f.name == name || f.name.split_once(':').is_some_and(|(_, n)| n == name))
}

impl Food {
    /// Its size in the stomach.
    pub fn units(&self) -> f32 {
        self.grams / 10.0
    }

    /// Grams of each nutrient in a unit. Greens are fiber and vitamins (a tenth of the vitamin
    /// C's milligrams), standing for what fruit and vegetables bring.
    pub fn per_unit(&self) -> [f32; NUTRIENTS] {
        let p = self.per100;
        [p.protein, p.carbs, p.fat, p.fiber + p.vitamin_c / 10.0, p.water].map(|g| g / 10.0)
    }

    /// Grams of each nutrient in a portion.
    pub fn portion(&self) -> [f32; NUTRIENTS] {
        self.per_unit().map(|g| g * self.units())
    }

    /// Units it digests a minute: carbohydrates fast, protein and fat slowly, fruit and drinks
    /// (over 75% water) fastest (proposed; a steak or a loaf about half an hour, a whole chicken
    /// forty minutes, an apple ten).
    pub fn digest_rate(&self) -> f32 {
        let p = self.per100;
        let slow = 1.0 + 4.0 * p.protein / 100.0 + 6.0 * p.fat / 100.0;
        let watery = 1.0 + 3.0 * (p.water / 100.0 - 0.75).max(0.0);
        1.5 * watery / slow
    }

    /// Minutes the whole portion takes to digest, at the normal pace.
    pub fn digest_minutes(&self) -> f32 {
        self.units() / self.digest_rate()
    }
}

/// How fresh a food is, from its age.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Freshness {
    Fresh,
    Stale,
    Spoiled,
}

impl Freshness {
    /// The share of its nutrients it still gives.
    pub fn quality(self) -> f32 {
        match self {
            Freshness::Fresh => 1.0,
            Freshness::Stale => 0.5,
            Freshness::Spoiled => 0.25,
        }
    }
}

/// A food's freshness at `age_secs` since it was made; `keep` stretches or shortens how long
/// food keeps (the world's difficulty, salting and smoking later).
pub fn freshness(food: &Food, age_secs: f32, keep: f32) -> Freshness {
    let days = age_secs / DAY_SECS / keep.max(0.01);
    if days < food.keeps.0 {
        Freshness::Fresh
    } else if days < food.keeps.0 + food.keeps.1 {
        Freshness::Stale
    } else {
        Freshness::Spoiled
    }
}

/// The chance that eating it gives food poisoning.
pub fn poisoning_chance(food: &Food, fresh: Freshness) -> f32 {
    let by_age = match fresh {
        Freshness::Fresh => 0.0,
        Freshness::Stale => 0.05,
        Freshness::Spoiled => 0.5,
    };
    let raw = if food.raw { 0.15 } else { 0.0 };
    food.poison.max(by_age).max(raw).min(1.0)
}

/// Seconds food poisoning lasts (proposed: three minutes).
pub const SICK_SECS: f32 = 180.0;
/// Seconds a poisonous food poisons its eater (a spider's eye).
pub const POISON_SECS: f32 = 15.0;

/// Seconds a cooked food stays hot after it is made.
pub const HOT_FOR: f32 = 300.0;

/// What a meal does to how warm its eater feels: degrees and for how many seconds (hot food
/// warms 4 °C for four minutes, cooling food cools 2 °C for three; proposed).
pub fn meal_warmth(food: &Food, age_secs: f32) -> Option<(f32, f32)> {
    match food.meal {
        Meal::Cooked if age_secs < HOT_FOR => Some((4.0, 240.0)),
        Meal::Cooling => Some((-2.0, 180.0)),
        _ => None,
    }
}

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
    /// Available carbohydrate, without the fiber: USDA's "carbohydrate, by difference" counts
    /// the fiber too, so it is that minus `fiber` (or water, protein, carbs, fat and fiber add up
    /// to more than 100 g, `balance`).
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
    /// What kind of food it is, for the animals' diets (`Category`).
    pub category: Category,
}

/// The kinds of food a diet is made of (`notes/ECOLOGY.md`, part 4): every food item, every food
/// in the world (`vitals::forage`) and every species' diet (`animals_core::species`) names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Grass,
    Leaves,
    WaterPlants,
    Fruit,
    Seeds,
    Roots,
    Lichen,
    Fungi,
    Insects,
    Meat,
    Fish,
    Carrion,
}

pub const CATEGORIES: [Category; 12] = [
    Category::Grass,
    Category::Leaves,
    Category::WaterPlants,
    Category::Fruit,
    Category::Seeds,
    Category::Roots,
    Category::Lichen,
    Category::Fungi,
    Category::Insects,
    Category::Meat,
    Category::Fish,
    Category::Carrion,
];

impl Category {
    /// Its name in translations and sheets.
    pub fn name(self) -> &'static str {
        match self {
            Category::Grass => "grass",
            Category::Leaves => "leaves",
            Category::WaterPlants => "water_plants",
            Category::Fruit => "fruit",
            Category::Seeds => "seeds",
            Category::Roots => "roots",
            Category::Lichen => "lichen",
            Category::Fungi => "fungi",
            Category::Insects => "insects",
            Category::Meat => "meat",
            Category::Fish => "fish",
            Category::Carrion => "carrion",
        }
    }

    /// Plants and fungi, not animals.
    pub fn plant(self) -> bool {
        !matches!(self, Category::Insects | Category::Meat | Category::Fish | Category::Carrion)
    }
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
        per100: per(85.6, 0.3, 11.4, 0.2, 2.4, 4.6),
        meal: Meal::Cooling,
        raw: false,
        keeps: (20.0, 20.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    Food {
        name: "base:bread",
        grams: 250.0,
        per100: per(36.0, 9.0, 49.0, 3.3, 2.7, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (5.0, 5.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:porkchop",
        grams: 150.0,
        per100: per(72.2, 21.0, 0.0, 5.7, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:cooked_porkchop",
        grams: 110.0,
        per100: per(61.0, 27.0, 0.0, 8.9, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:beef",
        grams: 200.0,
        per100: per(70.0, 20.9, 0.0, 8.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:cooked_beef",
        grams: 150.0,
        per100: per(59.0, 28.0, 0.0, 11.0, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:chicken",
        grams: 300.0,
        per100: per(66.0, 18.6, 0.0, 15.1, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:cooked_chicken",
        grams: 220.0,
        per100: per(59.5, 27.3, 0.0, 13.6, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:mutton",
        grams: 150.0,
        per100: per(64.0, 17.0, 0.0, 16.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:cooked_mutton",
        grams: 110.0,
        per100: per(57.0, 25.0, 0.0, 16.5, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "base:rotten_flesh",
        grams: 120.0,
        per100: per(65.0, 15.0, 0.0, 10.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (0.0, 0.0),
        poison: 0.8,
        category: Category::Carrion,
    },
    Food {
        name: "base:spider_eye",
        grams: 30.0,
        per100: per(75.0, 14.0, 2.0, 4.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (3.0, 3.0),
        poison: 1.0,
        category: Category::Insects,
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
        category: Category::Meat,
    },
    // A quarter of an apple pie (200 g): sweet and filling, a little fruit in it.
    Food {
        name: "base:apple_pie",
        grams: 200.0,
        per100: per(52.4, 1.9, 32.4, 12.5, 1.6, 1.7),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // A handful of wild berries (100 g), as blueberries: water, a little sugar, vitamin C.
    Food {
        name: "base:berries",
        grams: 100.0,
        per100: per(84.2, 0.7, 12.1, 0.3, 2.4, 9.7),
        meal: Meal::Cooling,
        raw: false,
        keeps: (6.0, 6.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Preserved (`preserve`): the water gone, so lighter to carry, and keeping for weeks.
    // Dried meat as plain jerky (a 200 g cut dries to 75 g); real jerky keeps one to two months.
    Food {
        name: "base:dried_meat",
        grams: 75.0,
        per100: per(20.0, 55.0, 2.0, 15.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (30.0, 30.0),
        poison: 0.0,
        category: Category::Meat,
    },
    // Hot-smoked meat (smoked over a fire, partly dried): keeps weeks.
    Food {
        name: "base:smoked_meat",
        grams: 120.0,
        per100: per(50.0, 30.0, 0.0, 14.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (14.0, 14.0),
        poison: 0.0,
        category: Category::Meat,
    },
    // Salt-cured meat (as corned beef or barrel pork): months to a year in salt, a game year here.
    Food {
        name: "base:salted_meat",
        grams: 160.0,
        per100: per(55.0, 22.0, 0.0, 14.0, 0.0, 0.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (45.0, 45.0),
        poison: 0.0,
        category: Category::Meat,
    },
    // Dried apple rings (an apple's 180 g dry to 40 g): six months to a year.
    Food {
        name: "base:dried_apple",
        grams: 40.0,
        per100: per(31.8, 0.9, 57.2, 0.3, 8.7, 3.9),
        meal: Meal::Plain,
        raw: false,
        keeps: (56.0, 56.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Dried berries, as dried blueberries (a handful's 100 g dry to 30 g).
    Food {
        name: "base:dried_berries",
        grams: 30.0,
        per100: per(15.0, 2.5, 72.5, 2.5, 7.5, 2.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (56.0, 56.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // The wild foods (`vitals::wild`): the north's berries, a handful (100 g) each. Lingonberries
    // as cranberries (USDA), keeping long on their own acid; cloudberries from Fineli (Finland's
    // food table), rich in vitamin C, soft; crowberries, which hold on the plant over winter, as
    // estimates near Fineli's.
    Food {
        name: "base:lingonberries",
        grams: 100.0,
        per100: per(87.3, 0.5, 8.4, 0.1, 3.6, 14.0),
        meal: Meal::Cooling,
        raw: false,
        keeps: (20.0, 20.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    Food {
        name: "base:cloudberries",
        grams: 100.0,
        per100: per(85.0, 2.4, 2.3, 0.8, 6.3, 100.0),
        meal: Meal::Cooling,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    Food {
        name: "base:crowberries",
        grams: 100.0,
        per100: per(85.0, 0.6, 7.0, 1.0, 4.5, 6.0),
        meal: Meal::Cooling,
        raw: false,
        keeps: (10.0, 10.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Brambles' raspberries (USDA), soft: two days fresh.
    Food {
        name: "base:raspberries",
        grams: 100.0,
        per100: per(85.8, 1.2, 5.4, 0.7, 6.5, 26.2),
        meal: Meal::Cooling,
        raw: false,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Prickly pear fruit (tunas, USDA), a large one.
    Food {
        name: "base:prickly_pear_fruit",
        grams: 100.0,
        per100: per(87.6, 0.7, 6.0, 0.5, 3.6, 14.0),
        meal: Meal::Cooling,
        raw: false,
        keeps: (7.0, 7.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Rose hips (USDA), a handful (30 g): sugar, fiber and a great deal of vitamin C, a winter
    // fruit.
    Food {
        name: "base:rose_hip",
        grams: 30.0,
        per100: per(58.7, 1.6, 14.1, 0.3, 24.1, 426.0),
        meal: Meal::Cooling,
        raw: false,
        keeps: (30.0, 30.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    // Nuts and seeds, a handful (30 g), keeping for months: hazelnuts, pine nuts and sunflower
    // kernels (USDA), fat and protein.
    Food {
        name: "base:hazelnut",
        grams: 30.0,
        per100: per(5.3, 15.0, 7.0, 60.8, 9.7, 6.3),
        meal: Meal::Plain,
        raw: false,
        keeps: (60.0, 60.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:pine_nut",
        grams: 30.0,
        per100: per(2.3, 13.7, 9.4, 68.4, 3.7, 0.8),
        meal: Meal::Plain,
        raw: false,
        keeps: (60.0, 60.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:sunflower_seeds",
        grams: 30.0,
        per100: per(4.7, 20.8, 11.4, 51.5, 8.6, 1.4),
        meal: Meal::Plain,
        raw: false,
        keeps: (90.0, 90.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    // A handful (50 g) of kelp (USDA, raw kelp): salty, a little starch, a sea vegetable.
    Food {
        name: "base:kelp",
        grams: 50.0,
        per100: per(81.6, 1.7, 9.6, 0.6, 1.3, 3.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::WaterPlants,
    },
    // A cattail's rhizome (100 g), spring's classic survival starch, roasted or raw: about 17 g of
    // starch in 100 g fresh (an estimate, Research round 3), the rest mostly water and fibre.
    Food {
        name: "base:cattail_root",
        grams: 100.0,
        per100: per(78.0, 1.2, 17.0, 0.2, 3.0, 2.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (14.0, 14.0),
        poison: 0.0,
        category: Category::Roots,
    },
    // A bowl (150 g) of wild rice cooked from its grain (USDA, cooked wild rice): spring's real
    // grain in the swamp and the llanos.
    Food {
        name: "base:cooked_wild_rice",
        grams: 150.0,
        per100: per(73.9, 4.0, 19.5, 0.3, 1.8, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    // Mushrooms (50 g), as white mushrooms (USDA); the red one is the fly agaric, poisonous.
    Food {
        name: "base:mushroom_brown",
        grams: 50.0,
        per100: per(92.4, 3.1, 3.3, 0.3, 1.0, 2.1),
        meal: Meal::Plain,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Fungi,
    },
    Food {
        name: "base:mushroom_red",
        grams: 50.0,
        per100: per(92.4, 3.1, 3.3, 0.3, 1.0, 2.1),
        meal: Meal::Plain,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 1.0,
        category: Category::Fungi,
    },
    // Wild meat (the animals mod's, `animals_core::meat`), from USDA rows: venison (deer, raw;
    // roasted), wild rabbit for the hare (raw; stewed), wild rainbow trout for the brown trout (raw;
    // dry heat), and tilapia standing in for the pacu (raw; dry heat), which has none. Portions
    // as the farm's: a venison steak 200 g raw and 150 g cooked, a hare's quarter or a fish's
    // fillets 150 g and 110 g. Fish keeps less long than meat.
    Food {
        name: "animals:venison",
        grams: 200.0,
        per100: per(73.6, 21.8, 0.0, 2.4, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "animals:cooked_venison",
        grams: 150.0,
        per100: per(65.0, 30.2, 0.0, 3.2, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "animals:hare",
        grams: 150.0,
        per100: per(72.8, 21.8, 0.0, 2.3, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "animals:cooked_hare",
        grams: 110.0,
        per100: per(63.6, 33.0, 0.0, 3.5, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Meat,
    },
    Food {
        name: "animals:trout",
        grams: 150.0,
        per100: per(71.9, 20.5, 0.0, 3.5, 0.0, 2.4),
        meal: Meal::Plain,
        raw: true,
        keeps: (1.0, 1.0),
        poison: 0.0,
        category: Category::Fish,
    },
    Food {
        name: "animals:cooked_trout",
        grams: 110.0,
        per100: per(69.0, 22.9, 0.0, 5.8, 0.0, 2.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Fish,
    },
    Food {
        name: "animals:pacu",
        grams: 150.0,
        per100: per(78.1, 20.1, 0.0, 1.7, 0.0, 0.0),
        meal: Meal::Plain,
        raw: true,
        keeps: (1.0, 1.0),
        poison: 0.0,
        category: Category::Fish,
    },
    Food {
        name: "animals:cooked_pacu",
        grams: 110.0,
        per100: per(71.6, 26.2, 0.0, 2.7, 0.0, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Fish,
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
        category: Category::Meat,
    },
    // The crops' foods (`crops::CROPS`; USDA, raw unless cooked), last so saved stomachs keep
    // their indexes. A carrot 60 g, kept a cellar's weeks; a potato 170 g, raw (starchy and hard
    // to digest, but food), baked 140 g with its skin (it loses water); a beetroot 80 g; an ear of
    // sweet corn's kernels 100 g, raw or boiled, which keeps only days off the stalk; a handful
    // (100 g) of green beans, the bean plant's young pods; a bowl (150 g) of white rice, boiled
    // from the paddy's grain; a cassava root's 100 g, which rots in two or three days once dug,
    // and raw holds the cyanide its cooking takes out (the bitter kinds' more): a chance of
    // poisoning.
    Food {
        name: "base:carrot",
        grams: 60.0,
        per100: per(88.3, 0.9, 6.8, 0.2, 2.8, 5.9),
        meal: Meal::Plain,
        raw: false,
        keeps: (20.0, 20.0),
        poison: 0.0,
        category: Category::Roots,
    },
    Food {
        name: "base:potato",
        grams: 170.0,
        per100: per(79.3, 2.0, 17.5, 0.1, 2.1, 19.7),
        meal: Meal::Plain,
        raw: false,
        keeps: (30.0, 30.0),
        poison: 0.0,
        category: Category::Roots,
    },
    Food {
        name: "base:baked_potato",
        grams: 140.0,
        per100: per(74.9, 2.5, 21.2, 0.1, 2.2, 9.6),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Roots,
    },
    Food {
        name: "base:beetroot",
        grams: 80.0,
        per100: per(87.6, 1.6, 6.8, 0.2, 2.8, 4.9),
        meal: Meal::Plain,
        raw: false,
        keeps: (20.0, 20.0),
        poison: 0.0,
        category: Category::Roots,
    },
    Food {
        name: "base:corn",
        grams: 100.0,
        per100: per(76.0, 3.3, 17.0, 1.4, 2.0, 6.8),
        meal: Meal::Plain,
        raw: false,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:roasted_corn",
        grams: 100.0,
        per100: per(73.4, 3.4, 18.6, 1.5, 2.4, 5.5),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:beans",
        grams: 100.0,
        per100: per(90.3, 1.8, 4.3, 0.2, 2.7, 12.2),
        meal: Meal::Plain,
        raw: false,
        keeps: (5.0, 5.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:cooked_rice",
        grams: 150.0,
        per100: per(68.4, 2.7, 28.2, 0.3, 0.4, 0.0),
        meal: Meal::Cooked,
        raw: false,
        keeps: (3.0, 3.0),
        poison: 0.0,
        category: Category::Seeds,
    },
    Food {
        name: "base:cassava",
        grams: 100.0,
        per100: per(59.7, 1.4, 36.3, 0.3, 1.8, 20.6),
        meal: Meal::Plain,
        raw: false,
        keeps: (2.0, 2.0),
        poison: 0.2,
        category: Category::Roots,
    },
    // The stems' fruit (`crops::CROPS`, USDA, raw; carbs without the fiber): a slice of pumpkin,
    // a fifth of a small one, and a slice of watermelon, cooling.
    Food {
        name: "base:pumpkin_slice",
        grams: 200.0,
        per100: per(91.6, 1.0, 6.0, 0.1, 0.5, 9.0),
        meal: Meal::Plain,
        raw: false,
        keeps: (4.0, 4.0),
        poison: 0.0,
        category: Category::Fruit,
    },
    Food {
        name: "base:melon_slice",
        grams: 150.0,
        per100: per(91.5, 0.6, 7.2, 0.2, 0.4, 8.1),
        meal: Meal::Cooling,
        raw: false,
        keeps: (2.0, 2.0),
        poison: 0.0,
        category: Category::Fruit,
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
/// food keeps (the world's difficulty). Preserved foods are foods of their own (`preserve`).
pub fn freshness(food: &Food, age_secs: f32, keep: f32) -> Freshness {
    freshness_of(food.keeps, age_secs, keep)
}

/// Freshness for food that keeps `keeps` game days fresh, then stale (a meal's, `meals::keeps`).
pub fn freshness_of(keeps: (f32, f32), age_secs: f32, keep: f32) -> Freshness {
    let days = age_secs / DAY_SECS / keep.max(0.01);
    if days < keeps.0 {
        Freshness::Fresh
    } else if days < keeps.0 + keeps.1 {
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

/// Degrees a hot meal warms and for how many seconds.
pub const HOT_MEAL: (f32, f32) = (4.0, 240.0);

/// What a meal does to how warm its eater feels: degrees and for how many seconds (hot food
/// warms 4 °C for four minutes, cooling food cools 2 °C for three; proposed).
pub fn meal_warmth(food: &Food, age_secs: f32) -> Option<(f32, f32)> {
    match food.meal {
        Meal::Cooked if age_secs < HOT_FOR => Some(HOT_MEAL),
        Meal::Cooling => Some((-2.0, 180.0)),
        _ => None,
    }
}

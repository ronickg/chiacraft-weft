//! Keeping food longer: drying in the sun and wind, smoking over a fire, salting. Each makes its
//! own food (`food::FOODS`: dried, smoked and salted meat, dried apple and berries) with its real
//! nutrition, the water gone, so it weighs less to carry and keeps far longer.
//!
//! A drying rack (`base:drying_rack`) dries what it holds by the air where it stands: in about a
//! game day in warm, dry, breezy air (25 °C, humidity 0.3), barely in damp or cold air, not at all
//! in the rain. Real sun-drying of jerky and fruit takes one to four days. A fire under it (within
//! `SMOKE_REACH` blocks below) smokes the meat instead, in any weather. Salt is rubbed in at the
//! crafting table; it comes from boiling water away in a furnace.

use crate::DAY_SECS;
use crate::food::{self, Category, FOODS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Dried,
    Smoked,
    Salted,
}

pub const METHODS: [Method; 3] = [Method::Dried, Method::Smoked, Method::Salted];

impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Method::Dried => "dried",
            Method::Smoked => "smoked",
            Method::Salted => "salted",
        }
    }
}

/// What a method makes of a food (`FOODS` index), if anything: raw meat and fish of any animal
/// dry, smoke or salt into one meat of each; every fresh fruit dries, apples into rings and the
/// rest as dried berries (wild berries, prickly pears); a fruit that keeps half as long as dried
/// fruit already (rose hips, which hold on the bush) has nothing to gain.
pub fn product(method: Method, food: usize) -> Option<&'static str> {
    let f = &FOODS[food];
    let meat = f.raw && matches!(f.category, Category::Meat | Category::Fish) && f.poison == 0.0;
    let preserved = PRODUCTS.contains(&f.name);
    let dried = food::find("base:dried_berries").map_or(0.0, |d| FOODS[d].keeps.0 + FOODS[d].keeps.1);
    let fruit = f.category == Category::Fruit
        && f.meal != food::Meal::Cooked
        && f.poison == 0.0
        && !preserved
        && (f.keeps.0 + f.keeps.1) * 2.0 <= dried;
    match method {
        Method::Dried if meat => Some("base:dried_meat"),
        Method::Dried if fruit && f.name == "base:apple" => Some("base:dried_apple"),
        Method::Dried if fruit => Some("base:dried_berries"),
        Method::Smoked if meat => Some("base:smoked_meat"),
        Method::Salted if meat => Some("base:salted_meat"),
        _ => None,
    }
}

/// Every preserved food.
pub const PRODUCTS: [&str; 5] = ["base:dried_meat", "base:smoked_meat", "base:salted_meat", "base:dried_apple", "base:dried_berries"];

/// Seconds a rack dries its food in the best air.
pub const DRY_SECS: f32 = DAY_SECS;
/// Seconds it smokes it over a fire (real hot smoking: hours to a day).
pub const SMOKE_SECS: f32 = DAY_SECS / 2.0;
/// Blocks below a rack a fire smokes it from.
pub const SMOKE_REACH: i32 = 2;
/// The heat a fire needs to smoke (`base_api::heat_of`: lava 1, a lit furnace 0.7; a torch's 0.25
/// isn't a fire to smoke over).
pub const SMOKE_HEAT: f32 = 0.5;
/// Ticks a furnace boils a bucket of water down to salt.
pub const SALT_TICKS: u32 = 400;

/// The share of drying done a second, in this air: warmth from 5 °C (none) to 25 (full, a little
/// more above), dryness from humidity 0.85 (none) to 0.3 (full), still air at 60% of a breeze's
/// 3 m/s; rain on it stops it.
pub fn dry_rate(celsius: f32, humidity: f32, wind: f32, rained_on: bool) -> f32 {
    if rained_on {
        return 0.0;
    }
    let warmth = ((celsius - 5.0) / 20.0).clamp(0.0, 1.25);
    let dryness = ((0.85 - humidity) / 0.55).clamp(0.0, 1.0);
    let breeze = 0.6 + 0.4 * (wind / 3.0).clamp(0.0, 1.0);
    warmth * dryness * breeze / DRY_SECS
}

/// The share of smoking done a second over a fire.
pub fn smoke_rate() -> f32 {
    1.0 / SMOKE_SECS
}

/// Game days a preserved food keeps over the fresh one it was made from (fresh + stale).
pub fn lasts_over(product: &str, from: &str) -> f32 {
    let days = |n: &str| food::find(n).map_or(0.0, |i| FOODS[i].keeps.0 + FOODS[i].keeps.1);
    days(product) / days(from).max(0.01)
}

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

/// What a method makes of a food (`FOODS` index), if anything: raw meat of any animal dries,
/// smokes or salts into one meat of each; apples and berries dry.
pub fn product(method: Method, food: usize) -> Option<&'static str> {
    let f = &FOODS[food];
    let meat = f.raw && f.category == Category::Meat && f.poison == 0.0;
    match method {
        Method::Dried if meat => Some("base:dried_meat"),
        Method::Dried if f.name == "base:apple" => Some("base:dried_apple"),
        Method::Dried if f.name == "base:berries" => Some("base:dried_berries"),
        Method::Smoked if meat => Some("base:smoked_meat"),
        Method::Salted if meat => Some("base:salted_meat"),
        _ => None,
    }
}

/// Seconds a rack dries its food in the best air.
pub const DRY_SECS: f32 = DAY_SECS;
/// Seconds it smokes it over a fire (real hot smoking: hours to a day).
pub const SMOKE_SECS: f32 = DAY_SECS / 2.0;
/// Blocks below a rack a fire smokes it from.
pub const SMOKE_REACH: i32 = 2;
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

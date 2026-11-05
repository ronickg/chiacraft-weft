//! Survival rules that need no game world, the same for players and animals
//! (`notes/SURVIVAL.md`): the nine stats and what points make of them, levels and XP, the stomach
//! and the nutrients of every food (from real nutrition facts), freshness, meals and preserving, body temperature and clothing, the
//! effects and what they change, stamina's costs, and the difficulty presets. The base mod wires
//! them to the player, the animals mod to its animals; both read the same numbers here.
//!
//! The rule that ties it together: **a stat sets the ceiling, food sets how much of it is
//! reached.** Well fed, max health and stamina are the stats' values; on an empty stomach they
//! fall to a floor, never to death.

pub mod air;
pub mod balance;
pub mod calendar;
pub mod climate;
pub mod clothing;
pub mod crafts;
pub mod crops;
pub mod difficulty;
pub mod effects;
pub mod felling;
pub mod first;
pub mod foliage;
pub mod food;
pub mod forage;
pub mod impact;
pub mod layers;
pub mod levels;
pub mod meals;
pub mod plants;
pub mod preserve;
pub mod sheet;
pub mod shelter;
pub mod soil;
pub mod stamina;
pub mod stats;
pub mod stomach;
pub mod temperature;
pub mod weather_sound;
pub mod wild;

/// Ticks a second, as the game's.
pub const TICKS: u32 = 20;
/// A game day in seconds (20 minutes, as the game's).
pub const DAY_SECS: f32 = 1200.0;

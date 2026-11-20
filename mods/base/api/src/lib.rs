//! What other mods use of the base game, as types: its events with their values, components,
//! value chains, recipe kinds, equipment slots, creative tabs and dimensions. A mod adds
//! `base_api` to its `Cargo.toml` and gets these checked when it compiles, without depending on
//! the game's code; the engine still compares the value layouts when the mods load.
//!
//! Names here are full (`base:..`): the statics compile into whichever mod uses them.

/// Shared components, their value types, and food-item helpers.
pub mod components;

/// Events emitted by the base game.
pub mod events;

/// Item and block tags exposed by the base game.
pub mod tags;

/// Attributes exposed by the base game.
pub mod attributes;

/// Value chains exposed by the base game.
pub mod chains;

/// HUD elements exposed by the base game.
pub mod hud;

pub mod dimensions;

/// Vanilla's entity kinds.
pub mod entities;

/// What is in the sky of the game's dimensions. Any of them can be changed or taken out by a mod:
/// `r.edit(OVERWORLD.sky(&MOON)).size(40.0)`, `r.remove(OVERWORLD.sky(&STARS))`.
pub mod sky_objects;

/// Vanilla's blocks.
pub mod blocks;

/// Vanilla's menus.
pub mod menus;

/// Vanilla's items.
pub mod items;

/// Vanilla's equipment slots.
pub mod slots;

/// Vanilla's recipe types.
pub mod recipes;

/// Vanilla's creative tabs.
pub mod tabs;

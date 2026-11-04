//! chiacraft's own parts of the engine, kept apart from Weft's so merging Weft stays easy
//! (`notes/PLAN.md`, rule 4; `notes/OVERVIEW.md` for the pieces): the account screen, trust in
//! signed packages, linked worlds, and the bridge that answers what mods ask beyond Weft's SDK.
//! The crate re-exports each module under its old name, so Weft's files that call them stay as
//! they were.

pub mod account_screen;
pub mod bridge;
pub mod chain;
pub mod did;
pub mod package_trust;
pub mod worlds;

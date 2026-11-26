//! Test only (Ronald, 11-06: "let's load that same Minecraft world ... it makes testing even
//! easier"): reads the Minecraft world our Bliss + Distant Horizons comparison shots were taken in,
//! so the game can load it block for block and be shot from the same camera.
//!
//! - `anvil`: a world's region files (`r.X.Z.mca`, NBT chunks with 1.18+ palette sections and
//!   biomes, as Minecraft 26.3 writes them).
//! - `dh`: Distant Horizons' far data (its block-resolution columns, exported from its sqlite by
//!   `tools/mcimport/dh_export.py`), for the far view beyond the real chunks.
//! - `map`: Minecraft block states and biomes to ours, each a list of candidates the engine
//!   tries in order (the first that exists wins; the rest are the nearest matches).
//! - `far`: the far-terrain grid the engine's distant tiles read instead of the terrain formula.
//!
//! Nothing here knows the engine: names go in and out, the engine resolves them
//! (`weft_engine::chiacraft::mc_import`). Ronald's world itself is local only, never committed.

pub mod anvil;
pub mod dh;
pub mod far;
pub mod map;
pub mod nbt;

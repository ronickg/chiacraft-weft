//! Waystones and the map (`notes/PROGRESSION.md`, "From the survival research": "a map that fills
//! where you walk, with waystones (ancient ley nodes you wake)"). The waystones stand on the ley
//! network's great crossings (`ley`); base's `waystones.rs` wakes them and travels between them,
//! `map.rs` draws the map.
//!
//! - **Waking:** at a sleeping waystone, hold use with the dowsing rod for `WAKE_TICKS` with the
//!   offering (`OFFERING`, a tektite: star glass given back to the ancients' stone, 5 effort minutes
//!   in the Stone Age by `PROGRESSION_SHEET.md`) in the inventory. Woken waystones are the player's
//!   own (a tribe will share its members' later).
//! - **Travel:** at a woken waystone, pick another woken one on the map; after a `CHANNEL_SECS`
//!   channel that moving, breaking a block or being hurt cancels, the player is there. It costs
//!   the stomach what walking `TRAVEL_SHARE` of the way would (`travel_secs`), so a long trip is
//!   cheap in time and still costs food: walking stays worth it for what it finds.
//! - **The map:** where the player walked or saw, in tiles of `TILE` blocks, kept by the server per
//!   player in regions of `REGION` x `REGION` tiles (a bit each).

use crate::curve::Em;

/// What waking a waystone takes from the inventory (item, count).
pub const OFFERING: (&str, u32) = ("base:tektite", 1);
/// The offering's price in effort minutes (`PROGRESSION_SHEET.md`: a tektite is 5.0).
pub const OFFERING_EM: Em = 5.0;
/// Holding use with the rod at the stone this long wakes it (ticks).
pub const WAKE_TICKS: u32 = 80;
/// How near the stone's node the player stands to wake it or travel from it (blocks, across).
pub const NEAR: f64 = 7.0;

/// Seconds of the travel channel, and how far the player may drift in it (blocks).
pub const CHANNEL_SECS: f32 = 5.0;
pub const CHANNEL_DRIFT: f64 = 0.6;
/// A walker's pace over open land (blocks a second): the game's walk, 4.3.
pub const WALK_SPEED: f32 = 4.3;
/// The share of the walk's food a trip costs.
pub const TRAVEL_SHARE: f32 = 0.2;
/// Where the traveller arrives: this many blocks south of the stone's node, on the ground.
pub const ARRIVE_SOUTH: i32 = 4;

/// The stomach's seconds of digesting a trip of `dist` blocks costs (`Stomach::digest` at the
/// world's pace): `TRAVEL_SHARE` of the walk.
pub fn travel_secs(dist: f64) -> f32 {
    TRAVEL_SHARE * dist as f32 / WALK_SPEED
}

/// A trip's price in effort minutes: the walk's minutes it saves, less what it costs, is the gain;
/// this is the cost (the food, at a minute of digesting a minute of effort).
pub fn travel_em(dist: f64) -> Em {
    travel_secs(dist) / 60.0
}

/// Blocks a map tile covers on a side: a chunk.
pub const TILE: i32 = 16;
/// Tiles a saved region holds on a side (32 x 32 tiles: 512 blocks, 128 bytes).
pub const REGION: i32 = 32;
/// Tiles round the player marked seen, each way (64 blocks: about what the eye takes in on foot).
pub const SEEN_TILES: i32 = 4;
/// Ticks between marking what a player sees.
pub const SEEN_EVERY: u64 = 20;
/// Ticks between saving the maps that changed.
pub const SAVE_EVERY: u64 = 1200;

/// The map's picture (GUI pixels a side, the image's own pixels) and its zooms: blocks a pixel.
pub const MAP_PX: u32 = 256;
pub const ZOOMS: [i32; 4] = [1, 2, 4, 8];
/// The zoom it opens at (an index into `ZOOMS`): 2 blocks a pixel, 512 blocks across.
pub const ZOOM_START: usize = 1;
/// Blocks a pixel the tiles' colours are worked out at (the finest zoom draws each twice).
pub const TILE_PX_BLOCKS: i32 = 2;

/// The minimap in the HUD's top right corner, round (Ronald: "a kind of round mini map", the compass
/// map on his UI sheet): its picture's side (GUI pixels, one image pixel each), cut to the circle
/// that fills it; the ring round it (`gui/minimap.png`, drawn by `notes/ui/kit2/minimap_ring.py`:
/// its side and its four letter plates' side, GUI pixels); its zooms (blocks a pixel; N steps them)
/// and how often it is painted again while the player moves (seconds). It reads the same atlas as
/// the map: no world rendering.
pub const MINI_PX: u32 = 64;
pub const MINI_RING: f32 = 76.0;
pub const MINI_PLATE: f32 = 11.0;
pub const MINI_ZOOMS: [i32; 3] = [1, 2, 4];
pub const MINI_ZOOM_START: usize = 1;
pub const MINI_REPAINT_SECS: f32 = 0.4;

/// Parchment where nothing is known, and its darker grain.
pub const PARCHMENT: [u8; 3] = [0xd9, 0xc8, 0x9e];
pub const PARCHMENT_DARK: [u8; 3] = [0xc6, 0xb2, 0x84];
/// The map's colour of each overworld biome: the land's own colour (`land_map`), laid on the
/// parchment so it reads as an old painted chart.
pub const BIOME_COLOURS: [(&str, [u8; 3]); 15] = [
    ("ocean", [0x4a, 0x76, 0x96]),
    ("coral_reef", [0x5a, 0x9a, 0xa4]),
    ("lagoon", [0x6a, 0xa6, 0xaa]),
    ("beach", [0xe0, 0xcf, 0x98]),
    ("mountains", [0x8e, 0x8c, 0x80]),
    ("snowy_plains", [0xe8, 0xe8, 0xe0]),
    ("taiga", [0x5e, 0x7a, 0x58]),
    ("sequoia_grove", [0xa4, 0x5e, 0x3e]),
    ("desert", [0xe0, 0xc4, 0x84]),
    ("savanna", [0xb8, 0xa8, 0x5c]),
    ("swamp", [0x6e, 0x78, 0x4e]),
    ("forest", [0x5c, 0x86, 0x48]),
    ("tropical_forest", [0x3e, 0x7a, 0x4a]),
    ("birch_forest", [0x8a, 0xa6, 0x6a]),
    ("plains", [0x9c, 0xb2, 0x68]),
];
/// Rivers and lakes (ground under the sea's level inland).
pub const WATER: [u8; 3] = [0x5a, 0x86, 0xa4];
/// The share of the parchment kept under a biome's colour.
pub const PARCHMENT_UNDER: f32 = 0.25;

/// A biome's map colour (by its bare name, `plains`), plains' for one not listed.
pub fn biome_colour(biome: &str) -> [u8; 3] {
    let b = biome.trim_start_matches("base:");
    let c = BIOME_COLOURS
        .iter()
        .find(|e| e.0 == b)
        .map_or(BIOME_COLOURS[14].1, |e| e.1);
    let mix = |a: u8, p: u8| (a as f32 * (1.0 - PARCHMENT_UNDER) + p as f32 * PARCHMENT_UNDER).round() as u8;
    [mix(c[0], PARCHMENT[0]), mix(c[1], PARCHMENT[1]), mix(c[2], PARCHMENT[2])]
}

/// The ley lines on the map (with the rod held or ley sight), the woken waystones, the player.
pub const MAP_LEY: [u8; 3] = [0x6c, 0xc8, 0xf0];
pub const MAP_WOKEN: [u8; 3] = [0x2c, 0x9c, 0xe0];
pub const MAP_PLAYER: [u8; 3] = [0xb0, 0x30, 0x24];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn travel_is_cheap_but_not_free() {
        for d in [200.0, 1000.0, 5000.0] {
            let walk_em = d as f32 / WALK_SPEED / 60.0;
            assert!(travel_em(d) > 0.0 && travel_em(d) <= walk_em * 0.25, "{d}: {} of {walk_em}", travel_em(d));
        }
        // A trip of a thousand blocks costs under a minute of food, and the offering is worth a
        // few such trips.
        assert!(travel_em(1000.0) < 1.0 && OFFERING_EM > travel_em(1000.0));
    }

    #[test]
    fn every_biome_has_a_map_colour() {
        for b in ["plains", "ocean", "coral_reef", "lagoon", "sequoia_grove", "tropical_forest"] {
            assert!(BIOME_COLOURS.iter().any(|e| e.0 == b), "{b}");
        }
        assert_ne!(biome_colour("desert"), biome_colour("plains"));
    }
}

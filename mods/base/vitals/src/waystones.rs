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
pub const BIOME_COLOURS: [(&str, [u8; 3]); 16] = [
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
    ("old_growth_forest", [0x3e, 0x6a, 0x36]),
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
        .map_or(BIOME_COLOURS[15].1, |e| e.1);
    let mix = |a: u8, p: u8| (a as f32 * (1.0 - PARCHMENT_UNDER) + p as f32 * PARCHMENT_UNDER).round() as u8;
    [mix(c[0], PARCHMENT[0]), mix(c[1], PARCHMENT[1]), mix(c[2], PARCHMENT[2])]
}

/// What tops the ground (`base_api::land::top`) by name, its colour on the map (Ronald 11-22: "what
/// is that gray area? Isn't it supposed to be green?": grassy mountains read as grey stone when the
/// map went by the biome). Rock faces by their rocks (`rock:<set>`).
pub const GROUND_COLOURS: [(&str, [u8; 3]); 11] = [
    ("grass", [0x6e, 0xa0, 0x48]),
    ("snowy_grass", [0xc8, 0xd8, 0xc0]),
    // The conifers' floor: a darker grass green (about 20 under the plains' in each channel), so the
    // patches it frays into at the taiga's border read as soft texture (runs mm1122_c, _d).
    ("podzol", [0x66, 0x92, 0x46]),
    ("sand", [0xe0, 0xcc, 0x8c]),
    ("gravel", [0x8a, 0x86, 0x80]),
    ("snow", [0xf0, 0xf2, 0xf4]),
    ("rock:strata", [0xb4, 0x6c, 0x44]),
    ("rock:slate", [0x5c, 0x60, 0x66]),
    ("rock:granite", [0x9a, 0x86, 0x7e]),
    ("rock:limestone", [0xb4, 0xb0, 0xa0]),
    ("rock:karst", [0xa4, 0xa4, 0x96]),
];
/// Grass takes a tint by its biome (the woods darker, the savanna dry, the heights alpine), this
/// share of it over `GROUND_COLOURS`' grass; a biome not listed keeps the plain grass.
pub const GRASS_TINTS: [(&str, [u8; 3]); 10] = [
    ("plains", [0x8c, 0xb4, 0x58]),
    ("forest", [0x4e, 0x84, 0x3c]),
    ("old_growth_forest", [0x36, 0x68, 0x30]),
    ("tropical_forest", [0x34, 0x7c, 0x40]),
    ("birch_forest", [0x80, 0xa8, 0x5c]),
    ("taiga", [0x58, 0x7a, 0x50]),
    ("swamp", [0x64, 0x74, 0x44]),
    ("savanna", [0xb0, 0xa8, 0x58]),
    ("mountains", [0x7a, 0x98, 0x5e]),
    ("sequoia_grove", [0x4a, 0x78, 0x40]),
];
pub const GRASS_TINT: f32 = 0.6;

/// The map's colour of a column whose top is `top` (a `GROUND_COLOURS` name, or `water`) in
/// `biome`: the seas by their biome's colour, other water `WATER`, grass tinted by the biome, the
/// rest by the ground alone; laid on the parchment like the biomes' (`PARCHMENT_UNDER`).
pub fn ground_colour(top: &str, biome: &str) -> [u8; 3] {
    let b = biome.trim_start_matches("base:");
    if top == "water" {
        return if matches!(b, "ocean" | "coral_reef" | "lagoon") { biome_colour(b) } else { WATER };
    }
    let own = |name: &str| GROUND_COLOURS.iter().find(|e| e.0 == name).map(|e| e.1);
    let grass = own("grass").unwrap();
    let c = match (top, GRASS_TINTS.iter().find(|e| e.0 == b)) {
        ("grass", Some(t)) => mix(grass, t.1, GRASS_TINT),
        _ => own(top).unwrap_or(grass),
    };
    mix(c, PARCHMENT, PARCHMENT_UNDER)
}

fn mix(a: [u8; 3], b: [u8; 3], share_b: f32) -> [u8; 3] {
    [0, 1, 2].map(|k| (a[k] as f32 * (1.0 - share_b) + b[k] as f32 * share_b).round() as u8)
}

/// The ley lines on the map (with the rod held or ley sight), the woken waystones, the player.
pub const MAP_LEY: [u8; 3] = [0x6c, 0xc8, 0xf0];
pub const MAP_WOKEN: [u8; 3] = [0x2c, 0x9c, 0xe0];
pub const MAP_PLAYER: [u8; 3] = [0xb0, 0x30, 0x24];

/// Creatures on the minimap and on the map round the player (Ronald 11-22: "show all mobs in
/// certain area that are visible"): those within `MARK_RADIUS` blocks whose eyes the player's eyes
/// can see (a ray through the blocks the client has, `clear_line`; hidden behind a hill or a wall,
/// not shown), the nearest `MARK_MAX`, as dots of `MARK_DOT` pixels across. Looked for again at the
/// minimap's repaint (`MINI_REPAINT_SECS`); setting `base:map_creatures`.
pub const MARK_RADIUS: f64 = 48.0;
pub const MARK_MAX: usize = 24;
pub const MARK_DOT: i32 = 3;
/// A dot nearer the player's arrow than this (pixels from its middle: the arrow reaches 5.5) is
/// moved out to it, so the arrow never hides one.
pub const MARK_CLEAR: f32 = 8.0;
/// Steps of the sight ray (blocks): a block thinner than this between could be missed.
pub const MARK_RAY_STEP: f64 = 0.5;
/// The dots' colours: hostile mobs, wild animals (a neutral cream), the player's own tames; and
/// the dark rim round each so it reads on any land.
pub const MARK_HOSTILE: [u8; 3] = [0xd8, 0x28, 0x20];
pub const MARK_WILD: [u8; 3] = [0xf4, 0xec, 0xd4];
pub const MARK_TAME: [u8; 3] = [0x48, 0xc8, 0x58];
pub const MARK_RIM: [u8; 3] = [0x24, 0x1c, 0x14];

/// What a creature is on the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    Hostile,
    Wild,
    Tame,
}

impl Mark {
    /// By whether it is hostile (the `hostile` tag or the `core:hostile` spawn category), a
    /// creature at all (a living thing: not an item, an arrow or a boat) and the player's own.
    /// `None`: not marked.
    pub fn of(hostile: bool, creature: bool, mine: bool) -> Option<Mark> {
        if !creature && !hostile {
            return None;
        }
        Some(if mine {
            Mark::Tame
        } else if hostile {
            Mark::Hostile
        } else {
            Mark::Wild
        })
    }

    pub fn colour(self) -> [u8; 3] {
        match self {
            Mark::Hostile => MARK_HOSTILE,
            Mark::Wild => MARK_WILD,
            Mark::Tame => MARK_TAME,
        }
    }
}

/// Whether nothing `blocked` stands between `from` and `to` (the blocks the ray passes in steps of
/// `MARK_RAY_STEP`, the end points' own blocks left out: the eyes are in air, and the creature's
/// own block is where it stands).
pub fn clear_line(from: [f64; 3], to: [f64; 3], mut blocked: impl FnMut([i32; 3]) -> bool) -> bool {
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let steps = (len / MARK_RAY_STEP).ceil() as i32;
    let block = |p: [f64; 3]| p.map(|v| v.floor() as i32);
    let (start, end) = (block(from), block(to));
    let mut last = start;
    for k in 1..steps {
        let t = k as f64 / steps as f64;
        let b = block([from[0] + d[0] * t, from[1] + d[1] * t, from[2] + d[2] * t]);
        if b == last || b == start || b == end {
            continue;
        }
        last = b;
        if blocked(b) {
            return false;
        }
    }
    true
}

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
        for b in ["plains", "ocean", "coral_reef", "lagoon", "sequoia_grove", "tropical_forest", "old_growth_forest"] {
            assert!(BIOME_COLOURS.iter().any(|e| e.0 == b), "{b}");
        }
        assert_ne!(biome_colour("desert"), biome_colour("plains"));
    }

    #[test]
    fn the_ground_shows_what_tops_it() {
        let green = |c: [u8; 3]| c[1] > c[0] && c[1] > c[2];
        // Grass is green in every biome, the mountains' included (they read grey stone before).
        for b in ["mountains", "plains", "taiga", "swamp", "savanna", "old_growth_forest", "desert", "base:forest"] {
            assert!(green(ground_colour("grass", b)), "{b}: {:?}", ground_colour("grass", b));
        }
        assert_ne!(ground_colour("grass", "forest"), ground_colour("grass", "plains"));
        // Snow white, a granite face grey, sand pale, the sea blue.
        let s = ground_colour("snow", "mountains");
        assert!(s.iter().all(|&v| v > 0xd0));
        let g = ground_colour("rock:granite", "mountains");
        assert!(!green(g) && g.iter().max().unwrap() - g.iter().min().unwrap() < 0x30);
        let sea = ground_colour("water", "ocean");
        assert!(sea[2] > sea[0] && ground_colour("water", "plains") == WATER);
        assert!(ground_colour("sand", "desert")[0] > 0xd0);
        // Every ground block the land tops a column with has a colour.
        for top in
            ["grass", "snowy_grass", "podzol", "sand", "snow", "rock:strata", "rock:slate", "rock:granite", "rock:limestone", "rock:karst"]
        {
            assert!(GROUND_COLOURS.iter().any(|e| e.0 == top), "{top}");
        }
    }

    #[test]
    fn creatures_are_marked_by_kind() {
        assert_eq!(Mark::of(true, true, false), Some(Mark::Hostile));
        assert_eq!(Mark::of(false, true, false), Some(Mark::Wild));
        assert_eq!(Mark::of(false, true, true), Some(Mark::Tame));
        assert_eq!(Mark::of(false, false, false), None);
        let c = [Mark::Hostile, Mark::Wild, Mark::Tame].map(Mark::colour);
        assert!(c[0] != c[1] && c[1] != c[2] && c[0] != c[2]);
        // The radius fits the minimap's widest zoom (its circle's radius in blocks) and the cap
        // keeps the dots readable.
        let widest = MINI_PX as f64 / 2.0 * *MINI_ZOOMS.last().unwrap() as f64;
        assert!(MARK_RADIUS <= widest && MARK_RADIUS >= MINI_PX as f64 / 2.0 && (8..=64).contains(&MARK_MAX));
    }

    #[test]
    fn sight_stops_at_walls_and_hills() {
        let eye = [0.5, 65.6, 0.5];
        // Open ground: nothing blocks.
        assert!(clear_line(eye, [20.5, 65.6, 3.5], |_| false));
        // A wall at x = 10 across the way.
        let wall = |b: [i32; 3]| b[0] == 10 && (60..70).contains(&b[1]);
        assert!(!clear_line(eye, [20.5, 65.6, 3.5], wall));
        // Seen over it from higher up.
        assert!(clear_line([0.5, 75.6, 0.5], [20.5, 75.6, 3.5], wall));
        // A hill: ground up to 70 between x 5 and 15, the creature in the valley beyond.
        let hill = |b: [i32; 3]| (5..15).contains(&b[0]) && b[1] < 70 || b[1] < 64;
        assert!(!clear_line(eye, [25.5, 64.6, 0.5], hill));
        // Its own block and the eyes' are never asked.
        assert!(clear_line(eye, [3.5, 65.6, 0.5], |b| b == [3, 65, 0] || b == [0, 65, 0]));
    }
}

//! A room's warmth (`notes/CONTENT.md` decision 7, taken overnight 11-04; the numbers are the
//! content research's, `reports/content/Building and workstations.md` B3): an enclosed space keeps
//! the wind and the sky off, its walls even out the day's swing by their material, and a fire in it
//! warms it by how well its walls keep heat in. Logs and earth keep heat (a log wall loses a third
//! to a quarter of what a stone wall does), stone and brick hold it like a cellar but let it go,
//! hide and felt stop the wind and little else, glass is poor.
//!
//! The base game finds the room (`survival/shelter.rs`: a flood fill of the air around the player)
//! and classes its walls; here is what they come to.

/// What a wall face is made of, for warmth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wall {
    /// Logs (a chinked log wall, an izba).
    Log,
    /// Earth: dirt, sod, adobe, cob, sand, clay.
    Earth,
    /// Snow blocks (a snow house).
    Snow,
    /// Planks, wattle and daub, and anything else built of wood.
    Plank,
    /// Stone, cobble, brick.
    Stone,
    /// Hide, felt, wool, leaves (a tent, a hut of brush).
    Hide,
    /// Glass.
    Glass,
}

pub const WALLS: [Wall; 7] = [Wall::Log, Wall::Earth, Wall::Snow, Wall::Plank, Wall::Stone, Wall::Hide, Wall::Glass];

impl Wall {
    pub fn name(self) -> &'static str {
        match self {
            Wall::Log => "log",
            Wall::Earth => "earth",
            Wall::Snow => "snow",
            Wall::Plank => "plank",
            Wall::Stone => "stone",
            Wall::Hide => "hide",
            Wall::Glass => "glass",
        }
    }

    /// Degrees a fire or lamp in a closed room of these walls warms it (B3: hide and felt +12;
    /// wattle, plank, stone, brick +16; log, sod, adobe, cob, snow +25; glass, the research's
    /// "poor", taken as hide's).
    pub fn fire_gain(self) -> f32 {
        match self {
            Wall::Log | Wall::Earth | Wall::Snow => 25.0,
            Wall::Plank | Wall::Stone => 16.0,
            Wall::Hide | Wall::Glass => 12.0,
        }
    }

    /// The share of the outside's day swing that gets in (B3: earth, stone, adobe, sod 0.15; log,
    /// brick 0.4; plank, wattle 0.6; hide, felt, snow 0.9; glass as hide).
    pub fn swing(self) -> f32 {
        match self {
            Wall::Earth | Wall::Stone => 0.15,
            Wall::Log => 0.4,
            Wall::Plank => 0.6,
            Wall::Hide | Wall::Snow | Wall::Glass => 0.9,
        }
    }
}

/// Degrees an empty closed room keeps over the outside air (B3: +2).
pub const EMPTY_ROOM: f32 = 2.0;
/// The most air a room holds, in blocks: past this the fill gives up, and it's the outdoors (a
/// big hall is no warmer than a roof).
pub const ROOM_MAX: usize = 400;
/// A room a fire warms fully, in blocks of air (a cabin of 4 x 5 x 3); a bigger one less, as its
/// heat spreads (proposed).
pub const ROOM_FULL: usize = 60;
/// The least share of a fire's gain a big room gets.
pub const BIG_ROOM_SHARE: f32 = 0.4;

/// A room's walls, counted by face.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Room {
    /// Faces of each class, in `WALLS` order.
    pub faces: [u32; 7],
    /// Blocks of air in it.
    pub volume: usize,
}

impl Room {
    fn total(&self) -> f32 {
        self.faces.iter().sum::<u32>().max(1) as f32
    }

    /// The walls' average, weighted by face.
    fn mean(&self, f: impl Fn(Wall) -> f32) -> f32 {
        WALLS
            .iter()
            .zip(self.faces)
            .map(|(&w, n)| f(w) * n as f32)
            .sum::<f32>()
            / self.total()
    }

    /// The class most of its faces are.
    pub fn main_wall(&self) -> Wall {
        let i = (0..WALLS.len()).max_by_key(|&i| self.faces[i]).unwrap_or(0);
        WALLS[i]
    }

    /// The share of the day's swing that gets in.
    pub fn swing(&self) -> f32 {
        self.mean(Wall::swing)
    }

    /// Degrees a fire in it warms it, for its walls and its size.
    pub fn fire_gain(&self) -> f32 {
        let size = (ROOM_FULL as f32 / self.volume.max(1) as f32).clamp(BIG_ROOM_SHARE, 1.0);
        self.mean(Wall::fire_gain) * size
    }

    /// The air in the room, from the outside's day mean, the air out of the wind without a fire
    /// (`still`), and a fire's degrees here (`fire`, 0 for none): the swing evened out by the
    /// walls, then the fire's gain (at least what it gives beside it) or the empty room's.
    pub fn air(&self, mean: f32, still: f32, fire: f32) -> f32 {
        let inside = mean + (still - mean) * self.swing();
        let gain = if fire > 0.0 { self.fire_gain().max(fire) } else { EMPTY_ROOM };
        inside + gain
    }
}

//! Clothes: warmth against the cold (and some against the heat), how much of it they keep wet, and
//! how much wind they stop. No armor: clothes are worn in their own four slots under armor (hat,
//! coat, trousers, boots), and armor keeps its own insulation (`temperature::set_insulation`).
//!
//! A material's warmth is its real insulation in clo (ISO 9920's garments, arctic fur
//! ensembles) at `PER_CLO` degrees of comfort each, for a person moving about; a piece gives its
//! share of the full set (`temperature::PIECE_SHARE`: head 0.15, body 0.4, legs 0.3, feet 0.15).

use crate::temperature::{self, PIECE_SHARE};

/// Degrees lower the comfortable air goes for each clo worn (about 6 for someone walking about:
/// 1 clo is a business suit, the line between 15 °C and 21 °C).
pub const PER_CLO: f32 = 6.0;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    /// Its name in items (`base:fur_coat`) and translations.
    pub name: &'static str,
    /// The full set's insulation, clo.
    pub clo: f32,
    /// Degrees of heat insulation: light, loose cloth shades one in the sun (+), fur overheats (-).
    pub heat: f32,
    /// The share of its warmth it keeps soaked: wool and fur stay warm wet, linen doesn't.
    pub wet_keeps: f32,
    /// The share of the wind it stops (leather and fur are windproof, knitted wool isn't).
    pub windproof: f32,
    /// What it's made of (`materials/<name>`... in the recipes), for the sheet.
    pub from: &'static str,
}

impl Material {
    /// Degrees of cold insulation of the full set.
    pub fn cold(&self) -> f32 {
        self.clo * PER_CLO
    }
}

/// The clothing materials. Fur: caribou and wolf pelts, Inuit-style (3.5 clo for the suit); wool:
/// a heavy sweater, trousers and socks (1.8); hide: buckskin (1.2); linen from plant fibre: a light
/// shirt and trousers (0.5), cooler than bare skin in the sun.
pub const MATERIALS: [Material; 4] = [
    Material { name: "fur", clo: 3.5, heat: -8.0, wet_keeps: 0.5, windproof: 0.8, from: "pelts of furred wild animals" },
    Material { name: "wool", clo: 1.8, heat: -3.0, wet_keeps: 0.6, windproof: 0.3, from: "sheep's wool" },
    Material { name: "hide", clo: 1.2, heat: -1.0, wet_keeps: 0.7, windproof: 0.7, from: "leather" },
    Material { name: "linen", clo: 0.5, heat: 4.0, wet_keeps: 0.1, windproof: 0.1, from: "plant fibre spun and woven" },
];

/// The four pieces, head to feet, as items are named (`<material>_<piece>`).
pub const PIECES: [&str; 4] = ["hat", "coat", "trousers", "boots"];

/// The chance cutting tall grass gives plant fibre, for string and linen.
pub const FIBRE_CHANCE: f32 = 0.125;

/// What each piece takes to make: pieces of its material, and string to sew it.
pub const MAKES: [(u32, u32); 4] = [(2, 1), (4, 2), (3, 1), (2, 1)];

pub fn find(name: &str) -> Option<&'static Material> {
    MATERIALS.iter().find(|m| m.name == name)
}

/// A clothing item's material and piece (0 head to 3 feet) by its name (`base:wool_coat`).
pub fn piece_of(item: &str) -> Option<(&'static Material, usize)> {
    let n = item.split_once(':').map_or(item, |(_, n)| n);
    let (m, p) = n.split_once('_')?;
    Some((find(m)?, PIECES.iter().position(|&x| x == p)?))
}

/// What a body has on, summed over its armor and its clothes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Worn {
    pub cold: f32,
    pub heat: f32,
    /// The cold insulation still kept wet.
    pub cold_wet: f32,
    /// The share of the wind stopped, 0 to 1 (each piece over its share of the body).
    pub windproof: f32,
}

impl Worn {
    /// Puts on a piece of armor of that material in slot `piece` (0 head to 3 feet).
    pub fn armor(&mut self, material: &str, piece: usize) {
        let (c, h) = temperature::set_insulation(material);
        let share = PIECE_SHARE[piece.min(3)];
        self.cold += c * share;
        self.heat += h * share;
        self.cold_wet += c * share * temperature::ARMOR_WET_KEEPS;
    }

    /// Puts on a piece of clothing.
    pub fn clothes(&mut self, m: &Material, piece: usize) {
        let share = PIECE_SHARE[piece.min(3)];
        self.cold += m.cold() * share;
        self.heat += m.heat * share;
        self.cold_wet += m.cold() * share * m.wet_keeps;
        self.windproof = (self.windproof + m.windproof * share).min(1.0);
    }

    /// A full set of clothes of one material.
    pub fn set(m: &Material) -> Worn {
        let mut w = Worn::default();
        for p in 0..4 {
            w.clothes(m, p);
        }
        w
    }
}

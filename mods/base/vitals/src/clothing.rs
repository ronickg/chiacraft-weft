//! Clothes: warmth against the cold (and some against the heat), how much of it they keep wet, and
//! how much wind they stop. No armor: clothes are worn in their own four slots under armor (hat,
//! coat, trousers, boots), and armor keeps its own insulation (`temperature::set_insulation`).
//!
//! A material's warmth is its real insulation in clo at `PER_CLO` degrees of comfort each, for a
//! person moving about; a piece gives its share of the full set (`temperature::PIECE_SHARE`: head
//! 0.15, body 0.4, legs 0.3, feet 0.15). The sets and their numbers are the content plan's research
//! (`reports/content/Clothing and armor.md`, section E, on its branch): linen, buckskin, wool and
//! a single caribou-fur suit of its eleven; more sets are rows here.

use crate::temperature::{self, PIECE_SHARE};

/// Degrees lower the comfortable air goes for each clo worn, walking (ISO 11079, computed in the
/// content plan's research; about 7 at rest, so sleeping out could halve it later).
pub const PER_CLO: f32 = 12.0;

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
    /// Pieces of its material each piece takes, head to feet (string to sew it: `STRING`).
    pub makes: [u32; 4],
}

impl Material {
    /// Degrees of cold insulation of the full set.
    pub fn cold(&self) -> f32 {
        self.clo * PER_CLO
    }
}

/// The clothing materials (the research's sets). Fur: a single caribou suit, fur in (2.5 clo; the
/// double suit for the Arctic winter, 4 clo, is a later set); wool: hood, tunic, hose and socks
/// (1.2); hide: buckskin (0.8); linen: cap, shirt and trousers (0.5), loose enough with its hat to
/// shade one in the sun (between the research's linen, 0, and its desert robe, +6).
pub const MATERIALS: [Material; 4] = [
    Material {
        name: "fur",
        clo: 2.5,
        heat: -9.0,
        wet_keeps: 0.3,
        windproof: 0.8,
        from: "pelts of furred wild animals",
        makes: [2, 4, 3, 2],
    },
    Material { name: "wool", clo: 1.2, heat: -2.0, wet_keeps: 0.6, windproof: 0.3, from: "sheep's wool", makes: [1, 2, 1, 1] },
    Material { name: "hide", clo: 0.8, heat: -1.0, wet_keeps: 0.3, windproof: 0.7, from: "leather", makes: [1, 2, 1, 1] },
    Material {
        name: "linen",
        clo: 0.5,
        heat: 4.0,
        wet_keeps: 0.2,
        windproof: 0.1,
        from: "plant fibre spun and woven",
        makes: [2, 4, 3, 2],
    },
];

/// The four pieces, head to feet, as items are named (`<material>_<piece>`).
pub const PIECES: [&str; 4] = ["hat", "coat", "trousers", "boots"];

/// The chance cutting tall grass gives plant fibre, for string and linen.
pub const FIBRE_CHANCE: f32 = 0.125;

/// String each piece takes to sew, head to feet. Its material: `Material::makes` (hide and wool to
/// real hide counts, Research round 3 B3: a buckskin suit is three or four deer hides, a fleece
/// knits more than a sweater; fur stays near a caribou suit's, about four reindeer).
pub const STRING: [u32; 4] = [1, 2, 1, 1];
/// Plant fibre a string takes.
pub const FIBRE_PER_STRING: u32 = 2;

/// What a new player arrives in (Research round 3, B1): a linen hood, a wool sweater, buckskin
/// trousers and moccasins, about 0.9 clo, ordinary clothing's 0.8 to 1.2 (ISO 9920). With the year
/// starting in spring it makes the temperate biomes' first week cold rather than deadly.
pub const ARRIVAL: [(&str, usize); 4] = [("linen", 0), ("wool", 1), ("hide", 2), ("hide", 3)];

/// The clothing material an armor material is the same stuff as: leather armor is hide (Research
/// round 3, C1: it was warm two ways with two sets of numbers).
pub fn armor_cloth(material: &str) -> Option<&'static Material> {
    match material {
        "leather" => find("hide"),
        _ => None,
    }
}

pub fn find(name: &str) -> Option<&'static Material> {
    MATERIALS.iter().find(|m| m.name == name)
}

/// A clothing item's material and piece (0 head to 3 feet) by its name (`base:wool_coat`).
pub fn piece_of(item: &str) -> Option<(&'static Material, usize)> {
    let n = item.split_once(':').map_or(item, |(_, n)| n);
    let (m, p) = n.split_once('_')?;
    Some((find(m)?, PIECES.iter().position(|&x| x == p)?))
}

/// A cloak, worn over everything in a slot of its own (`notes/CONTENT.md` decision 3, taken
/// overnight 11-04: two layers and a cloak, nine slots). The research's cloaks: a wool cloak +4, a
/// hare-skin or fur robe +8, a gut parka or oiled cape +1 that keeps the rain off.
#[derive(Clone, Copy, Debug)]
pub struct Cloak {
    /// Its item (`base:wool_cloak`).
    pub name: &'static str,
    /// Degrees of cold and heat insulation.
    pub cold: f32,
    pub heat: f32,
    pub wet_keeps: f32,
    /// The share of the wind it stops, over the body it covers (`CLOAK_COVERS`).
    pub windproof: f32,
    /// Rain doesn't make one wet under it (water still does).
    pub keeps_dry: bool,
    pub from: &'static str,
}

pub const CLOAKS: [Cloak; 3] = [
    Cloak { name: "wool_cloak", cold: 4.0, heat: -1.0, wet_keeps: 0.6, windproof: 0.3, keeps_dry: false, from: "sheep's wool" },
    Cloak { name: "fur_robe", cold: 8.0, heat: -3.0, wet_keeps: 0.3, windproof: 0.6, keeps_dry: false, from: "pelts" },
    Cloak { name: "rain_cape", cold: 1.0, heat: 0.0, wet_keeps: 1.0, windproof: 0.5, keeps_dry: true, from: "leather" },
];

/// The share of the body a cloak covers, for the wind it stops.
pub const CLOAK_COVERS: f32 = 0.6;

/// A cloak by its item's name (`base:fur_robe`).
pub fn cloak_of(item: &str) -> Option<&'static Cloak> {
    let n = item.split_once(':').map_or(item, |(_, n)| n);
    CLOAKS.iter().find(|c| c.name == n)
}

/// How much of a piece of clothing an armour lining of its cloth is (a gambeson's padding under
/// mail, a fur-lined helmet; proposed).
pub const LINING_SHARE: f32 = 0.5;

/// What a body has on, summed over its armor and its clothes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Worn {
    pub cold: f32,
    pub heat: f32,
    /// The cold insulation still kept wet.
    pub cold_wet: f32,
    /// The share of the wind stopped, 0 to 1 (each piece over its share of the body).
    pub windproof: f32,
    /// Rain doesn't wet one (a rain cape).
    pub dry_in_rain: bool,
}

impl Worn {
    /// Puts on a piece of armor of that material in slot `piece` (0 head to 3 feet). Armor and
    /// clothes stack: leather armor over a hide coat is two layers of hide.
    pub fn armor(&mut self, material: &str, piece: usize) {
        if let Some(m) = armor_cloth(material) {
            self.clothes(m, piece);
            return;
        }
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

    /// An armour piece's lining in slot `piece` (`crafting`: a lining is clothing in numbers):
    /// `LINING_SHARE` of a piece of clothing of its cloth, a padded layer thinner than a coat.
    pub fn lining(&mut self, m: &Material, piece: usize) {
        let share = PIECE_SHARE[piece.min(3)] * LINING_SHARE;
        self.cold += m.cold() * share;
        self.heat += m.heat * share;
        self.cold_wet += m.cold() * share * m.wet_keeps;
        self.windproof = (self.windproof + m.windproof * share).min(1.0);
    }

    /// Puts on a cloak over everything.
    pub fn cloak(&mut self, c: &Cloak) {
        self.cold += c.cold;
        self.heat += c.heat;
        self.cold_wet += c.cold * c.wet_keeps;
        self.windproof = (self.windproof + c.windproof * CLOAK_COVERS).min(1.0);
        self.dry_in_rain |= c.keeps_dry;
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

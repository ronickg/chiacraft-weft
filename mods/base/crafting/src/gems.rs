//! Gems in gear (Ronald, 11-05: "pieces and items should also get like gem slots because that
//! will make it even cooler"; `notes/CRAFTING.md`, "Gem sockets"):
//! - **Sockets:** a built item has as many as its working part's grade gives (`grades::Grade::slots`:
//!   more on fine and masterwork parts), at most what its material can hold (`max_sockets`).
//! - **Gems** are cut from found stones at a station. Each rolls 1-3 boosts with the web's own
//!   roll (`vitals::sockets::roll_among`, `GEAR_FAMILIES`) in its stone's item level's band, so a
//!   deeper find is a little better and none is past the web's per-stone band.
//! - **Boosts** count only where they fit the item (`stat_of`): melee damage on a blade or a
//!   tool's hit, ranged on a bow, defence and health on armour, mining and gathering on a tool.
//!   Grade and gems together stay under the curve's ceiling (`vitals::curve::GRADE_CEILING`, +60%:
//!   "a perfect tier-N item is about a plain tier-N+1").
//! - **Out again:** taking a gem out gives it back; a cheap one (`cracks`) breaks one time in
//!   `CRACK_ONE_IN`.

use crate::kinds::{Kind, Use};
use crate::materials::{Material, Works};
use vitals::web::Family;

/// A kind of gem: the stone it's cut from, and whether it's cheap enough to crack coming out.
#[derive(Clone, Copy, Debug)]
pub struct Gem {
    pub id: &'static str,
    /// Its name's key (`gem.base.<id>`); its item is `base:gem_<id>`.
    pub key: &'static str,
    /// The found stone it's cut from (an item, `mod:name`).
    pub from: &'static str,
}

/// Item levels under this are cheap stones: they crack one time in `CRACK_ONE_IN` coming out.
pub const CHEAP_BELOW: f32 = 20.0;
pub const CRACK_ONE_IN: u32 = 4;

/// The gems, from today's found stones; a new one is a row (its level in `vitals::gear`).
pub const GEMS: &[Gem] = &[
    Gem { id: "diamond", key: "gem.base.diamond", from: "base:diamond" },
    // Peridot: a pallasite's olivine (`stars.rs`), the fallen stars' gem.
    Gem { id: "peridot", key: "gem.base.peridot", from: "base:peridot" },
    // Tektite: impact glass, moldavite's green.
    Gem { id: "tektite", key: "gem.base.tektite", from: "base:tektite" },
    // Amber: the animals mod's fossil resin, a polished cabochon.
    Gem { id: "amber", key: "gem.base.amber", from: "animals:amber" },
];

impl Gem {
    /// Its item level (`vitals::gear::MATERIALS`): where its stone is found.
    pub fn level(&self) -> f32 {
        vitals::gear::material(self.id).map_or(0.0, |m| m.level())
    }

    /// Whether it may crack when taken out.
    pub fn cracks(&self) -> bool {
        self.level() < CHEAP_BELOW
    }

    pub fn item(&self) -> String {
        format!("gem_{}", self.id)
    }
}

pub fn gem(id: &str) -> Option<&'static Gem> {
    GEMS.iter().find(|g| g.id == id)
}

/// The most sockets a working part of `m` can hold: a gem glued into knapped stone, bone or wood
/// holds one; leather (sewn) and diamond (hard to set into) two; a metal setting three.
pub fn max_sockets(m: &Material) -> u8 {
    match m.works {
        Works::Edge | Works::Ground | Works::Bone | Works::Wood => 1,
        Works::Skin | Works::Gem => 2,
        Works::Metal => 3,
        Works::None => 0,
    }
}

/// Sockets on an item whose working part is of `material` at grade `grade`.
pub fn sockets(material: &Material, grade: u8) -> u8 {
    crate::grades::grade(grade).slots.min(max_sockets(material))
}

/// What a boost multiplies on an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Damage,
    /// Mining, chopping, digging speed.
    Speed,
    Armour,
    Health,
}

/// What `family` multiplies on an item of `kind`, if it fits there.
pub fn stat_of(kind: &Kind, family: Family) -> Option<Stat> {
    let tool = matches!(kind.use_, Use::Chop | Use::Mine | Use::Dig | Use::Till);
    match (kind.use_, family) {
        (Use::Cut | Use::Thrust, Family::MeleeDamage) => Some(Stat::Damage),
        (Use::Shoot, Family::RangedDamage) => Some(Stat::Damage),
        (Use::Guard, Family::Defence) => Some(Stat::Armour),
        (Use::Guard, Family::Health) => Some(Stat::Health),
        (_, Family::MeleeDamage) if tool => Some(Stat::Damage),
        (_, Family::MineSpeed) if kind.use_ == Use::Mine || kind.use_ == Use::Dig => Some(Stat::Speed),
        (_, Family::GatherSpeed) if kind.use_ == Use::Chop || kind.use_ == Use::Till => Some(Stat::Speed),
        _ => None,
    }
}

/// The multipliers gems give an item of `kind` at its working part's grade `grade`, by stat
/// (`Stat` order: damage, speed, armour, health), each `1 + UNIT × units`, capped so grade and gems
/// together stay at most `GRADE_CEILING` times a plain item.
pub fn multipliers(kind: &Kind, grade: u8, boosts: &[(Family, f32)]) -> [f32; 4] {
    let mut units = [0.0f32; 4];
    for &(f, u) in boosts {
        if let Some(s) = stat_of(kind, f) {
            units[s as usize] += u;
        }
    }
    let cap = (vitals::curve::GRADE_CEILING / crate::grades::grade(grade).mult).max(1.0);
    units.map(|u| (1.0 + vitals::web::UNIT * u).min(cap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::kind;
    use crate::materials::material;

    #[test]
    fn every_gem_has_a_level() {
        for g in GEMS {
            assert!(vitals::gear::material(g.id).is_some(), "{}: no level in vitals::gear", g.id);
        }
        assert!(!gem("diamond").unwrap().cracks() && gem("tektite").unwrap().cracks());
    }

    #[test]
    fn sockets_from_grade_and_material() {
        let (flint, steel) = (material("flint").unwrap(), material("steel").unwrap());
        assert_eq!(sockets(steel, 3), 0, "Plain parts have none");
        assert_eq!(sockets(steel, 4), 1);
        assert_eq!(sockets(steel, 9), 3, "masterwork has the most");
        assert_eq!(sockets(flint, 9), 1, "knapped stone holds one");
    }

    #[test]
    fn boosts_fit_and_stay_under_the_ceiling() {
        let sword = kind("sword").unwrap();
        let m = multipliers(sword, 4, &[(Family::MeleeDamage, 5.0), (Family::Defence, 5.0)]);
        assert!((m[Stat::Damage as usize] - 1.1).abs() < 1e-5, "{m:?}");
        assert_eq!(m[Stat::Armour as usize], 1.0, "defence does nothing on a sword");
        // A legendary blade (x1.5) gains at most the rest of the ceiling.
        let top = multipliers(sword, 10, &[(Family::MeleeDamage, 30.0)]);
        assert!((1.5 * top[0] - vitals::curve::GRADE_CEILING).abs() < 1e-4);
        let coat = kind("chestplate").unwrap();
        assert!(multipliers(coat, 4, &[(Family::Health, 4.0)])[Stat::Health as usize] > 1.0);
    }
}

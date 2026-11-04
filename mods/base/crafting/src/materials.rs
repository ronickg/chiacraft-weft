//! What parts are made of (`notes/CRAFTING.md`, "Materials and traits"; the ladder and its
//! numbers are Content plan's, `notes/CONTENT.md`). Every row is a real material with its real
//! density; harvest level, uses and speed follow Content's table (today's iron tool is 250 uses,
//! today's wood 2 to diamond 8 in speed).
//!
//! Proposed until Content plan builds `vitals::gear`: these rows move there, and this crate reads
//! them from it.

use crate::kinds::Role;

/// How a part of this material is made the first time by hand (`notes/CRAFTING.md`, "Making
/// parts"), and so which station and game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Making {
    /// Struck off a core at the knapping stone.
    Knap,
    /// Split and carved (wood), ground and carved (bone, antler).
    Carve,
    /// Melted and poured into a mould, then the edge hammered.
    Cast,
    /// Worked hot at the forge and anvil (copper: cold, softened in the fire).
    Forge,
    /// Twisted, cut or cooked by hand: cordage, sinew, rawhide, birch tar.
    Hands,
    /// Cut and sewn at the sewing table.
    Sew,
}

/// One material.
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub id: &'static str,
    /// Its name's key.
    pub key: &'static str,
    /// What it can be, by role.
    pub roles: &'static [Role],
    /// g/cm³, for a part's mass.
    pub density: f32,
    /// How much bigger than a metal part of the same shape a part of it is (stone is thick, so it
    /// doesn't snap; Content, A3).
    pub bulk: f32,
    /// What a working part of it harvests (`abi::tool_tier`, a hundred apart).
    pub harvest: u32,
    /// Uses as a working part at Sound (grade 4), before shape and haft.
    pub uses: f32,
    /// Speed as a working part (wood 2 to steel 8).
    pub speed: f32,
    /// Damage it adds as a working part.
    pub damage: f32,
    /// As a holding part: what it adds to the working part's uses and speed (0.10 = +10%).
    pub hold_uses: f32,
    pub hold_speed: f32,
    /// As a fitting: uses before it gives (lashings go first, rivets last).
    pub fitting_uses: f32,
    /// Its trait (`traits::TRAITS`), "" for none.
    pub trait_id: &'static str,
    pub making: Making,
}

use Role::{Fitting as F, Holding as H, Working as W};

// A table row: one argument a column reads better here than a builder.
#[allow(clippy::too_many_arguments)]
const fn m(
    id: &'static str,
    key: &'static str,
    roles: &'static [Role],
    density: f32,
    bulk: f32,
    harvest: u32,
    uses: f32,
    speed: f32,
    damage: f32,
    hold: (f32, f32),
    fitting_uses: f32,
    trait_id: &'static str,
    making: Making,
) -> Material {
    Material {
        id,
        key,
        roles,
        density,
        bulk,
        harvest,
        uses,
        speed,
        damage,
        hold_uses: hold.0,
        hold_speed: hold.1,
        fitting_uses,
        trait_id,
        making,
    }
}

/// Densities: flint and chert 2.6, obsidian 2.4, bone and antler 1.9, copper 8.96, bronze 8.8,
/// wrought iron 7.7, steel 7.85, gold 19.3; seasoned oak 0.75, birch 0.65, spruce 0.45, mesquite
/// 0.88. Working-part numbers from Content's ladder (`notes/CONTENT.md`, "The ladder").
pub const MATERIALS: &[Material] = &[
    // Stone and bone (tier 1).
    m("flint", "material.base.flint", &[W], 2.6, 1.8, 100, 75.0, 3.0, 1.0, (0.0, 0.0), 0.0, "keen", Making::Knap),
    m("chert", "material.base.chert", &[W], 2.6, 1.8, 100, 70.0, 3.0, 1.0, (0.0, 0.0), 0.0, "common", Making::Knap),
    m("obsidian", "material.base.obsidian", &[W], 2.4, 1.8, 100, 40.0, 3.2, 1.5, (0.0, 0.0), 0.0, "razor", Making::Knap),
    m("bone", "material.base.bone", &[W, H], 1.9, 1.5, 50, 90.0, 2.5, 0.5, (0.05, 0.0), 0.0, "tough", Making::Carve),
    m("antler", "material.base.antler", &[W, H], 1.9, 1.5, 50, 90.0, 2.5, 0.5, (0.08, 0.0), 0.0, "tough", Making::Carve),
    // Today's plain stone tools (provisional until flint and chert), and diamond, a tier above
    // steel; more above it later.
    m("stone", "material.base.stone", &[W], 2.7, 1.8, 100, 70.0, 3.0, 1.0, (0.0, 0.0), 0.0, "common", Making::Knap),
    m("diamond", "material.base.diamond", &[W], 3.5, 1.0, 400, 1500.0, 8.0, 3.0, (0.0, 0.0), 0.0, "", Making::Carve),
    // Metals (tiers 3-6).
    m("copper", "material.base.copper", &[W, F], 8.96, 1.0, 150, 150.0, 5.0, 1.5, (0.0, 0.0), 120.0, "soft", Making::Forge),
    m("bronze", "material.base.bronze", &[W, F], 8.8, 1.0, 200, 250.0, 7.0, 2.0, (0.0, 0.0), 220.0, "recastable", Making::Cast),
    m("wrought_iron", "material.base.wrought_iron", &[W, F], 7.7, 1.0, 220, 275.0, 7.0, 2.0, (0.0, 0.0), 260.0, "weldable", Making::Forge),
    m("steel", "material.base.steel", &[W, F], 7.85, 1.0, 300, 500.0, 8.0, 3.0, (0.0, 0.0), 400.0, "tempered", Making::Forge),
    m("gold", "material.base.gold", &[F], 19.3, 1.0, 0, 30.0, 6.0, 0.0, (0.0, 0.0), 40.0, "", Making::Cast),
    // Wood, for hafts, shafts and grips (and wooden heads: a club, a digging stick).
    m("oak", "material.base.oak", &[W, H], 0.75, 1.6, 0, 60.0, 2.0, 0.0, (0.10, -0.03), 0.0, "tough", Making::Carve),
    m("mesquite", "material.base.mesquite", &[W, H], 0.88, 1.6, 0, 70.0, 2.0, 0.0, (0.15, -0.05), 0.0, "tough", Making::Carve),
    m("birch", "material.base.birch", &[W, H], 0.65, 1.6, 0, 55.0, 2.0, 0.0, (0.0, 0.05), 0.0, "light", Making::Carve),
    // Bow woods (Content: yew and elm in the forest, Osage orange on the plains), provisional until
    // their trees grow in the world.
    m("yew", "material.base.yew", &[W, H], 0.67, 1.6, 0, 60.0, 2.0, 0.0, (0.05, 0.0), 0.0, "", Making::Carve),
    m("elm", "material.base.elm", &[W, H], 0.56, 1.6, 0, 55.0, 2.0, 0.0, (0.05, 0.0), 0.0, "", Making::Carve),
    m("osage_orange", "material.base.osage_orange", &[W, H], 0.86, 1.6, 0, 65.0, 2.0, 0.0, (0.08, 0.0), 0.0, "", Making::Carve),
    m("spruce", "material.base.spruce", &[W, H], 0.45, 1.6, 0, 45.0, 2.0, 0.0, (-0.05, 0.05), 0.0, "light", Making::Carve),
    // A stick shaved for a haft (`vitals::first`): the first stone axe's, before split hafts.
    // Green or dead wood off the ground, weaker than a haft split from a log.
    m("branch", "material.base.branch", &[H], 0.6, 1.6, 0, 40.0, 2.0, 0.0, (-0.10, 0.0), 0.0, "", Making::Carve),
    // Fittings: what holds a head on.
    m("plant_cordage", "material.base.plant_cordage", &[F], 1.3, 1.0, 0, 0.0, 0.0, 0.0, (0.0, 0.0), 30.0, "cheap", Making::Hands),
    m("sinew", "material.base.sinew", &[F], 1.3, 1.0, 0, 0.0, 0.0, 0.0, (0.0, 0.0), 60.0, "shrinks_tight", Making::Hands),
    m("rawhide", "material.base.rawhide", &[F, H], 1.2, 1.0, 0, 0.0, 0.0, 0.0, (0.0, 0.0), 80.0, "shrinks_tight", Making::Hands),
    m("birch_tar", "material.base.birch_tar", &[F], 1.0, 1.0, 0, 0.0, 0.0, 0.0, (0.0, 0.0), 70.0, "glued", Making::Hands),
    m("leather", "material.base.leather", &[F, H], 0.9, 1.0, 0, 0.0, 0.0, 0.0, (0.02, 0.0), 90.0, "", Making::Sew),
];

/// Rows kept only until Content plan's flint, chert and obsidian replace them: today's plain
/// stone tools. Diamond is a real tier above steel (Ronald, 2026-11-04: "we will probably have
/// more ores / items with higher tiers"), and the ladder stays open at the top: a new tier is a
/// row (a package's too), and grades and stats scale past steel by the same rule.
pub const PROVISIONAL: &[&str] = &["stone", "yew", "elm", "osage_orange"];

/// How well a wood stores a bow's draw (yew 1): a stave's power. Proposed from bowyers' rankings;
/// what isn't listed makes no bow.
pub const SPRING: &[(&str, f32)] =
    &[("yew", 1.0), ("osage_orange", 1.0), ("elm", 0.85), ("oak", 0.7), ("mesquite", 0.65), ("birch", 0.6), ("spruce", 0.5)];

/// A material's `SPRING`, 0 if it makes no bow.
pub fn spring(id: &str) -> f32 {
    SPRING.iter().find(|(m, _)| *m == id).map_or(0.0, |s| s.1)
}

/// Material `id`.
pub fn material(id: &str) -> Option<&'static Material> {
    MATERIALS.iter().find(|m| m.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_traits_exist() {
        for (i, a) in MATERIALS.iter().enumerate() {
            assert!(MATERIALS[i + 1..].iter().all(|b| b.id != a.id), "{} twice", a.id);
            assert!(a.trait_id.is_empty() || crate::traits::trait_of(a.trait_id).is_some(), "{}: no trait {}", a.id, a.trait_id);
            assert!(a.density > 0.0 && a.bulk >= 1.0);
        }
    }

    #[test]
    fn the_ladder_is_in_order() {
        // Content's check: speed and uses rise tier by tier; bronze near iron; steel the best.
        let ladder = ["flint", "copper", "bronze", "wrought_iron", "steel"].map(|id| material(id).unwrap());
        for w in ladder.windows(2) {
            assert!(w[1].harvest >= w[0].harvest && w[1].speed >= w[0].speed && w[1].uses >= w[0].uses, "{} > {}", w[1].id, w[0].id);
        }
        let (bronze, iron) = (material("bronze").unwrap(), material("wrought_iron").unwrap());
        assert!((bronze.uses / iron.uses - 1.0).abs() < 0.15, "bronze is near iron");
        assert!(
            MATERIALS
                .iter()
                // Steel is the best of the metals; diamond and what comes later sit above it.
                .filter(|m| m.harvest <= material("steel").unwrap().harvest)
                .all(|m| m.uses <= material("steel").unwrap().uses)
        );
    }

    #[test]
    fn every_role_has_materials() {
        for role in [Role::Working, Role::Holding, Role::Fitting] {
            assert!(MATERIALS.iter().any(|m| m.roles.contains(&role)), "{role:?}");
        }
    }
}

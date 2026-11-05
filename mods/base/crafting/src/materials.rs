//! What parts are made of (`notes/CRAFTING.md`, "Materials and traits"; the ladder is Content
//! plan's, `notes/CONTENT.md`). Every row is a real material with its real density and what it
//! can really be (`Works`, `fits`). A working part's harvest level, uses and speed come from its
//! material's item level on the curve (`vitals::gear`), with hand tweaks of ×0.75-1.25 that say
//! why.

use crate::kinds::Role;
use vitals::curve::Tweak;

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

/// What a material can be as an item's working part (Ronald, 11-05: "only, let's say, the blade
/// can be of diamond and the hilt, but not like the handle out of diamond ... a bit more realistic,
/// but not completely"). Each class takes the kinds real ones were made of it (`works`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Works {
    /// Knapped stone and glass (flint, chert, obsidian, tektite): every edge and head; a sword
    /// blade as a macuahuitl's (flint set in a wooden blade), not armour.
    Edge,
    /// Ground stone: axe, pick, shovel and hoe heads, no edge to cut with.
    Ground,
    /// Bone and antler: antler picks, scapula shovels, bone hoes, knives and points, bone-slat
    /// armour; no sword.
    Bone,
    /// Wood: a digging stick's or wooden spade's blade, a hoe, a fire-hardened spear point, a bow
    /// stave; no axe, pick, sword or armour.
    Wood,
    /// Copper, bronze, iron, steel: everything, armour too (scales, mail, plate).
    Metal,
    /// Diamond: every edge and head, and armour of scales sewn on a backing (the one fantasy kept:
    /// "not completely" realistic).
    Gem,
    /// Leather and rawhide: armour shells only (cuir bouilli, rawhide lamellar).
    Skin,
    /// Never a working part: gold, cordage, sinew, tar, cloth.
    None,
}

/// The armour kinds (`kinds::KINDS`), head to feet.
pub const ARMOUR: [&str; 4] = ["helmet", "chestplate", "leggings", "boots"];

impl Works {
    /// Whether a working part of this class can be `kind`'s.
    pub fn makes(self, kind: &str) -> bool {
        let armour = ARMOUR.contains(&kind);
        match self {
            Works::Edge => !armour && kind != "bow",
            Works::Ground => matches!(kind, "axe" | "pickaxe" | "shovel" | "hoe"),
            Works::Bone => !matches!(kind, "sword" | "bow"),
            Works::Wood => matches!(kind, "shovel" | "hoe" | "spear" | "bow"),
            Works::Metal | Works::Gem => kind != "bow",
            Works::Skin => armour,
            Works::None => false,
        }
    }
}

/// One material.
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub id: &'static str,
    /// Its name's key.
    pub key: &'static str,
    /// What it can be, by role.
    pub roles: &'static [Role],
    /// What its working part can be (`Works`).
    pub works: Works,
    /// g/cm³, for a part's mass.
    pub density: f32,
    /// How much bigger than a metal part of the same shape a part of it is (stone is thick, so it
    /// doesn't snap; Content, A3).
    pub bulk: f32,
    /// Hand tweaks on the curve's uses and speed (`vitals::gear::tool_uses`, `mining_speed`):
    /// ×0.75-1.25, each with its reason.
    pub tweak_uses: Tweak,
    pub tweak_speed: Tweak,
    /// As a holding part: what it adds to the working part's uses and speed (0.10 = +10%).
    pub hold_uses: f32,
    pub hold_speed: f32,
    /// As a fitting: uses before it gives (lashings go first, rivets last).
    pub fitting_uses: f32,
    /// Its trait (`traits::TRAITS`), "" for none.
    pub trait_id: &'static str,
    pub making: Making,
}

impl Material {
    /// Its item level (`vitals::gear::MATERIALS`: its age's tier × 10 plus a depth).
    pub fn level(&self) -> f32 {
        vitals::gear::material(self.id).map_or(0.0, |m| m.level())
    }

    /// What a working part of it harvests (`abi::tool_tier`), from its level; nothing for what
    /// is never a working part.
    pub fn harvest(&self) -> u32 {
        if self.works == Works::None { 0 } else { vitals::gear::harvest(self.level()) }
    }

    /// Uses as a working part at Sound (grade 4), before shape and haft: the curve's, tweaked.
    pub fn uses(&self) -> f32 {
        vitals::gear::tool_uses(self.level()) * self.tweak_uses.x
    }

    /// Speed as a working part: the curve's mining speed, tweaked.
    pub fn speed(&self) -> f32 {
        vitals::gear::mining_speed(self.level()) * self.tweak_speed.x
    }
}

use Role::{Extra as X, Fitting as F, Holding as H, Lining as L, Working as W};

const N: Tweak = Tweak::NONE;
const fn tw(x: f32, why: &'static str) -> Tweak {
    Tweak::new(x, why)
}

// A table row: one argument a column reads better here than a builder.
#[allow(clippy::too_many_arguments)]
const fn m(
    id: &'static str,
    key: &'static str,
    roles: &'static [Role],
    works: Works,
    density: f32,
    bulk: f32,
    tweak: (Tweak, Tweak),
    hold: (f32, f32),
    fitting_uses: f32,
    trait_id: &'static str,
    making: Making,
) -> Material {
    Material {
        id,
        key,
        roles,
        works,
        density,
        bulk,
        tweak_uses: tweak.0,
        tweak_speed: tweak.1,
        hold_uses: hold.0,
        hold_speed: hold.1,
        fitting_uses,
        trait_id,
        making,
    }
}

/// Densities: flint and chert 2.6, obsidian 2.4, bone and antler 1.9, copper 8.96, bronze 8.8,
/// wrought iron 7.7, meteoric iron 7.9 (iron with 5-12 % nickel), steel 7.85, gold 19.3; seasoned
/// oak 0.75, birch 0.65, spruce 0.45, mesquite 0.88; felted wool 0.3, linen 0.5, fur 0.4. A working
/// part's harvest, uses and speed come from its level on the curve (`vitals::gear`); the roles are
/// checked against the real things (`notes/CRAFTING.md`, "What each material can be").
pub const MATERIALS: &[Material] = &[
    // Stone and bone (the Stone Age).
    m("flint", "material.base.flint", &[W], Works::Edge, 2.6, 1.8, (N, N), (0.0, 0.0), 0.0, "keen", Making::Knap),
    m(
        "chert",
        "material.base.chert",
        &[W],
        Works::Edge,
        2.6,
        1.8,
        (tw(0.93, "grainier than flint, a little quicker to dull"), N),
        (0.0, 0.0),
        0.0,
        "common",
        Making::Knap,
    ),
    // Tektite (big impacts): the ground melted and thrown out, moldavite's green glass; obsidian's
    // edge for a newcomer who reaches a crater.
    m("tektite", "material.base.tektite", &[W], Works::Edge, 2.4, 1.8, (N, N), (0.0, 0.0), 0.0, "razor", Making::Knap),
    m("obsidian", "material.base.obsidian", &[W], Works::Edge, 2.4, 1.8, (N, N), (0.0, 0.0), 0.0, "razor", Making::Knap),
    m(
        "bone",
        "material.base.bone",
        &[W, H, X],
        Works::Bone,
        1.9,
        1.5,
        (tw(1.25, "dense and springy: bone outlasts its age's wood"), N),
        (0.05, 0.0),
        0.0,
        "tough",
        Making::Carve,
    ),
    m(
        "antler",
        "material.base.antler",
        &[W, H, X],
        Works::Bone,
        1.9,
        1.5,
        (tw(1.25, "antler took the blows of flint mining"), N),
        (0.08, 0.0),
        0.0,
        "tough",
        Making::Carve,
    ),
    // Today's plain stone tools (provisional until flint and chert), and diamond, a tier above
    // steel; more above it later.
    m("stone", "material.base.stone", &[W], Works::Ground, 2.7, 1.8, (N, N), (0.0, 0.0), 0.0, "common", Making::Knap),
    m(
        "diamond",
        "material.base.diamond",
        &[W],
        Works::Gem,
        3.5,
        1.0,
        (tw(1.25, "the hardest edge there is hardly wears"), N),
        (0.0, 0.0),
        0.0,
        "",
        Making::Carve,
    ),
    // Metals (the Bronze and Iron Ages): heads and blades, rivets, wedges and collars, guards and
    // pommels, armour.
    m("copper", "material.base.copper", &[W, F, X], Works::Metal, 8.96, 1.0, (N, N), (0.0, 0.0), 120.0, "soft", Making::Forge),
    m(
        "bronze",
        "material.base.bronze",
        &[W, F, X],
        Works::Metal,
        8.8,
        1.0,
        (tw(1.25, "work-hardened tin bronze wears about as well as wrought iron (Content)"), N),
        (0.0, 0.0),
        220.0,
        "recastable",
        Making::Cast,
    ),
    m(
        "wrought_iron",
        "material.base.wrought_iron",
        &[W, F, X],
        Works::Metal,
        7.7,
        1.0,
        (tw(0.85, "bloomery iron is soft and slaggy; steel is what holds an edge"), N),
        (0.0, 0.0),
        260.0,
        "weldable",
        Making::Forge,
    ),
    // Meteoric iron (sky events, Research round 17's addendum): iron-nickel from fallen stars,
    // hammered cold as the Inuit worked Cape York's, so iron-grade heads before a player can smelt;
    // a lucky early step between bronze and wrought iron, found a few lumps at a time.
    m("meteoric_iron", "material.base.meteoric_iron", &[W, F, X], Works::Metal, 7.9, 1.0, (N, N), (0.0, 0.0), 220.0, "", Making::Forge),
    m("steel", "material.base.steel", &[W, F, X], Works::Metal, 7.85, 1.0, (N, N), (0.0, 0.0), 400.0, "tempered", Making::Forge),
    // Star steel (big impacts, Research round 17's Addendum 2): the impact's nickel-iron core, its
    // etched plates the Widmanstätten pattern; a tier between steel and diamond, worked at the forge.
    m("star_steel", "material.base.star_steel", &[W, F, X], Works::Metal, 7.9, 1.0, (N, N), (0.0, 0.0), 500.0, "patterned", Making::Forge),
    // Gold only as a fitting or an ornament (Ronald, 11-05): soft, heavy, beautiful.
    m("gold", "material.base.gold", &[F, X], Works::None, 19.3, 1.0, (N, N), (0.0, 0.0), 40.0, "", Making::Cast),
    // Wood, for hafts, shafts and grips (and wooden heads: a digging stick, a wooden spade).
    m("oak", "material.base.oak", &[W, H, X], Works::Wood, 0.75, 1.6, (N, N), (0.10, -0.03), 0.0, "tough", Making::Carve),
    m(
        "mesquite",
        "material.base.mesquite",
        &[W, H, X],
        Works::Wood,
        0.88,
        1.6,
        (tw(1.2, "dense desert hardwood"), N),
        (0.15, -0.05),
        0.0,
        "tough",
        Making::Carve,
    ),
    m("birch", "material.base.birch", &[W, H, X], Works::Wood, 0.65, 1.6, (N, N), (0.0, 0.05), 0.0, "light", Making::Carve),
    // Bow woods (Content: yew and elm in the forest, Osage orange on the plains), provisional until
    // their trees grow in the world.
    m("yew", "material.base.yew", &[W, H, X], Works::Wood, 0.67, 1.6, (N, N), (0.05, 0.0), 0.0, "", Making::Carve),
    m("elm", "material.base.elm", &[W, H, X], Works::Wood, 0.56, 1.6, (N, N), (0.05, 0.0), 0.0, "", Making::Carve),
    m(
        "osage_orange",
        "material.base.osage_orange",
        &[W, H, X],
        Works::Wood,
        0.86,
        1.6,
        (tw(1.1, "the hardest of the bow woods"), N),
        (0.08, 0.0),
        0.0,
        "",
        Making::Carve,
    ),
    m(
        "spruce",
        "material.base.spruce",
        &[W, H, X],
        Works::Wood,
        0.45,
        1.6,
        (tw(0.8, "soft, light softwood"), N),
        (-0.05, 0.05),
        0.0,
        "light",
        Making::Carve,
    ),
    // A stick shaved for a haft (`vitals::first`): the first stone axe's, before split hafts.
    // Green or dead wood off the ground, weaker than a haft split from a log.
    m(
        "branch",
        "material.base.branch",
        &[H],
        Works::None,
        0.6,
        1.6,
        (tw(0.75, "green or dead wood off the ground"), N),
        (-0.10, 0.0),
        0.0,
        "",
        Making::Carve,
    ),
    // Fittings: what holds a head on.
    // Cordage lashing 50 (was 30): a hand-lashed first axe (Plain, x0.9: 45) fells a 60-log oak
    // (21) and cuts its pieces (15) before it gives (PC run fh_natural4 broke at the fell); sinew
    // and rawhide still last longer.
    m("plant_cordage", "material.base.plant_cordage", &[F], Works::None, 1.3, 1.0, (N, N), (0.0, 0.0), 50.0, "cheap", Making::Hands),
    m("sinew", "material.base.sinew", &[F], Works::None, 1.3, 1.0, (N, N), (0.0, 0.0), 60.0, "shrinks_tight", Making::Hands),
    m("rawhide", "material.base.rawhide", &[F, H, W], Works::Skin, 1.2, 1.0, (N, N), (0.0, 0.0), 80.0, "shrinks_tight", Making::Hands),
    m("birch_tar", "material.base.birch_tar", &[F], Works::None, 1.0, 1.0, (N, N), (0.0, 0.0), 70.0, "glued", Making::Hands),
    // Sole leather: grip wraps, lacing and straps, armour of hardened leather, a hide lining.
    m("leather", "material.base.leather", &[F, H, W, L], Works::Skin, 0.9, 1.0, (N, N), (0.02, 0.0), 90.0, "", Making::Sew),
    // Linings (`vitals::clothing`'s materials): what pads armour and keeps its wearer warm.
    m("linen", "material.base.linen", &[L], Works::None, 0.5, 1.0, (N, N), (0.0, 0.0), 0.0, "", Making::Sew),
    m("wool", "material.base.wool", &[L], Works::None, 0.3, 1.0, (N, N), (0.0, 0.0), 0.0, "", Making::Sew),
    m("fur", "material.base.fur", &[L], Works::None, 0.4, 1.0, (N, N), (0.0, 0.0), 0.0, "", Making::Sew),
];

/// Whether a part of `m` can be `kind`'s slot `slot`, which has role `role` (`notes/CRAFTING.md`,
/// "What each material can be"): its roles, its working class for a working part (and a spring
/// for a bow's stave), a wrap only as a grip, and no glue holding armour.
pub fn fits(m: &Material, kind: &str, slot: &str, role: Role) -> bool {
    if !m.roles.contains(&role) {
        return false;
    }
    match role {
        Role::Working => m.works.makes(kind) && (kind != "bow" || spring(m.id) > 0.0),
        Role::Holding => m.works != Works::Skin || slot == "grip",
        // Tar glues a head on; it holds no armour together.
        Role::Fitting => !(ARMOUR.contains(&kind) && m.id == "birch_tar"),
        Role::Extra | Role::Lining => true,
    }
}

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
            assert!(a.tweak_uses.ok() && a.tweak_speed.ok(), "{}: a tweak out of range or without a reason", a.id);
            assert!(vitals::gear::material(a.id).is_some(), "{}: no level in vitals::gear", a.id);
            assert_eq!(a.works != Works::None, a.roles.contains(&Role::Working), "{}: works and roles agree", a.id);
        }
    }

    #[test]
    fn the_ladder_is_in_order() {
        // Content's check: speed and uses rise tier by tier; bronze near iron; steel the best.
        let ladder = ["flint", "copper", "bronze", "wrought_iron", "steel"].map(|id| material(id).unwrap());
        for w in ladder.windows(2) {
            assert!(
                w[1].harvest() >= w[0].harvest() && w[1].speed() >= w[0].speed() && w[1].uses() >= w[0].uses(),
                "{} > {}",
                w[1].id,
                w[0].id
            );
        }
        let (bronze, iron) = (material("bronze").unwrap(), material("wrought_iron").unwrap());
        assert!((bronze.uses() / iron.uses() - 1.0).abs() < 0.15, "bronze is near iron");
        let steel = material("steel").unwrap();
        assert!(
            MATERIALS
                .iter()
                // Steel is the best of the metals; diamond and what comes later sit above it.
                .filter(|m| m.works != Works::None && m.harvest() <= steel.harvest())
                .all(|m| m.uses() <= steel.uses()),
        );
    }

    #[test]
    fn every_role_has_materials() {
        for role in [Role::Working, Role::Holding, Role::Fitting, Role::Extra, Role::Lining] {
            assert!(MATERIALS.iter().any(|m| m.roles.contains(&role)), "{role:?}");
        }
    }

    #[test]
    fn real_things_only() {
        // Ronald, 11-05: a diamond blade yes, a diamond handle no; gold only a fitting or inlay.
        let d = material("diamond").unwrap();
        assert!(fits(d, "sword", "blade", Role::Working) && fits(d, "chestplate", "shell", Role::Working));
        assert!(
            !fits(d, "sword", "grip", Role::Holding)
                && !fits(d, "axe", "fitting", Role::Fitting)
                && !fits(d, "sword", "pommel", Role::Extra)
        );
        let gold = material("gold").unwrap();
        assert!(fits(gold, "sword", "pommel", Role::Extra) && fits(gold, "axe", "fitting", Role::Fitting));
        assert!(!fits(gold, "axe", "head", Role::Working) && !fits(gold, "chestplate", "shell", Role::Working));
        // No wooden axes or picks, no stone sword blades, no leather hafts.
        let oak = material("oak").unwrap();
        assert!(!fits(oak, "pickaxe", "head", Role::Working) && fits(oak, "shovel", "head", Role::Working));
        assert!(!fits(material("stone").unwrap(), "sword", "blade", Role::Working));
        let leather = material("leather").unwrap();
        assert!(fits(leather, "sword", "grip", Role::Holding) && !fits(leather, "axe", "haft", Role::Holding));
        assert!(fits(leather, "chestplate", "shell", Role::Working) && !fits(leather, "axe", "head", Role::Working));
        // A bow's stave is a springy wood.
        assert!(
            fits(material("yew").unwrap(), "bow", "stave", Role::Working)
                && !fits(material("steel").unwrap(), "bow", "stave", Role::Working)
        );
    }
}

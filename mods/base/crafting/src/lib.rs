//! Items built from parts (`notes/CRAFTING.md`), the rules that need no game world: the
//! materials and what they're like, the kinds of tool and their slots, the part shapes, the ten
//! grades and how one is rolled, the traits, and how an item's numbers come from its parts. The
//! base mod turns them into items, screens and wear; checks and the sheet read them here.
//!
//! The rule that ties it together: **the working part sets the numbers, the holding parts
//! multiply them, the fitting adds a trait and its own wear.** Nothing hidden: every number in an
//! item's tooltip is one of these steps.

pub mod combine;
pub mod grades;
pub mod kinds;
pub mod looks;
pub mod materials;
pub mod sheet;
pub mod skills;
pub mod traits;

pub use combine::{Part, ToolStats, combine};
pub use grades::{GRADES, Grade, Points};
pub use kinds::{KINDS, Kind, Role, SHAPES, Shape};
pub use materials::{MATERIALS, Material};
pub use traits::{TRAITS, Trait};

/// A stable number for an id, so an item carries its parts as numbers that survive new rows and
/// reordering (FNV-1a).
pub const fn code(id: &str) -> u32 {
    let b = id.as_bytes();
    let mut h: u32 = 0x811c_9dc5;
    let mut i = 0;
    while i < b.len() {
        h ^= b[i] as u32;
        h = h.wrapping_mul(0x0100_0193);
        i += 1;
    }
    h
}

/// The material, kind or shape whose id has `code`.
pub fn material_of(code: u32) -> Option<&'static Material> {
    MATERIALS.iter().find(|m| crate::code(m.id) == code)
}

pub fn kind_of(code: u32) -> Option<&'static Kind> {
    KINDS.iter().find(|k| crate::code(k.id) == code)
}

pub fn shape_of(kind: &str, code: u32) -> Option<&'static Shape> {
    SHAPES
        .iter()
        .find(|s| s.kind == kind && crate::code(s.id) == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique() {
        let mut seen: Vec<(u32, &str)> = Vec::new();
        let ids = MATERIALS
            .iter()
            .map(|m| m.id)
            .chain(KINDS.iter().map(|k| k.id))
            .chain(SHAPES.iter().map(|s| s.id));
        for id in ids {
            let c = code(id);
            assert!(seen.iter().all(|(x, other)| *x != c || *other == id), "{id} and another share a code");
            seen.push((c, id));
        }
        assert_eq!(material_of(code("flint")).unwrap().id, "flint");
    }
}

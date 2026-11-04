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
pub mod materials;
pub mod sheet;
pub mod traits;

pub use combine::{Part, ToolStats, combine};
pub use grades::{GRADES, Grade, Points};
pub use kinds::{KINDS, Kind, Role, SHAPES, Shape};
pub use materials::{MATERIALS, Material};
pub use traits::{TRAITS, Trait};

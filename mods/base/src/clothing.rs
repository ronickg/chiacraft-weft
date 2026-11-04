//! Clothes (`vitals::clothing`, `notes/SURVIVAL.md`): a hat, a coat, trousers and boots of fur,
//! wool, hide or linen, worn in four slots of their own under armor (`player.rs`), warm and no
//! armor. Body temperature reads them (`survival/temperature.rs`). What they're made of: fur from
//! furred wild animals (the animals mod's loot, `animals_core::hides`), sheep's wool, leather, and
//! linen woven from string, which plant fibre from cutting tall grass spins into.

use base_api::{CLOTHES, WARMTH, Warmth};
use vitals::clothing::{self, MATERIALS, PIECES};
use vitals::temperature::PIECE_SHARE;
use weft_sdk::*;

pub use base_api::CRAFTING;

/// A pelt, from furred wild animals.
pub static FUR: Item = Item::named("fur");
/// From cutting tall grass, spun into string.
pub static PLANT_FIBRE: Item = Item::named("plant_fibre");
/// Linen woven from string.
pub static CLOTH: Item = Item::named("cloth");

/// What each material is sewn from, as a recipe ingredient.
fn sewn_from(material: &str) -> &'static str {
    match material {
        "fur" => "#pelts",
        "wool" => "wool",
        "hide" => "#materials/leather",
        _ => "#cloth",
    }
}

/// How far out from the body each piece is drawn, under armor's (1, leggings 0.5).
const INFLATE: [f32; 4] = [0.45, 0.3, 0.25, 0.35];

pub fn register(r: &mut Registry) {
    r.item(&FUR).tag("pelts").tab(&crate::tabs::INGREDIENTS);
    r.item(&PLANT_FIBRE).tag("plant_fibre").tab(&crate::tabs::INGREDIENTS);
    r.item(&CLOTH).tag("cloth").set(&base_api::FLAMMABLE, 30).tab(&crate::tabs::INGREDIENTS);
    // Fibre spun into string (three to one), string woven into linen (three to a cloth).
    r.shapeless(&CRAFTING, "string", 1, &["#plant_fibre", "#plant_fibre", "#plant_fibre"], 0);
    r.shaped(&CRAFTING, "cloth", 1, &["sss"], &[('s', "#string")]);

    for m in &MATERIALS {
        for (piece, name) in PIECES.iter().enumerate() {
            let share = PIECE_SHARE[piece];
            let warmth = Warmth {
                cold: (m.cold() * share).round() as u32,
                heat: (m.heat * share).round() as i32,
                wet: (m.wet_keeps * 100.0).round() as u32,
                wind: (m.windproof * 100.0).round() as u32,
            };
            // Trousers are the inner layer of the skin's layout, like leggings.
            let layer = if piece == 2 { 2 } else { 1 };
            let item = format!("{}_{name}", m.name);
            r.item(Item::generated(item.clone()))
                .max_stack(1)
                .set(&WARMTH, warmth)
                .tag("clothes")
                .tag(&format!("clothes/{}", m.name))
                .wearable(CLOTHES[piece], INFLATE[piece], &format!("clothes/{}_{layer}", m.name))
                .tab(&crate::tabs::COMBAT);
            let (pieces, string) = clothing::MAKES[piece];
            let mut parts = vec![sewn_from(m.name); pieces as usize];
            parts.extend(std::iter::repeat_n("#string", string as usize));
            r.shapeless(&CRAFTING, &item, 1, &parts, 0);
        }
    }
}

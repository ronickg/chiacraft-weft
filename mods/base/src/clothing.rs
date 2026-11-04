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

/// A piece's shape in the grid by how much material it takes (`X`, with `vitals::clothing::STRING`
/// of string `s`): each piece of a material its own, so a hat and boots of the same stuff differ.
fn pattern(piece: usize, material: u32) -> Option<&'static [&'static str]> {
    Some(match (piece, material) {
        (0, 1) => &["X", "s"],
        (0, 2) => &["XsX"],
        (1, 2) => &["XsX", " s "],
        (1, 4) => &["XsX", "XsX"],
        (2, 1) => &["X ", " s"],
        (2, 3) => &["XXX", " s "],
        (3, 1) => &["s", "X"],
        (3, 2) => &["X X", " s "],
        _ => return None,
    })
}

/// How far out from the body each piece is drawn, under armor's (1, leggings 0.5).
const INFLATE: [f32; 4] = [0.45, 0.3, 0.25, 0.35];

/// Whether a player has had their arrival clothes (`vitals::clothing::ARRIVAL`), saved and kept
/// through death.
static DRESSED: Component<u8> = Component::named("base:dressed");

pub fn register(r: &mut Registry) {
    r.entity_component(&DRESSED, 0, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &DRESSED);
    // A new player arrives in plain clothes, once (Research round 3, B1).
    r.on(&PLAYER_JOINED, |b| {
        let w = b.world();
        for p in b.iter() {
            let Some(d) = w.component_mut(p.player, &DRESSED) else { continue };
            if *d != 0 {
                continue;
            }
            *d = 1;
            let Some(slots) = w.inventory(p.player).find_group(EQUIPMENT_GROUP) else { continue };
            for (material, piece) in clothing::ARRIVAL {
                let at = (4 + piece) as u32;
                let Some(item) = Item::lookup(&format!("base:{material}_{}", PIECES[piece])) else { continue };
                if at < slots.len() && slots.get(at).is_empty() {
                    slots.set(at, ItemStack::new(item, 1));
                }
            }
        }
    });
    r.item(&FUR).tag("pelts").tab(&crate::tabs::INGREDIENTS);
    r.item(&PLANT_FIBRE).tag("plant_fibre").tab(&crate::tabs::INGREDIENTS);
    r.item(&CLOTH).tag("cloth").set(&base_api::FLAMMABLE, 30).tab(&crate::tabs::INGREDIENTS);
    // Fibre spun into string, string woven into linen (three to a cloth).
    r.shapeless(&CRAFTING, "string", 1, &vec!["#plant_fibre"; clothing::FIBRE_PER_STRING as usize], 0);
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
            let keys = [('X', sewn_from(m.name)), ('s', "#string")];
            match pattern(piece, m.makes[piece]) {
                Some(rows) => r.shaped(&CRAFTING, &item, 1, rows, &keys),
                None => {
                    let mut parts = vec![sewn_from(m.name); m.makes[piece] as usize];
                    parts.extend(std::iter::repeat_n("#string", clothing::STRING[piece] as usize));
                    r.shapeless(&CRAFTING, &item, 1, &parts, 0)
                }
            };
        }
    }
}

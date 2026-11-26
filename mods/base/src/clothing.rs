//! Clothes (`vitals::clothing`, `notes/SURVIVAL.md`): a hat, a coat, trousers and boots of fur,
//! wool, hide or linen, worn in four slots of their own under armor (`player.rs`), warm and no
//! armor. Body temperature reads them (`survival/temperature.rs`). What they're made of: fur from
//! furred wild animals (the animals mod's loot, `animals_core::hides`), sheep's wool, leather, and
//! linen woven from string, which plant fibre from cutting tall grass spins into.

use base_api::{CLOTHES, WARMTH, Warmth};
use vitals::clothing::{self, MATERIALS, PIECES};
use vitals::temperature::PIECE_SHARE;
use weft_sdk::*;

pub use base_api::recipes::CRAFTING;

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
        "hide" => "#leather",
        _ => "#cloth",
    }
}

/// Clothes and the cloak a player wore when they died (item ids, the four clothing slots then the
/// cloak), put back on at the respawn.
static KEPT: Component<[u32; 5]> = Component::named("base:kept_clothes");

/// Takes the clothes and the cloak off a dying player and keeps them for the respawn, before the
/// tombstone takes the rest and the engine drops what's left (`levels.rs`).
pub fn set_aside(w: &World, player: u64) {
    let Some(slots) = w.inventory(player).find_group(EQUIPMENT_GROUP) else { return };
    let mut kept = [0u32; 5];
    for (i, k) in kept.iter_mut().enumerate() {
        let at = 4 + i as u32;
        if at < slots.len() {
            *k = slots.get(at).item;
            slots.set(at, ItemStack::EMPTY);
        }
    }
    if let Some(c) = w.component_mut(player, &KEPT) {
        *c = kept;
    }
}

/// Puts the arrival clothes (`vitals::clothing::ARRIVAL`) on `player` if no clothes are worn.
fn dress(w: &World, player: u64) {
    let Some(slots) = w.inventory(player).find_group(EQUIPMENT_GROUP) else { return };
    if (4..slots.len().min(8)).any(|i| !slots.get(i).is_empty()) {
        return;
    }
    for (material, piece) in clothing::ARRIVAL {
        let at = (4 + piece) as u32;
        let Some(item) = Item::lookup(&format!("base:{material}_{}", PIECES[piece])) else { continue };
        if at < slots.len() {
            slots.set(at, ItemStack::new(item, 1));
        }
    }
}

/// What each cloak is sewn from, its shape (`X` the material, `s` string), and how far out it's drawn
/// (over armor's 1).
fn cloak_recipe(name: &str) -> (&'static str, &'static [&'static str]) {
    match name {
        "wool_cloak" => ("wool", &["XsX", " X "]),
        "fur_robe" => ("#pelts", &["XXX", "XsX"]),
        _ => ("#leather", &["XsX", "XXX"]),
    }
}
const CLOAK_INFLATE: f32 = 1.2;

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
    r.entity_component(&DRESSED, 0, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&DRESSED);
    r.entity_component(&base_api::MILESTONES, 0, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&base_api::MILESTONES);
    r.entity_component(
        &base_api::GRAVE,
        base_api::Grave { x: 0, y: 0, z: 0, set: 0 },
        component_flags::SAVE | component_flags::KEEP_ON_RESPAWN | component_flags::SYNC_OWNER,
    );
    r.edit(&PLAYER).attach(&base_api::GRAVE);
    // A new player arrives in plain clothes, once (Research round 3, B1).
    r.on_event(&PLAYER_JOINED, |b| {
        let w = b.world();
        for p in b.iter() {
            let Some(d) = w.component_mut(p.player, &DRESSED) else { continue };
            if *d != 0 {
                continue;
            }
            *d = 1;
            dress(&w, p.player);
        }
    });
    r.entity_component(&KEPT, [0; 5], component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&KEPT);
    // Back from the dead in what one wore: the tombstone leaves clothes and the cloak out (`set_aside`),
    // so a death costs no clothes and gives none (integration review: each death was a free set);
    // with nothing worn, the plain arrival set (Research round 4, H1).
    r.on_event(&PLAYER_RESPAWNED, |b| {
        let w = b.world();
        for p in b.iter() {
            if w.outside_survival(p.player) {
                continue;
            }
            let kept = w.component(p.player, &KEPT).unwrap_or([0; 5]);
            if let Some(slots) = w.inventory(p.player).find_group(EQUIPMENT_GROUP) {
                for (i, &item) in kept.iter().enumerate() {
                    let at = 4 + i as u32;
                    if item != 0 && at < slots.len() && slots.get(at).is_empty() {
                        slots.set(at, ItemStack::new(item, 1));
                    }
                }
            }
            if let Some(k) = w.component_mut(p.player, &KEPT) {
                *k = [0; 5];
            }
            dress(&w, p.player);
        }
    });
    r.item(&FUR).tag("pelts").tab(&crate::tabs::INGREDIENTS);
    r.item(&PLANT_FIBRE)
        .tag("plant_fibre")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&CLOTH)
        .tag("cloth")
        .set(&base_api::components::FLAMMABLE, 30)
        .tab(&crate::tabs::INGREDIENTS);
    // Fibre spun into string, string woven into linen (three to a cloth).
    r.shapeless(&CRAFTING, "string", 1, &vec!["#plant_fibre"; clothing::FIBRE_PER_STRING as usize], 0);
    r.shaped(&CRAFTING, "cloth", 1, &["sss"], &[('s', "#string")]);

    for c in &clothing::CLOAKS {
        let warmth = Warmth {
            cold: c.cold.round() as u32,
            heat: c.heat.round() as i32,
            wet: (c.wet_keeps * 100.0).round() as u32,
            wind: (c.windproof * clothing::CLOAK_COVERS * 100.0).round() as u32,
        };
        r.item(Item::generated(c.name.into()))
            .max_stack(1)
            .set(&WARMTH, warmth)
            .tag("clothes")
            .tag("cloaks")
            .wearable(&base_api::CLOAK, CLOAK_INFLATE, &format!("clothes/{}", c.name))
            .tab(&crate::tabs::COMBAT);
        let (from, rows) = cloak_recipe(c.name);
        r.shaped(&CRAFTING, c.name, 1, rows, &[('X', from), ('s', "#string")]);
    }
    for m in &MATERIALS {
        for (piece, name) in PIECES.iter().enumerate() {
            let share = PIECE_SHARE[piece];
            let warmth = Warmth {
                cold: (m.cold() * share).round() as u32,
                heat: (m.heat * share).round() as i32,
                wet: (m.wet_keeps * 100.0).round() as u32,
                // The piece's share of the wind it stops (a fur hat 12%, not the suit's 80%).
                wind: (m.windproof * share * 100.0).round() as u32,
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

//! Recipes found by touching their materials (`notes/CRAFTING.md`, choice 2A, Valheim's way): a
//! recipe is known once the player has held every one of its ingredients. One they've held some
//! of shows as a "?" with how many ("2 of 3"); one they've held none of doesn't show. Creative
//! knows everything.
//!
//! What a player has held is a set of item names kept as a fixed bitset (`KNOWN`), two bits an
//! item, so it survives new items and a changed load order; a false "held" is rare (a few in a
//! thousand at three hundred items) and only shows a recipe early.

use weft_sdk::*;

/// Bits in the set: 128 words.
const WORDS: usize = 128;
const BITS: u32 = (WORDS * 64) as u32;
/// Ticks between looks at each player's bag (two seconds).
const EVERY: u64 = 40;

value! {
    /// The items a player has held, as a bitset of name hashes (`bits_of`).
    pub struct Known { pub bits: [u64; 128] }
}
pub static KNOWN: Component<Known> = Component::named("base:known_items");

pub fn register(r: &mut Registry) {
    r.entity_component(&KNOWN, Known { bits: [0; WORDS] }, component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP);
    r.attach(&PLAYER, &KNOWN);
    r.world_tick(|w| {
        if w.time() % EVERY != 0 {
            return;
        }
        for player in w.players() {
            let inv = w.inventory(player);
            let held: Vec<u32> = (0..inv.size())
                .map(|i| inv.get(i))
                .filter(|s| !s.is_empty())
                .map(|s| s.item)
                .collect();
            if let Some(k) = w.component_mut(player, &KNOWN) {
                for item in held {
                    for b in bits_of(item_name(item)) {
                        k.bits[(b / 64) as usize] |= 1 << (b % 64);
                    }
                }
            }
        }
    });
}

/// The two bits of an item's name (FNV-1a, split).
fn bits_of(name: &str) -> [u32; 2] {
    let h = crafting::code(name);
    let h2 = h.rotate_left(16).wrapping_mul(0x9E37_79B1);
    [h % BITS, h2 % BITS]
}

/// Whether `known` holds the item named `name`.
pub fn held(known: &Known, name: &str) -> bool {
    bits_of(name)
        .iter()
        .all(|&b| known.bits[(b / 64) as usize] & (1 << (b % 64)) != 0)
}

/// How a recipe stands for a player who has held `items`: (ingredients held, ingredients, an
/// item of the first one not held). Each different ingredient counts once; it's held if any item
/// it takes was.
pub fn standing(r: &Recipe, items: &[u32]) -> (u32, u32, Option<u32>) {
    let inputs = r.inputs();
    let mut seen: Vec<u32> = Vec::new();
    let (mut have, mut all, mut missing) = (0, 0, None);
    for (i, input) in inputs.iter().enumerate() {
        if input.kind == abi::recipe_input_kind::EMPTY || seen.contains(&input.shown) {
            continue;
        }
        seen.push(input.shown);
        all += 1;
        if items.iter().any(|&item| r.accepts(i as u32, item)) {
            have += 1;
        } else if missing.is_none() && input.shown != 0 {
            missing = Some(input.shown);
        }
    }
    (have, all, missing)
}

/// Recipes everyone knows from the start, whatever they've held: what a first night needs.
const STARTERS: &[&str] = &["base:stick", "base:crafting_table", "base:campfire", "base:torch", "base:string", "base:cloth"];

/// Whether `r` is known from the start: planks of any wood, and the `STARTERS`.
pub fn starter(r: &Recipe) -> bool {
    r.outputs()
        .first()
        .is_some_and(|(o, _)| item_has_tag(o.item, "planks") || STARTERS.contains(&item_name(o.item)))
}

/// Every item the local player has held, by id (for the book: built once per screen).
pub fn held_items(view: &ClientView) -> Vec<u32> {
    let Some(k) = view.component(&KNOWN) else { return Vec::new() };
    (1..)
        .map(|i| (i, item_name(i)))
        .take_while(|(_, n)| !n.is_empty())
        .filter(|(_, n)| held(&k, n))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_once_stays_held() {
        let mut k = Known { bits: [0; WORDS] };
        for b in bits_of("base:oak_log") {
            k.bits[(b / 64) as usize] |= 1 << (b % 64);
        }
        assert!(held(&k, "base:oak_log"));
        assert!(!held(&k, "base:iron_ingot"));
    }

    #[test]
    fn few_false_ones() {
        // Three hundred items held, a thousand others asked: almost none read as held.
        let mut k = Known { bits: [0; WORDS] };
        for i in 0..300 {
            for b in bits_of(&format!("held:{i}")) {
                k.bits[(b / 64) as usize] |= 1 << (b % 64);
            }
        }
        let wrong = (0..1000)
            .filter(|i| held(&k, &format!("other:{i}")))
            .count();
        assert!(wrong < 20, "{wrong} of 1000");
    }
}

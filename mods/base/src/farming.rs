//! Farming on fertile soil (`notes/ECOLOGY.md`, parts 8 and 9; the numbers in `vitals::soil`):
//! a hoe tills grass or dirt into farmland; farmland's fertility (its `worn` levels) sets how fast
//! crops grow on it (`growth.rs`); a ripe harvest wears it a level, dung and compost give levels
//! back, and a season unplanted brings one back. A compost bin takes spoiled food, dung and plant
//! waste, and once full rots into compost.

use vitals::soil;
use weft_sdk::*;

use crate::blocks::FARMLAND;
use base_api::{FRESHNESS, freshness};

pub static COMPOST: Item = Item::named("compost");
static GRASS: Block = Block::named("grass");
static DIRT: Block = Block::named("dirt");

pub fn register(r: &mut Registry) {
    // Placeholder icons until the generated ones (`notes/STYLE.md`).
    r.item(Item::generated("dung".into()))
        .on_use(fertilize)
        .tab(&crate::tabs::NATURAL);
    r.item(Item::generated("compost".into()))
        .on_use(fertilize)
        .tab(&crate::tabs::NATURAL);
    r.shaped(&base_api::CRAFTING, "compost_bin", 1, &["# #", "# #", "###"], &[('#', "#planks")]);
    // A ripe harvest wears the farmland under it, or feeds it (beans; `vitals::crops`).
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            let name = block_name(e.state).split('[').next().unwrap_or("");
            let Some(c) = vitals::crops::of(name) else { continue };
            if state_prop(e.state, "age").and_then(|a| a.parse::<u32>().ok()) == Some(c.ripe) {
                let below = [e.pos[0], e.pos[1] - 1, e.pos[2]];
                wear(&w, below, if c.soil == vitals::crops::Soil::Tires { 1 } else { -1 });
            }
        }
    });
}

/// Farmland's fertility at `pos` (`None`: not farmland; `base_api::grow::fertility`).
pub fn fertility(w: &World, pos: [i32; 3]) -> Option<u8> {
    base_api::grow::fertility(w, pos)
}

/// Farmland at `pos` worn by `by` levels (negative: given back).
fn wear(w: &World, pos: [i32; 3], by: i32) {
    let Some(f) = fertility(w, pos) else { return };
    let worn = (soil::MAX as i32 - f as i32 + by).clamp(0, soil::MAX as i32);
    w.set_prop(pos, "worn", &worn.to_string());
}

/// A hoe on the top of grass or dirt with air above: farmland, the hoe worn a little.
pub fn till(u: &mut ItemUse) -> bool {
    let Some((p, face)) = u.target() else { return false };
    let w = u.world();
    let state = w.get(p);
    if face != 3 || !(GRASS.is(state) || DIRT.is(state)) || w.get([p[0], p[1] + 1, p[2]]) != 0 {
        return false;
    }
    w.set(p, FARMLAND.id());
    if !w.creative(u.player()) {
        let s = u.stack();
        let damage = s.damage() + 1;
        let max = DURABILITY.on_item(s.item).unwrap_or(0);
        u.set_stack(if max > 0 && damage >= max { ItemStack::EMPTY } else { s.with_damage(damage) });
    }
    true
}

/// Dung or compost on farmland: its levels back.
fn fertilize(u: &mut ItemUse) -> bool {
    let Some((p, _)) = u.target() else { return false };
    let w = u.world();
    let Some(levels) = soil::feeds(item_name(u.stack().item)) else { return false };
    if fertility(&w, p).is_none_or(|f| f >= soil::MAX) {
        return false;
    }
    wear(&w, p, -(levels as i32));
    if !w.creative(u.player()) {
        u.set_stack(StackData::shrink(u.stack(), 1));
    }
    true
}

/// A growth step of farmland: left unplanted, a level comes back as fast as grass grows there.
pub fn fallow(e: &mut BlockEvent) {
    let w = e.world();
    let p = e.pos();
    if fertility(&w, p).is_none_or(|f| f >= soil::MAX) || w.get([p[0], p[1] + 1, p[2]]) != 0 {
        return;
    }
    crate::growth::grow(e, "base:grazed_grass", false, |e| wear(&e.world(), e.pos(), -1));
}

/// Using a compost bin: a full one gives its compost; else what one holds goes in, if it rots.
pub fn bin(u: &mut Use) -> bool {
    let w = u.world();
    let (p, player) = (u.pos(), u.player());
    let level = state_prop(u.state(), "level")
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);
    if level > soil::BIN_FULL {
        w.spawn_item([p[0] as f64 + 0.5, p[1] as f64 + 1.1, p[2] as f64 + 0.5], ItemStack::new(COMPOST.id(), 1));
        w.set_prop(p, "level", "0");
        return true;
    }
    if level >= soil::BIN_FULL {
        return false;
    }
    let inv = w.inventory(player);
    let slot = w.selected_slot(player);
    let held = inv.get(slot);
    if held.is_empty() {
        return false;
    }
    let spoiled = FRESHNESS.stack_only(&held) == Some(freshness::SPOILED);
    let Some(adds) = (if spoiled { Some(soil::SPOILED) } else { soil::composts(item_name(held.item)) }) else { return false };
    w.set_prop(p, "level", &(level + adds).min(soil::BIN_FULL).to_string());
    if !w.creative(player) {
        inv.set(slot, StackData::shrink(held, 1));
    }
    true
}

/// A full bin rots into compost (its growth step, `soil::compost_secs` on average).
pub fn rot(e: &mut BlockEvent) {
    if state_prop(e.state(), "level") == Some(&*soil::BIN_FULL.to_string()) {
        e.world()
            .set_prop(e.pos(), "level", &(soil::BIN_FULL + 1).to_string());
    }
}

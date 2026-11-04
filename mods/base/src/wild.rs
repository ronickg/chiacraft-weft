//! The wild plants (`notes/ECOLOGY.md`, part 3; the numbers in `vitals::wild`, their growth in
//! `vitals::plants`): every `base:` row of `vitals::wild::WILD` as a block, growing by its form.
//! Tufts and mats grazed down grow back, bushes ripen their fruit through four stages, fruit and
//! seed heads ripen in their season, mushrooms and water hyacinth spread near the old ones, the
//! rest stand. What picking one gives plants it again. `worldgen.rs` puts them where they grow.

use std::sync::OnceLock;

use vitals::wild::{self, Form, Ground, Wild};
use weft_sdk::*;

use crate::growth;

static PLANT_SOIL: Tag = Tag::named("plant_soil");
static WATER: Tag = Tag::named("water");

/// The base mod's wild plants.
fn mine() -> impl Iterator<Item = (usize, &'static Wild)> {
    wild::WILD
        .iter()
        .enumerate()
        .filter(|(_, p)| p.block.starts_with("base:"))
}

/// The name without its mod.
fn short(block: &'static str) -> &'static str {
    block.split_once(':').map_or(block, |(_, n)| n)
}

pub fn register(r: &mut Registry) {
    for (_, p) in mine() {
        let name = short(p.block);
        let block = Block::generated(name.into());
        let b = r.block(block).sound_group("grass").plant();
        let b = match p.ground {
            Ground::Soil => b.needs_below("#plant_soil"),
            Ground::Sand => b.needs_below("#sand"),
            Ground::Shore => b.needs_below("#plant_soil").needs_near_support("#water"),
            Ground::Water => b.needs_below("#water"),
        };
        let full = p.block;
        // Spreading plants come up in the shade too (mushrooms); the others need daylight.
        let light = if p.ground == Ground::Soil && p.form == Form::Spreads { 0 } else { 9 };
        let b = match p.form {
            // Grazed down (age 0) and grown back (age 1); the world makes them grown.
            Form::Tuft => b
                .prop("age", 0..=1)
                .texture(name)
                .no_item()
                .growth(growth::step_secs(full, 1, 600.0), light)
                .on_grow(move |e| growth::grow(e, full, false, |e| growth::next_age(e, 1))),
            Form::Bush => b
                .prop("age", 0..=3)
                .states(move |p, s| s.texture(&format!("{name}_stage_{}", p.get("age"))))
                .no_item()
                .growth(growth::step_secs(full, 3, 300.0), light)
                .on_grow(move |e| {
                    let last = growth::age(e) == 2;
                    growth::grow(e, full, last, |e| growth::next_age(e, 3))
                }),
            Form::Ripens => b
                .prop("age", 0..=1)
                .texture(name)
                .no_item()
                .growth(growth::step_secs(full, 1, 300.0), light)
                .on_grow(move |e| growth::grow(e, full, true, |e| growth::next_age(e, 1))),
            Form::Spreads => {
                let b = b
                    .texture(name)
                    .growth(growth::step_secs(full, 1, 300.0), light)
                    .on_grow(move |e| growth::grow(e, full, true, |e| spread(e, full)));
                if p.gives.is_some() { b.no_item() } else { b.tab(&crate::tabs::NATURAL) }
            }
            Form::Still => b.texture(name).tab(&crate::tabs::NATURAL),
        };
        drop(b);
        // Picking gives what plants it again: one at any stage, two more when ripe.
        if let Some(gives) = p.gives.map(short) {
            let table = r.loot(block).pool(Pool::new().entry(Entry::item(gives)));
            if matches!(p.form, Form::Bush | Form::Ripens) {
                let ripe = if p.form == Form::Bush { "age=3" } else { "age=1" };
                table
                    .pool(
                        Pool::new()
                            .when(Cond::BlockState(ripe))
                            .entry(Entry::item(gives)),
                    )
                    .pool(
                        Pool::new()
                            .when(Cond::BlockState(ripe))
                            .entry(Entry::item(gives)),
                    );
            }
        }
    }
}

/// The block a picked item of that name plants again, if any (`items.rs`): a wild plant that
/// gives it.
pub fn planted_by(item: &str) -> Option<&'static Block> {
    mine()
        .find(|(_, p)| p.gives.map(short) == Some(item))
        .map(|(_, p)| Block::generated(short(p.block).into()))
}

/// The state the world makes each wild plant in (`worldgen.rs`, any mod's: the Cretaceous's are
/// the animals mod's), by its index in `WILD`: tufts grown, bushes and seed heads unripe;
/// `u32::MAX` for one not registered.
pub fn state(i: usize) -> u32 {
    static STATES: OnceLock<Vec<u32>> = OnceLock::new();
    STATES.get_or_init(|| {
        wild::WILD
            .iter()
            .map(|p| {
                let Some(id) = Block::lookup(p.block) else { return u32::MAX };
                if p.form == Form::Tuft { state_with(id, "age", "1").unwrap_or(id) } else { id }
            })
            .collect()
    })[i]
}

/// A spreading plant's growth step: a new one nearby on its ground, unless `CROWD` are there.
fn spread(e: &mut BlockEvent, block: &str) {
    let w = e.world();
    let [x, y, z] = e.pos();
    let me = e.state();
    let r = wild::REACH;
    let mut near = 0;
    for dz in -r..=r {
        for dy in -1..=1 {
            for dx in -r..=r {
                if w.get([x + dx, y + dy, z + dz]) == me {
                    near += 1;
                }
            }
        }
    }
    if near >= wild::CROWD {
        return;
    }
    let roll = w.random();
    let span = (2 * r + 1) as u64;
    let at = [x + (roll % span) as i32 - r, y + ((roll >> 8) % 3) as i32 - 1, z + ((roll >> 16) % span) as i32 - r];
    let below = w.get([at[0], at[1] - 1, at[2]]);
    let ground = wild::of(block).map(|p| p.ground);
    let fits = match ground {
        Some(Ground::Water) => WATER.on_block(below),
        _ => PLANT_SOIL.on_block(below),
    };
    if w.get(at) == 0 && fits {
        w.set(at, me);
    }
}

/// What a frost does to a wild plant (`growth::grow`): a bush loses its flowers and fruit, a seed
/// head, a mushroom or a floating plant dies.
pub fn frost(e: &mut BlockEvent, block: &str) {
    let w = e.world();
    match wild::of(block).map(|p| p.form) {
        Some(Form::Bush) if growth::age(e) > 1 => {
            w.set_prop(e.pos(), "age", "1");
        }
        Some(Form::Ripens | Form::Spreads) => {
            w.set(e.pos(), 0);
        }
        _ => {}
    }
}

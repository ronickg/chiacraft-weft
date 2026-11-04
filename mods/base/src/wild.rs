//! The wild plants (`notes/ECOLOGY.md`, part 3; the numbers in `vitals::wild`, their growth in
//! `vitals::plants`): every `base:` row of `vitals::wild::WILD` as a block, growing by its form.
//! Tufts and mats grazed down (a bitten look) grow back, bushes ripen their fruit through four
//! stages, fruit and seed heads ripen in their season (a ripe look), mushrooms and water hyacinth spread near the old ones, kelp
//! grows up towards the surface, the rest stand. What picking one gives plants it again. Plants of
//! the bed stand in the water (`holds_liquid`): breaking one leaves the water. `worldgen.rs` puts
//! them where they grow.

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
    // What plants of the bed root in, and what kelp stands on: the bed or kelp under it.
    r.tag("water_bed", &["#dirt", "#sand", "gravel", "clay"]);
    r.tag("kelp_soil", &["#water_bed", "kelp"]);
    for (_, p) in mine() {
        let name = short(p.block);
        let block = Block::generated(name.into());
        let b = r.block(block).sound_group("grass").plant();
        let b = match p.ground {
            Ground::Soil => b.needs_below("#plant_soil"),
            Ground::Sand => b.needs_below("#sand"),
            Ground::Shore => b.needs_below("#plant_soil").needs_near_support("#water"),
            Ground::Water => b.needs_below("#water"),
            Ground::Bed => b
                .needs_below(if p.form == Form::Column { "#kelp_soil" } else { "#water_bed" })
                .holds_liquid(&crate::blocks::WATER),
        };
        let full = p.block;
        // Spreading plants come up in the shade too (mushrooms), and the bed's under water that
        // dims the light; the others need daylight.
        let light = if p.ground == Ground::Soil && p.form == Form::Spreads || p.ground == Ground::Bed { 0 } else { 9 };
        let b = match p.form {
            // Grazed down (age 0) and grown back (age 1); the world makes them grown.
            Form::Tuft => b
                .prop("age", 0..=1)
                .states(move |p, s| s.texture(&if p.get("age") == "0" { format!("{name}_grazed") } else { name.to_string() }))
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
                .states(move |p, s| s.texture(&if p.get("age") == "1" { format!("{name}_ripe") } else { name.to_string() }))
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
            // A lily pad lies flat on the water.
            Form::Still if name == "lily_pad" => b
                .texture(name)
                .model_box([0.0, 0.0, 0.0], [16.0, 0.5, 16.0])
                // A slot shows its own picture, not a half-pixel sliver of the model.
                .item_texture(name)
                .tab(&crate::tabs::NATURAL),
            // A common reed, its purple plumes on top.
            Form::Still if name == "reed" => b.texture("reed").tab(&crate::tabs::NATURAL),
            // No base plant lies as a log yet (the Cretaceous's rotten log is the animals mod's).
            Form::Still | Form::Log => b.texture(name).tab(&crate::tabs::NATURAL),
            // Kelp: its top block (no kelp above) shows the fronds' tip, `<name>_top`; a block
            // changing above it sets which it is, so growing, placing and breaking keep it right.
            Form::Column => b
                .prop("top", ["true", "false"])
                .states(move |p, s| s.texture(&if p.get("top") == "true" { format!("{name}_top") } else { name.to_string() }))
                .on_neighbor(move |e| {
                    let w = e.world();
                    let [x, y, z] = e.pos();
                    let top = if block.is(w.get([x, y + 1, z])) { "false" } else { "true" };
                    if let Some(s) = state_with(e.state(), "top", top).filter(|&s| s != e.state()) {
                        w.set(e.pos(), s);
                    }
                })
                .growth(growth::step_secs(full, 1, 270.0), light)
                .on_grow(move |e| growth::grow(e, full, false, grow_up))
                // Picked, it gives a handful of kelp, the food (`items.rs` registers its item).
                .no_item(),
            // A flat mat on the ground, a sixteenth high; picked, it gives its plant. Wrack is
            // kelp washed up, drawn as kelp.
            Form::Mat => b
                .texture(if name == "kelp_wrack" { "kelp" } else { name })
                .model_box([0.0, 0.0, 0.0], [16.0, 1.0, 16.0])
                .no_item(),
        };
        drop(b);
        // Picking gives what plants it again: a bush or a seed head three when ripe and nothing out
        // of season, the others one; a cordage plant fibre besides (`wild::FIBRE`).
        let fibre = wild::FIBRE
            .iter()
            .find(|(b, _)| *b == p.block)
            .map(|&(_, chance)| chance);
        if p.gives.is_some() || fibre.is_some() {
            let mut table = r.loot(block);
            if let Some(gives) = p.gives.map(short) {
                let ripe = match p.form {
                    Form::Bush => Some("age=3"),
                    Form::Ripens => Some("age=1"),
                    _ => None,
                };
                table = table.pool(match ripe {
                    Some(ripe) => Pool::new()
                        .when(Cond::BlockState(ripe))
                        .entry(Entry::item(gives).count(3, 3)),
                    None => Pool::new().entry(Entry::item(gives)),
                });
            }
            if let Some(chance) = fibre {
                table.pool(Pool::new().when(Cond::Chance(chance)).item("plant_fibre"));
            }
        }
    }
    flowers(r);
}

/// Each biome's wild flowers (`vitals::wild::FLOWERS`): plants that stand and give themselves,
/// in bloom (`bloom` 1) only in their own seasons and green the rest of the year, a growth step
/// every few minutes opening or closing them as the seasons turn.
fn flowers(r: &mut Registry) {
    for f in &wild::FLOWERS {
        let name = short(f.block);
        let blooms = f.blooms;
        r.block(Block::generated(name.into()))
            .sound_group("grass")
            .plant()
            .needs_below(if f.ground == Ground::Sand { "#sand" } else { "#plant_soil" })
            .prop("bloom", 0..=1)
            .states(move |p, s| s.texture(&if p.get("bloom") == "1" { name.to_string() } else { format!("{name}_green") }))
            .growth(BLOOM_SECS, 0)
            .on_grow(move |e| {
                let season = base_api::ground_at(&e.world(), e.pos()).season;
                let open = season.is_none_or(|s| blooms >> s & 1 != 0);
                let want = if open { "1" } else { "0" };
                if state_prop(e.state(), "bloom") != Some(want) {
                    e.world().set_prop(e.pos(), "bloom", want);
                }
            })
            .tab(&crate::tabs::NATURAL);
    }
}

/// Game seconds between a flower's looks at the season, on average.
const BLOOM_SECS: f32 = 300.0;

/// A flower of `biome` on `ground` for the world to put (`worldgen.rs`), `kind` choosing among
/// them; `None`: the biome has none of its own there.
pub fn flower(biome: &str, ground: Ground, kind: u64) -> Option<u32> {
    // A world starts in spring: the spring bloomers come open, the others green.
    static IDS: OnceLock<Vec<u32>> = OnceLock::new();
    let ids = IDS.get_or_init(|| {
        wild::FLOWERS
            .iter()
            .map(|f| {
                let Some(id) = Block::lookup(f.block) else { return u32::MAX };
                let open = if f.blooms & 1 != 0 { "1" } else { "0" };
                state_with(id, "bloom", open).unwrap_or(id)
            })
            .collect()
    });
    let here: Vec<usize> = (0..wild::FLOWERS.len())
        .filter(|&i| wild::FLOWERS[i].ground == ground && wild::FLOWERS[i].found.contains(&biome))
        .collect();
    let i = *here.get(kind as usize % here.len().max(1))?;
    (ids[i] != u32::MAX).then_some(ids[i])
}

/// The block a picked item of that name plants again, if any (`items.rs`): a wild plant that
/// gives it.
pub fn planted_by(item: &str) -> Option<&'static Block> {
    mine()
        .find(|(_, p)| p.gives.map(short) == Some(item))
        .map(|(_, p)| Block::generated(short(p.block).into()))
}

/// The state the world makes each wild plant in (`worldgen.rs`, any mod's: the Cretaceous's are
/// the animals mod's), by its index in `WILD`: tufts grown, bushes and seed heads unripe but for
/// fruit that holds over winter (`wild::HOLDS_FRUIT`);
/// `u32::MAX` for one not registered.
pub fn state(i: usize) -> u32 {
    static STATES: OnceLock<Vec<u32>> = OnceLock::new();
    STATES.get_or_init(|| {
        wild::WILD
            .iter()
            .map(|p| {
                let Some(id) = Block::lookup(p.block) else { return u32::MAX };
                match p.form {
                    Form::Tuft => state_with(id, "age", "1").unwrap_or(id),
                    // Fruit that holds on the plant over winter is there whenever you come.
                    Form::Bush if wild::HOLDS_FRUIT.contains(&p.block) => state_with(id, "age", "3").unwrap_or(id),
                    _ => id,
                }
            })
            .collect()
    })[i]
}

/// A column's growth step (kelp): one more on top while it is still a block below the surface.
fn grow_up(e: &mut BlockEvent) {
    let w = e.world();
    let [x, y, z] = e.pos();
    let (above, over) = ([x, y + 1, z], [x, y + 2, z]);
    // A water source, the first of its states.
    let still = |p| w.get(p) == crate::blocks::WATER.id();
    if still(above) && still(over) {
        w.set(above, e.state());
    }
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
        Some(Ground::Water) => WATER.in_cell(below),
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

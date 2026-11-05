//! The crops beyond wheat (`vitals::crops`, their growth in `vitals::plants`): barley, rye,
//! carrots, potatoes, beetroot, maize, paddy rice, beans, sorghum, millet and cassava. Each grows
//! on farmland by the place's warmth and water (`growth.rs`), four stages to ripe; picked unripe
//! it gives back what sowed it, ripe its harvest and seed; its ripe harvest tires the farmland, or
//! feeds it (beans; `farming.rs`). Paddy rice grows only by water. Their seeds come from tall
//! grass in the biomes where they grow wild (`vitals::crops::Crop::wild`); wheat's from tall grass
//! anywhere. Potatoes bake, corn roasts and the paddy's grain boils into a bowl of white rice;
//! rye, barley, sorghum and millet bake into bread as wheat does.

use vitals::crops::{self, CROPS, Crop};
use weft_sdk::*;

use crate::blocks::TALL_GRASS;
use crate::growth;

/// The stems' fruit (`vitals::crops::Crop::fruit`): full blocks set beside a ripe stem, broken
/// into slices.
static PUMPKIN: Block = Block::named("pumpkin");
static MELON: Block = Block::named("melon");
static PLANT_SOIL: Tag = Tag::named("plant_soil");

fn short(name: &'static str) -> &'static str {
    name.split_once(':').map_or(name, |n| n.1)
}

/// The crop blocks' look: `paddy_rice` is drawn as `rice`, the plurals by their singular.
fn texture(block: &str) -> &str {
    match block {
        "paddy_rice" => "rice",
        "carrots" => "carrot",
        "potatoes" => "potato",
        "beetroots" => "beetroot",
        b => b,
    }
}

pub fn register(r: &mut Registry) {
    // Wheat is the blocks' own (`blocks.rs`), at eight stages.
    for c in CROPS.iter().filter(|c| c.block != "base:wheat") {
        crop(r, c);
    }
    // The seeds and grains that aren't foods (the foods are `vitals::food::FOODS`' rows; what
    // they sow is set in `items.rs`).
    for c in CROPS.iter().filter(|c| c.block != "base:wheat") {
        if vitals::food::find(c.seed).is_none() {
            r.item(Item::generated(short(c.seed).into()))
                .places(Block::generated(short(c.block).into()))
                .tag(&format!("crops/{}", short(c.seed)))
                .tab(&crate::tabs::NATURAL);
        }
        if c.gives != c.seed && vitals::food::find(c.gives).is_none() {
            r.item(Item::generated(short(c.gives).into()))
                .tag(&format!("crops/{}", short(c.gives)))
                .tab(&crate::tabs::INGREDIENTS);
        }
    }
    // The foods a crop gives, by tag, for the recipes.
    for name in ["carrot", "potato", "beetroot", "corn", "beans", "cassava"] {
        r.tag(&format!("crops/{name}"), &[name]);
    }
    r.tag("bread_grain", &crops::BREAD_GRAINS.map(short));
    let s = &base_api::SMELTING;
    r.shapeless(s, "baked_potato", 1, &["#crops/potato"], 200);
    r.shapeless(s, "roasted_corn", 1, &["#crops/corn"], 200);
    r.shapeless(s, "cooked_rice", 1, &["#crops/rice_seeds"], 200);

    // The stems' fruit: a pumpkin breaks into three to five slices, a melon three to seven; a
    // slice gives its seeds.
    for (b, top, side, slice, (lo, hi)) in
        [(&PUMPKIN, "pumpkin_top", "pumpkin_side", "pumpkin_slice", (3, 5)), (&MELON, "melon_top", "melon_side", "melon_slice", (3, 7))]
    {
        r.block(b)
            .sound_group("wood")
            .textures(top, side, top)
            .hardness(1.0)
            .mineable("axe")
            .tab(&crate::tabs::NATURAL);
        r.loot(b)
            .pool(Pool::new().entry(Entry::item(slice).count(lo, hi)));
    }
    let c = &base_api::CRAFTING;
    r.shapeless(c, "pumpkin_seeds", 2, &["pumpkin_slice"], 0);
    r.shapeless(c, "melon_seeds", 2, &["melon_slice"], 0);

    // Tall grass broken by a player gives now and then a seed of one of its biome's crops.
    r.on(&actions::BREAK_BLOCK, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| e.player != 0 && TALL_GRASS.is(e.state)) {
            if w.outside_survival(e.player) || unit(w.random()) >= crops::WILD_SEED_CHANCE {
                continue;
            }
            let biome = name_of(abi::entry_kind::BIOME, w.biome(e.pos));
            let found: Vec<&Crop> = crops::wild_in(biome).collect();
            if found.is_empty() {
                continue;
            }
            let c = found[(w.random() % found.len() as u64) as usize];
            if let Some(id) = Item::lookup(c.seed) {
                let p = e.pos;
                w.spawn_item([p.x as f64 + 0.5, p.y as f64 + 0.3, p.z as f64 + 0.5], ItemStack::new(id, 1));
            }
        }
    });
}

/// 0 to 1 from a random number.
fn unit(r: u64) -> f32 {
    (r >> 40) as f32 / (1u64 << 24) as f32
}

/// A crop's block and loot.
fn crop(r: &mut Registry, c: &'static Crop) {
    let name = short(c.block);
    let block = Block::generated(name.into());
    let look = texture(name);
    let ripe = c.ripe;
    let b = r
        .block(block)
        .sound_group("grass")
        .prop("age", 0..=ripe)
        .states(move |p, s| s.texture(&format!("{look}_stage_{}", p.get("age"))))
        .plant()
        .needs_below("#crop_soil")
        .no_item()
        // By warmth and water (`growth.rs`): `ripe` stages over its real days' pace; its last
        // waits for the season it ripens in (winter rye's summer).
        .grows(growth::step_secs(c.block, ripe, 205.0), 9)
        .on_grow(move |e| {
            let age = growth::age(e);
            // A ripe stem sets its fruit beside it in its season (`set_fruit`), again and again.
            if let Some(fruit) = c.fruit.filter(|_| age >= ripe) {
                growth::bear(e, c.block, |e| set_fruit(e, fruit));
                return;
            }
            // A stem grows in any season; the others' last step waits for their ripening one.
            let last = c.fruit.is_none() && age + 1 == ripe;
            growth::grow(e, c.block, last, |e| grow_up(e, c))
        });
    // Paddy rice stands by water.
    let b = if vitals::plants::of(c.block).is_some_and(|p| p.water == vitals::plants::Water::Wet) { b.needs_near("#water") } else { b };
    drop(b);
    if let Some(from) = c.tall {
        top(r, c, from);
    }
    let ripe_state: &'static str = Box::leak(format!("age={ripe}").into_boxed_str());
    let loot = r
        .loot(block)
        .pool(Pool::new().entry(Entry::item(short(c.seed))))
        .pool(
            Pool::new()
                .when(Cond::BlockState(ripe_state))
                .entry(Entry::item(short(c.gives)).count(c.count.0, c.count.1)),
        );
    if c.seed != c.gives {
        drop(
            loot.pool(
                Pool::new()
                    .when(Cond::BlockState(ripe_state))
                    .entry(Entry::item(short(c.seed)).count(0, crops::EXTRA_SEEDS)),
            ),
        );
    }
}

/// A tall crop's upper half (`vitals::crops::Crop::tall`): its stages' tops, standing on the crop
/// itself, so breaking the crop takes it (it loses its support, and drops nothing). Broken by a
/// player, it breaks the crop below as that player did, so the harvest drops once, as from the
/// crop.
fn top(r: &mut Registry, c: &'static Crop, from: u32) {
    let name = short(c.block);
    let look = texture(name);
    let stalk = format!("{name}_stalk");
    r.tag(&stalk, &[name]);
    drop(
        r.block(Block::generated(format!("{name}_top")))
            .sound_group("grass")
            .prop("age", from..=c.ripe)
            .states(move |p, s| s.texture(&format!("{look}_stage_{}_top", p.get("age"))))
            .plant()
            .needs_below(&format!("#{stalk}"))
            .no_item()
            .on_broken(|w, b| {
                if b.player == 0 {
                    return;
                }
                let below = b.pos.below();
                let tool = if b.item == 0 { ItemStack::EMPTY } else { ItemStack::new(b.item, 1) };
                w.break_block(below, tool, b.player, !w.outside_survival(b.player));
            }),
    );
}

/// A crop's next stage. A tall crop (`Crop::tall`) growing into its two-block stages sets or
/// turns its top first, and waits while something stands above it.
fn grow_up(e: &mut BlockEvent, c: &Crop) {
    let next = growth::age(e) + 1;
    if next > c.ripe {
        return;
    }
    if c.tall_at(next) {
        let w = e.world();
        let above = e.pos().above();
        let Some(top) = Block::lookup(&format!("{}_top", c.block)) else { return };
        let here = w.block(above);
        if here == 0 {
            w.set_block(above, top);
        } else if block_name(here).split('[').next() != block_name(top).split('[').next() {
            return;
        }
        w.prop_set(above, "age", &next.to_string());
    }
    growth::next_age(e, c.ripe)
}

/// A ripe stem's fruit, set on an empty block beside it that stands on soil, unless one already
/// lies beside it (a vine bears one at a time where it lies).
fn set_fruit(e: &mut BlockEvent, fruit: &str) {
    let w = e.world();
    let [x, y, z] = e.pos().to_array();
    let Some(id) = Block::lookup(fruit) else { return };
    const SIDES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    if SIDES
        .iter()
        .any(|&(dx, dz)| w.block([x + dx, y, z + dz]) == id)
    {
        return;
    }
    let start = (w.random() % 4) as usize;
    for k in 0..4 {
        let (dx, dz) = SIDES[(start + k) % 4];
        let at = [x + dx, y, z + dz];
        if w.block(at) == 0 && PLANT_SOIL.on_block(w.block([x + dx, y - 1, z + dz])) {
            w.set_block(at, id);
            return;
        }
    }
}

/// The crop block a food item sows (carrots, potatoes, beetroot, cassava).
pub fn planted_by(item: &str) -> Option<&'static Block> {
    let c = crops::sown_by(&format!("base:{item}"))?;
    Some(Block::generated(short(c.block).into()))
}

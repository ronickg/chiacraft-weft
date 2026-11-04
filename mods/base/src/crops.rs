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
                .placed_block(Block::generated(short(c.block).into()))
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

    // Tall grass broken by a player gives now and then a seed of one of its biome's crops.
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| e.player != 0 && TALL_GRASS.is(e.state)) {
            if w.creative(e.player) || unit(w.random()) >= crops::WILD_SEED_CHANCE {
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
                w.spawn_item([p[0] as f64 + 0.5, p[1] as f64 + 0.3, p[2] as f64 + 0.5], ItemStack::new(id, 1));
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
        .growth(growth::step_secs(c.block, ripe, 205.0), 9)
        .on_grow(move |e| {
            let last = growth::age(e) + 1 == ripe;
            growth::grow(e, c.block, last, |e| growth::next_age(e, ripe))
        });
    // Paddy rice stands by water.
    let b =
        if vitals::plants::of(c.block).is_some_and(|p| p.water == vitals::plants::Water::Wet) { b.needs_near_support("#water") } else { b };
    drop(b);
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

/// The crop block a food item sows (carrots, potatoes, beetroot, cassava).
pub fn planted_by(item: &str) -> Option<&'static Block> {
    let c = crops::sown_by(&format!("base:{item}"))?;
    Some(Block::generated(short(c.block).into()))
}

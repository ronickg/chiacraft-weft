//! Meat from wild animals (`animals_core::meat`): each meat a raw item and a cooked one, eaten as
//! the farm's are (`base_api::eat`, nutrition from `vitals::food::FOODS`), and tagged
//! `raw_meat/<meat>` and `cooked_meat/<meat>` so the base game's smelting, salting, drying,
//! smoking and stews take them with no code of their own. The carcass drops (`lib.rs`).

use weft_sdk::*;

static FOOD_TAB: CreativeTab = CreativeTab::named("base:food");
/// What's left of a dead animal that gives no meat (`animals_core::meat::CARCASS`): carrion for
/// the scavengers, no food for a player (its own icon: a torn hide with ribs, Art 11-05).
static CARCASS: Item = Item::named("carcass");

/// Hunger and saturation, raw and cooked: venison as beef, hare as chicken, fish as the cod.
fn filling(meat: &str) -> [(u32, f32); 2] {
    match meat {
        "venison" => [(3, 1.8), (8, 12.8)],
        "hare" => [(2, 1.2), (6, 7.2)],
        _ => [(2, 0.4), (5, 6.0)],
    }
}

pub fn init(r: &mut Registry) {
    r.item(&CARCASS).tab(&FOOD_TAB);
    for &meat in animals_core::meat::MEATS {
        let [raw, cooked] = filling(meat);
        for (name, (hunger, saturation), tag) in
            [(meat.to_string(), raw, format!("raw_meat/{meat}")), (format!("cooked_{meat}"), cooked, format!("cooked_meat/{meat}"))]
        {
            // What a portion holds (from the vitals' food table), shown in its tooltip.
            let [units, protein, energy, fat, greens, water] = vitals::food::find(&format!("animals:{name}"))
                .map_or([0.0; 6], |f| {
                    let f = &vitals::food::FOODS[f];
                    let g = f.portion();
                    [f.units(), g[0], g[1], g[2], g[3], g[4]]
                })
                .map(|v| v.round() as u16);
            let i = r
                .item(Item::generated(name))
                .set(&base_api::FOOD, base_api::Food { hunger, saturation })
                .set(&base_api::NUTRITION, base_api::Nutrition { units, protein, energy, fat, greens, water })
                .set(&base_api::FRESHNESS, base_api::freshness::FRESH)
                .use_ticks(32)
                .on_use(base_api::eat)
                .tab(&FOOD_TAB)
                .tag(&tag);
            drop(i.tag(if tag.starts_with("raw") { "raw_meat" } else { "cooked_meat" }));
        }
    }
}

/// The pool of meat a species' carcass drops, if it gives meat.
pub fn pool(species: &str) -> Option<Pool> {
    animals_core::meat::drops()
        .find(|&(name, ..)| name == species)
        .map(|(_, meat, lo, hi)| Pool::new().items(&format!("animals:{meat}"), lo, hi))
}

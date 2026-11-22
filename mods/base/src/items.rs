use weft_sdk::*;

pub use base_api::items::*;

// Ours: armour is our stack component (`base_api::ARMOR`, a piece built from parts carries its
// points); upstream's armor attributes and `ARMOR_PIECE` tag stay in base_api for their code.
pub use base_api::components::{FLAMMABLE, FOOD, FUEL, Food};
pub use base_api::{ARMOR, Armor};
/// Tags of armor pieces, head to feet (every armour kind's items).
pub const ARMOR_SLOTS: [&str; 4] = ["armor_head", "armor_chest", "armor_legs", "armor_feet"];

pub fn register(r: &mut Registry) {
    r.item(&SHIELD)
        .tag("offhand_preferred")
        .max_stack(1)
        .durability(336)
        .use_until_released()
        .set(&comps::BLOCKS_ATTACKS, Blocking { delay_ticks: 0, arc: 180.0, wear_threshold: 3.0, wear_base: 1, wear_factor: 1.0 })
        .set(
            &comps::CATCH_PROJECTILES,
            ProjectileCatch { bone: "guard".into(), min: [-4.0, -6.0, -4.0], max: [4.0, 6.0, -4.0], direction: [0.0, 0.0, 1.0] },
        )
        .tab(&crate::tabs::COMBAT);
    r.shaped(&crate::recipes::CRAFTING, "shield", 1, &["pip", "ppp", " p "], &[('p', "#planks"), ('i', "#ingots/iron")]);
    r.item(&STICK)
        .tag("stick")
        .handheld()
        .set(&FUEL, 100)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&COAL)
        .tag("gems/coal")
        .set(&FUEL, 1600)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&IRON_INGOT)
        .tag("ingots/iron")
        .tag("materials/iron")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&GOLD_INGOT)
        .tag("ingots/gold")
        .tag("materials/gold")
        .tab(&crate::tabs::INGREDIENTS);
    // Metal ores drop raw pieces (like modern versions) that smelt into ingots.
    r.item(Item::generated("raw_iron".into()))
        .tag("raw_ores/iron")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(Item::generated("raw_gold".into()))
        .tag("raw_ores/gold")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&DIAMOND)
        .tag("gems/diamond")
        .tag("materials/diamond")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&WHEAT_SEEDS)
        .places(&crate::blocks::WHEAT)
        .tab(&crate::tabs::NATURAL);
    r.item(&WHEAT)
        .tag("crops/wheat")
        .tab(&crate::tabs::INGREDIENTS);
    // What gravel and clay drop.
    // Flint is `first.rs`'s (struck on a rock).
    r.item(&CLAY_BALL).tab(&crate::tabs::INGREDIENTS);
    // Filled buckets come with their liquids (blocks.rs); the engine fills and pours them.
    r.item(&BUCKET).max_stack(16).tab(&crate::tabs::TOOLS);
    r.set("lava_bucket", &FUEL, 20000);
    r.tag("buckets/lava", &["lava_bucket"]);
    r.tag("bread", &["bread"]);
    r.tag("apple", &["apple"]);

    // Every food of the base game is a row of the vitals' food table (`vitals::food::FOODS`), the one
    // list: its nutrition, freshness and tags come from there. The base game's hunger and
    // saturation (only the tooltip's food icons read them) follow its size: a unit is 10 g.
    for f in vitals::food::FOODS
        .iter()
        .filter(|f| f.name.starts_with("base:"))
    {
        let name = &f.name["base:".len()..];
        let hunger = ((f.units() / 4.0).round() as u32).clamp(1, 10);
        let saturation = hunger as f32 * 1.2;
        // Raw meat with a cooked row pairs with it by name (`#raw_meat/beef` smelts into
        // `#cooked_meat/beef`), so cooking, salting, drying and stews take any meat.
        let cooked_of = name
            .strip_prefix("cooked_")
            .filter(|raw| vitals::food::find(&format!("base:{raw}")).is_some());
        let raw_with_cooked = f.raw && vitals::food::find(&format!("base:cooked_{name}")).is_some();
        let tag = match (cooked_of, raw_with_cooked) {
            (Some(raw), _) => format!("cooked_meat/{raw}"),
            (None, true) => format!("raw_meat/{name}"),
            _ => String::new(),
        };
        let tag = tag.as_str();
        // What a portion holds and how fresh it is (stamped as it ages, `survival/food.rs`): shown in
        // its tooltip.
        let g = f.portion();
        let [units, protein, energy, fat, greens, water] = [f.units(), g[0], g[1], g[2], g[3], g[4]].map(|v| v.round() as u16);
        let i = r
            .item(Item::generated(name.into()))
            .set(&FOOD, Food { hunger, saturation })
            .set(&base_api::NUTRITION, base_api::Nutrition { units, protein, energy, fat, greens, water })
            .set(&base_api::FRESHNESS, base_api::freshness::FRESH)
            .use_ticks(32)
            .on_use(base_api::eat)
            .tab(&crate::tabs::FOOD);
        let i = if tag.starts_with("cooked_meat/") { i.tag("cooked_meat") } else { i };
        // Dried, smoked or salted (`vitals::preserve`).
        let i = if vitals::preserve::PRODUCTS.contains(&f.name) { i.tag("preserved") } else { i };
        // Any raw meat salts (`cooking.rs`).
        let i = if tag.starts_with("raw_meat/") { i.tag("raw_meat") } else { i };
        // Berries are planted like seeds: a new bush; so are the wild plants' fruit and seeds,
        // and the roots a crop is sown from (`crops.rs`).
        let i = match crate::wild::planted_by(name)
            .or_else(|| crate::crops::planted_by(name))
            .or_else(|| crate::blocks::sapling_planted_by(name))
        {
            Some(b) => i.places(b),
            None => i,
        };
        if !tag.is_empty() {
            i.tag(tag);
        }
    }
    // Seeds animals eat off the ground (`vitals::forage::DROPPED`): acorns, which plant an oak,
    // and wild rice's grain, which plants wild rice.
    r.item(Item::generated("acorn".into()))
        .places(&crate::blocks::SAPLING)
        .tab(&crate::tabs::NATURAL);
    let rice = r
        .item(Item::generated("rice".into()))
        .tag("rice")
        .tab(&crate::tabs::NATURAL);
    match crate::wild::planted_by("rice") {
        Some(b) => drop(rice.places(b)),
        None => drop(rice),
    }
    // Upstream's mushrooms and berries (Weft sync 11-22, kept until Ronald's call; their blocks
    // are in `blocks.rs`). The edible ones (raspberry, snowberry, toadstool, cooked dryad's saddle)
    // are rows of the vitals' food table, registered by the food loop above and eaten our way:
    // the toadstool poisons and the snowberry may make one sick (`vitals::food`), in place of
    // upstream's nausea and poison effects. The raw dryad's saddle is only cooked. Upstream's
    // porcini and chanterelle items are our mushroom blocks' (`blocks.rs`, the same names): their
    // `porcini_mushroom` and `chanterelle_mushroom` drop ours. Cutting them is this block, their
    // food rows and their blocks.
    r.item(&DRYADS_SADDLE)
        .tag("mushrooms")
        .tab(&crate::tabs::INGREDIENTS);
    r.tag("mushrooms", &["toadstool", "cooked_dryads_saddle"]);
    // Buckskin, brain-tanned from a hide at a tanning frame (`crafts.rs`): clothes and books. Leather
    // armor is sole leather's (`materials/leather`), bark-tanned for days in a pit.
    r.item(Item::generated("leather".into()))
        .tag("leather")
        .tab(&crate::tabs::INGREDIENTS);
    // paper (from sugar cane) and books (paper and leather) make bookshelves
    for name in ["bone", "string", "sugar", "gunpowder", "feather", "arrow", "egg", "paper", "book"] {
        r.item(Item::generated(name.into()))
            .tag(name)
            .tab(&crate::tabs::INGREDIENTS);
    }
    // Tools, weapons and armour are built from parts (`parts.rs`; Ronald, 11-05: no whole ones).
}

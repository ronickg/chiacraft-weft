use weft_sdk::*;

pub static STICK: Item = Item::named("stick");
pub static COAL: Item = Item::named("coal");
pub static IRON_INGOT: Item = Item::named("iron_ingot");
pub static GOLD_INGOT: Item = Item::named("gold_ingot");
pub static DIAMOND: Item = Item::named("diamond");
pub static WHEAT_SEEDS: Item = Item::named("wheat_seeds");
pub static WHEAT: Item = Item::named("wheat");
pub static BUCKET: Item = Item::named("bucket");

pub use base_api::{ARMOR, Armor, FLAMMABLE, FOOD, FUEL, Food};
/// Tags of armor pieces, head to feet; `<tag>/<material>` picks the recipe (see recipes.rs).
pub const ARMOR_SLOTS: [&str; 4] = ["armor_head", "armor_chest", "armor_legs", "armor_feet"];

struct Material {
    name: &'static str,
    level: u32,
    speed: f32,
    durability: u32,
}

/// Today's tool sets: harvest level, mining speed and uses. Their damage and swings come from the
/// curve by their material's age (`vitals::gear`); a material the parts know harvests at its
/// level there (diamond above steel, `notes/CRAFTING.md`).
const MATERIALS: [Material; 5] = [
    Material { name: "wood", level: tool_tier::WOOD, speed: 2.0, durability: 59 },
    Material { name: "stone", level: tool_tier::STONE, speed: 4.0, durability: 131 },
    Material { name: "iron", level: tool_tier::IRON, speed: 6.0, durability: 250 },
    Material { name: "gold", level: tool_tier::WOOD, speed: 12.0, durability: 32 },
    Material { name: "diamond", level: tool_tier::DIAMOND, speed: 8.0, durability: 1561 },
];

pub fn register(r: &mut Registry) {
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

    for m in &MATERIALS {
        // `.tool(kind)` tags it `tools/<kind>`; `<kind>s/<material>` picks its recipe.
        let tool = |r: &mut Registry, kind: &str| {
            let (damage, speed) = vitals::gear::weapon_of(kind, m.name).unwrap_or((1.0, 1.0));
            let b = r
                .item(Item::generated(format!("{}_{kind}", m.name)))
                .tool(kind)
                .tool_level(crafting::materials::material(m.name).map_or(m.level, |x| x.harvest))
                .mining_speed(m.speed)
                .durability(m.durability)
                .attack_damage(damage)
                .attack_speed(speed)
                .tag(&format!("{kind}s/{}", m.name))
                // Vanilla: weapons wear by one per hit, tools (made for blocks) by two.
                .set(&WEAR_PER_HIT, if kind == "sword" { 1 } else { 2 })
                .tab(if kind == "sword" { &crate::tabs::COMBAT } else { &crate::tabs::TOOLS });
            if kind == "sword" {
                b.sweeps().no_creative_break();
            } else if kind == "hoe" {
                // Tills grass and dirt into farmland (`farming.rs`).
                b.on_use(crate::farming::till);
            }
        };
        for kind in ["pickaxe", "shovel", "sword", "axe", "hoe"] {
            tool(r, kind);
        }
    }

    // Points per piece, head to feet, as in modern.
    for (mat, points, toughness, durability) in [
        ("leather", [1u32, 3, 2, 1], 0.0f32, 5u32),
        ("iron", [2, 6, 5, 2], 0.0, 15),
        ("gold", [2, 5, 3, 1], 0.0, 7),
        ("diamond", [3, 8, 6, 3], 2.0, 33),
    ] {
        for (slot, piece, dur) in [(0, "helmet", 11), (1, "chestplate", 16), (2, "leggings", 15), (3, "boots", 13)] {
            let points = points[slot];
            let tag = ARMOR_SLOTS[slot];
            // Leggings are the inner layer, drawn closer to the body.
            let (inflate, layer) = if slot == 2 { (0.5, 2) } else { (1.0, 1) };
            // Its warmth line too (leather is hide; metal is cold in snow and hot in the sun).
            let mut worn = vitals::clothing::Worn::default();
            worn.armor(mat, slot as usize);
            let warmth = base_api::Warmth {
                cold: worn.cold.round().max(0.0) as u32,
                heat: worn.heat.round() as i32,
                wet: if worn.cold > 0.0 { (worn.cold_wet / worn.cold * 100.0).round() as u32 } else { 0 },
                wind: (worn.windproof * 100.0).round() as u32,
            };
            r.item(Item::generated(format!("{mat}_{piece}")))
                .durability(durability * dur)
                .set(&ARMOR, Armor { points, toughness })
                .set(&base_api::WARMTH, warmth)
                .tag(tag)
                .tag(&format!("{tag}/{mat}"))
                .wearable(crate::player::EQUIPMENT[slot as usize], inflate, &format!("armor/{mat}_{layer}"))
                .tab(&crate::tabs::COMBAT);
        }
    }
}

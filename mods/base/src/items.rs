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
    damage: f32,
}

const MATERIALS: [Material; 5] = [
    Material { name: "wood", level: tool_tier::WOOD, speed: 2.0, durability: 59, damage: 0.0 },
    Material { name: "stone", level: tool_tier::STONE, speed: 4.0, durability: 131, damage: 1.0 },
    Material { name: "iron", level: tool_tier::IRON, speed: 6.0, durability: 250, damage: 2.0 },
    Material { name: "diamond", level: tool_tier::DIAMOND, speed: 8.0, durability: 1561, damage: 3.0 },
    Material { name: "gold", level: tool_tier::WOOD, speed: 12.0, durability: 32, damage: 0.0 },
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
        .placed_block(&crate::blocks::WHEAT)
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
        // Berries are planted like seeds: a new bush; so are the wild plants' fruit and seeds.
        let i = if name == "berries" { i.placed_block(&crate::blocks::BERRY_BUSH) } else { i };
        let i = match crate::wild::planted_by(name) {
            Some(b) => i.placed_block(b),
            None => i,
        };
        if !tag.is_empty() {
            i.tag(tag);
        }
    }
    // Seeds animals eat off the ground (`vitals::forage::DROPPED`): acorns, which plant an oak,
    // and wild rice's grain, which plants wild rice.
    r.item(Item::generated("acorn".into()))
        .placed_block(&crate::blocks::SAPLING)
        .tab(&crate::tabs::NATURAL);
    let rice = r
        .item(Item::generated("rice".into()))
        .tab(&crate::tabs::NATURAL);
    match crate::wild::planted_by("rice") {
        Some(b) => drop(rice.placed_block(b)),
        None => drop(rice),
    }
    r.item(Item::generated("leather".into()))
        .tag("materials/leather")
        .tab(&crate::tabs::INGREDIENTS);
    // paper (from sugar cane) and books (paper and leather) make bookshelves
    for name in ["bone", "string", "sugar", "gunpowder", "feather", "arrow", "egg", "paper", "book"] {
        r.item(Item::generated(name.into()))
            .tag(name)
            .tab(&crate::tabs::INGREDIENTS);
    }

    for m in &MATERIALS {
        // `.tool(kind)` tags it `tools/<kind>`; `<kind>s/<material>` picks its recipe.
        let tool = |r: &mut Registry, kind: &str, damage: f32, speed: f32| {
            let b = r
                .item(Item::generated(format!("{}_{kind}", m.name)))
                .tool(kind)
                .tool_level(m.level)
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
        tool(r, "pickaxe", 2.0 + m.damage, 1.2);
        tool(r, "shovel", 2.5 + m.damage, 1.0);
        tool(r, "sword", 4.0 + m.damage, 1.6);
        // Modern's: slow and strong axes, hoes barely hit but swing fast with the material.
        let (axe_damage, axe_speed, hoe_speed) = match m.name {
            "wood" => (7.0, 0.8, 1.0),
            "stone" => (9.0, 0.8, 2.0),
            "iron" => (9.0, 0.9, 3.0),
            "diamond" => (9.0, 1.0, 4.0),
            _ => (7.0, 1.0, 1.0),
        };
        tool(r, "axe", axe_damage, axe_speed);
        tool(r, "hoe", 1.0, hoe_speed);
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
            r.item(Item::generated(format!("{mat}_{piece}")))
                .durability(durability * dur)
                .set(&ARMOR, Armor { points, toughness })
                .tag(tag)
                .tag(&format!("{tag}/{mat}"))
                .wearable(crate::player::EQUIPMENT[slot as usize], inflate, &format!("armor/{mat}_{layer}"))
                .tab(&crate::tabs::COMBAT);
        }
    }
}

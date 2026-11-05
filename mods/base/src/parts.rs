//! Items built from parts (`notes/CRAFTING.md`; the rules are the `crafting` crate's):
//! - **Part items**, one per part shape, carrying their material, grade and wear (`PART`). They
//!   are made from the recipe book at a crafting table (`PART_RECIPES`, each recipe naming its
//!   material), where their grade is rolled.
//! - **Kind items** (an axe, a pickaxe ...), carrying their parts (`PARTS`) and the numbers those
//!   make on the stack itself (durability, mining speed, tool level, attack damage), which the
//!   engine prefers over the item's.
//! - **Assembly** at the crafting table: parts in the four assembly slots show the tool they make;
//!   taking it uses them. A tool in the "apart" slot comes back apart into its parts: lashings
//!   and glue are cut and lost, wedges and rivets come back.
//!
//! - **Armour** the same way: a shell, a fastening and a lining make a helmet, chestplate,
//!   leggings or boots (`ARMOUR_LOOKS` for how it's worn).
//! - **`/gear`** builds one (or one part) for checks and scripts.

use crafting::combine::{ArmourStats, Life, Part, ToolStats, armour, combine};
use crafting::gems::Stat;
use crafting::kinds::Use;
use crafting::materials::fits;
use crafting::{KINDS, Kind, MATERIALS, Role, SHAPES, code, grades, kind_of, material_of, shape_of};
use weft_sdk::*;

value! {
    /// A part item's own: its material, grade (1-10), the uses it has worn, and its history so
    /// far: the day it was made (`vitals::calendar`) and how often it's been repaired.
    pub struct PartOf { pub material: u32, pub grade: u32, pub worn: u32, pub made_day: u32, pub repairs: u32 }
}
value! {
    /// An item's parts, slot by slot (codes, `crafting::code`), as many as `n`.
    pub struct Parts {
        pub kind: u32,
        pub n: u32,
        pub slots: [u32; 4],
        pub shapes: [u32; 4],
        pub materials: [u32; 4],
        pub grades: [u32; 4],
        pub worn: [u32; 4],
        pub made_days: [u32; 4],
        pub repairs: [u32; 4],
        /// The item's grade: its working part's (for its tooltip).
        pub grade: u32,
    }
}
value! {
    /// A built armour piece's numbers from its parts (`crafting::combine::armour`): armour points
    /// and the health it adds over the base, engine points (`player::armor_of`).
    pub struct ArmourOf { pub armour: f32, pub health: f32 }
}
pub static PART: Component<PartOf> = Component::named("base:part");
pub static ARMOUR_OF: Component<ArmourOf> = Component::named("base:armour_of");
pub static PARTS: Component<Parts> = Component::named("base:parts");
/// The material a part recipe makes its part of (a recipe value).
pub static PART_MATERIAL: Component<u32> = Component::named("base:part_material");
pub static PART_RECIPES: RecipeKind = RecipeKind::named("base:parts");

/// The item of a part shape: `base:part_<shape>`.
fn part_item(shape: &str) -> &'static Item {
    Item::generated(format!("part_{shape}"))
}

/// What a material is made from here, as an ingredient (today's items; Content plan's will add
/// flint, sinew, bronze ...). Wooden heads are cut from planks; hafts, grips and shafts are split
/// from a log of their wood (`holding_source`).
const SOURCES: &[(&str, &str)] = &[
    ("oak", "#planks/oak"),
    ("birch", "#planks/birch"),
    ("spruce", "#planks/spruce"),
    ("mesquite", "#planks/mesquite"),
    ("stone", "#cobblestone"),
    ("bone", "bone"),
    ("wrought_iron", "#ingots/iron"),
    ("meteoric_iron", "#meteoric_iron"),
    ("star_steel", "#ingots/star_steel"),
    ("tektite", "#tektite"),
    ("gold", "#ingots/gold"),
    ("diamond", "#gems/diamond"),
    ("plant_cordage", "#string"),
    ("leather", "#materials/leather"),
    // Linings, as clothes are sewn (`clothing.rs`).
    ("linen", "#cloth"),
    ("wool", "wool"),
    ("fur", "#pelts"),
];

/// A holding part of wood is split from a log of it.
fn holding_source(material: &str) -> Option<String> {
    matches!(material, "oak" | "birch" | "spruce" | "mesquite" | "yew" | "elm" | "osage_orange").then(|| format!("#logs/{material}"))
}

/// How many of its material a shape takes.
fn units(shape: &str) -> u32 {
    match shape {
        "felling_axe_head" | "bearded_axe_head" | "pick_head" => 3,
        "chest_shell" | "leg_shell" => 3,
        "hoe_blade" | "straight_blade" | "long_blade" | "long_haft" | "shaft" | "self_stave" | "helmet_shell" | "boot_shell" => 2,
        _ => 1,
    }
}

/// Fittings of these are cut when a tool comes apart.
fn cut_when_apart(material: &str) -> bool {
    matches!(material, "plant_cordage" | "sinew" | "rawhide" | "birch_tar" | "leather")
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SKILLS,
        Skills { xp: [0; 6] },
        component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN,
    );
    r.attach(&PLAYER, &SKILLS);
    // What the held part or tool is made of and has been through, and the player's crafts.
    r.command("inspect", "", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        inspect(&w, player, |key, args| cmd.reply(key, args));
    })
    .permission(permission::EVERYONE);
    r.command("skills", "", |cmd| {
        let xp = cmd
            .world()
            .component(cmd.player(), &SKILLS)
            .map_or([0; 6], |s| s.xp);
        for (i, (_, key)) in crafting::skills::SKILLS.iter().enumerate() {
            let level = crafting::skills::level(xp[i]);
            cmd.reply("command.base.skills.line", &[Arg::Key(key), Arg::Int(level as i64), Arg::Int(xp[i] as i64)]);
        }
    })
    .permission(permission::EVERYONE);
    r.stack_component(&PART, 0);
    r.stack_component(&PARTS, 0);
    r.stack_component(&ARMOUR_OF, 0);
    // A built item's own item level and warmth, over its kind item's (the caps read the level:
    // `animals::armour`; Survival the warmth).
    r.stack_component(&base_api::ITEM_LEVEL, 0);
    r.stack_component(&base_api::WARMTH, 0);
    // A built piece's armour in the tooltip's and the HUD's points (`ARMOUR_PER_POINT`).
    r.stack_component(&crate::items::ARMOR, 0);
    r.recipe_kind(&PART_RECIPES).inputs(3, 1).outputs(1, 1);
    let mut made: Vec<&str> = Vec::new();
    for s in SHAPES {
        if made.contains(&s.id) {
            continue;
        }
        made.push(s.id);
        r.item(part_item(s.id))
            .max_stack(16)
            .tab(&crate::tabs::INGREDIENTS);
    }
    for k in KINDS.iter().filter(|k| k.use_ == Use::Guard) {
        register_armour(r, k);
    }
    for k in KINDS.iter().filter(|k| k.use_ != Use::Guard) {
        let tool = match k.id {
            "axe" | "pickaxe" | "shovel" | "hoe" | "sword" => Some(k.id),
            _ => None,
        };
        // Plain numbers here; an assembled stack carries its own.
        let b = r
            .item(Item::generated(k.id.to_string()))
            .durability(1)
            // Worn out, it stays: a part is repaired or replaced, not the tool lost.
            .keeps_broken()
            // Its kind's swings a second (`vitals::gear`).
            .attack_speed(vitals::gear::spread(k.id).map_or(1.0, |s| s.speed))
            .tab(if matches!(k.id, "sword" | "spear" | "knife" | "bow") { &crate::tabs::COMBAT } else { &crate::tabs::TOOLS });
        let b = match tool {
            Some(t) => b.tool(t),
            None => b,
        };
        // Vanilla: weapons wear by one per hit, tools (made for blocks) by two.
        let b = b.set(&WEAR_PER_HIT, if matches!(k.id, "sword" | "spear" | "knife") { 1 } else { 2 });
        if k.id == "sword" {
            b.sweeps().no_creative_break();
        } else if k.id == "hoe" {
            b.on_use(crate::farming::till);
        }
    }
    r.command("gear", "<kind:word> [materials:text]", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        let Some(kind) = cmd.text("kind").map(str::to_string) else { return };
        let materials: Vec<String> = cmd
            .text("materials")
            .unwrap_or("")
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let grade = materials
            .iter()
            .find_map(|m| m.strip_prefix("grade=").and_then(|g| g.parse().ok()))
            .unwrap_or(4u8);
        // `slot=N` puts it in that inventory slot (36-39 are armour's), replacing what's there.
        let slot = materials
            .iter()
            .find_map(|m| m.strip_prefix("slot=").and_then(|g| g.parse::<u32>().ok()));
        let names: Vec<&str> = materials
            .iter()
            .filter(|m| !m.contains('='))
            .map(String::as_str)
            .collect();
        let day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
        // `/gear part <shape> <material>`: one part, to assemble by hand.
        let made = if kind == "gem" {
            // `/gear gem <id>`: a freshly cut gem, rolled.
            names
                .first()
                .and_then(|id| crafting::gems::gem(id))
                .map_or(ItemStack::EMPTY, |g| {
                    crate::gems::stack_of(&crate::gems::cut(g, || (w.random() >> 40) as f32 / (1u64 << 24) as f32))
                })
        } else if kind == "part" {
            names
                .first()
                .zip(names.get(1))
                .map_or(ItemStack::EMPTY, |(shape, m)| part_of(shape, m, grade, day))
        } else {
            made_of(&kind, &names, grade, day)
        };
        if made.is_empty() {
            cmd.reply("command.base.gear.cant", &[Arg::Text(&kind)]);
            return;
        }
        match slot {
            Some(i) if i < w.inventory(player).len() => w.inventory(player).set(i, made),
            _ => give(&w, player, made),
        }
        cmd.reply("command.base.gear.made", &[Arg::Item(made.item)]);
    })
    .permission(permission::GAME_MASTER);
    // One recipe per shape and material that can be it, carrying the material.
    let mut shapes: Vec<&crafting::Shape> = Vec::new();
    for s in SHAPES {
        if !shapes.iter().any(|x| x.id == s.id) {
            shapes.push(s);
        }
    }
    for s in shapes {
        let role = KINDS
            .iter()
            .find(|k| k.id == s.kind)
            .and_then(|k| k.slots.iter().find(|x| x.name == s.slot))
            .map(|x| x.role)
            .unwrap_or(Role::Extra);
        for m in MATERIALS {
            // What the real thing could be made of (`crafting::materials::fits`).
            let Some(&(_, from)) = SOURCES.iter().find(|(id, _)| *id == m.id) else { continue };
            if !fits(m, s.kind, s.slot, role) {
                continue;
            }
            // Hafts, grips, shafts and bow staves are split from a log.
            let log = if role == Role::Holding || s.id.ends_with("_stave") { holding_source(m.id) } else { None };
            let from = log.as_deref().unwrap_or(from);
            let inputs: Vec<&str> = (0..units(s.id)).map(|_| from).collect();
            r.shapeless(&PART_RECIPES, &format!("part_{}", s.id), 1, &inputs, 0)
                .value(&PART_MATERIAL, code(m.id));
        }
    }
}

// ---------------------------------------------------------------- making parts

value! {
    /// A player's experience in each craft (`crafting::skills::SKILLS`), in parts made.
    pub struct Skills { pub xp: [u32; 6] }
}
pub static SKILLS: Component<Skills> = Component::named("base:craft_skills");

/// Grade points for a part made from the book (`crafting::grades`): the material's share, work
/// from memory by the maker's skill in its craft, and a crafting table's.
fn points(material: &str, skill: u32) -> grades::Points {
    let material = match material {
        "diamond" | "steel" | "star_steel" => 30,
        "flint" | "obsidian" | "wrought_iron" | "bronze" | "oak" | "mesquite" => 24,
        _ => 20,
    };
    grades::Points { material, work: grades::work_from_memory(skill), station: 10 }
}

/// A part recipe's result, made by `player`: its material from the recipe, its grade rolled with
/// `u` (0..1) by their skill in its craft, which the making then raises.
pub fn finish(w: &World, player: u64, r: &Recipe, out: ItemStack, u: f32) -> ItemStack {
    // A gem cut from its stone: rolled now.
    if let Some(gem) = crate::gems::finish(w, r, out) {
        return gem;
    }
    let Some(material) = r.value(&PART_MATERIAL) else { return out };
    let Some(m) = material_of(material) else { return out };
    let craft = crafting::skills::index(m.making);
    let xp = w.component(player, &SKILLS).map_or(0, |s| s.xp[craft]);
    let grade = grades::roll(points(m.id, crafting::skills::level(xp)), u);
    if let Some(s) = w.component_mut(player, &SKILLS) {
        s.xp[craft] = s.xp[craft].saturating_add(crafting::skills::xp_for(grade));
    }
    let made_day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    let out = out.with(&PART, PartOf { material, grade: grade as u32, worn: 0, made_day, repairs: 0 });
    drawn_part(out, m.id)
}

/// A loose part drawn in its material where its shape has a map (`crafting::looks::part_look`).
fn drawn_part(stack: ItemStack, material: &str) -> ItemStack {
    match shape_id(stack.item).and_then(|s| crafting::looks::part_look(s, material)) {
        Some(look) => with_look(stack, Some(look.as_str())),
        None => stack,
    }
}

/// The skill a part recipe uses, as (its name's key, the local player's level), for the book.
pub fn skill_of(view: &ClientView, r: &Recipe) -> Option<(&'static str, u32)> {
    let m = material_of(r.value(&PART_MATERIAL)?)?;
    let i = crafting::skills::index(m.making);
    let xp = view.component(&SKILLS).map_or(0, |s| s.xp[i]);
    Some((crafting::skills::SKILLS[i].1, crafting::skills::level(xp)))
}

/// A part recipe's material's name key.
pub fn material_key(r: &Recipe) -> Option<&'static str> {
    material_of(r.value(&PART_MATERIAL)?).map(|m| m.key)
}

// ---------------------------------------------------------------- assembly

/// The shape id a part item is (`base:part_<shape>`).
fn shape_id(item: u32) -> Option<&'static str> {
    let name = item_name(item);
    let id = name.strip_prefix("base:part_")?;
    SHAPES.iter().find(|s| s.id == id).map(|s| s.id)
}

/// The kind and its parts the stacks in `slots` build, if they build one.
fn build(stacks: &[ItemStack]) -> Option<(&'static Kind, Vec<(Part, PartOf)>)> {
    let parts: Vec<(&'static str, PartOf)> = stacks
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| Some((shape_id(s.item)?, PART.on_stack_own(s)?)))
        .collect::<Option<_>>()?;
    // The working part names the kind: the only one whose shape is a working slot of a kind.
    let kind = KINDS.iter().find(|k| {
        let working = k
            .slots
            .iter()
            .find(|x| x.role == Role::Working)
            .map(|x| x.name);
        parts.iter().any(|(id, _)| {
            SHAPES
                .iter()
                .any(|s| s.kind == k.id && s.id == *id && Some(s.slot) == working)
        })
    })?;
    if parts.len() != kind.slots.len() {
        return None;
    }
    let mut out = Vec::new();
    for slot in kind.slots {
        let (id, p) = parts.iter().find(|(id, _)| {
            SHAPES
                .iter()
                .any(|s| s.kind == kind.id && s.id == *id && s.slot == slot.name)
        })?;
        let m = material_of(p.material)?;
        let shape = SHAPES.iter().find(|s| s.kind == kind.id && s.id == *id)?;
        out.push((Part { slot: slot.name, shape: shape.id, material: m.id, grade: p.grade.clamp(1, 10) as u8 }, *p));
    }
    Some((kind, out))
}

/// Each wearing part's life in `kind` built from `parts`: a tool's or a piece of armour's.
fn lives(kind: &Kind, parts: &[Part]) -> Option<Vec<Life>> {
    if kind.use_ == Use::Guard { armour(kind.id, parts).ok().map(|a| a.life) } else { combine(kind.id, parts).ok().map(|t| t.life) }
}

/// The parts as the item carries them, and the item's wear so far: the largest share any wearing
/// part has used (each part keeps its own).
fn parts_of(kind: &Kind, parts: &[(Part, PartOf)], life: &[Life]) -> (Parts, f32) {
    let mut p = Parts {
        kind: code(kind.id),
        n: parts.len() as u32,
        slots: [0; 4],
        shapes: [0; 4],
        materials: [0; 4],
        grades: [0; 4],
        worn: [0; 4],
        made_days: [0; 4],
        repairs: [0; 4],
        grade: 0,
    };
    let working = kind.working();
    for (i, (part, of)) in parts.iter().enumerate().take(4) {
        p.slots[i] = code(part.slot);
        p.shapes[i] = code(part.shape);
        p.materials[i] = code(part.material);
        p.grades[i] = part.grade as u32;
        p.worn[i] = of.worn;
        p.made_days[i] = of.made_day;
        p.repairs[i] = of.repairs;
        if part.slot == working {
            p.grade = part.grade as u32;
        }
    }
    let used = parts
        .iter()
        .filter_map(|(part, worn)| {
            life.iter()
                .find(|l| l.slot == part.slot)
                .map(|l| worn.worn as f32 / l.uses.max(1.0))
        })
        .fold(0.0f32, f32::max);
    (p, used)
}

/// The tool `parts` make, with its numbers on the stack.
fn tool_stack(kind: &Kind, parts: &[(Part, PartOf)], stats: &ToolStats, gems: &[crate::gems::Set]) -> ItemStack {
    let (p, used) = parts_of(kind, parts, &stats.life);
    let (sockets, set, mult) = gems_for(kind, parts, gems);
    let uses = stats.weakest().map_or(1.0, |l| l.uses).max(1.0) as u32;
    // Its working part's material's item level (`vitals::gear`): what an over-tier cap reads.
    let level = parts
        .iter()
        .find(|(part, _)| part.slot == kind.working())
        .and_then(|(part, _)| vitals::gear::material(part.material))
        .map_or(0.0, |m| m.level());
    let stack = ItemStack::new(Item::generated(kind.id.to_string()).id(), 1)
        .with(&PARTS, p)
        .with(&DURABILITY, uses)
        .with(&MINING_SPEED, stats.stat * mult[Stat::Speed as usize])
        .with(&TOOL_LEVEL, stats.harvest)
        .with(&ATTACK_DAMAGE, stats.damage * mult[Stat::Damage as usize])
        .with(&base_api::ITEM_LEVEL, level)
        .with(&crate::gems::GEMS_ON, crate::gems::gems_of(sockets, &set));
    // Drawn from its parts (the engine's `core:look`), its gems set on it.
    let plain: Vec<Part> = parts.iter().map(|p| p.0).collect();
    let look = look_text(kind.id, &plain);
    // Gems over a drawn tool only (a kind with no layers shows its plain icon).
    let look = if look.is_empty() { look } else { look + &gem_layers(kind.id, &set) };
    let stack = with_look(stack, (!look.is_empty()).then_some(look.as_str()));
    let worn = (used * uses as f32) as u32;
    if worn > 0 { stack.with(&DAMAGE, worn) } else { stack }
}

/// How armour looks when worn, by its shell, until it has textures by part (`notes/CRAFTING.md`,
/// "How parts look": region maps per part): today's four worn sets. Its item is `<kind>` for a
/// leather look, else `<kind>_<look>`; the look families are items only so the worn texture can be
/// theirs, and none is made or found but by assembling.
pub const ARMOUR_LOOKS: [(&str, &str); 4] = [("leather", ""), ("iron", "_mail"), ("gold", "_scale"), ("diamond", "_gem")];

/// The worn set a shell of `material` looks like: bronze and copper the yellow one, the irons and
/// steels the grey one, diamond its own, hides and bone the leather one.
pub fn armour_look(material: &str) -> &'static str {
    match material {
        "copper" | "bronze" => "gold",
        "wrought_iron" | "meteoric_iron" | "steel" | "star_steel" => "iron",
        "diamond" => "diamond",
        _ => "leather",
    }
}

/// The item a piece of armour of kind `kind` with a shell of `material` is.
pub fn armour_item(kind: &str, material: &str) -> String {
    let look = armour_look(material);
    let suffix = ARMOUR_LOOKS
        .iter()
        .find(|(l, _)| *l == look)
        .map_or("", |l| l.1);
    format!("{kind}{suffix}")
}

/// An armour kind's items, one per look family: worn on its slot, worn out it stays (a part is
/// repaired or replaced).
fn register_armour(r: &mut Registry, k: &Kind) {
    let Some(slot) = k.armour_slot() else { return };
    // Leggings are the inner layer, drawn closer to the body.
    let (inflate, layer) = if slot == 2 { (0.5, 2) } else { (1.0, 1) };
    for (look, suffix) in ARMOUR_LOOKS {
        let b = r
            .item(Item::generated(format!("{}{suffix}", k.id)))
            // Drawn as today's piece of the set it looks like, until part art for armour (the
            // game-art sheet waiting on Ronald).
            .texture(&format!("{look}_{}", k.id))
            .durability(1)
            .keeps_broken()
            .set(&crate::items::ARMOR, crate::items::Armor { points: 0, toughness: 0.0 })
            .tag(crate::items::ARMOR_SLOTS[slot])
            .wearable(crate::player::EQUIPMENT[slot], inflate, &format!("armor/{look}_{layer}"));
        // Only the leather-look item shows in creative; the others are what assembly makes.
        if suffix.is_empty() {
            drop(b.tab(&crate::tabs::COMBAT));
        }
    }
}

/// A piece of armour's warmth from its parts: its shell's own insulation by the set it looks like,
/// and its lining's as Survival's clothing (`vitals::clothing::Worn::lining`).
pub fn armour_worn(worn: &mut vitals::clothing::Worn, a_shell: &str, lining: &str, slot: usize) {
    worn.armor(armour_look(a_shell), slot);
    let cloth = if lining == "leather" { "hide" } else { lining };
    if let Some(m) = vitals::clothing::find(cloth) {
        worn.lining(m, slot);
    }
}

/// The armour `parts` make, with its numbers on the stack.
fn armour_stack(kind: &Kind, parts: &[(Part, PartOf)], a: &ArmourStats, gems: &[crate::gems::Set]) -> ItemStack {
    let (p, used) = parts_of(kind, parts, &a.life);
    let (sockets, set, mult) = gems_for(kind, parts, gems);
    let (armour, health) = (a.armour * mult[Stat::Armour as usize], a.health * mult[Stat::Health as usize]);
    let uses = a.weakest().map_or(1.0, |l| l.uses).max(1.0) as u32;
    let mut worn = vitals::clothing::Worn::default();
    armour_worn(&mut worn, a.shell, a.lining, a.slot);
    let warmth = base_api::Warmth {
        cold: worn.cold.round().max(0.0) as u32,
        heat: worn.heat.round() as i32,
        wet: if worn.cold > 0.0 { (worn.cold_wet / worn.cold * 100.0).round() as u32 } else { 0 },
        wind: (worn.windproof * 100.0).round() as u32,
    };
    let stack = ItemStack::new(Item::generated(armour_item(kind.id, a.shell)).id(), 1)
        .with(&PARTS, p)
        .with(&DURABILITY, uses)
        .with(&ARMOUR_OF, ArmourOf { armour, health })
        .with(&crate::items::ARMOR, crate::items::Armor { points: (armour / ARMOUR_PER_POINT).round() as u32, toughness: 0.0 })
        .with(&base_api::ITEM_LEVEL, a.level)
        .with(&base_api::WARMTH, warmth)
        .with(&crate::gems::GEMS_ON, crate::gems::gems_of(sockets, &set));
    // Its icon is its shell's, in the shell's material (the worn look is its item's set).
    let shell = parts
        .iter()
        .find(|(p, _)| p.slot == "shell")
        .map(|(p, _)| p.shape);
    let stack = match shell.and_then(|s| crafting::looks::part_look(s, a.shell)) {
        Some(look) => with_look(stack, Some((look + &gem_layers(kind.id, &set)).as_str())),
        None => stack,
    };
    let worn = (used * uses as f32) as u32;
    if worn > 0 { stack.with(&DAMAGE, worn) } else { stack }
}

/// Armour points (`vitals::combat`) per point shown in the tooltip and the HUD (20 fill its row):
/// a Sound Iron Age set (`vitals::gear`, about 74 for wrought iron) shows about 15.
pub const ARMOUR_PER_POINT: f32 = 5.0;

/// Whether a stack is worn out (a kept-broken tool or armour piece: its damage reached its
/// durability).
pub fn broken(stack: &ItemStack) -> bool {
    let max = DURABILITY.on_stack(stack).unwrap_or(0);
    max > 0 && damage_of(stack) >= max
}

/// How a tool of `kind` built from `parts` is drawn (`crafting::looks`, the engine's `core:look`):
/// its layers back to front; the layer with the working part is that material's painted layer,
/// the others a shade map with one ramp per region. Empty for a kind with no layers yet.
pub fn look_text(kind: &str, parts: &[Part]) -> String {
    let Some(look) = crafting::looks::look(kind) else { return String::new() };
    let working = KINDS
        .iter()
        .find(|k| k.id == kind)
        .and_then(|k| k.slots.iter().find(|s| s.role == Role::Working))
        .map(|s| s.name);
    let material = |slot: &str| parts.iter().find(|p| p.slot == slot).map(|p| p.material);
    // Heads never move: the handle shifts to meet its head's anchor.
    let shift = working
        .and_then(material)
        .map_or([0, 0], |m| crafting::looks::shift(kind, m));
    look.layers
        .iter()
        .filter_map(|layer| {
            let at = if layer.moves && shift != [0, 0] { format!("+{},{}", shift[0], shift[1]) } else { String::new() };
            // The layer's drawing is named by the shape of its region-0 part.
            let region0 = *layer.regions.first()?;
            let first = parts.iter().find(|p| p.slot == region0)?;
            if layer.painted {
                return Some(crafting::looks::painted(kind, first.shape, first.material) + &at);
            }
            let ramps: Vec<String> = layer
                .regions
                .iter()
                .map(|r| material(r).map(crafting::looks::ramp))
                .collect::<Option<_>>()?;
            Some(format!("{}@{}{at}", crafting::looks::map(kind, first.shape), ramps.join(",")))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// What the assembly slots make, for the output slot (empty when nothing).
pub fn assembled(stacks: &[ItemStack]) -> ItemStack {
    let Some((kind, parts)) = build(stacks) else { return ItemStack::EMPTY };
    built(kind, &parts)
}

/// `kind` from `parts`, with its numbers on the stack; empty if they don't make one.
fn built(kind: &Kind, parts: &[(Part, PartOf)]) -> ItemStack {
    built_with(kind, parts, &[])
}

/// `kind` from `parts` with `gems` set in it (as many as it has sockets).
fn built_with(kind: &Kind, parts: &[(Part, PartOf)], gems: &[crate::gems::Set]) -> ItemStack {
    let plain: Vec<Part> = parts.iter().map(|p| p.0).collect();
    if kind.use_ == Use::Guard {
        return armour(kind.id, &plain).map_or(ItemStack::EMPTY, |a| armour_stack(kind, parts, &a, gems));
    }
    combine(kind.id, &plain).map_or(ItemStack::EMPTY, |stats| tool_stack(kind, parts, &stats, gems))
}

/// An item's sockets (`crafting::gems::sockets`, by its working part), the gems that fit in them,
/// and what they multiply (`crafting::gems::multipliers`: damage, speed, armour, health).
fn gems_for(kind: &Kind, parts: &[(Part, PartOf)], gems: &[crate::gems::Set]) -> (u8, Vec<crate::gems::Set>, [f32; 4]) {
    let Some((work, _)) = parts.iter().find(|(p, _)| p.slot == kind.working()) else { return (0, Vec::new(), [1.0; 4]) };
    let Some(m) = crafting::materials::material(work.material) else { return (0, Vec::new(), [1.0; 4]) };
    let sockets = crafting::gems::sockets(m, work.grade);
    let set: Vec<crate::gems::Set> = gems.iter().take(sockets as usize).copied().collect();
    let boosts: Vec<(vitals::web::Family, f32)> = set.iter().flat_map(|s| s.boosts()).collect();
    (sockets, set, crafting::gems::multipliers(kind, work.grade, &boosts))
}

/// The look layers of `gems` set in an item of `kind` (`crafting::looks::gem_look`), each after a
/// `;`; empty without gems.
fn gem_layers(kind: &str, gems: &[crate::gems::Set]) -> String {
    gems.iter()
        .enumerate()
        .filter_map(|(i, g)| crafting::looks::gem_look(kind, i, g.gem.id))
        .map(|l| format!(";{l}"))
        .collect()
}

/// A built item with its gems set again (its parts as they are now, so wear and repairs carry).
fn rebuilt(item: &ItemStack, gems: &[crate::gems::Set]) -> ItemStack {
    PARTS.on_stack_own(item).map_or(ItemStack::EMPTY, |_| {
        let parts = apart_into(item, false);
        build(&parts).map_or(ItemStack::EMPTY, |(kind, parts)| built_with(kind, &parts, gems))
    })
}

/// A built item and gems in the assembly slots: the item with them set, if it has room for all.
fn socketed(stacks: &[ItemStack]) -> ItemStack {
    let full: Vec<&ItemStack> = stacks.iter().filter(|s| !s.is_empty()).collect();
    let items: Vec<&&ItemStack> = full
        .iter()
        .filter(|s| PARTS.on_stack_own(s).is_some())
        .collect();
    let new: Vec<crate::gems::Set> = full.iter().filter_map(|s| crate::gems::loose(s)).collect();
    let ([item], false) = (&items[..], new.is_empty()) else { return ItemStack::EMPTY };
    if items.len() + new.len() != full.len() {
        return ItemStack::EMPTY;
    }
    let mut gems = crate::gems::on(item);
    if gems.len() + new.len() > sockets_of(item) as usize {
        return ItemStack::EMPTY;
    }
    gems.extend(new);
    rebuilt(item, &gems)
}

/// The sockets a built item has, from its working part's material and grade.
pub fn sockets_of(item: &ItemStack) -> u8 {
    let Some(p) = PARTS.on_stack_own(item) else { return 0 };
    let Some(kind) = kind_of(p.kind) else { return 0 };
    (0..p.n.min(4) as usize)
        .find(|&i| p.slots[i] == code(kind.working()))
        .and_then(|i| Some((material_of(p.materials[i])?, p.grades[i] as u8)))
        .map_or(0, |(m, g)| crafting::gems::sockets(m, g))
}

/// A built item with gems alone in the assembly slots: it bare (its gems come back on taking it).
fn unsocketed(stacks: &[ItemStack]) -> Option<(ItemStack, Vec<crate::gems::Set>)> {
    let full: Vec<&ItemStack> = stacks.iter().filter(|s| !s.is_empty()).collect();
    let [item] = full[..] else { return None };
    let gems = crate::gems::on(item);
    (!gems.is_empty()).then(|| (rebuilt(item, &[]), gems))
}

/// What a part of `kind`'s slot `slot` usually is when nothing names it (`/gear`):
/// an oak haft or grip, a leather wrap, a wrought-iron wedge, leather straps, a wool lining.
fn usual(kind: &Kind, role: Role, working: &str) -> &'static str {
    let metal = material_of(code(working))
        .is_some_and(|m| m.works == crafting::materials::Works::Metal || m.works == crafting::materials::Works::Gem);
    match role {
        Role::Holding if kind.id == "bow" => "leather",
        Role::Holding => "oak",
        Role::Fitting if kind.use_ == Use::Guard => "leather",
        Role::Fitting if kind.id == "bow" => "sinew",
        Role::Fitting if metal => "wrought_iron",
        Role::Fitting => "plant_cordage",
        Role::Extra if metal => "wrought_iron",
        Role::Extra => "oak",
        Role::Lining => "wool",
        Role::Working => "flint",
    }
}

/// `kind_id` built of `materials` (its slots' in order, the usual ones for those left out), every
/// part of grade `grade` and made on `day`: for `/gear` and scripts. Empty if they
/// don't make one.
pub fn made_of(kind_id: &str, materials: &[&str], grade: u8, day: u32) -> ItemStack {
    let Some(kind) = crafting::kinds::kind(kind_id) else { return ItemStack::EMPTY };
    let working = materials.first().copied().unwrap_or("flint");
    let mut parts = Vec::new();
    for (i, slot) in kind.slots.iter().enumerate() {
        let name = materials
            .get(i)
            .copied()
            .unwrap_or_else(|| usual(kind, slot.role, working));
        let Some(m) = crafting::materials::material(name) else { return ItemStack::EMPTY };
        let Some(shape) = crafting::kinds::shapes_for(kind.id, slot.name).next() else { return ItemStack::EMPTY };
        let part = Part { slot: slot.name, shape: shape.id, material: m.id, grade: grade.clamp(1, 10) };
        parts.push((part, PartOf { material: code(m.id), grade: part.grade as u32, worn: 0, made_day: day, repairs: 0 }));
    }
    built(kind, &parts)
}

/// A part item of shape `shape` and material `material`, of grade `grade`, made on `day`; empty
/// if that material can't be that part.
pub fn part_of(shape: &str, material: &str, grade: u8, day: u32) -> ItemStack {
    let Some(m) = crafting::materials::material(material) else { return ItemStack::EMPTY };
    let can = SHAPES.iter().filter(|s| s.id == shape).any(|s| {
        crafting::kinds::kind(s.kind)
            .and_then(|k| k.slots.iter().find(|x| x.name == s.slot))
            .is_some_and(|x| fits(m, s.kind, s.slot, x.role))
    });
    if !can {
        return ItemStack::EMPTY;
    }
    let part = ItemStack::new(part_item(shape).id(), 1)
        .with(&PART, PartOf { material: code(m.id), grade: grade.clamp(1, 10) as u32, worn: 0, made_day: day, repairs: 0 });
    drawn_part(part, m.id)
}

/// Puts `stack` where it goes: armour on its slot if that's free, else the selected hotbar slot,
/// else the bag; dropped at the feet if all are full.
pub fn give(w: &World, player: u64, stack: ItemStack) {
    let inv = w.inventory(player);
    if let Some(slot) = crate::items::ARMOR_SLOTS
        .iter()
        .position(|t| item_has_tag(stack.item, t))
        && let Some(worn) = inv.find_group(EQUIPMENT_GROUP)
        && worn.get(slot as u32).is_empty()
    {
        worn.set(slot as u32, stack);
        return;
    }
    let hotbar = inv.group("hotbar");
    let at = w.selected_slot(player);
    if hotbar.get(at).is_empty() {
        hotbar.set(at, stack);
        return;
    }
    for i in 0..inv.len() {
        if inv.get(i).is_empty() {
            inv.set(i, stack);
            return;
        }
    }
    if let Some(e) = w.entity(player) {
        w.spawn_item(e.pos, stack);
    }
}

/// A built item back into its parts, each with its share of the item's wear; fittings that are
/// cut come back as nothing.
pub fn apart(stack: &ItemStack) -> Vec<ItemStack> {
    apart_into(stack, true)
}

/// As `apart`; with `cut` false every part comes back (for a repair that puts it back together).
fn apart_into(stack: &ItemStack, cut: bool) -> Vec<ItemStack> {
    let Some(p) = PARTS.on_stack_own(stack) else { return Vec::new() };
    let Some(kind) = kind_of(p.kind) else { return Vec::new() };
    // The item's wear since it was put together goes to each part by its wear rate.
    let plain: Vec<Part> = (0..p.n.min(4) as usize)
        .filter_map(|i| {
            let slot = kind.slots.iter().find(|s| code(s.name) == p.slots[i])?;
            Some(Part {
                slot: slot.name,
                shape: shape_of(kind.id, p.shapes[i])?.id,
                material: material_of(p.materials[i])?.id,
                grade: p.grades[i] as u8,
            })
        })
        .collect();
    let life = lives(kind, &plain).unwrap_or_default();
    let max = DURABILITY.on_stack_own(stack).unwrap_or(1).max(1) as f32;
    let share = damage_of(stack) as f32 / max;
    let mut out = Vec::new();
    for (i, part) in plain.iter().enumerate() {
        let life = life
            .iter()
            .find(|l| l.slot == part.slot)
            .map_or(0.0, |l| l.uses);
        let role = kind
            .slots
            .iter()
            .find(|s| s.name == part.slot)
            .map(|s| s.role);
        if cut && role == Some(Role::Fitting) && cut_when_apart(part.material) {
            continue;
        }
        let worn = (p.worn[i] as f32).max(share * life) as u32;
        out.push(drawn_part(
            ItemStack::new(part_item(part.shape).id(), 1).with(
                &PART,
                PartOf { material: p.materials[i], grade: p.grades[i], worn, made_day: p.made_days[i], repairs: p.repairs[i] },
            ),
            part.material,
        ));
    }
    out
}

/// Uses a stack has worn (the engine's damage on it).
fn damage_of(stack: &ItemStack) -> u32 {
    DAMAGE.on_stack_own(stack).unwrap_or(0)
}

// ---------------------------------------------------------------- repair

/// Whether `item` is the material `material` is made from here (`SOURCES`).
fn is_source(material: &str, item: u32) -> bool {
    SOURCES
        .iter()
        .filter(|(id, _)| *id == material)
        .any(|(_, from)| match from.strip_prefix('#') {
            Some(tag) => item_has_tag(item, tag),
            None => item_name(item) == format!("base:{from}"),
        })
}

/// The role of part item `item` (its shape's slot, in the first kind that has it).
fn role_of_item(item: u32) -> Option<Role> {
    let id = shape_id(item)?;
    let s = SHAPES.iter().find(|s| s.id == id)?;
    KINDS
        .iter()
        .find(|k| k.id == s.kind)?
        .slots
        .iter()
        .find(|x| x.name == s.slot)
        .map(|x| x.role)
}

/// A worn part with one unit of its material: the part with `repair_share` of its life back.
fn repair_part(part: &ItemStack, with: &ItemStack) -> Option<ItemStack> {
    let p = PART.on_stack_own(part)?;
    let m = material_of(p.material)?;
    if p.worn == 0 || !is_source(m.id, with.item) {
        return None;
    }
    let life = crafting::combine::life_of(role_of_item(part.item)?, m.id, p.grade as u8);
    let back = (life * crafting::combine::repair_share(m.id)).ceil() as u32;
    Some(ItemStack { count: 1, ..*part }.with(&PART, PartOf { worn: p.worn.saturating_sub(back.max(1)), repairs: p.repairs + 1, ..p }))
}

/// A tool with one unit of a material: its most worn part of that material repaired, the tool
/// put back together.
fn repair_tool(tool: &ItemStack, with: &ItemStack) -> Option<ItemStack> {
    PARTS.on_stack_own(tool)?;
    let mut parts = apart_into(tool, false);
    let worst = parts
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let p = PART.on_stack_own(s)?;
            let m = material_of(p.material)?;
            let life = crafting::combine::life_of(role_of_item(s.item)?, m.id, p.grade as u8).max(1.0);
            (p.worn > 0 && is_source(m.id, with.item)).then_some((i, p.worn as f32 / life))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))?
        .0;
    parts[worst] = repair_part(&parts[worst], with)?;
    // Its gems stay set.
    let again = build(&parts).map_or(ItemStack::EMPTY, |(kind, p)| built_with(kind, &p, &crate::gems::on(tool)));
    (!again.is_empty()).then_some(again)
}

/// Two stacks in the assembly slots, a worn part or tool and a unit of its material: what the
/// repair makes.
fn repaired(stacks: &[ItemStack]) -> ItemStack {
    let full: Vec<&ItemStack> = stacks.iter().filter(|s| !s.is_empty()).collect();
    let [a, b] = full[..] else { return ItemStack::EMPTY };
    for (thing, with) in [(a, b), (b, a)] {
        if let Some(r) = repair_part(thing, with).or_else(|| repair_tool(thing, with)) {
            return r;
        }
    }
    ItemStack::EMPTY
}

// ---------------------------------------------------------------- the crafting table's slots

/// The table's parts slots changed: a tool in "apart" comes apart into empty assembly slots, and
/// "assembled" shows what the assembly slots make: a tool from parts, or a repair (a worn part
/// or tool and one unit of its material).
pub fn on_change(m: &MenuCtx) {
    let (apart_slot, out) = (m.group("apart"), m.group("assembled"));
    if m.group(ASSEMBLY[0]).is_empty() {
        return;
    }
    let held = apart_slot.get(0);
    if !held.is_empty() && PARTS.on_stack_own(&held).is_some() && (0..ASSEMBLY.len()).all(|i| asm_get(m, i).is_empty()) {
        for (i, part) in apart(&held).into_iter().enumerate().take(ASSEMBLY.len()) {
            asm_set(m, i, part);
        }
        // Its gems come back to the bag (a cheap one may crack).
        crate::gems::give_back(&m.world(), m.player(), &crate::gems::on(&held));
        apart_slot.set(0, ItemStack::EMPTY);
    }
    let stacks: Vec<ItemStack> = (0..ASSEMBLY.len()).map(|i| asm_get(m, i)).collect();
    // A tool from its parts, else a repair (a worn part or tool and a unit of its material), else
    // gems set in a built item, else a built item's gems taken out.
    let built = assembled(&stacks);
    let made = [built, repaired(&stacks), socketed(&stacks)]
        .into_iter()
        .find(|s| !s.is_empty())
        .or_else(|| unsocketed(&stacks).map(|u| u.0))
        .unwrap_or(ItemStack::EMPTY);
    out.set(0, made);
}

/// Something was taken: true if it was the assembled tool (its parts are used up), so the grid's
/// result isn't.
pub fn on_take(m: &MenuCtx) -> bool {
    if m.group("assembled").index_of(m.slot()).is_none() {
        return false;
    }
    // Smithing for the maker, by the tool its parts make (the result slot is already taken).
    let stacks: Vec<ItemStack> = (0..ASSEMBLY.len()).map(|i| asm_get(m, i)).collect();
    let parts = stacks.iter().filter(|s| !s.is_empty()).count() as u32;
    crate::skills::made(&m.world(), m.player(), &assembled(&stacks), parts);
    // Gems taken out of an item come back to the bag.
    if assembled(&stacks).is_empty()
        && repaired(&stacks).is_empty()
        && socketed(&stacks).is_empty()
        && let Some((_, gems)) = unsocketed(&stacks)
    {
        crate::gems::give_back(&m.world(), m.player(), &gems);
    }
    for i in 0..ASSEMBLY.len() {
        let s = asm_get(m, i);
        if !s.is_empty() {
            asm_set(m, i, if s.count > 1 { ItemStack { count: s.count - 1, ..s } } else { ItemStack::EMPTY });
        }
    }
    true
}

/// The table's four assembly slots, a group each so each shows its own ghost while empty (a
/// head, a haft, a fitting, the UI redo; parts go in any order).
pub const ASSEMBLY: [&str; 4] = ["assembly_head", "assembly_haft", "assembly_fitting", "assembly_more"];

fn asm_get(m: &MenuCtx, i: usize) -> ItemStack {
    m.group(ASSEMBLY[i]).get(0)
}

fn asm_set(m: &MenuCtx, i: usize, s: ItemStack) {
    m.group(ASSEMBLY[i]).set(0, s);
}

/// The table's parts row: four assembly slots, what they make, and the slot that takes a tool
/// apart.
pub fn ui(ui: &mut Ui) {
    ui.row(S::row().gap(4.0), |ui| {
        ui.text_styled(S::column().class("dim"), "crafting.parts.assemble", &[]);
        ui.row(S::row(), |ui| {
            for g in ASSEMBLY {
                ui.group(g);
            }
        });
        ui.recipe_arrow();
        ui.group("assembled");
        ui.spacer();
        ui.text_styled(S::column().class("dim"), "crafting.parts.apart", &[]);
        ui.group("apart");
    });
}

// ---------------------------------------------------------------- inspecting

/// Tells what the held stack is: a part's material, grade and history, or a tool's grade and each
/// of its parts, as chat lines (`/inspect`).
fn inspect(w: &World, player: u64, mut reply: impl FnMut(&str, &[Arg])) {
    let held = w
        .inventory(player)
        .group("hotbar")
        .get(w.selected_slot(player));
    let grade_key = |g: u32| grades::grade(g.clamp(1, 10) as u8).key;
    if let Some(p) = PART.on_stack_own(&held) {
        let m = material_of(p.material).map_or("", |m| m.key);
        reply(
            "command.base.inspect.part",
            &[
                Arg::Item(held.item),
                Arg::Key(m),
                Arg::Key(grade_key(p.grade)),
                Arg::Int(p.made_day as i64),
                Arg::Int(p.repairs as i64),
                Arg::Int(p.worn as i64),
            ],
        );
    } else if let Some(t) = PARTS.on_stack_own(&held) {
        reply("command.base.inspect.tool", &[Arg::Item(held.item), Arg::Key(grade_key(t.grade)), Arg::Int(t.n as i64)]);
        let kind = kind_of(t.kind);
        for i in 0..t.n.min(4) as usize {
            let shape = kind.and_then(|k| shape_of(k.id, t.shapes[i]));
            let Some(item) = shape.map(|s| part_item(s.id).id()) else { continue };
            let m = material_of(t.materials[i]).map_or("", |m| m.key);
            reply(
                "command.base.inspect.tool_part",
                &[
                    Arg::Item(item),
                    Arg::Key(m),
                    Arg::Key(grade_key(t.grades[i])),
                    Arg::Int(t.made_days[i] as i64),
                    Arg::Int(t.repairs[i] as i64),
                ],
            );
        }
        let gems = crate::gems::on(&held);
        reply("command.base.inspect.gems", &[Arg::Int(gems.len() as i64), Arg::Int(sockets_of(&held) as i64)]);
        for g in &gems {
            let pct: i64 = g
                .boosts()
                .iter()
                .map(|b| (b.1 * vitals::web::UNIT * 100.0).round() as i64)
                .sum();
            reply("command.base.inspect.gem", &[Arg::Key(g.gem.key), Arg::Int(g.data.level as i64), Arg::Int(pct)]);
        }
    } else {
        reply("command.base.inspect.plain", &[]);
    }
}

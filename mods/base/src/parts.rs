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
//! Until the look hook (`notes/CRAFTING.md`, "The engine hook for looks"), every part and kind
//! shows its plain icon.

use crafting::combine::{Part, ToolStats, combine};
use crafting::materials::Making;
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
pub static PART: Component<PartOf> = Component::named("base:part");
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
    ("gold", "#ingots/gold"),
    ("diamond", "#gems/diamond"),
    ("plant_cordage", "#string"),
    ("leather", "#materials/leather"),
];

/// A holding part of wood is split from a log of it.
fn holding_source(material: &str) -> Option<String> {
    matches!(material, "oak" | "birch" | "spruce" | "mesquite" | "yew" | "elm" | "osage_orange").then(|| format!("#logs/{material}"))
}

/// How many of its material a shape takes.
fn units(shape: &str) -> u32 {
    match shape {
        "felling_axe_head" | "bearded_axe_head" | "pick_head" => 3,
        "hoe_blade" | "straight_blade" | "long_blade" | "long_haft" | "shaft" | "self_stave" => 2,
        _ => 1,
    }
}

/// Fittings of these are cut when a tool comes apart.
fn cut_when_apart(material: &str) -> bool {
    matches!(material, "plant_cordage" | "sinew" | "rawhide" | "birch_tar" | "leather")
}

pub fn register(r: &mut Registry) {
    r.entity_component(&SKILLS, Skills { xp: [0; 6] }, component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP);
    r.attach(&PLAYER, &SKILLS);
    r.stack_component(&PART, 0);
    r.stack_component(&PARTS, 0);
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
    for k in KINDS {
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
            .tab(if matches!(k.id, "sword" | "spear" | "knife" | "bow") { &crate::tabs::COMBAT } else { &crate::tabs::TOOLS });
        let b = match tool {
            Some(t) => b.tool(t),
            None => b,
        };
        if k.id == "hoe" {
            b.on_use(crate::farming::till);
        }
    }
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
            // A guard or pommel is cast, forged or carved: not cordage or leather.
            let can = match role {
                Role::Extra => {
                    m.roles.contains(&Role::Working) || m.roles.contains(&Role::Fitting) && !matches!(m.making, Making::Hands | Making::Sew)
                }
                r => m.roles.contains(&r),
            };
            let Some(&(_, from)) = SOURCES.iter().find(|(id, _)| *id == m.id) else { continue };
            if !can {
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
        "diamond" | "steel" => 30,
        "flint" | "obsidian" | "wrought_iron" | "bronze" | "oak" | "mesquite" => 24,
        _ => 20,
    };
    grades::Points { material, work: grades::work_from_memory(skill), station: 10 }
}

/// A part recipe's result, made by `player`: its material from the recipe, its grade rolled with
/// `u` (0..1) by their skill in its craft, which the making then raises.
pub fn finish(w: &World, player: u64, r: &Recipe, out: ItemStack, u: f32) -> ItemStack {
    let Some(material) = r.value(&PART_MATERIAL) else { return out };
    let Some(m) = material_of(material) else { return out };
    let craft = crafting::skills::index(m.making);
    let xp = w.component(player, &SKILLS).map_or(0, |s| s.xp[craft]);
    let grade = grades::roll(points(m.id, crafting::skills::level(xp)), u);
    if let Some(s) = w.component_mut(player, &SKILLS) {
        s.xp[craft] = s.xp[craft].saturating_add(crafting::skills::xp_for(grade));
    }
    let made_day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    out.with(&PART, PartOf { material, grade: grade as u32, worn: 0, made_day, repairs: 0 })
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
        .map(|s| Some((shape_id(s.item)?, PART.stack_only(s)?)))
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

/// The tool `parts` make, with its numbers on the stack. Its wear so far is its most worn part's
/// share (each part keeps its own).
fn tool_stack(kind: &Kind, parts: &[(Part, PartOf)], stats: &ToolStats) -> ItemStack {
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
    let working = kind
        .slots
        .iter()
        .find(|s| s.role == Role::Working)
        .map(|s| s.name);
    for (i, (part, of)) in parts.iter().enumerate().take(4) {
        p.slots[i] = code(part.slot);
        p.shapes[i] = code(part.shape);
        p.materials[i] = code(part.material);
        p.grades[i] = part.grade as u32;
        p.worn[i] = of.worn;
        p.made_days[i] = of.made_day;
        p.repairs[i] = of.repairs;
        if Some(part.slot) == working {
            p.grade = part.grade as u32;
        }
    }
    let uses = stats.weakest().map_or(1.0, |l| l.uses).max(1.0) as u32;
    // Wear so far: the largest share any wearing part has used.
    let used = parts
        .iter()
        .filter_map(|(part, worn)| {
            stats
                .life
                .iter()
                .find(|l| l.slot == part.slot)
                .map(|l| worn.worn as f32 / l.uses.max(1.0))
        })
        .fold(0.0f32, f32::max);
    let stack = ItemStack::new(Item::generated(kind.id.to_string()).id(), 1)
        .with(&PARTS, p)
        .with(&DURABILITY, uses)
        .with(&MINING_SPEED, stats.stat)
        .with(&TOOL_LEVEL, stats.harvest)
        .with(&ATTACK_DAMAGE, stats.damage);
    // Drawn from its parts (the engine's `core:look`).
    let plain: Vec<Part> = parts.iter().map(|p| p.0).collect();
    let look = look_text(kind.id, &plain);
    let stack = with_look(stack, (!look.is_empty()).then_some(look.as_str()));
    let worn = (used * uses as f32) as u32;
    if worn > 0 { stack.with(&DAMAGE, worn) } else { stack }
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
    let plain: Vec<Part> = parts.iter().map(|p| p.0).collect();
    match combine(kind.id, &plain) {
        Ok(stats) => tool_stack(kind, &parts, &stats),
        Err(_) => ItemStack::EMPTY,
    }
}

/// A built item back into its parts, each with its share of the item's wear; fittings that are
/// cut come back as nothing.
pub fn apart(stack: &ItemStack) -> Vec<ItemStack> {
    apart_into(stack, true)
}

/// As `apart`; with `cut` false every part comes back (for a repair that puts it back together).
fn apart_into(stack: &ItemStack, cut: bool) -> Vec<ItemStack> {
    let Some(p) = PARTS.stack_only(stack) else { return Vec::new() };
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
    let stats = combine(kind.id, &plain).ok();
    let max = DURABILITY.stack_only(stack).unwrap_or(1).max(1) as f32;
    let share = damage_of(stack) as f32 / max;
    let mut out = Vec::new();
    for (i, part) in plain.iter().enumerate() {
        let life = stats
            .as_ref()
            .and_then(|s| s.life.iter().find(|l| l.slot == part.slot))
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
        out.push(
            ItemStack::new(part_item(part.shape).id(), 1).with(
                &PART,
                PartOf { material: p.materials[i], grade: p.grades[i], worn, made_day: p.made_days[i], repairs: p.repairs[i] },
            ),
        );
    }
    out
}

/// Uses a stack has worn (the engine's damage on it).
fn damage_of(stack: &ItemStack) -> u32 {
    DAMAGE.stack_only(stack).unwrap_or(0)
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
    let p = PART.stack_only(part)?;
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
    PARTS.stack_only(tool)?;
    let mut parts = apart_into(tool, false);
    let worst = parts
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let p = PART.stack_only(s)?;
            let m = material_of(p.material)?;
            let life = crafting::combine::life_of(role_of_item(s.item)?, m.id, p.grade as u8).max(1.0);
            (p.worn > 0 && is_source(m.id, with.item)).then_some((i, p.worn as f32 / life))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))?
        .0;
    parts[worst] = repair_part(&parts[worst], with)?;
    let again = assembled(&parts);
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
    let (asm, apart_slot, out) = (m.group("assembly"), m.group("apart"), m.group("assembled"));
    if asm.is_empty() {
        return;
    }
    let held = apart_slot.get(0);
    if !held.is_empty() && PARTS.stack_only(&held).is_some() && (0..asm.len()).all(|i| asm.get(i).is_empty()) {
        for (i, part) in apart(&held)
            .into_iter()
            .enumerate()
            .take(asm.len() as usize)
        {
            asm.set(i as u32, part);
        }
        apart_slot.set(0, ItemStack::EMPTY);
    }
    let stacks: Vec<ItemStack> = (0..asm.len()).map(|i| asm.get(i)).collect();
    // A tool from its parts, else a repair: a worn part or tool and a unit of its material.
    let built = assembled(&stacks);
    out.set(0, if built.is_empty() { repaired(&stacks) } else { built });
}

/// Something was taken: true if it was the assembled tool (its parts are used up), so the grid's
/// result isn't.
pub fn on_take(m: &MenuCtx) -> bool {
    if m.group("assembled").index_of(m.slot()).is_none() {
        return false;
    }
    let asm = m.group("assembly");
    for i in 0..asm.len() {
        let s = asm.get(i);
        if !s.is_empty() {
            asm.set(i, if s.count > 1 { ItemStack { count: s.count - 1, ..s } } else { ItemStack::EMPTY });
        }
    }
    true
}

/// The table's parts row: four assembly slots, what they make, and the slot that takes a tool
/// apart.
pub fn ui(ui: &mut Ui) {
    ui.row(S::row().gap(4.0), |ui| {
        ui.text_styled(S::col().class("dim"), "crafting.parts.assemble", &[]);
        ui.group("assembly");
        ui.recipe_arrow();
        ui.group("assembled");
        ui.spacer();
        ui.text_styled(S::col().class("dim"), "crafting.parts.apart", &[]);
        ui.group("apart");
    });
}

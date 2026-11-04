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
    /// A part item's own: its material, grade (1-10) and the uses it has worn.
    pub struct PartOf { pub material: u32, pub grade: u32, pub worn: u32 }
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
    matches!(material, "oak" | "birch" | "spruce" | "mesquite").then(|| format!("#logs/{material}"))
}

/// How many of its material a shape takes.
fn units(shape: &str) -> u32 {
    match shape {
        "felling_axe_head" | "bearded_axe_head" | "pick_head" => 3,
        "hoe_blade" | "straight_blade" | "long_blade" | "long_haft" | "shaft" => 2,
        _ => 1,
    }
}

/// Fittings of these are cut when a tool comes apart.
fn cut_when_apart(material: &str) -> bool {
    matches!(material, "plant_cordage" | "sinew" | "rawhide" | "birch_tar" | "leather")
}

pub fn register(r: &mut Registry) {
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
            .tab(if matches!(k.id, "sword" | "spear" | "knife") { &crate::tabs::COMBAT } else { &crate::tabs::TOOLS });
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
            let log = if role == Role::Holding { holding_source(m.id) } else { None };
            let from = log.as_deref().unwrap_or(from);
            let inputs: Vec<&str> = (0..units(s.id)).map(|_| from).collect();
            r.shapeless(&PART_RECIPES, &format!("part_{}", s.id), 1, &inputs, 0)
                .value(&PART_MATERIAL, code(m.id));
        }
    }
}

// ---------------------------------------------------------------- making parts

/// Grade points for a part made from the book (`crafting::grades`): the material's share, work
/// from memory (skills come later: a beginner's 10) and a crafting table's.
fn points(material: &str) -> grades::Points {
    let material = match material {
        "diamond" | "steel" => 30,
        "flint" | "obsidian" | "wrought_iron" | "bronze" | "oak" | "mesquite" => 24,
        _ => 20,
    };
    grades::Points { material, work: grades::work_from_memory(0), station: 10 }
}

/// A part recipe's result, made: its material from the recipe, its grade rolled with `u` (0..1).
pub fn finish(r: &Recipe, out: ItemStack, u: f32) -> ItemStack {
    let Some(material) = r.value(&PART_MATERIAL) else { return out };
    let id = material_of(material).map_or("", |m| m.id);
    let grade = grades::roll(points(id), u) as u32;
    out.with(&PART, PartOf { material, grade, worn: 0 })
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
fn build(stacks: &[ItemStack]) -> Option<(&'static Kind, Vec<(Part, u32)>)> {
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
        out.push((Part { slot: slot.name, shape: shape.id, material: m.id, grade: p.grade.clamp(1, 10) as u8 }, p.worn));
    }
    Some((kind, out))
}

/// The tool `parts` make, with its numbers on the stack. Its wear so far is its most worn part's
/// share (each part keeps its own).
fn tool_stack(kind: &Kind, parts: &[(Part, u32)], stats: &ToolStats) -> ItemStack {
    let mut p = Parts {
        kind: code(kind.id),
        n: parts.len() as u32,
        slots: [0; 4],
        shapes: [0; 4],
        materials: [0; 4],
        grades: [0; 4],
        worn: [0; 4],
    };
    for (i, (part, worn)) in parts.iter().enumerate().take(4) {
        p.slots[i] = code(part.slot);
        p.shapes[i] = code(part.shape);
        p.materials[i] = code(part.material);
        p.grades[i] = part.grade as u32;
        p.worn[i] = *worn;
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
                .map(|l| *worn as f32 / l.uses.max(1.0))
        })
        .fold(0.0f32, f32::max);
    let stack = ItemStack::new(Item::generated(kind.id.to_string()).id(), 1)
        .with(&PARTS, p)
        .with(&DURABILITY, uses)
        .with(&MINING_SPEED, stats.stat)
        .with(&TOOL_LEVEL, stats.harvest)
        .with(&ATTACK_DAMAGE, stats.damage);
    let worn = (used * uses as f32) as u32;
    if worn > 0 { stack.with(&DAMAGE, worn) } else { stack }
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
        if role == Some(Role::Fitting) && cut_when_apart(part.material) {
            continue;
        }
        let worn = (p.worn[i] as f32).max(share * life) as u32;
        out.push(ItemStack::new(part_item(part.shape).id(), 1).with(&PART, PartOf { material: p.materials[i], grade: p.grades[i], worn }));
    }
    out
}

/// Uses a stack has worn (the engine's damage on it).
fn damage_of(stack: &ItemStack) -> u32 {
    DAMAGE.stack_only(stack).unwrap_or(0)
}

// ---------------------------------------------------------------- the crafting table's slots

/// The table's parts slots changed: a tool in "apart" comes apart into empty assembly slots, and
/// "assembled" shows what the assembly slots make.
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
    out.set(0, assembled(&stacks));
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

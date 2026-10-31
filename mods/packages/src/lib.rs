//! Registers data-only content packages (chiacraft plan, phase 2; the format is
//! `chiacraft-packages`). Packages go through the same SDK calls a code mod makes, so a block from
//! a package behaves like one from a mod: same components, sounds, drops, tags and recipes.
//!
//! The engine gives every package its own mod slot (its namespace and `assets/`) and calls
//! [`chiacraft_register_package`] once per package after the code mods have registered. The mod's
//! own `init` registers one block: `packages:unknown`, what a saved block shows as while the mod or
//! package it came from is missing (the save keeps its real name).

use std::sync::Mutex;

use chiacraft_packages::{BlockDef, Drops, ItemDef, Package, RecipeDef, Render};
use weft_sdk::abi::{Host, ModId, Registrar, Str};
use weft_sdk::*;

declare_mod!(deps: [], init: init);

static UNKNOWN: Block = Block::named("unknown");

fn init(r: &mut Registry) {
    r.block(&UNKNOWN).hardness(0.5).drops_nothing().no_item();
}

/// The package being registered: `__init` takes a plain function.
static CURRENT: Mutex<Option<Package>> = Mutex::new(None);

/// Registers the package in folder `dir` into `reg`, the engine's slot `me` for it. Mistakes are
/// load errors. Returns nonzero if the registration panicked.
///
/// # Safety
/// The engine's host table and a live registrar, as for a mod's `init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chiacraft_register_package(host: *const Host, me: ModId, reg: *mut Registrar, dir: Str) -> u32 {
    let dir = std::path::PathBuf::from(unsafe { dir.as_str() });
    // Someone else's files must not take the game down, and a panic may not cross into the engine.
    let loaded = std::panic::catch_unwind(|| chiacraft_packages::load(&dir)).unwrap_or_else(|_| Err(vec!["reading the package failed".into()]));
    let pkg = match loaded {
        Ok(p) => p,
        Err(errors) => {
            for e in errors {
                unsafe { ((*host).reg_error)(reg, Str::new(&e)) };
            }
            return 0;
        }
    };
    let id: &'static str = Box::leak(pkg.id().to_string().into_boxed_str());
    *CURRENT.lock().unwrap() = Some(pkg);
    unsafe { weft_sdk::__init(host, me, reg, id, register_current) }
}

fn register_current(r: &mut Registry) {
    let Some(p) = CURRENT.lock().unwrap().take() else { return };
    for b in &p.blocks {
        block(r, &b.id, &b.def);
    }
    for i in &p.items {
        item(r, &i.id, &i.def);
    }
    for rc in &p.recipes {
        recipe(r, &rc.def);
    }
    for t in &p.tags {
        let names: Vec<&str> = t.def.entries.iter().map(String::as_str).collect();
        r.tag(&t.id, &names);
    }
}

/// `stone` is base's sound group; `othermod:stone` another mod's.
fn sound_group(group: &str) -> String {
    let (ns, g) = group.split_once(':').unwrap_or(("base", group));
    format!("{ns}:block.{g}")
}

fn block(r: &mut Registry, id: &str, d: &BlockDef) {
    let handle = Block::generated(id.to_string());
    let mut b = r.block(handle);
    if let Some(t) = &d.texture {
        b = b.texture(t);
    }
    if let Some(f) = &d.textures {
        let side = f.side.clone().unwrap_or_else(|| id.to_string());
        let pick = |face: &Option<String>| face.clone().unwrap_or_else(|| side.clone());
        // -X, +X, -Y, +Y, -Z, +Z
        let faces = [pick(&f.west), pick(&f.east), pick(&f.bottom), pick(&f.top), pick(&f.north), pick(&f.south)];
        b = b.faces(faces.each_ref().map(String::as_str));
    }
    if let Some(h) = d.hardness {
        b = b.hardness(h);
    }
    if d.unbreakable {
        b = b.unbreakable();
    }
    if let Some(l) = d.light {
        b = b.light(l);
    }
    if let Some(s) = &d.sound {
        b = b.sound_group(&sound_group(s));
    }
    if let Some(m) = &d.mineable {
        b = b.mineable(m);
    }
    if let Some(l) = d.tool_level {
        b = b.needs_tool(l);
    }
    match &d.drops {
        Some(Drops::Name(n)) if n == "nothing" => b = b.drops_nothing(),
        Some(Drops::Item { item, count }) => b = b.drops(item, *count),
        _ => {}
    }
    match d.render {
        Render::Solid => {}
        Render::Cutout => b = b.cutout(),
        Render::Translucent => b = b.translucent(),
    }
    if !d.solid {
        b = b.not_solid();
    }
    if d.climbable {
        b = b.climbable();
    }
    if let Some(f) = d.friction {
        b = b.friction(f);
    }
    for t in &d.tags {
        b = b.tag(t);
    }
    if !d.item {
        b = b.no_item();
    }
    if let Some(t) = &d.item_texture {
        b = b.item_texture(t);
    }
    if let Some(t) = &d.tab {
        b = b.tab(CreativeTab::generated(t.clone()));
    }
    drop(b);
}

fn item(r: &mut Registry, id: &str, d: &ItemDef) {
    let handle = Item::generated(id.to_string());
    let mut i = r.item(handle);
    if let Some(t) = &d.texture {
        i = i.texture(t);
    }
    if let Some(n) = d.max_stack {
        i = i.max_stack(n);
    }
    if let Some(n) = d.durability {
        i = i.durability(n);
    }
    if let Some(t) = &d.tool {
        i = i.tool(t);
    }
    if let Some(l) = d.tool_level {
        i = i.tool_level(l);
    }
    if let Some(s) = d.mining_speed {
        i = i.mining_speed(s);
    }
    if let Some(a) = d.attack_damage {
        i = i.attack_damage(a);
    }
    if let Some(b) = &d.places {
        i = i.placed_block(Block::generated(b.clone()));
    }
    if let Some(t) = &d.tab {
        i = i.tab(CreativeTab::generated(t.clone()));
    }
    drop(i);
    for t in &d.tags {
        r.tag(t, &[id]);
    }
    if let Some(ticks) = d.fuel {
        static FUEL: Component<u32> = Component::named("base:fuel");
        r.set(id, &FUEL, ticks);
    }
}

fn recipe(r: &mut Registry, d: &RecipeDef) {
    let kind = RecipeKind::generated(d.kind.clone());
    if d.pattern.is_empty() {
        let inputs: Vec<&str> = d.inputs.iter().map(String::as_str).collect();
        r.shapeless(kind, &d.result, d.count, &inputs, d.time);
    } else {
        let pattern: Vec<&str> = d.pattern.iter().map(String::as_str).collect();
        let keys: Vec<(char, &str)> = d.key.iter().filter_map(|(k, v)| Some((k.chars().next()?, v.as_str()))).collect();
        r.shaped(kind, &d.result, d.count, &pattern, &keys);
    }
}

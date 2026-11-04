//! The first hours (`notes/FIRST_HOURS.md`; the numbers are `vitals::first`'s). Bare hands can't
//! fell a tree, so the way in is off the ground:
//! - **Loose things** lie on the surface (`worldgen.rs` places them): a rock (a hammerstone), a
//!   flint nodule, a fallen branch. Picked up at once by hand; a rock or a nodule is put down again
//!   by using it.
//! - **Striking:** a rock used on a nodule (or a rock) lying on the ground strikes a sharp flake
//!   off it now and then; sneaking, it shapes the nodule into an axe head over a few strikes.
//! - **Cutting:** a flake cuts fibre plants for more fibre than pulling them by hand, and wears.
//! - **Leaves** broken by a bare hand give a stick now and then.
//! - **The first axe:** an axe head, a stick and two cordage in the 2 × 2 make a stone axe with
//!   its parts (`finish`): the head, a haft of the stick (`branch` wood), a cordage lashing.

use crafting::{code, grades, skills};
use vitals::first::{self as f, Ground};
use weft_sdk::*;

use crate::parts::{PART, PARTS, PartOf, SKILLS};

pub static ROCK: Item = Item::named("rock");
pub static FLINT: Item = Item::named("flint");
pub static SHARP_FLAKE: Item = Item::named("sharp_flake");
pub static LOOSE_ROCK: Block = Block::named("loose_rock");
pub static LOOSE_FLINT: Block = Block::named("loose_flint");
pub static FALLEN_BRANCH: Block = Block::named("fallen_branch");

/// The axe head knapped by hand, and the axe it makes.
const HEAD: &str = "base:part_felling_axe_head";
const AXE: &str = "base:axe";

static LEAVES: Tag = Tag::named("leaves");
/// Edges that cut fibre plants (a flake; a knife later).
pub static CUTTERS: Tag = Tag::named("cutters");
static STRIKE: Sound = Sound::named("block.stone.hit");
static CRACK: Sound = Sound::named("block.stone.break");

/// When each player last struck, so a quicker click only knocks (`f::STRIKE_SECS`).
static LAST_STRIKE: std::sync::Mutex<Vec<(u64, std::time::Instant)>> = std::sync::Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    // The loose things: small boxes lying on the ground, no collision, gone at a touch.
    let strikes = [("struck", f::FLAKES_PER_STONE - 1)];
    let shapes = [("struck", f::FLAKES_PER_STONE - 1), ("shaped", f::HEAD_STRIKES - 1)];
    loose(r, &LOOSE_ROCK, &strikes, "stone", "cobblestone", &[([4.0, 0.0, 5.0], [10.0, 3.0, 11.0]), ([10.0, 0.0, 8.0], [13.0, 1.5, 11.0])]);
    loose(r, &LOOSE_FLINT, &shapes, "stone", "flint_nodule", &[([5.0, 0.0, 5.0], [10.0, 3.0, 10.0]), ([9.0, 0.0, 9.0], [12.0, 2.0, 12.0])]);
    loose(r, &FALLEN_BRANCH, &[], "wood", "fallen_branch", &[([1.0, 0.0, 7.0], [15.0, 1.5, 8.5]), ([9.0, 0.0, 8.5], [12.0, 1.0, 12.0])]);
    r.loot(&LOOSE_ROCK).pool(Pool::new().item("rock"));
    r.loot(&LOOSE_FLINT).pool(Pool::new().item("flint"));
    r.loot(&FALLEN_BRANCH)
        .pool(Pool::new().item("stick"))
        .pool(Pool::new().when(Cond::Chance(f::BRANCH_SECOND_STICK)).item("stick"));

    r.item(&ROCK)
        .tag("rocks")
        .placed_block(&LOOSE_ROCK)
        .on_use(strike)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&FLINT)
        .tag("flint")
        .placed_block(&LOOSE_FLINT)
        .on_use(strike)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&SHARP_FLAKE)
        .tag("cutters")
        .handheld()
        .max_stack(1)
        .durability(f::FLAKE_USES)
        .tab(&crate::tabs::TOOLS);

    // The first axe, lashed by hand (`finish` gives it its parts).
    r.shapeless(&crate::recipes::CRAFTING, AXE, 1, &[HEAD, "#stick", "#string", "#string"], 0);

    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            if e.player == 0 || w.creative(e.player) {
                continue;
            }
            let full = block_name(e.state);
            let name = full.split('[').next().unwrap_or(full);
            let at = [e.pos[0] as f64 + 0.5, e.pos[1] as f64 + 0.2, e.pos[2] as f64 + 0.5];
            // A bare hand in leaves (not a player's own, placed ones): a stick now and then.
            if e.item == 0 && LEAVES.on_block(e.state) && !full.contains("placed=true") && unit(&w) < f::leaf_stick_extra() {
                w.spawn_item(at, ItemStack::new(crate::items::STICK.id(), 1));
            }
            // An edge on a fibre plant: more fibre, and the edge wears.
            if CUTTERS.on_item(e.item)
                && let Some(by_hand) = f::fibre_by_hand(name)
            {
                if unit(&w) < f::fibre_cut_extra(by_hand) {
                    w.spawn_item(at, ItemStack::new(crate::clothing::PLANT_FIBRE.id(), 1));
                }
                let hotbar = w.inventory(e.player).group("hotbar");
                let slot = w.selected_slot(e.player);
                let held = hotbar.get(slot);
                if held.item == e.item {
                    hotbar.set(slot, crate::felling::worn(held, 1, DURABILITY.on_item(held.item).unwrap_or(0)));
                }
            }
        }
    });
}

/// A loose thing's block: its counting props (0 to the most), then small boxes lying on the ground
/// with `tex` on them, no collision, broken at a touch, on something solid.
fn loose(r: &mut Registry, b: &'static Block, props: &[(&str, u32)], group: &str, tex: &str, boxes: &[([f32; 3], [f32; 3])]) {
    let mut bb = r.block(b);
    for &(name, most) in props {
        bb = bb.prop(name, 0..=most);
    }
    let mut bb = bb
        .sound_group(group)
        .texture(tex)
        .not_solid()
        .replaceable()
        .hardness(0.0)
        .needs(2, "")
        .no_item()
        .tab(&crate::tabs::NATURAL);
    for &(min, max) in boxes {
        bb = bb.model_box(min, max);
    }
    drop(bb);
}

/// A random number 0..1 from the world.
fn unit(w: &World) -> f32 {
    (w.random() >> 40) as f32 / (1u64 << 24) as f32
}

/// A rock or a nodule used on a loose rock or nodule on the ground: one strike (`vitals::first`).
/// Anything else it's used on, it's put down (its `placed_block`).
fn strike(u: &mut ItemUse) -> bool {
    let Some((p, _)) = u.target() else { return false };
    let w = u.world();
    let state = w.get(p);
    let (flint, rock) = (LOOSE_FLINT.is(state), LOOSE_ROCK.is(state));
    if !flint && !rock {
        return false;
    }
    let player = u.player();
    let at = [p[0] as f64 + 0.5, p[1] as f64 + 0.2, p[2] as f64 + 0.5];
    // Too soon after the last strike: a knock, nothing more.
    {
        let mut last = LAST_STRIKE.lock().unwrap();
        let now = std::time::Instant::now();
        match last.iter_mut().find(|(id, _)| *id == player) {
            Some((_, t)) if now.duration_since(*t).as_secs_f32() < f::STRIKE_SECS => {
                return true;
            }
            Some((_, t)) => *t = now,
            None => last.push((player, now)),
        }
    }
    w.play_sound(&STRIKE, at, 1.0);
    let int = |name: &str| w.prop(p, name).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    let sneaking = w.keys(player) & abi::held_keys::SNEAK != 0;
    if sneaking && flint {
        // Shaping a head: it may break into a flake; the last strike frees the head.
        if unit(&w) < f::HEAD_SPOIL {
            w.set(p, 0);
            w.spawn_item(at, ItemStack::new(SHARP_FLAKE.id(), 1));
            w.play_sound(&CRACK, at, 1.0);
            return true;
        }
        let shaped = int("shaped") + 1;
        if shaped >= f::HEAD_STRIKES {
            w.set(p, 0);
            w.spawn_item(at, head(&w, player));
            w.play_sound(&CRACK, at, 1.0);
        } else {
            w.set_prop(p, "shaped", &shaped.to_string());
        }
        return true;
    }
    let chance = if flint { f::FLAKE_FROM_FLINT } else { f::FLAKE_FROM_ROCK };
    if unit(&w) < chance {
        w.spawn_item(at, ItemStack::new(SHARP_FLAKE.id(), 1));
        w.play_sound(&CRACK, at, 0.8);
        let struck = int("struck") + 1;
        if struck >= f::FLAKES_PER_STONE {
            w.set(p, 0);
        } else {
            w.set_prop(p, "struck", &struck.to_string());
        }
    }
    true
}

/// A flint axe head knapped by `player`: its grade from flint, their knapping skill (which it
/// raises) and no station, as a part made by hand (`notes/CRAFTING.md`, the grades).
fn head(w: &World, player: u64) -> ItemStack {
    let craft = skills::index(crafting::materials::Making::Knap);
    let xp = w.component(player, &SKILLS).map_or(0, |s| s.xp[craft]);
    let points = grades::Points {
        material: f::HEAD_MATERIAL_POINTS,
        work: grades::work_from_memory(skills::level(xp)),
        station: 0,
    };
    let grade = grades::roll(points, unit(w));
    if let Some(s) = w.component_mut(player, &SKILLS) {
        s.xp[craft] = s.xp[craft].saturating_add(skills::xp_for(grade));
    }
    let made_day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    ItemStack::new(Item::generated(HEAD["base:".len()..].into()).id(), 1)
        .with(&PART, PartOf { material: code("flint"), grade: grade as u32, worn: 0, made_day, repairs: 0 })
}

/// A part of `shape` in `material` made by hand on `day`, at `f::HAND_PART_GRADE`.
fn hand_part(shape: &str, material: &str, day: u32) -> ItemStack {
    ItemStack::new(Item::generated(format!("part_{shape}")).id(), 1).with(
        &PART,
        PartOf { material: code(material), grade: f::HAND_PART_GRADE, worn: 0, made_day: day, repairs: 0 },
    )
}

/// What the grid's (or the book's) `made` really is, from the stacks it is made of: an axe lashed
/// by hand from a head gets its parts and the numbers they make. Anything else as it is.
pub fn finish(cells: &[ItemStack], made: ItemStack, day_time: u64) -> ItemStack {
    if made.is_empty() || item_name(made.item) != AXE || PARTS.stack_only(&made).is_some() {
        return made;
    }
    let Some(head) = cells.iter().find(|s| item_name(s.item) == HEAD) else { return made };
    if PART.stack_only(head).is_none() {
        return ItemStack::EMPTY;
    }
    let day = (day_time / vitals::calendar::DAY_TICKS) as u32;
    let head = ItemStack::new(head.item, 1).with(&PART, PART.stack_only(head).unwrap());
    crate::parts::assembled(&[head, hand_part("haft", "branch", day), hand_part("lashing", "plant_cordage", day)])
}

/// Which ground thing a column gets, for worldgen (`vitals::first::ground_at`): its block's id.
pub fn ground_block(g: Ground) -> u32 {
    match g {
        Ground::Rock => LOOSE_ROCK.id(),
        Ground::Flint => LOOSE_FLINT.id(),
        Ground::Branch => FALLEN_BRANCH.id(),
    }
}

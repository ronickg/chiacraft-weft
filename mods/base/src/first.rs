//! The first hours (`notes/FIRST_HOURS.md`; the numbers are `vitals::first`'s). Bare hands can't
//! fell a tree, so the way in is off the ground:
//! - **Loose things** lie on the surface (`worldgen.rs` places them): a rock (a hammerstone), a
//!   flint nodule, a fallen branch. Picked up at once by hand; a rock or a nodule is put down again
//!   by using it.
//! - **Striking:** a rock used on a nodule (or a rock) lying on the ground strikes a sharp flake
//!   off it now and then; sneaking, it opens the knapping screen on a nodule (`knapping.rs`).
//! - **Cutting:** a flake cuts fibre plants for more fibre than pulling them by hand, and wears.
//! - **Leaves** broken by a bare hand give a stick now and then.
//! - **Hafting by hand:** a knapped head, a stick and cordage in the 2 × 2 make a stone axe, pick or
//!   shovel with its parts (`finish`): the head, a haft of the stick (`branch` wood), a cordage
//!   lashing; a flake, a stick and cordage a knife.
//! - **Heads from the book:** once a player has held a head of a shape, flint makes it from the
//!   book or the grid, from memory (`locked`, `finish`).

use crafting::code;
use vitals::first::{self as f, Ground};
use weft_sdk::*;

use crate::parts::{PART, PARTS, PartOf};

pub static ROCK: Item = Item::named("rock");
pub static FLINT: Item = Item::named("flint");
pub static SHARP_FLAKE: Item = Item::named("sharp_flake");
pub static LOOSE_ROCK: Block = Block::named("loose_rock");
pub static LOOSE_FLINT: Block = Block::named("loose_flint");
pub static FALLEN_BRANCH: Block = Block::named("fallen_branch");

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
    loose(r, &LOOSE_ROCK, &strikes, "stone", "cobblestone", &[([3.0, 0.0, 3.0], [11.0, 4.0, 11.0]), ([9.0, 0.0, 8.0], [14.0, 2.5, 13.0])]);
    loose(
        r,
        &LOOSE_FLINT,
        &strikes,
        "stone",
        "flint_nodule",
        &[([4.0, 0.0, 4.0], [11.0, 4.0, 11.0]), ([9.0, 0.0, 9.0], [13.0, 2.5, 13.0])],
    );
    loose(r, &FALLEN_BRANCH, &[], "wood", "fallen_branch", &[([1.0, 0.0, 7.0], [15.0, 1.5, 8.5]), ([9.0, 0.0, 8.5], [12.0, 1.0, 12.0])]);
    r.loot(&LOOSE_ROCK).pool(Pool::new().item("rock"));
    r.loot(&LOOSE_FLINT).pool(Pool::new().item("flint"));
    r.loot(&FALLEN_BRANCH).pool(Pool::new().item("stick")).pool(
        Pool::new()
            .when(Cond::Chance(f::BRANCH_SECOND_STICK))
            .item("stick"),
    );

    // Not `placed_block`: the engine places an item's block before its use, so a rock used on a
    // nodule would replace it (PC run fh1_tools). `strike` puts them down itself.
    r.item(&ROCK)
        .tag("rocks")
        .on_use(strike)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&FLINT)
        .tag("flint")
        .on_use(strike)
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&SHARP_FLAKE)
        .tag("cutters")
        .handheld()
        .max_stack(1)
        .durability(f::FLAKE_USES)
        .tab(&crate::tabs::TOOLS);

    // Heads from memory, by the grid's pattern of nodules, and tools hafted by hand (`finish`
    // gives each its parts; `locked` keeps a head out of the book until one is made by hand).
    for k in f::KNAPPED {
        let rows: Vec<&str> = k.book.to_vec();
        r.shaped(&crate::recipes::CRAFTING, &format!("part_{}", k.shape), 1, &rows, &[('F', "flint")]);
    }
    for (kind, _, cordage) in f::HAFTED {
        let working = match f::KNAPPED.iter().find(|k| k.kind == kind) {
            Some(k) => format!("part_{}", k.shape),
            None => "sharp_flake".to_string(),
        };
        let mut inputs = vec![working.as_str(), "#stick"];
        inputs.extend(std::iter::repeat_n("#string", cordage as usize));
        r.shapeless(&crate::recipes::CRAFTING, kind, 1, &inputs, 0);
    }

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
            if (CUTTERS.on_item(e.item) || item_name(e.item) == "base:knife")
                && let Some(by_hand) = f::fibre_by_hand(name)
            {
                if unit(&w) < f::fibre_cut_extra(by_hand) {
                    w.spawn_item(at, ItemStack::new(crate::clothing::PLANT_FIBRE.id(), 1));
                }
                let hotbar = w.inventory(e.player).group("hotbar");
                let slot = w.selected_slot(e.player);
                let held = hotbar.get(slot);
                if held.item == e.item {
                    let max = DURABILITY.on_stack(&held).unwrap_or(0);
                    hotbar.set(slot, crate::felling::worn(held, 1, max));
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
/// Used on the top of anything else, it's put down there.
fn strike(u: &mut ItemUse) -> bool {
    let Some((aimed, face)) = u.target() else { return false };
    let w = u.world();
    // The stone itself, or the top of what it lies on: a small thing on the ground is easy to aim
    // past (PC run fh4_axe: the ray passed over the nodule to the ground under it).
    let above = [aimed[0], aimed[1] + 1, aimed[2]];
    let loose = |q: [i32; 3]| LOOSE_FLINT.is(w.get(q)) || LOOSE_ROCK.is(w.get(q));
    let p = if loose(aimed) {
        aimed
    } else if face == 3 && loose(above) {
        above
    } else {
        return put_down(u, aimed, face);
    };
    let state = w.get(p);
    let flint = LOOSE_FLINT.is(state);
    let player = u.player();
    let at = [p[0] as f64 + 0.5, p[1] as f64 + 0.2, p[2] as f64 + 0.5];
    // Sneaking: the key held, or crouched (the server has both).
    let sneaking = w.keys(player) & abi::held_keys::SNEAK != 0
        || w.entity(player)
            .is_some_and(|e| e.pose == abi::pose::CROUCH);
    weft_sdk::info!("first: strike at {p:?}, flint {flint}, sneaking {sneaking}");
    if sneaking && flint {
        crate::knapping::open(&w, player, p);
        return true;
    }
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
    let int = |name: &str| {
        w.prop(p, name)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0)
    };
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

/// A rock or a nodule put down on the top face of the block at `p` (face 3, +Y), where there's
/// room: its loose block, one fewer in hand.
fn put_down(u: &mut ItemUse, p: [i32; 3], face: u32) -> bool {
    let w = u.world();
    let at = [p[0], p[1] + 1, p[2]];
    if face != 3 || w.get(at) != 0 || state_flags(w.get(p)) & abi::block_flags::SOLID == 0 {
        return false;
    }
    let s = u.stack();
    let block = if s.item == FLINT.id() { LOOSE_FLINT.id() } else { LOOSE_ROCK.id() };
    w.set(at, block);
    if !w.creative(u.player()) {
        u.set_stack(StackData::shrink(s, 1));
    }
    true
}

/// A part of `shape` in `material` made by hand on `day`, at `f::HAND_PART_GRADE`.
fn hand_part(shape: &str, material: &str, day: u32) -> ItemStack {
    ItemStack::new(Item::generated(format!("part_{shape}")).id(), 1)
        .with(&PART, PartOf { material: code(material), grade: f::HAND_PART_GRADE, worn: 0, made_day: day, repairs: 0 })
}

/// Whether `player` may make the knapped head `r` makes from memory: once they've held one.
fn knows(w: &World, player: u64, item: u32) -> bool {
    w.creative(player)
        || w.component(player, &crate::discovery::KNOWN)
            .is_some_and(|k| crate::discovery::held(k, item_name(item)))
}

/// The knapped head a recipe's output is, if it is one.
fn knapped_of(item: u32) -> Option<&'static f::Knapped> {
    f::knapped(item_name(item).strip_prefix("base:part_")?)
}

/// Whether the book keeps `r` out of sight for a player who has held `touched`: a knapped head
/// from memory before a head of its shape has been made (held) by hand.
pub fn locked(r: &Recipe, touched: &[u32]) -> bool {
    r.kind() == crate::recipes::CRAFTING.name()
        && r.outputs()
            .first()
            .is_some_and(|(o, _)| knapped_of(o.item).is_some() && !touched.contains(&o.item))
}

/// The same on the server, before the book takes anything.
pub fn locked_for(w: &World, player: u64, r: &Recipe) -> bool {
    r.kind() == crate::recipes::CRAFTING.name()
        && r.outputs()
            .first()
            .is_some_and(|(o, _)| knapped_of(o.item).is_some() && !knows(w, player, o.item))
}

/// What the grid's (or the book's) `made` really is, from the stacks it is made of, for
/// `player`. A knapped head from memory gets its flint and grade (rolled with `roll`, the grid's
/// preview none), or nothing before one has been made by hand; a tool hafted by hand gets its
/// parts and the numbers they make. Anything else as it is.
pub fn finish(w: &World, player: u64, cells: &[ItemStack], made: ItemStack, roll: Option<f32>) -> ItemStack {
    if made.is_empty() || PART.stack_only(&made).is_some() || PARTS.stack_only(&made).is_some() {
        return made;
    }
    if let Some(k) = knapped_of(made.item) {
        if !knows(w, player, made.item) {
            return ItemStack::EMPTY;
        }
        return crate::knapping::knapped_head(w, player, k.shape, false, roll);
    }
    let kind = item_name(made.item).strip_prefix("base:").unwrap_or("");
    let Some(&(_, holding, _)) = f::HAFTED.iter().find(|h| h.0 == kind) else { return made };
    let day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    let working = match f::KNAPPED.iter().find(|k| k.kind == kind) {
        Some(k) => {
            let Some(head) = cells
                .iter()
                .find(|s| item_name(s.item) == format!("base:part_{}", k.shape))
            else {
                return made;
            };
            // A head with no part of its own (another mod's?) makes nothing.
            let Some(of) = PART.stack_only(head) else { return ItemStack::EMPTY };
            ItemStack::new(head.item, 1).with(&PART, of)
        }
        // A knife: a flake on a short grip.
        None if cells.iter().any(|s| s.item == SHARP_FLAKE.id()) => hand_part("knife_blade", "flint", day),
        None => return made,
    };
    crate::parts::assembled(&[working, hand_part(holding, "branch", day), hand_part("lashing", "plant_cordage", day)])
}

/// Which ground thing a column gets, for worldgen (`vitals::first::ground_at`): its block's id.
pub fn ground_block(g: Ground) -> u32 {
    match g {
        Ground::Rock => LOOSE_ROCK.id(),
        Ground::Flint => LOOSE_FLINT.id(),
        Ground::Branch => FALLEN_BRANCH.id(),
    }
}

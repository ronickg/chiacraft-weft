//! Trees (`notes/ECOLOGY.md`, part 3), for any mod: a log names its leaves (`TREE_LEAVES`); a
//! species with a recipe (`tree_recipes`, by the Animal models session) grows from branches
//! (`branchy`), the others by a crown shape their log's tag gives (`CONE_TREE`, `FLAT_TREE`,
//! `PALM_TREE`, `SLIM_TREE`; none: round). In the wild a few lie fallen or stand as stumps. Base's worldgen grows
//! them where a biome's `tree_kinds` names their logs, and a sapling's growth step grows one in a
//! live world (`grow_tree`), whichever mod's log it is.

use crate::branchy;
use weft_sdk::noise::hash;
use weft_sdk::*;

/// On a log, the leaves of the tree it grows into.
pub static TREE_LEAVES: Component<BlockRef> = Component::named("tree_leaves");
/// On a log, its thin branch block (the study, Ronald 11-05: "branches ... a bit like a fence"):
/// a tree grown from branches puts its wood under `BRANCH_UNDER` wide as this, a post 4 or 8 px
/// thick (`size`) with an arm to each wood neighbour (`n` `s` `e` `w` `u` `d`).
pub static TREE_BRANCH: Component<BlockRef> = Component::named("tree_branch");
/// Wood under this wide is a branch block: under `BRANCH_THIN` a 4 px one, else 8 px.
pub const BRANCH_UNDER: f32 = 0.75;
pub const BRANCH_THIN: f32 = 0.4;
/// The woods whose trees grow their thin wood as branch blocks (`base:<wood>_branch`, named on
/// the log as `TREE_BRANCH`): the broadleaf trees (Ronald, 11-05: "All broadleaf trees"); needles
/// and fronds hide a conifer's or a palm's wood, which keep their logs. Felling counts a branch
/// block a quarter of a log (`tree_recipes::felling_wood`).
pub const BRANCH_WOODS: &[&str] = &["oak", "birch", "acacia", "mesquite", "dogwood", "ipe", "andiroba", "matamata", "brazil_nut"];
/// Logs with this tag grow into cones (spruces), the others round (oaks, birches).
pub static CONE_TREE: Tag = Tag::named("cone_tree");
/// A tall bare trunk under one wide, flat crown (the acacia).
pub static FLAT_TREE: Tag = Tag::named("flat_tree");
/// A taller bare trunk under a crown of fronds that droop at their tips (palms, tree ferns).
pub static PALM_TREE: Tag = Tag::named("palm_tree");
/// A tall slim trunk under a narrow, long crown (birches).
pub static SLIM_TREE: Tag = Tag::named("slim_tree");

/// Leaves that turn differently in autumn (`vitals::wild::Turn`): evergreens never, birch-like
/// trees yellow; the rest keep `tints::FOLIAGE`. Each biome gives all three its foliage colour, and
/// the lighting session's seasonal tints change them through the year.
pub static EVERGREEN: TintKind = TintKind::named("base:evergreen");
pub static YELLOW_LEAVES: TintKind = TintKind::named("base:yellow_leaves");

/// The tint kind for leaves that turn so (`vitals::wild::turn`).
pub fn leaf_tint(turn: vitals::wild::Turn) -> &'static TintKind {
    match turn {
        vitals::wild::Turn::Evergreen => &EVERGREEN,
        vitals::wild::Turn::Yellow => &YELLOW_LEAVES,
        vitals::wild::Turn::Orange => &tints::FOLIAGE,
    }
}

static DIRT: Block = Block::named("base:dirt");
static TALL_GRASS: Block = Block::named("base:tall_grass");
/// A shelf fungus on dead wood, on a trunk's north side (base's `bracket_fungus`).
static BRACKET_FUNGUS: Block = Block::named("base:bracket_fungus");

/// A tree's crown, by its log's tag.
#[derive(Clone, Copy, PartialEq)]
enum Crown {
    Round,
    Cone,
    Flat,
    Palm,
    Slim,
}

/// How this one grew (`h` picks it): most stand straight; some grew big, crooked or as a twin pair;
/// in the wild a few lie fallen or stand as a stump (no leaves).
#[derive(Clone, Copy, PartialEq)]
enum Form {
    Standing,
    Big,
    Crooked,
    Twin,
    Fallen,
    Stump,
    /// A standing dead trunk, bare (the taiga's fire-killed spruces, a wood's old snags).
    Snag,
}

pub struct TreeAt {
    x: i32,
    y: i32,
    z: i32,
    log: u32,
    leaves: u32,
    crown: Crown,
    form: Form,
    height: i32,
    /// The way a crooked trunk bends, a twin stands, a fallen one lies.
    lean: (i32, i32),
    /// Its seed, for a tree grown from branches (`branchy`).
    seed: u64,
}

const LEANS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// A tree growing from `log` (its `tree` component), `h` deciding its height and form: a sapling's,
/// always standing.
pub fn tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    shaped(log, x, y, z, h, false)
}

/// A tree the world grows, which may also lie fallen or stand as a stump (forest trees only).
pub fn wild_tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    shaped(log, x, y, z, h, true)
}

fn shaped(log: u32, x: i32, y: i32, z: i32, h: u64, wild: bool) -> Option<TreeAt> {
    let leaves = TREE_LEAVES.on_block(log)?.0;
    let crown = if PALM_TREE.on_block(log) {
        Crown::Palm
    } else if FLAT_TREE.on_block(log) {
        Crown::Flat
    } else if CONE_TREE.on_block(log) {
        Crown::Cone
    } else if SLIM_TREE.on_block(log) {
        Crown::Slim
    } else {
        Crown::Round
    };
    let roll = (h >> 8) % 100;
    let forest = matches!(crown, Crown::Round | Crown::Cone | Crown::Slim);
    let form = match roll {
        0..4 if wild && forest => Form::Fallen,
        4..8 if wild && forest => Form::Stump,
        42..45 if wild && forest => Form::Snag,
        8..22 if crown != Crown::Palm => Form::Big,
        22..34 if crown != Crown::Cone => Form::Crooked,
        34..42 if crown == Crown::Round => Form::Twin,
        _ => Form::Standing,
    };
    let big = (form == Form::Big) as i32;
    let height = match crown {
        // Spruces stand tall and narrow, the big ones half again as tall.
        Crown::Cone => 6 + (h % 4) as i32 + 3 * big,
        Crown::Palm => 7 + (h % 3) as i32,
        Crown::Flat => 5 + (h % 2) as i32 + big,
        Crown::Round => 5 + (h % 3) as i32 + 2 * big,
        Crown::Slim => 6 + (h % 3) as i32 + 2 * big,
    };
    let lean = LEANS[(h >> 20) as usize % 4];
    Some(TreeAt { x, y, z, log, leaves, crown, form, height, lean, seed: h })
}

/// Calls `put(x, y, z, state, force)` for every block of the tree.
pub fn tree_blocks(t: &TreeAt, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let log = t.log;
    match t.form {
        // A trunk lying on the ground along its lean, 3 to 5 long, its log turned on its side.
        Form::Fallen => {
            let axis = if t.lean.0 != 0 { "x" } else { "z" };
            let lying = state_with(log, "axis", axis).unwrap_or(log);
            for k in 0..3 + t.height % 3 {
                put(t.x + t.lean.0 * k, t.y, t.z + t.lean.1 * k, lying, false);
            }
            // Shelf fungus on its north side, on half of them.
            if t.seed >> 30 & 1 == 0 {
                put(t.x + t.lean.0, t.y, t.z + t.lean.1 - 1, BRACKET_FUNGUS.id(), false);
            }
            return;
        }
        // A bare trunk three to six high, a stub of a branch near its top.
        Form::Snag => {
            let top = t.y + 3 + t.height % 4;
            for y in t.y..top {
                put(t.x, y, t.z, log, true);
            }
            let axis = if t.lean.0 != 0 { "x" } else { "z" };
            put(t.x + t.lean.0, top - 2, t.z + t.lean.1, state_with(log, "axis", axis).unwrap_or(log), false);
            // Shelf fungus rotting it, on its north side, on half of them.
            if t.seed >> 30 & 1 == 0 {
                put(t.x, t.y + 1 + (t.seed >> 31 & 1) as i32, t.z - 1, BRACKET_FUNGUS.id(), false);
            }
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        Form::Stump => {
            for y in t.y..t.y + 1 + t.height % 2 {
                put(t.x, y, t.z, log, true);
            }
            if t.seed >> 30 & 1 == 0 {
                put(t.x, t.y, t.z - 1, BRACKET_FUNGUS.id(), false);
            }
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        _ => {}
    }
    // A species with a recipe grows from branches (`branchy`), its log turned along each limb. A
    // giant's species stands here as its young one, the world's as a sapling's: the grown giants
    // are the giant scatter's (`giant_tree_blocks`), too tall for a tree's site.
    let name = block_name(log).split('[').next().unwrap_or("");
    // A broad-crowned species grows by space colonization (`colonize`, Ronald's hybrid, 11-05).
    // (A giant's species in a tree's site grows its young recipe, below: the grown giant is the
    // giant scatter's.)
    let giant_young = crate::tree_recipes::YOUNG.iter().any(|y| y.log == name);
    if let Some(c) = crate::tree_recipes::colonized(name).filter(|_| !giant_young) {
        if let Some(branch) = TREE_BRANCH.on_block(log).map(|b| b.0) {
            put_with_branches(&crate::colonize::blocks(c, t.seed), None, log, t.leaves, branch, t.x, t.y, t.z, &mut put);
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        for (dx, dy, dz, part, _) in crate::colonize::blocks(c, t.seed) {
            let (x, y, z) = (t.x + dx, t.y + dy, t.z + dz);
            match part {
                branchy::Part::Log { axis } => {
                    let along = ["x", "y", "z"][axis as usize];
                    put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
                }
                branchy::Part::Leaves => put(x, y, z, t.leaves, false),
            }
        }
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    let recipe = crate::tree_recipes::sapling_of(name);
    if let (Some(recipe), Some(branch)) = (recipe, TREE_BRANCH.on_block(log).map(|b| b.0)) {
        grow_with_branches(recipe, log, t.leaves, branch, t.x, t.y, t.z, t.seed, &mut put);
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    if let Some(recipe) = recipe {
        branchy::grow(recipe, t.x, t.y, t.z, t.seed, |x, y, z, part| match part {
            branchy::Part::Log { axis } => {
                let along = ["x", "y", "z"][axis as usize];
                put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
            }
            branchy::Part::Leaves => put(x, y, z, t.leaves, false),
        });
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    let top = t.y + t.height;
    // A crooked trunk steps aside once, two or three blocks under its top, and its crown with it.
    let bend = if t.form == Form::Crooked { top - 2 - t.height % 2 } else { i32::MAX };
    let (cx, cz) = if t.form == Form::Crooked { (t.x + t.lean.0, t.z + t.lean.1) } else { (t.x, t.z) };
    crown(t, cx, cz, top, &mut put);
    for y in t.y..top {
        let (x, z) = if y >= bend { (cx, cz) } else { (t.x, t.z) };
        put(x, y, z, log, true);
    }
    // A twin: a second, shorter trunk beside the first, with its own crown.
    if t.form == Form::Twin {
        let (sx, sz) = (t.x + t.lean.0 + t.lean.1, t.z + t.lean.1 - t.lean.0);
        let second = top - 1 - t.height % 2;
        crown(t, sx, sz, second, &mut put);
        for y in t.y..second {
            put(sx, y, sz, log, true);
        }
    }
    put(t.x, t.y - 1, t.z, DIRT.id(), true);
}

/// The leaves of a crown whose trunk tops out at `top`, over (`cx`, `cz`).
fn crown(t: &TreeAt, cx: i32, cz: i32, top: i32, put: &mut impl FnMut(i32, i32, i32, u32, bool)) {
    let leaves = t.leaves;
    match t.crown {
        Crown::Palm => {
            // A tuft on top, and fronds out four blocks each way that droop as they go, their
            // corners shorter.
            put(cx, top, cz, leaves, false);
            put(cx, top + 1, cz, leaves, false);
            for (dx, dz) in LEANS {
                for k in 1..=4 {
                    put(cx + dx * k, top + 1 - k.min(3), cz + dz * k, leaves, false);
                }
            }
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                for k in 1..=2 {
                    put(cx + dx * k, top + 1 - k, cz + dz * k, leaves, false);
                }
            }
        }
        Crown::Flat => {
            // One wide flat layer on the trunk's top, a small one over its middle; a big one wider.
            let wide = if t.form == Form::Big { 13 } else { 10 };
            for (y, r2) in [(top, wide), (top + 1, 2)] {
                for dz in -4i32..=4 {
                    for dx in -4i32..=4 {
                        let d = dx * dx + dz * dz;
                        if d <= r2 && (d < r2 || hash(1, (cx + dx) as i64, y as i64, (cz + dz) as i64) & 1 == 0) {
                            put(cx + dx, y, cz + dz, leaves, false);
                        }
                    }
                }
            }
        }
        Crown::Cone => {
            // Narrow rings stepping in toward a spire, from two blocks up the trunk.
            for dy in 2..=t.height + 1 {
                let y = t.y + dy;
                let r = if dy == t.height + 1 { 0 } else { 1 + ((t.height + 1 - dy) % 3).min(2) };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz <= r * r + 1 {
                            put(cx + dx, y, cz + dz, leaves, false);
                        }
                    }
                }
            }
        }
        Crown::Slim => {
            // A narrow crown down the top half of the trunk, widest in its middle.
            for y in top - 4..=top {
                let r: i32 = if y == top || y == top - 4 { 1 } else { 2 };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() == r && dz.abs() == r {
                            continue;
                        }
                        put(cx + dx, y, cz + dz, leaves, false);
                    }
                }
            }
        }
        Crown::Round => {
            // Wide layers under two narrower ones, corners ragged: a broadleaf's crown about seven
            // blocks across, a big one nine and a layer deeper (real mature crowns are 6-12 m).
            let wide = if t.form == Form::Big { 4 } else { 3 };
            let depth = 3 + (t.form == Form::Big) as i32;
            for y in top - depth..=top {
                let r: i32 = if y >= top - 1 { wide - 1 } else { wide };
                for dz in -r..=r {
                    for dx in -r..=r {
                        let corner = dx.abs() == r && dz.abs() == r;
                        if corner && (y == top || hash(0, (cx + dx) as i64, y as i64, (cz + dz) as i64) & 1 == 0) {
                            continue;
                        }
                        put(cx + dx, y, cz + dz, leaves, false);
                    }
                }
            }
        }
    }
}

/// A grown giant of `log` (its recipe's full size, `tree_recipes::of`; a sequoia's 60 blocks), for
/// a scatter of its own: nothing for a log without a recipe. Calls `put(x, y, z, state, force)`.
pub fn giant_tree_blocks(log: u32, x: i32, y: i32, z: i32, h: u64, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let Some(leaves) = TREE_LEAVES.on_block(log).map(|l| l.0) else { return };
    let name = block_name(log).split('[').next().unwrap_or("");
    // A colonized giant (the Brazil nut): its trunk from its recipe, its crown by colonization.
    if let Some(c) = crate::tree_recipes::colonized(name) {
        let bark = state_with(log, "axis", "bark");
        if let Some(branch) = TREE_BRANCH.on_block(log).map(|b| b.0) {
            put_with_branches(&crate::colonize::blocks(c, h), Some(bark.unwrap_or(log)), log, leaves, branch, x, y, z, &mut put);
            put(x, y - 1, z, DIRT.id(), true);
            return;
        }
        for (dx, dy, dz, part, _) in crate::colonize::blocks(c, h) {
            let (x, y, z) = (x + dx, y + dy, z + dz);
            match part {
                branchy::Part::Log { axis: 1 } => put(x, y, z, bark.unwrap_or(log), true),
                branchy::Part::Log { axis } => {
                    let along = ["x", "y", "z"][axis as usize];
                    put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
                }
                branchy::Part::Leaves => put(x, y, z, leaves, false),
            }
        }
        put(x, y - 1, z, DIRT.id(), true);
        return;
    }
    let Some(recipe) = crate::tree_recipes::of(name) else { return };
    // The upright trunk in bark all round: its flared foot spreads out at the ground, where an
    // upright log's top would show its rings (the PC look, 11-04).
    let bark = state_with(log, "axis", "bark");
    branchy::grow(recipe, x, y, z, h, |x, y, z, part| match part {
        branchy::Part::Log { axis: 1 } => put(x, y, z, bark.unwrap_or(log), true),
        branchy::Part::Log { axis } => {
            let along = ["x", "y", "z"][axis as usize];
            put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
        }
        branchy::Part::Leaves => put(x, y, z, leaves, false),
    });
    put(x, y - 1, z, DIRT.id(), true);
}

/// A young tree of `log` (the understory: in clearings and at a wood's edge), `h` its seed: a
/// species with a recipe grows a small one of its own shape (`tree_recipes::understory`, within
/// `UNDERSTORY_REACH`); the rest a trunk two to four high under a small crown of its leaves, a
/// spire for the cone trees, a ragged ball for the rest. None for palms and flat-topped trees, which grow up in the open.
/// Calls `put(x, y, z, state, force)` for every block.
pub fn young_tree_blocks(log: u32, x: i32, y: i32, z: i32, h: u64, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let Some(leaves) = TREE_LEAVES.on_block(log).map(|l| l.0) else { return };
    if PALM_TREE.on_block(log) || FLAT_TREE.on_block(log) {
        return;
    }
    // A species with a recipe grows its young from it, small (`tree_recipes::understory`), so a
    // young oak reads as an oak and a young birch as a birch.
    let name = block_name(log).split('[').next().unwrap_or("");
    if let Some(recipe) = crate::tree_recipes::of(name) {
        let young = crate::tree_recipes::understory(recipe);
        branchy::grow(&young, x, y, z, h, |x, y, z, part| match part {
            branchy::Part::Log { axis } => {
                let along = ["x", "y", "z"][axis as usize];
                put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
            }
            branchy::Part::Leaves => put(x, y, z, leaves, false),
        });
        return;
    }
    let height = 2 + (h % 3) as i32;
    let top = y + height;
    if CONE_TREE.on_block(log) {
        // Rings of one round the trunk from its first block up, and a tip on top.
        for k in 1..height {
            for (dx, dz) in LEANS {
                if k % 2 == 1 || hash(h, dx as i64, k as i64, dz as i64) & 1 == 0 {
                    put(x + dx, y + k, z + dz, leaves, false);
                }
            }
        }
        put(x, top, z, leaves, false);
        put(x, top + 1, z, leaves, false);
    } else {
        for cy in top - 1..=top {
            for dz in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx.abs() + dz.abs() == 2 && hash(h, dx as i64, cy as i64, dz as i64) & 3 != 0 {
                        continue;
                    }
                    put(x + dx, cy, z + dz, leaves, false);
                }
            }
        }
        put(x, top + 1, z, leaves, false);
    }
    for k in 0..height {
        put(x, y + k, z, log, true);
    }
}

/// Places a tree of `log` into a live world (saplings). Checks for room first.
pub fn grow_tree(w: World, p: [i32; 3], log: u32) {
    let Some(t) = tree_at(log, p[0], p[1], p[2], w.random()) else { return };
    for y in t.y + 1..t.y + t.height {
        if w.block([t.x, y, t.z]) != 0 {
            return;
        }
    }
    tree_blocks(&t, |x, y, z, s, overwrite| {
        let cur = w.block([x, y, z]);
        if overwrite || cur == 0 || TALL_GRASS.is(cur) {
            w.set_block([x, y, z], s);
        }
    });
}

/// A tree grown from `recipe` whose thin wood is `branch` blocks (`TREE_BRANCH`), each joined to
/// its wood neighbours; the rest logs along their limbs and leaves.
#[allow(clippy::too_many_arguments)]
fn grow_with_branches(
    recipe: &branchy::Recipe,
    log: u32,
    leaves: u32,
    branch: u32,
    x: i32,
    y: i32,
    z: i32,
    seed: u64,
    put: &mut impl FnMut(i32, i32, i32, u32, bool),
) {
    let widths: std::collections::HashMap<_, _> = branchy::wood_widths(recipe, seed)
        .into_iter()
        .map(|(x, y, z, w)| ((x, y, z), w))
        .collect();
    let b: Vec<_> = branchy::blocks(recipe, seed)
        .into_iter()
        .map(|(x, y, z, p)| (x, y, z, p, widths.get(&(x, y, z)).copied().unwrap_or(1.0)))
        .collect();
    put_with_branches(&b, None, log, leaves, branch, x, y, z, put);
}

/// A tree's blocks with each wood block's width (`colonize::blocks`, `branchy::wood_widths`), its
/// thin wood as `branch` blocks joined to their wood neighbours, the rest logs and leaves.
#[allow(clippy::too_many_arguments)]
fn put_with_branches(
    b: &[(i32, i32, i32, branchy::Part, f32)],
    upright: Option<u32>,
    log: u32,
    leaves: u32,
    branch: u32,
    x: i32,
    y: i32,
    z: i32,
    put: &mut impl FnMut(i32, i32, i32, u32, bool),
) {
    let wood: std::collections::HashSet<_> = b
        .iter()
        .filter(|e| matches!(e.3, branchy::Part::Log { .. }))
        .map(|e| (e.0, e.1, e.2))
        .collect();
    for &(dx, dy, dz, part, w) in b {
        match part {
            branchy::Part::Leaves => put(x + dx, y + dy, z + dz, leaves, false),
            branchy::Part::Log { axis } => {
                if w >= BRANCH_UNDER {
                    let along = ["x", "y", "z"][axis as usize];
                    let state = match upright {
                        Some(u) if axis == 1 => u,
                        _ => state_with(log, "axis", along).unwrap_or(log),
                    };
                    put(x + dx, y + dy, z + dz, state, true);
                    continue;
                }
                let mut s = state_with(branch, "size", if w < BRANCH_THIN { "4" } else { "8" }).unwrap_or(branch);
                for (name, (ox, oy, oz)) in
                    [("n", (0, 0, -1)), ("s", (0, 0, 1)), ("e", (1, 0, 0)), ("w", (-1, 0, 0)), ("u", (0, 1, 0)), ("d", (0, -1, 0))]
                {
                    if wood.contains(&(dx + ox, dy + oy, dz + oz)) {
                        s = state_with(s, name, "1").unwrap_or(s);
                    }
                }
                put(x + dx, y + dy, z + dz, s, true);
            }
        }
    }
}

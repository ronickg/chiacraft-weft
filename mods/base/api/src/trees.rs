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
    /// The world grew it (not a sapling): a sapling of a giant grows its young one's recipe.
    wild: bool,
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
    Some(TreeAt { x, y, z, log, leaves, crown, form, height, lean, seed: h, wild })
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
            return;
        }
        Form::Stump => {
            for y in t.y..t.y + 1 + t.height % 2 {
                put(t.x, y, t.z, log, true);
            }
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        _ => {}
    }
    // A species with a recipe grows from branches (`branchy`), its log turned along each limb.
    let name = block_name(log).split('[').next().unwrap_or("");
    let recipe = if t.wild { crate::tree_recipes::of(name) } else { crate::tree_recipes::sapling_of(name) };
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
    let Some(recipe) = crate::tree_recipes::of(block_name(log).split('[').next().unwrap_or("")) else { return };
    branchy::grow(recipe, x, y, z, h, |x, y, z, part| match part {
        branchy::Part::Log { axis } => {
            let along = ["x", "y", "z"][axis as usize];
            put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
        }
        branchy::Part::Leaves => put(x, y, z, leaves, false),
    });
    put(x, y - 1, z, DIRT.id(), true);
}

/// Places a tree of `log` into a live world (saplings). Checks for room first.
pub fn grow_tree(w: World, p: [i32; 3], log: u32) {
    let Some(t) = tree_at(log, p[0], p[1], p[2], w.random()) else { return };
    for y in t.y + 1..t.y + t.height {
        if w.get([t.x, y, t.z]) != 0 {
            return;
        }
    }
    tree_blocks(&t, |x, y, z, s, overwrite| {
        let cur = w.get([x, y, z]);
        if overwrite || cur == 0 || TALL_GRASS.is(cur) {
            w.set([x, y, z], s);
        }
    });
}

//! Trees (`notes/ECOLOGY.md`, part 3), for any mod: a log names its leaves (`TREE_LEAVES`) and its
//! shape by a tag (`CONE_TREE`, `FLAT_TREE`, `PALM_TREE`; none: round). Base's worldgen grows
//! them where a biome's `tree_kinds` names their logs, and a sapling's growth step grows one in a
//! live world (`grow_tree`), whichever mod's log it is.

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

static DIRT: Block = Block::named("base:dirt");
static TALL_GRASS: Block = Block::named("base:tall_grass");

pub struct TreeAt {
    x: i32,
    y: i32,
    z: i32,
    log: u32,
    leaves: u32,
    cone: bool,
    flat: bool,
    palm: bool,
    height: i32,
}

/// A tree growing from `log` (its `tree` component), `h` deciding the height.
pub fn tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    let leaves = TREE_LEAVES.on_block(log)?.0;
    let cone = CONE_TREE.on_block(log);
    let flat = FLAT_TREE.on_block(log);
    let palm = PALM_TREE.on_block(log);
    let height = if cone {
        6 + (h % 4) as i32
    } else if palm {
        6 + (h % 3) as i32
    } else if flat {
        5 + (h % 2) as i32
    } else {
        4 + (h % 3) as i32
    };
    Some(TreeAt { x, y, z, log, leaves, cone, flat, palm, height })
}

/// Calls `put(x, y, z, state, force)` for every block of the tree.
pub fn tree_blocks(t: &TreeAt, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let (log, leaves) = (t.log, t.leaves);
    let top = t.y + t.height;
    match t.cone {
        _ if t.palm => {
            // A tuft on top, fronds out two blocks each way, their tips a block lower.
            put(t.x, top, t.z, leaves, false);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                for k in 1..=2 {
                    put(t.x + dx * k, top - 1, t.z + dz * k, leaves, false);
                }
                put(t.x + dx * 3, top - 2, t.z + dz * 3, leaves, false);
            }
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                put(t.x + dx, top - 1, t.z + dz, leaves, false);
                put(t.x + dx * 2, top - 2, t.z + dz * 2, leaves, false);
            }
        }
        _ if t.flat => {
            // One wide flat layer on the trunk's top, a small one over its middle.
            for (y, r2) in [(top, 10), (top + 1, 2)] {
                for dz in -3i32..=3 {
                    for dx in -3i32..=3 {
                        let d = dx * dx + dz * dz;
                        if d <= r2 && (d < r2 || hash(1, (t.x + dx) as i64, y as i64, (t.z + dz) as i64) & 1 == 0) {
                            put(t.x + dx, y, t.z + dz, leaves, false);
                        }
                    }
                }
            }
        }
        true => {
            for dy in 2..=t.height + 1 {
                let y = t.y + dy;
                let r = if dy == t.height + 1 { 0 } else { 1 + ((t.height + 1 - dy) % 3).min(2) };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz <= r * r + 1 {
                            put(t.x + dx, y, t.z + dz, leaves, false);
                        }
                    }
                }
            }
        }
        false => {
            for y in top - 3..=top {
                let r: i32 = if y >= top - 1 { 1 } else { 2 };
                for dz in -r..=r {
                    for dx in -r..=r {
                        let corner = dx.abs() == r && dz.abs() == r;
                        if corner && (y == top || hash(0, (t.x + dx) as i64, y as i64, (t.z + dz) as i64) & 1 == 0) {
                            continue;
                        }
                        put(t.x + dx, y, t.z + dz, leaves, false);
                    }
                }
            }
        }
    }
    for y in t.y..top {
        put(t.x, y, t.z, log, true);
    }
    put(t.x, t.y - 1, t.z, DIRT.id(), true);
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

//! Felling (Ronald, 11-04: "Only with an axe"): a tree's lowest log broken with an axe brings
//! the trunk down. Every log of the tree above it breaks, and, as with any block a player breaks,
//! the logs land at their feet. Bare hands still take a tree a block at a time, as does an axe
//! anywhere but the foot.
//!
//! A tree is the logs joined to the broken one at its height or above (corners too, for
//! branches), with leaves among them: a cabin's wall of logs has none and stays standing. A tree
//! of more than `MAX_LOGS` (a giant) is chopped by the block.

use std::collections::HashSet;
use std::sync::Mutex;

use weft_sdk::*;

static AXES: Tag = Tag::named("tools/axe");
static LOGS: Tag = Tag::named("logs");
static LEAVES: Tag = Tag::named("leaves");

/// The most logs one felling brings down; a bigger tree is chopped by the block.
const MAX_LOGS: usize = 128;
/// How far from the foot, across, a tree's logs may reach (an acacia's branches, a big oak's).
const REACH: i32 = 6;
/// Leaves touching its logs a tree has at least: a log wall or a pile has none.
const MIN_LEAVES: usize = 4;

/// The logs a felling is breaking right now: their own `BLOCK_BROKEN` must not fell again.
static FELLING: Mutex<Option<HashSet<[i32; 3]>>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            if FELLING
                .lock()
                .unwrap()
                .get_or_insert_with(HashSet::new)
                .remove(&e.pos)
            {
                continue;
            }
            if e.player == 0 || !AXES.on_item(e.item) || !LOGS.on_block(e.state) {
                continue;
            }
            let [x, y, z] = e.pos;
            // Only from the foot: the block below is no log.
            if LOGS.on_block(w.get([x, y - 1, z])) {
                continue;
            }
            let kind = |p| {
                let state = w.get(p);
                if LOGS.on_block(state) {
                    Kind::Log
                } else if LEAVES.on_block(state) {
                    Kind::Leaves
                } else {
                    Kind::Other
                }
            };
            let Some(logs) = tree(kind, e.pos) else { continue };
            let axe = w
                .inventory(e.player)
                .group("hotbar")
                .get(w.selected_slot(e.player));
            FELLING
                .lock()
                .unwrap()
                .get_or_insert_with(HashSet::new)
                .extend(logs.iter().copied());
            // From the top down, so every log still has its own below while it breaks; in
            // creative, as there, without drops.
            let drops = !w.creative(e.player);
            for p in logs {
                w.break_block(p, axe, e.player, drops);
            }
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Log,
    Leaves,
    Other,
}

/// The tree whose foot log was at `foot` (now broken): its logs, highest first, if it is one
/// to fell.
fn tree(kind: impl Fn([i32; 3]) -> Kind, foot: [i32; 3]) -> Option<Vec<[i32; 3]>> {
    let mut seen = HashSet::from([foot]);
    let mut todo = vec![foot];
    let mut logs = Vec::new();
    let mut leaves = HashSet::new();
    while let Some([x, y, z]) = todo.pop() {
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let p = [x + dx, y + dy, z + dz];
                    if p[1] < foot[1] || (p[0] - foot[0]).abs() > REACH || (p[2] - foot[2]).abs() > REACH || !seen.insert(p) {
                        continue;
                    }
                    match kind(p) {
                        Kind::Log => {
                            logs.push(p);
                            if logs.len() > MAX_LOGS {
                                return None;
                            }
                            todo.push(p);
                        }
                        Kind::Leaves => {
                            leaves.insert(p);
                        }
                        Kind::Other => {}
                    }
                }
            }
        }
    }
    if logs.is_empty() || leaves.len() < MIN_LEAVES {
        return None;
    }
    logs.sort_by_key(|p| std::cmp::Reverse(p[1]));
    Some(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world of `logs` and `leaves`, the rest air.
    fn world(logs: &[[i32; 3]], leaves: &[[i32; 3]]) -> impl Fn([i32; 3]) -> Kind {
        let (logs, leaves): (HashSet<_>, HashSet<_>) = (logs.iter().copied().collect(), leaves.iter().copied().collect());
        move |p| {
            if logs.contains(&p) {
                Kind::Log
            } else if leaves.contains(&p) {
                Kind::Leaves
            } else {
                Kind::Other
            }
        }
    }

    #[test]
    fn a_tree_falls_top_down_a_wall_and_a_giant_stand() {
        // A birch: trunk 1..=5 over a chopped foot at 0, a crown round its top.
        let trunk: Vec<_> = (1..=5).map(|y| [0, y, 0]).collect();
        let crown = [[1, 5, 0], [-1, 5, 0], [0, 5, 1], [0, 5, -1], [0, 6, 0]];
        let logs = tree(world(&trunk, &crown), [0, 0, 0]).expect("a tree");
        assert_eq!(logs.len(), 5);
        assert_eq!(logs.first(), Some(&[0, 5, 0]), "the top first");
        assert_eq!(logs.last(), Some(&[0, 1, 0]));
        // A branch off to the side, joined at a corner, falls with it.
        let mut branched = trunk.clone();
        branched.push([1, 4, 1]);
        assert_eq!(tree(world(&branched, &crown), [0, 0, 0]).unwrap().len(), 6);
        // A cabin's log wall: no leaves, it stays.
        let wall: Vec<_> = (0..4)
            .flat_map(|x| (1..=3).map(move |y| [x, y, 0]))
            .collect();
        assert_eq!(tree(world(&wall, &[]), [0, 0, 0]), None);
        // Nothing under the foot's height and nothing beyond reach comes with it.
        let mut far = trunk.clone();
        far.extend([[0, -1, 0], [7, 1, 0]]);
        assert_eq!(tree(world(&far, &crown), [0, 0, 0]).unwrap().len(), 5);
        // A giant: more than MAX_LOGS, chopped by the block.
        let giant: Vec<_> = (1..=40)
            .flat_map(|y| [[0, y, 0], [1, y, 0], [0, y, 1], [1, y, 1]])
            .collect();
        assert_eq!(tree(world(&giant, &crown), [0, 0, 0]), None);
    }
}

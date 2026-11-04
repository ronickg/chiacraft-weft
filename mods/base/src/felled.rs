//! A felled tree on the ground (Ronald, 11-05: "once it's on the ground, you still got to chop
//! the logs in pieces, kind of like Valheim does it"). `felling` lays the tree down instead of
//! dropping its logs: its logs turn a quarter over the stump's far edge, away from the player
//! (a block turned by a right angle stays a block), each slice across the fall dropping to the
//! ground below it (`vitals::felling::LAND_DROP` at most), and lie marked `felled`, in
//! alternating pieces (`a`, `b`) `vitals::felling::PIECE` logs long. An axe on any log of a piece
//! cuts the whole piece, as long as felling a tree of its logs takes, and its wood lands at the
//! player's feet; bare hands still take one block. What the trunk lands on stays (its logs there
//! are lost), and leaves and plants in its way give way.

use std::collections::{HashMap, HashSet};

use weft_sdk::*;

static AXES: Tag = Tag::named("tools/axe");
static LOGS: Tag = Tag::named("logs");
static LEAVES: Tag = Tag::named("leaves");

/// The piece a felled log belongs to (`a` or `b`), none for a standing or placed log.
pub fn piece_of(state: u32) -> Option<&'static str> {
    if !LOGS.on_block(state) {
        return None;
    }
    state_prop(state, "felled").filter(|&f| f != "no")
}

/// The way a tree falls from `foot`, away from a player at `from` (x and z): one of the four
/// ways along the ground, so its logs lie along x or z.
pub fn away(foot: [i32; 3], from: [f64; 3]) -> [i32; 2] {
    let (dx, dz) = (foot[0] as f64 + 0.5 - from[0], foot[2] as f64 + 0.5 - from[2]);
    if dx.abs() >= dz.abs() { [if dx >= 0.0 { 1 } else { -1 }, 0] } else { [0, if dz >= 0.0 { 1 } else { -1 }] }
}

/// Where each of a tree's `logs` lies after falling from `foot` toward `d`, turned a quarter over
/// the far edge of its base (the logs at the foot's height): (where, how far along the fall it
/// lies, counted from the first lying slice). A log turned below the ground (a branch on the far
/// side, low) has none.
pub fn turned(foot: [i32; 3], logs: &[[i32; 3]], d: [i32; 2]) -> Vec<Option<([i32; 3], i32)>> {
    let along = |p: [i32; 3]| (p[0] - foot[0]) * d[0] + (p[2] - foot[2]) * d[1];
    // The far edge: the base's farthest log along the fall.
    let far = logs
        .iter()
        .filter(|p| p[1] == foot[1])
        .map(|&p| along(p))
        .max()
        .unwrap_or(0)
        .max(0);
    // With other logs at the foot's height (a 2 x 2's, a disc) they lie first, past the edge;
    // a trunk of one lies from the stump on, where its chopped foot was.
    let first = if logs.iter().any(|p| p[1] == foot[1]) { far + 1 } else { 0 };
    logs.iter()
        .map(|&p| {
            let (a, h) = (along(p), p[1] - foot[1]);
            // Up becomes along the fall past the edge; along the fall becomes down from it.
            let (a2, up) = (first + h, far - a);
            if up < 0 {
                return None;
            }
            // Across the fall stays where it was.
            let at = if d[0] != 0 { [foot[0] + d[0] * a2, foot[1] + up, p[2]] } else { [p[0], foot[1] + up, foot[2] + d[1] * a2] };
            Some((at, h))
        })
        .collect()
}

/// A log's `axis` turned with the tree falling along x (`along_x`) or z: up becomes the fall's
/// axis, the fall's axis up, across stays.
pub fn turned_axis(axis: &str, along_x: bool) -> &'static str {
    let fall = if along_x { "x" } else { "z" };
    match axis {
        "y" => fall,
        "bark" => "bark",
        a if a == fall => "y",
        "x" => "x",
        _ => "z",
    }
}

/// A creature (a player too) stands in `cell`: a log falling there drops as its item instead, so
/// nobody is buried in a trunk (fell_hang1 buried the player standing in its way).
pub fn occupied(w: &World, cell: [i32; 3]) -> bool {
    let mid = [cell[0] as f64 + 0.5, cell[1] as f64 + 0.5, cell[2] as f64 + 0.5];
    !in_box(w, mid, 0.5).is_empty()
}

/// The creatures (players too) whose box reaches within `half` of `mid` each way.
pub fn in_box(w: &World, mid: [f64; 3], half: f64) -> Vec<u64> {
    w.targets_near(mid, half + 3.0, None)
        .into_iter()
        .filter(|&id| {
            w.entity(id).is_some_and(|e| {
                let r = e.size.width as f64 / 2.0;
                let (lo, hi) = ([e.pos.x - r, e.pos.y, e.pos.z - r], [e.pos.x + r, e.pos.y + e.size.height as f64, e.pos.z + r]);
                (0..3).all(|i| lo[i] < mid[i] + half && hi[i] > mid[i] - half)
            })
        })
        .collect()
}

/// What a lying trunk may take the place of: air, leaves and what breaks at a touch (plants).
fn gives_way(state: u32) -> bool {
    state == 0 || LEAVES.on_block(state) || HARDNESS.on_block(state).is_some_and(|h| h <= 0.0)
}

/// Lays a felled tree down: `logs` (their states in `states`), just taken out of the world, fall
/// from `foot` toward `d`. Returns how many came to lie.
pub fn lay(w: &World, foot: [i32; 3], logs: &[[i32; 3]], states: &[u32], d: [i32; 2]) -> usize {
    let spots = turned(foot, logs, d);
    // Slices across the fall, each dropped to the ground under its lowest log.
    let mut lowest: HashMap<i32, i32> = HashMap::new();
    for (at, h) in spots.iter().flatten() {
        let low = lowest.entry(*h).or_insert(at[1]);
        *low = (*low).min(at[1]);
    }
    let mut drop: HashMap<i32, i32> = HashMap::new();
    for (&h, &y) in &lowest {
        // The slice's cells at its lowest: each may sink while all below them give way.
        let cells: Vec<[i32; 3]> = spots
            .iter()
            .flatten()
            .filter(|(at, sh)| *sh == h && at[1] == y)
            .map(|(at, _)| *at)
            .collect();
        let sink = (0..vitals::felling::LAND_DROP)
            .take_while(|&n| {
                cells
                    .iter()
                    .all(|c| gives_way(w.block([c[0], y - n - 1, c[2]])))
            })
            .count() as i32;
        drop.insert(h, sink);
    }
    let along_x = d[0] != 0;
    let mut laid = 0;
    let mut taken = HashSet::new();
    for (spot, &state) in spots.iter().zip(states) {
        let Some((at, h)) = spot else { continue };
        let at = [at[0], at[1] - drop[h], at[2]];
        if !taken.insert(at) || !gives_way(w.block(at)) {
            continue;
        }
        let piece = if (h / vitals::felling::PIECE) % 2 == 0 { "a" } else { "b" };
        let axis = state_prop(state, "axis").map_or("y", |a| turned_axis(a, along_x));
        let lying = state_with(state, "axis", axis)
            .and_then(|s| state_with(s, "felled", piece))
            .unwrap_or(state);
        if occupied(w, at) {
            let item = Item::lookup(
                name_of(abi::entry_kind::BLOCK, block_of(state))
                    .split('[')
                    .next()
                    .unwrap_or(""),
            );
            if let Some(item) = item {
                w.spawn_item([at[0] as f64 + 0.5, at[1] as f64 + 0.5, at[2] as f64 + 0.5], ItemStack::new(item, 1));
            }
            continue;
        }
        if w.block(at) != 0 {
            // A plant or leaves in the way: broken as if by nobody, its loot where it was.
            w.break_block(at, ItemStack::EMPTY, 0, true);
        }
        if w.set_block(at, lying) {
            laid += 1;
        }
    }
    laid
}

/// The felled logs of the piece around `from` (its own cell excluded, as it was just cut): the
/// same block in the same piece, joined face to face, at most `max`.
pub fn piece(w: &World, from: [i32; 3], state: u32, max: usize) -> Vec<[i32; 3]> {
    let (block, letter) = (block_of(state), piece_of(state));
    let same = |s: u32| block_of(s) == block && piece_of(s) == letter;
    let mut seen = HashSet::from([from]);
    let mut todo = vec![from];
    let mut out = Vec::new();
    while let Some([x, y, z]) = todo.pop() {
        for d in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
            let p = [x + d[0], y + d[1], z + d[2]];
            if seen.insert(p) && same(w.block(p)) {
                out.push(p);
                if out.len() >= max {
                    return out;
                }
                todo.push(p);
            }
        }
    }
    out
}

/// An axe on a felled log (`state`, at `pos`, still there as mining starts): the piece it cuts,
/// counting itself.
pub fn piece_logs(w: &World, pos: [i32; 3], state: u32, item: u32) -> Option<usize> {
    (AXES.on_item(item) && piece_of(state).is_some()).then(|| 1 + piece(w, pos, state, MAX_PIECE).len())
}

/// The most logs one cut takes: a sequoia's trunk is some 28 across, so a piece of it is ~110.
pub const MAX_PIECE: usize = 512;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_turns_a_quarter_over_its_far_edge() {
        // A trunk of 3 over a chopped foot, falling toward +x: it lies from x 1 on (next to the
        // stump), at the foot's height, in the order it stood.
        let trunk = [[0, 1, 0], [0, 2, 0], [0, 3, 0]];
        let lying = turned([0, 0, 0], &trunk, [1, 0]);
        assert_eq!(lying, vec![Some(([1, 0, 0], 1)), Some(([2, 0, 0], 2)), Some(([3, 0, 0], 3))]);
        // A 2 x 2 falling toward -z: its far edge is z -1, its near column lies a block up.
        let pair = [[0, 0, -1], [0, 1, 0], [0, 1, -1]];
        let lying = turned([0, 0, 0], &pair, [0, -1]);
        assert_eq!(lying[0], Some(([0, 0, -2], 0)));
        assert_eq!(lying[1], Some(([0, 1, -3], 1)), "the near column, a block up");
        assert_eq!(lying[2], Some(([0, 0, -3], 1)));
        // A low branch on the far side would turn into the ground: it has no place.
        assert_eq!(turned([0, 0, 0], &[[2, 1, 0]], [1, 0]), vec![None]);
        // Across the fall stays.
        assert_eq!(turned([0, 0, 0], &[[0, 2, 1]], [1, 0])[0], Some(([2, 0, 1], 2)));
    }

    #[test]
    fn axes_turn_with_the_tree_and_it_falls_away() {
        assert_eq!(turned_axis("y", true), "x");
        assert_eq!(turned_axis("x", true), "y");
        assert_eq!(turned_axis("z", true), "z");
        assert_eq!(turned_axis("y", false), "z");
        assert_eq!(turned_axis("z", false), "y");
        assert_eq!(turned_axis("bark", false), "bark");
        assert_eq!(away([5, 0, 0], [0.5, 0.0, 1.5]), [1, 0]);
        assert_eq!(away([0, 0, -5], [0.5, 0.0, 0.5]), [0, -1]);
    }
}

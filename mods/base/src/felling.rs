//! Felling (Ronald, 11-04: "Only with an axe"): a tree's lowest log broken with an axe brings
//! the trunk down. Every log of the tree above it breaks, and, as with any block a player breaks,
//! the logs land at their feet. Bare hands still take a tree a block at a time, as does an axe
//! anywhere but the foot.
//!
//! That chop is hard (Ronald, 11-05: "it takes longer as you always chop down the bottom part"):
//! the foot takes as long as `vitals::felling` says for the tree's logs and the axe, done through
//! the engine's own mining timer. When a player starts mining a tree's foot with an axe
//! (`question::BREAK_BLOCK`, asked as mining starts), a `MINING_SPEED` modifier slows them by the
//! felling's chops; the client reads the attribute, so the cracks creep at that pace, and the
//! server times the break by it too. Mining anything else takes it off. A tree past the axe's
//! tier still falls, slower still, and gives no wood ("Slow and fruitless").
//!
//! A tree is the logs joined to the broken one at its height or above (corners too, for
//! branches), with leaves among them: a cabin's wall of logs has none and stays standing. Another
//! trunk's foot (a log standing on the ground) ends it, so trees touching at a corner fall one at
//! a time. A tree of more than `MAX_LOGS` is chopped by the block (none grows that big: a grown
//! sequoia has up to 1421). The axe wears by one for each log it brings down, as if each were
//! chopped. The leaves left in the air decay (`leaf_decay`).

use std::collections::HashSet;
use std::sync::Mutex;

use weft_sdk::*;

static AXES: Tag = Tag::named("tools/axe");
static LOGS: Tag = Tag::named("logs");
static LEAVES: Tag = Tag::named("leaves");

/// The most logs one felling brings down; a bigger tree is chopped by the block.
const MAX_LOGS: usize = 2048;
/// How far from the foot, across, a tree's logs may reach (an acacia's branches, a big oak's; a
/// sequoia's reach 6: `cargo run -p base_api --example tree_logs`).
const REACH: i32 = 6;
/// Leaves touching its logs a tree has at least: a log wall or a pile has none.
const MIN_LEAVES: usize = 4;
/// The modifier that slows a player chopping a tree's foot.
const CHOPPING: &str = "base:felling";

/// The logs a felling is breaking right now: their own `BLOCK_BROKEN` must not fell again.
static FELLING: Mutex<Option<HashSet<[i32; 3]>>> = Mutex::new(None);

/// Modifier changes asked for while mining started (applied on the next world tick, outside the
/// question): (dimension, player, the mining speed multiplier, or none to take it off).
static SLOWING: Mutex<Vec<(u32, u64, Option<f32>)>> = Mutex::new(Vec::new());
/// Players slowed now, so only they are looked at when they mine something else.
static SLOWED: Mutex<Option<HashSet<u64>>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    // Mining starts (and ends): a tree's foot with an axe is slow, anything else as it is.
    r.intercept(&question::BREAK_BLOCK, |q| {
        let player = q.entity();
        if player == 0 || felling_now(q.pos()) {
            return Verdict::Abstain;
        }
        let w = q.world();
        let cut = crate::felled::piece_logs(&w, q.pos(), q.state(), q.item())
            .map(|n| vitals::felling::piece(n as u32))
            .or_else(|| {
                foot_of_tree(&w, q.pos(), q.state(), q.item()).map(|logs| {
                    let axe = held(&w, player);
                    vitals::felling::felling(logs.len() as u32, TOOL_LEVEL.on_stack(&axe).unwrap_or(0))
                })
            });
        if let Some(f) = cut {
            let axe = held(&w, player);
            let hardness = HARDNESS.on_block(q.state()).unwrap_or(2.0);
            let one = vitals::felling::log_secs(hardness, MINING_SPEED.on_stack(&axe).unwrap_or(1.0));
            SLOWING
                .lock()
                .unwrap()
                .push((w.dimension(), player, Some(vitals::felling::speed(f, one))));
        } else if slowed(player) {
            SLOWING.lock().unwrap().push((w.dimension(), player, None));
        }
        Verdict::Abstain
    });
    r.world_tick(|w| {
        let dim = w.dimension();
        let mut due = SLOWING.lock().unwrap();
        if due.is_empty() {
            return;
        }
        let mut slowed = SLOWED.lock().unwrap();
        let slowed = slowed.get_or_insert_with(HashSet::new);
        due.retain(|&(d, player, speed)| {
            if d != dim {
                return true;
            }
            match speed {
                // Till taken off, or an hour and a bit at most (a player who walked away).
                Some(s) => {
                    let secs = vitals::felling::LONGEST_SECS as u64 + 60;
                    w.add_modifier(player, &attribute::MINING_SPEED, CHOPPING, attr_op::MUL_TOTAL, s as f64 - 1.0, secs * 20);
                    slowed.insert(player);
                }
                None => {
                    w.remove_modifier(player, &attribute::MINING_SPEED, CHOPPING);
                    slowed.remove(&player);
                }
            }
            false
        });
    });
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
            if e.player == 0 {
                continue;
            }
            // The chop is done: the next mining is at the player's own pace.
            if slowed(e.player) {
                SLOWING
                    .lock()
                    .unwrap()
                    .push((w.dimension(), e.player, None));
            }
            let (hotbar, slot) = (w.inventory(e.player).group("hotbar"), w.selected_slot(e.player));
            let axe = hotbar.get(slot);
            let survival = !w.creative(e.player);
            // A piece of a felled trunk: the rest of it goes too, its wood at the player's feet.
            if AXES.on_item(e.item) && crate::felled::piece_of(e.state).is_some() {
                let piece = crate::felled::piece(&w, e.pos, e.state, crate::felled::MAX_PIECE);
                FELLING
                    .lock()
                    .unwrap()
                    .get_or_insert_with(HashSet::new)
                    .extend(piece.iter().copied());
                for &p in &piece {
                    w.break_block(p, axe, e.player, survival);
                }
                if survival {
                    let by = vitals::felling::wear(piece.len() as u32 + 1);
                    hotbar.set(slot, worn(axe, by, DURABILITY.on_item(axe.item).unwrap_or(0)));
                }
                continue;
            }
            let Some(logs) = foot_of_tree(&w, e.pos, e.state, e.item) else { continue };
            let felling = vitals::felling::felling(logs.len() as u32, TOOL_LEVEL.on_stack(&axe).unwrap_or(0));
            // It falls (`fall`): lifted out with its crown, it leans over away from the player and
            // lies down to be cut in pieces (`felled`) in survival within the axe's tier; in
            // creative and past the tier it shatters. If it can't be lifted it is taken down
            // here, from the top down so every log still has its own below while it breaks.
            let lays = survival && felling.fruitful;
            let states: Vec<u32> = logs.iter().map(|&p| w.get(p)).collect();
            let from = w
                .entity(e.player)
                .map_or([e.pos[0] as f64, 0.0, e.pos[2] as f64 - 1.0], |st| st.pos);
            let d = crate::felled::away(e.pos, from);
            let leaves = crown(&w, &logs);
            if crate::fall::start(&w, e.pos, logs.clone(), states.clone(), leaves, d, lays) {
                crate::leaf_decay::logs_gone(&w, logs.iter().copied());
            } else {
                FELLING
                    .lock()
                    .unwrap()
                    .get_or_insert_with(HashSet::new)
                    .extend(logs.iter().copied());
                for &p in &logs {
                    w.break_block(p, axe, e.player, false);
                }
                if lays {
                    crate::felled::lay(&w, e.pos, &logs, &states, d);
                }
            }
            if survival {
                let by = vitals::felling::wear(logs.len() as u32);
                hotbar.set(slot, worn(axe, by, DURABILITY.on_item(axe.item).unwrap_or(0)));
            }
        }
    });
}

fn slowed(player: u64) -> bool {
    SLOWED
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|s| s.contains(&player))
}

fn felling_now(p: [i32; 3]) -> bool {
    FELLING
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|f| f.contains(&p))
}

fn held(w: &World, player: u64) -> ItemStack {
    w.inventory(player)
        .group("hotbar")
        .get(w.selected_slot(player))
}

/// The logs of the tree above `pos` (now or just broken, `state` it), highest first, if it is a
/// tree's foot mined with an axe (`item`).
fn foot_of_tree(w: &World, pos: [i32; 3], state: u32, item: u32) -> Option<Vec<[i32; 3]>> {
    if !AXES.on_item(item) {
        return None;
    }
    tree_on(w, pos, state)
}

/// The logs of the tree whose foot is at `pos` (`state` it, there or just gone), highest first,
/// with the foot itself left out (it stays as a stump when the tree is knocked down: `fall`).
pub fn tree_on(w: &World, pos: [i32; 3], state: u32) -> Option<Vec<[i32; 3]>> {
    let [x, y, z] = pos;
    // Only from the foot: the block below is no log. A felled log is no tree's.
    let felled = |s: u32| crate::felled::piece_of(s).is_some();
    if !LOGS.on_block(state) || felled(state) || LOGS.on_block(w.get([x, y - 1, z])) {
        return None;
    }
    let kind = |p: [i32; 3]| {
        let state = w.get(p);
        if p == pos {
            // Mining starts with the foot still there; it is the tree's, not ground.
            Kind::Air
        } else if felled(state) {
            Kind::Ground
        } else if LOGS.on_block(state) {
            Kind::Log
        } else if LEAVES.on_block(state) {
            Kind::Leaves
        } else if state == 0 {
            Kind::Air
        } else {
            Kind::Ground
        }
    };
    tree(kind, pos)
}

/// The most leaves a falling tree takes along (a sequoia's crown is a few thousand).
const MAX_CROWN: usize = 8192;

/// A tree's crown: the tree leaves (none a player placed) joined face to face to its `logs`
/// within `base_api::branchy::LEAF_REACH` steps, as leaf decay counts them.
pub fn crown(w: &World, logs: &[[i32; 3]]) -> Vec<([i32; 3], u32)> {
    let mut seen: HashSet<[i32; 3]> = logs.iter().copied().collect();
    let mut ring: Vec<[i32; 3]> = logs.to_vec();
    let mut out = Vec::new();
    for _ in 0..base_api::branchy::LEAF_REACH {
        let mut next = Vec::new();
        for [x, y, z] in ring {
            for d in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
                let p = [x + d[0], y + d[1], z + d[2]];
                if seen.insert(p) && crate::leaf_decay::decays(w, p) {
                    out.push((p, w.get(p)));
                    if out.len() >= MAX_CROWN {
                        return out;
                    }
                    next.push(p);
                }
            }
        }
        ring = next;
    }
    out
}

/// `axe` worn by `by` more (its `max` durability, 0 for none): gone once used up.
fn worn(axe: ItemStack, by: u32, max: u32) -> ItemStack {
    let damage = axe.damage() + by;
    if max > 0 && damage >= max {
        ItemStack::EMPTY
    } else if max > 0 {
        axe.with_damage(damage)
    } else {
        axe
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Log,
    Leaves,
    Air,
    /// Anything else: what a trunk stands on.
    Ground,
}

/// The tree whose foot log was at `foot` (now broken, or being chopped: `kind` says air there):
/// its logs, highest first, if it is one to fell.
///
/// Its logs are those joined to the foot whose column, straight down, ends in the air (a branch)
/// or on the ground in the trunk's own base: the logs on the ground at the foot's height joined
/// to it across (a 2 x 2's or a sequoia's disc). A log over another trunk's foot is that tree's,
/// so a neighbour whose branches touch stays standing.
fn tree(kind: impl Fn([i32; 3]) -> Kind, foot: [i32; 3]) -> Option<Vec<[i32; 3]>> {
    let below = |[x, y, z]: [i32; 3]| [x, y - 1, z];
    let near = |p: [i32; 3]| (p[0] - foot[0]).abs() <= REACH && (p[2] - foot[2]).abs() <= REACH;
    // The trunk's base.
    let mut base = HashSet::from([foot]);
    let mut todo = vec![foot];
    while let Some([x, y, z]) = todo.pop() {
        for (dx, dz) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
            let p = [x + dx, y, z + dz];
            if near(p) && !base.contains(&p) && kind(p) == Kind::Log && kind(below(p)) == Kind::Ground {
                base.insert(p);
                todo.push(p);
            }
        }
    }
    // Another trunk's: its column stands on the ground outside the base.
    let elsewhere = |p: [i32; 3]| {
        let mut q = p;
        while kind(below(q)) == Kind::Log {
            q = below(q);
        }
        kind(below(q)) == Kind::Ground && !base.contains(&q)
    };
    let mut seen = HashSet::from([foot]);
    let mut todo = vec![foot];
    let mut logs = Vec::new();
    let mut leaves = HashSet::new();
    while let Some([x, y, z]) = todo.pop() {
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let p = [x + dx, y + dy, z + dz];
                    if p[1] < foot[1] || !near(p) || !seen.insert(p) {
                        continue;
                    }
                    match kind(p) {
                        Kind::Log if elsewhere(p) => {}
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
                        Kind::Air | Kind::Ground => {}
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

    /// A world of `logs` and `leaves` over the ground (y 0 and under), the rest air, and the
    /// chopped foot at (0, 0, 0) air.
    fn world(logs: &[[i32; 3]], leaves: &[[i32; 3]]) -> impl Fn([i32; 3]) -> Kind {
        let (logs, leaves): (HashSet<_>, HashSet<_>) = (logs.iter().copied().collect(), leaves.iter().copied().collect());
        move |p| {
            if p == [0, 0, 0] {
                Kind::Air
            } else if logs.contains(&p) {
                Kind::Log
            } else if leaves.contains(&p) {
                Kind::Leaves
            } else if p[1] <= 0 {
                Kind::Ground
            } else {
                Kind::Air
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
        // A neighbour touching at a corner, standing on the ground: it stays.
        let mut pair = trunk.clone();
        pair.extend((1..=5).map(|y| [1, y, 1]));
        assert_eq!(tree(world(&pair, &crown), [0, 0, 0]).unwrap().len(), 5, "one tree only");
        // But a branch reaching down toward the ground, over air, is the tree's own.
        let mut low = trunk.clone();
        low.extend([[1, 3, 0], [2, 2, 0]]);
        assert_eq!(tree(world(&low, &crown), [0, 0, 0]).unwrap().len(), 7);
        // A giant 2 x 2 falls whole, its foot's three other logs too (a grown sequoia has up
        // to 1421 logs; how long it takes and whether it gives wood is `vitals::felling`'s).
        let column = |top: i32| -> Vec<[i32; 3]> {
            (0..top)
                .flat_map(|y| [[0, y, 0], [1, y, 0], [0, y, 1], [1, y, 1]])
                .filter(|&p| p != [0, 0, 0])
                .collect()
        };
        let high = [[-1, 10, 0], [-1, 11, 0], [2, 10, 0], [2, 11, 0]];
        assert_eq!(tree(world(&column(300), &high), [0, 0, 0]).map(|l| l.len()), Some(4 * 300 - 1));
        // Past MAX_LOGS (nothing grows so big), it is chopped by the block.
        let tower = column(520);
        assert_eq!(tree(world(&tower, &high), [0, 0, 0]), None);
    }

    #[test]
    fn the_axe_wears_by_a_log_each() {
        let axe = ItemStack::new(7, 1);
        assert_eq!(worn(axe, 5, 131).damage(), 5);
        assert_eq!(worn(axe.with_damage(129), 5, 131), ItemStack::EMPTY, "used up");
        assert_eq!(worn(axe, 5, 0), axe, "no durability, no wear");
    }
}

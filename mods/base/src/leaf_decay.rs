//! Leaf decay, vanilla-like: a tree's leaves with no log within `REACH` fall over the next second
//! to a minute, dropping what their loot gives (saplings, sticks, their species' fruit). Leaves are
//! looked at only round a log that went (chopped or felled, `felling`): nothing scans the world.
//! Leaves a player placed (`placed`) keep, and so do the shrubs' leaves (no `leaves/tree` tag: a
//! shrub has no log).

use std::collections::HashSet;
use std::sync::Mutex;

use weft_sdk::*;

static LOGS: Tag = Tag::named("logs");
static TREE_LEAVES: Tag = Tag::named("leaves/tree");

/// Leaves this far (each way) from a log that went are looked at.
const NEAR: i32 = 5;
/// A log this close (each way) keeps leaves on.
const REACH: i32 = 4;
/// Ticks after the log went that a leaf may fall: a second to a minute.
const SOONEST: u64 = 20;
const LATEST: u64 = 1200;

/// Leaves to look at again: (dimension, tick, position), and the positions waiting.
#[derive(Default)]
struct Due {
    list: Vec<(u32, u64, [i32; 3])>,
    waiting: Option<HashSet<(u32, [i32; 3])>>,
}

static DUE: Mutex<Due> = Mutex::new(Due { list: Vec::new(), waiting: None });

pub fn register(r: &mut Registry) {
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        let (dim, now) = (w.dimension(), w.time());
        for e in b.iter().filter(|e| LOGS.on_block(e.state)) {
            let mut due = DUE.lock().unwrap();
            let Due { list, waiting } = &mut *due;
            let waiting = waiting.get_or_insert_with(HashSet::new);
            let [x, y, z] = e.pos;
            for dy in -NEAR..=NEAR {
                for dz in -NEAR..=NEAR {
                    for dx in -NEAR..=NEAR {
                        let p = [x + dx, y + dy, z + dz];
                        if decays(&w, p) && waiting.insert((dim, p)) {
                            let wait = SOONEST + (w.random() % (LATEST - SOONEST));
                            list.push((dim, now + wait, p));
                        }
                    }
                }
            }
        }
    });
    // A player's leaves are theirs to keep.
    r.on(&BLOCK_PLACED, |b| {
        let w = b.world();
        for e in b
            .iter()
            .filter(|e| e.player != 0 && TREE_LEAVES.on_block(e.state))
        {
            w.set_prop(e.pos, "placed", "true");
        }
    });
    r.world_tick(|w| {
        let (dim, now) = (w.dimension(), w.time());
        let ready: Vec<[i32; 3]> = {
            let mut due = DUE.lock().unwrap();
            let Due { list, waiting } = &mut *due;
            if list.is_empty() {
                return;
            }
            let mut ready = Vec::new();
            list.retain(|&(d, at, p)| {
                let go = d == dim && at <= now;
                if go {
                    ready.push(p);
                    if let Some(waiting) = waiting.as_mut() {
                        waiting.remove(&(d, p));
                    }
                }
                !go
            });
            ready
        };
        for p in ready {
            if decays(w, p) && !log_near(w, p) {
                w.break_block(p, ItemStack::EMPTY, 0, true);
            }
        }
    });
}

/// A tree's leaves no player placed.
fn decays(w: &World, p: [i32; 3]) -> bool {
    TREE_LEAVES.on_block(w.get(p)) && w.prop(p, "placed") != Some("true")
}

fn log_near(w: &World, [x, y, z]: [i32; 3]) -> bool {
    (-REACH..=REACH).any(|dy| (-REACH..=REACH).any(|dz| (-REACH..=REACH).any(|dx| LOGS.on_block(w.get([x + dx, y + dy, z + dz])))))
}

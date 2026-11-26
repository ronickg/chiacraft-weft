//! Rooms (`vitals::shelter`, `notes/CONTENT.md` decision 7): the enclosed air around a player, found
//! once a second by filling the open blocks from where they stand. If the fill reaches open sky or
//! grows past a room's size, it's the outdoors; otherwise its wall faces are counted by what they're
//! made of, and the room's air replaces the outside's for the body (`temperature.rs`).

use std::collections::{HashSet, VecDeque};

use vitals::shelter::{ROOM_MAX, Room, WALLS, Wall};
use weft_sdk::*;

/// What a solid block is, as a wall (by its name: logs, planks, stone and brick, earth, snow, wool
/// and leaves, glass; anything else wooden).
pub fn wall_of(state: u32) -> Wall {
    let n = block_name(state).split('[').next().unwrap_or("");
    let n = n.split_once(':').map_or(n, |(_, n)| n);
    let has = |s: &str| n.contains(s);
    if has("glass") {
        Wall::Glass
    } else if has("log") || has("wood") && !has("planks") {
        Wall::Log
    } else if has("planks") || has("door") || has("bookshelf") || has("chest") || has("crafting") {
        Wall::Plank
    } else if has("snow") {
        Wall::Snow
    } else if has("stone") || has("brick") || has("cobble") || has("obsidian") || has("ore") || has("basalt") || has("furnace") {
        Wall::Stone
    } else if has("dirt") || has("grass") || has("podzol") || has("sand") || has("gravel") || has("clay") || has("mud") || has("farmland") {
        Wall::Earth
    } else if has("wool") || has("leaves") || has("hay") || has("thatch") {
        Wall::Hide
    } else {
        Wall::Plank
    }
}

fn open(state: u32) -> bool {
    state_flags(state) & abi::block_flags::SOLID == 0
}

/// The room `feet` stands in, if it is one.
pub fn room(w: &World, feet: [i32; 3]) -> Option<Room> {
    let mut seen: HashSet<[i32; 3]> = HashSet::new();
    let mut todo: VecDeque<[i32; 3]> = VecDeque::new();
    for start in [feet, [feet[0], feet[1] + 1, feet[2]]] {
        if open(w.block(start)) && seen.insert(start) {
            todo.push_back(start);
        }
    }
    let mut room = Room::default();
    const SIDES: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    while let Some(p) = todo.pop_front() {
        if seen.len() > ROOM_MAX || w.sky_light(p) >= 15 {
            return None;
        }
        for d in SIDES {
            let q = [p[0] + d[0], p[1] + d[1], p[2] + d[2]];
            let state = w.block(q);
            if open(state) {
                if seen.insert(q) {
                    todo.push_back(q);
                }
            } else {
                let wall = wall_of(state);
                if let Some(i) = WALLS.iter().position(|&x| x == wall) {
                    room.faces[i] += 1;
                }
            }
        }
    }
    room.volume = seen.len();
    (room.volume > 0).then_some(room)
}

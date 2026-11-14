// use crate::worldgen::tree_kinds;

use crate::worldgen::util::sphere;
use crate::worldgen::{TreePlaceData, register_tree};

use weft_sdk::*;

use crate::blocks::*;

use weft_sdk::noise::*;


struct TreeSkeleton {
    height: i32,
    top: (i32, i32, i32),
    bottom: (i32, i32, i32),
    branch_tips: Vec<(i32, i32, i32)>,
}
// TODO
// struct TreeAge {
//     trunk_height: &mut dyn FnMut(&mut Rng) -> u16,
//     allow_branching: bool,
//     thin_trunk: bool,
// }
// impl TreeAge {
//     const YOUNG = Config {
//         trunk_height: |rng| 4u16 + (rng.next() as u16) % 2u16,
//         allow_branching: false,
//         thin_trunk: true,
//     };
//     const MATURE = Config {
//         trunk_height: |rng| 6u16 + (rng.next() as u16) % 3u16,
//         allow_branching: false,
//         thin_trunk: false,
//     };
//     const OLD = Config {
//         trunk_height: |rng| 6u16 + (rng.next() as u16) % 3u16,
//         allow_branching: true,
//         thin_trunk: false,
//     };
//     const ALL = [&YOUNG, &MATURE, &OLD];
// }

const DIRECTIONS: [(i32, i32, &str); 4] = [(1, 0, "east"), (-1, 0, "west"), (0, 1, "south"), (0, -1, "north")];
fn random_direction(rng: &mut Rng) -> (i32, i32, &str) {
    DIRECTIONS[rng.next() as usize % 4usize]
}
fn place_trunk(
    d: &mut TreePlaceData,
    height: u16,
    log_block: &Block,
    wood_block: &Block,
    allow_branches: bool,
    allow_roots: bool,
) -> TreeSkeleton {
    let (x, y, z) = d.pos;
    let top = y + height as i32;
    let log = log_block.id();
    let mut branch_tips: Vec<(i32, i32, i32)> = Vec::new();
    for y in y..top {
        (d.place)(x, y, z, log, true);
        if !allow_branches {
            continue;
        }
        let left = top - y;
        if left < 5 && left > 1 && (d.rng.next() % (left as u64 + 8u64)) <= 2 {
            let roll = d.rng.float();
            if roll < 0.25 {
                (d.place)(x + 1, y, z, log_block.with(&[("axis", "x")]), true);
                (d.place)(x + 2, y, z, log_block.with(&[("axis", "x")]), true);
                branch_tips.push((x + 2, y, z));
            } else if roll < 0.5 {
                (d.place)(x - 1, y, z, log_block.with(&[("axis", "x")]), true);
                (d.place)(x - 2, y, z, log_block.with(&[("axis", "x")]), true);
                branch_tips.push((x - 2, y, z));
            } else if roll < 0.75 {
                (d.place)(x, y, z + 1, log_block.with(&[("axis", "z")]), true);
                (d.place)(x, y, z + 2, log_block.with(&[("axis", "z")]), true);
                branch_tips.push((x, y, z + 2));
            } else {
                (d.place)(x, y, z - 1, log_block.with(&[("axis", "z")]), true);
                (d.place)(x, y, z - 2, log_block.with(&[("axis", "z")]), true);
                branch_tips.push((x, y, z - 2));
            }
        }
    }
    if allow_roots {
        let mut grow_root = |dx: i32, dz: i32, tall: bool| {
            let wood = wood_block.with(&[(
                "facing",
                match (dx, dz) {
                    (1, 0) => "east",
                    (-1, 0) => "west",
                    (0, 1) => "south",
                    (0, -1) => "north",
                    _ => return,
                },
            )]);
            let x = x + dx;
            let z = z + dz;
            if tall {
                (d.place)(x, y + 1, z, wood, true);
            }
            (d.place)(x, y, z, wood, true);
            (d.place)(x, y - 1, z, log, true);
        };
        let root_positions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut root_count = 0;
        for dpos in root_positions {
            if d.rng.float() < 0.666 {
                grow_root(dpos.0, dpos.1, d.rng.float() < 0.5);
                root_count += 1;
            }
        }
        if root_count == 0 {
            let root_pos = root_positions[d.rng.next() as usize % root_positions.len()];
            grow_root(root_pos.0, root_pos.1, d.rng.float() < 0.5);
        }
        (d.place)(x, y - 1, z, log, true);
        (d.place)(x, y - 2, z, DIRT.id(), true);
    }

    TreeSkeleton { height: top - y, top: (x, top, z), bottom: (x, y, z), branch_tips }
}
fn place_cone_foliage(d: &mut TreePlaceData, height: u16, leaves: u32) {
    let (x, y, z) = d.pos;
    for dy in 2..=(height as i32) + 1 {
        let y = y + dy;
        let r = if dy == (height as i32) + 1 { 0 } else { 1 + (((height as i32) + 1 - dy) % 3).min(2) };
        for dz in -r..=r {
            for dx in -r..=r {
                if dx * dx + dz * dz <= r * r + 1 {
                    (d.place)(x + dx, y, z + dz, leaves, false);
                }
            }
        }
    }
}
fn place_foliage(skeleton: &TreeSkeleton, d: &mut TreePlaceData, _height: u16, leaves: u32) {
    fn leaf_kept(rx: f32, ry: f32, rz: f32, dx: i32, dy: i32, dz: i32, wx: i32, wy: i32, wz: i32, salt: u64) -> bool {
        let q = (dx as f32 / rx).powi(2) + (dy as f32 / ry).powi(2) + (dz as f32 / rz).powi(2);
        if q > 1.0 {
            return false;
        }
        let d = q.sqrt();
        let shell = 1.0 / rx.min(ry).min(rz).max(1.0);
        let core = (1.0 - shell).max(0.0);
        if d <= core {
            return true;
        }
        let t = ((d - core) / (1.0 - core).max(0.001)).clamp(0.0, 1.0);
        let keep = 0.95 - 0.55 * t;
        let fine = unit(hash(salt, wx as i64, wy as i64, wz as i64)) as f32;
        let coarse = unit(hash(salt ^ 0xBEEF, (wx / 2) as i64, (wy / 2) as i64, (wz / 2) as i64)) as f32;
        fine < keep || coarse < 0.20
    }
    let (x, y, z) = skeleton.top;
    let rx = skeleton.height as f32 * 0.66 + d.rng.float() - 0.5;
    let ry = rx * 0.75;
    let rz = rx;
    let salt = hash(0x1EAF, x as i64, y as i64, z as i64);
    for (dx, dy, dz) in sphere(rx) {
        let bx = x + dx;
        let by = y + dy;
        let bz = z + dz;
        if leaf_kept(rx, ry, rz, dx, dy, dz, bx, by, bz, salt) {
            (d.place)(bx, by, bz, leaves, false);
        }
    }
    for (x, y, z) in &skeleton.branch_tips {
        let rx = 2.0 + d.rng.float();
        let ry = rx * 0.75;
        let rz = rx;
        for (dx, dy, dz) in sphere(rx) {
            let bx = x + dx;
            let by = y + dy;
            let bz = z + dz;
            if leaf_kept(rx, ry, rz, dx, dy, dz, bx, by, bz, salt) {
                (d.place)(bx, by, bz, leaves, false);
            }
        }
    }
}
fn moss_on_side(side: Dir) -> u32 {
    let props: Vec<(&str, &str)> = Dir::ALL
        .iter()
        .map(|&d| (d.name(), if d == side { "true" } else { "false" }))
        .collect();
    MOSS.with(&props)
}
fn decorate_trunk(skeleton: &TreeSkeleton, d: &mut TreePlaceData) {
    let mossy = d.rng.float() < 0.25;
    for y in skeleton.bottom.1..skeleton.top.1 {
        if d.rng.float() >= 0.5 {
            continue;
        }
        if mossy {
            let bz = skeleton.bottom.2 - 1;
            let state = moss_on_side(Dir::South);
            if state != u32::MAX {
                (d.place)(skeleton.bottom.0, y, bz, state, false);
            }
            continue;
        }
        if d.rng.float() >= 0.25 {
            continue;
        }
        let (dx, dz, facing) = random_direction(d.rng);
        let bx = skeleton.bottom.0 + dx;
        let bz = skeleton.bottom.2 + dz;
        let state = DRYADS_SADDLE_MUSHROOM.with(&[("facing", facing)]);
        (d.place)(bx, y, bz, state, false);
    }
}
fn place_oak(data: &mut TreePlaceData) {
    let height = 6u16 + (data.rng.next() as u16) % 3u16;
    let skeleton = place_trunk(data, height, &OAK_LOG, &OAK_WOOD, true, true);
    place_foliage(&skeleton, data, height, OAK_LEAVES.id());
    decorate_trunk(&skeleton, data);
}
fn place_birch(data: &mut TreePlaceData) {
    let height = 6u16 + (data.rng.next() as u16) % 4u16;
    let skeleton = place_trunk(data, height, &BIRCH_LOG, &BIRCH_WOOD, true, false);
    place_foliage(&skeleton, data, height, BIRCH_LEAVES.id());
    decorate_trunk(&skeleton, data);
}
fn place_spruce(data: &mut TreePlaceData) {
    let height = 6u16 + (data.rng.next() as u16) % 4u16;
    place_trunk(data, height, &SPRUCE_LOG, &SPRUCE_WOOD, false, false);
    place_cone_foliage(data, height, SPRUCE_LEAVES.id());
}
pub fn register() {
    register_tree("base:oak", place_oak);
    register_tree("base:birch", place_birch);
    register_tree("base:spruce", place_spruce);
}

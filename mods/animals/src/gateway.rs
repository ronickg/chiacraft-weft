//! The way to the Cretaceous (`cretaceous.rs`): a gateway built from amber. Amber is fossil tree
//! resin, found in seams in the rock (`amber_ore`, rarely, underground; more often in the
//! Cretaceous); four make an amber block. A frame of amber blocks round an empty space at least
//! 2 wide and 3 tall (up to 21 each), standing upright, lit with a torch, fills with the
//! gateway. Stand in it two seconds and it takes you across, to the same place in the other
//! world: through the gateway there if one is near, or a new one built where you arrive, so the
//! way back is always open.

use base_api::OVERWORLD;
use weft_sdk::*;

use crate::cretaceous::CRETACEOUS;

pub static AMBER: Item = Item::named("amber");
pub static AMBER_ORE: Block = Block::named("amber_ore");
pub static AMBER_BLOCK: Block = Block::named("amber_block");
pub static GATEWAY: Block = Block::named("gateway");
static TORCH: Item = Item::named("base:torch");
static PLAYER: EntityKind = EntityKind::named("core:player");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static NATURAL: CreativeTab = CreativeTab::named("base:natural");
static BUILDING: CreativeTab = CreativeTab::named("base:building");
static INGREDIENTS: CreativeTab = CreativeTab::named("base:ingredients");

value! {
    #[derive(Default)]
    /// A player's way through gateways: ticks stood in one, and whether they just came through
    /// one (1: on the way, a gateway to find or build where they arrive; 2: arrived, not to go
    /// back until they step out).
    pub struct Crossing { pub ticks: u32, pub state: u32 }
}
static CROSSING: Component<Crossing> = Component::named("crossing");

/// Ticks stood in a gateway before it takes you.
const STAND: u32 = 40;
/// The biggest opening a frame may hold, each way.
const MOST: i32 = 21;
/// How far round an arrival a gateway counts as the one to come out of.
const NEAR: i32 = 24;
/// Sea level, both worlds'.
const SEA: i32 = 63;

pub fn init(r: &mut Registry) {
    r.item(&AMBER).tab(&INGREDIENTS);
    r.block(&AMBER_ORE)
        .sound_group("base:block.stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .needs_tool_level(1)
        .drops("amber", 1)
        .tab(&NATURAL);
    r.block(&AMBER_BLOCK)
        .sound_group("base:block.glass")
        .hardness(1.5)
        .mineable("pickaxe")
        .light(4)
        .tab(&BUILDING)
        .on_use(|u| {
            let w = u.world();
            let held = w.inventory(u.player()).get(w.selected_slot(u.player()));
            held.item == TORCH.id() && light(&w, u.pos().to_array())
        });
    r.block(&GATEWAY)
        .sound_group("base:block.glass")
        .not_solid()
        .translucent()
        .light(11)
        .hardness(0.0)
        .no_item();
    r.shaped(&CRAFTING, "amber_block", 1, &["aa", "aa"], &[('a', "amber")]);
    r.shapeless(&CRAFTING, "amber", 4, &["amber_block"], 0);
    // Seams of it in the rock: rare here, commoner in the Cretaceous.
    r.ore(&AMBER_ORE)
        .dimension(&OVERWORLD)
        .y(0, 64)
        .veins(3)
        .vein_size(4)
        .replace("#stone_ore_replaceables");
    r.ore(&AMBER_ORE)
        .dimension(&CRETACEOUS)
        .y(0, 80)
        .veins(6)
        .vein_size(5)
        .replace("#stone_ore_replaceables");

    r.entity_component(&CROSSING, Crossing::default(), 0);
    r.attach(&PLAYER, &CROSSING);
    r.world_tick(cross);
}

const AIR: u32 = 0;

/// Lights the frame round `at` (an amber block of it): fills its opening with the gateway.
fn light(w: &World, at: [i32; 3]) -> bool {
    match opening(w, at) {
        Some(cells) => {
            for c in &cells {
                w.set_block(*c, GATEWAY.id());
            }
            true
        }
        None => false,
    }
}

/// The opening of an upright amber frame that `at` is part of: an empty rectangle 2..21 wide and
/// 3..21 tall in the x or z plane, with amber blocks all round (corners may be anything).
fn opening(w: &World, at: [i32; 3]) -> Option<Vec<[i32; 3]>> {
    for axis in [0usize, 2] {
        let step = |p: [i32; 3], d: i32| {
            let mut q = p;
            q[axis] += d;
            q
        };
        let up = |p: [i32; 3], d: i32| [p[0], p[1] + d, p[2]];
        for start in [step(at, 1), step(at, -1), up(at, 1), up(at, -1)] {
            if w.block(start) != AIR {
                continue;
            }
            // The empty cells joined to it in this plane, as long as they stay small.
            let mut cells = vec![start];
            let mut i = 0;
            let mut ok = true;
            while i < cells.len() {
                let c = cells[i];
                i += 1;
                for n in [step(c, 1), step(c, -1), up(c, 1), up(c, -1)] {
                    if cells.contains(&n) {
                        continue;
                    }
                    match w.block(n) {
                        AIR => cells.push(n),
                        s if AMBER_BLOCK.is(s) => {}
                        _ => ok = false,
                    }
                }
                if !ok || cells.len() > (MOST * MOST) as usize {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            let (lo_a, hi_a) = (cells.iter().map(|c| c[axis]).min()?, cells.iter().map(|c| c[axis]).max()?);
            let (lo_y, hi_y) = (cells.iter().map(|c| c[1]).min()?, cells.iter().map(|c| c[1]).max()?);
            let (wide, tall) = (hi_a - lo_a + 1, hi_y - lo_y + 1);
            if (2..=MOST).contains(&wide) && (3..=MOST).contains(&tall) && (wide * tall) as usize == cells.len() {
                return Some(cells);
            }
        }
    }
    None
}

/// Every tick: players standing in a gateway long enough go across; arrivals find or build
/// theirs.
fn cross(w: &World) {
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        let feet = st.pos.to_array().map(|v| v.floor() as i32);
        let inside = GATEWAY.is(w.block(feet)) || GATEWAY.is(w.block([feet[0], feet[1] + 1, feet[2]]));
        let Some(c) = w.component_mut(id, &CROSSING) else { continue };
        match c.state {
            1 => {
                if arrive(w, id, feet) {
                    c.state = 2;
                }
                continue;
            }
            2 if inside => continue,
            2 => c.state = 0,
            _ => {}
        }
        c.ticks = if inside { c.ticks + 1 } else { 0 };
        if c.ticks < STAND {
            continue;
        }
        *c = Crossing { ticks: 0, state: 1 };
        let here = name_of(abi::entry_kind::DIMENSION, w.dimension());
        let to = if here == "animals:cretaceous" { &OVERWORLD } else { &CRETACEOUS };
        let ground = to.column(w.seed(), feet[0], feet[2]).height;
        let y = if ground == i32::MIN { 100 } else { ground.max(SEA) + 1 };
        w.teleport(id, Some(to), [feet[0] as f64 + 0.5, y as f64, feet[2] as f64 + 0.5]);
    }
}

/// Where player `id` came out at `feet`: a gateway near to step out of, or one built beside
/// them. False while the place isn't loaded yet.
fn arrive(w: &World, id: u64, feet: [i32; 3]) -> bool {
    if w.block_loaded(feet).is_none() {
        return false;
    }
    if let Some(g) = nearest_gateway(w, feet) {
        // Out beside it.
        let p = beside(w, g).unwrap_or(g);
        w.teleport(id, None, [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5]);
        weft_sdk::info!("gateway: came out of the one at {g:?}, beside it at {p:?}");
        return true;
    }
    build(w, feet);
    weft_sdk::info!("gateway: none near {feet:?}, built one");
    true
}

/// The nearest gateway block within `NEAR` of `at`.
fn nearest_gateway(w: &World, at: [i32; 3]) -> Option<[i32; 3]> {
    let mut best: Option<(i32, [i32; 3])> = None;
    for dy in -NEAR..=NEAR {
        for dz in -NEAR..=NEAR {
            for dx in -NEAR..=NEAR {
                let p = [at[0] + dx, at[1] + dy, at[2] + dz];
                let d = dx * dx + dy * dy + dz * dz;
                if best.is_none_or(|b| d < b.0) && w.block_loaded(p).is_some_and(|s| GATEWAY.is(s)) {
                    best = Some((d, p));
                }
            }
        }
    }
    best.map(|b| b.1)
}

/// A spot beside gateway block `g` with room to stand and ground under it.
fn beside(w: &World, g: [i32; 3]) -> Option<[i32; 3]> {
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    for d in [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]] {
        for dy in -3..=0 {
            let p = [g[0] + d[0], g[1] + dy, g[2] + d[2]];
            if !solid(p) && !solid([p[0], p[1] + 1, p[2]]) && solid([p[0], p[1] - 1, p[2]]) && !GATEWAY.is(w.block(p)) {
                return Some(p);
            }
        }
    }
    None
}

/// A gateway built where a player came out: a frame along x two blocks in front of them, with an
/// opening 2 wide and 3 tall, on a floor of amber (over water too), the space round it cleared.
fn build(w: &World, feet: [i32; 3]) {
    let [x, y, z] = [feet[0], feet[1], feet[2] + 2];
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    for dx in -1..=2 {
        for dz in -2..=1 {
            for dy in 0..=4 {
                if dz != 0 || dy == 4 {
                    w.set_block([x + dx, y + dy, z + dz], AIR);
                }
            }
            // Ground to stand on round it.
            if !solid([x + dx, y - 1, z + dz]) {
                w.set_block([x + dx, y - 1, z + dz], AMBER_BLOCK.id());
            }
        }
    }
    for dx in -1..=2 {
        w.set_block([x + dx, y - 1, z], AMBER_BLOCK.id());
        w.set_block([x + dx, y + 3, z], AMBER_BLOCK.id());
    }
    for dy in 0..3 {
        w.set_block([x - 1, y + dy, z], AMBER_BLOCK.id());
        w.set_block([x + 2, y + dy, z], AMBER_BLOCK.id());
        w.set_block([x, y + dy, z], GATEWAY.id());
        w.set_block([x + 1, y + dy, z], GATEWAY.id());
    }
}

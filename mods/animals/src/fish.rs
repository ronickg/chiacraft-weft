//! Fish (`notes/ECOLOGY.md`, wave one: the middle of the water webs): the brown trout (`BROWN_TROUT`)
//! of cool rivers and lakes and the pacu (`PACU`) of the swamp's and the llanos' waters, and the
//! Cretaceous's lungfish (`LUNGFISH`) of the floodplain and Lycoptera (`LYCOPTERA`) of the fern
//! forest's lakes, four species on one model (`source/fish`), drawn deeper, bigger or smaller by
//! their genes. Wild, never tamed.
//!
//! The engine's natural spawning puts animals on the ground, never in water, so fish come by their
//! own spawner (`spawn`): every two seconds near each player, into water two blocks deep or more in
//! their habitats, as often as their rarity has it there and while few of their kind are near. They
//! swim (`swim`, by `World::fly`): drifting from point to point under the water, darting off from a
//! walker, never swimming out of it; stranded on land, one lies on its side flapping (a look of its
//! own) and suffocates. They breed in their seasons (`court`): in love, one swims to a mate of
//! its kind near, and met, the female spawns; her fry come after the eggs' time
//! (`life::carrying`), near her. The spawner tops up what breeding doesn't.

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

use crate::species::SPECIES;

pub(crate) static BROWN_TROUT: EntityKind = EntityKind::named("brown_trout");
pub(crate) static PACU: EntityKind = EntityKind::named("pacu");
pub(crate) static LUNGFISH: EntityKind = EntityKind::named("lungfish");
pub(crate) static LYCOPTERA: EntityKind = EntityKind::named("lycoptera");
/// The fish's kinds.
static KINDS: [&EntityKind; 4] = [&BROWN_TROUT, &PACU, &LUNGFISH, &LYCOPTERA];
const FLOP: &str = "fish_flop";
/// How many of a kind may swim within `ROOM_RADIUS` blocks before no more come.
const ROOM: usize = 6;
const ROOM_RADIUS: f64 = 32.0;

value! {
    #[derive(Default)]
    /// Where a fish is swimming to (not saved: it picks again).
    pub struct Swim {
        pub to: [f32; 3],
        /// Ticks left to get there (0: pick again), and to keep darting off.
        pub left: u32,
        pub flee: u32,
        /// Whether it lies stranded (the flop look is on).
        pub stranded: bool,
    }
}

static STATE: Component<Swim> = Component::named("fish_state");

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Swim::default(), 0);
    // Stranded on land: on its side, the tail slapping.
    drop(
        r.entity(EntityKind::generated(FLOP.to_string()))
            .size(0.35, 0.15)
            .passive()
            .model_file("fish")
            .animations(FLOP)
            .texture("fish/fish"),
    );
    for kind in KINDS {
        let index = crate::species::of(kind);
        r.entity(kind)
            .size(animals_core::bodies::of("fish").0, animals_core::bodies::of("fish").1)
            .max_health(3.0)
            // Cruising slowly, darting off at a few metres a second.
            .walk_speed(3.0)
            .passive()
            .model_file("fish")
            .animations("fish")
            .texture("fish/fish")
            .attach(&STATE)
            .on_interact(|e| {
                let (w, id, player) = (e.world(), e.entity(), e.other());
                crate::screen::interact(&w, id, player);
                false
            })
            .goals(
                Goals::new()
                    .then(crate::animal::upkeep(index))
                    .then(move |w: &World, id: u64, st: &EntityState| court(w, id, st, index))
                    .then(move |w: &World, id: u64, st: &EntityState| swim(w, id, st, index)),
            );
    }
    r.world_tick(spawn);
}

fn wet(w: &World, at: [f64; 3]) -> bool {
    w.in_liquid(at)
        && block_name(
            w.liquid_in_cell([at[0].floor() as i32, at[1].floor() as i32, at[2].floor() as i32])
                .unwrap_or(0),
        )
        .starts_with("base:water")
}

fn random(w: &World) -> f64 {
    w.random_below(10_000) as f64 / 10_000.0
}

/// Every tick: swim about under the water, dart off from a walker, flop when stranded.
fn swim(w: &World, id: u64, st: &EntityState, index: usize) -> bool {
    let Some(mut s) = w.component(id, &STATE) else { return false };
    let mid = [st.pos.x, st.pos.y + 0.1, st.pos.z];
    if !wet(w, mid) {
        if !s.stranded {
            s.stranded = true;
            w.set_appearance(id, Some(&format!("animals:{FLOP}")));
        }
        // Flapping about, a hop now and then (maybe back into the water), suffocating slowly.
        if w.random_below(20) == 0 {
            let a = random(w) * std::f64::consts::TAU;
            w.walk(id, [a.cos() as f32, a.sin() as f32], 0.6, true);
        }
        if (w.game_time() + id) % 40 == 0 {
            w.damage(id, 1.0, 0, damage_cause::SUFFOCATE);
        }
        set(w, id, s);
        return true;
    }
    if s.stranded {
        s.stranded = false;
        w.set_appearance(id, None);
    }
    // A walker near: dart away (a sneaking player gets close, as on land).
    let shy = SPECIES[index].shy as f64;
    if (w.game_time() + id) % 10 == 0
        && let Some(p) = w
            .players()
            .into_iter()
            .filter_map(|p| w.entity(p))
            .filter(|p| p.pose != abi::pose::CROUCH)
            .find(|p| crate::dist2(p.pos, st.pos) < shy * shy)
    {
        let (x, z) = (st.pos.x - p.pos.x, st.pos.z - p.pos.z);
        let n = (x * x + z * z).sqrt().max(0.01);
        s.to = [(st.pos.x + x / n * 8.0) as f32, st.pos.y as f32, (st.pos.z + z / n * 8.0) as f32];
        (s.left, s.flee) = (60, 40);
    }
    let to = s.to.map(|v| v as f64);
    let d = [to[0] - st.pos.x, to[1] - st.pos.y, to[2] - st.pos.z];
    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let ahead = |k: f64| [st.pos.x + d[0] / dist.max(0.01) * k, mid[1] + d[1] / dist.max(0.01) * k, st.pos.z + d[2] / dist.max(0.01) * k];
    if s.left == 0 || dist < 0.5 || !wet(w, ahead(0.6)) {
        // Somewhere else under the water within a few blocks (a few tries, or hold still).
        let pick = (0..6).find_map(|_| {
            let a = random(w) * std::f64::consts::TAU;
            let r = 1.5 + random(w) * 5.0;
            let p = [st.pos.x + a.cos() * r, st.pos.y + (random(w) - 0.5) * 2.0, st.pos.z + a.sin() * r];
            (wet(w, p) && wet(w, [p[0], p[1] + 0.4, p[2]])).then_some(p)
        });
        s.to = pick.unwrap_or(st.pos.to_array()).map(|v| v as f32);
        s.left = 80 + w.random_below(160);
        s.flee = 0;
        set(w, id, s);
        return true;
    }
    s.left = s.left.saturating_sub(w.tick_elapsed(id) as _);
    s.flee = s.flee.saturating_sub(w.tick_elapsed(id) as _);
    let speed = if s.flee > 0 { 1.0 } else { 0.3 };
    w.fly(id, [(d[0] / dist) as f32, (d[1] / dist) as f32, (d[2] / dist) as f32], speed);
    set(w, id, s);
    true
}

/// In love (`breeding`): a mate of its kind near is where it swims (`swim` takes it there), and
/// met, the female spawns (`breeding::mated`). Never taken: swimming goes on.
fn court(w: &World, id: u64, st: &EntityState, index: usize) -> bool {
    if !(w.game_time() + id).is_multiple_of(10) {
        return false;
    }
    let Some(mate) = crate::breeding::mate_near(w, id, st) else { return false };
    let Some(m) = w.entity(mate) else { return false };
    if crate::dist2(m.pos, st.pos) > 1.5 * 1.5 {
        if let Some(s) = w.component_mut(id, &STATE)
            && s.flee == 0
        {
            s.to = m.pos.to_array().map(|v| v as f32);
            s.left = 20;
        }
        return false;
    }
    if w.component(id, &crate::animal::ANIMAL)
        .is_some_and(|a| a.female == 1)
    {
        crate::breeding::mated(w, id, mate, index);
    }
    false
}

fn set(w: &World, id: u64, s: Swim) {
    if let Some(c) = w.component_mut(id, &STATE) {
        *c = s;
    }
}

/// Every two seconds, near each player: fish into water two blocks deep or more, in their habitats.
fn spawn(w: &World) {
    if w.game_time() % 40 != 0 {
        return;
    }
    let dim = name_of(abi::entry_kind::DIMENSION, w.dimension());
    for p in w.players() {
        let Some(at) = w.entity(p) else { continue };
        for _ in 0..2 {
            let a = random(w) * std::f64::consts::TAU;
            let r = 12.0 + random(w) * 24.0;
            let (x, z) = ((at.pos.x + a.cos() * r).floor() as i32, (at.pos.z + a.sin() * r).floor() as i32);
            let top = at.pos.y.floor() as i32 + 8;
            // The water's surface, scanning down; two blocks deep or more.
            let water = |y: i32| block_name(w.liquid_in_cell([x, y, z]).unwrap_or(0)).starts_with("base:water");
            let Some(y) = (top - 32..top).rev().find(|&y| water(y) && water(y - 1)) else { continue };
            let spot = [x as f64 + 0.5, (y - 1) as f64 + 0.2, z as f64 + 0.5];
            let biome = name_of(abi::entry_kind::BIOME, w.biome([x, y - 1, z]));
            for kind in KINDS {
                let s = &SPECIES[crate::species::of(kind)];
                // In its own world only (`habitats::world`): the two share their seas.
                if crate::habitats::world(s) != dim {
                    continue;
                }
                if random(w) as f32 >= s.spawn_chance(biome) || w.entities_near(spot, ROOM_RADIUS, Some(kind)).len() >= ROOM {
                    continue;
                }
                let (few, most) = s.bio.social.group();
                for _ in 0..few + w.random_below(most - few + 1) {
                    w.spawn(kind, [spot[0] + (random(w) - 0.5) * 0.8, spot[1], spot[2] + (random(w) - 0.5) * 0.8]);
                }
            }
        }
    }
}

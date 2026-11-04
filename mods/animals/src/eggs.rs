//! Eggs and nests (`notes/ANIMALS.md`, "Birth"), for the species that lay (`Species::gestation`
//! 0: the raptors). An egg holds its young from the moment it is laid (`Egg`: genes, family,
//! wild points) and hatches after its species' incubation, as long as the air at it (the
//! player's, with the heat of fires near by) is between 25 and 40 °C: a nest holds it closer to a
//! brooding parent's 37 °C and makes it twice as fast.
//!
//! Wild females now and then build a nest (`NEST`, an earthen scrape like their relatives', a prop
//! anyone can also make from sticks and wheat and place) and lay a clutch in it, fathered by a male of their place's gene pool. A player can
//! take an egg (right click: it becomes that species' egg item, holding the egg) and the wild
//! ones of its kind near by go for the thief; placed again (on a nest, or anywhere), it hatches
//! into a young that is the placer's: taming by raising. A tamed mother lays into a nest within
//! eight blocks if there is one.

use std::sync::OnceLock;

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::animal::{ANIMAL, OWNER};
use crate::breeding::{self, BREEDING, Young};
use crate::coats::{self, GENOME};
use crate::species::SPECIES;
use animals_core::life;

value! {
    #[derive(Default)]
    /// An egg's young (decided when it was laid) and how warm it has been kept; saved, and kept
    /// on the egg item while carried.
    pub struct Egg {
        /// The young's species + 1.
        pub species: u32,
        /// Ticks of warmth left before it hatches.
        pub left: u32,
        pub generation: u32,
        pub mother: u64,
        pub father: u64,
        pub genes: [u32; 10],
        pub dam: [u32; 10],
        pub sire: [u32; 10],
        pub wild: [u8; 9],
        /// The rest of `genes`, `dam` and `sire` (`breeding::pack`).
        pub genes_more: breeding::More,
        pub dam_more: breeding::More,
        pub sire_more: breeding::More,
    }
}

pub(crate) static EGG: Component<Egg> = Component::named("egg_young");
/// The size an egg is drawn at (0: not since it loaded).
static EGG_SCALED: Component<f32> = Component::named("egg_scaled");
pub static EGG_ENTITY: Entity = Entity::named("egg");
pub static NEST: Entity = Entity::named("nest");
static NEST_ITEM: Item = Item::named("nest");
static AIR: Block = Block::named("core:air");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
/// Each laying species' egg item, by its row in `SPECIES` (made once).
static ITEMS: OnceLock<Vec<(usize, &'static Item)>> = OnceLock::new();

/// How egg size follows body size: a species twice as tall lays eggs about half again as long
/// (eggs grow more slowly than bodies).
const EGG_GROWTH: f32 = 0.6;
/// A wild female builds a nest about every two days, while her kind isn't crowded.
const NESTING: u32 = 48_000;
const CROWDED: usize = 8;
/// How far the wild ones of its kind hear an egg being taken.
const GUARD_RANGE: f64 = 24.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&EGG, Egg::default(), component_flags::SAVE);
    r.entity_component(&EGG_SCALED, 0.0f32, 0);
    // An egg: a Velociraptor's, built from `source/egg` (elongated, standing on its pointed end,
    // blue-green with brown blotches like its relative Deinonychus's); a bigger species' scaled up.
    r.entity(&EGG_ENTITY)
        .no_egg()
        .size(0.1, 0.14)
        .health(4.0)
        .speed(0.0)
        .passive()
        .unpushable()
        .model_file("egg")
        .texture("egg/egg")
        .attach(&EGG)
        .attach(&EGG_SCALED)
        .attach(&OWNER)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            take(&w, id, player);
        })
        .goals(Goals::new().then(incubate));
    // A nest: an earthen scrape in a raised rim with dry grass, built from `source/nest`; a prop,
    // placed with its item, broken back into it. Right clicked, it gives up an egg, or takes the
    // egg in hand.
    r.entity(&NEST)
        .no_egg()
        .size(NEST_WIDTH, 0.15)
        .health(6.0)
        .speed(0.0)
        .passive()
        .unpushable()
        .model_file("nest")
        .texture("nest/nest")
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            nest_clicked(&w, id, player);
        })
        .on_death(|w, d| w.spawn_item([d.pos[0], d.pos[1] + 0.2, d.pos[2]], ItemStack::new(NEST_ITEM.id(), 1)));
    r.item(&NEST_ITEM).tab(&TOOLS).on_use(|u| {
        let Some((p, face)) = u.target() else { return false };
        let side = [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]][face as usize % 6];
        let w = u.world();
        w.spawn(&NEST, [(p[0] + side[0]) as f64 + 0.5, (p[1] + side[1]) as f64, (p[2] + side[2]) as f64 + 0.5]);
        if !w.creative(u.player()) {
            u.set_stack(StackData::shrink(u.stack(), 1));
        }
        true
    });
    r.shaped(&CRAFTING, "nest", 1, &["s s", "sws"], &[('s', "base:stick"), ('w', "base:wheat")]);
    let mut items = Vec::new();
    for (i, s) in SPECIES
        .iter()
        .enumerate()
        .filter(|(_, s)| s.gestation() == 0)
    {
        let item = Item::generated(format!("{}_egg", s.kind_name()));
        r.item(item)
            .max_stack(1)
            .texture("raptor_egg")
            .tab(&TOOLS)
            .on_use(place);
        items.push((i, item));
    }
    let _ = ITEMS.set(items);
}

fn item_of(index: usize) -> Option<&'static Item> {
    ITEMS
        .get()?
        .iter()
        .find(|(i, _)| *i == index)
        .map(|(_, it)| *it)
}

/// Where a nest's eggs stand: on the floor of its dish (the slabs round its middle are 1.5 to
/// 2.5 units up, `source/nest`), pushed a little into it.
const NEST_FLOOR: f64 = 1.0 / 64.0;
/// How far an egg sits in the ground, as its relatives laid them (a quarter of a Velociraptor
/// egg's 10 units, more for bigger eggs); in a nest, into its soft floor, less.
const SINK: f64 = 2.5 / 64.0;

/// A free spot in the nest at `n`: the middle, else round it.
fn nest_spot(w: &World, n: [f64; 3]) -> [f64; 3] {
    let k = nest_scale(w, n);
    let spots = std::iter::once((0.0, 0.0)).chain((0..7).map(|i| {
        let a = i as f64 / 7.0 * std::f64::consts::TAU;
        (a.cos() * 0.2 * k, a.sin() * 0.2 * k)
    }));
    let mut first = None;
    for (dx, dz) in spots {
        let at = [n[0] + dx, n[1] + NEST_FLOOR * k, n[2] + dz];
        first.get_or_insert(at);
        if w.near(at, 0.1 * k, Some(&EGG_ENTITY)).is_empty() {
            return at;
        }
    }
    first.unwrap_or(n)
}

/// An egg of species `index`'s size against the Velociraptor's (eggs grow more slowly than
/// bodies).
pub(crate) fn egg_size(index: usize) -> f32 {
    (SPECIES[index].height.0 / SPECIES[crate::species::of(&crate::raptor::RAPTOR)].height.0).powf(EGG_GROWTH)
}

/// A nest's width.
const NEST_WIDTH: f32 = 0.9;
/// A bigger species' nest is its eggs' size bigger (a wyvern's, about 3 m across).
fn nest_size(index: usize) -> f32 {
    egg_size(index).max(1.0)
}

/// How much bigger than the plain one the nest at `at` is (1 for none).
fn nest_scale(w: &World, at: [f64; 3]) -> f64 {
    nest_near(w, at, 0.1)
        .and_then(|(n, _)| w.entity(n))
        .map_or(1.0, |e| (e.size[0] / NEST_WIDTH) as f64)
}

/// The nest within `r` blocks of `at`, if any: its id and position.
fn nest_near(w: &World, at: [f64; 3], r: f64) -> Option<(u64, [f64; 3])> {
    w.near(at, r, Some(&NEST))
        .into_iter()
        .find_map(|n| Some((n, w.entity(n)?.pos)))
}

/// Whether `at` is in a nest.
fn in_nest(w: &World, at: [f64; 3]) -> bool {
    w.near(at, 2.5, Some(&NEST)).into_iter().any(|n| {
        w.entity(n).is_some_and(|e| {
            let r = 0.6 * (e.size[0] / NEST_WIDTH) as f64;
            (e.pos[0] - at[0]).powi(2) + (e.pos[2] - at[2]).powi(2) < r * r
        })
    })
}

/// A player right clicks nest `id`: holding an egg, they put it in; else they take one out.
fn nest_clicked(w: &World, id: u64, player: u64) {
    let Some(st) = w.entity(id) else { return };
    let held = w.inventory(player).get(w.selected_slot(player));
    if let Some(e) = EGG.on_stack(&held).filter(|_| {
        ITEMS
            .get()
            .is_some_and(|i| i.iter().any(|(_, it)| it.id() == held.item))
    }) {
        spawn_egg(w, nest_spot(w, st.pos), &e, owner_of(player));
        let slot = w.selected_slot(player);
        w.inventory(player).set(slot, StackData::shrink(held, 1));
        return;
    }
    if let Some(egg) = w.near(st.pos, 0.6, Some(&EGG_ENTITY)).into_iter().next() {
        take(w, egg, player);
    }
}

/// A player as an owner (their key), if they have one.
fn owner_of(player: u64) -> Option<[u8; 64]> {
    let key = crate::player_key(player);
    let mut owner = [0u8; 64];
    (key.len() == 64).then(|| {
        owner.copy_from_slice(key.as_bytes());
        owner
    })
}

// ------------------------------------------------------------------ laying

/// A clutch (`young`) is laid by mother `mother` at `at`: into a nest within eight blocks if
/// there is one, else around her.
pub fn lay(w: &World, at: [f64; 3], young: &[Young], owner: Option<[u8; 64]>) {
    let nest = nest_near(w, at, 8.0).map(|n| n.1);
    let n = young.len().max(1) as f64;
    for (k, y) in young.iter().enumerate() {
        let a = k as f64 / n * std::f64::consts::TAU;
        let spot = match nest {
            Some(p) => nest_spot(w, p),
            None => {
                let r = 0.25 * egg_size(y.index) as f64;
                [at[0] + a.cos() * r, at[1], at[2] + a.sin() * r]
            }
        };
        let ((genes, genes_more), (dam, dam_more), (sire, sire_more)) =
            (breeding::pack(&y.genome), breeding::pack(&y.dam), breeding::pack(&y.sire));
        let egg = Egg {
            species: y.index as u32 + 1,
            left: SPECIES[y.index].incubation(),
            generation: y.generation,
            mother: y.mother,
            father: y.father,
            genes,
            dam,
            sire,
            wild: y.wild,
            genes_more,
            dam_more,
            sire_more,
        };
        spawn_egg(w, spot, &egg, owner);
    }
}

/// An egg put down at `at`: a quarter sunk into the ground, leaning (its model leans forward) with
/// its top toward the nest's middle, or any way outside a nest.
fn spawn_egg(w: &World, at: [f64; 3], egg: &Egg, owner: Option<[u8; 64]>) -> u64 {
    let size = (egg.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
        .map_or(1.0, egg_size);
    let sink = if nest_near(w, at, 0.8).is_some() { 0.0 } else { SINK * size as f64 };
    let id = w.spawn(&EGG_ENTITY, [at[0], at[1] - sink, at[2]]);
    if id == 0 {
        return 0;
    }
    if let Some(mut st) = w.entity(id) {
        // Yaw 0 faces -z; the model leans toward where it faces.
        st.yaw = match nest_near(w, at, 0.8) {
            Some((_, n)) if (n[0] - at[0]).hypot(n[2] - at[2]) > 0.05 => (-(n[0] - at[0])).atan2(-(n[2] - at[2])) as f32,
            _ => w.random_below(628) as f32 / 100.0,
        };
        w.set_entity(id, &st);
    }
    if let Some(e) = w.component_mut(id, &EGG) {
        *e = *egg;
    }
    if let (Some(o), Some(own)) = (owner, w.component_mut(id, &OWNER)) {
        *own = o;
    }
    id
}

/// A goal for wild females of laying species `index`, now and then (about every two days, while
/// their kind isn't crowded and there is no nest near): build a nest where she stands and lay a
/// clutch in it, fathered by a male of her place's pool.
pub fn nest(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        if w.random_below(NESTING) != 0 {
            return false;
        }
        wild_clutch(w, id, index, st);
        false
    }
}

/// Wild female `id` builds a nest and lays in it, if she can: grown, wild, not expecting, her
/// kind not crowded, no nest near, room at her feet. True if she did.
pub fn wild_clutch(w: &World, id: u64, index: usize, st: &EntityState) -> bool {
    let s = &SPECIES[index];
    let (Some(a), Some(mother)) = (w.component(id, &ANIMAL).copied(), w.component(id, &GENOME).copied()) else { return false };
    if crate::animal::tamed(w, id) || a.female == 0 || breeding::young(w, id) {
        return false;
    }
    if w.near(st.pos, 32.0, Some(s.kind)).len() >= CROWDED || nest_near(w, st.pos, 24.0).is_some() {
        return false;
    }
    let feet = st.pos.map(|v| v.floor() as i32);
    if w.get(feet) != AIR.id() || w.get([feet[0], feet[1] - 1, feet[2]]) == AIR.id() {
        return false;
    }
    let at = [feet[0] as f64 + 0.5, feet[1] as f64, feet[2] as f64 + 0.5];
    let nest = w.spawn(&NEST, at);
    if nest_size(index) > 1.0 {
        w.set_scale(nest, nest_size(index));
    }
    let mut r = coats::world_rng(w);
    let Some(pool) = crate::evolution::pool_at(w, index, st.pos) else { return false };
    let Some(father) = coats::sample(index, false, &pool, &mut r) else { return false };
    let generation = w.component(id, &BREEDING).map_or(0, |b| b.generation) + 1;
    let n = s.litter().0 + w.random_below(s.litter().1 - s.litter().0 + 1);
    let young: Vec<Young> = (0..n)
        .filter_map(|_| {
            Some(Young {
                index,
                genome: coats::breed(&mother, &father, &mut r)?,
                mother: id,
                father: 0,
                generation,
                dam: mother,
                sire: father,
                wild: a.wild,
            })
        })
        .collect();
    lay(w, at, &young, None);
    weft_sdk::info!("wild {} {id} nested at {feet:?}: {n} eggs", s.kind_name());
    true
}

// ------------------------------------------------------------------ incubating and hatching

/// An egg's goal: drawn at its species' size; developing while the air at it is right (twice as
/// fast in a nest; not at all too cold or too hot); hatching.
fn incubate(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(e) = w.component(id, &EGG).copied() else { return true };
    let Some(index) = (e.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
    else {
        return true;
    };
    let size = egg_size(index);
    if w.component(id, &EGG_SCALED).is_some_and(|&v| v != size) {
        w.set_scale(id, size);
        if let Some(v) = w.component_mut(id, &EGG_SCALED) {
            *v = size;
        }
    }
    if (w.time() + id) % 20 != 0 {
        return true;
    }
    let nest = in_nest(w, st.pos);
    // The air itself: wind chill is what a body loses, and an egg never falls below the air (B5).
    let air = base_api::air_at(w, st.pos, 0.3, st.in_liquid != 0, vitals::temperature::FIRE_REACH, |_| {}).still;
    let step = match life::develops(life::egg_air(air, nest)) {
        false => 0,
        true if nest => 40,
        true => 20,
    };
    let left = e.left.saturating_sub(step * breeding::speed());
    if let Some(e) = w.component_mut(id, &EGG) {
        e.left = left;
    }
    if left == 0 {
        let owner = w.component(id, &OWNER).copied().filter(|o| o[0] != 0);
        if let Some(y) = young_of(&e, index) {
            w.remove(id);
            breeding::born(w, st.pos, &y, owner);
        }
    }
    true
}

fn young_of(e: &Egg, index: usize) -> Option<Young> {
    Some(Young {
        index,
        genome: breeding::unpack(&e.genes, &e.genes_more)?,
        mother: e.mother,
        father: e.father,
        generation: e.generation,
        dam: breeding::unpack(&e.dam, &e.dam_more).unwrap_or_default(),
        sire: breeding::unpack(&e.sire, &e.sire_more).unwrap_or_default(),
        wild: e.wild,
    })
}

// ------------------------------------------------------------------ taking and placing

/// A player right clicks egg `id`: they take it (its species' egg item, holding it). Taken from
/// the wild (no owner), the wild ones of its kind near by go for them.
fn take(w: &World, id: u64, player: u64) {
    let Some(e) = w.component(id, &EGG).copied() else { return };
    let Some(index) = (e.species as usize).checked_sub(1) else { return };
    let Some(item) = item_of(index) else { return };
    let Some(st) = w.entity(id) else { return };
    let wild = w.component(id, &OWNER).is_none_or(|o| o[0] == 0);
    w.remove(id);
    let stack = ItemStack::new(item.id(), 1).with(&EGG, e);
    let left = w.inventory(player).insert(stack);
    if !left.is_empty() {
        w.spawn_item(st.pos, left);
    }
    if wild {
        let guards: Vec<u64> = w
            .near(st.pos, GUARD_RANGE, Some(SPECIES[index].kind))
            .into_iter()
            .filter(|&g| !crate::animal::tamed(w, g) && !breeding::young(w, g))
            .collect();
        for g in &guards {
            crate::knockout::anger(w, *g, player);
        }
        weft_sdk::info!("egg {id} taken by {player}: {} guards angry", guards.len());
    }
}

/// An egg item used on a block: the egg is put there (in the nest, if it is one), and it hatches
/// into a young that is the placer's.
fn place(u: &mut ItemUse) -> bool {
    let Some((p, face)) = u.target() else { return false };
    let Some(e) = EGG.on_stack(&u.stack()) else { return false };
    let (w, player) = (u.world(), u.player());
    let side = [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]][face as usize % 6];
    let mut at = [(p[0] + side[0]) as f64 + 0.5, (p[1] + side[1]) as f64, (p[2] + side[2]) as f64 + 0.5];
    // Into a nest there.
    if let Some((_, n)) = nest_near(&w, at, 0.8) {
        at = nest_spot(&w, n);
    }
    spawn_egg(&w, at, &e, owner_of(player));
    // Used up even in creative: an egg is one of a kind.
    u.set_stack(StackData::shrink(u.stack(), 1));
    true
}

/// `/animal hurry` (for testing): every egg near `at` hatches within a second.
pub fn hurry(w: &World, at: [f64; 3]) {
    for id in w.near(at, 16.0, Some(&EGG_ENTITY)) {
        if let Some(e) = w.component_mut(id, &EGG) {
            e.left = e.left.min(20);
        }
    }
}

/// `/animal nest` (for testing): the nearest wild female of a laying species builds her nest
/// now.
pub fn nest_now(w: &World, player: u64) -> bool {
    let Some(p) = w.entity(player) else { return false };
    for id in w.near(p.pos, 16.0, None) {
        let Some(index) = w
            .component(id, &ANIMAL)
            .and_then(|a| (a.species as usize).checked_sub(1))
        else {
            continue;
        };
        if SPECIES.get(index).is_some_and(|s| s.gestation() == 0)
            && let Some(st) = w.entity(id)
            && wild_clutch(w, id, index, &st)
        {
            return true;
        }
    }
    false
}

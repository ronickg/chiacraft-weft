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
//!
//! Birds (`animals_core::nests`) build their nest at their kind of site when they lay
//! (`lay_in_nest`): on a treetop's leaves, a high bare ledge, or the ground, and brood it: in a
//! bird's nest the eggs are held at a parent's warmth whatever the air, as a sitting bird does.

use std::sync::OnceLock;

use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

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
/// Each egg look's entity kind (`animals_core::eggs::LOOKS`: a raptor's, a crocodile's, roe...),
/// by the look's name (made once).
static KINDS: OnceLock<Vec<(&'static str, &'static EntityKind)>> = OnceLock::new();
pub static NEST: EntityKind = EntityKind::named("nest");
static NEST_ITEM: Item = Item::named("nest");
static AIR: Block = Block::named("core:air");
static LEAVES: Tag = Tag::named("leaves");
static LOGS: Tag = Tag::named("logs");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
/// Each laying species' egg item, by its row in `SPECIES` (made once).
static ITEMS: OnceLock<Vec<(usize, &'static Item)>> = OnceLock::new();

/// The egg a plain nest is made for: its box's width, blocks (a little under a Velociraptor's
/// drawn egg; a wider egg's nest is as much bigger, so a ring of seven doesn't overlap), and the
/// ring a nest's eggs lie on round the middle one, blocks at the plain nest's size.
const NEST_EGG_W: f32 = 0.2;
const NEST_RING: f64 = 0.25;
/// How far out round a full nest more eggs are laid, rings of the dish (on the ground beside it).
const NEST_BESIDE: f64 = 2.6;
/// The egg an egg's mass is given at, real cm (`hunt::EGG_KG`).
const MASS_EGG_CM: f32 = 15.0;
/// A wild female builds a nest about every two days, while her kind isn't crowded.
const NESTING: u32 = 48_000;
const CROWDED: usize = 8;
/// How far the wild ones of its kind hear an egg being taken.
const GUARD_RANGE: f64 = 24.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&EGG, Egg::default(), component_flags::SAVE);
    r.entity_component(&EGG_SCALED, 0.0f32, 0);
    // Eggs: one kind per look (`animals_core::eggs`, built from `source/egg`): a raptor's long
    // blue-green speckled egg stood on end, a Citipati's ridged teal one, a caiman's banded
    // capsule, roe, the wyvern's scaled stone... each species' own drawn at its egg's length.
    let mut kinds = Vec::new();
    for look in animals_core::eggs::LOOKS {
        let kind = EntityKind::generated(format!("egg_{}", look.name));
        let (bw, bh) = animals_core::bodies::of(look.model);
        r.entity(kind)
            .no_egg()
            .size(bw, bh)
            .max_health(4.0)
            .walk_speed(0.0)
            .passive()
            .unpushable()
            .model_file(look.model)
            .texture(&format!("egg/{}", look.name))
            .attach(&EGG)
            .attach(&EGG_SCALED)
            .attach(&OWNER)
            .on_interact(|e| {
                let (w, id, player) = (e.world(), e.entity(), e.other());
                take(&w, id, player);
                false
            })
            .goals(Goals::new().then(incubate));
        kinds.push((look.name, kind));
    }
    let _ = KINDS.set(kinds);
    // A nest: an earthen scrape in a raised rim with dry grass, built from `source/nest`; a prop,
    // placed with its item, broken back into it. Right clicked, it gives up an egg, or takes the
    // egg in hand.
    r.entity(&NEST)
        .no_egg()
        .size(animals_core::bodies::of("nest").0, animals_core::bodies::of("nest").1)
        .max_health(6.0)
        .walk_speed(0.0)
        .passive()
        .unpushable()
        .model_file("nest")
        .texture("nest/nest")
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            nest_clicked(&w, id, player);
            false
        })
        .on_died(|w, d| {
            w.spawn_item([d.pos.x, d.pos.y + 0.2, d.pos.z], ItemStack::new(NEST_ITEM.id(), 1));
        });
    r.item(&NEST_ITEM).tab(&TOOLS).on_use(|u| {
        let Some((p, face)) = u.target_block().map(|(p, f)| (p.to_array(), f)) else { return false };
        let side = [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]][face as usize % 6];
        let w = u.world();
        w.spawn(&NEST, [(p[0] + side[0]) as f64 + 0.5, (p[1] + side[1]) as f64, (p[2] + side[2]) as f64 + 0.5]);
        if !w.outside_survival(u.player()) {
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
        let look = animals_core::eggs::of(s.name).map_or("dromaeosaur", |(l, _)| l.name);
        r.item(item)
            .max_stack(1)
            .texture(&format!("egg_{look}"))
            .tab(&TOOLS)
            .on_use(place);
        items.push((i, item));
    }
    let _ = ITEMS.set(items);
}

/// Species `index`'s egg kind (its look's).
fn kind_of(index: usize) -> Option<&'static EntityKind> {
    let (look, _) = animals_core::eggs::of(SPECIES[index].name)?;
    KINDS
        .get()?
        .iter()
        .find(|(n, _)| *n == look.name)
        .map(|(_, k)| *k)
}

/// The eggs within `r` of `at`, any species'.
fn eggs_near(w: &World, at: [f64; 3], r: f64) -> Vec<u64> {
    w.entities_near(at, r, None)
        .into_iter()
        .filter(|&id| w.component(id, &EGG).is_some_and(|e| e.species != 0))
        .collect()
}

fn item_of(index: usize) -> Option<&'static Item> {
    ITEMS
        .get()?
        .iter()
        .find(|(i, _)| *i == index)
        .map(|(_, it)| *it)
}

/// Where a nest's eggs stand: on the floor of its dish (one floor 1.5 units up, `source/nest`),
/// pushed half a unit into it.
const NEST_FLOOR: f64 = 1.0 / 64.0;
/// Where an egg put down at `at` rests: on the ground there, its bottom on the block's top. It
/// once sank a quarter into the ground as its relatives laid them, but an entity can't stand
/// partly inside a block: it fell through the grass to the stone below and suffocated in a tick
/// (B7, the creatures playtest: no utahraptor egg ever hatched). A nest's eggs sit on its floor
/// (`nest_spot`).
fn egg_rest(at: [f64; 3]) -> [f64; 3] {
    at
}

/// A free spot in the nest at `n`: the middle, else round it; a full nest's next eggs go on the
/// ground round it.
fn nest_spot(w: &World, n: [f64; 3]) -> [f64; 3] {
    let k = nest_scale(w, n);
    let free = |r: f64, y: f64| {
        std::iter::once((0.0, 0.0))
            .filter(move |_| r == NEST_RING)
            .chain((0..7).map(move |i| {
                let a = (i as f64 + if r == NEST_RING { 0.0 } else { 0.5 }) / 7.0 * std::f64::consts::TAU;
                (a.cos() * r * k, a.sin() * r * k)
            }))
            .map(move |(dx, dz)| [n[0] + dx, n[1] + y, n[2] + dz])
            .find(|&at| eggs_near(w, at, 0.1 * k).is_empty())
    };
    // In the dish, else round the nest on the ground (a full nest's eggs piled on each other in
    // one dish: Tester 2's eggs playtest, 11-06), else the middle.
    free(NEST_RING, NEST_FLOOR * k)
        .or_else(|| free(NEST_RING * NEST_BESIDE, 0.0))
        .unwrap_or([n[0], n[1] + NEST_FLOOR * k, n[2]])
}

/// The scale species `index`'s egg is drawn at: its egg's drawn length against its look's built
/// one (`animals_core::eggs`: within 15%).
fn egg_size(index: usize) -> f32 {
    animals_core::eggs::of(SPECIES[index].name).map_or(1.0, |(_, s)| s)
}

/// Species `index`'s egg's width as drawn (its box's, blocks): a nest and a clutch grow with it.
fn egg_width(index: usize) -> f32 {
    animals_core::eggs::of(SPECIES[index].name).map_or(NEST_EGG_W, |(l, s)| animals_core::bodies::of(l.model).0 * s)
}

/// Species `index`'s real egg length against a 15 cm egg (`hunt`: an egg's mass goes with its
/// cube).
pub(crate) fn egg_length(index: usize) -> f32 {
    animals_core::eggs::real_cm(SPECIES[index].name).unwrap_or(MASS_EGG_CM) / MASS_EGG_CM
}

/// Whether entity kind `kind` (an id) is an egg's.
pub(crate) fn is_egg_kind(kind: u32) -> bool {
    KINDS
        .get()
        .is_some_and(|k| k.iter().any(|(_, e)| e.id() == kind))
}

/// A nest's width (its box, `animals_core::bodies`).
fn nest_width() -> f32 {
    animals_core::bodies::of("nest").0
}
/// A species with wider eggs gets a bigger nest (a wyvern's, about 3 blocks across).
fn nest_size(index: usize) -> f32 {
    (egg_width(index) / NEST_EGG_W).max(1.0)
}

/// How much bigger than the plain one the nest at `at` is (1 for none).
fn nest_scale(w: &World, at: [f64; 3]) -> f64 {
    nest_near(w, at, 0.1)
        .and_then(|(n, _)| w.entity(n))
        .map_or(1.0, |e| (e.size.width / nest_width()) as f64)
}

/// The nest within `r` blocks of `at`, if any: its id and position.
fn nest_near(w: &World, at: [f64; 3], r: f64) -> Option<(u64, [f64; 3])> {
    w.entities_near(at, r, Some(&NEST))
        .into_iter()
        .find_map(|n| Some((n, w.entity(n)?.pos.to_array())))
}

/// Whether `at` is in a nest.
fn in_nest(w: &World, at: [f64; 3]) -> bool {
    w.entities_near(at, 2.5, Some(&NEST)).into_iter().any(|n| {
        w.entity(n).is_some_and(|e| {
            let r = 0.6 * (e.size.width / nest_width()) as f64;
            (e.pos.x - at[0]).powi(2) + (e.pos.z - at[2]).powi(2) < r * r
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
        spawn_egg(w, nest_spot(w, st.pos.to_array()), &e, owner_of(player));
        let slot = w.selected_slot(player);
        w.inventory(player).set(slot, StackData::shrink(held, 1));
        return;
    }
    if let Some(egg) = eggs_near(w, st.pos.to_array(), 0.6).into_iter().next() {
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
                let r = 0.6 * egg_width(y.index) as f64;
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

/// Mother `mother` of bird species `index` lays `young` (`breeding::deliver`): a nest built at her
/// kind of site near `at` (`nests::Site`; the ground under her where there's none, or where one
/// is built already), the clutch laid in it.
pub fn lay_in_nest(w: &World, index: usize, mother: u64, at: [f64; 3], young: &[Young], owner: Option<[u8; 64]>) {
    let s = &SPECIES[index];
    let site = animals_core::nests::site(s.name);
    let ground = ground_under(w, at);
    let found = site.and_then(|k| nest_site(w, k, at, ground));
    let spot = found.unwrap_or(ground);
    if nest_near(w, spot, 1.0).is_none() {
        let nest = w.spawn(&NEST, spot);
        if nest_size(index) > 1.0 {
            w.set_scale(nest, nest_size(index));
        }
    }
    lay(w, spot, young, owner);
    let at = spot.map(|v| v.floor() as i32);
    let place = match (site, found) {
        (Some(animals_core::nests::Site::Ground), _) => "on the ground",
        (_, Some(p)) if p[1] > ground[1] => "up high",
        _ => "on the ground, no site near",
    };
    weft_sdk::info!("{} {mother} nested at {at:?} {place} ({site:?}, ground {}): {} eggs", s.kind_name(), ground[1], young.len());
}

/// Where a nest of `site` goes near `at` (`ground` under it, the search's middle): the highest
/// open treetop (leaves or a trunk's tip) `CLIFF_ABOVE` over the ground, or for a cliff nester
/// the highest bare ledge as high (else a treetop);
/// `None` for the ground or when nothing is near.
fn nest_site(w: &World, site: animals_core::nests::Site, at: [f64; 3], ground: [f64; 3]) -> Option<[f64; 3]> {
    use animals_core::nests::{APART, CLIFF_ABOVE, LOOK, LOOK_DOWN, LOOK_UP, Site};
    // Round the ground under her: in the air, her own height says nothing of the trees'.
    let feet = [at[0].floor() as i32, ground[1] as i32, at[2].floor() as i32];
    let floor = ground[1] as i32;
    // Nests already built near: a site this close to one is taken.
    let built: Vec<[f64; 3]> = w
        .entities_near(ground, LOOK as f64 + APART + LOOK_UP as f64, Some(&NEST))
        .into_iter()
        .filter_map(|n| Some(w.entity(n)?.pos.to_array()))
        .collect();
    let taken = |p: [i32; 3]| {
        let c = [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5];
        built.iter().any(|b| crate::dist2(*b, c) < APART * APART)
    };
    let (mut tree, mut cliff): (Option<[i32; 3]>, Option<[i32; 3]>) = (None, None);
    // The higher, and of two as high the nearer.
    let better = |old: Option<[i32; 3]>, new: [i32; 3]| {
        let d = |p: [i32; 3]| (p[0] - feet[0]).pow(2) + (p[2] - feet[2]).pow(2);
        old.is_none_or(|o| new[1] > o[1] || (new[1] == o[1] && d(new) < d(o)))
    };
    for dx in -LOOK..=LOOK {
        for dz in -LOOK..=LOOK {
            let (x, z) = (feet[0] + dx, feet[2] + dz);
            // The column's top within reach: the first block down from above.
            let Some(y) = (feet[1] - LOOK_DOWN..=feet[1] + LOOK_UP)
                .rev()
                .find(|&y| w.block([x, y, z]) != AIR.id())
            else {
                continue;
            };
            if w.block([x, y + 1, z]) != AIR.id() || w.block([x, y + 2, z]) != AIR.id() {
                continue;
            }
            let b = w.block([x, y, z]);
            let top = [x, y + 1, z];
            if taken(top) {
                continue;
            }
            // A crown: leaves, or a trunk's tip above them (our spruces end in a log: aw_nests3's
            // ravens found no leaves on top and nested on the ground), high over the ground.
            let crown = LEAVES.on_block(b) || LOGS.on_block(b);
            if crown {
                if y + 1 >= floor + CLIFF_ABOVE && better(tree, top) {
                    tree = Some(top);
                }
            } else if site == Site::Cliff
                && y + 1 >= floor + CLIFF_ABOVE
                && weft_sdk::block_info(b).is_some_and(|i| i.solid())
                && better(cliff, top)
            {
                cliff = Some(top);
            }
        }
    }
    let pick = match site {
        Site::Ground => None,
        Site::Tree => tree,
        Site::Cliff => cliff.or(tree),
    }?;
    Some([pick[0] as f64 + 0.5, pick[1] as f64, pick[2] as f64 + 0.5])
}

/// The ground under `at`: down through the air and treetops to the first other block (at most
/// 64 down).
fn ground_under(w: &World, at: [f64; 3]) -> [f64; 3] {
    let p = at.map(|v| v.floor() as i32);
    // Through the air and any treetops (a bird over a forest).
    let open = |b: u32| b == AIR.id() || LEAVES.on_block(b);
    let y = (p[1] - 64..=p[1])
        .rev()
        .find(|&y| !open(w.block([p[0], y - 1, p[2]])) && open(w.block([p[0], y, p[2]])))
        .unwrap_or(p[1]);
    [p[0] as f64 + 0.5, y as f64, p[2] as f64 + 0.5]
}

/// An egg put down at `at`: resting on the ground (`egg_rest`), leaning (its model leans forward)
/// with its top toward the nest's middle, or any way outside a nest.
fn spawn_egg(w: &World, at: [f64; 3], egg: &Egg, owner: Option<[u8; 64]>) -> u64 {
    let Some(kind) = (egg.species as usize).checked_sub(1).and_then(kind_of) else { return 0 };
    let id = w.spawn(kind, egg_rest(at));
    if id == 0 {
        return 0;
    }
    if w.entity(id).is_some() {
        // Yaw 0 faces -z; the model leans toward where it faces.
        let yaw = match nest_near(w, at, 0.8) {
            Some((_, n)) if (n[0] - at[0]).hypot(n[2] - at[2]) > 0.05 => (-(n[0] - at[0])).atan2(-(n[2] - at[2])) as f32,
            _ => w.random_below(628) as f32 / 100.0,
        };
        w.entity_edit(id).yaw(yaw);
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
    let (Some(a), Some(mother)) = (w.component(id, &ANIMAL), w.component(id, &GENOME)) else { return false };
    if crate::animal::tamed(w, id) || a.female == 0 || breeding::young(w, id) {
        return false;
    }
    if w.entities_near(st.pos, 32.0, Some(s.kind)).len() >= CROWDED || nest_near(w, st.pos.to_array(), 24.0).is_some() {
        return false;
    }
    let feet = st.pos.to_array().map(|v| v.floor() as i32);
    if w.block(feet) != AIR.id() || w.block([feet[0], feet[1] - 1, feet[2]]) == AIR.id() {
        return false;
    }
    let at = [feet[0] as f64 + 0.5, feet[1] as f64, feet[2] as f64 + 0.5];
    let nest = w.spawn(&NEST, at);
    if nest_size(index) > 1.0 {
        w.set_scale(nest, nest_size(index));
    }
    let mut r = coats::world_rng(w);
    let Some(pool) = crate::evolution::pool_at(w, index, st.pos.to_array()) else { return false };
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
    let Some(e) = w.component(id, &EGG) else { return true };
    let Some(index) = (e.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
    else {
        return true;
    };
    let size = egg_size(index);
    if w.component(id, &EGG_SCALED).is_some_and(|v| v != size) {
        w.set_scale(id, size);
        if let Some(v) = w.component_mut(id, &EGG_SCALED) {
            *v = size;
        }
    }
    if (w.game_time() + id) % 20 != 0 {
        return true;
    }
    let nest = in_nest(w, st.pos.to_array());
    // The air itself: wind chill is what a body loses, and an egg never falls below the air (B5).
    let air = base_api::air_at(w, st.pos.to_array(), 0.3, st.in_liquid != 0, vitals::temperature::FIRE_REACH, |_| {}).still;
    // A bird's nest is brooded: held at a sitting parent's warmth.
    let brooded = animals_core::nests::site(SPECIES[index].name).is_some();
    let step = match life::develops(life::egg_air_of(air, nest, brooded)) {
        false => 0,
        true if nest => 40,
        true => 20,
    };
    let left = e.left.saturating_sub(step * breeding::speed());
    if let Some(e) = w.component_mut(id, &EGG) {
        e.left = left;
    }
    if left == 0 {
        let owner = w.component(id, &OWNER).filter(|o| o[0] != 0);
        if let Some(y) = young_of(&e, index) {
            w.despawn(id);
            breeding::born(w, st.pos.to_array(), &y, owner);
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
    let Some(e) = w.component(id, &EGG) else { return };
    let Some(index) = (e.species as usize).checked_sub(1) else { return };
    let Some(item) = item_of(index) else { return };
    let Some(st) = w.entity(id) else { return };
    let wild = w.component(id, &OWNER).is_none_or(|o| o[0] == 0);
    w.despawn(id);
    let stack = ItemStack::new(item.id(), 1).with(&EGG, e);
    let left = w.inventory(player).insert(stack);
    if !left.is_empty() {
        w.spawn_item(st.pos, left);
    }
    if wild {
        let guards: Vec<u64> = w
            .entities_near(st.pos, GUARD_RANGE, Some(SPECIES[index].kind))
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
    let Some((p, face)) = u.target_block().map(|(p, f)| (p.to_array(), f)) else { return false };
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
    // Nothing left: it hatches at its next step whatever the air (with a little left, an egg in
    // air too cool to develop never did, B7's playtest).
    for id in eggs_near(w, at, 16.0) {
        if let Some(e) = w.component_mut(id, &EGG) {
            e.left = 0;
        }
    }
}

/// `/animal nest` (for testing): the nearest wild female of a laying species builds her nest
/// now.
pub fn nest_now(w: &World, player: u64) -> bool {
    let Some(p) = w.entity(player) else { return false };
    for id in w.entities_near(p.pos, 16.0, None) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// An egg laid on grass (the mother's feet on the block's top, y 107) rests on that top, not in
    /// the block (B7).
    #[test]
    fn an_egg_laid_on_grass_sits_on_the_block() {
        let [_, y, _] = egg_rest([22.8, 107.0, -3.7]);
        assert_eq!(y, 107.0);
        assert!(y.fract() == 0.0 && y >= 107.0, "the egg's bottom {y} is inside the block under it");
    }
}

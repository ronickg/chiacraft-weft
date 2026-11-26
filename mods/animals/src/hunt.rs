//! Hunters and prey (`notes/ECOLOGY.md`, part 6). A hungry wild hunter looks round for prey of
//! the sizes it takes (`Species::prey`, from its research: as shares of its own mass), chases the
//! nearest and fights it (a knock-out kind through its anger, so a pack's kin join in); when it
//! kills, it eats the kill (`ENTITY_DIED`): it fills it by the prey's mass against its own
//! (`animals_core::needs::fills`: a mouse is a twentieth of a fox's meal, a hare all of it), and
//! what the kill drops is left as the carcass. It gives up a long chase. Animals of its own family
//! it leaves alone, and players for now.
//!
//! Its hunger is its prey's: it gets hungry in the wild where it has prey (`hungry_in_the_wild`),
//! arrives as fed as its prey are (`animal::arrive`), and a full one leaves them be.
//!
//! Prey keep watch (`wary`): a wild animal that sees a hunter able to take it runs off. In fog
//! both see less far (`base_api::sight`). In a blood moon's totality (`animals_core::sky`) they
//! watch half again as far, and a herd animal strayed from its herd walks back to its middle
//! (`huddle`).

use weft_sdk::*;

use crate::dist2;

use crate::animal::{self, ANIMAL};
use crate::species::{self, OTHER_PREY, SPECIES, Species};

/// The prey it chases (0: none), since when, when it looks again, and whether it hunts with its
/// pack (1: from when the hunt began, so the pack strung out in the chase still runs it down; not
/// saved).
static HUNT: Component<[u64; 4]> = Component::named("animal_hunt");
/// Until when it runs from a hunter (not saved).
static WARY: Component<u64> = Component::named("animal_wary");
/// When an ambusher broke cover to charge (0: still stalking; not saved).
static CHARGING: Component<u64> = Component::named("animal_charging");
/// When it last stood its ground (`stands_ground`; not saved): a stand that begins is shown by a rear.
static STOOD: Component<u64> = Component::named("animal_stood");
/// What hurt it and it turns on, until when, and when it last struck (`turn_on`; not saved).
static TURNED: Component<[u64; 3]> = Component::named("animal_turned_on");
/// Until when it walks a prowl's leg (`prowl`; not saved).
static PROWL: Component<u64> = Component::named("animal_prowl");
/// The pack's leader it follows as they prowl (0: none), and its legs so far (`prowl`; not saved).
/// The prey it last gave up on (`hunt_`): a kill of it a moment later is still its pack's meal.
static LAST: Component<u64> = Component::named("animal_last_prey");
static FOLLOW: Component<[u64; 2]> = Component::named("animal_follow");
/// A kill site it went to and found nothing at (x, z), and till when it leaves it (`hunt_`; not
/// saved).
static SITE: Component<[f64; 3]> = Component::named("animal_site");
/// The water a prowl last reached, `[x, z, until]`: not a leg's goal again until then
/// (`needs::SITE_AGAIN_SECS`).
static WATER_SEEN: Component<[f64; 3]> = Component::named("animal_water_seen");
/// The blood trail it follows: when the spot it walks to fell (game tick; 0: none; `blood::trail`) and where it lies
/// (not saved).
static TRAIL: Component<[f64; 4]> = Component::named("animal_trail");
/// The prey it set off after and what that weighed then, `[id, kg as f32 bits]` (not saved): the
/// dead can't be read at the kill, and its kind's grown mass fed hunters a young one's carcass as
/// a grown one's (cov_hunters run 4: a 2.7 kg young capybara eaten as 50 kg).
static PREY_KG: Component<[u64; 2]> = Component::named("animal_prey_kg");

/// Blocks a hunter looks for prey and prey watches for hunters, in clear air: fog, a blizzard or
/// a dust storm close both in (`base_api::sight`).
pub(crate) const LOOK: f64 = 16.0;
const WATCH: f64 = 10.0;
/// It hunts under this share of its food, until this full (as foraging, `needs`).
use animals_core::needs::{FULL, HUNTS_AT};
/// Ticks: a chase it gives up, between looks, and a flight.
const GIVE_UP: u64 = 600;
const LOOK_EVERY: u64 = 60;
/// Blocks from the blood spot it walks to at which it goes on to the next.
const TRAIL_AT: f64 = 1.5;
const RUN: u64 = 80;
/// Ticks a herd animal walks back to its herd before it looks again (`huddle`).
const HUDDLE_WALK: u64 = 40;

pub fn init(r: &mut Registry) {
    r.entity_component(&HUNT, [0u64; 4], 0);
    r.entity_component(&WARY, 0u64, 0);
    r.entity_component(&CHARGING, 0u64, 0);
    r.entity_component(&STOOD, 0u64, 0);
    r.entity_component(&TURNED, [0u64; 3], 0);
    r.entity_component(&PROWL, 0u64, 0);
    r.entity_component(&FOLLOW, [0u64; 2], 0);
    r.entity_component(&LAST, 0u64, 0);
    r.entity_component(&SITE, [0.0; 3], 0);
    r.entity_component(&TRAIL, [0.0; 4], 0);
    r.entity_component(&WATER_SEEN, [0.0; 3], 0);
    r.entity_component(&PREY_KG, [0u64; 2], 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&PREY_KG);
        r.edit(s.kind).attach(&HUNT);
        r.edit(s.kind).attach(&WARY);
        r.edit(s.kind).attach(&CHARGING);
        r.edit(s.kind).attach(&STOOD);
        r.edit(s.kind).attach(&TURNED);
        r.edit(s.kind).attach(&PROWL);
        r.edit(s.kind).attach(&FOLLOW);
        r.edit(s.kind).attach(&LAST);
        r.edit(s.kind).attach(&SITE);
        r.edit(s.kind).attach(&TRAIL);
        r.edit(s.kind).attach(&WATER_SEEN);
    }
    // A kill feeds the hunter that made it and its pack near (`needs::KILL_SHARED`), the prey's
    // mass shared out against each one's own. Fed only the killer, each wolf of three hunted on
    // its own hunger, and they took all six deer of a herd in ten days (ai1124_wolvesDays1b);
    // fed together, a pack hunts together when it is hungry again, a deer every few days.
    r.on_event(&actions::DIE, |b| {
        let w = b.world();
        for d in b.iter() {
            let Some(a) = w.component(d.source, &ANIMAL) else { continue };
            let Some(s) = a.species().filter(|s| s.hunts()) else { continue };
            // The prey's own mass if it can still be read (its genes and growth), else what it weighed
            // when the killer set off after it, else its kind's.
            // A pack's kill may be made by another than the one that set off after it (r12_hunters:
            // utahraptor 117 hunted, 115 killed): its kin near who knew the prey's mass count too.
            let knew = |o: u64| {
                w.component(o, &PREY_KG)
                    .filter(|p| p[0] == d.entity)
                    .map(|p| f32::from_bits(p[1] as u32))
            };
            let Some(prey) = mass(&w, d.entity)
                .or_else(|| knew(d.source))
                .or_else(|| {
                    let at = w.entity(d.source)?.pos;
                    w.entities_near(at, animals_core::needs::KILL_SHARED * 4.0, None)
                        .into_iter()
                        .find_map(knew)
                })
                .or_else(|| kind_mass(d.kind, None))
            else {
                continue;
            };
            let Some(at) = w.entity(d.source).map(|e| e.pos) else { continue };
            let kind = w.entity(d.source).map(|e| e.kind);
            let mut eaters = vec![d.source];
            if s.pack_prey().is_some() && !animal::tamed(&w, d.source) {
                eaters.extend(
                    w.entities_near(at, animals_core::needs::KILL_SHARED, None)
                        .into_iter()
                        .filter(|&o| {
                            o != d.source
                                && w.entity(o)
                                    .is_some_and(|e| e.health > 0.0 && Some(e.kind) == kind)
                                && !animal::tamed(&w, o)
                                && w.component(o, &ANIMAL)
                                    .is_some_and(|a| (a.food as f32) < a.stat(species::FOOD) * FULL)
                        }),
                );
            }
            // Those on the same hunt eat of it wherever the chase left them: in wolvesFar2 two
            // wolves gave its prey up as gone at the kill and each took a deer of its own.
            if s.pack_prey().is_some() {
                let on_it: Vec<u64> = w
                    .entities_near(at, animals_core::needs::KILL_SHARED * 4.0, None)
                    .into_iter()
                    .filter(|&o| {
                        !eaters.contains(&o)
                            && w.entity(o).is_some_and(|e| Some(e.kind) == kind)
                            && !animal::tamed(&w, o)
                            && (w.component(o, &HUNT).is_some_and(|h| h[0] == d.entity) || w.component(o, &LAST) == Some(d.entity))
                            && w.component(o, &ANIMAL)
                                .is_some_and(|a| (a.food as f32) < a.stat(species::FOOD) * FULL)
                    })
                    .collect();
                eaters.extend(on_it);
            }
            let kg = animals_core::needs::share(prey, eaters.len());
            let mut ate = 0.0;
            for &e in &eaters {
                let (Some(a), Some(own)) = (w.component(e, &ANIMAL), mass(&w, e)) else { continue };
                // A full meal is the most it eats of its share (`needs::fills`).
                ate += kg.min(own * animals_core::needs::MEAL);
                let max = a.stat(species::FOOD);
                let meal = (max * animals_core::needs::fills(own, kg)).round().max(1.0) as u16;
                animal::feed(&w, e, meal);
                weft_sdk::info!(
                    "{} {e} eats {}'s kill ({kg:.3} of {prey:.3} kg of {}): food {} of {max:.0}",
                    s.name,
                    d.source,
                    name_of(abi::entry_kind::ENTITY, d.kind),
                    a.food.saturating_add(meal).min(max as u16)
                );
                w.component_mut(e, &HUNT).map(|h| *h = [0; 4]);
            }
            // What they ate is gone off the body (`carcass`): a pack's meal opens it, or leaves bones.
            crate::carcass::taken(&w, d.entity, d.pos, ate);
        }
    });
}

/// How fed its prey are where `pos` is this season, 0 to 1: the best fed of its prey that live in
/// that biome (`needs::prey_in`), by how much of their own food grows there now
/// (`forage::season_lush`). `None`: none of its prey live there (only farm animals, which don't
/// thin with the seasons).
pub fn prey_lush(w: &World, s: &Species, pos: [i32; 3]) -> Option<f32> {
    let biome = name_of(abi::entry_kind::BIOME, w.biome(pos));
    animals_core::needs::prey_in(s, biome)
        .map(|o| crate::forage::season_lush(w, o, pos))
        .reduce(f32::max)
}

/// An egg's mass, kg, at 15 cm long real (about a goose egg's; `eggs::egg_length` scales the
/// others by their length): eggs lying in a nest are food for the egg eaters of their sizes (the report's tier
/// 3: "eggs are real food and nests are built already"), Citipati, the raptors, the foxes.
const EGG_KG: f32 = 0.2;

/// The species `index` an egg `id` holds, if it is one.
pub(crate) fn egg_of(w: &World, id: u64) -> Option<usize> {
    let e = w.component(id, &crate::eggs::EGG)?;
    (e.species as usize)
        .checked_sub(1)
        .filter(|&i| i < SPECIES.len())
}

/// A player's egg (its nest's, or one laid by a tame): the wild's egg eaters leave those alone.
pub(crate) fn owned_egg(w: &World, id: u64) -> bool {
    egg_of(w, id).is_some() && w.component(id, &animal::OWNER).is_some_and(|o| o[0] != 0)
}

/// The mass in kg of an entity of `kind`: a species' grown mass (scaled by `size`, its genes and
/// growth), an egg's, or a farm animal's wild kind's (`OTHER_PREY`). `None`: not prey to anything.
fn kind_mass(kind: u32, size: Option<f32>) -> Option<f32> {
    if crate::eggs::is_egg_kind(kind) {
        return Some(EGG_KG * size.unwrap_or(1.0).powi(3));
    }
    if let Some(s) = SPECIES.iter().find(|s| s.kind.id() == kind) {
        return Some(s.bio.mass * size.unwrap_or(1.0).powi(3));
    }
    let name = name_of(abi::entry_kind::ENTITY, kind);
    OTHER_PREY
        .iter()
        .find(|&&(n, _)| n == name)
        .map(|&(_, kg)| kg)
}

/// What `id` weighs now.
pub(crate) fn mass(w: &World, id: u64) -> Option<f32> {
    let e = w.entity(id)?;
    if let Some(index) = egg_of(w, id) {
        return kind_mass(e.kind, Some(crate::eggs::egg_length(index)));
    }
    let size = w.component(id, &ANIMAL).and_then(|a| {
        let index = (a.species as usize).checked_sub(1)?;
        // Against its species' average size: its row's mass is an average grown one's.
        Some(crate::coats::size_of(w, id)? / crate::coats::mean_size(index) * crate::breeding::growth(w, id, index))
    });
    kind_mass(e.kind, size)
}

/// A goal for wild hunters: hungry, it goes after the nearest prey it can take; it takes the turn
/// while it chases (a knock-out kind fights through its anger, `knockout::behave`, after stalking
/// first if it ambushes: `stalks_first`).
pub fn hunt(w: &World, id: u64, st: &EntityState) -> bool {
    let t = crate::prof::stamp();
    let r = hunt_(w, id, st);
    crate::prof::slow(t, || format!("hunt of {id}"));
    r
}

fn hunt_(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    let Some(s) = a.species().filter(|s| s.hunts()) else { return false };
    let Some(mut h) = w.component(id, &HUNT) else { return false };
    let full = a.food as f32 / a.stat(species::FOOD).max(1.0);
    // A fierce one hunts sooner (`roles::FIERCE_HUNGER`), a timid one later.
    let fierce = crate::senses::facts(w, id).map_or(0.0, |f| animals_core::roles::lean(f.persona()[2]));
    let hungry = (HUNTS_AT + animals_core::roles::FIERCE_HUNGER * fierce).min(FULL - 0.05);
    if animal::tamed(w, id) || full >= FULL || (h[0] == 0 && full >= hungry) {
        if h[0] != 0 {
            w.component_mut(id, &HUNT).map(|h| *h = [0; 4]);
        }
        w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
        // Not hungry enough to hunt itself, a pack's member not yet full goes with its leader
        // on the hunt: a timid wolf at 29 % slept 110 s while its pack hunted and killed 40
        // blocks off (ai1124_moveFar1).
        if !animal::tamed(w, id)
            && full < FULL
            && s.pack_prey().is_some()
            && let Some(l) = with_pack(w, id, st, s.name)
        {
            return follow_on(w, id, st, l, w.game_time());
        }
        return false;
    }
    let now = w.game_time();
    let knockout = w.component(id, &crate::knockout::KNOCKOUT).is_some();
    if h[0] != 0 {
        // A pack runs its prey down: they take turns at the front, faster than one alone could
        // keep up, and keep at it twice as long (a wolf pack's chase of an elk lasts minutes).
        let pack = h[3] == 1;
        // A pack's member keeps on prey as far as its pack reaches past its own look: joined
        // from 16 blocks behind its leader, two wolves gave up "gone" within a second at 31
        // blocks (ai1124_wolves1c).
        let keep = LOOK * 1.5 + if pack { PACK_RANGE } else { 0.0 };
        let alive = w
            .entity(h[0])
            .is_some_and(|e| e.health > 0.0 && dist2(e.pos, st.pos) < keep * keep);
        // A fierce one keeps at it longer (`roles::FIERCE_CHASE`).
        let give_up = ((if pack { 2 * GIVE_UP } else { GIVE_UP }) as f32 * (1.0 + animals_core::roles::FIERCE_CHASE * fierce)) as u64;
        if alive && now < h[1] + give_up {
            if knockout {
                if stalks_first(w, id, st, s, h[0], pack, now) {
                    return true;
                }
                // Its anger does the chasing; kept up while the chase lasts.
                if w.component(id, &crate::knockout::KNOCKOUT)
                    .is_some_and(|k| k.angry == 0 && k.out == 0)
                {
                    crate::knockout::anger(w, id, h[0]);
                }
                return false;
            }
            let (aim, speed) = chase(w, id, st, s, h[0], pack, now);
            crate::commands::fight_aiming(w, id, st, h[0], speed, aim);
            return true;
        }
        weft_sdk::info!("{} {id} gives up on {} ({})", s.name, h[0], if alive { "too long" } else { "gone" });
        // Its prey's death reaches it before the kill is eaten (`actions::DIE`): wolvesDays1f's
        // leader gave its deer up as gone, 20 blocks off, missed its share and took a second
        // deer at once.
        w.component_mut(id, &LAST).map(|c| *c = h[0]);
        h = [0, 0, now + LOOK_EVERY, 0];
        w.component_mut(id, &CHARGING).map(|c| *c = 0);
    }
    if now < h[2] {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        // On a blood trail it keeps the turn between its looks, walking or at a spot: r3's wolf, giving it up at its
        // first spot, searched for food instead, and that walk then passed for the trail's.
        if w.component(id, &TRAIL).is_some_and(|t| t[0] != 0.0) {
            return true;
        }
        // On a prowl's leg it keeps the turn, looking again at `LOOK_EVERY`; one following its
        // leader keeps up with it every tick.
        if !w.component(id, &PROWL).is_some_and(|p| now < p) {
            return false;
        }
        if let Some(l) = w.component(id, &FOLLOW).map(|f| f[0]).filter(|&l| l != 0) {
            return follow_on(w, id, st, l, now);
        }
        return w.navigating(id);
    }
    h[2] = now + LOOK_EVERY;
    // Out of its hours it rests rather than hunts, unless hunger presses (`routines::HUNTS_FROM`).
    if crate::routines::activity(w, s.index()) < animals_core::routines::HUNTS_FROM && full >= animals_core::routines::HUNTS_ANYWAY {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
        return false;
    }
    // Food lying near (a carcass, dropped meat) is a meal without a chase: it eats that first
    // (`forage.rs`).
    if crate::forage::ground_food(w, s, st.pos.to_array(), crate::forage::ITEM_LOOK).is_some() {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        // A trail's end in sight: the turn is the forager's.
        w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
        return false;
    }
    // Its prey and its pack from its last look round (`senses`): a pack's kind with others of it
    // near hunts together, bigger prey (`Species::takes_with`).
    let seen = crate::senses::sense(w, id, st, |sight| watch(w, sight));
    let pack = if s.pack_prey().is_some() { seen.pack.max(1) as u32 } else { 1 };
    let prey = (seen.prey != 0)
        .then_some((seen.prey, seen.prey_kg))
        .filter(|&(t, _)| {
            w.entity(t)
                .is_some_and(|e| e.health > 0.0 && dist2(e.pos, st.pos) < (LOOK * 1.5).powi(2))
        });
    let Some((t, kg)) = prey else {
        w.component_mut(id, &HUNT).map(|c| *c = h);
        // Thirsty, it goes to water before it looks further for prey (`drink::drink` after this
        // goal): a starving, thirsty wolf went between a kill site and water it couldn't reach
        // each look, stood 64 s and died of thirst (ai1124_wolvesDays1b).
        if w.component(id, &ANIMAL)
            .is_some_and(|a| 1.0 - a.thirst < animals_core::needs::THIRSTY)
        {
            w.component_mut(id, &PROWL).map(|p| *p = 0);
            w.component_mut(id, &FOLLOW).map(|f| f[0] = 0);
            w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
            return false;
        }
        // No prey in sight: blood it smells it follows, spot by spot as it fell (`blood::trail`), to a wounded animal it
        // takes or to where one bled out. Walked as a prowl's leg, keeping the turn.
        let own = mass(w, id).unwrap_or(s.bio.mass);
        let trail = w.component(id, &TRAIL).unwrap_or_default();
        let after = trail[0] as u64;
        // On a trail it looks again every second, and walks on to the next spot once it is at this one.
        if after != 0 {
            w.component_mut(id, &HUNT).map(|c| c[2] = now + 20);
            let left = (st.pos.x - trail[1]).powi(2) + (st.pos.z - trail[3]).powi(2);
            if left > TRAIL_AT * TRAIL_AT && w.navigating(id) {
                return true;
            }
        }
        if let Some((spot, at)) = crate::blood::trail(w, id, st.pos, after, |b| mass(w, b).is_some_and(|kg| s.takes_at_most(own, kg))) {
            if after == 0 {
                weft_sdk::info!("{} {id} follows a blood trail from {:.0} {:.0}", s.name, spot[0], spot[2]);
            } else {
                weft_sdk::debug!("{} {id} on the blood trail to {:.0} {:.0}", s.name, spot[0], spot[2]);
            }
            w.component_mut(id, &TRAIL)
                .map(|c| *c = [at as f64, spot[0], spot[1], spot[2]]);
            w.component_mut(id, &HUNT).map(|c| c[2] = now + 20);
            w.navigate(id, spot, 1.0);
            w.component_mut(id, &PROWL)
                .map(|p| *p = now + (animals_core::needs::PROWL_SECS * 20.0) as u64);
            return true;
        }
        if after != 0 {
            weft_sdk::info!("{} {id} loses the blood trail", s.name);
            w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
        }
        // Nor blood: it goes where kills were (`memory::haunt`), where prey come and die.
        // A site it has just been to and found nothing at it leaves for `needs::SITE_AGAIN_SECS`
        // (a wolf went between one and a prowl's leg 12 times in 150 s, ai1124_wolves1c).
        let site = crate::memory::haunt(w, st.pos.to_array()).filter(|site| {
            let checked = w
                .component(id, &SITE)
                .is_some_and(|c| now < c[2] as u64 && (c[0] - site[0]).powi(2) + (c[1] - site[2]).powi(2) < 9.0);
            if !checked && dist2(*site, st.pos) <= 16.0 {
                w.component_mut(id, &SITE)
                    .map(|c| *c = [site[0], site[2], (now + (animals_core::needs::SITE_AGAIN_SECS * 20.0) as u64) as f64]);
                return false;
            }
            !checked
        });
        if let Some(site) = site
            && !w.navigating(id)
        {
            // Walked as a prowl's leg, keeping the turn: it went every 3 s and its rest took the
            // turn back each time, so two wolves lay 80 s "going" to one (ai1124_wolves1b).
            weft_sdk::info!("{} {id} goes to a kill site at {:.0} {:.0}", s.name, site[0], site[2]);
            w.navigate(id, site, 1.0);
            // Checked once it sets off: the walk keeps the turn as a prowl's leg does, and one it
            // can't reach isn't set off for again at every look (465 times in 60 s, wolvesDays1b).
            w.component_mut(id, &SITE)
                .map(|c| *c = [site[0], site[2], (now + (animals_core::needs::SITE_AGAIN_SECS * 20.0) as u64) as f64]);
            w.component_mut(id, &PROWL)
                .map(|p| *p = now + (animals_core::needs::PROWL_SECS * 20.0) as u64);
            return true;
        }
        // Nor any kill site: it prowls for prey (`prowl`), a pack behind its leader.
        // The leader it follows already, even out of its kin's reach (`KIN`): fallen 16 blocks
        // behind, a wolf counted itself its own leader and went its own way (ai1124_wolves1e).
        let following = w.component(id, &FOLLOW).map_or(0, |f| f[0]);
        let leader = if following != 0 {
            Some(following)
        } else {
            (s.pack_prey().is_some() && seen.leader != id && seen.leader != 0).then_some(seen.leader)
        };
        return prowl(w, id, st, s, now, leader);
    };
    w.component_mut(id, &PROWL).map(|p| *p = 0);
    w.component_mut(id, &FOLLOW).map(|f| *f = [0; 2]);
    w.component_mut(id, &TRAIL).map(|c| *c = [0.0; 4]);
    weft_sdk::info!("{} {id} hunts {} ({kg:.1} kg, a pack of {pack})", s.name, t);
    w.component_mut(id, &PREY_KG)
        .map(|p| *p = [t, kg.to_bits() as u64]);
    // A pack hunts what it finds together, prey it could take alone too (wolves run deer down
    // as a pack): in ai1124_wolves1b each of three wolves took its own deer alone while the
    // others lay by.
    // Two are enough to run prey down together (a pair of wolves hunts deer); bigger prey takes
    // the full pack (`Species::takes_with`). Three wolves spread 9 blocks apart counted two and
    // never rallied (ai1124_wolves1d).
    if pack >= animals_core::needs::RALLY_PACK {
        rally(w, id, st, t, now);
    }
    h = [t, now, now + LOOK_EVERY, (pack >= animals_core::needs::RALLY_PACK) as u64];
    w.component_mut(id, &HUNT).map(|c| *c = h);
    if knockout {
        if stalks_first(w, id, st, s, t, pack >= species::PACK, now) {
            return true;
        }
        crate::knockout::anger(w, id, t);
        return false;
    }
    crate::commands::fight(w, id, st, t);
    true
}

/// A hungry hunter with nothing in sight and no kill site: it walks a leg of
/// `needs::PROWL_LEG` blocks, onto ground its paths reach, and keeps the turn while it walks it
/// (`needs::PROWL_SECS` at most), looking for prey at each `LOOK_EVERY`. Every other leg heads
/// for the water it knows, where prey come to drink (`needs::PROWL_WATER`); the others a new way.
/// A pack's other members follow its leader (`keep_up`). True while it prowls.
fn prowl(w: &World, id: u64, st: &EntityState, s: &Species, now: u64, leader: Option<u64>) -> bool {
    use animals_core::needs::{FLEE_DROP, PROWL_LEG, PROWL_SECS, PROWL_WATER};
    let until = now + (PROWL_SECS * 20.0) as u64;
    // A pack prowls together: one not its leader keeps up with it (each on its own legs, the
    // pack spread over the savanna and never counted three to hunt as one, ai1124_wolves1b).
    // Only a leader that is hungry too (prowling or about to): a fed one lying by its kill would
    // hold the hungry back. Waiting for its leader's first leg, each of three wolves set off its
    // own way (ai1124_wolves1d).
    let hungry = |l: u64| {
        w.component(l, &PROWL).is_some_and(|p| now < p)
            // Or out on a hunt: wolvesPack3's second hungry wolf dropped its leader as it joined
            // a chase (too fed to prowl itself) and prowled alone 240 s, far from both.
            || w.component(l, &HUNT).is_some_and(|h| h[0] != 0)
            || w.component(l, &ANIMAL)
                .is_some_and(|a| (a.food as f32) < a.stat(species::FOOD) * HUNTS_AT)
    };
    if let Some(l) = leader.filter(|&l| hungry(l)) {
        w.component_mut(id, &FOLLOW).map(|f| f[0] = l);
        w.component_mut(id, &PROWL).map(|p| *p = until);
        return follow_on(w, id, st, l, now);
    }
    w.component_mut(id, &FOLLOW).map(|f| f[0] = 0);
    if w.component(id, &PROWL).is_some_and(|p| now < p) && w.navigating(id) {
        return true;
    }
    let legs = w.component(id, &FOLLOW).map_or(0, |f| f[1]);
    w.component_mut(id, &FOLLOW).map(|f| f[1] = legs + 1);
    let y = st.pos.y.floor() as i32;
    // Up a slope too, as far as `needs::PROWL_CLIMB`: held to a block up, a pack in a pond's
    // hollow walked legs to and fro below a herd on the rim and never saw it (ai1124_wolvesFar4).
    let ground = |x: f64, z: f64| {
        (y - (FLEE_DROP + 2)..=y + animals_core::needs::PROWL_CLIMB)
            .rev()
            .find(|&g| crate::comfort::ground(w, x.floor() as i32, g, z.floor() as i32) == Some(g))
    };
    // Prey drink: every other leg toward the water it knows, while it is further than
    // `PROWL_WATER` (in wolves1e a pack's random legs took 100 s to find a herd 24 blocks off by
    // the pond).
    // Reached, a water is left for `needs::SITE_AGAIN_SECS`: every other leg took a wolf back to
    // the one pond it knew, 16 blocks out and back for 160 s, the herd at another 50 blocks off
    // (wolvesPack2).
    let pos = st.pos.to_array();
    let seen = w.component(id, &WATER_SEEN).unwrap_or_default();
    // A stray from its pack heads for it first (`group::regroup_leg`: it howls, they answer).
    let pack = crate::group::regroup_leg(w, id, st);
    let water = (legs % 2 == 0 && pack.is_none())
        .then(|| crate::drink::known_water(w, id, pos))
        .flatten()
        .filter(|p| !((p[0] - seen[0]).abs() < 1.0 && (p[2] - seen[1]).abs() < 1.0 && (now as f64) < seen[2]))
        .filter(|p| ((p[0] - pos[0]).powi(2) + (p[2] - pos[2]).powi(2)).sqrt() > PROWL_WATER)
        .and_then(|p| {
            let d = ((p[0] - pos[0]).powi(2) + (p[2] - pos[2]).powi(2)).sqrt();
            let k = (PROWL_LEG / d).min(1.0);
            let (x, z) = (pos[0] + (p[0] - pos[0]) * k, pos[2] + (p[2] - pos[2]) * k);
            let to = [x, ground(x, z)? as f64, z];
            // This leg reaches it: looked at, it is left a while.
            if k >= 1.0 {
                w.component_mut(id, &WATER_SEEN)
                    .map(|c| *c = [p[0], p[2], (now + (animals_core::needs::SITE_AGAIN_SECS * 20.0) as u64) as f64]);
            }
            Some(to)
        });
    let start = w.random_below(8);
    let leg = pack.or(water).or_else(|| {
        (0..8).find_map(|k| {
            let a = (start + k) as f64 * std::f64::consts::TAU / 8.0;
            let (x, z) = (st.pos.x + a.cos() * PROWL_LEG, st.pos.z + a.sin() * PROWL_LEG);
            Some([x, ground(x, z)? as f64, z])
        })
    });
    let Some(to) = leg else { return false };
    weft_sdk::info!(
        "{} {id} prowls for prey toward {:.0} {:.0} {:.0}{}",
        s.name,
        to[0],
        to[1],
        to[2],
        if pack.is_some() {
            " (its pack)"
        } else if water.is_some() {
            " (water)"
        } else {
            ""
        }
    );
    w.navigate(id, to, 1.0);
    w.component_mut(id, &PROWL).map(|p| *p = until);
    true
}

/// The leader it goes with on a hunt it isn't hungry enough for itself (`hunt_`): the one it
/// follows already (until `keep_up` loses it: fed, gone, past `needs::PACK_LOSE`), its pack's
/// leader from its last look round once that one hunts or prowls, or, its pack's leader itself
/// (or that one at rest), the nearest of its kin within `senses::KIN` out on a hunt: led by the
/// lowest id, a pack's leader at 70 % lay by while its two hungry ones hunted (wolvesPack1).
fn with_pack(w: &World, id: u64, st: &EntityState, name: &str) -> Option<u64> {
    let now = w.game_time();
    let out =
        |l: u64| l != 0 && l != id && (w.component(l, &HUNT).is_some_and(|h| h[0] != 0) || w.component(l, &PROWL).is_some_and(|p| now < p));
    let following = w.component(id, &FOLLOW).map_or(0, |f| f[0]);
    if following != 0 {
        return Some(following);
    }
    let leader = w
        .component(id, &crate::senses::SENSES)
        .map_or(0, |s| s.leader);
    let leader = if out(leader) {
        leader
    } else if (now + id) % 20 == 0 {
        // Looked for once a second: it is the leader, or its leader is at rest.
        w.entities_near(st.pos, crate::senses::KIN, None)
            .into_iter()
            .filter(|&o| out(o) && !animal::tamed(w, o))
            .filter_map(|o| {
                w.entity(o)
                    .filter(|e| e.kind == st.kind)
                    .map(|e| (dist2(e.pos, st.pos), o))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(0, |(_, o)| o)
    } else {
        0
    };
    if leader == 0 {
        return None;
    }
    weft_sdk::info!("{name} {id} goes with its leader {leader} on the hunt");
    crate::rest::stop(w, id);
    w.component_mut(id, &FOLLOW).map(|f| f[0] = leader);
    Some(leader)
}

/// It follows its leader (`keep_up`), and joins the hunt that one is on once within
/// `PACK_RANGE` of it: a follower behind a leader already chasing kept up and never hunted
/// (wolvesPack3: a pack of 2 while a third hungry one trailed).
fn follow_on(w: &World, id: u64, st: &EntityState, leader: u64, now: u64) -> bool {
    let prey = w.component(leader, &HUNT).map_or(0, |h| h[0]);
    if prey != 0
        && w.entity(leader)
            .is_some_and(|l| dist2(l.pos, st.pos) < PACK_RANGE * PACK_RANGE)
        && let Some(c) = w.component_mut(id, &HUNT)
        && c[0] == 0
    {
        *c = [prey, now, now + LOOK_EVERY, 1];
        w.component_mut(id, &FOLLOW).map(|f| f[0] = 0);
        w.component_mut(id, &PROWL).map(|p| *p = 0);
        weft_sdk::info!("animal {id} joins {leader}'s hunt of {prey}");
        return true;
    }
    keep_up(w, id, st, leader)
}

/// A pack's member keeps within `needs::PACK_FOLLOW` of the leader it follows as they prowl, at
/// a share faster than a walk to close up (`needs::PACK_CATCH_UP`). True while it follows; false
/// once its leader is gone, fed or past `needs::PACK_LOSE` (it prowls on its own).
fn keep_up(w: &World, id: u64, st: &EntityState, leader: u64) -> bool {
    use animals_core::needs::{PACK_CATCH_UP, PACK_FOLLOW, PACK_LOSE};
    let fed = w
        .component(leader, &ANIMAL)
        .is_some_and(|a| a.food as f32 >= a.stat(species::FOOD) * FULL);
    let Some(l) = w
        .entity(leader)
        .filter(|l| l.health > 0.0 && !fed && dist2(l.pos, st.pos) < PACK_LOSE * PACK_LOSE)
    else {
        w.component_mut(id, &FOLLOW).map(|f| f[0] = 0);
        return false;
    };
    if dist2(l.pos, st.pos) > PACK_FOLLOW * PACK_FOLLOW {
        // To its own place beside its leader, not onto it: walked to the leader itself, a pack
        // stood stacked on one block behind it (reports/Creature AI.md, step 10).
        let a = (id % 8) as f64 * std::f64::consts::TAU / 8.0;
        let to = [l.pos.x + a.cos() * PACK_FOLLOW * 0.75, l.pos.y, l.pos.z + a.sin() * PACK_FOLLOW * 0.75];
        w.navigate(id, to, PACK_CATCH_UP);
    }
    true
}

/// A knock-out kind that ambushes (a jaguar) stalks to its burst range as any ambusher does
/// (`chase`); its anger takes over as it breaks cover with the pounce. False once it has broken
/// cover (or isn't an ambusher alone): its anger does the rest.
fn stalks_first(w: &World, id: u64, st: &EntityState, s: &Species, prey: u64, pack: bool, now: u64) -> bool {
    if pack || !animals_core::roles::ambushes(s.family.name) || w.component(id, &CHARGING).unwrap_or(0) != 0 {
        return false;
    }
    let (aim, speed) = chase(w, id, st, s, prey, pack, now);
    if w.component(id, &CHARGING).unwrap_or(0) != 0 {
        return false;
    }
    crate::commands::fight_aiming(w, id, st, prey, speed, aim);
    true
}

/// While it stands its ground and doesn't charge, it rears once in this many of its looks
/// (`senses::EVERY` ticks each): every 4 seconds.
const REAR_EVERY: u64 = 8;

/// Its kind's speed a pack chases at (one alone: 1.4, `commands::fight`): a wolf 5.5 blocks a
/// second, at this nearly 10, past a fleeing deer's 8.
const PACK_SPEED: f32 = 1.75;

/// Blocks within which a pack's kind hunt together (and count each other, `senses::KIN`).
const PACK_RANGE: f64 = crate::senses::KIN;

/// Big prey only a pack takes: the others of the pack near that aren't hunting already go after it
/// too, so it isn't one wolf against a red deer.
fn rally(w: &World, id: u64, st: &EntityState, target: u64, now: u64) {
    let kind = w.entity(id).map(|e| e.kind);
    for o in w.entities_near(st.pos, PACK_RANGE, None) {
        // Only the hungry join: a wolf full from its own kill joined the next hunt 18 s later and
        // took half that deer too (ai1124_wolvesDays1g).
        if o == id
            || w.entity(o).map(|e| e.kind) != kind
            || animal::tamed(w, o)
            || !w
                .component(o, &ANIMAL)
                .is_some_and(|a| (a.food as f32) < a.stat(species::FOOD) * FULL)
        {
            continue;
        }
        if let Some(c) = w.component_mut(o, &HUNT)
            && c[0] == 0
        {
            *c = [target, now, now + LOOK_EVERY, 1];
            weft_sdk::info!("animal {o} joins {id}'s hunt of {target}");
        }
    }
}

/// A goal for wild animals: a hunter near that could take it, and it runs the other way.
/// Whether it is on a hunt now (chasing its prey).
pub fn on_hunt(w: &World, id: u64) -> bool {
    w.component(id, &HUNT).is_some_and(|h| h[0] != 0)
}

/// `id` turns on `target`, which hurt it (`taming::hurt`, a big plant eater): it charges it for
/// `roles::TURN_SECS`.
pub fn turn_on(w: &World, id: u64, target: u64) {
    let until = w.game_time() + (animals_core::roles::TURN_SECS * 20.0) as u64;
    if let Some(c) = w.component_mut(id, &TURNED) {
        *c = [target, until, 0];
    }
}

/// Charging what hurt it now (`turn_on`).
pub fn turned_on(w: &World, id: u64) -> bool {
    w.component(id, &TURNED)
        .is_some_and(|c| c[0] != 0 && w.game_time() < c[1])
}

/// The charge at what hurt it (`turn_on`): it runs at it and strikes within `roles::CHARGE`, until
/// its time is up, the target is gone (dead, or a player out of the game's reach: a spectator)
/// or farther than `roles::TURN_GIVES_UP`.
fn turned(w: &World, id: u64, st: &EntityState) -> bool {
    use animals_core::roles;
    let Some(c) = w.component(id, &TURNED).filter(|c| c[0] != 0) else { return false };
    let now = w.game_time();
    let target = w
        .entity(c[0])
        .filter(|t| t.health > 0.0)
        .filter(|_| w.game_mode(c[0]) != abi::game_mode::SPECTATOR);
    let Some(t) = target.filter(|t| now < c[1] && dist2(t.pos, st.pos) < roles::TURN_GIVES_UP * roles::TURN_GIVES_UP) else {
        if let Some(c) = w.component_mut(id, &TURNED) {
            *c = [0; 3];
        }
        weft_sdk::info!("animal {id} lets {} go", c[0]);
        return false;
    };
    crate::senses::face(w, id, st, [t.pos.x as f32, t.pos.y as f32, t.pos.z as f32]);
    crate::anim::hold(w, id, crate::anim::Act::Charge);
    if (now + id).is_multiple_of(10) || !w.navigating(id) {
        crate::wild::wake(w, id);
        w.navigate(id, t.pos, roles::CHARGE_SPEED);
    }
    if dist2(t.pos, st.pos) <= roles::CHARGE * roles::CHARGE && now >= c[2] + (roles::TURN_HIT_SECS * 20.0) as u64 {
        w.damage(c[0], Damage::new(t.max_health * roles::CHARGE_HIT, damage_cause::ATTACK).by(id));
        if let Some(c) = w.component_mut(id, &TURNED) {
            c[2] = now;
        }
        weft_sdk::info!("animal {id} charges {} (it hurt it)", c[0]);
    }
    true
}

pub fn wary(w: &World, id: u64, st: &EntityState) -> bool {
    let now = w.game_time();
    // It turned on what hurt it: it charges, whatever else it notices.
    if turned(w, id, st) {
        return true;
    }
    // Running already; else it looks round once in `senses::EVERY` ticks (a tame never does: it
    // trusts its owner to keep it safe).
    if w.component(id, &WARY).is_some_and(|until| now < until) {
        return true;
    }
    if !(now + id).is_multiple_of(crate::senses::EVERY) || animal::tamed(w, id) {
        return false;
    }
    // What it has noticed of hunters (`senses`: seen, heard, smelled, a fight near): half aware
    // it stops and stares that way, aware it runs, over ground it can run on.
    let seen = crate::senses::sense(w, id, st, |sight| watch(w, sight));
    let h = seen.threat_pos.map(f64::from);
    // A hunter on a hunt doesn't run from a noise (its own chase's fight and its prey's calls):
    // a pack ran from "0" twice in its chase (wolvesPack5).
    if seen.threat == 0 && on_hunt(w, id) {
        return false;
    }
    if seen.aware >= animals_core::perception::NOTICED && (seen.threat != 0 || h != [0.0; 3]) {
        // A big plant eater faces a lone hunter, a mother stays between it and her young.
        if seen.threat != 0 && stands_ground(w, id, st, seen.threat) {
            w.component_mut(id, &WARY)
                .map(|u| *u = now + crate::senses::EVERY);
            return true;
        }
        // It calls: its herd or pack near know at once and run with it.
        let heard = crate::senses::alarm(w, id, st, seen.threat, seen.threat_pos);
        if heard > 0 {
            weft_sdk::info!("animal {id} calls the alarm: {heard} of its kind near run too");
        }
        let to = crate::senses::escape(w, st, h, 10.0, crate::senses::herd_mid(w, id));
        weft_sdk::debug!("animal {id} runs from {} (awareness {:.2}) to {to:?}", seen.threat, seen.aware);
        w.navigate(id, to, 1.6);
        // The alarm as it bolts: a snort, the tail flagged, a bark (whatever its kind's is).
        crate::wild::alarm(w, id);
        w.component_mut(id, &WARY).map(|u| *u = now + RUN);
        crate::senses::ran(w, id, false);
        return true;
    }
    if seen.aware >= animals_core::perception::STARE && seen.threat != 0 {
        crate::senses::stare(w, id, st, seen.threat_pos);
        // Held till its next look.
        w.component_mut(id, &WARY)
            .map(|u| *u = now + crate::senses::EVERY);
        return true;
    }
    false
}

/// Blocks it watches for hunters: `WATCH` by the weather's `sight`, half again in a blood moon.
pub(crate) fn watch(w: &World, sight: f32) -> f64 {
    let blood = animals_core::sky::totality(w.seed(), w.day_time());
    WATCH * sight as f64 * if blood { animals_core::sky::BLOOD_MOON_WATCH as f64 } else { 1.0 }
}

/// A goal for wild herd animals on the ground (not birds: a flock's `wary` takes off): in a blood
/// moon's totality, one more than `HUDDLE` from the middle of its herd near walks back to it (held,
/// as a flight from a hunter, for `HUDDLE_WALK` ticks).
pub fn huddle(w: &World, id: u64, st: &EntityState) -> bool {
    use animals_core::sky::HUDDLE;
    let now = w.game_time();
    if !(now + id).is_multiple_of(20) || animal::tamed(w, id) || !animals_core::sky::totality(w.seed(), w.day_time()) {
        return false;
    }
    let herd = w
        .component(id, &ANIMAL)
        .and_then(|a| a.species())
        .is_some_and(|s| s.bio.social == animals_core::biology::Social::Herd);
    if !herd {
        return false;
    }
    // Its herd within `HUDDLE_LOOK` from its last look round (`senses`: a herd's counts its kin
    // that far in a blood moon), itself taken out of the middle.
    let seen = crate::senses::sense(w, id, st, |sight| watch(w, sight));
    let n = seen.kin as f64 - 1.0;
    if n <= 0.0 {
        return false;
    }
    let p = [st.pos.x, st.pos.y, st.pos.z];
    let mid: [f64; 3] = std::array::from_fn(|i| (seen.kin_mid[i] as f64 * seen.kin as f64 - p[i]) / n);
    if dist2(mid, st.pos) <= (HUDDLE * HUDDLE) as f64 {
        return false;
    }
    w.navigate(id, mid, 1.2);
    weft_sdk::info!("animal {id} huddles back to its herd ({:.1} blocks out)", dist2(mid, st.pos).sqrt());
    if let Some(u) = w.component_mut(id, &WARY) {
        *u = now + HUDDLE_WALK;
    }
    true
}

/// Whether `id` stands its ground against hunter `threat` instead of running
/// (`animals_core::roles`): a big plant eater against a lone hunter, or a mother big enough with
/// her young near. It faces the hunter, and charges one that comes within `CHARGE` blocks.
fn stands_ground(w: &World, id: u64, st: &EntityState, threat: u64) -> bool {
    use animals_core::roles;
    let Some(me) = crate::senses::facts(w, id) else { return false };
    let Some(s) = me.species() else { return false };
    if s.hunts() || me.young != 0 || me.tamed != 0 {
        return false;
    }
    let Some(t) = w.entity(threat) else { return false };
    // A fierce one stands from a smaller size (`roles::FIERCE_STAND`).
    let stands_kg = roles::STANDS_KG * (1.0 - (1.0 - roles::FIERCE_STAND) * roles::lean(me.persona()[2]).max(0.0));
    let big_alone = me.mass >= stands_kg && {
        // Lone: no other of its kind near it.
        w.targets_near(t.pos, roles::LONE, None)
            .into_iter()
            .filter(|&o| o != threat && w.entity(o).is_some_and(|e| e.kind == t.kind))
            .count()
            == 0
    };
    let mother = me.mass >= roles::GUARDS_KG
        && w.component(id, &ANIMAL).is_some_and(|a| a.female != 0)
        && w.entities_near(st.pos, roles::YOUNG_NEAR, Some(s.kind))
            .into_iter()
            .any(|o| {
                o != id
                    && w.component(o, &crate::breeding::BREEDING)
                        .is_some_and(|b| b.mother == id && b.growing > 0)
            });
    if !(big_alone || mother) {
        return false;
    }
    crate::senses::face(w, id, st, [t.pos.x as f32, t.pos.y as f32, t.pos.z as f32]);
    let d2 = dist2(t.pos, st.pos);
    {
        use crate::anim::{self, Act};
        if d2 <= roles::CHARGE * roles::CHARGE * 4.0 {
            anim::hold(w, id, Act::Charge);
        } else {
            // It rears as a stand begins (it didn't stand at its last look; staring before isn't
            // the start: the PC look's deer stared first and never reared) and every few seconds
            // after (not over a rear still on), staring it down.
            let now = w.game_time();
            let began = w
                .component(id, &STOOD)
                .is_none_or(|t| now > t + 3 * crate::senses::EVERY);
            let again = !anim::busy(w, id) && ((now + id) / crate::senses::EVERY).is_multiple_of(REAR_EVERY);
            if began || again {
                anim::act(w, id, Act::Rear);
            }
            anim::hold(w, id, Act::Stare);
        }
    }
    if let Some(c) = w.component_mut(id, &STOOD) {
        *c = w.game_time();
    }
    if d2 <= roles::CHARGE * roles::CHARGE * 4.0 {
        w.navigate(id, t.pos, roles::CHARGE_SPEED);
    }
    if d2 <= roles::CHARGE * roles::CHARGE {
        w.damage(threat, Damage::new(t.max_health * roles::CHARGE_HIT, damage_cause::ATTACK).by(id));
        weft_sdk::info!("{} {id} charges {} ({})", s.name, threat, if mother { "guarding its young" } else { "a lone hunter" });
    } else if (w.game_time() + id).is_multiple_of(40) {
        weft_sdk::info!(
            "{} {id} stands its ground against {} ({})",
            s.name,
            threat,
            if mother { "guarding its young" } else { "a lone hunter" }
        );
    }
    true
}

/// Where a chasing hunter runs and how fast (`animals_core::roles`): at where its prey will be
/// (`LEAD_SECS` ahead at most); in a pack, all but the driver wide at its flanks while it is
/// farther than `FLANK_FROM`; an ambusher stalks slowly to `BURST` blocks, then charges (giving
/// up a charge after `BURST_TICKS`).
fn chase(w: &World, id: u64, st: &EntityState, s: &Species, prey: u64, pack: bool, now: u64) -> (Option<[f64; 3]>, f32) {
    use animals_core::roles;
    let Some(e) = w.entity(prey) else { return (None, 1.4) };
    let (dx, dz) = (e.pos.x - st.pos.x, e.pos.z - st.pos.z);
    let d = (dx * dx + dz * dz).sqrt();
    // Where it will be: its velocity (blocks a second) over the time to close in, at most a second.
    let lead = (d / 8.0).min(roles::LEAD_SECS);
    let mut aim = [e.pos.x + e.vel.x * lead, e.pos.y, e.pos.z + e.vel.z * lead];
    let speed = if pack { PACK_SPEED } else { 1.4 };
    if pack && d > roles::FLANK_FROM {
        // The driver straight in; the others to its sides, a little ahead.
        let side = match id % 3 {
            1 => 1.0,
            2 => -1.0,
            _ => 0.0,
        };
        let (hx, hz) = {
            let v = (e.vel.x * e.vel.x + e.vel.z * e.vel.z).sqrt();
            if v > 0.5 { (e.vel.x / v, e.vel.z / v) } else { (dx / d.max(0.01), dz / d.max(0.01)) }
        };
        aim = [
            aim[0] + (-hz * side + hx * 0.5 * side.abs()) * roles::FLANK,
            aim[1],
            aim[2] + (hx * side + hz * 0.5 * side.abs()) * roles::FLANK,
        ];
    }
    if !pack && roles::ambushes(s.family.name) {
        let started = w.component(id, &CHARGING).unwrap_or(0);
        if d > roles::BURST && started == 0 {
            // Creeping in, crouched low.
            crate::anim::hold(w, id, crate::anim::Act::Stalk);
            return (Some(aim), roles::STALK);
        }
        if started == 0 {
            w.component_mut(id, &CHARGING).map(|c| *c = now.max(1));
            weft_sdk::info!("{} {id} breaks cover and charges {prey} ({d:.1} blocks)", s.name);
            crate::anim::clear(w, id);
            crate::anim::act(w, id, crate::anim::Act::Pounce);
        } else if now > started + roles::BURST_TICKS {
            // A charge that failed: it gives up and stalks again later.
            w.component_mut(id, &CHARGING).map(|c| *c = 0);
            w.component_mut(id, &HUNT)
                .map(|h| *h = [0, 0, now + LOOK_EVERY, 0]);
            return (None, 0.0);
        }
        return (Some(aim), 1.6);
    }
    (Some(aim), speed)
}

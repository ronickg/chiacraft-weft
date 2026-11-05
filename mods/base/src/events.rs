//! Sky events in the world (`vitals::events` has the schedule and numbers; reports/Research round
//! 17.md's addendum, Ronald: "Yes, as designed"): the dusk line foretelling tonight's event, and
//! the blood moon's monsters, and the big impacts (Addendum 2): where one lands, the line telling
//! each player which way it fell, and the pointer over the hotbar.
//!
//! A blood moon's monsters come on top of the hostile ones (`BLOOD_MOON_CAP` within 64 blocks of
//! each player, `Difficulty::blood_moon_cap`), spawned here every two seconds in totality by the
//! engine's own rules for monsters: 20-45 blocks out, light 6 or less, on the ground, and through
//! `question::NATURAL_SPAWN`, so other mods' vetoes hold. They keep out of a campfire's ring and
//! leave a player in their respawn grace be, as every monster does (`mobs::keep_from_fire`): a
//! blood moon is a night to sit by the fire. They're bolder (more health and speed, a longer
//! reach), not deadlier, and at its end they walk off and are gone.

use std::collections::HashMap;
use std::sync::Mutex;

use vitals::events::{self as ev, Event};
use weft_sdk::*;

/// The blood moon's monsters alive: (entity, dimension).
static WALKING: Mutex<Vec<(u64, u32)>> = Mutex::new(Vec::new());
/// The day each player was last told tonight's event.
static TOLD: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);
/// Players out for a blood moon from its start, by day: "Red Night" if they see it through.
static WATCHING: Mutex<Option<HashMap<u64, u64>>> = Mutex::new(None);

const MODIFIER: &str = "base:blood_moon";
/// Ticks between the spawner's rounds (the engine's natural spawner's), and tries a player a round.
const ROUND_TICKS: u64 = 40;
const TRIES: u32 = 4;
/// Blocks out from a player a monster may appear (the engine's), and how far its cap counts.
const SPAWN_REACH: (f64, f64) = (20.0, 45.0);
const CAP_REACH: f64 = 64.0;
/// Up and down from the player's feet a spot is looked for.
const SPAWN_UP: i32 = 16;
/// Monsters walking off at the end are gone this far from every player.
const GONE_AT: f64 = 40.0;

weft_sdk::value! {
    /// The coarse cells (`vitals::events::BUILT_CELL` across) a player placed blocks in, the
    /// newest `BUILT_CELLS` as (x, z) pairs: a big impact keeps away from them.
    pub struct Built { pub cells: [i32; 64], pub len: u32, pub next: u32 }
}
weft_sdk::value! {
    /// The fallen star a player is pointed to, until the game day `until` or until they reach it.
    pub struct Star { pub x: i32, pub z: i32, pub until: u64, pub set: u32 }
}
/// One in `ECLIPSE_ASH_ONE_IN` blood-moon monsters leaves a pinch of it (a dye for a red-black
/// cloak; cosmetic, so never a grind).
pub static ECLIPSE_ASH: Item = Item::named("eclipse_ash");
pub static BUILT: Component<Built> = Component::named("built_cells");
pub static STAR: Component<Star> = Component::named("fallen_star");
/// Big impacts: off, the looks only (the streak, the boom, a plume in the sea; no crater), or on.
pub static SKY_IMPACTS: Config<u32> = Config::named("sky_impacts");
static STAR_MARK: HudElement = HudElement::named("fallen_star");
/// The day the last big impact fell in each dimension, so it falls once.
static FELL: Mutex<Vec<(u32, u64)>> = Mutex::new(Vec::new());
/// The day each dimension's impact was last said to be waiting (logged once).
static WAITED: Mutex<Vec<(u32, u64)>> = Mutex::new(Vec::new());
/// The 8 directions' keys, north first and clockwise (`vitals::events::compass`).
static LEAVES: Tag = Tag::named("leaves");
const DIRECTIONS: [&str; 8] = ["north", "north_east", "east", "south_east", "south", "south_west", "west", "north_west"];

pub fn register(r: &mut Registry) {
    r.config(&SKY_IMPACTS)
        .choice(2, &["off", "looks", "on"])
        .world();
    r.entity_component(&BUILT, Built { cells: [0; 64], len: 0, next: 0 }, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.attach(&PLAYER, &BUILT);
    r.entity_component(
        &STAR,
        Star { x: 0, z: 0, until: 0, set: 0 },
        component_flags::SAVE | component_flags::KEEP_ON_RESPAWN | component_flags::SYNC_OWNER,
    );
    r.attach(&PLAYER, &STAR);
    // Where players build, for big impacts to keep away from.
    r.on(&actions::PLACE_BLOCK, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| e.player != 0) {
            built_at(&w, e.player, e.pos);
        }
    });
    // Over the hotbar, for a game day or until reached: the way to the fallen star.
    r.hud_element(&STAR_MARK, hud_anchor::ABOVE_HOTBAR, 3, |ui| {
        ui.text_bind(|c, out| {
            let (Some(s), Some(me)) = (c.component(&STAR).filter(|s| s.set != 0), c.player_state()) else {
                out.text("");
                return;
            };
            if me.health <= 0.0 {
                out.text("");
                return;
            }
            let (dx, dz) = (s.x as f64 + 0.5 - me.pos.x, s.z as f64 + 0.5 - me.pos.z);
            let dist = (dx * dx + dz * dz).sqrt().round() as i64;
            let way = format!("hud.base.way.{}", crate::death::way((-dx).atan2(-dz) as f32, me.yaw));
            out.set("hud.base.fallen_star", &[Arg::Int(dist), Arg::Key(&way)]);
        });
    });
    r.world_tick(tick);
    r.item(&ECLIPSE_ASH)
        .tag("dyes")
        .tab(&crate::tabs::INGREDIENTS);
    // Dying ends a player's watch of the blood moon; a blood-moon monster's death may leave ash.
    r.on(&actions::DIE, |b| {
        let w = b.world();
        let mut watching = WATCHING.lock().unwrap();
        let watching = watching.get_or_insert_with(HashMap::new);
        for d in b.iter() {
            watching.remove(&d.entity);
            if is_blood(d.entity) && w.random_below(ev::ECLIPSE_ASH_ONE_IN) == 0 {
                w.spawn_item(d.pos, ItemStack::new(ECLIPSE_ASH.id(), 1));
            }
        }
    });
}

/// Tonight's event in `w`, if any.
pub fn tonight(w: &World) -> Option<Event> {
    ev::tonight(w.seed(), w.day_time() / vitals::calendar::DAY_TICKS)
}

/// Whether a blood moon is in totality now, when its monsters walk and kills give more XP.
pub fn blood_totality(w: &World) -> bool {
    tonight(w) == Some(Event::BloodMoon) && ev::in_totality(ev::hour_of(w.day_time()))
}

/// Whether `id` is one of the blood moon's monsters (they see players further: `mobs.rs`).
pub fn is_blood(id: u64) -> bool {
    WALKING.lock().unwrap().iter().any(|e| e.0 == id)
}

/// The dusk line for `player`, once a day, `DUSK_LINE_HOURS` before `sunset` (`hints.rs` calls it
/// each second with the hour and the player's sunset).
pub fn dusk(w: &World, player: u64, hour: f32, sunset: f32) {
    let Some(e) = tonight(w).filter(|e| e.announced()) else { return };
    if !(sunset - ev::DUSK_LINE_HOURS..sunset + 1.0).contains(&hour) {
        return;
    }
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    let mut told = TOLD.lock().unwrap();
    let told = told.get_or_insert_with(HashMap::new);
    if told.get(&player) == Some(&day) {
        return;
    }
    told.insert(player, day);
    // On gentle the blood moon is a look only: no warning of monsters.
    let key = if e == Event::BloodMoon && !base_api::difficulty().blood_moon_gameplay {
        "hint.base.blood_moon_gentle".to_string()
    } else {
        format!("hint.base.{}", e.name())
    };
    weft_sdk::info!("events: tonight is {} (day {day}); told {player}", e.name());
    base_api::hint(w, player, &key, &[]);
}

fn tick(w: &World) {
    let now = w.game_time();
    if !now.is_multiple_of(ROUND_TICKS) {
        return;
    }
    if w.dimension() == crate::OVERWORLD.id() {
        impacts(w);
    }
    pointers(w);
    let event = tonight(w);
    sky_watchers(w, event);
    let strength = if event == Some(Event::BloodMoon) { ev::blood_moon_strength(ev::hour_of(w.day_time())) } else { 0.0 };
    watch(w, strength);
    let dim = w.dimension();
    WALKING
        .lock()
        .unwrap()
        .retain(|&(id, d)| d != dim || w.entity(id).is_some());
    if strength >= 1.0 {
        let diff = base_api::difficulty();
        if diff.blood_moon_gameplay && settings::MOB_SPAWNING.get() {
            for player in w.players() {
                spawn_near(w, player, diff.blood_moon_cap, diff.blood_moon_health);
            }
        }
    } else if strength == 0.0 {
        walk_off(w);
    }
}

/// Up to `TRIES` spots near `player` for a blood-moon monster, while fewer than `cap` are near.
fn spawn_near(w: &World, player: u64, cap: u32, health: f32) {
    if w.outside_survival(player) || crate::death::in_grace(w, player) {
        return;
    }
    let Some(p) = w.entity(player) else { return };
    let dim = w.dimension();
    let near = WALKING
        .lock()
        .unwrap()
        .iter()
        .filter(|&&(id, d)| {
            d == dim
                && w.entity(id)
                    .is_some_and(|e| (e.pos - p.pos).length_squared() < CAP_REACH * CAP_REACH)
        })
        .count() as u32;
    if near >= cap {
        return;
    }
    for _ in 0..TRIES {
        let angle = unit(w) as f64 * std::f64::consts::TAU;
        let reach = SPAWN_REACH.0 + (SPAWN_REACH.1 - SPAWN_REACH.0) * unit(w) as f64;
        let (x, z) = ((p.pos.x + angle.cos() * reach).floor() as i32, (p.pos.z + angle.sin() * reach).floor() as i32);
        let Some(spot) = ground_spot(w, x, p.pos.y.floor() as i32, z) else { continue };
        if abi::light::effective(w.light(spot), w.daylight()) > 6 || crate::mobs::campfire_near(w, spot.center()).is_some() {
            continue;
        }
        let kind = pick_kind(w);
        let mut q = abi::Query { pos: spot, other: kind.id() as u64, amount: 1.0, ..abi::Query::DEFAULT };
        if !w.ask(&question::NATURAL_SPAWN, &mut q) {
            continue;
        }
        let id = w.spawn(kind, [spot.x as f64 + 0.5, spot.y as f64, spot.z as f64 + 0.5]);
        if id == 0 {
            continue;
        }
        w.attribute_add_modifier(id, &attribute::MAX_HEALTH, MODIFIER, attr_op::MULTIPLY_BASE, (health - 1.0) as f64, 0);
        w.attribute_add_modifier(id, &attribute::MOVEMENT_SPEED, MODIFIER, attr_op::MULTIPLY_BASE, (ev::BLOOD_MOON_SPEED - 1.0) as f64, 0);
        w.heal(id, 1000.0);
        WALKING.lock().unwrap().push((id, dim));
        weft_sdk::info!(
            "events: a blood-moon {} at {spot:?} for {player} ({} near)",
            name_of(abi::entry_kind::ENTITY, kind.id()),
            near + 1
        );
        return;
    }
}

/// Where a monster may stand in column (`x`, `z`) near height `y`: on something solid, two free
/// above, the highest such within `SPAWN_UP`.
fn ground_spot(w: &World, x: i32, y: i32, z: i32) -> Option<BlockPos> {
    let solid = |p: BlockPos| w.block_flags(p) & abi::block_flags::SOLID != 0;
    (y - SPAWN_UP..=y + SPAWN_UP)
        .rev()
        .map(|y| BlockPos::new(x, y, z))
        .find(|&p| solid(p.below()) && !solid(p) && !solid(p.above()) && w.block_flags(p) & abi::block_flags::LIQUID == 0)
}

/// A monster by `BLOOD_MOON_WEIGHTS`.
fn pick_kind(w: &World) -> &'static EntityKind {
    let total: u32 = ev::BLOOD_MOON_WEIGHTS.iter().map(|k| k.1).sum();
    let mut pick = w.random_below(total);
    for (name, weight) in ev::BLOOD_MOON_WEIGHTS {
        if pick < weight {
            return match name {
                "zombie" => &crate::mobs::ZOMBIE,
                "spider" => &crate::mobs::SPIDER,
                "skeleton" => &crate::mobs::SKELETON,
                _ => &crate::mobs::CREEPER,
            };
        }
        pick -= weight;
    }
    &crate::mobs::ZOMBIE
}

/// The blood moon over: its monsters walk away from the nearest player and are gone out of sight.
fn walk_off(w: &World) {
    let dim = w.dimension();
    let players: Vec<Vec3> = w
        .players()
        .into_iter()
        .filter_map(|p| w.entity(p))
        .map(|e| e.pos)
        .collect();
    WALKING.lock().unwrap().retain(|&(id, d)| {
        if d != dim {
            return true;
        }
        let Some(e) = w.entity(id) else { return false };
        let nearest = players.iter().min_by(|a, b| {
            (**a - e.pos)
                .length_squared()
                .total_cmp(&(**b - e.pos).length_squared())
        });
        match nearest {
            Some(&p) if (p - e.pos).length_squared() < GONE_AT * GONE_AT => {
                let away = e.pos - p;
                let len = away.length_squared().sqrt().max(0.1);
                w.navigate(id, e.pos + away * (GONE_AT / len), 1.0);
                true
            }
            nearest => {
                let far = nearest.map_or(f64::INFINITY, |&p| (p - e.pos).length_squared().sqrt());
                weft_sdk::info!("events: a blood-moon monster ({id}) walked off, gone {far:.0} blocks from the nearest player");
                w.despawn(id);
                false
            }
        }
    });
}

/// Players out under the open sky in the dark of a shower's peak (from 23:00) or an aurora storm
/// (from full dark, 21:00) to 05:00: Wish Upon and Sky on Fire.
fn sky_watchers(w: &World, event: Option<Event>) {
    let (from, milestone) = match event {
        Some(Event::Meteors { peak: true }) => (23.0, base_api::milestones::WISH_UPON),
        Some(Event::AuroraStorm) => (21.0, base_api::milestones::SKY_ON_FIRE),
        _ => return,
    };
    let hour = ev::hour_of(w.day_time());
    if !(hour >= from || hour < 5.0) {
        return;
    }
    for p in w.players() {
        let Some(e) = w.entity(p) else { continue };
        let head = BlockPos::new(e.pos.x.floor() as i32, (e.pos.y + e.size.eye as f64).floor() as i32, e.pos.z.floor() as i32);
        if w.sky_light(head) >= 15 {
            base_api::milestone(w, p, milestone);
        }
    }
}

/// "Red Night": players out when a blood moon starts are watched; those still alive when it has
/// cleared (`strength` 0 again, after 03:00) get the milestone.
fn watch(w: &World, strength: f32) {
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    let hour = ev::hour_of(w.day_time());
    let mut watching = WATCHING.lock().unwrap();
    let watching = watching.get_or_insert_with(HashMap::new);
    if strength > 0.0 && strength < 1.0 && hour > 12.0 {
        // Reddening: whoever is out now watches it.
        for p in w.players() {
            if !w.outside_survival(p) {
                watching.entry(p).or_insert(day);
            }
        }
    } else if strength == 0.0 && hour < 12.0 {
        watching.retain(|&p, &mut d| {
            if d == day && w.entity(p).is_some() {
                base_api::milestone(w, p, base_api::milestones::RED_NIGHT);
                weft_sdk::info!("events: {p} saw the blood moon through");
            }
            false
        });
    }
}

// ---------------------------------------------------------------- big impacts

/// `player` placed a block at `pos`: its cell joins their built cells if new.
fn built_at(w: &World, player: u64, pos: BlockPos) {
    let (cx, cz) = (pos.x.div_euclid(ev::BUILT_CELL), pos.z.div_euclid(ev::BUILT_CELL));
    let Some(b) = w.component_mut(player, &BUILT) else { return };
    let known = (0..b.len as usize).any(|i| b.cells[2 * i] == cx && b.cells[2 * i + 1] == cz);
    if !known {
        let i = b.next as usize % ev::BUILT_CELLS;
        (b.cells[2 * i], b.cells[2 * i + 1]) = (cx, cz);
        b.next = ((i + 1) % ev::BUILT_CELLS) as u32;
        b.len = (b.len + 1).min(ev::BUILT_CELLS as u32);
    }
}

/// Today's big impact, once, at its hour (the warning `IMPACT_WARN_SECS` before).
fn impacts(w: &World) {
    let rule = SKY_IMPACTS.get();
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    let Some(at) = ev::impact_on(w.seed(), day) else { return };
    // Through the day from 06:00 (0) to the next 06:00 (1), the impact's hour on the same scale.
    let now = (w.day_time() % vitals::calendar::DAY_TICKS) as f32 / vitals::calendar::DAY_TICKS as f32;
    let due = (at - 6.0).rem_euclid(24.0) / 24.0;
    if rule == 0 || now < due {
        return;
    }
    let dim = w.dimension();
    if FELL
        .lock()
        .unwrap()
        .iter()
        .any(|&(d, f)| d == dim && f == day)
    {
        return;
    }
    // From its hour it falls the first round a site is free, when a player is out in the wilds
    // (none is near spawn or anyone's base: PC run fh_impact1 stood at spawn and found none), and
    // into the sea in the day's last minutes if none was.
    let last = now >= 1.0 - ev::IMPACT_LAST_HOURS / 24.0;
    let site = match site(w) {
        Ok(p) => Some(p),
        Err(why) if last => {
            weft_sdk::info!("events: no site for day {day}'s impact by its end ({why}): into the sea");
            None
        }
        Err(why) => {
            let mut told = WAITED.lock().unwrap();
            if !told.contains(&(dim, day)) {
                told.retain(|&(d, _)| d != dim);
                told.push((dim, day));
                weft_sdk::info!("events: day {day}'s impact is due and waits for a site ({why})");
            }
            return;
        }
    };
    {
        let mut fell = FELL.lock().unwrap();
        fell.retain(|&(d, _)| d != dim);
        fell.push((dim, day));
    }
    weft_sdk::info!(
        "events: a big impact on day {day} (due {at:.1}h, now {:.1}h): {}",
        ev::hour_of(w.day_time()),
        match site {
            Some(p) => format!("at {p:?}"),
            None => "into the sea".into(),
        }
    );
    let Some(site) = site else {
        tell_all(w, None);
        return;
    };
    tell_all(w, Some(site));
    if rule == 2 {
        carve(w, site);
    }
}

/// The world's simulation distance in cubes: what is loaded round each player.
pub(crate) fn simulation() -> i32 {
    static SIMULATION: Config<u32> = Config::named("core:simulation_distance");
    // The engine's default (8) when the setting reads nothing (PC run fh_impact8 tried 16-40 blocks
    // out, a reach from 0).
    match SIMULATION.get() as i32 {
        n if n >= 2 => n,
        _ => ev::SIMULATION_DEFAULT,
    }
}

/// Where a big impact lands: loaded land between `vitals::events::impact_reach` from a
/// player, open to the sky, level and dry over the crater's width, and far from every player,
/// their built cells, beds and tombstones, and the world spawn. None: it falls in the sea.
fn site(w: &World) -> Result<BlockPos, String> {
    let players: Vec<(u64, Vec3)> = w
        .players()
        .into_iter()
        .filter_map(|p| w.entity(p).map(|e| (p, e.pos)))
        .collect();
    if players.is_empty() {
        return Err("no players".into());
    }
    // What a site keeps away from: (x, z, how far).
    let (near, far) = ev::impact_reach(simulation());
    let mut away: Vec<(f64, f64, f64)> = players
        .iter()
        .map(|&(_, p)| (p.x, p.z, near as f64))
        .collect();
    let build = ev::IMPACT_FROM_BUILDS as f64;
    let spawn = crate::fair_spawn(w.seed());
    away.push((spawn[0] as f64, spawn[2] as f64, build));
    let half = ev::BUILT_CELL as f64 / 2.0;
    for &(p, _) in &players {
        if let Some(b) = w.component(p, &BUILT) {
            for i in 0..b.len as usize {
                let (cx, cz) = (b.cells[2 * i] as f64, b.cells[2 * i + 1] as f64);
                away.push((cx * ev::BUILT_CELL as f64 + half, cz * ev::BUILT_CELL as f64 + half, build + half * 1.5));
            }
        }
        if let Some(bed) = w.component(p, &base_api::BED_SPAWN).filter(|b| b.set != 0) {
            away.push((bed.bed[0] as f64, bed.bed[2] as f64, build));
        }
        if let Some(g) = w.component(p, &base_api::GRAVE).filter(|g| g.set != 0) {
            away.push((g.x as f64, g.z as f64, build));
        }
    }
    let fields = base_api::land::fields(w.seed());
    let clear = |x: f64, z: f64| {
        away.iter()
            .all(|&(ax, az, r)| (ax - x).powi(2) + (az - z).powi(2) >= r * r)
    };
    let radius = ev::IMPACT_SITE_RADIUS;
    // Why tries failed: near something, no open dry ground there, not level.
    let (mut near, mut ground, mut uneven) = (0, 0, 0);
    for _ in 0..ev::IMPACT_SITE_TRIES {
        let (_, from) = players[w.random_below(players.len() as u32) as usize];
        let angle = unit(w) as f64 * std::f64::consts::TAU;
        let reach = near as f64 + (far - near) as f64 * unit(w) as f64;
        let (x, z) = (from.x + angle.cos() * reach, from.z + angle.sin() * reach);
        if !clear(x, z) {
            near += 1;
            continue;
        }
        let (x, z) = (x.floor() as i32, z.floor() as i32);
        // From the land's own height there, not the player's: one flying, up a cliff or in a
        // cave looked 48 blocks round itself and found no ground (PC run fh_impact2).
        let Some(top) = open_ground(w, x, land_height(&fields, x, z), z) else {
            ground += 1;
            continue;
        };
        // Level and dry across the crater: eight points round it within `IMPACT_LEVEL`.
        let level = (0..8).all(|k| {
            let a = k as f64 * std::f64::consts::FRAC_PI_4;
            let (px, pz) = (x + (a.cos() * radius as f64) as i32, z + (a.sin() * radius as f64) as i32);
            open_ground(w, px, top.y, pz).is_some_and(|p| (p.y - top.y).abs() <= ev::IMPACT_LEVEL)
        });
        if level {
            return Ok(top);
        }
        uneven += 1;
    }
    Err(format!(
        "{near} near players, builds or spawn, {ground} not open dry loaded ground, {uneven} not level; reach {lo}-{hi} at simulation {}",
        simulation(),
        lo = ev::impact_reach(simulation()).0,
        hi = ev::impact_reach(simulation()).1
    ))
}

/// The land's height at block (`x`, `z`), as worldgen shapes it (before caves and plants).
pub(crate) fn land_height(f: &weft_sdk::noise::fixed::Fields, x: i32, z: i32) -> i32 {
    base_api::land::height(f, x * 16, z * 16) >> 4
}

/// The ground's top in column (`x`, `z`) near `y`: loaded, open to the sky, not water.
pub(crate) fn open_ground(w: &World, x: i32, y: i32, z: i32) -> Option<BlockPos> {
    const LOOK: i32 = 48;
    for gy in (y - LOOK..=y + LOOK).rev() {
        let p = BlockPos::new(x, gy, z);
        // Air cubes high over the land may not be loaded at all: passed over, not the end of
        // the look (PC run fh_impact5: 96 of 96 tries stopped above the ground).
        let Some(state) = w.block_loaded(p) else { continue };
        if state == 0 || LEAVES.on_block(state) {
            continue;
        }
        let flags = w.block_flags(p);
        if flags & abi::block_flags::LIQUID != 0 {
            return None;
        }
        if flags & abi::block_flags::SOLID == 0 {
            continue;
        }
        return (w.sky_light(p.above()) >= 15).then_some(p);
    }
    None
}

/// Whether `pos` is within `clear` blocks of a cell an online player built in.
pub(crate) fn near_built(w: &World, pos: BlockPos, clear: i32) -> bool {
    let half = ev::BUILT_CELL as f64 / 2.0;
    let reach = clear as f64 + half * 1.5;
    w.players().into_iter().any(|p| {
        w.component(p, &BUILT).is_some_and(|b| {
            (0..b.len as usize).any(|i| {
                let (cx, cz) =
                    (b.cells[2 * i] as f64 * ev::BUILT_CELL as f64 + half, b.cells[2 * i + 1] as f64 * ev::BUILT_CELL as f64 + half);
                (cx - pos.x as f64).powi(2) + (cz - pos.z as f64).powi(2) < reach * reach
            })
        })
    })
}

/// Every player in the dimension hears which way it fell (and how far, in bands), and on land
/// gets the pointer for a game day.
fn tell_all(w: &World, site: Option<BlockPos>) {
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    for p in w.players() {
        let Some(e) = w.entity(p) else { continue };
        let at = site.map_or(e.pos + Vec3::new(0.0, 0.0, -1000.0), |s| s.center());
        let (dx, dz) = (at.x - e.pos.x, at.z - e.pos.z);
        let dist = (dx * dx + dz * dz).sqrt();
        let band = if dist < ev::IMPACT_BANDS.0 {
            "near"
        } else if dist < ev::IMPACT_BANDS.1 {
            "far"
        } else {
            "very_far"
        };
        let dir = format!("events.base.dir.{}", DIRECTIONS[ev::compass(dx, dz) as usize]);
        let band = format!("events.base.band.{band}");
        match site {
            Some(s) => {
                w.hint(p, "hint.base.star_fell", &[Arg::Key(&band), Arg::Key(&dir)]);
                if let Some(star) = w.component_mut(p, &STAR) {
                    *star = Star { x: s.x, z: s.z, until: day + ev::POINTER_DAYS, set: 1 };
                }
            }
            None => {
                w.hint(p, "hint.base.star_sea", &[]);
            }
        }
    }
}

/// Pointers end after their day, or once their player is within `POINTER_NEAR` of the star.
fn pointers(w: &World) {
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    for p in w.players() {
        let Some(e) = w.entity(p) else { continue };
        let Some(star) = w.component_mut(p, &STAR) else { continue };
        if star.set == 0 {
            continue;
        }
        let near = (star.x as f64 + 0.5 - e.pos.x).powi(2) + (star.z as f64 + 0.5 - e.pos.z).powi(2) < ev::POINTER_NEAR.powi(2);
        if near || day >= star.until {
            star.set = 0;
        }
    }
}

/// The crater and what lies in it: Terrain's live carve (`impact::start`, on its branch, not in
/// dev yet). Until it lands an impact on land is the looks only, as `sky_impacts` 1 has it.
fn carve(w: &World, site: BlockPos) {
    // What the core will hold, decided by the site (`stars::core_block` places it with the carve).
    crate::stars::core_begins(w, w.seed_at(site));
    weft_sdk::info!("events: the crater at {site:?} waits for Terrain's impact::start");
}

fn unit(w: &World) -> f32 {
    (w.random() >> 40) as f32 / (1u64 << 24) as f32
}

//! The player's vitals (`notes/SURVIVAL.md`, the `vitals` crate): what's eaten digests and sets
//! how much of the health and stamina stats is reached, stamina pays for sprinting, jumping,
//! swimming, fighting and mining and comes back from food, health comes back from protein and
//! greens, breath runs out under water. No hunger: an empty stomach makes one weak, never dead.
//! Dying empties the stomach and costs some XP. Creative skips it all.

use base_api::{FOOD, FRESHNESS, HEALTH_SCALE, HOT, MADE, VITALS, Vitals, freshness};
use vitals::effects::{self, Effect, Group};
use vitals::stats::Stat;
use vitals::temperature::{self, Body};
use vitals::{calendar, difficulty, food, levels, stamina, stomach};
use weft_sdk::*;

pub const MAX_AIR: f32 = 300.0;
const DT: f32 = 1.0 / 20.0;
/// The engine's max health a player has before food (the base game's 20).
const ENGINE_HEALTH: f32 = 20.0;
/// Stamina it takes to start sprinting again once run out.
const SPRINT_AGAIN: f32 = 10.0;
static WATER_LIKE: Tag = Tag::named("water");
/// Seconds one stays wet after the rain or the water.
const WET_SECS: f32 = 120.0;
/// Blocks around one searched for heat and comfort.
const HEAT_REACH: i32 = 7;
/// Seconds of resting (under a roof by a fire, or asleep) that make one rested.
pub const REST_SECS: f32 = 20.0;
static HANDHELD: Tag = Tag::named("handheld");
/// What a bandage heals, for its tooltip.
static HEALS: Component<u8> = Component::named("base:heals");
/// Torpor that wears off a second, as a share of the torpor stat (out at the stat, awake again
/// under half: about half a minute; proposed).
const TORPOR_DECAY: f32 = 0.03;
/// A blow that takes this share of max health or more opens a wound that bleeds this long.
const WOUND: f32 = 0.2;
const BLEED_SECS: f32 = 20.0;
/// Health (the vitals' 100 a person) a bandage gives back as it stops the bleeding.
pub const BANDAGE_HEALS: f32 = 10.0;

pub fn register(r: &mut Registry) {
    // How hard survival is, picked when the world is made (`vitals::difficulty`).
    r.config(&base_api::DIFFICULTY)
        .choice(1, &difficulty::ALL.map(|d| d.name))
        .world();
    r.entity_component(&VITALS, Vitals::new(MAX_AIR), component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP);
    r.attach(&PLAYER, &VITALS);
    // Food's stamps, kept and saved with each stack (`age_food`).
    r.stack_component(&MADE, 0);
    r.stack_component(&HOT, 0);
    r.stack_component(&FRESHNESS, 0);
    r.world_tick(tick);
    // A bandage (from wool): stops the bleeding and gives a little health back.
    r.item(Item::generated("bandage".into()))
        .set(&HEALS, BANDAGE_HEALS as u8)
        .use_ticks(40)
        .on_use(bandage)
        .tab(&crate::tabs::COMBAT);
    r.shapeless(&base_api::CRAFTING, "bandage", 3, &["wool"], 0);
    // No room in the stomach: it doesn't start eating (the other way round it would only find out
    // at the end).
    r.intercept(&question::USE_ITEM, |q| {
        let w = q.world();
        if FOOD.on_item(q.item()).is_some() && !w.creative(q.entity()) && !base_api::has_room(&w, q.entity(), q.item()) {
            Verdict::Deny
        } else {
            Verdict::Abstain
        }
    });
    // Mining and chopping: a swing's worth for every block, by how hard it is.
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            let hardness = HARDNESS.on_block(e.state).unwrap_or(1.0).max(0.0);
            spend(&w, e.player, stamina::work(hardness));
        }
    });
    // A blow that lands: by what it was struck with (a weapon or tool, else the fist). A heavy one
    // on a player opens a wound that bleeds.
    r.on(&ENTITY_HURT, |b| {
        let w = b.world();
        for e in b.iter().filter(|e| e.cause == damage::ATTACK) {
            let heavy = w
                .component(e.entity, &VITALS)
                .is_some_and(|v| e.amount * HEALTH_SCALE >= v.max_health * WOUND);
            if heavy {
                base_api::bleed(&w, e.entity, BLEED_SECS);
            }
        }
        for e in b.iter().filter(|e| w.players().contains(&e.source)) {
            let held = w
                .inventory(e.source)
                .group("hotbar")
                .get(w.selected_slot(e.source));
            spend(&w, e.source, stamina::swing(if HANDHELD.on_item(held.item) { 1.5 } else { 0.5 }));
        }
    });
    // Dying empties the stomach and costs part of the XP into the level.
    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            let Some(v) = w.component_mut(d.entity, &VITALS) else { continue };
            v.set_stomach(&stomach::Stomach::default());
            v.stamina = 0.0;
            v.xp = levels::after_death(v.xp, base_api::difficulty().death_xp);
            v.air = MAX_AIR;
            v.wet = 0.0;
            v.core = vitals::temperature::CORE;
            (v.poison, v.bleed, v.torpor, v.out) = (0.0, 0.0, 0.0, 0);
            // Respawning drops the body's modifiers: the next tick sets max health and speed again.
            v.max_health = -1.0;
            v.speed = -1.0;
        }
    });
    // Jumps to a day of a season (the first without one), keeping the hour.
    r.command("season", "[season:word(spring,summer,autumn,winter)] [day:int]", |cmd| {
        let w = cmd.world();
        let Some(season) = calendar::SEASONS
            .iter()
            .position(|s| Some(s.name()) == cmd.text("season"))
        else {
            return;
        };
        let day = cmd
            .int("day")
            .unwrap_or(1)
            .clamp(1, calendar::SEASON_DAYS as i64) as u32;
        w.set_day_time(calendar::on_day(w.day_time(), season as u32 * calendar::SEASON_DAYS + day - 1));
        cmd.reply("command.base.season", &[Arg::Key(&format!("season.base.{}", calendar::SEASONS[season].name())), Arg::Int(day as i64)]);
    });
    // Fills the stomach with a balanced meal (an apple, a loaf, a steak, over and over) up to that
    // many units; 0 empties it. For testing.
    r.command("feed", "[units:int]", |cmd| {
        let w = cmd.world();
        let Some(v) = w.component_mut(cmd.player(), &VITALS) else { return };
        let units = cmd.int("units").map_or(v.capacity(), |u| u.max(0) as f32);
        let mut s = stomach::Stomach::default();
        let meal = ["base:apple", "base:bread", "base:cooked_beef"].map(|n| food::find(n).unwrap_or(0));
        for &f in meal.iter().cycle().take(stomach::PORTIONS) {
            if s.used() + food::FOODS[f].units() > units || s.eat(f, 1.0, food::HOT_FOR, v.capacity()).is_err() {
                break;
            }
        }
        v.set_stomach(&s);
        cmd.reply("command.base.feed", &[Arg::Int(s.used().round() as i64)]);
    })
    .permission(permission::EVERYONE);
}

/// Spends stamina on an action (nothing in creative).
fn spend(w: &World, player: u64, amount: f32) {
    if w.creative(player) {
        return;
    }
    let mods = w
        .component(player, &VITALS)
        .map_or(effects::NONE, |v| effects::combine(v.effects()));
    if let Some(v) = w.component_mut(player, &VITALS) {
        v.stamina = (v.stamina - amount * base_api::difficulty().stamina_cost * mods.stamina_cost).max(0.0);
        v.rest_wait = (stamina::REGEN_DELAY * 20.0) as u32;
    }
}

fn tick(w: &World) {
    let d = base_api::difficulty();
    let pulse = w.time() % 20 == 0;
    for id in w.players() {
        let Some(mut st) = w.entity(id) else { continue };
        let eye = [st.pos[0], st.pos[1] + st.size[2] as f64, st.pos[2]];
        let eye_block = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
        // Only water takes the breath, whatever else burns or drags.
        let underwater = WATER_LIKE.on_block(w.get(eye_block)) && w.in_liquid(eye);
        let creative = w.creative(id);
        let Some(v) = w.component_mut(id, &VITALS) else { continue };

        // Breath: nothing takes it in creative, and what was used comes back.
        if underwater && !creative {
            v.air -= 1.0;
            if v.air <= -20.0 {
                v.air = 0.0;
                w.damage(id, 2.0, 0, damage::DROWN);
            }
        } else if v.air < MAX_AIR {
            v.air = (v.air + 4.0).min(MAX_AIR);
        }
        if creative {
            v.stamina = v.max_stamina;
            if v.speed != 1.0 {
                w.remove_modifier(id, &attribute::MOVEMENT_SPEED, "base:vitals");
                w.remove_modifier(id, &attribute::JUMP_STRENGTH, "base:vitals");
                v.speed = 1.0;
            }
            continue;
        }

        // Digesting, and what the stomach holds now.
        let mut s = v.stomach();
        s.digest(DT, d.digest);
        v.set_stomach(&s);
        let stats = v.stats();
        let capacity = v.capacity();
        let full = s.fullness(capacity);
        let (max_health, max_stamina) = stomach::max_health_stamina(stats[0], stats[1], &s, capacity);

        // The place, a second at a time: the body's temperature, wet, near a fire, sheltered.
        if pulse {
            feel(w, id, &st, v, &s, stats[Stat::Fortitude.index()], full[food::FAT]);
        }
        set(v, Effect::WellFed, s.well_fed(capacity));
        // Food poisoning, poison and bleeding, while they last.
        v.sick = (v.sick - DT).max(0.0);
        v.poison = (v.poison - DT).max(0.0);
        v.bleed = (v.bleed - DT).max(0.0);
        set(v, Effect::FoodPoisoning, v.sick > 0.0);
        set(v, Effect::Poisoned, v.poison > 0.0);
        set(v, Effect::Bleeding, v.bleed > 0.0);
        // Torpor wears off; past the torpor stat one is out cold until it falls under half.
        let torpor = stats[Stat::Torpor.index()];
        v.torpor = (v.torpor - torpor * TORPOR_DECAY * DT).max(0.0);
        if v.out == 0 && v.torpor >= torpor {
            v.out = 1;
        } else if v.out != 0 && v.torpor < torpor * 0.5 {
            v.out = 0;
        }
        set(v, Effect::Sedated, v.out != 0 || v.torpor > torpor * 0.25);
        if pulse {
            age_food(w, id);
            v.load = carried(w, id);
            let load = effects::load(v.load, stats[Stat::Weight.index()]);
            set(v, Effect::Encumbered, load == Some(Effect::Encumbered));
            set(v, Effect::Overloaded, load == Some(Effect::Overloaded));
        }
        let mods = effects::combine(v.effects());

        // Walking speed: the speed stat and what slows one; out cold, none (set again now and
        // then, as a respawn or a rejoin drops it).
        let speed = if v.out != 0 { 0.0 } else { stats[Stat::Speed.index()] * mods.speed };
        if (v.speed - speed).abs() > 0.001 || w.time() % 100 == 0 {
            v.speed = speed;
            w.add_modifier(id, &attribute::MOVEMENT_SPEED, "base:vitals", attr_op::MUL_TOTAL, (speed - 1.0) as f64, 0);
            w.add_modifier(id, &attribute::JUMP_STRENGTH, "base:vitals", attr_op::MUL_TOTAL, if v.out != 0 { -1.0 } else { 0.0 }, 0);
        }

        // Moving: sprinting, jumping and swimming cost stamina.
        let mut cost = 0.0;
        if st.sprinting != 0 && st.moved > 0.0 {
            cost += stamina::SPRINT * DT;
        }
        if st.jumped != 0 {
            cost += stamina::JUMP;
        }
        if st.in_liquid != 0 && st.moved > 0.0 {
            cost += stamina::SWIM * DT;
        }
        if cost > 0.0 {
            v.stamina -= cost * d.stamina_cost * mods.stamina_cost;
            v.rest_wait = (stamina::REGEN_DELAY * 20.0) as u32;
        }
        if v.rest_wait > 0 {
            v.rest_wait -= 1;
        } else {
            v.stamina += stamina::stamina_regen(max_stamina, &full, &mods) * DT;
        }
        v.stamina = v.stamina.clamp(0.0, max_stamina);
        v.max_stamina = max_stamina;
        // Run out, no sprinting until some has come back.
        let tired = v.stamina < 1.0 || (v.stamina < SPRINT_AGAIN && st.sprinting == 0);
        if tired || !mods.can_sprint || v.out != 0 {
            w.add_modifier(id, &attribute::CAN_SPRINT, "base:tired", attr_op::ADD, -1.0, 2);
        }

        // Max health follows the stomach; health comes back a second at a time.
        let engine_max = max_health / HEALTH_SCALE;
        if (v.max_health - max_health).abs() > 0.01 {
            v.max_health = max_health;
            w.add_modifier(id, &attribute::MAX_HEALTH, "base:fed", attr_op::ADD, (engine_max - ENGINE_HEALTH) as f64, 0);
            if st.health > engine_max {
                st.health = engine_max;
                w.set_entity(id, &st);
            }
        }
        if pulse && st.health > 0.0 && st.health < engine_max {
            let back = stamina::health_regen(max_health, &full, &mods) / HEALTH_SCALE;
            st.health = (st.health + back).min(engine_max);
            w.set_entity(id, &st);
        }
        // Freezing, heatstroke, poison and bleeding take a share of max health a second (a gentle
        // world stops them at a tenth), heatstroke stamina too; a death is put down to the worst.
        if pulse && mods.health_loss > 0.0 && st.health > 0.0 {
            let floor = if d.deadly { 0.0 } else { engine_max * 0.1 };
            let loss = (mods.health_loss * max_health * d.harm / HEALTH_SCALE).min((st.health - floor).max(0.0));
            if loss > 0.0 {
                let worst = v.effects().max_by(|a, b| {
                    a.modifiers()
                        .health_loss
                        .total_cmp(&b.modifiers().health_loss)
                });
                w.damage(id, loss, 0, worst.map_or(damage::GENERIC, cause_of));
            }
        }
        if pulse && mods.stamina_loss > 0.0 {
            v.stamina = (v.stamina - mods.stamina_loss * max_stamina * d.harm).max(0.0);
        }
    }
}

/// Binds a wound: only when bleeding or hurt.
fn bandage(u: &mut ItemUse) -> bool {
    let w = u.world();
    let player = u.player();
    let Some(mut st) = w.entity(player) else { return false };
    let Some(v) = w.component_mut(player, &VITALS) else { return false };
    let max = v.max_health / HEALTH_SCALE;
    if v.bleed <= 0.0 && st.health >= max {
        return false;
    }
    v.bleed = 0.0;
    st.health = (st.health + BANDAGE_HEALS / HEALTH_SCALE).min(max);
    w.set_entity(player, &st);
    if !w.creative(player) {
        u.set_stack(u.stack().shrink(1));
    }
    true
}

/// Turns an effect on or off.
fn set(v: &mut Vitals, e: Effect, on: bool) {
    v.effects = (v.effects & !(1 << e.index())) | if on { 1 << e.index() } else { 0 };
}

/// What a death from an effect's loss of health is put down to.
fn cause_of(e: Effect) -> u32 {
    match e {
        Effect::Heatstroke => damage::HEAT,
        Effect::Poisoned => damage::POISON,
        Effect::Bleeding => damage::BLEED,
        _ => damage::FREEZE,
    }
}

/// Kilograms one carries, everything in the inventory and worn (`weight`).
fn carried(w: &World, player: u64) -> f32 {
    let inv = w.inventory(player);
    (0..inv.size())
        .map(|i| inv.get(i))
        .filter(|s| !s.is_empty())
        .map(|s| weight(item_name(s.item)) * s.count as f32)
        .sum()
}

/// What one of an item weighs, in kilograms: game weights, not real ones (a block of stone is a
/// tenth, so a hotbar of stone stacks is about all one carries freely; proposed). Food by its
/// portion, the rest by its kind of name.
pub fn weight(name: &str) -> f32 {
    if let Some(f) = food::find(name) {
        return food::FOODS[f].grams / 1000.0;
    }
    let n = name.split_once(':').map_or(name, |(_, n)| n);
    let piece = |p: &str| n.ends_with(p);
    if piece("_helmet") || piece("_chestplate") || piece("_leggings") || piece("_boots") {
        let per = match n.split('_').next().unwrap_or("") {
            "leather" => 0.5,
            "chainmail" => 1.5,
            "iron" => 2.0,
            "gold" | "golden" => 2.5,
            "diamond" => 2.0,
            "netherite" => 3.0,
            _ => 1.0,
        };
        let share = if piece("_chestplate") {
            2.0
        } else if piece("_leggings") {
            1.5
        } else {
            1.0
        };
        return per * share;
    }
    if piece("_sword") || piece("_pickaxe") || piece("_axe") || piece("_shovel") || piece("_hoe") || n == "bow" || n == "shield" {
        return 1.0;
    }
    if piece("_ingot") || n.starts_with("raw_") || piece("_ore") || n == "coal" || n == "diamond" || n == "emerald" {
        return 0.2;
    }
    if piece("bucket") {
        return if n == "bucket" { 0.5 } else { 1.5 };
    }
    if piece("_log") || piece("_wood") {
        return 0.15;
    }
    if piece("_planks") || piece("_slab") || piece("_stairs") || n == "stick" {
        return 0.05;
    }
    if [
        "stone",
        "cobble",
        "brick",
        "deepslate",
        "granite",
        "diorite",
        "andesite",
        "sandstone",
        "obsidian",
        "dirt",
        "sand",
        "gravel",
        "clay",
        "block",
    ]
    .iter()
    .any(|m| n.contains(m))
    {
        return 0.1;
    }
    0.02
}

/// Stamps the food one carries with the day it was made (cooked food fresh off the fire as hot,
/// with the second) and moves its freshness on as it ages: stale, then spoiled (`food::freshness`);
/// hot food cools after a while and then stacks with the day's.
fn age_food(w: &World, player: u64) {
    let inv = w.inventory(player);
    let now = w.day_time();
    let today = (now / calendar::DAY_TICKS) as u32 + 1;
    for slot in 0..inv.size() {
        let stack = inv.get(slot);
        if stack.is_empty() || FRESHNESS.on_item(stack.item).is_none() {
            continue;
        }
        let Some(f) = food::find(item_name(stack.item)).map(|f| &food::FOODS[f]) else { continue };
        let mut new = stack;
        if MADE.stack_only(&stack).unwrap_or(0) == 0 {
            new = new.with(&MADE, today);
            if f.meal == food::Meal::Cooked {
                new = new.with(&HOT, (now / 20) as u32);
            }
        }
        let (age, hot) = base_api::food_age(&new, now);
        if HOT.stack_only(&new).is_some() && !hot {
            new = new.without(&HOT);
        }
        let state = if hot {
            freshness::HOT
        } else {
            match food::freshness(f, age, base_api::difficulty().keep) {
                food::Freshness::Fresh => freshness::FRESH,
                food::Freshness::Stale => freshness::STALE,
                food::Freshness::Spoiled => freshness::SPOILED,
            }
        };
        if FRESHNESS.stack_only(&new) != Some(state) {
            new = new.with(&FRESHNESS, state);
        }
        if new != stack {
            inv.set(slot, new);
        }
    }
}

/// Heat a block gives, as a share of a fire's (`temperature::fire_heat`).

/// The kind of comfort a block gives, a bit each (Valheim's comfort: each kind counts once): a
/// fire, a bed, a table, storage, something soft, flowers, a lamp.
fn comfort_of(state: u32) -> u32 {
    match block_name(state).split('[').next().unwrap_or("") {
        "base:lava" | "base:torch" | "base:wall_torch" => 1,
        "base:furnace" if state_prop(state, "lit") == Some("true") => 1,
        "base:bed" => 2,
        "base:crafting_table" => 4,
        "base:chest" => 8,
        "base:wool" => 16,
        "base:dandelion" | "base:rose" => 32,
        "base:glowstone" | "base:redstone_lamp" | "base:copper_lamp" => 64,
        _ => 0,
    }
}

/// What the place does to the body this second: the air where one stands (the biome's climate,
/// the hour, height, sun, rain or snow, water, caves, heat close by), wet from rain and water and
/// drying (faster by a fire), insulation from armor, and the core temperature moving toward what
/// that leaves it at. Sets the weather's and camp's effects.
fn feel(w: &World, id: u64, st: &EntityState, v: &mut Vitals, s: &stomach::Stomach, fortitude: f32, fat: f32) {
    let mut comfort = 0u32;
    let here = base_api::air_at(w, st.pos, st.size[2], st.in_liquid != 0, HEAT_REACH, |state| comfort |= comfort_of(state));
    (v.date, v.calendar) = (here.day, here.calendar);
    let (air, fire, sky, underground) = (here.celsius, here.fire, here.sky, here.underground);
    if st.in_liquid != 0 || (here.open_sky && here.raining) {
        v.wet = WET_SECS;
    } else {
        v.wet = (v.wet - 1.0 - fire / 3.0).max(0.0);
    }
    // Armor's insulation, a share a piece (head to feet).
    let (mut cold, mut heat) = (0.0, 0.0);
    if let Some(worn) = w.inventory(id).find_group(EQUIPMENT_GROUP) {
        for i in 0..worn.len().min(4) {
            let stack = worn.get(i);
            if stack.is_empty() {
                continue;
            }
            let name = item_name(stack.item);
            let material = name
                .split_once(':')
                .map_or(name, |(_, n)| n)
                .split('_')
                .next()
                .unwrap_or("");
            let (c, h) = temperature::set_insulation(material);
            cold += c * temperature::PIECE_SHARE[i as usize];
            heat += h * temperature::PIECE_SHARE[i as usize];
        }
    }
    let body = Body {
        comfort: temperature::PERSON,
        cold_insulation: cold,
        heat_insulation: heat,
        fortitude,
        fat,
        meal: s.meal_warmth(),
        wet: v.wet > 0.0,
    };
    if v.core < 20.0 {
        v.core = temperature::CORE; // saved before the body had a temperature
    }
    v.core = temperature::step_core(v.core, body.target(air), 1.0, 70.0);
    v.outside = air;

    let mut on = Vec::new();
    on.extend(effects::from_feeling(temperature::feeling(v.core)));
    if v.wet > 0.0 {
        on.push(Effect::Wet);
    }
    if fire >= 3.0 {
        on.push(Effect::NearFire);
    }
    let sheltered = sky < 13 && !underground;
    if sheltered {
        on.push(Effect::Sheltered);
    }
    // Resting (Valheim's): under a roof by a fire for a while, or asleep in a bed, makes one
    // rested for eight minutes and one more for each comfort level, kept full while one rests on.
    v.comfort = 1 + comfort.count_ones() as u8;
    if (sheltered && fire >= 3.0) || st.pose == abi::pose::SLEEP {
        v.resting = (v.resting + 1.0).min(REST_SECS);
    } else {
        v.resting = 0.0;
    }
    if v.resting >= REST_SECS {
        v.rested = effects::rested_secs(v.comfort as u32);
    } else {
        v.rested = (v.rested - 1.0).max(0.0);
    }
    if v.rested > 0.0 {
        on.push(Effect::Rested);
    }
    let here = effects::ALL
        .into_iter()
        .filter(|e| matches!(e.group(), Group::Weather | Group::Camp) && *e != Effect::WellFed)
        .fold(0u32, |m, e| m | 1 << e.index());
    v.effects = (v.effects & !here) | on.iter().fold(0, |m, e| m | 1 << e.index());
}

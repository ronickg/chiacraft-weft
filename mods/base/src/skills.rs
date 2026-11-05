//! Skills that rise by doing and the perk web (`notes/PROGRESSION.md`, "Skills and perks (like
//! Skyrim)"; the numbers and the web are `vitals::skills` and `vitals::perks`, the shared API
//! `base_api::skills`). Here: the player's component, what doing raises (mining, chopping, hitting,
//! taking hits, crafting, smelting, cooking, building, sowing, harvesting, riding, exploring; the
//! animals mod adds taming), skill-ups feeding the level, taking and refunding nodes, the
//! unlearning stone, and what the web does in the game, through value chains and attributes. The
//! screen is `web.rs`.

use base_api::skills::{
    COOK_EXTRA, Family, HARVEST_EXTRA, HUNGER_PACE, MAX_HEALTH, MAX_STAMINA, SKILL_WEB, Skill, SkillWeb, TAMING_SPEED, give_skill, times,
};
use base_api::{VITALS, give_xp};
use vitals::curve::{Age, age_of, item_level};
use vitals::perks::{self, Refusal};
use vitals::skills::{self, gain};
use weft_sdk::*;

/// A node taken from the web screen, by id; one refunded.
pub static TAKE: Channel<u32> = Channel::named("web_take");
pub static REFUND: Channel<u32> = Channel::named("web_refund");
/// The rare item that refunds a few nodes (`perks::REFUNDS_PER_ITEM`).
pub static UNLEARNING_STONE: Item = Item::named("unlearning_stone");
/// The attribute modifiers the web sets.
const MODIFIER: &str = "base:web";
/// How near a player stands to a fire or furnace to be its cook or smith (blocks).
const TENDING: f64 = 8.0;
/// Blocks past which a hit counts as a shot (Archery), not a blow (Melee).
const SHOT_PAST: f64 = 4.5;

pub fn register(r: &mut Registry) {
    // Kept through death: skill is in the hands.
    r.entity_component(
        &SKILL_WEB,
        SkillWeb::default(),
        component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN,
    );
    r.attach(&PLAYER, &SKILL_WEB);
    gains(r);
    effects(r);
    r.packet(&TAKE, |p, id| take(&p.world(), p.player(), id as u16));
    r.packet(&REFUND, |p, id| refund(&p.world(), p.player(), id as u16));
    r.item(&UNLEARNING_STONE)
        .max_stack(16)
        .on_use(|u| {
            let w = u.world();
            let Some(s) = w.component_mut(u.player(), &SKILL_WEB) else { return false };
            s.refunds = s.refunds.saturating_add(perks::REFUNDS_PER_ITEM);
            let left = s.refunds;
            if !w.outside_survival(u.player()) {
                u.set_stack(StackData::shrink(u.stack(), 1));
            }
            w.hint(u.player(), "web.base.unlearn", &[Arg::Int(left as i64)]);
            true
        })
        .tab(&crate::tabs::INGREDIENTS);
    // A peridot ground with two tektites: both from fallen stars, so it's rare (`notes/PROGRESSION.md`).
    r.shapeless(&crate::recipes::CRAFTING, "unlearning_stone", 1, &["peridot", "tektite", "tektite"], 0);
    r.world_tick(tick);
    commands(r);
}

// ------------------------------------------------------------------ taking and refunding

/// Perk points `player` has to spend: one a level past the first, less the nodes taken.
pub fn points(w: &World, player: u64) -> u32 {
    let level = w
        .component(player, &VITALS)
        .map_or(1, |v| vitals::levels::level_of(v.xp).0);
    let taken = w
        .component(player, &SKILL_WEB)
        .map_or(0, |s| perks::count(&s.taken));
    points_of(level, taken)
}

pub fn points_of(level: u32, taken: u32) -> u32 {
    (level.saturating_sub(1) * vitals::web::POINTS_PER_LEVEL).saturating_sub(taken)
}

fn take(w: &World, player: u64, id: u16) {
    let points = points(w, player);
    let Some(s) = w.component_mut(player, &SKILL_WEB) else { return };
    match perks::can_take(&s.taken, id, &s.levels(), points) {
        Ok(()) => {
            perks::set(&mut s.taken, id, true);
            weft_sdk::info!("player {player} took web node {id}");
            refresh(w, player);
        }
        Err(why) => {
            let key = match why {
                Refusal::NoPoints => "web.base.no_points",
                Refusal::Skill => "web.base.too_low",
                Refusal::NotNextTo => "web.base.not_next",
                Refusal::Taken | Refusal::Unknown => return,
            };
            w.hint(player, key, &[]);
        }
    }
}

fn refund(w: &World, player: u64, id: u16) {
    let Some(s) = w.component_mut(player, &SKILL_WEB) else { return };
    if s.refunds == 0 {
        w.hint(player, "web.base.no_refunds", &[]);
        return;
    }
    if !perks::can_refund(&s.taken, id) {
        w.hint(player, "web.base.holds_others", &[]);
        return;
    }
    perks::set(&mut s.taken, id, false);
    s.refunds -= 1;
    refresh(w, player);
}

// ------------------------------------------------------------------ the second's work

/// Each second: skill-ups (their XP and toast), riding, and the web's attributes.
fn tick(w: &World) {
    if !w.game_time().is_multiple_of(20) {
        return;
    }
    for id in w.players() {
        // Riding: a second in the saddle, moving, raises Taming (`gain::RIDE_SEC`).
        if let Some((mount, _)) = w.vehicle(id)
            && let Some(m) = w.entity(mount)
            && m.vel.x.hypot(m.vel.z) > 0.05
        {
            give_skill(w, id, Skill::Taming, gain::RIDE_SEC, skills::for_age(Age::Stone));
        }
        let ups: Vec<(usize, u32)> = {
            let Some(s) = w.component_mut(id, &SKILL_WEB) else { continue };
            let mut ups = Vec::new();
            for (i, sk) in skills::SKILLS.iter().enumerate() {
                let level = s.level(*sk);
                if level > s.shown[i] as u32 {
                    ups.push((i, level));
                    s.shown[i] = level.min(255) as u8;
                }
            }
            ups
        };
        for (i, level) in ups {
            // Every level passed gives its XP (a big gain can pass several).
            let from = level.saturating_sub(10);
            let xp: u64 = (from + 1..=level).map(skills::up_xp).sum();
            give_xp(w, id, xp);
            let key = format!("web.base.skill.{}", skills::SKILLS[i].id());
            w.hint(id, "web.base.skill_up", &[Arg::Key(&key), Arg::Int(level as i64)]);
        }
        refresh(w, id);
    }
}

/// Sets the attributes the web drives: mining speed and walking speed.
fn refresh(w: &World, player: u64) {
    for (attr, f) in [(&attribute::MINING_SPEED, Family::MineSpeed), (&attribute::MOVEMENT_SPEED, Family::MoveSpeed)] {
        let x = times(w, player, f) as f64 - 1.0;
        if x.abs() < 1e-6 {
            w.attribute_remove_modifier(player, attr, MODIFIER);
        } else {
            w.attribute_add_modifier(player, attr, MODIFIER, attr_op::MULTIPLY_TOTAL, x, 0);
        }
    }
}

// ------------------------------------------------------------------ what doing raises

/// The level of an item: its parts' best material (`vitals::gear`), else its tag's tier at the
/// age's middle (`progression`), else the Stone Age's middle.
pub fn level_of_stack(st: &ItemStack) -> f32 {
    if st.is_empty() {
        return Age::Stone.reference();
    }
    if let Some(p) = crate::parts::PARTS.on_stack_own(st) {
        let best = p
            .materials
            .iter()
            .filter_map(|&c| crafting::material_of(c))
            .filter_map(|m| vitals::gear::material(m.id))
            .map(|m| m.level())
            .fold(f32::MIN, f32::max);
        if best > f32::MIN {
            return best;
        }
    }
    level_of_name(item_name(st.item))
}

/// The level of a thing by its id: its tag's tier at the age's middle, cached.
pub fn level_of_name(name: &str) -> f32 {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: Mutex<Option<HashMap<String, f32>>> = Mutex::new(None);
    let name = name.split('[').next().unwrap_or(name);
    let mut c = CACHE.lock().unwrap();
    let map = c.get_or_insert_with(HashMap::new);
    if let Some(&l) = map.get(name) {
        return l;
    }
    let l = progression::content::tier(name).map_or(Age::Stone.reference(), |t| item_level(t, 5));
    map.insert(name.to_string(), l);
    l
}

/// The age a player's held thing belongs to.
fn held_age(w: &World, player: u64) -> Age {
    let held = w
        .inventory(player)
        .group("hotbar")
        .get(w.selected_slot(player));
    age_of(level_of_stack(&held))
}

/// Whether an item is a tool, weapon or armour (Smithing makes those; Survival the rest).
fn smithed(name: &str) -> bool {
    let path = name.split(':').nth(1).unwrap_or(name);
    path.starts_with("part_")
        || [
            "pickaxe",
            "axe",
            "shovel",
            "hoe",
            "sword",
            "spear",
            "knife",
            "helmet",
            "chestplate",
            "leggings",
            "boots",
            "shears",
            "hammer",
            "ingot",
            "nugget",
        ]
        .iter()
        .any(|t| path.contains(t))
}

/// Something was made by `player` from `inputs` things: Smithing for tools, weapons, armour and
/// metal, Survival for the rest, by the made thing's level.
pub fn made(w: &World, player: u64, out: &ItemStack, inputs: u32) {
    if out.is_empty() {
        return;
    }
    let name = item_name(out.item);
    let skill = if smithed(name) { Skill::Smithing } else { Skill::Survival };
    give_skill(w, player, skill, gain::craft(inputs), skills::for_level(level_of_stack(out)));
}

/// The nearest player within `TENDING` of a block entity (its cook or smith).
fn tender(w: &World, id: u64) -> Option<u64> {
    let at = w.entity(id)?.pos;
    w.players()
        .into_iter()
        .filter_map(|p| Some((p, w.entity(p)?.pos)))
        .map(|(p, pos)| (p, (pos.x - at.x).powi(2) + (pos.y - at.y).powi(2) + (pos.z - at.z).powi(2)))
        .filter(|&(_, d2)| d2 <= TENDING * TENDING)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(p, _)| p)
}

/// A fire at `fire` made `out` from `inputs` things: Alchemy for whoever tends it, and their web's
/// chance of a second portion (`COOK_EXTRA`), which comes back as the count to make.
pub fn cooked(w: &World, fire: u64, out: &ItemStack, inputs: u32) -> u32 {
    let Some(cook) = tender(w, fire) else { return out.count };
    give_skill(w, cook, Skill::Alchemy, gain::cook(inputs), skills::for_level(level_of_stack(out)));
    let q = abi::Query { entity: cook, item: out.item, ..query() };
    let extra = w.chain_compute(&COOK_EXTRA, 0.0, &q).max(0.0);
    out.count + whole_and_chance(w, extra)
}

/// `x` as whole extras plus one more at its fraction's chance.
fn whole_and_chance(w: &World, x: f64) -> u32 {
    let whole = x.floor();
    let u = (w.random() >> 40) as f64 / (1u64 << 24) as f64;
    whole as u32 + (u < x - whole) as u32
}

fn gains(r: &mut Registry) {
    // Mining, chopping and harvesting: by the block's hardness and level.
    r.on(&actions::BREAK_BLOCK, |b| {
        let w = b.world();
        for e in b.iter() {
            let name = block_name(e.state);
            let base = name.split('[').next().unwrap_or(name);
            let work = skills::for_level(level_of_name(base));
            if let Some(c) = vitals::crops::of(base) {
                if state_prop(e.state, "age").and_then(|a| a.parse::<u32>().ok()) == Some(c.ripe) {
                    give_skill(&w, e.player, Skill::Building, gain::HARVEST, work);
                    harvest_extra(&w, e.player, e.state, e.pos.to_array());
                }
                continue;
            }
            let hardness = HARDNESS.on_block(e.state).unwrap_or(1.0).max(0.0);
            if hardness <= 0.0 {
                continue;
            }
            let path = base.split(':').nth(1).unwrap_or(base);
            if path.contains("log") || path.ends_with("_wood") || path.contains("stem") {
                give_skill(&w, e.player, Skill::Survival, gain::mine(hardness), work);
            } else if path.contains("ore") {
                give_skill(&w, e.player, Skill::Mining, gain::ore(hardness), work);
            } else {
                give_skill(&w, e.player, Skill::Mining, gain::mine(hardness), work);
            }
        }
    });
    // Building and sowing.
    r.on(&actions::PLACE_BLOCK, |b| {
        let w = b.world();
        for e in b.iter() {
            let name = block_name(e.state);
            let base = name.split('[').next().unwrap_or(name);
            let em = if vitals::crops::of(base).is_some() { gain::SOW } else { gain::PLACE };
            give_skill(&w, e.player, Skill::Building, em, skills::for_level(level_of_name(base)));
        }
    });
    // Hitting (Melee near, Archery far) and taking hits (Defence), by the damage and the age of
    // what's held or worn.
    r.on(&actions::HURT, |b| {
        let w = b.world();
        let players = w.players();
        for h in b.iter() {
            if h.amount <= 0.0 || h.source == 0 || h.source == h.entity {
                continue;
            }
            let damage = h.amount;
            if players.contains(&h.source) && !players.contains(&h.entity) {
                let age = held_age(&w, h.source);
                let far = match (w.entity(h.source), w.entity(h.entity)) {
                    (Some(a), Some(t)) => (a.pos.x - t.pos.x).hypot(a.pos.z - t.pos.z) > SHOT_PAST,
                    _ => false,
                };
                let skill = if far { Skill::Archery } else { Skill::Melee };
                give_skill(&w, h.source, skill, gain::hit(damage, age), skills::for_age(age));
            } else if players.contains(&h.entity) && !players.contains(&h.source) && crate::player::armor_applies(h.cause) {
                // The armour's age: the level whose reference armour it is.
                let armour = crate::player::armor_of(&w, h.entity).0;
                let level = vitals::curve::levels_for((armour / vitals::combat::REF_ARMOUR).max(1.0));
                let age = age_of(level);
                give_skill(&w, h.entity, Skill::Defence, gain::taken(damage, age), skills::for_level(level));
            }
        }
    });
    // Smelting: the smith at the furnace.
    r.on(&base_api::SMELTED, |b| {
        let w = b.world();
        for e in b.iter() {
            if let Some(p) = tender(&w, e.furnace) {
                give_skill(&w, p, Skill::Smithing, gain::SMELT, skills::for_level(level_of_stack(&e.output)));
            }
        }
    });
}

/// A ripe crop broken by `player`: their web's chance of one more of it (`HARVEST_EXTRA`).
fn harvest_extra(w: &World, player: u64, state: u32, pos: [i32; 3]) {
    let q = abi::Query { entity: player, state, ..query() };
    let extra = w.chain_compute(&HARVEST_EXTRA, 0.0, &q).max(0.0);
    let n = whole_and_chance(w, extra);
    if n == 0 {
        return;
    }
    let Some(gives) = vitals::crops::of(block_name(state).split('[').next().unwrap_or("")).map(|c| c.gives) else { return };
    let Some(item) = Item::lookup(gives) else { return };
    let at = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.3, pos[2] as f64 + 0.5];
    w.spawn_item(at, ItemStack::new(item, n));
}

/// Exploring: a biome or dimension found (`levels.rs`).
pub fn discovered(w: &World, player: u64, dimension: bool) {
    let em = if dimension { gain::DISCOVER_DIMENSION } else { gain::DISCOVER };
    give_skill(w, player, Skill::Exploring, em, skills::MAX as f32);
}

// ------------------------------------------------------------------ what the web does

fn effects(r: &mut Registry) {
    // Melee damage: a player's blows (and a keystone's cost).
    r.modify(&chain::ATTACK_DAMAGE, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        if w.players().contains(&q.entity) {
            m.value *= times(&w, q.entity, Family::MeleeDamage) as f64;
        }
    });
    // Defence: what creatures' hits do to a player, divided.
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        if crate::player::armor_applies(q.cause) && w.players().contains(&q.entity) {
            m.value /= times(&w, q.entity, Family::Defence) as f64;
        }
    });
    r.modify(&TAMING_SPEED, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        if q.other != 0 {
            m.value *= times(&w, q.other, Family::TamingSpeed) as f64;
        }
    });
    r.modify(&HUNGER_PACE, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        m.value /= times(&w, q.entity, Family::Hunger) as f64;
    });
    r.modify(&MAX_HEALTH, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        m.value *= times(&w, q.entity, Family::Health) as f64;
    });
    r.modify(&MAX_STAMINA, stage::MULTIPLY, |m| {
        let (w, q) = (m.world(), *m.query());
        m.value *= times(&w, q.entity, Family::Stamina) as f64;
    });
    // Harvest and cooking: the bonus over 1 is the chance of one more.
    r.modify(&HARVEST_EXTRA, stage::ADD, |m| {
        let (w, q) = (m.world(), *m.query());
        m.value += times(&w, q.entity, Family::Harvest) as f64 - 1.0;
    });
    r.modify(&COOK_EXTRA, stage::ADD, |m| {
        let (w, q) = (m.world(), *m.query());
        m.value += times(&w, q.entity, Family::Cooking) as f64 - 1.0;
    });
}

// ------------------------------------------------------------------ commands

fn commands(r: &mut Registry) {
    // Sets a skill's level (for testing), or tells them all.
    r.command("skill", "[skill:word] [level:int]", |cmd| {
        let w = cmd.world();
        let player = cmd.player();
        if let (Some(id), Some(level)) = (cmd.text("skill").map(str::to_string), cmd.int("level")) {
            let Some(sk) = Skill::of_id(&id) else {
                cmd.reply("command.base.skill.unknown", &[Arg::Text(&id)]);
                return;
            };
            if let Some(s) = w.component_mut(player, &SKILL_WEB) {
                s.em[sk.index()] = skills::em_to(level.clamp(0, skills::MAX as i64) as f32);
            }
        }
        let Some(s) = w.component(player, &SKILL_WEB) else { return };
        let line = skills::SKILLS
            .iter()
            .map(|sk| format!("{} {}", sk.id(), s.level(*sk)))
            .collect::<Vec<_>>()
            .join(", ");
        cmd.reply("command.base.skill", &[Arg::Text(&line), Arg::Int(points(&w, player) as i64), Arg::Int(s.refunds as i64)]);
    })
    .permission(permission::GAME_MASTER);
    // Takes a node by id (for testing: the screen's click), or refunds it.
    r.command("perk", "<action:word(take,refund)> <node:int>", |cmd| {
        let w = cmd.world();
        let id = cmd.int("node").unwrap_or(0).clamp(0, u16::MAX as i64) as u16;
        match cmd.text("action") {
            Some("refund") => refund(&w, cmd.player(), id),
            _ => take(&w, cmd.player(), id),
        }
    })
    .permission(permission::GAME_MASTER);
}

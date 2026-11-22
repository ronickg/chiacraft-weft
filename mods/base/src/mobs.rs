//! Mobs: zombies and spiders at night or in the dark, pigs and cows on grass by day.

use weft_sdk::ai;
use weft_sdk::*;

use base_api::entities::*;
use base_api::items::{ARROW as ARROW_ITEM, BOW, EGG};
/// Ticks of whatever a mob counts: a skeleton's reload, a chicken's next egg.
static FUSE: Component<u32> = Component::named("fuse");
/// Ours: vanilla's creeper (upstream removed it).
pub static CREEPER: EntityKind = EntityKind::named("creeper");

use base_api::entities::COW;

/// The mobs `vitals::mobs` gives numbers, by their row's name.
static MOB_KINDS: [(&EntityKind, &str); 4] = [(&ZOMBIE, "zombie"), (&SPIDER, "spider"), (&SKELETON, "skeleton"), (&CREEPER, "creeper")];

/// "player", or a mob's row name, for the night checks' log lines.
fn fighter(w: &World, id: u64) -> Option<&'static str> {
    if w.players_iter().any(|p| p == id) {
        return Some("player");
    }
    let kind = w.entity(id)?.kind;
    MOB_KINDS
        .iter()
        .find(|(k, _)| k.id() == kind)
        .map(|(_, n)| *n)
}

/// A mob's numbers on the curve (`vitals::mobs`).
fn curve(name: &str) -> vitals::combat::Stats {
    vitals::mobs::mob(name)
        .map(|m| m.stats())
        .expect("a row in vitals::mobs")
}

pub fn register(r: &mut Registry) {
    // The engine's player kind, drawn as the same humanoid as the zombie.
    r.set_model(&PLAYER, humanoid(false));
    r.edit(&PLAYER).hitboxes(Hitboxes::humanoid());
    r.entity(&ZOMBIE)
        .hitboxes(Hitboxes::humanoid().reaching())
        .tag("hostile")
        // Undead, as vanilla's: poison and regeneration take no hold.
        .tag("immune_effect/base:poison")
        .tag("immune_effect/base:regeneration")
        .equipment()
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .size(0.6, 1.95)
        .max_health(curve("zombie").health)
        .walk_speed(2.6)
        .hostile()
        .burns_in_sun()
        .attack_damage(curve("zombie").hit)
        .reach(1.3)
        .goals(hostile(ai::Goals::new().aggressive(35.0)));
    r.entity(&SPIDER)
        .hitboxes(spider_parts())
        .tag("hostile")
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .size(1.3, 0.9)
        .max_health(curve("spider").health)
        .walk_speed(3.6)
        .hostile()
        .attack_damage(curve("spider").hit)
        // Vanilla's reach goes by a mob's width: a body this wide stops further off.
        .reach(2.4)
        // In daylight it only fights back.
        .goals(hostile(
            ai::Goals::new()
                .react(ai::Trigger::sight(35.0).only_if(|w, st| !in_daylight(w, st)), ai::react::anger(2.0, 1))
                .neutral(),
        ));
    r.entity_component(&FUSE, 0u32, 0);
    // A skeleton's arrow hits for its row's hit (`vitals::mobs`), not by its speed: set first, so
    // armour and the rest still apply.
    r.modify(&chain::DAMAGE, &phase::BASE, |m| {
        let q = *m.query();
        if q.cause != damage_cause::PROJECTILE {
            return;
        }
        if m.world()
            .entity(q.other)
            .is_some_and(|e| e.kind == SKELETON.id())
        {
            // In the head, half as much again, as every arrow (`PROJECTILE_DAMAGE` above).
            let head = if q.part == hitbox_part::HEAD { 2.0 } else { 1.0 };
            m.value = curve("skeleton").hit as f64 * head;
        }
    });
    // One line per hit between a player and a mob, and per death of either (night checks:
    // `mods/testkit/scripts/night_survival.txt`).
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            let (Some(target), Some(by)) = (fighter(&w, h.entity), fighter(&w, h.source)) else { continue };
            if (target == "player") == (by == "player") {
                continue;
            }
            let (health, max) = w
                .entity(h.entity)
                .map_or((0.0, 0.0), |e| (e.health, e.max_health));
            weft_sdk::info!(
                "mob_fight: {by} {} hit {target} {} for {:.2} (cause {}), health {health:.1}/{max:.1}",
                h.source,
                h.entity,
                h.amount,
                h.cause
            );
        }
    });
    r.on_event(&actions::DIE, |b| {
        let w = b.world();
        for d in b.iter() {
            let who = if w.players_iter().any(|p| p == d.entity) {
                Some("player")
            } else {
                MOB_KINDS
                    .iter()
                    .find(|(k, _)| k.id() == d.kind)
                    .map(|(_, n)| *n)
            };
            if let Some(who) = who {
                weft_sdk::info!("mob_fight: {who} {} died (cause {}, by {})", d.entity, d.cause, d.source);
            }
        }
    });
    // Over-tier armour against them counts at most their size's cap (`base_api::chains::GEAR_COUNTS`).
    r.modify(&base_api::chains::GEAR_COUNTS, &phase::MULTIPLY, |m| {
        let q = *m.query();
        let Some(kind) = m.world().entity(q.other).map(|e| e.kind) else { return };
        let Some(mob) = MOB_KINDS
            .iter()
            .find(|(k, _)| k.id() == kind)
            .and_then(|(_, n)| vitals::mobs::mob(n))
        else {
            return;
        };
        m.value *= vitals::combat::cap_share(mob.size(), q.amount as f32, mob.level()) as f64;
    });
    // Their armour (`vitals::mobs`), through the same formula as the player's and the animals'.
    r.modify(&chain::DAMAGE, &phase::MULTIPLY, |m| {
        let q = *m.query();
        if !crate::player::armor_applies(q.cause) {
            return;
        }
        let Some(kind) = m.world().entity(q.entity).map(|e| e.kind) else { return };
        let Some(mob) = MOB_KINDS
            .iter()
            .find(|(k, _)| k.id() == kind)
            .and_then(|(_, n)| vitals::mobs::mob(n))
        else {
            return;
        };
        m.value *= 1.0 - vitals::combat::armour_cut(mob.stats().armour, m.value as f32) as f64;
    });
    // Ours: the creeper stays (upstream removed it; our blood moon sends creepers, `events.rs`).
    // Its look: `models/creeper.geo.json` (vanilla's layout, 64 x 32) and `textures/entity/creeper.png`.
    r.entity(&CREEPER)
        .sound("hurt")
        .sound("death")
        .tag("hostile")
        .size(0.6, 1.7)
        .max_health(curve("creeper").health)
        .walk_speed(2.5)
        .hostile()
        .attach(&FUSE)
        .goals(creeper());
    r.entity(&SKELETON)
        .hitboxes(Hitboxes::humanoid().slim(2.0).reaching())
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .tag("hostile")
        .tag("immune_effect/base:poison")
        .tag("immune_effect/base:regeneration")
        .equipment()
        .size(0.6, 1.99)
        .max_health(curve("skeleton").health)
        .walk_speed(2.5)
        .hostile()
        .burns_in_sun()
        .attach(&FUSE)
        .goals(skeleton());
    r.on_event(&actions::SPAWN, |batch| {
        for spawned in batch.iter() {
            if spawned.kind == SKELETON.id() {
                batch
                    .world()
                    .inventory(spawned.entity)
                    .set(0, ItemStack::new(BOW.id(), 1));
            }
        }
    });
    // Drawn as long as the use button is held, shot when it is let go (upstream's). The bow itself
    // is ours, a tool of parts (`parts.rs`), so this only adds the drawing and shooting.
    r.edit(&BOW).use_until_released().on_release(shoot_bow);
    // An arrow in the head hurts half as much again, a bow's critical hit: `part` is the part
    // of the hitbox it went into.
    r.modify(&chain::PROJECTILE_DAMAGE, &phase::MULTIPLY, |m| {
        let q = *m.query();
        let arrow = m
            .world()
            .entity(q.entity)
            .is_some_and(|e| e.kind == ARROW.id());
        if arrow && q.part == hitbox_part::HEAD {
            m.value *= 2.0;
        }
    });
    // Drawing it narrows the view, more the further it is drawn (vanilla's 15% at full draw).
    r.modify(&attribute::FOV, &phase::MULTIPLY, |m| {
        let Some(s) = m.world().entity(m.query().entity) else { return };
        if s.using == BOW.id() {
            let drawn = (s.using_ticks as f64 / 20.0).min(1.0);
            m.value *= 1.0 - drawn * drawn * 0.15;
        }
    });
    r.shaped(&crate::recipes::CRAFTING, "bow", 1, &[" s#", "s #", " s#"], &[('s', "#stick"), ('#', "#string")]);
    // Vanilla's arrow: 0.05 a tick down, stuck a minute where it hits a block.
    r.entity(&ARROW)
        .no_egg()
        .size(0.5, 0.5)
        .unpushable()
        .max_health(1.0)
        .walk_speed(0.0)
        .attribute(&attribute::GRAVITY, 0.625)
        .projectile(Projectile::new().sticks(1200));
    r.entity(&SHEEP)
        .hitboxes(sheep_parts())
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .tag("livestock")
        .size(0.9, 1.3)
        .max_health(8.0)
        .walk_speed(2.3)
        .passive()
        .goals(passive());
    r.entity(&CHICKEN)
        .hitboxes(chicken_parts())
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .tag("livestock")
        .size(0.4, 0.7)
        .max_health(4.0)
        .walk_speed(2.5)
        .passive()
        .attach(&FUSE)
        .goals(passive_chicken());
    r.entity(&PIG)
        .hitboxes(pig_parts())
        .tag("livestock")
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .size(0.9, 0.9)
        .max_health(10.0)
        .walk_speed(2.5)
        .passive()
        .goals(passive());
    r.entity(&COW)
        .hitboxes(Hitboxes::quadruped())
        .tag("livestock")
        .sound("ambient")
        .sound("hurt")
        .sound("death")
        .size(0.9, 1.3)
        .max_health(10.0)
        .walk_speed(2.2)
        .passive()
        .goals(passive());
    // Vanilla amounts; meat comes out cooked when the animal burns.
    r.loot(&ZOMBIE)
        .pool(Pool::new().items("rotten_flesh", 0, 2));
    r.loot(&SPIDER)
        .pool(Pool::new().items("string", 0, 2))
        .pool(
            Pool::new()
                .when(Cond::killed_by_player())
                .when(Cond::chance(0.33))
                .item("spider_eye"),
        );
    r.loot(&CREEPER).pool(Pool::new().items("gunpowder", 0, 2));
    r.loot(&SKELETON)
        .pool(Pool::new().items("arrow", 0, 2))
        .pool(Pool::new().items("bone", 0, 2));
    r.loot(&SHEEP).pool(Pool::new().item("wool")).pool(
        Pool::new().entry(
            Entry::item("mutton")
                .count(1, 2)
                .smelted(&crate::recipes::SMELTING),
        ),
    );
    r.loot(&CHICKEN)
        .pool(Pool::new().items("feather", 0, 2))
        .pool(
            Pool::new().entry(
                Entry::item("chicken")
                    .count(1, 1)
                    .smelted(&crate::recipes::SMELTING),
            ),
        );
    r.loot(&PIG).pool(
        Pool::new().entry(
            Entry::item("porkchop")
                .count(1, 3)
                .smelted(&crate::recipes::SMELTING),
        ),
    );
    r.loot(&COW)
        .pool(
            Pool::new().entry(
                Entry::item("beef")
                    .count(1, 3)
                    .smelted(&crate::recipes::SMELTING),
            ),
        )
        // A raw hide, to tan (`crafts.rs`).
        .pool(Pool::new().items("hide", 0, 2));
    // Natural spawning, as rules the engine runs: monsters in the dark (spiders more in deep
    // caves), animals in small herds on grass by day, rarely once the world is loaded.
    r.natural_spawn(&ZOMBIE).light(0, 6).weight(2);
    r.natural_spawn(&SKELETON).light(0, 6).weight(2);
    r.natural_spawn(&CREEPER).light(0, 6).weight(2);
    r.natural_spawn(&SPIDER).light(0, 6);
    r.natural_spawn(&SPIDER)
        .light(0, 6)
        .biome("#cave")
        .weight(4);
    for animal in [&PIG, &COW, &SHEEP, &CHICKEN] {
        r.natural_spawn(animal)
            .biome("#animals")
            .ground("#animal_spawn")
            .light(7, 15)
            .pack_size(2, 3)
            .chance(0.05);
    }
}

/// Vanilla's creeper: comes for the player, and within 3 blocks starts its 30 tick fuse, which
/// runs back down once the player is 7 away. A radius 3 blast at the end.
fn creeper() -> ai::Goals {
    ai::Goals::new()
        .aggressive(16.0)
        .then(keep_from_fire())
        .then(|w: &World, id: u64, st: &EntityState| {
            let near = ai::target(w, id, st, 7.0);
            let d2 = near
                .as_ref()
                .map_or(f64::MAX, |(_, p)| (p.pos - st.pos).length_squared());
            let Some(fuse) = w.component_mut(id, &FUSE) else { return false };
            if d2 < 9.0 || (*fuse > 0 && d2 < 49.0) {
                *fuse += 1;
            } else {
                *fuse = fuse.saturating_sub(1);
            }
            let lit = *fuse;
            // Faster as it nears the end.
            if lit > 0 && lit % (if lit > 20 { 3 } else { 8 }) == 0 {
                w.flash(id);
            }
            if lit >= 30 {
                blast(w, id, st.pos.to_array().into(), 3.0);
                return true;
            }
            lit > 0
        })
        .then(ai::watch_player(8.0))
        .then(ai::chase_target(16.0, 1.0))
        .then(ai::wander(0.6, 10.0))
}

/// Vanilla's explosion in short: a hole, and damage falling off to nothing at twice the radius.
fn blast(w: &World, id: u64, at: Vec3, radius: f64) {
    for other in w.entities_near(at, radius * 2.0, None) {
        let Some(s) = w.entity(other).filter(|_| other != id) else { continue };
        let d = (s.pos - at).length_squared().sqrt();
        let impact = (1.0 - d / (radius * 2.0)).max(0.0);
        let damage = (impact * impact + impact) / 2.0 * 7.0 * radius * 2.0 + 1.0;
        w.damage(other, Damage::new(damage as f32, damage_cause::EXPLOSION).by(id));
    }
    // Vanilla's: one going off in a liquid breaks nothing (a waterlogged block's cell too), and
    // liquids don't break at all.
    if w.liquid_in_cell(block_pos_of(at)).is_none() {
        w.fill(Shape::blast(at, radius, w.random()), 0)
            .keep("#unbreakable")
            .keep("#liquids");
    }
    w.spawn_particles(&Particle::named("core:poof"), at)
        .count(40)
        .spread(radius as f32 * 0.5)
        .speed(2.0);
    w.despawn(id);
}

/// Vanilla's skeleton: keeps to about 10 blocks and shoots once in two seconds.
fn skeleton() -> ai::Goals {
    ai::Goals::new()
        .aggressive(16.0)
        .then(ai::watch_player(16.0))
        .then(keep_from_fire())
        .then(|w: &World, id: u64, st: &EntityState| {
            let Some((_, p)) = ai::target(w, id, st, 15.0) else { return false };
            let Some(reload) = w.component_mut(id, &FUSE) else { return false };
            *reload = (*reload + 1) % 40;
            if *reload == 0 {
                shoot(w, id, st, &p);
            }
            let d2 = (p.pos.x - st.pos.x).powi(2) + (p.pos.z - st.pos.z).powi(2);
            if d2 > 100.0 {
                w.navigate(id, p.pos, 1.0);
            }
            true
        })
        .then(ai::chase_target(16.0, 1.0))
        .then(ai::wander(0.6, 10.0))
}

/// Vanilla's bow: full strength after a second's draw, the curve easing in; an arrow from the
/// inventory, none taken in creative.
fn shoot_bow(u: &mut ItemUse) -> bool {
    let (w, id) = (u.world(), u.entity());
    let t = u.held_ticks() as f32 / 20.0;
    let power = ((t * t + t * 2.0) / 3.0).min(1.0) as f64;
    if power < 0.1 {
        return false;
    }
    let creative = w.creative(id);
    let inv = w.inventory(id);
    let arrow = (0..inv.len()).find(|&s| inv.get(s).item == ARROW_ITEM.id());
    if arrow.is_none() && !creative {
        return false;
    }
    let Some(s) = w.entity(id) else { return false };
    // Three blocks a tick at full draw.
    let v = look_dir(s.yaw, s.pitch) * (power * 3.0 * 20.0);
    w.spawn_with(
        &ARROW,
        s.eye(),
        &Spawn::new()
            .owner(id)
            .velocity(v)
            .inherit_velocity()
            .yaw(s.yaw),
    );
    if !creative {
        if let Some(slot) = arrow {
            inv.set(slot, consume_one(inv.get(slot)));
        }
        u.set_stack(u.stack().add_wear(1));
    }
    true
}

fn shoot(w: &World, id: u64, from: &EntityState, at: &EntityState) {
    let eye = from.eye().y;
    let d = [at.pos.x - from.pos.x, at.pos.y + at.size.eye as f64 * 0.7 - eye, at.pos.z - from.pos.z];
    let flat = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let len = (flat * flat + d[1] * d[1]).sqrt().max(0.1);
    // A little above the target for the drop on the way.
    const SPEED: f64 = 32.0;
    let v = [d[0] / len * SPEED, d[1] / len * SPEED + flat * 0.4, d[2] / len * SPEED];
    w.spawn_with(
        &ARROW,
        [from.pos.x, eye, from.pos.z],
        &Spawn::new()
            .owner(id)
            .velocity(v)
            .inherit_velocity()
            .yaw(yaw_toward([0.0; 3], d)),
    );
}

/// A chicken: flutters down instead of falling, and lays an egg now and then.
fn passive_chicken() -> ai::Goals {
    let goals = ai::Goals::new().then(|w: &World, id: u64, st: &EntityState| {
        if st.on_ground == 0 && st.vel.y < 0.0 {
            w.entity_modify(id)
                .velocity(Vec3 { y: st.vel.y * 0.6, ..st.vel });
        }
        if let Some(next) = w.component_mut(id, &FUSE) {
            if *next == 0 {
                *next = 6000 + w.random_below(6000) as u32;
            } else {
                // Far from players it ticks only now and then.
                *next = next.saturating_sub(w.tick_elapsed(id));
                if *next == 0 {
                    w.spawn_item(st.pos, ItemStack::new(EGG.id(), 1));
                }
            }
        }
        false
    });
    with_passive(goals)
}

/// Blocks from a campfire monsters keep out of, across and up or down (Ronald, 11-04: "Campfire
/// keeps you safe"; its light 15 is 7 or more out to here, and monsters spawn only at 6 or less).
const FIRE_RING: i32 = 8;
const FIRE_RING_UP: i32 = 4;
/// Ticks between a monster's looks for a campfire.
const FIRE_LOOK: u64 = 20;

static CAMPFIRES: Tag = Tag::named("campfires");
/// The campfire near each monster as it last looked (when, and where if any).
static FIRES_SEEN: std::sync::Mutex<Option<std::collections::HashMap<u64, (u64, Option<BlockPos>)>>> = std::sync::Mutex::new(None);

/// The campfire within the ring of `pos`, if any.
pub(crate) fn campfire_near(w: &World, pos: Vec3) -> Option<BlockPos> {
    let p = block_pos_of(pos);
    for dy in -FIRE_RING_UP..=FIRE_RING_UP {
        for dz in -FIRE_RING..=FIRE_RING {
            for dx in -FIRE_RING..=FIRE_RING {
                let at = BlockPos::new(p.x + dx, p.y + dy, p.z + dz);
                if CAMPFIRES.on_block(w.block(at)) {
                    return Some(at);
                }
            }
        }
    }
    None
}

/// `campfire_near` for entity `id` (a monster, or the player it's after), looked up again once a
/// second.
fn fire_for(w: &World, id: u64, pos: Vec3) -> Option<BlockPos> {
    let now = w.game_time();
    let mut seen = FIRES_SEEN.lock().unwrap();
    let map = seen.get_or_insert_with(Default::default);
    if let Some(&(at, fire)) = map.get(&id)
        && now < at + FIRE_LOOK
    {
        return fire;
    }
    if map.len() > 4096 {
        map.retain(|_, (at, _)| now < *at + FIRE_LOOK * 10);
    }
    let fire = campfire_near(w, pos);
    map.insert(id, (now, fire));
    fire
}

/// Monsters keep out of a campfire's light: one inside the ring walks out of it, and a player
/// inside it isn't chased, shot or blown up; the monster waits at the edge, watching.
fn keep_from_fire() -> impl ai::Goal {
    move |w: &World, id: u64, st: &EntityState| {
        if let Some(f) = fire_for(w, id, st.pos) {
            let (dx, dz) = (st.pos.x - (f.x as f64 + 0.5), st.pos.z - (f.z as f64 + 0.5));
            let len = (dx * dx + dz * dz).sqrt().max(0.1);
            let out = (FIRE_RING + 3) as f64;
            w.navigate(id, [f.x as f64 + 0.5 + dx / len * out, st.pos.y, f.z as f64 + 0.5 + dz / len * out], 1.0);
            return true;
        }
        // The player's own campfire, looked for once a second (players' ids share the map), or a
        // player back from the dead on their way to their tombstone (`death.rs`).
        match ai::target(w, id, st, 35.0) {
            Some((p, ps)) if crate::death::in_grace(w, p) || fire_for(w, p, ps.pos).is_some() => {
                if w.navigating(id) {
                    w.navigate(id, st.pos, 1.0);
                }
                true
            }
            _ => false,
        }
    }
}

/// Monsters chase the player they are after (what `goals` sets them off by) and hit it when in
/// reach (`ai::melee`, once a second). Otherwise they wander.
fn hostile(goals: ai::Goals) -> ai::Goals {
    goals
        .then(ai::watch_player(8.0))
        .then(keep_from_fire())
        .then(ai::melee())
        .then(blood_chase())
        .then(ai::chase_target(35.0, 1.0))
        .then(ai::wander(0.6, 10.0))
}

/// A blood moon's monster sees players further (`vitals::events::BLOOD_MOON_REACH`).
fn blood_chase() -> impl ai::Goal {
    let chase = ai::chase_target(35.0 * vitals::events::BLOOD_MOON_REACH as f64, 1.0);
    move |w: &World, id: u64, st: &EntityState| crate::events::is_blood(id) && ai::Goal::run(&chase, w, id, st)
}

/// A spider's peace: out in daylight.
fn in_daylight(w: &World, st: &EntityState) -> bool {
    let feet = [st.pos.x.floor() as i32, (st.pos.y + 0.5).floor() as i32, st.pos.z.floor() as i32];
    w.daylight() >= 0.5 && w.sky_light(feet) >= 10
}

/// Animals run about for three seconds after being hurt, otherwise wander.
fn passive() -> ai::Goals {
    with_passive(ai::Goals::new())
}

fn with_passive(goals: ai::Goals) -> ai::Goals {
    goals
        .timid(3.0)
        .then(ai::watch_player(6.0))
        .then(ai::panic(1.6))
        .then(ai::wander(0.5, 10.0))
}

// Where each of base's mobs is hit, part by part, in the pixels of its model: what its body
// looks like, not the model itself (`Hitboxes`).
fn four_legs(h: Hitboxes, x: [f32; 2], tall: f32, front: [f32; 2], back: [f32; 2]) -> Hitboxes {
    h.bone("front_right_leg", "leg3", [-x[1], 0.0, front[0]], [-x[0], tall, front[1]])
        .bone("front_left_leg", "leg2", [x[0], 0.0, front[0]], [x[1], tall, front[1]])
        .bone("back_right_leg", "leg0", [-x[1], 0.0, back[0]], [-x[0], tall, back[1]])
        .bone("back_left_leg", "leg1", [x[0], 0.0, back[0]], [x[1], tall, back[1]])
}

fn sheep_parts() -> Hitboxes {
    let h = Hitboxes::new()
        .bone("head", "head", [-4.0, 14.0, -14.0], [4.0, 22.0, -6.0])
        .bone("body", "body", [-4.5, 10.0, -8.0], [4.5, 18.0, 10.0]);
    four_legs(h, [1.0, 5.0], 12.0, [-7.0, -3.0], [5.0, 9.0])
}

fn pig_parts() -> Hitboxes {
    let h = Hitboxes::new()
        .bone("head", "head", [-4.0, 8.0, -15.0], [4.0, 16.0, -6.0])
        .bone("body", "body", [-5.0, 6.0, -8.0], [5.0, 14.0, 9.0]);
    four_legs(h, [1.0, 5.0], 6.0, [-7.0, -3.0], [5.0, 9.0])
}

fn chicken_parts() -> Hitboxes {
    Hitboxes::new()
        .bone("head", "head", [-2.0, 9.0, -8.0], [2.0, 15.0, -3.0])
        .bone("body", "body", [-3.0, 5.0, -4.0], [3.0, 11.0, 4.0])
        .bone("right_wing", "wing0", [-4.0, 7.0, -3.0], [-3.0, 11.0, 3.0])
        .bone("left_wing", "wing1", [3.0, 7.0, -3.0], [4.0, 11.0, 3.0])
        .bone("right_leg", "rightLeg", [-2.5, 0.0, -1.0], [-1.5, 5.0, 1.0])
        .bone("left_leg", "leftLeg", [0.5, 0.0, -1.0], [1.5, 5.0, 1.0])
}

fn spider_parts() -> Hitboxes {
    Hitboxes::new()
        .head("head", [-4.0, 5.0, -11.0], [4.0, 13.0, -3.0], [0.0, 9.0, -3.0])
        .part("body", [-3.0, 6.0, -3.0], [3.0, 12.0, 3.0])
        .part("abdomen", [-5.0, 5.0, 3.0], [5.0, 13.0, 15.0])
        .part("right_legs", [-15.0, 0.0, -4.0], [-3.0, 9.0, 4.0])
        .part("left_legs", [3.0, 0.0, -4.0], [15.0, 9.0, 4.0])
}

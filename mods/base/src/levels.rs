//! Levels (`notes/SURVIVAL.md`, step 4): XP from what one does (fighting, mining, building,
//! crafting, exploring, advancements; other mods add theirs with `base_api::give_xp`), a level up
//! now and then with a point to place in a stat (the stats screen, P), and a tombstone where one
//! dies that holds everything one carried.

use std::sync::Mutex;
use std::time::Instant;

use base_api::{HEALTH_SCALE, VITALS, give_xp};
use vitals::levels::{self, xp};
use vitals::stats::{self, STATS, Stat};
use weft_sdk::*;

static LEVEL_UP: Channel<u32> = Channel::named("level_up");
static PLACE: Channel<u32> = Channel::named("place_stat");
static SCREEN: Screen = Screen::named("stats");
static OPEN: Key = Key::named("stats");
static FANFARE: Sound = Sound::named("level_up");
pub static TOMBSTONE: Block = Block::named("tombstone");
static TOMBSTONE_MENU: Menu = Menu::named("tombstone");

/// Seconds a level up's banner shows.
const BANNER_SECS: f32 = 4.0;

/// Points to place: a level's each, less those placed.
pub fn unspent(v: &base_api::Vitals) -> u32 {
    let level = levels::level_of(v.xp).0;
    (level - 1).saturating_sub(v.points.iter().map(|&p| p as u32).sum())
}

pub fn register(r: &mut Registry) {
    r.sound(&FANFARE)
        .files(&["random/advancement"])
        .category(&sound_categories::UI);
    // Mining, chopping, digging: by how hard the block is.
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            give_xp(&w, e.player, xp::gather(HARDNESS.on_block(e.state).unwrap_or(1.0).max(0.0)));
        }
    });
    // Building: a little a block.
    r.on(&BLOCK_PLACED, |b| {
        let w = b.world();
        for e in b.iter() {
            give_xp(&w, e.player, xp::BUILD);
        }
    });
    // Killing: by the creature's strength (a monster more than a chicken).
    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            if d.killer != 0 && d.killer != d.entity && w.players().contains(&d.killer) {
                let danger = if w.players().contains(&d.entity) { 3.0 } else { 1.5 };
                give_xp(&w, d.killer, xp::kill(1, danger));
            }
            if w.players().contains(&d.entity) {
                tombstone(&w, d.entity, d.pos);
            }
        }
    });
    // A point placed in a stat from the stats screen.
    r.packet(&PLACE, |p, stat| {
        let w = p.world();
        let Some(v) = w.component_mut(p.player(), &VITALS) else { return };
        if (stat as usize) < STATS && unspent(v) > 0 {
            v.points[stat as usize] += 1;
        }
    });
    // Gives XP (for testing), or tells it.
    r.command("xp", "[amount:int]", |cmd| {
        let w = cmd.world();
        if let Some(n) = cmd.int("amount") {
            give_xp(&w, cmd.player(), n.max(0) as u64);
        }
        let Some(v) = w.component(cmd.player(), &VITALS) else { return };
        let (level, into) = levels::level_of(v.xp);
        cmd.reply("command.base.xp", &[Arg::Int(level as i64), Arg::Int(into as i64), Arg::Int(levels::to_next(level) as i64)]);
    })
    .permission(permission::GAME_MASTER);
    r.world_tick(tick);
    tombstone_block(r);
    client(r);
}

/// Level ups, and exploring: a biome or dimension seen for the first time.
fn tick(w: &World) {
    if w.time() % 20 != 0 {
        return;
    }
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        let biome = w.biome([st.pos[0].floor() as i32, st.pos[1].floor() as i32, st.pos[2].floor() as i32]);
        let dim = w.dimension();
        let creative = w.creative(id);
        let Some(v) = w.component_mut(id, &VITALS) else { continue };
        let mut found = 0;
        if (biome as usize) < v.visited.len() * 32 && v.visited[biome as usize / 32] & 1 << (biome % 32) == 0 {
            v.visited[biome as usize / 32] |= 1 << (biome % 32);
            found += xp::DISCOVER;
        }
        if dim < 32 && v.dimensions & 1 << dim == 0 {
            v.dimensions |= 1 << dim;
            found += xp::DISCOVER * 10;
        }
        // The world one starts in, and its first biome, are no discovery.
        if v.level == 0 {
            found = 0;
        }
        if found > 0 && !creative {
            give_xp(w, id, found);
        }
        let Some(v) = w.component_mut(id, &VITALS) else { continue };
        let level = levels::level_of(v.xp).0;
        if level as u16 > v.level {
            if v.level > 0 {
                w.send(id, &LEVEL_UP, &level);
            }
            v.level = level as u16;
        }
    }
}

/// What one carried goes into a tombstone where one died (the first open space at or over the
/// spot), before the engine drops what's left.
fn tombstone(w: &World, player: u64, pos: [f64; 3]) {
    let inv = w.inventory(player);
    let carried: Vec<ItemStack> = (0..inv.size()).map(|i| inv.get(i)).collect();
    if carried.iter().all(|s| s.is_empty()) || w.creative(player) {
        return;
    }
    let base = [pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32];
    let Some(at) = (0..8)
        .map(|dy| [base[0], base[1] + dy, base[2]])
        .find(|&p| w.get(p) == 0 || block_name(w.get(p)).starts_with("core:air"))
    else {
        return;
    };
    w.set(at, TOMBSTONE.id());
    let grave = w.block_entity(at);
    if grave == 0 {
        return;
    }
    let into = w.inventory(grave);
    let mut slot = 0;
    for (i, s) in carried.into_iter().enumerate() {
        if s.is_empty() {
            continue;
        }
        if slot >= into.size() {
            break;
        }
        into.set(slot, s);
        inv.set(i as u32, ItemStack::EMPTY);
        slot += 1;
    }
}

/// A gravestone holding a dead one's things (a chest's menu with room for a whole inventory);
/// broken, it drops what's left.
fn tombstone_block(r: &mut Registry) {
    r.machine(&TOMBSTONE, &TOMBSTONE_MENU)
        .block(|b| {
            b.sound_group("stone")
                .facing(Facings::HORIZONTAL, Toward::Player)
                .states(|_, s| {
                    s.faces(["mossy_cobblestone"; 6])
                        .model_box([3.0, 0.0, 6.0], [13.0, 15.0, 10.0])
                })
                .hardness(1.0)
                .tab(&crate::tabs::FUNCTIONAL)
        })
        .group("grave", Slots::chest(5))
        .screen(|ui| {
            ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
                ui.title();
                ui.group("grave");
                ui.player_inventory(true);
            });
        });
}

/// A stat's value as the screen shows it.
fn shown(stat: Stat, value: f32) -> String {
    match stat {
        Stat::Damage | Stat::Speed => format!("{:.0}%", value * 100.0),
        Stat::Weight => format!("{value:.0} kg"),
        Stat::Fortitude => format!("+{value:.0}"),
        Stat::Food => format!("{value:.0} u"),
        _ => format!("{value:.0}"),
    }
}

/// Level ups waiting to show, the first with the moment it appeared.
static BANNERS: Mutex<Vec<(u32, Option<Instant>)>> = Mutex::new(Vec::new());

fn client(r: &mut Registry) {
    r.client_packet(&LEVEL_UP, |_, level| BANNERS.lock().unwrap().push((level, None)));
    r.client_frame(|c| {
        let mut banners = BANNERS.lock().unwrap();
        let Some((level, shown)) = banners.first_mut() else { return };
        let start = *shown.get_or_insert_with(|| {
            c.play_sound(&FANFARE, None, 1.0);
            Instant::now()
        });
        if start.elapsed().as_secs_f32() > BANNER_SECS {
            banners.remove(0);
            return;
        }
        c.screen_text(0.5, 0.22, "levels.base.up", &[Arg::Int(*level as i64)], 0xffbe45ff, 2.0);
        c.screen_text(0.5, 0.22 + 0.05, "levels.base.place", &[], 0xf5e4bcff, 1.0);
    });
    // The stats: each one's value, the points in it, and a + while there are points to place.
    r.screen(&SCREEN, |ui| {
        ui.panel("panel", S::col().pad(8.0).gap(4.0), |ui| {
            ui.row(S::row().gap(8.0), |ui| {
                ui.text_bind_styled(S::col().class("title").scale(1.5).no_wrap(), |c, t| {
                    let Some(v) = c.component(&VITALS) else { return };
                    t.set("levels.base.title", &[Arg::Int(levels::level_of(v.xp).0 as i64)]);
                });
                ui.spacer();
                ui.text_bind_styled(S::col().class("dim").no_wrap(), |c, t| {
                    let Some(v) = c.component(&VITALS) else { return };
                    let (level, into) = levels::level_of(v.xp);
                    t.set(
                        "levels.base.progress",
                        &[Arg::Int(into as i64), Arg::Int(levels::to_next(level) as i64), Arg::Int(unspent(&v) as i64)],
                    );
                });
            });
            ui.panel("well", S::col().pad(6.0).gap(3.0), |ui| {
                for (i, stat) in stats::ALL.into_iter().enumerate() {
                    ui.row(S::row().gap(6.0).center(), |ui| {
                        ui.text_styled(S::default().w(70.0).no_wrap(), &format!("stat.base.{}", stat.name()), &[]);
                        ui.text_bind_styled(S::default().w(50.0).no_wrap(), move |c, t| {
                            let Some(v) = c.component(&VITALS) else { return };
                            t.text(&shown(stat, v.stats()[i]));
                        });
                        ui.text_bind_styled(S::default().w(30.0).no_wrap(), move |c, t| {
                            let Some(v) = c.component(&VITALS) else { return };
                            t.text(&v.points[i].to_string());
                        });
                        ui.button(
                            "levels.base.plus",
                            S::default()
                                .size(14.0, 12.0)
                                .tooltip(&format!("stat.base.{}.desc", stat.name())),
                            move |c| {
                                c.client().send(&PLACE, &(i as u32));
                            },
                        );
                    });
                }
            });
            ui.text_bind_styled(S::col().class("dim").no_wrap(), |c, t| {
                let (Some(v), Some(me)) = (c.component(&VITALS), c.me()) else { return };
                t.set("levels.base.now", &[Arg::Int((me.health * HEALTH_SCALE).round() as i64), Arg::Int(v.max_health.round() as i64)]);
            });
            ui.text_bind_styled(S::col().class("dim").no_wrap(), |c, t| {
                let Some(v) = c.component(&VITALS) else { return };
                let weight = v.stats()[Stat::Weight.index()];
                t.set("levels.base.load", &[Arg::Float(v.load as f64, 1), Arg::Int(weight.round() as i64)]);
            });
        });
    });
    r.key(&OPEN, "P").on_press(|c| {
        c.open_screen(&SCREEN);
    });
}

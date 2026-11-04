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
pub static SCREEN: Screen = Screen::named("stats");
static OPEN: Key = Key::named("stats");
static FANFARE: Sound = Sound::named("level_up");
pub static TOMBSTONE: Block = Block::named("tombstone");
static TOMBSTONE_MENU: Menu = Menu::named("tombstone");

/// Seconds a level up's banner shows (a milestone's, every `MILESTONE` levels), and its small toast
/// (the others), and how far above the window's bottom the toast's text starts (GUI px: over the
/// hotbar, 23, the level row, 12, and 2 of air, its line 8.5).
const BANNER_SECS: f32 = 4.0;
const TOAST_SECS: f32 = 2.5;
const TOAST_ABOVE: f32 = 23.0 + 12.0 + 2.0 + 8.5;
const MILESTONE: u32 = 5;

/// A level that gets the big banner.
fn milestone(level: u32) -> bool {
    level.is_multiple_of(MILESTONE)
}

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
                // What killed them and where their things are, for the death screen.
                crate::death::tell(&w, d);
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
    if !w.creative(player) {
        crate::clothing::set_aside(w, player);
    }
    // This death's tombstone or none: never the last one's (integration review: dying in lava
    // pointed the way to an older one).
    if let Some(g) = w.component_mut(player, &base_api::GRAVE) {
        g.set = 0;
    }
    if let Some(d) = w.component_mut(player, &crate::death::DROPPED) {
        *d = 0;
    }
    let inv = w.inventory(player);
    let carried: Vec<ItemStack> = (0..inv.size()).map(|i| inv.get(i)).collect();
    if carried.iter().all(|s| s.is_empty()) || w.creative(player) {
        return;
    }
    // Into air, or water, lava or anything a block replaces (tall grass, snow), up to 8 above.
    let base = [pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32];
    let room = |s: u32| {
        s == 0 || block_name(s).starts_with("core:air") || state_flags(s) & (abi::block_flags::REPLACEABLE | abi::block_flags::LIQUID) != 0
    };
    let Some(at) = (0..8)
        .map(|dy| [base[0], base[1] + dy, base[2]])
        .find(|&p| room(w.get(p)))
    else {
        // Nowhere: the engine drops it all where one died.
        if let Some(d) = w.component_mut(player, &crate::death::DROPPED) {
            *d = 1;
        }
        return;
    };
    w.set(at, TOMBSTONE.id());
    if let Some(g) = w.component_mut(player, &base_api::GRAVE) {
        // `set` is the dimension + 1: the marker and the grace look for it only there.
        *g = base_api::Grave { x: at[0], y: at[1], z: at[2], set: w.dimension() + 1 };
    }
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
            crate::window::window(ui, S::col().pad(7.0).gap(4.0), &[], |ui| {
                ui.group("grave");
                ui.player_inventory(true);
            });
        });
}

/// A stat's value as a number alone, its unit in a column of its own (`unit_key`), so the
/// numbers line up on their right edge.
fn number(stat: Stat, value: f32) -> String {
    match stat {
        Stat::Damage | Stat::Speed => format!("{:.0}", value * 100.0),
        _ => format!("{value:.0}"),
    }
}

/// A stat's unit, a translation key (empty for none).
fn unit_key(stat: Stat) -> &'static str {
    match stat {
        Stat::Damage | Stat::Speed => "levels.base.unit.percent",
        Stat::Weight => "levels.base.unit.kg",
        Stat::Food => "levels.base.unit.units",
        _ => "",
    }
}

/// Points placed on the stats screen and not kept yet (Ronald's round 2: "see a point before you
/// keep it", as Elden Ring and Baldur's Gate 3 do). This client's only: Keep sends them, Undo
/// drops them.
static PENDING: Mutex<[u16; vitals::stats::STATS]> = Mutex::new([0; vitals::stats::STATS]);

/// The points waiting, none if there are more than the points left to place (after a death).
fn pending(v: &base_api::Vitals) -> [u16; vitals::stats::STATS] {
    let p = *PENDING.lock().unwrap();
    if p.iter().map(|&n| n as u32).sum::<u32>() > unspent(v) { [0; vitals::stats::STATS] } else { p }
}

/// The vitals as they would be with the waiting points placed.
fn after(v: &base_api::Vitals) -> base_api::Vitals {
    let mut a = *v;
    for (i, n) in pending(v).into_iter().enumerate() {
        a.points[i] += n;
    }
    a
}

/// The stats screen's columns after the bar (GUI pixels): the value now (right-aligned), its unit,
/// the value after the waiting points (right-aligned), the points waiting (centred).
const NOW_W: f32 = 18.0;
const UNIT_W: f32 = 16.0;
const AFTER_W: f32 = 18.0;
const WAITING_W: f32 = 8.0;

/// Level ups waiting to show, the first with the moment it appeared.
static BANNERS: Mutex<Vec<(u32, Option<Instant>)>> = Mutex::new(Vec::new());

/// The stats screen's bars (the kit's `stat_<name>`, 150 x 12 pixels) in GUI pixels, and the
/// points that fill one: a bar shows the points placed in a stat.
const STAT_BAR: (f32, f32) = (60.0, 6.0);
const POINTS_FULL: f32 = 20.0;
/// A stat's bar with no points placed: a third full, so it reads as the stat, growing with points.
const BAR_BASE: f32 = 0.3;

fn client(r: &mut Registry) {
    r.client_packet(&LEVEL_UP, |_, level| {
        let mut banners = BANNERS.lock().unwrap();
        // Level ups early come every few chops: a small one not shown yet gives way to the next
        // (Ronald's first-day test, 11-04: "Level 3!", "Level 4!" over the screen's middle).
        if !milestone(level) {
            banners.retain(|&(l, shown)| milestone(l) || shown.is_some());
        }
        banners.push((level, None));
    });
    r.client_frame(|c| {
        let mut banners = BANNERS.lock().unwrap();
        let Some((level, shown)) = banners.first_mut() else { return };
        // Never over a window (Tester's Retina run: "Level 3!" printed over the stats window's
        // header, the animal screen's chip): it waits while a screen is open, and starts over,
        // sound and all, once it closes.
        if !c.screen().is_empty() {
            *shown = None;
            return;
        }
        let big = milestone(*level);
        let start = *shown.get_or_insert_with(|| {
            c.play_sound(&FANFARE, None, if big { 1.0 } else { 0.5 });
            Instant::now()
        });
        if start.elapsed().as_secs_f32() > if big { BANNER_SECS } else { TOAST_SECS } {
            banners.remove(0);
            return;
        }
        if big {
            c.screen_text(0.5, 0.22, "levels.base.up", &[Arg::Int(*level as i64)], 0xffbe45ff, 2.0);
            c.screen_text(0.5, 0.22 + 0.05, "levels.base.place", &[], 0xf5e4bcff, 1.0);
            return;
        }
        // The small one: one line just above the level row over the hotbar (`hud.rs`, HUD_LEVEL),
        // clear of the screen's middle.
        let st = c.state();
        let gh = st.window[1] as f32 / st.gui_scale.max(1.0);
        c.screen_text(0.5, (gh - TOAST_ABOVE) / gh, "levels.base.toast", &[Arg::Int(*level as i64)], 0xffbe45ff, 1.0);
    });
    // The stats (the UI redo's stats screen): the level on a badge with the XP bar and the points
    // to place, then each stat as a bar with its value inside, its points and a +.
    r.screen(&SCREEN, |ui| {
        let tabs: [crate::window::Tab; 1] = [("window.base.tab.advancements", |c| {
            c.client().open_screen(&crate::advancements::SCREEN);
        })];
        crate::window::window_titled(ui, S::col().pad(8.0).gap(4.0), "window.base.tab.stats", &tabs, |ui| {
            ui.row(S::row().gap(5.0).center(), |ui| {
                ui.panel("hud_badge", S::row().size(20.0, 24.0).center(), |ui| {
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let Some(v) = c.component(&VITALS) else { return };
                        t.set("levels.base.badge", &[Arg::Int(levels::level_of(v.xp).0 as i64)]);
                    });
                });
                ui.column(S::col().gap(2.0), |ui| {
                    ui.row(S::row().gap(6.0).center(), |ui| {
                        ui.text_bind_styled(S::default().class("title").no_wrap(), |c, t| {
                            let Some(v) = c.component(&VITALS) else { return };
                            t.set("levels.base.title", &[Arg::Int(levels::level_of(v.xp).0 as i64)]);
                        });
                        ui.panel("chip_points", S::row().pad(2.0), |ui| {
                            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                                let Some(v) = c.component(&VITALS) else { return };
                                let waiting: u32 = pending(&v).iter().map(|&n| n as u32).sum();
                                t.set("levels.base.points", &[Arg::Int((unspent(&v) - waiting) as i64)]);
                            });
                        });
                    });
                    ui.row(S::row().gap(4.0).center(), |ui| {
                        ui.progress_bind("stat_xp", S::default().size(110.0, 4.0), |c| {
                            let Some(v) = c.component(&VITALS) else { return -1.0 };
                            let (level, into) = levels::level_of(v.xp);
                            (2.0 + into as f32 / levels::to_next(level).max(1) as f32 * 216.0) / 220.0
                        });
                        ui.text_bind_styled(S::default().class("dim").no_wrap(), |c, t| {
                            let Some(v) = c.component(&VITALS) else { return };
                            let (level, into) = levels::level_of(v.xp);
                            t.set("levels.base.xp", &[Arg::Int(into as i64), Arg::Int(levels::to_next(level) as i64)]);
                        });
                    });
                });
            });
            ui.column(S::col().gap(2.0), |ui| {
                // The header in the rows' columns: "now" over the values, "after" over what the
                // waiting points make of them, each lined up as its values are (right).
                ui.row(S::row().gap(4.0), |ui| {
                    ui.row(S::row().w(8.0 + 4.0 + 40.0 + 4.0 + STAT_BAR.0), |_| {});
                    ui.row(S::row().w(NOW_W).end(), |ui| ui.text_styled(S::default().class("dim").no_wrap(), "levels.base.now_head", &[]));
                    ui.row(S::row().w(UNIT_W), |_| {});
                    ui.row(S::row().w(AFTER_W).end(), |ui| {
                        ui.text_styled(S::default().class("dim").no_wrap(), "levels.base.after_head", &[])
                    });
                });
                for (i, stat) in stats::ALL.into_iter().enumerate() {
                    ui.row(S::row().gap(4.0).center(), |ui| {
                        ui.image(&format!("base:gui/stat_icon_{}", stat.name()), 8.0, 8.0);
                        ui.text_styled(S::default().w(40.0).no_wrap(), &format!("stat.base.{}", stat.name()), &[]);
                        ui.progress_bind(
                            Box::leak(format!("stat_{}", stat.name()).into_boxed_str()),
                            S::default().size(STAT_BAR.0, STAT_BAR.1),
                            move |c| {
                                let Some(v) = c.component(&VITALS) else { return -1.0 };
                                BAR_BASE + (1.0 - BAR_BASE) * (v.points[i] as f32 / POINTS_FULL).min(1.0)
                            },
                        );
                        ui.row(S::row().w(NOW_W).end(), move |ui| {
                            ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                                let Some(v) = c.component(&VITALS) else { return };
                                t.text(&number(stat, v.stats()[i]));
                            });
                        });
                        ui.row(S::row().w(UNIT_W), move |ui| {
                            let unit = unit_key(stat);
                            if !unit.is_empty() {
                                ui.text_styled(S::default().no_wrap(), unit, &[]);
                            }
                        });
                        ui.row(S::row().w(AFTER_W).end(), move |ui| {
                            ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                                let Some(v) = c.component(&VITALS) else { return };
                                if pending(&v)[i] == 0 {
                                    return t.text("");
                                }
                                t.set("levels.base.after", &[Arg::Text(&number(stat, after(&v).stats()[i]))]);
                            });
                        });
                        ui.button(
                            "levels.base.minus",
                            S::default()
                                .class("button_secondary")
                                .size(10.0, 9.0)
                                .pad(0.5)
                                .tooltip("levels.base.minus_tip"),
                            move |_| {
                                let mut p = PENDING.lock().unwrap();
                                p[i] = p[i].saturating_sub(1);
                            },
                        );
                        ui.row(S::row().w(WAITING_W).center(), move |ui| {
                            ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                                let Some(v) = c.component(&VITALS) else { return };
                                match pending(&v)[i] {
                                    0 => t.set("levels.base.waiting_none", &[]),
                                    n => t.set("levels.base.waiting", &[Arg::Int(n as i64)]),
                                }
                            });
                        });
                        ui.button(
                            "levels.base.plus",
                            S::default()
                                .size(10.0, 9.0)
                                .pad(0.5)
                                .tooltip(&format!("stat.base.{}.desc", stat.name())),
                            move |c| {
                                let Some(v) = c.component(&VITALS) else { return };
                                let waiting: u32 = pending(&v).iter().map(|&n| n as u32).sum();
                                if waiting < unspent(&v) {
                                    PENDING.lock().unwrap()[i] += 1;
                                }
                            },
                        );
                    });
                }
            });
            // Keep sends the waiting points (one place each, as the + used to); Undo drops them.
            ui.row(S::row().gap(4.0), |ui| {
                ui.button("levels.base.keep", S::default().min_w(60.0).h(11.0), |c| {
                    let Some(v) = c.component(&VITALS) else { return };
                    let p = pending(&v);
                    *PENDING.lock().unwrap() = [0; vitals::stats::STATS];
                    for (i, n) in p.into_iter().enumerate() {
                        for _ in 0..n {
                            c.client().send(&PLACE, &(i as u32));
                        }
                    }
                });
                ui.button("levels.base.undo", S::default().class("button_secondary").min_w(44.0).h(11.0), |_| {
                    *PENDING.lock().unwrap() = [0; vitals::stats::STATS];
                });
            });
            ui.text_styled(S::col().class("dim").max_w(170.0), "levels.base.after_legend", &[]);
            ui.text_bind_styled(S::col().class("dim").max_w(170.0), |c, t| {
                let (Some(v), Some(me)) = (c.component(&VITALS), c.me()) else { return };
                t.set("levels.base.now", &[Arg::Int((me.health * HEALTH_SCALE).round() as i64), Arg::Int(v.max_health.round() as i64)]);
            });
            ui.text_bind_styled(S::col().class("dim").max_w(170.0), |c, t| {
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

//! The HUD's information around the screen's edges, after Ronald's reference: health and stamina
//! bars, the stomach and the effects in the top left (the vitals, `survival/`), armor and breath
//! under them (`player.rs`), the keys of the main screens down the left, the time of day and where
//! you are in the top right.

use base_api::{HEALTH_SCALE, VITALS, Vitals};
use vitals::effects;
use vitals::food::FOODS;
use weft_sdk::*;

static HUD_CLOCK: HudElement = HudElement::named("clock");
static HUD_POSITION: HudElement = HudElement::named("position");
static HUD_SHORTCUTS: HudElement = HudElement::named("shortcuts");
static HUD_HEALTH: HudElement = HudElement::named("health");
static HUD_LEVEL: HudElement = HudElement::named("level");
static HUD_STOMACH: HudElement = HudElement::named("food");
static HUD_EFFECTS: HudElement = HudElement::named("effects");
static HUD_TEMPERATURE: HudElement = HudElement::named("temperature");

/// A bar's size in GUI pixels: kit 2's HUD bar (`notes/ui/kit2/install.py`, two texels a GUI
/// pixel), drawn at exactly its size so its pixels stay whole: 19 texels, an outline and a 2 px
/// frame round a 13-row track, so the numbers in it have 2 rows above their capitals and 2 below
/// the baseline. The XP bar is thinner, with a 1 px frame.
const BAR: (f32, f32) = (96.0, 9.5);
/// The bars' rims in texels (outline and frame), of their 192.
const BAR_RIM: f32 = 3.0;
/// How far a bar's numbers end from its right end, GUI pixels: its rim (1.5) and 1.5 of padding.
const BAR_TEXT_INSET: f32 = 3.0;
/// The level row above the hotbar (`HUD_LEVEL`), GUI px: as wide as the hotbar (9 slots of 20)
/// and as tall as its chips; the XP bar in it (`hud_xpbar`, 360 x 8 texels, a 1 px frame: 2 texels
/// of rim), centred on the row's 22 UI px (7 above, 7 below); the chips (`hud_level_chip`,
/// `hud_points_chip`, 32 x 22 texels: built like kit 2's buttons, their face 17 rows, the number
/// 4 above and 4 below); the gap before the points chip.
const LEVEL_ROW: (f32, f32) = (180.0, 11.0);
const XPBAR_H: f32 = 4.0;
const XPBAR_RIM: f32 = 2.0;
const XPBAR_TEXELS: f32 = 360.0;
const CHIP: (f32, f32) = (16.0, 11.0);
const LEVEL_GAP: f32 = 2.0;

/// `v` rounded down to the UI pixel (half a GUI pixel).
fn down_half(v: f32) -> f32 {
    (v * 2.0).floor() / 2.0
}

/// A stretched bar's fill for `share` filled, its fill starting `rim` of its `texels` in.
fn fill_of(share: f32, rim: f32, texels: f32) -> f32 {
    (rim + share.clamp(0.0, 1.0) * (texels - 2.0 * rim)) / texels
}

/// The points chip's pulse: lit for half of every second.
fn pulse_lit() -> bool {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis()
        % 1000
        < 500
}

/// Where a bar's fill ends for a share filled: the kit's fill starts `rim` of the bar's 192 texels
/// in and spans the rest but the rim at the other end.
fn bar_fill(share: f32, rim: f32) -> f32 {
    (rim + share.clamp(0.0, 1.0) * (192.0 - 2.0 * rim)) / 192.0
}

/// The thermometer's size (the kit's gauge, 128 x 10 pixels), and the body temperatures at its
/// ends, °C.
const THERMOMETER: (f32, f32) = (64.0, 5.0);
const COLDEST: f32 = 31.0;
const HOTTEST: f32 = 43.0;

/// The player's vitals, none in creative or while dead (the bars hide: a dead player's are
/// placeholders, health "0 / -1" over the death screen).
fn vitals_of(c: &ClientView) -> Option<Vitals> {
    if c.creative() || c.player_state().is_none_or(|m| m.health <= 0.0) {
        return None;
    }
    c.component(&VITALS)
        .filter(|v| v.max_health > 0.0 && v.max_stamina > 0.0)
}

/// The screens a key opens, each with its picture (`gui/hud_<name>.png`) and its key
/// (`hud.base.shortcut.<name>`, as the player has bound it).
/// The effects the HUD names at once (the rest wait their turn).
const EFFECTS_SHOWN: usize = 6;

/// The food tiles (`food_tile`): how many show, their size (`hud_food`, 44 x 52 texels) and the
/// gap between them (GUI px), and the bar under each (`hud_food_bar`, 44 x 6 texels). In a tile
/// (UI px, two a GUI px): a 3 px rim, 1 px, the picture (32 px), 2 px, the minutes' capitals
/// (9), 2 px below their baseline, the rim. Item pictures are drawn after all text, so the minutes
/// sit under the picture, not over it, as on Ronald's sheet.
const FOOD_TILES: usize = 6;
const FOOD_TILE: (f32, f32) = (22.0, 26.0);
const FOOD_GAP: f32 = 2.0;
const FOOD_BAR: (f32, f32) = (22.0, 3.0);
/// Where the picture's top left is in the tile, and the top of the minutes' line (GUI px).
const FOOD_PICTURE: (f32, f32) = (3.0, 2.0);
const FOOD_MINUTES_Y: f32 = 17.0;

/// The stomach's foods, each once in the order first eaten: the food, how many portions of it, and
/// its longest-lasting portion's units left.
fn foods_held(v: &base_api::Vitals) -> Vec<(&'static vitals::food::Food, u32, f32)> {
    let mut out: Vec<(&'static vitals::food::Food, u32, f32)> = Vec::new();
    for i in 0..base_api::PORTIONS {
        let Some(f) = (v.foods[i] as usize)
            .checked_sub(1)
            .and_then(|f| FOODS.get(f))
        else {
            continue;
        };
        if v.units[i] <= 0.0 {
            continue;
        }
        match out.iter_mut().find(|(g, _, _)| std::ptr::eq(*g, f)) {
            Some(e) => {
                e.1 += 1;
                e.2 = e.2.max(v.units[i]);
            }
            None => out.push((f, 1, v.units[i])),
        }
    }
    out
}

/// Food tile `k`: a slot (a gauge, so it hides when there is no `k`th food), the food's picture at
/// twice the text's size (16 GUI px, its pixels whole: an item in text is 16 font px at its line's
/// top left), the minutes under it, the count in the top left as a stack's, and the bar under the
/// tile.
fn food_tile(ui: &mut Ui, k: usize) {
    let held = move |c: &ClientView| vitals_of(c).and_then(|v| foods_held(&v).get(k).copied());
    ui.column(S::column().gap(1.0), move |ui| {
        ui.row(S::row().size(FOOD_TILE.0, FOOD_TILE.1), move |ui| {
            ui.progress_bind("hud_food", S::default().size(FOOD_TILE.0, FOOD_TILE.1).at(0.0, 0.0), move |c| {
                if held(c).is_some() { 1.0 } else { -1.0 }
            });
            ui.text_bind_styled(
                S::default()
                    .no_wrap()
                    .scale(2.0)
                    .at(FOOD_PICTURE.0, FOOD_PICTURE.1),
                move |c, t| match held(c).and_then(|(f, _, _)| Item::lookup(f.name)) {
                    Some(item) => t.set("", &[Arg::Item(item)]),
                    None => t.text(""),
                },
            );
            // The minutes, centred on the tile (the layout rounds their odd leftover down).
            ui.row(
                S::row()
                    .size(FOOD_TILE.0, 8.5)
                    .center()
                    .at(0.0, FOOD_MINUTES_Y),
                move |ui| {
                    ui.text_bind_styled(S::default().no_wrap(), move |c, t| match held(c) {
                        Some((f, _, units)) => t.set("hud.base.food_minutes", &[Arg::Int((units / f.digest_rate()).ceil() as i64)]),
                        None => t.text(""),
                    });
                },
            );
            ui.text_bind_styled(S::default().no_wrap().at(2.0, 0.0), move |c, t| match held(c) {
                Some((_, n, _)) if n > 1 => t.set("hud.base.food_count", &[Arg::Int(n as i64)]),
                _ => t.text(""),
            });
        });
        ui.progress_bind("hud_food_bar", S::default().size(FOOD_BAR.0, FOOD_BAR.1), move |c| match held(c) {
            // the bar's rim is 1 of its 44 texels at each end
            Some((f, _, units)) => (1.0 + (units / f.units()).clamp(0.0, 1.0) * 42.0) / 44.0,
            None => -1.0,
        });
    });
}

const SHORTCUTS: [&str; 4] = ["inventory", "recipes", "stats", "advancements"];

pub fn register(r: &mut Registry) {
    // Health, stamina and experience as Ronald's sheet has them: the level on a badge, and beside
    // it the bars, each with its numbers in the vitals' units; the points waiting to be placed (P)
    // after the XP bar.
    // Health and stamina as Ronald's sheet has them, each bar with its numbers in the vitals'
    // units. The level, the XP and the points to place are above the hotbar (`HUD_LEVEL`; Ronald,
    // 11-04: "1: above the hotbar").
    r.hud_element(&HUD_HEALTH, hud_anchor::TOP_LEFT, 0, |ui| {
        ui.column(S::column().gap(1.0), |ui| {
            // The numbers inside the bar's right end, as on Ronald's sheet.
            ui.row(S::row().size(BAR.0, BAR.1), |ui| {
                ui.progress_bind("hud_health", S::default().size(BAR.0, BAR.1), |c| {
                    let (Some(v), Some(me)) = (vitals_of(c), c.player_state()) else { return -1.0 };
                    bar_fill(me.health * HEALTH_SCALE / v.max_health.max(1.0), BAR_RIM)
                });
                ui.text_bind_styled(
                    S::default()
                        .no_wrap()
                        .at_pivot(BAR.0 - BAR_TEXT_INSET, BAR.1 / 2.0, [1.0, 0.5]),
                    |c, t| {
                        let (Some(v), Some(me)) = (vitals_of(c), c.player_state()) else { return t.text("") };
                        t.set(
                            "hud.base.bar_value",
                            &[Arg::Int((me.health * HEALTH_SCALE).round() as i64), Arg::Int(v.max_health.round() as i64)],
                        );
                    },
                );
            });
            ui.row(S::row().size(BAR.0, BAR.1), |ui| {
                ui.progress_bind("hud_stamina", S::default().size(BAR.0, BAR.1), |c| {
                    let Some(v) = vitals_of(c) else { return -1.0 };
                    bar_fill(v.stamina / v.max_stamina.max(1.0), BAR_RIM)
                });
                ui.text_bind_styled(
                    S::default()
                        .no_wrap()
                        .at_pivot(BAR.0 - BAR_TEXT_INSET, BAR.1 / 2.0, [1.0, 0.5]),
                    |c, t| {
                        let Some(v) = vitals_of(c) else { return t.text("") };
                        t.set("hud.base.bar_value", &[Arg::Int(v.stamina.round() as i64), Arg::Int(v.max_stamina.round() as i64)]);
                    },
                );
            });
        });
    });
    // The level, the XP and the points to place, as Minecraft has them (Ronald picked it from the
    // other games, 11-04): a slim framed XP bar as wide as the hotbar, just above it, the level on
    // a chip in its middle, and while there are points to place an orange chip at its right end
    // that pulses. Everything is a gauge, so it hides with the bars (creative, dead).
    r.hud_element(&HUD_LEVEL, hud_anchor::HOTBAR, 1, |ui| {
        ui.row(S::row().size(LEVEL_ROW.0, LEVEL_ROW.1), |ui| {
            ui.progress_bind(
                "hud_xpbar",
                S::default()
                    .size(LEVEL_ROW.0, XPBAR_H)
                    .at(0.0, (LEVEL_ROW.1 - XPBAR_H) / 2.0),
                |c| {
                    let Some(v) = vitals_of(c) else { return -1.0 };
                    let (level, into) = vitals::levels::level_of(v.xp);
                    fill_of(into as f32 / vitals::levels::to_next(level).max(1) as f32, XPBAR_RIM, XPBAR_TEXELS)
                },
            );
            // the level's chip, centred on the bar
            let chip_x = down_half((LEVEL_ROW.0 - CHIP.0) / 2.0);
            ui.progress_bind("hud_level_chip", S::default().size(CHIP.0, CHIP.1).at(chip_x, 0.0), |c| {
                if vitals_of(c).is_some() { 1.0 } else { -1.0 }
            });
            ui.row(S::row().size(CHIP.0, CHIP.1).center().at(chip_x, 0.0), |ui| {
                ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                    let Some(v) = vitals_of(c) else { return t.text("") };
                    t.set("hud.base.badge", &[Arg::Int(vitals::levels::level_of(v.xp).0 as i64)]);
                });
            });
            // the points to place at the bar's right end, its outline lit every other half second
            let px = LEVEL_ROW.0 + LEVEL_GAP;
            for (texture, lit) in [("hud_points_chip", false), ("hud_points_chip_lit", true)] {
                ui.progress_bind(texture, S::default().size(CHIP.0, CHIP.1).at(px, 0.0), move |c| {
                    let points = vitals_of(c).map_or(0, |v| crate::levels::unspent(&v));
                    if points > 0 && pulse_lit() == lit { 1.0 } else { -1.0 }
                });
            }
            ui.row(S::row().size(CHIP.0, CHIP.1).center().at(px, 0.0), |ui| {
                ui.text_bind_styled(S::default().no_wrap().class("chip_points"), |c, t| {
                    match vitals_of(c).map(|v| crate::levels::unspent(&v)) {
                        Some(n) if n > 0 => t.set("hud.base.points_chip", &[Arg::Int(n as i64)]),
                        _ => t.text(""),
                    }
                });
            });
        });
    });
    // What's in the stomach: each food with the minutes it has left, and how full of each nutrient.
    r.hud_element(&HUD_STOMACH, hud_anchor::TOP_LEFT, 3, |ui| {
        ui.column(S::column().gap(1.0), |ui| {
            // A tile per food as on Ronald's sheet ("Yes, tiles", 11-04): its picture, the minutes
            // until the last of it is digested, how many portions if more than one, and under it a
            // bar of how much of that portion is left. The places are fixed, so the HUD doesn't
            // move as food comes and goes.
            ui.row(S::row().gap(FOOD_GAP), |ui| {
                for k in 0..FOOD_TILES {
                    food_tile(ui, k);
                }
            });
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = vitals_of(c) else { return t.text("") };
                let s = v.stomach();
                if s.used() <= 0.0 {
                    t.set("hud.base.stomach_empty", &[]);
                    return;
                }
                let full = s
                    .fullness(v.capacity())
                    .map(|f| Arg::Int((f * 100.0).round() as i64));
                t.set("hud.base.nutrients", &full);
            });
        });
    });
    // How warm: an icon for how the body feels, the thermometer filled up to its core temperature,
    // and the air's temperature.
    r.hud_element(&HUD_TEMPERATURE, hud_anchor::TOP_LEFT, 4, |ui| {
        ui.row(S::row().gap(3.0).center(), |ui| {
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = vitals_of(c) else { return t.text("") };
                use vitals::temperature::Feeling as F;
                let icon = match vitals::temperature::feeling(v.core) {
                    F::Freezing => "freezing",
                    F::Cold | F::Chilly => "cold",
                    F::Comfortable => "comfortable",
                    F::Warm => "warm",
                    F::Hot | F::Heatstroke => "hot",
                };
                t.set(&format!("hud.base.feels.{icon}"), &[]);
            });
            ui.progress_bind("hud_temperature", S::default().size(THERMOMETER.0, THERMOMETER.1), |c| {
                let Some(v) = vitals_of(c) else { return -1.0 };
                // The kit's gauge: the gradient starts 2 of its 128 pixels in and spans 124.
                (2.0 + ((v.core - COLDEST) / (HOTTEST - COLDEST)).clamp(0.0, 1.0) * 124.0) / 128.0
            });
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = vitals_of(c) else { return t.text("") };
                t.set("hud.base.outside", &[Arg::Int(v.outside.round() as i64)]);
            });
        });
    });
    // An icon and a name for each effect on, on one line (two spaces apart); rested with its
    // minutes after them.
    r.hud_element(&HUD_EFFECTS, hud_anchor::TOP_LEFT, 5, |ui| {
        ui.row(S::row().gap(4.0), |ui| {
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = vitals_of(c) else { return t.text("") };
                let on: Vec<String> = effects::ALL
                    .iter()
                    .filter(|&&e| e != effects::Effect::Rested && v.has(e))
                    .take(EFFECTS_SHOWN)
                    .map(|e| format!("hud.base.effect.{}", e.name()))
                    .collect();
                if on.is_empty() {
                    return t.text("");
                }
                let args: Vec<Arg> = on.iter().map(|k| Arg::Key(k)).collect();
                t.set(&format!("hud.base.effects_{}", args.len()), &args);
            });
            ui.text_bind_styled(S::default().no_wrap(), |c, t| match vitals_of(c).filter(|v| v.has(effects::Effect::Rested)) {
                Some(v) => t.set("hud.base.effect.rested", &[Arg::Int((v.rested / 60.0).ceil() as i64)]),
                None => t.text(""),
            });
            // Resting by a fire: the seconds until rested; rested: the comfort that sets how long.
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = vitals_of(c) else { return t.text("") };
                if v.resting > 0.0 && v.resting < crate::survival::REST_SECS {
                    t.set("hud.base.resting", &[Arg::Int((crate::survival::REST_SECS - v.resting) as i64)]);
                } else if v.rested > 0.0 {
                    t.set("hud.base.comfort", &[Arg::Int(v.comfort as i64)]);
                } else {
                    t.text("");
                }
            });
        });
    });
    // 07:20, with the sun while it's up and a sleeper at night (a day is 24000 ticks from 06:00;
    // the sun rises earlier in summer, by the calendar), and
    // the day of the season ("Spring 3"; "Wet season 3" in the Cretaceous).
    r.hud_element(&HUD_CLOCK, hud_anchor::TOP_RIGHT, 0, |ui| {
        ui.panel("tooltip", S::column().pad(3.0).gap(1.0).center(), |ui| {
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let ticks = c.day_time() % 24000;
                let minutes = (ticks * 60 / 1000 + 6 * 60) % (24 * 60);
                let clock = format!("{:02}:{:02}", minutes / 60, minutes % 60);
                // The date from the world's own clock, which every client has (the server writes
                // it into the vitals only for players mods see, and spectators aren't: their clock
                // stayed on its old day after `/season`, 11-04); the calendar's kind from the vitals.
                let light = c
                    .component(&VITALS)
                    .filter(|v| v.calendar > 0)
                    .map_or(12.0, |v| {
                        let day = vitals::calendar::date(c.day_time()).day as f32;
                        vitals::calendar::daylight_hours(day, base_api::calendars::swing(v.calendar))
                    });
                let up = (12.0 - light / 2.0) * 60.0..(12.0 + light / 2.0) * 60.0;
                let key = if up.contains(&(minutes as f32)) { "hud.base.clock_day" } else { "hud.base.clock_night" };
                t.set(key, &[Arg::Text(&clock)]);
            });
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = c.component(&VITALS).filter(|v| v.calendar > 0) else { return t.text("") };
                let day = vitals::calendar::date(c.day_time()).day as u32;
                let season = vitals::calendar::SEASONS[(day / vitals::calendar::SEASON_DAYS) as usize % 4];
                let name =
                    if v.calendar == base_api::calendars::WET_AND_DRY { if season.wet() { "wet" } else { "dry" } } else { season.name() };
                t.set(&format!("hud.base.season.{name}"), &[Arg::Int((day % vitals::calendar::SEASON_DAYS + 1) as i64)]);
            });
        });
    });
    // x and z, as on a map.
    r.hud_element(&HUD_POSITION, hud_anchor::TOP_RIGHT, 1, |ui| {
        ui.text_bind_styled(S::default().no_wrap(), |c, t| {
            let Some(me) = c.player_state() else { return };
            t.set("hud.base.position", &[Arg::Int(me.pos.x.floor() as i64), Arg::Int(me.pos.z.floor() as i64)]);
        });
    });
    // A framed picture per screen, its key beside it.
    r.hud_element(&HUD_SHORTCUTS, hud_anchor::TOP_LEFT, 10, |ui| {
        ui.column(S::column().gap(2.0), |ui| {
            for name in SHORTCUTS {
                ui.row(S::row().gap(3.0).center(), |ui| {
                    ui.panel("hotbar", S::row().size(18.0, 18.0).center(), |ui| {
                        ui.image(&format!("base:gui/hud_{name}"), 14.0, 14.0);
                    });
                    ui.text_styled(S::default().no_wrap(), &format!("hud.base.shortcut.{name}"), &[]);
                });
            }
        });
    });
}

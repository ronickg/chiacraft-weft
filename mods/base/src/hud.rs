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
static HUD_STOMACH: HudElement = HudElement::named("food");
static HUD_EFFECTS: HudElement = HudElement::named("effects");
static HUD_TEMPERATURE: HudElement = HudElement::named("temperature");

/// A bar's size in GUI pixels: the UI kit's bar at the item pixel (`pxkit.py`, two image pixels a
/// GUI pixel), drawn at exactly its size so its pixels stay whole. The XP bar is thinner.
const BAR: (f32, f32) = (96.0, 7.0);
const BAR_XP: (f32, f32) = (96.0, 4.0);
/// How far a bar's numbers end from its right end, GUI pixels: its rim (1) and two of padding.
const BAR_TEXT_INSET: f32 = 3.0;
/// The level badge beside the bars (`hud_badge`, 40 x 48 pixels).
const BADGE: (f32, f32) = (20.0, 24.0);

/// Where a bar's fill ends for a share filled: the kit's fill starts 2 of the bar's 192 pixels in
/// and spans 188 (its rim).
fn bar_fill(share: f32) -> f32 {
    (2.0 + share.clamp(0.0, 1.0) * 188.0) / 192.0
}

/// The thermometer's size (the kit's gauge, 128 x 10 pixels), and the body temperatures at its
/// ends, °C.
const THERMOMETER: (f32, f32) = (64.0, 5.0);
const COLDEST: f32 = 31.0;
const HOTTEST: f32 = 43.0;

/// The player's vitals, none in creative or while dead (the bars hide: a dead player's are
/// placeholders, health "0 / -1" over the death screen).
fn vitals_of(c: &ClientView) -> Option<Vitals> {
    if c.creative() || c.me().is_none_or(|m| m.health <= 0.0) {
        return None;
    }
    c.component(&VITALS).filter(|v| v.max_health > 0.0 && v.max_stamina > 0.0)
}

/// The screens a key opens, each with its picture (`gui/hud_<name>.png`) and its key
/// (`hud.base.shortcut.<name>`, as the player has bound it).
/// The effects the HUD names at once (the rest wait their turn).
const EFFECTS_SHOWN: usize = 6;

const SHORTCUTS: [&str; 4] = ["inventory", "recipes", "stats", "advancements"];

pub fn register(r: &mut Registry) {
    // Health, stamina and experience as Ronald's sheet has them: the level on a badge, and beside
    // it the bars, each with its numbers in the vitals' units; the points waiting to be placed (P)
    // after the XP bar.
    r.hud_element(&HUD_HEALTH, hud_anchor::TOP_LEFT, 0, |ui| {
        ui.row(S::row().gap(3.0).center(), |ui| {
            ui.panel("hud_badge", S::row().size(BADGE.0, BADGE.1).center(), |ui| {
                ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                    let Some(v) = vitals_of(c) else { return t.text("") };
                    t.set("hud.base.badge", &[Arg::Int(vitals::levels::level_of(v.xp).0 as i64)]);
                });
            });
            ui.column(S::col().gap(1.0), |ui| {
                // The numbers inside the bar's right end, as on Ronald's sheet.
                ui.row(S::row().size(BAR.0, BAR.1), |ui| {
                    ui.progress_bind("hud_health", S::default().size(BAR.0, BAR.1), |c| {
                        let (Some(v), Some(me)) = (vitals_of(c), c.me()) else { return -1.0 };
                        bar_fill(me.health * HEALTH_SCALE / v.max_health.max(1.0))
                    });
                    ui.text_bind_styled(
                        S::default()
                            .no_wrap()
                            .at_pivot(BAR.0 - BAR_TEXT_INSET, BAR.1 / 2.0, [1.0, 0.5]),
                        |c, t| {
                            let (Some(v), Some(me)) = (vitals_of(c), c.me()) else { return t.text("") };
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
                        bar_fill(v.stamina / v.max_stamina.max(1.0))
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
                ui.row(S::row().gap(3.0).center(), |ui| {
                    ui.progress_bind("hud_xp", S::default().size(BAR_XP.0, BAR_XP.1), |c| {
                        let Some(v) = vitals_of(c) else { return -1.0 };
                        let (level, into) = vitals::levels::level_of(v.xp);
                        bar_fill(into as f32 / vitals::levels::to_next(level) as f32)
                    });
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let Some(v) = vitals_of(c) else { return t.text("") };
                        let unspent = crate::levels::unspent(&v);
                        if unspent > 0 {
                            t.set("hud.base.level_points", &[Arg::Int(unspent as i64)]);
                        } else {
                            t.text("");
                        }
                    });
                });
            });
        });
    });
    // What's in the stomach: each food with the minutes it has left, and how full of each nutrient.
    r.hud_element(&HUD_STOMACH, hud_anchor::TOP_LEFT, 3, |ui| {
        ui.column(S::col().gap(1.0), |ui| {
            ui.row(S::row().gap(2.0).center(), |ui| {
                for i in 0..base_api::PORTIONS {
                    ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                        let Some(v) = vitals_of(c) else { return t.text("") };
                        let Some(f) = (v.foods[i] as usize)
                            .checked_sub(1)
                            .and_then(|f| FOODS.get(f))
                        else {
                            return t.text("");
                        };
                        let Some(item) = Item::lookup(f.name) else { return t.text("") };
                        let minutes = (v.units[i] / f.digest_rate()).ceil() as i64;
                        t.set("hud.base.portion", &[Arg::Item(item), Arg::Int(minutes)]);
                    });
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
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                match vitals_of(c).filter(|v| v.has(effects::Effect::Rested)) {
                    Some(v) => t.set("hud.base.effect.rested", &[Arg::Int((v.rested / 60.0).ceil() as i64)]),
                    None => t.text(""),
                }
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
        ui.panel("tooltip", S::col().pad(3.0).gap(1.0).center(), |ui| {
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let ticks = c.day_time() % 24000;
                let minutes = (ticks * 60 / 1000 + 6 * 60) % (24 * 60);
                let clock = format!("{:02}:{:02}", minutes / 60, minutes % 60);
                let light = c
                    .component(&VITALS)
                    .filter(|v| v.date > 0)
                    .map_or(12.0, |v| vitals::calendar::daylight_hours((v.date - 1) as f32, base_api::calendars::swing(v.calendar)));
                let up = (12.0 - light / 2.0) * 60.0..(12.0 + light / 2.0) * 60.0;
                let key = if up.contains(&(minutes as f32)) { "hud.base.clock_day" } else { "hud.base.clock_night" };
                t.set(key, &[Arg::Text(&clock)]);
            });
            ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                let Some(v) = c.component(&VITALS).filter(|v| v.date > 0) else { return t.text("") };
                let day = (v.date - 1) as u32;
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
            let Some(me) = c.me() else { return };
            t.set("hud.base.position", &[Arg::Int(me.pos[0].floor() as i64), Arg::Int(me.pos[2].floor() as i64)]);
        });
    });
    // A framed picture per screen, its key beside it.
    r.hud_element(&HUD_SHORTCUTS, hud_anchor::TOP_LEFT, 10, |ui| {
        ui.column(S::col().gap(2.0), |ui| {
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

//! DLSS's own Video tab (Ronald, 11-06: DLSS and the rest as settings to turn on and off,
//! "as much freedom as possible", the model choice too): the mode (Off, DLAA, Quality, Balanced,
//! Performance, Ultra Performance), its model for each mode, Ray Reconstruction and its model,
//! DLSS 5 and its strength. Only where the renderer has an upscaler (`core:gpu_dlss*`, which
//! the engine puts at start: DLSS through Streamline on Windows, NGX on Linux later), and only the
//! rows that do something there: the models with a mode on, Ray Reconstruction where it runs, its
//! model with it on, DLSS 5 where NVIDIA's plugin is (not public yet). Changing the mode, Ray
//! Reconstruction or DLSS 5 builds the tab again, so rows come and go. Everything is the engine's
//! `core:dlss*` settings, live. Frame generation gets its rows when the renderer has it
//! (`core:gpu_frame_gen`).

use weft_sdk::*;

pub static SCREEN: Screen = Screen::named("video_dlss");

static NAME: TextConfig = TextConfig::named("core:gpu_dlss");
/// What the upscaler runs, as bits: modes by number, models by their number.
static MODES: Config<i32> = Config::named("core:gpu_dlss_modes");
static MODELS: Config<i32> = Config::named("core:gpu_dlss_models");
static RR_OK: Config<bool> = Config::named("core:gpu_dlss_rr");
static RR_MODELS: Config<i32> = Config::named("core:gpu_dlss_rr_models");
static DLSS5_OK: Config<bool> = Config::named("core:gpu_dlss5");

static MODE: Config<i32> = Config::named("core:dlss");
/// The model per mode, in mode order from DLAA (mode 1).
static MODEL: [Config<i32>; 5] = [
    Config::named("core:dlss_model_dlaa"),
    Config::named("core:dlss_model_quality"),
    Config::named("core:dlss_model_balanced"),
    Config::named("core:dlss_model_performance"),
    Config::named("core:dlss_model_ultra_performance"),
];
static RR: Config<bool> = Config::named("core:dlss_rr");
static RR_MODEL: Config<i32> = Config::named("core:dlss_rr_model");
static DLSS5: Config<bool> = Config::named("core:dlss5");
static DLSS5_INTENSITY: Config<f32> = Config::named("core:dlss5_intensity");

const MODE_NAMES: [&str; 6] = ["off", "dlaa", "quality", "balanced", "performance", "ultra_performance"];
/// DLSS's model numbers and their words (`video.base.value.model_<key>`), M (13) the engine's
/// default for every mode: 0 NVIDIA's pick for the mode, E and F the older CNN models, J and K the first transformer, L and M the second
/// (NVIDIA's DLSS Programming Guide 3.12); the upscaler says which it has.
const MODEL_KEYS: [(i32, &str); 7] = [(0, "default"), (5, "e"), (6, "f"), (10, "j"), (11, "k"), (12, "l"), (13, "m")];
/// Ray Reconstruction's: D, E and F (F the latest, its default).
const RR_MODEL_KEYS: [(i32, &str); 4] = [(0, "default"), (4, "rr_d"), (5, "rr_e"), (6, "rr_f")];
const INTENSITIES: [f32; 4] = [0.25, 0.5, 0.75, 1.0];

/// Whether the renderer has an upscaler at all: the Video screen offers this tab then.
pub fn available() -> bool {
    MODES.get() != 0
}

fn bit(bits: i32, n: i32) -> bool {
    (0..31).contains(&n) && bits & (1 << n) != 0
}

/// The next of `values` that `ok` allows, from `now`, `dir` steps on (round again).
fn cycle(values: &[i32], now: i32, dir: i32, ok: impl Fn(i32) -> bool) -> i32 {
    let allowed: Vec<i32> = values.iter().copied().filter(|&v| ok(v)).collect();
    if allowed.is_empty() {
        return now;
    }
    let at = allowed.iter().position(|&v| v == now).unwrap_or(0) as i32;
    allowed[(at + dir).rem_euclid(allowed.len() as i32) as usize]
}

fn mode() -> i32 {
    MODE.get().clamp(0, 5)
}

fn model_word(keys: &[(i32, &'static str)], id: i32) -> (String, i64) {
    match keys.iter().find(|k| k.0 == id) {
        Some((_, key)) => (format!("model_{key}"), 0),
        None => (String::new(), id as i64),
    }
}

/// A row: its name, its value (a word, or a number with an empty word), the arrows step it and
/// with `rebuild` open the tab again (rows that come and go with it), and what it does.
fn row(ui: &mut Ui, key: &'static str, shown: fn() -> (String, i64), step: fn(i32), rebuild: bool) {
    row_costed(ui, key, shown, step, rebuild, None);
}

/// `row`, with what its value costs here (`video_costs`) from `value` (the setting as a number).
fn row_costed(ui: &mut Ui, key: &'static str, shown: fn() -> (String, i64), step: fn(i32), rebuild: bool, value: Option<fn() -> f64>) {
    ui.row(S::row().gap(3.0).center(), move |ui| {
        ui.text_styled(S::default().w(84.0).no_wrap(), &format!("video.base.{key}"), &[]);
        ui.button("video.base.less", S::default().size(12.0, 12.0).pad(0.5), move |c| {
            step(-1);
            if rebuild {
                c.client().screen_open(&SCREEN);
            }
        });
        ui.row(S::row().w(96.0).center(), move |ui| {
            ui.text_bind_styled(S::default().no_wrap(), move |_, t| {
                let (k, n) = shown();
                if k.is_empty() {
                    t.set("video.base.value.number", &[Arg::Int(n)]);
                } else {
                    t.set(&format!("video.base.value.{k}"), &[Arg::Int(n)]);
                }
            });
        });
        ui.button("video.base.more", S::default().size(12.0, 12.0).pad(0.5), move |c| {
            step(1);
            if rebuild {
                c.client().screen_open(&SCREEN);
            }
        });
        match value {
            Some(get) => crate::video_costs::cost_cell(ui, key, get),
            None => ui.row(S::row().w(40.0), |_| {}),
        }
        ui.text_styled(S::default().class("dim").max_w(150.0), &format!("video.base.{key}.what"), &[]);
    });
}

fn mode_row(ui: &mut Ui) {
    row_costed(
        ui,
        "dlss",
        || (MODE_NAMES[mode() as usize].to_string(), 0),
        |dir| MODE.set(cycle(&[0, 1, 2, 3, 4, 5], mode(), dir, |m| m == 0 || bit(MODES.get(), m))),
        true,
        Some(|| mode() as f64),
    );
}

/// The rows the DLSS tab's measuring steps through (`video_costs`): the modes the upscaler offers,
/// and Ray Reconstruction where it runs; none without an upscaler.
pub(crate) fn measured_rows() -> Vec<crate::video_costs::Measured> {
    if !available() {
        return Vec::new();
    }
    let mut rows = vec![crate::video_costs::Measured {
        key: "dlss",
        values: (0..6)
            .filter(|&m| m == 0 || bit(MODES.get(), m))
            .map(|m| m as f64)
            .collect(),
        get: || mode() as f64,
        set: |v| MODE.set(v as i32),
        // A new render size for the world each change.
        settle: 1.0,
    }];
    if RR_OK.get() {
        rows.push(crate::video_costs::Measured {
            key: "dlss_rr",
            values: vec![0.0, 1.0],
            get: || RR.get() as u8 as f64,
            set: |v| RR.set(v > 0.0),
            settle: 1.0,
        });
    }
    rows
}

fn model_row(ui: &mut Ui) {
    row(
        ui,
        "dlss_model",
        || model_word(&MODEL_KEYS, MODEL[(mode() - 1).max(0) as usize].get()),
        |dir| {
            let c = &MODEL[(mode() - 1).max(0) as usize];
            let ids: Vec<i32> = MODEL_KEYS.iter().map(|k| k.0).collect();
            c.set(cycle(&ids, c.get(), dir, |id| bit(MODELS.get(), id)));
        },
        false,
    );
}

fn rr_row(ui: &mut Ui) {
    row_costed(
        ui,
        "dlss_rr",
        || (if RR.get() { "on" } else { "off" }.into(), 0),
        |_| RR.set(!RR.get()),
        true,
        Some(|| RR.get() as u8 as f64),
    );
}

fn rr_model_row(ui: &mut Ui) {
    row(
        ui,
        "dlss_rr_model",
        || model_word(&RR_MODEL_KEYS, RR_MODEL.get()),
        |dir| {
            let ids: Vec<i32> = RR_MODEL_KEYS.iter().map(|k| k.0).collect();
            RR_MODEL.set(cycle(&ids, RR_MODEL.get(), dir, |id| bit(RR_MODELS.get(), id)));
        },
        false,
    );
}

fn dlss5_rows(ui: &mut Ui) {
    row(ui, "dlss5", || (if DLSS5.get() { "on" } else { "off" }.into(), 0), |_| DLSS5.set(!DLSS5.get()), true);
    if DLSS5.get() {
        row(
            ui,
            "dlss5_intensity",
            || ("percent".into(), (DLSS5_INTENSITY.get() * 100.0).round() as i64),
            |dir| {
                let at = INTENSITIES
                    .iter()
                    .position(|&v| (v - DLSS5_INTENSITY.get()).abs() < 0.01)
                    .unwrap_or(1) as i32;
                DLSS5_INTENSITY.set(INTENSITIES[(at + dir).rem_euclid(INTENSITIES.len() as i32) as usize]);
            },
            false,
        );
    }
}

pub fn register(r: &mut Registry) {
    r.screen(&SCREEN, |ui| {
        let tabs: [crate::window::Tab; 3] = [
            ("video.base.title", |c| {
                c.client().screen_open(&crate::video::SCREEN);
            }),
            ("video.base.fantasy.title", |c| {
                c.client().screen_open(&crate::video::FANTASY_SCREEN);
            }),
            ("video.base.extra.title", |c| {
                c.client().screen_open(&crate::video::EXTRA_SCREEN);
            }),
        ];
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.dlss.title", &tabs, |ui| {
            crate::video_costs::top_line(ui, &SCREEN, measured_rows, |_| None);
            ui.row(S::row().gap(3.0), |ui| {
                ui.row(S::row().w(84.0), |_| {});
                ui.text_bind_styled(S::default().class("dim").max_w(260.0), |_, t| {
                    t.set("video.base.dlss.name", &[Arg::Text(&NAME.get())]);
                });
            });
            mode_row(ui);
            if mode() != 0 {
                if MODELS.get() != 0 {
                    model_row(ui);
                }
                if RR_OK.get() {
                    rr_row(ui);
                    if RR.get() && RR_MODELS.get() != 0 {
                        rr_model_row(ui);
                    }
                }
            }
            if DLSS5_OK.get() {
                dlss5_rows(ui);
            }
        });
    });
}

//! Video and quality (Ronald: the game lags on his MacBook, and the look had no quality
//! settings): one screen with four presets, Low, Medium, High and Ultra, and every setting they
//! set on a row of its own (‹ value ›), as on Ronald's UI sheet's settings. Changing a row makes
//! the quality Custom. The first time the game runs, the preset is picked from the screen: a
//! high-density one (a Retina MacBook) starts at Medium, the rest at High.
//!
//! What each setting does is its owner's: the engine's view and distant-terrain distances, vsync
//! and fps cap; `core:render_scale` (the world drawn at a share of the pixels, then scaled up; 0
//! Auto, Distant terrain's),
//! `core:shadows` and `core:shadow_size` as the engine reads them (Distant terrain and Lighting
//! add them); the pack's screen effects (`shaderpack.rs`), each off, low or on.
//!
//! Render scale and shadow detail apply at the next start, never mid-game (Planner, 11-05: on the
//! PC, render scale 1.0 to 0.5 in a world lost the GPU 5 s later, `NVRM Xid 109`, and a GPU hang
//! can take the Mac down): both make the engine destroy and remake render targets while frames
//! are in flight. Their rows set `render_scale_next` / `shadow_size_next`, which go to the
//! engine's settings only while no world is open (the title screen, at the next start).

use weft_sdk::*;

/// The quality chosen: 0 low .. 3 ultra, 4 custom, -1 not chosen yet (picked from the machine).
pub static QUALITY: Config<i32> = Config::named("quality");
/// The pack's screen effects: 0 off, 1 low (fewer steps), 2 on. Bloom has no low.
pub static FX_REFLECT: Config<i32> = Config::named("fx_reflect");
pub static FX_CONTACT: Config<i32> = Config::named("fx_contact");
/// Light in the air through the shadow maps (Lighting's `volume`, on `look-air`): until that lands,
/// the engine logs "no such effect" and the row changes nothing.
pub static FX_VOLUME: Config<i32> = Config::named("fx_volume");
pub static FX_SHAFTS: Config<i32> = Config::named("fx_shafts");
pub static FX_BLOOM: Config<i32> = Config::named("fx_bloom");

static RENDER_SCALE: Config<f32> = Config::named("core:render_scale");
static SHADOWS: Config<i32> = Config::named("core:shadows");
static SHADOW_SIZE: Config<i32> = Config::named("core:shadow_size");
static VIEW: Config<i32> = Config::named("core:view_distance");
static LOD: Config<i32> = Config::named("core:lod_distance");
static VSYNC: Config<bool> = Config::named("core:vsync");
static FPS_CAP: Config<i32> = Config::named("core:fps_cap");
/// What render scale and shadow detail will be at the next start (-1 and 0: as they are now).
static RENDER_SCALE_NEXT: Config<f32> = Config::named("render_scale_next");
static SHADOW_SIZE_NEXT: Config<i32> = Config::named("shadow_size_next");

/// Render scale as it will be: the one waiting for the next start, else the engine's.
fn render_scale() -> f64 {
    let next = RENDER_SCALE_NEXT.get();
    if next >= 0.0 { next as f64 } else { RENDER_SCALE.get() as f64 }
}

fn shadow_size() -> f64 {
    let next = SHADOW_SIZE_NEXT.get();
    if next > 0 { next as f64 } else { SHADOW_SIZE.get() as f64 }
}

/// A row whose value waits for the next start, and whether it waits now.
fn waiting(key: &str) -> bool {
    match key {
        "render_scale" => (render_scale() - RENDER_SCALE.get() as f64).abs() > 1e-3,
        "shadow_size" => (shadow_size() - SHADOW_SIZE.get() as f64).abs() > 0.5,
        _ => false,
    }
}

/// The values waiting go to the engine, while no world is open (render targets made afresh then,
/// with no frame of a world in flight).
fn take_waiting(c: &Client) {
    if c.state().flags & abi::client_flags::IN_WORLD != 0 {
        return;
    }
    let scale = RENDER_SCALE_NEXT.get();
    if scale >= 0.0 {
        if (scale - RENDER_SCALE.get()).abs() > 1e-3 {
            weft_sdk::info!("video: render scale {scale} from the last game");
            RENDER_SCALE.set(scale);
        }
        RENDER_SCALE_NEXT.set(-1.0);
    }
    let size = SHADOW_SIZE_NEXT.get();
    if size > 0 {
        if size != SHADOW_SIZE.get() {
            weft_sdk::info!("video: shadow detail {size} from the last game");
            SHADOW_SIZE.set(size);
        }
        SHADOW_SIZE_NEXT.set(0);
    }
}

pub static SCREEN: Screen = Screen::named("video");

/// A setting's row: its name (`video.base.<key>`), the values it steps through, and how each is
/// shown (`video.base.value.<shown>` or a number).
struct Row {
    key: &'static str,
    values: &'static [f64],
    get: fn() -> f64,
    set: fn(f64),
    shown: fn(f64) -> (String, i64),
}

/// The presets, Low to Ultra, per row (in `ROWS` order).
const PRESETS: [[f64; 12]; 4] = [
    // scale shadows size  reflect contact volume shafts bloom view lod   vsync cap
    [0.5, 0.0, 1024.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.0, 64.0, 1.0, 60.0],
    [0.67, 1.0, 1024.0, 1.0, 1.0, 1.0, 1.0, 0.0, 8.0, 128.0, 1.0, 60.0],
    [0.0, 2.0, 2048.0, 2.0, 2.0, 2.0, 2.0, 1.0, 12.0, 192.0, 1.0, 0.0],
    [1.0, 2.0, 4096.0, 2.0, 2.0, 2.0, 2.0, 1.0, 16.0, 256.0, 1.0, 0.0],
];
const PRESET_NAMES: [&str; 4] = ["low", "medium", "high", "ultra"];

fn word(k: &str) -> (String, i64) {
    (k.to_string(), 0)
}

fn number(v: f64) -> (String, i64) {
    (String::new(), v.round() as i64)
}

const ROWS: [Row; 12] = [
    Row {
        key: "render_scale",
        // 0 is Auto (Distant terrain's render pass: 0.67 on a high-density screen, 1 elsewhere).
        values: &[0.0, 0.5, 0.6, 0.67, 0.75, 0.85, 1.0],
        get: render_scale,
        set: |v| RENDER_SCALE_NEXT.set(v as f32),
        shown: |v| if v == 0.0 { word("auto") } else { ("percent".into(), (v * 100.0).round() as i64) },
    },
    Row {
        key: "shadows",
        values: &[0.0, 1.0, 2.0],
        get: || SHADOWS.get() as f64,
        set: |v| SHADOWS.set(v as i32),
        shown: |v| word(["off", "near", "near_far"][v as usize % 3]),
    },
    Row { key: "shadow_size", values: &[1024.0, 2048.0, 4096.0], get: shadow_size, set: |v| SHADOW_SIZE_NEXT.set(v as i32), shown: number },
    Row { key: "reflect", values: &[0.0, 1.0, 2.0], get: || FX_REFLECT.get() as f64, set: |v| FX_REFLECT.set(v as i32), shown: off_low_on },
    Row { key: "contact", values: &[0.0, 1.0, 2.0], get: || FX_CONTACT.get() as f64, set: |v| FX_CONTACT.set(v as i32), shown: off_low_on },
    Row { key: "volume", values: &[0.0, 1.0, 2.0], get: || FX_VOLUME.get() as f64, set: |v| FX_VOLUME.set(v as i32), shown: off_low_on },
    Row { key: "shafts", values: &[0.0, 1.0, 2.0], get: || FX_SHAFTS.get() as f64, set: |v| FX_SHAFTS.set(v as i32), shown: off_low_on },
    Row {
        key: "bloom",
        values: &[0.0, 1.0],
        get: || FX_BLOOM.get() as f64,
        set: |v| FX_BLOOM.set(v as i32),
        shown: |v| word(if v > 0.0 { "on" } else { "off" }),
    },
    Row {
        key: "view",
        values: &[4.0, 6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 24.0, 32.0],
        get: || VIEW.get() as f64,
        set: |v| VIEW.set(v as i32),
        shown: number,
    },
    Row {
        key: "lod",
        values: &[0.0, 64.0, 128.0, 192.0, 256.0, 512.0],
        get: || LOD.get() as f64,
        set: |v| LOD.set(v as i32),
        shown: |v| if v == 0.0 { word("off") } else { number(v) },
    },
    Row {
        key: "vsync",
        values: &[0.0, 1.0],
        get: || VSYNC.get() as u8 as f64,
        set: |v| VSYNC.set(v > 0.0),
        shown: |v| word(if v > 0.0 { "on" } else { "off" }),
    },
    Row {
        key: "fps_cap",
        values: &[0.0, 30.0, 60.0, 90.0, 120.0, 144.0],
        get: || FPS_CAP.get() as f64,
        set: |v| FPS_CAP.set(v as i32),
        shown: |v| if v == 0.0 { word("unlimited") } else { number(v) },
    },
];

fn off_low_on(v: f64) -> (String, i64) {
    word(["off", "low", "on"][(v as usize).min(2)])
}

/// Sets every row to preset `p` (0 low .. 3 ultra).
fn apply(p: usize) {
    for (row, &v) in ROWS.iter().zip(PRESETS[p].iter()) {
        (row.set)(v);
    }
    QUALITY.set(p as i32);
}

/// One row a step on (`dir` 1) or back (-1) through its values; the quality becomes Custom.
fn step(i: usize, dir: i32) {
    let row = &ROWS[i];
    let now = (row.get)();
    let at = row
        .values
        .iter()
        .position(|&v| (v - now).abs() < 1e-3)
        .unwrap_or_else(|| {
            row.values
                .iter()
                .position(|&v| v > now)
                .unwrap_or(row.values.len() - 1)
        });
    let next = (at as i32 + dir).rem_euclid(row.values.len() as i32) as usize;
    (row.set)(row.values[next]);
    QUALITY.set(4);
}

pub fn register(r: &mut Registry) {
    r.config(&QUALITY).int(-1, -1, 4);
    r.config(&FX_REFLECT).int(2, 0, 2);
    r.config(&FX_CONTACT).int(2, 0, 2);
    // Cheap (Tester 2's PC bench, 11-04: +0.04 ms of effects at 1280x720): low until a preset
    // says otherwise; off on the Low preset until the Mac's Retina numbers are in.
    r.config(&FX_VOLUME).int(1, 0, 2);
    r.config(&FX_SHAFTS).int(2, 0, 2);
    r.config(&FX_BLOOM).int(1, 0, 1);
    r.config(&RENDER_SCALE_NEXT).float(-1.0, -1.0, 1.0);
    r.config(&SHADOW_SIZE_NEXT).int(0, 0, 4096);
    // The first run: a preset from the screen (a high-density one draws many more pixels).
    r.client_frame(|c| {
        if QUALITY.get() >= 0 {
            return;
        }
        // Scripts, tests and benchmarks keep their own settings (`--config` sets them per run).
        if ["script", "test", "config"]
            .iter()
            .any(|o| weft_sdk::client::launch_option(o).is_some())
        {
            return;
        }
        let [w, h] = c.state().window;
        if w == 0 || h == 0 {
            return;
        }
        let pixels = w as u64 * h as u64;
        let p = if pixels >= 5_000_000 { 1 } else { 2 };
        weft_sdk::info!("video: first run, {w}x{h}: the {} preset", PRESET_NAMES[p]);
        apply(p);
    });
    r.client_frame(take_waiting);
    r.screen(&SCREEN, |ui| {
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.title", &[], |ui| {
            // The presets, and which one the settings are now.
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().w(84.0).no_wrap(), "video.base.quality", &[]);
                for (p, name) in PRESET_NAMES.iter().enumerate() {
                    ui.button(&format!("video.base.preset.{name}"), S::default().w(52.0), move |_| apply(p));
                }
            });
            ui.row(S::row().gap(3.0), |ui| {
                ui.row(S::row().w(84.0), |_| {});
                ui.text_bind_styled(S::default().class("dim").no_wrap(), |_, t| {
                    let q = QUALITY.get();
                    let name = PRESET_NAMES.get(q as usize).copied().unwrap_or("custom");
                    t.set("video.base.now", &[Arg::Key(&format!("video.base.preset.{name}"))]);
                });
            });
            for (i, row) in ROWS.iter().enumerate() {
                ui.row(S::row().gap(3.0).center(), move |ui| {
                    ui.text_styled(S::default().w(84.0).no_wrap(), &format!("video.base.{}", row.key), &[]);
                    ui.button("video.base.less", S::default().size(12.0, 12.0).pad(0.5), move |_| step(i, -1));
                    ui.row(S::row().w(96.0).center(), move |ui| {
                        ui.text_bind_styled(S::default().no_wrap(), move |_, t| {
                            let (k, n) = (ROWS[i].shown)((ROWS[i].get)());
                            if k.is_empty() {
                                t.set("video.base.value.number", &[Arg::Int(n)]);
                            } else {
                                t.set(&format!("video.base.value.{k}"), &[Arg::Int(n)]);
                            }
                        });
                    });
                    ui.button("video.base.more", S::default().size(12.0, 12.0).pad(0.5), move |_| step(i, 1));
                    // A value waiting for the next start says so in place of what the row does.
                    let key = row.key;
                    ui.text_bind_styled(S::default().class("dim").max_w(150.0), move |_, t| {
                        if waiting(key) {
                            t.set("video.base.after_restart", &[]);
                        } else {
                            t.set(&format!("video.base.{key}.what"), &[]);
                        }
                    });
                });
            }
        });
    });
    // On the settings screen, beside the engine's own.
    r.screen_part(&screens::SETTINGS, "buttons", 0, |ui| {
        ui.button("video.base.open", S::column().min_w(200.0).self_align(abi::ui_align::CENTER), |c| {
            c.client().screen_open(&SCREEN);
        });
    });
}

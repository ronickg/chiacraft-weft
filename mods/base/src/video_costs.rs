//! What each quality setting costs on this machine, on its own row of the Video tabs (Ronald,
//! 11-06: "attach an estimated FPS or how much it costs, for each setting, because that often
//! gives you a better indication of like what to turn on and off"; his picks: the measuring offered
//! once, each cost against the row's cheapest value).
//!
//! - A top line on every tab: the GPU's milliseconds a frame, the frames a second and the whole
//!   frame's milliseconds, live (the engine's GPU timestamps, `ClientState::gpu_ms`), and what a
//!   change waiting for a restart will cost.
//! - On each row, what its value costs over the cheapest: measured on this GPU (plain), or estimated
//!   from the table (`≈`); amber past 1 ms or a tenth of the frame.
//! - Measuring (`Measure costs`, or `/videocosts all` for a script): with the world behind the
//!   menu, each row that applies at once steps through its values, settles, and the GPU time is
//!   averaged; then it goes back. The costs are kept per GPU, picture size and view distance
//!   (`video_costs`), so another machine or window measures again. Rows read at start (the HDR
//!   world, anti-aliasing, render scale, shadow detail) keep the table's estimate.
//! - The table (`TABLE`): our own sweep on the PC's RTX 5090 at 1920x1080 and the Pixel 10 (Android),
//!   scaled to this machine by the pixels drawn and the kind of GPU.

use std::sync::Mutex;

use weft_sdk::*;

/// A row as the sweep measures it: its key (`video.base.<key>`), its values, how to read and set it,
/// and how long it takes to settle after a change (seconds: new shaders, shadow maps, particles).
pub struct Measured {
    pub key: &'static str,
    pub values: Vec<f64>,
    pub get: fn() -> f64,
    pub set: fn(f64),
    pub settle: f32,
}

/// Which tab's rows a sweep measures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Video,
    Extra,
    Fantasy,
    Dlss,
}

/// What was measured, and where: `<signature>##key=value:ms;...`.
static COSTS: TextConfig = TextConfig::named("video_costs");
/// The measuring was offered once (Ronald: "Offer once").
static OFFERED: Config<bool> = Config::named("video_costs_offered");
static GPU_NAME: TextConfig = TextConfig::named("core:gpu_name");
static GPU_CLASS: Config<i32> = Config::named("core:gpu_class");
static VIEW: Config<i32> = Config::named("core:view_distance");
static RENDER_SCALE: Config<f32> = Config::named("core:render_scale");

/// The cost of a row's value over its cheapest at 1920x1080 on the PC's RTX 5090 (ms; Max preset
/// around it, the base game's PC sweep `/videocosts measure` and the benches of 11-06), and on the
/// Pixel 10 at the Phone preset (ms, Android's numbers; NaN: not measured there, scaled from the PC).
/// Rows read at start come from paired runs.
const TABLE: &[(&str, f64, f64, f64)] = &[
    // key, value, PC ms, Pixel ms. The PC's from the sweep at mc_forest noon, view 32, the rest of
    // Max around it (tester-tools/temporal-aa/costs_sweep4.txt, 11-06: ±0.02 ms a value); rows it
    // found nothing for there (mist, fireflies by night, contact shadows under trees) cost where
    // they show, which a player's own measuring finds.
    ("shadows", 1.0, 0.28, f64::NAN),
    ("shadows", 2.0, 0.49, f64::NAN),
    ("reflect", 1.0, 0.07, f64::NAN),
    ("reflect", 2.0, 0.06, f64::NAN),
    ("volume", 1.0, 0.07, f64::NAN),
    ("volume", 2.0, 0.20, f64::NAN),
    ("rt_shadows", 1.0, 4.08, f64::NAN),
    ("rt_shadows", 2.0, 10.98, f64::NAN),
    ("fireflies", 1.0, 0.07, f64::NAN),
    ("clouds", 1.0, 0.07, f64::NAN),
    // From paired benches (11-06): TAA +0.07-0.10 ms on the PC; on the Pixel the motion image cost
    // its world pass 9 ms and the resolve 2.4 (Android).
    ("aa", 1.0, 0.09, 11.4),
    ("aa", 2.0, 0.07, 2.4),
    // Look's HDR numbers (+0.1-0.24 ms at 3456 wide), an estimate at 1080p.
    ("hdr", 1.0, 0.15, f64::NAN),
];

/// The 5090 at 1920x1080 against this machine: the pixels the world is drawn at, and how much
/// slower the GPU is likely to be (a rough guess by its kind: the estimates say `~`).
fn scale(window: [u32; 2]) -> f64 {
    let s = RENDER_SCALE.get();
    let s = if s > 0.0 { s as f64 } else { 1.0 };
    let pixels = window[0] as f64 * window[1] as f64 * s * s / (1920.0 * 1080.0);
    let name = GPU_NAME.get().to_lowercase();
    let slower = if name.contains("5090") || name.contains("4090") {
        1.0
    } else if name.contains("apple") {
        if name.contains("max") || name.contains("ultra") {
            3.0
        } else if name.contains("pro") {
            5.0
        } else {
            10.0
        }
    } else {
        match GPU_CLASS.get() {
            1 => 15.0,
            2 => 3.0,
            3 => 30.0,
            _ => 4.0,
        }
    };
    pixels.max(0.05) * slower
}

fn on_phone() -> bool {
    cfg!(target_os = "android") || weft_sdk::client::launch_option("fake-gpu").as_deref() == Some("phone")
}

/// Where the measured costs hold: the GPU, the window's size, the render scale and the view.
fn signature(window: [u32; 2]) -> String {
    format!("{}|{}x{}|s{:.2}|v{}", GPU_NAME.get(), window[0], window[1], RENDER_SCALE.get(), VIEW.get())
}

/// A cost on a row.
#[derive(Clone, Copy, Debug)]
pub enum Cost {
    Measured(f64),
    Estimate(f64),
    None,
}

/// Between the signature (which holds `|`) and the costs.
const SEP: &str = "##";

/// The measured costs, parsed once per change of the setting: (the setting, its signature, and
/// key, value, ms).
type Parsed = (String, String, Vec<(String, f64, f64)>);
static PARSED: Mutex<Parsed> = Mutex::new((String::new(), String::new(), Vec::new()));

fn measured(window: [u32; 2], key: &str, value: f64) -> Option<f64> {
    let raw = COSTS.get();
    let mut p = PARSED.lock().unwrap();
    if p.0 != raw {
        let (sig, rest) = raw.split_once(SEP).unwrap_or(("", ""));
        let costs = rest
            .split(';')
            .filter_map(|e| {
                let (k, ms) = e.split_once(':')?;
                let (key, v) = k.split_once('=')?;
                Some((key.to_string(), v.parse().ok()?, ms.parse().ok()?))
            })
            .collect();
        *p = (raw.clone(), sig.to_string(), costs);
    }
    if p.1 != signature(window) {
        return None;
    }
    p.2.iter()
        .find(|c| c.0 == key && (c.1 - value).abs() < 1e-6)
        .map(|c| c.2)
}

/// The table's estimate for this machine.
fn estimate(window: [u32; 2], key: &str, value: f64) -> Option<f64> {
    let (_, _, pc, phone) = TABLE
        .iter()
        .find(|t| t.0 == key && (t.1 - value).abs() < 1e-6)?;
    if on_phone() && !phone.is_nan() {
        return Some(*phone);
    }
    Some(pc * scale(window))
}

/// What `key` at `value` costs over its cheapest here: measured, else estimated.
pub fn cost(window: [u32; 2], key: &str, value: f64) -> Cost {
    if let Some(ms) = measured(window, key, value) {
        return Cost::Measured(ms);
    }
    match estimate(window, key, value) {
        Some(ms) => Cost::Estimate(ms),
        None => Cost::None,
    }
}

/// The frame as it goes, smoothed over about a second: GPU ms, whole frame ms, fps.
static LIVE: Mutex<[f32; 3]> = Mutex::new([0.0; 3]);

/// The window's size as the last frame had it (a screen's bindings see none).
static WINDOW: Mutex<[u32; 2]> = Mutex::new([0, 0]);

fn window() -> [u32; 2] {
    *WINDOW.lock().unwrap()
}

/// The GPU's ms a frame now, smoothed.
pub fn live_gpu() -> f64 {
    LIVE.lock().unwrap()[0] as f64
}

/// A sweep in progress: its rows, the (row, value) steps, the step it is at, how long it has
/// settled or measured, the sum of GPU ms measured, what each step gave, and each row's value
/// before it.
struct Sweep {
    rows: Vec<Measured>,
    steps: Vec<(usize, f64)>,
    at: usize,
    measuring: bool,
    secs: f32,
    frames: u32,
    sum: f64,
    sum2: f64,
    /// Each step's mean GPU ms and its standard error.
    results: Vec<(f64, f64)>,
    before: Vec<f64>,
    window: [u32; 2],
}

static SWEEP: Mutex<Option<Sweep>> = Mutex::new(None);

/// What a sweep of `rows` takes, seconds.
pub fn sweep_secs(rows: &[Measured]) -> f32 {
    rows.iter()
        .map(|r| r.values.len() as f32 * (r.settle + MEASURE_SECS))
        .sum()
}

/// How long each value is measured: long enough that a few tenths of a millisecond stand out of a
/// frame's own spread (the PC's sweep: ±0.4 ms between frames under traced shadows).
const MEASURE_SECS: f32 = 0.8;

/// Starts measuring `rows` (none while one runs).
pub fn start(rows: Vec<Measured>, window: [u32; 2]) {
    let mut s = SWEEP.lock().unwrap();
    if s.is_some() || rows.is_empty() {
        return;
    }
    let steps = rows
        .iter()
        .enumerate()
        .flat_map(|(i, r)| r.values.iter().map(move |&v| (i, v)))
        .collect();
    let before = rows.iter().map(|r| (r.get)()).collect();
    weft_sdk::info!("video costs: measuring {} rows", rows.len());
    *s = Some(Sweep {
        rows,
        steps,
        at: 0,
        measuring: false,
        secs: 0.0,
        frames: 0,
        sum: 0.0,
        sum2: 0.0,
        results: Vec::new(),
        before,
        window,
    });
    OFFERED.set(true);
}

/// Measuring now: (step, steps).
pub fn progress() -> Option<(usize, usize)> {
    SWEEP
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| (s.at, s.steps.len()))
}

/// Each frame: the live numbers, and a sweep's next bit.
fn frame(c: &Client) {
    let st = c.state();
    *WINDOW.lock().unwrap() = st.window;
    let dt = st.dt.clamp(0.0, 0.5);
    {
        let mut l = LIVE.lock().unwrap();
        let k = (dt * 2.0).min(1.0);
        let now = [st.gpu_ms, st.frame_ms, st.fps];
        for i in 0..3 {
            l[i] = if l[i] == 0.0 { now[i] } else { l[i] + (now[i] - l[i]) * k };
        }
    }
    let mut guard = SWEEP.lock().unwrap();
    let Some(s) = guard.as_mut() else { return };
    if s.at >= s.steps.len() {
        finish(s);
        *guard = None;
        return;
    }
    let (row, value) = s.steps[s.at];
    if !s.measuring && s.secs == 0.0 && s.frames == 0 {
        // A new row: the one before back as it was.
        if s.at > 0 && s.steps[s.at - 1].0 != row {
            let prev = s.steps[s.at - 1].0;
            (s.rows[prev].set)(s.before[prev]);
        }
        (s.rows[row].set)(value);
    }
    s.secs += dt;
    s.frames += 1;
    if !s.measuring {
        if s.secs >= s.rows[row].settle && s.frames >= 10 {
            (s.measuring, s.secs, s.frames, s.sum, s.sum2) = (true, 0.0, 0, 0.0, 0.0);
        }
        return;
    }
    s.sum += st.gpu_ms as f64;
    s.sum2 += (st.gpu_ms as f64).powi(2);
    if s.secs >= MEASURE_SECS && s.frames >= 30 {
        let n = s.frames as f64;
        let mean = s.sum / n;
        let var = (s.sum2 / n - mean * mean).max(0.0);
        s.results.push((mean, (var / n).sqrt()));
        (s.measuring, s.secs, s.frames, s.sum, s.sum2) = (false, 0.0, 0, 0.0, 0.0);
        s.at += 1;
    }
}

/// The sweep's costs kept (each over its row's first value), and every row as it was.
fn finish(s: &mut Sweep) {
    for (i, r) in s.rows.iter().enumerate() {
        (r.set)(s.before[i]);
    }
    let sig = signature(s.window);
    // What is kept already for this signature, the sweep's rows replaced.
    let keys: Vec<&str> = s.rows.iter().map(|r| r.key).collect();
    let raw = COSTS.get();
    let mut kept: Vec<String> = Vec::new();
    if let Some(rest) = raw.strip_prefix(&format!("{sig}{SEP}")) {
        kept.extend(
            rest.split(';')
                .filter(|e| !e.is_empty())
                .filter(|e| e.split_once('=').is_none_or(|(k, _)| !keys.contains(&k)))
                .map(str::to_string),
        );
    }
    for (i, r) in s.rows.iter().enumerate() {
        let got: Vec<(f64, (f64, f64))> = s
            .steps
            .iter()
            .zip(&s.results)
            .filter(|(st, _)| st.0 == i)
            .map(|(st, m)| (st.1, *m))
            .collect();
        // Against the row's first value, its Off or least (Ronald: "Against Off").
        let Some(&(_, (base, base_se))) = got.first() else { continue };
        for (v, (ms, se)) in got {
            // Within twice the two means' error it is the frame's own spread, not a cost.
            let d = ms - base;
            let noise = 2.0 * (se * se + base_se * base_se).sqrt();
            let cost = if d > noise { d } else { 0.0 };
            weft_sdk::info!("video cost: {} {} {:.3} ms (frame {:.3} ms ±{:.3})", r.key, v, cost, ms, se);
            kept.push(format!("{}={}:{:.3}", r.key, v, cost));
        }
    }
    COSTS.set(&format!("{sig}{SEP}{}", kept.join(";")));
    weft_sdk::info!("video costs: done ({sig})");
}

/// A row's cost, between its value and what it does.
pub fn cost_cell(ui: &mut Ui, key: &'static str, get: fn() -> f64) {
    ui.text_bind_styled(S::default().w(40.0).no_wrap(), move |_, t| {
        let window = window();
        let frame = LIVE.lock().unwrap()[0].max(0.1) as f64;
        let (ms, measured) = match cost(window, key, get()) {
            Cost::Measured(ms) => (ms, true),
            Cost::Estimate(ms) => (ms, false),
            Cost::None => {
                t.set("video.base.cost.none", &[]);
                return;
            }
        };
        if ms < 0.005 {
            t.set("video.base.cost.none", &[]);
            return;
        }
        let high = ms >= 1.0 || ms > frame * 0.1;
        let shown = if ms >= 10.0 {
            format!("{ms:.0}")
        } else if ms >= 1.0 {
            format!("{ms:.1}")
        } else {
            format!("{ms:.2}")
        };
        let key = match (measured, high) {
            (true, false) => "video.base.cost.measured",
            (true, true) => "video.base.cost.high",
            (false, false) => "video.base.cost.estimate",
            (false, true) => "video.base.cost.high_estimate",
        };
        t.set(key, &[Arg::Text(&shown)]);
    });
}

/// The top of a tab: the frame now (and after a restart, `after`: ms more or less than now, an
/// estimate), the Measure button for `rows`, and once, in a world, the offer to measure.
pub fn top_line(ui: &mut Ui, screen: &'static Screen, rows: fn() -> Vec<Measured>, after: fn([u32; 2]) -> Option<f64>) {
    ui.row(S::row().gap(3.0).center(), move |ui| {
        ui.text_bind_styled(S::default().max_w(250.0), move |_, t| {
            let l = *LIVE.lock().unwrap();
            let gpu = format!("{:.2}", l[0]);
            let fps = format!("{:.0}", l[2]);
            let frame = format!("{:.1}", l[1]);
            match after(window()) {
                Some(d) if d.abs() >= 0.01 => {
                    let then = format!("{:.2}", (l[0] as f64 + d).max(0.0));
                    t.set("video.base.cost.now_after", &[Arg::Text(&gpu), Arg::Text(&fps), Arg::Text(&frame), Arg::Text(&then)]);
                }
                _ => t.set("video.base.cost.now", &[Arg::Text(&gpu), Arg::Text(&fps), Arg::Text(&frame)]),
            }
        });
        ui.button("video.base.cost.measure", S::default().h(12.0).pad(2.0), move |_| {
            start(rows(), window());
        });
        ui.text_bind_styled(S::default().class("dim").max_w(120.0), move |_, t| match progress() {
            Some((at, n)) => t.set("video.base.cost.measuring", &[Arg::Int(at as i64 + 1), Arg::Int(n as i64)]),
            None => {
                let secs = sweep_secs(&rows()).ceil() as i64;
                t.set("video.base.cost.how", &[Arg::Int(secs)]);
            }
        });
    });
    // Once, the first time a tab opens in a world: measure now?
    if !OFFERED.get() {
        ui.row(S::row().gap(3.0).center(), move |ui| {
            ui.text_styled(S::default().class("dim").max_w(250.0), "video.base.cost.offer", &[]);
            ui.button("video.base.cost.measure_now", S::default().h(12.0).pad(2.0), move |c| {
                start(rows(), window());
                c.client().screen_open(screen);
            });
            ui.button("video.base.cost.not_now", S::default().h(12.0).pad(2.0), move |c| {
                OFFERED.set(true);
                c.client().screen_open(screen);
            });
        });
    }
}

pub fn register(r: &mut Registry, all: fn(Tab) -> Vec<Measured>) {
    r.text_config(&COSTS, "");
    r.config(&OFFERED).bool(false);
    r.client_frame(frame);
    // For a script or a curious player: `/videocosts video|extra|fantasy|dlss|all`, the
    // costs in the log (`video cost: <key> <value> <ms>`).
    static ALL: std::sync::OnceLock<fn(Tab) -> Vec<Measured>> = std::sync::OnceLock::new();
    let _ = ALL.set(all);
    r.client_command("videocosts", "<what:word(video,extra,fantasy,dlss,all)>", |cmd| {
        let Some(all) = ALL.get() else { return };
        let rows = match cmd.text("what").unwrap_or("all") {
            "video" => all(Tab::Video),
            "extra" => all(Tab::Extra),
            "fantasy" => all(Tab::Fantasy),
            "dlss" => all(Tab::Dlss),
            _ => [Tab::Video, Tab::Extra, Tab::Fantasy, Tab::Dlss]
                .into_iter()
                .flat_map(all)
                .collect(),
        };
        let secs = sweep_secs(&rows).ceil() as i64;
        start(rows, window());
        cmd.reply("video.base.cost.started", &[Arg::Int(secs)]);
    });
}

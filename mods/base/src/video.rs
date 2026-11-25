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

/// The quality chosen: 0 low .. 3 ultra, 4 custom, 5 phone, 6 max, -1 not chosen yet (picked from the machine).
pub static QUALITY: Config<i32> = Config::named("quality");
/// The preset last chosen, still known once the quality is Custom: a feature row whose values
/// several steps share keeps that preset's name (Medium's bloom is Low's, and reads Medium).
static PRESET_LAST: Config<i32> = Config::named("preset_last");
/// The pack's screen effects: 0 off, 1 low (fewer steps), 2 on. Bloom has no low.
pub static FX_REFLECT: Config<i32> = Config::named("fx_reflect");
pub static FX_CONTACT: Config<i32> = Config::named("fx_contact");
/// The far shade from the sky (`far_ao`, an Extra row): 0 off, 1 low, 2 on.
pub static FX_FAR_AO: Config<i32> = Config::named("fx_far_ao");
/// Light in the air through the shadow maps (Lighting's `volume`, on `look-air`): until that lands,
/// the engine logs "no such effect" and the row changes nothing.
pub static FX_VOLUME: Config<i32> = Config::named("fx_volume");
pub static FX_SHAFTS: Config<i32> = Config::named("fx_shafts");
pub static FX_BLOOM: Config<i32> = Config::named("fx_bloom");

static RENDER_SCALE: Config<f32> = Config::named("core:render_scale");
/// How the world drawn at a render scale under 1 is scaled up: 0 bilinear, 1 FSR 1, 2 Snapdragon GSR 1.
static UPSCALER: Config<i32> = Config::named("core:upscaler");
static SHADOWS: Config<i32> = Config::named("core:shadows");
static SHADOW_SIZE: Config<i32> = Config::named("core:shadow_size");
static VIEW: Config<i32> = Config::named("core:view_distance");
static LOD: Config<i32> = Config::named("core:lod_distance");
static VSYNC: Config<bool> = Config::named("core:vsync");
/// Upstream's frame limit (0 off, 0 to 500 in steps of 5), which replaced our `core:fps_cap` at the 11-22 sync.
static FPS_LIMIT: Config<i32> = Config::named("core:fps_limit");
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

/// The HDR world and anti-aliasing as the game started with them (they are read at start).
static AT_START: std::sync::OnceLock<(bool, i32)> = std::sync::OnceLock::new();

/// A row whose value waits for the next start, and whether it waits now.
fn waiting(key: &str) -> bool {
    let start = AT_START.get().copied();
    match key {
        "hdr" => start.is_some_and(|s| s.0 != HDR_WORLD.get()),
        "aa" => start.is_some_and(|s| s.1 != AA.get()),
        "render_scale" => (render_scale() - RENDER_SCALE.get() as f64).abs() > 1e-3,
        "shadow_size" => (shadow_size() - SHADOW_SIZE.get() as f64).abs() > 0.5,
        _ => false,
    }
}

/// What the render scale row's Auto (0) is on a window of `window` pixels: 0.65 on a big
/// high-density screen (5 million pixels or more, the line `detect` draws for a Retina MacBook),
/// 1 elsewhere. Every scale here is a 0.05 step, as the engine keeps `core:render_scale` (a 0.67
/// was stored as 0.65, and Auto set it again at every start). The engine never gets the 0 itself: since the 11-22 sync it clamps the render
/// scale to 0.25 and up, and a 0 drew the world at a quarter of the pixels (Ronald's M3 Max,
/// 1028x629 of a 4112x2516 window, Planner 11-24).
fn auto_render_scale(window: [u32; 2]) -> f32 {
    if window[0] as u64 * window[1] as u64 >= 5_000_000 { 0.65 } else { 1.0 }
}

/// The values waiting go to the engine, while no world is open (render targets made afresh then,
/// with no frame of a world in flight). Auto goes as the scale it stands for, and a 0 an earlier
/// game wrote is put right the same way.
fn take_waiting(c: &Client) {
    let state = c.state();
    if state.flags & abi::client_flags::IN_WORLD != 0 || state.window[0] == 0 || state.window[1] == 0 {
        return;
    }
    let auto = auto_render_scale(state.window);
    let scale = RENDER_SCALE_NEXT.get();
    if scale >= 0.0 {
        if scale == 0.0 {
            weft_sdk::info!("video: render scale Auto is {auto} on this {}x{} window", state.window[0], state.window[1]);
        }
        let scale = if scale == 0.0 { auto } else { scale };
        if (scale - RENDER_SCALE.get()).abs() > 1e-3 {
            weft_sdk::info!("video: render scale {scale} from the last game");
            RENDER_SCALE.set(scale);
        }
        RENDER_SCALE_NEXT.set(-1.0);
    } else if RENDER_SCALE.get() < 0.25 {
        weft_sdk::info!("video: render scale {} was Auto's 0: now {auto}", RENDER_SCALE.get());
        RENDER_SCALE.set(auto);
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

pub static SCREEN: Menu = Menu::named("video");

/// The GPU as the engine found it (`core:gpu_*`): its name, its kind (0 not known, 1 integrated,
/// 2 discrete, 3 another), its own memory in MB.
static GPU_NAME: TextConfig = TextConfig::named("core:gpu_name");
static GPU_CLASS: Config<i32> = Config::named("core:gpu_class");
static GPU_MEMORY: Config<i32> = Config::named("core:gpu_memory_mb");
/// The GPU can trace rays (`core:gpu_ray_query`): the traced shadows' row is shown.
static GPU_RAY_QUERY: Config<bool> = Config::named("core:gpu_ray_query");
static RT_SHADOWS: Config<i32> = Config::named("core:rt_shadows");
static RT_REFLECTIONS: Config<bool> = Config::named("core:rt_reflections");
/// The path tracer's denoiser (`core:pt_denoiser`): 0 raw, 1 ours.
static PT_DENOISER: Config<i32> = Config::named("core:pt_denoiser");
/// The newer quality settings, parts of the features' rows or rows of their own on the Developer
/// screen (`PARTS`): the HDR world and temporal anti-aliasing (read at start), the mist as a
/// volume, the soft clouds' shadow, the bloom's levels, the far shadow map's reach, the screen
/// effects' half-size parts (off: every effect at full size), the leaves' see-through holes.
static HDR_WORLD: Config<bool> = Config::named("core:hdr_world");
static AA: Config<i32> = Config::named("core:aa");
static FROXELS: Config<i32> = Config::named("core:froxels");
static CLOUD_SHADOWS: Config<bool> = Config::named("core:cloud_shadows");
static BLOOM_LEVELS: Config<i32> = Config::named("core:bloom");
static SHADOW_FAR: Config<i32> = Config::named("core:shadow_far");
static POST_LOW: Config<bool> = Config::named("core:post_low");
/// Distant terrain's detail (`core:lod_detail`, Far view): 4 as made, 12 Distant Horizons' cells.
static LOD_DETAIL: Config<i32> = Config::named("core:lod_detail");
/// DLSS where the renderer has it (windows-dlss5's `dlss.rs`, its own tab): the modes it offers
/// (bit 1 DLAA; 0 none, also before that branch is in), whether Ray Reconstruction runs, the mode
/// (0 off, 1 DLAA), DLAA's model and Ray Reconstruction on.
static GPU_DLSS_MODES: Config<i32> = Config::named("core:gpu_dlss_modes");
static GPU_DLSS_RR: Config<bool> = Config::named("core:gpu_dlss_rr");
static DLSS: Config<i32> = Config::named("core:dlss");
static DLSS_MODEL_DLAA: Config<i32> = Config::named("core:dlss_model_dlaa");
static DLSS_MODEL_PERFORMANCE: Config<i32> = Config::named("core:dlss_model_performance");
static DLSS_RR: Config<bool> = Config::named("core:dlss_rr");
/// What the detection chose (Research round 19, 7: "quality chosen by the GPU at first start"):
/// the preset (-1 none yet), and whether the render scale went to half with it.
static DETECTED: Config<i32> = Config::named("gpu_preset");
static DETECTED_HALF: Config<bool> = Config::named("gpu_preset_half");

/// The GPU's kind, or the one a run pretends to have (`--fake-gpu integrated|discrete|other`).
fn gpu_class() -> i32 {
    match weft_sdk::client::launch_option("fake-gpu").as_deref() {
        Some("integrated") => 1,
        Some("discrete") => 2,
        Some("other") => 3,
        _ => GPU_CLASS.get(),
    }
}

/// A phone (Android), or a run pretending to be one (`--fake-gpu phone`): the Phone preset is
/// offered. Never chosen for the player: it trades some of the look for speed (Ronald, via
/// Planner, 11-06: graphics are optimised, not tuned down, unless he chooses it).
fn on_phone() -> bool {
    cfg!(target_os = "android") || weft_sdk::client::launch_option("fake-gpu").as_deref() == Some("phone")
}

/// The preset for this machine, and whether the world is drawn at half its pixels: an
/// integrated GPU (Intel, an AMD APU, the Steam Deck: fill-bound) Medium at half scale (its
/// effects low, shadows 1024: Research round 19); a discrete one High, Medium with under 3 GB of
/// its own; Apple's M chips by their class (Pro, Max and Ultra High, the others Medium, the
/// screen's density choosing the scale as before); a CPU or virtual GPU Low; one not known by
/// the window, as before (a high-density one draws many more pixels).
fn detect(window: [u32; 2]) -> (usize, bool) {
    let name = GPU_NAME.get().to_lowercase();
    let pixels = window[0] as u64 * window[1] as u64;
    if name.contains("apple") {
        let big = ["pro", "max", "ultra"].iter().any(|w| name.contains(w));
        return (if big { 2 } else { 1 }, false);
    }
    match gpu_class() {
        1 => (1, true),
        2 if GPU_MEMORY.get() > 0 && GPU_MEMORY.get() < 3000 => (1, false),
        2 => (2, false),
        3 => (0, false),
        _ => (if pixels >= 5_000_000 { 1 } else { 2 }, false),
    }
}

/// Chooses the preset from the GPU (`detect`) and says so on the Video screen.
fn apply_detected(window: [u32; 2]) {
    let (p, half) = detect(window);
    weft_sdk::info!(
        "video: {} ({}, {} MB): the {} preset{}",
        GPU_NAME.get(),
        gpu_class(),
        GPU_MEMORY.get(),
        preset_name(p as i32).unwrap_or("?"),
        if half { ", half render scale" } else { "" }
    );
    apply(p);
    if half {
        RENDER_SCALE_NEXT.set(0.5);
        QUALITY.set(4);
    }
    DETECTED.set(p as i32);
    DETECTED_HALF.set(half);
}

/// A setting's row: its name (`video.base.<key>`), the values it steps through, and how each is
/// shown (`video.base.value.<shown>` or a number).
struct Row {
    key: &'static str,
    values: &'static [f64],
    get: fn() -> f64,
    set: fn(f64),
    shown: fn(f64) -> (String, i64),
}

/// The presets (Ronald, 11-24: "The normal user is more like setting how, low, high, max, ultra",
/// and "all those things need their own quality setting as well"): each sets the player's rows
/// (`ROWS`: render scale, view, distant terrain, vsync, frame cap) and every feature's quality row
/// (`FEATURES`) to its step. Low, Medium, High and Max here, then Phone and Ultra (`PHONE`,
/// `ULTRA`); 4 is Custom. The look is always on: no feature's lowest step is off (Look's table,
/// `notes/STYLE.md` "The look is always on").
const PRESETS: [[f64; 5]; 4] = [
    // scale view lod   vsync cap
    [0.5, 6.0, 64.0, 1.0, 60.0],
    [0.65, 8.0, 128.0, 1.0, 60.0],
    [0.0, 12.0, 192.0, 1.0, 0.0],
    [1.0, 16.0, 256.0, 1.0, 0.0],
];
const PRESET_NAMES: [&str; 4] = ["low", "medium", "high", "max"];

/// The Phone preset (`QUALITY` 5; Ronald, 11-06: "Yes, make the preset"), offered on a phone and
/// chosen only by the player, from his Pixel 10's sweep (notes/ANDROID.md): Low's view and distant
/// terrain, and every feature at Low.
const PHONE: usize = 5;
const PHONE_PRESET: [f64; 5] = [0.5, 6.0, 64.0, 1.0, 60.0];

/// The Ultra preset (`QUALITY` 6; called Max until 11-24, when Max became the step under it):
/// every feature at its top, traced where the GPU traces rays, a view of 32 chunks and distant
/// terrain to 512, then the extras (`ultra_extras`). Allowed past the Mac's budget: the PC benches
/// it (Ronald, 11-06: "I want to test with always sort of the maximum settings on because that's
/// sort of our benchmark"). The PC queue's `--preset max` (tester-tools/t2/t2q.py) is this list
/// under its old name: change both together.
const ULTRA: usize = 6;
const ULTRA_PRESET: [f64; 5] = [1.0, 32.0, 512.0, 1.0, 0.0];

/// Preset `q`'s player rows.
fn preset(q: usize) -> Option<&'static [f64; 5]> {
    match q {
        PHONE => Some(&PHONE_PRESET),
        ULTRA => Some(&ULTRA_PRESET),
        _ => PRESETS.get(q),
    }
}

/// Preset `q`'s step on every feature's row: Low 0 to Ultra 4, Phone as Low.
fn preset_step(q: usize) -> Option<usize> {
    match q {
        0..=3 => Some(q),
        PHONE => Some(0),
        ULTRA => Some(4),
        _ => None,
    }
}

/// The look's parts no row steps, on in every preset: the HDR world, soft clouds and their
/// shadows, the mist banks (their ground map follows them, `look.rs`; their low march isn't built
/// yet, so every preset runs them in full, STYLE.md) and the leaves' holes. The pack itself is
/// turned on at every start that has none chosen (`shaderpack::install`).
fn look_on() {
    HDR_WORLD.set(true);
    SOFT_CLOUDS.set(true);
    CLOUD_SHADOWS.set(true);
    crate::look::LOOK_MIST_DOMES.set(1.0);
    crate::look::LOOK_LEAF_HOLES.set(1.0);
}

/// What Ultra adds past the features' top steps, as far as this GPU goes: anti-aliasing, the far
/// shadows' auto reach, the path tracer's denoiser, wind life at ultra, DLAA.
fn ultra_extras() {
    // Temporal anti-aliasing with the motion image; from the camera alone on a phone, whose tiled
    // GPU the motion image cost a third of its world pass (Android, Pixel 10, 11-06).
    AA.set(if on_phone() { 2 } else { 1 });
    // Auto: 2000 blocks with the HDR world (Far view's, at no measurable cost); further only
    // spreads the far map's texels thinner.
    SHADOW_FAR.set(0);
    // (Far detail stays at 4: 12 cost +0.33 ms for a small gain, Planner 11-06.)
    // Ours wherever DLSS Ray Reconstruction doesn't denoise the path tracer's light.
    PT_DENOISER.set(1);
    FX_WIND_LIFE.set(3);
    // DLAA (DLSS at the screen's own size) where the renderer has it, with its newest model (M,
    // 13) and Ray Reconstruction where that runs (windows-dlss5, 11-06): the engine then skips its
    // own gathering for the frame. Elsewhere TAA as above.
    // On an RTX 20 or 30 (`older_rtx`) Performance with M instead: there M costs about 10.6 ms at
    // DLAA (E 1.4, K 2.3) and DLAA with Ray Reconstruction 45 ms, Performance M 12.5 in all, and in
    // motion E and K drew stripes beside the pool where M stayed clean (3060 Laptop, w4 and w8,
    // 11-25; Planner agreed). Quality with E stays the cheaper choice in the DLSS tab.
    let modes = GPU_DLSS_MODES.get();
    if older_rtx() && modes & (1 << 4) != 0 {
        DLSS.set(4);
        DLSS_MODEL_PERFORMANCE.set(13);
        DLSS_RR.set(GPU_DLSS_RR.get());
    } else if modes & (1 << 1) != 0 {
        DLSS.set(1);
        DLSS_MODEL_DLAA.set(13);
        DLSS_RR.set(GPU_DLSS_RR.get());
    }
    weft_sdk::info!("video: Ultra's DLSS mode {} (modes {modes:#b}, RTX 20/30: {})", DLSS.get(), older_rtx());
}

/// A GeForce RTX 20 or 30, a TITAN RTX or an RTX A (workstation Ampere), by the GPU's name
/// (`--fake-gpu-name` pretends): Turing and Ampere, which run DLSS's transformer models (K, M) at
/// several times the CNN's cost. An RTX 40, 50 or a name not known: the newest models at full size.
fn older_rtx() -> bool {
    let name = weft_sdk::client::launch_option("fake-gpu-name")
        .unwrap_or_else(|| GPU_NAME.get())
        .to_lowercase();
    // (Ada's workstation cards are named "RTX 2000 Ada" and the like.)
    !name.contains("ada")
        && ["rtx 20", "rtx 30", "titan rtx", "rtx a"]
            .iter()
            .any(|w| name.contains(w))
}

/// Preset `q`'s name (`video.base.preset.<name>`); None for Custom and not chosen.
fn preset_name(q: i32) -> Option<&'static str> {
    if q == PHONE as i32 {
        Some("phone")
    } else if q == ULTRA as i32 {
        Some("ultra")
    } else {
        usize::try_from(q)
            .ok()
            .and_then(|q| PRESET_NAMES.get(q).copied())
    }
}

fn word(k: &str) -> (String, i64) {
    (k.to_string(), 0)
}

fn number(v: f64) -> (String, i64) {
    (String::new(), v.round() as i64)
}

/// The player's rows the presets set (`PRESETS`): the first 5 of `ROWS`. Anti-aliasing and the
/// upscaler after them are the player's own choice (Ronald, 11-06: the upscaler opt-in), so no
/// preset but Ultra changes them and they never make the quality Custom.
const PRESET_ROWS: usize = 5;

const ROWS: [Row; 7] = [
    Row {
        key: "render_scale",
        // 0 is Auto: 0.65 on a big high-density screen, 1 elsewhere (`auto_render_scale`), written
        // as that number at the next start.
        values: &[0.0, 0.5, 0.6, 0.65, 0.75, 0.85, 1.0],
        get: render_scale,
        set: |v| RENDER_SCALE_NEXT.set(v as f32),
        shown: |v| if v == 0.0 { word("auto") } else { ("percent".into(), (v * 100.0).round() as i64) },
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
        values: &[0.0, 30.0, 60.0, 90.0, 120.0, 165.0, 240.0],
        get: || FPS_LIMIT.get() as f64,
        set: |v| FPS_LIMIT.set(v as i32),
        shown: |v| if v == 0.0 { word("unlimited") } else { number(v) },
    },
    Row {
        key: "aa",
        values: &[0.0, 1.0, 2.0],
        get: || AA.get() as f64,
        set: |v| AA.set(v as i32),
        shown: |v| word(["off", "taa", "taa_camera"][(v as usize).min(2)]),
    },
    Row {
        key: "upscaler",
        values: &[0.0, 1.0, 2.0],
        get: || UPSCALER.get() as f64,
        set: |v| UPSCALER.set(v as i32),
        shown: |v| word(["smooth", "fsr1", "gsr1"][(v as usize).min(2)]),
    },
];

/// A feature's steps, Low to Ultra (`video.base.value.<name>`).
const STEP_NAMES: [&str; 5] = ["low", "medium", "high", "max", "ultra"];

/// A feature's quality row (Ronald, 11-24: "shadows have a quality setting, like all those things
/// need their own quality setting as well"): the parts it sets (`PARTS`' keys, the Developer
/// screen's rows) and their values at each step, Low to Ultra. Its cost is its parts' costs.
struct Feature {
    key: &'static str,
    parts: &'static [&'static str],
    steps: [&'static [f64]; 5],
}

/// The features' rows on the Video screen, from Look's table (11-24: Phone and Low the same, then
/// Medium, High, Max and Ultra). A traced part runs only where the GPU traces rays (`runs`).
const FEATURES: [Feature; 6] = [
    // The sun's shadows: near (1) or near and far (2), their map's size, how far they reach
    // (blocks), contact shadows; at Ultra traced, soft-edged.
    Feature {
        key: "shadow_quality",
        parts: &["shadows", "shadow_size", "shadow_reach", "contact", "rt_shadows"],
        steps: [
            &[1.0, 1024.0, 64.0, 1.0, 0.0],
            &[1.0, 1024.0, 96.0, 1.0, 0.0],
            &[2.0, 2048.0, 128.0, 1.0, 0.0],
            &[2.0, 4096.0, 128.0, 2.0, 0.0],
            &[2.0, 4096.0, 128.0, 2.0, 2.0],
        ],
    },
    // Reflections on water and glass: the screen's, low or full; at Ultra traced.
    Feature {
        key: "reflect_quality",
        parts: &["reflect", "rt_reflections"],
        steps: [&[1.0, 0.0], &[1.0, 0.0], &[2.0, 0.0], &[2.0, 0.0], &[2.0, 1.0]],
    },
    // The light in the air: the mist's light, its detail (`core:froxels`) and the light shafts.
    Feature {
        key: "air_quality",
        parts: &["volume", "mist_volume", "shafts"],
        steps: [&[1.0, 1.0, 1.0], &[1.0, 1.0, 1.0], &[2.0, 2.0, 1.0], &[2.0, 2.0, 2.0], &[2.0, 2.0, 2.0]],
    },
    // The far shade from the sky between far crowns and at the foot of far steps.
    Feature { key: "shade_quality", parts: &["far_ao"], steps: [&[1.0], &[1.0], &[2.0], &[2.0], &[2.0]] },
    // Bloom: its glow's levels, four (1) or six (2) (Look, 11-24).
    Feature { key: "bloom_quality", parts: &["bloom_levels"], steps: [&[1.0], &[1.0], &[1.0], &[2.0], &[2.0]] },
    // The screen effects' half-size parts (1) or every effect at full size (0): a cost row, which
    // the benches may move (Look, 11-24).
    Feature { key: "effect_quality", parts: &["effect_detail"], steps: [&[1.0], &[1.0], &[0.0], &[0.0], &[0.0]] },
];

/// Whether this GPU runs a part: the traced ones only where it traces rays, never on a phone.
fn runs(key: &str) -> bool {
    !key.starts_with("rt_") || (GPU_RAY_QUERY.get() && !on_phone())
}

fn part(key: &str) -> &'static Row {
    PARTS
        .iter()
        .find(|r| r.key == key)
        .expect("a feature's part is in PARTS")
}

/// What feature `key`'s step sets each of its parts to here (a part this GPU doesn't run, 0), or
/// None if `key` is no feature (`video_costs` sums a feature's cost from these).
pub(crate) fn feature_parts(key: &str, step: usize) -> Option<Vec<(&'static str, f64)>> {
    let f = FEATURES.iter().find(|f| f.key == key)?;
    let values = f.steps.get(step)?;
    Some(
        f.parts
            .iter()
            .zip(values.iter())
            .map(|(&k, &v)| (k, if runs(k) { v } else { 0.0 }))
            .collect(),
    )
}

/// Whether feature `f`'s parts all hold step `s`'s values.
fn feature_at(f: usize, s: usize) -> bool {
    feature_parts(FEATURES[f].key, s)
        .unwrap_or_default()
        .iter()
        .all(|&(k, v)| ((part(k).get)() - v).abs() < 1e-3)
}

/// Feature `f`'s step now, as its row names it: the preset's own step (or, once Custom, the last
/// preset's) when the parts still hold it (steps can share values: High's reflections are Max's), else
/// the lowest step they hold, or -1 (Custom: a part moved on the Developer screen).
fn feature_now(f: usize) -> f64 {
    for q in [QUALITY.get(), PRESET_LAST.get()] {
        if let Some(s) = usize::try_from(q).ok().and_then(preset_step)
            && feature_at(f, s)
        {
            return s as f64;
        }
    }
    (0..STEP_NAMES.len())
        .find(|&s| feature_at(f, s))
        .map_or(-1.0, |s| s as f64)
}

fn feature_set(f: usize, step: f64) {
    let Some(parts) = feature_parts(FEATURES[f].key, step.max(0.0) as usize) else { return };
    for (k, v) in parts {
        (part(k).set)(v);
    }
}

fn feature_get_n<const F: usize>() -> f64 {
    feature_now(F)
}

fn feature_set_n<const F: usize>(v: f64) {
    feature_set(F, v)
}

fn step_shown(v: f64) -> (String, i64) {
    word(if v < 0.0 { "custom" } else { STEP_NAMES[(v as usize).min(4)] })
}

const STEPS: &[f64] = &[0.0, 1.0, 2.0, 3.0, 4.0];

/// The features' rows on the Video screen (`FEATURES`, in its order).
const FEATURE_ROWS: [Row; 6] = [
    Row { key: FEATURES[0].key, values: STEPS, get: feature_get_n::<0>, set: feature_set_n::<0>, shown: step_shown },
    Row { key: FEATURES[1].key, values: STEPS, get: feature_get_n::<1>, set: feature_set_n::<1>, shown: step_shown },
    Row { key: FEATURES[2].key, values: STEPS, get: feature_get_n::<2>, set: feature_set_n::<2>, shown: step_shown },
    Row { key: FEATURES[3].key, values: STEPS, get: feature_get_n::<3>, set: feature_set_n::<3>, shown: step_shown },
    Row { key: FEATURES[4].key, values: STEPS, get: feature_get_n::<4>, set: feature_set_n::<4>, shown: step_shown },
    Row { key: FEATURES[5].key, values: STEPS, get: feature_get_n::<5>, set: feature_set_n::<5>, shown: step_shown },
];

fn off_low_on(v: f64) -> (String, i64) {
    word(["off", "low", "on"][(v as usize).min(2)])
}

/// Sets the player's rows and every feature to preset `p`, the look's always-on parts, Ultra's
/// extras, and the fantasy rows that cost to what it allows (`fantasy_caps`).
fn apply(p: usize) {
    let (Some(values), Some(at)) = (preset(p), preset_step(p)) else { return };
    for (row, &v) in ROWS.iter().zip(values.iter()) {
        (row.set)(v);
    }
    for f in 0..FEATURES.len() {
        feature_set(f, at as f64);
    }
    look_on();
    QUALITY.set(p as i32);
    PRESET_LAST.set(p as i32);
    weft_sdk::info!("video: the {} preset, every feature at {}", preset_name(p as i32).unwrap_or("?"), STEP_NAMES[at]);
    if p == ULTRA {
        ultra_extras();
    } else if GPU_DLSS_MODES.get() != 0 {
        // DLSS is live: off again with any other preset, as Ultra is the one that turns it on.
        DLSS.set(0);
    }
    fantasy_caps(p, true);
}

/// One player's row a step on (`dir` 1) or back (-1); a row a preset sets makes the quality Custom.
fn step(i: usize, dir: i32) {
    step_row(&ROWS[i], dir);
    if i < PRESET_ROWS {
        QUALITY.set(4);
    }
}

/// One feature a step on or back: the quality becomes Custom.
/// It skips the steps that would change nothing (High's reflections are Max's) and goes round
/// like the other rows.
fn step_feature(i: usize, dir: i32) {
    let n = STEP_NAMES.len() as i32;
    let now = feature_now(i);
    let from = if now >= 0.0 {
        now as i32
    } else if dir > 0 {
        -1
    } else {
        n
    };
    if let Some(s) = (1..=n)
        .map(|k| (from + dir * k).rem_euclid(n) as usize)
        .find(|&s| !feature_at(i, s))
    {
        feature_set(i, s as f64);
    }
    QUALITY.set(4);
}

fn step_row(row: &Row, dir: i32) {
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
}

// ---------------------------------------------------------------- fantasy

/// The Fantasy screen (Research round 17, "the option to turn on off parts"): a master that sets
/// the taste rows (Off, Subtle the default, Storybook; Custom once a row is changed), and the
/// rows. The quality presets touch only what costs (`fantasy_caps`). The pack reads the look
/// rows through its options (Lighting's `look.rs`); fireflies, motes and sky events are the base
/// mod's own.
pub static FANTASY_SCREEN: Menu = Menu::named("video_fantasy");
/// 0 off, 1 subtle, 2 storybook, 3 custom.
pub static FANTASY: Config<i32> = Config::named("fantasy");
pub static FX_FIREFLIES: Config<i32> = Config::named("fx_fireflies");
/// Leaves, petals, pollen, spindrift and sand in the wind (`wind_life.rs`): 0 off, 1 few, 2 many,
/// 3 ultra (`vitals::wind_life::DENSITY`).
pub static FX_WIND_LIFE: Config<i32> = Config::named("fx_wind_life");
pub static FX_MOTES: Config<i32> = Config::named("fx_motes");
pub static LOOK_STORYBOOK: Config<f32> = Config::named("look_storybook");
pub static FX_NIGHT_SKY: Config<i32> = Config::named("fx_night_sky");
pub static FX_SHOOTING_STARS: Config<i32> = Config::named("fx_shooting_stars");
/// The moon's size as a factor (the pack scales it in `sky.wgsl`).
pub static MOON_SIZE: Config<f32> = Config::named("moon_size");
pub static FX_GLOW: Config<f32> = Config::named("fx_glow");
pub static FX_BIOME_AIR: Config<f32> = Config::named("fx_biome_air");
/// 0 off, 1 looks only (no gameplay), 2 on.
pub static FX_SKY_EVENTS: Config<i32> = Config::named("fx_sky_events");
pub static FX_SPARKLE: Config<i32> = Config::named("fx_sparkle");
/// The sky's clouds: blocky (off) or soft volumetric ones (Lighting's; Ronald: "Both, a setting",
/// blocky by default). The engine's, at once, no restart.
static SOFT_CLOUDS: Config<bool> = Config::named("core:soft_clouds");

const FANTASY_NAMES: [&str; 3] = ["off", "subtle", "storybook"];

fn percent(v: f64) -> (String, i64) {
    ("percent".into(), (v * 100.0).round() as i64)
}

const FANTASY_ROWS: [Row; 12] = [
    Row {
        key: "fireflies",
        values: &[0.0, 1.0, 2.0],
        get: || FX_FIREFLIES.get() as f64,
        set: |v| FX_FIREFLIES.set(v as i32),
        shown: |v| word(["off", "few", "many"][(v as usize).min(2)]),
    },
    Row { key: "motes", values: &[0.0, 1.0], get: || FX_MOTES.get() as f64, set: |v| FX_MOTES.set(v as i32), shown: off_on },
    Row {
        key: "wind_life",
        values: &[0.0, 1.0, 2.0, 3.0],
        get: || FX_WIND_LIFE.get() as f64,
        set: |v| FX_WIND_LIFE.set(v as i32),
        shown: |v| word(["off", "few", "many", "ultra"][(v as usize).min(3)]),
    },
    Row {
        key: "storybook",
        values: &[0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
        get: || LOOK_STORYBOOK.get() as f64,
        set: |v| LOOK_STORYBOOK.set(v as f32),
        shown: percent,
    },
    Row {
        key: "night_sky",
        values: &[0.0, 1.0, 2.0],
        get: || FX_NIGHT_SKY.get() as f64,
        set: |v| FX_NIGHT_SKY.set(v as i32),
        shown: |v| word(["off", "subtle", "vivid"][(v as usize).min(2)]),
    },
    Row {
        key: "shooting_stars",
        values: &[0.0, 1.0],
        get: || FX_SHOOTING_STARS.get() as f64,
        set: |v| FX_SHOOTING_STARS.set(v as i32),
        shown: off_on,
    },
    Row {
        key: "moon_size",
        values: &[1.0, 1.5, 2.0],
        get: || MOON_SIZE.get() as f64,
        set: |v| MOON_SIZE.set(v as f32),
        shown: |v| {
            word(if v >= 1.75 {
                "moon_2"
            } else if v >= 1.25 {
                "moon_15"
            } else {
                "moon_1"
            })
        },
    },
    Row { key: "glow", values: &[0.0, 0.5, 1.0, 1.25, 1.5], get: || FX_GLOW.get() as f64, set: |v| FX_GLOW.set(v as f32), shown: percent },
    Row {
        key: "biome_air",
        values: &[0.0, 0.3, 0.6, 0.8, 1.0],
        get: || FX_BIOME_AIR.get() as f64,
        set: |v| FX_BIOME_AIR.set(v as f32),
        shown: percent,
    },
    Row {
        key: "sky_events",
        values: &[0.0, 1.0, 2.0],
        get: || FX_SKY_EVENTS.get() as f64,
        set: |v| FX_SKY_EVENTS.set(v as i32),
        shown: |v| word(["off", "looks_only", "on"][(v as usize).min(2)]),
    },
    Row { key: "sparkle", values: &[0.0, 1.0], get: || FX_SPARKLE.get() as f64, set: |v| FX_SPARKLE.set(v as i32), shown: off_on },
    Row {
        key: "clouds",
        values: &[0.0, 1.0],
        get: || SOFT_CLOUDS.get() as u8 as f64,
        set: |v| SOFT_CLOUDS.set(v > 0.0),
        shown: |v| word(if v > 0.0 { "clouds_soft" } else { "blocky" }),
    },
];

/// What the master sets, Off, Subtle and Storybook, per row (in `FANTASY_ROWS` order); NaN leaves
/// a row as it is: sky events carry gameplay (Ronald: "Gameplay too"), so a look's master
/// doesn't take them away. Then bloom: soft, soft, dreamy.
const FANTASY_PRESETS: [[f64; 11]; 3] = [
    // fireflies motes storybook night shooting moon glow  air  events    sparkle clouds
    [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, f64::NAN, 0.0, 0.0],
    [1.0, 1.0, 0.4, 1.0, 1.0, 1.0, 1.0, 0.6, f64::NAN, 1.0, 0.0],
    [2.0, 1.0, 1.0, 2.0, 1.0, 1.5, 1.5, 1.0, f64::NAN, 1.0, 1.0],
];
const FANTASY_BLOOM: [i32; 3] = [1, 1, 2];

fn off_on(v: f64) -> (String, i64) {
    word(if v > 0.0 { "on" } else { "off" })
}

/// Sets the fantasy rows to master `f` (0 off .. 2 storybook), within what the quality allows.
fn apply_fantasy(f: usize) {
    for (row, &v) in FANTASY_ROWS.iter().zip(FANTASY_PRESETS[f].iter()) {
        if !v.is_nan() {
            (row.set)(v);
        }
    }
    FX_BLOOM.set(FANTASY_BLOOM[f]);
    FANTASY.set(f as i32);
    if let Ok(q) = usize::try_from(QUALITY.get()) {
        fantasy_caps(q, false);
    }
}

/// What quality preset `q` allows of the rows that cost (Research round 17's table): Low few
/// fireflies, no motes, a subtle night sky, no sparkle, soft bloom; Medium soft bloom. Soft clouds
/// cost no more than blocky (Lighting, on the PC and the Mac), so they follow the master alone. With
/// `bloom_up`, High, Max and Ultra take the master's bloom (a quality preset just chosen).
fn fantasy_caps(q: usize, bloom_up: bool) {
    // Phone: as Low, its bloom as Medium's.
    if q == PHONE {
        FX_FIREFLIES.set(FX_FIREFLIES.get().min(1));
        FX_MOTES.set(0);
        FX_NIGHT_SKY.set(FX_NIGHT_SKY.get().min(1));
        FX_SPARKLE.set(0);
        FX_BLOOM.set(FX_BLOOM.get().min(1));
    } else if q == 0 {
        FX_FIREFLIES.set(FX_FIREFLIES.get().min(1));
        FX_WIND_LIFE.set(FX_WIND_LIFE.get().min(1));
        FX_MOTES.set(0);
        FX_NIGHT_SKY.set(FX_NIGHT_SKY.get().min(1));
        FX_SPARKLE.set(0);
        // Soft, never off: the look is always on (Ronald 11-24).
        FX_BLOOM.set(1);
    } else if q == 1 {
        FX_BLOOM.set(FX_BLOOM.get().min(1));
    } else if bloom_up
        && (q <= 3 || q == ULTRA)
        && let Some(&b) = FANTASY_BLOOM.get(FANTASY.get() as usize)
    {
        FX_BLOOM.set(b);
    }
}

/// A row's line on a settings screen: its name, ‹ value ›, and what it does (or that it waits
/// for a restart).
/// Runs `f`, and builds the screen again when it moved the quality (a preset chosen, a row making
/// it Custom): a mod's screen is built once, and the presets' buttons show which one is on.
fn quality_changing(c: &ClientView, f: impl FnOnce()) {
    let was = QUALITY.get();
    f();
    if QUALITY.get() != was {
        c.client().rebuild_screen();
    }
}

fn row_ui(ui: &mut Ui, rows: &'static [Row], i: usize, step: fn(usize, i32)) {
    let row = &rows[i];
    ui.row(S::row().gap(3.0).center(), move |ui| {
        ui.text_styled(S::default().w(84.0).no_wrap(), &format!("video.base.{}", row.key), &[]);
        ui.button("video.base.less", S::default().size(12.0, 12.0).pad(0.5), move |c| quality_changing(c, || step(i, -1)));
        ui.row(S::row().w(96.0).center(), move |ui| {
            ui.text_bind_styled(S::default().no_wrap(), move |_, t| {
                let (k, n) = (rows[i].shown)((rows[i].get)());
                if k.is_empty() {
                    t.set("video.base.value.number", &[Arg::Int(n)]);
                } else {
                    t.set(&format!("video.base.value.{k}"), &[Arg::Int(n)]);
                }
            });
        });
        ui.button("video.base.more", S::default().size(12.0, 12.0).pad(0.5), move |c| quality_changing(c, || step(i, 1)));
        // What its value costs here (`video_costs`).
        crate::video_costs::cost_cell(ui, row.key, row.get);
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

/// Every part the features set, and the parts no feature steps, each a row of its own on the
/// Developer screen (`DEV_SCREEN`), where a developer moves one alone (the quality becomes Custom).
/// Players see the features' rows instead (Ronald, 11-24: the tuning "is more for developer
/// things"). Traced shadows, reflections and the path tracer's denoiser only where the GPU traces
/// rays (`traced`).
const PARTS: [Row; 22] = [
    Row {
        key: "shadows",
        values: &[0.0, 1.0, 2.0],
        get: || SHADOWS.get() as f64,
        set: |v| SHADOWS.set(v as i32),
        shown: |v| word(["off", "near", "near_far"][v as usize % 3]),
    },
    Row { key: "shadow_size", values: &[1024.0, 2048.0, 4096.0], get: shadow_size, set: |v| SHADOW_SIZE_NEXT.set(v as i32), shown: number },
    // How far the sun's shadows reach before the sky light's mask takes over (Look, 11-24).
    Row {
        key: "shadow_reach",
        values: &[32.0, 64.0, 96.0, 128.0, 192.0, 256.0],
        get: || crate::look::LOOK_SHADOW_REACH.get() as f64,
        set: |v| crate::look::LOOK_SHADOW_REACH.set(v as f32),
        shown: |v| ("blocks".into(), v as i64),
    },
    Row { key: "contact", values: &[0.0, 1.0, 2.0], get: || FX_CONTACT.get() as f64, set: |v| FX_CONTACT.set(v as i32), shown: off_low_on },
    Row {
        key: "rt_shadows",
        // Ultra: soft edges from more rays a pixel (with TAA), about three times the cost.
        values: &[0.0, 1.0, 2.0],
        get: || RT_SHADOWS.get() as f64,
        set: |v| RT_SHADOWS.set(v as i32),
        shown: |v| word(["off", "on", "ultra"][(v as usize).min(2)]),
    },
    Row { key: "reflect", values: &[0.0, 1.0, 2.0], get: || FX_REFLECT.get() as f64, set: |v| FX_REFLECT.set(v as i32), shown: off_low_on },
    // Reflections traced on water, glass, polished stone and metal, in place of the screen's.
    Row {
        key: "rt_reflections",
        values: &[0.0, 1.0],
        get: || RT_REFLECTIONS.get() as u8 as f64,
        set: |v| RT_REFLECTIONS.set(v > 0.0),
        shown: off_on,
    },
    Row { key: "volume", values: &[0.0, 1.0, 2.0], get: || FX_VOLUME.get() as f64, set: |v| FX_VOLUME.set(v as i32), shown: off_low_on },
    Row {
        key: "mist_volume",
        values: &[0.0, 1.0, 2.0],
        get: || FROXELS.get() as f64,
        set: |v| FROXELS.set(v as i32),
        shown: |v| word(["off", "on", "ultra"][(v as usize).min(2)]),
    },
    Row { key: "shafts", values: &[0.0, 1.0, 2.0], get: || FX_SHAFTS.get() as f64, set: |v| FX_SHAFTS.set(v as i32), shown: off_low_on },
    // The far shade from the sky in the gaps between far crowns and at the foot of far steps
    // (Look, 11-06: Bliss's shading with distance, our own code).
    Row { key: "far_ao", values: &[0.0, 1.0, 2.0], get: || FX_FAR_AO.get() as f64, set: |v| FX_FAR_AO.set(v as i32), shown: off_low_on },
    Row {
        key: "bloom",
        // Soft, or dreamy (Storybook's): halos and bloomy fog (Research round 17).
        values: &[0.0, 1.0, 2.0],
        get: || FX_BLOOM.get() as f64,
        set: |v| FX_BLOOM.set(v as i32),
        shown: |v| word(["off", "soft", "dreamy"][(v as usize).min(2)]),
    },
    Row {
        key: "bloom_levels",
        values: &[0.0, 1.0, 2.0],
        get: || BLOOM_LEVELS.get() as f64,
        set: |v| BLOOM_LEVELS.set(v as i32),
        shown: |v| word(["off", "narrow", "wide"][(v as usize).min(2)]),
    },
    Row {
        key: "effect_detail",
        // The half-size parts on (1) or every effect at full size (0).
        values: &[1.0, 0.0],
        get: || POST_LOW.get() as u8 as f64,
        set: |v| POST_LOW.set(v > 0.0),
        shown: |v| word(if v > 0.0 { "half" } else { "full" }),
    },
    Row { key: "hdr", values: &[0.0, 1.0], get: || HDR_WORLD.get() as u8 as f64, set: |v| HDR_WORLD.set(v > 0.0), shown: off_on },
    Row {
        key: "cloud_shadows",
        values: &[0.0, 1.0],
        get: || CLOUD_SHADOWS.get() as u8 as f64,
        set: |v| CLOUD_SHADOWS.set(v > 0.0),
        shown: off_on,
    },
    // The mist's banks walked per pixel on the ground the engine maps (stage 2, 11-24): on as made
    // and in every preset (the look always on, Ronald 11-24).
    Row {
        key: "mist_banks",
        values: &[0.0, 1.0],
        get: || (crate::look::LOOK_MIST_DOMES.get() > 0.5) as u8 as f64,
        set: |v| crate::look::LOOK_MIST_DOMES.set(if v > 0.0 { 1.0 } else { 0.0 }),
        shown: off_on,
    },
    Row {
        key: "leaf_holes",
        values: &[0.0, 1.0],
        get: || (crate::look::LOOK_LEAF_HOLES.get() > 0.5) as u8 as f64,
        set: |v| crate::look::LOOK_LEAF_HOLES.set(if v > 0.0 { 1.0 } else { 0.0 }),
        shown: off_on,
    },
    // The light through leaves (Trees, 11-06): off by default and in no preset for now (Planner:
    // a later call), as Ronald's Bliss has none.
    Row {
        key: "leaf_light",
        values: &[0.0, 1.0],
        get: || (crate::look::LOOK_LEAF_LIGHT.get() > 0.5) as u8 as f64,
        set: |v| crate::look::LOOK_LEAF_LIGHT.set(if v > 0.0 { 1.0 } else { 0.0 }),
        shown: off_on,
    },
    Row { key: "far_detail", values: &[4.0, 8.0, 12.0], get: || LOD_DETAIL.get() as f64, set: |v| LOD_DETAIL.set(v as i32), shown: number },
    Row {
        key: "shadow_far",
        values: &[0.0, 400.0, 1000.0, 2000.0, 2400.0],
        get: || SHADOW_FAR.get() as f64,
        set: |v| SHADOW_FAR.set(v as i32),
        shown: |v| if v == 0.0 { word("auto") } else { ("blocks".into(), v as i64) },
    },
    Row {
        key: "pt_denoiser",
        // Off shows the path tracer's raw light (checks only).
        values: &[0.0, 1.0],
        get: || PT_DENOISER.get() as f64,
        set: |v| PT_DENOISER.set(v as i32),
        shown: |v| word(if v > 0.0 { "pt_denoiser_svgf" } else { "pt_denoiser_raw" }),
    },
];

/// A part only a GPU that traces rays shows (traced shadows, reflections, the path tracer's denoiser).
fn traced(key: &str) -> bool {
    key.starts_with("rt_") || key.starts_with("pt_")
}

/// A part a step on, on the Developer screen: the quality becomes Custom.
fn step_part(i: usize, dir: i32) {
    step_row(&PARTS[i], dir);
    QUALITY.set(4);
}

/// The Developer screen (`developer.rs`): the parts' rows, then the look's tuning knobs.
pub static DEV_SCREEN: Menu = Menu::named("video_developer");

/// The Video screens' tabs, but the one shown (`here`, its title key): Video, Fantasy, DLSS where
/// the renderer has it, and Developer while `base:developer_menu` is on.
pub(crate) fn tabs(here: &str) -> Vec<crate::window::Tab> {
    let mut tabs: Vec<crate::window::Tab> = vec![
        ("video.base.title", |c| {
            c.client().menu_open(&SCREEN);
        }),
        ("video.base.fantasy.title", |c| {
            c.client().menu_open(&FANTASY_SCREEN);
        }),
    ];
    if crate::dlss::available() {
        tabs.push(("video.base.dlss.title", |c| {
            c.client().menu_open(&crate::dlss::SCREEN);
        }));
    }
    if crate::developer::MENU.get() {
        tabs.push(("video.base.developer.title", |c| {
            c.client().menu_open(&DEV_SCREEN);
        }));
    }
    tabs.retain(|t| t.0 != here);
    tabs
}

fn step_fantasy(i: usize, dir: i32) {
    step_row(&FANTASY_ROWS[i], dir);
    FANTASY.set(3);
}

/// Which defaults the saved settings have caught up with (`new_defaults`).
static DEFAULTS: Config<i32> = Config::named("video_defaults");

/// Saved settings take the new defaults once (the engine saves every setting, so a changed
/// default alone reaches no one who has played): on a preset, its values for the rows whose
/// defaults changed; on Custom, a row still at its old default takes the new one, a row the
/// player changed stays. 1: on a preset, its features' steps (11-05's contact shadows and light
/// shafts, since 11-24 the feature rows). 2: temporal
/// anti-aliasing on (11-06), from the camera alone on a phone, where it is still off.
fn new_defaults() {
    if DEFAULTS.get() >= 2 || QUALITY.get() < 0 {
        return;
    }
    // A script's or benchmark's `--config` values are meant as given.
    if ["script", "test", "config"]
        .iter()
        .any(|o| weft_sdk::client::launch_option(o).is_some())
    {
        return;
    }
    if DEFAULTS.get() >= 1 {
        // Ronald, 11-06: "Yes, on by default". Saved settings kept the old default (off); a player
        // who chose a mode keeps it. Applied at the next start (it is read there).
        if AA.get() == 0 {
            let aa = if on_phone() { 2 } else { 1 };
            weft_sdk::info!("video: new defaults, anti-aliasing 0 -> {aa}");
            AA.set(aa);
        }
        DEFAULTS.set(2);
        return;
    }
    // On a preset, its features' steps.
    if let Some(at) = usize::try_from(QUALITY.get()).ok().and_then(preset_step) {
        weft_sdk::info!("video: new defaults, every feature at {}", STEP_NAMES[at]);
        for f in 0..FEATURES.len() {
            feature_set(f, at as f64);
        }
    }
    DEFAULTS.set(1);
    // And the next step at once (`DEFAULTS` 2).
    new_defaults();
}

/// The rows a tab's measuring steps through (`video_costs`): those that apply at once and cost GPU
/// time. Not render scale, shadow detail, the HDR world or anti-aliasing (read at start), the view
/// and distant terrain (their land loads for long after), vsync and the frame cap (not costs), nor
/// the far shadows' reach (its map is drawn again only when the light moves) or the looks that cost
/// nothing (storybook, moon, glow, sky events).
pub(crate) fn measured_rows(tab: crate::video_costs::Tab) -> Vec<crate::video_costs::Measured> {
    use crate::video_costs::{Measured, Tab};
    let pick = |rows: &'static [Row], keys: &[(&str, f32)]| -> Vec<Measured> {
        rows.iter()
            .filter_map(|r| {
                let settle = keys.iter().find(|k| k.0 == r.key)?.1;
                Some(Measured { key: r.key, values: r.values.to_vec(), get: r.get, set: r.set, settle })
            })
            .collect()
    };
    match tab {
        Tab::Video => {
            let mut keys = vec![
                ("shadows", 0.6),
                ("contact", 0.4),
                ("reflect", 0.4),
                ("volume", 0.4),
                ("mist_volume", 0.6),
                ("shafts", 0.4),
                ("far_ao", 0.4),
                ("bloom_levels", 0.4),
                ("effect_detail", 0.4),
            ];
            if GPU_RAY_QUERY.get() {
                // A new world shader each change, and its scene.
                keys.push(("rt_shadows", 2.0));
                keys.push(("rt_reflections", 2.0));
            }
            pick(&PARTS, &keys)
        }
        Tab::Extra => {
            let mut keys = vec![("bloom", 0.4), ("cloud_shadows", 0.6), ("leaf_holes", 0.4), ("far_detail", 0.6), ("leaf_light", 0.4)];
            if GPU_RAY_QUERY.get() {
                keys.push(("pt_denoiser", 1.0));
            }
            pick(&PARTS, &keys)
        }
        Tab::Fantasy => pick(
            &FANTASY_ROWS,
            &[
                ("fireflies", 1.5),
                ("motes", 1.5),
                ("wind_life", 1.5),
                ("night_sky", 0.4),
                ("shooting_stars", 0.4),
                ("biome_air", 0.4),
                ("sparkle", 0.4),
                ("clouds", 1.0),
            ],
        ),
        Tab::Dlss => crate::dlss::measured_rows(),
    }
}

/// How many GPU ms a frame will take more (or less) once the changes waiting for a restart apply:
/// the HDR world and anti-aliasing by their costs, render scale by the pixels (an estimate).
fn after_restart(window: [u32; 2]) -> Option<f64> {
    use crate::video_costs::{Cost, cost};
    let ms = |c: Cost| match c {
        Cost::Measured(m) | Cost::Estimate(m) => m,
        Cost::None => 0.0,
    };
    let start = AT_START.get().copied()?;
    let mut d = 0.0;
    let mut any = false;
    if start.0 != HDR_WORLD.get() {
        d += ms(cost(window, "hdr", HDR_WORLD.get() as u8 as f64)) - ms(cost(window, "hdr", start.0 as u8 as f64));
        any = true;
    }
    if start.1 != AA.get() {
        d += ms(cost(window, "aa", AA.get() as f64)) - ms(cost(window, "aa", start.1 as f64));
        any = true;
    }
    if waiting("render_scale") {
        let now = RENDER_SCALE.get() as f64;
        let next = render_scale();
        let (now, next) = (if now > 0.0 { now } else { 1.0 }, if next > 0.0 { next } else { 1.0 });
        d += crate::video_costs::live_gpu() * ((next * next) / (now * now) - 1.0);
        any = true;
    }
    any.then_some(d)
}

pub fn register(r: &mut Registry) {
    crate::developer::register(r);
    crate::developer::video(r.config(&QUALITY).int(-1, -1, 6));
    crate::developer::page(r.config(&PRESET_LAST).int(-1, -1, 6));
    crate::developer::video(r.config(&FX_REFLECT).int(2, 0, 2));
    // Low by default, as every look part (Ronald 11-24: on by default; 11-05 had contact shadows off).
    crate::developer::video(r.config(&FX_CONTACT).int(1, 0, 2));
    crate::developer::video(r.config(&FX_FAR_AO).int(1, 0, 2));
    // Cheap (Tester 2's PC bench, 11-04: +0.04 ms of effects at 1280x720): low until a preset
    // says otherwise; off on the Low preset until the Mac's Retina numbers are in.
    crate::developer::video(r.config(&FX_VOLUME).int(1, 0, 2));
    crate::developer::video(r.config(&FX_SHAFTS).int(1, 0, 2));
    crate::developer::video(r.config(&DEFAULTS).int(0, 0, 9));
    crate::developer::video(r.config(&FX_BLOOM).int(1, 0, 2));
    // Fantasy: Subtle's values by default (Research round 17; the master defaults to Subtle).
    crate::developer::video(r.config(&FANTASY).int(1, 0, 3));
    crate::developer::video(r.config(&FX_FIREFLIES).int(1, 0, 2));
    crate::developer::video(r.config(&FX_WIND_LIFE).int(1, 0, 3));
    crate::developer::video(r.config(&FX_MOTES).int(1, 0, 1));
    crate::developer::video(r.config(&LOOK_STORYBOOK).float(0.4, 0.0, 1.0));
    crate::developer::video(r.config(&FX_NIGHT_SKY).int(1, 0, 2));
    crate::developer::video(r.config(&FX_SHOOTING_STARS).int(1, 0, 1));
    crate::developer::video(r.config(&MOON_SIZE).float(1.0, 1.0, 2.0));
    crate::developer::video(r.config(&FX_GLOW).float(1.0, 0.0, 1.5));
    crate::developer::video(r.config(&FX_BIOME_AIR).float(0.6, 0.0, 1.0));
    crate::developer::video(r.config(&FX_SKY_EVENTS).int(2, 0, 2));
    crate::developer::video(r.config(&FX_SPARKLE).int(1, 0, 1));
    crate::developer::video(r.config(&DETECTED).int(-1, -1, 5));
    crate::developer::video(r.config(&DETECTED_HALF).bool(false));
    crate::developer::video(r.config(&RENDER_SCALE_NEXT).float(-1.0, -1.0, 1.0));
    crate::developer::video(r.config(&SHADOW_SIZE_NEXT).int(0, 0, 4096));
    r.client_frame(|_| new_defaults());
    // The first run: a preset from the screen (a high-density one draws many more pixels).
    r.client_frame(|c| {
        if QUALITY.get() >= 0 {
            return;
        }
        // Scripts, tests and benchmarks keep their own settings (`--config` sets them per run),
        // unless one pretends a GPU to check this (`--fake-gpu`) or plays a player's start
        // (`--player-start`, with an empty `--run-dir` for a first start).
        let fake = ["fake-gpu", "player-start"]
            .iter()
            .any(|o| weft_sdk::client::launch_option(o).is_some());
        if !fake
            && ["script", "test", "config"]
                .iter()
                .any(|o| weft_sdk::client::launch_option(o).is_some())
        {
            return;
        }
        let [w, h] = c.state().window;
        if w == 0 || h == 0 {
            return;
        }
        apply_detected([w, h]);
    });
    r.client_frame(take_waiting);
    // What the HDR world and anti-aliasing started as: their rows say when a change waits.
    r.client_frame(|_| {
        AT_START.get_or_init(|| (HDR_WORLD.get(), AA.get()));
    });
    r.menu(&SCREEN).screen(|ui| {
        let tabs = tabs("video.base.title");
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.title", &tabs, |ui| {
            crate::video_costs::top_line(ui, &SCREEN, || measured_rows(crate::video_costs::Tab::Video), after_restart);
            // The presets, and which one the settings are now.
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().w(84.0).no_wrap(), "video.base.quality", &[]);
                // The preset the settings are now stands pressed (amber); none while Custom.
                let preset = |ui: &mut Ui, name: &str, p: usize| {
                    let on = QUALITY.get() == p as i32;
                    ui.button(&format!("video.base.preset.{name}"), S::default().w(52.0).selected(on), move |c| {
                        quality_changing(c, || apply(p))
                    });
                };
                if on_phone() {
                    preset(ui, "phone", PHONE);
                }
                for (p, name) in PRESET_NAMES.iter().enumerate() {
                    preset(ui, name, p);
                }
                preset(ui, "ultra", ULTRA);
            });
            ui.row(S::row().gap(3.0), |ui| {
                ui.row(S::row().w(84.0), |_| {});
                ui.text_bind_styled(S::default().class("dim").no_wrap(), |_, t| {
                    let name = preset_name(QUALITY.get()).unwrap_or("custom");
                    t.set("video.base.now", &[Arg::Key(&format!("video.base.preset.{name}"))]);
                });
            });
            // What the GPU is, what it chose, and choosing again (Research round 19, 7).
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.row(S::row().w(84.0), |_| {});
                ui.text_bind_styled(S::default().class("dim").max_w(260.0), |_, t| {
                    let name = GPU_NAME.get();
                    let class = ["unknown", "integrated", "discrete", "other"][gpu_class().clamp(0, 3) as usize];
                    let shown = if name.is_empty() { "?".to_string() } else { name };
                    match preset_name(DETECTED.get()) {
                        Some(p) => {
                            let key = if DETECTED_HALF.get() { "video.base.detected.half" } else { "video.base.detected.preset" };
                            t.set(
                                key,
                                &[
                                    Arg::Text(&shown),
                                    Arg::Key(&format!("video.base.gpu.{class}")),
                                    Arg::Key(&format!("video.base.preset.{p}")),
                                ],
                            );
                        }
                        None => t.set("video.base.detected.gpu", &[Arg::Text(&shown), Arg::Key(&format!("video.base.gpu.{class}"))]),
                    }
                });
                ui.button("video.base.detect", S::default().h(12.0).pad_xy(5.0, 2.0), |c| {
                    quality_changing(c, || apply_detected(c.client().state().window))
                });
            });
            // Each feature's quality, Low to Ultra (what a preset sets), then the player's own rows,
            // scrolled: they don't fit a 720p window at 2x.
            ui.column(S::column().gap(3.0).h(196.0).scroll(), |ui| {
                for i in 0..FEATURE_ROWS.len() {
                    row_ui(ui, &FEATURE_ROWS, i, step_feature);
                }
                for i in 0..ROWS.len() {
                    row_ui(ui, &ROWS, i, step);
                }
            });
        });
    });
    // The Developer screen: shown only with `base:developer_menu` on (its tab, `tabs`); a menu
    // opened another way says so and shows nothing.
    r.menu(&DEV_SCREEN).screen(|ui| {
        let tabs = tabs("video.base.developer.title");
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.developer.title", &tabs, |ui| {
            if !crate::developer::MENU.get() {
                ui.text_styled(S::default().class("dim").max_w(320.0), "video.base.developer.off", &[]);
                return;
            }
            crate::video_costs::top_line(ui, &DEV_SCREEN, || measured_rows(crate::video_costs::Tab::Extra), after_restart);
            ui.column(S::column().gap(3.0).h(240.0).scroll(), |ui| {
                ui.text_styled(S::default().class("dim").max_w(320.0), "video.base.developer.parts", &[]);
                for i in 0..PARTS.len() {
                    // Traced shadows, reflections and the denoiser only where the GPU traces rays.
                    if traced(PARTS[i].key) && !GPU_RAY_QUERY.get() {
                        continue;
                    }
                    row_ui(ui, &PARTS, i, step_part);
                }
                crate::developer::knob_rows(ui);
            });
        });
    });
    r.menu(&FANTASY_SCREEN).screen(|ui| {
        let tabs = tabs("video.base.fantasy.title");
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.fantasy.title", &tabs, |ui| {
            crate::video_costs::top_line(ui, &FANTASY_SCREEN, || measured_rows(crate::video_costs::Tab::Fantasy), after_restart);
            // The master, and which of its looks the rows are now.
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().w(84.0).no_wrap(), "video.base.fantasy", &[]);
                for (f, name) in FANTASY_NAMES.iter().enumerate() {
                    ui.button(&format!("video.base.fantasy.{name}"), S::default().w(70.0), move |_| apply_fantasy(f));
                }
            });
            ui.row(S::row().gap(3.0), |ui| {
                ui.row(S::row().w(84.0), |_| {});
                ui.text_bind_styled(S::default().class("dim").no_wrap(), |_, t| {
                    let name = FANTASY_NAMES
                        .get(FANTASY.get() as usize)
                        .copied()
                        .unwrap_or("custom");
                    t.set("video.base.now", &[Arg::Key(&format!("video.base.fantasy.{name}"))]);
                });
            });
            // Scrolled as the Video tab's: in a long language the rows pass a 720p window at 2x
            // (the text-fit check, pseudo-locale: 6.5 GUI px off the screen).
            ui.column(S::column().gap(3.0).h(196.0).scroll(), |ui| {
                for i in 0..FANTASY_ROWS.len() {
                    row_ui(ui, &FANTASY_ROWS, i, step_fantasy);
                }
            });
        });
    });
    // On the settings screen, beside the engine's own.
    r.screen_part(&screens::SETTINGS, "buttons", 0, |ui| {
        ui.button("video.base.open", S::column().min_w(200.0).self_align(abi::ui_align::CENTER), |c| {
            c.client().menu_open(&SCREEN);
        });
    });
}

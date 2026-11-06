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
/// The pack's screen effects: 0 off, 1 low (fewer steps), 2 on. Bloom has no low.
pub static FX_REFLECT: Config<i32> = Config::named("fx_reflect");
pub static FX_CONTACT: Config<i32> = Config::named("fx_contact");
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

/// The GPU as the engine found it (`core:gpu_*`): its name, its kind (0 not known, 1 integrated,
/// 2 discrete, 3 another), its own memory in MB.
static GPU_NAME: TextConfig = TextConfig::named("core:gpu_name");
static GPU_CLASS: Config<i32> = Config::named("core:gpu_class");
static GPU_MEMORY: Config<i32> = Config::named("core:gpu_memory_mb");
/// The GPU can trace rays (`core:gpu_ray_query`): the traced shadows' row is shown.
static GPU_RAY_QUERY: Config<bool> = Config::named("core:gpu_ray_query");
static RT_SHADOWS: Config<i32> = Config::named("core:rt_shadows");
/// The newer quality settings, each a row on the Extra quality tab (`EXTRA_ROWS`) and all at their
/// most in the Max preset: the HDR world and temporal anti-aliasing (read at start), the mist as a
/// volume, the soft clouds' shadow, the bloom's levels, the far shadow map's reach, the screen
/// effects' half-size parts (off: every effect at full size), the leaves' see-through holes.
static HDR_WORLD: Config<bool> = Config::named("core:hdr_world");
static AA: Config<i32> = Config::named("core:aa");
static FROXELS: Config<i32> = Config::named("core:froxels");
static CLOUD_SHADOWS: Config<bool> = Config::named("core:cloud_shadows");
static BLOOM_LEVELS: Config<i32> = Config::named("core:bloom");
static SHADOW_FAR: Config<i32> = Config::named("core:shadow_far");
static POST_LOW: Config<bool> = Config::named("core:post_low");
/// DLSS where the renderer has it (windows-dlss5's `dlss.rs`, its own tab): the modes it offers
/// (bit 1 DLAA; 0 none, also before that branch is in), whether Ray Reconstruction runs, the mode
/// (0 off, 1 DLAA), DLAA's model and Ray Reconstruction on.
static GPU_DLSS_MODES: Config<i32> = Config::named("core:gpu_dlss_modes");
static GPU_DLSS_RR: Config<bool> = Config::named("core:gpu_dlss_rr");
static DLSS: Config<i32> = Config::named("core:dlss");
static DLSS_MODEL_DLAA: Config<i32> = Config::named("core:dlss_model_dlaa");
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

/// The presets, Low to Ultra, per row (in `ROWS` order). Contact shadows on at Ultra only and
/// light shafts low below it (Ronald, 11-05, on the speed Researcher's Mac numbers: contact
/// shadows cost ~2.2 ms under trees, full shafts ~1.1-1.5 ms more than low).
const PRESETS: [[f64; 12]; 4] = [
    // scale shadows size  reflect contact volume shafts bloom view lod   vsync cap
    [0.5, 0.0, 1024.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.0, 64.0, 1.0, 60.0],
    [0.67, 1.0, 1024.0, 1.0, 0.0, 1.0, 1.0, 1.0, 8.0, 128.0, 1.0, 60.0],
    [0.0, 2.0, 2048.0, 2.0, 0.0, 2.0, 1.0, 1.0, 12.0, 192.0, 1.0, 0.0],
    [1.0, 2.0, 4096.0, 2.0, 2.0, 2.0, 2.0, 1.0, 16.0, 256.0, 1.0, 0.0],
];
const PRESET_NAMES: [&str; 4] = ["low", "medium", "high", "ultra"];

/// The Phone preset (`QUALITY` 5; Ronald, 11-06: "Yes, make the preset"), offered on a phone and
/// chosen only by the player, from his Pixel 10's sweep (notes/ANDROID.md): Low's view, distant
/// terrain and no sun shadows (the vista 17 -> 23 fps, the forest 18 -> 27 against Medium at half
/// scale), with Medium's screen effects kept: on the phone they cost about 2 ms.
const PHONE: usize = 5;
const PHONE_PRESET: [f64; 12] = [0.5, 0.0, 1024.0, 1.0, 0.0, 1.0, 1.0, 1.0, 6.0, 64.0, 1.0, 60.0];

/// The Max preset (`QUALITY` 6; Ronald, 11-06: "I want to test with always sort of the maximum
/// settings on because that's sort of our benchmark"): every row at its most (Ultra's, a view of
/// 32 chunks, Bliss's, and distant terrain to 512), then every extra row too (`max_extras`), as far
/// as the GPU can: traced shadows only where it traces rays, temporal anti-aliasing from the
/// camera alone on a phone. Never picked for the player. The PC queue's `--preset max`
/// (tester-tools/t2/t2q.py `PRESETS`) is the same list: change both together.
const MAX: usize = 6;
const MAX_PRESET: [f64; 12] = [1.0, 2.0, 4096.0, 2.0, 2.0, 2.0, 2.0, 1.0, 32.0, 512.0, 1.0, 0.0];

/// Preset `q`'s values, Low to Ultra, Phone and Max.
fn preset(q: usize) -> Option<&'static [f64; 12]> {
    match q {
        PHONE => Some(&PHONE_PRESET),
        MAX => Some(&MAX_PRESET),
        _ => PRESETS.get(q),
    }
}

/// The extra rows at their most, as far as this GPU goes (`MAX`), and the fantasy rows that cost
/// (wind life ultra, soft clouds).
fn max_extras() {
    HDR_WORLD.set(true);
    // Temporal anti-aliasing with the motion image; from the camera alone on a phone, whose tiled
    // GPU the motion image cost a third of its world pass (Android, Pixel 10, 11-06). (DLAA in its
    // place where DLSS is, once it lands.)
    AA.set(if on_phone() { 2 } else { 1 });
    FROXELS.set(2);
    CLOUD_SHADOWS.set(true);
    BLOOM_LEVELS.set(2);
    // Auto: 2000 blocks with the HDR world (Far view's, at no measurable cost); further only
    // spreads the far map's texels thinner.
    SHADOW_FAR.set(0);
    POST_LOW.set(false);
    crate::look::LOOK_LEAF_HOLES.set(1.0);
    // Ultra (soft edges from more rays, with TAA) where the GPU traces rays.
    RT_SHADOWS.set(if GPU_RAY_QUERY.get() && !on_phone() { 2 } else { 0 });
    FX_WIND_LIFE.set(3);
    SOFT_CLOUDS.set(true);
    // DLAA (DLSS at the screen's own size) where the renderer has it, with its newest model (M,
    // 13) and Ray Reconstruction where that runs (windows-dlss5, 11-06): the engine then skips its
    // own gathering for the frame. Elsewhere TAA as above.
    if GPU_DLSS_MODES.get() & (1 << 1) != 0 {
        DLSS.set(1);
        DLSS_MODEL_DLAA.set(13);
        DLSS_RR.set(GPU_DLSS_RR.get());
    }
}

/// Preset `q`'s name (`video.base.preset.<name>`); None for Custom and not chosen.
fn preset_name(q: i32) -> Option<&'static str> {
    if q == PHONE as i32 {
        Some("phone")
    } else if q == MAX as i32 {
        Some("max")
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

/// The rows the presets set (`PRESETS`, `PHONE_PRESET`): the first 12 of `ROWS`. The upscaler after
/// them is the player's own choice, opt-in (Ronald, 11-06), so no preset changes it and it never
/// makes the quality Custom.
const PRESET_ROWS: usize = 12;

const ROWS: [Row; 13] = [
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
        // Soft, or dreamy (Storybook's): halos and bloomy fog (Research round 17).
        values: &[0.0, 1.0, 2.0],
        get: || FX_BLOOM.get() as f64,
        set: |v| FX_BLOOM.set(v as i32),
        shown: |v| word(["off", "soft", "dreamy"][(v as usize).min(2)]),
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
    Row {
        key: "upscaler",
        values: &[0.0, 1.0, 2.0],
        get: || UPSCALER.get() as f64,
        set: |v| UPSCALER.set(v as i32),
        shown: |v| word(["smooth", "fsr1", "gsr1"][(v as usize).min(2)]),
    },
];

fn off_low_on(v: f64) -> (String, i64) {
    word(["off", "low", "on"][(v as usize).min(2)])
}

/// Sets every row to preset `p` (0 low .. 3 ultra), and the fantasy rows that cost to what it
/// allows (`fantasy_caps`).
fn apply(p: usize) {
    let Some(values) = preset(p) else { return };
    for (row, &v) in ROWS.iter().zip(values.iter()) {
        (row.set)(v);
    }
    QUALITY.set(p as i32);
    if p == MAX {
        max_extras();
    } else if GPU_DLSS_MODES.get() != 0 {
        // DLSS is live: off again with any other preset, as Max is the one that turns it on.
        DLSS.set(0);
    }
    fantasy_caps(p, true);
}

/// One row a step on (`dir` 1) or back (-1) through its values; the quality becomes Custom.
fn step(i: usize, dir: i32) {
    step_row(&ROWS[i], dir);
    if i < PRESET_ROWS {
        QUALITY.set(4);
    }
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
pub static FANTASY_SCREEN: Screen = Screen::named("video_fantasy");
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
/// fireflies, no motes, a subtle night sky, no sparkle, no bloom; Medium soft bloom. Soft clouds
/// cost no more than blocky (Lighting, on the PC and the Mac), so they follow the master alone. With
/// `bloom_up`, High and Ultra take the master's bloom (a quality preset just chosen).
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
        FX_BLOOM.set(0);
    } else if q == 1 {
        FX_BLOOM.set(FX_BLOOM.get().min(1));
    } else if bloom_up && (q <= 3 || q == MAX) {
        if let Some(&b) = FANTASY_BLOOM.get(FANTASY.get() as usize) {
            FX_BLOOM.set(b);
        }
    }
}

/// A row's line on a settings screen: its name, ‹ value ›, and what it does (or that it waits
/// for a restart).
fn row_ui(ui: &mut Ui, rows: &'static [Row], i: usize, step: fn(usize, i32)) {
    let row = &rows[i];
    ui.row(S::row().gap(3.0).center(), move |ui| {
        ui.text_styled(S::default().w(84.0).no_wrap(), &format!("video.base.{}", row.key), &[]);
        ui.button("video.base.less", S::default().size(12.0, 12.0).pad(0.5), move |_| step(i, -1));
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

/// The Extra quality tab's rows (`EXTRA_SCREEN`): the newer features, each a setting of its own,
/// outside Low to Ultra so no preset changes the look by them; Max sets them all (`max_extras`).
/// Traced sun and moon shadows only where the GPU can trace rays (the last row, `RT_ROW`).
const EXTRA_ROWS: [Row; 9] = [
    Row { key: "hdr", values: &[0.0, 1.0], get: || HDR_WORLD.get() as u8 as f64, set: |v| HDR_WORLD.set(v > 0.0), shown: off_on },
    Row {
        key: "aa",
        values: &[0.0, 1.0, 2.0],
        get: || AA.get() as f64,
        set: |v| AA.set(v as i32),
        shown: |v| word(["off", "taa", "taa_camera"][(v as usize).min(2)]),
    },
    Row {
        key: "mist_volume",
        values: &[0.0, 1.0, 2.0],
        get: || FROXELS.get() as f64,
        set: |v| FROXELS.set(v as i32),
        shown: |v| word(["off", "on", "ultra"][(v as usize).min(2)]),
    },
    Row {
        key: "cloud_shadows",
        values: &[0.0, 1.0],
        get: || CLOUD_SHADOWS.get() as u8 as f64,
        set: |v| CLOUD_SHADOWS.set(v > 0.0),
        shown: off_on,
    },
    Row {
        key: "bloom_levels",
        values: &[0.0, 1.0, 2.0],
        get: || BLOOM_LEVELS.get() as f64,
        set: |v| BLOOM_LEVELS.set(v as i32),
        shown: |v| word(["off", "narrow", "wide"][(v as usize).min(2)]),
    },
    Row {
        key: "shadow_far",
        values: &[0.0, 400.0, 1000.0, 2000.0, 2400.0],
        get: || SHADOW_FAR.get() as f64,
        set: |v| SHADOW_FAR.set(v as i32),
        shown: |v| if v == 0.0 { word("auto") } else { ("blocks".into(), v as i64) },
    },
    Row {
        key: "effect_detail",
        // The half-size parts on (1) or every effect at full size (0).
        values: &[1.0, 0.0],
        get: || POST_LOW.get() as u8 as f64,
        set: |v| POST_LOW.set(v > 0.0),
        shown: |v| word(if v > 0.0 { "half" } else { "full" }),
    },
    Row {
        key: "leaf_holes",
        values: &[0.0, 1.0],
        get: || (crate::look::LOOK_LEAF_HOLES.get() > 0.5) as u8 as f64,
        set: |v| crate::look::LOOK_LEAF_HOLES.set(if v > 0.0 { 1.0 } else { 0.0 }),
        shown: off_on,
    },
    Row {
        key: "rt_shadows",
        // Ultra: soft edges from more rays a pixel (with TAA), about three times the cost.
        values: &[0.0, 1.0, 2.0],
        get: || RT_SHADOWS.get() as f64,
        set: |v| RT_SHADOWS.set(v as i32),
        shown: |v| word(["off", "on", "ultra"][(v as usize).min(2)]),
    },
];
const RT_ROW: usize = 8;

/// An extra row a step on: Max becomes Custom (the Low to Ultra presets leave these rows as they are).
fn step_extra(i: usize, dir: i32) {
    step_row(&EXTRA_ROWS[i], dir);
    if QUALITY.get() == MAX as i32 {
        QUALITY.set(4);
    }
}

pub static EXTRA_SCREEN: Screen = Screen::named("video_extra");

fn step_fantasy(i: usize, dir: i32) {
    step_row(&FANTASY_ROWS[i], dir);
    FANTASY.set(3);
}

/// Which defaults the saved settings have caught up with (`new_defaults`).
static DEFAULTS: Config<i32> = Config::named("video_defaults");

/// Saved settings take the new defaults once (the engine saves every setting, so a changed
/// default alone reaches no one who has played): on a preset, its values for the rows whose
/// defaults changed; on Custom, a row still at its old default takes the new one, a row the
/// player changed stays. 1: contact shadows off, light shafts low (11-05). 2: temporal
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
    let q = QUALITY.get();
    let (contact, shafts) = match usize::try_from(q).ok().and_then(preset) {
        Some(p) => (p[4] as i32, p[6] as i32),
        None => (if FX_CONTACT.get() == 2 { 0 } else { FX_CONTACT.get() }, if FX_SHAFTS.get() == 2 { 1 } else { FX_SHAFTS.get() }),
    };
    weft_sdk::info!("video: new defaults, contact shadows {} -> {contact}, light shafts {} -> {shafts}", FX_CONTACT.get(), FX_SHAFTS.get());
    FX_CONTACT.set(contact);
    FX_SHAFTS.set(shafts);
    DEFAULTS.set(1);
    // And the next step at once (`DEFAULTS` 2).
    new_defaults();
}

pub fn register(r: &mut Registry) {
    r.config(&QUALITY).int(-1, -1, 6);
    r.config(&FX_REFLECT).int(2, 0, 2);
    // Off by default, low shafts (Ronald, 11-05: "Off by default", "Default to Low").
    r.config(&FX_CONTACT).int(0, 0, 2);
    // Cheap (Tester 2's PC bench, 11-04: +0.04 ms of effects at 1280x720): low until a preset
    // says otherwise; off on the Low preset until the Mac's Retina numbers are in.
    r.config(&FX_VOLUME).int(1, 0, 2);
    r.config(&FX_SHAFTS).int(1, 0, 2);
    r.config(&DEFAULTS).int(0, 0, 9);
    r.config(&FX_BLOOM).int(1, 0, 2);
    // Fantasy: Subtle's values by default (Research round 17; the master defaults to Subtle).
    r.config(&FANTASY).int(1, 0, 3);
    r.config(&FX_FIREFLIES).int(1, 0, 2);
    r.config(&FX_WIND_LIFE).int(1, 0, 3);
    r.config(&FX_MOTES).int(1, 0, 1);
    r.config(&LOOK_STORYBOOK).float(0.4, 0.0, 1.0);
    r.config(&FX_NIGHT_SKY).int(1, 0, 2);
    r.config(&FX_SHOOTING_STARS).int(1, 0, 1);
    r.config(&MOON_SIZE).float(1.0, 1.0, 2.0);
    r.config(&FX_GLOW).float(1.0, 0.0, 1.5);
    r.config(&FX_BIOME_AIR).float(0.6, 0.0, 1.0);
    r.config(&FX_SKY_EVENTS).int(2, 0, 2);
    r.config(&FX_SPARKLE).int(1, 0, 1);
    r.config(&DETECTED).int(-1, -1, 5);
    r.config(&DETECTED_HALF).bool(false);
    r.config(&RENDER_SCALE_NEXT).float(-1.0, -1.0, 1.0);
    r.config(&SHADOW_SIZE_NEXT).int(0, 0, 4096);
    r.client_frame(|_| new_defaults());
    // The first run: a preset from the screen (a high-density one draws many more pixels).
    r.client_frame(|c| {
        if QUALITY.get() >= 0 {
            return;
        }
        // Scripts, tests and benchmarks keep their own settings (`--config` sets them per run),
        // unless one pretends a GPU to check this (`--fake-gpu`).
        let fake = weft_sdk::client::launch_option("fake-gpu").is_some();
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
    r.screen(&SCREEN, |ui| {
        let mut tabs: Vec<crate::window::Tab> = vec![
            ("video.base.fantasy.title", |c| {
                c.client().screen_open(&FANTASY_SCREEN);
            }),
            ("video.base.extra.title", |c| {
                c.client().screen_open(&EXTRA_SCREEN);
            }),
        ];
        // DLSS's tab, where the renderer has it.
        if crate::dlss::available() {
            tabs.push(("video.base.dlss.title", |c| {
                c.client().screen_open(&crate::dlss::SCREEN);
            }));
        }
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.title", &tabs, |ui| {
            // The presets, and which one the settings are now.
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().w(84.0).no_wrap(), "video.base.quality", &[]);
                if on_phone() {
                    ui.button("video.base.preset.phone", S::default().w(52.0), move |_| apply(PHONE));
                }
                for (p, name) in PRESET_NAMES.iter().enumerate() {
                    ui.button(&format!("video.base.preset.{name}"), S::default().w(52.0), move |_| apply(p));
                }
                ui.button("video.base.preset.max", S::default().w(52.0), move |_| apply(MAX));
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
                ui.button("video.base.detect", S::default().h(12.0).pad(2.0), |c| apply_detected(c.client().state().window));
            });
            for i in 0..ROWS.len() {
                row_ui(ui, &ROWS, i, step);
            }
        });
    });
    r.screen(&EXTRA_SCREEN, |ui| {
        let mut tabs: Vec<crate::window::Tab> = vec![
            ("video.base.title", |c| {
                c.client().screen_open(&SCREEN);
            }),
            ("video.base.fantasy.title", |c| {
                c.client().screen_open(&FANTASY_SCREEN);
            }),
        ];
        if crate::dlss::available() {
            tabs.push(("video.base.dlss.title", |c| {
                c.client().screen_open(&crate::dlss::SCREEN);
            }));
        }
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.extra.title", &tabs, |ui| {
            for i in 0..EXTRA_ROWS.len() {
                // Traced shadows only where the GPU traces rays.
                if i == RT_ROW && !GPU_RAY_QUERY.get() {
                    continue;
                }
                row_ui(ui, &EXTRA_ROWS, i, step_extra);
            }
        });
    });
    r.screen(&FANTASY_SCREEN, |ui| {
        let mut tabs: Vec<crate::window::Tab> = vec![
            ("video.base.title", |c| {
                c.client().screen_open(&SCREEN);
            }),
            ("video.base.extra.title", |c| {
                c.client().screen_open(&EXTRA_SCREEN);
            }),
        ];
        if crate::dlss::available() {
            tabs.push(("video.base.dlss.title", |c| {
                c.client().screen_open(&crate::dlss::SCREEN);
            }));
        }
        crate::window::window_titled(ui, S::column().pad(8.0).gap(3.0), "video.base.fantasy.title", &tabs, |ui| {
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
            for i in 0..FANTASY_ROWS.len() {
                row_ui(ui, &FANTASY_ROWS, i, step_fantasy);
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

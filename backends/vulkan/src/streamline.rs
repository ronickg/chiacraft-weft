//! NVIDIA DLSS through Streamline (the `streamline` feature; Windows only at run time).
//!
//! Our own bindings to Streamline 2.14.16's C API, written from its public headers
//! (github.com/NVIDIAGameWorks/Streamline, MIT). Nothing of NVIDIA's is in this repository:
//! `sl.interposer.dll`, its plugins (`sl.common.dll`, `sl.dlss.dll`, `sl.dlss_d.dll`) and DLSS
//! itself (`nvngx_dlss.dll`, `nvngx_dlssd.dll`) are loaded at run time from a folder: the
//! `WEFT_STREAMLINE_DIR` variable, or `streamline/` beside the game. Without that folder the game
//! starts as always. `WEFT_NO_STREAMLINE=1` skips it even when present.
//!
//! How it fits in (Streamline's Programming Guide, "Vulkan"): `slInit` runs before the Vulkan
//! instance, and the game takes Vulkan from `sl.interposer.dll` instead of `vulkan-1.dll`, so
//! Streamline adds the extensions and features DLSS needs to our instance and device itself.
//! Each `Cmd::Upscale` then sets the frame's constants and options and evaluates DLSS (or Ray
//! Reconstruction) on the frame's command buffer, with its images tagged for that call only.
//!
//! DLSS 5 (`kFeatureDLSS_NR` = 1004) is reserved in the public header but has no plugin, header or
//! guide in the public SDK (11-06): `UpscalerInfo::neural` stays false until NVIDIA gives Ronald
//! access and its options are bound here.
//!
//! The models offered (`UpscalerInfo::models`), from `sl_dlss.h` and NVIDIA's DLSS Programming
//! Guide (2026-07, 3.12): E and F the older CNN models (deprecated), J and K the first transformer
//! model (K NVIDIA's default for DLAA, Quality and Balanced), L and M the second (L the default
//! for Ultra Performance, M for Performance); G, H, I, N and O are left out (they fall back to the
//! default). Ray Reconstruction (`sl_dlss_d.h`): D, E and F, transformer models, F the latest and
//! default.

use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

use ash::vk;
use ash::vk::Handle;
use weft_render::{UpscaleMode, UpscaleModel, UpscaleParams, UpscalerInfo};

use crate::upscale::{Image, Inputs, Upscaler};

/// The SDK version these bindings follow (`sl_version.h`): 2.14.16.
const SDK_VERSION: u64 = (2u64 << 48) | (14u64 << 32) | (16u64 << 16) | 0xfedc;

/// chiacraft's id for NVIDIA (NGX's project id for a custom engine).
const PROJECT_ID: &CStr = c"f264d7e7-3c55-4a0e-90d7-35933f37130f";
const ENGINE_VERSION: &CStr = c"26.11";

const FEATURE_DLSS: u32 = 0;
const FEATURE_DLSS_RR: u32 = 1001;

const OK: u32 = 0;

#[cfg(windows)]
type WChar = u16;
#[cfg(not(windows))]
type WChar = u32;

#[repr(C)]
#[derive(Clone, Copy)]
struct StructType(u32, u16, u16, [u8; 8]);

#[repr(C)]
struct Base {
    next: *mut Base,
    ty: StructType,
    version: usize,
}

impl Base {
    fn new(ty: StructType, version: usize) -> Base {
        Base { next: null_mut(), ty, version }
    }
}

const T_PREFERENCES: StructType = StructType(0x1ca10965, 0xbf8e, 0x432b, [0x8d, 0xa1, 0x67, 0x16, 0xd8, 0x79, 0xfb, 0x14]);
const T_RESOURCE: StructType = StructType(0x3a9d70cf, 0x2418, 0x4b72, [0x83, 0x91, 0x13, 0xf8, 0x72, 0x1c, 0x72, 0x61]);
const T_RESOURCE_TAG: StructType = StructType(0x4c6a5aad, 0xb445, 0x496c, [0x87, 0xff, 0x1a, 0xf3, 0x84, 0x5b, 0xe6, 0x53]);
const T_VIEWPORT: StructType = StructType(0x171b6435, 0x9b3c, 0x4fc8, [0x99, 0x94, 0xfb, 0xe5, 0x25, 0x69, 0xaa, 0xa4]);
const T_CONSTANTS: StructType = StructType(0xdcd35ad7, 0x4e4a, 0x4bad, [0xa9, 0x0c, 0xe0, 0xc4, 0x9e, 0xb2, 0x3a, 0xfe]);
const T_DLSS_OPTIONS: StructType = StructType(0x6ac826e4, 0x4c61, 0x4101, [0xa9, 0x2d, 0x63, 0x8d, 0x42, 0x10, 0x57, 0xb8]);
const T_DLSS_OPTIMAL: StructType = StructType(0xef1d0957, 0xfd58, 0x4df7, [0xb5, 0x04, 0x8b, 0x69, 0xd8, 0xaa, 0x6b, 0x76]);
const T_DLSSD_OPTIONS: StructType = StructType(0x0ad87504, 0x774e, 0x4bf3, [0x96, 0x33, 0xa4, 0x4d, 0x1f, 0x7f, 0x9c, 0xb8]);
const T_DLSSD_OPTIMAL: StructType = StructType(0xfbd0c637, 0xa28f, 0x41f2, [0xbc, 0x91, 0xb4, 0x21, 0xfa, 0xee, 0x8e, 0x1e]);
const T_ADAPTER: StructType = StructType(0x0677315f, 0xa746, 0x4492, [0x9f, 0x42, 0xcb, 0x61, 0x42, 0xc9, 0xc3, 0xd4]);
const T_FEATURE_VERSION: StructType = StructType(0x6d5b51f0, 0x076b, 0x486d, [0x99, 0x95, 0x5a, 0x56, 0x10, 0x43, 0xf5, 0xc1]);

/// `sl::Preferences` (version 1).
#[repr(C)]
struct Preferences {
    base: Base,
    show_console: bool,
    log_level: u32,
    paths_to_plugins: *const *const WChar,
    num_paths_to_plugins: u32,
    path_to_logs_and_data: *const WChar,
    allocate_callback: *const c_void,
    release_callback: *const c_void,
    log_message_callback: Option<unsafe extern "C" fn(u32, *const c_char)>,
    flags: u64,
    features_to_load: *const u32,
    num_features_to_load: u32,
    application_id: u32,
    engine: u32,
    engine_version: *const c_char,
    project_id: *const c_char,
    render_api: u32,
}

/// `sl::Resource` (version 1): a Vulkan image as Streamline needs it described.
#[repr(C)]
struct Resource {
    base: Base,
    ty: u8,
    native: *mut c_void,
    memory: *mut c_void,
    view: *mut c_void,
    /// The image's layout now (`VkImageLayout`).
    state: u32,
    width: u32,
    height: u32,
    native_format: u32,
    mip_levels: u32,
    array_layers: u32,
    gpu_virtual_address: u64,
    flags: u32,
    usage: u32,
    internal_flags: u16,
    reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Extent {
    top: u32,
    left: u32,
    width: u32,
    height: u32,
}

/// `sl::ResourceTag` (version 1).
#[repr(C)]
struct ResourceTag {
    base: Base,
    resource: *mut Resource,
    ty: u32,
    lifecycle: u32,
    extent: Extent,
}

/// `sl::ViewportHandle`: we draw one view, 0.
#[repr(C)]
struct Viewport {
    base: Base,
    value: u32,
}

type Float4x4 = [f32; 16];

/// `sl::Constants` (version 2). `sl::Boolean`s are bytes: 0 false, 1 true.
#[repr(C)]
struct Constants {
    base: Base,
    camera_view_to_clip: Float4x4,
    clip_to_camera_view: Float4x4,
    clip_to_lens_clip: Float4x4,
    clip_to_prev_clip: Float4x4,
    prev_clip_to_clip: Float4x4,
    jitter_offset: [f32; 2],
    mvec_scale: [f32; 2],
    camera_pinhole_offset: [f32; 2],
    camera_pos: [f32; 3],
    camera_up: [f32; 3],
    camera_right: [f32; 3],
    camera_fwd: [f32; 3],
    camera_near: f32,
    camera_far: f32,
    camera_fov: f32,
    camera_aspect_ratio: f32,
    motion_vectors_invalid_value: f32,
    depth_inverted: u8,
    camera_motion_included: u8,
    motion_vectors_3d: u8,
    reset: u8,
    orthographic_projection: u8,
    motion_vectors_dilated: u8,
    motion_vectors_jittered: u8,
    min_relative_linear_depth_object_separation: f32,
}

/// `sl::DLSSOptions` (version 3).
#[repr(C)]
struct DlssOptions {
    base: Base,
    mode: u32,
    output_width: u32,
    output_height: u32,
    sharpness: f32,
    pre_exposure: f32,
    exposure_scale: f32,
    color_buffers_hdr: u8,
    indicator_invert_x: u8,
    indicator_invert_y: u8,
    dlaa_preset: u32,
    quality_preset: u32,
    balanced_preset: u32,
    performance_preset: u32,
    ultra_performance_preset: u32,
    ultra_quality_preset: u32,
    use_auto_exposure: u8,
    alpha_upscaling_enabled: u8,
}

/// `sl::DLSSDOptions` (Ray Reconstruction, version 3).
#[repr(C)]
struct DlssdOptions {
    base: Base,
    mode: u32,
    output_width: u32,
    output_height: u32,
    sharpness: f32,
    pre_exposure: f32,
    exposure_scale: f32,
    color_buffers_hdr: u8,
    indicator_invert_x: u8,
    indicator_invert_y: u8,
    normal_roughness_mode: u32,
    world_to_camera_view: Float4x4,
    camera_view_to_world: Float4x4,
    alpha_upscaling_enabled: u8,
    dlaa_preset: u32,
    quality_preset: u32,
    balanced_preset: u32,
    performance_preset: u32,
    ultra_performance_preset: u32,
    ultra_quality_preset: u32,
}

/// `sl::DLSSOptimalSettings` and `sl::DLSSDOptimalSettings` (version 1, the same fields).
#[repr(C)]
struct OptimalSettings {
    base: Base,
    optimal_render_width: u32,
    optimal_render_height: u32,
    optimal_sharpness: f32,
    render_width_min: u32,
    render_height_min: u32,
    render_width_max: u32,
    render_height_max: u32,
}

/// `sl::AdapterInfo` (version 1).
#[repr(C)]
struct AdapterInfo {
    base: Base,
    device_luid: *mut u8,
    device_luid_size: u32,
    vk_physical_device: *mut c_void,
}

/// `sl::FeatureVersion` (version 1).
#[repr(C)]
struct FeatureVersion {
    base: Base,
    sl: [u32; 3],
    ngx: [u32; 3],
}

// The sizes the C++ headers give on x86_64 (MSVC), checked where it is built.
const _: () = {
    use std::mem::size_of;
    assert!(size_of::<Base>() == 32);
    assert!(size_of::<Preferences>() == 144);
    assert!(size_of::<Resource>() == 112);
    assert!(size_of::<ResourceTag>() == 64);
    assert!(size_of::<Viewport>() == 40);
    assert!(size_of::<Constants>() == 456);
    assert!(size_of::<DlssOptions>() == 88);
    assert!(size_of::<DlssdOptions>() == 224);
    assert!(size_of::<OptimalSettings>() == 64);
    assert!(size_of::<AdapterInfo>() == 56);
    assert!(size_of::<FeatureVersion>() == 56);
};

/// The models offered for Super Resolution and DLAA, and for Ray Reconstruction (see the top).
const MODELS: [UpscaleModel; 7] = [
    UpscaleModel { id: 0, key: "default" },
    UpscaleModel { id: 5, key: "e" },
    UpscaleModel { id: 6, key: "f" },
    UpscaleModel { id: 10, key: "j" },
    UpscaleModel { id: 11, key: "k" },
    UpscaleModel { id: 12, key: "l" },
    UpscaleModel { id: 13, key: "m" },
];
const RR_MODELS: [UpscaleModel; 4] = [
    UpscaleModel { id: 0, key: "default" },
    UpscaleModel { id: 4, key: "rr_d" },
    UpscaleModel { id: 5, key: "rr_e" },
    UpscaleModel { id: 6, key: "rr_f" },
];

// `sl::DLSSMode`.
const DLSS_ULTRA_PERFORMANCE: u32 = 4;
const DLSS_MAX_PERFORMANCE: u32 = 1;
const DLSS_BALANCED: u32 = 2;
const DLSS_MAX_QUALITY: u32 = 3;
const DLSS_DLAA: u32 = 6;

// Buffer types (`sl_core_types.h`).
const TAG_DEPTH: u32 = 0;
const TAG_MOTION: u32 = 1;
const TAG_INPUT_COLOR: u32 = 3;
const TAG_OUTPUT_COLOR: u32 = 4;
const TAG_ALBEDO: u32 = 7;
const TAG_SPECULAR_ALBEDO: u32 = 8;
const TAG_SPECULAR_MOTION: u32 = 10;
const TAG_NORMAL_ROUGHNESS: u32 = 14;

/// `sl::ResourceLifecycle::eValidUntilEvaluate`: tagged for the one evaluate call.
const VALID_UNTIL_EVALUATE: u32 = 2;

type Result_ = u32;
type FnInit = unsafe extern "C" fn(*const Preferences, u64) -> Result_;
type FnShutdown = unsafe extern "C" fn() -> Result_;
type FnIsFeatureSupported = unsafe extern "C" fn(u32, *const AdapterInfo) -> Result_;
type FnSetConstants = unsafe extern "C" fn(*const Constants, *const c_void, *const Viewport) -> Result_;
type FnEvaluate = unsafe extern "C" fn(u32, *const c_void, *const *const Base, u32, *mut c_void) -> Result_;
type FnGetFeatureFunction = unsafe extern "C" fn(u32, *const c_char, *mut *mut c_void) -> Result_;
type FnNewFrameToken = unsafe extern "C" fn(*mut *mut c_void, *const u32) -> Result_;
type FnFeatureVersion = unsafe extern "C" fn(u32, *mut FeatureVersion) -> Result_;
type FnFreeResources = unsafe extern "C" fn(u32, *const Viewport) -> Result_;
type FnDlssOptimal = unsafe extern "C" fn(*const DlssOptions, *mut OptimalSettings) -> Result_;
type FnDlssSetOptions = unsafe extern "C" fn(*const Viewport, *const DlssOptions) -> Result_;
type FnDlssdOptimal = unsafe extern "C" fn(*const DlssdOptions, *mut OptimalSettings) -> Result_;
type FnDlssdSetOptions = unsafe extern "C" fn(*const Viewport, *const DlssdOptions) -> Result_;

/// What `slInit` and the interposer gave, before the Vulkan device exists.
pub(crate) struct Streamline {
    /// Kept loaded for the game's life (the interposer is also Vulkan's loader now).
    _lib: libloading::Library,
    pub interposer: PathBuf,
    shutdown: FnShutdown,
    is_supported: FnIsFeatureSupported,
    set_constants: FnSetConstants,
    evaluate: FnEvaluate,
    get_function: FnGetFeatureFunction,
    new_frame: FnNewFrameToken,
    feature_version: FnFeatureVersion,
    free_resources: FnFreeResources,
    // Kept alive while Streamline may read them.
    _plugin_dir: Vec<WChar>,
    _plugin_dirs: [*const WChar; 1],
    _features: Vec<u32>,
}

// The raw pointers are only Streamline's copies of our own strings; the game drives it from one
// thread at a time (the render thread).
unsafe impl Send for Streamline {}

fn wide(p: &Path) -> Vec<WChar> {
    p.to_string_lossy()
        .chars()
        .flat_map(|c| {
            let mut b = [0u16; 2];
            c.encode_utf16(&mut b)
                .iter()
                .map(|&u| u as WChar)
                .collect::<Vec<_>>()
        })
        .chain(std::iter::once(0))
        .collect()
}

unsafe extern "C" fn log_message(ty: u32, msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    let text = text.trim_end();
    match ty {
        2 => log::error!("streamline: {text}"),
        1 => log::warn!("streamline: {text}"),
        _ => log::info!("streamline: {text}"),
    }
}

/// The folder Streamline's files are in, if the game should use it.
fn folder() -> Option<PathBuf> {
    if !cfg!(windows) || std::env::var_os("WEFT_NO_STREAMLINE").is_some() {
        return None;
    }
    let dir = match std::env::var_os("WEFT_STREAMLINE_DIR") {
        Some(d) => PathBuf::from(d),
        None => std::env::current_exe().ok()?.parent()?.join("streamline"),
    };
    dir.join("sl.interposer.dll").is_file().then_some(dir)
}

impl Streamline {
    /// Loads Streamline and calls `slInit`, before anything of Vulkan. None (and the game goes on
    /// with plain Vulkan) when the files are not there or it fails.
    pub fn load() -> Option<Streamline> {
        let dir = folder()?;
        let interposer = dir.join("sl.interposer.dll");
        let lib = match unsafe { libloading::Library::new(&interposer) } {
            Ok(l) => l,
            Err(e) => {
                log::warn!("streamline: loading {}: {e}", interposer.display());
                return None;
            }
        };
        macro_rules! sym {
            ($name:literal, $ty:ty) => {
                match unsafe { lib.get::<$ty>($name) } {
                    Ok(f) => *f,
                    Err(e) => {
                        log::warn!("streamline: {}: {e}", String::from_utf8_lossy($name));
                        return None;
                    }
                }
            };
        }
        let init = sym!(b"slInit", FnInit);
        let mut s = Streamline {
            shutdown: sym!(b"slShutdown", FnShutdown),
            is_supported: sym!(b"slIsFeatureSupported", FnIsFeatureSupported),
            set_constants: sym!(b"slSetConstants", FnSetConstants),
            evaluate: sym!(b"slEvaluateFeature", FnEvaluate),
            get_function: sym!(b"slGetFeatureFunction", FnGetFeatureFunction),
            new_frame: sym!(b"slGetNewFrameToken", FnNewFrameToken),
            feature_version: sym!(b"slGetFeatureVersion", FnFeatureVersion),
            free_resources: sym!(b"slFreeResources", FnFreeResources),
            _lib: lib,
            interposer,
            _plugin_dir: wide(&dir),
            _plugin_dirs: [null()],
            _features: Vec::new(),
        };
        // Only the plugins that are there (`sl.dlss.dll` Super Resolution and DLAA,
        // `sl.dlss_d.dll` Ray Reconstruction).
        if dir.join("sl.dlss.dll").is_file() {
            s._features.push(FEATURE_DLSS);
        }
        if dir.join("sl.dlss_d.dll").is_file() {
            s._features.push(FEATURE_DLSS_RR);
        }
        s._plugin_dirs = [s._plugin_dir.as_ptr()];
        let verbose = std::env::var_os("WEFT_STREAMLINE_VERBOSE").is_some();
        let prefs = Preferences {
            base: Base::new(T_PREFERENCES, 1),
            show_console: false,
            log_level: if verbose { 2 } else { 1 },
            paths_to_plugins: s._plugin_dirs.as_ptr(),
            num_paths_to_plugins: 1,
            // Streamline's own log goes beside its files (sl.log), for Ronald's report.
            path_to_logs_and_data: s._plugin_dir.as_ptr(),
            allocate_callback: null(),
            release_callback: null(),
            log_message_callback: Some(log_message),
            // eDisableCLStateTracking: the backend binds again after each evaluate. No updates
            // over the air: the DLSS that was tested is the DLSS that runs.
            flags: 1,
            features_to_load: s._features.as_ptr(),
            num_features_to_load: s._features.len() as u32,
            application_id: 0,
            engine: 0,
            engine_version: ENGINE_VERSION.as_ptr(),
            project_id: PROJECT_ID.as_ptr(),
            render_api: 2,
        };
        let r = unsafe { init(&prefs, SDK_VERSION) };
        if r != OK {
            log::warn!("streamline: slInit failed ({r}), going on without DLSS");
            return None;
        }
        log::info!("streamline: loaded from {} (features {:?})", dir.display(), s._features);
        Some(s)
    }

    /// Once the Vulkan device is made (through the interposer): which of DLSS's parts this GPU
    /// and driver run, and their functions.
    pub fn ready(self, pdev: vk::PhysicalDevice) -> Option<Box<dyn Upscaler>> {
        let adapter = AdapterInfo {
            base: Base::new(T_ADAPTER, 1),
            device_luid: null_mut(),
            device_luid_size: 0,
            vk_physical_device: pdev.as_raw() as *mut c_void,
        };
        let (is_supported, loaded) = (self.is_supported, self._features.clone());
        let supported = |f: u32| {
            let r = unsafe { is_supported(f, &adapter) };
            if r != OK {
                log::info!("streamline: feature {f} not supported here ({r})");
            }
            r == OK && loaded.contains(&f)
        };
        let (dlss, rr) = (supported(FEATURE_DLSS), supported(FEATURE_DLSS_RR));
        let get_function = self.get_function;
        let function = |f: u32, name: &CStr| -> *mut c_void {
            let mut p = null_mut();
            let r = unsafe { get_function(f, name.as_ptr(), &mut p) };
            if r != OK || p.is_null() {
                log::warn!("streamline: {} missing ({r})", name.to_string_lossy());
                return null_mut();
            }
            p
        };
        let mut up = Dlss {
            dlss_optimal: None,
            dlss_options: None,
            rr_optimal: None,
            rr_options: None,
            name: String::new(),
            modes: 0,
            rr: false,
            sl: self,
        };
        unsafe {
            if dlss {
                let (o, s) = (function(FEATURE_DLSS, c"slDLSSGetOptimalSettings"), function(FEATURE_DLSS, c"slDLSSSetOptions"));
                if !o.is_null() && !s.is_null() {
                    up.dlss_optimal = Some(std::mem::transmute::<*mut c_void, FnDlssOptimal>(o));
                    up.dlss_options = Some(std::mem::transmute::<*mut c_void, FnDlssSetOptions>(s));
                    for m in &UpscaleMode::ALL[1..] {
                        up.modes |= 1 << *m as u32;
                    }
                }
            }
            if rr {
                let (o, s) = (function(FEATURE_DLSS_RR, c"slDLSSDGetOptimalSettings"), function(FEATURE_DLSS_RR, c"slDLSSDSetOptions"));
                if !o.is_null() && !s.is_null() {
                    up.rr_optimal = Some(std::mem::transmute::<*mut c_void, FnDlssdOptimal>(o));
                    up.rr_options = Some(std::mem::transmute::<*mut c_void, FnDlssdSetOptions>(s));
                    up.rr = true;
                }
            }
        }
        if up.modes == 0 {
            log::warn!("streamline: DLSS doesn't run here (Ray Reconstruction needs it too)");
            return None;
        }
        let version = |f: u32| {
            let mut v = FeatureVersion { base: Base::new(T_FEATURE_VERSION, 1), sl: [0; 3], ngx: [0; 3] };
            let r = unsafe { (up.sl.feature_version)(f, &mut v) };
            (r == OK).then(|| format!("{}.{}.{}", v.ngx[0], v.ngx[1], v.ngx[2]))
        };
        up.name = format!(
            "DLSS {} RR {} (Streamline 2.14.16)",
            if dlss { version(FEATURE_DLSS).unwrap_or_else(|| "?".into()) } else { "-".into() },
            if rr { version(FEATURE_DLSS_RR).unwrap_or_else(|| "?".into()) } else { "-".into() },
        );
        log::info!("streamline: {} ready", up.name);
        Some(Box::new(up))
    }
}

struct Dlss {
    sl: Streamline,
    dlss_optimal: Option<FnDlssOptimal>,
    dlss_options: Option<FnDlssSetOptions>,
    rr_optimal: Option<FnDlssdOptimal>,
    rr_options: Option<FnDlssdSetOptions>,
    name: String,
    modes: u32,
    rr: bool,
}

/// `sl::DLSSMode` for ours.
fn dlss_mode(mode: UpscaleMode) -> u32 {
    match mode {
        UpscaleMode::Quality => DLSS_MAX_QUALITY,
        UpscaleMode::Balanced => DLSS_BALANCED,
        UpscaleMode::Performance => DLSS_MAX_PERFORMANCE,
        UpscaleMode::UltraPerformance => DLSS_ULTRA_PERFORMANCE,
        _ => DLSS_DLAA,
    }
}

fn viewport() -> Viewport {
    Viewport { base: Base::new(T_VIEWPORT, 1), value: 0 }
}

fn optimal() -> OptimalSettings {
    OptimalSettings {
        base: Base::new(T_DLSS_OPTIMAL, 1),
        optimal_render_width: 0,
        optimal_render_height: 0,
        optimal_sharpness: 0.0,
        render_width_min: 0,
        render_height_min: 0,
        render_width_max: 0,
        render_height_max: 0,
    }
}

/// `model` (a `sl::DLSSPreset`) goes in every mode's slot: only `mode`'s is read.
fn dlss_options(mode: UpscaleMode, output: (u32, u32), hdr: bool, model: u32) -> DlssOptions {
    DlssOptions {
        base: Base::new(T_DLSS_OPTIONS, 3),
        mode: dlss_mode(mode),
        output_width: output.0,
        output_height: output.1,
        sharpness: 0.0,
        pre_exposure: 1.0,
        exposure_scale: 1.0,
        color_buffers_hdr: hdr as u8,
        indicator_invert_x: 0,
        indicator_invert_y: 0,
        dlaa_preset: model,
        quality_preset: model,
        balanced_preset: model,
        performance_preset: model,
        ultra_performance_preset: model,
        ultra_quality_preset: model,
        // No exposure image of ours is in the form DLSS reads: it works the exposure out.
        use_auto_exposure: 1,
        alpha_upscaling_enabled: 0,
    }
}

fn rr_options(mode: UpscaleMode, output: (u32, u32), hdr: bool, p: &UpscaleParams) -> DlssdOptions {
    DlssdOptions {
        base: Base::new(T_DLSSD_OPTIONS, 3),
        mode: dlss_mode(mode),
        output_width: output.0,
        output_height: output.1,
        sharpness: 0.0,
        pre_exposure: 1.0,
        exposure_scale: 1.0,
        color_buffers_hdr: hdr as u8,
        indicator_invert_x: 0,
        indicator_invert_y: 0,
        // ePacked: our normals image carries the roughness in its alpha.
        normal_roughness_mode: 1,
        world_to_camera_view: p.world_to_view,
        camera_view_to_world: p.view_to_world,
        alpha_upscaling_enabled: 0,
        dlaa_preset: p.rr_model,
        quality_preset: p.rr_model,
        balanced_preset: p.rr_model,
        performance_preset: p.rr_model,
        ultra_performance_preset: p.rr_model,
        ultra_quality_preset: p.rr_model,
    }
}

fn resource(i: &Image) -> Resource {
    Resource {
        base: Base::new(T_RESOURCE, 1),
        ty: 0,
        native: i.image.as_raw() as *mut c_void,
        memory: i.memory.as_raw() as *mut c_void,
        view: i.view.as_raw() as *mut c_void,
        state: i.layout.as_raw() as u32,
        width: i.width,
        height: i.height,
        native_format: i.format.as_raw() as u32,
        mip_levels: 1,
        array_layers: 1,
        gpu_virtual_address: 0,
        flags: 0,
        usage: i.usage.as_raw(),
        internal_flags: 0,
        reserved: 0,
    }
}

/// `WEFT_DLSS_JITTER=<x>,<y>`: what the jitter is multiplied by before DLSS gets it (the sign
/// convention is the usual trap). Default 1, 1: DLSS takes our jitter as it is (`post::jitter_pixels`,
/// how far the projection moved the picture, x right, y down). Measured under Proton (11-06):
/// with -1, -1 SR Quality speckled the canopy and DLAA the shaded leaves; with 1, 1 both were clean.
fn jitter_sign() -> [f32; 2] {
    static SIGN: std::sync::OnceLock<[f32; 2]> = std::sync::OnceLock::new();
    *SIGN.get_or_init(|| {
        std::env::var("WEFT_DLSS_JITTER")
            .ok()
            .and_then(|v| {
                let mut it = v.split(',').map(|s| s.trim().parse::<f32>().ok());
                Some([it.next()??, it.next()??])
            })
            .unwrap_or([1.0, 1.0])
    })
}

impl Upscaler for Dlss {
    fn info(&self) -> UpscalerInfo {
        UpscalerInfo {
            name: self.name.clone(),
            modes: self.modes,
            reconstruct: self.rr,
            neural: false,
            frame_gen: 0,
            models: MODELS.to_vec(),
            rr_models: if self.rr { RR_MODELS.to_vec() } else { Vec::new() },
        }
    }

    fn render_size(&mut self, mode: UpscaleMode, reconstruct: bool, output: (u32, u32)) -> Option<(u32, u32)> {
        if mode == UpscaleMode::Off {
            return None;
        }
        let mut s = optimal();
        let r = unsafe {
            if reconstruct {
                s.base = Base::new(T_DLSSD_OPTIMAL, 1);
                let opts = rr_options(mode, output, true, &UpscaleParams::default());
                (self.rr_optimal?)(&opts, &mut s)
            } else {
                (self.dlss_optimal?)(&dlss_options(mode, output, true, 0), &mut s)
            }
        };
        (r == OK && s.optimal_render_width > 0 && s.optimal_render_height > 0).then_some((s.optimal_render_width, s.optimal_render_height))
    }

    unsafe fn record(&mut self, cmd: vk::CommandBuffer, inputs: &Inputs, p: &UpscaleParams) -> bool {
        let mode = UpscaleMode::from_number(p.mode);
        let rr = p.reconstruct != 0;
        let feature = if rr { FEATURE_DLSS_RR } else { FEATURE_DLSS };
        if mode == UpscaleMode::Off || self.modes & (1 << mode as u32) == 0 || (rr && !self.rr) {
            return false;
        }
        let vp = viewport();
        let mut token: *mut c_void = null_mut();
        if unsafe { (self.sl.new_frame)(&mut token, null()) } != OK || token.is_null() {
            log::warn!("streamline: no frame token");
            return false;
        }
        let sign = jitter_sign();
        let consts = Constants {
            base: Base::new(T_CONSTANTS, 2),
            camera_view_to_clip: p.view_to_clip,
            clip_to_camera_view: p.clip_to_view,
            clip_to_lens_clip: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            clip_to_prev_clip: p.clip_to_prev_clip,
            prev_clip_to_clip: p.prev_clip_to_clip,
            jitter_offset: [p.jitter[0] * sign[0], p.jitter[1] * sign[1]],
            mvec_scale: p.mv_scale,
            camera_pinhole_offset: [0.0, 0.0],
            // Camera-relative: the camera is the origin.
            camera_pos: [0.0, 0.0, 0.0],
            camera_up: [p.up[0], p.up[1], p.up[2]],
            camera_right: [p.right[0], p.right[1], p.right[2]],
            camera_fwd: [p.forward[0], p.forward[1], p.forward[2]],
            camera_near: p.near,
            // Reverse Z with the far plane at infinity: as far as a float goes.
            camera_far: 1.0e7,
            camera_fov: p.fov_y,
            camera_aspect_ratio: p.aspect,
            motion_vectors_invalid_value: 0.0,
            depth_inverted: 1,
            camera_motion_included: 1,
            motion_vectors_3d: 0,
            reset: (p.reset != 0) as u8,
            orthographic_projection: 0,
            motion_vectors_dilated: 0,
            motion_vectors_jittered: 0,
            min_relative_linear_depth_object_separation: 40.0,
        };
        let output = (p.output[0], p.output[1]);
        let hdr = p.hdr != 0;
        unsafe {
            if (self.sl.set_constants)(&consts, token, &vp) != OK {
                log::warn!("streamline: slSetConstants failed");
                return false;
            }
            let r = if rr {
                (self.rr_options.unwrap())(&vp, &rr_options(mode, output, hdr, p))
            } else {
                (self.dlss_options.unwrap())(&vp, &dlss_options(mode, output, hdr, p.model))
            };
            if r != OK {
                log::warn!("streamline: setting options failed ({r})");
                return false;
            }
        }
        let render = Extent { top: 0, left: 0, width: p.render[0], height: p.render[1] };
        let full = Extent { top: 0, left: 0, width: output.0, height: output.1 };
        let mut res: Vec<(Resource, u32, Extent)> = vec![
            (resource(&inputs.color), TAG_INPUT_COLOR, render),
            (resource(&inputs.output), TAG_OUTPUT_COLOR, full),
            (resource(&inputs.depth), TAG_DEPTH, render),
            (resource(&inputs.motion), TAG_MOTION, render),
        ];
        if rr {
            let (Some(n), Some(a), Some(s)) = (&inputs.normals, &inputs.albedo, &inputs.specular) else {
                log::warn!("streamline: Ray Reconstruction without its guides");
                return false;
            };
            res.push((resource(n), TAG_NORMAL_ROUGHNESS, render));
            res.push((resource(a), TAG_ALBEDO, render));
            res.push((resource(s), TAG_SPECULAR_ALBEDO, render));
            // No mirror-like surfaces of their own yet: reflections move as the surfaces do.
            res.push((resource(&inputs.motion), TAG_SPECULAR_MOTION, render));
        }
        let tags: Vec<ResourceTag> = res
            .iter_mut()
            .map(|(r, ty, extent)| ResourceTag {
                base: Base::new(T_RESOURCE_TAG, 1),
                resource: r as *mut Resource,
                ty: *ty,
                lifecycle: VALID_UNTIL_EVALUATE,
                extent: *extent,
            })
            .collect();
        let mut list: Vec<*const Base> = vec![&vp.base as *const Base];
        list.extend(tags.iter().map(|t| &t.base as *const Base));
        let r = unsafe { (self.sl.evaluate)(feature, token, list.as_ptr(), list.len() as u32, cmd.as_raw() as *mut c_void) };
        if r != OK {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| log::warn!("streamline: evaluating {} failed ({r})", mode.name()));
            return false;
        }
        true
    }
}

impl Drop for Dlss {
    fn drop(&mut self) {
        unsafe {
            let vp = viewport();
            for f in self.sl._features.clone() {
                (self.sl.free_resources)(f, &vp);
            }
            (self.sl.shutdown)();
        }
    }
}

//! DLSS through NVIDIA's NGX directly (the `ngx` feature): DLAA, Super Resolution and Ray
//! Reconstruction on Linux without Streamline (which is Windows-only) or Proton.
//!
//! Our own bindings to the DLSS SDK's C entry points (310.9.16, NVIDIA/DLSS on GitHub): the
//! structs and names the SDK's headers define, written out here by hand (no bindgen, nothing of
//! NVIDIA's copied into the repository). The SDK's static loader `libnvsdk_ngx.a` is linked from
//! outside the repository (`build.rs`, `NGX_SDK_DIR`); NGX itself loads DLSS's own libraries
//! (`libnvidia-ngx-dlss.so.*`, `libnvidia-ngx-dlssd.so.*`) at run time from the folders in
//! `paths()`, or the driver's copy.
//!
//! The parameters NGX's inline C helpers set (`nvsdk_ngx_helpers_vk.h`,
//! `nvsdk_ngx_helpers_dlssd_vk.h`) are set here through the exported
//! `NVSDK_NGX_Parameter_Set*` functions instead. `WEFT_NO_NGX=1` skips it; `WEFT_NGX_DIR` adds a
//! folder with DLSS's libraries; `WEFT_NGX_VERBOSE=1` puts NGX's verbose log in ours.
//! Shares Streamline's conventions (`streamline.rs`), so pictures from the two match: jitter as
//! is (`WEFT_DLSS_JITTER`), auto exposure, packed normals and roughness for Ray Reconstruction.

use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use ash::vk;
use weft_render::{UpscaleMode, UpscaleModel, UpscaleParams, UpscalerInfo};

use crate::upscale::{Image, Inputs, Upscaler};

#[cfg(windows)]
type WChar = u16;
#[cfg(not(windows))]
type WChar = u32;

type NgxResult = u32;

fn failed(r: NgxResult) -> bool {
    r & 0xfff0_0000 == 0xbad0_0000
}

/// `NVSDK_NGX_Version_API` of SDK 310.9.16 (1.5.0).
const SDK_VERSION: c_int = 0x15;
/// The same project as under Streamline (`streamline.rs`), as NVIDIA's custom-engine id.
const PROJECT_ID: &CStr = c"f264d7e7-3c55-4a0e-90d7-35933f37130f";
const ENGINE_VERSION: &CStr = c"26.11";
const ENGINE_CUSTOM: c_int = 0;
const IDENTIFIER_PROJECT: c_int = 1;

const FEATURE_SUPER_SAMPLING: c_int = 1;
const FEATURE_RAY_RECONSTRUCTION: c_int = 13;

// `NVSDK_NGX_PerfQuality_Value`.
const PERF_MAX_PERF: c_int = 0;
const PERF_BALANCED: c_int = 1;
const PERF_MAX_QUALITY: c_int = 2;
const PERF_ULTRA_PERFORMANCE: c_int = 3;
const PERF_DLAA: c_int = 5;

// `NVSDK_NGX_DLSS_Feature_Flags`.
const FLAG_HDR: c_int = 1 << 0;
const FLAG_MV_LOW_RES: c_int = 1 << 1;
const FLAG_DEPTH_INVERTED: c_int = 1 << 3;
const FLAG_AUTO_EXPOSURE: c_int = 1 << 6;

const LOG_ON: c_int = 1;
const LOG_VERBOSE: c_int = 2;

/// `NVSDK_NGX_Parameter` and `NVSDK_NGX_Handle`: NGX's own, only ever behind a pointer.
#[repr(C)]
struct Parameter {
    _opaque: [u8; 0],
}
#[repr(C)]
struct FeatureHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
struct PathListInfo {
    path: *const *const WChar,
    length: c_uint,
}

type LogCallback = unsafe extern "C" fn(*const c_char, c_int, c_int);

#[repr(C)]
struct LoggingInfo {
    callback: Option<LogCallback>,
    minimum_level: c_int,
    disable_other_sinks: bool,
}

#[repr(C)]
struct FeatureCommonInfo {
    path_list: PathListInfo,
    internal: *mut c_void,
    logging: LoggingInfo,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProjectIdDescription {
    project_id: *const c_char,
    engine_type: c_int,
    engine_version: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
union IdentifierValue {
    project: ProjectIdDescription,
    application_id: u64,
}

#[repr(C)]
struct ApplicationIdentifier {
    ty: c_int,
    v: IdentifierValue,
}

#[repr(C)]
struct FeatureDiscoveryInfo {
    sdk_version: c_int,
    feature: c_int,
    identifier: ApplicationIdentifier,
    data_path: *const WChar,
    feature_info: *const FeatureCommonInfo,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ImageViewInfo {
    view: vk::ImageView,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    format: vk::Format,
    width: c_uint,
    height: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BufferInfo {
    buffer: vk::Buffer,
    size: c_uint,
}

#[repr(C)]
union ResourceValue {
    image: ImageViewInfo,
    buffer: BufferInfo,
}

/// `NVSDK_NGX_Resource_VK`.
#[repr(C)]
struct Resource {
    value: ResourceValue,
    ty: c_int,
    read_write: bool,
}

const _: () = {
    assert!(size_of::<PathListInfo>() == 16);
    assert!(size_of::<LoggingInfo>() == 16);
    assert!(size_of::<FeatureCommonInfo>() == 40);
    assert!(size_of::<ApplicationIdentifier>() == 32);
    assert!(size_of::<FeatureDiscoveryInfo>() == 56);
    assert!(size_of::<ImageViewInfo>() == 48);
    assert!(size_of::<Resource>() == 56);
};

type OptimalSettingsFn = unsafe extern "C" fn(*mut Parameter) -> NgxResult;

unsafe extern "C" {
    fn NVSDK_NGX_VULKAN_GetFeatureInstanceExtensionRequirements(
        info: *const FeatureDiscoveryInfo,
        count: *mut u32,
        props: *mut *mut vk::ExtensionProperties,
    ) -> NgxResult;
    fn NVSDK_NGX_VULKAN_GetFeatureDeviceExtensionRequirements(
        instance: vk::Instance,
        pdev: vk::PhysicalDevice,
        info: *const FeatureDiscoveryInfo,
        count: *mut u32,
        props: *mut *mut vk::ExtensionProperties,
    ) -> NgxResult;
    fn NVSDK_NGX_VULKAN_Init_with_ProjectID(
        project_id: *const c_char,
        engine: c_int,
        engine_version: *const c_char,
        data_path: *const WChar,
        instance: vk::Instance,
        pdev: vk::PhysicalDevice,
        device: vk::Device,
        gipa: Option<vk::PFN_vkGetInstanceProcAddr>,
        gdpa: Option<vk::PFN_vkGetDeviceProcAddr>,
        info: *const FeatureCommonInfo,
        version: c_int,
    ) -> NgxResult;
    fn NVSDK_NGX_VULKAN_Shutdown1(device: vk::Device) -> NgxResult;
    fn NVSDK_NGX_VULKAN_GetCapabilityParameters(out: *mut *mut Parameter) -> NgxResult;
    fn NVSDK_NGX_VULKAN_AllocateParameters(out: *mut *mut Parameter) -> NgxResult;
    fn NVSDK_NGX_VULKAN_DestroyParameters(params: *mut Parameter) -> NgxResult;
    fn NVSDK_NGX_VULKAN_CreateFeature1(
        device: vk::Device,
        cmd: vk::CommandBuffer,
        feature: c_int,
        params: *mut Parameter,
        out: *mut *mut FeatureHandle,
    ) -> NgxResult;
    fn NVSDK_NGX_VULKAN_ReleaseFeature(handle: *mut FeatureHandle) -> NgxResult;
    fn NVSDK_NGX_VULKAN_EvaluateFeature_C(
        cmd: vk::CommandBuffer,
        handle: *const FeatureHandle,
        params: *const Parameter,
        progress: Option<unsafe extern "C" fn(f32, *mut bool)>,
    ) -> NgxResult;
    fn NVSDK_NGX_Parameter_SetUI(p: *mut Parameter, name: *const c_char, v: c_uint);
    fn NVSDK_NGX_Parameter_SetI(p: *mut Parameter, name: *const c_char, v: c_int);
    fn NVSDK_NGX_Parameter_SetF(p: *mut Parameter, name: *const c_char, v: f32);
    fn NVSDK_NGX_Parameter_SetVoidPointer(p: *mut Parameter, name: *const c_char, v: *mut c_void);
    fn NVSDK_NGX_Parameter_GetI(p: *mut Parameter, name: *const c_char, v: *mut c_int) -> NgxResult;
    fn NVSDK_NGX_Parameter_GetUI(p: *mut Parameter, name: *const c_char, v: *mut c_uint) -> NgxResult;
    fn NVSDK_NGX_Parameter_GetVoidPointer(p: *mut Parameter, name: *const c_char, v: *mut *mut c_void) -> NgxResult;
    fn GetNGXResultAsString(r: NgxResult) -> *const WChar;
}

/// NGX's words for a result, for the log.
#[allow(clippy::useless_conversion)]
fn describe(r: NgxResult) -> String {
    let p = unsafe { GetNGXResultAsString(r) };
    if p.is_null() {
        return format!("{r:#x}");
    }
    let mut s = String::new();
    for i in 0..256 {
        let c = unsafe { *p.add(i) };
        if c == 0 {
            break;
        }
        s.push(char::from_u32(u32::from(c)).unwrap_or('?'));
    }
    format!("{s} ({r:#x})")
}

/// A path as NGX's `wchar_t` string (UTF-32 on Linux, UTF-16 on Windows), ending in 0.
fn wide(s: &std::path::Path) -> Vec<WChar> {
    let text = s.to_string_lossy();
    #[cfg(windows)]
    let mut v: Vec<WChar> = text.encode_utf16().collect();
    #[cfg(not(windows))]
    let mut v: Vec<WChar> = text.chars().map(|c| c as WChar).collect();
    v.push(0);
    v
}

unsafe extern "C" fn log_message(msg: *const c_char, level: c_int, _feature: c_int) {
    if msg.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    let text = text.trim_end();
    if level >= LOG_VERBOSE {
        log::debug!(target: "ngx", "{text}");
    } else {
        log::info!(target: "ngx", "{text}");
    }
}

/// Where NGX looks for DLSS's libraries: `WEFT_NGX_DIR`, `ngx/` beside the game, the game's
/// folder, and the SDK's own release folder on the machine that built it (`NGX_SDK_DIR` then).
fn paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(d) = std::env::var_os("WEFT_NGX_DIR") {
        v.push(PathBuf::from(d));
    }
    if let Some(sdk) = option_env!("NGX_SDK_DIR") {
        v.push(PathBuf::from(sdk).join("lib/Linux_x86_64/rel"));
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.to_path_buf()))
    {
        v.push(dir.join("ngx"));
        v.push(dir);
    }
    v.retain(|p| p.is_dir());
    v
}

/// The models offered (`NVSDK_NGX_DLSS_Hint_Render_Preset`, `..._RayReconstruction_...`), the
/// same as under Streamline.
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

fn perf_quality(mode: UpscaleMode) -> c_int {
    match mode {
        UpscaleMode::Quality => PERF_MAX_QUALITY,
        UpscaleMode::Balanced => PERF_BALANCED,
        UpscaleMode::Performance => PERF_MAX_PERF,
        UpscaleMode::UltraPerformance => PERF_ULTRA_PERFORMANCE,
        _ => PERF_DLAA,
    }
}

/// The mode's word in NGX's preset parameter names (`DLSS.Hint.Render.Preset.<word>`).
fn preset_word(mode: UpscaleMode) -> &'static str {
    match mode {
        UpscaleMode::Quality => "Quality",
        UpscaleMode::Balanced => "Balanced",
        UpscaleMode::Performance => "Performance",
        UpscaleMode::UltraPerformance => "UltraPerformance",
        _ => "DLAA",
    }
}

fn name(s: &str) -> CString {
    CString::new(s).expect("parameter name")
}

/// What NGX needs before the Vulkan instance and device are made: the extensions DLSS and Ray
/// Reconstruction ask for, kept here while the backend's create infos point at them.
pub(crate) struct Early {
    instance_exts: Vec<CString>,
    device_exts: Vec<CString>,
    /// What `path_ptrs` points into.
    _paths: Vec<Vec<WChar>>,
    path_ptrs: Vec<*const WChar>,
    data_path: Vec<WChar>,
}

unsafe impl Send for Early {}

impl Early {
    pub fn load() -> Option<Early> {
        if std::env::var_os("WEFT_NO_NGX").is_some() {
            return None;
        }
        let paths: Vec<Vec<WChar>> = paths().iter().map(|p| wide(p)).collect();
        let data = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut e = Early {
            instance_exts: Vec::new(),
            device_exts: Vec::new(),
            path_ptrs: paths.iter().map(|p| p.as_ptr()).collect(),
            _paths: paths,
            data_path: wide(&data),
        };
        for f in [FEATURE_SUPER_SAMPLING, FEATURE_RAY_RECONSTRUCTION] {
            let (common, info) = e.discovery(f);
            let info = FeatureDiscoveryInfo { feature_info: &common, ..info };
            let mut count = 0u32;
            let mut props: *mut vk::ExtensionProperties = null_mut();
            let r = unsafe { NVSDK_NGX_VULKAN_GetFeatureInstanceExtensionRequirements(&info, &mut count, &mut props) };
            if failed(r) {
                log::info!("ngx: feature {f}: no instance extensions known ({})", describe(r));
                continue;
            }
            e.instance_exts.extend(unsafe { ext_names(props, count) });
        }
        e.instance_exts.sort();
        e.instance_exts.dedup();
        Some(e)
    }

    fn common(&self) -> FeatureCommonInfo {
        FeatureCommonInfo {
            path_list: PathListInfo { path: self.path_ptrs.as_ptr(), length: self.path_ptrs.len() as c_uint },
            internal: null_mut(),
            logging: LoggingInfo {
                callback: Some(log_message),
                minimum_level: if std::env::var_os("WEFT_NGX_VERBOSE").is_some() { LOG_VERBOSE } else { LOG_ON },
                disable_other_sinks: false,
            },
        }
    }

    /// The feature-discovery info for `feature` (its `feature_info` set by the caller to the
    /// first, which must outlive the call).
    fn discovery(&self, feature: c_int) -> (FeatureCommonInfo, FeatureDiscoveryInfo) {
        let info = FeatureDiscoveryInfo {
            sdk_version: SDK_VERSION,
            feature,
            identifier: ApplicationIdentifier {
                ty: IDENTIFIER_PROJECT,
                v: IdentifierValue {
                    project: ProjectIdDescription {
                        project_id: PROJECT_ID.as_ptr(),
                        engine_type: ENGINE_CUSTOM,
                        engine_version: ENGINE_VERSION.as_ptr(),
                    },
                },
            },
            data_path: self.data_path.as_ptr(),
            feature_info: null(),
        };
        (self.common(), info)
    }

    /// The instance extensions to add, those of them the loader has (the rest are logged).
    pub fn instance_extensions(&self, available: &[vk::ExtensionProperties]) -> Vec<*const c_char> {
        self.instance_exts
            .iter()
            .filter(|n| {
                let has = available
                    .iter()
                    .any(|e| e.extension_name_as_c_str() == Ok(n.as_c_str()));
                if !has {
                    log::info!("ngx: instance extension {} not here", n.to_string_lossy());
                }
                has
            })
            .map(|n| n.as_ptr())
            .collect()
    }

    /// The device extensions to add for `pdev`, those it has and `already` doesn't hold.
    ///
    /// # Safety
    /// `instance` and `pdev` are live.
    pub unsafe fn device_extensions(
        &mut self,
        instance: &ash::Instance,
        pdev: vk::PhysicalDevice,
        already: &[*const c_char],
    ) -> Vec<*const c_char> {
        let mut names = Vec::new();
        for f in [FEATURE_SUPER_SAMPLING, FEATURE_RAY_RECONSTRUCTION] {
            let (common, info) = self.discovery(f);
            let info = FeatureDiscoveryInfo { feature_info: &common, ..info };
            let mut count = 0u32;
            let mut props: *mut vk::ExtensionProperties = null_mut();
            let r =
                unsafe { NVSDK_NGX_VULKAN_GetFeatureDeviceExtensionRequirements(instance.handle(), pdev, &info, &mut count, &mut props) };
            if failed(r) {
                log::info!("ngx: feature {f}: no device extensions known ({})", describe(r));
                continue;
            }
            names.extend(unsafe { ext_names(props, count) });
        }
        names.sort();
        names.dedup();
        let available = unsafe { instance.enumerate_device_extension_properties(pdev) }.unwrap_or_default();
        let held: Vec<&CStr> = already
            .iter()
            .map(|p| unsafe { CStr::from_ptr(*p) })
            .collect();
        self.device_exts = names
            .into_iter()
            .filter(|n| !held.contains(&n.as_c_str()))
            .filter(|n| {
                let has = available
                    .iter()
                    .any(|e| e.extension_name_as_c_str() == Ok(n.as_c_str()));
                if !has {
                    log::info!("ngx: device extension {} not here", n.to_string_lossy());
                }
                has
            })
            .collect();
        self.device_exts.iter().map(|n| n.as_ptr()).collect()
    }

    /// Once the device is made: NGX started on it, and what of DLSS runs here.
    pub fn ready(
        self,
        entry: &ash::Entry,
        instance: &ash::Instance,
        pdev: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Option<Box<dyn Upscaler>> {
        let common = self.common();
        let r = unsafe {
            NVSDK_NGX_VULKAN_Init_with_ProjectID(
                PROJECT_ID.as_ptr(),
                ENGINE_CUSTOM,
                ENGINE_VERSION.as_ptr(),
                self.data_path.as_ptr(),
                instance.handle(),
                pdev,
                device.handle(),
                Some(entry.static_fn().get_instance_proc_addr),
                Some(instance.fp_v1_0().get_device_proc_addr),
                &common,
                SDK_VERSION,
            )
        };
        if failed(r) {
            log::warn!("ngx: init failed: {}", describe(r));
            return None;
        }
        let mut caps: *mut Parameter = null_mut();
        let r = unsafe { NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut caps) };
        if failed(r) || caps.is_null() {
            log::warn!("ngx: no capability parameters: {}", describe(r));
            unsafe { NVSDK_NGX_VULKAN_Shutdown1(device.handle()) };
            return None;
        }
        let get_i = |key: &str| {
            let mut v: c_int = 0;
            let r = unsafe { NVSDK_NGX_Parameter_GetI(caps, name(key).as_ptr(), &mut v) };
            (!failed(r)).then_some(v)
        };
        let dlss = get_i("SuperSampling.Available") == Some(1);
        let rr = get_i("SuperSamplingDenoising.Available") == Some(1);
        if !dlss {
            log::warn!(
                "ngx: DLSS doesn't run here (init result {:?}, needs a newer driver: {:?})",
                get_i("SuperSampling.FeatureInitResult"),
                get_i("SuperSampling.NeedsUpdatedDriver"),
            );
            unsafe {
                NVSDK_NGX_VULKAN_DestroyParameters(caps);
                NVSDK_NGX_VULKAN_Shutdown1(device.handle());
            }
            return None;
        }
        if !rr {
            log::info!("ngx: Ray Reconstruction doesn't run here (init result {:?})", get_i("SuperSamplingDenoising.FeatureInitResult"));
        }
        let modes = UpscaleMode::ALL[1..]
            .iter()
            .fold(0, |bits, m| bits | 1 << *m as u32);
        let name = format!("DLSS 310.9.16{} (NGX)", if rr { " + RR" } else { "" });
        log::info!("ngx: {name} ready");
        Some(Box::new(Ngx { device: device.clone(), caps, rr, modes, name, feature: None, _early: self }))
    }
}

/// The extension names in what NGX handed back (its own memory).
unsafe fn ext_names(props: *const vk::ExtensionProperties, count: u32) -> Vec<CString> {
    if props.is_null() {
        return Vec::new();
    }
    (0..count as usize)
        .filter_map(|i| {
            unsafe { &*props.add(i) }
                .extension_name_as_c_str()
                .ok()
                .map(CStr::to_owned)
        })
        .collect()
}

/// A DLSS feature made for one mode, size and model; made again when any of them changes.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    mode: UpscaleMode,
    rr: bool,
    render: [u32; 2],
    output: [u32; 2],
    hdr: bool,
    model: u32,
    rr_model: u32,
}

struct Feature {
    key: Key,
    handle: *mut FeatureHandle,
    params: *mut Parameter,
}

struct Ngx {
    device: ash::Device,
    caps: *mut Parameter,
    rr: bool,
    modes: u32,
    name: String,
    feature: Option<Feature>,
    /// The paths NGX was given, which it may still read.
    _early: Early,
}

unsafe impl Send for Ngx {}

fn range(i: &Image) -> vk::ImageSubresourceRange {
    let depth = matches!(i.format, vk::Format::D32_SFLOAT | vk::Format::D24_UNORM_S8_UINT | vk::Format::D32_SFLOAT_S8_UINT);
    vk::ImageSubresourceRange {
        aspect_mask: if depth { vk::ImageAspectFlags::DEPTH } else { vk::ImageAspectFlags::COLOR },
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    }
}

fn resource(i: &Image, read_write: bool) -> Resource {
    Resource {
        value: ResourceValue {
            image: ImageViewInfo { view: i.view, image: i.image, range: range(i), format: i.format, width: i.width, height: i.height },
        },
        ty: 0,
        read_write,
    }
}

/// Same as Streamline's (`streamline.rs`): `WEFT_DLSS_JITTER=<x>,<y>` multiplies the jitter.
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

impl Ngx {
    fn set_ui(p: *mut Parameter, key: &str, v: u32) {
        unsafe { NVSDK_NGX_Parameter_SetUI(p, name(key).as_ptr(), v) }
    }
    fn set_i(p: *mut Parameter, key: &str, v: i32) {
        unsafe { NVSDK_NGX_Parameter_SetI(p, name(key).as_ptr(), v) }
    }
    fn set_f(p: *mut Parameter, key: &str, v: f32) {
        unsafe { NVSDK_NGX_Parameter_SetF(p, name(key).as_ptr(), v) }
    }
    fn set_ptr<T>(p: *mut Parameter, key: &str, v: *const T) {
        unsafe { NVSDK_NGX_Parameter_SetVoidPointer(p, name(key).as_ptr(), v as *mut c_void) }
    }

    fn release(&mut self) {
        if let Some(f) = self.feature.take() {
            unsafe {
                // The GPU may still run the last frame's evaluate; features change rarely.
                self.device.device_wait_idle().ok();
                NVSDK_NGX_VULKAN_ReleaseFeature(f.handle);
                NVSDK_NGX_VULKAN_DestroyParameters(f.params);
            }
        }
    }

    /// The feature for `key`, made in `cmd` if it isn't there yet.
    fn feature(&mut self, cmd: vk::CommandBuffer, key: Key) -> Option<(*mut FeatureHandle, *mut Parameter)> {
        if let Some(f) = self.feature.as_ref().filter(|f| f.key == key) {
            return Some((f.handle, f.params));
        }
        self.release();
        let mut params: *mut Parameter = null_mut();
        let r = unsafe { NVSDK_NGX_VULKAN_AllocateParameters(&mut params) };
        if failed(r) || params.is_null() {
            log::warn!("ngx: no parameters: {}", describe(r));
            return None;
        }
        let mut flags = FLAG_MV_LOW_RES | FLAG_DEPTH_INVERTED | FLAG_AUTO_EXPOSURE;
        if key.hdr {
            flags |= FLAG_HDR;
        }
        Self::set_ui(params, "CreationNodeMask", 1);
        Self::set_ui(params, "VisibilityNodeMask", 1);
        Self::set_ui(params, "Width", key.render[0]);
        Self::set_ui(params, "Height", key.render[1]);
        Self::set_ui(params, "OutWidth", key.output[0]);
        Self::set_ui(params, "OutHeight", key.output[1]);
        Self::set_i(params, "PerfQualityValue", perf_quality(key.mode));
        Self::set_i(params, "DLSS.Feature.Create.Flags", flags);
        Self::set_i(params, "DLSS.Enable.Output.Subrects", 0);
        // The model goes in every mode's slot: only this mode's is read.
        for m in &UpscaleMode::ALL[1..] {
            Self::set_ui(params, &format!("DLSS.Hint.Render.Preset.{}", preset_word(*m)), key.model);
            Self::set_ui(params, &format!("RayReconstruction.Hint.Render.Preset.{}", preset_word(*m)), key.rr_model);
        }
        let feature = if key.rr {
            // The unified denoiser, roughness in the normals' alpha, depth as the hardware wrote it.
            Self::set_i(params, "DLSS.Denoise.Mode", 1);
            Self::set_ui(params, "DLSS.Roughness.Mode", 1);
            Self::set_ui(params, "DLSS.Use.HW.Depth", 1);
            FEATURE_RAY_RECONSTRUCTION
        } else {
            FEATURE_SUPER_SAMPLING
        };
        let mut handle: *mut FeatureHandle = null_mut();
        let r = unsafe { NVSDK_NGX_VULKAN_CreateFeature1(self.device.handle(), cmd, feature, params, &mut handle) };
        if failed(r) || handle.is_null() {
            log::warn!(
                "ngx: making {} {}x{} -> {}x{}{} failed: {}",
                key.mode.name(),
                key.render[0],
                key.render[1],
                key.output[0],
                key.output[1],
                if key.rr { " (RR)" } else { "" },
                describe(r)
            );
            unsafe { NVSDK_NGX_VULKAN_DestroyParameters(params) };
            return None;
        }
        log::info!(
            "ngx: {} {}x{} -> {}x{}{} model {} ready",
            key.mode.name(),
            key.render[0],
            key.render[1],
            key.output[0],
            key.output[1],
            if key.rr { " RR" } else { "" },
            if key.rr { key.rr_model } else { key.model }
        );
        self.feature = Some(Feature { key, handle, params });
        Some((handle, params))
    }
}

impl Upscaler for Ngx {
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
        let key = if reconstruct && self.rr { "DLSSDOptimalSettingsCallback" } else { "DLSSOptimalSettingsCallback" };
        let mut cb: *mut c_void = null_mut();
        let r = unsafe { NVSDK_NGX_Parameter_GetVoidPointer(self.caps, name(key).as_ptr(), &mut cb) };
        if failed(r) || cb.is_null() {
            return None;
        }
        let cb: OptimalSettingsFn = unsafe { std::mem::transmute::<*mut c_void, OptimalSettingsFn>(cb) };
        Self::set_ui(self.caps, "Width", output.0);
        Self::set_ui(self.caps, "Height", output.1);
        Self::set_i(self.caps, "PerfQualityValue", perf_quality(mode));
        Self::set_i(self.caps, "RTXValue", 0);
        if failed(unsafe { cb(self.caps) }) {
            return None;
        }
        let (mut w, mut h) = (0u32, 0u32);
        unsafe {
            NVSDK_NGX_Parameter_GetUI(self.caps, name("OutWidth").as_ptr(), &mut w);
            NVSDK_NGX_Parameter_GetUI(self.caps, name("OutHeight").as_ptr(), &mut h);
        }
        (w > 0 && h > 0).then_some((w, h))
    }

    unsafe fn record(&mut self, cmd: vk::CommandBuffer, inputs: &Inputs, p: &UpscaleParams) -> bool {
        let mode = UpscaleMode::from_number(p.mode);
        let rr = p.reconstruct != 0;
        if mode == UpscaleMode::Off || self.modes & (1 << mode as u32) == 0 || (rr && !self.rr) {
            return false;
        }
        let key = Key { mode, rr, render: p.render, output: p.output, hdr: p.hdr != 0, model: p.model, rr_model: p.rr_model };
        let Some((handle, params)) = self.feature(cmd, key) else { return false };
        let color = resource(&inputs.color, false);
        let output = resource(&inputs.output, true);
        let depth = resource(&inputs.depth, false);
        let motion = resource(&inputs.motion, false);
        Self::set_ptr(params, "Color", &color);
        Self::set_ptr(params, "Output", &output);
        Self::set_ptr(params, "Depth", &depth);
        Self::set_ptr(params, "MotionVectors", &motion);
        let sign = jitter_sign();
        Self::set_f(params, "Jitter.Offset.X", p.jitter[0] * sign[0]);
        Self::set_f(params, "Jitter.Offset.Y", p.jitter[1] * sign[1]);
        Self::set_i(params, "Reset", (p.reset != 0) as i32);
        // Ours are in UV (`weft_render::MOTION_FORMAT`): NGX wants the world's pixels.
        Self::set_f(params, "MV.Scale.X", p.mv_scale[0] * p.render[0] as f32);
        Self::set_f(params, "MV.Scale.Y", p.mv_scale[1] * p.render[1] as f32);
        Self::set_ui(params, "DLSS.Render.Subrect.Dimensions.Width", p.render[0]);
        Self::set_ui(params, "DLSS.Render.Subrect.Dimensions.Height", p.render[1]);
        Self::set_f(params, "DLSS.Pre.Exposure", 1.0);
        Self::set_f(params, "DLSS.Exposure.Scale", 1.0);
        Self::set_f(params, "Sharpness", 0.0);
        let guides;
        if rr {
            let (Some(n), Some(a), Some(s)) = (&inputs.normals, &inputs.albedo, &inputs.specular) else {
                log::warn!("ngx: Ray Reconstruction without its guides");
                return false;
            };
            let hit = inputs.hit_distance.as_ref().unwrap_or(s);
            guides = [resource(n, false), resource(a, false), resource(s, false), resource(hit, false)];
            Self::set_ptr(params, "GBuffer.Normals", &guides[0]);
            Self::set_ptr(params, "DLSS.Input.DiffuseAlbedo", &guides[1]);
            Self::set_ptr(params, "DLSS.Input.SpecularAlbedo", &guides[2]);
            // No mirror-like surfaces of their own yet: reflections move as the surfaces do
            // (as under Streamline).
            Self::set_ptr(params, "GBuffer.SpecularMvec", &motion);
            // The path tracer's specular hit distance (r), when there is one.
            if inputs.hit_distance.is_some() {
                Self::set_ptr(params, "DLSSD.SpecularHitDistance", &guides[3]);
            }
            // The same matrices, in the same layout, Streamline is given.
            Self::set_ptr(params, "WorldToViewMatrix", p.world_to_view.as_ptr());
            Self::set_ptr(params, "ViewToClipMatrix", p.view_to_clip.as_ptr());
        }
        let r = unsafe { NVSDK_NGX_VULKAN_EvaluateFeature_C(cmd, handle, params, None) };
        // What NGX was pointed at lives only for this call.
        for k in [
            "Color",
            "Output",
            "Depth",
            "MotionVectors",
            "GBuffer.Normals",
            "DLSS.Input.DiffuseAlbedo",
            "DLSS.Input.SpecularAlbedo",
            "GBuffer.SpecularMvec",
            "DLSSD.SpecularHitDistance",
            "WorldToViewMatrix",
            "ViewToClipMatrix",
        ] {
            Self::set_ptr::<c_void>(params, k, null());
        }
        if failed(r) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| log::warn!("ngx: evaluating {} failed: {}", mode.name(), describe(r)));
            return false;
        }
        true
    }
}

impl Drop for Ngx {
    fn drop(&mut self) {
        self.release();
        unsafe {
            NVSDK_NGX_VULKAN_DestroyParameters(self.caps);
            NVSDK_NGX_VULKAN_Shutdown1(self.device.handle());
        }
    }
}

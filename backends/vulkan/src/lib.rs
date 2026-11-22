//! Vulkan 1.1 backend with descriptor indexing, which phones have too: render passes, bindless
//! sampled images, a descriptor set for each draw's buffers. Draws many at once and with a count
//! a shader wrote where the GPU can, one by one and compacted where it can't.

use std::collections::HashMap;
use std::ffi::{CString, c_char, c_void};
use std::mem::ManuallyDrop;

use ash::{ext, khr, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc};
use weft_render::*;

#[cfg(feature = "ngx")]
mod ngx;
mod rt;
#[cfg(feature = "streamline")]
mod streamline;
mod upscale;

const FRAMES: usize = 2;
/// Set 0 has the textures, 1 each draw's buffers, this one the samplers.
const SAMPLER_SET: u32 = 2;
/// Timestamp queries per frame: start, after uploads, marks, end.
const MAX_MARKS: u32 = 16;
const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

struct Buffer {
    raw: vk::Buffer,
    alloc: Allocation,
    size: u64,
}

struct Texture {
    image: vk::Image,
    view: vk::ImageView,
    alloc: Allocation,
    desc: TextureDesc,
    layout: vk::ImageLayout,
    /// Index into the matching bindless array (depth too: screen effects read it).
    index: u32,
    /// What it was made for (an upscaler is told: `upscale`).
    usage: vk::ImageUsageFlags,
}

struct Pipeline {
    raw: vk::Pipeline,
    layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    bind_point: vk::PipelineBindPoint,
    buffers: Vec<(u32, vk::DescriptorType)>,
}

enum Deferred {
    Buffer(vk::Buffer, Allocation),
    Texture(vk::Image, vk::ImageView, Allocation, u32),
    Pipeline(vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout),
    Accel(vk::AccelerationStructureKHR, vk::Buffer, Allocation),
}

/// Upload memory of one frame slot: a list of chunks filled in order. When a chunk is full the
/// next one is used (made if needed) instead of growing and copying: reading back mapped upload
/// memory is very slow (write-combined), a 100 MB copy took half a second.
struct Staging {
    chunks: Vec<(vk::Buffer, Allocation, u64)>,
    current: usize,
    used: u64,
}

struct Frame {
    pool: vk::CommandPool,
    cmd: vk::CommandBuffer,
    acquired: vk::Semaphore,
    fence: vk::Fence,
    staging: Staging,
    deferred: Vec<Deferred>,
    /// Sets for the buffers of draws, all freed at once when the slot comes round; a new pool
    /// when the last is full.
    set_pools: Vec<vk::DescriptorPool>,
    set_pool: usize,
    framebuffers: Vec<vk::Framebuffer>,
    /// Compacted draw records, where draws with a count don't work: (buffer, size).
    compacted: Option<(BufferId, u64)>,
    waited: bool,
    /// Timestamps written by the frame now in flight in this slot.
    marks: u32,
}

/// What makes render passes differ: the color formats (the color, then the motion image and the
/// extra images, `Cmd::BeginPass::motion` and `Cmd::PassExtras`) and how each attachment starts.
type PassKey = (Vec<(vk::Format, vk::AttachmentLoadOp)>, Option<vk::AttachmentLoadOp>);

/// Where a write kept on the CPU before the slot was ready goes (`VulkanBackend::early`).
enum Early {
    Buffer { dst: u32, offset: u64 },
    Texture { dst: u32, mip: u32 },
}

enum Upload {
    Buffer { src: (u32, u64), dst: u32, offset: u64, size: u64 },
    Texture { src: (u32, u64), dst: u32, mip: u32 },
    InitTexture(u32),
}

struct Swapchain {
    raw: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
    rendered: Vec<vk::Semaphore>,
    layouts: Vec<vk::ImageLayout>,
    format: vk::Format,
    extent: vk::Extent2D,
    mode: vk::PresentModeKHR,
    transform: vk::SurfaceTransformFlagsKHR,
}

pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug: Option<(ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    surface_fn: khr::surface::Instance,
    surface: vk::SurfaceKHR,
    pdev: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    swapchain_fn: khr::swapchain::Device,
    /// Draws with a count a shader wrote; without them, `compaction` zeroes the records past it.
    draw_count_fn: Option<khr::draw_indirect_count::Device>,
    compaction: Option<PipelineId>,
    /// Several indirect draws in one call.
    multi_draw: bool,
    passes: HashMap<PassKey, vk::RenderPass>,
    /// Compiled pipelines, kept in `cache_file` between runs; `cache_dirty` once one was added.
    pipeline_cache: vk::PipelineCache,
    cache_file: Option<std::path::PathBuf>,
    cache_dirty: bool,
    /// `precompile`'s threads, how many are still at it, and whether one finished since the
    /// cache was last written.
    compile_threads: Vec<std::thread::JoinHandle<()>>,
    compiling: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    compiled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    allocator: ManuallyDrop<Allocator>,
    /// The memory types outside the GPU's own heaps (system memory), where a GPU-only allocation goes
    /// when the GPU's memory is full (`allocate`). None on a GPU that shares the system's memory.
    spill: u32,
    swapchain: Option<Swapchain>,
    frames: Vec<Frame>,
    slot: usize,
    image_index: Option<u32>,
    want_size: (u32, u32),
    vsync: bool,
    dirty_swapchain: bool,
    /// Drawn upright on a turned phone: Android calls every frame suboptimal then, which is no
    /// reason to make the swapchain again (resizes still come as out of date and `resize`).
    turned: bool,
    buffers: Slab<Buffer>,
    textures: Slab<Texture>,
    pipelines: Slab<Pipeline>,
    uploads: Vec<Upload>,
    /// Writes made before this frame slot's fence was waited on, kept on the CPU and staged right
    /// after that wait, in order (`ensure_slot_ready`). Staging them waited for the GPU at the
    /// first write of a frame, in the middle of the game's update, which then could not overlap
    /// the GPU's work (perf review: it showed as 3-15 ms of "solid cells" on the Mac).
    /// `WEFT_EARLY_WAIT=1` waits at the first write again, to compare.
    early: Vec<(Early, Vec<u8>)>,
    global_layout: vk::DescriptorSetLayout,
    global_pool: vk::DescriptorPool,
    global_set: vk::DescriptorSet,
    sampler_layout: vk::DescriptorSetLayout,
    sampler_pool: vk::DescriptorPool,
    sampler_set: vk::DescriptorSet,
    samplers: [vk::Sampler; 4],
    free_tex: Vec<u32>,
    next_tex: u32,
    /// What a freed texture's slot shows until it is taken again: whoever still has its number
    /// samples nothing rather than freed memory (a lost device on Intel, garbage elsewhere).
    blank_view: vk::ImageView,
    capture_requested: bool,
    capture: Option<(vk::Buffer, Allocation, u32, u32, vk::Format)>,
    device_name: String,
    /// `Backend::device_class` and `device_memory_mb`.
    device_class: u32,
    device_memory_mb: u64,
    queries: vk::QueryPool,
    ts_period: f32,
    gpu_times: Vec<f32>,
    /// Where `begin_frame`'s and `end_frame`'s time goes (waiting for the GPU, `vkAcquireNextImageKHR`,
    /// recording, `vkQueueSubmit`, `vkQueuePresentKHR`), logged each second (`log_submit`):
    /// whether the frame waits for the GPU, for the display or works.
    submit_split: SubmitSplit,
    /// What `Cmd::Upscale` runs (`upscale.rs`); none without one (the game's own TAA then).
    upscaler: Option<Box<dyn upscale::Upscaler>>,
    /// Ray queries and acceleration structures, where the device has them (`rt::wanted`).
    rt: Option<rt::Rt>,
}

#[derive(Default)]
struct SubmitSplit {
    /// This frame's `begin_frame`: waiting for the slot's last frame on the GPU, and the acquire.
    begun: [f32; 2],
    sum: [f64; 5],
    max: [f32; 5],
    frames: u32,
    since: Option<std::time::Instant>,
    mode: i32,
}

// Raw Vulkan handles are plain integers/pointers that may be used from any thread as long as access
// is externally synchronized, which `&mut self` guarantees.
unsafe impl Send for VulkanBackend {}

unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _ty: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user: *mut c_void,
) -> vk::Bool32 {
    let msg = unsafe { (*data).message_as_c_str() }
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        log::error!(target: "vulkan", "{msg}");
    } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        log::warn!(target: "vulkan", "{msg}");
    } else {
        log::debug!(target: "vulkan", "{msg}");
    }
    vk::FALSE
}

fn err<E: std::fmt::Debug>(what: &str) -> impl FnOnce(E) -> Error + '_ {
    move |e| Error(format!("{what}: {e:?}"))
}

/// The system's Vulkan loader. macOS has none of its own: a release carries MoltenVK beside the
/// game, which stands in for the loader and goes first, since a loader found without a driver
/// fails only later. Built from source, the loader is also looked for by path, as Homebrew's
/// prefix is not searched.
/// With Streamline (the `streamline` feature and its files present), its interposer stands in for
/// the loader, so it can add what DLSS needs to the instance and device (`streamline.rs`).
unsafe fn load_vulkan(early: &upscale::Early) -> Result<ash::Entry, Error> {
    #[cfg(feature = "streamline")]
    if let upscale::Early::Streamline(sl) = early {
        match unsafe { ash::Entry::load_from(&sl.interposer) } {
            Ok(entry) => return Ok(entry),
            Err(e) => log::warn!("streamline: Vulkan from {}: {e:?}", sl.interposer.display()),
        }
    }
    let _ = early;
    if cfg!(target_os = "macos") {
        let beside = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join("libMoltenVK.dylib")));
        if let Some(path) = beside.filter(|p| p.is_file()) {
            match unsafe { ash::Entry::load_from(&path) } {
                Ok(entry) => return Ok(entry),
                Err(e) => log::warn!("{}: {e:?}", path.display()),
            }
        }
    }
    let first = match unsafe { ash::Entry::load() } {
        Ok(entry) => return Ok(entry),
        Err(e) => e,
    };
    if cfg!(target_os = "macos") {
        for dir in ["", "/opt/homebrew/lib/", "/usr/local/lib/"] {
            for lib in ["libvulkan.1.dylib", "libvulkan.dylib", "libMoltenVK.dylib"] {
                if let Ok(entry) = unsafe { ash::Entry::load_from(format!("{dir}{lib}")) } {
                    return Ok(entry);
                }
            }
        }
        return Err(Error(format!("loading Vulkan: {first:?}. On macOS it comes from MoltenVK: brew install molten-vk vulkan-loader")));
    }
    Err(Error(format!("loading Vulkan: {first:?}")))
}

impl VulkanBackend {
    pub fn new(desc: &BackendDesc) -> Result<VulkanBackend, Error> {
        unsafe { Self::create(desc) }
    }

    unsafe fn create(desc: &BackendDesc) -> Result<VulkanBackend, Error> {
        unsafe {
            let mut early = upscale::early();
            let entry = load_vulkan(&early)?;
            let display = desc
                .window
                .display_handle()
                .map_err(err("display handle"))?
                .as_raw();
            let window = desc
                .window
                .window_handle()
                .map_err(err("window handle"))?
                .as_raw();

            let mut exts: Vec<*const c_char> = ash_window::enumerate_required_extensions(display)
                .map_err(err("surface extensions"))?
                .to_vec();
            let mut layers: Vec<*const c_char> = Vec::new();
            let validation = c"VK_LAYER_KHRONOS_validation";
            let debug = desc.debug
                && entry
                    .enumerate_instance_layer_properties()
                    .unwrap_or_default()
                    .iter()
                    .any(|l| l.layer_name_as_c_str() == Ok(validation));
            if debug {
                layers.push(validation.as_ptr());
                exts.push(ext::debug_utils::NAME.as_ptr());
            } else if desc.debug {
                log::warn!("validation layer requested but not installed");
            }
            // macOS: MoltenVK's own errors in our log (a pipeline Metal would not build says why
            // only there: `calm world pipelines: ... ERROR_INITIALIZATION_FAILED` came with no
            // reason, 11-06), through a messenger for errors alone, no validation layer.
            let mvk_errors = !debug
                && cfg!(target_os = "macos")
                && entry
                    .enumerate_instance_extension_properties(None)
                    .unwrap_or_default()
                    .iter()
                    .any(|e| e.extension_name_as_c_str() == Ok(ext::debug_utils::NAME));
            if mvk_errors {
                exts.push(ext::debug_utils::NAME.as_ptr());
            }

            // MoltenVK is a portability driver: the loader hides those unless asked to list them.
            let portability = cfg!(target_os = "macos")
                && entry
                    .enumerate_instance_extension_properties(None)
                    .unwrap_or_default()
                    .iter()
                    .any(|e| e.extension_name_as_c_str() == Ok(khr::portability_enumeration::NAME));
            if portability {
                exts.push(khr::portability_enumeration::NAME.as_ptr());
            }
            // What the upscaler asks for (NGX), of what the loader has.
            for e in early.instance_extensions(
                &entry
                    .enumerate_instance_extension_properties(None)
                    .unwrap_or_default(),
            ) {
                if !exts
                    .iter()
                    .any(|x| std::ffi::CStr::from_ptr(*x) == std::ffi::CStr::from_ptr(e))
                {
                    exts.push(e);
                }
            }
            // The newest Vulkan the loader has, up to 1.3: the device still only needs 1.1, but ray
            // queries (`rt.rs`) and DLSS use what 1.2 brought where the GPU has it.
            let loader = entry
                .try_enumerate_instance_version()
                .ok()
                .flatten()
                .unwrap_or(vk::API_VERSION_1_0);
            let api = if loader >= vk::API_VERSION_1_3 {
                vk::API_VERSION_1_3
            } else if loader >= vk::API_VERSION_1_2 {
                vk::API_VERSION_1_2
            } else {
                vk::API_VERSION_1_1
            };
            let app = vk::ApplicationInfo::default()
                .application_name(c"Weft")
                .engine_name(c"Weft")
                .api_version(api);
            let instance = entry
                .create_instance(
                    &vk::InstanceCreateInfo::default()
                        .flags(if portability {
                            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
                        } else {
                            vk::InstanceCreateFlags::empty()
                        })
                        .application_info(&app)
                        .enabled_extension_names(&exts)
                        .enabled_layer_names(&layers),
                    None,
                )
                .map_err(err("vkCreateInstance"))?;

            let debug = if debug || mvk_errors {
                let f = ext::debug_utils::Instance::new(&entry, &instance);
                let severity = if debug {
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                } else {
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                };
                let info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                    .message_severity(severity)
                    .message_type(
                        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                    )
                    .pfn_user_callback(Some(debug_callback));
                let m = f
                    .create_debug_utils_messenger(&info, None)
                    .map_err(err("debug messenger"))?;
                Some((f, m))
            } else {
                None
            };

            let surface_fn = khr::surface::Instance::new(&entry, &instance);
            let surface = ash_window::create_surface(&entry, &instance, display, window, None).map_err(err("creating surface"))?;

            let mut best = None;
            for pdev in instance
                .enumerate_physical_devices()
                .map_err(err("enumerating GPUs"))?
            {
                let props = instance.get_physical_device_properties(pdev);
                if props.api_version < vk::API_VERSION_1_1 {
                    continue;
                }
                let families = instance.get_physical_device_queue_family_properties(pdev);
                let Some(qf) = families.iter().enumerate().position(|(i, f)| {
                    f.queue_flags
                        .contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
                        && surface_fn
                            .get_physical_device_surface_support(pdev, i as u32, surface)
                            .unwrap_or(false)
                }) else {
                    continue;
                };
                let score = match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                    _ => 1,
                };
                if best.map_or(true, |(_, _, s)| score > s) {
                    best = Some((pdev, qf as u32, score));
                }
            }
            let (pdev, queue_family, _) = best.ok_or_else(|| Error("no Vulkan 1.1 GPU with presentation".into()))?;
            let props = instance.get_physical_device_properties(pdev);
            let device_name = props
                .device_name_as_c_str()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let device_class = match props.device_type {
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::DISCRETE_GPU => 2,
                vk::PhysicalDeviceType::VIRTUAL_GPU | vk::PhysicalDeviceType::CPU => 3,
                _ => 0,
            };
            let memory = instance.get_physical_device_memory_properties(pdev);
            let device_memory_mb = memory.memory_heaps[..memory.memory_heap_count as usize]
                .iter()
                .filter(|h| h.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|h| h.size)
                .max()
                .unwrap_or(0)
                / (1024 * 1024);
            let available: Vec<_> = instance
                .enumerate_device_extension_properties(pdev)
                .unwrap_or_default()
                .iter()
                .filter_map(|e| e.extension_name_as_c_str().ok().map(|n| n.to_owned()))
                .collect();
            let has_ext = |name: &std::ffi::CStr| available.iter().any(|e| e.as_c_str() == name);
            let required = [khr::swapchain::NAME, ext::descriptor_indexing::NAME, ext::scalar_block_layout::NAME];
            let missing: Vec<_> = required
                .iter()
                .filter(|&&e| !has_ext(e))
                .map(|e| e.to_string_lossy())
                .collect();
            if !missing.is_empty() {
                return Err(Error(format!("{device_name} lacks what the game needs: {}", missing.join(", "))));
            }
            let mut indexing = vk::PhysicalDeviceDescriptorIndexingProperties::default();
            let mut driver = vk::PhysicalDeviceDriverProperties::default();
            // The driver's name where the GPU can say it (1.2, or the extension).
            let says_driver = (api >= vk::API_VERSION_1_2 && props.api_version >= vk::API_VERSION_1_2) || has_ext(khr::driver_properties::NAME);
            let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut indexing);
            if says_driver {
                props2 = props2.push_next(&mut driver);
            }
            instance.get_physical_device_properties2(pdev, &mut props2);
            // Which driver too: on a Mac MoltenVK or Mesa's KosmicKrisp can stand behind the
            // same GPU, and runs comparing them must say which they had.
            let text = |s: Result<&std::ffi::CStr, _>| {
                s.map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            };
            log::info!(
                "GPU: {device_name}, driver {} {} ({:?})",
                text(driver.driver_name_as_c_str()),
                text(driver.driver_info_as_c_str()),
                driver.driver_id
            );
            let images = indexing
                .max_per_stage_descriptor_update_after_bind_sampled_images
                .min(indexing.max_descriptor_set_update_after_bind_sampled_images);
            if images < MAX_TEXTURES {
                return Err(Error(format!("{device_name} can bind {images} textures at once, the game needs {MAX_TEXTURES}")));
            }

            let mut has16 = vk::PhysicalDevice16BitStorageFeatures::default();
            let mut has_indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::default();
            let mut has_scalar = vk::PhysicalDeviceScalarBlockLayoutFeatures::default();
            let mut has = vk::PhysicalDeviceFeatures2::default()
                .push_next(&mut has16)
                .push_next(&mut has_indexing)
                .push_next(&mut has_scalar);
            instance.get_physical_device_features2(pdev, &mut has);
            let base = has.features;
            let missing: Vec<&str> = [
                ("drawIndirectFirstInstance", base.draw_indirect_first_instance),
                ("samplerAnisotropy", base.sampler_anisotropy),
                ("shaderInt16", base.shader_int16),
                ("storageBuffer16BitAccess", has16.storage_buffer16_bit_access),
                ("runtimeDescriptorArray", has_indexing.runtime_descriptor_array),
                ("descriptorBindingPartiallyBound", has_indexing.descriptor_binding_partially_bound),
                ("descriptorBindingSampledImageUpdateAfterBind", has_indexing.descriptor_binding_sampled_image_update_after_bind),
                ("shaderSampledImageArrayNonUniformIndexing", has_indexing.shader_sampled_image_array_non_uniform_indexing),
                ("scalarBlockLayout", has_scalar.scalar_block_layout),
            ]
            .into_iter()
            .filter(|f| f.1 == vk::FALSE)
            .map(|f| f.0)
            .collect();
            if !missing.is_empty() {
                return Err(Error(format!("{device_name} lacks what the game needs: {}", missing.join(", "))));
            }
            let multi_draw = base.multi_draw_indirect != vk::FALSE && props.limits.max_draw_indirect_count > 1;
            // Some drivers have the extension and still draw one at a time.
            let draw_count = multi_draw && has_ext(khr::draw_indirect_count::NAME);
            log::info!("multi-draw: {multi_draw}, draw counts: {draw_count}");

            let features = vk::PhysicalDeviceFeatures::default()
                .multi_draw_indirect(multi_draw)
                .draw_indirect_first_instance(true)
                .sampler_anisotropy(true)
                .shader_int16(true);
            let mut f16 = vk::PhysicalDevice16BitStorageFeatures::default().storage_buffer16_bit_access(true);
            // Texture slots are written while the frame before may still be drawing, which only
            // this makes legal; where it is missing, it was so before too.
            let unused_while_pending = has_indexing.descriptor_binding_update_unused_while_pending != vk::FALSE;
            let mut f_indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::default()
                .runtime_descriptor_array(true)
                .descriptor_binding_partially_bound(true)
                .descriptor_binding_sampled_image_update_after_bind(true)
                .descriptor_binding_update_unused_while_pending(unused_while_pending)
                .shader_sampled_image_array_non_uniform_indexing(true);
            let mut f_scalar = vk::PhysicalDeviceScalarBlockLayoutFeatures::default().scalar_block_layout(true);
            let mut f2 = vk::PhysicalDeviceFeatures2::default()
                .features(features)
                .push_next(&mut f16)
                .push_next(&mut f_indexing)
                .push_next(&mut f_scalar);
            let mut dev_exts: Vec<_> = required.iter().map(|e| e.as_ptr()).collect();
            if draw_count {
                dev_exts.push(khr::draw_indirect_count::NAME.as_ptr());
            }
            // Ray queries (`rt.rs`): Vulkan 1.2's device addresses and both extensions, on the
            // platforms `rt::wanted` allows. The TLAS is bound like the buffers, in each draw's set.
            let mut has_bda = vk::PhysicalDeviceBufferDeviceAddressFeatures::default();
            let mut has_accel = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
            let mut has_query = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
            let ray_query = rt::wanted()
                && api >= vk::API_VERSION_1_2
                && props.api_version >= vk::API_VERSION_1_2
                && has_ext(khr::acceleration_structure::NAME)
                && has_ext(khr::ray_query::NAME)
                && has_ext(khr::deferred_host_operations::NAME)
                && {
                    let mut f = vk::PhysicalDeviceFeatures2::default()
                        .push_next(&mut has_bda)
                        .push_next(&mut has_accel)
                        .push_next(&mut has_query);
                    instance.get_physical_device_features2(pdev, &mut f);
                    has_bda.buffer_device_address != vk::FALSE
                        && has_accel.acceleration_structure != vk::FALSE
                        && has_query.ray_query != vk::FALSE
                };
            let mut bda_features = vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);
            let mut accel_features = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default().acceleration_structure(true);
            let mut query_features = vk::PhysicalDeviceRayQueryFeaturesKHR::default().ray_query(true);
            if ray_query {
                dev_exts.push(khr::acceleration_structure::NAME.as_ptr());
                dev_exts.push(khr::ray_query::NAME.as_ptr());
                dev_exts.push(khr::deferred_host_operations::NAME.as_ptr());
                log::info!("{device_name}: ray queries");
            }
            // A driver that has this extension requires it to be enabled.
            if has_ext(khr::portability_subset::NAME) {
                dev_exts.push(khr::portability_subset::NAME.as_ptr());
            }
            // What the upscaler asks for (NGX), of what the GPU has.
            let more = early.device_extensions(&instance, pdev, &dev_exts);
            dev_exts.extend(more);
            let prio = [1.0];
            let qinfo = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&prio)];
            let mut device_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&qinfo)
                .enabled_extension_names(&dev_exts)
                .push_next(&mut f2);
            if ray_query {
                device_info = device_info
                    .push_next(&mut bda_features)
                    .push_next(&mut accel_features)
                    .push_next(&mut query_features);
            }
            let device = instance
                .create_device(pdev, &device_info, None)
                .map_err(err("vkCreateDevice"))?;
            let queue = device.get_device_queue(queue_family, 0);

            let allocator = Allocator::new(&AllocatorCreateDesc {
                instance: instance.clone(),
                device: device.clone(),
                physical_device: pdev,
                debug_settings: Default::default(),
                buffer_device_address: ray_query,
                allocation_sizes: Default::default(),
            })
            .map_err(err("GPU allocator"))?;
            let spill = spill_types(&instance.get_physical_device_memory_properties(pdev));

            let swapchain_fn = khr::swapchain::Device::new(&instance, &device);
            let draw_count_fn = draw_count.then(|| khr::draw_indirect_count::Device::new(&instance, &device));
            // The driver checks the data is its own and starts empty otherwise.
            let cache_file = desc.cache.map(|d| d.join("vulkan_pipelines.bin"));
            let cached = cache_file
                .as_ref()
                .and_then(|f| std::fs::read(f).ok())
                .unwrap_or_default();
            let pipeline_cache = device
                .create_pipeline_cache(&vk::PipelineCacheCreateInfo::default().initial_data(&cached), None)
                .or_else(|_| device.create_pipeline_cache(&Default::default(), None))
                .map_err(err("pipeline cache"))?;

            let mut frames = Vec::new();
            for _ in 0..FRAMES {
                let pool = device
                    .create_command_pool(
                        &vk::CommandPoolCreateInfo::default()
                            .queue_family_index(queue_family)
                            .flags(vk::CommandPoolCreateFlags::TRANSIENT),
                        None,
                    )
                    .map_err(err("command pool"))?;
                let cmd = device
                    .allocate_command_buffers(
                        &vk::CommandBufferAllocateInfo::default()
                            .command_pool(pool)
                            .command_buffer_count(1),
                    )
                    .map_err(err("command buffer"))?[0];
                frames.push(Frame {
                    pool,
                    cmd,
                    acquired: device
                        .create_semaphore(&Default::default(), None)
                        .map_err(err("semaphore"))?,
                    fence: device
                        .create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None)
                        .map_err(err("fence"))?,
                    staging: Staging { chunks: Vec::new(), current: 0, used: 0 },
                    deferred: Vec::new(),
                    set_pools: Vec::new(),
                    set_pool: 0,
                    framebuffers: Vec::new(),
                    compacted: None,
                    waited: false,
                    marks: 0,
                });
            }

            let sampler = |filter: vk::Filter, mode: vk::SamplerAddressMode, mips: bool| {
                // The filtered sampler gets anisotropy: block faces at grazing angles stay sharp.
                let aniso = filter == vk::Filter::LINEAR && mips;
                device.create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(filter)
                        .min_filter(filter)
                        .mipmap_mode(if mips { vk::SamplerMipmapMode::LINEAR } else { vk::SamplerMipmapMode::NEAREST })
                        .address_mode_u(mode)
                        .address_mode_v(mode)
                        .address_mode_w(mode)
                        .anisotropy_enable(aniso)
                        .max_anisotropy(if aniso { 8.0 } else { 1.0 })
                        .max_lod(if mips { vk::LOD_CLAMP_NONE } else { 0.0 }),
                    None,
                )
            };
            let samplers = [
                sampler(vk::Filter::NEAREST, vk::SamplerAddressMode::CLAMP_TO_EDGE, false).map_err(err("sampler"))?,
                sampler(vk::Filter::LINEAR, vk::SamplerAddressMode::CLAMP_TO_EDGE, false).map_err(err("sampler"))?,
                sampler(vk::Filter::NEAREST, vk::SamplerAddressMode::CLAMP_TO_EDGE, true).map_err(err("sampler"))?,
                sampler(vk::Filter::LINEAR, vk::SamplerAddressMode::CLAMP_TO_EDGE, true).map_err(err("sampler"))?,
            ];

            let mut flags = vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
            if unused_while_pending {
                flags |= vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING;
            }
            let binding_flags = [flags];
            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(MAX_TEXTURES)
                .stage_flags(vk::ShaderStageFlags::ALL)];
            let mut flag_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);
            let global_layout = device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default()
                        .bindings(&bindings)
                        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                        .push_next(&mut flag_info),
                    None,
                )
                .map_err(err("global set layout"))?;
            let sizes = [vk::DescriptorPoolSize { ty: vk::DescriptorType::SAMPLED_IMAGE, descriptor_count: MAX_TEXTURES }];
            let global_pool = device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .pool_sizes(&sizes)
                        .max_sets(1)
                        .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND),
                    None,
                )
                .map_err(err("descriptor pool"))?;
            let layouts = [global_layout];
            let global_set = device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(global_pool)
                        .set_layouts(&layouts),
                )
                .map_err(err("global set"))?[0];

            // The samplers, a set of their own (`SAMPLER_SET`): in the textures' set, Mali's driver
            // samples nothing through them. Fixed in the layout, as they never change.
            let sampler_bindings: Vec<_> = samplers
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(i as u32)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(vk::ShaderStageFlags::ALL)
                        .immutable_samplers(std::slice::from_ref(s))
                })
                .collect();
            let sampler_layout = device
                .create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&sampler_bindings), None)
                .map_err(err("sampler set layout"))?;
            let sizes = [vk::DescriptorPoolSize { ty: vk::DescriptorType::SAMPLER, descriptor_count: samplers.len() as u32 }];
            let sampler_pool = device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .pool_sizes(&sizes)
                        .max_sets(1),
                    None,
                )
                .map_err(err("sampler pool"))?;
            let layouts = [sampler_layout];
            let sampler_set = device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(sampler_pool)
                        .set_layouts(&layouts),
                )
                .map_err(err("sampler set"))?[0];

            let queries = device
                .create_query_pool(
                    &vk::QueryPoolCreateInfo::default()
                        .query_type(vk::QueryType::TIMESTAMP)
                        .query_count(MAX_MARKS * FRAMES as u32),
                    None,
                )
                .map_err(err("query pool"))?;
            let upscaler =
                upscale::ready(early, &upscale::Made { entry: &entry, instance: &instance, pdev, device: &device }, desc.upscale_standin);
            let mut backend = VulkanBackend {
                _entry: entry,
                instance,
                debug,
                surface_fn,
                surface,
                pdev,
                device,
                queue,
                swapchain_fn,
                draw_count_fn,
                compaction: None,
                multi_draw,
                passes: HashMap::new(),
                pipeline_cache,
                cache_file,
                cache_dirty: false,
                compile_threads: Vec::new(),
                compiling: Default::default(),
                compiled: Default::default(),
                allocator: ManuallyDrop::new(allocator),
                spill,
                swapchain: None,
                frames,
                slot: 0,
                image_index: None,
                want_size: (desc.width, desc.height),
                vsync: desc.vsync,
                dirty_swapchain: true,
                turned: false,
                buffers: Slab::default(),
                textures: Slab::default(),
                pipelines: Slab::default(),
                uploads: Vec::new(),
                early: Vec::new(),
                global_layout,
                global_pool,
                global_set,
                sampler_layout,
                sampler_pool,
                sampler_set,
                samplers,
                free_tex: Vec::new(),
                next_tex: 0,
                blank_view: vk::ImageView::null(),
                capture_requested: false,
                capture: None,
                device_name,
                device_class,
                device_memory_mb,
                queries,
                ts_period: props.limits.timestamp_period,
                gpu_times: Vec::new(),
                submit_split: SubmitSplit::default(),
                upscaler,
                rt: None,
            };
            if ray_query {
                backend.rt = Some(rt::Rt::new(&backend.instance, &backend.device, backend.pdev)?);
            }
            backend.recreate_swapchain()?;
            let blank = backend.create_texture(
                &TextureDesc { width: 1, height: 1, mips: 1, format: Format::Rgba8Srgb, render_target: false },
                "freed slot",
            );
            backend.write_texture(blank, 0, &[0; 4]);
            backend.blank_view = backend.textures.get(blank.0).view;
            if backend.draw_count_fn.is_none() {
                let shader = Shader::from_wgsl("draw compaction", DRAW_COMPACTION).map_err(|e| Error(e.0))?;
                backend.compaction =
                    Some(backend.create_compute(&ComputeDesc { label: "draw compaction", shader: &shader, entry: "main" })?);
            }
            Ok(backend)
        }
    }

    /// Adds a frame to `submit_split`; once a second, with `--log frame=debug`, a line of where
    /// the frame's GPU calls spent it: in `begin_frame` (the app's "acquire") waiting for this
    /// slot's frame before last to finish on the GPU (long when the GPU is the bottleneck: the
    /// 3456 x 2234 rows, 11-04) and acquiring an image; in `end_frame` (the app's "submit")
    /// recording our commands, `vkQueueSubmit` (on MoltenVK the Metal encoding, and waiting for a
    /// drawable to draw into: the display) and `vkQueuePresentKHR`.
    fn log_submit(&mut self, began: std::time::Instant, queued: std::time::Instant, submitted: std::time::Instant) {
        let now = std::time::Instant::now();
        let s = &mut self.submit_split;
        let ends = [queued - began, submitted - queued, now - submitted].map(|d| d.as_secs_f32() * 1000.0);
        let parts = [s.begun[0], s.begun[1], ends[0], ends[1], ends[2]];
        for ((sum, max), p) in s.sum.iter_mut().zip(&mut s.max).zip(parts) {
            *sum += p as f64;
            *max = max.max(p);
        }
        s.frames += 1;
        let since = *s.since.get_or_insert(began);
        if now - since < std::time::Duration::from_secs(1) {
            return;
        }
        let n = s.frames.max(1) as f64;
        log::debug!(
            target: "frame",
            "submit split: gpu wait {:.2} (max {:.2}), acquire {:.2} (max {:.2}), record {:.2} (max {:.2}), queue {:.2} (max {:.2}), present {:.2} (max {:.2}) ms over {} frames, {:?}",
            s.sum[0] / n,
            s.max[0],
            s.sum[1] / n,
            s.max[1],
            s.sum[2] / n,
            s.max[2],
            s.sum[3] / n,
            s.max[3],
            s.sum[4] / n,
            s.max[4],
            s.frames,
            vk::PresentModeKHR::from_raw(s.mode)
        );
        *s = SubmitSplit { mode: s.mode, since: Some(now), ..SubmitSplit::default() };
    }

    unsafe fn recreate_swapchain(&mut self) -> Result<(), Error> {
        unsafe {
            let caps = self
                .surface_fn
                .get_physical_device_surface_capabilities(self.pdev, self.surface)
                .map_err(err("surface caps"))?;
            let extent = if caps.current_extent.width != u32::MAX {
                caps.current_extent
            } else {
                vk::Extent2D {
                    width: self
                        .want_size
                        .0
                        .clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                    height: self
                        .want_size
                        .1
                        .clamp(caps.min_image_extent.height, caps.max_image_extent.height),
                }
            };
            // Drawn upright (`transform` below), the picture is the window's way round, whichever
            // way a phone held sideways gives its size.
            let upright = caps
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY);
            let quarter = caps
                .current_transform
                .intersects(vk::SurfaceTransformFlagsKHR::ROTATE_90 | vk::SurfaceTransformFlagsKHR::ROTATE_270);
            let (w, h) = self.want_size;
            let extent = if upright && quarter && w > 0 && h > 0 && (w > h) != (extent.width > extent.height) {
                vk::Extent2D { width: extent.height, height: extent.width }
            } else {
                extent
            };
            if extent.width == 0 || extent.height == 0 {
                return Ok(());
            }
            self.device.device_wait_idle().ok();

            let formats = self
                .surface_fn
                .get_physical_device_surface_formats(self.pdev, self.surface)
                .map_err(err("surface formats"))?;
            let format = formats
                .iter()
                .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .or_else(|| {
                    formats
                        .iter()
                        .find(|f| f.format == vk::Format::R8G8B8A8_SRGB)
                })
                .unwrap_or(&formats[0]);
            let modes = self
                .surface_fn
                .get_physical_device_surface_present_modes(self.pdev, self.surface)
                .map_err(err("present modes"))?;
            // Off is as many frames as come: mailbox caps them at the refresh rate on some drivers.
            use vk::PresentModeKHR as P;
            let want = if self.vsync { [P::FIFO; 2] } else { [P::IMMEDIATE, P::MAILBOX] };
            let mode = want
                .into_iter()
                .find(|m| modes.contains(m))
                .unwrap_or(P::FIFO);
            // Drawn upright: a phone held sideways has the compositor turn it, rather than the
            // picture coming out turned (`current_transform` there is a quarter turn).
            let transform = if upright { vk::SurfaceTransformFlagsKHR::IDENTITY } else { caps.current_transform };
            // Android's surfaces may offer only `INHERIT`.
            let alpha = [vk::CompositeAlphaFlagsKHR::OPAQUE, vk::CompositeAlphaFlagsKHR::INHERIT]
                .into_iter()
                .find(|&a| caps.supported_composite_alpha.contains(a))
                .unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE);
            let mut count = caps.min_image_count + 1;
            if caps.max_image_count > 0 {
                count = count.min(caps.max_image_count);
            }
            // A resize to the size it already has: remaking it for nothing crashed Intel Arc's driver at start.
            if let Some(sc) = &self.swapchain
                && sc.extent == extent
                && sc.format == format.format
                && sc.mode == mode
                && sc.transform == transform
            {
                self.dirty_swapchain = false;
                return Ok(());
            }
            self.turned = transform != caps.current_transform;
            // Destroyed before the new one instead of handed over as `old_swapchain`: nothing is in flight after the wait above.
            if let Some(old) = self.swapchain.take() {
                self.destroy_swapchain(old);
            }
            let raw = self
                .swapchain_fn
                .create_swapchain(
                    &vk::SwapchainCreateInfoKHR::default()
                        .surface(self.surface)
                        .min_image_count(count)
                        .image_format(format.format)
                        .image_color_space(format.color_space)
                        .image_extent(extent)
                        .image_array_layers(1)
                        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .pre_transform(transform)
                        .composite_alpha(alpha)
                        .present_mode(mode)
                        .clipped(true),
                    None,
                )
                .map_err(err("vkCreateSwapchainKHR"))?;
            let images = self
                .swapchain_fn
                .get_swapchain_images(raw)
                .map_err(err("swapchain images"))?;
            let mut views = Vec::new();
            let mut rendered = Vec::new();
            for &image in &images {
                views.push(
                    self.device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::default()
                                .image(image)
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(format.format)
                                .subresource_range(color_range(1)),
                            None,
                        )
                        .map_err(err("swapchain view"))?,
                );
                rendered.push(
                    self.device
                        .create_semaphore(&Default::default(), None)
                        .map_err(err("semaphore"))?,
                );
            }
            log::info!(
                "swapchain {}x{} {:?} {:?}, {:?} on a {:?} screen",
                extent.width,
                extent.height,
                format.format,
                mode,
                transform,
                caps.current_transform
            );
            self.submit_split.mode = mode.as_raw();
            self.swapchain = Some(Swapchain {
                raw,
                layouts: vec![vk::ImageLayout::UNDEFINED; images.len()],
                images,
                views,
                rendered,
                format: format.format,
                extent,
                mode,
                transform,
            });
            self.dirty_swapchain = false;
            Ok(())
        }
    }

    unsafe fn destroy_swapchain(&mut self, sc: Swapchain) {
        unsafe {
            for v in sc.views {
                self.device.destroy_image_view(v, None);
            }
            for s in sc.rendered {
                self.device.destroy_semaphore(s, None);
            }
            self.swapchain_fn.destroy_swapchain(sc.raw, None);
        }
    }

    fn vk_format(&self, f: Format) -> vk::Format {
        match f {
            Format::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
            Format::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
            Format::R8Unorm => vk::Format::R8_UNORM,
            Format::Rgba16Float => vk::Format::R16G16B16A16_SFLOAT,
            Format::Depth32 => DEPTH_FORMAT,
            Format::Surface => self
                .swapchain
                .as_ref()
                .map_or(vk::Format::B8G8R8A8_SRGB, |s| s.format),
        }
    }

    /// The slot's previous submission must be finished before its staging memory or command
    /// buffer is reused. Called lazily so uploads before `begin_frame` are safe too.
    fn ensure_slot_ready(&mut self) {
        let frame = &mut self.frames[self.slot];
        if frame.waited {
            return;
        }
        unsafe {
            self.device
                .wait_for_fences(&[frame.fence], true, u64::MAX)
                .ok();
        }
        frame.waited = true;
        frame.staging.current = 0;
        frame.staging.used = 0;
        for (to, data) in std::mem::take(&mut self.early) {
            let src = self.stage(&data);
            self.uploads.push(match to {
                Early::Buffer { dst, offset } => Upload::Buffer { src, dst, offset, size: data.len() as u64 },
                Early::Texture { dst, mip } => Upload::Texture { src, dst, mip },
            });
        }
        let frame = &mut self.frames[self.slot];
        if frame.marks > 1 {
            let mut ts = vec![0u64; frame.marks as usize];
            let first = self.slot as u32 * MAX_MARKS;
            if unsafe {
                self.device
                    .get_query_pool_results(self.queries, first, &mut ts, vk::QueryResultFlags::TYPE_64)
            }
            .is_ok()
            {
                let period = self.ts_period as f64 / 1e6;
                self.gpu_times = ts
                    .windows(2)
                    .map(|w| (w[1].saturating_sub(w[0]) as f64 * period) as f32)
                    .collect();
            }
        }
        for &pool in &frame.set_pools {
            unsafe {
                self.device
                    .reset_descriptor_pool(pool, vk::DescriptorPoolResetFlags::empty())
            }
            .ok();
        }
        frame.set_pool = 0;
        for f in frame.framebuffers.drain(..) {
            unsafe { self.device.destroy_framebuffer(f, None) };
        }
        let deferred = std::mem::take(&mut frame.deferred);
        for d in deferred {
            self.destroy_now(d);
        }
        self.rt_slot_ready();
    }

    /// Writes the compiled pipelines out, if any were added: once a frame at most, as a phone's
    /// app may be ended without notice.
    fn save_pipeline_cache(&mut self) {
        let added = std::mem::take(&mut self.cache_dirty)
            | self
                .compiled
                .swap(false, std::sync::atomic::Ordering::Relaxed);
        let Some(file) = self.cache_file.as_ref().filter(|_| added) else { return };
        let Ok(data) = (unsafe { self.device.get_pipeline_cache_data(self.pipeline_cache) }) else { return };
        let part = file.with_extension("part");
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(&part, &data))
            .and_then(|_| std::fs::rename(&part, file));
        if let Err(e) = written {
            log::warn!("{}: {e}", file.display());
        }
    }

    /// A set of `layout` from this frame's pools.
    unsafe fn buffer_set(&mut self, layout: vk::DescriptorSetLayout) -> vk::DescriptorSet {
        let layouts = [layout];
        loop {
            let frame = &mut self.frames[self.slot];
            if frame.set_pool == frame.set_pools.len() {
                let mut sizes = vec![
                    vk::DescriptorPoolSize { ty: vk::DescriptorType::STORAGE_BUFFER, descriptor_count: 8192 },
                    vk::DescriptorPoolSize { ty: vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 1024 },
                ];
                // The TLAS of ray queries (`Cmd::BindAccel`), where the device has them.
                if self.rt.is_some() {
                    sizes.push(vk::DescriptorPoolSize { ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR, descriptor_count: 256 });
                }
                let pool = unsafe {
                    self.device.create_descriptor_pool(
                        &vk::DescriptorPoolCreateInfo::default()
                            .pool_sizes(&sizes)
                            .max_sets(2048),
                        None,
                    )
                }
                .expect("descriptor pool");
                frame.set_pools.push(pool);
            }
            let info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(frame.set_pools[frame.set_pool])
                .set_layouts(&layouts);
            match unsafe { self.device.allocate_descriptor_sets(&info) } {
                Ok(sets) => return sets[0],
                Err(vk::Result::ERROR_OUT_OF_POOL_MEMORY | vk::Result::ERROR_FRAGMENTED_POOL) => frame.set_pool += 1,
                Err(e) => panic!("allocating a descriptor set: {e:?}"),
            }
        }
    }

    /// Binds the buffers `bound` to pipeline `p`'s set 1, those of its bindings that have one, and
    /// the acceleration structures `accels` (ray queries, `Cmd::BindAccel`) likewise.
    unsafe fn bind_buffers(
        &mut self,
        cmd: vk::CommandBuffer,
        p: u32,
        bound: &[(u32, vk::DescriptorBufferInfo)],
        accels: &[(u32, vk::AccelerationStructureKHR)],
    ) {
        let pl = self.pipelines.get(p);
        if pl.buffers.is_empty() {
            return;
        }
        let (layout, set_layout, bind_point) = (pl.layout, pl.set_layout, pl.bind_point);
        let set = unsafe { self.buffer_set(set_layout) };
        let pl = self.pipelines.get(p);
        let accel = vk::DescriptorType::ACCELERATION_STRUCTURE_KHR;
        let infos: Vec<_> = pl
            .buffers
            .iter()
            .filter(|&&(_, ty)| ty != accel)
            .filter_map(|&(b, ty)| bound.iter().find(|x| x.0 == b).map(|x| (b, ty, [x.1])))
            .collect();
        let raws: Vec<_> = pl
            .buffers
            .iter()
            .filter(|&&(_, ty)| ty == accel)
            .filter_map(|&(b, _)| accels.iter().find(|x| x.0 == b).map(|x| (b, [x.1])))
            .collect();
        let mut accel_writes: Vec<_> = raws
            .iter()
            .map(|(_, raw)| vk::WriteDescriptorSetAccelerationStructureKHR::default().acceleration_structures(raw))
            .collect();
        let mut writes: Vec<_> = infos
            .iter()
            .map(|(b, ty, info)| {
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(*b)
                    .descriptor_type(*ty)
                    .buffer_info(info)
            })
            .collect();
        for ((b, _), w) in raws.iter().zip(&mut accel_writes) {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(*b)
                    .descriptor_type(accel)
                    .descriptor_count(1)
                    .push_next(w),
            );
        }
        unsafe {
            self.device.update_descriptor_sets(&writes, &[]);
            self.device
                .cmd_bind_descriptor_sets(cmd, bind_point, layout, 1, &[set], &[]);
        }
    }

    /// A render pass of these attachments, which stay in their attachment layouts throughout:
    /// barriers around the pass move them.
    fn render_pass(&mut self, key: PassKey) -> vk::RenderPass {
        if let Some(&p) = self.passes.get(&key) {
            return p;
        }
        let mut attachments = Vec::new();
        let mut color_ref = Vec::new();
        for &(format, load) in &key.0 {
            color_ref.push(vk::AttachmentReference { attachment: attachments.len() as u32, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL });
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(format)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(load)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            );
        }
        let depth_ref =
            vk::AttachmentReference { attachment: attachments.len() as u32, layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL };
        if let Some(load) = key.1 {
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(DEPTH_FORMAT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(load)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            );
        }
        let mut subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_ref);
        if key.1.is_some() {
            subpass = subpass.depth_stencil_attachment(&depth_ref);
        }
        let subpasses = [subpass];
        let pass = unsafe {
            self.device.create_render_pass(
                &vk::RenderPassCreateInfo::default()
                    .attachments(&attachments)
                    .subpasses(&subpasses),
                None,
            )
        }
        .expect("vkCreateRenderPass");
        self.passes.insert(key, pass);
        pass
    }

    /// Draws `count` indirect records from `offset`, in one call where the GPU can.
    unsafe fn draw_indirect(&self, cmd: vk::CommandBuffer, buffer: vk::Buffer, offset: u64, count: u32, indexed: bool) {
        let stride = if indexed { size_of::<DrawIndexedArgs>() } else { size_of::<DrawArgs>() } as u32;
        let (calls, per_call) = if self.multi_draw || count <= 1 { (1, count) } else { (count, 1) };
        for i in 0..calls {
            let at = offset + (i * stride) as u64;
            unsafe {
                if indexed {
                    self.device
                        .cmd_draw_indexed_indirect(cmd, buffer, at, per_call, stride);
                } else {
                    self.device
                        .cmd_draw_indirect(cmd, buffer, at, per_call, stride);
                }
            }
        }
    }

    /// Without draws with a count: copies the records of the count draws in the pass starting at
    /// `cmds[0]` into this frame's compacted buffer, zeroing those past their count. Where each
    /// one's copy starts, in order.
    unsafe fn compact_draws(&mut self, cmd: vk::CommandBuffer, cmds: &[Cmd]) -> Vec<u64> {
        let Some(compaction) = self.compaction else { return Vec::new() };
        let (draws, total) = count_draws(cmds);
        if draws.is_empty() {
            return Vec::new();
        }
        let scratch = match self.frames[self.slot].compacted {
            Some((b, size)) if size >= total => b,
            old => {
                if let Some((b, _)) = old {
                    self.destroy_buffer(b);
                }
                let size = total.max(16).next_power_of_two();
                let b = self.create_buffer(size, usage::STORAGE | usage::INDIRECT, "compacted draws");
                self.frames[self.slot].compacted = Some((b, size));
                b
            }
        };
        let pl = self.pipelines.get(compaction.0);
        let (raw, layout) = (pl.raw, pl.layout);
        unsafe {
            // What wrote the counts is done, and so is any draw still reading the copies.
            self.memory_barrier(cmd);
            self.device
                .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, raw);
            self.device
                .cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, layout, 0, &[self.global_set], &[]);
            for d in &draws {
                let info = |b: BufferId| vk::DescriptorBufferInfo { buffer: self.buffers.get(b.0).raw, offset: 0, range: vk::WHOLE_SIZE };
                let bound = [(0, info(d.buffer)), (1, info(d.count_buffer)), (2, info(scratch))];
                self.bind_buffers(cmd, compaction.0, &bound, &[]);
                let bytes: Vec<u8> = d.args.iter().flat_map(|a| a.to_ne_bytes()).collect();
                self.device
                    .cmd_push_constants(cmd, layout, vk::ShaderStageFlags::ALL, 0, &bytes);
                self.device.cmd_dispatch(cmd, d.max.div_ceil(64), 1, 1);
            }
            self.memory_barrier(cmd);
        }
        draws.iter().map(|d| d.at).collect()
    }

    /// Frees `d` once the GPU is done with it: after the next wait on this slot's fence. That wait
    /// must not be one still to come for the submission before (a destroy between `end_frame` and
    /// `begin_frame`, like an asset reload's): it would only cover the frame two back, and the one
    /// just submitted could still be drawing with `d` when it goes (a GPU fault, which hangs the
    /// whole screen on Apple GPUs). So the slot is made ready first, as uploads do.
    fn defer(&mut self, d: Deferred) {
        self.ensure_slot_ready();
        self.frames[self.slot].deferred.push(d);
    }

    fn destroy_now(&mut self, d: Deferred) {
        unsafe {
            match d {
                Deferred::Buffer(b, a) => {
                    self.device.destroy_buffer(b, None);
                    self.allocator.free(a).ok();
                }
                Deferred::Texture(image, view, a, index) => {
                    if index != u32::MAX {
                        let info = [vk::DescriptorImageInfo::default()
                            .image_view(self.blank_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
                        let write = vk::WriteDescriptorSet::default()
                            .dst_set(self.global_set)
                            .dst_binding(0)
                            .dst_array_element(index)
                            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                            .image_info(&info);
                        self.device.update_descriptor_sets(&[write], &[]);
                        self.free_tex.push(index);
                    }
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.allocator.free(a).ok();
                }
                Deferred::Pipeline(p, l, s) => {
                    self.device.destroy_pipeline(p, None);
                    self.device.destroy_pipeline_layout(l, None);
                    self.device.destroy_descriptor_set_layout(s, None);
                }
                Deferred::Accel(raw, buffer, alloc) => self.destroy_accel_now(raw, buffer, alloc),
            }
        }
    }

    /// Copies `data` into this frame's upload memory: (chunk, offset).
    fn stage(&mut self, data: &[u8]) -> (u32, u64) {
        self.ensure_slot_ready();
        let size = data.len() as u64;
        let frame = &mut self.frames[self.slot];
        let st = &mut frame.staging;
        let mut offset = (st.used + 15) & !15;
        if st
            .chunks
            .get(st.current)
            .is_none_or(|c| offset + size > c.2)
        {
            // Move on to the next chunk that fits, making one if none does.
            if !st.chunks.is_empty() {
                st.current += 1;
            }
            while st.chunks.get(st.current).is_some_and(|c| size > c.2) {
                st.current += 1;
            }
            if st.current >= st.chunks.len() {
                let chunk = size.max(st.chunks.last().map_or(8 << 20, |c| c.2 * 2));
                let (buffer, alloc) = raw_buffer(
                    &self.device,
                    &mut self.allocator,
                    self.spill,
                    chunk,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    MemoryLocation::CpuToGpu,
                    "staging",
                );
                st.chunks.push((buffer, alloc, chunk));
                st.current = st.chunks.len() - 1;
            }
            offset = 0;
        }
        let c = &mut st.chunks[st.current];
        c.1.mapped_slice_mut().unwrap()[offset as usize..(offset + size) as usize].copy_from_slice(data);
        st.used = offset + size;
        (st.current as u32, offset)
    }

    unsafe fn barrier_image(
        &self,
        cmd: vk::CommandBuffer,
        image: vk::Image,
        range: vk::ImageSubresourceRange,
        old: vk::ImageLayout,
        new: vk::ImageLayout,
    ) {
        let ((src_stage, src_access), (dst_stage, dst_access)) = (layout_stages(old, true), layout_stages(new, false));
        let b = [vk::ImageMemoryBarrier::default()
            .src_access_mask(src_access)
            .dst_access_mask(dst_access)
            .old_layout(old)
            .new_layout(new)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)];
        unsafe {
            self.device
                .cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &[], &[], &b)
        };
    }

    unsafe fn memory_barrier(&self, cmd: vk::CommandBuffer) {
        let b = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE | vk::AccessFlags::HOST_READ)];
        unsafe {
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS | vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &b,
                &[],
                &[],
            )
        };
    }

    /// Orders copies: those recorded after it start once those before it are written.
    unsafe fn copy_barrier(&self, cmd: vk::CommandBuffer) {
        let b = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)];
        let copy = vk::PipelineStageFlags::TRANSFER;
        unsafe {
            self.device
                .cmd_pipeline_barrier(cmd, copy, copy, vk::DependencyFlags::empty(), &b, &[], &[])
        };
    }

    unsafe fn record_uploads(&mut self, cmd: vk::CommandBuffer) {
        if self.uploads.is_empty() {
            return;
        }
        unsafe {
            self.memory_barrier(cmd);
            let chunks: Vec<vk::Buffer> = self.frames[self.slot]
                .staging
                .chunks
                .iter()
                .map(|c| c.0)
                .collect();
            let uploads = std::mem::take(&mut self.uploads);
            // Byte ranges copied into each buffer since the last barrier. Copies are unordered
            // without one, so a later write to the same bytes could land first and be overwritten
            // by the older data; a copy that overlaps one of these waits for them. Keyed by
            // (buffer, start) to the end; the ranges never overlap, so only the one starting
            // last before a copy's end can overlap it.
            let mut written: std::collections::BTreeMap<(u32, u64), u64> = Default::default();
            for u in &uploads {
                match *u {
                    Upload::Buffer { src, dst, offset, size } => {
                        let Some(Some(b)) = self.buffers.items.get(dst as usize) else { continue };
                        let end = offset + size;
                        if written
                            .range(..(dst, end))
                            .next_back()
                            .is_some_and(|(&(d, _), &e)| d == dst && e > offset)
                        {
                            self.copy_barrier(cmd);
                            written.clear();
                        }
                        written.insert((dst, offset), end);
                        let region = [vk::BufferCopy { src_offset: src.1, dst_offset: offset, size }];
                        self.device
                            .cmd_copy_buffer(cmd, chunks[src.0 as usize], b.raw, &region);
                    }
                    Upload::Texture { src, dst, mip } => {
                        let staging = chunks[src.0 as usize];
                        let src = src.1;
                        let Some(Some(t)) = self.textures.items.get(dst as usize) else { continue };
                        let range = color_range(t.desc.mips);
                        let (image, layout) = (t.image, t.layout);
                        self.barrier_image(cmd, image, range, layout, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
                        let region = [vk::BufferImageCopy::default()
                            .buffer_offset(src)
                            .image_subresource(vk::ImageSubresourceLayers {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                mip_level: mip,
                                base_array_layer: 0,
                                layer_count: 1,
                            })
                            .image_extent(vk::Extent3D {
                                width: (t.desc.width >> mip).max(1),
                                height: (t.desc.height >> mip).max(1),
                                depth: 1,
                            })];
                        self.device
                            .cmd_copy_buffer_to_image(cmd, staging, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &region);
                        self.barrier_image(
                            cmd,
                            image,
                            range,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        );
                        self.textures.get_mut(dst).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                    }
                    Upload::InitTexture(id) => {
                        let Some(Some(t)) = self.textures.items.get(id as usize) else { continue };
                        if t.layout != vk::ImageLayout::UNDEFINED {
                            continue;
                        }
                        let depth = t.desc.format == Format::Depth32;
                        let (range, new) = if depth {
                            (depth_range(), vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        } else {
                            (color_range(t.desc.mips), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        };
                        let image = t.image;
                        self.barrier_image(cmd, image, range, vk::ImageLayout::UNDEFINED, new);
                        self.textures.get_mut(id).layout = new;
                    }
                }
            }
            self.memory_barrier(cmd);
        }
    }
}

fn color_range(mips: u32) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: mips,
        base_array_layer: 0,
        layer_count: 1,
    }
}

fn depth_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::DEPTH,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    }
}

/// Whether the fragment entry `entry` of `shader` has an output at `@location(location)`.
fn writes_location(shader: &Shader, entry: &str, location: u32) -> bool {
    let m = &shader.module;
    let Some(ep) = m
        .entry_points
        .iter()
        .find(|e| e.name == entry && e.stage == naga::ShaderStage::Fragment)
    else {
        return false;
    };
    let Some(result) = &ep.function.result else { return false };
    let at = |b: &Option<naga::Binding>| matches!(b, Some(naga::Binding::Location { location: l, .. }) if *l == location);
    if at(&result.binding) {
        return true;
    }
    match &m.types[result.ty].inner {
        naga::TypeInner::Struct { members, .. } => members.iter().any(|mb| at(&mb.binding)),
        _ => false,
    }
}

/// The memory types off the GPU's own heaps, as a mask of their indices: system memory on a
/// discrete GPU, none where the GPU shares the system's memory.
fn spill_types(mem: &vk::PhysicalDeviceMemoryProperties) -> u32 {
    (0..mem.memory_type_count)
        .filter(|&i| {
            let heap = mem.memory_heaps[mem.memory_types[i as usize].heap_index as usize];
            !heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL)
        })
        .fold(0, |m, i| m | 1 << i)
}

/// Allocates for `desc`. When the GPU's memory is full, a GPU-only allocation goes to system memory
/// (`spill`), which the GPU reads over the bus: slower, but the game goes on instead of stopping.
fn allocate(allocator: &mut Allocator, desc: &AllocationCreateDesc, spill: u32) -> gpu_allocator::Result<Allocation> {
    match allocator.allocate(desc) {
        Err(gpu_allocator::AllocationError::OutOfMemory)
            if desc.location == MemoryLocation::GpuOnly && desc.requirements.memory_type_bits & spill != 0 =>
        {
            log::warn!(
                target: "vulkan",
                "GPU memory is full: {} ({} bytes) goes to system memory",
                desc.name,
                desc.requirements.size
            );
            let mut desc = desc.clone();
            desc.requirements.memory_type_bits &= spill;
            desc.location = MemoryLocation::Unknown;
            allocator.allocate(&desc)
        }
        r => r,
    }
}

fn raw_buffer(
    device: &ash::Device,
    allocator: &mut Allocator,
    spill: u32,
    size: u64,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
    name: &str,
) -> (vk::Buffer, Allocation) {
    unsafe {
        let buffer = device
            .create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(size.max(16))
                    .usage(usage),
                None,
            )
            .expect("vkCreateBuffer");
        let requirements = device.get_buffer_memory_requirements(buffer);
        let desc =
            AllocationCreateDesc { name, requirements, location, linear: true, allocation_scheme: AllocationScheme::GpuAllocatorManaged };
        let alloc = allocate(allocator, &desc, spill).unwrap_or_else(|e| panic!("allocating {size} bytes for {name}: {e}"));
        device
            .bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
            .expect("bind buffer memory");
        (buffer, alloc)
    }
}

/// Marks `NonUniform` each sampled image made of a texture that is: naga 30 marks only the texture,
/// and the rule wants the operand the sampling reads. NVIDIA doesn't mind; Mali samples zeros.
fn non_uniform_samples(words: &mut Vec<u32>) {
    const DECORATE: u32 = 71;
    const SAMPLED_IMAGE: u32 = 86;
    const NON_UNIFORM: u32 = 5300;
    // After the five words of the header: (word count << 16 | opcode) and the operands.
    let mut marked = Vec::new();
    let mut add = Vec::new();
    let mut first_decoration = None;
    let mut at = 5;
    while at < words.len() {
        let (count, op) = ((words[at] >> 16) as usize, words[at] & 0xffff);
        if count == 0 {
            return;
        }
        let operands = &words[at + 1..(at + count).min(words.len())];
        match op {
            DECORATE => {
                first_decoration.get_or_insert(at);
                if operands.get(1) == Some(&NON_UNIFORM) {
                    marked.push(operands[0]);
                }
            }
            // Result type, result, image, sampler.
            SAMPLED_IMAGE if operands.len() == 4 && marked.contains(&operands[2]) => add.push(operands[1]),
            _ => {}
        }
        at += count;
    }
    let Some(first) = first_decoration else { return };
    let new: Vec<u32> = add
        .iter()
        .flat_map(|&id| [3 << 16 | DECORATE, id, NON_UNIFORM])
        .collect();
    words.splice(first..first, new);
}

fn compare_op(c: Compare) -> vk::CompareOp {
    match c {
        Compare::Always => vk::CompareOp::ALWAYS,
        Compare::Greater => vk::CompareOp::GREATER,
        Compare::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
        Compare::Less => vk::CompareOp::LESS,
        Compare::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
        Compare::Equal => vk::CompareOp::EQUAL,
    }
}

impl VulkanBackend {
    /// `Cmd::Upscale`: the inputs made shader-readable, the output `GENERAL` (DLSS writes it as a
    /// storage image), the upscaler run, or the colour scaled up plainly if it could not, and the
    /// output left shader-readable for the effects.
    unsafe fn record_upscale(&mut self, cmd: vk::CommandBuffer, images: [TextureId; 4], guides: [TextureId; 4], params: &UpscaleParams) {
        let readable = |b: &mut Self, t: TextureId| {
            let tex = b.textures.get(t.0);
            let (image, was, depth) = (tex.image, tex.layout, tex.desc.format == Format::Depth32);
            if was != vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
                let range = if depth { depth_range() } else { color_range(1) };
                unsafe { b.barrier_image(cmd, image, range, was, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) };
                b.textures.get_mut(t.0).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            }
        };
        let [color, depth, motion, output] = images;
        for t in [color, depth, motion]
            .into_iter()
            .chain(guides.into_iter().filter(|g| *g != NO_TEXTURE))
        {
            readable(self, t);
        }
        let out = self.textures.get(output.0);
        let (out_image, out_was) = (out.image, out.layout);
        // The whole output is written: what was in it doesn't matter.
        unsafe { self.barrier_image(cmd, out_image, color_range(1), vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL) };
        let _ = out_was;
        self.textures.get_mut(output.0).layout = vk::ImageLayout::GENERAL;
        let describe = |b: &Self, t: TextureId| {
            let tex = b.textures.get(t.0);
            upscale::Image {
                image: tex.image,
                view: tex.view,
                memory: unsafe { tex.alloc.memory() },
                layout: tex.layout,
                format: b.vk_format(tex.desc.format),
                width: tex.desc.width,
                height: tex.desc.height,
                usage: tex.usage,
            }
        };
        let guide = |b: &Self, t: TextureId| (t != NO_TEXTURE).then(|| describe(b, t));
        let inputs = upscale::Inputs {
            color: describe(self, color),
            depth: describe(self, depth),
            motion: describe(self, motion),
            output: describe(self, output),
            normals: guide(self, guides[0]),
            albedo: guide(self, guides[1]),
            specular: guide(self, guides[2]),
            hit_distance: guide(self, guides[3]),
        };
        let done = match self.upscaler.as_mut() {
            Some(u) => unsafe { u.record(cmd, &inputs, params) },
            None => false,
        };
        if !done {
            // The colour scaled up to the output, filtered: the frame still has its picture.
            let (src, sw, sh) = (inputs.color.image, inputs.color.width, inputs.color.height);
            let (dw, dh) = (inputs.output.width, inputs.output.height);
            unsafe {
                self.barrier_image(
                    cmd,
                    src,
                    color_range(1),
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                );
                self.barrier_image(cmd, out_image, color_range(1), vk::ImageLayout::GENERAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
                let layers = vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                };
                let region = vk::ImageBlit {
                    src_subresource: layers,
                    src_offsets: [vk::Offset3D::default(), vk::Offset3D { x: sw as i32, y: sh as i32, z: 1 }],
                    dst_subresource: layers,
                    dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: dw as i32, y: dh as i32, z: 1 }],
                };
                self.device.cmd_blit_image(
                    cmd,
                    src,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    out_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[region],
                    vk::Filter::LINEAR,
                );
                self.barrier_image(
                    cmd,
                    src,
                    color_range(1),
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
                self.barrier_image(
                    cmd,
                    out_image,
                    color_range(1),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            }
        } else {
            // DLSS's compute writes, then the effects sample it.
            unsafe {
                self.barrier_image(cmd, out_image, color_range(1), vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            };
        }
        self.textures.get_mut(output.0).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    }

    fn make_texture(&mut self, desc: &TextureDesc, label: &str, extra: vk::ImageUsageFlags) -> TextureId {
        let format = self.vk_format(desc.format);
        let depth = desc.format == Format::Depth32;
        // Depth is also sampled: screen effects read how far each pixel is (it stays bindless,
        // read in SHADER_READ_ONLY_OPTIMAL between the passes that draw into it).
        let usage = if depth {
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC
        } else {
            let mut u = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
            if desc.render_target {
                u |= vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC;
            }
            u | extra
        };
        unsafe {
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D { width: desc.width, height: desc.height, depth: 1 })
                        .mip_levels(desc.mips)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(usage),
                    None,
                )
                .expect("vkCreateImage");
            let requirements = self.device.get_image_memory_requirements(image);
            let alloc_desc = AllocationCreateDesc {
                name: label,
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            };
            let alloc =
                allocate(&mut self.allocator, &alloc_desc, self.spill).unwrap_or_else(|e| panic!("allocating texture {label}: {e}"));
            self.device
                .bind_image_memory(image, alloc.memory(), alloc.offset())
                .expect("bind image memory");
            let range = if depth { depth_range() } else { color_range(desc.mips) };
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(range),
                    None,
                )
                .expect("vkCreateImageView");

            let index = {
                let index = self.free_tex.pop().unwrap_or_else(|| {
                    self.next_tex += 1;
                    self.next_tex - 1
                });
                assert!(index < MAX_TEXTURES, "more than {MAX_TEXTURES} textures");
                let info = [vk::DescriptorImageInfo::default()
                    .image_view(view)
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
                let write = vk::WriteDescriptorSet::default()
                    .dst_set(self.global_set)
                    .dst_binding(0)
                    .dst_array_element(index)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .image_info(&info);
                self.device.update_descriptor_sets(&[write], &[]);
                index
            };
            let id = self
                .textures
                .insert(Texture { image, view, alloc, desc: desc.clone(), layout: vk::ImageLayout::UNDEFINED, index, usage });
            self.uploads.push(Upload::InitTexture(id));
            TextureId(id)
        }
    }

    /// `create_pipeline`, with `extra` color images after the motion image (`create_pipeline_mrt`).
    fn create_pipeline_with(&mut self, desc: &PipelineDesc, extra: &[Format]) -> Result<PipelineId, Error> {
        let vs = self.spirv(desc.shader, naga::ShaderStage::Vertex, desc.vertex)?;
        let fs = match desc.fragment {
            Some(f) => Some(self.spirv(desc.shader, naga::ShaderStage::Fragment, f)?),
            None => None,
        };
        let (layout, set_layout, buffers) = self.layout_for(desc.shader)?;
        let vs_name = CString::new(desc.vertex).unwrap();
        let fs_name = CString::new(desc.fragment.unwrap_or("main")).unwrap();
        let mut stages = vec![
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vs)
                .name(&vs_name),
        ];
        if let Some(fs) = fs {
            stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(fs)
                    .name(&fs_name),
            );
        }
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(match desc.topology {
            Topology::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
            Topology::Lines => vk::PrimitiveTopology::LINE_LIST,
        });
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(if desc.cull_back { vk::CullModeFlags::BACK } else { vk::CullModeFlags::NONE })
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth = match desc.depth {
            Some(d) => vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(d.write)
                .depth_compare_op(compare_op(d.compare)),
            None => vk::PipelineDepthStencilStateCreateInfo::default(),
        };
        let blend_attachment = {
            let a = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA);
            match desc.blend {
                Blend::None => a,
                Blend::Alpha => a
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD),
                Blend::Premultiplied => a
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD),
                Blend::Additive => a
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD),
            }
        };
        // The motion image (`PipelineDesc::motion`): written, never blended, by a fragment entry
        // with a `@location(1)` output; left as it is by the rest.
        let written = |location: u32| {
            if desc
                .fragment
                .is_some_and(|f| writes_location(desc.shader, f, location))
            {
                vk::ColorComponentFlags::RGBA
            } else {
                vk::ColorComponentFlags::empty()
            }
        };
        let mut attachments = vec![blend_attachment, vk::PipelineColorBlendAttachmentState::default().color_write_mask(written(1))];
        // The extra color images (`create_pipeline_mrt`) after the motion image, at location 2 on:
        // written by a fragment entry with that output, never blended.
        for i in 0..extra.len() {
            attachments.push(vk::PipelineColorBlendAttachmentState::default().color_write_mask(written(2 + i as u32)));
        }
        let count = match (desc.color.is_some(), desc.motion) {
            (false, _) => 0,
            (true, false) => 1,
            (true, true) => 2 + extra.len(),
        };
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments[..count]);
        let dynamic = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic);
        // Any pass of the same formats will do for drawing with it.
        let load = vk::AttachmentLoadOp::LOAD;
        let colors: Vec<_> = desc
            .color
            .into_iter()
            .chain(desc.motion.then_some(MOTION_FORMAT))
            .chain(extra.iter().copied())
            .take(count)
            .map(|f| (self.vk_format(f), load))
            .collect();
        let pass = self.render_pass((colors, desc.depth.map(|_| load)));
        let info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .render_pass(pass);
        let result = unsafe {
            self.device
                .create_graphics_pipelines(self.pipeline_cache, &[info], None)
        };
        unsafe {
            self.device.destroy_shader_module(vs, None);
            if let Some(fs) = fs {
                self.device.destroy_shader_module(fs, None);
            }
        }
        let raw = result.map_err(|(_, e)| Error(format!("pipeline {}: {e:?}", desc.label)))?[0];
        self.cache_dirty = true;
        Ok(PipelineId(
            self.pipelines
                .insert(Pipeline { raw, layout, set_layout, bind_point: vk::PipelineBindPoint::GRAPHICS, buffers }),
        ))
    }

    fn spirv(&self, shader: &Shader, stage: naga::ShaderStage, entry: &str) -> Result<vk::ShaderModule, Error> {
        let mut options = naga::back::spv::Options::default();
        options.lang_version = (1, 3);
        // The prelude's samplers, group 0 bindings 1 to 4, are a set of their own here.
        for binding in 1..5 {
            options.binding_map.insert(
                naga::ResourceBinding { group: 0, binding },
                naga::back::spv::BindingInfo { descriptor_set: SAMPLER_SET, binding: binding - 1, binding_array_size: None },
            );
        }
        options.force_loop_bounding = false;
        let mut words = naga::back::spv::write_vec(
            &shader.module,
            &shader.info,
            &options,
            Some(&naga::back::spv::PipelineOptions { shader_stage: stage, entry_point: entry.to_string() }),
        )
        .map_err(|e| Error(format!("SPIR-V for {entry}: {e}")))?;
        non_uniform_samples(&mut words);
        unsafe {
            self.device
                .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)
                .map_err(err("shader module"))
        }
    }

    fn layout_for(&self, shader: &Shader) -> Result<(vk::PipelineLayout, vk::DescriptorSetLayout, Vec<(u32, vk::DescriptorType)>), Error> {
        let buffers: Vec<(u32, vk::DescriptorType)> = shader
            .buffers
            .iter()
            .map(|&(b, kind)| {
                let ty = match kind {
                    BufferKind::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                    BufferKind::Accel => vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                    _ => vk::DescriptorType::STORAGE_BUFFER,
                };
                (b, ty)
            })
            .collect();
        let bindings: Vec<_> = buffers
            .iter()
            .map(|&(b, ty)| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(b)
                    .descriptor_type(ty)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL)
            })
            .collect();
        unsafe {
            let set_layout = self
                .device
                .create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None)
                .map_err(err("buffer set layout"))?;
            let sets = [self.global_layout, set_layout, self.sampler_layout];
            let ranges = [vk::PushConstantRange { stage_flags: vk::ShaderStageFlags::ALL, offset: 0, size: shader.push_constant_size }];
            let mut info = vk::PipelineLayoutCreateInfo::default().set_layouts(&sets);
            if shader.push_constant_size > 0 {
                info = info.push_constant_ranges(&ranges);
            }
            let layout = self
                .device
                .create_pipeline_layout(&info, None)
                .map_err(err("pipeline layout"))?;
            Ok((layout, set_layout, buffers))
        }
    }
}

impl Backend for VulkanBackend {
    fn name(&self) -> &str {
        "vulkan"
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }

    fn device_class(&self) -> u32 {
        self.device_class
    }

    fn device_memory_mb(&self) -> u64 {
        self.device_memory_mb
    }

    fn frames_in_flight(&self) -> usize {
        FRAMES
    }

    fn surface_size(&self) -> (u32, u32) {
        self.swapchain
            .as_ref()
            .map_or(self.want_size, |s| (s.extent.width, s.extent.height))
    }

    fn resize(&mut self, width: u32, height: u32) {
        if self.want_size != (width, height) {
            self.want_size = (width, height);
            self.dirty_swapchain = true;
        }
    }

    fn set_vsync(&mut self, vsync: bool) {
        if self.vsync != vsync {
            self.vsync = vsync;
            self.dirty_swapchain = true;
        }
    }

    fn set_surface(&mut self, window: Option<&dyn Window>) {
        unsafe {
            self.device.device_wait_idle().ok();
            if let Some(sc) = self.swapchain.take() {
                self.destroy_swapchain(sc);
            }
            if self.surface != vk::SurfaceKHR::null() {
                self.surface_fn.destroy_surface(self.surface, None);
                self.surface = vk::SurfaceKHR::null();
            }
            let Some(window) = window else { return };
            let handles = window
                .display_handle()
                .and_then(|d| Ok((d.as_raw(), window.window_handle()?.as_raw())));
            match handles
                .map_err(err("window handles"))
                .and_then(|(display, window)| {
                    ash_window::create_surface(&self._entry, &self.instance, display, window, None).map_err(err("creating surface"))
                }) {
                Ok(surface) => {
                    self.surface = surface;
                    // Now, so what is made before the next frame gets the surface's format.
                    if let Err(e) = self.recreate_swapchain() {
                        log::error!("{e}");
                        self.dirty_swapchain = true;
                    }
                }
                Err(e) => log::error!("{e}"),
            }
        }
    }

    fn create_buffer(&mut self, size: u64, usage: u32, label: &str) -> BufferId {
        let mut flags = vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC;
        if usage & usage::STORAGE != 0 {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if usage & usage::INDEX != 0 {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if usage & usage::INDIRECT != 0 {
            flags |= vk::BufferUsageFlags::INDIRECT_BUFFER;
        }
        if usage & usage::UNIFORM != 0 {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if usage & usage::ACCEL_INPUT != 0 && self.rt.is_some() {
            flags |= vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS;
        }
        let location = if usage & usage::READBACK != 0 {
            MemoryLocation::GpuToCpu
        } else if usage & usage::HOST != 0 {
            MemoryLocation::CpuToGpu
        } else {
            MemoryLocation::GpuOnly
        };
        let (raw, alloc) = raw_buffer(&self.device, &mut self.allocator, self.spill, size, flags, location, label);
        BufferId(self.buffers.insert(Buffer { raw, alloc, size }))
    }

    fn mapped(&mut self, buffer: BufferId) -> &mut [u8] {
        let b = self.buffers.get_mut(buffer.0);
        let size = b.size as usize;
        &mut b
            .alloc
            .mapped_slice_mut()
            .expect("buffer is not host visible")[..size]
    }

    fn write_buffer(&mut self, buffer: BufferId, offset: u64, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        debug_assert!(offset + data.len() as u64 <= self.buffers.get(buffer.0).size);
        if !self.frames[self.slot].waited && !early_wait() {
            self.early
                .push((Early::Buffer { dst: buffer.0, offset }, data.to_vec()));
            return;
        }
        let src = self.stage(data);
        self.uploads
            .push(Upload::Buffer { src, dst: buffer.0, offset, size: data.len() as u64 });
    }

    fn destroy_buffer(&mut self, buffer: BufferId) {
        if let Some(b) = self.buffers.remove(buffer.0) {
            // Writes still queued for it go with it: its slot is free at once, and a buffer made
            // before the uploads run (a reload makes many) would take them, past its end.
            self.uploads
                .retain(|u| !matches!(u, Upload::Buffer { dst, .. } if *dst == buffer.0));
            self.early
                .retain(|e| !matches!(e.0, Early::Buffer { dst, .. } if dst == buffer.0));
            self.defer(Deferred::Buffer(b.raw, b.alloc));
        }
    }

    fn create_texture(&mut self, desc: &TextureDesc, label: &str) -> TextureId {
        self.make_texture(desc, label, vk::ImageUsageFlags::empty())
    }

    fn upscaler(&self) -> UpscalerInfo {
        self.upscaler
            .as_ref()
            .map_or_else(UpscalerInfo::default, |u| u.info())
    }

    fn upscale_render_size(&mut self, mode: UpscaleMode, reconstruct: bool, output: (u32, u32)) -> (u32, u32) {
        let s = mode.scale();
        let fallback = (((output.0 as f32 * s).round() as u32).max(1), ((output.1 as f32 * s).round() as u32).max(1));
        match &mut self.upscaler {
            Some(u) => u.render_size(mode, reconstruct, output).unwrap_or(fallback),
            None => fallback,
        }
    }

    /// Written by the upscaler's compute shaders (DLSS writes its output as a storage image).
    fn create_upscale_target(&mut self, width: u32, height: u32, label: &str) -> TextureId {
        let desc = TextureDesc { width, height, mips: 1, format: Format::Rgba16Float, render_target: true };
        self.make_texture(&desc, label, vk::ImageUsageFlags::STORAGE)
    }

    fn write_texture(&mut self, texture: TextureId, mip: u32, data: &[u8]) {
        if !self.frames[self.slot].waited && !early_wait() {
            self.early
                .push((Early::Texture { dst: texture.0, mip }, data.to_vec()));
            return;
        }
        let src = self.stage(data);
        self.uploads
            .push(Upload::Texture { src, dst: texture.0, mip });
    }

    fn texture_index(&self, texture: TextureId) -> u32 {
        self.textures.get(texture.0).index
    }

    fn destroy_texture(&mut self, texture: TextureId) {
        if let Some(t) = self.textures.remove(texture.0) {
            // As for buffers: uploads still queued for it must not reach a texture that takes its
            // slot.
            self.uploads.retain(|u| match u {
                Upload::Texture { dst, .. } => *dst != texture.0,
                Upload::InitTexture(id) => *id != texture.0,
                Upload::Buffer { .. } => true,
            });
            self.early
                .retain(|e| !matches!(e.0, Early::Texture { dst, .. } if dst == texture.0));
            self.defer(Deferred::Texture(t.image, t.view, t.alloc, t.index));
        }
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineId, Error> {
        self.create_pipeline_with(desc, &[])
    }

    fn create_pipeline_mrt(&mut self, desc: &PipelineDesc, extra: &[Format]) -> Result<PipelineId, Error> {
        self.create_pipeline_with(desc, extra)
    }

    fn create_compute(&mut self, desc: &ComputeDesc) -> Result<PipelineId, Error> {
        let cs = self.spirv(desc.shader, naga::ShaderStage::Compute, desc.entry)?;
        let (layout, set_layout, buffers) = self.layout_for(desc.shader)?;
        let name = CString::new(desc.entry).unwrap();
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(cs)
            .name(&name);
        let info = vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(layout);
        let result = unsafe {
            self.device
                .create_compute_pipelines(self.pipeline_cache, &[info], None)
        };
        unsafe { self.device.destroy_shader_module(cs, None) };
        let raw = result.map_err(|(_, e)| Error(format!("compute pipeline {}: {e:?}", desc.label)))?[0];
        self.cache_dirty = true;
        Ok(PipelineId(
            self.pipelines
                .insert(Pipeline { raw, layout, set_layout, bind_point: vk::PipelineBindPoint::COMPUTE, buffers }),
        ))
    }

    fn precompile(&mut self, desc: &ComputeDesc) {
        use std::sync::atomic::Ordering;
        let Ok(cs) = self.spirv(desc.shader, naga::ShaderStage::Compute, desc.entry) else { return };
        let Ok((layout, set_layout, _)) = self.layout_for(desc.shader) else {
            unsafe { self.device.destroy_shader_module(cs, None) };
            return;
        };
        let (device, cache, entry, label) =
            (self.device.clone(), self.pipeline_cache, CString::new(desc.entry).unwrap(), desc.label.to_string());
        let (left, done) = (self.compiling.clone(), self.compiled.clone());
        left.fetch_add(1, Ordering::SeqCst);
        // The pipeline itself goes: what stays is the driver's compiled code in the cache, which a
        // layout made alike finds again.
        self.compile_threads
            .push(std::thread::spawn(move || unsafe {
                let started = std::time::Instant::now();
                let stage = vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .module(cs)
                    .name(&entry);
                let info = vk::ComputePipelineCreateInfo::default()
                    .stage(stage)
                    .layout(layout);
                match device.create_compute_pipelines(cache, &[info], None) {
                    Ok(p) => device.destroy_pipeline(p[0], None),
                    Err((_, e)) => log::warn!("compiling {label} ahead: {e:?}"),
                }
                device.destroy_shader_module(cs, None);
                device.destroy_pipeline_layout(layout, None);
                device.destroy_descriptor_set_layout(set_layout, None);
                log::info!("compiled {label} in {:.1} s", started.elapsed().as_secs_f32());
                done.store(true, Ordering::Relaxed);
                left.fetch_sub(1, Ordering::SeqCst);
            }));
    }

    fn compiling(&self) -> bool {
        self.compiling.load(std::sync::atomic::Ordering::SeqCst) > 0
    }

    fn destroy_pipeline(&mut self, pipeline: PipelineId) {
        if let Some(p) = self.pipelines.remove(pipeline.0) {
            self.defer(Deferred::Pipeline(p.raw, p.layout, p.set_layout));
        }
    }

    fn ray_query(&self) -> bool {
        self.rt.is_some()
    }

    fn create_blas(&mut self, geometries: &[BlasGeometry], label: &str) -> AccelId {
        self.rt_create_blas(geometries, label)
    }

    fn create_tlas(&mut self, max_instances: u32, label: &str) -> AccelId {
        self.rt_create_tlas(max_instances, label)
    }

    fn set_tlas_instances(&mut self, tlas: AccelId, instances: &[TlasInstance]) {
        self.rt_set_instances(tlas, instances);
    }

    fn destroy_accel(&mut self, accel: AccelId) {
        if self.rt.is_some() {
            self.rt_destroy(accel);
        }
    }

    fn accel_memory(&self) -> u64 {
        self.rt.as_ref().map_or(0, |rt| rt.memory)
    }

    fn accel_generation(&self) -> u64 {
        self.rt.as_ref().map_or(0, |rt| rt.generation)
    }

    fn prepare_uploads(&mut self) {
        self.ensure_slot_ready();
    }

    fn begin_frame(&mut self) -> Option<FrameInfo> {
        let t0 = std::time::Instant::now();
        self.ensure_slot_ready();
        let t1 = std::time::Instant::now();
        if self.surface == vk::SurfaceKHR::null() {
            return None;
        }
        if self.dirty_swapchain || self.swapchain.is_none() {
            if let Err(e) = unsafe { self.recreate_swapchain() } {
                log::error!("{e}");
                return None;
            }
        }
        let sc = self.swapchain.as_ref()?;
        let acquired = self.frames[self.slot].acquired;
        let result = unsafe {
            self.swapchain_fn
                .acquire_next_image(sc.raw, u64::MAX, acquired, vk::Fence::null())
        };
        self.submit_split.begun = [t1 - t0, t1.elapsed()].map(|d| d.as_secs_f32() * 1000.0);
        match result {
            Ok((index, suboptimal)) => {
                if suboptimal && !self.turned {
                    self.dirty_swapchain = true;
                }
                self.image_index = Some(index);
                Some(FrameInfo { width: sc.extent.width, height: sc.extent.height, slot: self.slot })
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.dirty_swapchain = true;
                None
            }
            Err(e) => {
                log::error!("vkAcquireNextImageKHR: {e:?}");
                None
            }
        }
    }

    fn end_frame(&mut self, list: &CommandList) -> bool {
        let Some(image_index) = self.image_index.take() else { return false };
        let began = std::time::Instant::now();
        let queued;
        let submitted;
        let frame_cmd = self.frames[self.slot].cmd;
        let frame_pool = self.frames[self.slot].pool;
        unsafe {
            let d = &self.device;
            d.reset_command_pool(frame_pool, vk::CommandPoolResetFlags::empty())
                .ok();
            d.begin_command_buffer(frame_cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))
                .ok();
        }
        let cmd = frame_cmd;
        let presented = unsafe {
            let first_query = self.slot as u32 * MAX_MARKS;
            self.device
                .cmd_reset_query_pool(cmd, self.queries, first_query, MAX_MARKS);
            self.device
                .cmd_write_timestamp(cmd, vk::PipelineStageFlags::TOP_OF_PIPE, self.queries, first_query);
            self.record_uploads(cmd);
            self.rt_record_start(cmd);
            self.device
                .cmd_write_timestamp(cmd, vk::PipelineStageFlags::BOTTOM_OF_PIPE, self.queries, first_query + 1);
            let mut marks = 2;
            let sets = [self.global_set];
            let mut current: Option<u32> = None;
            // Buffers bound since, and whether the pipeline's set has them yet.
            let mut bound: Vec<(u32, vk::DescriptorBufferInfo)> = Vec::new();
            // The TLASes bound since (`Cmd::BindAccel`), which go in the same set.
            let mut accels: Vec<(u32, vk::AccelerationStructureKHR)> = Vec::new();
            let mut stale = false;
            // The last immediates, as bytes of `list.data`.
            let mut pushed: Option<(u32, u32)> = None;
            let mut compacted: Vec<u64> = Vec::new();
            let mut in_pass_targets: Vec<u32> = Vec::new();
            let mut in_pass_depth: Option<u32> = None;
            // Consecutive BLAS builds, recorded as one batch (`rt_build_blases`).
            let mut blas_batch: Vec<u32> = Vec::new();
            // `Cmd::PassExtras`: the color images the next pass adds after its motion image.
            let mut pass_extras: [Option<(TextureId, Option<[f32; 4]>)>; 3] = [None; 3];
            let (sc_image, sc_view, sc_extent) = {
                let sc = self.swapchain.as_ref().unwrap();
                (sc.images[image_index as usize], sc.views[image_index as usize], sc.extent)
            };
            for (i, cmd_item) in list.cmds.iter().enumerate() {
                if let Cmd::BuildBlas(a) = *cmd_item {
                    blas_batch.push(a.0);
                    continue;
                }
                if !blas_batch.is_empty() {
                    self.rt_build_blases(cmd, &blas_batch);
                    blas_batch.clear();
                }
                if stale
                    && matches!(
                        cmd_item,
                        Cmd::Draw { .. }
                            | Cmd::DrawIndirect { .. }
                            | Cmd::DrawIndexedIndirect { .. }
                            | Cmd::DrawIndexedIndirectCount { .. }
                            | Cmd::DrawIndirectCount { .. }
                            | Cmd::Dispatch { .. }
                    )
                    && let Some(p) = current
                {
                    self.bind_buffers(cmd, p, &bound, &accels);
                    stale = false;
                }
                match *cmd_item {
                    Cmd::PassExtras(e) => pass_extras = e,
                    Cmd::BeginPass { color, depth, motion } => {
                        compacted = self.compact_draws(cmd, &list.cmds[i + 1..]);
                        compacted.reverse();
                        // Its pipeline, buffers and immediates are not the list's: those come back.
                        if let (false, Some(p)) = (compacted.is_empty(), current) {
                            let pl = self.pipelines.get(p);
                            self.device.cmd_bind_pipeline(cmd, pl.bind_point, pl.raw);
                            self.device
                                .cmd_bind_descriptor_sets(cmd, pl.bind_point, pl.layout, 0, &sets, &[]);
                            self.device
                                .cmd_bind_descriptor_sets(cmd, pl.bind_point, pl.layout, SAMPLER_SET, &[self.sampler_set], &[]);
                            if let Some((start, len)) = pushed {
                                let bytes = &list.data[start as usize..(start + len) as usize];
                                self.device
                                    .cmd_push_constants(cmd, pl.layout, vk::ShaderStageFlags::ALL, 0, bytes);
                            }
                            stale = true;
                        }
                        let mut extent = sc_extent;
                        let mut color_att = None;
                        if let Some((target, clear)) = color {
                            let (image, view, old) = match target {
                                Target::Surface => {
                                    let sc = self.swapchain.as_mut().unwrap();
                                    let old = sc.layouts[image_index as usize];
                                    sc.layouts[image_index as usize] = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                                    (sc_image, sc_view, old)
                                }
                                Target::Texture(t) => {
                                    let tex = self.textures.get_mut(t.0);
                                    extent = vk::Extent2D { width: tex.desc.width, height: tex.desc.height };
                                    let old = tex.layout;
                                    tex.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                                    in_pass_targets.push(t.0);
                                    (tex.image, tex.view, old)
                                }
                            };
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { old };
                            // Also when the layout stays: a pass loading what the last one wrote.
                            self.barrier_image(cmd, image, color_range(1), old, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                            let load = if clear.is_some() {
                                vk::AttachmentLoadOp::CLEAR
                            } else if old == vk::ImageLayout::UNDEFINED {
                                vk::AttachmentLoadOp::DONT_CARE
                            } else {
                                vk::AttachmentLoadOp::LOAD
                            };
                            let format = match target {
                                Target::Surface => self.swapchain.as_ref().unwrap().format,
                                Target::Texture(t) => self.vk_format(self.textures.get(t.0).desc.format),
                            };
                            color_att = Some((view, format, load, clear.unwrap_or_default()));
                        }
                        // The motion image, a second color attachment (`Cmd::BeginPass::motion`),
                        // then the extra ones (`Cmd::PassExtras`).
                        let mut more_atts = Vec::new();
                        for (t, clear) in motion
                            .into_iter()
                            .chain(std::mem::take(&mut pass_extras).into_iter().flatten())
                        {
                            let tex = self.textures.get_mut(t.0);
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { tex.layout };
                            tex.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                            let (image, view, format) = (tex.image, tex.view, tex.desc.format);
                            in_pass_targets.push(t.0);
                            self.barrier_image(cmd, image, color_range(1), old, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                            let load = if clear.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };
                            more_atts.push((view, self.vk_format(format), load, clear.unwrap_or_default()));
                        }
                        // Depth back to an attachment if a screen effect read it since (`EndPass`).
                        if let Some((t, clear)) = depth {
                            let tex = self.textures.get_mut(t.0);
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { tex.layout };
                            tex.layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                            let image = tex.image;
                            if old != vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
                                self.barrier_image(cmd, image, depth_range(), old, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
                            }
                            in_pass_depth = Some(t.0);
                        }
                        let depth_att = depth.map(|(t, clear)| {
                            let tex = self.textures.get(t.0);
                            extent = vk::Extent2D { width: tex.desc.width, height: tex.desc.height };
                            let load = if clear.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };
                            (tex.view, load, clear.unwrap_or(0.0))
                        });
                        let colors: Vec<_> = color_att.into_iter().chain(more_atts).collect();
                        let pass = self.render_pass((colors.iter().map(|c| (c.1, c.2)).collect(), depth_att.map(|d| d.1)));
                        let views: Vec<_> = colors
                            .iter()
                            .map(|c| c.0)
                            .chain(depth_att.map(|d| d.0))
                            .collect();
                        let framebuffer = self
                            .device
                            .create_framebuffer(
                                &vk::FramebufferCreateInfo::default()
                                    .render_pass(pass)
                                    .attachments(&views)
                                    .width(extent.width)
                                    .height(extent.height)
                                    .layers(1),
                                None,
                            )
                            .expect("vkCreateFramebuffer");
                        self.frames[self.slot].framebuffers.push(framebuffer);
                        let mut clears = Vec::new();
                        for c in &colors {
                            clears.push(vk::ClearValue { color: vk::ClearColorValue { float32: c.3 } });
                        }
                        if let Some(d) = depth_att {
                            clears.push(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: d.2, stencil: 0 } });
                        }
                        self.device.cmd_begin_render_pass(
                            cmd,
                            &vk::RenderPassBeginInfo::default()
                                .render_pass(pass)
                                .framebuffer(framebuffer)
                                .render_area(vk::Rect2D { offset: Default::default(), extent })
                                .clear_values(&clears),
                            vk::SubpassContents::INLINE,
                        );
                        self.device.cmd_set_viewport(
                            cmd,
                            0,
                            &[vk::Viewport {
                                x: 0.0,
                                y: 0.0,
                                width: extent.width as f32,
                                height: extent.height as f32,
                                min_depth: 0.0,
                                max_depth: 1.0,
                            }],
                        );
                        self.device
                            .cmd_set_scissor(cmd, 0, &[vk::Rect2D { offset: Default::default(), extent }]);
                    }
                    Cmd::EndPass => {
                        self.device.cmd_end_render_pass(cmd);
                        // What the pass drew its depth into can be read now.
                        if let Some(t) = in_pass_depth.take() {
                            let image = self.textures.get(t).image;
                            self.barrier_image(
                                cmd,
                                image,
                                depth_range(),
                                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                            );
                            self.textures.get_mut(t).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                        }
                        for t in in_pass_targets.drain(..) {
                            let image = self.textures.get(t).image;
                            self.barrier_image(
                                cmd,
                                image,
                                color_range(1),
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                            );
                            self.textures.get_mut(t).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                        }
                    }
                    Cmd::SetPipeline(p) => {
                        let pl = self.pipelines.get(p.0);
                        self.device.cmd_bind_pipeline(cmd, pl.bind_point, pl.raw);
                        self.device
                            .cmd_bind_descriptor_sets(cmd, pl.bind_point, pl.layout, 0, &sets, &[]);
                        self.device
                            .cmd_bind_descriptor_sets(cmd, pl.bind_point, pl.layout, SAMPLER_SET, &[self.sampler_set], &[]);
                        current = Some(p.0);
                        stale = true;
                    }
                    Cmd::BindBuffer { binding, buffer, offset, size } => {
                        // A binding the shader never reads is not in its reflection (a shader
                        // pack's that skips a buffer): it is left out when the set is made.
                        let range = if size == u64::MAX { vk::WHOLE_SIZE } else { size };
                        let info = vk::DescriptorBufferInfo { buffer: self.buffers.get(buffer.0).raw, offset, range };
                        match bound.iter_mut().find(|b| b.0 == binding) {
                            Some(b) => b.1 = info,
                            None => bound.push((binding, info)),
                        }
                        stale = true;
                    }
                    Cmd::PushConstants { start, len } => {
                        pushed = Some((start, len));
                        let Some(p) = current else { continue };
                        let pl = self.pipelines.get(p);
                        let bytes = &list.data[start as usize..(start + len) as usize];
                        self.device
                            .cmd_push_constants(cmd, pl.layout, vk::ShaderStageFlags::ALL, 0, bytes);
                    }
                    Cmd::Scissor { x, y, w, h } => {
                        self.device.cmd_set_scissor(
                            cmd,
                            0,
                            &[vk::Rect2D { offset: vk::Offset2D { x, y }, extent: vk::Extent2D { width: w, height: h } }],
                        );
                    }
                    Cmd::Draw { vertices, instances, first_vertex, first_instance } => {
                        self.device
                            .cmd_draw(cmd, vertices, instances, first_vertex, first_instance);
                    }
                    Cmd::DrawIndirect { buffer, offset, count } => {
                        self.draw_indirect(cmd, self.buffers.get(buffer.0).raw, offset, count, false);
                    }
                    Cmd::IndexBuffer(buffer) => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_bind_index_buffer(cmd, b, 0, vk::IndexType::UINT32);
                    }
                    Cmd::DrawIndexedIndirect { buffer, offset, count } => {
                        self.draw_indirect(cmd, self.buffers.get(buffer.0).raw, offset, count, true);
                    }
                    Cmd::DrawIndexedIndirectCount { buffer, offset, count_buffer, count_offset, max }
                    | Cmd::DrawIndirectCount { buffer, offset, count_buffer, count_offset, max } => {
                        let indexed = matches!(cmd_item, Cmd::DrawIndexedIndirectCount { .. });
                        let b = self.buffers.get(buffer.0).raw;
                        let c = self.buffers.get(count_buffer.0).raw;
                        match &self.draw_count_fn {
                            Some(f) if indexed => {
                                f.cmd_draw_indexed_indirect_count(cmd, b, offset, c, count_offset, max, size_of::<DrawIndexedArgs>() as u32)
                            }
                            Some(f) => f.cmd_draw_indirect_count(cmd, b, offset, c, count_offset, max, size_of::<DrawArgs>() as u32),
                            None => {
                                let (b, _) = self.frames[self.slot].compacted.unwrap();
                                let at = compacted.pop().unwrap();
                                self.draw_indirect(cmd, self.buffers.get(b.0).raw, at, max, indexed);
                            }
                        }
                    }
                    Cmd::Dispatch { x, y, z } => self.device.cmd_dispatch(cmd, x, y, z),
                    Cmd::Barrier => self.memory_barrier(cmd),
                    Cmd::FillBuffer { buffer, offset, size, value } => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device.cmd_fill_buffer(cmd, b, offset, size, value);
                    }
                    Cmd::CopyDepthToBuffer { depth, buffer } => {
                        let t = self.textures.get(depth.0);
                        // From the layout it is in (`EndPass` leaves a sampled depth target
                        // shader-readable), and back to it: assuming the attachment layout was
                        // wrong, and left the effects sampling it in the attachment layout.
                        let (image, width, height, was) = (t.image, t.desc.width, t.desc.height, t.layout);
                        let raw = self.buffers.get(buffer.0).raw;
                        self.barrier_image(cmd, image, depth_range(), was, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
                        let region = vk::BufferImageCopy::default()
                            .image_subresource(vk::ImageSubresourceLayers {
                                aspect_mask: vk::ImageAspectFlags::DEPTH,
                                mip_level: 0,
                                base_array_layer: 0,
                                layer_count: 1,
                            })
                            .image_extent(vk::Extent3D { width, height, depth: 1 });
                        self.device
                            .cmd_copy_image_to_buffer(cmd, image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, raw, &[region]);
                        // An image never used yet goes back to being a depth attachment.
                        let back = if was == vk::ImageLayout::UNDEFINED { vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL } else { was };
                        self.barrier_image(cmd, image, depth_range(), vk::ImageLayout::TRANSFER_SRC_OPTIMAL, back);
                        self.textures.get_mut(depth.0).layout = back;
                    }
                    Cmd::Upscale { color, depth, motion, output, normals, albedo, specular, hit_distance, start } => {
                        let size = size_of::<UpscaleParams>();
                        let params: UpscaleParams = bytemuck::pod_read_unaligned(&list.data[start as usize..start as usize + size]);
                        self.record_upscale(cmd, [color, depth, motion, output], [normals, albedo, specular, hit_distance], &params);
                        // An upscaler leaves its own pipeline and bindings behind.
                        current = None;
                        bound.clear();
                        accels.clear();
                    }
                    // Batched above.
                    Cmd::BuildBlas(_) => {}
                    Cmd::BuildTlas(a) => self.rt_build_tlas(cmd, a.0),
                    Cmd::BindAccel { binding, accel } => {
                        let Some(raw) = self.rt_tlas_raw(accel.0) else { continue };
                        match accels.iter_mut().find(|a| a.0 == binding) {
                            Some(a) => a.1 = raw,
                            None => accels.push((binding, raw)),
                        }
                        stale = true;
                    }
                    Cmd::Mark => {
                        if marks < MAX_MARKS - 1 {
                            self.device
                                .cmd_write_timestamp(cmd, vk::PipelineStageFlags::BOTTOM_OF_PIPE, self.queries, first_query + marks);
                            marks += 1;
                        }
                    }
                }
            }

            if !blas_batch.is_empty() {
                self.rt_build_blases(cmd, &blas_batch);
            }
            let old = self.swapchain.as_ref().unwrap().layouts[image_index as usize];
            if self.capture_requested {
                self.capture_requested = false;
                let (w, h) = (sc_extent.width, sc_extent.height);
                let (buf, alloc) = raw_buffer(
                    &self.device,
                    &mut self.allocator,
                    self.spill,
                    (w * h * 4) as u64,
                    vk::BufferUsageFlags::TRANSFER_DST,
                    MemoryLocation::GpuToCpu,
                    "capture",
                );
                self.barrier_image(cmd, sc_image, color_range(1), old, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
                let region = [vk::BufferImageCopy::default()
                    .image_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image_extent(vk::Extent3D { width: w, height: h, depth: 1 })];
                self.device
                    .cmd_copy_image_to_buffer(cmd, sc_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, buf, &region);
                self.barrier_image(cmd, sc_image, color_range(1), vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR);
                let fmt = self.swapchain.as_ref().unwrap().format;
                self.capture = Some((buf, alloc, w, h, fmt));
            } else {
                self.barrier_image(cmd, sc_image, color_range(1), old, vk::ImageLayout::PRESENT_SRC_KHR);
            }
            self.swapchain.as_mut().unwrap().layouts[image_index as usize] = vk::ImageLayout::PRESENT_SRC_KHR;
            self.device
                .cmd_write_timestamp(cmd, vk::PipelineStageFlags::BOTTOM_OF_PIPE, self.queries, first_query + marks);
            self.frames[self.slot].marks = marks + 1;
            self.device.end_command_buffer(cmd).ok();

            let frame = &mut self.frames[self.slot];
            let rendered = self.swapchain.as_ref().unwrap().rendered[image_index as usize];
            let waits = [frame.acquired];
            // Only drawing into the swapchain image waits for it (its barriers from UNDEFINED or
            // PRESENT start at this stage): the frame's uploads, compute and offscreen passes go first.
            let stages =
                [if all_barriers() { vk::PipelineStageFlags::ALL_COMMANDS } else { vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT }];
            let signals = [rendered];
            let cmds = [cmd];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&waits)
                .wait_dst_stage_mask(&stages)
                .command_buffers(&cmds)
                .signal_semaphores(&signals);
            self.device.reset_fences(&[frame.fence]).ok();
            queued = std::time::Instant::now();
            if let Err(e) = self.device.queue_submit(self.queue, &[submit], frame.fence) {
                log::error!("vkQueueSubmit: {e:?}");
            }
            frame.waited = false;
            submitted = std::time::Instant::now();

            let swapchains = [self.swapchain.as_ref().unwrap().raw];
            let indices = [image_index];
            let wait = [rendered];
            let present = vk::PresentInfoKHR::default()
                .wait_semaphores(&wait)
                .swapchains(&swapchains)
                .image_indices(&indices);
            let result = self.swapchain_fn.queue_present(self.queue, &present);
            match result {
                Ok(suboptimal) if !suboptimal || self.turned => {}
                Ok(_) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.dirty_swapchain = true,
                Err(e) => log::error!("vkQueuePresentKHR: {e:?}"),
            }
            result.is_ok()
        };
        self.slot = (self.slot + 1) % FRAMES;
        self.save_pipeline_cache();
        self.log_submit(began, queued, submitted);
        presented
    }

    fn request_capture(&mut self) {
        self.capture_requested = true;
    }

    fn gpu_times(&self) -> &[f32] {
        &self.gpu_times
    }

    fn take_capture(&mut self) -> Option<Capture> {
        let (buf, alloc, w, h, fmt) = self.capture.take()?;
        unsafe { self.device.device_wait_idle().ok() };
        let mut rgba = alloc.mapped_slice().unwrap()[..(w * h * 4) as usize].to_vec();
        if matches!(fmt, vk::Format::B8G8R8A8_SRGB | vk::Format::B8G8R8A8_UNORM) {
            for px in rgba.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        self.defer(Deferred::Buffer(buf, alloc));
        Some(Capture { width: w, height: h, rgba })
    }

    fn wait_idle(&mut self) {
        unsafe { self.device.device_wait_idle().ok() };
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        for t in self.compile_threads.drain(..) {
            let _ = t.join();
        }
        unsafe {
            self.device.device_wait_idle().ok();
            // Streamline frees its own before the device goes.
            self.upscaler = None;
            for i in 0..self.frames.len() {
                let deferred = std::mem::take(&mut self.frames[i].deferred);
                for d in deferred {
                    self.destroy_now(d);
                }
                for (buffer, alloc, _) in std::mem::take(&mut self.frames[i].staging.chunks) {
                    self.device.destroy_buffer(buffer, None);
                    self.allocator.free(alloc).ok();
                }
                let f = &self.frames[i];
                for &p in &f.set_pools {
                    self.device.destroy_descriptor_pool(p, None);
                }
                for &fb in &f.framebuffers {
                    self.device.destroy_framebuffer(fb, None);
                }
                self.device.destroy_command_pool(f.pool, None);
                self.device.destroy_semaphore(f.acquired, None);
                self.device.destroy_fence(f.fence, None);
            }
            self.rt_drop();
            self.device.destroy_query_pool(self.queries, None);
            if let Some((buf, alloc, ..)) = self.capture.take() {
                self.device.destroy_buffer(buf, None);
                self.allocator.free(alloc).ok();
            }
            for i in 0..self.buffers.items.len() as u32 {
                if let Some(b) = self.buffers.remove(i) {
                    self.device.destroy_buffer(b.raw, None);
                    self.allocator.free(b.alloc).ok();
                }
            }
            for i in 0..self.textures.items.len() as u32 {
                if let Some(t) = self.textures.remove(i) {
                    self.device.destroy_image_view(t.view, None);
                    self.device.destroy_image(t.image, None);
                    self.allocator.free(t.alloc).ok();
                }
            }
            for i in 0..self.pipelines.items.len() as u32 {
                if let Some(p) = self.pipelines.remove(i) {
                    self.device.destroy_pipeline(p.raw, None);
                    self.device.destroy_pipeline_layout(p.layout, None);
                    self.device
                        .destroy_descriptor_set_layout(p.set_layout, None);
                }
            }
            for s in self.samplers {
                self.device.destroy_sampler(s, None);
            }
            for (_, p) in self.passes.drain() {
                self.device.destroy_render_pass(p, None);
            }
            self.save_pipeline_cache();
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None);
            self.device.destroy_descriptor_pool(self.global_pool, None);
            self.device.destroy_descriptor_pool(self.sampler_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.sampler_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.global_layout, None);
            if let Some(sc) = self.swapchain.take() {
                self.destroy_swapchain(sc);
            }
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
            self.surface_fn.destroy_surface(self.surface, None);
            if let Some((f, m)) = self.debug.take() {
                f.destroy_debug_utils_messenger(m, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

/// `WEFT_EARLY_WAIT=1`: a write before the frame slot is ready waits for the GPU there and then,
/// as before (to compare with keeping it on the CPU until the slot is waited on).
fn early_wait() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WEFT_EARLY_WAIT").is_some())
}

/// The stages and accesses an image barrier waits on (`src`, for the layout it leaves) or makes
/// wait (for the layout it enters), from how an image in that layout is used here. They were all
/// `ALL_COMMANDS` both ways: a full drain at each of a frame's 40-50 pass edges on desktop GPUs,
/// and on MoltenVK every pass's vertex stage waiting on the last one's fragments (perf review).
/// `WEFT_ALL_BARRIERS=1` puts every barrier back to `ALL_COMMANDS`, to compare.
fn layout_stages(layout: vk::ImageLayout, src: bool) -> (vk::PipelineStageFlags, vk::AccessFlags) {
    use vk::{AccessFlags as A, ImageLayout as L, PipelineStageFlags as S};
    let shaders = S::VERTEX_SHADER | S::FRAGMENT_SHADER | S::COMPUTE_SHADER;
    if all_barriers() {
        return if src { (S::ALL_COMMANDS, A::MEMORY_WRITE) } else { (S::ALL_COMMANDS, A::MEMORY_READ | A::MEMORY_WRITE) };
    }
    match (layout, src) {
        // Contents dropped: only the work that last used the image (for writing it again after
        // it was read) and the swapchain's acquire, which waits at colour output.
        (L::UNDEFINED, true) => (
            shaders | S::COLOR_ATTACHMENT_OUTPUT | S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS | S::TRANSFER,
            A::empty(),
        ),
        (L::COLOR_ATTACHMENT_OPTIMAL, true) => (S::COLOR_ATTACHMENT_OUTPUT, A::COLOR_ATTACHMENT_WRITE),
        (L::COLOR_ATTACHMENT_OPTIMAL, false) => (S::COLOR_ATTACHMENT_OUTPUT, A::COLOR_ATTACHMENT_READ | A::COLOR_ATTACHMENT_WRITE),
        (L::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, true) => (S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS, A::DEPTH_STENCIL_ATTACHMENT_WRITE),
        (L::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, false) => {
            (S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS, A::DEPTH_STENCIL_ATTACHMENT_READ | A::DEPTH_STENCIL_ATTACHMENT_WRITE)
        }
        // Sampled: reads only, so what follows just waits for them to finish (no access to flush).
        (L::SHADER_READ_ONLY_OPTIMAL, true) => (shaders, A::empty()),
        (L::SHADER_READ_ONLY_OPTIMAL, false) => (shaders, A::SHADER_READ),
        (L::TRANSFER_DST_OPTIMAL, true) => (S::TRANSFER, A::TRANSFER_WRITE),
        (L::TRANSFER_DST_OPTIMAL, false) => (S::TRANSFER, A::TRANSFER_WRITE),
        (L::TRANSFER_SRC_OPTIMAL, true) => (S::TRANSFER, A::empty()),
        (L::TRANSFER_SRC_OPTIMAL, false) => (S::TRANSFER, A::TRANSFER_READ),
        // From presenting: after the acquire, which waits at colour output. To presenting: the
        // semaphore that present waits on covers it (nothing waits on the barrier itself).
        (L::PRESENT_SRC_KHR, true) => (S::COLOR_ATTACHMENT_OUTPUT, A::empty()),
        (L::PRESENT_SRC_KHR, false) => (S::BOTTOM_OF_PIPE, A::empty()),
        (_, true) => (S::ALL_COMMANDS, A::MEMORY_WRITE),
        (_, false) => (S::ALL_COMMANDS, A::MEMORY_READ | A::MEMORY_WRITE),
    }
}

/// `WEFT_ALL_BARRIERS=1`: image barriers and the acquire wait at `ALL_COMMANDS`, as before.
fn all_barriers() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WEFT_ALL_BARRIERS").is_some())
}

#[cfg(test)]
mod tests {
    /// Every sampled image of a texture picked per pixel ends up marked, whatever naga does.
    #[test]
    fn sampled_images_of_varying_textures_are_non_uniform() {
        let shader = weft_render::Shader::from_wgsl(
            "test",
            "@fragment fn fs(@location(0) @interpolate(flat) t: u32, @location(1) uv: vec2<f32>) -> @location(0) vec4<f32> {
                return textureSampleLevel(textures[t], nearest_clamp, uv, 0.0);
            }",
        )
        .unwrap();
        let options = naga::back::spv::PipelineOptions { shader_stage: naga::ShaderStage::Fragment, entry_point: "fs".into() };
        let mut words = naga::back::spv::write_vec(&shader.module, &shader.info, &Default::default(), Some(&options)).unwrap();
        super::non_uniform_samples(&mut words);
        let (mut marked, mut sampled) = (Vec::new(), Vec::new());
        let mut at = 5;
        while at < words.len() {
            let count = (words[at] >> 16) as usize;
            match words[at] & 0xffff {
                71 if words[at + 2] == 5300 => marked.push(words[at + 1]),
                86 => sampled.push(words[at + 2]),
                _ => {}
            }
            at += count;
        }
        assert!(!sampled.is_empty());
        assert!(sampled.iter().all(|s| marked.contains(s)), "sampled {sampled:?}, marked {marked:?}");
    }

    use ash::vk;

    fn memory(heaps: &[vk::MemoryHeapFlags], types: &[(vk::MemoryPropertyFlags, u32)]) -> vk::PhysicalDeviceMemoryProperties {
        let mut mem = vk::PhysicalDeviceMemoryProperties {
            memory_heap_count: heaps.len() as u32,
            memory_type_count: types.len() as u32,
            ..Default::default()
        };
        for (i, &flags) in heaps.iter().enumerate() {
            mem.memory_heaps[i] = vk::MemoryHeap { size: 1 << 30, flags };
        }
        for (i, &(property_flags, heap_index)) in types.iter().enumerate() {
            mem.memory_types[i] = vk::MemoryType { property_flags, heap_index };
        }
        mem
    }

    #[test]
    fn spill_goes_to_system_memory_only() {
        use vk::MemoryPropertyFlags as P;
        let host = P::HOST_VISIBLE | P::HOST_COHERENT;
        // A discrete GPU (NVIDIA's layout): its own heap, system memory, and the small BAR heap
        // that is device local too.
        let discrete = memory(
            &[vk::MemoryHeapFlags::DEVICE_LOCAL, vk::MemoryHeapFlags::empty(), vk::MemoryHeapFlags::DEVICE_LOCAL],
            &[(P::empty(), 1), (P::DEVICE_LOCAL, 0), (host, 1), (host | P::HOST_CACHED, 1), (P::DEVICE_LOCAL | host, 2)],
        );
        assert_eq!(super::spill_types(&discrete), 0b01101);
        // Shared memory (Apple): every heap is the GPU's, so nothing to spill to.
        let unified = memory(&[vk::MemoryHeapFlags::DEVICE_LOCAL], &[(P::DEVICE_LOCAL, 0), (P::DEVICE_LOCAL | host, 0)]);
        assert_eq!(super::spill_types(&unified), 0);
    }
}

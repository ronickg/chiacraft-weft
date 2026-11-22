//! Vulkan 1.3 backend: dynamic rendering, synchronization2, bindless sampled images,
//! push descriptors for per-draw buffers.

use std::ffi::{CString, c_void};
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
/// Timestamp queries per frame: start, after uploads, marks, end.
const MAX_MARKS: u32 = 16;
const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

struct Slab<T> {
    items: Vec<Option<T>>,
    free: Vec<u32>,
}

impl<T> Slab<T> {
    fn new() -> Self {
        Slab { items: Vec::new(), free: Vec::new() }
    }
    fn insert(&mut self, v: T) -> u32 {
        if let Some(i) = self.free.pop() {
            self.items[i as usize] = Some(v);
            i
        } else {
            self.items.push(Some(v));
            self.items.len() as u32 - 1
        }
    }
    fn remove(&mut self, i: u32) -> Option<T> {
        let v = self.items.get_mut(i as usize)?.take();
        if v.is_some() {
            self.free.push(i);
        }
        v
    }
    fn get(&self, i: u32) -> &T {
        self.items[i as usize]
            .as_ref()
            .expect("stale render handle")
    }
    fn get_mut(&mut self, i: u32) -> &mut T {
        self.items[i as usize]
            .as_mut()
            .expect("stale render handle")
    }
}

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
    /// Without push descriptors: the pools this frame's buffer sets come from, reset when the
    /// slot is ready again, and the one in use.
    set_pools: Vec<vk::DescriptorPool>,
    set_pool: usize,
    waited: bool,
    /// Timestamps written by the frame now in flight in this slot.
    marks: u32,
}

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
    /// The images' own size (turned: the display's natural orientation).
    extent: vk::Extent2D,
    /// The size the game draws at (`FrameInfo`): `extent`, or it turned a quarter.
    logical: vk::Extent2D,
    transform: vk::SurfaceTransformFlagsKHR,
    /// Pre-rotation (Android, a display turned a quarter): the image the game's passes for the
    /// screen draw into, at `logical`, and the quarter turns (1 or 3) the last pass gives it on its
    /// way into the swapchain image (`TURN_WGSL`), so the display shows it as it is.
    turn: Option<(TextureId, u32)>,
    mode: vk::PresentModeKHR,
}

/// Pre-rotation's last pass: the game's picture, drawn upright into its own image, copied texel
/// for texel into the swapchain image turned a quarter (`Swapchain::turn`). The instance is the
/// picture's bindless index; vertices 0-2 turn it for `ROTATE_90`, 3-5 for `ROTATE_270`.
const TURN_WGSL: &str = r#"
struct TurnOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) @interpolate(flat) info: vec2<u32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> TurnOut {
    let k = vi % 3u;
    let p = vec2<f32>(f32((k << 1u) & 2u), f32(k & 2u));
    var out: TurnOut;
    out.clip = vec4(p * 2.0 - 1.0, 0.0, 1.0);
    out.info = vec2(ii, vi / 3u);
    return out;
}

@fragment
fn fs(in: TurnOut) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(textures[in.info.x]));
    let p = vec2<i32>(in.clip.xy);
    var at = vec2(p.y, size.y - 1 - p.x);
    if in.info.y != 0u {
        at = vec2(size.x - 1 - p.y, p.x);
    }
    return textureLoad(textures[in.info.x], at, 0);
}
"#;

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
    /// None where the device has no `VK_KHR_push_descriptor` (the Android emulator): per-draw
    /// buffers then go through descriptor sets from each frame's pools (`bind_buffer_set`).
    push_fn: Option<khr::push_descriptor::Device>,
    /// Android with `VK_EXT_image_compression_control`: render targets are made uncompressed
    /// (`create_texture`). See `compression_off`.
    no_compression: bool,
    /// `Backend::hand_own_pass`: on Imagination's driver (`WEFT_HAND_PASS=0/1` to compare).
    hand_own_pass: bool,
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
    samplers: [vk::Sampler; 4],
    free_tex: Vec<u32>,
    next_tex: u32,
    capture_requested: bool,
    capture: Option<(vk::Buffer, Allocation, u32, u32, vk::Format)>,
    /// Pre-rotation's pipeline (`TURN_WGSL`), made with the first turned swapchain; and whether it
    /// failed, so the swapchain is then left to the compositor to turn (IDENTITY).
    turn_pipe: Option<PipelineId>,
    turn_failed: bool,
    device_name: String,
    /// `Backend::device_class` and `device_memory_mb`.
    device_class: u32,
    device_memory_mb: u64,
    /// The device takes indirect draws' counts from a buffer (`drawIndirectCount`). MoltenVK
    /// doesn't: then a count draw draws all its records (`Backend::indirect_count`).
    indirect_count: bool,
    queries: vk::QueryPool,
    /// Pipelines compiled before, kept on disk between starts (`load_pipeline_cache`).
    pipeline_cache: vk::PipelineCache,
    ts_period: f32,
    gpu_times: Vec<f32>,
    /// Where `begin_frame`'s and `end_frame`'s time goes (waiting for the GPU, `vkAcquireNextImageKHR`,
    /// recording, `vkQueueSubmit2`, `vkQueuePresentKHR`), logged each second (`log_submit`):
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

            let mut exts: Vec<*const std::ffi::c_char> = ash_window::enumerate_required_extensions(display)
                .map_err(err("surface extensions"))?
                .to_vec();
            let mut layers: Vec<*const std::ffi::c_char> = Vec::new();
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
            let app = vk::ApplicationInfo::default()
                .application_name(c"Weft")
                .engine_name(c"Weft")
                .api_version(vk::API_VERSION_1_3);
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
                if props.api_version < vk::API_VERSION_1_3 {
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
            let (pdev, queue_family, _) = best.ok_or_else(|| Error("no Vulkan 1.3 GPU with presentation".into()))?;
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
            let mut indexing = vk::PhysicalDeviceDescriptorIndexingProperties::default();
            let mut driver = vk::PhysicalDeviceDriverProperties::default();
            let mut props2 = vk::PhysicalDeviceProperties2::default()
                .push_next(&mut indexing)
                .push_next(&mut driver);
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

            let mut has11 = vk::PhysicalDeviceVulkan11Features::default();
            let mut has12 = vk::PhysicalDeviceVulkan12Features::default();
            let mut has13 = vk::PhysicalDeviceVulkan13Features::default();
            let mut has = vk::PhysicalDeviceFeatures2::default()
                .push_next(&mut has11)
                .push_next(&mut has12)
                .push_next(&mut has13);
            instance.get_physical_device_features2(pdev, &mut has);
            let base = has.features;
            let has_bda = has12.buffer_device_address != vk::FALSE;
            let missing: Vec<&str> = [
                ("multiDrawIndirect", base.multi_draw_indirect),
                ("drawIndirectFirstInstance", base.draw_indirect_first_instance),
                ("samplerAnisotropy", base.sampler_anisotropy),
                ("fillModeNonSolid", base.fill_mode_non_solid),
                ("shaderInt16", base.shader_int16),
                ("storageBuffer16BitAccess", has11.storage_buffer16_bit_access),
                ("descriptorIndexing", has12.descriptor_indexing),
                ("runtimeDescriptorArray", has12.runtime_descriptor_array),
                ("descriptorBindingPartiallyBound", has12.descriptor_binding_partially_bound),
                ("descriptorBindingSampledImageUpdateAfterBind", has12.descriptor_binding_sampled_image_update_after_bind),
                ("shaderSampledImageArrayNonUniformIndexing", has12.shader_sampled_image_array_non_uniform_indexing),
                ("scalarBlockLayout", has12.scalar_block_layout),
                ("dynamicRendering", has13.dynamic_rendering),
                ("synchronization2", has13.synchronization2),
            ]
            .into_iter()
            .filter(|f| f.1 == vk::FALSE)
            .map(|f| f.0)
            .collect();
            if !missing.is_empty() {
                return Err(Error(format!("{device_name} lacks what the game needs: {}", missing.join(", "))));
            }
            // Optional: without it the count draws draw all their records (MoltenVK has none).
            // `WEFT_NO_INDIRECT_COUNT=1` does without it on any GPU, to check that path.
            let imagination = driver.driver_id == vk::DriverId::IMAGINATION_PROPRIETARY;
            let indirect_count = has12.draw_indirect_count != vk::FALSE && std::env::var_os("WEFT_NO_INDIRECT_COUNT").is_none();
            if !indirect_count {
                log::info!("{device_name}: no drawIndirectCount, indirect count draws draw every record");
            }

            let features = vk::PhysicalDeviceFeatures::default()
                .multi_draw_indirect(true)
                .draw_indirect_first_instance(true)
                .sampler_anisotropy(true)
                .fill_mode_non_solid(true)
                .shader_int16(true);
            let mut f11 = vk::PhysicalDeviceVulkan11Features::default().storage_buffer16_bit_access(true);
            let mut f12 = vk::PhysicalDeviceVulkan12Features::default()
                .descriptor_indexing(true)
                .runtime_descriptor_array(true)
                .descriptor_binding_partially_bound(true)
                .descriptor_binding_sampled_image_update_after_bind(true)
                .shader_sampled_image_array_non_uniform_indexing(true)
                // Metal has no such draw: `indirect_count`.
                .draw_indirect_count(indirect_count)
                .scalar_block_layout(true);
            let mut f13 = vk::PhysicalDeviceVulkan13Features::default()
                .dynamic_rendering(true)
                .synchronization2(true);
            let has_ext = |name: &std::ffi::CStr| {
                instance
                    .enumerate_device_extension_properties(pdev)
                    .unwrap_or_default()
                    .iter()
                    .any(|e| e.extension_name_as_c_str() == Ok(name))
            };
            let mut dev_exts = vec![khr::swapchain::NAME.as_ptr()];
            // Optional: the Android emulator's Vulkan has none (descriptor sets instead).
            let push = has_ext(khr::push_descriptor::NAME);
            if push {
                dev_exts.push(khr::push_descriptor::NAME.as_ptr());
            } else {
                log::info!("{device_name}: no VK_KHR_push_descriptor, per-draw buffers through descriptor sets");
            }
            // Android, `WEFT_NO_COMPRESSION=1`: render targets without the GPU's framebuffer
            // compression (`compression_off`). Off by default since 11-06: the garbage tiles that
            // made it needed (11-05) no longer show (scripted shots of sky, water and the edge with
            // it on), and it saves the Pixel 10 about 1 ms a frame at full resolution (vista 68.2 ->
            // 67.2 ms, edge 73.5 -> 72.7, forest 61.7 -> 61.2; 0.2 ms at half).
            let mut compression = vk::PhysicalDeviceImageCompressionControlFeaturesEXT::default();
            let no_compression = cfg!(target_os = "android")
                && std::env::var_os("WEFT_NO_COMPRESSION").is_some()
                && has_ext(ext::image_compression_control::NAME)
                && {
                    let mut f = vk::PhysicalDeviceFeatures2::default().push_next(&mut compression);
                    instance.get_physical_device_features2(pdev, &mut f);
                    compression.image_compression_control != vk::FALSE
                };
            if no_compression {
                dev_exts.push(ext::image_compression_control::NAME.as_ptr());
                log::info!("{device_name}: render targets without framebuffer compression");
            }
            let mut compression = vk::PhysicalDeviceImageCompressionControlFeaturesEXT::default().image_compression_control(no_compression);
            // Ray queries (`rt.rs`): with push descriptors (the TLAS is pushed like the buffers),
            // device addresses and both extensions, on the platforms `rt::wanted` allows.
            let mut has_accel = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
            let mut has_query = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
            let ray_query = rt::wanted()
                && push
                && has_bda
                && has_ext(khr::acceleration_structure::NAME)
                && has_ext(khr::ray_query::NAME)
                && has_ext(khr::deferred_host_operations::NAME)
                && {
                    let mut f = vk::PhysicalDeviceFeatures2::default()
                        .push_next(&mut has_accel)
                        .push_next(&mut has_query);
                    instance.get_physical_device_features2(pdev, &mut f);
                    has_accel.acceleration_structure != vk::FALSE && has_query.ray_query != vk::FALSE
                };
            let mut accel_features = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default().acceleration_structure(true);
            let mut query_features = vk::PhysicalDeviceRayQueryFeaturesKHR::default().ray_query(true);
            if ray_query {
                dev_exts.push(khr::acceleration_structure::NAME.as_ptr());
                dev_exts.push(khr::ray_query::NAME.as_ptr());
                dev_exts.push(khr::deferred_host_operations::NAME.as_ptr());
                f12 = f12.buffer_device_address(true);
                log::info!("{device_name}: ray queries");
            }
            let mut f2 = vk::PhysicalDeviceFeatures2::default()
                .features(features)
                .push_next(&mut f11)
                .push_next(&mut f12)
                .push_next(&mut f13);
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
                .push_next(&mut f2)
                .push_next(&mut compression);
            if ray_query {
                device_info = device_info
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
            let push_fn = push.then(|| khr::push_descriptor::Device::new(&instance, &device));

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
                    waited: false,
                    marks: 0,
                });
            }

            let flags = vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
            let binding_flags = [flags, vk::DescriptorBindingFlags::empty(), Default::default(), Default::default(), Default::default()];
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(MAX_TEXTURES)
                    .stage_flags(vk::ShaderStageFlags::ALL),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(2)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(3)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(4)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL),
            ];
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
            let sizes = [
                vk::DescriptorPoolSize { ty: vk::DescriptorType::SAMPLED_IMAGE, descriptor_count: MAX_TEXTURES },
                vk::DescriptorPoolSize { ty: vk::DescriptorType::SAMPLER, descriptor_count: 4 },
            ];
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
            let infos: Vec<[vk::DescriptorImageInfo; 1]> = samplers
                .iter()
                .map(|&s| [vk::DescriptorImageInfo::default().sampler(s)])
                .collect();
            let writes: Vec<_> = infos
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    vk::WriteDescriptorSet::default()
                        .dst_set(global_set)
                        .dst_binding(1 + i as u32)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .image_info(info)
                })
                .collect();
            device.update_descriptor_sets(&writes, &[]);

            let queries = device
                .create_query_pool(
                    &vk::QueryPoolCreateInfo::default()
                        .query_type(vk::QueryType::TIMESTAMP)
                        .query_count(MAX_MARKS * FRAMES as u32),
                    None,
                )
                .map_err(err("query pool"))?;
            let pipeline_cache = device
                .create_pipeline_cache(&vk::PipelineCacheCreateInfo::default(), None)
                .map_err(err("pipeline cache"))?;
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
                push_fn,
                no_compression,
                hand_own_pass: match std::env::var("WEFT_HAND_PASS").as_deref() {
                    Ok("0") => false,
                    Ok(_) => true,
                    Err(_) => imagination,
                },
                allocator: ManuallyDrop::new(allocator),
                spill,
                swapchain: None,
                frames,
                slot: 0,
                image_index: None,
                want_size: (desc.width, desc.height),
                vsync: desc.vsync,
                dirty_swapchain: true,
                buffers: Slab::new(),
                textures: Slab::new(),
                pipelines: Slab::new(),
                uploads: Vec::new(),
                early: Vec::new(),
                global_layout,
                global_pool,
                global_set,
                samplers,
                free_tex: Vec::new(),
                next_tex: 0,
                capture_requested: false,
                turn_pipe: None,
                turn_failed: false,
                capture: None,
                device_name,
                device_class,
                device_memory_mb,
                indirect_count,
                queries,
                pipeline_cache,
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
            Ok(backend)
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Adds a frame to `submit_split`; once a second, with `--log frame=debug`, a line of where
    /// the frame's GPU calls spent it: in `begin_frame` (the app's "acquire") waiting for this
    /// slot's frame before last to finish on the GPU (long when the GPU is the bottleneck: the
    /// 3456 x 2234 rows, 11-04) and acquiring an image; in `end_frame` (the app's "submit")
    /// recording our commands, `vkQueueSubmit2` (on MoltenVK the Metal encoding, and waiting for a
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
            let mut count = caps.min_image_count + 1;
            if caps.max_image_count > 0 {
                count = count.min(caps.max_image_count);
            }
            // Android, a display turned a quarter (a phone held sideways): the swapchain takes the
            // display's own turn and its natural size, the game draws upright into an image of its
            // own, and the last pass turns that into the swapchain image (`Swapchain::turn`), so the
            // display shows it as it is. Left to the compositor (IDENTITY), Android turned every
            // frame on the GPU itself (SurfaceFlinger's client composition: 1.95 ms a frame on the
            // Pixel 10, Android port, 11-06). `WEFT_NO_PRETURN=1` leaves it to the compositor.
            // Otherwise the game draws upright and the compositor turns it. Android also often has
            // only INHERIT for alpha.
            let quarter = if cfg!(target_os = "android") && !self.turn_failed && std::env::var_os("WEFT_NO_PRETURN").is_none() {
                [(vk::SurfaceTransformFlagsKHR::ROTATE_90, 1), (vk::SurfaceTransformFlagsKHR::ROTATE_270, 3)]
                    .into_iter()
                    .find(|(t, _)| caps.current_transform == *t && caps.supported_transforms.contains(*t))
                    .map(|(_, q)| q)
            } else {
                None
            };
            let transform = if quarter.is_some() {
                caps.current_transform
            } else if cfg!(target_os = "android")
                && caps
                    .supported_transforms
                    .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                caps.current_transform
            };
            // The game's size stays the surface's as it is held; the images are the display's own.
            let logical = extent;
            let extent = if quarter.is_some() { vk::Extent2D { width: extent.height, height: extent.width } } else { extent };
            let alpha = [vk::CompositeAlphaFlagsKHR::OPAQUE, vk::CompositeAlphaFlagsKHR::INHERIT]
                .into_iter()
                .find(|a| caps.supported_composite_alpha.contains(*a))
                .unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE);
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
            // Destroyed before the new one instead of handed over as `old_swapchain`: nothing is in flight after the wait above.
            let first = self.swapchain.is_none();
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
            if self.submit_split.mode != mode.as_raw() || first {
                log::info!(
                    "swapchain: {}x{}, {count} images, present mode {mode:?} (vsync {}){}",
                    logical.width,
                    logical.height,
                    self.vsync,
                    if quarter.is_some() { ", turned by the game (pre-rotation)" } else { "" }
                );
            }
            self.submit_split.mode = mode.as_raw();
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
            log::debug!("swapchain {}x{} {:?} {:?}", extent.width, extent.height, format.format, mode);
            self.swapchain = Some(Swapchain {
                raw,
                layouts: vec![vk::ImageLayout::UNDEFINED; images.len()],
                images,
                views,
                rendered,
                format: format.format,
                extent,
                logical,
                transform,
                turn: None,
                mode,
            });
            self.dirty_swapchain = false;
            if let Some(q) = quarter {
                if self.turn_pipe.is_none() {
                    let made = Shader::from_wgsl("turn", TURN_WGSL)
                        .map_err(|e| Error(e.0))
                        .and_then(|shader| {
                            self.create_pipeline(&PipelineDesc {
                                label: "pre-rotation",
                                shader: &shader,
                                vertex: "vs",
                                fragment: Some("fs"),
                                color: Some(Format::Surface),
                                depth: None,
                                blend: Blend::None,
                                cull_back: false,
                                topology: Topology::Triangles,
                                motion: false,
                            })
                        });
                    match made {
                        Ok(pipe) => self.turn_pipe = Some(pipe),
                        Err(e) => {
                            // Made again the plain way, the compositor turning it.
                            log::error!("pre-rotation: {}", e.0);
                            self.turn_failed = true;
                            self.dirty_swapchain = true;
                            return Ok(());
                        }
                    }
                }
                let picture = self.create_texture(
                    &TextureDesc { width: logical.width, height: logical.height, mips: 1, format: Format::Surface, render_target: true },
                    "turned screen",
                );
                if let Some(sc) = self.swapchain.as_mut() {
                    sc.turn = Some((picture, q));
                }
            }
            Ok(())
        }
    }

    /// The swapchain still works but no longer fits the surface exactly: made again. On Android a
    /// phone held sideways says so on every present, as the game leaves the turning to the
    /// compositor (IDENTITY, `recreate_swapchain`): there only a new size counts. Made again each
    /// time, the Pixel 10 rebuilt its swapchain (and waited for the GPU) every few frames.
    fn suboptimal(&mut self) {
        if cfg!(target_os = "android") {
            let caps = unsafe {
                self.surface_fn
                    .get_physical_device_surface_capabilities(self.pdev, self.surface)
            };
            if let (Ok(caps), Some(sc)) = (caps, self.swapchain.as_ref())
                && caps.current_extent == sc.logical
                && (sc.turn.is_none() || caps.current_transform == sc.transform)
            {
                return;
            }
        }
        self.dirty_swapchain = true;
    }

    unsafe fn destroy_swapchain(&mut self, sc: Swapchain) {
        if let Some((picture, _)) = sc.turn {
            self.destroy_texture(picture);
        }
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

    /// Without push descriptors: a set for `set_layout` from this frame's pools with every buffer
    /// in `bound` the pipeline reads (`wanted`), bound as set 1.
    unsafe fn bind_buffer_set(
        &mut self,
        cmd: vk::CommandBuffer,
        layout: vk::PipelineLayout,
        set_layout: vk::DescriptorSetLayout,
        bind_point: vk::PipelineBindPoint,
        wanted: &[(u32, vk::DescriptorType)],
        bound: &[(u32, vk::DescriptorType, vk::DescriptorBufferInfo)],
    ) {
        unsafe {
            let layouts = [set_layout];
            let set = loop {
                let frame = &mut self.frames[self.slot];
                let fresh = frame.set_pool == frame.set_pools.len();
                if fresh {
                    let sizes = [
                        vk::DescriptorPoolSize { ty: vk::DescriptorType::STORAGE_BUFFER, descriptor_count: 8192 },
                        vk::DescriptorPoolSize { ty: vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 2048 },
                    ];
                    match self.device.create_descriptor_pool(
                        &vk::DescriptorPoolCreateInfo::default()
                            .max_sets(2048)
                            .pool_sizes(&sizes),
                        None,
                    ) {
                        Ok(p) => frame.set_pools.push(p),
                        Err(e) => {
                            log::error!("descriptor pool: {e:?}");
                            return;
                        }
                    }
                }
                let pool = frame.set_pools[frame.set_pool];
                match self.device.allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                ) {
                    Ok(sets) => break sets[0],
                    Err(e) if fresh => {
                        log::error!("descriptor set: {e:?}");
                        return;
                    }
                    // Full: the next pool (made when there is none).
                    Err(_) => frame.set_pool += 1,
                }
            };
            let infos: Vec<[vk::DescriptorBufferInfo; 1]> = wanted
                .iter()
                .filter_map(|&(b, ty)| bound.iter().find(|x| x.0 == b && x.1 == ty).map(|x| [x.2]))
                .collect();
            let writes: Vec<vk::WriteDescriptorSet> = wanted
                .iter()
                .filter(|&&(b, ty)| bound.iter().any(|x| x.0 == b && x.1 == ty))
                .zip(&infos)
                .map(|(&(b, ty), info)| {
                    vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(b)
                        .descriptor_type(ty)
                        .buffer_info(info)
                })
                .collect();
            self.device.update_descriptor_sets(&writes, &[]);
            self.device
                .cmd_bind_descriptor_sets(cmd, bind_point, layout, 1, &[set], &[]);
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
        for &pool in &frame.set_pools {
            unsafe {
                self.device
                    .reset_descriptor_pool(pool, vk::DescriptorPoolResetFlags::empty())
                    .ok();
            }
        }
        frame.set_pool = 0;
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
        let deferred = std::mem::take(&mut frame.deferred);
        for d in deferred {
            self.destroy_now(d);
        }
        self.rt_slot_ready();
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
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.allocator.free(a).ok();
                    if index != u32::MAX {
                        self.free_tex.push(index);
                    }
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
        let b = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src_stage)
            .src_access_mask(src_access)
            .dst_stage_mask(dst_stage)
            .dst_access_mask(dst_access)
            .old_layout(old)
            .new_layout(new)
            .image(image)
            .subresource_range(range)];
        unsafe {
            self.device
                .cmd_pipeline_barrier2(cmd, &vk::DependencyInfo::default().image_memory_barriers(&b))
        };
    }

    unsafe fn memory_barrier(&self, cmd: vk::CommandBuffer) {
        let b = [vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS | vk::PipelineStageFlags2::HOST)
            .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::HOST_READ)];
        unsafe {
            self.device
                .cmd_pipeline_barrier2(cmd, &vk::DependencyInfo::default().memory_barriers(&b))
        };
    }

    /// Orders copies: those recorded after it start once those before it are written.
    unsafe fn copy_barrier(&self, cmd: vk::CommandBuffer) {
        let b = [vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::COPY)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::COPY)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)];
        unsafe {
            self.device
                .cmd_pipeline_barrier2(cmd, &vk::DependencyInfo::default().memory_barriers(&b))
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
                            (depth_range(), vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                        } else {
                            (color_range(t.desc.mips), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        };
                        let image = t.image;
                        // Cleared to zeros, not left as the memory was: a desktop driver's new
                        // memory happened to be zeros, a phone's (the Pixel 10's PowerVR) held
                        // leftovers and NaNs, which screen effects reading a target before it was
                        // first written (histories, half-size targets) showed as black blocks
                        // with coloured specks.
                        self.barrier_image(cmd, image, range, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
                        if depth {
                            self.device.cmd_clear_depth_stencil_image(
                                cmd,
                                image,
                                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                                &vk::ClearDepthStencilValue { depth: 0.0, stencil: 0 },
                                &[range],
                            );
                        } else {
                            self.device.cmd_clear_color_image(
                                cmd,
                                image,
                                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                                &vk::ClearColorValue { float32: [0.0; 4] },
                                &[range],
                            );
                        }
                        self.barrier_image(cmd, image, range, vk::ImageLayout::TRANSFER_DST_OPTIMAL, new);
                        self.textures.get_mut(id).layout = new;
                    }
                }
            }
            self.memory_barrier(cmd);
        }
    }
}

/// **(Weft bug, naga 30)** naga marks a texture loaded from the bindless array with a
/// non-uniform index `NonUniform`, but not the `OpSampledImage` made from it, which is the operand
/// the sample reads and which Vulkan requires marked (VUID-RuntimeSpirv-NonUniform-06274). NVIDIA
/// samples right without it; the Pixel 10's PowerVR sampled one texture for a whole group of
/// pixels where two textures met (black blocks with coloured specks at plants' edges and water).
/// So each `OpSampledImage` of a `NonUniform` image gets the decoration too.
fn non_uniform_sampled_images(words: Vec<u32>) -> Vec<u32> {
    const OP_DECORATE: u32 = 71;
    const OP_SAMPLED_IMAGE: u32 = 86;
    const NON_UNIFORM: u32 = 5300;
    if words.len() < 5 {
        return words;
    }
    // Instructions after the five-word header: (where it starts, opcode, length).
    let mut at = 5;
    let mut ops = Vec::new();
    while at < words.len() {
        let (len, op) = ((words[at] >> 16) as usize, words[at] & 0xffff);
        if len == 0 || at + len > words.len() {
            return words;
        }
        ops.push((at, op, len));
        at += len;
    }
    let mut marked: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut last_decorate = None;
    for &(at, op, len) in &ops {
        if op == OP_DECORATE {
            last_decorate = Some(at + len);
            if len >= 3 && words[at + 2] == NON_UNIFORM {
                marked.insert(words[at + 1]);
            }
        }
    }
    let add: Vec<u32> = ops
        .iter()
        .filter(|&&(at, op, len)| op == OP_SAMPLED_IMAGE && len >= 5 && marked.contains(&words[at + 3]) && !marked.contains(&words[at + 2]))
        .map(|&(at, ..)| words[at + 2])
        .collect();
    let Some(insert) = last_decorate.filter(|_| !add.is_empty()) else { return words };
    let mut out = Vec::with_capacity(words.len() + add.len() * 3);
    out.extend_from_slice(&words[..insert]);
    for id in add {
        out.extend_from_slice(&[(3 << 16) | OP_DECORATE, id, NON_UNIFORM]);
    }
    out.extend_from_slice(&words[insert..]);
    out
}

/// **(Weft bug, naga 30)** naga gives arrays in workgroup memory an `ArrayStride`, which Vulkan
/// forbids there (VUID-StandaloneSpirv-None-10684; the validation layer names three of the
/// engine's compute shaders). NVIDIA ignores it; on the Pixel 10's PowerVR the terrain those
/// shaders build came out wrong (pieces of ground floating in the air, faces from under the ground
/// showing through it). The decoration is dropped from arrays only workgroup memory uses.
fn workgroup_arrays_without_stride(words: Vec<u32>) -> Vec<u32> {
    use std::collections::{HashMap, HashSet};
    const OP_TYPE_ARRAY: u32 = 28;
    const OP_TYPE_RUNTIME_ARRAY: u32 = 29;
    const OP_TYPE_STRUCT: u32 = 30;
    const OP_TYPE_POINTER: u32 = 32;
    const OP_VARIABLE: u32 = 59;
    const OP_DECORATE: u32 = 71;
    const ARRAY_STRIDE: u32 = 6;
    const WORKGROUP: u32 = 4;
    // Storage classes whose memory has an explicit layout: a type also used there keeps it.
    const LAID_OUT: [u32; 4] = [2, 9, 12, 5349];
    if words.len() < 5 {
        return words;
    }
    let mut at = 5;
    let mut ops = Vec::new();
    while at < words.len() {
        let (len, op) = ((words[at] >> 16) as usize, words[at] & 0xffff);
        if len == 0 || at + len > words.len() {
            return words;
        }
        ops.push((at, op, len));
        at += len;
    }
    // What each array or struct type is made of, and each pointer type's class and pointee.
    let mut parts: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut arrays: HashSet<u32> = HashSet::new();
    let mut pointers: HashMap<u32, (u32, u32)> = HashMap::new();
    let mut workgroup_vars: Vec<u32> = Vec::new();
    for &(at, op, len) in &ops {
        let w = &words[at..at + len];
        match op {
            OP_TYPE_ARRAY | OP_TYPE_RUNTIME_ARRAY if len >= 3 => {
                arrays.insert(w[1]);
                parts.insert(w[1], vec![w[2]]);
            }
            OP_TYPE_STRUCT if len >= 2 => {
                parts.insert(w[1], w[2..].to_vec());
            }
            OP_TYPE_POINTER if len >= 4 => {
                pointers.insert(w[1], (w[2], w[3]));
            }
            OP_VARIABLE if len >= 4 && w[3] == WORKGROUP => workgroup_vars.push(w[1]),
            _ => {}
        }
    }
    let reach = |from: u32| {
        let mut seen = HashSet::new();
        let mut todo = vec![from];
        while let Some(t) = todo.pop() {
            if seen.insert(t) {
                todo.extend(parts.get(&t).into_iter().flatten().copied());
            }
        }
        seen
    };
    let mut shared: HashSet<u32> = HashSet::new();
    let mut laid_out: HashSet<u32> = HashSet::new();
    for &(class, pointee) in pointers.values() {
        if class == WORKGROUP {
            shared.extend(reach(pointee));
        } else if LAID_OUT.contains(&class) {
            laid_out.extend(reach(pointee));
        }
    }
    if workgroup_vars.is_empty() {
        return words;
    }
    let drop: HashSet<u32> = shared
        .iter()
        .copied()
        .filter(|t| arrays.contains(t) && !laid_out.contains(t))
        .collect();
    if drop.is_empty() {
        return words;
    }
    let mut out = Vec::with_capacity(words.len());
    out.extend_from_slice(&words[..5]);
    for &(at, op, len) in &ops {
        let w = &words[at..at + len];
        if op == OP_DECORATE && len >= 4 && w[2] == ARRAY_STRIDE && drop.contains(&w[1]) {
            continue;
        }
        out.extend_from_slice(w);
    }
    out
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
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                | vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
        } else {
            let mut u = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
            if desc.render_target {
                u |= vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC;
            }
            u | extra
        };
        // The Pixel 10's PowerVR: a target cleared by its pass and then read as a texture came back
        // as garbage in every tile only the clear had touched (black and random-coloured squares
        // where the sky was, the water's copy of what is behind it); uncompressed, it reads right.
        let mut compression_off = vk::ImageCompressionControlEXT::default().flags(vk::ImageCompressionFlagsEXT::DISABLED);
        let mut info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D { width: desc.width, height: desc.height, depth: 1 })
            .mip_levels(desc.mips)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage);
        if self.no_compression && (desc.render_target || depth) {
            info = info.push_next(&mut compression_off);
        }
        unsafe {
            let image = self
                .device
                .create_image(&info, None)
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
            let id =
                self.textures
                    .insert(Texture { image, view, alloc, desc: desc.clone(), layout: vk::ImageLayout::UNDEFINED, index, usage });
            self.uploads.push(Upload::InitTexture(id));
            TextureId(id)
        }
    }

    fn spirv(&self, shader: &Shader, stage: naga::ShaderStage, entry: &str) -> Result<vk::ShaderModule, Error> {
        let mut options = naga::back::spv::Options::default();
        options.lang_version = (1, 3);
        options.force_loop_bounding = false;
        let words = naga::back::spv::write_vec(
            &shader.module,
            &shader.info,
            &options,
            Some(&naga::back::spv::PipelineOptions { shader_stage: stage, entry_point: entry.to_string() }),
        )
        .map_err(|e| Error(format!("SPIR-V for {entry}: {e}")))?;
        let words = workgroup_arrays_without_stride(non_uniform_sampled_images(words));
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
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default()
                        .bindings(&bindings)
                        .flags(if self.push_fn.is_some() {
                            vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR
                        } else {
                            vk::DescriptorSetLayoutCreateFlags::empty()
                        }),
                    None,
                )
                .map_err(err("push descriptor layout"))?;
            let sets = [self.global_layout, set_layout];
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

impl VulkanBackend {
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
                Blend::NoColor => a.color_write_mask(vk::ColorComponentFlags::empty()),
            }
        };
        // The motion image (`PipelineDesc::motion`): written, never blended, by a fragment entry
        // with a `@location(1)` output; left as it is by the rest.
        let motion_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(
            if desc
                .fragment
                .is_some_and(|f| writes_location(desc.shader, f, 1))
            {
                vk::ColorComponentFlags::RGBA
            } else {
                vk::ColorComponentFlags::empty()
            },
        );
        // The extra color images (`create_pipeline_mrt`) after the motion image, at location 2 on:
        // written by a fragment entry with that output, never blended.
        let mut attachments = vec![blend_attachment, motion_attachment];
        for i in 0..extra.len() {
            let writes = desc
                .fragment
                .is_some_and(|f| writes_location(desc.shader, f, 2 + i as u32));
            attachments.push(vk::PipelineColorBlendAttachmentState::default().color_write_mask(if writes {
                vk::ColorComponentFlags::RGBA
            } else {
                vk::ColorComponentFlags::empty()
            }));
        }
        let count = match (desc.color.is_some(), desc.motion) {
            (false, _) => 0,
            (true, false) => 1,
            (true, true) => 2 + extra.len(),
        };
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments[..count]);
        let dynamic = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic);
        let color_formats: Vec<vk::Format> = desc
            .color
            .map(|f| self.vk_format(f))
            .into_iter()
            .chain(desc.motion.then(|| self.vk_format(MOTION_FORMAT)))
            .chain(extra.iter().map(|&f| self.vk_format(f)))
            .take(count)
            .collect();
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats)
            .depth_attachment_format(if desc.depth.is_some() { DEPTH_FORMAT } else { vk::Format::UNDEFINED });
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
            .push_next(&mut rendering);
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
        Ok(PipelineId(
            self.pipelines
                .insert(Pipeline { raw, layout, set_layout, bind_point: vk::PipelineBindPoint::GRAPHICS, buffers }),
        ))
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

    fn indirect_count(&self) -> bool {
        self.indirect_count
    }

    fn hand_own_pass(&self) -> bool {
        self.hand_own_pass
    }

    fn surface_size(&self) -> (u32, u32) {
        self.swapchain
            .as_ref()
            .map_or(self.want_size, |s| (s.extent.width, s.extent.height))
    }

    fn surface_lost(&mut self) {
        unsafe {
            self.device.device_wait_idle().ok();
            if let Some(sc) = self.swapchain.take() {
                self.destroy_swapchain(sc);
            }
            if self.surface != vk::SurfaceKHR::null() {
                self.surface_fn.destroy_surface(self.surface, None);
                self.surface = vk::SurfaceKHR::null();
            }
        }
        self.image_index = None;
        log::info!("surface lost: swapchain and surface dropped");
    }

    fn surface_restored(&mut self, window: &dyn Window) -> Result<(), Error> {
        if self.surface != vk::SurfaceKHR::null() {
            return Ok(());
        }
        let display = window
            .display_handle()
            .map_err(err("display handle"))?
            .as_raw();
        let raw = window
            .window_handle()
            .map_err(err("window handle"))?
            .as_raw();
        self.surface =
            unsafe { ash_window::create_surface(&self._entry, &self.instance, display, raw, None) }.map_err(err("creating surface"))?;
        self.dirty_swapchain = true;
        log::info!("surface restored");
        Ok(())
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

    fn texture_index(&self, texture: TextureId) -> u32 {
        self.textures.get(texture.0).index
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
        Ok(PipelineId(
            self.pipelines
                .insert(Pipeline { raw, layout, set_layout, bind_point: vk::PipelineBindPoint::COMPUTE, buffers }),
        ))
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

    fn begin_frame(&mut self) -> Option<FrameInfo> {
        // Between `surface_lost` and `surface_restored` (the app in the background on Android).
        if self.surface == vk::SurfaceKHR::null() {
            return None;
        }
        let t0 = std::time::Instant::now();
        self.ensure_slot_ready();
        let t1 = std::time::Instant::now();
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
                let extent = sc.logical;
                if suboptimal {
                    self.suboptimal();
                }
                self.image_index = Some(index);
                Some(FrameInfo { width: extent.width, height: extent.height, slot: self.slot })
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

    fn end_frame(&mut self, list: &CommandList) {
        let Some(image_index) = self.image_index.take() else { return };
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
        unsafe {
            let first_query = self.slot as u32 * MAX_MARKS;
            self.device
                .cmd_reset_query_pool(cmd, self.queries, first_query, MAX_MARKS);
            self.device
                .cmd_write_timestamp2(cmd, vk::PipelineStageFlags2::TOP_OF_PIPE, self.queries, first_query);
            self.record_uploads(cmd);
            self.rt_record_start(cmd);
            self.device
                .cmd_write_timestamp2(cmd, vk::PipelineStageFlags2::ALL_COMMANDS, self.queries, first_query + 1);
            let mut marks = 2;
            let sets = [self.global_set];
            let mut current: Option<u32> = None;
            // Without push descriptors: the buffers bound so far (binding, type, buffer), which
            // each new set carries, as a push keeps the bindings it doesn't write.
            let mut bound: Vec<(u32, vk::DescriptorType, vk::DescriptorBufferInfo)> = Vec::new();
            let mut in_pass_targets: Vec<u32> = Vec::new();
            let mut in_pass_depth: Option<u32> = None;
            // The game's passes for the screen see `logical`; turned, they draw into `turn`'s image.
            let (sc_image, sc_view, sc_extent, turn, sc_size) = {
                let sc = self.swapchain.as_ref().unwrap();
                (sc.images[image_index as usize], sc.views[image_index as usize], sc.logical, sc.turn, sc.extent)
            };
            // `Cmd::PassBreak`: the pass in progress ended and begun again, loading what it drew.
            let broken;
            let cmds: &[Cmd] = if list.cmds.iter().any(|c| matches!(c, Cmd::PassBreak)) {
                let mut out = Vec::with_capacity(list.cmds.len() + 8);
                let mut open: Option<Cmd> = None;
                let mut open_extras: Option<Cmd> = None;
                let mut extras: Option<Cmd> = None;
                for c in &list.cmds {
                    match *c {
                        Cmd::PassExtras(e) => {
                            extras = Some(Cmd::PassExtras(e.map(|x| x.map(|(t, _)| (t, None)))));
                            out.push(*c);
                        }
                        Cmd::BeginPass { color, depth, motion } => {
                            open_extras = extras.take();
                            open = Some(Cmd::BeginPass {
                                color: color.map(|(t, _)| (t, None)),
                                depth: depth.map(|(t, _)| (t, None)),
                                motion: motion.map(|(t, _)| (t, None)),
                            });
                            out.push(*c);
                        }
                        Cmd::EndPass => {
                            open = None;
                            out.push(*c);
                        }
                        Cmd::PassBreak => {
                            if let Some(again) = open {
                                out.push(Cmd::EndPass);
                                out.extend(open_extras);
                                out.push(again);
                            }
                        }
                        _ => out.push(*c),
                    }
                }
                broken = out;
                &broken
            } else {
                &list.cmds
            };
            // Consecutive BLAS builds, recorded as one batch (`rt_build_blases`).
            let mut blas_batch: Vec<u32> = Vec::new();
            // `Cmd::PassExtras`: the color images the next pass adds after its motion image.
            let mut pass_extras: [Option<(TextureId, Option<[f32; 4]>)>; 3] = [None; 3];
            for cmd_item in cmds {
                if let Cmd::BuildBlas(a) = *cmd_item {
                    blas_batch.push(a.0);
                    continue;
                }
                if !blas_batch.is_empty() {
                    self.rt_build_blases(cmd, &blas_batch);
                    blas_batch.clear();
                }
                match *cmd_item {
                    Cmd::PassExtras(e) => pass_extras = e,
                    Cmd::BeginPass { color, depth, motion } => {
                        let mut extent = sc_extent;
                        // Its extra color images (`Cmd::PassExtras`), after the motion image.
                        let mut extra_atts: Vec<vk::RenderingAttachmentInfo> = Vec::new();
                        for (t, clear) in std::mem::take(&mut pass_extras).into_iter().flatten() {
                            let tex = self.textures.get_mut(t.0);
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { tex.layout };
                            tex.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                            let (image, view) = (tex.image, tex.view);
                            in_pass_targets.push(t.0);
                            self.barrier_image(cmd, image, color_range(1), old, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                            extra_atts.push(
                                vk::RenderingAttachmentInfo::default()
                                    .image_view(view)
                                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                    .load_op(if clear.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                                    .store_op(vk::AttachmentStoreOp::STORE)
                                    .clear_value(vk::ClearValue { color: vk::ClearColorValue { float32: clear.unwrap_or_default() } }),
                            );
                        }
                        let mut color_att = None;
                        // The motion image, a second color attachment (`Cmd::BeginPass::motion`).
                        let motion_att = motion.map(|(t, clear)| {
                            let tex = self.textures.get_mut(t.0);
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { tex.layout };
                            tex.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                            let (image, view) = (tex.image, tex.view);
                            in_pass_targets.push(t.0);
                            self.barrier_image(cmd, image, color_range(1), old, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                            vk::RenderingAttachmentInfo::default()
                                .image_view(view)
                                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                .load_op(if clear.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .clear_value(vk::ClearValue { color: vk::ClearColorValue { float32: clear.unwrap_or_default() } })
                        });
                        if let Some((target, clear)) = color {
                            let (image, view, old) = match target {
                                Target::Surface => match turn {
                                    Some((picture, _)) => {
                                        let tex = self.textures.get_mut(picture.0);
                                        let old = tex.layout;
                                        tex.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                                        in_pass_targets.push(picture.0);
                                        (tex.image, tex.view, old)
                                    }
                                    None => {
                                        let sc = self.swapchain.as_mut().unwrap();
                                        let old = sc.layouts[image_index as usize];
                                        sc.layouts[image_index as usize] = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                                        (sc_image, sc_view, old)
                                    }
                                },
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
                            color_att = Some(
                                vk::RenderingAttachmentInfo::default()
                                    .image_view(view)
                                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                    .load_op(if clear.is_some() {
                                        vk::AttachmentLoadOp::CLEAR
                                    } else if old == vk::ImageLayout::UNDEFINED {
                                        vk::AttachmentLoadOp::DONT_CARE
                                    } else {
                                        vk::AttachmentLoadOp::LOAD
                                    })
                                    .store_op(vk::AttachmentStoreOp::STORE)
                                    .clear_value(vk::ClearValue { color: vk::ClearColorValue { float32: clear.unwrap_or_default() } }),
                            );
                        }
                        // Depth back to an attachment if a screen effect read it since (`EndPass`).
                        if let Some((t, clear)) = depth {
                            let tex = self.textures.get_mut(t.0);
                            let old = if clear.is_some() { vk::ImageLayout::UNDEFINED } else { tex.layout };
                            tex.layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
                            let image = tex.image;
                            if old != vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
                                self.barrier_image(cmd, image, depth_range(), old, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
                            }
                            in_pass_depth = Some(t.0);
                        }
                        let depth_att = depth.map(|(t, clear)| {
                            let tex = self.textures.get(t.0);
                            extent = vk::Extent2D { width: tex.desc.width, height: tex.desc.height };
                            vk::RenderingAttachmentInfo::default()
                                .image_view(tex.view)
                                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                                .load_op(if clear.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .clear_value(vk::ClearValue {
                                    depth_stencil: vk::ClearDepthStencilValue { depth: clear.unwrap_or(0.0), stencil: 0 },
                                })
                        });
                        let colors: Vec<_> = color_att
                            .into_iter()
                            .chain(motion_att)
                            .chain(extra_atts)
                            .collect();
                        let mut info = vk::RenderingInfo::default()
                            .render_area(vk::Rect2D { offset: Default::default(), extent })
                            .layer_count(1)
                            .color_attachments(&colors);
                        if let Some(d) = depth_att.as_ref() {
                            info = info.depth_attachment(d);
                        }
                        self.device.cmd_begin_rendering(cmd, &info);
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
                        self.device.cmd_end_rendering(cmd);
                        // What the pass drew its depth into can be read now.
                        if let Some(t) = in_pass_depth.take() {
                            let image = self.textures.get(t).image;
                            self.barrier_image(
                                cmd,
                                image,
                                depth_range(),
                                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
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
                        current = Some(p.0);
                    }
                    Cmd::BindBuffer { binding, buffer, offset, size } => {
                        let Some(p) = current else { continue };
                        let pl = self.pipelines.get(p);
                        // A binding the shader never reads is not in its reflection (a shader
                        // pack's that skips a buffer): nothing to bind.
                        let Some(&(_, ty)) = pl.buffers.iter().find(|b| b.0 == binding) else { continue };
                        let b = self.buffers.get(buffer.0);
                        let range = if size == u64::MAX { vk::WHOLE_SIZE } else { size };
                        let info = [vk::DescriptorBufferInfo { buffer: b.raw, offset, range }];
                        if let Some(push_fn) = &self.push_fn {
                            let write = vk::WriteDescriptorSet::default()
                                .dst_binding(binding)
                                .descriptor_type(ty)
                                .buffer_info(&info);
                            push_fn.cmd_push_descriptor_set(cmd, pl.bind_point, pl.layout, 1, &[write]);
                        } else {
                            bound.retain(|b| b.0 != binding);
                            bound.push((binding, ty, info[0]));
                            let (layout, set_layout, bind_point) = (pl.layout, pl.set_layout, pl.bind_point);
                            let wanted: Vec<(u32, vk::DescriptorType)> = pl.buffers.clone();
                            self.bind_buffer_set(cmd, layout, set_layout, bind_point, &wanted, &bound);
                        }
                    }
                    Cmd::PushConstants { start, len } => {
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
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_draw_indirect(cmd, b, offset, count, size_of::<DrawArgs>() as u32);
                    }
                    Cmd::IndexBuffer(buffer) => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_bind_index_buffer(cmd, b, 0, vk::IndexType::UINT32);
                    }
                    Cmd::DrawIndexedIndirect { buffer, offset, count } => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_draw_indexed_indirect(cmd, b, offset, count, size_of::<DrawIndexedArgs>() as u32);
                    }
                    Cmd::DrawIndexedIndirectCount { buffer, offset, max, .. } if !self.indirect_count => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_draw_indexed_indirect(cmd, b, offset, max, size_of::<DrawIndexedArgs>() as u32);
                    }
                    Cmd::DrawIndirectCount { buffer, offset, max, .. } if !self.indirect_count => {
                        let b = self.buffers.get(buffer.0).raw;
                        self.device
                            .cmd_draw_indirect(cmd, b, offset, max, size_of::<DrawArgs>() as u32);
                    }
                    Cmd::DrawIndexedIndirectCount { buffer, offset, count_buffer, count_offset, max } => {
                        let b = self.buffers.get(buffer.0).raw;
                        let c = self.buffers.get(count_buffer.0).raw;
                        self.device.cmd_draw_indexed_indirect_count(
                            cmd,
                            b,
                            offset,
                            c,
                            count_offset,
                            max,
                            size_of::<DrawIndexedArgs>() as u32,
                        );
                    }
                    Cmd::DrawIndirectCount { buffer, offset, count_buffer, count_offset, max } => {
                        let b = self.buffers.get(buffer.0).raw;
                        let c = self.buffers.get(count_buffer.0).raw;
                        self.device
                            .cmd_draw_indirect_count(cmd, b, offset, c, count_offset, max, size_of::<DrawArgs>() as u32);
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
                        let back = if was == vk::ImageLayout::UNDEFINED { vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL } else { was };
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
                    }
                    // Expanded into an end and a begin above.
                    Cmd::PassBreak => {}
                    // Batched above.
                    Cmd::BuildBlas(_) => {}
                    Cmd::BuildTlas(a) => self.rt_build_tlas(cmd, a.0),
                    Cmd::BindAccel { binding, accel } => {
                        let (Some(p), Some(push_fn)) = (current, &self.push_fn) else { continue };
                        let pl = self.pipelines.get(p);
                        if !pl.buffers.iter().any(|b| b.0 == binding) {
                            continue;
                        }
                        let Some(raw) = self.rt_tlas_raw(accel.0) else { continue };
                        let raws = [raw];
                        let mut accel_write = vk::WriteDescriptorSetAccelerationStructureKHR::default().acceleration_structures(&raws);
                        let write = vk::WriteDescriptorSet::default()
                            .dst_binding(binding)
                            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                            .descriptor_count(1)
                            .push_next(&mut accel_write);
                        push_fn.cmd_push_descriptor_set(cmd, pl.bind_point, pl.layout, 1, &[write]);
                    }
                    Cmd::Mark => {
                        if marks < MAX_MARKS - 1 {
                            self.device
                                .cmd_write_timestamp2(cmd, vk::PipelineStageFlags2::ALL_COMMANDS, self.queries, first_query + marks);
                            marks += 1;
                        }
                    }
                }
            }

            if !blas_batch.is_empty() {
                self.rt_build_blases(cmd, &blas_batch);
            }
            // Pre-rotation: the game's picture into the swapchain image, turned (`TURN_WGSL`).
            if let (Some((picture, quarter)), Some(pipe)) = (turn, self.turn_pipe) {
                let (pic_image, pic_layout, pic_index) = {
                    let t = self.textures.get(picture.0);
                    (t.image, t.layout, t.index)
                };
                if pic_layout != vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
                    self.barrier_image(cmd, pic_image, color_range(1), pic_layout, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
                    self.textures.get_mut(picture.0).layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                }
                self.barrier_image(cmd, sc_image, color_range(1), vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                let colors = [vk::RenderingAttachmentInfo::default()
                    .image_view(sc_view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .store_op(vk::AttachmentStoreOp::STORE)];
                let info = vk::RenderingInfo::default()
                    .render_area(vk::Rect2D { offset: Default::default(), extent: sc_size })
                    .layer_count(1)
                    .color_attachments(&colors);
                self.device.cmd_begin_rendering(cmd, &info);
                self.device.cmd_set_viewport(
                    cmd,
                    0,
                    &[vk::Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: sc_size.width as f32,
                        height: sc_size.height as f32,
                        min_depth: 0.0,
                        max_depth: 1.0,
                    }],
                );
                self.device
                    .cmd_set_scissor(cmd, 0, &[vk::Rect2D { offset: Default::default(), extent: sc_size }]);
                let pl = self.pipelines.get(pipe.0);
                self.device.cmd_bind_pipeline(cmd, pl.bind_point, pl.raw);
                self.device
                    .cmd_bind_descriptor_sets(cmd, pl.bind_point, pl.layout, 0, &sets, &[]);
                self.device
                    .cmd_draw(cmd, 3, 1, if quarter == 1 { 0 } else { 3 }, pic_index);
                self.device.cmd_end_rendering(cmd);
                self.swapchain.as_mut().unwrap().layouts[image_index as usize] = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            }
            let old = self.swapchain.as_ref().unwrap().layouts[image_index as usize];
            if self.capture_requested {
                self.capture_requested = false;
                let (w, h) = (sc_extent.width, sc_extent.height);
                // Turned: the picture as the game drew it, upright.
                let (cap_image, cap_old) = match turn {
                    Some((picture, _)) => {
                        let t = self.textures.get(picture.0);
                        (t.image, t.layout)
                    }
                    None => (sc_image, old),
                };
                let (buf, alloc) = raw_buffer(
                    &self.device,
                    &mut self.allocator,
                    self.spill,
                    (w * h * 4) as u64,
                    vk::BufferUsageFlags::TRANSFER_DST,
                    MemoryLocation::GpuToCpu,
                    "capture",
                );
                self.barrier_image(cmd, cap_image, color_range(1), cap_old, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
                let region = [vk::BufferImageCopy::default()
                    .image_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image_extent(vk::Extent3D { width: w, height: h, depth: 1 })];
                self.device
                    .cmd_copy_image_to_buffer(cmd, cap_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, buf, &region);
                if turn.is_some() {
                    self.barrier_image(cmd, cap_image, color_range(1), vk::ImageLayout::TRANSFER_SRC_OPTIMAL, cap_old);
                    self.barrier_image(cmd, sc_image, color_range(1), old, vk::ImageLayout::PRESENT_SRC_KHR);
                } else {
                    self.barrier_image(
                        cmd,
                        sc_image,
                        color_range(1),
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        vk::ImageLayout::PRESENT_SRC_KHR,
                    );
                }
                let fmt = self.swapchain.as_ref().unwrap().format;
                self.capture = Some((buf, alloc, w, h, fmt));
            } else {
                self.barrier_image(cmd, sc_image, color_range(1), old, vk::ImageLayout::PRESENT_SRC_KHR);
            }
            self.swapchain.as_mut().unwrap().layouts[image_index as usize] = vk::ImageLayout::PRESENT_SRC_KHR;
            self.device
                .cmd_write_timestamp2(cmd, vk::PipelineStageFlags2::ALL_COMMANDS, self.queries, first_query + marks);
            self.frames[self.slot].marks = marks + 1;
            self.device.end_command_buffer(cmd).ok();

            let frame = &mut self.frames[self.slot];
            let rendered = self.swapchain.as_ref().unwrap().rendered[image_index as usize];
            // Only drawing into the swapchain image waits for it (its barriers from UNDEFINED or
            // PRESENT start at this stage): the frame's uploads, compute and offscreen passes go first.
            let acquire_at =
                if all_barriers() { vk::PipelineStageFlags2::ALL_COMMANDS } else { vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT };
            let waits = [vk::SemaphoreSubmitInfo::default()
                .semaphore(frame.acquired)
                .stage_mask(acquire_at)];
            let signals = [vk::SemaphoreSubmitInfo::default()
                .semaphore(rendered)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)];
            let cmds = [vk::CommandBufferSubmitInfo::default().command_buffer(cmd)];
            let submit = vk::SubmitInfo2::default()
                .wait_semaphore_infos(&waits)
                .command_buffer_infos(&cmds)
                .signal_semaphore_infos(&signals);
            self.device.reset_fences(&[frame.fence]).ok();
            queued = std::time::Instant::now();
            if let Err(e) = self
                .device
                .queue_submit2(self.queue, &[submit], frame.fence)
            {
                log::error!("vkQueueSubmit2: {e:?}");
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
            match self.swapchain_fn.queue_present(self.queue, &present) {
                Ok(false) => {}
                Ok(true) => self.suboptimal(),
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.dirty_swapchain = true,
                Err(e) => log::error!("vkQueuePresentKHR: {e:?}"),
            }
        }
        self.slot = (self.slot + 1) % FRAMES;
        self.log_submit(began, queued, submitted);
    }

    fn request_capture(&mut self) {
        self.capture_requested = true;
    }

    fn pipeline_cache_data(&self) -> Vec<u8> {
        unsafe { self.device.get_pipeline_cache_data(self.pipeline_cache) }.unwrap_or_default()
    }

    fn load_pipeline_cache(&mut self, data: &[u8]) {
        // Vulkan's header: its length (32), version 1, vendor, device, then the cache's UUID. Only
        // this very driver's own is passed on.
        let props = unsafe { self.instance.get_physical_device_properties(self.pdev) };
        let word = |i: usize| {
            data.get(i..i + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        let ours = data.len() > 32
            && word(0) == Some(32)
            && word(4) == Some(vk::PipelineCacheHeaderVersion::ONE.as_raw() as u32)
            && word(8) == Some(props.vendor_id)
            && word(12) == Some(props.device_id)
            && data[16..32] == props.pipeline_cache_uuid;
        if !ours {
            return;
        }
        let Ok(cache) = (unsafe {
            self.device
                .create_pipeline_cache(&vk::PipelineCacheCreateInfo::default().initial_data(data), None)
        }) else {
            return;
        };
        unsafe {
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None)
        };
        self.pipeline_cache = cache;
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
                self.device.destroy_command_pool(f.pool, None);
                for &pool in &f.set_pools {
                    self.device.destroy_descriptor_pool(pool, None);
                }
                self.device.destroy_semaphore(f.acquired, None);
                self.device.destroy_fence(f.fence, None);
            }
            self.rt_drop();
            self.device.destroy_query_pool(self.queries, None);
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None);
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
            self.device.destroy_descriptor_pool(self.global_pool, None);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A texture from the bindless array by a per-pixel index, sampled in a helper function: the
    /// `OpSampledImage` gets `NonUniform` like the image it is made from (naga 30 leaves it off).
    #[test]
    fn sampled_images_of_non_uniform_textures_are_marked() {
        let src = r#"enable wgpu_binding_array;
@group(0) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(0) @binding(1) var samp: sampler;
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) @interpolate(flat) layer: u32, @location(1) uv: vec2<f32> };
fn sample(in: VOut) -> vec4<f32> { return textureSample(textures[in.layer], samp, in.uv); }
@fragment fn fs(in: VOut) -> @location(0) vec4<f32> { return sample(in); }
"#;
        let module = naga::front::wgsl::parse_str(src).unwrap();
        let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .unwrap();
        let words = naga::back::spv::write_vec(&module, &info, &Default::default(), None).unwrap();
        let fixed = non_uniform_sampled_images(words.clone());
        assert_eq!(fixed.len(), words.len() + 3, "one decoration added");
        // The added decoration names the OpSampledImage's result.
        let mut at = 5;
        let mut sampled = None;
        while at < fixed.len() {
            let (len, op) = ((fixed[at] >> 16) as usize, fixed[at] & 0xffff);
            if op == 86 {
                sampled = Some(fixed[at + 2]);
            }
            at += len;
        }
        let sampled = sampled.unwrap();
        assert!(
            fixed
                .windows(3)
                .any(|w| w == [(3 << 16) | 71, sampled, 5300])
        );
        // Again changes nothing.
        assert_eq!(non_uniform_sampled_images(fixed.clone()), fixed);
    }

    /// A workgroup array loses its `ArrayStride`; one also in a storage buffer keeps it.
    #[test]
    fn workgroup_arrays_lose_their_stride() {
        let src = r#"
var<workgroup> tile: array<f32, 64>;
@group(0) @binding(0) var<storage, read_write> out: array<u32, 16>;
@compute @workgroup_size(64) fn cs(@builtin(local_invocation_index) i: u32) {
    tile[i] = f32(i);
    workgroupBarrier();
    if i < 16u { out[i] = u32(tile[63u - i]); }
}
"#;
        let module = naga::front::wgsl::parse_str(src).unwrap();
        let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .unwrap();
        let words = naga::back::spv::write_vec(&module, &info, &Default::default(), None).unwrap();
        let strides = |w: &[u32]| {
            let mut at = 5;
            let mut n = 0;
            while at < w.len() {
                let (len, op) = ((w[at] >> 16) as usize, w[at] & 0xffff);
                if op == 71 && w[at + 2] == 6 {
                    n += 1;
                }
                at += len;
            }
            n
        };
        let fixed = workgroup_arrays_without_stride(words.clone());
        assert_eq!(strides(&fixed), strides(&words) - 1, "only the workgroup array's stride goes");
        assert_eq!(workgroup_arrays_without_stride(fixed.clone()), fixed);
    }

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
        assert_eq!(spill_types(&discrete), 0b01101);
        // Shared memory (Apple): every heap is the GPU's, so nothing to spill to.
        let unified = memory(&[vk::MemoryHeapFlags::DEVICE_LOCAL], &[(P::DEVICE_LOCAL, 0), (P::DEVICE_LOCAL | host, 0)]);
        assert_eq!(spill_types(&unified), 0);
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
fn layout_stages(layout: vk::ImageLayout, src: bool) -> (vk::PipelineStageFlags2, vk::AccessFlags2) {
    use vk::{AccessFlags2 as A, ImageLayout as L, PipelineStageFlags2 as S};
    let shaders = S::VERTEX_SHADER | S::FRAGMENT_SHADER | S::COMPUTE_SHADER;
    if all_barriers() {
        return if src { (S::ALL_COMMANDS, A::MEMORY_WRITE) } else { (S::ALL_COMMANDS, A::MEMORY_READ | A::MEMORY_WRITE) };
    }
    match (layout, src) {
        // Contents dropped: only the work that last used the image (for writing it again after
        // it was read) and the swapchain's acquire, which waits at colour output.
        (L::UNDEFINED, true) => {
            (shaders | S::COLOR_ATTACHMENT_OUTPUT | S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS | S::ALL_TRANSFER, A::NONE)
        }
        (L::COLOR_ATTACHMENT_OPTIMAL, true) => (S::COLOR_ATTACHMENT_OUTPUT, A::COLOR_ATTACHMENT_WRITE),
        (L::COLOR_ATTACHMENT_OPTIMAL, false) => (S::COLOR_ATTACHMENT_OUTPUT, A::COLOR_ATTACHMENT_READ | A::COLOR_ATTACHMENT_WRITE),
        (L::DEPTH_ATTACHMENT_OPTIMAL, true) => (S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS, A::DEPTH_STENCIL_ATTACHMENT_WRITE),
        (L::DEPTH_ATTACHMENT_OPTIMAL, false) => {
            (S::EARLY_FRAGMENT_TESTS | S::LATE_FRAGMENT_TESTS, A::DEPTH_STENCIL_ATTACHMENT_READ | A::DEPTH_STENCIL_ATTACHMENT_WRITE)
        }
        // Sampled: reads only, so what follows just waits for them to finish (no access to flush).
        (L::SHADER_READ_ONLY_OPTIMAL, true) => (shaders, A::NONE),
        (L::SHADER_READ_ONLY_OPTIMAL, false) => (shaders, A::SHADER_SAMPLED_READ | A::SHADER_READ),
        (L::TRANSFER_DST_OPTIMAL, true) => (S::ALL_TRANSFER, A::TRANSFER_WRITE),
        (L::TRANSFER_DST_OPTIMAL, false) => (S::ALL_TRANSFER, A::TRANSFER_WRITE),
        (L::TRANSFER_SRC_OPTIMAL, true) => (S::ALL_TRANSFER, A::NONE),
        (L::TRANSFER_SRC_OPTIMAL, false) => (S::ALL_TRANSFER, A::TRANSFER_READ),
        // From presenting: after the acquire, which waits at colour output. To presenting: the
        // semaphore that present waits on covers it.
        (L::PRESENT_SRC_KHR, true) => (S::COLOR_ATTACHMENT_OUTPUT, A::NONE),
        (L::PRESENT_SRC_KHR, false) => (S::NONE, A::NONE),
        (_, true) => (S::ALL_COMMANDS, A::MEMORY_WRITE),
        (_, false) => (S::ALL_COMMANDS, A::MEMORY_READ | A::MEMORY_WRITE),
    }
}

/// `WEFT_ALL_BARRIERS=1`: image barriers and the acquire wait at `ALL_COMMANDS`, as before.
fn all_barriers() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("WEFT_ALL_BARRIERS").is_some())
}

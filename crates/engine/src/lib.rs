pub mod app;
pub mod assets;
pub mod audio;
pub mod gpu_layout;
pub mod kv;

pub mod bench;
pub mod block_model;
pub mod client;
pub mod clipboard;
pub mod commands;
pub mod components;
pub mod conditions;
pub mod config;
pub mod content;
pub mod dedicated;
pub mod diag;
pub mod game_screens;
pub mod host;
pub mod hud;
pub mod hud_editor;
pub mod input;
pub mod log;
pub mod loot;
pub mod model;
pub mod modloader;
pub mod net;
pub mod packs;
pub mod paths;
pub mod physics;
pub mod post;
pub mod refs;
pub mod registry;
pub mod saves;
pub mod screens;
pub mod server;
pub mod settings;
pub mod shaders;
pub mod shape;
pub mod stack;
pub mod touch;
pub mod ui;
pub mod wire;
pub mod world;

// chiacraft's own parts (`chiacraft/`), under their old names too.
pub mod chiacraft;
pub use chiacraft::worlds::{admission, friends, links, saved_worlds, travel, world_address};
pub use chiacraft::{account_screen, bridge, package_trust};

pub use app::api::{exit_code, set_launch_options};
/// Android (crates/android): the activity glue winit runs on, and the game started on it.
#[cfg(target_os = "android")]
pub use app::run_android;
pub use app::{BackendFactory, Options, run};
#[cfg(target_os = "android")]
pub use winit::platform::android::activity as android_activity;

/// Background workers (generation, meshing, region ticks) leave two cores to the render and
/// server threads: a full pool starved the render thread during loading and frames missed vsync.
pub fn init_workers() {
    // The frame's own parallel work (culling, taking terrain) runs here and the main thread
    // waits for it: above the bulk pools, which keep every core busy while loading.
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(workers())
        .start_handler(|_| set_priority(PRIORITY_ABOVE_NORMAL))
        .build_global();
    set_priority(PRIORITY_ABOVE_NORMAL);
}

pub(crate) fn workers() -> usize {
    std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .saturating_sub(2)
        .max(1)
}

/// The pool for bulk work queued by the thousand (generating and saving cubes). Work that
/// something waits for (region ticks, path searches, meshing) runs on the global pool: queued
/// behind a full generation backlog there, a tick or a frame waited for all of it.
pub fn background() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers())
            .thread_name(|i| format!("background {i}"))
            .start_handler(|_| lower_priority())
            .build()
            .expect("background pool")
    })
}

/// The client's own terrain: finishing the cubes the GPU made, the CPU's where it can't, and the
/// mods' features under them. Apart from `background`, where it waited in one queue behind the
/// integrated server's generation (a saturated pool while loading: finishing a cube took 0.3 to
/// 11 ms, perf review). `WEFT_SHARED_TERRAIN_POOL=1` puts it back on `background`, to compare.
pub fn client_terrain() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    static SHARED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *SHARED.get_or_init(|| std::env::var_os("WEFT_SHARED_TERRAIN_POOL").is_some()) {
        return background();
    }
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers())
            .thread_name(|i| format!("client terrain {i}"))
            .start_handler(|_| lower_priority())
            .build()
            .expect("client terrain pool")
    })
}

/// Meshing: queued by the thousand while loading, so apart from the global pool, where the
/// frame's culling waited behind it.
pub fn meshing() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers())
            .thread_name(|i| format!("mesh {i}"))
            .start_handler(|_| lower_priority())
            .build()
            .expect("mesh pool")
    })
}

/// Meshing of what the player is looking at changing (around them, edits): a few threads of
/// normal priority, not behind the loading queue.
pub fn meshing_urgent() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .thread_name(|i| format!("mesh urgent {i}"))
            .build()
            .expect("urgent mesh pool")
    })
}

/// Bulk work runs below normal priority: with every core busy loading, the frame and the
/// server still get theirs when they need them.
pub fn lower_priority() {
    set_priority(PRIORITY_BELOW_NORMAL);
}

const PRIORITY_BELOW_NORMAL: i32 = -1;
pub(crate) const PRIORITY_ABOVE_NORMAL: i32 = 1;

/// The calling thread's scheduling priority (Windows values; Linux maps them onto nice; macOS only
/// with `WEFT_BULK_QOS=utility`).
pub(crate) fn set_priority(_priority: i32) {
    // macOS: nothing is set unless `WEFT_BULK_QOS=utility`, which runs bulk work as "utility" and
    // the frame's own as "user interactive", for an A/B on a Mac: utility may move far terrain
    // onto the efficiency cores and slow its fill (Planner, 11-05: not shipped until measured).
    #[cfg(target_os = "macos")]
    if std::env::var("WEFT_BULK_QOS").as_deref() == Ok("utility") {
        use libc::qos_class_t::{QOS_CLASS_DEFAULT, QOS_CLASS_USER_INTERACTIVE, QOS_CLASS_UTILITY};
        let class = match _priority {
            p if p > 0 => QOS_CLASS_USER_INTERACTIVE,
            p if p < 0 => QOS_CLASS_UTILITY,
            _ => QOS_CLASS_DEFAULT,
        };
        unsafe {
            libc::pthread_set_qos_class_self_np(class, 0);
        }
    }
    #[cfg(windows)]
    {
        unsafe extern "system" {
            fn GetCurrentThread() -> isize;
            fn SetThreadPriority(thread: isize, priority: i32) -> i32;
        }
        unsafe {
            SetThreadPriority(GetCurrentThread(), _priority);
        }
    }
    // Windows: +1 above normal, -1 below. Linux nice: lower is higher, so they flip. Raising
    // the priority needs a privilege and quietly does nothing without it.
    // Below normal is nice 10 (nice 1 hardly yielded to the frame with every core loading;
    // `WEFT_BULK_NICE=<n>` to compare).
    #[cfg(target_os = "linux")]
    {
        let below: i32 = std::env::var("WEFT_BULK_NICE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let nice = if _priority < 0 { below } else { -_priority.clamp(0, 1) };
        let tid = unsafe { libc::syscall(libc::SYS_gettid) } as libc::id_t;
        unsafe {
            libc::setpriority(libc::PRIO_PROCESS, tid, nice);
        }
    }
}

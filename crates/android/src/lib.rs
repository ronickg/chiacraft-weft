//! chiacraft on Android: the native library a NativeActivity starts (`android_main`).
//!
//! The APK carries the game as one asset, `game.zip` (the engine's assets in `assets/core`, our
//! mods packed in `run/mods`, as `weft --dist` lays them out), unpacked into the app's storage
//! when its build changes. Saves, settings, logs and unpacked mods live beside it:
//!
//! ```text
//! <files>/game/   what the APK brought (replaced on an update)
//! <files>/run/    the player's: saves, config, logs, their own mods and packs
//! <files>/cache/  unpacked mods, the pipeline cache
//! ```
//!
//! Launch options, for checks: one per line or space separated, in `<files>/args.txt` or the
//! app's external folder (`/sdcard/Android/data/<package>/files/args.txt`, which `adb push`
//! reaches), as on a desktop's command line (`--world droid1 --log frame=debug`), and
//! `--env NAME=value` for the engine's environment switches.
#![cfg(target_os = "android")]

use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

use weft_engine::Options;
use weft_engine::android_activity::AndroidApp;
use weft_render::{Backend, BackendDesc, Error};

#[global_allocator]
static ALLOC: weft_engine::diag::CountingAlloc = weft_engine::diag::CountingAlloc;

fn vulkan(desc: &BackendDesc) -> Result<Box<dyn Backend>, Error> {
    Ok(Box::new(weft_vulkan::VulkanBackend::new(desc)?))
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // A NativeActivity made again in the same process starts another `android_main`; the engine
    // is once per process (one event loop, its statics), so a fresh process has to start it.
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        eprintln!("chiacraft: android_main again in the same process; ending it for a fresh start");
        std::process::exit(0);
    }
    stdio_to_logcat();
    let Some(files) = app.internal_data_path() else {
        eprintln!("chiacraft: the app has no storage");
        return;
    };
    let game = files.join("game");
    let run = files.join("run");
    let cache = files.join("cache");
    if let Err(e) = unpack(&app, &game) {
        eprintln!("chiacraft: cannot unpack the game: {e}");
        std::process::exit(1);
    }
    let args = launch_args(&files, app.external_data_path().as_deref());
    let mut opts = Options {
        run_dir: run.clone(),
        cache_dir: cache,
        mod_dirs: Vec::new(),
        package_dirs: vec![run.join("packages"), game.join("packages")],
        gpu_debug: false,
        world: None,
        seed: None,
        screen: None,
        size: (0, 0),
        dev: false,
        automated: false,
        no_vsync: false,
        view: None,
    };
    let (mut verbose, mut log_spec, mut extra) = (false, String::new(), Vec::new());
    let mut it = args.into_iter().peekable();
    while let Some(a) = it.next() {
        let mut value = || it.next().unwrap_or_default();
        match a.as_str() {
            "--world" => opts.world = Some(value()),
            "--seed" => opts.seed = value().parse().ok(),
            "--screen" => opts.screen = Some(value()),
            "--view" => opts.view = value().parse().ok(),
            "--no-vsync" => opts.no_vsync = true,
            "--gpu-debug" => opts.gpu_debug = true,
            "--verbose" => verbose = true,
            "--log" => log_spec = value(),
            // An environment variable for this run (`--env WEFT_ALL_BARRIERS=1`), as a desktop's
            // shell would set it: the engine's switches for comparing ways of drawing.
            "--env" => {
                if let Some((k, v)) = value().split_once('=') {
                    // SAFETY: before any other thread starts that reads the environment.
                    unsafe { std::env::set_var(k, v) };
                }
            }
            other => {
                if let Some(name) = other.strip_prefix("--") {
                    let v = if it.peek().is_some_and(|n| !n.starts_with("--")) { it.next().unwrap_or_default() } else { "1".into() };
                    extra.push((name.to_string(), v));
                }
            }
        }
    }
    // The player's own mods first, then the game's.
    opts.mod_dirs = vec![run.join("mods"), game.join("run").join("mods")];
    opts.automated = extra.iter().any(|(k, _)| k == "script" || k == "test");
    weft_engine::paths::set_config(run.join("config"));
    weft_engine::paths::set_engine_assets(game.join("assets").join("core"));
    if let Some(dir) = own_dir() {
        weft_engine::paths::set_library_dir(dir);
    }
    weft_engine::set_launch_options(extra);
    weft_engine::log::init(Some(&run.join("logs").join("latest.log")), verbose, &log_spec);
    weft_engine::diag::install_crash_handler(&run);
    log::info!("android: game {}, run {}, build {}", game.display(), run.display(), read_stamp(&game).unwrap_or_default());
    weft_engine::init_workers();
    let code = match weft_engine::run_android(app, vulkan, opts) {
        Ok(()) => weft_engine::exit_code(),
        Err(e) => {
            log::error!("{e}");
            1
        }
    };
    weft_engine::log::flush();
    // The process may be kept for the next start, but the engine's state is once per process.
    std::process::exit(code);
}

/// The folder this library was loaded from: the APK's native libraries for this CPU, where the
/// mods' libraries are too.
fn own_dir() -> Option<PathBuf> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
    if unsafe { libc::dladdr(android_main as *const libc::c_void, &mut info) } == 0 || info.dli_fname.is_null() {
        return None;
    }
    let path = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) };
    Path::new(&*path.to_string_lossy())
        .parent()
        .map(Path::to_path_buf)
}

fn read_stamp(game: &Path) -> Option<String> {
    std::fs::read_to_string(game.join(".stamp"))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Unpacks `game.zip` from the APK into `game` when the APK's `game.stamp` (its build) differs
/// from the one unpacked last.
fn unpack(app: &AndroidApp, game: &Path) -> Result<(), String> {
    let assets = app.asset_manager();
    let mut stamp = String::new();
    assets
        .open(c"game.stamp")
        .ok_or("no game.stamp in the APK")?
        .read_to_string(&mut stamp)
        .map_err(|e| e.to_string())?;
    let stamp = stamp.trim().to_string();
    if read_stamp(game).as_deref() == Some(stamp.as_str()) {
        return Ok(());
    }
    let started = std::time::Instant::now();
    let _ = std::fs::remove_dir_all(game);
    std::fs::create_dir_all(game).map_err(|e| e.to_string())?;
    let zip = assets.open(c"game.zip").ok_or("no game.zip in the APK")?;
    let mut archive = zip::ZipArchive::new(zip).map_err(|e| e.to_string())?;
    archive.extract(game).map_err(|e| e.to_string())?;
    std::fs::write(game.join(".stamp"), &stamp).map_err(|e| e.to_string())?;
    eprintln!("chiacraft: unpacked build {stamp} ({} files) in {:.1} s", archive.len(), started.elapsed().as_secs_f32());
    Ok(())
}

/// What `args.txt` holds, in the app's folder or its external one (the first found).
fn launch_args(files: &Path, external: Option<&Path>) -> Vec<String> {
    let mut places: Vec<PathBuf> = vec![files.join("args.txt")];
    places.extend(external.map(|e| e.join("args.txt")));
    places
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .map(|t| {
            t.lines()
                .filter(|l| !l.trim_start().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
                .collect()
        })
        .unwrap_or_default()
}

/// Android drops what a native app writes to stdout and stderr: send it to logcat (tag
/// `chiacraft`), where the engine's log lines and panics then show.
fn stdio_to_logcat() {
    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_write(prio: libc::c_int, tag: *const libc::c_char, text: *const libc::c_char) -> libc::c_int;
    }
    let mut fds = [0; 2];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return;
        }
        libc::dup2(fds[1], libc::STDOUT_FILENO);
        libc::dup2(fds[1], libc::STDERR_FILENO);
    }
    let read = unsafe { <std::fs::File as std::os::fd::FromRawFd>::from_raw_fd(fds[0]) };
    let _ = std::thread::Builder::new()
        .name("logcat".into())
        .spawn(move || {
            let mut lines = std::io::BufReader::new(read);
            let mut line = Vec::new();
            while lines.read_until(b'\n', &mut line).is_ok_and(|n| n > 0) {
                // Without the terminal's colours.
                let text: Vec<u8> = strip_ansi(&line)
                    .into_iter()
                    .filter(|&b| b != b'\n' && b != 0)
                    .chain([0])
                    .collect();
                unsafe {
                    // ANDROID_LOG_INFO
                    __android_log_write(4, c"chiacraft".as_ptr(), text.as_ptr().cast());
                }
                line.clear();
            }
        });
}

fn strip_ansi(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == 0x1b && s.get(i + 1) == Some(&b'[') {
            i += 2;
            while i < s.len() && !(s[i] as char).is_ascii_alphabetic() {
                i += 1;
            }
            i += 1;
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    out
}

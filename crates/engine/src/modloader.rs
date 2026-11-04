//! Finds mod libraries, loads them and runs registration in parallel.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;

use rayon::prelude::*;
use weft_abi::{self as abi, ModId};

use crate::content::{self, ModMeta, RawRegistrar};

pub struct LoadedMod {
    pub id: String,
    pub assets: PathBuf,
    _lib: libloading::Library,
    decl: &'static abi::ModDecl,
}

/// Mod and package ids by `ModId`, for the content being built or in force. Kept for the life of
/// the game (a few per package set), so names handed out stay valid.
static NAMES: parking_lot::RwLock<&'static [String]> = parking_lot::RwLock::new(&[]);

pub fn mod_id_name(m: ModId) -> &'static str {
    let names: &'static [String] = *NAMES.read();
    names.get(m.0 as usize).map_or("?", |s| s.as_str())
}

fn set_names(names: Vec<String>) {
    if **NAMES.read() != names[..] {
        *NAMES.write() = Box::leak(names.into_boxed_slice());
    }
}

/// Mods found in `mod_dirs`, earlier dirs winning: a folder (`assets/` and the library) or the
/// same packed into `<name>.zip`, unpacked into `cache` once per version of the file. A folder's
/// library may also sit next to the engine executable instead (development builds,
/// `target/<profile>/`).
pub fn discover(mod_dirs: &[PathBuf], cache: &Path) -> Vec<(String, PathBuf, PathBuf)> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let mut found = Vec::new();
    for dir in mod_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let zipped = path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
            if !path.is_dir() && !zipped {
                continue;
            }
            let name = if zipped { path.file_stem() } else { path.file_name() }
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            // For a name found twice, the first dir's copy stays. Before unpacking: another
            // version unpacked takes away the one before it.
            if found
                .iter()
                .any(|f: &(String, PathBuf, PathBuf)| f.0 == name)
            {
                continue;
            }
            let root = if zipped {
                match unpack(&path, &cache.join("unpacked"), &name) {
                    Ok(root) => root,
                    Err(e) => {
                        log::error!("mod {}: {e}", path.display());
                        continue;
                    }
                }
            } else {
                path
            };
            let file = libloading::library_filename(&name);
            let lib = [Some(root.join(&file)), exe_dir.as_ref().map(|d| d.join(&file))]
                .into_iter()
                .flatten()
                .find(|p| p.exists());
            match lib {
                Some(lib) => found.push((name, root, lib)),
                // The library is named after the mod (its crate), and so must be the folder or zip.
                None => log::error!(
                    "mod {}: no {} in it; a mod's folder or zip is named like its crate",
                    root.display(),
                    file.to_string_lossy()
                ),
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Unpacks a mod archive into `<cache>/<name>-<size>-<mtime>`, unless that is already there;
/// older unpacked versions of the mod are removed.
fn unpack(archive: &Path, cache: &Path, name: &str) -> Result<PathBuf, String> {
    let meta = std::fs::metadata(archive).map_err(|e| e.to_string())?;
    let stamp = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    let dest = cache.join(format!("{name}-{}-{stamp}", meta.len()));
    if dest.join(".done").exists() {
        return Ok(dest);
    }
    if let Ok(old) = std::fs::read_dir(cache) {
        for e in old.flatten() {
            if e.file_name().to_string_lossy().rsplitn(3, '-').nth(2) == Some(name) {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    zip.extract(&dest).map_err(|e| e.to_string())?;
    std::fs::write(dest.join(".done"), b"").map_err(|e| e.to_string())?;
    log::info!("unpacked mod {name}");
    Ok(dest)
}

/// The libraries of mod `name` to pack, with their file names: those in its folder, one per
/// system, so that one zip serves them all (`discover` picks the system's own), and this system's
/// from next to the engine executable if the folder has none.
fn libraries(dir: &Path, name: &str) -> Vec<(String, PathBuf)> {
    let mut libs: Vec<(String, PathBuf)> = [format!("{name}.dll"), format!("lib{name}.so"), format!("lib{name}.dylib")]
        .into_iter()
        .map(|f| (dir.join(&f), f))
        .filter(|(p, _)| p.exists())
        .map(|(p, f)| (f, p))
        .collect();
    let own = libloading::library_filename(name)
        .to_string_lossy()
        .into_owned();
    if !libs.iter().any(|l| l.0 == own) {
        let beside = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(&own)));
        libs.extend(beside.filter(|p| p.exists()).map(|p| (own, p)));
    }
    libs
}

/// Packs a mod folder (its `assets/` and `libraries`) into `<out>/<name>.zip`, the single file
/// players drop into `mods/`.
pub fn pack(dir: &Path, out: &Path) -> Result<PathBuf, String> {
    use std::io::Write;
    let name = dir
        .file_name()
        .ok_or("not a mod folder")?
        .to_string_lossy()
        .into_owned();
    let libs = libraries(dir, &name);
    if libs.is_empty() {
        return Err(format!("{} is not built", libloading::library_filename(&name).to_string_lossy()));
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let path = out.join(format!("{name}.zip"));
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).map_err(|e| e.to_string())?);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut add = |rel: String, src: &Path| -> Result<(), String> {
        zip.start_file(rel, opts).map_err(|e| e.to_string())?;
        zip.write_all(&std::fs::read(src).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    for (file, lib) in &libs {
        add(file.clone(), lib)?;
    }
    let mut files = Vec::new();
    collect(&dir.join("assets"), &mut files);
    for f in files {
        let rel = f
            .strip_prefix(dir)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        add(rel, &f)?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(path)
}

/// `pack`, unless `<out>/<name>.zip` is already newer than the library and every asset. Returns
/// the archive and whether it was written.
pub fn pack_if_stale(dir: &Path, out: &Path) -> Result<(PathBuf, bool), String> {
    let name = dir
        .file_name()
        .ok_or("not a mod folder")?
        .to_string_lossy()
        .into_owned();
    let zip = out.join(format!("{name}.zip"));
    let mut inputs = Vec::new();
    collect(&dir.join("assets"), &mut inputs);
    inputs.extend(libraries(dir, &name).into_iter().map(|l| l.1));
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let newest = inputs.iter().filter_map(|p| modified(p)).max();
    if let (Some(z), Some(n)) = (modified(&zip), newest) {
        if z >= n {
            return Ok((zip, false));
        }
    }
    pack(dir, out).map(|p| (p, true))
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// Loads every library, then calls all `init`s in parallel and freezes the registries.
/// The `packages` mod's entry for registering one data package (chiacraft), see
/// `mods/packages`.
type RegisterPackage = unsafe extern "C" fn(*const abi::Host, ModId, *mut abi::Registrar, abi::Str) -> u32;
const REGISTER_PACKAGE: &[u8] = b"chiacraft_register_package\0";

/// Where the mods turned off are listed, ids one a line. Automated runs keep theirs apart.
pub fn off_file(run_dir: &Path, automated: bool) -> PathBuf {
    let dir = if automated { run_dir.join("test-config") } else { crate::paths::config(run_dir) };
    dir.join("disabled_mods.txt")
}

/// What turning mods on and off has come to, and what this run started with.
struct Switches {
    file: PathBuf,
    wanted: Vec<String>,
    at_start: Vec<String>,
}

static OFF: OnceLock<Vec<ModMeta>> = OnceLock::new();
static SWITCHES: parking_lot::Mutex<Option<Switches>> = parking_lot::Mutex::new(None);

/// Mods found but not running, for the mods screen: turned off, or needing one that is.
pub fn turned_off() -> &'static [ModMeta] {
    OFF.get().map_or(&[], |v| v.as_slice())
}

/// Whether a mod would run on the next start, as things are set now.
pub fn wanted_on(id: &str) -> bool {
    let wanted = SWITCHES
        .lock()
        .as_ref()
        .map_or(Vec::new(), |s| s.wanted.clone());
    let metas: Vec<&ModMeta> = content::try_content()
        .map_or(Vec::new(), |c| c.mods.iter().collect())
        .into_iter()
        .chain(turned_off())
        .collect();
    runs(id, &wanted, &|m| {
        metas
            .iter()
            .find(|o| o.id == m)
            .map_or(Vec::new(), |o| o.deps.clone())
    })
}

/// Not itself turned off, and neither is anything it needs, however far down.
fn runs(id: &str, off: &[String], deps: &dyn Fn(&str) -> Vec<String>) -> bool {
    !off.iter().any(|o| o == id) && deps(id).iter().all(|d| d != id && runs(d, off, deps))
}

/// Turns a mod on or off from the next start.
pub fn toggle(id: &str) {
    let mut guard = SWITCHES.lock();
    let Some(s) = guard.as_mut() else { return };
    match s.wanted.iter().position(|w| w == id) {
        Some(i) => drop(s.wanted.remove(i)),
        None => s.wanted.push(id.to_string()),
    }
    s.wanted.sort();
    if let Some(d) = s.file.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Err(e) = std::fs::write(&s.file, s.wanted.join("\n")) {
        log::error!("{}: {e}", s.file.display());
    }
}

/// Something was switched that a restart would bring in.
pub fn restart_pending() -> bool {
    SWITCHES
        .lock()
        .as_ref()
        .is_some_and(|s| s.wanted != s.at_start)
}

pub fn load(mod_dirs: &[PathBuf], package_dirs: &[PathBuf], cache: &Path, off_file: &Path) -> Result<Vec<LoadedMod>, String> {
    let start = Instant::now();
    let mut wanted: Vec<String> = std::fs::read_to_string(off_file)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    wanted.sort();
    *SWITCHES.lock() = Some(Switches { file: off_file.to_path_buf(), at_start: wanted.clone(), wanted: wanted.clone() });
    let mut off: Vec<ModMeta> = Vec::new();
    let shadow = cache.join("mods");
    std::fs::create_dir_all(&shadow).ok();
    if let Ok(entries) = std::fs::read_dir(&shadow) {
        for e in entries.flatten() {
            let _ = std::fs::remove_file(e.path());
        }
    }

    let mut mods = Vec::new();
    for (name, dir, lib_path) in discover(mod_dirs, cache) {
        // Windows locks loaded libraries; loading a copy keeps the original rebuildable.
        let copy = shadow.join(format!("{}-{}", std::process::id(), lib_path.file_name().unwrap().to_string_lossy()));
        let load_from = if std::fs::copy(&lib_path, &copy).is_ok() { copy } else { lib_path.clone() };
        let lib = match unsafe { libloading::Library::new(&load_from) } {
            Ok(lib) => lib,
            // A mod that won't load is a good reason to have turned it off.
            Err(_) if wanted.contains(&name) => {
                off.push(ModMeta {
                    id: name.clone(),
                    version: String::new(),
                    deps: Vec::new(),
                    order: 0,
                    assets: dir.join("assets"),
                    package: None,
                });
                continue;
            }
            Err(e) => return Err(format!("loading mod {name} from {}: {e}", lib_path.display())),
        };
        let entry: libloading::Symbol<unsafe extern "C" fn() -> *const abi::ModDecl> =
            unsafe { lib.get(abi::ENTRY_SYMBOL) }.map_err(|_| format!("{} is not a Weft mod", lib_path.display()))?;
        let decl = unsafe { &*entry() };
        if decl.abi_version != abi::ABI_VERSION {
            return Err(format!(
                "mod {name} was built against another mod ABI ({:016x}, this engine has {:016x}); rebuild it against the current weft_sdk",
                decl.abi_version,
                abi::ABI_VERSION
            ));
        }
        let id = unsafe { decl.id.as_str() }.to_string();
        if id != name {
            return Err(format!("mod {name} is the crate {id}: name its folder or zip {id}"));
        }
        mods.push(LoadedMod { id, assets: dir.join("assets"), _lib: lib, decl });
    }
    mods.sort_by(|a, b| a.id.cmp(&b.id));
    // Out go the ones turned off, then what needs them, and what needs that.
    let deps_of = |m: &LoadedMod| -> Vec<String> {
        unsafe {
            m.decl
                .deps
                .as_slice()
                .iter()
                .map(|d| d.as_str().to_string())
                .collect()
        }
    };
    loop {
        let Some(i) = mods
            .iter()
            .position(|m| wanted.contains(&m.id) || deps_of(m).iter().any(|d| off.iter().any(|o| &o.id == d)))
        else {
            break;
        };
        let m = mods.remove(i);
        if wanted.contains(&m.id) {
            log::info!("mod {} is turned off", m.id);
        } else {
            log::warn!("mod {} is off, as it needs one that is", m.id);
        }
        off.push(ModMeta {
            id: m.id.clone(),
            version: unsafe { m.decl.version.as_str() }.to_string(),
            deps: deps_of(&m),
            order: 0,
            assets: m.assets.clone(),
            package: None,
        });
    }
    off.sort_by(|a, b| a.id.cmp(&b.id));
    let _ = OFF.set(off);

    let register = mods.iter().find_map(|m| unsafe {
        m._lib
            .get::<RegisterPackage>(REGISTER_PACKAGE)
            .ok()
            .map(|f| *f)
    });
    // chiacraft's own mods get the engine's answers to what Weft's interface doesn't cover.
    for m in &mods {
        if let Ok(install) = unsafe { m._lib.get::<crate::bridge::Install>(crate::bridge::INSTALL) } {
            unsafe { install(crate::bridge::query) };
        }
    }
    let metas: Vec<ModMeta> = mods
        .iter()
        .map(|m| unsafe {
            ModMeta {
                id: m.id.clone(),
                version: m.decl.version.as_str().to_string(),
                deps: m
                    .decl
                    .deps
                    .as_slice()
                    .iter()
                    .map(|d| d.as_str().to_string())
                    .collect(),
                order: 0,
                assets: m.assets.clone(),
                package: None,
            }
        })
        .collect();
    let regs: Vec<RawRegistrar> = mods
        .par_iter()
        .enumerate()
        .map(|(i, m)| {
            let mut reg = RawRegistrar::new(ModId(i as u32));
            let t = Instant::now();
            let r = unsafe { (m.decl.init)(&crate::host::HOST, ModId(i as u32), &mut reg as *mut _ as *mut abi::Registrar) };
            if r != 0 {
                reg.errors.push("init panicked".into());
            }
            log::debug!("{} registered in {:.1} ms", m.id, t.elapsed().as_secs_f64() * 1000.0);
            reg
        })
        .collect();
    log::info!("{} mods registered in {:.1} ms", mods.len(), start.elapsed().as_secs_f64() * 1000.0);
    let store = chiacraft_packages::store::Store::new(cache);
    let installed = install(&store, package_dirs, &metas);
    *LOADER.lock() = Some(Loader { metas, regs, register, store, installed: installed.clone(), current: None, built: Vec::new() });
    use_packages(&installed)?;
    Ok(mods)
}

// ---------------------------------------------------------------- data packages (chiacraft)
//
// The code mods register once, at start; their registrations are kept. The content is then
// built (frozen) from them plus a set of data packages, and built again from them plus another
// set when a world needs one (`use_packages`). Code mods' content takes the first ids, so their
// ids are the same in every build. Each build is kept for the life of the game, once per set.

/// A data package ready to use: in the store, checked, under its hash.
#[derive(Clone, Debug)]
pub struct PackageRef {
    pub id: String,
    pub version: String,
    pub deps: Vec<String>,
    /// Its laid-out folder in the store.
    pub dir: PathBuf,
    pub hash: chiacraft_packages::identity::Hash,
}

struct Loader {
    /// The code mods' metas and registrations, in `ModId` order.
    metas: Vec<ModMeta>,
    regs: Vec<RawRegistrar>,
    register: Option<RegisterPackage>,
    store: chiacraft_packages::store::Store,
    /// The packages in the package folders: what a new world gets.
    installed: Vec<PackageRef>,
    /// The package hashes of the content in force.
    current: Option<Vec<String>>,
    /// Every content built: by its sorted package hashes.
    built: Vec<(Vec<String>, &'static content::Content)>,
}

static LOADER: parking_lot::Mutex<Option<Loader>> = parking_lot::Mutex::new(None);

/// Reads a package from the store (checking it again) as a `PackageRef`.
fn package_ref(store: &chiacraft_packages::store::Store, hash: chiacraft_packages::identity::Hash) -> Result<PackageRef, String> {
    let dir = store.verify(&hash)?;
    let m = chiacraft_packages::load(&dir)
        .map_err(|e| e.join("\n  "))?
        .manifest;
    Ok(PackageRef { id: m.id, version: m.version, deps: m.dependencies.keys().cloned().collect(), dir, hash })
}

/// The packages in `dirs`, each copied into the store and checked. A package is data someone else
/// wrote: one with mistakes is left out, not fatal.
fn install(store: &chiacraft_packages::store::Store, dirs: &[PathBuf], mods: &[ModMeta]) -> Vec<PackageRef> {
    let mut out: Vec<PackageRef> = Vec::new();
    for src in chiacraft_packages::discover(dirs) {
        let p = match store.import_dir(&src).and_then(|h| package_ref(store, h)) {
            Ok(p) => p,
            Err(e) => {
                log::error!("package {} left out:\n  {e}", src.display());
                continue;
            }
        };
        if mods.iter().any(|m| m.id == p.id) || out.iter().any(|o| o.id == p.id) {
            log::error!("package {} left out: the id {} is taken by a mod or another package", src.display(), p.id);
            continue;
        }
        match store
            .signature(&p.hash)
            .map(|sig| crate::package_trust::verify(&sig, &p.hash.hex()))
        {
            Some(Ok(c)) => log::info!("package {} is signed by content key {}", p.id, crate::package_trust::fingerprint(&c.key)),
            Some(Err(e)) => log::warn!("package {}: its signature is refused ({e}); players joining see it as unsigned", p.id),
            None => {}
        }
        out.push(p);
    }
    out
}

/// The package store (in `run/cache`).
pub fn store() -> Option<chiacraft_packages::store::Store> {
    LOADER
        .lock()
        .as_ref()
        .map(|l| chiacraft_packages::store::Store::new(l.store.root()))
}

/// A package in the store, checked again, as a `PackageRef`.
pub fn stored_package(hash: chiacraft_packages::identity::Hash) -> Result<PackageRef, String> {
    let store = store().ok_or("mods are not loaded")?;
    package_ref(&store, hash)
}

/// The data packages of the content in force: (id, version, hash, size in bytes, signature
/// envelope or empty).
pub fn current_packages() -> Vec<crate::net::p2p::Offer> {
    let Some(store) = store() else { return Vec::new() };
    content::content()
        .mods
        .iter()
        .filter_map(|m| {
            let hex = m.package.as_ref()?;
            let h = chiacraft_packages::identity::Hash::from_hex(hex)?;
            let manifest = store.manifest(&h).ok()?;
            let size = manifest.files.iter().map(|f| f.size).sum();
            // The API its code needs, read from the component's imports (unreadable code would
            // not have loaded here; "?" stands for it so the guest refuses rather than guesses).
            let api = match manifest
                .files
                .iter()
                .find(|f| f.path == chiacraft_packages::identity::SERVER_CODE)
            {
                None => String::new(),
                Some(f) => store
                    .blob_path(&f.hash)
                    .and_then(|p| std::fs::read(p).ok())
                    .and_then(|b| chiacraft_packages::code::api_of(&b).ok())
                    .map_or_else(|| "?".into(), |v| chiacraft_packages::code::text(Some(v.unwrap_or((1, 0))))),
            };
            Some((m.id.clone(), m.version.clone(), hex.clone(), size, store.signature(&h).unwrap_or_default(), api))
        })
        .collect()
}

/// The packages in the package folders.
pub fn installed() -> Vec<PackageRef> {
    LOADER
        .lock()
        .as_ref()
        .map_or_else(Vec::new, |l| l.installed.clone())
}

/// The packages a world plays with: the exact version it was played with (from the store), and
/// installed packages it has not met yet. Also, as "id version", what it needs and the store
/// lacks.
pub fn for_world(meta: &crate::saves::WorldMeta) -> (Vec<PackageRef>, Vec<String>) {
    let guard = LOADER.lock();
    let Some(l) = guard.as_ref() else { return (Vec::new(), Vec::new()) };
    let mut set = Vec::new();
    let mut missing = Vec::new();
    for (id, hex) in &meta.packages {
        match chiacraft_packages::identity::Hash::from_hex(hex)
            .ok_or_else(|| "a bad hash".to_string())
            .and_then(|h| package_ref(&l.store, h))
        {
            Ok(p) => set.push(p),
            Err(_) => match l.installed.iter().find(|p| &p.id == id) {
                Some(p) => missing.push(format!(
                    "{id}: another version (the world's {}, this one {})",
                    hex.get(..12).unwrap_or(hex),
                    p.hash.short()
                )),
                None => {
                    let version = meta
                        .mods
                        .iter()
                        .find(|(m, _)| m == id)
                        .map_or("", |(_, v)| v.as_str());
                    missing.push(format!("{id} {version}").trim_end().to_string());
                }
            },
        }
    }
    for p in &l.installed {
        if !meta.packages.iter().any(|(id, _)| id == &p.id) {
            set.push(p.clone());
        }
    }
    (set, missing)
}

/// Makes the content the code mods' plus exactly these packages, building it the first time this
/// set is asked for. True if the content changed. This is the game's content: a world still
/// running keeps the content it was opened with (`server::Scope`).
pub fn use_packages(set: &[PackageRef]) -> Result<bool, String> {
    let start = Instant::now();
    let mut guard = LOADER.lock();
    let l = guard.as_mut().ok_or("mods are not loaded")?;
    let mut key: Vec<String> = set.iter().map(|p| p.hash.hex()).collect();
    key.sort();
    if l.current.as_ref() == Some(&key) {
        return Ok(false);
    }
    let content = match l.built.iter().find(|(k, _)| *k == key) {
        Some((_, c)) => *c,
        None => {
            let c: &'static content::Content = Box::leak(Box::new(build(l, set)?));
            l.built.push((key.clone(), c));
            log::info!(
                "content: {} mods and packages, {} blocks ({} states), {} items, {} dimensions in {:.1} ms",
                c.mods.len(),
                c.blocks.len(),
                c.states.len(),
                c.items.len(),
                c.dims.len(),
                start.elapsed().as_secs_f64() * 1000.0
            );
            // Load order is part of the game's behaviour (later mods win on conflicts).
            let mut order: Vec<&ModMeta> = c.mods.iter().collect();
            order.sort_by_key(|m| m.order);
            for m in order {
                match &m.package {
                    Some(h) => log::info!("loaded package {} {} ({}) (load position {})", m.id, m.version, &h[..12], m.order),
                    None => log::info!("loaded mod {} {} (load position {})", m.id, m.version, m.order),
                }
            }
            c
        }
    };
    set_names(content.mods.iter().map(|m| m.id.clone()).collect());
    content::set(content);
    l.current = Some(key);
    for p in set {
        l.store.touch(&p.hash);
    }
    Ok(true)
}

/// Makes `c`, a content built earlier, the game's again (going back to a world kept running).
/// True if the content changed.
pub fn use_built(c: &'static content::Content) -> bool {
    let mut guard = LOADER.lock();
    let Some(l) = guard.as_mut() else { return false };
    let Some((key, _)) = l.built.iter().find(|(_, b)| std::ptr::eq(*b, c)) else { return false };
    if l.current.as_ref() == Some(key) {
        return false;
    }
    l.current = Some(key.clone());
    set_names(c.mods.iter().map(|m| m.id.clone()).collect());
    content::set(c);
    true
}

/// Packages no world here pins are kept up to this much, those used most recently first: the
/// worlds joined lately, which are likely joined again.
const SPARE_PACKAGES: u64 = 512 << 20;

/// Tidies the package store: keeps the installed packages and every package a world in `saves`
/// pins, the most recently used others within `SPARE_PACKAGES`, and removes the rest.
pub fn tidy_store(saves: &Path) {
    let Some(store) = store() else { return };
    let mut keep: Vec<chiacraft_packages::identity::Hash> = installed().iter().map(|p| p.hash).collect();
    for w in crate::saves::list(saves) {
        keep.extend(
            w.packages
                .iter()
                .filter_map(|(_, hex)| chiacraft_packages::identity::Hash::from_hex(hex)),
        );
    }
    let done = store.tidy(&keep, SPARE_PACKAGES);
    if done != Default::default() {
        log::info!("package store: removed {} packages and {} files ({} KiB)", done.packages, done.files, done.bytes >> 10);
    }
}

fn build(l: &Loader, set: &[PackageRef]) -> Result<content::Content, String> {
    let mut metas = l.metas.clone();
    metas.extend(set.iter().map(|p| ModMeta {
        id: p.id.clone(),
        version: p.version.clone(),
        deps: p.deps.clone(),
        order: 0,
        assets: p.dir.join("assets"),
        package: Some(p.hash.hex()),
    }));
    for m in &metas {
        for d in &m.deps {
            if !metas.iter().any(|o| &o.id == d) {
                return Err(format!("mod {} requires missing mod {d}", m.id));
            }
        }
    }
    // Names of the slots for registration messages, before the content is in force.
    set_names(metas.iter().map(|m| m.id.clone()).collect());
    let mut regs = l.regs.clone();
    for (i, p) in set.iter().enumerate() {
        let id = ModId((l.metas.len() + i) as u32);
        let register = l.register.ok_or("data packages need the packages mod")?;
        let mut reg = RawRegistrar::new(id);
        let path = p.dir.to_string_lossy();
        let r = unsafe { register(&crate::host::HOST, id, &mut reg as *mut _ as *mut abi::Registrar, abi::Str::new(&path)) };
        if r != 0 {
            reg.errors.push("registration panicked".into());
        }
        regs.push(reg);
    }
    content::freeze(metas, regs).map_err(|e| e.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mod_needs_what_it_depends_on_running() {
        let deps = |m: &str| -> Vec<String> {
            match m {
                "top" => vec!["middle".to_string()],
                "middle" => vec!["base".to_string()],
                _ => Vec::new(),
            }
        };
        let off = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(runs("top", &off(&[]), &deps));
        assert!(!runs("top", &off(&["base"]), &deps), "all the way down");
        assert!(runs("base", &off(&["top"]), &deps), "not the other way");
    }

    /// A player's copy of a mod that also came with the game: theirs is the one, still unpacked.
    #[test]
    fn a_mod_in_two_folders_is_the_first_ones() {
        let tmp = std::env::temp_dir().join(format!("weft-twice-{}", std::process::id()));
        let lib = libloading::library_filename("mymod");
        let mut zips = Vec::new();
        for (dir, text) in [("mine", "mine"), ("shipped", "shipped, a longer one")] {
            let src = tmp.join(dir).join("mymod");
            std::fs::create_dir_all(&src).unwrap();
            std::fs::write(src.join(&lib), text).unwrap();
            zips.push(
                pack(&src, &tmp.join(format!("{dir}-mods")))
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_path_buf(),
            );
        }
        let found = discover(&zips, &tmp.join("cache"));
        assert_eq!(found.len(), 1);
        assert_eq!(std::fs::read_to_string(&found[0].2).unwrap(), "mine");
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    /// One zip carries a library per system, and each system finds its own in it.
    #[test]
    fn a_zip_serves_every_system() {
        let tmp = std::env::temp_dir().join(format!("weft-pack-{}", std::process::id()));
        let dir = tmp.join("src").join("mymod");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("assets").join("a.txt"), "a").unwrap();
        for lib in ["mymod.dll", "libmymod.so", "libmymod.dylib"] {
            std::fs::write(dir.join(lib), lib).unwrap();
        }
        let mods = tmp.join("mods");
        let zip = pack(&dir, &mods).unwrap();
        let archive = zip::ZipArchive::new(std::fs::File::open(&zip).unwrap()).unwrap();
        let mut names: Vec<&str> = archive.file_names().collect();
        names.sort();
        assert_eq!(names, ["assets/a.txt", "libmymod.dylib", "libmymod.so", "mymod.dll"]);
        let found = discover(&[mods], &tmp.join("cache"));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].2.file_name().unwrap(), libloading::library_filename("mymod"));
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}

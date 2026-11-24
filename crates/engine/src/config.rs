//! Settings the engine (namespace `core`) and mods declare: typed, ranged values read with one
//! atomic load (text ones under a lock), edited on screens built from the declarations. The scope
//! says where a value lives: `CLIENT` in each player's `config/<ns>.txt`; `SERVER` in the
//! server's file, sent to its players whenever it changes; `WORLD` saved with the world (game
//! rules, `/rule`) and sent the same way. The engine's own settings and key bindings are `core:`
//! ones like any other, in `config/core.txt`.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};

use weft_abi::{config_kind as ck, config_scope as scope, entry_kind as ek};

use crate::content::content;
use crate::entries::{Entry, RawEntry};

#[derive(Clone, Debug)]
pub struct ConfigDef {
    pub name: String,
    pub kind: u32,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub choices: Vec<String>,
    pub flags: u32,
    pub logarithmic: bool,
    pub scope: u32,
    /// The default of a `TEXT` setting.
    pub text: String,
    /// The settings page it shows on, empty for its mod's; and its place there.
    pub page: String,
    pub order: i32,
}

impl ConfigDef {
    /// The server's value counts for its players.
    pub fn synced(&self) -> bool {
        self.scope != scope::CLIENT
    }

    /// `v` made valid for this setting.
    pub fn fit(&self, v: f64) -> f64 {
        if !v.is_finite() {
            return self.default;
        }
        match self.kind {
            ck::BOOL => (v != 0.0) as u8 as f64,
            ck::CHOICE => v
                .round()
                .clamp(0.0, self.choices.len().saturating_sub(1) as f64),
            ck::TEXT => 0.0,
            _ => {
                let mut v = v.clamp(self.min.min(self.max), self.max.max(self.min));
                if self.step > 0.0 {
                    v = (self.min + ((v - self.min) / self.step).round() * self.step).clamp(self.min.min(self.max), self.max.max(self.min));
                }
                if self.kind == ck::INT { v.round() } else { v }
            }
        }
    }

    /// How many steps a slider of it goes through: its own, or `unit` (what one shown digit
    /// is worth) where it has none or a finer one.
    pub fn steps(&self, unit: f64) -> u32 {
        ((self.max - self.min).abs() / self.step.max(unit))
            .round()
            .clamp(1.0, u32::MAX as f64) as u32
    }

    pub fn slider_step(&self) -> f64 {
        if self.step > 0.0 {
            self.step
        } else if self.kind == ck::INT {
            1.0
        } else {
            ((self.max - self.min).abs() / 100.0).max(f64::EPSILON)
        }
    }

    pub fn slider_value(&self, fraction: f32) -> f64 {
        let range = self.max - self.min;
        let value = if self.logarithmic {
            let unit = self.slider_step();
            self.min + unit * ((1.0 + range / unit).ln() * fraction as f64).exp_m1()
        } else {
            self.min + fraction as f64 * range
        };
        self.fit(value)
    }

    pub fn slider_position(&self, value: f64) -> f32 {
        let range = (self.max - self.min).max(1e-9);
        if self.logarithmic {
            let unit = self.slider_step();
            (((value - self.min).max(0.0) / unit).ln_1p() / (range / unit).ln_1p()) as f32
        } else {
            ((value - self.min) / range) as f32
        }
    }

    fn parse(&self, s: &str) -> Option<f64> {
        let s = s.trim();
        match self.kind {
            ck::BOOL => match s {
                "true" | "on" | "1" => Some(1.0),
                "false" | "off" | "0" => Some(0.0),
                _ => None,
            },
            ck::CHOICE => self.choices.iter().position(|c| c == s).map(|i| i as f64),
            _ => s.parse().ok(),
        }
    }

    /// A value typed for it (`/rule`): what `parse` takes, and for numbers only those within
    /// its range, so a typo is refused rather than clamped into a different value.
    pub fn read(&self, s: &str) -> Option<f64> {
        let v = self.parse(s)?;
        let in_range = matches!(self.kind, ck::BOOL | ck::CHOICE) || (v >= self.min.min(self.max) && v <= self.max.max(self.min));
        in_range.then_some(v)
    }

    pub fn show(&self, v: f64) -> String {
        match self.kind {
            ck::BOOL => (v != 0.0).to_string(),
            ck::CHOICE => self.choices.get(v as usize).cloned().unwrap_or_default(),
            ck::INT => format!("{}", v as i64),
            _ => {
                let s = format!("{v:.3}");
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            }
        }
    }
}

/// Value bits by config id, for the content in force. Ids follow each content's settings in
/// name order, so a content switch that adds or drops a setting (a package with a key) moves
/// the ids after it: [`content_switched`] then carries every value over by name to a table for
/// the new content. A table replaced so is kept (a few hundred values per switch, a handful of
/// switches a session), so a slice someone holds stays valid.
static VALUES: AtomicPtr<Vec<AtomicU64>> = AtomicPtr::new(std::ptr::null_mut());
/// Values of text settings by config id (empty for the others).
static TEXTS: RwLock<Vec<String>> = RwLock::new(Vec::new());
/// Counts changes, so whoever applies settings (the client) sees that something moved.
static GENERATION: AtomicU64 = AtomicU64::new(0);
static RUN_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
/// Automated runs change settings for themselves only: they read the player's files and write
/// to `test-config/` beside them.
static AUTOMATED: AtomicBool = AtomicBool::new(false);
/// Values given on the command line for this run: (id, value in the file). Saving keeps the
/// file's value unless the setting was changed since.
static OVERRIDES: Mutex<Vec<(u32, String, String)>> = Mutex::new(Vec::new());
/// Values of the remote server this player is on, kept the same way until leaving it.
static FROM_SERVER: Mutex<Vec<(u32, String, String)>> = Mutex::new(Vec::new());

fn values() -> &'static [AtomicU64] {
    let p = VALUES.load(Ordering::Acquire);
    if !p.is_null() {
        // SAFETY: only Box::leak'd tables are stored, and never freed.
        return unsafe { &*p };
    }
    let fresh: &'static mut Vec<AtomicU64> = Box::leak(Box::new(
        content()
            .game
            .configs
            .iter()
            .map(|d| AtomicU64::new(d.default.to_bits()))
            .collect(),
    ));
    match VALUES.compare_exchange(std::ptr::null_mut(), fresh, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => fresh,
        // Another thread made one first: use that (this one stays unused).
        // SAFETY: as above.
        Err(other) => unsafe { &*other },
    }
}

/// The values of settings `old` (with `old_bits` and `old_texts` by its ids) laid out for
/// settings `new`: each by its name, a setting `new` adds at its default, one it drops gone.
fn rekeyed(old: &[ConfigDef], old_bits: &[u64], old_texts: &[String], new: &[ConfigDef]) -> (Vec<u64>, Vec<String>) {
    let by_name: std::collections::HashMap<&str, usize> = old
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name.as_str(), i))
        .collect();
    new.iter()
        .map(|d| match by_name.get(d.name.as_str()) {
            Some(&i) => {
                (old_bits.get(i).copied().unwrap_or(d.default.to_bits()), old_texts.get(i).cloned().unwrap_or_else(|| d.text.clone()))
            }
            None => (d.default.to_bits(), d.text.clone()),
        })
        .unzip()
}

/// The content in force switched from `old` to `new` (`content::set`): the game's values follow
/// their settings by name, so a join into a world whose packages add a setting reads every
/// other setting (the player's name, the keys) where it was (chiacraft, phase 6).
pub(crate) fn content_switched(old: &crate::content::Content, new: &crate::content::Content) {
    let (old_defs, new_defs) = (&old.game.configs, &new.game.configs);
    // Mods' handles read the ids of the content in force: a content built earlier comes back
    // with its own (building another wrote theirs into the same cells).
    for &(cell, id) in &new.game.config_cells {
        crate::registry::write_cell(cell, id);
    }
    if old_defs.len() == new_defs.len() && old_defs.iter().zip(new_defs).all(|(a, b)| a.name == b.name) {
        return;
    }
    if VALUES.load(Ordering::Acquire).is_null() {
        // Nothing read yet: the first read lays the table out for the content then in force.
        return;
    }
    let old_bits: Vec<u64> = values().iter().map(|v| v.load(Ordering::Relaxed)).collect();
    let mut texts = TEXTS.write().unwrap();
    let (bits, new_texts) = rekeyed(old_defs, &old_bits, &texts, new_defs);
    let table: &'static mut Vec<AtomicU64> = Box::leak(Box::new(bits.into_iter().map(AtomicU64::new).collect()));
    VALUES.store(table, Ordering::Release);
    *texts = new_texts;
    drop(texts);
    // What this run and a remote server set are kept by id: follow the names there too.
    let move_id = |id: u32| {
        let name = &old_defs.get(id as usize)?.name;
        new.game.catalog.find(ek::CONFIG, name)
    };
    for list in [&OVERRIDES, &FROM_SERVER] {
        let mut l = list.lock().unwrap();
        *l = l
            .drain(..)
            .filter_map(|(id, now, saved)| Some((move_id(id)?, now, saved)))
            .collect();
    }
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// A server's own values of the settings it decides (chiacraft). The values above are the game's:
/// its player's settings and, while playing, the server's synced ones. A world kept running while
/// its player visits another must not take that world's rules, so each server keeps its synced
/// values here, read and written on its threads ([`for_thread`]).
pub struct ServerValues(Vec<AtomicU64>);

thread_local! {
    static SERVER: std::cell::RefCell<Option<std::sync::Arc<ServerValues>>> = const { std::cell::RefCell::new(None) };
}

/// The game's values as they are now, for a server to keep as its own.
pub fn server_values() -> std::sync::Arc<ServerValues> {
    std::sync::Arc::new(ServerValues(
        values()
            .iter()
            .map(|v| AtomicU64::new(v.load(Ordering::Relaxed)))
            .collect(),
    ))
}

/// The values of the server this game's player is playing in, when it runs in this process:
/// the synced settings the game changes (a mod, the settings screen) go to it too, or the host's
/// changes would never reach the world or its other players. None while on a remote server or
/// visiting another world (a visited world's rules must not reach home).
static PLAYING: Mutex<Option<std::sync::Arc<ServerValues>>> = Mutex::new(None);

/// The player is now in the world whose server keeps `v` (None: no world run here).
pub fn playing_in(v: Option<std::sync::Arc<ServerValues>>) {
    *PLAYING.lock().unwrap() = v;
}

/// This thread reads and writes the synced settings in `v` from now on (`None`: the game's).
pub fn for_thread(v: Option<std::sync::Arc<ServerValues>>) {
    SERVER.with(|s| *s.borrow_mut() = v);
}

/// This thread's server values, if it is a server's.
pub fn thread_values() -> Option<std::sync::Arc<ServerValues>> {
    SERVER.with(|s| s.borrow().clone())
}

/// Runs `f` with `v` as this thread's server values, then puts back what was there.
pub fn with<R>(v: std::sync::Arc<ServerValues>, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<std::sync::Arc<ServerValues>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SERVER.with(|s| *s.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(SERVER.with(|s| s.borrow_mut().replace(v)));
    f()
}

/// The server's slot for setting `id` if this is a server's thread and the server decides it.
fn server_slot<R>(id: u32, f: impl FnOnce(&AtomicU64) -> R) -> Option<R> {
    SERVER.with(|s| {
        let s = s.borrow();
        let v = s.as_ref()?;
        if !def(id)?.synced() {
            return None;
        }
        v.0.get(id as usize).map(f)
    })
}

fn def(id: u32) -> Option<&'static ConfigDef> {
    content().game.configs.get(id as usize)
}

pub fn get(id: u32) -> f64 {
    if let Some(v) = server_slot(id, |s| f64::from_bits(s.load(Ordering::Relaxed))) {
        return v;
    }
    values()
        .get(id as usize)
        .map_or(0.0, |v| f64::from_bits(v.load(Ordering::Relaxed)))
}

pub fn text(id: u32) -> String {
    TEXTS
        .read()
        .unwrap()
        .get(id as usize)
        .cloned()
        .unwrap_or_default()
}

/// Changes so far; compare with an earlier value to see whether anything was set.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// Sets a value without saving (server values arriving, loading files).
pub fn put(id: u32, v: f64) {
    if let Some(d) = def(id)
        && server_slot(id, |s| s.store(d.fit(v).to_bits(), Ordering::Relaxed)).is_some()
    {
        return;
    }
    if let (Some(slot), Some(d)) = (values().get(id as usize), def(id)) {
        let bits = d.fit(v).to_bits();
        if slot.swap(bits, Ordering::Relaxed) != bits {
            GENERATION.fetch_add(1, Ordering::Relaxed);
        }
        if d.synced()
            && let Some(server) = PLAYING.lock().unwrap().as_ref()
            && let Some(s) = server.0.get(id as usize)
        {
            s.store(bits, Ordering::Relaxed);
        }
    }
}

pub fn put_text(id: u32, v: &str) {
    let mut t = TEXTS.write().unwrap();
    if let Some(slot) = t.get_mut(id as usize).filter(|s| s.as_str() != v) {
        *slot = v.to_string();
        GENERATION.fetch_add(1, Ordering::Relaxed);
    }
}

/// Sets a value and saves it where its scope keeps it (a world's are saved with the world).
pub fn set(id: u32, v: f64) {
    if held_for_run(id) {
        return;
    }
    put(id, v);
    save_of(id);
}

pub fn set_text(id: u32, v: &str) {
    if held_for_run(id) {
        return;
    }
    put_text(id, v);
    save_of(id);
}

/// Whether `id` was set for this run (`override_for_run`): it keeps that value however it is
/// changed after. The test kit put back vsync, a view of 12 and no shader pack when each test
/// began, so every lag bench (11-04) ran with vsync on, `--view 8` at 12 and without the pack,
/// whatever its `--no-vsync`, `--view` or `--config` said.
pub(crate) fn held_for_run(id: u32) -> bool {
    let held = OVERRIDES.lock().unwrap().iter().any(|o| o.0 == id);
    if held {
        log::debug!("config {}: kept as set for this run", def(id).map_or("?", |d| d.name.as_str()));
    }
    held
}

fn save_of(id: u32) {
    let (Some(dir), Some(d)) = (RUN_DIR.get(), def(id)) else { return };
    if let (Some((ns, _)), false) = (d.name.split_once(':'), d.scope == scope::WORLD) {
        save(dir, ns);
    }
}

pub fn by_name(name: &str) -> Option<u32> {
    content().game.catalog.find(ek::CONFIG, name)
}

pub fn automated() -> bool {
    AUTOMATED.load(Ordering::Relaxed)
}

pub fn set_automated(on: bool) {
    AUTOMATED.store(on, Ordering::Relaxed);
}

/// A value for this run only (`--view`, `--no-vsync`, `--config`), as the file writes it; it
/// holds for the run (`held_for_run`).
pub fn override_for_run(name: &str, value: &str) {
    let Some(id) = by_name(name) else { return };
    let saved = shown(id);
    put_value(id, value);
    let now = shown(id);
    OVERRIDES.lock().unwrap().push((id, now, saved));
}

/// A value the server decides, in force while connected and not written to this player's file.
pub fn put_from_server(id: u32, v: f64) {
    let saved = shown(id);
    put(id, v);
    let now = shown(id);
    if now != saved {
        let mut server = FROM_SERVER.lock().unwrap();
        // A later value from the server: what the file should keep is still the first one's.
        let saved = server
            .iter()
            .position(|o| o.0 == id)
            .map_or(saved, |i| server.remove(i).2);
        server.push((id, now, saved));
    }
}

/// Off a remote server: what it decided is over, the player's own settings are back.
pub fn left_server(run_dir: &Path) {
    FROM_SERVER.lock().unwrap().clear();
    load(run_dir);
}

/// A value as its file writes it.
fn shown(id: u32) -> String {
    match def(id) {
        Some(d) if d.kind == ck::TEXT => text(id),
        Some(d) => d.show(get(id)),
        None => String::new(),
    }
}

/// Puts a value written as text; false if it does not fit the setting.
fn put_value(id: u32, v: &str) -> bool {
    let Some(d) = def(id) else { return false };
    if d.kind == ck::TEXT {
        put_text(id, v.trim());
        return true;
    }
    match d.parse(v) {
        Some(n) => {
            put(id, n);
            true
        }
        None => false,
    }
}

/// Every setting back to its default, then each namespace's `config/<ns>.txt` (not the world's);
/// missing lines keep their defaults, unknown or bad ones are reported.
pub fn load(run_dir: &Path) {
    let _ = RUN_DIR.set(run_dir.to_path_buf());
    let c = content();
    {
        let mut t = TEXTS.write().unwrap();
        *t = c.game.configs.iter().map(|d| d.text.clone()).collect();
    }
    for (id, d) in c.game.configs.iter().enumerate() {
        put(id as u32, d.default);
    }
    let mut namespaces: Vec<&str> = c
        .game
        .configs
        .iter()
        .filter_map(|d| d.name.split_once(':').map(|x| x.0))
        .collect();
    namespaces.dedup();
    for ns in namespaces {
        let Ok(src) = std::fs::read_to_string(file(run_dir, false, ns)) else { continue };
        for (_, k, v) in crate::kv::pairs(&src) {
            let name = format!("{ns}:{k}");
            match by_name(&name).filter(|&id| c.game.configs[id as usize].scope != scope::WORLD) {
                Some(id) => {
                    if !put_value(id, v) {
                        let d = &c.game.configs[id as usize];
                        log::warn!("config {name}: cannot use '{}', keeping {}", v.trim(), d.show(d.default));
                    }
                }
                None => log::warn!("config/{ns}.txt: no setting {}", k.trim()),
            }
        }
    }
    for (id, value, _) in OVERRIDES.lock().unwrap().iter() {
        put_value(*id, value);
    }
}

fn file(run_dir: &Path, automated: bool, ns: &str) -> std::path::PathBuf {
    let dir = if automated { run_dir.join("test-config") } else { crate::paths::config(run_dir) };
    dir.join(format!("{ns}.txt"))
}

/// Writes a namespace's settings, every one of them so players see what exists.
pub fn save(run_dir: &Path, ns: &str) {
    let c = content();
    let (overrides, server) = (OVERRIDES.lock().unwrap().clone(), FROM_SERVER.lock().unwrap().clone());
    let mut out = String::new();
    for (id, d) in c
        .game
        .configs
        .iter()
        .enumerate()
        .filter(|(_, d)| d.scope != scope::WORLD)
    {
        let Some(key) = d.name.strip_prefix(ns).and_then(|k| k.strip_prefix(':')) else { continue };
        let kept = |list: &[(u32, String, String)], now: String| {
            list.iter()
                .find(|o| o.0 == id as u32 && o.1 == now)
                .map_or(now.clone(), |o| o.2.clone())
        };
        let value = kept(&overrides, kept(&server, shown(id as u32)));
        out.push_str(&format!("{key} = {value}\n"));
    }
    let path = file(run_dir, AUTOMATED.load(Ordering::Relaxed), ns);
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    if let Err(e) = std::fs::write(&path, out) {
        log::error!("saving {}: {e}", path.display());
    }
}

/// Values of the settings the server decides, for its players.
pub fn synced() -> Vec<(u32, f64)> {
    content()
        .game
        .configs
        .iter()
        .enumerate()
        .filter(|(_, d)| d.synced() && d.kind != ck::TEXT)
        .map(|(i, _)| (i as u32, get(i as u32)))
        .collect()
}

/// A world's settings as it saves them.
pub fn world_values() -> Vec<(String, f64)> {
    content()
        .game
        .configs
        .iter()
        .enumerate()
        .filter(|(_, d)| d.scope == scope::WORLD)
        .map(|(i, d)| (d.name.clone(), get(i as u32)))
        .collect()
}

/// Opening a world: its settings as it saved them, defaults for the rest.
pub fn load_world(saved: &[(String, f64)]) {
    for (id, d) in content()
        .game
        .configs
        .iter()
        .enumerate()
        .filter(|(_, d)| d.scope == scope::WORLD)
    {
        put(
            id as u32,
            saved
                .iter()
                .find(|s| s.0 == d.name)
                .map_or(d.default, |s| s.1),
        );
    }
    // A world rule set for the run (`--config core:mob_spawning=0`) holds in the worlds it opens
    // too: the world's own value came after it, and the hold (`held_for_run`) then refused the
    // script's `/rule core:mob_spawning 0`, so monsters spawned in every look on the PC (11-04).
    for (id, value, _) in OVERRIDES.lock().unwrap().iter() {
        if def(*id).is_some_and(|d| d.scope == scope::WORLD) {
            put_value(*id, value);
        }
    }
}

/// Settings the engine itself reads: the client's options, world rules, and
/// a binding per key (`core:key.<action>`, from `keys`: action and default key); each sound
/// category adds its volume (`volume`).
pub fn engine_configs(keys: &[(String, String)]) -> Vec<RawEntry> {
    use weft_abi::comp::config as c;
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Player".into());
    // A phone's GPU spends most of its time on distant terrain's triangles and on pixels.
    let phone = cfg!(target_os = "android");
    let mut out = vec![
        volume("core:volume_master").set(c::ORDER, -1),
        string("core:language", &crate::settings::system_language()),
        number("core:gui_scale", ck::INT, 0.0, 0.0, 16.0, 1.0),
        // The world drawn at this part of the screen's size and stretched over it; the interface
        // stays sharp. What a phone's GPU needs, most of its time going to pixels.
        number("core:render_scale", ck::FLOAT, if phone { 0.6 } else { 1.0 }, 0.25, 1.0, 0.05),
        number("core:view_distance", ck::INT, 12.0, 2.0, 64.0, 1.0),
        // Distant terrain beyond the view (`client::lod`), in cubes; 0 off.
        number("core:lod_distance", ck::INT, if phone { 32.0 } else { 256.0 }, 0.0, 1_048_576.0, 16.0).tag("config.logarithmic"),
        // Quality maps to a target cell size of 24 / quality render pixels.
        number("core:lod_detail", ck::FLOAT, if phone { 2.0 } else { 4.0 }, 1.0, 4.0, 0.5),
        // chiacraft: the far ground's look for shader packs (`Globals.extra.w`'s top byte), bits:
        // 1 small plants thin out, 2 block-scale shading softens, 4 textures give way to their
        // average colour, all with distance (Ronald, 11-06: all three).
        number("core:far_look", ck::INT, 7.0, 0.0, 255.0, 1.0),
        // How a world drawn smaller than the window (`core:render_scale`) is scaled up: 0 bilinear,
        // 1 FSR 1's edge-adaptive upscale (`easu` in common.wgsl), 2 Snapdragon GSR 1 (`gsr`). Live.
        number("core:upscaler", ck::INT, 0.0, 0.0, 2.0, 1.0),
        // The GPU, as the renderer found it (`Backend::device_*`), for mods to read: put at start,
        // never saved by a player (the base game's first quality preset, Research round 19, 7).
        string("core:gpu_name", ""),
        number("core:gpu_class", ck::INT, 0.0, 0.0, 3.0, 1.0),
        number("core:gpu_memory_mb", ck::INT, 0.0, 0.0, 1.0e7, 1.0),
        // 1 when the GPU can trace rays (`Backend::ray_query`): `core:rt_shadows` then works.
        number("core:gpu_ray_query", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // The renderer's upscaler (`Backend::upscaler`), put at start like the GPU's: its name, its
        // modes and models as bits (`UpscalerInfo`), Ray Reconstruction, DLSS 5, frame generation.
        string("core:gpu_dlss", ""),
        number("core:gpu_dlss_modes", ck::INT, 0.0, 0.0, 1.0e9, 1.0),
        number("core:gpu_dlss_models", ck::INT, 0.0, 0.0, 4.3e9, 1.0),
        number("core:gpu_dlss_rr", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:gpu_dlss_rr_models", ck::INT, 0.0, 0.0, 4.3e9, 1.0),
        number("core:gpu_dlss5", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:gpu_frame_gen", ck::INT, 0.0, 0.0, 8.0, 1.0),
        // Blocks clouds float above where their dimension puts them.
        number("core:cloud_height", ck::INT, 0.0, 0.0, 512.0, 16.0),
        number("core:cloud_distance", ck::INT, if phone { 2048.0 } else { 6144.0 }, 0.0, 32768.0, 16.0).tag("config.logarithmic"),
        number("core:cloud_impostor_distance", ck::INT, if phone { 1024.0 } else { 2048.0 }, 256.0, 16384.0, 16.0)
            .tag("config.logarithmic"),
        // Sun shadows, for shader packs that draw them: 0 off, 1 the near map only (80 blocks
        // about the player), 2 near and far (400). The maps' size in texels: 1024, 2048 or 4096.
        number("core:shadows", ck::INT, 2.0, 0.0, 2.0, 1.0),
        number("core:shadow_size", ck::INT, 2048.0, 1024.0, 4096.0, 1024.0),
        // The far shadow map's reach to each side (blocks): 0 auto, 2000 with the HDR world (at no
        // measurable cost, more far hills and crowns shadowed: Far view, 11-05) and 400 without
        // (today's look as made); a number sets it (`client::prepare_shadows`).
        number("core:shadow_far", ck::INT, 0.0, 0.0, 2400.0, 100.0),
        // Sun and moon shadows traced through the scene instead of the shadow maps, on a GPU with
        // ray queries (Vulkan's; not the Mac or phones): ultra, off unless chosen.
        // 2: their soft edges from more rays a pixel (with TAA; about three times the cost).
        number("core:rt_shadows", ck::INT, 0.0, 0.0, 2.0, 1.0),
        // Reflections traced through the scene on smooth surfaces (water, glass, polished stone,
        // metals: their LabPBR maps), in place of the screen's; same GPUs.
        number("core:rt_reflections", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // Path traced lighting over the traced scene (reports/Path tracing plan.md; M0: the sun and
        // one bounce on the terrain, with a shader pack that has `pathtrace.wgsl`); its debug views
        // (0 the picture, 1 albedo, 2 normals, 3 direct light, 4 the bounce, 5 the specular light, 6 the radiance cache, 7 the picture with a line where tracing ends).
        number("core:path_traced", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:pt_debug", ck::INT, 0.0, 0.0, 7.0, 1.0),
        // How the path tracer's light is denoised (`denoise::Denoiser`) where Ray Reconstruction
        // doesn't do it: 0 not at all (raw, for checks), 1 ours (SVGF-like), live.
        number("core:pt_denoiser", ck::INT, 1.0, 0.0, 1.0, 1.0),
        // Its tuning (checks only, no menu row): how far light may differ between joined texels
        // (standard deviations) and its à-trous steps, live.
        number("core:pt_dn_phi", ck::FLOAT, 1.0, 0.25, 16.0, 0.25),
        number("core:pt_dn_steps", ck::INT, 4.0, 1.0, 5.0, 1.0),
        // Its debug views (checks only): 0 none, 1-6 (`denoise.wgsl` `dn_debug_view`).
        number("core:pt_dn_debug", ck::INT, 0.0, 0.0, 6.0, 1.0),
        // chiacraft: our soft clouds (`client::soft_clouds`), on unless a look test turns them off
        // (stage 1/2 like-for-like shots; kept until the A/B against upstream's clouds).
        number("core:soft_clouds", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // chiacraft, test only (the Minecraft comparison): leaves as Minecraft's fancy leaves, the
        // faces between them drawn and shaded as its smooth light does (`client::mesher`), their
        // smaller mips darkened as its are (`assets::dark_leaf_mips`).
        number("core:mc_leaves", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // The soft clouds' shadow on the land (with the HDR world; `client::cloud_shadow`).
        number("core:cloud_shadows", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Vanilla's: in GUI pixels, and percent of the dark behind the text.
        number("core:chat_width", ck::INT, 320.0, 40.0, 1000.0, 1.0),
        number("core:chat_height", ck::INT, 180.0, 20.0, 1000.0, 1.0),
        number("core:chat_height_closed", ck::INT, 110.0, 20.0, 1000.0, 1.0),
        number("core:chat_opacity", ck::INT, 50.0, 0.0, 100.0, 1.0),
        number("core:chat_visibility", ck::CHOICE, 0.0, 0.0, 2.0, 0.0)
            .set(c::CHOICES, ["all", "system", "hidden"].map(String::from).to_vec()),
        number("core:vsync", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        number("core:fps_limit", ck::INT, 0.0, 0.0, 500.0, 5.0),
        // Taken at the next start; the others are tried if it can't run.
        number("core:renderer", ck::CHOICE, 0.0, 0.0, 1.0, 0.0).set(c::CHOICES, ["vulkan", "wgpu"].map(String::from).to_vec()),
        number("core:fullscreen", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:terrain_gpu", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        number("core:terrain_check", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:client_visibility", ck::INT, 0.0, 0.0, 2.0, 1.0),
        number("core:fov", ck::FLOAT, 70.0, 30.0, 110.0, 1.0),
        // How much sprinting, flying and speed widen the view, as vanilla's FOV effects.
        number("core:fov_effects", ck::FLOAT, 1.0, 0.0, 1.0, 0.0),
        // Off, the view doesn't bob walking; the hand and what it holds still do.
        number("core:camera_bob", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        number("core:sensitivity", ck::FLOAT, 1.0, 0.01, 3.0, 0.01),
        number("core:brightness", ck::FLOAT, 0.5, 0.0, 1.0, 0.0),
        number("core:debug_groups", ck::INT, u32::MAX as f64, 0.0, u32::MAX as f64, 1.0),
        number("core:advanced_tooltips", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:invert_y", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // Buttons and steering for fingers (`touch`), on where the screen is the only input.
        number("core:touch_controls", ck::BOOL, cfg!(target_os = "android") as u8 as f64, 0.0, 1.0, 0.0),
        number("core:auto_jump", ck::BOOL, cfg!(target_os = "android") as u8 as f64, 0.0, 1.0, 0.0),
        string("core:debug_lines", ""),
        string("core:name", &user),
        string("core:relays", ""),
        // chiacraft (phase 4): the Chia network DIDs are read from (`off`, `testnet11`; mainnet
        // only once Ronald says yes), own full nodes instead of the DNS introducers (`host:port`,
        // comma separated), and DID sponsors' addresses, tried in order.
        string("core:chain", "testnet11"),
        string("core:chain_peers", ""),
        // Ours: testnet11, run on Ronald's Mac from a throwaway wallet while we test (HANDOFF.md,
        // "Our DID sponsor").
        string("core:sponsors", "d468728d7e0e3f417c822af6e87ff924c0adbdeeb89a3c0f2d0745b2de40a792"),
        // chiacraft (phase 5): what listing a world locks, in mojos (0.0001 TXCH; the game spends
        // only on testnet11). Taken back by delisting.
        number("core:listing_deposit", ck::INT, 100_000_000.0, 1.0, 1e15, 1.0),
        // chiacraft (phase 5): a fee a listing may pay, in mojos, for a busy testnet11 to take it
        // in sooner; the game spends only on testnet11. None by default: a wallet holding only
        // a sponsor's small gift has no coin for the deposit and a fee of 0.00005 TXCH, and
        // couldn't list at all (Tester 2, 11-05); with none a listing took 17 blocks (207 s).
        number("core:listing_fee", ck::INT, 0.0, 0.0, 1e12, 1.0),
        // Where others reach this game by address; forward it, TCP and UDP.
        number("core:server_port", ck::INT, 27182.0, 1.0, 65535.0, 1.0),
        // A test setting, on no screen (Ronald, 11-04: "Yes, relay-only test"): every connection goes
        // through the relay, never a direct path, to see joins work over the internet from one
        // home network (`--config core:net_relay_only=true`; `net: path to` then reads `relay`).
        number("core:net_relay_only", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        string("core:resource_packs", ""),
        string("core:shader_pack", ""),
        number("core:mob_spawning", ck::BOOL, 1.0, 0.0, 1.0, 0.0).set(c::SCOPE, scope::WORLD),
        // chiacraft: the world signs an achievement credential for each player who earns one.
        number("core:sign_achievements", ck::BOOL, 1.0, 0.0, 1.0, 0.0).set(c::SCOPE, scope::WORLD),
        // chiacraft: the owner's game commits each day's achievements to the chain (one spend from
        // the owner's wallet a day); off unless the owner turns it on.
        number("core:anchor_achievements", ck::BOOL, 0.0, 0.0, 1.0, 0.0).set(c::SCOPE, scope::WORLD),
        number("core:simulation_distance", ck::INT, 8.0, 2.0, 64.0, 1.0).set(c::SCOPE, scope::WORLD),
        // Entities are sent to players within this many blocks (at most the simulation
        // distance); their moves every tick within a third of it, every 2nd within two
        // thirds, every 4th beyond.
        number("core:entity_send_distance", ck::INT, 96.0, 16.0, 1024.0, 16.0).set(c::SCOPE, scope::WORLD),
        // Entities with no player this near tick their mods and paths only once in
        // `core:inactive_entity_interval` ticks; 0: all tick every tick.
        number("core:entity_activation_distance", ck::INT, 48.0, 0.0, 512.0, 16.0).set(c::SCOPE, scope::WORLD),
        number("core:inactive_entity_interval", ck::INT, 20.0, 1.0, 200.0, 1.0).set(c::SCOPE, scope::WORLD),
        // Most cubes mods' tickets keep loaded with no player near (`Host::force_load`).
        number("core:max_forced_cubes", ck::INT, 4096.0, 0.0, 1048576.0, 1.0).set(c::SCOPE, scope::WORLD),
        // chiacraft: off by default, as before upstream 86aad52d0: the hand and held item in view.
        number("core:first_person_model", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:show_avatars", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        number("core:skin_model", ck::CHOICE, 0.0, 0.0, 1.0, 0.0).set(c::CHOICES, ["classic", "slim"].map(String::from).to_vec()),
        number("core:occlusion_culling", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Off, what is behind the camera is drawn too: for shaders that bend the world around.
        number("core:frustum_culling", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Screen effects' half-size parts (`post_low`); off, every effect at full size, as before.
        number("core:post_low", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // The world drawn in 16-bit float (linear light past 1) with the screen effects after it and
        // the picture finished in one last pass (chiacraft, the HDR world); read at start.
        number("core:hdr_world", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // Anti-aliasing: 0 off, 1 temporal (`post::Post`: the world drawn a fraction of a pixel
        // aside each frame and gathered over frames by where each pixel was, the motion image), 2
        // temporal from the camera alone (no motion image: cheaper on phones' tiled GPUs; leaves and
        // mobs left to the clipping). Read at start, as the world's pipelines then draw the motion
        // image too (1). On by default (Ronald, 11-06: "Yes, on by default"): 1, and 2 on a phone,
        // whose tiled GPU the motion image cost a third of its world pass.
        number("core:aa", ck::INT, if cfg!(target_os = "android") { 2.0 } else { 1.0 }, 0.0, 2.0, 1.0),
        // Temporal anti-aliasing: the least share of each new frame (0.1: about ten gathered;
        // less, smoother and slower to follow), live.
        number("core:taa_blend", ck::FLOAT, 0.1, 0.03, 0.5, 0.01),
        // DLSS, the renderer's temporal upscaler, in anti-aliasing's place (`weft_render::UpscaleMode`;
        // not `core:upscaler`, the spatial filter under a render scale): 0 off, 1 DLAA, 2 Quality,
        // 3 Balanced, 4 Performance, 5 Ultra Performance. Live; only where the renderer has it (the
        // `streamline` build on Windows with NVIDIA's files; `core:gpu_dlss*` say what runs), else off.
        number("core:dlss", ck::INT, 0.0, 0.0, 5.0, 1.0),
        // Its model for each mode (the upscaler's own numbers, `UpscalerInfo::models`; DLSS: 0
        // NVIDIA's pick, 5-6 E-F the CNN models, 10-13 J-M the transformer ones), live. 13 (M, the
        // newest transformer) by default (Ronald via Planner, 11-06: M clean where E dithered the
        // leaves); an upscaler without it uses its own default.
        number("core:dlss_model_dlaa", ck::INT, 13.0, 0.0, 31.0, 1.0),
        number("core:dlss_model_quality", ck::INT, 13.0, 0.0, 31.0, 1.0),
        number("core:dlss_model_balanced", ck::INT, 13.0, 0.0, 31.0, 1.0),
        number("core:dlss_model_performance", ck::INT, 13.0, 0.0, 31.0, 1.0),
        number("core:dlss_model_ultra_performance", ck::INT, 13.0, 0.0, 31.0, 1.0),
        // Ray Reconstruction (denoising with the upscaling, at the mode's sizes) and its model
        // (DLSS: 0 default, 4-6 D-F), live.
        number("core:dlss_rr", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:dlss_rr_model", ck::INT, 0.0, 0.0, 31.0, 1.0),
        // DLSS 5's neural rendering over the upscaled picture (not public yet: never runs), live.
        number("core:dlss5", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // A stand-in upscaler that only scales the picture up, for checking the upscaler's path
        // where the renderer has none (the Linux PC); read at start.
        number("core:upscale_standin", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // A check of the path tracer's denoiser on today's frame, before the path tracer exists
        // (`denoise::test_on`; checks only, no menu row): today's picture as the albedo, a noisy light.
        number("core:pt_denoise_test", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // DLSS 5's strength (its intensity control), live.
        number("core:dlss5_intensity", ck::FLOAT, 0.5, 0.0, 1.0, 0.05),
        // The HDR world's bloom (chiacraft): 0 none, 1 a pyramid of 4 levels, 2 of 6 (wider glow);
        // live. Without the HDR world the pack's own bloom effect glows as before.
        number("core:bloom", ck::INT, 2.0, 0.0, 2.0, 1.0),
        // The mist as a volume read by every surface (`client::froxels`); off, each walks it. 2 (ultra,
        // not held to the Mac's budget): cells a quarter of the pixels each way instead of an eighth.
        number("core:froxels", ck::INT, 1.0, 0.0, 2.0, 1.0),
        // The ground's height about the camera for packs (`client::ground_map`; chiacraft's mist
        // banks turn it on with them), live.
        number("core:ground_map", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        number("core:allow_avatars", ck::BOOL, 1.0, 0.0, 1.0, 0.0).set(c::SCOPE, scope::WORLD),
        number("core:keep_inventory", ck::BOOL, 0.0, 0.0, 1.0, 0.0).set(c::SCOPE, scope::WORLD),
        rule("core:ship_thrust", ck::FLOAT, 7.0, 0.0, 64.0),
        rule("core:ship_water_drag", ck::FLOAT, 1.8, 0.0, 16.0),
        rule("core:ship_water_side_drag", ck::FLOAT, 0.4, 0.0, 2.0),
        // Vanilla's: more than this many pushing on one hurt it (0: never).
        rule("core:max_entity_cramming", ck::INT, 24.0, 0.0, 1000.0),
        // Deaths a player has before staying a spectator (0: no end); 1 is hardcore.
        rule("core:lives", ck::INT, 0.0, 0.0, 100.0),
        // Set when the world is made, nobody changes the rules after.
        rule("core:rules_locked", ck::BOOL, 0.0, 0.0, 1.0),
    ];
    out.extend(
        keys.iter()
            .map(|(action, key)| string(&format!("core:key.{action}"), key)),
    );
    // The settings pages, in the order they show.
    let pages: [(&str, &[&str]); 6] = [
        // What anyone changes now and then; the rest is a page away.
        ("core:settings", &["fov", "view_distance"]),
        (
            "core:video",
            &[
                "gui_scale",
                "render_scale",
                "brightness",
                "lod_distance",
                "lod_detail",
                "cloud_height",
                "cloud_distance",
                "cloud_impostor_distance",
                "vsync",
                "fps_limit",
                "fullscreen",
                "renderer",
            ],
        ),
        ("core:controls", &["sensitivity", "invert_y", "auto_jump"]),
        ("core:accessibility", &["fov_effects", "camera_bob"]),
        ("core:profile", &["skin_model", "first_person_model", "show_avatars"]),
        ("core:chat_settings", &["chat_width", "chat_height", "chat_height_closed", "chat_opacity", "chat_visibility"]),
    ];
    let percent = ["core:sensitivity", "core:brightness", "core:fov_effects", "core:render_scale"];
    // Applied on the way, the scale would move the slider under the cursor.
    let on_release = ["core:gui_scale"];
    out.into_iter()
        .map(|e| {
            let at = pages.iter().find_map(|(page, names)| {
                names
                    .iter()
                    .position(|n| e.name.strip_prefix("core:") == Some(*n))
                    .map(|i| (*page, i as i32))
            });
            let e = if percent.contains(&e.name.as_str()) { e.tag(c::PERCENT) } else { e };
            let e = if on_release.contains(&e.name.as_str()) { e.tag(c::ON_RELEASE) } else { e };
            match at {
                Some((page, order)) => e.set(c::PAGE, page.to_string()).set(c::ORDER, order),
                None => e,
            }
        })
        .collect()
}

/// A world rule (`/rule`).
pub fn rule(name: &str, kind: u32, default: f64, min: f64, max: f64) -> RawEntry {
    let step = if kind == ck::FLOAT { 0.0 } else { 1.0 };
    number(name, kind, default, min, max, step).set(weft_abi::comp::config::SCOPE, scope::WORLD)
}

/// A volume setting in percent, on the sound page.
pub fn volume(name: &str) -> RawEntry {
    number(name, ck::INT, 100.0, 0.0, 100.0, 1.0).set(weft_abi::comp::config::PAGE, "core:sound".to_string())
}

fn number(name: &str, kind: u32, default: f64, min: f64, max: f64, step: f64) -> RawEntry {
    use weft_abi::comp::config as c;
    RawEntry::engine(ek::CONFIG, name)
        .set(c::KIND, kind)
        .set(c::DEFAULT, default)
        .set(c::RANGE, [min, max])
        .set(c::STEP, step)
}

fn string(name: &str, default: &str) -> RawEntry {
    number(name, ck::TEXT, 0.0, 0.0, 0.0, 0.0).set(weft_abi::comp::config::TEXT, default.to_string())
}

/// A setting from its entry. The engine's defaults stand as written; a mod's are made to fit.
pub fn bake(e: &Entry, errors: &mut Vec<String>) -> ConfigDef {
    use weft_abi::comp::config as c;
    let [min, max] = e.or(c::RANGE, [0.0, 1.0]);
    let mut d = ConfigDef {
        name: e.name.clone(),
        kind: e.or(c::KIND, ck::BOOL),
        default: e.or(c::DEFAULT, 0.0),
        min,
        max,
        step: e.or(c::STEP, 0.0),
        choices: e.or(c::CHOICES, Vec::new()),
        flags: e.flags(crate::tag_bits::config::TAGS),
        scope: e.or(c::SCOPE, scope::CLIENT),
        text: e.text(c::TEXT).unwrap_or_default().to_string(),
        page: e.text(c::PAGE).unwrap_or_default().to_string(),
        order: e.or(c::ORDER, 0),
        logarithmic: e.has("config.logarithmic"),
    };
    if d.kind == ck::TEXT && d.scope != scope::CLIENT {
        errors.push(format!("setting {}: text settings are the client's own", d.name));
        d.scope = scope::CLIENT;
    }
    if e.owner != crate::registry::ENGINE {
        d.default = d.fit(d.default);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A setting as the engine bakes it from its entry (`number`).
    fn num_def(name: &str, kind: u32, default: f64, min: f64, max: f64, step: f64) -> ConfigDef {
        let f = crate::entries::fold("setting", vec![number(name, kind, default, min, max, step)], &|_| String::new(), &mut Vec::new());
        bake(&f.entries[0], &mut Vec::new())
    }

    /// A text setting as the engine bakes it from its entry (`string`).
    fn text_def(name: &str, default: &str) -> ConfigDef {
        let f = crate::entries::fold("setting", vec![string(name, default)], &|_| String::new(), &mut Vec::new());
        bake(&f.entries[0], &mut Vec::new())
    }

    /// A content switch that adds a setting before `core:name` (a package's key, which made a
    /// joining player nameless) keeps the name and every other value where it was.
    #[test]
    fn a_content_switch_keeps_values_by_name() {
        let old =
            [text_def("core:debug_lines", "fps=always"), text_def("core:name", "Steve"), num_def("core:zoom", ck::INT, 1.0, 0.0, 9.0, 1.0)];
        let bits = [0f64.to_bits(), 0f64.to_bits(), 7f64.to_bits()];
        let texts = ["fps=always".to_string(), "bob".to_string(), String::new()];
        let new = [
            old[0].clone(),
            text_def("core:key.bell.ping", "N"),
            old[1].clone(),
            num_def("core:new", ck::INT, 3.0, 0.0, 9.0, 1.0),
            old[2].clone(),
        ];
        let (b, t) = rekeyed(&old, &bits, &texts, &new);
        assert_eq!(t[2], "bob", "core:name keeps the player's name");
        assert_eq!(t[1], "N", "a setting the new content adds starts at its default");
        assert_eq!((f64::from_bits(b[3]), f64::from_bits(b[4])), (3.0, 7.0));
        // And back: the added ones go, the rest stay.
        let (b2, t2) = rekeyed(&new, &b, &t, &old);
        assert_eq!((t2[1].as_str(), f64::from_bits(b2[2])), ("bob", 7.0));
    }

    #[test]
    fn values_fit_their_setting() {
        let f = crate::entries::fold("setting", vec![number("t:x", ck::INT, 5.0, 0.0, 10.0, 2.0)], &|_| String::new(), &mut Vec::new());
        let d = bake(&f.entries[0], &mut Vec::new());
        assert_eq!((d.fit(3.2), d.fit(99.0), d.fit(f64::NAN)), (4.0, 10.0, 5.0));
        let c = ConfigDef { kind: ck::CHOICE, choices: vec!["low".into(), "high".into()], ..d.clone() };
        assert_eq!((c.parse("high"), c.parse("mid"), c.show(0.0)), (Some(1.0), None, "low".into()));
        let logarithmic = ConfigDef { max: 1_048_576.0, step: 16.0, logarithmic: true, ..d.clone() };
        for value in [0.0, 16.0, 32.0, 256.0, 2048.0, 1_048_576.0] {
            assert_eq!(logarithmic.slider_value(logarithmic.slider_position(value)), value);
        }
        assert!(logarithmic.slider_position(2048.0) < 0.5);
        let b = ConfigDef { kind: ck::BOOL, ..d };
        assert_eq!((b.parse("on"), b.show(1.0)), (Some(1.0), "true".into()));
    }

    /// `/rule core:mob_spawning false` was refused as not a whole number, and `2` was clamped to
    /// true: a switch takes its words and 0/1, a number its range, and nothing else.
    #[test]
    fn typed_values_are_read_or_refused() {
        let b = num_def("t:mobs", ck::BOOL, 1.0, 0.0, 1.0, 0.0);
        assert_eq!([b.read("false"), b.read("0"), b.read("true"), b.read("1")], [Some(0.0), Some(0.0), Some(1.0), Some(1.0)]);
        assert_eq!([b.read("2"), b.read("no"), b.read("")], [None, None, None]);
        let n = num_def("t:dist", ck::INT, 8.0, 2.0, 64.0, 1.0);
        assert_eq!([n.read("12"), n.read("1"), n.read("65"), n.read("far")], [Some(12.0), None, None, None]);
    }
}

//! Settings the engine (namespace `core`) and mods declare: typed, ranged values read with one
//! atomic load (text ones under a lock), edited on screens built from the declarations. The scope
//! says where a value lives: `CLIENT` in each player's `config/<ns>.txt`; `SERVER` in the
//! server's file, sent to its players whenever it changes; `WORLD` saved with the world (game
//! rules, `/rule`) and sent the same way. The engine's own settings and key bindings are `core:`
//! ones like any other, in `config/core.txt`.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};

use weft_abi::{config_kind as ck, config_scope as scope};

use crate::content::content;

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
    pub scope: u32,
    /// The default of a `TEXT` setting.
    pub text: String,
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

    pub fn number(name: &str, kind: u32, default: f64, min: f64, max: f64, step: f64) -> ConfigDef {
        ConfigDef {
            name: name.into(),
            kind,
            default,
            min,
            max,
            step,
            choices: Vec::new(),
            flags: 0,
            scope: scope::CLIENT,
            text: String::new(),
        }
    }

    pub fn text(name: &str, default: &str) -> ConfigDef {
        ConfigDef { text: default.into(), ..ConfigDef::number(name, ck::TEXT, 0.0, 0.0, 0.0, 0.0) }
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
        new.game.config_names.get(name).copied()
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
    content().game.config_names.get(name).copied()
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

/// Settings the engine itself reads: the client's options, `core:mob_spawning` for worlds, and
/// a binding per key (`core:key.<action>`, from `keys`: action and default key); each sound
/// category adds its volume (`volume`).
pub fn engine_configs(keys: &[(String, String)]) -> Vec<ConfigDef> {
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Player".into());
    let mut out = vec![
        volume("core:volume_master"),
        ConfigDef::text("core:language", &crate::settings::system_language()),
        ConfigDef::number("core:gui_scale", ck::INT, 0.0, 0.0, 16.0, 1.0),
        ConfigDef::number("core:view_distance", ck::INT, 12.0, 2.0, 64.0, 1.0),
        // Distant terrain beyond the view (`client::lod`), in cubes; 0 off.
        ConfigDef::number("core:lod_distance", ck::INT, 256.0, 0.0, 2048.0, 16.0),
        // chiacraft: the far ground's look for shader packs (`Globals.extra.w`'s top byte), bits:
        // 1 small plants thin out, 2 block-scale shading softens, 4 textures give way to their
        // average colour, all with distance (Ronald, 11-06: all three).
        ConfigDef::number("core:far_look", ck::INT, 7.0, 0.0, 255.0, 1.0),
        // The 3D world's share of the window's pixels each way (`app::world_frame`), the UI
        // always whole; 0 picks it by the screen (fewer on high-density ones).
        ConfigDef::number("core:render_scale", ck::FLOAT, 0.0, 0.0, 1.0, 0.05),
        // How a world drawn smaller than the window (`core:render_scale`) is scaled up: 0 bilinear,
        // 1 FSR 1's edge-adaptive upscale (`easu` in common.wgsl), 2 Snapdragon GSR 1 (`gsr`). Live.
        ConfigDef::number("core:upscaler", ck::INT, 0.0, 0.0, 2.0, 1.0),
        // The GPU, as the renderer found it (`Backend::device_*`), for mods to read: put at start,
        // never saved by a player (the base game's first quality preset, Research round 19, 7).
        ConfigDef::text("core:gpu_name", ""),
        ConfigDef::number("core:gpu_class", ck::INT, 0.0, 0.0, 3.0, 1.0),
        ConfigDef::number("core:gpu_memory_mb", ck::INT, 0.0, 0.0, 1.0e7, 1.0),
        // 1 when the GPU can trace rays (`Backend::ray_query`): `core:rt_shadows` then works.
        ConfigDef::number("core:gpu_ray_query", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // Blocks clouds float above where their dimension puts them.
        ConfigDef::number("core:cloud_height", ck::INT, 0.0, 0.0, 512.0, 8.0),
        // Sun shadows, for shader packs that draw them: 0 off, 1 the near map only (80 blocks
        // about the player), 2 near and far (400). The maps' size in texels: 1024, 2048 or 4096.
        ConfigDef::number("core:shadows", ck::INT, 2.0, 0.0, 2.0, 1.0),
        ConfigDef::number("core:shadow_size", ck::INT, 2048.0, 1024.0, 4096.0, 1024.0),
        // The far shadow map's reach to each side (blocks): 0 auto, 2000 with the HDR world (at no
        // measurable cost, more far hills and crowns shadowed: Far view, 11-05) and 400 without
        // (today's look as made); a number sets it (`client::prepare_shadows`).
        ConfigDef::number("core:shadow_far", ck::INT, 0.0, 0.0, 2400.0, 100.0),
        // Sun and moon shadows traced through the scene instead of the shadow maps, on a GPU with
        // ray queries (Vulkan's; not the Mac or phones): ultra, off unless chosen.
        // 2: their soft edges from more rays a pixel (with TAA; about three times the cost).
        ConfigDef::number("core:rt_shadows", ck::INT, 0.0, 0.0, 2.0, 1.0),
        // The soft clouds' shadow on the land (with the HDR world; `client::cloud_shadow`).
        ConfigDef::number("core:cloud_shadows", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Vanilla's: in GUI pixels, and percent of the dark behind the text.
        ConfigDef::number("core:chat_width", ck::INT, 320.0, 40.0, 1000.0, 10.0),
        ConfigDef::number("core:chat_height", ck::INT, 180.0, 20.0, 1000.0, 10.0),
        ConfigDef::number("core:chat_height_closed", ck::INT, 110.0, 20.0, 1000.0, 10.0),
        ConfigDef::number("core:chat_opacity", ck::INT, 50.0, 0.0, 100.0, 5.0),
        ConfigDef {
            kind: ck::CHOICE,
            choices: vec!["all".into(), "system".into(), "hidden".into()],
            ..ConfigDef::number("core:chat_visibility", ck::CHOICE, 0.0, 0.0, 2.0, 0.0)
        },
        ConfigDef::number("core:vsync", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Frames a second at most (0: as many as it can draw): less heat and battery, steadier.
        ConfigDef::number("core:fps_cap", ck::INT, 0.0, 0.0, 360.0, 10.0),
        ConfigDef::number("core:fullscreen", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        ConfigDef::number("core:terrain_gpu", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        ConfigDef::number("core:terrain_check", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        ConfigDef::number("core:client_visibility", ck::INT, 0.0, 0.0, 2.0, 1.0),
        ConfigDef::number("core:fov", ck::FLOAT, 70.0, 30.0, 110.0, 1.0),
        ConfigDef::number("core:sensitivity", ck::FLOAT, 1.0, 0.01, 3.0, 0.05),
        ConfigDef::number("core:brightness", ck::FLOAT, 0.5, 0.0, 1.0, 0.0),
        ConfigDef::number("core:debug_groups", ck::INT, u32::MAX as f64, 0.0, u32::MAX as f64, 1.0),
        ConfigDef::number("core:invert_y", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        ConfigDef::text("core:debug_lines", ""),
        ConfigDef::text("core:name", &user),
        ConfigDef::text("core:relays", ""),
        // chiacraft (phase 4): the Chia network DIDs are read from (`off`, `testnet11`; mainnet
        // only once Ronald says yes), own full nodes instead of the DNS introducers (`host:port`,
        // comma separated), and DID sponsors' addresses, tried in order.
        ConfigDef::text("core:chain", "testnet11"),
        ConfigDef::text("core:chain_peers", ""),
        // Ours: testnet11, run on Ronald's Mac from a throwaway wallet while we test (HANDOFF.md,
        // "Our DID sponsor").
        ConfigDef::text("core:sponsors", "d468728d7e0e3f417c822af6e87ff924c0adbdeeb89a3c0f2d0745b2de40a792"),
        // chiacraft (phase 5): what listing a world locks, in mojos (0.0001 TXCH; the game spends
        // only on testnet11). Taken back by delisting.
        ConfigDef::number("core:listing_deposit", ck::INT, 100_000_000.0, 1.0, 1e15, 1.0),
        // chiacraft (phase 5): a fee a listing may pay, in mojos, for a busy testnet11 to take it
        // in sooner; the game spends only on testnet11. None by default: a wallet holding only
        // a sponsor's small gift has no coin for the deposit and a fee of 0.00005 TXCH, and
        // couldn't list at all (Tester 2, 11-05); with none a listing took 17 blocks (207 s).
        ConfigDef::number("core:listing_fee", ck::INT, 0.0, 0.0, 1e12, 1.0),
        // Where others reach this game by address; forward it, TCP and UDP.
        ConfigDef::number("core:server_port", ck::INT, 27182.0, 1.0, 65535.0, 1.0),
        // A test setting, on no screen (Ronald, 11-04: "Yes, relay-only test"): every connection goes
        // through the relay, never a direct path, to see joins work over the internet from one
        // home network (`--config core:net_relay_only=true`; `net: path to` then reads `relay`).
        ConfigDef::number("core:net_relay_only", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        ConfigDef::text("core:resource_packs", ""),
        ConfigDef::text("core:shader_pack", ""),
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:mob_spawning", ck::BOOL, 1.0, 0.0, 1.0, 0.0) },
        // chiacraft: the world signs an achievement credential for each player who earns one.
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:sign_achievements", ck::BOOL, 1.0, 0.0, 1.0, 0.0) },
        // chiacraft: the owner's game commits each day's achievements to the chain (one spend from
        // the owner's wallet a day); off unless the owner turns it on.
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:anchor_achievements", ck::BOOL, 0.0, 0.0, 1.0, 0.0) },
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:simulation_distance", ck::INT, 8.0, 2.0, 64.0, 1.0) },
        // Entities are sent to players within this many blocks (at most the simulation
        // distance); their moves every tick within a third of it, every 2nd within two
        // thirds, every 4th beyond.
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:entity_send_distance", ck::INT, 96.0, 16.0, 1024.0, 16.0) },
        // Entities with no player this near tick their mods and paths only once in
        // `core:inactive_entity_interval` ticks; 0: all tick every tick.
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:entity_activation_distance", ck::INT, 48.0, 0.0, 512.0, 16.0) },
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:inactive_entity_interval", ck::INT, 20.0, 1.0, 200.0, 1.0) },
        // Most cubes mods' tickets keep loaded with no player near (`Host::force_load`).
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:max_forced_cubes", ck::INT, 4096.0, 0.0, 1048576.0, 1.0) },
        ConfigDef::number("core:first_person_model", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        ConfigDef::number("core:show_avatars", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        ConfigDef::number("core:occlusion_culling", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Soft volumetric clouds traced at a quarter of the pixels and blended over frames, for a
        // pack that draws them (chiacraft's); off, the pack's own (blocky) clouds. On by default
        // (Tester 2 and the shader look sheets, 11-05: real cumulus where the blocky ones are flat
        // slabs, at the same GPU time).
        ConfigDef::number("core:soft_clouds", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // Screen effects' half-size parts (`post_low`); off, every effect at full size, as before.
        ConfigDef::number("core:post_low", ck::BOOL, 1.0, 0.0, 1.0, 0.0),
        // The world drawn in 16-bit float (linear light past 1) with the screen effects after it and
        // the picture finished in one last pass (chiacraft, the HDR world); read at start.
        ConfigDef::number("core:hdr_world", ck::BOOL, 0.0, 0.0, 1.0, 0.0),
        // Anti-aliasing: 0 off, 1 temporal (`post::Post`: the world drawn a fraction of a pixel
        // aside each frame and gathered over frames by where each pixel was, the motion image).
        // Read at start, as the world's pipelines then draw the motion image too.
        ConfigDef::number("core:aa", ck::INT, 0.0, 0.0, 1.0, 1.0),
        // Temporal anti-aliasing: the least share of each new frame (0.1: about ten gathered;
        // less, smoother and slower to follow), live.
        ConfigDef::number("core:taa_blend", ck::FLOAT, 0.1, 0.03, 0.5, 0.01),
        // The HDR world's bloom (chiacraft): 0 none, 1 a pyramid of 4 levels, 2 of 6 (wider glow);
        // live. Without the HDR world the pack's own bloom effect glows as before.
        ConfigDef::number("core:bloom", ck::INT, 2.0, 0.0, 2.0, 1.0),
        // The mist as a volume read by every surface (`client::froxels`); off, each walks it. 2 (ultra,
        // not held to the Mac's budget): cells a quarter of the pixels each way instead of an eighth.
        ConfigDef::number("core:froxels", ck::INT, 1.0, 0.0, 2.0, 1.0),
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:allow_avatars", ck::BOOL, 1.0, 0.0, 1.0, 0.0) },
        ConfigDef { scope: scope::WORLD, ..ConfigDef::number("core:keep_inventory", ck::BOOL, 0.0, 0.0, 1.0, 0.0) },
    ];
    out.extend(
        keys.iter()
            .map(|(action, key)| ConfigDef::text(&format!("core:key.{action}"), key)),
    );
    out
}

/// A volume setting in percent.
pub fn volume(name: &str) -> ConfigDef {
    ConfigDef::number(name, ck::INT, 100.0, 0.0, 100.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A content switch that adds a setting before `core:name` (a package's key, which made a
    /// joining player nameless) keeps the name and every other value where it was.
    #[test]
    fn a_content_switch_keeps_values_by_name() {
        let old = [
            ConfigDef::text("core:debug_lines", "fps=always"),
            ConfigDef::text("core:name", "Steve"),
            ConfigDef::number("core:zoom", ck::INT, 1.0, 0.0, 9.0, 1.0),
        ];
        let bits = [0f64.to_bits(), 0f64.to_bits(), 7f64.to_bits()];
        let texts = ["fps=always".to_string(), "bob".to_string(), String::new()];
        let new = [
            old[0].clone(),
            ConfigDef::text("core:key.bell.ping", "N"),
            old[1].clone(),
            ConfigDef::number("core:new", ck::INT, 3.0, 0.0, 9.0, 1.0),
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
        let d = ConfigDef::number("t:x", ck::INT, 5.0, 0.0, 10.0, 2.0);
        assert_eq!((d.fit(3.2), d.fit(99.0), d.fit(f64::NAN)), (4.0, 10.0, 5.0));
        let c = ConfigDef { kind: ck::CHOICE, choices: vec!["low".into(), "high".into()], ..d.clone() };
        assert_eq!((c.parse("high"), c.parse("mid"), c.show(0.0)), (Some(1.0), None, "low".into()));
        let b = ConfigDef { kind: ck::BOOL, ..d };
        assert_eq!((b.parse("on"), b.show(1.0)), (Some(1.0), "true".into()));
    }

    /// `/rule core:mob_spawning false` was refused as not a whole number, and `2` was clamped to
    /// true: a switch takes its words and 0/1, a number its range, and nothing else.
    #[test]
    fn typed_values_are_read_or_refused() {
        let b = ConfigDef::number("t:mobs", ck::BOOL, 1.0, 0.0, 1.0, 0.0);
        assert_eq!([b.read("false"), b.read("0"), b.read("true"), b.read("1")], [Some(0.0), Some(0.0), Some(1.0), Some(1.0)]);
        assert_eq!([b.read("2"), b.read("no"), b.read("")], [None, None, None]);
        let n = ConfigDef::number("t:dist", ck::INT, 8.0, 2.0, 64.0, 1.0);
        assert_eq!([n.read("12"), n.read("1"), n.read("65"), n.read("far")], [Some(12.0), None, None, None]);
    }
}

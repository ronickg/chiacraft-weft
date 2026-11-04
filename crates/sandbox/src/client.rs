//! Client halves (API 1.2; `docs/sandboxed-mods.md`, "Client halves"): a package's
//! `code/client.wasm`, a component for the world `chiacraft:mods/client-mod`, run in each player's
//! own game. It reads what that game shows the player and adds HUD elements, screens, keys,
//! packets from its server half and frame hooks; the game asks it for the parts of a HUD element
//! or screen as data ([`Part`]) and replays them into its own UI, so a client half draws with the
//! game's kit. Same engine, limits and strike rule as server halves.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Store, StoreLimits};

use crate::{Arg, ItemStack, Level, STRIKE_WINDOW, STRIKES, clip, clip_args, engine, log, why};

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit/v1.3",
        world: "client-mod",
        with: {
            "chiacraft:mods/types": crate::bindings::chiacraft::mods::types,
            "chiacraft:mods/log": crate::bindings::chiacraft::mods::log,
            "chiacraft:mods/names": crate::bindings::chiacraft::mods::names,
        },
    });
}

use crate::bindings::chiacraft::mods::{log as mod_log, names};
use bindings::ClientMod as ClientBindings;
pub use bindings::chiacraft::mods::ui::{Part, PartKind, Style};
use bindings::chiacraft::mods::{client as wclient, client_registry};

/// Parts one build may have, and the anchors a HUD element may take.
pub const PARTS: usize = 512;
pub const ANCHORS: &[&str] =
    &["top_left", "top_right", "bottom_left", "bottom_right", "hotbar", "hotbar_left", "hotbar_right", "above_hotbar", "under_crosshair"];
const SHOWS: u32 = 8;
const SENDS: u32 = 32;
const HOOKS: usize = 128;

/// The player's game's side of one call into a client half.
pub trait ClientGame: Send {
    /// Feet, health, max health.
    fn me(&mut self) -> Option<([f64; 3], f32, f32)>;
    fn day_time(&mut self) -> u64;
    fn raining(&mut self) -> bool;
    fn creative(&mut self) -> bool;
    /// The player's own value of component `name` (a full name of the package's).
    fn component(&mut self, name: &str) -> Option<Vec<u8>>;
    fn inventory_slot(&mut self, slot: u32) -> ItemStack;
    fn screen(&mut self) -> String;
    /// Whether this call may drive the game (a key, button, packet or frame; not a binding).
    fn can_act(&self) -> bool;
    fn send(&mut self, channel: &str, data: &[u8]);
    fn open_screen(&mut self, name: &str);
    fn close_screen(&mut self);
    fn show(&mut self, key: &str, args: &[Arg]);

    fn block_name(&mut self, state: u32) -> String;
    fn block_state(&mut self, name: &str) -> Option<u32>;
    fn prop(&mut self, state: u32, prop: &str) -> Option<String>;
    fn with_prop(&mut self, state: u32, prop: &str, value: &str) -> Option<u32>;
    fn item_name(&mut self, id: u32) -> String;
    fn item_id(&mut self, name: &str) -> Option<u32>;
    fn entity_kind(&mut self, id: u32) -> String;
    fn entity_kind_id(&mut self, name: &str) -> Option<u32>;
}

/// One registration of a client half's `init`; its index is the callback's number. Names are
/// full (`<package>:<name>`).
#[derive(Clone, Debug, PartialEq)]
pub enum ClientHook {
    Hud { name: String, anchor: String, order: i32 },
    Screen { name: String, pauses: bool },
    Key { name: String, default: String },
    Packet(String),
    Frame,
}

struct ClientState {
    package: String,
    game: Option<Box<dyn ClientGame>>,
    registering: Option<Vec<ClientHook>>,
    limits: StoreLimits,
    logs: u32,
    shows: u32,
    sends: u32,
}

impl ClientState {
    fn game(&mut self) -> Option<&mut (dyn ClientGame + 'static)> {
        self.game.as_deref_mut()
    }

    /// The game, only when this call may act.
    fn acting(&mut self) -> Option<&mut (dyn ClientGame + 'static)> {
        self.game.as_deref_mut().filter(|g| g.can_act())
    }

    fn full(&self, name: &str) -> Option<String> {
        let local = name
            .strip_prefix(&format!("{}:", self.package))
            .unwrap_or(name);
        crate::local_name_ok(local).then(|| format!("{}:{local}", self.package))
    }

    fn register(&mut self, hook: ClientHook) -> Result<u32, String> {
        let Some(list) = self.registering.as_mut() else {
            return Err("registering is only possible while `init` runs".into());
        };
        if list.len() >= HOOKS {
            return Err(format!("at most {HOOKS} registrations"));
        }
        list.push(hook);
        Ok(list.len() as u32 - 1)
    }
}

impl mod_log::Host for ClientState {
    fn write(&mut self, level: mod_log::Level, message: String) {
        self.logs += 1;
        if self.logs > 64 {
            return;
        }
        let level = match level {
            mod_log::Level::Error => Level::Error,
            mod_log::Level::Warn => Level::Warn,
            mod_log::Level::Info => Level::Info,
            mod_log::Level::Debug => Level::Debug,
        };
        log(level, &self.package, clip(&message, 512));
    }
}

impl crate::bindings::chiacraft::mods::types::Host for ClientState {}
impl bindings::chiacraft::mods::ui::Host for ClientState {}

impl names::Host for ClientState {
    fn block(&mut self, state: u32) -> String {
        self.game().map_or(String::new(), |g| g.block_name(state))
    }
    fn block_state(&mut self, name: String) -> Option<u32> {
        (name.len() <= 256).then_some(())?;
        self.game().and_then(|g| g.block_state(&name))
    }
    fn prop(&mut self, state: u32, prop: String) -> Option<String> {
        self.game().and_then(|g| g.prop(state, clip(&prop, 64)))
    }
    fn with_prop(&mut self, state: u32, prop: String, value: String) -> Option<u32> {
        self.game()
            .and_then(|g| g.with_prop(state, clip(&prop, 64), clip(&value, 64)))
    }
    fn item(&mut self, id: u32) -> String {
        self.game().map_or(String::new(), |g| g.item_name(id))
    }
    fn item_id(&mut self, name: String) -> Option<u32> {
        (name.len() <= 256).then_some(())?;
        self.game().and_then(|g| g.item_id(&name))
    }
    fn entity_kind(&mut self, id: u32) -> String {
        self.game().map_or(String::new(), |g| g.entity_kind(id))
    }
    fn entity_kind_id(&mut self, name: String) -> Option<u32> {
        (name.len() <= 256).then_some(())?;
        self.game().and_then(|g| g.entity_kind_id(&name))
    }
}

impl wclient::Host for ClientState {
    fn me(&mut self) -> Option<((f64, f64, f64), f32, f32)> {
        self.game()
            .and_then(|g| g.me())
            .map(|(p, h, m)| ((p[0], p[1], p[2]), h, m))
    }
    fn day_time(&mut self) -> u64 {
        self.game().map_or(0, |g| g.day_time())
    }
    fn raining(&mut self) -> bool {
        self.game().is_some_and(|g| g.raining())
    }
    fn creative(&mut self) -> bool {
        self.game().is_some_and(|g| g.creative())
    }
    fn component(&mut self, name: String) -> Option<Vec<u8>> {
        let full = self.full(&name)?;
        self.game().and_then(|g| g.component(&full))
    }
    fn inventory_slot(&mut self, slot: u32) -> ItemStack {
        self.game()
            .map_or(ItemStack { item: 0, count: 0, data: 0 }, |g| g.inventory_slot(slot))
    }
    fn screen(&mut self) -> String {
        self.game().map_or(String::new(), |g| g.screen())
    }
    fn send(&mut self, channel: String, data: Vec<u8>) {
        self.sends += 1;
        if self.sends > SENDS || data.len() > crate::PACKET_BYTES {
            return;
        }
        let Some(full) = self.full(&channel) else { return };
        if let Some(g) = self.acting() {
            g.send(&full, &data)
        }
    }
    fn open_screen(&mut self, name: String) {
        let Some(full) = self.full(&name) else { return };
        if let Some(g) = self.acting() {
            g.open_screen(&full)
        }
    }
    fn close_screen(&mut self) {
        if let Some(g) = self.acting() {
            g.close_screen()
        }
    }
    fn show(&mut self, key: String, args: Vec<Arg>) {
        self.shows += 1;
        if self.shows > SHOWS || key.len() > 128 {
            return;
        }
        let args = clip_args(args);
        if let Some(g) = self.acting() {
            g.show(&key, &args)
        }
    }
}

impl client_registry::Host for ClientState {
    fn hud_element(&mut self, name: String, anchor: String, order: i32) -> Result<u32, String> {
        let full = self
            .full(&name)
            .ok_or_else(|| format!("HUD element `{}`: a name of 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)))?;
        if !ANCHORS.contains(&anchor.as_str()) {
            return Err(format!("HUD element `{name}`: no anchor `{}`", clip(&anchor, 40)));
        }
        self.register(ClientHook::Hud { name: full, anchor, order: order.clamp(-1000, 1000) })
    }
    fn screen(&mut self, name: String, pauses: bool) -> Result<u32, String> {
        let full = self
            .full(&name)
            .ok_or_else(|| format!("screen `{}`: a name of 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)))?;
        self.register(ClientHook::Screen { name: full, pauses })
    }
    fn key(&mut self, name: String, default: String) -> Result<u32, String> {
        let full = self
            .full(&name)
            .ok_or_else(|| format!("key `{}`: a name of 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)))?;
        let fine = default.len() <= 32
            && default
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+_".contains(c));
        if !fine {
            return Err(format!("key `{name}`: its default binding `{}` isn't one", clip(&default, 40)));
        }
        self.register(ClientHook::Key { name: full, default })
    }
    fn packet(&mut self, channel: String) -> Result<u32, String> {
        let full = self
            .full(&channel)
            .ok_or_else(|| format!("channel `{}`: a name of 1 to 32 lowercase letters, digits and `_`", clip(&channel, 40)))?;
        self.register(ClientHook::Packet(full))
    }
    fn frame(&mut self) -> u32 {
        self.register(ClientHook::Frame).unwrap_or(u32::MAX)
    }
}

fn linker() -> Result<Arc<Linker<ClientState>>, String> {
    static LINKER: OnceLock<Result<Arc<Linker<ClientState>>, String>> = OnceLock::new();
    LINKER
        .get_or_init(|| {
            let mut linker = Linker::new(engine::engine()?);
            ClientBindings::add_to_linker::<ClientState, HasSelf<ClientState>>(&mut linker, |s| s).map_err(|e| format!("{e:#}"))?;
            Ok(Arc::new(linker))
        })
        .clone()
}

/// A compiled client half and what its `init` registers.
pub struct ClientMod {
    package: String,
    component: Component,
    hooks: Vec<ClientHook>,
}

struct Live {
    store: Store<ClientState>,
    bindings: ClientBindings,
}

impl ClientMod {
    /// Compiles `wasm` (a component for `client-mod`) and runs its `init` once, to learn what it
    /// registers.
    pub fn load(package: &str, wasm: &[u8]) -> Result<ClientMod, String> {
        if wasm.len() > engine::MAX_COMPONENT_BYTES {
            return Err(format!("code/client.wasm is bigger than {} MiB", engine::MAX_COMPONENT_BYTES >> 20));
        }
        crate::check_imports(wasm)?;
        let component = crate::compiled(wasm)?;
        let mut m = ClientMod { package: package.to_string(), component, hooks: Vec::new() };
        let (_, hooks) = m.instantiate(None)?;
        m.hooks = hooks;
        Ok(m)
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn hooks(&self) -> &[ClientHook] {
        &self.hooks
    }

    fn instantiate(&self, game: Option<Box<dyn ClientGame>>) -> Result<(Live, Vec<ClientHook>), String> {
        let engine = engine::engine()?;
        let state = ClientState {
            package: self.package.clone(),
            game: None,
            registering: Some(Vec::new()),
            limits: engine::limits(),
            logs: 0,
            shows: 0,
            sends: 0,
        };
        let mut store = Store::new(engine, state);
        store.limiter(|s| &mut s.limits);
        store.epoch_deadline_trap();
        store.set_epoch_deadline(engine::WATCHDOG_EPOCHS);
        store
            .set_fuel(engine::fuel::INIT)
            .map_err(|e| e.to_string())?;
        let bindings = ClientBindings::instantiate(&mut store, &self.component, &*linker()?).map_err(|e| why(&e))?;
        store.data_mut().game = game;
        let r = bindings.chiacraft_mods_client_hooks().call_init(&mut store);
        store.data_mut().game = None;
        let hooks = store.data_mut().registering.take().unwrap_or_default();
        match r {
            Ok(Ok(())) => Ok((Live { store, bindings }, hooks)),
            Ok(Err(e)) => Err(format!("init: {}", clip(&e, 512))),
            Err(e) => Err(format!("init: {}", why(&e))),
        }
    }
}

/// A client half running in the player's game: started on first use, started again after a
/// strike, stopped after [`STRIKES`] within [`STRIKE_WINDOW`].
pub struct ClientInstance {
    m: Arc<ClientMod>,
    live: Option<Live>,
    strikes: VecDeque<Instant>,
    stopped: Option<String>,
}

impl ClientInstance {
    pub fn new(m: Arc<ClientMod>) -> ClientInstance {
        ClientInstance { m, live: None, strikes: VecDeque::new(), stopped: None }
    }

    pub fn stopped(&self) -> Option<&str> {
        self.stopped.as_deref()
    }

    fn strike(&mut self, why: String) {
        self.live = None;
        let now = Instant::now();
        while self
            .strikes
            .front()
            .is_some_and(|t| now.duration_since(*t) > STRIKE_WINDOW)
        {
            self.strikes.pop_front();
        }
        self.strikes.push_back(now);
        let package = self.m.package.clone();
        if self.strikes.len() >= STRIKES {
            log(Level::Error, &package, &format!("client half: {why}; stopped after {STRIKES} strikes"));
            self.stopped = Some(why);
        } else {
            log(Level::Warn, &package, &format!("client half: {why}; starting it over (strike {} of {STRIKES})", self.strikes.len()));
        }
    }

    fn call<T>(
        &mut self,
        game: Box<dyn ClientGame>,
        fuel: u64,
        f: impl FnOnce(&ClientBindings, &mut Store<ClientState>) -> wasmtime::Result<T>,
    ) -> Option<T> {
        if self.stopped.is_some() {
            return None;
        }
        if self.live.is_none() {
            match self.m.instantiate(None) {
                Ok((live, hooks)) if hooks == self.m.hooks => self.live = Some(live),
                Ok(_) => {
                    self.stopped = Some("registered other things than when it loaded".into());
                    return None;
                }
                Err(e) => {
                    self.strike(e);
                    return None;
                }
            }
        }
        let live = self.live.as_mut()?;
        let store = &mut live.store;
        let data = store.data_mut();
        data.game = Some(game);
        data.logs = 0;
        data.shows = 0;
        data.sends = 0;
        store.set_epoch_deadline(engine::WATCHDOG_EPOCHS);
        store.set_fuel(fuel).ok()?;
        let r = f(&live.bindings, store);
        store.data_mut().game = None;
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                let why = why(&e);
                self.strike(why);
                None
            }
        }
    }

    /// The parts of a HUD element or screen, checked: at most [`PARTS`], and every part's
    /// children within the list. A failed or malformed build gives none.
    pub fn build(&mut self, game: Box<dyn ClientGame>, callback: u32) -> Vec<Part> {
        let parts = self
            .call(game, engine::fuel::EVENT, |b, s| b.chiacraft_mods_client_hooks().call_build(s, callback))
            .unwrap_or_default();
        if well_formed(&parts) { parts } else { Vec::new() }
    }

    /// A bar's or icons' value; not finite or failed: hidden (-1).
    pub fn value(&mut self, game: Box<dyn ClientGame>, bind: u32) -> f32 {
        self.call(game, engine::fuel::QUESTION, |b, s| b.chiacraft_mods_client_hooks().call_value(s, bind))
            .filter(|v| v.is_finite())
            .unwrap_or(-1.0)
    }

    /// A text binding's line; failed: empty.
    pub fn text(&mut self, game: Box<dyn ClientGame>, bind: u32) -> (String, Vec<Arg>) {
        self.call(game, engine::fuel::QUESTION, |b, s| b.chiacraft_mods_client_hooks().call_text(s, bind))
            .map(|(k, a)| (clip(&k, 128).to_string(), clip_args(a)))
            .unwrap_or_default()
    }

    pub fn click(&mut self, game: Box<dyn ClientGame>, bind: u32) {
        self.call(game, engine::fuel::EVENT, |b, s| b.chiacraft_mods_client_hooks().call_click(s, bind));
    }

    pub fn key(&mut self, game: Box<dyn ClientGame>, callback: u32) {
        self.call(game, engine::fuel::EVENT, |b, s| b.chiacraft_mods_client_hooks().call_key(s, callback));
    }

    pub fn packet(&mut self, game: Box<dyn ClientGame>, callback: u32, data: &[u8]) {
        if data.len() > crate::PACKET_BYTES {
            return;
        }
        self.call(game, engine::fuel::EVENT, |b, s| {
            b.chiacraft_mods_client_hooks()
                .call_packet(s, callback, data)
        });
    }

    pub fn frame(&mut self, game: Box<dyn ClientGame>, callback: u32, seconds: f32) {
        let seconds = if seconds.is_finite() { seconds.clamp(0.0, 10.0) } else { 0.0 };
        self.call(game, engine::fuel::TICK, |b, s| {
            b.chiacraft_mods_client_hooks()
                .call_frame(s, callback, seconds)
        });
    }
}

/// Every part's children lie within the list, texts and classes are short, numbers finite.
pub fn well_formed(parts: &[Part]) -> bool {
    fn subtree(parts: &[Part], i: usize, depth: u32) -> Option<usize> {
        let p = parts.get(i)?;
        if depth > 32 || p.class.len() > 64 || p.text.len() > 128 || p.args.len() > 16 || p.count > 1024 {
            return None;
        }
        let s = &p.style;
        let nums = [s.gap, s.pad, s.grow, s.scale, p.size.0, p.size.1];
        let opt = [s.width, s.height].into_iter().flatten();
        let at = s.at.map_or([0.0; 4], |a| [a.0, a.1, a.2, a.3]);
        if !nums.iter().chain(at.iter()).all(|v| v.is_finite()) || !opt.clone().all(f32::is_finite) || s.class.len() > 64 {
            return None;
        }
        let mut next = i + 1;
        for _ in 0..p.children {
            next = subtree(parts, next, depth + 1)?;
        }
        Some(next)
    }
    if parts.len() > PARTS {
        return false;
    }
    let mut i = 0;
    while i < parts.len() {
        match subtree(parts, i, 0) {
            Some(next) => i = next,
            None => return false,
        }
    }
    true
}

/// A client half's instance, shared by the callbacks the game registered for it.
pub type Shared = Arc<Mutex<ClientInstance>>;

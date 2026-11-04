//! Sandboxed code mods (chiacraft plan, phase 6; `docs/sandboxed-mods.md`).
//!
//! A package's server half is a WebAssembly component for the WIT world `chiacraft:mods/server-mod`
//! (`wit/v1.0/mod.wit`). This crate compiles it, runs it under fuel, wall-time and memory limits
//! with nothing to import but that world's calls, and calls its hooks. It knows nothing of Weft:
//! the game answers the mod's world calls through [`Game`], one per callback, and turns the
//! mod's registrations ([`Hook`]) into its own (the `packages` mod does both over Weft's SDK).
//!
//! - [`Mod::load`] compiles a component and runs its `init` once with no world, to learn what it
//!   registers.
//! - [`Instance`] is the mod running in one world: its own memory and storage, started on the
//!   first call there. A call that runs out of fuel, traps or grows memory past the cap is a
//!   strike: the instance is dropped and started again on the next call. Three strikes within
//!   [`STRIKE_WINDOW`] stop it in that world.

pub mod client;
pub mod engine;
pub mod storage;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use wasmtime::component::{Component, HasSelf, Linker, TypedFunc};
use wasmtime::{Store, StoreLimits, Trap};

pub use storage::Storage;

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit/v1.2",
        world: "host",
    });
}

use bindings::Host_ as ServerMod;
pub use bindings::chiacraft::mods::registry::{Chain, ComponentFlags, Event, Question};
pub use bindings::chiacraft::mods::types::{
    Arg, BlockUpdateEvent, BlockUseEvent, EntityState, ItemStack, ItemUseEvent, ItemUseResult, Query,
};
use bindings::chiacraft::mods::{command, log as mod_log, names, registry, storage as mod_storage, types, world};
pub use bindings::exports::chiacraft::mods::hooks::{BlockChange, Death, EventValue, Hurt, Invocation, ItemUsed, Pickup, Value, Verdict};

/// The API version this game serves.
/// Its version is `chiacraft_packages::code::API`; a test checks the two agree.
pub const API: &str = "1.2.0";
/// Strikes within this long stop a mod in a world.
pub const STRIKE_WINDOW: Duration = Duration::from_secs(600);
pub const STRIKES: usize = 3;

/// Lines a mod may log, chat lines it may send, and so on, per call.
const LOG_LINES: u32 = 64;
const LOG_CHARS: usize = 512;
const TELLS: u32 = 32;
const REPLIES: u32 = 32;
const BLOCK_SETS: u32 = 4096;
const SPAWNS: u32 = 64;
const NEAR_RADIUS: f64 = 64.0;
const NEAR_MAX: usize = 256;
const TEXT_CHARS: usize = 1024;
const ARGS: usize = 16;
const HOOKS: usize = 256;
const SOUNDS: u32 = 64;
const BURSTS: u32 = 64;
const PARTICLES: u32 = 256;
const COMPONENTS: usize = 64;
const ATTACHES: usize = 256;
pub const COMPONENT_BYTES: u32 = 256;
const SCHEDULES: u32 = 256;
const MAX_SCHEDULE: u32 = 72_000;
const SENDS: u32 = 32;
pub const PACKET_BYTES: usize = 16 << 10;
/// Packets a mod is handed per channel and tick; the game drops the rest.
pub const PACKETS_PER_TICK: u32 = 64;

// ------------------------------------------------------------------ what the game provides

/// The game's side of one callback: the world it runs in, and for a command whoever ran it.
/// Everything the mod may reach goes through here; the sandbox checks counts and sizes first.
pub trait Game: Send {
    fn time(&mut self) -> u64;
    fn seed(&mut self) -> u64;
    fn random(&mut self) -> u64;
    fn dimension(&mut self) -> u32;
    fn get_block(&mut self, pos: [i32; 3]) -> u32;
    fn set_block(&mut self, pos: [i32; 3], state: u32) -> bool;
    fn light(&mut self, pos: [i32; 3]) -> u32;
    fn biome(&mut self, pos: [i32; 3]) -> u32;
    fn players(&mut self) -> Vec<u64>;
    fn entity(&mut self, id: u64) -> Option<EntityState>;
    fn near(&mut self, pos: [f64; 3], radius: f64) -> Vec<u64>;
    fn teleport(&mut self, id: u64, pos: [f64; 3]);
    fn push(&mut self, id: u64, velocity: [f64; 3]);
    fn damage(&mut self, id: u64, amount: f32, source: u64, cause: u32) -> bool;
    /// A new entity of the kind named `kind`; 0 if there is none.
    fn spawn(&mut self, kind: &str, pos: [f64; 3]) -> u64;
    /// Never a player.
    fn remove(&mut self, id: u64);
    fn creative(&mut self, player: u64) -> bool;
    fn give(&mut self, owner: u64, stack: ItemStack) -> ItemStack;
    fn spawn_item(&mut self, pos: [f64; 3], stack: ItemStack);
    fn tell(&mut self, player: u64, key: &str, args: &[Arg]);
    fn player_key(&mut self, player: u64) -> String;
    /// A command's answer to whoever ran it; nothing outside a command.
    fn reply(&mut self, key: &str, args: &[Arg]);

    fn block_name(&mut self, state: u32) -> String;
    fn block_state(&mut self, name: &str) -> Option<u32>;
    fn prop(&mut self, state: u32, prop: &str) -> Option<String>;
    fn with_prop(&mut self, state: u32, prop: &str, value: &str) -> Option<u32>;
    fn item_name(&mut self, id: u32) -> String;
    fn item_id(&mut self, name: &str) -> Option<u32>;
    fn entity_kind(&mut self, id: u32) -> String;
    fn entity_kind_id(&mut self, name: &str) -> Option<u32>;

    // 1.1. Defaults so a game written for 1.0 still builds: nothing carries components, no
    // sounds or particles.
    /// An entity's value of component `name` (a full name, one the mod declared).
    fn get_component(&mut self, _id: u64, _name: &str) -> Option<Vec<u8>> {
        None
    }
    /// Sets it; `value` is exactly the component's size. False if the entity doesn't carry it.
    fn set_component(&mut self, _id: u64, _name: &str, _value: &[u8]) -> bool {
        false
    }
    fn play_sound(&mut self, _sound: &str, _pos: [f64; 3], _volume: f32) {}
    fn spawn_particles(&mut self, _particle: &str, _pos: [f64; 3], _count: u32) {}

    // 1.2, defaults likewise.
    /// A scheduled tick for the block at `pos` in `ticks`.
    fn schedule(&mut self, _pos: [i32; 3], _ticks: u32) {}
    /// Bytes to a player's game on channel `channel` (a full name the mod registered).
    fn send(&mut self, _player: u64, _channel: &str, _data: &[u8]) {}
}

/// A game with nothing in it, for tools and benchmarks: every call answers empty.
pub struct NullGame;

impl Game for NullGame {
    fn time(&mut self) -> u64 {
        0
    }
    fn seed(&mut self) -> u64 {
        0
    }
    fn random(&mut self) -> u64 {
        0
    }
    fn dimension(&mut self) -> u32 {
        0
    }
    fn get_block(&mut self, _: [i32; 3]) -> u32 {
        0
    }
    fn set_block(&mut self, _: [i32; 3], _: u32) -> bool {
        false
    }
    fn light(&mut self, _: [i32; 3]) -> u32 {
        0
    }
    fn biome(&mut self, _: [i32; 3]) -> u32 {
        0
    }
    fn players(&mut self) -> Vec<u64> {
        Vec::new()
    }
    fn entity(&mut self, _: u64) -> Option<EntityState> {
        None
    }
    fn near(&mut self, _: [f64; 3], _: f64) -> Vec<u64> {
        Vec::new()
    }
    fn teleport(&mut self, _: u64, _: [f64; 3]) {}
    fn push(&mut self, _: u64, _: [f64; 3]) {}
    fn damage(&mut self, _: u64, _: f32, _: u64, _: u32) -> bool {
        false
    }
    fn spawn(&mut self, _: &str, _: [f64; 3]) -> u64 {
        0
    }
    fn remove(&mut self, _: u64) {}
    fn creative(&mut self, _: u64) -> bool {
        false
    }
    fn give(&mut self, _: u64, stack: ItemStack) -> ItemStack {
        stack
    }
    fn spawn_item(&mut self, _: [f64; 3], _: ItemStack) {}
    fn tell(&mut self, _: u64, _: &str, _: &[Arg]) {}
    fn player_key(&mut self, _: u64) -> String {
        String::new()
    }
    fn reply(&mut self, _: &str, _: &[Arg]) {}
    fn block_name(&mut self, _: u32) -> String {
        String::new()
    }
    fn block_state(&mut self, _: &str) -> Option<u32> {
        None
    }
    fn prop(&mut self, _: u32, _: &str) -> Option<String> {
        None
    }
    fn with_prop(&mut self, _: u32, _: &str, _: &str) -> Option<u32> {
        None
    }
    fn item_name(&mut self, _: u32) -> String {
        String::new()
    }
    fn item_id(&mut self, _: &str) -> Option<u32> {
        None
    }
    fn entity_kind(&mut self, _: u32) -> String {
        String::new()
    }
    fn entity_kind_id(&mut self, _: &str) -> Option<u32> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

/// Where the mods' log lines and the sandbox's own notes go: `(level, package, line)`.
pub type Logger = fn(Level, &str, &str);
static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Sends log lines to the game's log (once; the first wins). Without it they go to stderr.
pub fn set_logger(f: Logger) {
    let _ = LOGGER.set(f);
}

fn log(level: Level, package: &str, line: &str) {
    match LOGGER.get() {
        Some(f) => f(level, package, line),
        None => eprintln!("[{level:?}] mod:{package}: {line}"),
    }
}

// ------------------------------------------------------------------ what a mod registers

/// One registration of a mod's `init`, in order: its index is the callback's number.
#[derive(Clone, Debug, PartialEq)]
pub enum Hook {
    Command {
        name: String,
        usage: String,
        permission: u32,
    },
    Event(Event),
    Question(Question),
    Modify {
        chain: Chain,
        stage: u32,
    },
    WorldTick,
    /// 1.2: the package's own block or item, or channel, by full name.
    BlockUse(String),
    BlockNeighbor(String),
    BlockScheduled(String),
    ItemUse(String),
    Channel(String),
}

impl Hook {
    /// Called through `content-hooks` (1.2), which a component must then export.
    fn needs_content_hooks(&self) -> bool {
        matches!(self, Hook::BlockUse(_) | Hook::BlockNeighbor(_) | Hook::BlockScheduled(_) | Hook::ItemUse(_) | Hook::Channel(_))
    }
}

/// A declaration of `init` that calls nothing back (1.1): a component, or a kind carrying one.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    /// `name` is the full name, `<package>:<name>`.
    Component { name: String, size: u32, flags: ComponentFlags, default: Vec<u8> },
    /// `component` is the full name.
    Attach { kind: String, component: String },
}

/// What a mod registered in `init`: its numbered hooks and its declarations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Registered {
    pub hooks: Vec<Hook>,
    pub decls: Vec<Decl>,
}

/// A component's or command's own name: lowercase letters, digits and `_`, 1 to 32.
fn local_name_ok(name: &str) -> bool {
    command_name_ok(name)
}

/// A command name the game would take: lowercase letters, digits and `_`, 1 to 32.
fn command_name_ok(name: &str) -> bool {
    (1..=32).contains(&name.len())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// A usage string of Weft's shape, checked loosely here; the game's parser has the last word.
fn usage_ok(usage: &str) -> bool {
    usage.len() <= 512
        && usage
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " _<>[]:(),|".contains(c))
}

// ------------------------------------------------------------------ the store's state

struct State {
    package: String,
    game: Option<Box<dyn Game>>,
    /// The registrations while `init` runs.
    registering: Option<Registered>,
    /// The components the mod declared, by full name, with their sizes.
    components: Vec<(String, u32)>,
    /// The channels it registered, by full name (1.2).
    channels: Vec<String>,
    storage: Storage,
    limits: StoreLimits,
    counts: Counts,
}

/// What this call has used of its per-call allowances.
#[derive(Default)]
struct Counts {
    logs: u32,
    tells: u32,
    replies: u32,
    block_sets: u32,
    spawns: u32,
    sounds: u32,
    bursts: u32,
    /// Bytes `storage.set` was asked to write.
    stored: u64,
    schedules: u32,
    sends: u32,
}

impl State {
    fn new(package: &str, storage: Storage) -> State {
        State {
            package: package.to_string(),
            game: None,
            registering: None,
            components: Vec::new(),
            channels: Vec::new(),
            storage,
            limits: engine::limits(),
            counts: Counts::default(),
        }
    }

    fn game(&mut self) -> Option<&mut (dyn Game + 'static)> {
        self.game.as_deref_mut()
    }
}

fn clip(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Arguments a mod hands the game: few, and texts short.
fn clip_args(args: Vec<Arg>) -> Vec<Arg> {
    args.into_iter()
        .take(ARGS)
        .map(|a| match a {
            Arg::Text(s) => Arg::Text(clip(&s, TEXT_CHARS).to_string()),
            Arg::Key(s) => Arg::Key(clip(&s, 128).to_string()),
            Arg::Name(s) => Arg::Name(clip(&s, 128).to_string()),
            Arg::Float((v, d)) => Arg::Float((v, d.min(8))),
            other => other,
        })
        .collect()
}

fn finite(v: (f64, f64, f64)) -> Option<[f64; 3]> {
    let a = [v.0, v.1, v.2];
    a.iter().all(|x| x.is_finite()).then_some(a)
}

fn pos(p: types::BlockPos) -> [i32; 3] {
    [p.0, p.1, p.2]
}

impl types::Host for State {}

impl mod_log::Host for State {
    fn write(&mut self, level: mod_log::Level, message: String) {
        self.counts.logs += 1;
        if self.counts.logs > LOG_LINES {
            return;
        }
        let level = match level {
            mod_log::Level::Error => Level::Error,
            mod_log::Level::Warn => Level::Warn,
            mod_log::Level::Info => Level::Info,
            mod_log::Level::Debug => Level::Debug,
        };
        log(level, &self.package, clip(&message, LOG_CHARS));
    }
}

impl State {
    fn register(&mut self, hook: Hook) -> Result<u32, String> {
        let Some(reg) = self.registering.as_mut() else {
            return Err("registering is only possible while `init` runs".into());
        };
        let list = &mut reg.hooks;
        if list.len() >= HOOKS {
            return Err(format!("at most {HOOKS} registrations"));
        }
        list.push(hook);
        Ok(list.len() as u32 - 1)
    }

    fn declare(&mut self, decl: Decl) -> Result<(), String> {
        let Some(reg) = self.registering.as_mut() else {
            return Err("declaring is only possible while `init` runs".into());
        };
        let (components, attaches) = reg
            .decls
            .iter()
            .fold((0, 0), |(c, a), d| if matches!(d, Decl::Component { .. }) { (c + 1, a) } else { (c, a + 1) });
        match &decl {
            Decl::Component { name, size, .. } => {
                if components >= COMPONENTS {
                    return Err(format!("at most {COMPONENTS} components"));
                }
                if reg
                    .decls
                    .iter()
                    .any(|d| matches!(d, Decl::Component { name: n, .. } if n == name))
                {
                    return Err(format!("component `{name}` is declared twice"));
                }
                self.components.push((name.clone(), *size));
            }
            Decl::Attach { .. } => {
                if attaches >= ATTACHES {
                    return Err(format!("at most {ATTACHES} attaches"));
                }
            }
        }
        reg.decls.push(decl);
        Ok(())
    }

    /// `name` as a full name of the package's own content (`<package>:<path>`, `ruby` for
    /// short): callbacks go on the package's own blocks and items only.
    fn own(&self, name: &str, what: &str) -> Result<String, String> {
        let full = if name.contains(':') { name.to_string() } else { format!("{}:{name}", self.package) };
        let ok = full.len() <= 128
            && full.split_once(':').is_some_and(|(ns, path)| {
                ns == self.package
                    && !path.is_empty()
                    && path
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_/".contains(c))
            });
        if ok { Ok(full) } else { Err(format!("`{}` is no {what} of this package's own", clip(name, 64))) }
    }

    /// The full name and size of the mod's component called `name` (its own name, or the full one).
    fn component(&self, name: &str) -> Option<(String, u32)> {
        let full = if name.contains(':') { name.to_string() } else { format!("{}:{name}", self.package) };
        self.components.iter().find(|(n, _)| *n == full).cloned()
    }
}

impl registry::Host for State {
    fn command(&mut self, name: String, usage: String, permission: u32) -> Result<u32, String> {
        if !command_name_ok(&name) {
            return Err(format!("command `{}`: use 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)));
        }
        if !usage_ok(&usage) {
            return Err(format!("command `{name}`: its usage has characters a usage doesn't"));
        }
        if permission > 4 {
            return Err(format!("command `{name}`: permission is 0 to 4"));
        }
        if let Some(reg) = &self.registering
            && reg
                .hooks
                .iter()
                .any(|h| matches!(h, Hook::Command { name: n, .. } if *n == name))
        {
            return Err(format!("command `{name}` is registered twice"));
        }
        self.register(Hook::Command { name, usage, permission })
    }

    // Outside `init` these give `u32::MAX`, which names no callback.
    fn on(&mut self, event: Event) -> u32 {
        self.register(Hook::Event(event)).unwrap_or(u32::MAX)
    }

    fn intercept(&mut self, question: Question) -> u32 {
        self.register(Hook::Question(question)).unwrap_or(u32::MAX)
    }

    fn modify(&mut self, chain: Chain, stage: u32) -> u32 {
        self.register(Hook::Modify { chain, stage: stage.min(1000) })
            .unwrap_or(u32::MAX)
    }

    fn world_tick(&mut self) -> u32 {
        self.register(Hook::WorldTick).unwrap_or(u32::MAX)
    }

    fn on_block_use(&mut self, block: String) -> Result<u32, String> {
        let b = self.own(&block, "block")?;
        self.register(Hook::BlockUse(b))
    }

    fn on_block_neighbor(&mut self, block: String) -> Result<u32, String> {
        let b = self.own(&block, "block")?;
        self.register(Hook::BlockNeighbor(b))
    }

    fn on_block_scheduled(&mut self, block: String) -> Result<u32, String> {
        let b = self.own(&block, "block")?;
        self.register(Hook::BlockScheduled(b))
    }

    fn on_item_use(&mut self, item: String) -> Result<u32, String> {
        let i = self.own(&item, "item")?;
        self.register(Hook::ItemUse(i))
    }

    fn channel(&mut self, name: String) -> Result<u32, String> {
        if !local_name_ok(&name) {
            return Err(format!("channel `{}`: use 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)));
        }
        let full = format!("{}:{name}", self.package);
        if self.channels.contains(&full) {
            return Err(format!("channel `{name}` is registered twice"));
        }
        let id = self.register(Hook::Channel(full.clone()))?;
        self.channels.push(full);
        Ok(id)
    }

    fn component(&mut self, name: String, size: u32, flags: ComponentFlags, mut default: Vec<u8>) -> Result<(), String> {
        if !local_name_ok(&name) {
            return Err(format!("component `{}`: use 1 to 32 lowercase letters, digits and `_`", clip(&name, 40)));
        }
        if !(1..=COMPONENT_BYTES).contains(&size) {
            return Err(format!("component `{name}`: 1 to {COMPONENT_BYTES} bytes"));
        }
        if default.len() > size as usize {
            return Err(format!("component `{name}`: its default is bigger than {size} bytes"));
        }
        default.resize(size as usize, 0);
        let full = format!("{}:{name}", self.package);
        self.declare(Decl::Component { name: full, size, flags, default })
    }

    fn attach(&mut self, kind: String, component: String) -> Result<(), String> {
        let Some((full, _)) = State::component(self, &component) else {
            return Err(format!("attach: `{}` is no component of this mod (declare it first)", clip(&component, 64)));
        };
        let kind_ok = kind.len() <= 128
            && kind.split_once(':').is_some_and(|(ns, n)| {
                !ns.is_empty()
                    && !n.is_empty()
                    && kind
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_:/.".contains(c))
            });
        if !kind_ok {
            return Err(format!("attach: `{}` is not a full kind name (`core:player`, `base:pig`)", clip(&kind, 64)));
        }
        self.declare(Decl::Attach { kind, component: full })
    }
}

impl world::Host for State {
    fn time(&mut self) -> u64 {
        self.game().map_or(0, |g| g.time())
    }
    fn seed(&mut self) -> u64 {
        self.game().map_or(0, |g| g.seed())
    }
    fn random(&mut self) -> u64 {
        self.game().map_or(0, |g| g.random())
    }
    fn dimension(&mut self) -> u32 {
        self.game().map_or(0, |g| g.dimension())
    }
    fn get_block(&mut self, p: types::BlockPos) -> u32 {
        self.game().map_or(0, |g| g.get_block(pos(p)))
    }
    fn set_blocks(&mut self, changes: Vec<(types::BlockPos, u32)>) -> u32 {
        changes
            .into_iter()
            .take(BLOCK_SETS as usize + 1)
            .filter(|(p, s)| world::Host::set_block(self, *p, *s))
            .count() as u32
    }
    fn set_block(&mut self, p: types::BlockPos, state: u32) -> bool {
        self.counts.block_sets += 1;
        if self.counts.block_sets > BLOCK_SETS {
            return false;
        }
        self.game().is_some_and(|g| g.set_block(pos(p), state))
    }
    fn light(&mut self, p: types::BlockPos) -> u32 {
        self.game().map_or(0, |g| g.light(pos(p)))
    }
    fn biome(&mut self, p: types::BlockPos) -> u32 {
        self.game().map_or(0, |g| g.biome(pos(p)))
    }
    fn players(&mut self) -> Vec<u64> {
        self.game().map_or(Vec::new(), |g| g.players())
    }
    fn get_entity(&mut self, id: u64) -> Option<EntityState> {
        self.game().and_then(|g| g.entity(id))
    }
    fn near(&mut self, p: types::Vec3, radius: f64) -> Vec<u64> {
        let (Some(p), true) = (finite(p), radius.is_finite()) else { return Vec::new() };
        let mut v = self
            .game()
            .map_or(Vec::new(), |g| g.near(p, radius.clamp(0.0, NEAR_RADIUS)));
        v.truncate(NEAR_MAX);
        v
    }
    fn teleport(&mut self, id: u64, p: types::Vec3) {
        if let (Some(p), Some(g)) = (finite(p), self.game()) {
            g.teleport(id, p)
        }
    }
    fn push(&mut self, id: u64, v: types::Vec3) {
        if let (Some(v), Some(g)) = (finite(v), self.game()) {
            g.push(id, v.map(|x| x.clamp(-10.0, 10.0)))
        }
    }
    fn damage(&mut self, id: u64, amount: f32, source: u64, cause: u32) -> bool {
        if !amount.is_finite() || amount < 0.0 {
            return false;
        }
        self.game()
            .is_some_and(|g| g.damage(id, amount.min(1.0e6), source, cause))
    }
    fn spawn(&mut self, kind: String, p: types::Vec3) -> u64 {
        self.counts.spawns += 1;
        let Some(p) = finite(p) else { return 0 };
        if self.counts.spawns > SPAWNS || kind.len() > 128 {
            return 0;
        }
        self.game().map_or(0, |g| g.spawn(&kind, p))
    }
    fn remove(&mut self, id: u64) {
        if let Some(g) = self.game() {
            g.remove(id)
        }
    }
    fn creative(&mut self, player: u64) -> bool {
        self.game().is_some_and(|g| g.creative(player))
    }
    fn give(&mut self, owner: u64, stack: ItemStack) -> ItemStack {
        match self.game() {
            Some(g) => g.give(owner, stack),
            None => stack,
        }
    }
    fn spawn_item(&mut self, p: types::Vec3, stack: ItemStack) {
        if let (Some(p), Some(g)) = (finite(p), self.game()) {
            g.spawn_item(p, stack)
        }
    }
    fn tell(&mut self, player: u64, key: String, args: Vec<Arg>) {
        self.counts.tells += 1;
        if self.counts.tells > TELLS || key.len() > 128 {
            return;
        }
        let args = clip_args(args);
        if let Some(g) = self.game() {
            g.tell(player, &key, &args)
        }
    }
    fn player_key(&mut self, player: u64) -> String {
        self.game().map_or(String::new(), |g| g.player_key(player))
    }

    fn get_component(&mut self, id: u64, component: String) -> Option<Vec<u8>> {
        let (full, size) = State::component(self, &component)?;
        let mut v = self.game()?.get_component(id, &full)?;
        v.resize(size as usize, 0);
        Some(v)
    }
    fn set_component(&mut self, id: u64, component: String, mut value: Vec<u8>) -> bool {
        let Some((full, size)) = State::component(self, &component) else { return false };
        if value.len() > size as usize {
            return false;
        }
        value.resize(size as usize, 0);
        self.game()
            .is_some_and(|g| g.set_component(id, &full, &value))
    }
    fn play_sound(&mut self, sound: String, p: types::Vec3, volume: f32) {
        self.counts.sounds += 1;
        if self.counts.sounds > SOUNDS || sound.len() > 128 || !volume.is_finite() {
            return;
        }
        if let (Some(p), Some(g)) = (finite(p), self.game()) {
            g.play_sound(&sound, p, volume.clamp(0.0, 4.0))
        }
    }
    fn spawn_particles(&mut self, particle: String, p: types::Vec3, count: u32) {
        self.counts.bursts += 1;
        if self.counts.bursts > BURSTS || particle.len() > 128 || count == 0 {
            return;
        }
        if let (Some(p), Some(g)) = (finite(p), self.game()) {
            g.spawn_particles(&particle, p, count.min(PARTICLES))
        }
    }
    fn schedule(&mut self, p: types::BlockPos, ticks: u32) {
        self.counts.schedules += 1;
        if self.counts.schedules > SCHEDULES {
            return;
        }
        if let Some(g) = self.game() {
            g.schedule(pos(p), ticks.clamp(1, MAX_SCHEDULE))
        }
    }
    fn send(&mut self, player: u64, channel: String, data: Vec<u8>) {
        self.counts.sends += 1;
        if self.counts.sends > SENDS || data.len() > PACKET_BYTES {
            return;
        }
        let full = if channel.contains(':') { channel } else { format!("{}:{channel}", self.package) };
        if !self.channels.contains(&full) {
            return;
        }
        if let Some(g) = self.game() {
            g.send(player, &full, &data)
        }
    }
}

impl names::Host for State {
    fn block(&mut self, state: u32) -> String {
        self.game().map_or(String::new(), |g| g.block_name(state))
    }
    fn block_state(&mut self, name: String) -> Option<u32> {
        if name.len() > 256 {
            return None;
        }
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
        if name.len() > 256 {
            return None;
        }
        self.game().and_then(|g| g.item_id(&name))
    }
    fn entity_kind(&mut self, id: u32) -> String {
        self.game().map_or(String::new(), |g| g.entity_kind(id))
    }
    fn entity_kind_id(&mut self, name: String) -> Option<u32> {
        if name.len() > 256 {
            return None;
        }
        self.game().and_then(|g| g.entity_kind_id(&name))
    }
}

impl mod_storage::Host for State {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        self.storage.get(&key)
    }
    fn set(&mut self, key: String, value: Vec<u8>) -> Result<(), String> {
        self.counts.stored += value.len() as u64;
        if self.counts.stored > storage::MAX_WRITE_PER_CALL {
            return Err(format!("at most {} MiB written per call", storage::MAX_WRITE_PER_CALL >> 20));
        }
        self.storage.set(&key, value)
    }
    fn remove(&mut self, key: String) {
        self.storage.remove(&key)
    }
    fn keys(&mut self) -> Vec<String> {
        self.storage.keys()
    }
}

impl command::Host for State {
    fn reply(&mut self, key: String, args: Vec<Arg>) {
        self.counts.replies += 1;
        if self.counts.replies > REPLIES || key.len() > 128 {
            return;
        }
        let args = clip_args(args);
        if let Some(g) = self.game() {
            g.reply(&key, &args)
        }
    }
}

// ------------------------------------------------------------------ compiled mods

/// A compiled server half and what its `init` registers.
pub struct Mod {
    package: String,
    component: Component,
    linker: Arc<Linker<State>>,
    registered: Registered,
}

impl std::fmt::Debug for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mod")
            .field("package", &self.package)
            .field("registered", &self.registered)
            .finish()
    }
}

fn linker() -> Result<Arc<Linker<State>>, String> {
    static LINKER: OnceLock<Result<Arc<Linker<State>>, String>> = OnceLock::new();
    LINKER
        .get_or_init(|| {
            let mut linker = Linker::new(engine::engine()?);
            ServerMod::add_to_linker::<State, HasSelf<State>>(&mut linker, |s| s).map_err(|e| format!("{e:#}"))?;
            Ok(Arc::new(linker))
        })
        .clone()
}

/// Compiled components by the BLAKE3 of their bytes: a package used by several worlds' contents
/// compiles once.
fn compiled(wasm: &[u8]) -> Result<Component, String> {
    static CACHE: Mutex<Vec<([u8; 32], Component)>> = Mutex::new(Vec::new());
    let key = *blake3::hash(wasm).as_bytes();
    if let Some((_, c)) = CACHE.lock().unwrap().iter().find(|(k, _)| *k == key) {
        return Ok(c.clone());
    }
    let c = Component::new(engine::engine()?, wasm).map_err(|e| format!("not a component this game runs: {e:#}"))?;
    let mut cache = CACHE.lock().unwrap();
    if cache.len() >= 64 {
        cache.remove(0);
    }
    cache.push((key, c.clone()));
    Ok(c)
}

impl Mod {
    /// Compiles `wasm` (a component for `server-mod`) and runs its `init` once, with no world and
    /// storage of its own, to learn what it registers.
    pub fn load(package: &str, wasm: &[u8]) -> Result<Mod, String> {
        if wasm.len() > engine::MAX_COMPONENT_BYTES {
            return Err(format!("code/server.wasm is bigger than {} MiB", engine::MAX_COMPONENT_BYTES >> 20));
        }
        check_imports(wasm)?;
        let component = compiled(wasm)?;
        let linker = linker()?;
        let mut m = Mod { package: package.to_string(), component, linker, registered: Registered::default() };
        let live = m.instantiate(Storage::memory(), None)?;
        m.registered = live.registered;
        Ok(m)
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    /// What `init` registered: a hook's index is its callback number.
    pub fn hooks(&self) -> &[Hook] {
        &self.registered.hooks
    }

    /// What `init` declared (1.1): components and the kinds carrying them.
    pub fn decls(&self) -> &[Decl] {
        &self.registered.decls
    }

    /// A new instance with its `init` run.
    fn instantiate(&self, storage: Storage, game: Option<Box<dyn Game>>) -> Result<Live, String> {
        let engine = engine::engine()?;
        let mut store = Store::new(engine, State::new(&self.package, storage));
        store.limiter(|s| &mut s.limits);
        store.epoch_deadline_trap();
        store.set_epoch_deadline(engine::WATCHDOG_EPOCHS);
        store
            .set_fuel(engine::fuel::INIT)
            .map_err(|e| e.to_string())?;
        let instance = self
            .linker
            .instantiate(&mut store, &self.component)
            .map_err(|e| why(&e))?;
        let hooks = ServerMod::new(&mut store, &instance).map_err(|e| why(&e))?;
        let content = ContentHooks::find(&mut store, &self.component, &instance)?;
        store.data_mut().game = game;
        store.data_mut().registering = Some(Registered::default());
        let r = hooks.chiacraft_mods_hooks().call_init(&mut store);
        store.data_mut().game = None;
        let registered = store.data_mut().registering.take().unwrap_or_default();
        match r {
            Ok(Ok(())) if content.is_none() && registered.hooks.iter().any(Hook::needs_content_hooks) => {
                Err("init: it registers block, item or channel callbacks but exports no content-hooks (build it with the 1.2 SDK)".into())
            }
            Ok(Ok(())) => Ok(Live { store, bindings: Exports { hooks, content }, registered }),
            Ok(Err(e)) => Err(format!("init: {}", clip(&e, 512))),
            Err(e) => Err(format!("init: {}", why(&e))),
        }
    }
}

/// A component may import only this API, in a version this game has: anything else (WASI, a
/// newer 1.x, 2.0) is refused at load with a reason, before it is compiled.
fn check_imports(wasm: &[u8]) -> Result<(), String> {
    use chiacraft_packages::code;
    match code::api_of(wasm)? {
        Some(v) if !code::runs(v) => {
            Err(format!("it was built for API {}; this game runs {API}: a newer game runs it", code::text(Some(v))))
        }
        _ => Ok(()),
    }
}

/// Why a call failed, in words.
fn why(e: &wasmtime::Error) -> String {
    match e.downcast_ref::<Trap>() {
        Some(Trap::OutOfFuel) => "ran out of fuel".into(),
        Some(Trap::Interrupt) => format!("ran longer than {} ms", engine::WATCHDOG.as_millis()),
        _ => {
            let text = format!("{e:#}");
            if text.contains("forcing trap when growing memory") {
                format!("used more than {} MiB of memory", engine::MEMORY >> 20)
            } else {
                clip(&text, 512).to_string()
            }
        }
    }
}

struct Live {
    store: Store<State>,
    bindings: Exports,
    registered: Registered,
}

/// What a component exports: `hooks` always, `content-hooks` from 1.2 on.
struct Exports {
    hooks: ServerMod,
    content: Option<ContentHooks>,
}

impl Exports {
    fn chiacraft_mods_hooks(&self) -> &bindings::exports::chiacraft::mods::hooks::Guest {
        self.hooks.chiacraft_mods_hooks()
    }
}

/// `content-hooks` (1.2), found at run time so that older components, which lack it, still
/// instantiate through the game's `host` world.
struct ContentHooks {
    block_use: TypedFunc<(u32, BlockUseEvent), (bool,)>,
    block_update: TypedFunc<(u32, BlockUpdateEvent), ()>,
    item_use: TypedFunc<(u32, ItemUseEvent), (ItemUseResult,)>,
    packet: TypedFunc<(u32, u64, Vec<u8>), ()>,
}

impl ContentHooks {
    fn find(
        store: &mut Store<State>,
        component: &Component,
        instance: &wasmtime::component::Instance,
    ) -> Result<Option<ContentHooks>, String> {
        let engine = engine::engine()?;
        let Some(name) = component
            .component_type()
            .exports(engine)
            .map(|(n, _)| n.to_string())
            .find(|n| n.starts_with("chiacraft:mods/content-hooks@1."))
        else {
            return Ok(None);
        };
        let missing = |f: &str| format!("its content-hooks lacks {f}");
        let iface = instance
            .get_export_index(&mut *store, None, &name)
            .ok_or_else(|| missing("itself"))?;
        let func = |store: &mut Store<State>, f: &str| {
            instance
                .get_export_index(&mut *store, Some(&iface), f)
                .ok_or_else(|| missing(f))
        };
        let (u, b, i, p) = (func(store, "block-use")?, func(store, "block-update")?, func(store, "item-use")?, func(store, "packet")?);
        let typed = |e: wasmtime::Error| format!("its content-hooks doesn't fit API {API}: {}", why(&e));
        Ok(Some(ContentHooks {
            block_use: instance.get_typed_func(&mut *store, u).map_err(typed)?,
            block_update: instance.get_typed_func(&mut *store, b).map_err(typed)?,
            item_use: instance.get_typed_func(&mut *store, i).map_err(typed)?,
            packet: instance.get_typed_func(&mut *store, p).map_err(typed)?,
        }))
    }
}

/// Calls a typed export and lets the component clean up after it.
fn typed<P, R>(f: &TypedFunc<P, R>, store: &mut Store<State>, params: P) -> wasmtime::Result<R>
where
    P: wasmtime::component::ComponentNamedList + wasmtime::component::Lower + Send + Sync,
    R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift + Send + Sync,
{
    let r = f.call(&mut *store, params)?;
    f.post_return(&mut *store)?;
    Ok(r)
}

// ------------------------------------------------------------------ a mod in one world

/// A mod running in one world: started on its first call there, started again after a strike,
/// stopped after [`STRIKES`] strikes within [`STRIKE_WINDOW`].
pub struct Instance {
    m: Arc<Mod>,
    storage_dir: Option<PathBuf>,
    live: Option<Live>,
    strikes: VecDeque<Instant>,
    stopped: Option<String>,
}

impl Instance {
    /// The mod `m` in a world whose save keeps its storage in `storage_dir` (none: in memory).
    pub fn new(m: Arc<Mod>, storage_dir: Option<PathBuf>) -> Instance {
        Instance { m, storage_dir, live: None, strikes: VecDeque::new(), stopped: None }
    }

    /// Why the mod no longer runs in this world, if it doesn't.
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
            log(Level::Error, &package, &format!("{why}; stopped in this world after {STRIKES} strikes"));
            self.stopped = Some(why);
        } else {
            log(Level::Warn, &package, &format!("{why}; starting it over (strike {} of {STRIKES})", self.strikes.len()));
        }
    }

    /// Runs `f` on the live instance with `game` installed and `fuel` to spend. `None` if the mod
    /// doesn't run here or the call failed (a strike).
    fn call<T>(&mut self, game: Box<dyn Game>, fuel: u64, f: impl FnOnce(&Exports, &mut Store<State>) -> wasmtime::Result<T>) -> Option<T> {
        if self.stopped.is_some() {
            return None;
        }
        let live = self.live.as_mut()?;
        let store = &mut live.store;
        let data = store.data_mut();
        data.game = Some(game);
        data.counts = Counts::default();
        store.set_epoch_deadline(engine::WATCHDOG_EPOCHS);
        if store.set_fuel(fuel).is_err() {
            return None;
        }
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

    /// Starts the mod here (`init` with this world) if it isn't running; false if it doesn't
    /// run here.
    pub fn start(&mut self, game: Box<dyn Game>) -> bool {
        if self.stopped.is_some() {
            return false;
        }
        if self.live.is_some() {
            return true;
        }
        let storage = match &self.storage_dir {
            Some(d) => Storage::open(d.clone()),
            None => Ok(Storage::memory()),
        };
        match storage.and_then(|s| self.m.instantiate(s, Some(game))) {
            Ok(live) if live.registered != self.m.registered => {
                let why = "registered other things than when it loaded".to_string();
                log(Level::Error, &self.m.package, &format!("{why}; stopped in this world"));
                self.stopped = Some(why);
                false
            }
            Ok(live) => {
                self.live = Some(live);
                true
            }
            Err(e) => {
                self.strike(e);
                false
            }
        }
    }

    /// One of the mod's commands.
    pub fn command(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, call: &Invocation) {
        self.ensure(&game);
        self.call(game(), engine::fuel::COMMAND, |b, s| b.chiacraft_mods_hooks().call_command(s, callback, call));
    }

    /// A batch of an event.
    pub fn event(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, batch: &[EventValue]) {
        self.ensure(&game);
        self.call(game(), engine::fuel::EVENT, |b, s| b.chiacraft_mods_hooks().call_event(s, callback, batch));
    }

    /// A question; a mod that fails abstains.
    pub fn question(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, q: Query) -> Verdict {
        self.ensure(&game);
        self.call(game(), engine::fuel::QUESTION, |b, s| b.chiacraft_mods_hooks().call_question(s, callback, q))
            .unwrap_or(Verdict::Abstain)
    }

    /// A chain step; a mod that fails leaves the value as it was, as does one giving a number
    /// that isn't finite.
    pub fn modify(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, q: Query, base: f64, value: f64) -> f64 {
        self.ensure(&game);
        self.call(game(), engine::fuel::MODIFY, |b, s| {
            b.chiacraft_mods_hooks()
                .call_modify(s, callback, q, base, value)
        })
        .filter(|v| v.is_finite())
        .unwrap_or(value)
    }

    /// A world tick.
    pub fn world_tick(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32) {
        self.ensure(&game);
        self.call(game(), engine::fuel::TICK, |b, s| b.chiacraft_mods_hooks().call_world_tick(s, callback));
    }

    /// A player used one of the package's blocks (1.2): whether the use did something.
    pub fn block_use(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, used: BlockUseEvent) -> bool {
        self.ensure(&game);
        self.call(game(), engine::fuel::EVENT, |b, s| match &b.content {
            Some(c) => typed(&c.block_use, s, (callback, used)).map(|(v,)| v),
            None => Ok(false),
        })
        .unwrap_or(false)
    }

    /// A neighbour of one of the package's blocks changed, or its scheduled tick came (1.2).
    pub fn block_update(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, update: BlockUpdateEvent) {
        self.ensure(&game);
        self.call(game(), engine::fuel::TICK, |b, s| match &b.content {
            Some(c) => typed(&c.block_update, s, (callback, update)),
            None => Ok(()),
        });
    }

    /// A player used one of the package's items (1.2); a failed call did nothing.
    pub fn item_use(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, used: ItemUseEvent) -> ItemUseResult {
        self.ensure(&game);
        self.call(game(), engine::fuel::EVENT, |b, s| match &b.content {
            Some(c) => typed(&c.item_use, s, (callback, used)).map(|(v,)| v),
            None => Ok(ItemUseResult { handled: false, stack: None }),
        })
        .unwrap_or(ItemUseResult { handled: false, stack: None })
    }

    /// Bytes a player's game sent on one of the mod's channels (1.2). The caller keeps to
    /// [`PACKETS_PER_TICK`] and [`PACKET_BYTES`].
    pub fn packet(&mut self, game: impl Fn() -> Box<dyn Game>, callback: u32, player: u64, data: &[u8]) {
        if data.len() > PACKET_BYTES {
            return;
        }
        self.ensure(&game);
        self.call(game(), engine::fuel::EVENT, |b, s| match &b.content {
            Some(c) => typed(&c.packet, s, (callback, player, data.to_vec())),
            None => Ok(()),
        });
    }

    fn ensure(&mut self, game: &impl Fn() -> Box<dyn Game>) {
        if self.live.is_none() {
            self.start(game());
        }
    }
}

/// Turns a `wasm32-unknown-unknown` module built with the mod SDK into the component the game
/// loads.
#[cfg(feature = "pack")]
pub fn componentize(core: &[u8]) -> Result<Vec<u8>, String> {
    wit_component::ComponentEncoder::default()
        .module(core)
        .and_then(|e| e.validate(true).encode())
        .map_err(|e| format!("{e:#}"))
}

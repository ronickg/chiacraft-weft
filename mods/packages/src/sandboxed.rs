//! A package's sandboxed server half (chiacraft plan, phase 6; `docs/sandboxed-mods.md`):
//! `code/server.wasm`, run by `chiacraft-sandbox`. Its registrations become this mod's own
//! through Weft's SDK, in the package's slot; each callback finds the mod's instance for the
//! world it runs in (started there on first use, its storage in that world's save) and hands it a
//! [`WorldGame`]: the world calls the sandbox lets it make, answered with the SDK.
//!
//! A mod's own calls into the world can ask questions or run value chains that it intercepts
//! itself; such a nested callback is skipped (the mod is busy), never re-entered.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use chiacraft_sandbox::{
    self as sandbox, Arg as WArg, BlockUpdateEvent, BlockUseEvent, Decl, EntityState as WEntity, EventValue, Game, Hook, Instance,
    Invocation, ItemUseEvent, Mod, Value,
};
use weft_sdk::abi::{self, Str};
use weft_sdk::*;

/// Commands of the engine a sandboxed mod may not take (Weft lets a mod's command replace the
/// engine's of the same name: a downloaded mod must not replace `/op`).
const RESERVED: &[&str] = &[
    "help", "tp", "dim", "give", "time", "summon", "kill", "setblock", "insert", "block", "gamemode", "heal", "tick", "regions", "seed",
    "weather", "op", "deop", "rule", "log", "gizmo", "music",
];

/// Worlds a package's mod runs in at once: a world opened again is a new run.
const RUNS_KEPT: usize = 4;

/// Server to a client: a line from a sandboxed mod, `key` then its arguments, a line each
/// (`t<text>`, `k<key>`, `i<int>`, `f<decimals> <float>`, `n<name>`, `m<item>`).
pub static SAY: Channel<String> = Channel::named("say");

/// The mod slot of the package being registered (`chiacraft_register_package`).
pub static CURRENT_SLOT: AtomicU32 = AtomicU32::new(u32::MAX);
/// Each package's slot, so its mod's lines log under its own name (the SDK keeps one slot per
/// library, the last package's).
static SLOTS: Mutex<Vec<(String, u32)>> = Mutex::new(Vec::new());

pub fn logger(level: sandbox::Level, package: &str, line: &str) {
    let level = match level {
        sandbox::Level::Error => abi::log_level::ERROR,
        sandbox::Level::Warn => abi::log_level::WARN,
        sandbox::Level::Info => abi::log_level::INFO,
        sandbox::Level::Debug => abi::log_level::DEBUG,
    };
    let slot = SLOTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(p, _)| p == package)
        .map(|(_, s)| *s);
    match slot {
        Some(me) => unsafe { (host().log_at)(abi::ModId(me), level, Str::new(""), Str::new(line)) },
        None => weft_sdk::__log(level, package, format_args!("{line}")),
    }
}

/// A mod's channel: its full name, the SDK's handle, and (tick, packets handed over in it).
type ModChannel = (String, &'static Channel<Vec<u8>>, Mutex<(u64, u32)>);

/// A package's mod, registered with one content build, in each world it has run in.
pub struct Runs {
    m: Arc<Mod>,
    package: String,
    runs: Mutex<Vec<(u64, Instance)>>,
    /// The mod's components (1.1), registered with the content this `Runs` belongs to.
    comps: Comps,
    /// Sounds and particles the mod has named, as the SDK's handles (made once each).
    effects: Mutex<Vec<(String, Effect)>>,
    /// Its channels (1.2), by full name, with the tick and count of packets handed over.
    channels: Vec<ModChannel>,
    /// The block and item callbacks the package's builders took (by callback number).
    attached: Mutex<Vec<u32>>,
    /// The API its code was built for (`1.2`).
    api: String,
}

/// Every package's mod in every content built, for `/mods`.
static ALL: Mutex<Vec<std::sync::Weak<Runs>>> = Mutex::new(Vec::new());

/// A sandboxed mod's component as the SDK holds it: bytes in a size of the next bucket up.
#[derive(Clone, Copy)]
enum Comp {
    B8(&'static Component<[u8; 8]>),
    B16(&'static Component<[u8; 16]>),
    B32(&'static Component<[u8; 32]>),
    B64(&'static Component<[u8; 64]>),
    B128(&'static Component<[u8; 128]>),
    B256(&'static Component<[u8; 256]>),
}

type Comps = Vec<(String, Comp)>;

#[derive(Clone, Copy)]
enum Effect {
    Sound(&'static Sound),
    Particle(&'static Particle),
}

/// At most this many sound and particle names a mod may use.
const EFFECTS: usize = 256;

fn padded<const N: usize>(default: &[u8]) -> [u8; N] {
    let mut v = [0u8; N];
    let n = default.len().min(N);
    v[..n].copy_from_slice(&default[..n]);
    v
}

/// Registers a component of `size` bytes in the smallest bucket that holds it.
fn register_component(r: &mut Registry, name: &str, size: u32, default: &[u8], flags: u32) -> Comp {
    fn one<const N: usize>(r: &mut Registry, name: &'static str, default: &[u8], flags: u32) -> &'static Component<[u8; N]> {
        let c: &'static Component<[u8; N]> = Box::leak(Box::new(Component::named(name)));
        r.entity_component(c, padded::<N>(default), flags);
        c
    }
    let name: &'static str = Box::leak(name.to_string().into_boxed_str());
    match size {
        0..=8 => Comp::B8(one(r, name, default, flags)),
        9..=16 => Comp::B16(one(r, name, default, flags)),
        17..=32 => Comp::B32(one(r, name, default, flags)),
        33..=64 => Comp::B64(one(r, name, default, flags)),
        65..=128 => Comp::B128(one(r, name, default, flags)),
        _ => Comp::B256(one(r, name, default, flags)),
    }
}

/// Every sandboxed component registered in this game, newest first: a client half reads the
/// player's own value through them (its bucket is known only here).
static COMPONENTS: Mutex<Vec<(String, Comp)>> = Mutex::new(Vec::new());

/// The player's own value of the sandboxed component `name` on the client, its bucket's bytes.
pub fn client_component(c: Client, name: &str) -> Option<Vec<u8>> {
    let comp = COMPONENTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, c)| *c)?;
    let v = c.view();
    match comp {
        Comp::B8(c) => v.component(c).map(|b| b.to_vec()),
        Comp::B16(c) => v.component(c).map(|b| b.to_vec()),
        Comp::B32(c) => v.component(c).map(|b| b.to_vec()),
        Comp::B64(c) => v.component(c).map(|b| b.to_vec()),
        Comp::B128(c) => v.component(c).map(|b| b.to_vec()),
        Comp::B256(c) => v.component(c).map(|b| b.to_vec()),
    }
}

fn attach_component(r: &mut Registry, kind: &str, c: Comp) {
    let kind = EntityKind::generated(kind.to_string());
    match c {
        Comp::B8(c) => r.attach(kind, c),
        Comp::B16(c) => r.attach(kind, c),
        Comp::B32(c) => r.attach(kind, c),
        Comp::B64(c) => r.attach(kind, c),
        Comp::B128(c) => r.attach(kind, c),
        Comp::B256(c) => r.attach(kind, c),
    }
}

fn get_component(w: &World, id: u64, c: Comp) -> Option<Vec<u8>> {
    match c {
        Comp::B8(c) => w.component(id, c).map(|v| v.to_vec()),
        Comp::B16(c) => w.component(id, c).map(|v| v.to_vec()),
        Comp::B32(c) => w.component(id, c).map(|v| v.to_vec()),
        Comp::B64(c) => w.component(id, c).map(|v| v.to_vec()),
        Comp::B128(c) => w.component(id, c).map(|v| v.to_vec()),
        Comp::B256(c) => w.component(id, c).map(|v| v.to_vec()),
    }
}

fn set_component(w: &World, id: u64, c: Comp, value: &[u8]) -> bool {
    fn one<const N: usize>(w: &World, id: u64, c: &'static Component<[u8; N]>, value: &[u8]) -> bool {
        match w.component_mut(id, c) {
            Some(v) => {
                *v = padded::<N>(value);
                true
            }
            None => false,
        }
    }
    match c {
        Comp::B8(c) => one(w, id, c, value),
        Comp::B16(c) => one(w, id, c, value),
        Comp::B32(c) => one(w, id, c, value),
        Comp::B64(c) => one(w, id, c, value),
        Comp::B128(c) => one(w, id, c, value),
        Comp::B256(c) => one(w, id, c, value),
    }
}

/// Weft's flags for the sandbox's.
fn flag_bits(f: sandbox::ComponentFlags) -> u32 {
    use sandbox::ComponentFlags as F;
    [
        (F::SAVE, component_flags::SAVE),
        (F::SYNC_OWNER, component_flags::SYNC_OWNER),
        (F::SYNC_WATCHERS, component_flags::SYNC_WATCHERS),
        (F::KEEP, component_flags::KEEP_ON_RESPAWN),
    ]
    .into_iter()
    .filter(|(b, _)| (f & *b) == *b)
    .fold(0, |acc, (_, bit)| acc | bit)
}

impl Runs {
    /// Whether one more packet on `channel` may reach the mod this tick.
    fn packet_allowed(&self, channel: &str, tick: u64) -> bool {
        let Some((_, _, count)) = self.channels.iter().find(|(n, ..)| n == channel) else { return false };
        let mut c = count.lock().unwrap_or_else(|e| e.into_inner());
        if c.0 != tick {
            *c = (tick, 0);
        }
        c.1 += 1;
        c.1 <= sandbox::PACKETS_PER_TICK
    }
}

thread_local! {
    /// Packages whose mods this thread is calling now, by their `Runs`' address.
    static BUSY: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

impl Runs {
    /// Runs `f` on the instance for the world this callback runs in. Skipped when the mod's own
    /// call led here, and outside a server. Inside another sandboxed call (one mod's world call
    /// asking a question another mod hooks) it never waits for the lock: two mods on two region
    /// threads could otherwise each wait for the other's. Busy, it skips the mod.
    fn with<T>(&self, f: impl FnOnce(&mut Instance) -> T) -> Option<T> {
        let me = self as *const Runs as usize;
        let nested = BUSY.with(|b| {
            let b = b.borrow();
            if b.contains(&me) { None } else { Some(!b.is_empty()) }
        })?;
        let (run, dir) = world()?;
        let mut runs = if nested {
            match self.runs.try_lock() {
                Ok(g) => g,
                Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => return None,
            }
        } else {
            self.runs.lock().unwrap_or_else(|e| e.into_inner())
        };
        BUSY.with(|b| b.borrow_mut().push(me));
        struct Done(usize);
        impl Drop for Done {
            fn drop(&mut self) {
                BUSY.with(|b| b.borrow_mut().retain(|x| *x != self.0));
            }
        }
        let _done = Done(me);
        let i = match runs.iter().position(|(r, _)| *r == run) {
            Some(i) => i,
            None => {
                if runs.len() >= RUNS_KEPT {
                    runs.remove(0);
                }
                let storage = dir.map(|d| d.join("mods").join("packages").join(&self.package));
                runs.push((run, Instance::new(self.m.clone(), storage)));
                runs.len() - 1
            }
        };
        Some(f(&mut runs[i].1))
    }
}

/// The run and folder of the world this thread's server runs, from the engine's bridge.
fn world() -> Option<(u64, Option<PathBuf>)> {
    let a = crate::ask("world", "");
    let (run, dir) = a.split_once('\t')?;
    let run = run.parse().ok()?;
    Some((run, (!dir.is_empty()).then(|| PathBuf::from(dir))))
}

/// Loads `dir/code/server.wasm`, if the package has one, and registers its components (before
/// the package's own content, whose blocks and items then take its callbacks: [`on_block`],
/// [`on_item`]; then [`register`] for the rest). Mistakes are load errors, and the package is
/// left out.
pub fn load(r: &mut Registry, package: &str, dir: &Path) -> Result<Option<Arc<Runs>>, String> {
    let path = dir.join(chiacraft_packages::identity::SERVER_CODE);
    if !path.is_file() {
        return Ok(None);
    }
    let slot = CURRENT_SLOT.load(Ordering::Relaxed);
    {
        let mut slots = SLOTS.lock().unwrap_or_else(|e| e.into_inner());
        slots.retain(|(p, _)| p != package);
        slots.push((package.to_string(), slot));
    }
    let wasm = std::fs::read(&path).map_err(|e| format!("{}: {e}", chiacraft_packages::identity::SERVER_CODE))?;
    let m = Mod::load(package, &wasm).map_err(|e| format!("{}: {e}", chiacraft_packages::identity::SERVER_CODE))?;
    for h in m.hooks() {
        if let Hook::Command { name, .. } = h
            && RESERVED.contains(&name.as_str())
        {
            return Err(format!("{}: the command /{name} is the game's", chiacraft_packages::identity::SERVER_CODE));
        }
    }
    // Components first (1.1): they are content, registered with the package's.
    let mut comps: Comps = Vec::new();
    for d in m.decls() {
        match d {
            Decl::Component { name, size, flags, default } => {
                let c = register_component(r, name, *size, default, flag_bits(*flags));
                comps.push((name.clone(), c));
                COMPONENTS
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(0, (name.clone(), c));
            }
            Decl::Attach { kind, component } => {
                if let Some((_, c)) = comps.iter().find(|(n, _)| n == component) {
                    attach_component(r, kind, *c);
                }
            }
        }
    }
    let channels = m
        .hooks()
        .iter()
        .filter_map(|h| match h {
            Hook::Channel(name) => {
                let c: &'static Channel<Vec<u8>> = Box::leak(Box::new(Channel::named(Box::leak(name.clone().into_boxed_str()))));
                Some((name.clone(), c, Mutex::new((0, 0))))
            }
            _ => None,
        })
        .collect();
    let api = chiacraft_packages::code::api_of(&wasm)
        .ok()
        .flatten()
        .map_or_else(|| "1.0".into(), |v| chiacraft_packages::code::text(Some(v)));
    let runs = Arc::new(Runs {
        m: Arc::new(m),
        package: package.to_string(),
        runs: Mutex::new(Vec::new()),
        comps,
        effects: Mutex::new(Vec::new()),
        channels,
        attached: Mutex::new(Vec::new()),
        api,
    });
    let mut all = ALL.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|w| w.strong_count() > 0);
    all.push(Arc::downgrade(&runs));
    Ok(Some(runs))
}

/// `/mods [resume <package>]` (slice 6): the sandboxed server halves running in this world, what
/// they cost and their strikes; an admin resumes one that stopped. Registered in this mod's own
/// `init`, the same in every game.
pub fn register_command(r: &mut Registry) {
    r.command("mods", "[action:word(resume)] [package:word]", |cmd| {
        let Some((run, _)) = world() else { return };
        let all: Vec<Arc<Runs>> = ALL
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter_map(|w| w.upgrade())
            .collect();
        if cmd.text("action") == Some("resume") {
            if cmd.world().permission(cmd.player()) < permission::ADMIN {
                cmd.reply("packages.mods.admin", &[]);
                return;
            }
            let wanted = cmd.text("package").unwrap_or("").to_string();
            let mut done = false;
            for runs in all.iter().filter(|r| r.package == wanted) {
                let mut list = runs.runs.lock().unwrap_or_else(|e| e.into_inner());
                if let Some((_, inst)) = list.iter_mut().find(|(r, _)| *r == run) {
                    inst.resume();
                    done = true;
                }
            }
            let key = if done { "packages.mods.resumed" } else { "packages.mods.unknown" };
            cmd.reply(key, &[Arg::Text(&wanted)]);
            return;
        }
        let mut any = false;
        for runs in &all {
            let list = runs.runs.lock().unwrap_or_else(|e| e.into_inner());
            let Some((_, inst)) = list.iter().find(|(r, _)| *r == run) else { continue };
            any = true;
            let st = inst.stats();
            cmd.reply(
                "packages.mods.line",
                &[
                    Arg::Text(&runs.package),
                    Arg::Text(&runs.api),
                    Arg::Int(st.calls as i64),
                    Arg::Float(st.average_us(), 1),
                    Arg::Int(st.slowest.as_micros() as i64),
                    Arg::Int(st.strikes as i64),
                ],
            );
            if let Some(why) = &st.stopped {
                cmd.reply("packages.mods.stopped", &[Arg::Text(&runs.package), Arg::Text(why)]);
            }
        }
        if !any {
            cmd.reply("packages.mods.none", &[]);
        }
    })
    .permission(permission::EVERYONE);
}

/// The package's own block `id` takes the mod's callbacks for it (1.2).
pub fn on_block<'a>(mut b: BlockBuilder<'a>, runs: &Arc<Runs>, id: &str) -> BlockBuilder<'a> {
    for (i, h) in runs.m.hooks().iter().enumerate() {
        let cb = i as u32;
        let rs = runs.clone();
        match h {
            Hook::BlockUse(name) if name == id => {
                b = b.on_use(move |u| {
                    let (w, ev) = (u.world(), BlockUseEvent { player: u.player(), pos: t3(u.pos()), state: u.state(), face: u.face().face() });
                    rs.with(|inst| inst.block_use(|| Box::new(WorldGame::of(w, &rs)), cb, ev))
                        .unwrap_or(false)
                });
            }
            Hook::BlockNeighbor(name) if name == id => {
                b = b.on_neighbor(move |e| block_update(&rs, cb, e));
            }
            Hook::BlockScheduled(name) if name == id => {
                b = b.on_scheduled(move |e| block_update(&rs, cb, e));
            }
            _ => continue,
        }
        runs.attached
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(cb);
    }
    b
}

fn block_update(runs: &Arc<Runs>, cb: u32, e: &mut BlockEvent) {
    let w = e.world();
    let ev = BlockUpdateEvent { pos: t3(e.pos()), state: e.state(), other: t3(e.neighbor()), entity: e.entity() };
    runs.with(|inst| inst.block_update(|| Box::new(WorldGame::of(w, runs)), cb, ev));
}

/// The package's own item `id` takes the mod's use callback for it (1.2).
pub fn on_item<'a>(mut i: ItemBuilder<'a>, runs: &Arc<Runs>, id: &str) -> ItemBuilder<'a> {
    for (n, h) in runs.m.hooks().iter().enumerate() {
        let Hook::ItemUse(name) = h else { continue };
        if name != id {
            continue;
        }
        let cb = n as u32;
        let rs = runs.clone();
        i = i.on_use(move |u| {
            let w = u.world();
            let (player, held) = (u.player(), u.stack());
            let ev = ItemUseEvent {
                player,
                stack: stack(held),
                slot: w.selected_slot(player),
                target: u.target_block().map(|(p, face)| (t3(p), face)),
            };
            let Some(result) = rs.with(|inst| inst.item_use(|| Box::new(WorldGame::of(w, &rs)), cb, ev)) else { return false };
            // What the hand holds: only a real item, or nothing; what the stack carries stays.
            if let Some(s) = result.stack {
                let next = if s.count == 0 {
                    ItemStack::EMPTY
                } else if known_item(s.item) {
                    ItemStack { item: s.item, count: s.count.min(64 * 64), data: if s.item == held.item { held.data } else { 0 } }
                } else {
                    held
                };
                u.set_stack(next);
            }
            result.handled
        });
        runs.attached
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(cb);
    }
    i
}

/// Registers the rest of what the mod registered (after the package's content), and refuses a
/// block or item callback for something the package doesn't have.
pub fn register(r: &mut Registry, runs: &Arc<Runs>) -> Result<(), String> {
    let attached = runs
        .attached
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    for (i, hook) in runs.m.hooks().to_vec().into_iter().enumerate() {
        let cb = i as u32;
        let runs = runs.clone();
        match hook {
            Hook::BlockUse(name) | Hook::BlockNeighbor(name) | Hook::BlockScheduled(name) if !attached.contains(&cb) => {
                return Err(format!("{}: `{name}` is no block of this package", chiacraft_packages::identity::SERVER_CODE));
            }
            Hook::ItemUse(name) if !attached.contains(&cb) => {
                return Err(format!("{}: `{name}` is no item of this package", chiacraft_packages::identity::SERVER_CODE));
            }
            Hook::BlockUse(_) | Hook::BlockNeighbor(_) | Hook::BlockScheduled(_) | Hook::ItemUse(_) => {}
            Hook::Channel(name) => {
                let Some((_, channel, _)) = runs.channels.iter().find(|(n, ..)| *n == name) else { continue };
                r.packet(*channel, move |p, data| {
                    let (w, player) = (p.world(), p.player());
                    if data.len() > sandbox::PACKET_BYTES || !runs.packet_allowed(&name, w.game_time()) {
                        return;
                    }
                    runs.with(|inst| inst.packet(|| Box::new(WorldGame::of(w, &runs)), cb, player, &data));
                });
            }
            Hook::Command { name, usage, permission } => {
                let params = params(&usage);
                r.command(&name, &usage, move |cmd| {
                    let call = invocation(cmd, &params);
                    let w = cmd.world();
                    let cmd_ptr = cmd as *mut ServerCommand as usize;
                    runs.with(|inst| inst.command(|| Box::new(WorldGame { w, cmd: cmd_ptr, runs: runs.clone() }), cb, &call));
                })
                .permission(permission);
            }
            Hook::Event(e) => event(r, e, cb, runs),
            Hook::Question(q) => {
                let question = match q {
                    sandbox::Question::BreakBlock => actions::BREAK_BLOCK.as_ref(),
                    sandbox::Question::PlaceBlock => actions::PLACE_BLOCK.as_ref(),
                    sandbox::Question::UseBlock => actions::USE_BLOCK.as_ref(),
                    sandbox::Question::UseItem => actions::USE_ITEM.as_ref(),
                    sandbox::Question::InteractEntity => actions::INTERACT_ENTITY.as_ref(),
                    sandbox::Question::Attack => &question::ATTACK,
                    sandbox::Question::Hurt => actions::HURT.as_ref(),
                    sandbox::Question::PickupItem => actions::PICKUP_ITEM.as_ref(),
                    sandbox::Question::EnterDimension => actions::ENTER_DIMENSION.as_ref(),
                    sandbox::Question::NaturalSpawn => &question::NATURAL_SPAWN,
                };
                r.intercept(question, move |a| {
                    let w = a.world();
                    let q = query(a.query());
                    match runs.with(|inst| inst.question(|| Box::new(WorldGame::of(w, &runs)), cb, q)) {
                        Some(sandbox::Verdict::Allow) => Verdict::Allow,
                        Some(sandbox::Verdict::Deny) => Verdict::Deny,
                        _ => Verdict::Abstain,
                    }
                });
            }
            Hook::Modify { chain: c, stage } => {
                let chain = match c {
                    sandbox::Chain::Damage => &chain::DAMAGE,
                    sandbox::Chain::AttackDamage => &chain::ATTACK_DAMAGE,
                };
                r.modify(chain, stage, move |m| {
                    let w = m.world();
                    let q = query(m.query());
                    let (base, value) = (m.base, m.value);
                    if let Some(v) = runs.with(|inst| inst.modify(|| Box::new(WorldGame::of(w, &runs)), cb, q, base, value)) {
                        m.value = v;
                    }
                });
            }
            Hook::WorldTick => r.world_tick(move |w| {
                let w = *w;
                runs.with(|inst| inst.world_tick(|| Box::new(WorldGame::of(w, &runs)), cb));
            }),
        }
    }
    Ok(())
}

fn event(r: &mut Registry, e: sandbox::Event, cb: u32, runs: Arc<Runs>) {
    use sandbox::{BlockChange as B, Death as D, Hurt as H, ItemUsed as U, Pickup as P};
    fn run(runs: &Arc<Runs>, w: World, cb: u32, batch: Vec<EventValue>) {
        if !batch.is_empty() {
            runs.with(|inst| inst.event(|| Box::new(WorldGame::of(w, runs)), cb, &batch));
        }
    }
    let block = |c: &abi::BlockChange| EventValue::BlockChange(B { player: c.player, pos: t3(c.pos), state: c.state, item: c.item });
    match e {
        sandbox::Event::BlockBroken => r.on(&actions::BREAK_BLOCK, move |b| run(&runs, b.world(), cb, b.iter().map(block).collect())),
        sandbox::Event::BlockPlaced => r.on(&actions::PLACE_BLOCK, move |b| run(&runs, b.world(), cb, b.iter().map(block).collect())),
        sandbox::Event::EntityDied => r.on(&actions::DIE, move |b| {
            let batch = b
                .iter()
                .map(|d| EventValue::Death(D { entity: d.entity, killer: d.source, kind: d.kind, cause: d.cause, pos: f3(d.pos) }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
        sandbox::Event::EntityHurt => r.on(&actions::HURT, move |b| {
            let batch = b
                .iter()
                .map(|h| EventValue::Hurt(H { entity: h.entity, source: h.source, amount: h.amount, cause: h.cause }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
        sandbox::Event::PlayerJoined => {
            r.on(&PLAYER_JOINED, move |b| run(&runs, b.world(), cb, b.iter().map(|p| EventValue::Player(p.player)).collect()))
        }
        sandbox::Event::PlayerRespawned => {
            r.on(&PLAYER_RESPAWNED, move |b| run(&runs, b.world(), cb, b.iter().map(|p| EventValue::Player(p.player)).collect()))
        }
        sandbox::Event::ItemUsed => r.on(&actions::USE_ITEM, move |b| {
            let batch = b
                .iter()
                .map(|u| EventValue::ItemUsed(U { player: u.player, stack: stack(u.stack), pos: t3(u.pos), on_block: u.has_target != 0 }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
        sandbox::Event::ItemPickedUp => r.on(&actions::PICKUP_ITEM, move |b| {
            let batch = b
                .iter()
                .map(|p| EventValue::Pickup(P { player: p.player, stack: stack(p.stack) }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
    }
}

fn t3(p: impl Into<abi::BlockPos>) -> (i32, i32, i32) {
    let p = p.into();
    (p.x, p.y, p.z)
}

fn f3(p: impl Into<abi::Vec3>) -> (f64, f64, f64) {
    let p = p.into();
    (p.x, p.y, p.z)
}

fn stack(s: ItemStack) -> sandbox::ItemStack {
    sandbox::ItemStack { item: s.item, count: s.count, data: s.data }
}

fn query(q: &abi::Query) -> sandbox::Query {
    sandbox::Query {
        entity: q.entity,
        other: q.other,
        pos: t3(q.pos),
        state: q.state,
        item: q.item,
        dim: q.dim,
        cause: q.cause,
        amount: q.amount,
    }
}

// ------------------------------------------------------------------ commands

/// A command's parameters: name and kind, from its usage (`[action:word(a,b)] <who:targets>`).
fn params(usage: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for tok in usage.split(['|', ' ']) {
        let Some(inner) = tok
            .strip_prefix('<')
            .and_then(|t| t.strip_suffix('>'))
            .or_else(|| tok.strip_prefix('[').and_then(|t| t.strip_suffix(']')))
        else {
            continue;
        };
        let Some((name, kind)) = inner.split_once(':') else { continue };
        let kind = kind.split('(').next().unwrap_or(kind);
        if !out.iter().any(|(n, _)| n == name) {
            out.push((name.to_string(), kind.to_string()));
        }
    }
    out
}

fn invocation(cmd: &ServerCommand, params: &[(String, String)]) -> Invocation {
    let mut args = Vec::new();
    for (name, kind) in params {
        let v = match kind.as_str() {
            "int" => cmd.int(name).map(Value::Int),
            "float" => cmd.float(name).map(Value::Float),
            "targets" => cmd.given(name).then(|| Value::Targets(cmd.targets(name))),
            "pos" => cmd.pos(name).map(|p| Value::Pos(f3(p))),
            "item" | "block" | "entity" | "dim" => cmd
                .id(name)
                .map(|id| Value::Id((id, cmd.text(name).unwrap_or("").to_string()))),
            _ => cmd
                .text(name)
                .map(|t| Value::Text(t.to_string()))
                .or_else(|| cmd.int(name).map(Value::Int)),
        };
        if let Some(v) = v {
            args.push((name.clone(), v));
        }
    }
    Invocation { player: cmd.player(), args }
}

// ------------------------------------------------------------------ the world, for the sandbox

/// The world calls of one callback, answered with Weft's SDK. Lives only as long as the callback
/// (the sandbox drops it when the call returns), which is what makes holding `World` and the
/// command sound.
struct WorldGame {
    w: World,
    /// The mod's components and effects.
    runs: Arc<Runs>,
    /// The `ServerCommand` being run, as an address (0: none).
    cmd: usize,
}

// SAFETY: a `WorldGame` is made for one callback on the thread running it and dropped before the
// callback returns; it never moves to another thread in between.
unsafe impl Send for WorldGame {}

impl WorldGame {
    fn of(w: World, runs: &Arc<Runs>) -> WorldGame {
        WorldGame { w, cmd: 0, runs: runs.clone() }
    }

    fn comp(&self, name: &str) -> Option<Comp> {
        self.runs
            .comps
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, c)| *c)
    }

    /// The SDK's handle for a sound or particle the game has, made once per name.
    fn effect(&self, kind: u32, name: &str) -> Option<Effect> {
        let mut effects = self.runs.effects.lock().unwrap_or_else(|e| e.into_inner());
        let sound = kind == abi::entry_kind::SOUND;
        if let Some((_, e)) = effects
            .iter()
            .find(|(n, e)| n == name && matches!(e, Effect::Sound(_)) == sound)
        {
            return Some(*e);
        }
        if effects.len() >= EFFECTS || find(kind, name).is_none() {
            return None;
        }
        let e =
            if sound { Effect::Sound(Sound::generated(name.to_string())) } else { Effect::Particle(Particle::generated(name.to_string())) };
        effects.push((name.to_string(), e));
        Some(e)
    }
}

fn find(kind: u32, name: &str) -> Option<u32> {
    let id = unsafe { (host().find)(kind, Str::new(name), std::ptr::null_mut()) };
    (id != u32::MAX).then_some(id)
}

fn known_item(id: u32) -> bool {
    id != 0 && !item_name(id).is_empty()
}

fn is_player(w: &World, id: u64) -> bool {
    w.players().contains(&id)
        || w.entity(id)
            .is_some_and(|e| name_of(abi::entry_kind::ENTITY, e.kind) == "core:player")
}

/// What a mod gives may only be a real item, without data (data indexes the engine's tables).
fn real_stack(s: sandbox::ItemStack) -> Option<ItemStack> {
    (known_item(s.item) && s.count > 0).then(|| ItemStack::new(s.item, s.count.min(64 * 64)))
}

/// A mod's line arguments as the SDK's: its texts are kept here while the line is sent.
pub fn line_text(key: &str, args: &[WArg]) -> String {
    let mut out = key.replace('\n', " ");
    for a in args {
        out.push('\n');
        match a {
            WArg::Text(s) => out.push_str(&format!("t{}", s.replace('\n', " "))),
            WArg::Key(s) => out.push_str(&format!("k{}", s.replace('\n', " "))),
            WArg::Int(v) => out.push_str(&format!("i{v}")),
            WArg::Float((v, d)) => out.push_str(&format!("f{d} {v}")),
            WArg::Item(v) => out.push_str(&format!("m{v}")),
            WArg::Name(s) => out.push_str(&format!("n{}", s.replace('\n', " "))),
        }
    }
    out
}

/// A line from [`args_text`] as a key and the SDK's arguments, handed to `f`.
pub fn with_line(text: &str, f: impl FnOnce(&str, &[Arg])) {
    let mut lines = text.split('\n');
    let key = lines.next().unwrap_or("");
    let parts: Vec<(char, &str)> = lines
        .filter_map(|l| {
            let c = l.chars().next()?;
            Some((c, &l[c.len_utf8()..]))
        })
        .collect();
    let args: Vec<Arg> = parts
        .iter()
        .filter_map(|(c, rest)| match c {
            't' => Some(Arg::Text(rest)),
            'k' => Some(Arg::Key(rest)),
            'n' => Some(Arg::Name(rest)),
            'i' => rest.parse().ok().map(Arg::Int),
            'm' => rest.parse().ok().map(Arg::Item),
            'f' => {
                let (d, v) = rest.split_once(' ')?;
                Some(Arg::Float(v.parse().ok()?, d.parse().ok()?))
            }
            _ => None,
        })
        .collect();
    f(key, &args)
}

impl Game for WorldGame {
    fn time(&mut self) -> u64 {
        self.w.game_time()
    }
    fn seed(&mut self) -> u64 {
        self.w.seed()
    }
    fn random(&mut self) -> u64 {
        self.w.random()
    }
    fn dimension(&mut self) -> u32 {
        self.w.dimension()
    }
    fn get_block(&mut self, pos: [i32; 3]) -> u32 {
        self.w.block_loaded(pos).unwrap_or(0)
    }
    fn set_block(&mut self, pos: [i32; 3], state: u32) -> bool {
        // Only states that exist: the engine indexes its tables with them.
        !block_name(state).is_empty() && self.w.set_block(pos, state)
    }
    fn light(&mut self, pos: [i32; 3]) -> u32 {
        self.w.light(pos)
    }
    fn biome(&mut self, pos: [i32; 3]) -> u32 {
        self.w.biome(pos)
    }
    fn players(&mut self) -> Vec<u64> {
        self.w.players()
    }
    fn entity(&mut self, id: u64) -> Option<WEntity> {
        self.w.entity(id).map(|e| WEntity {
            kind: e.kind,
            pos: f3(e.pos),
            vel: f3(e.vel),
            yaw: e.yaw,
            pitch: e.pitch,
            on_ground: e.on_ground != 0,
            in_liquid: e.in_liquid != 0,
            health: e.health,
            max_health: e.max_health,
            age: e.age,
        })
    }
    fn near(&mut self, pos: [f64; 3], radius: f64) -> Vec<u64> {
        self.w.entities_near(pos, radius, None)
    }
    fn teleport(&mut self, id: u64, pos: [f64; 3]) {
        if self.w.entity(id).is_some() {
            drop(self.w.entity_edit(id).pos(pos).velocity([0.0; 3]));
        }
    }
    fn push(&mut self, id: u64, velocity: [f64; 3]) {
        self.w.push(id, velocity)
    }
    fn damage(&mut self, id: u64, amount: f32, source: u64, cause: u32) -> bool {
        self.w.entity(id).is_some() && self.w.damage(id, amount, source, cause)
    }
    fn spawn(&mut self, kind: &str, pos: [f64; 3]) -> u64 {
        match find(abi::entry_kind::ENTITY, kind) {
            Some(k) if kind != "core:player" => self.w.spawn(k, pos),
            _ => 0,
        }
    }
    fn remove(&mut self, id: u64) {
        if self.w.entity(id).is_some() && !is_player(&self.w, id) {
            self.w.despawn(id)
        }
    }
    fn creative(&mut self, player: u64) -> bool {
        self.w.creative(player)
    }
    fn give(&mut self, owner: u64, s: sandbox::ItemStack) -> sandbox::ItemStack {
        match real_stack(s) {
            Some(st) if self.w.entity(owner).is_some() => stack(self.w.inventory(owner).insert(st)),
            _ => s,
        }
    }
    fn spawn_item(&mut self, pos: [f64; 3], s: sandbox::ItemStack) {
        if let Some(st) = real_stack(s) {
            self.w.spawn_item(pos, st);
        }
    }
    fn tell(&mut self, player: u64, key: &str, args: &[WArg]) {
        if self.w.players().contains(&player) {
            self.w.send(player, &SAY, &line_text(key, args));
        }
    }
    fn player_key(&mut self, player: u64) -> String {
        crate::ask("player_key", &player.to_string())
    }
    fn grant_achievement(&mut self, player: u64, id: &str, title: &str, description: &str, icon: Option<&str>) {
        // The engine's bridge signs it (chiacraft, phase 4: `achievements`); tabs and lines would
        // split the question.
        let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
        let arg = format!("{player}\t{}\t{}\t{}\t{}", clean(id), clean(title), clean(description), icon.map(clean).unwrap_or_default());
        crate::ask("achievement", &arg);
    }
    fn reply(&mut self, key: &str, args: &[WArg]) {
        if self.cmd == 0 {
            return;
        }
        // SAFETY: `cmd` is the command this callback runs, alive until it returns (see the type).
        let cmd = unsafe { &mut *(self.cmd as *mut ServerCommand) };
        with_line(&line_text(key, args), |key, args| cmd.reply(key, args));
    }

    fn get_component(&mut self, id: u64, name: &str) -> Option<Vec<u8>> {
        get_component(&self.w, id, self.comp(name)?)
    }
    fn set_component(&mut self, id: u64, name: &str, value: &[u8]) -> bool {
        self.comp(name)
            .is_some_and(|c| set_component(&self.w, id, c, value))
    }
    fn schedule(&mut self, pos: [i32; 3], ticks: u32) {
        self.w.schedule_tick(pos, ticks)
    }
    fn send(&mut self, player: u64, channel: &str, data: &[u8]) {
        let Some((_, c, _)) = self.runs.channels.iter().find(|(n, ..)| n == channel) else { return };
        if self.w.players().contains(&player) {
            self.w.send(player, c, &data.to_vec());
        }
    }
    fn play_sound(&mut self, sound: &str, pos: [f64; 3], volume: f32) {
        if let Some(Effect::Sound(s)) = self.effect(abi::entry_kind::SOUND, sound) {
            drop(self.w.play_sound(s, pos, volume));
        }
    }
    fn spawn_particles(&mut self, particle: &str, pos: [f64; 3], count: u32) {
        if let Some(Effect::Particle(p)) = self.effect(abi::entry_kind::PARTICLE, particle) {
            self.w.spawn_particles(p, pos).count(count);
        }
    }

    fn block_name(&mut self, state: u32) -> String {
        block_name(state).to_string()
    }
    fn block_state(&mut self, name: &str) -> Option<u32> {
        let (base, props) = match name.split_once('[') {
            Some((b, p)) => (b, p.strip_suffix(']')?),
            None => (name, ""),
        };
        let mut state = find(abi::entry_kind::BLOCK, base)?;
        for kv in props.split(',').filter(|s| !s.is_empty()) {
            let (k, v) = kv.split_once('=')?;
            state = state_with(state, k.trim(), v.trim())?;
        }
        Some(state)
    }
    fn prop(&mut self, state: u32, prop: &str) -> Option<String> {
        if block_name(state).is_empty() {
            return None;
        }
        state_prop(state, prop).map(str::to_string)
    }
    fn with_prop(&mut self, state: u32, prop: &str, value: &str) -> Option<u32> {
        if block_name(state).is_empty() {
            return None;
        }
        state_with(state, prop, value)
    }
    fn item_name(&mut self, id: u32) -> String {
        item_name(id).to_string()
    }
    fn item_id(&mut self, name: &str) -> Option<u32> {
        find(abi::entry_kind::ITEM, name)
    }
    fn entity_kind(&mut self, id: u32) -> String {
        name_of(abi::entry_kind::ENTITY, id).to_string()
    }
    fn entity_kind_id(&mut self, name: &str) -> Option<u32> {
        find(abi::entry_kind::ENTITY, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_its_parameters() {
        let p = params("[action:word(create,join,leave,info)] [name:text] | <who:targets> <n:int>");
        let names: Vec<&str> = p
            .iter()
            .flat_map(|(n, k)| [n.as_str(), k.as_str()])
            .collect();
        assert_eq!(names, ["action", "word", "name", "text", "who", "targets", "n", "int"]);
    }

    #[test]
    fn lines_survive_the_trip() {
        let text = line_text(
            "clans.invite",
            &[WArg::Text("a\nb".into()), WArg::Int(-3), WArg::Float((1.5, 2)), WArg::Key("k".into()), WArg::Item(7)],
        );
        let mut got = String::new();
        with_line(&text, |key, args| {
            got = key.to_string();
            for a in args {
                got.push_str(&match a {
                    Arg::Text(s) => format!(" text {s}"),
                    Arg::Key(s) => format!(" key {s}"),
                    Arg::Int(v) => format!(" int {v}"),
                    Arg::Float(v, d) => format!(" float {v} {d}"),
                    Arg::Item(v) => format!(" item {v}"),
                    Arg::Name(s) => format!(" name {s}"),
                });
            }
        });
        assert_eq!(got, "clans.invite text a b int -3 float 1.5 2 key k item 7");
    }
}

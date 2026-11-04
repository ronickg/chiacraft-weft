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

use chiacraft_sandbox::{self as sandbox, Arg as WArg, EntityState as WEntity, EventValue, Game, Hook, Instance, Invocation, Mod, Value};
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
        sandbox::Level::Error => abi::level::ERROR,
        sandbox::Level::Warn => abi::level::WARN,
        sandbox::Level::Info => abi::level::INFO,
        sandbox::Level::Debug => abi::level::DEBUG,
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

/// The package's mod in each world it has run in.
struct Runs {
    m: Arc<Mod>,
    package: String,
    runs: Mutex<Vec<(u64, Instance)>>,
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

/// Loads `dir/code/server.wasm`, if the package has one, and registers what it registers.
/// Mistakes are load errors (`r`'s), and the package is left out.
pub fn register(r: &mut Registry, package: &str, dir: &Path) -> Result<(), String> {
    let path = dir.join(chiacraft_packages::identity::SERVER_CODE);
    if !path.is_file() {
        return Ok(());
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
    let hooks = m.hooks().to_vec();
    let runs = Arc::new(Runs { m: Arc::new(m), package: package.to_string(), runs: Mutex::new(Vec::new()) });
    for (i, hook) in hooks.into_iter().enumerate() {
        let cb = i as u32;
        let runs = runs.clone();
        match hook {
            Hook::Command { name, usage, permission } => {
                let params = params(&usage);
                r.command(&name, &usage, move |cmd| {
                    let call = invocation(cmd, &params);
                    let w = cmd.world();
                    let cmd_ptr = cmd as *mut ServerCommand as usize;
                    runs.with(|inst| inst.command(|| Box::new(WorldGame { w, cmd: cmd_ptr }), cb, &call));
                })
                .permission(permission);
            }
            Hook::Event(e) => event(r, e, cb, runs),
            Hook::Question(q) => {
                let question = match q {
                    sandbox::Question::BreakBlock => &question::BREAK_BLOCK,
                    sandbox::Question::PlaceBlock => &question::PLACE_BLOCK,
                    sandbox::Question::UseBlock => &question::USE_BLOCK,
                    sandbox::Question::UseItem => &question::USE_ITEM,
                    sandbox::Question::InteractEntity => &question::INTERACT_ENTITY,
                    sandbox::Question::Attack => &question::ATTACK,
                    sandbox::Question::Hurt => &question::HURT,
                    sandbox::Question::PickupItem => &question::PICKUP_ITEM,
                    sandbox::Question::EnterDimension => &question::ENTER_DIMENSION,
                    sandbox::Question::NaturalSpawn => &question::NATURAL_SPAWN,
                };
                r.intercept(question, move |a| {
                    let w = a.world();
                    let q = query(a.query());
                    match runs.with(|inst| inst.question(|| Box::new(WorldGame::of(w)), cb, q)) {
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
                    if let Some(v) = runs.with(|inst| inst.modify(|| Box::new(WorldGame::of(w)), cb, q, base, value)) {
                        m.value = v;
                    }
                });
            }
            Hook::WorldTick => r.world_tick(move |w| {
                let w = *w;
                runs.with(|inst| inst.world_tick(|| Box::new(WorldGame::of(w)), cb));
            }),
        }
    }
    Ok(())
}

fn event(r: &mut Registry, e: sandbox::Event, cb: u32, runs: Arc<Runs>) {
    use sandbox::{BlockChange as B, Death as D, Hurt as H, ItemUsed as U, Pickup as P};
    fn run(runs: &Runs, w: World, cb: u32, batch: Vec<EventValue>) {
        if !batch.is_empty() {
            runs.with(|inst| inst.event(|| Box::new(WorldGame::of(w)), cb, &batch));
        }
    }
    let block = |c: &abi::BlockChange| EventValue::BlockChange(B { player: c.player, pos: t3(c.pos), state: c.state, item: c.item });
    match e {
        sandbox::Event::BlockBroken => r.on(&BLOCK_BROKEN, move |b| run(&runs, b.world(), cb, b.iter().map(block).collect())),
        sandbox::Event::BlockPlaced => r.on(&BLOCK_PLACED, move |b| run(&runs, b.world(), cb, b.iter().map(block).collect())),
        sandbox::Event::EntityDied => r.on(&ENTITY_DIED, move |b| {
            let batch = b
                .iter()
                .map(|d| EventValue::Death(D { entity: d.entity, killer: d.killer, kind: d.kind, cause: d.cause, pos: f3(d.pos) }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
        sandbox::Event::EntityHurt => r.on(&ENTITY_HURT, move |b| {
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
        sandbox::Event::ItemUsed => r.on(&ITEM_USED, move |b| {
            let batch = b
                .iter()
                .map(|u| EventValue::ItemUsed(U { player: u.player, stack: stack(u.stack), pos: t3(u.pos), on_block: u.has_target != 0 }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
        sandbox::Event::ItemPickedUp => r.on(&ITEM_PICKED_UP, move |b| {
            let batch = b
                .iter()
                .map(|p| EventValue::Pickup(P { player: p.player, stack: stack(p.stack) }))
                .collect();
            run(&runs, b.world(), cb, batch)
        }),
    }
}

fn t3(p: [i32; 3]) -> (i32, i32, i32) {
    (p[0], p[1], p[2])
}

fn f3(p: [f64; 3]) -> (f64, f64, f64) {
    (p[0], p[1], p[2])
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
            "targets" => cmd.has(name).then(|| Value::Targets(cmd.targets(name))),
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
    /// The `ServerCommand` being run, as an address (0: none).
    cmd: usize,
}

// SAFETY: a `WorldGame` is made for one callback on the thread running it and dropped before the
// callback returns; it never moves to another thread in between.
unsafe impl Send for WorldGame {}

impl WorldGame {
    fn of(w: World) -> WorldGame {
        WorldGame { w, cmd: 0 }
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
fn args_text(key: &str, args: &[WArg]) -> String {
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
        self.w.time()
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
        self.w.get_loaded(pos).unwrap_or(0)
    }
    fn set_block(&mut self, pos: [i32; 3], state: u32) -> bool {
        // Only states that exist: the engine indexes its tables with them.
        !block_name(state).is_empty() && self.w.set(pos, state)
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
        self.w.near(pos, radius, None)
    }
    fn teleport(&mut self, id: u64, pos: [f64; 3]) {
        if let Some(mut e) = self.w.entity(id) {
            e.pos = pos;
            e.vel = [0.0; 3];
            self.w.set_entity(id, &e);
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
            Some(k) if kind != "core:player" => self.w.spawn_kind(k, pos),
            _ => 0,
        }
    }
    fn remove(&mut self, id: u64) {
        if self.w.entity(id).is_some() && !is_player(&self.w, id) {
            self.w.remove(id)
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
            self.w.spawn_item(pos, st)
        }
    }
    fn tell(&mut self, player: u64, key: &str, args: &[WArg]) {
        if self.w.players().contains(&player) {
            self.w.send(player, &SAY, &args_text(key, args));
        }
    }
    fn player_key(&mut self, player: u64) -> String {
        crate::ask("player_key", &player.to_string())
    }
    fn reply(&mut self, key: &str, args: &[WArg]) {
        if self.cmd == 0 {
            return;
        }
        // SAFETY: `cmd` is the command this callback runs, alive until it returns (see the type).
        let cmd = unsafe { &mut *(self.cmd as *mut ServerCommand) };
        with_line(&args_text(key, args), |key, args| cmd.reply(key, args));
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
            .map(|(n, k)| [n.as_str(), k.as_str()])
            .flatten()
            .collect();
        assert_eq!(names, ["action", "word", "name", "text", "who", "targets", "n", "int"]);
    }

    #[test]
    fn lines_survive_the_trip() {
        let text = args_text(
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

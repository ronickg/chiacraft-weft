//! A hostile mod against the sandbox: `hostile-guest` runs each input as a program of API 1.0
//! calls with the worst arguments it finds, some in loops past every cap. The game here records
//! every call that reaches it and fails on anything the sandbox promised to keep out: positions
//! that aren't finite, pushes past ±10, damage that isn't finite, negative or past 1e6, a `near`
//! wider than 64, and more lines, replies, block changes or spawns in one call than the caps,
//! or lines with longer keys, more arguments or longer texts than allowed. Strikes and a stopped
//! mod are fine: a stopped one is started again in a fresh world.
//!
//! The mod is built for wasm32-unknown-unknown on first use (`rustup target add
//! wasm32-unknown-unknown` for the toolchain `cargo fuzz` runs), or read from the core module
//! `HOSTILE_GUEST_WASM` names.
#![no_main]

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use chiacraft_sandbox::*;
use libfuzzer_sys::fuzz_target;

/// `hostile-guest`'s report entity: it teleports there when it saw a promise broken.
const REPORT: u64 = 0x0BAD_0BAD_0BAD_0BAD;

/// The sandbox's per-call caps and sizes, as `docs/sandboxed-mods.md` and the WIT give them.
const TELLS: u32 = 32;
const REPLIES: u32 = 32;
const BLOCK_SETS: u32 = 4096;
const SPAWNS: u32 = 64;
const NEAR_RADIUS: f64 = 64.0;
const PUSH: f64 = 10.0;
const DAMAGE: f32 = 1.0e6;
const KEY_BYTES: usize = 128;
const ARGS: usize = 16;
const TEXT_BYTES: usize = 1024;

/// The mod as a component, built once.
fn guest() -> &'static [u8] {
    static WASM: OnceLock<Vec<u8>> = OnceLock::new();
    WASM.get_or_init(|| {
        let core = match std::env::var_os("HOSTILE_GUEST_WASM") {
            Some(path) => std::fs::read(&path).expect("HOSTILE_GUEST_WASM"),
            None => build_guest(),
        };
        componentize(&core).expect("componentize the hostile mod")
    })
}

fn build_guest() -> Vec<u8> {
    let fuzz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = fuzz.join("target/hostile-guest");
    // The cargo that built this, unless a toolchain update has moved it since.
    let cargo = Some(PathBuf::from(env!("CARGO")))
        .filter(|c| c.exists())
        .unwrap_or_else(|| "cargo".into());
    // Not the fuzz build's own flags: no sanitizers or coverage for wasm32.
    let status = std::process::Command::new(cargo)
        .current_dir(fuzz.join("hostile-guest"))
        .args(["build", "--release", "--target", "wasm32-unknown-unknown", "--target-dir"])
        .arg(&target)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("CARGO_TARGET_DIR")
        .status()
        .expect("cargo");
    assert!(status.success(), "building the hostile mod needs `rustup target add wasm32-unknown-unknown`");
    std::fs::read(target.join("wasm32-unknown-unknown/release/sandbox_hostile_guest.wasm")).expect("the hostile mod's module")
}

fn hostile() -> Arc<Mod> {
    static MOD: OnceLock<Arc<Mod>> = OnceLock::new();
    MOD.get_or_init(|| {
        // A strike a run is the point here: keep its notes out of the fuzzer's output.
        set_logger(|_, _, _| {});
        let m = Mod::load("hostile", guest()).expect("the hostile mod loads");
        assert_eq!(m.hooks(), &[Hook::Command { name: "probe".into(), usage: "<ops:text>".into(), permission: 0 }]);
        Arc::new(m)
    })
    .clone()
}

/// The game's side of one callback: what reached it so far in this call.
#[derive(Default)]
struct Watch {
    tells: u32,
    replies: u32,
    block_sets: u32,
    spawns: u32,
}

fn finite(what: &str, p: [f64; 3]) {
    assert!(p.iter().all(|v| v.is_finite()), "{what} at {p:?}");
}

fn line(what: &str, key: &str, args: &[Arg]) {
    assert!(key.len() <= KEY_BYTES, "{what} with a key of {} bytes", key.len());
    assert!(args.len() <= ARGS, "{what} with {} arguments", args.len());
    for a in args {
        match a {
            Arg::Text(s) | Arg::Key(s) | Arg::Name(s) => assert!(s.len() <= TEXT_BYTES, "{what} with a text of {} bytes", s.len()),
            Arg::Float((_, decimals)) => assert!(*decimals <= 8, "{what} with {decimals} decimals"),
            Arg::Int(_) | Arg::Item(_) => {}
        }
    }
}

impl Game for Watch {
    fn time(&mut self) -> u64 {
        u64::MAX
    }
    fn seed(&mut self) -> u64 {
        u64::MAX
    }
    fn random(&mut self) -> u64 {
        u64::MAX
    }
    fn dimension(&mut self) -> u32 {
        u32::MAX
    }
    fn get_block(&mut self, _: [i32; 3]) -> u32 {
        u32::MAX
    }
    fn set_block(&mut self, _: [i32; 3], _: u32) -> bool {
        self.block_sets += 1;
        assert!(self.block_sets <= BLOCK_SETS, "{} block changes in one call", self.block_sets);
        true
    }
    fn light(&mut self, _: [i32; 3]) -> u32 {
        15
    }
    fn biome(&mut self, _: [i32; 3]) -> u32 {
        0
    }
    fn players(&mut self) -> Vec<u64> {
        vec![7, u64::MAX]
    }
    fn entity(&mut self, id: u64) -> Option<EntityState> {
        // An entity the game has gone strange on: the mod hands these straight back.
        (id % 2 == 1).then_some(EntityState {
            kind: u32::MAX,
            pos: (f64::NAN, f64::INFINITY, f64::NEG_INFINITY),
            vel: (f64::MAX, f64::MIN, f64::NAN),
            yaw: f32::NAN,
            pitch: f32::INFINITY,
            on_ground: true,
            in_liquid: true,
            health: f32::NAN,
            max_health: -1.0,
            age: u64::MAX,
        })
    }
    fn near(&mut self, pos: [f64; 3], radius: f64) -> Vec<u64> {
        finite("near", pos);
        assert!((0.0..=NEAR_RADIUS).contains(&radius), "near with a radius of {radius}");
        // More than the sandbox passes on.
        (1..=1000).collect()
    }
    fn teleport(&mut self, id: u64, pos: [f64; 3]) {
        assert_ne!(id, REPORT, "the mod saw a promise broken: (code, detail, 0) = {pos:?}");
        finite("teleport", pos);
    }
    fn push(&mut self, _: u64, v: [f64; 3]) {
        finite("push", v);
        assert!(v.iter().all(|x| (-PUSH..=PUSH).contains(x)), "push by {v:?}");
    }
    fn damage(&mut self, _: u64, amount: f32, _: u64, _: u32) -> bool {
        assert!(amount.is_finite() && (0.0..=DAMAGE).contains(&amount), "damage of {amount}");
        true
    }
    fn spawn(&mut self, kind: &str, pos: [f64; 3]) -> u64 {
        self.spawns += 1;
        assert!(self.spawns <= SPAWNS, "{} spawns in one call", self.spawns);
        assert!(kind.len() <= KEY_BYTES, "spawn of a kind of {} bytes", kind.len());
        finite("spawn", pos);
        u64::MAX
    }
    fn remove(&mut self, _: u64) {}
    fn creative(&mut self, _: u64) -> bool {
        true
    }
    fn give(&mut self, _: u64, s: ItemStack) -> ItemStack {
        s
    }
    fn spawn_item(&mut self, pos: [f64; 3], _: ItemStack) {
        finite("spawn-item", pos);
    }
    fn tell(&mut self, _: u64, key: &str, args: &[Arg]) {
        self.tells += 1;
        assert!(self.tells <= TELLS, "{} lines in one call", self.tells);
        line("tell", key, args);
    }
    fn player_key(&mut self, p: u64) -> String {
        if p == 7 { "alice".into() } else { String::new() }
    }
    fn reply(&mut self, key: &str, args: &[Arg]) {
        self.replies += 1;
        assert!(self.replies <= REPLIES, "{} replies in one call", self.replies);
        line("reply", key, args);
    }
    fn block_name(&mut self, _: u32) -> String {
        "base:furnace[lit=true]".into()
    }
    fn block_state(&mut self, _: &str) -> Option<u32> {
        Some(u32::MAX)
    }
    fn prop(&mut self, _: u32, _: &str) -> Option<String> {
        Some("true".into())
    }
    fn with_prop(&mut self, s: u32, _: &str, _: &str) -> Option<u32> {
        Some(s)
    }
    fn item_name(&mut self, _: u32) -> String {
        "base:coal".into()
    }
    fn item_id(&mut self, _: &str) -> Option<u32> {
        Some(u32::MAX)
    }
    fn entity_kind(&mut self, _: u32) -> String {
        "base:pig".into()
    }
    fn entity_kind_id(&mut self, _: &str) -> Option<u32> {
        Some(u32::MAX)
    }
}

thread_local! {
    /// The mod in its world, kept across inputs (in-memory storage), as it would run in a game.
    static WORLD: RefCell<Option<Instance>> = const { RefCell::new(None) };
}

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fuzz_target!(|data: &[u8]| {
    let m = hostile();
    let call = Invocation { player: 7, args: vec![("ops".into(), Value::Text(hex(&data[..data.len().min(1 << 16)])))] };
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        if w.as_ref().is_none_or(|i| i.stopped().is_some()) {
            *w = Some(Instance::new(m, None));
        }
        let inst = w.as_mut().unwrap();
        inst.command(|| Box::new(Watch::default()) as Box<dyn Game>, 0, &call);
    });
});

//! The sandbox against a real mod (`tests/guest`): registrations, calls with a world, storage,
//! and every limit met on purpose. Needs `rustup target add wasm32-unknown-unknown`.
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use chiacraft_sandbox::*;

/// The test mod as a component, built once.
fn guest() -> &'static [u8] {
    static WASM: OnceLock<Vec<u8>> = OnceLock::new();
    WASM.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/guest");
        let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sandbox-guest");
        let status = std::process::Command::new(env!("CARGO"))
            .current_dir(&dir)
            .args(["build", "--release", "--target", "wasm32-unknown-unknown", "--target-dir"])
            .arg(&target)
            .status()
            .expect("cargo");
        assert!(status.success(), "building the test mod needs `rustup target add wasm32-unknown-unknown`");
        let core = std::fs::read(target.join("wasm32-unknown-unknown/release/sandbox_test_guest.wasm")).unwrap();
        componentize(&core).unwrap()
    })
}

/// What the mod did through its game.
#[derive(Default, Debug)]
struct Seen {
    tells: Vec<String>,
    replies: Vec<String>,
    /// Player 7's value of each component.
    components: std::collections::HashMap<String, Vec<u8>>,
    sets: u32,
    sounds: u32,
    particles: Vec<u32>,
    scheduled: Vec<u32>,
    sent: Vec<(String, Vec<u8>)>,
}

fn text(key: &str, args: &[Arg]) -> String {
    let mut s = key.to_string();
    for a in args {
        s.push(' ');
        s.push_str(&match a {
            Arg::Text(t) | Arg::Key(t) | Arg::Name(t) => t.clone(),
            Arg::Int(v) => v.to_string(),
            Arg::Float((v, _)) => v.to_string(),
            Arg::Item(v) => v.to_string(),
        });
    }
    s
}

/// A world of one player (entity 7, key "alice") where block (1, 2, 3) is state 9 and the time is 100.
struct Fake(Arc<Mutex<Seen>>);

impl Game for Fake {
    fn time(&mut self) -> u64 {
        100
    }
    fn seed(&mut self) -> u64 {
        1
    }
    fn random(&mut self) -> u64 {
        4
    }
    fn dimension(&mut self) -> u32 {
        0
    }
    fn get_block(&mut self, pos: [i32; 3]) -> u32 {
        if pos == [1, 2, 3] { 9 } else { 0 }
    }
    fn set_block(&mut self, _: [i32; 3], _: u32) -> bool {
        self.0.lock().unwrap().sets += 1;
        true
    }
    fn light(&mut self, _: [i32; 3]) -> u32 {
        15
    }
    fn biome(&mut self, _: [i32; 3]) -> u32 {
        0
    }
    fn players(&mut self) -> Vec<u64> {
        vec![7]
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
    fn give(&mut self, _: u64, s: ItemStack) -> ItemStack {
        s
    }
    fn spawn_item(&mut self, _: [f64; 3], _: ItemStack) {}
    fn tell(&mut self, _: u64, key: &str, args: &[Arg]) {
        self.0.lock().unwrap().tells.push(text(key, args));
    }
    fn player_key(&mut self, p: u64) -> String {
        if p == 7 { "alice".into() } else { String::new() }
    }
    fn reply(&mut self, key: &str, args: &[Arg]) {
        self.0.lock().unwrap().replies.push(text(key, args));
    }
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
    fn get_component(&mut self, id: u64, name: &str) -> Option<Vec<u8>> {
        // Player 7 carries every component, from its default (here: what `init` declared).
        (id == 7).then(|| self.0.lock().unwrap().components.get(name).cloned())?
    }
    fn set_component(&mut self, id: u64, name: &str, value: &[u8]) -> bool {
        id == 7
            && self
                .0
                .lock()
                .unwrap()
                .components
                .insert(name.to_string(), value.to_vec())
                .is_some()
    }
    fn play_sound(&mut self, _: &str, _: [f64; 3], volume: f32) {
        assert!((0.0..=4.0).contains(&volume));
        self.0.lock().unwrap().sounds += 1;
    }
    fn spawn_particles(&mut self, _: &str, _: [f64; 3], count: u32) {
        self.0.lock().unwrap().particles.push(count);
    }
    fn schedule(&mut self, _: [i32; 3], ticks: u32) {
        self.0.lock().unwrap().scheduled.push(ticks);
    }
    fn send(&mut self, _: u64, channel: &str, data: &[u8]) {
        self.0
            .lock()
            .unwrap()
            .sent
            .push((channel.to_string(), data.to_vec()));
    }
}

struct World {
    inst: Instance,
    seen: Arc<Mutex<Seen>>,
}

impl World {
    fn new(storage: Option<PathBuf>) -> World {
        let m = Arc::new(Mod::load("probe", guest()).unwrap());
        let seen: Arc<Mutex<Seen>> = Arc::default();
        // The game gives player 7 the declared components at their defaults.
        for d in m.decls() {
            if let Decl::Component { name, default, .. } = d {
                seen.lock()
                    .unwrap()
                    .components
                    .insert(name.clone(), default.clone());
            }
        }
        World { inst: Instance::new(m, storage), seen }
    }

    fn game(&self) -> impl Fn() -> Box<dyn Game> + use<> {
        let seen = self.seen.clone();
        move || Box::new(Fake(seen.clone())) as Box<dyn Game>
    }

    /// `/probe <action> [n]`; the replies it gave.
    fn probe(&mut self, action: &str, n: Option<i64>) -> Vec<String> {
        let mut args = vec![("action".to_string(), Value::Text(action.into()))];
        if let Some(n) = n {
            args.push(("n".into(), Value::Int(n)));
        }
        let game = self.game();
        self.inst.command(game, 0, &Invocation { player: 7, args });
        std::mem::take(&mut self.seen.lock().unwrap().replies)
    }
}

#[test]
fn registers_in_order() {
    let m = Mod::load("probe", guest()).unwrap();
    assert_eq!(
        m.hooks(),
        &[
            Hook::Command { name: "probe".into(), usage: "<action:word> [n:int]".into(), permission: 0 },
            Hook::Event(Event::BlockBroken),
            Hook::Question(Question::BreakBlock),
            Hook::Modify { chain: Chain::Damage, stage: 200 },
            Hook::WorldTick,
        ]
    );
}

#[test]
fn declares_components_and_attaches() {
    let m = Mod::load("probe", guest()).unwrap();
    let mut default = 5u32.to_le_bytes().to_vec();
    default.resize(16, 0);
    assert_eq!(
        m.decls(),
        &[
            Decl::Component { name: "probe:counter".into(), size: 16, flags: ComponentFlags::SAVE | ComponentFlags::KEEP, default },
            Decl::Attach { kind: "core:player".into(), component: "probe:counter".into() },
        ]
    );
}

#[test]
fn components_hold_values_per_entity() {
    let mut w = World::new(None);
    assert_eq!(w.probe("comp", None), ["probe.comp 6"]);
    assert_eq!(w.probe("comp", None), ["probe.comp 7"]);
    let v = w.seen.lock().unwrap().components["probe:counter"].clone();
    assert_eq!(v.len(), 16);
    assert_eq!(u64::from_le_bytes(v[8..16].try_into().unwrap()), 100, "the time it was last changed");
}

#[test]
fn batched_blocks_and_effects_keep_their_allowances() {
    let mut w = World::new(None);
    assert_eq!(w.probe("blocks", None), ["probe.blocks 4096"]);
    assert_eq!(w.seen.lock().unwrap().sets, 4096);
    w.probe("effects", None);
    let seen = w.seen.lock().unwrap();
    assert_eq!(seen.sounds, 64);
    assert_eq!(seen.particles.len(), 64);
    assert!(seen.particles.iter().all(|&c| c == 256));
}

#[test]
fn registers_its_blocks_items_and_channels() {
    let m = Mod::load("probe", guest()).unwrap();
    let tail: Vec<&Hook> = m.hooks().iter().skip(5).collect();
    assert_eq!(
        tail,
        [
            &Hook::BlockUse("probe:bell".into()),
            &Hook::BlockScheduled("probe:bell".into()),
            &Hook::ItemUse("probe:wand".into()),
            &Hook::Channel("probe:pings".into()),
        ]
    );
}

#[test]
fn block_item_and_packet_calls() {
    let mut w = World::new(None);
    let used = |y| BlockUseEvent { player: 7, pos: (0, y, 0), state: 1, face: 1 };
    assert!(w.inst.block_use(w.game(), 5, used(3)));
    assert!(!w.inst.block_use(w.game(), 5, used(-3)));
    let ev = BlockUpdateEvent { pos: (0, 3, 0), state: 1, other: (0, 3, 0), entity: 0 };
    w.inst.block_update(w.game(), 6, ev);
    {
        let seen = w.seen.lock().unwrap();
        assert_eq!(seen.tells, ["probe.rang 3", "probe.rang -3", "probe.scheduled 3"]);
        // A tick at least 1 away.
        assert_eq!(seen.scheduled, [1, 1]);
    }
    let stack = ItemStack { item: 9, count: 5, data: 0 };
    let r = w
        .inst
        .item_use(w.game(), 7, ItemUseEvent { player: 7, stack, slot: 0, target: None });
    assert!(r.handled);
    assert_eq!(r.stack.map(|s| s.count), Some(4));
    w.inst.packet(w.game(), 8, 7, b"hi");
    let seen = w.seen.lock().unwrap();
    assert_eq!(seen.sent.len(), 32, "32 sends a call");
    assert_eq!(seen.sent[0], ("probe:pings".to_string(), b"pong hi".to_vec()));
}

#[test]
fn a_component_built_for_1_1_still_loads() {
    let wasm = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/tally/code/server.wasm")).unwrap();
    let m = Mod::load("tally", &wasm).unwrap();
    assert_eq!(m.hooks().len(), 2);
    assert_eq!(m.decls().len(), 2);
}

#[test]
fn a_component_built_for_1_0_still_loads() {
    // The clans package's server half, built with the 1.0 SDK and committed.
    let wasm = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/clans/code/server.wasm")).unwrap();
    let m = Mod::load("clans", &wasm).unwrap();
    assert_eq!(m.hooks().len(), 3);
    assert!(m.decls().is_empty());
}

#[test]
fn calls_reach_the_mod_and_its_world() {
    let mut w = World::new(None);
    assert_eq!(w.probe("world", None), ["probe.world 100 9 alice"]);
    let game = w.game();
    w.inst
        .event(game, 1, &[EventValue::BlockChange(BlockChange { player: 7, pos: (0, -5, 0), state: 1, item: 0 })]);
    assert_eq!(w.seen.lock().unwrap().tells, ["probe.broken -5"]);
    let q = |y| Query { entity: 7, other: 0, pos: (0, y, 0), state: 1, item: 0, dim: 0, cause: 0, amount: 0.0 };
    assert_eq!(w.inst.question(w.game(), 2, q(-1)), Verdict::Deny);
    assert_eq!(w.inst.question(w.game(), 2, q(10)), Verdict::Abstain);
    assert_eq!(w.inst.modify(w.game(), 3, q(0), 8.0, 8.0), 4.0);
    w.inst.world_tick(w.game(), 4);
    assert!(w.inst.stopped().is_none());
}

#[test]
fn a_runaway_loop_is_a_strike_and_memory_starts_over() {
    let mut w = World::new(None);
    assert_eq!(w.probe("count", None), ["probe.count 1"]);
    assert_eq!(w.probe("count", None), ["probe.count 2"]);
    let started = std::time::Instant::now();
    assert!(w.probe("spin", None).is_empty());
    assert!(started.elapsed() < engine::WATCHDOG * 4, "fuel should stop it well before the watchdog's backstop");
    // Started over: its memory is new.
    assert_eq!(w.probe("count", None), ["probe.count 1"]);
}

#[test]
fn memory_is_capped() {
    let mut w = World::new(None);
    assert!(w.probe("grow", None).is_empty());
    assert_eq!(w.probe("count", None), ["probe.count 1"]);
}

#[test]
fn three_strikes_stop_it() {
    let mut w = World::new(None);
    for _ in 0..STRIKES {
        w.probe("panic", None);
    }
    assert!(w.inst.stopped().is_some());
    assert!(w.probe("count", None).is_empty());
    // A failing question abstains, a failing chain step leaves the value.
    let q = Query { entity: 0, other: 0, pos: (0, -1, 0), state: 0, item: 0, dim: 0, cause: 0, amount: 0.0 };
    assert_eq!(w.inst.question(w.game(), 2, q), Verdict::Abstain);
    assert_eq!(w.inst.modify(w.game(), 3, q, 8.0, 8.0), 8.0);
}

#[test]
fn per_call_allowances_hold() {
    let mut w = World::new(None);
    w.probe("flood", None);
    assert_eq!(w.seen.lock().unwrap().tells.len(), 32);
}

#[test]
fn storage_is_kept_and_contained() {
    let dir = std::env::temp_dir().join(format!("chiacraft-sandbox-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    {
        let mut w = World::new(Some(dir.clone()));
        assert_eq!(w.probe("store", Some(42)), ["probe.stored 42"]);
        assert_eq!(w.probe("bad_key", None).len(), 1);
    }
    // The same world opened again: a new instance, the same storage.
    let mut w = World::new(Some(dir.clone()));
    assert_eq!(w.probe("load", None), ["probe.loaded 42"]);
    let files: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(files, ["n"]);
    assert!(!dir.parent().unwrap().join("escape").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn writes_per_call_are_capped() {
    let mut w = World::new(None);
    assert_eq!(w.probe("write_flood", None), ["probe.written 4"]);
    // The next call has its own allowance.
    assert_eq!(w.probe("write_flood", None), ["probe.written 4"]);
}

#[test]
fn registering_after_init_is_refused() {
    let mut w = World::new(None);
    assert_eq!(w.probe("late", None), ["probe.late"]);
    assert!(w.inst.stopped().is_none());
}

#[test]
fn one_api_version() {
    let (major, minor) = chiacraft_packages::code::API;
    assert_eq!(API, format!("{major}.{minor}.0"));
    assert!(include_str!("../wit/v1.2/mod.wit").contains(&format!("package chiacraft:mods@{API};")));
}

#[test]
fn what_isnt_a_component_is_refused() {
    assert!(Mod::load("x", b"not wasm").is_err());
    assert!(Mod::load("x", &vec![0u8; engine::MAX_COMPONENT_BYTES + 1]).is_err());
    // A plain core module isn't a component either.
    assert!(Mod::load("x", b"\0asm\x01\0\0\0").is_err());
}
